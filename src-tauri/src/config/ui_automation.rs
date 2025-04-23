use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct UiAutomationConfig {
    pub collect_interval: u64,
}

