use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KeybindingConfig {
    pub global: GlobalKeybindingConfig,
    pub at_hint: AtHintKeybindingConfig,
    pub hotkey_buoy: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GlobalKeybindingConfig {
    pub move_to_hint: Vec<String>,
    pub move_to_hint_exit: Vec<String>,
    pub left_click_exit: Vec<String>,
    pub hold_at_hint: Vec<String>,
    pub exit: Vec<String>,
    pub translate: DirectionKeybindingsConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AtHintKeybindingConfig {
    pub exit: Vec<String>,
    pub left_click: Vec<String>,
    pub left_click_exit: Vec<String>,
    pub double_click: Vec<String>,
    pub double_click_exit: Vec<String>,
    pub right_click: Vec<String>,
    pub right_click_exit: Vec<String>,
    pub middle_click: Vec<String>,
    pub middle_click_exit: Vec<String>,
    pub translate: DirectionKeybindingsConfig,
    pub drag: DirectionKeybindingsConfig,
    pub scroll: DirectionKeybindingsConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DirectionKeybindingsConfig {
    pub up: Vec<String>,
    pub down: Vec<String>,
    pub left: Vec<String>,
    pub right: Vec<String>,
}

pub const MOVE_TO_HINT_CMD: &str = "move_to_hint";
pub const MOVE_TO_HINT_EXIT_CMD: &str = "move_to_hint_exit";
pub const HOLD_AT_HINT_CMD: &str = "hold_at_hint";
pub const LEFT_CLICK_CMD: &str = "left_click";
pub const LEFT_CLICK_EXIT_CMD: &str = "left_click_exit";
pub const DOUBLE_CLICK_CMD: &str = "double_click";
pub const DOUBLE_CLICK_EXIT_CMD: &str = "double_click_exit";
pub const RIGHT_CLICK_CMD: &str = "right_click";
pub const RIGHT_CLICK_EXIT_CMD: &str = "right_click_exit";
pub const MIDDLE_CLICK_CMD: &str = "middle_click";
pub const MIDDLE_CLICK_EXIT_CMD: &str = "middle_click_exit";
pub const EXIT_CMD: &str = "exit";
pub const TRANSLATE_UP_CMD: &str = "translate_up";
pub const TRANSLATE_DOWN_CMD: &str = "translate_down";
pub const TRANSLATE_LEFT_CMD: &str = "translate_left";
pub const TRANSLATE_RIGHT_CMD: &str = "translate_right";
pub const SCROLL_UP_CMD: &str = "scroll_up";
pub const SCROLL_DOWN_CMD: &str = "scroll_down";
pub const SCROLL_LEFT_CMD: &str = "scroll_left";
pub const SCROLL_RIGHT_CMD: &str = "scroll_right";
pub const DRAG_UP_CMD: &str = "drag_up";
pub const DRAG_DOWN_CMD: &str = "drag_down";
pub const DRAG_LEFT_CMD: &str = "drag_left";
pub const DRAG_RIGHT_CMD: &str = "drag_right";

pub static MODIFIERS: Lazy<HashSet<String>> = Lazy::new(|| {
    let mouse_config = super::get_config().unwrap().mouse;
    let mut modifiers = mouse_config.get_modifiers();
    let keyboard = super::get_config().unwrap().keyboard;
    modifiers.extend(keyboard.propagation_modifier.iter().map(|s| s.to_string()));
    modifiers
});

pub static GLOBAL_KEY_DOWN_KEYBINDINGS: Lazy<HashMap<&'static str, Vec<String>>> =
    Lazy::new(|| {
        let keybindings = super::get_config().unwrap().keybinding;
        keybindings.get_global_keybindings(true)
    });

pub static GLOBAL_KEY_UP_KEYBINDINGS: Lazy<HashMap<&'static str, Vec<String>>> = Lazy::new(|| {
    let keybindings = super::get_config().unwrap().keybinding;
    keybindings.get_global_keybindings(false)
});

pub static AT_HINT_KEYBINDINGS: Lazy<HashMap<&'static str, Vec<String>>> = Lazy::new(|| {
    let keybindings = super::get_config().unwrap().keybinding;
    keybindings.get_at_hint_keybindings()
});

