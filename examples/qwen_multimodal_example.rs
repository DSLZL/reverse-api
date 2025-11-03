use reverse_api::QwenClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get credentials from environment
    let token = env::var("QWEN_TOKEN").expect("QWEN_TOKEN environment variable not set");

    println!("🚀 Qwen Multimodal Chat Example\n");
    println!("This example demonstrates how to use images, documents, videos, and audio in conversations.\n");

    // Initialize client with token
    let client = QwenClient::with_token(token)?;
    println!("✅ Client initialized\n");

    // Example 1: Image Analysis
    println!("📷 Example 1: Image Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if std::path::Path::new("test_image.jpg").exists() {
        println!("📤 Uploading image...");
        let image_file = client.upload_file("test_image.jpg").await?;
        println!(
            "✅ Image uploaded: {} ({})",
            image_file.name, image_file.file_class
        );

        println!("💬 Sending message with image...");
        println!("🤖 Auto-selecting best vision model...");
        let response = client
            .start_convo_with_files(
                "请详细描述这张图片的内容",
                vec![image_file],
                None, // Auto-select best model for vision
                None,
            )
            .await?;

        println!("🤖 Response: {}", response.content);
        println!();
    } else {
        println!("⚠️  test_image.jpg not found, skipping image example\n");
    }

    // Example 2: Document Analysis
    println!("📄 Example 2: Document Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if std::path::Path::new("test_document.txt").exists() {
        println!("📤 Uploading document...");
        let doc_file = client.upload_file("test_document.txt").await?;
        println!(
            "✅ Document uploaded: {} ({})",
            doc_file.name, doc_file.file_class
        );

        println!("💬 Sending message with document...");
        let response = client
            .start_convo_with_files(
                "请总结这个文档的内容",
                vec![doc_file],
                None, // Auto-select best model
                None,
            )
            .await?;

        println!("🤖 Response: {}", response.content);
        println!();
    } else {
        println!("⚠️  test_document.txt not found, skipping document example\n");
    }

    // Example 3: Video Analysis
    println!("🎬 Example 3: Video Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if std::path::Path::new("test_video.mp4").exists() {
        println!("📤 Uploading video...");
        let video_file = client.upload_file("test_video.mp4").await?;
        println!(
            "✅ Video uploaded: {} ({})",
            video_file.name, video_file.file_class
        );
        println!(
            "⚠️  Note: greenNet status = {} (may need approval)",
            video_file.green_net
        );

        println!("💬 Sending message with video...");
        match client
            .start_convo_with_files(
                "请描述这个视频的内容",
                vec![video_file],
                None, // Auto-select best model
                None,
            )
            .await
        {
            Ok(response) => {
                println!("🤖 Response: {}", response.content);
            }
            Err(e) => {
                println!(
                    "❌ Error: {} (this may happen if video is still processing)",
                    e
                );
            }
        }
        println!();
    } else {
        println!("⚠️  test_video.mp4 not found, skipping video example\n");
    }

    // Example 4: Audio Transcription
    println!("🎵 Example 4: Audio Transcription");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if std::path::Path::new("test_audio.mp3").exists() {
        println!("📤 Uploading audio...");
        let audio_file = client.upload_file("test_audio.mp3").await?;
        println!(
            "✅ Audio uploaded: {} ({})",
            audio_file.name, audio_file.file_class
        );

        println!("💬 Sending message with audio...");
        let response = client
            .start_convo_with_files(
                "请转录这段音频的内容",
                vec![audio_file],
                None, // Auto-select best model
                None,
            )
            .await?;

        println!("🤖 Response: {}", response.content);
        println!();
    } else {
        println!("⚠️  test_audio.mp3 not found, skipping audio example\n");
    }

    // Example 5: Continuous Conversation with Files
    println!("🔄 Example 5: Continuous Conversation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if std::path::Path::new("test_image.jpg").exists() {
        println!("📤 Uploading image for continuous chat...");
        let image_file = client.upload_file("test_image.jpg").await?;

        println!("💬 First message: Asking about the image...");
        let response1 = client
            .start_convo_with_files(
                "这张图片的主要颜色是什么？",
                vec![image_file],
                None, // Auto-select best model
                None,
            )
            .await?;
        println!("🤖 Response 1: {}", response1.content);

        println!("\n💬 Follow-up message (using parent_id for context)...");
        let response2 = client
            .continue_convo(
                "能详细解释一下为什么是这个颜色吗？",
                response1.chat_id.as_ref().unwrap(),
                Some(&response1.response_id),
                None, // Auto-select best model
                None,
            )
            .await?;
        println!("🤖 Response 2: {}", response2.content);
        println!();
    }

    println!("✅ All examples completed!");
    println!("\n📝 Key Features Demonstrated:");
    println!("   • Image upload and analysis");
    println!("   • Document upload and summarization");
    println!("   • Video upload (with content moderation)");
    println!("   • Audio upload and transcription");
    println!("   • Continuous conversation with context");

    Ok(())
}
