use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct LanguageInfo {
    code: String,
    tray: [String; 3],
}

// One catalog defines the selectable languages for native menus, the app and site.
static LANGUAGES: Lazy<Vec<LanguageInfo>> = Lazy::new(|| {
    serde_json::from_str(include_str!("../../website/languages.json"))
        .expect("checked-in language registry must be valid")
});

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct LanguagePreference(String);

impl<'de> Deserialize<'de> for LanguagePreference {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let code = String::deserialize(deserializer)?;
        if code == "auto" || LANGUAGES.iter().any(|language| language.code == code) {
            Ok(Self(code))
        } else {
            Err(serde::de::Error::custom("unsupported language"))
        }
    }
}

impl Default for LanguagePreference {
    fn default() -> Self {
        Self("auto".into())
    }
}

impl LanguagePreference {
    pub fn as_str(&self) -> &str {
        if self.0.is_empty() {
            "auto"
        } else {
            &self.0
        }
    }
}

fn match_language(locale: &str) -> Option<&'static LanguageInfo> {
    let normalized = locale.to_ascii_lowercase().replace('_', "-");
    let normalized = normalized.split('.').next()?;
    let parts: Vec<_> = normalized.split('-').collect();
    let mut code = *parts.first()?;
    if code == "zh" {
        code = if parts.contains(&"hant")
            || (!parts.contains(&"hans")
                && parts.iter().any(|part| ["tw", "hk", "mo"].contains(part)))
        {
            "zh-TW"
        } else {
            "zh"
        };
    }
    LANGUAGES.iter().find(|language| language.code == code)
}

pub fn resolve_language(preference: &LanguagePreference, locales: &[String]) -> &'static str {
    if let Some(language) = LANGUAGES
        .iter()
        .find(|language| language.code == preference.as_str())
    {
        return &language.code;
    }
    locales
        .iter()
        .find_map(|locale| match_language(locale))
        .map(|language| language.code.as_str())
        .unwrap_or("en")
}

#[tauri::command]
pub fn get_system_languages() -> Vec<String> {
    sys_locale::get_locales().collect()
}

pub fn tray_labels(preference: &LanguagePreference) -> [&'static str; 3] {
    let code = resolve_language(preference, &get_system_languages());
    let language = LANGUAGES
        .iter()
        .find(|language| language.code == code)
        .unwrap();
    [&language.tray[0], &language.tray[1], &language.tray[2]]
}

#[cfg(test)]
mod tests {
    use super::*;
    fn preference(code: &str) -> LanguagePreference {
        serde_json::from_value(serde_json::json!(code)).unwrap()
    }
    fn resolve(code: &str, locales: &[&str]) -> &'static str {
        resolve_language(
            &preference(code),
            &locales.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        )
    }

    #[test]
    fn automatic_language_uses_ordered_supported_system_locales() {
        assert_eq!(resolve("auto", &["zh-Hans-CN", "en-US"]), "zh");
        assert_eq!(resolve("auto", &["en_GB.UTF-8", "zh-CN"]), "en");
        assert_eq!(resolve("auto", &["xx-XX", "ZH_tw"]), "zh-TW");
        assert_eq!(resolve("auto", &["zh-Hant-HK"]), "zh-TW");
        assert_eq!(resolve("auto", &["zh-Hans-TW"]), "zh");
        assert_eq!(resolve("auto", &["pt-BR"]), "pt");
        assert_eq!(resolve("auto", &["fr-FR", "de-DE"]), "fr");
        assert_eq!(resolve("auto", &["xx-XX"]), "en");
        assert_eq!(resolve("auto", &[]), "en");
    }

    #[test]
    fn all_selectable_languages_override_system_and_have_tray_labels() {
        for language in LANGUAGES.iter() {
            assert_eq!(resolve(&language.code, &["en-US"]), language.code);
            assert!(tray_labels(&preference(&language.code))
                .iter()
                .all(|label| !label.is_empty()));
        }
    }

    #[test]
    fn old_configs_default_to_auto_and_choices_round_trip() {
        let old = r#"start_in_tray = true
show_tray_icon = true
start_at_login = false
logging_level = "info"
debug_mode = false"#;
        let mut config: crate::config::SystemConfig = toml::from_str(old).unwrap();
        assert_eq!(config.language.as_str(), "auto");
        for code in ["auto", "zh-TW", "ar", "ja"] {
            config.language = preference(code);
            let saved = toml::to_string(&config).unwrap();
            assert_eq!(
                toml::from_str::<crate::config::SystemConfig>(&saved)
                    .unwrap()
                    .language
                    .as_str(),
                code
            );
        }
        assert!(toml::from_str::<crate::config::SystemConfig>(&format!(
            "{old}\nlanguage = \"invalid\""
        ))
        .is_err());
    }
}
