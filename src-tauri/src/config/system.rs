use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct SystemConfig {
    pub start_in_tray: bool,
    pub show_tray_icon: bool,
    pub start_at_login: bool,
} 