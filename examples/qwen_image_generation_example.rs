use reverse_api::QwenClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get token from environment
    let token = env::var("QWEN_TOKEN").expect("QWEN_TOKEN environment variable not set");

    println!("🎨 Qwen Image Generation Example\n");
    println!(
        "This example demonstrates how to generate images using Qwen's text-to-image capability.\n"
    );

    // Initialize client with token
    let client = QwenClient::with_token(token)?;
    println!("✅ Client initialized\n");

    // Example 1: Basic Image Generation
    println!("📷 Example 1: Basic Image Generation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💬 Prompt: 一只可爱的橙色小猫");

    let response1 = client
        .generate_image(
            "一只可爱的橙色小猫",
            Some("1:1"), // Square image
            None,        // Auto-select model (qwen3-max)
            None,        // New conversation
        )
        .await?;

    println!("🖼️  Generated Image URL: {}", response1.content);
    println!("📝 Response ID: {}", response1.response_id);
    println!("💬 Chat ID: {}", response1.chat_id.as_ref().unwrap());

    // Download the image
    let image_path = "generated_image_1.png";
    client
        .download_media(&response1.content, image_path)
        .await?;
    println!("✅ Image saved to: {}", image_path);
    println!();

    // Example 2: Generate Another Image in a New Conversation
    println!("📷 Example 2: Different Image Style");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💬 Prompt: 一幅精致细腻的工笔画，画面中心是一株蓬勃生长的红色牡丹");

    let response2 = client
        .generate_image(
            "一幅精致细腻的工笔画，画面中心是一株蓬勃生长的红色牡丹",
            Some("1:1"),
            None,
            None,
        )
        .await?;

    println!("🖼️  Generated Image URL: {}", response2.content);

    // Download the image
    let image_path2 = "generated_image_2.png";
    client
        .download_media(&response2.content, image_path2)
        .await?;
    println!("✅ Image saved to: {}", image_path2);
    println!();

    // Example 3: Continuous Image Generation (with context)
    println!("📷 Example 3: Continuous Image Generation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💬 First Prompt: 一只穿着西装的水豚");

    let response3 = client
        .generate_image("一只穿着西装的水豚", Some("1:1"), None, None)
        .await?;

    println!("🖼️  First Image URL: {}", response3.content);

    // Download the image
    let image_path3 = "generated_image_3.png";
    client
        .download_media(&response3.content, image_path3)
        .await?;
    println!("✅ Image saved to: {}", image_path3);

    // Generate a follow-up image in the same conversation
    println!("\n💬 Follow-up Prompt: 现在让它举着一个牌子，上面写着'Hello World'");

    use reverse_api::qwen::models::ExtraData;
    let extra_data = ExtraData {
        chat_id: response3.chat_id.clone().unwrap(),
        model_id: "qwen3-max".to_string(),
        parent_id: Some(response3.response_id.clone()),
    };

    let response4 = client
        .generate_image(
            "现在让它举着一个牌子，上面写着'Hello World'",
            Some("1:1"),
            None,
            Some(&extra_data),
        )
        .await?;

    println!("🖼️  Follow-up Image URL: {}", response4.content);

    // Download the image
    let image_path4 = "generated_image_4.png";
    client
        .download_media(&response4.content, image_path4)
        .await?;
    println!("✅ Image saved to: {}", image_path4);
    println!();

    // Example 4: Complex Prompt
    println!("📷 Example 4: Complex Detailed Prompt");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let complex_prompt = "杰作，最佳品质，官方艺术，极其精细的CG Unity 8K壁纸，\
        一张东京街头风格的地图放置在混凝土表面上，从地图中出现了一个照片般真实的迷你版东京——\
        樱花树、东京塔、涩谷十字路口、霓虹灯招牌和微型子弹列车。\
        所有元素像一个3D城市微缩模型一样从地图上生长出来。工作室柔和的灯光，电影般的深度";

    println!("💬 Prompt: {}", complex_prompt);

    let response5 = client
        .generate_image(complex_prompt, Some("1:1"), None, None)
        .await?;

    println!("🖼️  Generated Image URL: {}", response5.content);

    // Download the image
    let image_path5 = "generated_image_5.png";
    client
        .download_media(&response5.content, image_path5)
        .await?;
    println!("✅ Image saved to: {}", image_path5);
    println!();

    println!("✅ All examples completed!");
    println!("\n📝 Key Features Demonstrated:");
    println!("   • Basic image generation from text prompts");
    println!("   • Different artistic styles");
    println!("   • Continuous generation with context");
    println!("   • Complex detailed prompts");
    println!("   • Automatic image download to local files");
    println!("\n💡 Tips:");
    println!("   • Image URLs are temporary and include JWT authentication");
    println!("   • Images are automatically downloaded and saved locally");
    println!("   • Default size is 1:1 (square), typically 1328x1328 pixels");
    println!("   • Use detailed prompts for better results");
    println!("\n📂 Generated Files:");
    println!("   • {}", image_path);
    println!("   • {}", image_path2);
    println!("   • {}", image_path3);
    println!("   • {}", image_path4);
    println!("   • {}", image_path5);

    Ok(())
}
