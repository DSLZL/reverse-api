use crate::qwen::error::Result;
use crate::qwen::models::{ExtraData, Model, QwenFile, QwenResponse};

use super::modules::{
    auth::AuthManager,
    chat_manager::ChatManager,
    constants::build_json_headers,
    file_uploader::FileUploader,
    media_downloader::MediaDownloader,
    media_generator::MediaGenerator,
    model_selector::ModelSelector,
    streaming::{ConversationBuilder, StreamingHandler},
};
use std::sync::Arc;

pub struct QwenClient {
    client: rquest::Client,
    auth: Arc<AuthManager>,
    chat_manager: ChatManager,
    file_uploader: FileUploader,
    media_generator: MediaGenerator,
    media_downloader: MediaDownloader,
}

impl QwenClient {
    pub fn new(email: String, password: String) -> Result<Self> {
        let client = rquest::Client::builder().cookie_store(true).build()?;
        let auth = Arc::new(AuthManager::new(email, password, client.clone()));

        Ok(Self {
            chat_manager: ChatManager::new(client.clone(), auth.clone()),
            file_uploader: FileUploader::new(client.clone(), auth.clone()),
            media_generator: MediaGenerator::new(client.clone()),
            media_downloader: MediaDownloader::new(client.clone()),
            auth,
            client: rquest::Client::builder().cookie_store(true).build()?,
        })
    }

    pub fn with_token(token: String) -> Result<Self> {
        let client = rquest::Client::builder().cookie_store(true).build()?;
        let auth = Arc::new(AuthManager::with_token(token, client.clone()));

        Ok(Self {
            chat_manager: ChatManager::new(client.clone(), auth.clone()),
            file_uploader: FileUploader::new(client.clone(), auth.clone()),
            media_generator: MediaGenerator::new(client.clone()),
            media_downloader: MediaDownloader::new(client.clone()),
            auth,
            client,
        })
    }

    pub async fn get_models(&self) -> Result<Vec<Model>> {
        self.chat_manager.get_models().await
    }

    pub async fn model_supports_thinking(&self, model_id: &str) -> Result<bool> {
        let models = self.get_models().await?;
        Ok(ModelSelector::model_supports_thinking(&models, model_id))
    }

    pub async fn model_supports_search(&self, model_id: &str) -> Result<bool> {
        let models = self.get_models().await?;
        Ok(ModelSelector::model_supports_search(&models, model_id))
    }

    pub async fn get_model_thinking_budget(&self, model_id: &str) -> Result<Option<u32>> {
        let models = self.get_models().await?;
        Ok(ModelSelector::get_model_thinking_budget(&models, model_id))
    }

    pub async fn get_thinking_capable_models(&self) -> Result<Vec<Model>> {
        let models = self.get_models().await?;
        Ok(ModelSelector::get_thinking_capable_models(models))
    }

    pub async fn get_search_capable_models(&self) -> Result<Vec<Model>> {
        let models = self.get_models().await?;
        Ok(ModelSelector::get_search_capable_models(models))
    }

    pub async fn get_vision_capable_models(&self) -> Result<Vec<Model>> {
        let models = self.get_models().await?;
        Ok(ModelSelector::get_vision_capable_models(models))
    }

    pub async fn get_audio_capable_models(&self) -> Result<Vec<Model>> {
        let models = self.get_models().await?;
        Ok(ModelSelector::get_audio_capable_models(models))
    }

    pub async fn get_video_capable_models(&self) -> Result<Vec<Model>> {
        let models = self.get_models().await?;
        Ok(ModelSelector::get_video_capable_models(models))
    }

    pub async fn select_best_model(
        &self,
        requires_vision: bool,
        requires_audio: bool,
        requires_video: bool,
        requires_thinking: bool,
        requires_search: bool,
    ) -> Result<String> {
        let models = self.get_models().await?;
        Ok(ModelSelector::select_best_model(
            models,
            requires_vision,
            requires_audio,
            requires_video,
            requires_thinking,
            requires_search,
        ))
    }

    pub async fn select_model_for_files(&self, files: &[QwenFile]) -> Result<String> {
        let mut requires_vision = false;
        let mut requires_audio = false;
        let mut requires_video = false;

        for file in files {
            match file.file_class.as_str() {
                "vision" => requires_vision = true,
                "audio" => requires_audio = true,
                "video" => requires_video = true,
                _ => {}
            }
        }

        self.select_best_model(
            requires_vision,
            requires_audio,
            requires_video,
            false,
            false,
        )
        .await
    }

    pub async fn upload_file(&self, file_path: &str) -> Result<QwenFile> {
        let user_id = self.chat_manager.get_user_id().await?;
        self.file_uploader.upload_file(file_path, user_id).await
    }

    pub async fn ask_question(&self, message: &str, model_id: Option<&str>) -> Result<String> {
        let response = self.start_convo(message, model_id, None).await?;
        Ok(response.content)
    }

    pub async fn start_convo(
        &self,
        message: &str,
        model_id: Option<&str>,
        extra_data: Option<&ExtraData>,
    ) -> Result<QwenResponse> {
        self.start_convo_with_files(message, vec![], model_id, extra_data)
            .await
    }

