use crate::zto::error::{Result, ZtoError};
use futures_util::stream::StreamExt;
use rquest::Response;

pub async fn parse_stream_response(response: Response) -> Result<String> {
    let mut content = String::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| ZtoError::ParseError(e.to_string()))?;
        let text = String::from_utf8_lossy(&chunk);

        for line in text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if data == "[DONE]" {
                    break;
                }

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                        for choice in choices {
                            if let Some(delta) = choice.get("delta") {
                                if let Some(text_content) =
                                    delta.get("content").and_then(|c| c.as_str())
                                {
                                    content.push_str(text_content);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(content)
}

pub fn process_thinking_content(content: &str) -> String {
    let mut result = content.to_string();
    result = result.replace("<details>", "").replace("</details>", "");

    if result.starts_with("> ") {
        result = result[2..].to_string();
    }
    result = result.replace("\n> ", "\n");

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_thinking_content() {
        let input = "<details>这是思考内容</details>";
        let output = process_thinking_content(input);
        assert_eq!(output, "这是思考内容");
    }
}
