use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrokResponse {
    pub response: Option<String>,
    pub stream_response: Vec<String>,
    pub images: Option<Vec<String>>,
    pub extra_data: ExtraData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraData {
    pub anon_user: String,
    pub cookies: HashMap<String, String>,
    pub actions: Vec<String>,
    pub xsid_script: String,
    pub baggage: String,
    pub sentry_trace: String,
    pub conversation_id: Option<String>,
    #[serde(rename = "parentResponseId")]
    pub parent_response_id: Option<String>,
    #[serde(rename = "privateKey")]
    pub private_key: String,
}

pub struct Models {
    models: HashMap<String, (String, String)>,
}

impl Models {
    pub fn new() -> Self {
        let mut models = HashMap::new();
        models.insert(
            "grok-3-auto".to_string(),
            ("MODEL_MODE_AUTO".to_string(), "auto".to_string()),
        );
        models.insert(
            "grok-3-fast".to_string(),
            ("MODEL_MODE_FAST".to_string(), "fast".to_string()),
        );
        models.insert(
            "grok-4".to_string(),
            ("MODEL_MODE_EXPERT".to_string(), "expert".to_string()),
        );
        models.insert(
            "grok-4-mini-thinking-tahoe".to_string(),
            (
                "MODEL_MODE_GROK_4_MINI_THINKING".to_string(),
                "grok-4-mini-thinking".to_string(),
            ),
        );

        Self { models }
    }

    pub fn get_model_mode(&self, model: &str) -> &String {
        self.models
            .get(model)
            .map(|(mode, _)| mode)
            .unwrap_or(&self.models["grok-3-auto"].0)
    }

    pub fn get_mode(&self, model: &str) -> &String {
        self.models
            .get(model)
            .map(|(_, mode)| mode)
            .unwrap_or(&self.models["grok-3-auto"].1)
    }
}

impl Default for Models {
    fn default() -> Self {
        Self::new()
    }
}
