use std::sync::Mutex;

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KeyboardConfig {
    pub available_key: IndexMap<String, u16>,
    pub propagation_modifier: Vec<String>,
    pub map_left_right: IndexMap<String, LeftRightConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LeftRightConfig {
    pub left: Option<String>,
    pub right: Option<String>,
}

pub const HINT_KEY: &str = "HintKey";
pub const HINT_RIGHT_KEY: &str = "HintRightKey";
pub const HINT_LEFT_KEY: &str = "HintLeftKey";

pub static VIRTUAL_KEY_MAP: Lazy<Mutex<IndexMap<u16, String>>> = Lazy::new(|| Mutex::new({
    let mut map = IndexMap::new();
    for (key, vk) in &super::get_config().unwrap().keyboard.available_key {
        map.insert(*vk, key.clone());
    }
    map
}));

impl KeyboardConfig {
    pub fn get_left_key(&self, key: &str) -> Option<&str> {
        let config = self.map_left_right.get(key)?;
        if let Some(left) = &config.left {
            Some(left)
        } else {
            None
        }
    }

    pub fn get_right_key(&self, key: &str) -> Option<&str> {
        let config = self.map_left_right.get(key)?;
        if let Some(right) = &config.right {
            Some(right)
        } else {
            None
        }
    }
}

// 检查是否是键的右侧键
pub fn is_right_key_of(key: &str, last_key: &str) -> bool {
    let keyboard = &super::get_config().unwrap().keyboard;
    keyboard
        .get_right_key(last_key)
        .map_or(false, |right_key| right_key == key)
}

// 检查是否是键的左侧键
pub fn is_left_key_of(key: &str, last_key: &str) -> bool {
    let keyboard = &super::get_config().unwrap().keyboard;
    keyboard
        .get_left_key(last_key)
        .map_or(false, |left_key| left_key == key)
}
