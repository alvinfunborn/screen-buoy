use crate::config;
use crate::hint::{filter_hints, hide_hints};
use crate::input::{executor, mouse};
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

// 键盘映射表
static KEY_RIGHT_SIDE_MAP: Lazy<std::collections::HashMap<&str, &str>> = Lazy::new(|| {
    let mut map = std::collections::HashMap::new();
    map.insert("A", "S");
    map.insert("B", "N");
    map.insert("C", "V");
    map.insert("D", "F");
    map.insert("E", "R");
    map.insert("F", "G");
    map.insert("G", "H");
    map.insert("H", "J");
    map.insert("I", "O");
    map.insert("J", "K");
    map.insert("K", "L");
    map.insert("L", ";");
    map.insert("M", ",");
    map.insert("N", "M");
    map.insert("O", "P");
    map.insert("P", "[");
    map.insert("Q", "W");
    map.insert("R", "T");
    map.insert("S", "D");
    map.insert("T", "Y");
    map.insert("U", "I");
    map.insert("V", "B");
    map.insert("W", "E");
    map.insert("X", "C");
    map.insert("Y", "U");
    map.insert("Z", "X");
    map.insert("1", "2");
    map.insert("2", "3");
    map.insert("3", "4");
    map.insert("4", "5");
    map.insert("5", "6");
    map.insert("6", "7");
    map.insert("7", "8");
    map.insert("8", "9");
    map.insert("9", "0");
    map.insert("0", "-");
    map
});

static KEY_LEFT_SIDE_MAP: Lazy<std::collections::HashMap<&str, &str>> = Lazy::new(|| {
    let mut map = std::collections::HashMap::new();
    map.insert("W", "Q");
    map.insert("E", "W");
    map.insert("R", "E");
    map.insert("T", "R");
    map.insert("Y", "T");
    map.insert("U", "Y");
    map.insert("I", "U");
    map.insert("O", "I");
    map.insert("P", "O");
    map.insert("[", "P");
    map.insert("S", "A");
    map.insert("D", "S");
    map.insert("F", "D");
    map.insert("G", "F");
    map.insert("H", "G");
    map.insert("J", "H");
    map.insert("K", "J");
    map.insert("L", "K");
    map.insert(";", "L");
    map.insert("X", "Z");
    map.insert("C", "X");
    map.insert("V", "C");
    map.insert("B", "V");
    map.insert("N", "M");
    map.insert("M", ",");
    map.insert("1", "`");
    map.insert("2", "1");
    map.insert("3", "2");
    map.insert("4", "3");
    map.insert("5", "4");
    map.insert("6", "5");
    map.insert("7", "6");
    map.insert("8", "7");
    map.insert("9", "8");
    map.insert("0", "9");
    map.insert("-", "0");
    map
});

