pub mod hint;
pub mod keybinding;
pub mod keyboard;
pub mod mouse;
pub mod system;

pub use hint::HintConfig;
pub use keybinding::KeybindingConfig;
pub use keyboard::KeyboardConfig;
pub use mouse::MouseConfig;
pub use system::SystemConfig;

use serde::Deserialize;
use std::fs;
use std::path::Path;
use toml;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub hint: HintConfig,
    pub keybinding: KeybindingConfig,
    pub mouse: MouseConfig,
    pub keyboard: KeyboardConfig,
    pub system: SystemConfig,
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
