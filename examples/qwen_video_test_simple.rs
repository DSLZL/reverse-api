use reverse_api::QwenClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("QWEN_TOKEN").expect("QWEN_TOKEN environment variable not set");

    println!("🎬 Qwen Video Generation Test\n");

    let client = QwenClient::with_token(token)?;
    println!("✅ Client initialized\n");

    println!("🎥 Generating video: 一只可爱的小猫在玩耍");
    println!("⏳ This will take 1-3 minutes...\n");

    let response = client
        .generate_video_with_progress(
            "一只可爱的小猫在玩耍",
            Some("16:9"),
            None,
            None,
            |status, percent| {
                if percent % 20 == 0 || status == "success" {
                    println!("📊 Status: {} - {}%", status, percent);
                }
            },
        )
        .await?;

    println!("\n🎬 Video URL: {}", response.content);
    println!("📝 Response ID: {}", response.response_id);

    // Download video
    let video_path = "test_video.mp4";
    println!("\n⬇️  Downloading video...");
    client.download_media(&response.content, video_path).await?;
    println!("✅ Video saved to: {}", video_path);

    // Check file info
    let metadata = std::fs::metadata(video_path)?;
    println!(
        "📦 File size: {} bytes ({:.2} MB)",
        metadata.len(),
        metadata.len() as f64 / 1024.0 / 1024.0
    );

    Ok(())
}
