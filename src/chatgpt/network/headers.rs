use std::collections::HashMap;

/// HTTP headers for different types of requests
pub struct Headers;

impl Headers {
    /// Default headers for initial page load
    pub fn default_headers() -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string());
        headers.insert(
            "accept-language".to_string(),
            "de-DE,de;q=0.9,en-US;q=0.8,en;q=0.7".to_string(),
        );
        headers.insert("cache-control".to_string(), "no-cache".to_string());
        headers.insert("pragma".to_string(), "no-cache".to_string());
        headers.insert("priority".to_string(), "u=0, i".to_string());
        headers.insert(
            "sec-ch-ua".to_string(),
            "\"Chromium\";v=\"140\", \"Not=A?Brand\";v=\"24\", \"Google Chrome\";v=\"140\""
                .to_string(),
        );
        headers.insert("sec-ch-ua-mobile".to_string(), "?0".to_string());
        headers.insert("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string());
        headers.insert("sec-fetch-dest".to_string(), "document".to_string());
        headers.insert("sec-fetch-mode".to_string(), "navigate".to_string());
        headers.insert("sec-fetch-site".to_string(), "same-origin".to_string());
        headers.insert("sec-fetch-user".to_string(), "?1".to_string());
        headers.insert("upgrade-insecure-requests".to_string(), "1".to_string());
        headers.insert("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36".to_string());
        headers
    }

    /// Headers for requirements requests
    pub fn requirements() -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("accept".to_string(), "*/*".to_string());
        headers.insert(
            "accept-language".to_string(),
            "de-DE,de;q=0.9,en-US;q=0.8,en;q=0.7".to_string(),
        );
        headers.insert("cache-control".to_string(), "no-cache".to_string());
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("oai-language".to_string(), "de-DE".to_string());
        headers.insert("origin".to_string(), "https://chatgpt.com".to_string());
        headers.insert("pragma".to_string(), "no-cache".to_string());
        headers.insert("priority".to_string(), "u=1, i".to_string());
        headers.insert("referer".to_string(), "https://chatgpt.com/".to_string());
        headers.insert(
            "sec-ch-ua".to_string(),
            "\"Chromium\";v=\"140\", \"Not=A?Brand\";v=\"24\", \"Google Chrome\";v=\"140\""
                .to_string(),
        );
        headers.insert("sec-ch-ua-mobile".to_string(), "?0".to_string());
        headers.insert("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string());
        headers.insert("sec-fetch-dest".to_string(), "empty".to_string());
        headers.insert("sec-fetch-mode".to_string(), "cors".to_string());
        headers.insert("sec-fetch-site".to_string(), "same-origin".to_string());
        headers.insert("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36".to_string());
        headers
    }

    /// Headers for conduit requests
    pub fn conduit() -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("accept".to_string(), "*/*".to_string());
        headers.insert("accept-language".to_string(), "de-DE,de;q=0.9".to_string());
        headers.insert("cache-control".to_string(), "no-cache".to_string());
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("oai-language".to_string(), "de-DE".to_string());
        headers.insert("origin".to_string(), "https://chatgpt.com".to_string());
        headers.insert("pragma".to_string(), "no-cache".to_string());
        headers.insert("priority".to_string(), "u=1, i".to_string());
        headers.insert("referer".to_string(), "https://chatgpt.com/".to_string());
        headers.insert(
            "sec-ch-ua".to_string(),
            "\"Chromium\";v=\"140\", \"Not=A?Brand\";v=\"24\", \"Google Chrome\";v=\"140\""
                .to_string(),
        );
        headers.insert("sec-ch-ua-mobile".to_string(), "?0".to_string());
        headers.insert("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string());
        headers.insert("sec-fetch-dest".to_string(), "empty".to_string());
        headers.insert("sec-fetch-mode".to_string(), "cors".to_string());
        headers.insert("sec-fetch-site".to_string(), "same-origin".to_string());
        headers.insert("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36".to_string());
        headers.insert("x-conduit-token".to_string(), "no-token".to_string());
        headers
    }

    /// Headers for conversation requests
    pub fn conversation() -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("accept".to_string(), "text/event-stream".to_string());
        headers.insert(
            "accept-language".to_string(),
            "de-DE,de;q=0.9,en-US;q=0.8,en;q=0.7".to_string(),
        );
        headers.insert("cache-control".to_string(), "no-cache".to_string());
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("oai-language".to_string(), "de-DE".to_string());
        headers.insert("origin".to_string(), "https://chatgpt.com".to_string());
        headers.insert("pragma".to_string(), "no-cache".to_string());
        headers.insert("priority".to_string(), "u=1, i".to_string());
        headers.insert("referer".to_string(), "https://chatgpt.com/".to_string());
        headers.insert(
            "sec-ch-ua".to_string(),
            "\"Chromium\";v=\"140\", \"Not=A?Brand\";v=\"24\", \"Google Chrome\";v=\"140\""
                .to_string(),
        );
        headers.insert("sec-ch-ua-mobile".to_string(), "?0".to_string());
        headers.insert("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string());
        headers.insert("sec-fetch-dest".to_string(), "empty".to_string());
        headers.insert("sec-fetch-mode".to_string(), "cors".to_string());
        headers.insert("sec-fetch-site".to_string(), "same-origin".to_string());
        headers.insert("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36".to_string());
        headers
    }

    /// Headers for file upload requests
    pub fn file() -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert(
            "accept".to_string(),
            "application/json, text/plain, */*".to_string(),
        );
        headers.insert(
            "accept-language".to_string(),
            "de-DE,de;q=0.9,en-US;q=0.8,en;q=0.7".to_string(),
        );
        headers.insert("cache-control".to_string(), "no-cache".to_string());
        headers.insert("content-type".to_string(), "image/png".to_string());
        headers.insert("origin".to_string(), "https://chatgpt.com".to_string());
        headers.insert("pragma".to_string(), "no-cache".to_string());
        headers.insert("priority".to_string(), "u=1, i".to_string());
        headers.insert("referer".to_string(), "https://chatgpt.com/".to_string());
        headers.insert(
            "sec-ch-ua".to_string(),
            "\"Chromium\";v=\"140\", \"Not=A?Brand\";v=\"24\", \"Google Chrome\";v=\"140\""
                .to_string(),
        );
        headers.insert("sec-ch-ua-mobile".to_string(), "?0".to_string());
        headers.insert("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string());
        headers.insert("sec-fetch-dest".to_string(), "empty".to_string());
        headers.insert("sec-fetch-mode".to_string(), "cors".to_string());
        headers.insert("sec-fetch-site".to_string(), "same-origin".to_string());
        headers.insert("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36".to_string());
        headers
    }
}
