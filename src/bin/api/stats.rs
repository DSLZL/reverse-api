use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub last_request_time: Option<u64>,
    pub average_response_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveRequest {
    pub id: String,
    pub timestamp: u64,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: u64,
    pub user_agent: String,
}

pub struct StatsCollector {
    stats: Arc<RwLock<RequestStats>>,
    live_requests: Arc<RwLock<Vec<LiveRequest>>>,
}

const MAX_LIVE_REQUESTS: usize = 100;

impl StatsCollector {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(RequestStats {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                last_request_time: None,
                average_response_time: 0,
            })),
            live_requests: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn record_request(
        &self,
        method: &str,
        path: &str,
        status: u16,
        duration: Duration,
        user_agent: &str,
    ) {
        let mut stats = self.stats.write().await;
        let duration_ms = duration.as_millis() as u64;

        stats.total_requests += 1;
        stats.last_request_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        if (200..300).contains(&status) {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }

        if stats.total_requests > 0 {
            let total_duration =
                stats.average_response_time * (stats.total_requests - 1) + duration_ms;
            stats.average_response_time = total_duration / stats.total_requests;
        } else {
            stats.average_response_time = duration_ms;
        }

        drop(stats);

        let request = LiveRequest {
            id: format!(
                "{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            method: method.to_string(),
            path: path.to_string(),
            status,
            duration_ms,
            user_agent: user_agent.to_string(),
        };

        let mut requests = self.live_requests.write().await;
        requests.push(request);
        if requests.len() > MAX_LIVE_REQUESTS {
            requests.remove(0);
        }
    }

    pub async fn get_stats(&self) -> RequestStats {
        self.stats.read().await.clone()
    }

    pub async fn get_live_requests(&self) -> Vec<LiveRequest> {
        self.live_requests.read().await.clone()
    }
}

impl Clone for StatsCollector {
    fn clone(&self) -> Self {
        Self {
            stats: Arc::clone(&self.stats),
            live_requests: Arc::clone(&self.live_requests),
        }
    }
}
