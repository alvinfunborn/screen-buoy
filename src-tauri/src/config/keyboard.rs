use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct KeyboardConfig {
    pub available_key: HashMap<String, KeyConfig>,
    pub propagation_modifier: Vec<String>,
    pub map_left_right: HashMap<String, LeftRightConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KeyConfig {
    pub key: String,
    pub virtual_key: Option<u16>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LeftRightConfig {
    pub left: Option<String>,
    pub right: Option<String>,
}

pub const HINT_KEY_NAME: &str = "HINT_KEY";
pub const HINT_RIGHT_KEY_NAME: &str = "HINT_RIGHT_KEY";
pub const HINT_LEFT_KEY_NAME: &str = "HINT_LEFT_KEY";

impl KeyboardConfig {
    pub fn get_key_by_virtual_key(&self, virtual_key: u16) -> Option<&KeyConfig> {
        for (_, key) in &self.available_key {
            if key.virtual_key == Some(virtual_key) {
                return Some(key);
            }
        }
        None
    }

    pub fn get_key_by_name(&self, name: &str) -> Option<&str> {
        self.available_key.get(name).map(|key| key.key.as_str())
    }

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
