use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SystemConfig {
    pub start_in_tray: bool,
    pub show_tray_icon: bool,
    pub start_at_login: bool,
    pub logging_level: String,
    pub debug_mode: bool,
}
