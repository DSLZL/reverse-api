use crate::zto::error::{Result, ZtoError};
use crate::zto::models::TokenResponse;
use rquest::header::HeaderMap;
use rquest::Client;

const ORIGIN_BASE: &str = "https://chat.z.ai";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36";
const X_FE_VERSION: &str = "prod-fe-1.0.94";

pub fn build_auth_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", USER_AGENT.parse().unwrap());
    headers.insert("Accept", "*/*".parse().unwrap());
    headers.insert("Accept-Language", "zh-CN,zh;q=0.9".parse().unwrap());
    headers.insert("X-FE-Version", X_FE_VERSION.parse().unwrap());
    headers.insert(
        "sec-ch-ua",
        "\"Chromium\";v=\"140\", \"Not=A?Brand\";v=\"24\", \"Google Chrome\";v=\"140\""
            .parse()
            .unwrap(),
    );
    headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
    headers.insert("sec-ch-ua-platform", "\"Windows\"".parse().unwrap());
    headers.insert("Origin", format!("{}/", ORIGIN_BASE).parse().unwrap());
    headers.insert("Referer", format!("{}/", ORIGIN_BASE).parse().unwrap());
    headers
}

pub fn build_request_headers(token: &str) -> HeaderMap {
    let mut headers = build_auth_headers();
    headers.insert(
        "Authorization",
        format!("Bearer {}", token).parse().unwrap(),
    );
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Accept", "*/*".parse().unwrap());
    headers.insert("Connection", "keep-alive".parse().unwrap());
    headers.insert("Sec-Fetch-Dest", "empty".parse().unwrap());
    headers.insert("Sec-Fetch-Mode", "cors".parse().unwrap());
    headers.insert("Sec-Fetch-Site", "same-origin".parse().unwrap());
    headers
}

pub async fn get_anon_token(client: &Client) -> Result<String> {
    let url = format!("{}/api/v1/auths/", ORIGIN_BASE);

    let response = client
        .get(&url)
        .headers(build_auth_headers())
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| ZtoError::TokenFetch(format!("Request failed: {}", e)))?;

    if response.status() != 200 {
        return Err(ZtoError::TokenFetch(format!(
            "Token request failed with status: {}",
            response.status()
        )));
    }

    let token_resp: TokenResponse = response
        .json()
        .await
        .map_err(|e| ZtoError::TokenFetch(format!("Parse error: {}", e)))?;

    if token_resp.token.is_empty() {
        return Err(ZtoError::TokenFetch("Token is empty".to_string()));
    }

    Ok(token_resp.token)
}
