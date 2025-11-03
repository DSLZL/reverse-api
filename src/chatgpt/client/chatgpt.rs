use crate::chatgpt::crypto::Challenges;
use crate::chatgpt::network::{Headers, IpInfo};
use crate::chatgpt::utils::{ChatGptError, Result, Utils};
use crate::chatgpt::vm::VM;
use base64::{engine::general_purpose, Engine as _};
use chrono::prelude::*;
use image::ImageReader;
use rand::Rng;
use rquest::{Client, Proxy};
use serde_json::{json, Value};
use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Browser window keys for environment simulation
const WINDOW_KEYS: &[&str] = &[
    "window",
    "self",
    "document",
    "name",
    "location",
    "customElements",
    "history",
    "navigation",
    "locationbar",
    "menubar",
    "personalbar",
    "scrollbars",
    "statusbar",
    "toolbar",
    "status",
    "closed",
    "frames",
    "length",
    "top",
    "opener",
    "parent",
    "frameElement",
    "navigator",
    "origin",
    "external",
    "screen",
    "innerWidth",
    "innerHeight",
    "scrollX",
    "pageXOffset",
    "scrollY",
    "pageYOffset",
    "visualViewport",
    "screenX",
    "screenY",
    "outerWidth",
    "outerHeight",
    "devicePixelRatio",
    "event",
    "clientInformation",
    "screenLeft",
    "screenTop",
    "styleMedia",
    "onsearch",
    "trustedTypes",
    "performance",
    "onappinstalled",
    "onbeforeinstallprompt",
    "crypto",
    "indexedDB",
    "sessionStorage",
    "localStorage",
    "onbeforexrselect",
    "onabort",
    "onbeforeinput",
    "onbeforematch",
    "onbeforetoggle",
    "onblur",
    "oncancel",
    "oncanplay",
    "oncanplaythrough",
    "onchange",
    "onclick",
    "onclose",
    "oncontentvisibilityautostatechange",
    "oncontextlost",
    "oncontextmenu",
    "oncontextrestored",
    "oncuechange",
    "ondblclick",
    "ondrag",
    "ondragend",
    "ondragenter",
    "ondragleave",
    "ondragover",
    "ondragstart",
    "ondrop",
    "ondurationchange",
    "onemptied",
    "onended",
    "onerror",
    "onfocus",
    "onformdata",
    "oninput",
    "oninvalid",
    "onkeydown",
    "onkeypress",
    "onkeyup",
    "onload",
    "onloadeddata",
    "onloadedmetadata",
    "onloadstart",
    "onmousedown",
    "onmouseenter",
    "onmouseleave",
    "onmousemove",
    "onmouseout",
    "onmouseover",
    "onmouseup",
    "onmousewheel",
    "onpause",
    "onplay",
    "onplaying",
    "onprogress",
    "onratechange",
    "onreset",
    "onresize",
    "onscroll",
    "onsecuritypolicyviolation",
    "onseeked",
    "onseeking",
    "onselect",
    "onslotchange",
    "onstalled",
    "onsubmit",
    "onsuspend",
    "ontimeupdate",
    "ontoggle",
    "onvolumechange",
    "onwaiting",
    "onwebkitanimationend",
    "onwebkitanimationiteration",
    "onwebkitanimationstart",
    "onwebkittransitionend",
    "onwheel",
    "onauxclick",
    "ongotpointercapture",
    "onlostpointercapture",
    "onpointerdown",
    "onpointermove",
    "onpointerrawupdate",
    "onpointerup",
    "onpointercancel",
    "onpointerover",
    "onpointerout",
    "onpointerenter",
    "onpointerleave",
    "onselectstart",
    "onselectionchange",
    "onanimationend",
    "onanimationiteration",
    "onanimationstart",
    "ontransitionrun",
    "ontransitionstart",
    "ontransitionend",
    "ontransitioncancel",
    "onafterprint",
    "onbeforeprint",
    "onbeforeunload",
    "onhashchange",
    "onlanguagechange",
    "onmessage",
    "onmessageerror",
    "onoffline",
    "ononline",
    "onpagehide",
    "onpageshow",
    "onpopstate",
    "onrejectionhandled",
    "onstorage",
    "onunhandledrejection",
    "onunload",
    "isSecureContext",
    "crossOriginIsolated",
    "scheduler",
    "alert",
    "atob",
    "blur",
    "btoa",
    "cancelAnimationFrame",
    "cancelIdleCallback",
    "captureEvents",
    "clearInterval",
    "clearTimeout",
    "close",
    "confirm",
    "createImageBitmap",
    "fetch",
    "find",
    "focus",
    "getComputedStyle",
    "getSelection",
    "matchMedia",
    "moveBy",
    "moveTo",
    "open",
    "postMessage",
    "print",
    "prompt",
    "queueMicrotask",
    "releaseEvents",
    "reportError",
    "requestAnimationFrame",
    "requestIdleCallback",
    "resizeBy",
    "resizeTo",
    "scroll",
    "scrollBy",
    "scrollTo",
    "setInterval",
    "setTimeout",
    "stop",
    "structuredClone",
    "webkitCancelAnimationFrame",
    "webkitRequestAnimationFrame",
    "chrome",
    "caches",
    "cookieStore",
    "ondevicemotion",
    "ondeviceorientation",
    "ondeviceorientationabsolute",
    "sharedStorage",
    "documentPictureInPicture",
    "fetchLater",
    "getScreenDetails",
    "queryLocalFonts",
    "showDirectoryPicker",
    "showOpenFilePicker",
    "showSaveFilePicker",
    "originAgentCluster",
    "viewport",
    "onpageswap",
    "onpagereveal",
    "credentialless",
    "fence",
    "launchQueue",
    "speechSynthesis",
    "oncommand",
    "onscrollend",
    "onscrollsnapchange",
    "onscrollsnapchanging",
    "webkitRequestFileSystem",
    "webkitResolveLocalFileSystemURL",
    "define",
    "ethereum",
    "__oai_SSR_HTML",
    "__reactRouterContext",
    "$RC",
    "__oai_SSR_TTI",
    "__reactRouterManifest",
    "__reactRouterVersion",
    "DD_RUM",
    "__REACT_INTL_CONTEXT__",
    "regeneratorRuntime",
    "DD_LOGS",
    "__STATSIG__",
    "__mobxInstanceCount",
    "__mobxGlobals",
    "_g",
    "__reactRouterRouteModules",
    "__SEGMENT_INSPECTOR__",
    "__reactRouterDataRouter",
    "MotionIsMounted",
    "_oaiHandleSessionExpired",
];