    pub async fn start_convo_with_files(
        &self,
        message: &str,
        files: Vec<QwenFile>,
        model_id: Option<&str>,
        extra_data: Option<&ExtraData>,
    ) -> Result<QwenResponse> {
        let token = self.auth.get_token().await?;

        let model = if let Some(id) = model_id {
            id.to_string()
        } else if !files.is_empty() {
            self.select_model_for_files(&files).await?
        } else {
            "qwen3-max".to_string()
        };

        let chat_id = if let Some(data) = extra_data {
            data.chat_id.clone()
        } else {
            self.chat_manager.create_or_get_chat(&model).await?
        };

        let url = format!(
            "{}/api/v2/chat/completions?chat_id={}",
            super::modules::constants::BASE_URL,
            chat_id
        );
        let headers = build_json_headers(Some(&token));

        let parent_id = extra_data.and_then(|d| d.parent_id.clone());

        let completion_request = ConversationBuilder::build_completion_request(
            message,
            &model,
            files,
            chat_id.clone(),
            parent_id.clone(),
            false,
            false,
            None,
        );

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&completion_request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(crate::qwen::error::QwenError::ApiError(format!(
                "Chat completion failed ({}): {}",
                status, error_text
            )));
        }

        let output = StreamingHandler::handle_streaming_response(response).await?;

        Ok(QwenResponse {
            content: output.content,
            response_id: output.response_id,
            chat_id: Some(chat_id),
            parent_id,
            web_search_results: output.web_search_results,
            thinking_content: output.thinking_content,
        })
    }

    pub async fn continue_convo(
        &self,
        message: &str,
        chat_id: &str,
        parent_id: Option<&str>,
        model_id: Option<&str>,
        _extra_data: Option<&ExtraData>,
    ) -> Result<QwenResponse> {
        let extra = ExtraData {
            chat_id: chat_id.to_string(),
            model_id: model_id.unwrap_or("qwen3-max").to_string(),
            parent_id: parent_id.map(|s| s.to_string()),
        };

        self.start_convo_with_files(message, vec![], model_id, Some(&extra))
            .await
    }

    pub async fn start_convo_with_search(
        &self,
        message: &str,
        model_id: Option<&str>,
        extra_data: Option<&ExtraData>,
    ) -> Result<QwenResponse> {
        self.start_convo_with_options(message, model_id, extra_data, true, false, None)
            .await
    }

    pub async fn start_convo_with_thinking(
        &self,
        message: &str,
        model_id: Option<&str>,
        extra_data: Option<&ExtraData>,
        thinking_budget: Option<u32>,
    ) -> Result<QwenResponse> {
        self.start_convo_with_options(message, model_id, extra_data, false, true, thinking_budget)
            .await
    }

    async fn start_convo_with_options(
        &self,
        message: &str,
        model_id: Option<&str>,
        extra_data: Option<&ExtraData>,
        enable_search: bool,
        enable_thinking: bool,
        thinking_budget: Option<u32>,
    ) -> Result<QwenResponse> {
        let token = self.auth.get_token().await?;

        let model = model_id.unwrap_or("qwen3-max");
        let chat_id = if let Some(data) = extra_data {
            data.chat_id.clone()
        } else {
            self.chat_manager.create_or_get_chat(model).await?
        };

        let url = format!(
            "{}/api/v2/chat/completions?chat_id={}",
            super::modules::constants::BASE_URL,
            chat_id
        );
        let headers = build_json_headers(Some(&token));

        let parent_id = extra_data.and_then(|d| d.parent_id.clone());

        let completion_request = ConversationBuilder::build_completion_request(
            message,
            model,
            vec![],
            chat_id.clone(),
            parent_id.clone(),
            enable_search,
            enable_thinking,
            thinking_budget,
        );

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&completion_request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(crate::qwen::error::QwenError::ApiError(format!(
                "Chat completion failed ({}): {}",
                status, error_text
            )));
        }

        let output = StreamingHandler::handle_streaming_response(response).await?;

        Ok(QwenResponse {
            content: output.content,
            response_id: output.response_id,
            chat_id: Some(chat_id),
            parent_id: None,
            web_search_results: output.web_search_results,
            thinking_content: output.thinking_content,
        })
    }

    // ============================================================
    // Multimodal Output Generation
    // ============================================================

    /// Generate an image from text prompt
    pub async fn generate_image(
        &self,
        prompt: &str,
        size: Option<&str>,
        model_id: Option<&str>,
        extra_data: Option<&ExtraData>,
    ) -> Result<QwenResponse> {
        let token = self.auth.get_token().await?;
        self.media_generator
            .generate_image(
                prompt,
                size,
                model_id,
                extra_data,
                &token,
                &self.chat_manager,
            )
            .await
    }

    /// Generate a video from text prompt
    pub async fn generate_video(
        &self,
        prompt: &str,
        size: Option<&str>,
        model_id: Option<&str>,
        extra_data: Option<&ExtraData>,
    ) -> Result<QwenResponse> {
        let token = self.auth.get_token().await?;
        self.media_generator
            .generate_video(
                prompt,
                size,
                model_id,
                extra_data,
                &token,
                &self.chat_manager,
            )
            .await
    }

    /// Generate a video with progress callback
    pub async fn generate_video_with_progress<F>(
        &self,
        prompt: &str,
        size: Option<&str>,
        model_id: Option<&str>,
        extra_data: Option<&ExtraData>,
        progress_callback: F,
    ) -> Result<QwenResponse>
    where
        F: Fn(&str, u8) + Send + Sync,
    {
        let token = self.auth.get_token().await?;
        self.media_generator
            .generate_video_with_progress(
                prompt,
                size,
                model_id,
                extra_data,
                &token,
                &self.chat_manager,
                progress_callback,
            )
            .await
    }

    /// Download media (image or video) to local file
    pub async fn download_media(&self, url: &str, output_path: &str) -> Result<()> {
        self.media_downloader.download_media(url, output_path).await
    }
}
