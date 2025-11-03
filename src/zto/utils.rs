use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate_chat_id() -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}-{}", now.as_nanos(), now.as_secs())
}

pub fn generate_message_id() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string()
}
