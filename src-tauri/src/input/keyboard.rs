use crate::config;
use crate::hint::{filter_hints, hide_hints};
use crate::input::{executor, mouse};
use log::{debug, error, info};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

// 键盘状态
#[derive(Debug, Default)]
pub struct KeyboardState {
    pub pressed_hint_keys: Option<String>,
    pub final_hint_key: Option<String>,
    pub hint_starts_with_extra: bool,
    pub hint_length: usize,
    pub final_hint_key_hold: bool,
    pub final_hint_key_hold_start: u64,
    pub in_ctrl_session: bool,
    pub is_dragging: bool,
    pub hold_keys: HashMap<String, bool>,
    pub double_click_key_hold: bool,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            pressed_hint_keys: Some("".to_string()),
            final_hint_key: Some("".to_string()),
            hint_starts_with_extra: false,
            hint_length: 0,
            final_hint_key_hold: false,
            final_hint_key_hold_start: 0,
            in_ctrl_session: false,
            is_dragging: false,
            hold_keys: HashMap::<String, bool>::new(),
            double_click_key_hold: false,
        }
    }
}

// 全局键盘状态
pub static KEYBOARD_STATE: Lazy<Mutex<KeyboardState>> =
    Lazy::new(|| Mutex::new(KeyboardState::new()));

pub fn switch_keyboard_ctrl(visible: bool, app_handle: Option<&tauri::AppHandle>) {
    if let Ok(mut state) = KEYBOARD_STATE.lock() {
        let old_visible = state.in_ctrl_session;
        state.in_ctrl_session = visible;

        // 如果状态发生变化
        if old_visible != visible {
            if !visible {
                // 重置状态
                debug!("[switch_keyboard_ctrl] reset state");
                state.pressed_hint_keys = Some("".to_string());
                state.final_hint_key = Some("".to_string());
                state.final_hint_key_hold = false;
                state.final_hint_key_hold_start = 0;
                state.is_dragging = false;
                if let Some(app_handle) = app_handle {
                    let app_handle_clone = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        hide_hints(app_handle_clone).await;
                    });
                }
            }
        }
    }
}

// 检查字符是否在指定的字符集中
fn char_in_charset(c: char, charset: &[char]) -> bool {
    charset.contains(&c)
}

fn key_in_keys(key: &str, keys: &Vec<String>) -> bool {
    keys.contains(&key.to_string())
}

fn filter_hints_by_state(state: &mut KeyboardState, app_handle: &tauri::AppHandle) {
    let prefix = state.pressed_hint_keys.clone().unwrap();
    debug!("[filter_hints_by_state] prefix: {}", prefix);
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        filter_hints(app_handle_clone, prefix).await;
    });
}

fn hide_hints_when_session_end(state: &mut KeyboardState, app_handle: &tauri::AppHandle) {
    let app_handle_clone = app_handle.clone();
    let is_dragging = state.is_dragging;
    debug!("[hide_hints_when_session_end] is_dragging: {}", is_dragging);
    tauri::async_runtime::spawn(async move {
        if is_dragging {
            mouse::mouse_drag_end().await;
        }
        hide_hints(app_handle_clone).await;
    });
}

