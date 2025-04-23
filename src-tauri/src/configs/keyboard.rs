use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct KeyboardConfig {
    pub available_key: HashMap<String, KeyConfig>,
    pub propagation_modifier: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KeyConfig {
    pub key: String,
    pub virtual_key: Option<u16>,
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

}

