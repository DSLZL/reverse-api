use reverse_api::qwen::client::qwen::QwenClient;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Qwen Advanced Features Example ===");
    println!("This example demonstrates both web search and deep thinking features\n");

    // Get token from environment or file
    let token = if let Ok(token) = std::env::var("QWEN_TOKEN") {
        token
    } else {
        std::fs::read_to_string(".qwen_token")?.trim().to_string()
    };

    let client = QwenClient::with_token(token)?;

    // Part 1: Web Search - Get current information
    println!("\n╔═══════════════════════════════════════╗");
    println!("║   PART 1: Web Search Feature        ║");
    println!("╚═══════════════════════════════════════╝\n");

    let search_question = "2024年诺贝尔物理学奖获得者是谁？他们的研究成果是什么？";
    println!("Question (with web search): {}", search_question);
    print!("\nQwen is searching the web...\n");

    let search_response = client
        .start_convo_with_search(search_question, Some("qwen3-max"), None)
        .await?;
    println!();

    if let Some(results) = &search_response.web_search_results {
        println!("\n📚 Found {} web sources", results.len());
        println!("\nTop sources:");
        for (i, result) in results.iter().enumerate().take(3) {
            println!("\n  {}. {}", i + 1, result.title);
            println!("     🔗 {}", result.url);
            if !result.snippet.is_empty() {
                let snippet = if result.snippet.len() > 100 {
                    format!("{}...", &result.snippet[..100])
                } else {
                    result.snippet.clone()
                };
                println!("     📝 {}", snippet);
            }
        }
    }

    println!("\n✨ Answer based on web search:");
    println!("{}", search_response.content);

    // Part 2: Deep Thinking - Complex reasoning
    println!("\n\n╔═══════════════════════════════════════╗");
    println!("║   PART 2: Deep Thinking Feature     ║");
    println!("╚═══════════════════════════════════════╝\n");

    let thinking_question =
        "设计一个算法来检测一个有向图中是否存在环。请详细说明你的思考过程和算法复杂度。";
    println!("Question (with deep thinking): {}", thinking_question);
    println!("\n🧠 Qwen is thinking deeply...");
    println!("--- Thinking Process ---\n");

    let thinking_response = client
        .start_convo_with_thinking(
            thinking_question,
            Some("qwen3-vl-plus"),
            None,
            Some(15000), // 15k thinking budget
        )
        .await?;
    println!();

    if let Some(thinking) = &thinking_response.thinking_content {
        if !thinking.is_empty() {
            println!("\n💭 Internal Thinking Process:");
            println!("{}", thinking);
        }
    }

    println!("\n✨ Final Answer:");
    println!("{}", thinking_response.content);

    // Part 3: Combined - Search then think
    println!("\n\n╔═══════════════════════════════════════╗");
    println!("║   PART 3: Combined Approach         ║");
    println!("╚═══════════════════════════════════════╝\n");

    println!("Step 1: First, let's search for information...\n");
    let combined_search = "量子计算机目前的最新进展是什么？";
    println!("Search question: {}", combined_search);
    print!("\nSearching...\n");

    let search_result = client
        .start_convo_with_search(combined_search, Some("qwen3-max"), None)
        .await?;
    println!();

    if let Some(results) = &search_result.web_search_results {
        println!(
            "\n📚 Found {} sources about quantum computing",
            results.len()
        );
    }

    println!("\n✨ Search Result:");
    let summary = if search_result.content.len() > 200 {
        format!("{}...", &search_result.content[..200])
    } else {
        search_result.content.clone()
    };
    println!("{}", summary);

    println!("\n\nStep 2: Now let's think deeply about the implications...\n");
    let thinking_followup = "基于量子计算的最新进展，请分析它对现代密码学可能产生的影响，\
                            以及我们应该如何应对这种技术变革。";
    println!("Thinking question: {}", thinking_followup);
    println!("\n🧠 Deep thinking...\n");

    let extra_data = reverse_api::qwen::models::ExtraData {
        chat_id: search_result.chat_id.clone().unwrap(),
        model_id: "qwen3-vl-plus".to_string(),
        parent_id: Some(search_result.response_id.clone()),
    };

    let thinking_followup_response = client
        .start_convo_with_thinking(
            thinking_followup,
            Some("qwen3-vl-plus"),
            Some(&extra_data),
            Some(20000),
        )
        .await?;
    println!();

    if let Some(thinking) = &thinking_followup_response.thinking_content {
        if !thinking.is_empty() {
            println!("\n💭 Thought Process:");
            let thought_preview = if thinking.len() > 150 {
                format!("{}...", &thinking[..150])
            } else {
                thinking.clone()
            };
            println!("{}", thought_preview);
        }
    }

    println!("\n✨ Final Analysis:");
    println!("{}", thinking_followup_response.content);

    // Summary
    println!("\n\n╔═══════════════════════════════════════╗");
    println!("║   Summary                            ║");
    println!("╚═══════════════════════════════════════╝\n");

    println!("✅ Successfully demonstrated:");
    println!("   • Web Search: Real-time information retrieval");
    println!("   • Deep Thinking: Complex reasoning and analysis");
    println!("   • Combined Approach: Search + Think for comprehensive answers");
    println!("\n📊 Features tested:");
    println!("   • Continuous conversation");
    println!("   • Search result citations");
    println!("   • Thinking process visibility");
    println!("   • Model switching");

    Ok(())
}
