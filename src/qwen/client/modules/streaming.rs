use crate::qwen::error::Result;
use crate::qwen::models::{
    ChatCompletionRequest, Extra, FeatureConfig, Meta, QwenFile, QwenMessage,
};
use futures_util::stream::StreamExt;
use uuid::Uuid;

pub struct StreamingHandler;

impl StreamingHandler {
    pub async fn handle_streaming_response(response: rquest::Response) -> Result<StreamingOutput> {
        let mut stream = response.bytes_stream();
        let mut content = String::new();
        let mut response_id: Option<String> = None;
        let mut thinking_content = String::new();
        let mut web_search_results: Option<Vec<crate::qwen::models::WebSearchInfo>> = None;
        let mut current_phase = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);

            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = line.strip_prefix("data: ").unwrap().trim();
                    if data.is_empty() || data == "[DONE]" {
                        continue;
                    }

                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(error) = json["error"].as_object() {
                            let code = error["code"].as_str().unwrap_or("unknown");
                            let details = error["details"].as_str().unwrap_or("no details");
                            return Err(crate::qwen::error::QwenError::ApiError(format!(
                                "Server error: {} - {}",
                                code, details
                            )));
                        }

                        if response_id.is_none() {
                            if let Some(created) = json["response.created"].as_object() {
                                if let Some(rid) = created["response_id"].as_str() {
                                    response_id = Some(rid.to_string());
                                }
                            }
                        }

                        if let Some(choices) = json["choices"].as_array() {
                            for choice in choices {
                                if let Some(delta) = choice["delta"].as_object() {
                                    if let Some(phase) = delta.get("phase").and_then(|v| v.as_str())
                                    {
                                        current_phase = phase.to_string();
                                    }

                                    if current_phase == "thinking" {
                                        if let Some(think_content) =
                                            delta.get("content").and_then(|v| v.as_str())
                                        {
                                            thinking_content.push_str(think_content);
                                            print!("{}", think_content);
                                            std::io::Write::flush(&mut std::io::stdout()).ok();
                                        }
                                    }

                                    if current_phase == "web_search" {
                                        if let Some(extra) = delta.get("extra") {
                                            if let Some(search_info) = extra.get("web_search_info")
                                            {
                                                if let Ok(results) = serde_json::from_value::<
                                                    Vec<crate::qwen::models::WebSearchInfo>,
                                                >(
                                                    search_info.clone()
                                                ) {
                                                    web_search_results = Some(results);
                                                }
                                            }
                                        }
                                    }

                                    if current_phase == "image_gen" {
                                        if let Some(text) =
                                            delta.get("content").and_then(|v| v.as_str())
                                        {
                                            if !text.is_empty() {
                                                content = text.to_string(); // Replace instead of append for image URLs
                                            }
                                        }
                                    }

                                    if current_phase == "answer"
                                        || (!current_phase.contains("thinking")
                                            && !current_phase.contains("search")
                                            && current_phase != "image_gen")
                                    {
                                        if let Some(text) =
                                            delta.get("content").and_then(|v| v.as_str())
                                        {
                                            content.push_str(text);
                                            print!("{}", text);
                                            std::io::Write::flush(&mut std::io::stdout()).ok();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        println!();

        Ok(StreamingOutput {
            content,
            response_id: response_id.unwrap_or_default(),
            thinking_content: if thinking_content.is_empty() {
                None
            } else {
                Some(thinking_content)
            },
            web_search_results,
        })
    }
}

pub struct StreamingOutput {
    pub content: String,
    pub response_id: String,
    pub thinking_content: Option<String>,
    pub web_search_results: Option<Vec<crate::qwen::models::WebSearchInfo>>,
}

pub struct ConversationBuilder;

impl ConversationBuilder {
    pub fn build_message(
        message: &str,
        model: &str,
        files: Vec<QwenFile>,
        parent_id: Option<String>,
        chat_type: &str,
    ) -> QwenMessage {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        QwenMessage {
            fid: Uuid::new_v4().to_string(),
            parent_id: parent_id.clone(),
            children_ids: vec![],
            role: "user".to_string(),
            content: message.to_string(),
            user_action: "chat".to_string(),
            files,
            timestamp,
            models: vec![model.to_string()],
            chat_type: chat_type.to_string(),
            feature_config: FeatureConfig {
                thinking_enabled: false,
                output_schema: "phase".to_string(),
                research_mode: "normal".to_string(),
                thinking_budget: None,
            },
            extra: Extra {
                meta: Meta {
                    sub_chat_type: chat_type.to_string(),
                },
            },
            sub_chat_type: chat_type.to_string(),
        }
    }

    pub fn build_completion_request(
        message: &str,
        model: &str,
        files: Vec<QwenFile>,
        chat_id: String,
        parent_id: Option<String>,
        enable_search: bool,
        _enable_thinking: bool,
        _thinking_budget: Option<u32>,
    ) -> ChatCompletionRequest {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let chat_type = if enable_search { "search" } else { "t2t" };

        ChatCompletionRequest {
            stream: true,
            incremental_output: true,
            chat_id,
            chat_mode: "normal".to_string(),
            model: model.to_string(),
            parent_id: parent_id.clone(),
            messages: vec![Self::build_message(
                message, model, files, parent_id, chat_type,
            )],
            timestamp,
            size: None,
        }
    }

    pub fn build_completion_request_with_chat_type(
        message: &str,
        model: &str,
        files: Vec<QwenFile>,
        chat_id: String,
        parent_id: Option<String>,
        chat_type: &str,
        size: Option<String>,
    ) -> ChatCompletionRequest {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        ChatCompletionRequest {
            stream: true,
            incremental_output: true,
            chat_id,
            chat_mode: "normal".to_string(),
            model: model.to_string(),
            parent_id: parent_id.clone(),
            messages: vec![Self::build_message(
                message, model, files, parent_id, chat_type,
            )],
            timestamp,
            size,
        }
    }
}
