use rquest::header::{HeaderMap, HeaderValue};

pub const BASE_URL: &str = "https://chat.qwen.ai";

pub const FAKE_HEADERS: &[(&str, &str)] = &[
    ("Accept", "*/*"),
    ("Accept-Encoding", "gzip, deflate, br"),
    ("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8"),
    ("Origin", "https://chat.qwen.ai"),
    ("Referer", "https://chat.qwen.ai/"),
    ("Sec-Ch-Ua", "\"Chromium\";v=\"142\", \"Google Chrome\";v=\"142\", \"Not_A Brand\";v=\"99\""),
    ("Sec-Ch-Ua-Mobile", "?0"),
    ("Sec-Ch-Ua-Platform", "\"macOS\""),
    ("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36"),
    ("source", "web"),
    ("version", "0.0.235"),
];

pub fn build_headers(with_auth: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();

    if let Some(token) = with_auth {
        headers.insert(
            "authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );
    }

    for (key, value) in FAKE_HEADERS {
        headers.insert(*key, HeaderValue::from_static(value));
    }

    headers
}

pub fn build_json_headers(with_auth: Option<&str>) -> HeaderMap {
    let mut headers = build_headers(with_auth);
    headers.insert("content-type", HeaderValue::from_static("application/json"));
    headers
}
