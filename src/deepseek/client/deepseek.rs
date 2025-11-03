use crate::deepseek::error::{DeepSeekError, Result};
use crate::deepseek::models::{DeepSeekChatRequest, DeepSeekResponse, ExtraData};
use crate::deepseek::signature::{DeepSeekHash, DeepSeekSignature};
use base64::Engine as _;
use futures_util::stream::StreamExt;
use rand::distributions::Alphanumeric;
use rand::Rng;
use rquest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, COOKIE};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use uuid::Uuid;

const ACCESS_TOKEN_EXPIRES: u64 = 3600;
const WASM_PATH: &str = "./src/deepseek/wasm/sha3_wasm_bg.7b9ca65ddd.wasm";

const FAKE_HEADERS: &[(&str, &str)] = &[
    ("Accept", "*/*"),
    ("Accept-Encoding", "gzip, deflate, br, zstd"),
    ("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8"),
    ("Origin", "https://chat.deepseek.com"),
    ("Pragma", "no-cache"),
    ("Priority", "u=1, i"),
    ("Referer", "https://chat.deepseek.com/"),
    ("Sec-Ch-Ua", "\"Chromium\";v=\"134\", \"Not:A-Brand\";v=\"24\", \"Google Chrome\";v=\"134\""),
    ("Sec-Ch-Ua-Mobile", "?0"),
    ("Sec-Ch-Ua-Platform", "\"macOS\""),
    ("Sec-Fetch-Dest", "empty"),
    ("Sec-Fetch-Mode", "cors"),
    ("Sec-Fetch-Site", "same-origin"),
    ("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36"),
    ("X-App-Version", "20241129.1"),
    ("X-Client-Locale", "zh-CN"),
    ("X-Client-Platform", "web"),
    ("X-Client-Version", "1.0.0-always"),
];

// Helper to get current Unix timestamp
fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// Helper to generate a random string (simplified version of randomstring.generate)
fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

// Helper to generate cookies similar to the TypeScript project
fn generate_cookie() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let _rng = rand::thread_rng(); // Mark as unused

    let intercom_ts = timestamp;
    let hwafsesid = generate_random_string(18);
    let hm_lvt_uuid = Uuid::new_v4().as_simple().to_string().replace("-", "");
    let hm_lpvt_uuid = Uuid::new_v4().as_simple().to_string().replace("-", "");
    let frid_uuid = Uuid::new_v4().as_simple().to_string().replace("-", "");
    let fr_ssid_uuid = Uuid::new_v4().as_simple().to_string().replace("-", "");
    let fr_pvid_uuid = Uuid::new_v4().as_simple().to_string().replace("-", "");

    format!(
        "intercom-HWWAFSESTIME={}; HWWAFSESID={}; Hm_lvt_{}={},{}; Hm_lpvt_{}={}; _frid={}; _fr_ssid={}; _fr_pvid={}",
        intercom_ts,
        hwafsesid,
        hm_lvt_uuid,
        get_timestamp(),
        get_timestamp(),
        hm_lpvt_uuid,
        get_timestamp(),
        frid_uuid,
        fr_ssid_uuid,
        fr_pvid_uuid
    )
}

pub struct DeepSeekClient {
    api_key: String,
    client: rquest::Client,
    deepseek_hash: Arc<Mutex<DeepSeekHash>>,
    access_token_cache: Arc<Mutex<HashMap<String, (String, u64)>>>,
}

