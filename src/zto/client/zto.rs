use crate::zto::auth::{build_request_headers, get_anon_token};
use crate::zto::error::{Result, ZtoError};
use crate::zto::models::{Features, Message, ZtoRequest};
use crate::zto::parser::parse_stream_response;
use crate::zto::signature::generate_signature;
use crate::zto::utils::{generate_chat_id, generate_message_id};
use rquest::Client;

const UPSTREAM_URL: &str = "https://chat.z.ai/api/chat/completions";
const MAX_RETRIES: u32 = 3;

pub struct ZtoClient {
    client: Client,
    token: Option<String>,
    #[allow(dead_code)]
    proxy: Option<String>,
}

impl ZtoClient {
    pub async fn new(proxy: Option<&str>) -> Result<Self> {
        let client = if let Some(proxy_url) = proxy {
            let proxy = rquest::Proxy::all(proxy_url)
                .map_err(|e| ZtoError::Other(format!("Proxy error: {}", e)))?;
            Client::builder()
                .proxy(proxy)
                .build()
                .map_err(|e| ZtoError::Other(format!("Client build error: {}", e)))?
        } else {
            Client::new()
        };

        let token = get_anon_token(&client).await?;

        Ok(ZtoClient {
            client,
            token: Some(token),
            proxy: proxy.map(|s| s.to_string()),
        })
    }

    pub async fn ask_question(&mut self, question: &str) -> Result<String> {
        let messages = vec![Message {
            role: "user".to_string(),
            content: question.to_string(),
        }];

        self.call_api_with_retry(messages).await
    }

    pub async fn ask_question_with_context(
        &mut self,
        question: &str,
        mut context: Vec<Message>,
    ) -> Result<String> {
        context.push(Message {
            role: "user".to_string(),
            content: question.to_string(),
        });

        self.call_api_with_retry(context).await
    }

    async fn call_api_with_retry(&mut self, messages: Vec<Message>) -> Result<String> {
        let mut retry_count = 0;

        loop {
            match self.call_api(&messages).await {
                Ok(response) => return Ok(response),
                Err(ZtoError::UnauthorizedError(_)) if retry_count < MAX_RETRIES => {
                    retry_count += 1;
                    self.token = Some(get_anon_token(&self.client).await?);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn call_api(&mut self, messages: &[Message]) -> Result<String> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| ZtoError::UnauthorizedError("No token available".to_string()))?
            .clone();

        let request = ZtoRequest {
            stream: true,
            chat_id: generate_chat_id(),
            id: generate_message_id(),
            model: "GLM-4-6-API-V1".to_string(),
            messages: messages.to_vec(),
            features: Some(Features {
                enable_thinking: false,
            }),
            params: None,
            background_tasks: None,
        };

        let json_body = serde_json::to_vec(&request)
            .map_err(|e| ZtoError::ParseError(format!("JSON serialization error: {}", e)))?;

        let signature = generate_signature(&json_body);

        let mut headers = build_request_headers(&token);
        headers.insert(
            "X-Signature",
            signature
                .parse()
                .map_err(|e| ZtoError::Other(format!("Header parse error: {}", e)))?,
        );

        let response = self
            .client
            .post(UPSTREAM_URL)
            .headers(headers)
            .body(json_body)
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| ZtoError::RequestFailed(format!("Request failed: {}", e)))?;

        match response.status().as_u16() {
            200 => parse_stream_response(response).await,
            401 => Err(ZtoError::UnauthorizedError(
                "Token invalid or expired".to_string(),
            )),
            status => Err(ZtoError::ServerError(format!(
                "Server returned status: {}",
                status
            ))),
        }
    }

    pub async fn refresh_token(&mut self) -> Result<()> {
        self.token = Some(get_anon_token(&self.client).await?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_zto_client_creation() {
        let result = ZtoClient::new(None).await;
        assert!(result.is_ok());
    }
}
