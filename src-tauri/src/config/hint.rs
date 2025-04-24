use indexmap::IndexMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HintConfig {
    pub charsets: Vec<Vec<char>>,
    pub charset_extra: Vec<char>,
    pub types: IndexMap<String, HintType>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HintType {
    pub style: String,
    pub z_index: i32,
}

pub const HINT_TYPE_DEFAULT_NAME: &str = "default";
pub const HINT_TYPE_TEXT_NAME: &str = "text";
pub const HINT_TYPE_WINDOW_NAME: &str = "window";
pub const HINT_TYPE_PANE_NAME: &str = "pane";
pub const HINT_TYPE_TAB_NAME: &str = "tab";
pub const HINT_TYPE_BUTTON_NAME: &str = "button";
pub const HINT_TYPE_SCROLLBAR_NAME: &str = "scrollbar";

pub static HAS_EXTRA_CHARSET: Lazy<bool> = Lazy::new(|| {
    let config = super::get_config().unwrap().hint;
    !config.charset_extra.is_empty()
});

pub fn get_hint_type_index(hint_type_name: &str) -> usize {
    super::get_config()
        .unwrap()
        .hint
        .types
        .iter()
        .position(|(name, _)| name == hint_type_name)
        .unwrap()
}

#[tauri::command]
pub async fn get_hint_styles(
    state: tauri::State<'_, crate::config::Config>,
) -> Result<Vec<String>, String> {
    let mut styles: Vec<String> = Vec::new();
    for (_, hint_type) in state.hint.types.iter() {
        styles.push(hint_type.style.clone());
    }
    Ok(styles)
}
