pub mod hint;
pub mod keybinding;
pub mod keyboard;
pub mod mouse;
pub mod system;
pub mod ui_automation;

pub use hint::{get_hint_styles, HintConfig};
pub use keybinding::KeybindingConfig;
pub use keyboard::KeyboardConfig;
use log::{error, info};
pub use mouse::MouseConfig;
pub use system::SystemConfig;
pub use ui_automation::UiAutomationConfig;

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

pub fn get_config_path() -> Option<String> {
    let config_paths = vec!["config.toml", "src-tauri/config.toml", "../config.toml"];
    for path in config_paths {
        if Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    None
}

pub fn load_config() -> Config {
    if let Some(path) = get_config_path() {
        let config_str = fs::read_to_string(&path)
            .expect(format!("[load_config] Failed to read config file: {}", path).as_str());
        let config: Config = toml::from_str(&config_str)
            .expect(format!("[load_config] Failed to parse config file: {}", path).as_str());
        info!("[load_config] load config from{} : {:?}", path, config);
        return config;
    }
    panic!("please check the config file: config.toml exists");
}

// 全局配置实例
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));

// 初始化配置
pub fn init_config() -> Config {
    let config = load_config();
    let mut config_guard = CONFIG.lock().unwrap();
    *config_guard = Some(config.clone());
    config
}

// 获取配置
pub fn get_config() -> Option<Config> {
    CONFIG.lock().unwrap().clone()
}

// 为前端提供的配置获取命令
#[tauri::command]
pub fn get_config_for_frontend() -> Config {
    get_config().unwrap_or_else(|| {
        let config = load_config();
        let mut config_guard = CONFIG.lock().unwrap();
        *config_guard = Some(config.clone());
        config
    })
}

// 为前端提供的配置保存命令
#[tauri::command]
pub fn save_config_for_frontend(config: Config) {
    // 重排序 keyboard.available_key
    let mut config = config;
    let mut available_keys_vec = config
        .keyboard
        .available_key
        .into_iter()
        .collect::<Vec<_>>();
    available_keys_vec.sort_by_key(|k| k.1);
    config.keyboard.available_key = available_keys_vec.into_iter().collect();

    // 更新内存中的配置
    {
        let mut config_guard = CONFIG.lock().unwrap();
        *config_guard = Some(config.clone());
    }

    // 获取当前配置文件路径，如果不存在则使用默认路径
    let config_path = get_config_path().unwrap_or_else(|| {
        if cfg!(debug_assertions) {
            "src-tauri/config.toml".to_string()
        } else {
            "config.toml".to_string()
        }
    });

    // 确保目标目录存在
    if let Some(parent) = Path::new(&config_path).parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                error!(
                    "[save_config_for_frontend] Failed to create config directory: {}",
                    e
                );
            }
        }
    }

    match toml::to_string_pretty(&config) {
        Ok(config_str) => {
            if let Err(e) = fs::write(&config_path, config_str) {
                error!(
                    "[save_config_for_frontend] Failed to write config file: {}",
                    e
                );
            }
        }
        Err(e) => {
            error!(
                "[save_config_for_frontend] Failed to serialize config: {}",
                e
            );
        }
    }
}
