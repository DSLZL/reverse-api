use reverse_api::QwenClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::fs::read_to_string(".qwen_token")?;
    let token = token.trim();

    let client = QwenClient::with_token(token.to_string())?;

    println!("=== 测试 1: 第一条消息 ===");
    let msg1 = "我最喜欢的动物是猫";
    println!("USER: {}", msg1);
    print!("QWEN: ");
    let resp1 = client.start_convo(msg1, Some("qwen3-max"), None).await?;
    println!();
    println!("Content length: {}", resp1.content.len());
    println!("Chat ID: {:?}", resp1.chat_id);
    println!("Response ID: {}", resp1.response_id);
    println!();

    println!("=== 测试 2: 连续对话 ===");
    let msg2 = "我最喜欢的动物是什么？";
    println!("USER: {}", msg2);
    print!("QWEN: ");

    let extra = reverse_api::qwen::models::ExtraData {
        chat_id: resp1.chat_id.clone().unwrap(),
        model_id: "qwen3-max".to_string(),
        parent_id: Some(resp1.response_id.clone()),
    };

    let resp2 = client
        .start_convo(msg2, Some("qwen3-max"), Some(&extra))
        .await?;
    println!();
    println!("Content length: {}", resp2.content.len());
    println!("Content: {}", resp2.content);

    Ok(())
}
