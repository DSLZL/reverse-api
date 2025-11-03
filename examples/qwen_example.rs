use reverse_api::qwen::client::qwen::QwenClient;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ============================================================
    // HOW TO GET YOUR CREDENTIALS:
    // Option 1: Use Token (Recommended - easier)
    //   1. Go to https://chat.qwen.ai/ and log in
    //   2. Open DevTools (F12) -> Application -> Cookies
    //   3. Find "token" cookie and copy its value
    //   4. Run: export QWEN_TOKEN='your_token_here'
    //
    // Option 2: Use Email/Password
    //   1. Go to https://chat.qwen.ai/ and sign up/log in
    //   2. Run: export QWEN_EMAIL='your@email.com'
    //   3. Run: export QWEN_PASSWORD='your_password'
    // ============================================================

    println!("=== Qwen Conversation Example ===\n");

    // Try token first, then fall back to email/password
    let client = if let Ok(token) = std::env::var("QWEN_TOKEN") {
        println!("Using token authentication...\n");
        QwenClient::with_token(token)?
    } else {
        eprintln!("Error: No credentials provided!");
        eprintln!("\nPlease set either:");
        eprintln!("  export QWEN_TOKEN='your_token'");
        eprintln!("Or:");
        eprintln!("\nTo get a token:");
        eprintln!("1. Go to https://chat.qwen.ai/ and log in");
        eprintln!("2. Press F12 -> Application -> Cookies");
        eprintln!("3. Find 'token' and copy its value");
        std::process::exit(1);
    };

    // Get available models
    println!("=== Available Models ===");
    let models = client.get_models().await?;
    println!("Found {} models:", models.len());
    for model in models.iter().take(5) {
        println!("  - {}: {}", model.id, model.name);
    }
    println!();

    // Message 1 - Start a new conversation
    let message1 = "你好，请简单介绍一下你自己";
    println!("USER: {}", message1);
    print!("QWEN: ");
    let response1 = client.start_convo(message1, None, None).await?;
    println!();

    // Message 2 - Continue the conversation
    let message2 = "你能做什么？";
    println!("USER: {}", message2);
    print!("QWEN: ");
    let extra_data2 = reverse_api::qwen::models::ExtraData {
        chat_id: response1.chat_id.clone().unwrap(),
        model_id: "qwen3-max".to_string(),
        parent_id: Some(response1.response_id.clone()),
    };
    let response2 = client
        .start_convo(message2, None, Some(&extra_data2))
        .await?;
    println!();

    // Message 3 - Continue the conversation
    let message3 = "那我们来聊聊人工智能的发展吧";
    println!("USER: {}", message3);
    print!("QWEN: ");
    let extra_data3 = reverse_api::qwen::models::ExtraData {
        chat_id: response2.chat_id.clone().unwrap(),
        model_id: "qwen3-max".to_string(),
        parent_id: Some(response2.response_id.clone()),
    };
    let response3 = client
        .start_convo(message3, None, Some(&extra_data3))
        .await?;
    println!();

    // Message 4 - Continue the conversation
    let message4 = "你觉得AI对人类社会最大的影响是什么？";
    println!("USER: {}", message4);
    print!("QWEN: ");
    let extra_data4 = reverse_api::qwen::models::ExtraData {
        chat_id: response3.chat_id.clone().unwrap(),
        model_id: "qwen3-max".to_string(),
        parent_id: Some(response3.response_id.clone()),
    };
    let _response4 = client
        .start_convo(message4, None, Some(&extra_data4))
        .await?;
    println!();

    println!("=== Conversation Complete ===");
    println!("\nConversation Info:");
    println!("  Chat ID: {}", response1.chat_id.as_ref().unwrap());
    println!("  Model: qwen3-max");

    Ok(())
}
