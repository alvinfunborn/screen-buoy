use std::collections::HashMap;

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HintConfig {
    pub charsets: Vec<Vec<char>>,
    pub charset_extra: Vec<char>,
    pub style: String,
    pub types: IndexMap<String, HintType>,
    pub grid: GridConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HintType {
    pub style: String,
    pub z_index: i32,
    pub element_control_types: Vec<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GridConfig {
    pub rows: i32,
    pub columns: i32,
    pub show_at_rows: Vec<i32>,
    pub show_at_columns: Vec<i32>,
    pub hint_type: String,
}

pub static HAS_EXTRA_CHARSET: Lazy<bool> = Lazy::new(|| {
    let config = super::get_config().unwrap().hint;
    !config.charset_extra.is_empty()
});

pub static HINT_TYPE_ID_MAP: Lazy<HashMap<String, usize>> = Lazy::new(|| {
    let config = super::get_config().unwrap().hint;
    let mut map = HashMap::new();
    for (i, (name, _)) in config.types.iter().enumerate() {
        map.insert(name.clone(), i);
    }
    map
});

pub static HINT_CONTROL_TYPES_ID_Z_MAP: Lazy<IndexMap<i32, (usize, i32)>> = Lazy::new(|| {
    let config = super::get_config().unwrap().hint;
    let mut map = IndexMap::new();
    for (i, (_name, hint_type)) in config.types.iter().enumerate() {
        for control_type in hint_type.element_control_types.iter() {
            map.insert(*control_type, (i, hint_type.z_index));
        }
    }
    map
});

#[tauri::command]
pub async fn get_hint_default_style(
    state: tauri::State<'_, crate::config::Config>,
) -> Result<String, String> {
    Ok(state.hint.style.clone())
}

#[tauri::command]
pub async fn get_hint_types_styles(
    state: tauri::State<'_, crate::config::Config>,
) -> Result<Vec<String>, String> {
    let mut styles: Vec<String> = Vec::new();
    for (_, hint_type) in state.hint.types.iter() {
        styles.push(hint_type.style.clone());
    }
    Ok(styles)
}
