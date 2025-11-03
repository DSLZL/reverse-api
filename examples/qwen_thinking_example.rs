use reverse_api::qwen::client::qwen::QwenClient;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Qwen Deep Thinking Example ===\n");

    // Get token from environment or file
    let token = if let Ok(token) = std::env::var("QWEN_TOKEN") {
        token
    } else {
        std::fs::read_to_string(".qwen_token")?.trim().to_string()
    };

    let client = QwenClient::with_token(token)?;

    // Get available models
    println!("=== Available Models ===");
    let all_models = client.get_models().await?;
    println!("Found {} models:", all_models.len());

    // Get models that support thinking (dynamically from API)
    let thinking_models = client.get_thinking_capable_models().await?;

    println!(
        "\nModels with thinking support ({} models):",
        thinking_models.len()
    );
    for model in thinking_models.iter().take(10) {
        if let Some(info) = &model.info {
            let max_thinking = info.meta.max_thinking_generation_length;
            println!(
                "  - {}: {} (max thinking: {} tokens)",
                model.id, model.name, max_thinking
            );
        }
    }

    // Use the first available thinking-capable model
    let thinking_model = thinking_models
        .first()
        .map(|m| m.id.as_str())
        .unwrap_or("qwen3-vl-plus");

    println!("\n=== Using model: {} ===\n", thinking_model);

    // Get the model's thinking budget
    if let Some(max_budget) = client.get_model_thinking_budget(thinking_model).await? {
        println!(
            "This model supports up to {} tokens for thinking.\n",
            max_budget
        );
    }

    // Test 1: Complex math problem
    let message1 =
        "如果一个正方形的面积是144平方厘米，求这个正方形的对角线长度。请详细说明推理过程。";
    println!("USER: {}", message1);
    println!("QWEN (with deep thinking):");
    println!("--- Thinking Process ---");

    let response1 = client
        .start_convo_with_thinking(
            message1,
            Some(thinking_model),
            None,
            Some(10000), // thinking budget: 10000 tokens
        )
        .await?;
    println!();

    if let Some(thinking) = &response1.thinking_content {
        println!("\n=== Thinking Content ===");
        println!("{}", thinking);
    }

    println!("\n=== Final Answer ===");
    println!("{}", response1.content);

    // Test 2: Logic puzzle
    let message2 = "有三个人，A总是说真话，B总是说假话，C有时说真话有时说假话。\
                   现在你问其中一个人'你是A吗？'，他回答'是'。\
                   请问你能确定这个人是谁吗？为什么？";
    println!("\n\nUSER: {}", message2);
    println!("QWEN (with deep thinking):");
    println!("--- Thinking Process ---");

    let extra_data = reverse_api::qwen::models::ExtraData {
        chat_id: response1.chat_id.clone().unwrap(),
        model_id: thinking_model.to_string(),
        parent_id: Some(response1.response_id.clone()),
    };

    let response2 = client
        .start_convo_with_thinking(
            message2,
            Some(thinking_model),
            Some(&extra_data),
            Some(15000), // thinking budget: 15000 tokens
        )
        .await?;
    println!();

    if let Some(thinking) = &response2.thinking_content {
        println!("\n=== Thinking Content ===");
        println!("{}", thinking);
    }

    println!("\n=== Final Answer ===");
    println!("{}", response2.content);

    // Test 3: Strategy problem
    let message3 = "在井字棋(Tic-Tac-Toe)游戏中，如果你是先手玩家，\
                   请分析并说明最优的开局策略，以及为什么这个策略是最优的。";
    println!("\n\nUSER: {}", message3);
    println!("QWEN (with deep thinking):");
    println!("--- Thinking Process ---");

    let extra_data3 = reverse_api::qwen::models::ExtraData {
        chat_id: response2.chat_id.clone().unwrap(),
        model_id: thinking_model.to_string(),
        parent_id: Some(response2.response_id.clone()),
    };

    let response3 = client
        .start_convo_with_thinking(
            message3,
            Some(thinking_model),
            Some(&extra_data3),
            Some(20000), // thinking budget: 20000 tokens
        )
        .await?;
    println!();

    if let Some(thinking) = &response3.thinking_content {
        println!("\n=== Thinking Content ===");
        println!("{}", thinking);
    }

    println!("\n=== Final Answer ===");
    println!("{}", response3.content);

    println!("\n=== Deep Thinking Test Complete ===");
    println!("\nConversation Info:");
    println!("  Chat ID: {}", response3.chat_id.as_ref().unwrap());
    println!("  Model: {}", thinking_model);
    println!("  Total messages in conversation: 3");

    Ok(())
}
