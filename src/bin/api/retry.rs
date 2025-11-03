use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 10000,
            backoff_factor: 2.0,
        }
    }
}

pub struct RetryStrategy {
    config: RetryConfig,
}

impl RetryStrategy {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    pub fn get_delay(&self, attempt: u32) -> Duration {
        let delay_ms = (self.config.initial_delay_ms as f64
            * self.config.backoff_factor.powi(attempt as i32)) as u64;
        let delay_ms = delay_ms.min(self.config.max_delay_ms);
        Duration::from_millis(delay_ms)
    }

    pub fn should_retry(&self, attempt: u32, status_code: Option<u16>) -> bool {
        if attempt >= self.config.max_retries {
            return false;
        }

        match status_code {
            Some(code) => code == 429 || code >= 500,
            None => true,
        }
    }
}

pub async fn exponential_backoff_retry<F, T, E>(config: RetryConfig, mut f: F) -> Result<T, E>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>>>>,
{
    let strategy = RetryStrategy::new(config);
    let mut attempt = 0;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt >= strategy.config.max_retries {
                    return Err(e);
                }

                let delay = strategy.get_delay(attempt);
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
        }
    }
}