impl DeepSeekClient {
    pub async fn new(api_key: String) -> Result<Self> {
        let deepseek_hash = DeepSeekHash::new(WASM_PATH).await?;
        Ok(Self {
            api_key,
            client: rquest::Client::builder().cookie_store(true).build()?,
            deepseek_hash: Arc::new(Mutex::new(deepseek_hash)),
            access_token_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    async fn acquire_token(&self, refresh_token: &str) -> Result<String> {
        let mut cache = self.access_token_cache.lock().await;
        if let Some((token, expiry)) = cache.get(refresh_token) {
            if *expiry > get_timestamp() {
                return Ok(token.clone());
            }
        }

        let signature_client = DeepSeekSignature::new();
        let new_token = signature_client.get_token(refresh_token).await?;
        let expiry = get_timestamp() + ACCESS_TOKEN_EXPIRES;
        cache.insert(refresh_token.to_string(), (new_token.clone(), expiry));
        Ok(new_token)
    }

    pub async fn create_session(&self, access_token: &str) -> Result<String> {
        let url = "https://chat.deepseek.com/api/v0/chat_session/create";
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", access_token).parse().unwrap(),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(COOKIE, generate_cookie().parse().unwrap());
        // Add FAKE_HEADERS
        for (key, value) in FAKE_HEADERS {
            headers.insert(*key, HeaderValue::from_static(value));
        }

        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&json!({ "character_id": null }))
            .send()
            .await?;

        let response_text = response.text().await?;
        let json_response: serde_json::Value = serde_json::from_str(&response_text)?;

        // Check for error response
        if let Some(code) = json_response["code"].as_i64() {
            if code != 0 {
                let msg = json_response["msg"].as_str().unwrap_or("Unknown error");
                return Err(DeepSeekError::ApiError(format!(
                    "Create session failed (code {}): {}",
                    code, msg
                )));
            }
        }

        if let Some(session_id) = json_response["data"]["biz_data"]["id"].as_str() {
            Ok(session_id.to_string())
        } else {
            Err(DeepSeekError::ApiError(format!(
                "Failed to parse session ID from response: {}",
                response_text
            )))
        }
    }

    async fn get_challenge_response(
        &self,
        access_token: &str,
        target_path: &str,
    ) -> Result<serde_json::Value> {
        let url = "https://chat.deepseek.com/api/v0/chat/create_pow_challenge";
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", access_token).parse().unwrap(),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(COOKIE, generate_cookie().parse().unwrap());
        // Add FAKE_HEADERS
        for (key, value) in FAKE_HEADERS {
            headers.insert(*key, HeaderValue::from_static(value));
        }

        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&json!({ "target_path": target_path }))
            .send()
            .await?;

        let response_text = response.text().await?;
        let json_response: serde_json::Value = serde_json::from_str(&response_text)?;

        // Check for error response
        if let Some(code) = json_response["code"].as_i64() {
            if code != 0 {
                let msg = json_response["msg"].as_str().unwrap_or("Unknown error");
                return Err(DeepSeekError::ApiError(format!(
                    "Get challenge failed (code {}): {}",
                    code, msg
                )));
            }
        }

        if let Some(challenge) = json_response["data"]["biz_data"]["challenge"].as_object() {
            Ok(serde_json::Value::Object(challenge.clone()))
        } else {
            Err(DeepSeekError::ApiError(format!(
                "Failed to parse challenge from response: {}",
                response_text
            )))
        }
    }

    async fn answer_challenge(
        &self,
        challenge_response: serde_json::Value,
        target_path: &str,
    ) -> Result<String> {
        let algorithm = challenge_response["algorithm"]
            .as_str()
            .ok_or_else(|| DeepSeekError::ApiError("Missing algorithm".to_string()))?;
        let challenge = challenge_response["challenge"]
            .as_str()
            .ok_or_else(|| DeepSeekError::ApiError("Missing challenge".to_string()))?;
        let salt = challenge_response["salt"]
            .as_str()
            .ok_or_else(|| DeepSeekError::ApiError("Missing salt".to_string()))?;
        let difficulty = challenge_response["difficulty"]
            .as_i64()
            .ok_or_else(|| DeepSeekError::ApiError("Missing difficulty".to_string()))?
            as f64;
        let expire_at = challenge_response["expire_at"]
            .as_i64()
            .ok_or_else(|| DeepSeekError::ApiError("Missing expire_at".to_string()))?;
        let signature = challenge_response["signature"]
            .as_str()
            .ok_or_else(|| DeepSeekError::ApiError("Missing signature".to_string()))?;

        let mut deepseek_hash = self.deepseek_hash.lock().await;
        let answer_opt =
            deepseek_hash.calculate_hash(algorithm, challenge, salt, difficulty, expire_at)?;

        let pow_response = json!({
            "algorithm": algorithm,
            "challenge": challenge,
            "salt": salt,
            "answer": answer_opt,
            "signature": signature,
            "target_path": target_path,
        });

        Ok(base64::engine::general_purpose::STANDARD.encode(pow_response.to_string()))
    }

    pub async fn start_convo(
        &self,
        message: &str,
        extra_data: Option<&ExtraData>,
    ) -> Result<DeepSeekResponse> {
        let access_token = self.acquire_token(&self.api_key).await?;

        // Use existing session or create new one
        let session_id = if let Some(data) = extra_data {
            data.session_id.clone()
        } else {
            self.create_session(&access_token).await?
        };

        let target_path = "/api/v0/chat/completion";
        let challenge_response = self
            .get_challenge_response(&access_token, target_path)
            .await?;
        let pow_response = self
            .answer_challenge(challenge_response, target_path)
            .await?;

        // Build DeepSeek API request
        let deepseek_request = DeepSeekChatRequest {
            chat_session_id: session_id.clone(),
            parent_message_id: extra_data.map(|d| d.message_id.clone()),
            prompt: message.to_string(),
            ref_file_ids: vec![],
            search_enabled: false,
            thinking_enabled: false,
        };

        let url = "https://chat.deepseek.com/api/v0/chat/completion";
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", access_token).parse().unwrap(),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(COOKIE, generate_cookie().parse().unwrap());
        headers.insert("X-Ds-Pow-Response", pow_response.parse().unwrap());
        // Add FAKE_HEADERS
        for (key, value) in FAKE_HEADERS {
            headers.insert(*key, HeaderValue::from_static(value));
        }

        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&deepseek_request)
            .send()
            .await?;

        let status = response.status();

        // Check if it's a stream response
        if let Some(content_type) = response.headers().get("content-type") {
            if content_type
                .to_str()
                .unwrap_or("")
                .contains("text/event-stream")
            {
                // Handle streaming response
                let mut stream = response.bytes_stream();
                let mut content = String::new();
                let mut message_id = String::new();

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
                                // Handle "ready" event with message IDs
                                if let Some(resp_msg_id) = json["response_message_id"].as_i64() {
                                    message_id = resp_msg_id.to_string();
                                }

                                // Handle content updates - DeepSeek format: {"v": "text", "p": "response/content", "o": "APPEND"}
                                if let Some(text_value) = json["v"].as_str() {
                                    if let Some(path) = json["p"].as_str() {
                                        if path.contains("response/content") {
                                            content.push_str(text_value);
                                            print!("{}", text_value);
                                            std::io::Write::flush(&mut std::io::stdout()).ok();
                                        }
                                    } else if !text_value.is_empty() {
                                        // Sometimes "v" comes without "p"
                                        content.push_str(text_value);
                                        print!("{}", text_value);
                                        std::io::Write::flush(&mut std::io::stdout()).ok();
                                    }
                                }
                            }
                        }
                    }
                }

                println!(); // New line after streaming

                return Ok(DeepSeekResponse {
                    response: Some(content),
                    extra_data: ExtraData {
                        session_id,
                        message_id,
                    },
                });
            }
        }

        // Fall back to non-streaming response
        let response_text = response.text().await?;

        Err(DeepSeekError::ApiError(format!(
            "Unexpected non-streaming response ({}): {}",
            status, response_text
        )))
    }

    /// Simple method to ask a question (creates new session each time)
    pub async fn ask_question(&self, message: &str) -> Result<String> {
        let response = self.start_convo(message, None).await?;
        Ok(response.response.unwrap_or_default())
    }
}