/// ChatGPT configuration data
#[derive(Debug, Clone)]
pub struct ChatGptData {
    pub prod: String,
    pub device_id: String,
    pub token: String,
    pub conversation_id: Option<String>,
    pub parent_message_id: Option<String>,
    pub proofofwork: Option<Value>,
    pub bytecode: Option<String>,
    pub vm_token: Option<String>,
    pub file_id: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<usize>,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub config: Value,
}

impl Default for ChatGptData {
    fn default() -> Self {
        Self {
            prod: String::new(),
            device_id: String::new(),
            token: String::new(),
            conversation_id: None,
            parent_message_id: None,
            proofofwork: None,
            bytecode: None,
            vm_token: None,
            file_id: None,
            file_name: None,
            file_size: None,
            image_width: None,
            image_height: None,
            config: json!([]),
        }
    }
}

/// Main ChatGPT client
pub struct ChatGptClient {
    client: Client,
    data: ChatGptData,
    ip_info: IpInfo,
    timezone_offset: i32,
    start_time: u64,
    sid: String,
    window_keys: Vec<String>,
    reacts: Vec<String>,
}

impl ChatGptClient {
    /// Create new ChatGPT client
    pub async fn new(proxy: Option<&str>) -> Result<Self> {
        let mut client_builder = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")
            .cookie_store(true);

        if let Some(proxy_url) = proxy {
            let formatted_proxy = Utils::format_proxy(proxy_url)?;
            let proxy = Proxy::all(&formatted_proxy).map_err(|e| {
                ChatGptError::invalid_proxy(format!("Failed to create proxy: {}", e))
            })?;
            client_builder = client_builder.proxy(proxy);
        }

        let client = client_builder.build()?;
        let ip_info = IpInfo::fetch(&client).await?;

        // Calculate timezone offset
        let timezone_offset = match ip_info.timezone.parse::<chrono_tz::Tz>() {
            Ok(tz) => {
                let now = Utc::now().with_timezone(&tz);
                now.offset().fix().local_minus_utc() / 60
            }
            Err(_) => 0, // Default to UTC if parsing fails
        };

        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let sid = Uuid::new_v4().to_string();

        // Generate dynamic reacts and window_keys like Python version
        let reacts = vec![
            "location".to_string(),
            format!("__reactContainer${}", Utils::generate_react_id()),
            format!("_reactListening{}", Utils::generate_react_id()),
        ];

        let window_keys: Vec<String> = WINDOW_KEYS.iter().map(|s| s.to_string()).collect();

        let mut instance = Self {
            client,
            data: ChatGptData::default(),
            ip_info,
            timezone_offset,
            start_time,
            sid,
            window_keys,
            reacts,
        };

        instance.fetch_cookies().await?;
        Ok(instance)
    }

