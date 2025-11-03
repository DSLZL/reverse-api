use super::constants::{build_json_headers, BASE_URL};
use crate::qwen::error::{QwenError, Result};
use crate::qwen::models::{AuthResponse, SignInRequest};
use rquest::header::CONTENT_TYPE;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AuthManager {
    email: String,
    password: String,
    client: rquest::Client,
    token_cache: Arc<Mutex<Option<String>>>,
}

impl AuthManager {
    pub fn new(email: String, password: String, client: rquest::Client) -> Self {
        Self {
            email,
            password,
            client,
            token_cache: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_token(token: String, client: rquest::Client) -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            client,
            token_cache: Arc::new(Mutex::new(Some(token))),
        }
    }

    pub async fn get_token(&self) -> Result<String> {
        let mut cache = self.token_cache.lock().await;

        if let Some(token) = cache.as_ref() {
            return Ok(token.clone());
        }

        let url = format!("{}/api/v1/auths/signin", BASE_URL);
        let mut headers = build_json_headers(None);
        headers.insert(
            CONTENT_TYPE,
            rquest::header::HeaderValue::from_static("application/json"),
        );

        let signin_request = SignInRequest {
            email: self.email.clone(),
            password: self.password.clone(),
        };

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&signin_request)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(QwenError::ApiError(format!(
                "Login failed ({}): {}",
                status, response_text
            )));
        }

        let auth_response: AuthResponse = serde_json::from_str(&response_text)?;
        let token = auth_response.token.clone();
        *cache = Some(token.clone());

        Ok(token)
    }
}
