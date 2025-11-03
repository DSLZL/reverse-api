use super::chat_manager::ChatManager;
use super::constants::{build_json_headers, BASE_URL};
use super::streaming::ConversationBuilder;
use crate::qwen::error::Result;
use crate::qwen::models::{ExtraData, QwenResponse, TaskResponse, TaskStatus};

pub struct MediaGenerator {
    client: rquest::Client,
}

impl MediaGenerator {
    pub fn new(client: rquest::Client) -> Self {
        Self { client }
    }

    /// Generate an image from text prompt
    pub async fn generate_image(
        &self,
        prompt: &str,
        size: Option<&str>,
        model_id: Option<&str>,
        extra_data: Option<&ExtraData>,
        token: &str,
        chat_manager: &ChatManager,
    ) -> Result<QwenResponse> {
        let model = model_id.unwrap_or("qwen3-max");
        let chat_id = if let Some(data) = extra_data {
            data.chat_id.clone()
        } else {
            chat_manager.create_or_get_chat(model).await?
        };

        let url = format!("{}/api/v2/chat/completions?chat_id={}", BASE_URL, chat_id);
        let headers = build_json_headers(Some(token));

        let parent_id = extra_data.and_then(|d| d.parent_id.clone());
        let image_size = size.map(|s| s.to_string());

        let completion_request = ConversationBuilder::build_completion_request_with_chat_type(
            prompt,
            model,
            vec![],
            chat_id.clone(),
            parent_id.clone(),
            "t2i",
            image_size,
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
                "Image generation failed ({}): {}",
                status, error_text
            )));
        }

        let output =
            super::streaming::StreamingHandler::handle_streaming_response(response).await?;

        Ok(QwenResponse {
            content: output.content,
            response_id: output.response_id,
            chat_id: Some(chat_id),
            parent_id,
            web_search_results: None,
            thinking_content: None,
        })
    }

    /// Generate a video from text prompt
    pub async fn generate_video(
        &self,
        prompt: &str,
        size: Option<&str>,
        model_id: Option<&str>,
        extra_data: Option<&ExtraData>,
        token: &str,
        chat_manager: &ChatManager,
    ) -> Result<QwenResponse> {
        self.generate_video_with_progress(
            prompt,
            size,
            model_id,
            extra_data,
            token,
            chat_manager,
            |_, _| {},
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
        token: &str,
        chat_manager: &ChatManager,
        progress_callback: F,
    ) -> Result<QwenResponse>
    where
        F: Fn(&str, u8) + Send + Sync,
    {
        let model = model_id.unwrap_or("qwen3-max");
        let chat_id = if let Some(data) = extra_data {
            data.chat_id.clone()
        } else {
            chat_manager.create_or_get_chat(model).await?
        };

        let url = format!("{}/api/v2/chat/completions?chat_id={}", BASE_URL, chat_id);
        let headers = build_json_headers(Some(token));

        let parent_id = extra_data.and_then(|d| d.parent_id.clone());
        let video_size = size
            .map(|s| s.to_string())
            .or_else(|| Some("16:9".to_string()));

        // Build request with stream=false for video generation
        let mut completion_request = ConversationBuilder::build_completion_request_with_chat_type(
            prompt,
            model,
            vec![],
            chat_id.clone(),
            parent_id.clone(),
            "t2v",
            video_size,
        );

        // Override stream to false for video generation
        completion_request.stream = false;

        let response = self
            .client
            .post(&url)
            .headers(headers.clone())
            .json(&completion_request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(crate::qwen::error::QwenError::ApiError(format!(
                "Video generation failed ({}): {}",
                status, error_text
            )));
        }

        // Parse task response
        let task_response: TaskResponse = response.json().await?;

        // Extract task_id
        let task_id = task_response
            .data
            .messages
            .first()
            .and_then(|m| m.extra.as_ref())
            .and_then(|e| e.wanx.as_ref())
            .map(|w| w.task_id.clone())
            .ok_or_else(|| {
                crate::qwen::error::QwenError::ApiError("No task_id in response".to_string())
            })?;

        println!("🎬 Video generation started, task_id: {}", task_id);
        progress_callback("started", 0);

        // Poll task status
        let video_url = self
            .poll_task_status(&task_id, token, &progress_callback)
            .await?;

        Ok(QwenResponse {
            content: video_url,
            response_id: task_response.data.message_id,
            chat_id: Some(chat_id),
            parent_id: Some(task_response.data.parent_id),
            web_search_results: None,
            thinking_content: None,
        })
    }

    /// Poll task status until completion
    async fn poll_task_status<F>(
        &self,
        task_id: &str,
        token: &str,
        progress_callback: &F,
    ) -> Result<String>
    where
        F: Fn(&str, u8) + Send + Sync,
    {
        let url = format!("{}/api/v1/tasks/status/{}", BASE_URL, task_id);
        let headers = build_json_headers(Some(token));

        let max_attempts = 300; // 5 minutes with 1 second interval
        let poll_interval = std::time::Duration::from_secs(1);

        for attempt in 0..max_attempts {
            tokio::time::sleep(poll_interval).await;

            let response = self
                .client
                .get(&url)
                .headers(headers.clone())
                .send()
                .await?;

            if !response.status().is_success() {
                continue;
            }

            let task_status: TaskStatus = response.json().await?;

            match task_status.task_status.as_str() {
                "success" => {
                    progress_callback("success", 100);
                    println!("\n✅ Video generation completed!");
                    return Ok(task_status.content);
                }
                "failed" => {
                    progress_callback("failed", 0);
                    return Err(crate::qwen::error::QwenError::ApiError(format!(
                        "Video generation failed: {}",
                        task_status.message
                    )));
                }
                "running" => {
                    let progress = ((attempt as f32 / max_attempts as f32) * 100.0) as u8;
                    progress_callback("running", progress);
                    if attempt % 5 == 0 {
                        print!(".");
                        std::io::Write::flush(&mut std::io::stdout()).ok();
                    }
                }
                _ => {}
            }
        }

        Err(crate::qwen::error::QwenError::ApiError(
            "Video generation timeout".to_string(),
        ))
    }
}
