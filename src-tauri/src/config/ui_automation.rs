use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UiAutomationConfig {
    pub collect_interval: u64,
    pub cache_ttl: u64,
}