    /// Fetch initial cookies and configuration
    async fn fetch_cookies(&mut self) -> Result<()> {
        let response = self.client.get("https://chatgpt.com").send().await?;

        let html = response.text().await?;

        // Extract build version
        if let Some(prod) = Utils::between(&html, r#"data-build=""#, r#"""#) {
            self.data.prod = prod;
        }

        // Extract device ID from cookies
        if let Some(device_id) = self.get_cookie_value("oai-did") {
            self.data.device_id = device_id;
        }

        self.initialize_config()?;
        Ok(())
    }

    /// Initialize configuration array
    fn initialize_config(&mut self) -> Result<()> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Calculate timezone name
        let timezone_name = match self.ip_info.timezone.parse::<chrono_tz::Tz>() {
            Ok(tz) => {
                let now = Utc::now().with_timezone(&tz);
                format!(
                    "GMT{:+05} ({})",
                    now.offset().fix().local_minus_utc() / 3600,
                    tz.name()
                )
            }
            Err(_) => "GMT+0000 (UTC)".to_string(),
        };

        // Generate formatted timestamp
        let now = Utc::now();
        let formatted_time = format!(
            "{}",
            now.format(&format!("%a %b %d %Y %H:%M:%S {}", timezone_name))
        );

        // Random selection from reacts and window_keys
        let selected_react = if !self.reacts.is_empty() {
            self.reacts[rng.gen_range(0..self.reacts.len())].clone()
        } else {
            String::new()
        };
        let selected_window_key = if !self.window_keys.is_empty() {
            self.window_keys[rng.gen_range(0..self.window_keys.len())].clone()
        } else {
            String::new()
        };

        self.data.config = json!([
            4880,
            formatted_time,
            4294705152_u64,
            rng.gen::<f64>(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36",
            Value::Null,
            &self.data.prod,
            "de-DE",
            "de-DE,de,en-US,en",
            rng.gen::<f64>(),
            "webkitGetUserMedia−function webkitGetUserMedia() { [native code] }",
            selected_react,
            selected_window_key,
            rng.gen_range(800.0..1400.0) + rng.gen::<f64>(),
            &self.sid,
            "",
            20,
            self.start_time
        ]);

        Ok(())
    }

    /// Get cookie value by name
    fn get_cookie_value(&self, name: &str) -> Option<String> {
        // This is simplified - in practice you'd extract from the cookie store
        if name == "oai-did" {
            Some(Uuid::new_v4().to_string())
        } else {
            None
        }
    }

