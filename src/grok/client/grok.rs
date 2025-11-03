use crate::grok::*;
use base64::{engine::general_purpose, Engine as _};
use rquest::Client;
use rquest_util::Emulation;
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

pub struct Grok {
    client: Client,
    model: String,
    model_mode: String,
    mode: String,
    c_run: usize,
    keys: HashMap<String, Vec<u8>>,
    anon_user: Option<String>,
    challenge_dict: Option<HashMap<String, String>>,
    verification_token: Option<String>,
    anim: Option<String>,
    svg_data: Option<String>,
    numbers: Option<Vec<usize>>,
    actions: Vec<String>,
    xsid_script: String,
    baggage: String,
    sentry_trace: String,
    cookies: HashMap<String, String>,
}

impl Grok {
    pub fn new(model: &str, proxy: Option<&str>) -> Result<Self> {
        let models = Models::new();
        let model_mode = models.get_model_mode(model).clone();
        let mode = models.get_mode(model).clone();

        let mut client_builder = Client::builder()
            .emulation(Emulation::Chrome136)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")
            .cookie_store(true);

        if let Some(proxy_url) = proxy {
            let proxy = rquest::Proxy::all(proxy_url)
                .map_err(|e| GrokError::InvalidProxy(e.to_string()))?;
            client_builder = client_builder.proxy(proxy);
        }

        let client = client_builder
            .build()
            .map_err(|e| GrokError::HttpError(format!("{}", e)))?;

        let keys = Anon::generate_keys()?;

        Ok(Self {
            client,
            model: model.to_string(),
            model_mode,
            mode,
            c_run: 0,
            keys,
            anon_user: None,
            challenge_dict: None,
            verification_token: None,
            anim: None,
            svg_data: None,
            numbers: None,
            actions: Vec::new(),
            xsid_script: String::new(),
            baggage: String::new(),
            sentry_trace: String::new(),
            cookies: HashMap::new(),
        })
    }

