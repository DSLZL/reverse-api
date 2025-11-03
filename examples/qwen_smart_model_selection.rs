use reverse_api::qwen::client::qwen::QwenClient;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Qwen Smart Model Selection Example ===\n");

    // Get token from environment or file
    let token = if let Ok(token) = std::env::var("QWEN_TOKEN") {
        token
    } else {
        std::fs::read_to_string(".qwen_token")?.trim().to_string()
    };

    let client = QwenClient::with_token(token)?;

    println!("This example demonstrates how the client automatically selects");
    println!("the best model based on your requirements.\n");

    // Example 1: Need vision capability
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  Example 1: Task Requires Vision                              ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let best_vision_model = client
        .select_best_model(
            true,  // requires_vision
            false, // requires_audio
            false, // requires_video
            false, // requires_thinking
            false, // requires_search
        )
        .await?;

    println!("✓ Best model for vision tasks: {}", best_vision_model);

    // Verify it supports vision
    let vision_models = client.get_vision_capable_models().await?;
    println!("  Available vision models: {}", vision_models.len());

    // Example 2: Need thinking capability
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  Example 2: Task Requires Deep Thinking                       ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let best_thinking_model = client
        .select_best_model(
            false, // requires_vision
            false, // requires_audio
            false, // requires_video
            true,  // requires_thinking
            false, // requires_search
        )
        .await?;

    println!("✓ Best model for thinking tasks: {}", best_thinking_model);

    if let Ok(Some(budget)) = client.get_model_thinking_budget(&best_thinking_model).await {
        println!("  Thinking budget: {} tokens", budget);
    }

    // Example 3: Need search capability
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  Example 3: Task Requires Web Search                          ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let best_search_model = client
        .select_best_model(
            false, // requires_vision
            false, // requires_audio
            false, // requires_video
            false, // requires_thinking
            true,  // requires_search
        )
        .await?;

    println!("✓ Best model for search tasks: {}", best_search_model);

    // Example 4: Complex requirements - vision + thinking
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  Example 4: Complex Task (Vision + Thinking)                  ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let best_complex_model = client
        .select_best_model(
            true,  // requires_vision
            false, // requires_audio
            false, // requires_video
            true,  // requires_thinking
            false, // requires_search
        )
        .await?;

    println!("✓ Best model for vision + thinking: {}", best_complex_model);

    // Example 5: Audio processing
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  Example 5: Task Requires Audio Processing                    ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let best_audio_model = client
        .select_best_model(
            false, // requires_vision
            true,  // requires_audio
            false, // requires_video
            false, // requires_thinking
            false, // requires_search
        )
        .await?;

    println!("✓ Best model for audio tasks: {}", best_audio_model);

    let audio_models = client.get_audio_capable_models().await?;
    println!("  Available audio models: {}", audio_models.len());

    // Example 6: Video processing
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  Example 6: Task Requires Video Processing                    ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let best_video_model = client
        .select_best_model(
            false, // requires_vision
            false, // requires_audio
            true,  // requires_video
            false, // requires_thinking
            false, // requires_search
        )
        .await?;

    println!("✓ Best model for video tasks: {}", best_video_model);

    let video_models = client.get_video_capable_models().await?;
    println!("  Available video models: {}", video_models.len());

    // Example 7: Multimodal - everything!
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  Example 7: Ultimate Multimodal Task (All Capabilities)       ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let ultimate_model = client
        .select_best_model(
            true, // requires_vision
            true, // requires_audio
            true, // requires_video
            true, // requires_thinking
            true, // requires_search
        )
        .await?;

    println!("✓ Best all-around model: {}", ultimate_model);

    // Check what this model can do
    let models = client.get_models().await?;
    if let Some(model) = models.iter().find(|m| m.id == ultimate_model) {
        if let Some(info) = &model.info {
            let caps = &info.meta.capabilities;
            println!("\n  Capabilities of {}:", model.name);
            println!("    • Vision: {}", if caps.vision { "✅" } else { "❌" });
            println!("    • Audio: {}", if caps.audio { "✅" } else { "❌" });
            println!("    • Video: {}", if caps.video { "✅" } else { "❌" });
            println!(
                "    • Thinking: {}",
                if caps.thinking { "✅" } else { "❌" }
            );
            println!(
                "    • Document: {}",
                if caps.document { "✅" } else { "❌" }
            );
            println!(
                "    • Citations: {}",
                if caps.citations { "✅" } else { "❌" }
            );
            println!("    • Context: {} tokens", info.meta.max_context_length);
        }
    }

    // Practical example: File-based selection
    println!("\n\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  Practical Usage: Automatic Selection from Files              ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("When using start_convo_with_files(), the client automatically");
    println!("selects the best model based on file types:\n");

    println!("Examples:");
    println!("  • Image file → Selects vision-capable model");
    println!("  • Audio file → Selects audio-capable model");
    println!("  • Video file → Selects video-capable model");
    println!("  • Multiple types → Selects model supporting all types\n");

    println!("Code example:");
    println!("  let files = client.upload_files(&[\"image.jpg\"]).await?;");
    println!("  // No need to specify model - automatically selects best one!");
    println!("  let response = client.start_convo_with_files(");
    println!("      \"Describe this image\",");
    println!("      files,");
    println!("      None,  // Auto-select model");
    println!("      None,");
    println!("  ).await?;");

    // Summary
    println!("\n\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  Summary                                                       ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("✅ Smart model selection benefits:");
    println!("   1. No hardcoded model IDs - adapts to API changes");
    println!("   2. Always uses the best model for your task");
    println!("   3. Automatic fallback if preferred model unavailable");
    println!("   4. Optimized for performance and capabilities");
    println!("   5. Reduces code maintenance");

    println!("\n💡 Best practices:");
    println!("   • Let the client auto-select for multimodal tasks");
    println!("   • Only specify model_id if you need a specific one");
    println!("   • Use capability checks before making requests");
    println!("   • Handle edge cases where no suitable model exists");

    Ok(())
}
