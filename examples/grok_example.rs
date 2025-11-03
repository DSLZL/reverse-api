use reverse_api::{Grok, Logger};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proxy = ""; // Set your proxy here if needed

    // Message 1
    let message1 = "Hey how are you??";
    Logger::info(&format!("USER: {}", message1));

    let mut grok = Grok::new(
        "grok-3-auto",
        if proxy.is_empty() { None } else { Some(proxy) },
    )?;
    let data1 = grok.start_convo(message1, None).await?;
    Logger::info(&format!(
        "GROK: {}",
        data1
            .response
            .as_ref()
            .unwrap_or(&"No response".to_string())
    ));

    // Message 2
    let message2 = "cool stuff";
    Logger::info(&format!("USER: {}", message2));

    let mut grok2 = Grok::new(
        "grok-3-auto",
        if proxy.is_empty() { None } else { Some(proxy) },
    )?;
    let data2 = grok2.start_convo(message2, Some(&data1.extra_data)).await?;
    Logger::info(&format!(
        "GROK: {}",
        data2
            .response
            .as_ref()
            .unwrap_or(&"No response".to_string())
    ));

    // Message 3
    let message3 = "crazy";
    Logger::info(&format!("USER: {}", message3));

    let mut grok3 = Grok::new(
        "grok-3-auto",
        if proxy.is_empty() { None } else { Some(proxy) },
    )?;
    let data3 = grok3.start_convo(message3, Some(&data2.extra_data)).await?;
    Logger::info(&format!(
        "GROK: {}",
        data3
            .response
            .as_ref()
            .unwrap_or(&"No response".to_string())
    ));

    // Message 4
    let message4 = "Well this is the 4th message in our chat now omg";
    Logger::info(&format!("USER: {}", message4));

    let mut grok4 = Grok::new(
        "grok-3-auto",
        if proxy.is_empty() { None } else { Some(proxy) },
    )?;
    let data4 = grok4.start_convo(message4, Some(&data3.extra_data)).await?;
    Logger::info(&format!(
        "GROK: {}",
        data4
            .response
            .as_ref()
            .unwrap_or(&"No response".to_string())
    ));

    // Message 5
    let message5 = "And now the 5th omg";
    Logger::info(&format!("USER: {}", message5));

    let mut grok5 = Grok::new(
        "grok-3-auto",
        if proxy.is_empty() { None } else { Some(proxy) },
    )?;
    let data5 = grok5.start_convo(message5, Some(&data4.extra_data)).await?;
    Logger::info(&format!(
        "GROK: {}",
        data5
            .response
            .as_ref()
            .unwrap_or(&"No response".to_string())
    ));

    Ok(())
}
