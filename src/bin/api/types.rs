use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ThreadPath {
    pub thread_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateThreadRequest {
    #[serde(default)]
    pub messages: Vec<ThreadMessage>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub proxy: Option<String>,
    #[serde(default = "default_model")]
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct AddMessageRequest {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub files: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ThreadMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateResponseRequest {
    pub thread_id: String,
    #[serde(default = "default_model")]
    #[allow(dead_code)]
    pub model: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub instructions: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub stream: bool,
    #[serde(default)]
    pub proxy: Option<String>,
    #[serde(default)]
    pub file_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateImageRequest {
    pub prompt: String,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default = "default_qwen_model")]
    pub model: String,
    #[serde(default)]
    pub thread_id: Option<String>,
    #[serde(default)]
    pub download: bool,
}

#[derive(Debug, Deserialize)]
pub struct GenerateVideoRequest {
    pub prompt: String,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default = "default_qwen_model")]
    pub model: String,
    #[serde(default)]
    pub thread_id: Option<String>,
    #[serde(default)]
    pub download: bool,
}

fn default_qwen_model() -> String {
    "qwen3-max".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileUploadResponse {
    pub id: String,
    pub name: String,
    pub size: usize,
    pub file_class: String,
}

#[derive(Debug, Serialize)]
pub struct GenerateImageResponse {
    pub image_url: String,
    pub prompt: String,
    pub chat_id: Option<String>,
    pub response_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GenerateVideoResponse {
    pub video_url: String,
    pub prompt: String,
    pub chat_id: Option<String>,
    pub response_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Thread {
    pub id: String,
    pub object: String,
    pub created_at: u64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct Message {
    pub id: String,
    pub object: String,
    pub created_at: u64,
    pub thread_id: String,
    pub role: String,
    pub content: Vec<ContentPart>,
}

#[derive(Debug, Serialize)]
pub struct ContentPart {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: TextContent,
}

#[derive(Debug, Serialize)]
pub struct TextContent {
    pub value: String,
    pub annotations: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub id: String,
    pub object: String,
    pub created_at: u64,
    pub thread_id: String,
    pub status: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListThreadsResponse {
    pub object: String,
    pub data: Vec<Thread>,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct ListMessagesResponse {
    pub object: String,
    pub data: Vec<Message>,
    pub has_more: bool,
}

fn default_model() -> String {
    "grok-3-auto".to_string()
}
