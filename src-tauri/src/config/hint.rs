use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct HintConfig {
    pub charsets: Vec<Vec<char>>,
    pub charset_extra: Vec<char>,
}

pub static HAS_EXTRA_CHARSET: Lazy<bool> = Lazy::new(|| {
    let config = super::get_config().unwrap().hint;
    !config.charset_extra.is_empty()
});