    async fn load(&mut self, extra_data: Option<&ExtraData>) -> Result<()> {
        if let Some(data) = extra_data {
            self.cookies = data.cookies.clone();
            self.actions = data.actions.clone();
            self.xsid_script = data.xsid_script.clone();
            self.baggage = data.baggage.clone();
            self.sentry_trace = data.sentry_trace.clone();
        } else {
            let response = self
                .client
                .get("https://grok.com/c")
                .header("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
                .header("accept-language", "de-DE,de;q=0.9,en-US;q=0.8,en;q=0.7")
                .header("cache-control", "no-cache")
                .send()
                .await?;

            // Extract cookies
            if let Some(cookie_header) = response.headers().get("set-cookie") {
                if let Ok(cookie_str) = cookie_header.to_str() {
                    for cookie in cookie_str.split(';') {
                        let parts: Vec<&str> = cookie.split('=').collect();
                        if parts.len() >= 2 {
                            self.cookies
                                .insert(parts[0].trim().to_string(), parts[1].trim().to_string());
                        }
                    }
                }
            }

            let html = response.text().await?;

            // Parse HTML in a separate scope to ensure Html and Selector don't leak
            let (scripts, baggage, sentry_trace) = {
                use scraper::{Html, Selector};
                let document = Html::parse_document(&html);
                let script_selector = Selector::parse("script[src]").unwrap();

                let scripts: Vec<String> = document
                    .select(&script_selector)
                    .filter_map(|el| el.value().attr("src"))
                    .filter(|src| src.starts_with("/_next/static/chunks/"))
                    .map(|s| s.to_string())
                    .collect();

                let baggage = Utils::between(&html, "<meta name=\"baggage\" content=\"", "\"")?;
                let sentry_trace =
                    Utils::between(&html, "<meta name=\"sentry-trace\" content=\"", "-")?;

                (scripts, baggage, sentry_trace)
            };

            let (actions, xsid_script) = Parser::parse_grok(scripts, &self.client).await?;
            self.actions = actions;
            self.xsid_script = xsid_script;
            self.baggage = baggage;
            self.sentry_trace = sentry_trace;
        }

        Ok(())
    }

    async fn c_request(&mut self, next_action: &str) -> Result<()> {
        let sentry_trace = format!(
            "{}-{}-0",
            self.sentry_trace,
            &Uuid::new_v4().simple().to_string()[..16]
        );

        if self.c_run == 0 {
            // First request with multipart
            let public_key = self.keys.get("userPublicKey").unwrap();

            let form = rquest::multipart::Form::new()
                .part(
                    "1",
                    rquest::multipart::Part::bytes(public_key.clone())
                        .file_name("blob")
                        .mime_str("application/octet-stream")
                        .unwrap(),
                )
                .text("0", "[{\"userPublicKey\":\"$o1\"}]");

            let response = self
                .client
                .post("https://grok.com/c")
                .header("accept", "text/x-component")
                .header("baggage", &self.baggage)
                .header("next-action", next_action)
                .header("next-router-state-tree", "%5B%22%22%2C%7B%22children%22%3A%5B%22c%22%2C%7B%22children%22%3A%5B%5B%22slug%22%2C%22%22%2C%22oc%22%5D%2C%7B%22children%22%3A%5B%22__PAGE__%22%2C%7B%7D%2Cnull%2Cnull%5D%7D%2Cnull%2Cnull%5D%7D%2Cnull%2Cnull%5D%7D%2Cnull%2Cnull%2Ctrue%5D")
                .header("sentry-trace", sentry_trace)
                .multipart(form)
                .send()
                .await?;

            let text = response.text().await?;
            self.anon_user = Some(Utils::between(&text, "{\"anonUserId\":\"", "\"")?);
            self.c_run += 1;
        } else {
            let data = match self.c_run {
                1 => json!([{"anonUserId": self.anon_user.as_ref().unwrap()}]),
                2 => {
                    let mut obj = serde_json::Map::new();
                    obj.insert(
                        "anonUserId".to_string(),
                        json!(self.anon_user.as_ref().unwrap()),
                    );
                    if let Some(challenge) = &self.challenge_dict {
                        obj.insert(
                            "challenge".to_string(),
                            json!(challenge.get("challenge").unwrap()),
                        );
                        obj.insert(
                            "signature".to_string(),
                            json!(challenge.get("signature").unwrap()),
                        );
                    }
                    json!([obj])
                }
                _ => json!([]),
            };

            let response = self
                .client
                .post("https://grok.com/c")
                .header("accept", "text/x-component")
                .header("content-type", "text/plain;charset=UTF-8")
                .header("baggage", &self.baggage)
                .header("next-action", next_action)
                .header("next-router-state-tree", "%5B%22%22%2C%7B%22children%22%3A%5B%22c%22%2C%7B%22children%22%3A%5B%5B%22slug%22%2C%22%22%2C%22oc%22%5D%2C%7B%22children%22%3A%5B%22__PAGE__%22%2C%7B%7D%2Cnull%2Cnull%5D%7D%2Cnull%2Cnull%5D%7D%2Cnull%2Cnull%5D%7D%2Cnull%2Cnull%2Ctrue%5D")
                .header("sentry-trace", sentry_trace)
                .body(data.to_string())
                .send()
                .await?;

            let bytes = response.bytes().await?;
            let text = String::from_utf8_lossy(&bytes);

            match self.c_run {
                1 => {
                    // Extract challenge
                    let hex = hex::encode(&bytes);
                    if let Some(start_idx) = hex.find("3a6f38362c") {
                        let start = start_idx + "3a6f38362c".len();
                        if let Some(end_idx) = hex[start..].find("313a") {
                            let challenge_hex = &hex[start..start + end_idx];
                            if let Ok(challenge_bytes) = hex::decode(challenge_hex) {
                                let private_key_b64 = general_purpose::STANDARD
                                    .encode(self.keys.get("privateKey").unwrap());
                                self.challenge_dict =
                                    Some(Anon::sign_challenge(&challenge_bytes, &private_key_b64)?);
                                Logger::success(&format!(
                                    "Solved Challenge: {:?}",
                                    self.challenge_dict
                                ));
                            }
                        }
                    }
                }
                2 => {
                    let (verification_token, anim) =
                        Parser::get_anim(&text, "grok-site-verification")?;
                    self.verification_token = Some(verification_token.clone());
                    self.anim = Some(anim.clone());

                    let (svg_data, numbers) =
                        Parser::parse_values(&text, &anim, &self.xsid_script, &self.client)?;
                    self.svg_data = Some(svg_data);
                    self.numbers = Some(numbers);
                }
                _ => {}
            }

            self.c_run += 1;
        }

        Ok(())
    }

    pub async fn start_convo(
        &mut self,
        message: &str,
        extra_data: Option<&ExtraData>,
    ) -> Result<GrokResponse> {
        let xsid: String;
        let conversation_id: Option<String>;
        let url: String;
        let is_new_convo = extra_data.is_none();

        if is_new_convo {
            self.load(None).await?;
            self.c_request(&self.actions[0].clone()).await?;
            self.c_request(&self.actions[1].clone()).await?;
            self.c_request(&self.actions[2].clone()).await?;

            xsid = Signature::generate_sign(
                "/rest/app-chat/conversations/new",
                "POST",
                self.verification_token.as_ref().unwrap(),
                self.svg_data.as_ref().unwrap(),
                self.numbers.as_ref().unwrap(),
                None,
                None,
            )?;
            conversation_id = None;
            url = "https://grok.com/rest/app-chat/conversations/new".to_string();
        } else {
            let data = extra_data.unwrap();
            self.load(Some(data)).await?;
            self.c_run = 1;
            self.anon_user = Some(data.anon_user.clone());
            self.keys.insert(
                "privateKey".to_string(),
                general_purpose::STANDARD.decode(&data.private_key)?,
            );

            self.c_request(&self.actions[1].clone()).await?;
            self.c_request(&self.actions[2].clone()).await?;

            let conv_id = data
                .conversation_id
                .as_ref()
                .ok_or(GrokError::MissingField("conversationId".to_string()))?;

            xsid = Signature::generate_sign(
                &format!("/rest/app-chat/conversations/{}/responses", conv_id),
                "POST",
                self.verification_token.as_ref().unwrap(),
                self.svg_data.as_ref().unwrap(),
                self.numbers.as_ref().unwrap(),
                None,
                None,
            )?;
            conversation_id = Some(conv_id.clone());
            url = format!(
                "https://grok.com/rest/app-chat/conversations/{}/responses",
                conv_id
            );
        }

        let conversation_data = if is_new_convo {
            json!({
                "temporary": false,
                "modelName": self.model,
                "message": message,
                "fileAttachments": [],
                "imageAttachments": [],
                "disableSearch": false,
                "enableImageGeneration": true,
                "returnImageBytes": false,
                "returnRawGrokInXaiRequest": false,
                "enableImageStreaming": true,
                "imageGenerationCount": 2,
                "forceConcise": false,
                "toolOverrides": {},
                "enableSideBySide": true,
                "sendFinalMetadata": true,
                "isReasoning": false,
                "webpageUrls": [],
                "disableTextFollowUps": false,
                "responseMetadata": {
                    "requestModelDetails": {
                        "modelId": self.model,
                    },
                },
                "disableMemory": false,
                "forceSideBySide": false,
                "modelMode": self.model_mode,
                "isAsyncChat": false,
            })
        } else {
            json!({
                "message": message,
                "modelName": self.model,
                "parentResponseId": extra_data.unwrap().parent_response_id,
                "disableSearch": false,
                "enableImageGeneration": true,
                "imageAttachments": [],
                "returnImageBytes": false,
                "returnRawGrokInXaiRequest": false,
                "fileAttachments": [],
                "enableImageStreaming": true,
                "imageGenerationCount": 2,
                "forceConcise": false,
                "toolOverrides": {},
                "enableSideBySide": true,
                "sendFinalMetadata": true,
                "customPersonality": "",
                "isReasoning": false,
                "webpageUrls": [],
                "metadata": {
                    "requestModelDetails": {
                        "modelId": self.model,
                    },
                    "request_metadata": {
                        "model": self.model,
                        "mode": self.mode,
                    },
                },
                "disableTextFollowUps": false,
                "disableArtifact": false,
                "isFromGrokFiles": false,
                "disableMemory": false,
                "forceSideBySide": false,
                "modelMode": self.model_mode,
                "isAsyncChat": false,
                "skipCancelCurrentInflightRequests": false,
                "isRegenRequest": false,
            })
        };

        let sentry_trace = format!(
            "{}-{}-0",
            self.sentry_trace,
            &Uuid::new_v4().simple().to_string()[..16]
        );

        let response = self
            .client
            .post(&url)
            .header("accept", "*/*")
            .header("content-type", "application/json")
            .header("baggage", &self.baggage)
            .header("sentry-trace", sentry_trace)
            .header("x-statsig-id", xsid)
            .header("x-xai-request-id", Uuid::new_v4().to_string())
            .json(&conversation_data)
            .send()
            .await?;

        let text = response.text().await?;

        if text.contains("rejected by anti-bot rules") {
            return Err(GrokError::AntiBotRejection);
        }

        if !text.contains("modelResponse") {
            return Err(GrokError::Other(format!("Unexpected response: {}", text)));
        }

        let mut full_response: Option<String> = None;
        let mut stream_response = Vec::new();
        let mut conv_id = conversation_id;
        let mut parent_response_id: Option<String> = None;
        let mut image_urls: Option<Vec<String>> = None;

        for line in text.lines() {
            if let Ok(data) = serde_json::from_str::<Value>(line) {
                // Extract token
                if let Some(token) = data["result"]["response"]["token"].as_str() {
                    stream_response.push(token.to_string());
                } else if let Some(token) = data["result"]["token"].as_str() {
                    stream_response.push(token.to_string());
                }

                // Extract full response
                if full_response.is_none() {
                    if let Some(msg) =
                        data["result"]["response"]["modelResponse"]["message"].as_str()
                    {
                        full_response = Some(msg.to_string());
                    } else if let Some(msg) = data["result"]["modelResponse"]["message"].as_str() {
                        full_response = Some(msg.to_string());
                    }
                }

                // Extract conversation ID
                if conv_id.is_none() {
                    if let Some(id) = data["result"]["conversation"]["conversationId"].as_str() {
                        conv_id = Some(id.to_string());
                    }
                }

                // Extract parent response ID
                if parent_response_id.is_none() {
                    if let Some(id) =
                        data["result"]["response"]["modelResponse"]["responseId"].as_str()
                    {
                        parent_response_id = Some(id.to_string());
                    } else if let Some(id) = data["result"]["modelResponse"]["responseId"].as_str()
                    {
                        parent_response_id = Some(id.to_string());
                    }
                }

                // Extract image URLs
                if image_urls.is_none() {
                    if let Some(urls) =
                        data["result"]["response"]["modelResponse"]["generatedImageUrls"].as_array()
                    {
                        image_urls = Some(
                            urls.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect(),
                        );
                    } else if let Some(urls) =
                        data["result"]["modelResponse"]["generatedImageUrls"].as_array()
                    {
                        image_urls = Some(
                            urls.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect(),
                        );
                    }
                }
            }
        }

        let private_key_b64 =
            general_purpose::STANDARD.encode(self.keys.get("privateKey").unwrap());

        Ok(GrokResponse {
            response: full_response,
            stream_response,
            images: image_urls,
            extra_data: ExtraData {
                anon_user: self.anon_user.clone().unwrap(),
                cookies: self.cookies.clone(),
                actions: self.actions.clone(),
                xsid_script: self.xsid_script.clone(),
                baggage: self.baggage.clone(),
                sentry_trace: self.sentry_trace.clone(),
                conversation_id: conv_id,
                parent_response_id,
                private_key: private_key_b64,
            },
        })
    }
}