impl KeybindingConfig {
    fn get_global_keybindings(&self, key_down: bool) -> HashMap<&'static str, Vec<String>> {
        let mut keybindings = HashMap::new();
        if !key_down {
            keybindings.insert(HOLD_AT_HINT_CMD, self.global.hold_at_hint.clone());
            keybindings.insert(LEFT_CLICK_EXIT_CMD, self.global.left_click_exit.clone());
        } else {
            keybindings.insert(EXIT_CMD, self.global.exit.clone());
            keybindings.insert(HOLD_AT_HINT_CMD, self.global.hold_at_hint.clone());
            keybindings.insert(MOVE_TO_HINT_CMD, self.global.move_to_hint.clone());
            keybindings.insert(MOVE_TO_HINT_EXIT_CMD, self.global.move_to_hint_exit.clone());
            keybindings.insert(TRANSLATE_UP_CMD, self.global.translate.up.clone());
            keybindings.insert(TRANSLATE_DOWN_CMD, self.global.translate.down.clone());
            keybindings.insert(TRANSLATE_LEFT_CMD, self.global.translate.left.clone());
            keybindings.insert(TRANSLATE_RIGHT_CMD, self.global.translate.right.clone());
        }
        keybindings
    }

    fn get_at_hint_keybindings(&self) -> HashMap<&'static str, Vec<String>> {
        let mut keybindings = HashMap::new();
        keybindings.insert(HOLD_AT_HINT_CMD, self.global.hold_at_hint.clone());
        keybindings.insert(EXIT_CMD, self.at_hint.exit.clone());
        keybindings.insert(LEFT_CLICK_CMD, self.at_hint.left_click.clone());
        keybindings.insert(LEFT_CLICK_EXIT_CMD, self.at_hint.left_click_exit.clone());
        keybindings.insert(DOUBLE_CLICK_CMD, self.at_hint.double_click.clone());
        keybindings.insert(
            DOUBLE_CLICK_EXIT_CMD,
            self.at_hint.double_click_exit.clone(),
        );
        keybindings.insert(RIGHT_CLICK_CMD, self.at_hint.right_click.clone());
        keybindings.insert(RIGHT_CLICK_EXIT_CMD, self.at_hint.right_click_exit.clone());
        keybindings.insert(MIDDLE_CLICK_CMD, self.at_hint.middle_click.clone());
        keybindings.insert(
            MIDDLE_CLICK_EXIT_CMD,
            self.at_hint.middle_click_exit.clone(),
        );
        keybindings.insert(TRANSLATE_UP_CMD, self.at_hint.translate.up.clone());
        keybindings.insert(TRANSLATE_DOWN_CMD, self.at_hint.translate.down.clone());
        keybindings.insert(TRANSLATE_LEFT_CMD, self.at_hint.translate.left.clone());
        keybindings.insert(TRANSLATE_RIGHT_CMD, self.at_hint.translate.right.clone());
        keybindings.insert(DRAG_UP_CMD, self.at_hint.drag.up.clone());
        keybindings.insert(DRAG_DOWN_CMD, self.at_hint.drag.down.clone());
        keybindings.insert(DRAG_LEFT_CMD, self.at_hint.drag.left.clone());
        keybindings.insert(DRAG_RIGHT_CMD, self.at_hint.drag.right.clone());
        keybindings.insert(SCROLL_UP_CMD, self.at_hint.scroll.up.clone());
        keybindings.insert(SCROLL_DOWN_CMD, self.at_hint.scroll.down.clone());
        keybindings.insert(SCROLL_LEFT_CMD, self.at_hint.scroll.left.clone());
        keybindings.insert(SCROLL_RIGHT_CMD, self.at_hint.scroll.right.clone());
        keybindings
    }
}

impl GlobalKeybindingConfig {
    pub fn is_translate_key(&self, key: &str) -> bool {
        self.translate.up.contains(&key.to_string())
            || self.translate.down.contains(&key.to_string())
            || self.translate.left.contains(&key.to_string())
            || self.translate.right.contains(&key.to_string())
    }
}

impl AtHintKeybindingConfig {
    pub fn is_translate_key(&self, key: &str) -> bool {
        self.translate.up.contains(&key.to_string())
            || self.translate.down.contains(&key.to_string())
            || self.translate.left.contains(&key.to_string())
            || self.translate.right.contains(&key.to_string())
    }

    pub fn is_drag_key(&self, key: &str) -> bool {
        self.drag.up.contains(&key.to_string())
            || self.drag.down.contains(&key.to_string())
            || self.drag.left.contains(&key.to_string())
            || self.drag.right.contains(&key.to_string())
    }

    pub fn is_scroll_key(&self, key: &str) -> bool {
        self.scroll.up.contains(&key.to_string())
            || self.scroll.down.contains(&key.to_string())
            || self.scroll.left.contains(&key.to_string())
            || self.scroll.right.contains(&key.to_string())
    }
}
