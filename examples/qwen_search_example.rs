use reverse_api::qwen::client::qwen::QwenClient;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Qwen Web Search Example ===\n");

    // Get token from environment or file
    let token = if let Ok(token) = std::env::var("QWEN_TOKEN") {
        token
    } else {
        std::fs::read_to_string(".qwen_token")?.trim().to_string()
    };

    let client = QwenClient::with_token(token)?;

    // Check which models support search
    let search_models = client.get_search_capable_models().await?;
    println!(
        "=== Models with Search Support ({} models) ===",
        search_models.len()
    );
    for model in search_models.iter().take(5) {
        println!("  - {}: {}", model.id, model.name);
    }
    println!();

    println!("=== Testing Web Search Feature ===\n");

    // Test 1: Search for current events
    let message1 = "今天比特币的价格是多少？";
    println!("USER: {}", message1);
    print!("QWEN (with search): ");
    let response1 = client.start_convo_with_search(message1, None, None).await?;
    println!();

    if let Some(search_results) = &response1.web_search_results {
        println!(
            "\n=== Web Search Results ({} sources) ===",
            search_results.len()
        );
        for (i, result) in search_results.iter().enumerate().take(5) {
            println!("\n[{}] {}", i + 1, result.title);
            println!("    URL: {}", result.url);
            println!("    Snippet: {}", result.snippet);
        }
    }

    println!("\n=== Response Content ===");
    println!("{}", response1.content);

    // Test 2: Continue conversation with search
    let message2 = "那以太坊呢？";
    println!("\n\nUSER: {}", message2);
    print!("QWEN (with search): ");

    // Create extra_data for conversation continuation
    let extra_data = reverse_api::qwen::models::ExtraData {
        chat_id: response1.chat_id.clone().unwrap(),
        model_id: "qwen3-max".to_string(),
        parent_id: Some(response1.response_id.clone()),
    };

    let response2 = client
        .start_convo_with_search(message2, None, Some(&extra_data))
        .await?;
    println!();

    if let Some(search_results) = &response2.web_search_results {
        println!(
            "\n=== Web Search Results ({} sources) ===",
            search_results.len()
        );
        for (i, result) in search_results.iter().enumerate().take(3) {
            println!("\n[{}] {}", i + 1, result.title);
            println!("    URL: {}", result.url);
        }
    }

    println!("\n=== Response Content ===");
    println!("{}", response2.content);

    // Test 3: Search for recent news
    let message3 = "最近AI领域有什么重大新闻？";
    println!("\n\nUSER: {}", message3);
    print!("QWEN (with search): ");

    let extra_data3 = reverse_api::qwen::models::ExtraData {
        chat_id: response2.chat_id.clone().unwrap(),
        model_id: "qwen3-max".to_string(),
        parent_id: Some(response2.response_id.clone()),
    };

    let response3 = client
        .start_convo_with_search(message3, None, Some(&extra_data3))
        .await?;
    println!();

    if let Some(search_results) = &response3.web_search_results {
        println!(
            "\n=== Web Search Results ({} sources) ===",
            search_results.len()
        );
    }

    println!("\n=== Response Content ===");
    println!("{}", response3.content);

    println!("\n=== Search Test Complete ===");
    println!("\nConversation Info:");
    println!("  Chat ID: {}", response3.chat_id.as_ref().unwrap());
    println!("  Total messages in conversation: 3");

    Ok(())
}
