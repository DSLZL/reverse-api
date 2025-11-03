use serde::{Deserialize, Serialize};

/// Chat type for different interaction modes
#[derive(Debug, Clone)]
#[derive(Default)]
pub enum ChatType {
    /// Normal text-to-text conversation
    #[default]
    TextToText,
    /// Web search enabled
    Search,
    /// Deep research mode
    DeepResearch,
    /// Image editing
    ImageEdit,
    /// Text to video
    TextToVideo,
    /// Text to image  
    TextToImage,
    /// Web development mode
    WebDev,
    /// Artifacts mode
    Artifacts,
    /// Travel planning
    Travel,
}

impl ChatType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChatType::TextToText => "t2t",
            ChatType::Search => "search",
            ChatType::DeepResearch => "deep_research",
            ChatType::ImageEdit => "image_edit",
            ChatType::TextToVideo => "t2v",
            ChatType::TextToImage => "t2i",
            ChatType::WebDev => "web_dev",
            ChatType::Artifacts => "artifacts",
            ChatType::Travel => "travel",
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub object: String,
    pub owned_by: String,
    #[serde(default)]
    pub info: Option<ModelInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub meta: ModelMeta,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelMeta {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub short_description: String,
    #[serde(default)]
    pub capabilities: ModelCapabilities,
    #[serde(default)]
    pub abilities: ModelAbilities,
    #[serde(default)]
    pub chat_type: Vec<String>,
    #[serde(default)]
    pub modality: Vec<String>,
    #[serde(default)]
    pub max_context_length: u32,
    #[serde(default)]
    pub max_generation_length: u32,
    #[serde(default)]
    pub max_thinking_generation_length: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ModelCapabilities {
    #[serde(default)]
    pub vision: bool,
    #[serde(default)]
    pub document: bool,
    #[serde(default)]
    pub video: bool,
    #[serde(default)]
    pub audio: bool,
    #[serde(default)]
    pub citations: bool,
    #[serde(default)]
    pub thinking: bool,
    #[serde(default)]
    pub thinking_budget: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ModelAbilities {
    #[serde(default)]
    pub vision: u32,
    #[serde(default)]
    pub document: u32,
    #[serde(default)]
    pub video: u32,
    #[serde(default)]
    pub audio: u32,
    #[serde(default)]
    pub citations: u32,
    #[serde(default)]
    pub thinking: u32,
    #[serde(default)]
    pub thinking_budget: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<Model>,
}

// Auth request/response
#[derive(Debug, Serialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub token: String,
    pub token_type: String,
}

// Chat creation
#[derive(Debug, Serialize)]
pub struct CreateChatRequest {
    pub chat: ChatConfig,
}

#[derive(Debug, Serialize)]
pub struct ChatConfig {
    pub name: String,
    pub models: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateChatResponse {
    pub success: bool,
    pub request_id: String,
    pub data: ChatData,
}

#[derive(Debug, Deserialize)]
pub struct ChatData {
    pub id: String,
}

// Chat completion request (Qwen format)
#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub stream: bool,
    pub incremental_output: bool,
    pub chat_id: String,
    pub chat_mode: String,
    pub model: String,
    pub parent_id: Option<String>,
    pub messages: Vec<QwenMessage>,
    pub timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QwenFile {
    #[serde(rename = "type")]
    pub r#type: String, // "image", "video", "audio", "file"
    pub file: FileObject,
    pub id: String,
    pub url: String,
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub collection_name: String,
    pub progress: u32,
    pub status: String, // "uploaded"
    #[serde(rename = "greenNet")]
    pub green_net: String, // "success", "greening"
    pub size: usize,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub error: String,
    #[serde(rename = "itemId")]
    pub item_id: String,
    pub file_type: String, // Content-Type
    #[serde(rename = "showType")]
    pub show_type: String, // "image", "file"
    pub file_class: String, // "vision", "document", "video", "audio"
    #[serde(rename = "uploadTaskId")]
    pub upload_task_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileObject {
    pub id: String,
    pub filename: String,
    pub user_id: String,
    pub created_at: u64,
    pub update_at: u64,
    #[serde(default)]
    pub data: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    pub meta: FileMeta,
}

#[derive(Debug, Serialize)]
pub struct QwenMessage {
    pub fid: String,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    #[serde(rename = "childrenIds")]
    pub children_ids: Vec<String>,
    pub role: String,
    pub content: String,
    pub user_action: String,
    pub files: Vec<QwenFile>,
    pub timestamp: u64,
    pub models: Vec<String>,
    pub chat_type: String,
    pub feature_config: FeatureConfig,
    pub extra: Extra,
    pub sub_chat_type: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct FeatureConfig {
    pub thinking_enabled: bool,
    pub output_schema: String,
    pub research_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_budget: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct Extra {
    pub meta: Meta,
}

#[derive(Debug, Serialize)]
pub struct Meta {
    #[serde(rename = "subChatType")]
    pub sub_chat_type: String,
}

// File upload response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadResponse {
    pub id: String,
    pub user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    pub filename: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default)]
    pub data: serde_json::Value,
    pub meta: FileMeta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_inspection_status: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    pub name: String,
    pub content_type: String,
    pub size: usize,
}

// STS Token Request/Response for OSS upload
#[derive(Debug, Serialize)]
pub struct StsTokenRequest {
    pub filename: String,
    pub filesize: usize,
    pub filetype: String, // "file" for documents, "image", "video", "audio"
}

#[derive(Debug, Deserialize)]
pub struct StsTokenResponse {
    pub success: bool,
    pub request_id: String,
    pub data: StsTokenData,
}

#[derive(Debug, Deserialize)]
pub struct StsTokenData {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub security_token: String,
    pub file_url: String,
    pub file_path: String,
    pub file_id: String,
    pub bucketname: String,
    pub region: String,
    pub endpoint: String,
}

// Web search result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchInfo {
    pub url: String,
    pub title: String,
    pub snippet: String,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub hostlogo: Option<String>,
    #[serde(default)]
    pub date: String,
}

// Response data structure for continuous conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenResponse {
    pub content: String,           // The actual response content
    pub response_id: String,       // For tracking conversation
    pub chat_id: Option<String>,   // Chat ID for continuation
    pub parent_id: Option<String>, // Parent message ID
    #[serde(default)]
    pub web_search_results: Option<Vec<WebSearchInfo>>, // Web search results if search was enabled
    #[serde(default)]
    pub thinking_content: Option<String>, // Thinking process if thinking was enabled
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraData {
    pub chat_id: String,
    pub model_id: String,
    pub parent_id: Option<String>, // The response_id from last message for context
}

// Video generation task response structures
#[derive(Debug, Deserialize)]
pub struct TaskResponse {
    pub success: bool,
    pub data: TaskData,
}

#[derive(Debug, Deserialize)]
pub struct TaskData {
    pub message_id: String,
    pub messages: Vec<TaskMessage>,
    pub chat_id: String,
    pub parent_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TaskMessage {
    pub extra: Option<TaskExtra>,
}

#[derive(Debug, Deserialize)]
pub struct TaskExtra {
    pub wanx: Option<WanxTask>,
}

#[derive(Debug, Deserialize)]
pub struct WanxTask {
    pub task_id: String,
}

// Task status polling response
#[derive(Debug, Deserialize, Clone)]
pub struct TaskStatus {
    pub chat_type: String,
    pub task_status: String, // "running", "success", "failed"
    pub message: String,
    pub remaining_time: String,
    pub content: String, // Video URL when completed
}