    /// Get chat requirements tokens
    async fn get_tokens(&mut self) -> Result<()> {
        let mut headers = Headers::requirements();
        headers.insert("oai-client-version".to_string(), self.data.prod.clone());
        headers.insert("oai-device-id".to_string(), self.data.device_id.clone());

        // Update config with new timing
        let process_time = {
            let mut rng = rand::thread_rng();
            rng.gen_range(1400..2000)
        };
        self.update_config_timing(process_time)?;

        let p_value = Challenges::generate_token(self.data.config.clone())?;
        self.data.vm_token = Some(p_value.clone());

        let payload = json!({
            "p": p_value
        });

        let mut request = self
            .client
            .post("https://chatgpt.com/backend-anon/sentinel/chat-requirements")
            .json(&payload);

        // Apply headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let json: Value = response.json().await?;

            if let Some(token) = json.get("token").and_then(|v| v.as_str()) {
                self.data.token = token.to_string();
            }

            self.data.proofofwork = json.get("proofofwork").cloned();

            if let Some(turnstile) = json.get("turnstile") {
                if let Some(dx) = turnstile.get("dx").and_then(|v| v.as_str()) {
                    self.data.bytecode = Some(dx.to_string());
                }
            }
        } else {
            return Err(ChatGptError::authentication(
                "Failed to get chat requirements",
            ));
        }

