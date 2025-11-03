use reverse_api::chatgpt::{log_error, log_info, log_success, ChatGptClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Set your proxy here if needed
    let proxy = ""; // Example: "http://127.0.0.1:1082"

    let proxy_msg = if proxy.is_empty() {
        String::new()
    } else {
        format!(" with proxy: {}", proxy)
    };

    log_info!("Creating ChatGPT client{}...", proxy_msg);

    // Create client
    let mut client = ChatGptClient::new(if proxy.is_empty() { None } else { Some(proxy) }).await?;

    log_success!("Client created successfully!");

    // Ask a question
    let question = "你是怎么看待openai这个公司的";
    log_info!("Asking question: {}", question);

    match client.ask_question(question).await {
        Ok(response) => {
            log_success!("Got response:");
            println!("\n{}\n", response);
        }
        Err(e) => {
            log_error!("Error: {}", e);
        }
    }

    Ok(())
}
