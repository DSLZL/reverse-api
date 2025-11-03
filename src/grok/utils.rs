use crate::grok::error::{GrokError, Result};

pub struct Utils;

impl Utils {
    /// Extract text between two delimiters
    pub fn between(text: &str, start: &str, end: &str) -> Result<String> {
        let parts: Vec<&str> = text.split(start).collect();
        if parts.len() < 2 {
            return Err(GrokError::ParseError(format!(
                "Start delimiter '{}' not found",
                start
            )));
        }

        let remaining = parts[1];
        let end_parts: Vec<&str> = remaining.split(end).collect();
        if end_parts.is_empty() {
            return Err(GrokError::ParseError(format!(
                "End delimiter '{}' not found",
                end
            )));
        }

        Ok(end_parts[0].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_between() {
        let text = "hello [world] foo";
        let result = Utils::between(text, "[", "]").unwrap();
        assert_eq!(result, "world");
    }
}