// 处理键盘事件
pub fn handle_keyboard_event(app_handle: &tauri::AppHandle, key: &str, is_down: bool) -> bool {
    let mut state = KEYBOARD_STATE.lock().unwrap();

    // 如果hints不可见，不处理任何按键
    if !state.in_ctrl_session {
        return false;
    }
    info!(
        "[handle_keyboard_event] key: {}:{}, is_holding_at_hint:{}",
        key,
        if is_down { "down" } else { "up" },
        state.final_hint_key_hold
    );

    let configs = config::get_config().unwrap();
    let keybindings = &configs.keybinding;
    let modifier_keys = config::keybinding::MODIFIERS.clone();

    // 记录状态
    if modifier_keys.contains(&key.to_string()) {
        state.hold_keys.insert(key.to_string(), is_down);
    } else {
        if !state.final_hint_key_hold {
            if keybindings.global.is_translate_key(key) {
                state.hold_keys.insert(key.to_string(), is_down);
            }
        } else if state.final_hint_key_hold {
            if keybindings.at_hint.is_translate_key(key) {
                state.hold_keys.insert(key.to_string(), is_down);
            } else if keybindings.at_hint.is_scroll_key(key) {
                state.hold_keys.insert(key.to_string(), is_down);
            } else if keybindings.at_hint.is_drag_key(key) {
                state.hold_keys.insert(key.to_string(), is_down);
            }
        }
    }

    // 处理按键按下
    if is_down {
        let propagation_modifier: &Vec<String> = &configs.keyboard.propagation_modifier;
        for modifier_key in propagation_modifier {
            if let Some(pressed) = state.hold_keys.get(modifier_key) {
                if *pressed {
                    // 如果传播修饰键是按住的状态，则不处理
                    debug!("[handle_keyboard_event] propagation_modifier: {} is_down: {}", modifier_key, is_down);
                    return false;
                }
            }
        }

        let hint_config = &configs.hint;
        let charset_count = hint_config.charsets.len();
        if !state.final_hint_key_hold {
            // 非holding状态, 处理全局热键
            let mut current_key = key;
            let mut no_propagation = false;
            if key.len() == 1 {
                let key_char = key.chars().next().unwrap();
                // 可能是字符, 读取为hint charsets
                if state.pressed_hint_keys.clone().unwrap().is_empty() {
                    // 读取首位hint char
                    let start_by_extra = hint_config.charset_extra.contains(&key_char);
                    let mut start_by_charset = false;
                    if start_by_extra {
                        state.hint_starts_with_extra = true;
                        state.hint_length = 1 + charset_count as usize;
                    } else {
                        start_by_charset =
                            char_in_charset(key_char, hint_config.charsets[0].as_slice());
                        if start_by_charset {
                            state.hint_starts_with_extra = false;
                            state.hint_length = charset_count as usize;
                        }
                    }
                    if start_by_extra || start_by_charset {
                        // 读取到首字母
                        debug!("[handle_keyboard_event] read first hint char: {}, start_by_extra: {}, start_by_charset: {}, hint_length: {}",
                            key_char, start_by_extra, start_by_charset, state.hint_length);
                        state.pressed_hint_keys = Some(key.to_string());
                        if state.hint_length == 1 {
                            // hint一共只有一位
                            debug!("[handle_keyboard_event] hint_length == 1");
                            state.final_hint_key = Some(key.to_string());
                            current_key = config::keyboard::HINT_KEY;
                        }
                        filter_hints_by_state(&mut state, app_handle);
                        no_propagation = true;
                    }
                } else {
                    // 读取中间位hint char
                    let pressed_keys = state.pressed_hint_keys.clone().unwrap();
                    let current_length = pressed_keys.len();
                    let charset_index = if state.hint_starts_with_extra {
                        current_length - 1
                    } else {
                        current_length
                    };
                    if char_in_charset(key_char, hint_config.charsets[charset_index].as_slice()) {
                        // 读取到中间字母
                        let mut new_prefix = pressed_keys.clone();
                        new_prefix.push_str(key);
                        state.pressed_hint_keys = Some(new_prefix.clone());
                        if new_prefix.len() == state.hint_length {
                            // 达到完整长度
                            debug!("[handle_keyboard_event] reach hint_length: {} with prefix:{}, key:{}", state.hint_length, new_prefix, key);
                            state.final_hint_key = Some(key.to_string());
                            current_key = config::keyboard::HINT_KEY;
                        }
                        filter_hints_by_state(&mut state, app_handle);
                        no_propagation = true;
                    }
                }
            }
            let cmd_key = config::keybinding::GLOBAL_KEY_DOWN_KEYBINDINGS.clone();
            for (cmd, keys) in cmd_key {
                if key_in_keys(current_key, &keys) {
                    info!(
                        "[handle_keyboard_event] global_key_down cmd:{} triggered by key: {}",
                        cmd, key
                    );
                    let mut executor = executor::Executor::new(app_handle, &configs, &mut state);
                    no_propagation = executor.execute(Some(cmd));
                }
            }
            if no_propagation {
                return true;
            }
        }

        if state.final_hint_key_hold {
            // holding状态, 处理按住final_key后的操作
            let cmd_key = config::keybinding::AT_HINT_KEYBINDINGS.clone();
            let mut current_key = key;
            if let Some(last_key) = state.final_hint_key.as_ref() {
                // 先处理动态热键, 动态热键会覆盖配置的静态热键
                if current_key == last_key {
                    // 是当前holding的末尾hint键, 不传递
                    if state.hold_keys.contains_key(key) {
                        state.hold_keys.remove(key);
                    }
                    return true;
                } else if config::keyboard::is_right_key_of(key, last_key) {
                    current_key = config::keyboard::HINT_RIGHT_KEY;
                    debug!("[handle_keyboard_event] current_key: {} is_right_key_of: {}", current_key, last_key);
                    if state.hold_keys.contains_key(key) {
                        state.hold_keys.remove(key);
                    }
                } else if config::keyboard::is_left_key_of(key, last_key) {
                    current_key = config::keyboard::HINT_LEFT_KEY;
                    debug!("[handle_keyboard_event] current_key: {} is_left_key_of: {}", current_key, last_key);
                    if state.hold_keys.contains_key(key) {
                        state.hold_keys.remove(key);
                    }
                }
            }
            for (cmd, keys) in cmd_key {
                if key_in_keys(current_key, &keys) {
                    info!(
                        "[handle_keyboard_event] at_hint cmd:{} triggered by key: {}",
                        cmd, current_key
                    );
                    let mut executor = executor::Executor::new(app_handle, &configs, &mut state);
                    if executor.execute_at_hint(Some(cmd)) {
                        return true;
                    }
                }
            }
        }
    } else {
        // 处理按键释放
        let mut current_key = key;
        let mut end_session = false;
        if let Some(final_key) = state.final_hint_key.as_ref() {
            if key == final_key {
                current_key = config::keyboard::HINT_KEY;
            }
        }

        let cmd_key = config::keybinding::GLOBAL_KEY_UP_KEYBINDINGS.clone();
        for (cmd, keys) in cmd_key {
            if key_in_keys(current_key, &keys) {
                info!(
                    "[handle_keyboard_event] global_key_up cmd:{} triggered by key: {}",
                    cmd, current_key
                );
                let mut executor = executor::Executor::new(app_handle, &configs, &mut state);
                end_session = executor.execute(Some(cmd));
            }
        }

        if end_session {
            hide_hints_when_session_end(&mut state, app_handle);
            return true;
        }
    }

    false
}
