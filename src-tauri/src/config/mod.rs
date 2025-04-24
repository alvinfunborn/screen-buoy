pub mod hint;
pub mod keybinding;
pub mod keyboard;
pub mod mouse;
pub mod system;
pub mod ui_automation;

pub use hint::HintConfig;
pub use keybinding::KeybindingConfig;
pub use keyboard::KeyboardConfig;
pub use mouse::MouseConfig;
pub use system::SystemConfig;
pub use ui_automation::UiAutomationConfig;
pub use hint::{HINT_TYPE_DEFAULT_NAME, HINT_TYPE_TEXT_NAME, HINT_TYPE_WINDOW_NAME, HINT_TYPE_PANE_NAME, HINT_TYPE_TAB_NAME, HINT_TYPE_BUTTON_NAME, HINT_TYPE_SCROLLBAR_NAME};

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use toml;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub hint: HintConfig,
    pub keybinding: KeybindingConfig,
    pub mouse: MouseConfig,
    pub keyboard: KeyboardConfig,
    pub system: SystemConfig,
    pub ui_automation: UiAutomationConfig,
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_paths = vec!["config.toml", "src-tauri/config.toml", "../config.toml"];

    for path in config_paths {
        if Path::new(path).exists() {
            let config_str = fs::read_to_string(path)?;
            let config: Config = toml::from_str(&config_str)?;
            println!("[配置] 从 {} 加载配置成功: {:?}", path, config);
            return Ok(config);
        }
    }

    Err("未找到配置文件".into())
}

// 全局配置实例
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));

// 初始化配置
pub fn init_config() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    let mut config_guard = CONFIG.lock().unwrap();
    *config_guard = Some(config);
    Ok(())
}

// 获取配置
pub fn get_config() -> Option<Config> {
    CONFIG.lock().unwrap().clone()
}

// 为前端提供的配置获取命令
#[tauri::command]
pub fn get_config_for_frontend() -> Config {
    get_config().unwrap_or_else(|| {
        let config = load_config().expect("Failed to load config");
        let mut config_guard = CONFIG.lock().unwrap();
        *config_guard = Some(config.clone());
        config
    })
}

// 为前端提供的配置保存命令
#[tauri::command]
pub fn save_config_for_frontend(config: Config) -> Result<(), String> {
    // 更新内存中的配置
    {
        let mut config_guard = CONFIG.lock().unwrap();
        *config_guard = Some(config.clone());
    }

    // 保存到文件
    let config_str = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    let config_path = "src-tauri/config.toml";
    fs::write(config_path, config_str)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}
