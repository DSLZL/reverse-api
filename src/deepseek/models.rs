use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

// Internal DeepSeek API request format
#[derive(Debug, Serialize)]
pub struct DeepSeekChatRequest {
    pub chat_session_id: String,
    pub parent_message_id: Option<String>,
    pub prompt: String,
    pub ref_file_ids: Vec<String>,
    pub search_enabled: bool,
    pub thinking_enabled: bool,
}

// Response data structure for continuous conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekResponse {
    pub response: Option<String>,
    pub extra_data: ExtraData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraData {
    pub session_id: String,
    pub message_id: String,
}