pub fn switch_keyboard_ctrl(visible: bool, app_handle: Option<&tauri::AppHandle>) {
    println!("[键盘状态] 设置 keyboard_ctrl = {}", visible);
    if let Ok(mut state) = KEYBOARD_STATE.lock() {
        let old_visible = state.in_ctrl_session;
        state.in_ctrl_session = visible;

        // 如果状态发生变化
        if old_visible != visible {
            println!("[键盘状态] 状态从 {} 变为 {}", old_visible, visible);
            if !visible {
                // 重置状态
                state.pressed_hint_keys = Some("".to_string());
                state.final_hint_key = Some("".to_string());
                state.final_hint_key_hold = false;
                state.final_hint_key_hold_start = 0;
                state.is_dragging = false;
                println!("[键盘状态] 重置键盘状态");
                if let Some(app_handle) = app_handle {
                    let app_handle_clone = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = hide_hints(app_handle_clone).await {
                            eprintln!("[键盘状态] hide_hints失败: {}", e);
                        }
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

// 检查是否是末尾hint键的右侧键
fn is_right_key_of(key: &str, last_key: &str) -> bool {
    KEY_RIGHT_SIDE_MAP
        .get(last_key)
        .map_or(false, |&right_key| right_key == key)
}

// 检查是否是末尾hint键的左侧键
fn is_left_key_of(key: &str, last_key: &str) -> bool {
    KEY_LEFT_SIDE_MAP
        .get(last_key)
        .map_or(false, |&left_key| left_key == key)
}

fn filter_hints_by_state(state: &mut KeyboardState, app_handle: &tauri::AppHandle) {
    let prefix = state.pressed_hint_keys.clone().unwrap();
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = filter_hints(app_handle_clone, prefix).await {
            eprintln!("[键盘事件] filter-hints失败: {}", e);
        }
    });
}

fn hide_hints_when_session_end(state: &mut KeyboardState, app_handle: &tauri::AppHandle) {
    let app_handle_clone = app_handle.clone();
    let is_dragging = state.is_dragging;
    tauri::async_runtime::spawn(async move {
        if is_dragging {
            if let Err(e) = mouse::mouse_drag_end().await {
                eprintln!("[键盘事件] mouse_drag_end失败: {}", e);
            }
        }
        if let Err(e) = hide_hints(app_handle_clone).await {
            eprintln!("[键盘事件] hide_hints失败: {}", e);
        }
    });
}

// 处理键盘事件
pub fn handle_keyboard_event(app_handle: &tauri::AppHandle, key: &str, is_down: bool) -> bool {
    let mut state = KEYBOARD_STATE.lock().unwrap();

    // 如果hints不可见，不处理任何按键
    if !state.in_ctrl_session {
        return false;
    }
    println!(
        "[键盘事件] 按键: {}, 状态: {}, is_holding:{}",
        key,
        if is_down { "按下" } else { "释放" },
        state.final_hint_key_hold
    );

    let configs = config::get_config().expect("配置未初始化");
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
                    println!("[键盘事件] 传播修饰键: {}", modifier_key);
                    // 如果传播修饰键是按住的状态，则不处理
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
                        state.pressed_hint_keys = Some(key.to_string());
                        if state.hint_length == 1 {
                            // hint一共只有一位
                            state.final_hint_key = Some(key.to_string());
                            current_key = configs
                                .keyboard
                                .get_key_by_name(config::keyboard::HINT_KEY_NAME)
                                .unwrap();
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
                            state.final_hint_key = Some(key.to_string());
                            current_key = configs
                                .keyboard
                                .get_key_by_name(config::keyboard::HINT_KEY_NAME)
                                .unwrap();
                        }
                        filter_hints_by_state(&mut state, app_handle);
                        no_propagation = true;
                    }
                }
            }
            let cmd_key = config::keybinding::GLOBAL_KEY_DOWN_KEYBINDINGS.clone();
            for (cmd, keys) in cmd_key {
                if key_in_keys(current_key, &keys) {
                    println!(
                        "[keyboard] global_key_down cmd:{} triggered by key: {}",
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
                    return true;
                } else if is_right_key_of(key, last_key) {
                    current_key = configs
                        .keyboard
                        .get_key_by_name(config::keyboard::HINT_RIGHT_KEY_NAME)
                        .unwrap();
                } else if is_left_key_of(key, last_key) {
                    current_key = configs
                        .keyboard
                        .get_key_by_name(config::keyboard::HINT_LEFT_KEY_NAME)
                        .unwrap();
                }
            }
            for (cmd, keys) in cmd_key {
                if key_in_keys(current_key, &keys) {
                    println!(
                        "[keyboard] at_hint cmd:{} triggered by key: {}",
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
                current_key = configs
                    .keyboard
                    .get_key_by_name(config::keyboard::HINT_KEY_NAME)
                    .unwrap();
            }
        }

        let cmd_key = config::keybinding::GLOBAL_KEY_UP_KEYBINDINGS.clone();
        for (cmd, keys) in cmd_key {
            if key_in_keys(current_key, &keys) {
                println!(
                    "[keyboard] global_key_up cmd:{} triggered by key: {}",
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
