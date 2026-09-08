pub mod hint;
pub mod keybinding;
pub mod keyboard;
pub mod mouse;
pub mod system;
pub mod ui_automation;

pub use hint::{get_hint_types_styles, HintConfig};
pub use keybinding::KeybindingConfig;
pub use keyboard::KeyboardConfig;
use log::{debug, error, info};
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
    #[cfg(all(target_os = "macos", not(debug_assertions)))]
    {
        let user_path = macos_user_config_path().expect("Cannot locate macOS user configuration");
        if user_path.is_file() {
            return Some(user_path.to_string_lossy().into_owned());
        }
    }
    let config_filenames = if cfg!(target_os = "macos") {
        vec!["config_macos.toml", "config.toml"]
    } else {
        vec!["config.toml"]
    };

    // First, try paths relative to the executable (needed for macOS .app bundles
    // where the working directory is not the app's location)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            for filename in &config_filenames {
                // For macOS .app: executable is at screen-buoy.app/Contents/MacOS/screen-buoy
                // config is next to the .app bundle, so check ../../.. relative to exe
                let candidates = vec![
                    exe_dir.join(filename),              // next to exe
                    exe_dir.join(format!("../../../{}", filename)), // next to .app bundle (macOS)
                ];
                for candidate in candidates {
                    if let Ok(canonical) = candidate.canonicalize() {
                        return Some(canonical.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    // Fallback: relative to working directory (dev mode / Windows)
    for filename in &config_filenames {
        let paths = vec![
            filename.to_string(),
            format!("src-tauri/{}", filename),
            format!("../{}", filename),
        ];
        for path in paths {
            if Path::new(&path).exists() {
                return Some(path);
            }
        }
    }
    None
}

pub fn load_config() -> Config {
    #[cfg(all(target_os = "macos", not(debug_assertions)))]
    {
        let destination = macos_user_config_path().expect("Cannot locate macOS user configuration");
        // Migrate a sidecar config once. A standalone .app gets embedded macOS
        // defaults; subsequent upgrades preserve the user's writable copy.
        let legacy = get_config_path();
        initialize_user_config(&destination, legacy.as_deref().map(Path::new))
            .expect("Could not initialize macOS configuration");
    }
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

#[cfg(all(target_os = "macos", not(debug_assertions)))]
fn macos_user_config_path() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(|home| {
        std::path::PathBuf::from(home)
            .join("Library/Application Support/com.screen-buoy.dev/config_macos.toml")
    })
}

#[cfg(all(target_os = "macos", any(not(debug_assertions), test)))]
fn initialize_user_config(destination: &Path, legacy: Option<&Path>) -> std::io::Result<()> {
    use std::io::Write;
    if destination.is_file() {
        return Ok(());
    }
    let contents = match legacy {
        Some(path) => fs::read_to_string(path)?,
        None => include_str!("../../config_macos.toml").to_owned(),
    };
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    match fs::OpenOptions::new().write(true).create_new(true).open(destination) {
        Ok(mut file) => file.write_all(contents.as_bytes()),
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(err) => Err(err),
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    struct ConfigDir(std::path::PathBuf);
    impl ConfigDir {
        fn new() -> Self {
            let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
            Self(std::env::temp_dir().join(format!("screen-buoy-config-{}-{unique}", std::process::id())))
        }
    }
    impl Drop for ConfigDir {
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
    }

    #[test]
    fn standalone_app_creates_valid_macos_defaults() {
        let dir = ConfigDir::new();
        let destination = dir.0.join("settings/config_macos.toml");
        initialize_user_config(&destination, None).unwrap();
        let config: Config = toml::from_str(&fs::read_to_string(destination).unwrap()).unwrap();
        assert_eq!(config.keyboard.available_key["LCmd"], 55);
        assert_eq!(config.keybinding.hotkey_buoy, "Alt+H");
    }

    #[test]
    fn migration_preserves_legacy_config_and_never_overwrites_user_edits() {
        let dir = ConfigDir::new();
        fs::create_dir_all(&dir.0).unwrap();
        let legacy = dir.0.join("legacy.toml");
        let destination = dir.0.join("settings/config_macos.toml");
        fs::write(&legacy, "legacy preferences").unwrap();
        initialize_user_config(&destination, Some(&legacy)).unwrap();
        assert_eq!(fs::read_to_string(&destination).unwrap(), "legacy preferences");
        fs::write(&destination, "user edits").unwrap();
        initialize_user_config(&destination, Some(&legacy)).unwrap();
        assert_eq!(fs::read_to_string(&destination).unwrap(), "user edits");
        assert_eq!(fs::read_to_string(&legacy).unwrap(), "legacy preferences");
    }
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
    let mut config = config;
    debug!("[save_config_for_frontend] save config: {:?}", config);

    // 更新内存中的配置
    {
        let mut config_guard = CONFIG.lock().unwrap();
        *config_guard = Some(config.clone());
    }

    // 重排序 keyboard.available_key
    let mut available_keys_vec = config
        .keyboard
        .available_key
        .into_iter()
        .collect::<Vec<_>>();
    available_keys_vec.sort_by_key(|k| k.1);
    config.keyboard.available_key = available_keys_vec.into_iter().collect();

    // 获取当前配置文件路径，如果不存在则使用默认路径
    let config_path = get_config_path().unwrap_or_else(|| {
        if cfg!(debug_assertions) {
            "src-tauri/config.toml".to_string()
        } else {
            // Try to place config next to the executable
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("config.toml").to_string_lossy().to_string()))
                .unwrap_or_else(|| "config.toml".to_string())
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
