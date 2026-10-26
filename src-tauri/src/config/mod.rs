pub mod hint;
pub mod keybinding;
pub mod keyboard;
pub mod mouse;
pub mod system;
pub mod ui_automation;

pub use hint::{get_hint_types_styles, HintConfig};
pub use keybinding::KeybindingConfig;
pub use keyboard::KeyboardConfig;
use log::{debug, info};
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
    #[cfg(all(any(target_os = "macos", target_os = "windows"), not(debug_assertions)))]
    {
        let user_path = user_config_path().expect("Cannot locate user configuration");
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
    #[cfg(all(any(target_os = "macos", target_os = "windows"), not(debug_assertions)))]
    {
        let destination = user_config_path().expect("Cannot locate user configuration");
        // Migrate a sidecar config once. Standalone apps and Windows installers
        // get embedded defaults; subsequent upgrades preserve the writable copy.
        let legacy = get_config_path();
        initialize_user_config(&destination, legacy.as_deref().map(Path::new))
            .expect("Could not initialize user configuration");
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

#[cfg(all(any(target_os = "macos", target_os = "windows"), not(debug_assertions)))]
fn user_config_path() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "macos")]
    return std::env::var_os("HOME").map(|home| {
        std::path::PathBuf::from(home)
            .join("Library/Application Support/com.screen-buoy.dev/config_macos.toml")
    });
    #[cfg(target_os = "windows")]
    return std::env::var_os("APPDATA").map(|app_data| {
        std::path::PathBuf::from(app_data).join("com.screen-buoy.dev/config.toml")
    });
}

