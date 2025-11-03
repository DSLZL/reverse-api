use crate::chatgpt::utils::{Result, Utils};
use rquest::Client;
use serde::{Deserialize, Serialize};

/// IP information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpInfo {
    pub ip: String,
    pub city: String,
    pub region: String,
    pub latitude: String,
    pub longitude: String,
    pub timezone: String,
}

impl IpInfo {
    /// Fetch IP information from external services
    pub async fn fetch(client: &Client) -> Result<Self> {
        // First request to iplocation.com
        let response1 = client.get("https://iplocation.com/").send().await?;

        let html1 = response1.text().await?;

        let ip = Utils::between(&html1, r#"<td><b class="ip">"#, "<").unwrap_or_default();
        let city = Utils::between(&html1, r#"<td class="city">"#, "<").unwrap_or_default();
        let region =
            Utils::between(&html1, r#"<td><span class="region_name">"#, "<").unwrap_or_default();
        let latitude = Utils::between(&html1, r#"<td class="lat">"#, "<").unwrap_or_default();
        let longitude = Utils::between(&html1, r#"<td class="lng">"#, "<").unwrap_or_default();

        // Second request for timezone
        let response2 = client
            .get("https://ipaddresslocation.net/ip-to-timezone")
            .send()
            .await?;

        let html2 = response2.text().await?;
        let timezone = Utils::between(&html2, "Time Zone:</strong> ", " ").unwrap_or_default();

        Ok(IpInfo {
            ip,
            city,
            region,
            latitude,
            longitude,
            timezone,
        })
    }

    /// Convert to the format expected by the VM (as a list)
    pub fn to_list(&self) -> Vec<String> {
        vec![
            self.ip.clone(),
            self.city.clone(),
            self.region.clone(),
            self.latitude.clone(),
            self.longitude.clone(),
            self.timezone.clone(),
        ]
    }

    /// Get IP info as a string (without timezone for VM operations)
    pub fn without_timezone(&self) -> String {
        format!(
            "[{},{},{},{}]",
            serde_json::to_string(&self.ip).unwrap_or_default(),
            serde_json::to_string(&self.city).unwrap_or_default(),
            serde_json::to_string(&self.region).unwrap_or_default(),
            serde_json::to_string(&self.latitude).unwrap_or_default(),
        )
    }
}

impl Default for IpInfo {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            city: "Unknown".to_string(),
            region: "Unknown".to_string(),
            latitude: "0.0".to_string(),
            longitude: "0.0".to_string(),
            timezone: "UTC".to_string(),
        }
    }
}
