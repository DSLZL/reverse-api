/// Utility functions similar to Python's Utils class
pub struct Utils;

impl Utils {
    /// Extract substring between two delimiters
    pub fn between(main_text: &str, start: &str, end: &str) -> Option<String> {
        let start_pos = main_text.find(start)?;
        let start_idx = start_pos + start.len();
        let remaining = &main_text[start_idx..];
        let end_pos = remaining.find(end)?;
        Some(remaining[..end_pos].to_string())
    }

    /// Generate random base36 string (similar to Python's _generate_react)
    pub fn generate_react_id() -> String {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let n: f64 = rng.gen();
        let chars = "0123456789abcdefghijklmnopqrstuvwxyz";
        let mut x = (n * (36_u64.pow(10) as f64)) as u64;
        let mut result = String::new();

        for _ in 0..10 {
            let remainder = (x % 36) as usize;
            x /= 36;
            result.insert(0, chars.chars().nth(remainder).unwrap());
        }

        result
    }

    /// Validate and format proxy URL
    pub fn format_proxy(proxy: &str) -> crate::chatgpt::utils::error::Result<String> {
        use url::Url;

        let proxy_url = if !proxy.starts_with("http://") && !proxy.starts_with("https://") {
            format!("http://{}", proxy)
        } else {
            proxy.to_string()
        };

        let parsed = Url::parse(&proxy_url).map_err(|_| {
            crate::chatgpt::utils::error::ChatGptError::invalid_proxy("Invalid proxy URL format")
        })?;

        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return Err(crate::chatgpt::utils::error::ChatGptError::invalid_proxy(
                "Only HTTP/HTTPS proxies are supported",
            ));
        }

        if parsed.host().is_none() || parsed.port().is_none() {
            return Err(crate::chatgpt::utils::error::ChatGptError::invalid_proxy(
                "Proxy must have host and port",
            ));
        }

        if !parsed.username().is_empty() && parsed.password().is_some() {
            Ok(format!(
                "{}://{}:{}@{}:{}",
                parsed.scheme(),
                parsed.username(),
                parsed.password().unwrap_or(""),
                parsed.host().unwrap(),
                parsed.port().unwrap()
            ))
        } else {
            Ok(format!(
                "{}://{}:{}",
                parsed.scheme(),
                parsed.host().unwrap(),
                parsed.port().unwrap()
            ))
        }
    }

    /// XOR two strings (utility for VM operations)
    pub fn xor_strings(data: &str, key: &str) -> String {
        if key.is_empty() {
            return data.to_string();
        }

        let data_bytes = data.as_bytes();
        let key_bytes = key.as_bytes();
        let mut result = Vec::with_capacity(data_bytes.len());

        for (i, &byte) in data_bytes.iter().enumerate() {
            let key_byte = key_bytes[i % key_bytes.len()];
            result.push(byte ^ key_byte);
        }

        String::from_utf8_lossy(&result).to_string()
    }

    /// Parse event stream data (for ChatGPT responses)
    pub fn parse_event_stream(stream_data: &str) -> String {
        let mut result = Vec::new();

        for line in stream_data.lines() {
            if let Some(stripped) = line.strip_prefix("data:") {
                let data_str = stripped.trim();

                if data_str == "[DONE]" {
                    break;
                }

                if let Ok(data) = serde_json::from_str::<serde_json::Value>(data_str) {
                    // Handle direct append operations
                    if let (Some("append"), Some("/message/content/parts/0"), Some(value)) = (
                        data.get("o").and_then(|v| v.as_str()),
                        data.get("p").and_then(|v| v.as_str()),
                        data.get("v").and_then(|v| v.as_str()),
                    ) {
                        result.push(value.to_string());
                    }
                    // Handle patch operations with list of operations
                    else if let (Some(op), Some(operations)) = (
                        data.get("o").and_then(|v| v.as_str()),
                        data.get("v").and_then(|v| v.as_array()),
                    ) {
                        if op == "patch" {
                            for operation in operations {
                                if let (
                                    Some("append"),
                                    Some("/message/content/parts/0"),
                                    Some(value),
                                ) = (
                                    operation.get("o").and_then(|v| v.as_str()),
                                    operation.get("p").and_then(|v| v.as_str()),
                                    operation.get("v").and_then(|v| v.as_str()),
                                ) {
                                    result.push(value.to_string());
                                }
                            }
                        }
                    }
                    // Handle 'v' field containing list of operations
                    else if let Some(operations) = data.get("v").and_then(|v| v.as_array()) {
                        for operation in operations {
                            if let (Some("append"), Some("/message/content/parts/0"), Some(value)) = (
                                operation.get("o").and_then(|v| v.as_str()),
                                operation.get("p").and_then(|v| v.as_str()),
                                operation.get("v").and_then(|v| v.as_str()),
                            ) {
                                result.push(value.to_string());
                            }
                        }
                    }
                }
            }
        }

        result.join("")
    }
}
