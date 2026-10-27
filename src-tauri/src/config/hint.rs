use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;
use log::info;
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

impl HintConfig {
    /// The input handler uses the first key to choose the hint length.
    /// Extra prefixes may share later positions, but never the first one.
    pub fn prefix_conflicts(&self) -> Vec<char> {
        let first: HashSet<char> = self.charsets.first().into_iter().flatten()
            .map(|character| character.to_ascii_uppercase()).collect();
        let mut seen = HashSet::new();
        self.charset_extra.iter().map(|character| character.to_ascii_uppercase())
            .filter(|character| first.contains(character) && seen.insert(*character))
            .collect()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HintType {
    pub style: String,
    pub z_index: i32,
    #[serde(default)]
    pub element_control_types: Vec<i32>,
    /// macOS accessibility roles. None is a legacy config; Some([]) disables
    /// native element matching for this style (e.g. the grid-only style).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_ax_roles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GridConfig {
    pub rows: i32,
    pub columns: i32,
    pub show_at_rows: Vec<i32>,
    pub show_at_columns: Vec<i32>,
    pub hint_type: String,
}

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
    let style = state.hint.style.clone();
    info!("[get_hint_default_style] style: {:?}", style);
    Ok(style)
}

#[tauri::command]
pub async fn get_hint_types_styles(
    state: tauri::State<'_, crate::config::Config>,
) -> Result<Vec<String>, String> {
    let mut styles: Vec<String> = Vec::new();
    for (_, hint_type) in state.hint.types.iter() {
        styles.push(hint_type.style.clone());
    }
    info!("[get_hint_types_styles] styles: {:?}", styles);
    Ok(styles)
}
