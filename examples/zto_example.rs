use reverse_api::ZtoClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Z.ai GLM-4.6 Example\n");
    println!("Creating Z.ai client...");

    let mut client = ZtoClient::new(None).await?;
    println!("✓ Client created successfully\n");

    let question = "What is the capital of France?";
    println!("Question: {}", question);
    println!("Waiting for response...\n");

    let response = client.ask_question(question).await?;

    println!("Response from Z.ai GLM-4.6:");
    println!("{}\n", response);

    // Second question with context
    println!("\nAsking a follow-up question...");
    let followup = "Tell me more about it.";
    println!("Question: {}", followup);

    let followup_response = client.ask_question(followup).await?;
    println!("Response from Z.ai GLM-4.6:");
    println!("{}\n", followup_response);

    println!("✓ Example completed successfully");
    Ok(())
}
