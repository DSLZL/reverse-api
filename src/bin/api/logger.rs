use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: String,
    pub message: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub endpoint: String,
    pub method: String,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub avg_duration_ms: u64,
    pub call_count: u64,
    pub error_count: u64,
    pub success_rate: f64,
}

pub struct Logger {
    logs: Arc<RwLock<Vec<LogEntry>>>,
    metrics: Arc<RwLock<std::collections::HashMap<String, PerformanceMetric>>>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn log(&self, level: &str, message: &str, context: Option<&str>) {
        let entry = LogEntry {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            level: level.to_string(),
            message: message.to_string(),
            context: context.map(|s| s.to_string()),
        };

        let mut logs = self.logs.write().await;
        logs.push(entry);
    }

    pub async fn get_logs(&self) -> Vec<LogEntry> {
        self.logs.read().await.clone()
    }

    pub async fn record_metric(&self, endpoint: &str, method: &str, duration_ms: u64, status: u16) {
        let key = format!("{} {}", method, endpoint);
        let mut metrics = self.metrics.write().await;

        let metric = metrics
            .entry(key.clone())
            .and_modify(|m| {
                m.call_count += 1;
                if duration_ms < m.min_duration_ms {
                    m.min_duration_ms = duration_ms;
                }
                if duration_ms > m.max_duration_ms {
                    m.max_duration_ms = duration_ms;
                }
                m.avg_duration_ms =
                    (m.avg_duration_ms * (m.call_count - 1) + duration_ms) / m.call_count;

                if status >= 400 {
                    m.error_count += 1;
                }
                m.success_rate =
                    ((m.call_count - m.error_count) as f64 / m.call_count as f64) * 100.0;
            })
            .or_insert_with(|| PerformanceMetric {
                endpoint: endpoint.to_string(),
                method: method.to_string(),
                min_duration_ms: duration_ms,
                max_duration_ms: duration_ms,
                avg_duration_ms: duration_ms,
                call_count: 1,
                error_count: if status >= 400 { 1 } else { 0 },
                success_rate: if status >= 400 { 0.0 } else { 100.0 },
            });
    }

    pub async fn get_metrics(&self) -> Vec<PerformanceMetric> {
        self.metrics.read().await.values().cloned().collect()
    }
}

impl Clone for Logger {
    fn clone(&self) -> Self {
        Self {
            logs: Arc::clone(&self.logs),
            metrics: Arc::clone(&self.metrics),
        }
    }
}
