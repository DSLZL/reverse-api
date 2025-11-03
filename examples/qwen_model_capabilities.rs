use reverse_api::qwen::client::qwen::QwenClient;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Qwen Model Capabilities Explorer ===\n");

    // Get token from environment or file
    let token = if let Ok(token) = std::env::var("QWEN_TOKEN") {
        token
    } else {
        std::fs::read_to_string(".qwen_token")?.trim().to_string()
    };

    let client = QwenClient::with_token(token)?;

    // Get all models
    println!("📋 Fetching all available models...\n");
    let models = client.get_models().await?;

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!(
        "║  Total Models Available: {:2}                                   ║",
        models.len()
    );
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Categorize models by capabilities
    let mut vision_models = Vec::new();
    let mut thinking_models = Vec::new();
    let mut search_models = Vec::new();
    let mut audio_models = Vec::new();
    let mut video_models = Vec::new();

    for model in &models {
        if let Some(info) = &model.info {
            let caps = &info.meta.capabilities;

            if caps.vision {
                vision_models.push(&model.id);
            }
            if caps.thinking {
                thinking_models.push(&model.id);
            }
            if info.meta.chat_type.contains(&"search".to_string()) {
                search_models.push(&model.id);
            }
            if caps.audio {
                audio_models.push(&model.id);
            }
            if caps.video {
                video_models.push(&model.id);
            }
        }
    }

    // Print capability summary
    println!("📊 Capability Summary:");
    println!(
        "  ├─ 👁️  Vision Support:        {} models",
        vision_models.len()
    );
    println!(
        "  ├─ 🧠 Deep Thinking:          {} models",
        thinking_models.len()
    );
    println!(
        "  ├─ 🔍 Web Search:             {} models",
        search_models.len()
    );
    println!(
        "  ├─ 🎵 Audio Processing:       {} models",
        audio_models.len()
    );
    println!(
        "  └─ 🎬 Video Processing:       {} models",
        video_models.len()
    );
    println!();

    // Detail each model
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  Detailed Model Information                                    ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    for (i, model) in models.iter().enumerate() {
        println!("{}. {} ({})", i + 1, model.name, model.id);

        if let Some(info) = &model.info {
            let caps = &info.meta.capabilities;
            let meta = &info.meta;

            // Description
            if !meta.short_description.is_empty() {
                println!("   📝 {}", meta.short_description);
            }

            // Capabilities
            println!("   ✨ Capabilities:");
            let mut cap_list = Vec::new();
            if caps.vision {
                cap_list.push("Vision");
            }
            if caps.document {
                cap_list.push("Document");
            }
            if caps.video {
                cap_list.push("Video");
            }
            if caps.audio {
                cap_list.push("Audio");
            }
            if caps.thinking {
                cap_list.push("Deep Thinking");
            }
            if caps.citations {
                cap_list.push("Citations");
            }
            if meta.chat_type.contains(&"search".to_string()) {
                cap_list.push("Web Search");
            }

            println!("      {}", cap_list.join(", "));

            // Context and generation limits
            println!("   📏 Limits:");
            println!("      • Context: {} tokens", meta.max_context_length);
            if meta.max_generation_length > 0 {
                println!("      • Generation: {} tokens", meta.max_generation_length);
            }
            if caps.thinking && meta.max_thinking_generation_length > 0 {
                println!(
                    "      • Thinking: {} tokens",
                    meta.max_thinking_generation_length
                );
            }

            // Supported chat types
            if !meta.chat_type.is_empty() {
                println!("   💬 Chat Types: {}", meta.chat_type.join(", "));
            }

            // Modalities
            if !meta.modality.is_empty() {
                println!("   🎨 Modalities: {}", meta.modality.join(", "));
            }
        }

        println!();
    }

    // Interactive capability check
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  Capability Check Examples                                     ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Test a few models
    let test_models = vec!["qwen3-max", "qwen3-vl-plus", "qwen3-coder-plus"];

    for model_id in test_models {
        println!("🔍 Checking '{}' capabilities:", model_id);

        // Check thinking support
        if let Ok(supports_thinking) = client.model_supports_thinking(model_id).await {
            print!("   • Deep Thinking: ");
            if supports_thinking {
                if let Ok(Some(budget)) = client.get_model_thinking_budget(model_id).await {
                    println!("✅ Yes (max {} tokens)", budget);
                } else {
                    println!("✅ Yes");
                }
            } else {
                println!("❌ No");
            }
        }

        // Check search support
        if let Ok(supports_search) = client.model_supports_search(model_id).await {
            print!("   • Web Search: ");
            if supports_search {
                println!("✅ Yes");
            } else {
                println!("❌ No");
            }
        }

        println!();
    }

    // Recommendations
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  Recommendations                                               ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("💡 For complex reasoning tasks:");
    let thinking_models = client.get_thinking_capable_models().await?;
    for model in thinking_models.iter().take(3) {
        println!("   • {} ({})", model.name, model.id);
    }

    println!("\n🔍 For current information needs:");
    let search_models = client.get_search_capable_models().await?;
    for model in search_models.iter().take(3) {
        println!("   • {} ({})", model.name, model.id);
    }

    println!("\n👁️  For vision tasks:");
    for model in models
        .iter()
        .filter(|m| {
            m.info
                .as_ref()
                .map(|i| i.meta.capabilities.vision)
                .unwrap_or(false)
        })
        .take(3)
    {
        println!("   • {} ({})", model.name, model.id);
    }

    println!("\n✅ Done! Use these capability checks in your code to:");
    println!("   1. Validate model support before making requests");
    println!("   2. Select appropriate models based on task requirements");
    println!("   3. Handle model changes gracefully without code updates");

    Ok(())
}