        Ok(())
    }

    /// Update config timing values
    fn update_config_timing(&mut self, process_time: i32) -> Result<()> {
        if let Some(config_array) = self.data.config.as_array_mut() {
            let mut rng = rand::thread_rng();

            // Update timestamp
            config_array[1] = json!(format!(
                "{}",
                Utc::now().format("%a %b %d %Y %H:%M:%S GMT%z (UTC)")
            ));

            // Update random values
            config_array[3] = json!(rng.gen::<f64>());
            config_array[9] = json!(rng.gen::<f64>());

            // Update timing
            config_array[13] = json!(process_time as f64 + rng.gen::<f64>());
        }

        Ok(())
    }

    /// Get conduit token
    async fn get_conduit(&self, next: bool) -> Result<String> {
        let mut headers = Headers::conduit();
        headers.insert("oai-client-version".to_string(), self.data.prod.clone());
        headers.insert("oai-device-id".to_string(), self.data.device_id.clone());

        let payload = if next {
            json!({
                "action": "next",
                "fork_from_shared_post": false,
                "conversation_id": self.data.conversation_id,
                "parent_message_id": self.data.parent_message_id,
                "model": "auto",
                "timezone_offset_min": self.timezone_offset,
                "timezone": self.ip_info.timezone,
                "history_and_training_disabled": true,
                "conversation_mode": {
                    "kind": "primary_assistant"
                },
                "system_hints": [],
                "supports_buffering": true,
                "supported_encodings": ["v1"]
            })
        } else {
            json!({
                "action": "next",
                "fork_from_shared_post": false,
                "parent_message_id": "client-created-root",
                "model": "auto",
                "timezone_offset_min": self.timezone_offset,
                "timezone": self.ip_info.timezone,
                "history_and_training_disabled": true,
                "conversation_mode": {
                    "kind": "primary_assistant"
                },
                "system_hints": [],
                "supports_buffering": true,
                "supported_encodings": ["v1"]
            })
        };

        let mut request = self
            .client
            .post("https://chatgpt.com/backend-anon/f/conversation/prepare")
            .json(&payload);

        // Apply headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;

        let json: Value = response.json().await?;

        if let Some(conduit_token) = json.get("conduit_token").and_then(|v| v.as_str()) {
            Ok(conduit_token.to_string())
        } else {
            Err(ChatGptError::authentication("Failed to get conduit token"))
        }
    }

    /// Start a conversation
    pub async fn start_conversation(&mut self, message: &str) -> Result<String> {
        self.get_tokens().await?;
        let conduit_token = self.get_conduit(false).await?;

        let (echo_logs, time_since_loaded) = {
            let mut rng = rand::thread_rng();
            let time_1 = rng.gen_range(6000..9000);
            let echo_logs = format!("0,{},1,{}", time_1, time_1 + rng.gen_range(1000..1200));
            let time_since_loaded = rng.gen_range(3..6);
            (echo_logs, time_since_loaded)
        };

        // Solve proof of work
        let proof_token = if let Some(pow) = self.data.proofofwork.as_ref() {
            let seed = pow.get("seed").and_then(|v| v.as_str()).unwrap_or("");
            let difficulty = pow.get("difficulty").and_then(|v| v.as_str()).unwrap_or("");
            Challenges::solve_pow(seed, difficulty, self.data.config.clone())?
        } else {
            return Err(ChatGptError::authentication(
                "No proof of work challenge received",
            ));
        };

        // Get turnstile token
        let turnstile_token = if let (Some(bytecode), Some(vm_token)) =
            (self.data.bytecode.as_deref(), self.data.vm_token.as_deref())
        {
            VM::get_turnstile(bytecode, vm_token, &self.ip_info.without_timezone())?
        } else {
            return Err(ChatGptError::authentication("Missing bytecode or VM token"));
        };

        // Prepare conversation headers
        let mut headers = Headers::conversation();
        headers.insert("oai-client-version".to_string(), self.data.prod.clone());
        headers.insert("oai-device-id".to_string(), self.data.device_id.clone());
        headers.insert("oai-echo-logs".to_string(), echo_logs.clone());
        headers.insert(
            "openai-sentinel-chat-requirements-token".to_string(),
            self.data.token.clone(),
        );
        headers.insert("openai-sentinel-proof-token".to_string(), proof_token);
        headers.insert(
            "openai-sentinel-turnstile-token".to_string(),
            turnstile_token,
        );
        headers.insert("x-conduit-token".to_string(), conduit_token);

        let conversation_payload = json!({
            "action": "next",
            "messages": [{
                "id": Uuid::new_v4().to_string(),
                "author": {
                    "role": "user"
                },
                "create_time": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64(),
                "content": {
                    "content_type": "text",
                    "parts": [message]
                },
                "metadata": {
                    "selected_github_repos": [],
                    "selected_all_github_repos": false,
                    "serialization_metadata": {
                        "custom_symbol_offsets": []
                    }
                }
            }],
            "parent_message_id": "client-created-root",
            "model": "auto",
            "timezone_offset_min": self.timezone_offset,
            "timezone": self.ip_info.timezone,
            "history_and_training_disabled": true,
            "conversation_mode": {
                "kind": "primary_assistant"
            },
            "enable_message_followups": true,
            "system_hints": [],
            "supports_buffering": true,
            "supported_encodings": ["v1"],
            "client_contextual_info": {
                "is_dark_mode": true,
                "time_since_loaded": time_since_loaded,
                "page_height": 1219,
                "page_width": 3440,
                "pixel_ratio": 1,
                "screen_height": 1440,
                "screen_width": 3440
            },
            "paragen_cot_summary_display_override": "allow",
            "force_parallel_switch": "auto"
        });

        let mut request = self
            .client
            .post("https://chatgpt.com/backend-anon/f/conversation")
            .json(&conversation_payload);

        // Apply headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;

        let response_text = response.text().await?;

        if response_text.contains("Unusual activity") {
            return Err(ChatGptError::IpFlagged);
        }

        // Extract conversation data
        if let Some(conversation_id) =
            Utils::between(&response_text, r#""conversation_id": ""#, r#"""#)
        {
            self.data.conversation_id = Some(conversation_id);
        }

        if let Some(parent_message_id) =
            Utils::between(&response_text, r#""message_id": ""#, r#"""#)
        {
            self.data.parent_message_id = Some(parent_message_id);
        }

        let parsed_response = Utils::parse_event_stream(&response_text);
        Ok(parsed_response)
    }

    /// Upload an image for multimodal conversation
    pub async fn upload_image(&mut self, image_data: &str) -> Result<()> {
        let mut headers = Headers::requirements();
        headers.insert("oai-client-version".to_string(), self.data.prod.clone());
        headers.insert("oai-device-id".to_string(), self.data.device_id.clone());

        // Generate file name
        let file_name = Uuid::new_v4().to_string();
        self.data.file_name = Some(format!("{}.png", file_name));

        // Decode base64 image data
        let image_bytes = if image_data.starts_with("data:image") {
            let base64_part = image_data.split(',').nth(1).unwrap_or(image_data);
            general_purpose::STANDARD.decode(base64_part)?
        } else {
            general_purpose::STANDARD.decode(image_data)?
        };

        self.data.file_size = Some(image_bytes.len());

        // Get image dimensions
        let img = ImageReader::new(Cursor::new(&image_bytes))
            .with_guessed_format()?
            .decode()?;
        self.data.image_width = Some(img.width());
        self.data.image_height = Some(img.height());

        // Request file upload
        let image_payload = json!({
            "file_name": self.data.file_name.as_ref().unwrap(),
            "file_size": self.data.file_size.unwrap(),
            "use_case": "multimodal",
            "timezone_offset_min": self.timezone_offset,
            "reset_rate_limits": false
        });

        let mut request = self
            .client
            .post("https://chatgpt.com/backend-anon/files")
            .json(&image_payload);

        // Apply headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;

        let upload_response: Value = response.json().await?;

        if let Some(file_id) = upload_response.get("file_id").and_then(|v| v.as_str()) {
            self.data.file_id = Some(file_id.to_string());
        }

        if let Some(upload_url) = upload_response.get("upload_url").and_then(|v| v.as_str()) {
            // Upload the file
            let upload_response = self.client.put(upload_url).body(image_bytes).send().await?;

            if !upload_response.status().is_success() {
                return Err(ChatGptError::authentication("Failed to upload image"));
            }

            // Process the uploaded file
            let mut process_headers = Headers::requirements();
            process_headers.insert("oai-client-version".to_string(), self.data.prod.clone());
            process_headers.insert("oai-device-id".to_string(), self.data.device_id.clone());

            let process_payload = json!({
                "file_id": self.data.file_id.as_ref().unwrap(),
                "use_case": "multimodal",
                "index_for_retrieval": false,
                "file_name": self.data.file_name.as_ref().unwrap()
            });

            let mut process_request = self
                .client
                .post("https://chatgpt.com/backend-anon/files/process_upload_stream")
                .json(&process_payload);

            // Apply headers
            for (key, value) in process_headers {
                process_request = process_request.header(key, value);
            }

            let process_response = process_request.send().await?;

            let process_text = process_response.text().await?;
            if !process_text.contains("Succeeded processing") {
                return Err(ChatGptError::authentication(
                    "Failed to process uploaded image",
                ));
            }
        } else {
            return Err(ChatGptError::authentication("Failed to get upload URL"));
        }

        Ok(())
    }

    /// Start a conversation with an image
    pub async fn start_with_image(&mut self, message: &str, image_data: &str) -> Result<String> {
        self.get_tokens().await?;
        let conduit_token = self.get_conduit(false).await?;
        self.upload_image(image_data).await?;

        let (echo_logs, time_since_loaded) = {
            let mut rng = rand::thread_rng();
            let time_1 = rng.gen_range(6000..9000);
            let echo_logs = format!("0,{},1,{}", time_1, time_1 + rng.gen_range(1000..1200));
            let time_since_loaded = rng.gen_range(3..6);
            (echo_logs, time_since_loaded)
        };

        // Solve proof of work
        let proof_token = if let Some(pow) = self.data.proofofwork.as_ref() {
            let seed = pow.get("seed").and_then(|v| v.as_str()).unwrap_or("");
            let difficulty = pow.get("difficulty").and_then(|v| v.as_str()).unwrap_or("");
            Challenges::solve_pow(seed, difficulty, self.data.config.clone())?
        } else {
            return Err(ChatGptError::authentication(
                "No proof of work challenge received",
            ));
        };

        // Get turnstile token
        let turnstile_token = if let (Some(bytecode), Some(vm_token)) =
            (self.data.bytecode.as_deref(), self.data.vm_token.as_deref())
        {
            VM::get_turnstile(bytecode, vm_token, &self.ip_info.without_timezone())?
        } else {
            return Err(ChatGptError::authentication("Missing bytecode or VM token"));
        };

        // Prepare conversation headers
        let mut headers = Headers::conversation();
        headers.insert("oai-client-version".to_string(), self.data.prod.clone());
        headers.insert("oai-device-id".to_string(), self.data.device_id.clone());
        headers.insert("oai-echo-logs".to_string(), echo_logs.clone());
        headers.insert(
            "openai-sentinel-chat-requirements-token".to_string(),
            self.data.token.clone(),
        );
        headers.insert("openai-sentinel-proof-token".to_string(), proof_token);
        headers.insert(
            "openai-sentinel-turnstile-token".to_string(),
            turnstile_token,
        );
        headers.insert("x-conduit-token".to_string(), conduit_token);

        let conversation_payload = json!({
            "action": "next",
            "messages": [{
                "id": Uuid::new_v4().to_string(),
                "author": {
                    "role": "user"
                },
                "create_time": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64(),
                "content": {
                    "content_type": "multimodal_text",
                    "parts": [
                        {
                            "content_type": "image_asset_pointer",
                            "asset_pointer": format!("file-service://{}", self.data.file_id.as_ref().unwrap()),
                            "size_bytes": self.data.file_size.unwrap(),
                            "width": self.data.image_width.unwrap(),
                            "height": self.data.image_height.unwrap()
                        },
                        message
                    ]
                },
                "metadata": {
                    "attachments": [{
                        "id": self.data.file_id.as_ref().unwrap(),
                        "size": self.data.file_size.unwrap(),
                        "name": self.data.file_name.as_ref().unwrap(),
                        "mime_type": "image/png",
                        "width": self.data.image_width.unwrap(),
                        "height": self.data.image_height.unwrap(),
                        "source": "local"
                    }],
                    "selected_github_repos": [],
                    "selected_all_github_repos": false,
                    "serialization_metadata": {
                        "custom_symbol_offsets": []
                    }
                }
            }],
            "parent_message_id": "client-created-root",
            "model": "auto",
            "timezone_offset_min": self.timezone_offset,
            "timezone": self.ip_info.timezone,
            "history_and_training_disabled": true,
            "conversation_mode": {
                "kind": "primary_assistant"
            },
            "enable_message_followups": true,
            "system_hints": [],
            "supports_buffering": true,
            "supported_encodings": ["v1"],
            "client_contextual_info": {
                "is_dark_mode": true,
                "time_since_loaded": time_since_loaded,
                "page_height": 1219,
                "page_width": 3440,
                "pixel_ratio": 1,
                "screen_height": 1440,
                "screen_width": 3440
            },
            "paragen_cot_summary_display_override": "allow",
            "force_parallel_switch": "auto"
        });

        let mut request = self
            .client
            .post("https://chatgpt.com/backend-anon/f/conversation")
            .json(&conversation_payload);

        // Apply headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;

        let response_text = response.text().await?;

        if response_text.contains("Unusual activity") {
            return Err(ChatGptError::IpFlagged);
        }

        // Extract conversation data
        if let Some(conversation_id) =
            Utils::between(&response_text, r#""conversation_id": ""#, r#"""#)
        {
            self.data.conversation_id = Some(conversation_id);
        }

        if let Some(parent_message_id) =
            Utils::between(&response_text, r#""message_id": ""#, r#"""#)
        {
            self.data.parent_message_id = Some(parent_message_id);
        }

        let parsed_response = Utils::parse_event_stream(&response_text);
        Ok(parsed_response)
    }

    /// Send a question and get response
    pub async fn ask_question(&mut self, message: &str) -> Result<String> {
        self.start_conversation(message).await
    }

    /// Send a question with an image and get response
    pub async fn ask_question_with_image(
        &mut self,
        message: &str,
        image_data: &str,
    ) -> Result<String> {
        self.start_with_image(message, image_data).await
    }

    /// Hold a conversation with ability to continue chatting
    pub async fn hold_conversation(&mut self, message: &str, new: bool) -> Result<String> {
        let mut index = 2000;

        if new {
            self.start_conversation(message).await?;
        }

        // Get conduit token
        let conduit_token = self.get_conduit(true).await?;

        // Get tokens
        self.get_tokens().await?;
        index += 3000;

        let echo_logs = {
            let mut rng = rand::thread_rng();
            let time_1 = rng.gen_range(index..(index + 3000));
            format!("0,{},1,{}", time_1, time_1 + rng.gen_range(1000..1200))
        };

        // Solve proof of work
        let proof_token = if let Some(proofofwork) = &self.data.proofofwork {
            if let (Some(seed), Some(difficulty)) = (
                proofofwork.get("seed").and_then(|v| v.as_str()),
                proofofwork.get("difficulty").and_then(|v| v.as_str()),
            ) {
                Challenges::solve_pow(seed, difficulty, self.data.config.clone())?
            } else {
                return Err(ChatGptError::invalid_response(
                    "Missing proof of work data".to_string(),
                ));
            }
        } else {
            return Err(ChatGptError::invalid_response(
                "Missing proof of work configuration".to_string(),
            ));
        };

        // Get turnstile token
        let turnstile_token = if let (Some(bytecode), Some(vm_token)) =
            (self.data.bytecode.as_deref(), self.data.vm_token.as_deref())
        {
            VM::get_turnstile(bytecode, vm_token, &self.ip_info.ip)?
        } else {
            return Err(ChatGptError::invalid_response(
                "Missing VM data".to_string(),
            ));
        };

        // Prepare headers
        let mut headers = Headers::conversation();
        headers.insert("oai-client-version".to_string(), self.data.prod.clone());
        headers.insert("oai-device-id".to_string(), self.data.device_id.clone());
        headers.insert("oai-echo-logs".to_string(), echo_logs);
        headers.insert(
            "openai-sentinel-chat-requirements-token".to_string(),
            self.data.token.clone(),
        );
        headers.insert("openai-sentinel-proof-token".to_string(), proof_token);
        headers.insert(
            "openai-sentinel-turnstile-token".to_string(),
            turnstile_token,
        );
        headers.insert("x-conduit-token".to_string(), conduit_token);

        let new_message = if new {
            // In interactive mode, would prompt for input
            // For now, use the provided message
            message.to_string()
        } else {
            message.to_string()
        };

        let conversation_data = json!({
            "action": "next",
            "messages": [{
                "id": Uuid::new_v4().to_string(),
                "author": {
                    "role": "user"
                },
                "create_time": (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64()),
                "content": {
                    "content_type": "text",
                    "parts": [new_message]
                },
                "metadata": {
                    "selected_github_repos": [],
                    "selected_all_github_repos": false,
                    "serialization_metadata": {
                        "custom_symbol_offsets": []
                    }
                }
            }],
            "conversation_id": self.data.conversation_id,
            "parent_message_id": self.data.parent_message_id,
            "model": "auto",
            "timezone_offset_min": self.timezone_offset,
            "timezone": self.ip_info.timezone,
            "history_and_training_disabled": true,
            "conversation_mode": {
                "kind": "primary_assistant"
            },
            "enable_message_followups": true,
            "system_hints": [],
            "supports_buffering": true,
            "supported_encodings": ["v1"],
            "client_contextual_info": {
                "is_dark_mode": true,
                "time_since_loaded": 17,
                "page_height": 1219,
                "page_width": 3440,
                "pixel_ratio": 1,
                "screen_height": 1440,
                "screen_width": 3440
            },
            "paragen_cot_summary_display_override": "allow",
            "force_parallel_switch": "auto"
        });

        let mut request = self
            .client
            .post("https://chatgpt.com/backend-anon/f/conversation")
            .json(&conversation_data);

        // Apply headers
        for (key, value) in headers {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        let response_text = response.text().await?;

        if response_text.contains("Unusual activity") {
            return Err(ChatGptError::invalid_response(
                "Your IP got flagged by chatgpt, retry with a new IP".to_string(),
            ));
        }

        // Update conversation state
        if let Some(conversation_id) =
            Utils::between(&response_text, "\"conversation_id\": \"", "\"")
        {
            self.data.conversation_id = Some(conversation_id);
        }
        if let Some(parent_message_id) = Utils::between(&response_text, "\"message_id\": \"", "\"")
        {
            self.data.parent_message_id = Some(parent_message_id);
        }

        Ok(Utils::parse_event_stream(&response_text))
    }
}
