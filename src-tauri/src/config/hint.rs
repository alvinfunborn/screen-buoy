use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HintConfig {
    pub charsets: Vec<Vec<char>>,
    pub charset_extra: Vec<char>,
    pub styles: HintStyles,
}

pub static HAS_EXTRA_CHARSET: Lazy<bool> = Lazy::new(|| {
    let config = super::get_config().unwrap().hint;
    !config.charset_extra.is_empty()
});

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HintStyle {
    pub background_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HintTypeStyle {
    #[serde(rename = "type")]
    pub type_: u8,
    pub background_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HintStyles {
    pub default: HashMap<String, String>,
    pub types: Vec<HintTypeStyle>,
}

#[tauri::command]
pub async fn get_hint_styles(state: tauri::State<'_, crate::config::Config>) -> Result<HintStyles, String> {
    Ok(state.hint.styles.clone())
} 