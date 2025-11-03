use reverse_api::deepseek::client::deepseek::DeepSeekClient;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ============================================================
    // HOW TO GET YOUR TOKEN:
    // 1. Go to https://chat.deepseek.com/
    // 2. Login and start a conversation
    // 3. Press F12 to open Developer Tools
    // 4. Go to Application > LocalStorage > https://chat.deepseek.com
    // 5. Find the "userToken" key and copy its value
    // 6. Replace the token below with your actual userToken value
    // ============================================================

    let user_token = std::env::var("DEEPSEEK_TOKEN").unwrap_or_else(|_| {
        eprintln!("Error: DEEPSEEK_TOKEN environment variable not set!");
        eprintln!("\nTo get your token:");
        eprintln!("1. Go to https://chat.deepseek.com/");
        eprintln!("2. Login and start a conversation");
        eprintln!("3. Press F12 to open Developer Tools");
        eprintln!("4. Go to Application > LocalStorage > https://chat.deepseek.com");
        eprintln!("5. Find the 'userToken' key and copy its value");
        eprintln!("6. Run: export DEEPSEEK_TOKEN='your_token_here'");
        eprintln!("\nOr set it directly in the code (not recommended for production)");
        std::process::exit(1);
    });

    println!("=== DeepSeek Conversation Example ===\n");
    let client = DeepSeekClient::new(user_token).await?;

    // Message 1
    let message1 = "你好，请简单介绍一下你自己";
    println!("USER: {}", message1);
    let response1 = client.start_convo(message1, None).await?;
    println!(
        "DEEPSEEK: {}\n",
        response1
            .response
            .as_ref()
            .unwrap_or(&"No response".to_string())
    );

    // Message 2 - Continue conversation
    let message2 = "你能做什么？";
    println!("USER: {}", message2);
    let response2 = client
        .start_convo(message2, Some(&response1.extra_data))
        .await?;
    println!(
        "DEEPSEEK: {}\n",
        response2
            .response
            .as_ref()
            .unwrap_or(&"No response".to_string())
    );

    // Message 3 - Continue conversation
    let message3 = "那我们来聊聊人工智能的发展吧";
    println!("USER: {}", message3);
    let response3 = client
        .start_convo(message3, Some(&response2.extra_data))
        .await?;
    println!(
        "DEEPSEEK: {}\n",
        response3
            .response
            .as_ref()
            .unwrap_or(&"No response".to_string())
    );

    println!("=== Conversation Complete ===");

    Ok(())
}
