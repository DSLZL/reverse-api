use super::auth::AuthManager;
use super::constants::{build_json_headers, BASE_URL};
use crate::qwen::error::{QwenError, Result};
use crate::qwen::models::{
    ChatConfig, CreateChatRequest, CreateChatResponse, Model, ModelsResponse,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct ChatManager {
    client: rquest::Client,
    auth: Arc<AuthManager>,
    chat_cache: Arc<Mutex<HashMap<String, String>>>,
}

impl ChatManager {
    pub fn new(client: rquest::Client, auth: Arc<AuthManager>) -> Self {
        Self {
            client,
            auth,
            chat_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_models(&self) -> Result<Vec<Model>> {
        let token = self.auth.get_token().await?;
        let url = format!("{}/api/models", BASE_URL);
        let headers = build_json_headers(Some(&token));

        let response = self.client.get(&url).headers(headers).send().await?;

        let response_text = response.text().await?;
        let models_response: ModelsResponse = serde_json::from_str(&response_text)?;

        Ok(models_response.data)
    }

    pub async fn create_or_get_chat(&self, model_id: &str) -> Result<String> {
        let mut cache = self.chat_cache.lock().await;

        if let Some(chat_id) = cache.get(model_id) {
            return Ok(chat_id.clone());
        }

        let token = self.auth.get_token().await?;
        let url = format!("{}/api/v2/chats/new", BASE_URL);
        let headers = build_json_headers(Some(&token));

        let chat_name = format!("Chat {}", &Uuid::new_v4().to_string()[..8]);
        let create_request = CreateChatRequest {
            chat: ChatConfig {
                name: chat_name,
                models: vec![model_id.to_string()],
            },
        };

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&create_request)
            .send()
            .await?;

        let response_text = response.text().await?;
        let create_response: CreateChatResponse = serde_json::from_str(&response_text)?;

        if !create_response.success {
            return Err(QwenError::ApiError(format!(
                "Failed to create chat: {}",
                response_text
            )));
        }

        let chat_id = create_response.data.id.clone();
        cache.insert(model_id.to_string(), chat_id.clone());

        Ok(chat_id)
    }

    pub async fn get_user_id(&self) -> Result<String> {
        let token = self.auth.get_token().await?;
        let url = format!("{}/api/v1/users/user/settings", BASE_URL);
        let headers = build_json_headers(Some(&token));

        let response = self.client.get(&url).headers(headers).send().await?;

        if !response.status().is_success() {
            return Ok(Uuid::new_v4().to_string());
        }

        let text = response.text().await?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                return Ok(id.to_string());
            }
        }

        Ok(Uuid::new_v4().to_string())
    }
}
