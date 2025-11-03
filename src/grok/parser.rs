use crate::grok::error::{GrokError, Result};
use base64::{engine::general_purpose, Engine as _};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrokMapping {
    pub xsid_script: String,
    pub action_script: String,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XsidMapping {
    #[serde(flatten)]
    pub mappings: HashMap<String, Vec<usize>>,
}

pub struct Parser;

impl Parser {
    const GROK_JSON_PATH: &'static str = "core/grok.json";
    const MAPPING_JSON_PATH: &'static str = "core/mapping.json";

    /// Load XSID mapping from JSON file
    pub fn load_xsid_mapping() -> Result<HashMap<String, Vec<usize>>> {
        if Path::new(Self::MAPPING_JSON_PATH).exists() {
            let content = fs::read_to_string(Self::MAPPING_JSON_PATH)?;
            let mapping: HashMap<String, Vec<usize>> = serde_json::from_str(&content)?;
            Ok(mapping)
        } else {
            Ok(HashMap::new())
        }
    }

    /// Save XSID mapping to JSON file
    pub fn save_xsid_mapping(mapping: &HashMap<String, Vec<usize>>) -> Result<()> {
        let json = serde_json::to_string_pretty(mapping)?;
        fs::write(Self::MAPPING_JSON_PATH, json)?;
        Ok(())
    }

    /// Load Grok mapping from JSON file
    pub fn load_grok_mapping() -> Result<Vec<GrokMapping>> {
        if Path::new(Self::GROK_JSON_PATH).exists() {
            let content = fs::read_to_string(Self::GROK_JSON_PATH)?;
            let mapping: Vec<GrokMapping> = serde_json::from_str(&content)?;
            Ok(mapping)
        } else {
            Ok(Vec::new())
        }
    }

    /// Save Grok mapping to JSON file
    pub fn save_grok_mapping(mapping: &[GrokMapping]) -> Result<()> {
        let json = serde_json::to_string_pretty(mapping)?;
        fs::write(Self::GROK_JSON_PATH, json)?;
        Ok(())
    }

    /// Parse animation token from verification token
    pub fn get_anim(html: &str, verification: &str) -> Result<(String, String)> {
        let verification_token = crate::grok::utils::Utils::between(
            html,
            &format!("\"name\":\"{}\",\"content\":\"", verification),
            "\"",
        )?;

        let decoded = general_purpose::STANDARD
            .decode(&verification_token)
            .map_err(|e| GrokError::ParseError(format!("Base64 decode error: {}", e)))?;

        if decoded.len() <= 5 {
            return Err(GrokError::ParseError(
                "Invalid verification token".to_string(),
            ));
        }

        let anim = format!("loading-x-anim-{}", decoded[5] % 4);

        Ok((verification_token, anim))
    }

    /// Parse SVG values from HTML
    pub fn parse_values(
        html: &str,
        loading: &str,
        script_id: &str,
        client: &rquest::Client,
    ) -> Result<(String, Vec<usize>)> {
        // Extract all d values from SVG paths
        let re = Regex::new(r#""d":"(M[^"]{200,})""#).unwrap();
        let all_d_values: Vec<String> = re
            .captures_iter(html)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        // Get the specific SVG data based on animation index
        let anim_index: usize = loading
            .strip_prefix("loading-x-anim-")
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| GrokError::ParseError("Invalid loading animation".to_string()))?;

        if anim_index >= all_d_values.len() {
            return Err(GrokError::ParseError(
                "Animation index out of bounds".to_string(),
            ));
        }

        let svg_data = all_d_values[anim_index].clone();

        // Get the script URL and parse numbers
        if !script_id.is_empty() {
            let script_link = if script_id == "ondemand.s" {
                let script_hash =
                    crate::grok::utils::Utils::between(html, "\"ondemand.s\":\"", "\"")?;
                format!(
                    "https://abs.twimg.com/responsive-web/client-web/ondemand.s.{}a.js",
                    script_hash
                )
            } else {
                format!("https://grok.com/_next/{}", script_id)
            };

            // Check if we have cached mapping
            let mut mapping = Self::load_xsid_mapping()?;

            let numbers = if let Some(cached_numbers) = mapping.get(&script_link) {
                cached_numbers.clone()
            } else {
                // Fetch and parse the script
                let runtime = tokio::runtime::Handle::current();
                let script_content = runtime
                    .block_on(async { client.get(&script_link).send().await?.text().await })?;

                let re = Regex::new(r"x\[(\d+)\]\s*,\s*16").unwrap();
                let nums: Vec<usize> = re
                    .captures_iter(&script_content)
                    .filter_map(|cap| cap.get(1).and_then(|m| m.as_str().parse().ok()))
                    .collect();

                // Cache the result
                mapping.insert(script_link.clone(), nums.clone());
                Self::save_xsid_mapping(&mapping)?;

                nums
            };

            Ok((svg_data, numbers))
        } else {
            Ok((svg_data, vec![]))
        }
    }

    /// Parse Grok scripts to extract actions and XSID script
    pub async fn parse_grok(
        scripts: Vec<String>,
        client: &rquest::Client,
    ) -> Result<(Vec<String>, String)> {
        // Check cached mappings first
        let grok_mappings = Self::load_grok_mapping()?;

        for mapping in &grok_mappings {
            if scripts.contains(&mapping.action_script) {
                return Ok((mapping.actions.clone(), mapping.xsid_script.clone()));
            }
        }

        // Need to fetch and parse
        let mut script_content1 = String::new();
        let mut action_script = String::new();
        let mut script_content2 = String::new();

        for script in scripts {
            let url = format!("https://grok.com{}", script);
            let content = client.get(&url).send().await?.text().await?;

            if content.contains("anonPrivateKey") {
                script_content1 = content;
                action_script = script;
            } else if content.contains("880932)") {
                script_content2 = content;
            }
        }

        // Parse actions
        let re_actions = Regex::new(r#"createServerReference\)\("([a-f0-9]+)""#).unwrap();
        let actions: Vec<String> = re_actions
            .captures_iter(&script_content1)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        // Parse XSID script
        let re_xsid = Regex::new(r#""(static/chunks/[^"]+\.js)"[^}]*?a\(880932\)"#).unwrap();
        let xsid_script = re_xsid
            .captures(&script_content2)
            .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .ok_or_else(|| GrokError::ParseError("XSID script not found".to_string()))?;

        // Save to cache
        let new_mapping = GrokMapping {
            xsid_script: xsid_script.clone(),
            action_script,
            actions: actions.clone(),
        };

        let mut mappings = Self::load_grok_mapping()?;
        mappings.push(new_mapping);
        Self::save_grok_mapping(&mappings)?;

        Ok((actions, xsid_script))
    }
}