#[cfg(any(not(debug_assertions), test))]
fn initialize_user_config(destination: &Path, legacy: Option<&Path>) -> std::io::Result<()> {
    use std::io::Write;
    if destination.is_file() {
        return Ok(());
    }
    let contents = match legacy {
        Some(path) => fs::read_to_string(path)?,
        None => {
            #[cfg(target_os = "macos")]
            let defaults = include_str!("../../config_macos.toml");
            #[cfg(not(target_os = "macos"))]
            let defaults = include_str!("../../config.toml");
            defaults.to_owned()
        },
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_CONFIG_DIR: AtomicU64 = AtomicU64::new(0);

    struct ConfigDir(std::path::PathBuf);
    impl ConfigDir {
        fn new() -> Self {
            // SystemTime's nanosecond representation does not guarantee a
            // nanosecond clock resolution. Parallel tests can see the same
            // timestamp; reserve each directory atomically before using it.
            loop {
                let unique = NEXT_CONFIG_DIR.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!("screen-buoy-config-{}-{unique}", std::process::id()));
                match fs::create_dir(&path) {
                    Ok(()) => return Self(path),
                    Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(err) => panic!("Cannot create config test directory: {err}"),
                }
            }
        }
    }
    impl Drop for ConfigDir {
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
    }

    #[test]
    fn parallel_config_tests_own_distinct_directories() {
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(16));
        let threads: Vec<_> = (0..16).map(|_| {
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                ConfigDir::new()
            })
        }).collect();
        let dirs: Vec<_> = threads.into_iter().map(|thread| thread.join().unwrap()).collect();
        let paths: std::collections::HashSet<_> = dirs.iter().map(|dir| &dir.0).collect();
        assert_eq!(paths.len(), dirs.len());
        assert!(paths.iter().all(|path| path.is_dir()));
    }

    #[test]
    fn standalone_app_creates_valid_platform_defaults() {
        let dir = ConfigDir::new();
        let destination = dir.0.join("settings/config_macos.toml");
        initialize_user_config(&destination, None).unwrap();
        let config: Config = toml::from_str(&fs::read_to_string(destination).unwrap()).unwrap();
        #[cfg(target_os = "macos")]
        assert_eq!(config.keyboard.available_key["LCmd"], 55);
        assert_eq!(config.ui_automation.collect_interval, 250);
        assert_eq!(config.ui_automation.cache_ttl, 60000);
        assert_eq!(config.system.logging_level, "none");
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

    #[test]
    fn overlapping_hint_prefixes_leave_disk_and_memory_unchanged() {
        let dir = ConfigDir::new();
        let path = dir.0.join("config.toml");
        let original: Config = toml::from_str(include_str!("../../config_macos.toml")).unwrap();
        let before = toml::to_string_pretty(&original).unwrap();
        fs::write(&path, &before).unwrap();
        let mut current = Some(original.clone());
        let mut edited = original;
        edited.hint.charsets = vec![vec!['F', 'D'], vec!['A', 'Z']];
        edited.hint.charset_extra = vec!['d', 'Q', 'F', 'D'];
        edited.hint.style = "unsaved edit".into();
        let error = save_config_to_path(&path, &edited, &mut current).unwrap_err();
        assert_eq!(error, SaveConfigError::HintPrefixOverlap { characters: vec!['D', 'F'] });
        assert_eq!(serde_json::to_value(error).unwrap(), serde_json::json!({
            "code": "hint_prefix_overlap", "characters": ["D", "F"]
        }));
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
        assert_eq!(toml::to_string_pretty(current.as_ref().unwrap()).unwrap(), before);

        // Sharing only the final position is allowed, including edge keys.
        edited.hint.charset_extra = vec!['A', 'Z'];
        save_config_to_path(&path, &edited, &mut current).unwrap();
        let expected = toml::to_string_pretty(&edited).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), expected);
        assert_eq!(toml::to_string_pretty(current.as_ref().unwrap()).unwrap(), expected);
        edited.hint.charset_extra.clear();
        assert!(save_config_to_path(&path, &edited, &mut current).is_ok());
    }

    #[test]
    fn failed_config_write_preserves_in_memory_preferences() {
        let dir = ConfigDir::new();
        let config: Config = toml::from_str(include_str!("../../config_macos.toml")).unwrap();
        let mut current = None;
        assert!(matches!(save_config_to_path(&dir.0, &config, &mut current),
            Err(SaveConfigError::SaveFailed { .. })));
        assert!(current.is_none());
    }

    #[test]
    fn shipped_hint_defaults_have_disjoint_prefixes_and_comfortable_keys() {
        for contents in [include_str!("../../config.default"), include_str!("../../config.toml"),
            include_str!("../../config_macos.toml")] {
            let config: Config = toml::from_str(contents).unwrap();
            assert!(config.hint.prefix_conflicts().is_empty());
            assert_eq!(config.hint.charset_extra, vec!['A', 'Q', 'Z', ';', '-']);
            assert_eq!(config.hint.charsets.iter().map(Vec::len).collect::<Vec<_>>(), vec![28, 28]);
            for character in config.hint.charsets.last().unwrap() {
                assert!(config.keyboard.get_left_key(&character.to_string()).is_some());
                assert!(config.keyboard.get_right_key(&character.to_string()).is_some());
            }
        }
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

#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum SaveConfigError {
    HintPrefixOverlap { characters: Vec<char> },
    SaveFailed { message: String },
}

impl SaveConfigError {
    fn save_failed(error: impl std::fmt::Display) -> Self {
        Self::SaveFailed { message: error.to_string() }
    }
}

fn save_config_to_path(
    path: &Path,
    config: &Config,
    current: &mut Option<Config>,
) -> Result<bool, SaveConfigError> {
    let characters = config.hint.prefix_conflicts();
    if !characters.is_empty() {
        return Err(SaveConfigError::HintPrefixOverlap { characters });
    }
    // Validate before touching either the persisted or in-memory preferences.
    let serialized = toml::to_string_pretty(config).map_err(SaveConfigError::save_failed)?;
    fs::write(path, serialized).map_err(SaveConfigError::save_failed)?;
    let language_changed = current.as_ref().map(|old| &old.system.language) != Some(&config.system.language);
    *current = Some(config.clone());
    Ok(language_changed)
}

// 为前端提供的配置保存命令
#[tauri::command]
pub fn save_config_for_frontend(app: tauri::AppHandle, config: Config) -> Result<(), SaveConfigError> {
    debug!("[save_config_for_frontend] save config: {:?}", config);
    let config_path = get_config_path()
        .ok_or_else(|| SaveConfigError::save_failed("Cannot locate configuration file"))?;
    let language_changed = {
        let mut current = CONFIG.lock().map_err(SaveConfigError::save_failed)?;
        save_config_to_path(Path::new(&config_path), &config, &mut current)?
    };
    if language_changed {
        crate::update_tray_language(&app, &config.system.language);
    }
    Ok(())
}
