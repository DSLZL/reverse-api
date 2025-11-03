use crate::qwen::error::Result;

pub struct MediaDownloader {
    client: rquest::Client,
}

impl MediaDownloader {
    pub fn new(client: rquest::Client) -> Self {
        Self { client }
    }

    /// Download media (image or video) from URL to local file
    pub async fn download_media(&self, url: &str, output_path: &str) -> Result<()> {
        println!("⬇️  Downloading media to: {}", output_path);

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(crate::qwen::error::QwenError::ApiError(format!(
                "Failed to download media: HTTP {}",
                response.status()
            )));
        }

        let bytes = response.bytes().await?;
        std::fs::write(output_path, bytes)?;

        println!(
            "✅ Downloaded successfully: {} bytes",
            std::fs::metadata(output_path)?.len()
        );
        Ok(())
    }
}
