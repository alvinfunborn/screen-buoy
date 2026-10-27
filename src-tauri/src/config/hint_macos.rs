//! Native AX role matching and one-time, in-memory conversion of legacy IDs.
use super::hint::HintConfig;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

// Candidate order preserves the old Mac matcher for customized legacy configs.
// These IDs are also retained in diagnostics; native matching uses role names.
pub const LEGACY_AX_ROLES: &[(&str, &[i32])] = &[
    ("AXButton", &[50021, 50000, 50026]),
    ("AXDockItem", &[50021, 50000, 50026]),
    ("AXRadioButton", &[50013]),
    ("AXCheckBox", &[50002]),
    ("AXPopUpButton", &[50003]),
    ("AXComboBox", &[50003]),
    ("AXTextField", &[50004]),
    ("AXTextArea", &[50004]),
    ("AXSearchField", &[50004]),
    ("AXSecureTextField", &[50004]),
    ("AXLink", &[50005]),
    ("AXImage", &[50006]),
    ("AXList", &[50008]),
    ("AXMenu", &[50009]),
    ("AXMenuBar", &[50010]),
    ("AXMenuItem", &[50011]),
    ("AXMenuButton", &[50011]),
    ("AXProgressIndicator", &[50012]),
    ("AXScrollBar", &[50014]),
    ("AXSlider", &[50015]),
    ("AXIncrementor", &[50016]),
    ("AXTabGroup", &[50017]),
    ("AXTabButton", &[50018]),
    ("AXTab", &[50018]),
    ("AXOutline", &[50019, 50037]),
    ("AXToolbar", &[50020, 50021]),
    ("AXWindow", &[50032]),
    ("AXSheet", &[50032]),
    ("AXDrawer", &[50032]),
    ("AXGroup", &[50033, 50035, 50040]),
    ("AXLayoutArea", &[50033, 50035, 50040]),
    ("AXScrollArea", &[50033, 50035, 50040]),
    ("AXSplitGroup", &[50034]),
    ("AXSplitter", &[50034]),
    ("AXTable", &[50036]),
    ("AXCell", &[50036]),
    ("AXStaticText", &[50033, 50020]),
    ("AXWebArea", &[50033, 50035, 50004]),
];

const LEGACY_DEFAULTS: &[(&str, &[i32])] = &[
    (
        "button",
        &[
            50021, 50026, 50000, 50001, 50002, 50003, 50025, 50028, 50029, 50030, 50004, 50005,
            50006, 50008, 50007, 50010, 50009, 50011, 50012, 50013, 50039, 50031, 50036, 50020,
            50027, 50022, 50023, 50024,
        ],
    ),
    ("default", &[]),
    ("pane", &[50040, 50034, 50035, 50033, 50017, 50018, 50037]),
    ("scrollbar", &[50014, 50038, 50015, 50016]),
    ("tab", &[50019]),
    ("text", &[]),
    ("window", &[50032]),
];

pub fn migrate_legacy_roles(hint: &mut HintConfig) {
    if hint
        .types
        .values()
        .all(|kind| kind.element_ax_roles.is_some())
    {
        return;
    }
    // Upgrade the old stock grouping, even if its colors/z-index were edited.
    // Any change to membership, or explicit native role list, is user-owned.
    let stock = hint.types.len() == LEGACY_DEFAULTS.len()
        && LEGACY_DEFAULTS.iter().all(|(name, ids)| {
            hint.types.get(*name).is_some_and(|kind| {
                kind.element_ax_roles.is_none()
                    && kind.element_control_types.iter().collect::<HashSet<_>>()
                        == ids.iter().collect()
            })
        });
    if stock {
        let defaults: super::Config =
            toml::from_str(include_str!("../../config_macos.toml")).expect("valid macOS defaults");
        for (name, kind) in &mut hint.types {
            kind.element_ax_roles = defaults.hint.types[name].element_ax_roles.clone();
        }
        return;
    }

    // Last style owning an ID wins, just as in the original numeric map.
    let mut owners = HashMap::new();
    for (index, (_, kind)) in hint.types.iter().enumerate() {
        for id in &kind.element_control_types {
            owners.insert(*id, index);
        }
    }
    let mut roles = vec![Vec::new(); hint.types.len()];
    let explicit_roles: HashSet<_> = hint
        .types
        .values()
        .flat_map(|kind| kind.element_ax_roles.iter().flatten())
        .collect();
    for (role, candidates) in LEGACY_AX_ROLES {
        if explicit_roles.iter().any(|name| name.as_str() == *role) {
            continue;
        }
        if let Some(index) = candidates.iter().find_map(|id| owners.get(id)) {
            roles[*index].push((*role).to_owned());
        }
    }
    for ((_, kind), legacy_roles) in hint.types.iter_mut().zip(roles) {
        if kind.element_ax_roles.is_none() {
            kind.element_ax_roles = Some(legacy_roles);
        }
    }
}

// (style index, z-index, legacy diagnostic ID). Explicit empty lists remain empty;
// no numeric fallback can silently re-enable a role after editing it in settings.
fn role_map(hint: &HintConfig) -> HashMap<String, (usize, i32, i32)> {
    let mut map = HashMap::new();
    for (index, (_, kind)) in hint.types.iter().enumerate() {
        for role in kind.element_ax_roles.iter().flatten() {
            let legacy_id = LEGACY_AX_ROLES
                .iter()
                .find(|(name, _)| name == role)
                .map_or(0, |(_, ids)| ids[0]);
            map.insert(role.clone(), (index, kind.z_index, legacy_id));
        }
    }
    map
}

pub static HINT_AX_ROLE_MAP: Lazy<HashMap<String, (usize, i32, i32)>> =
    Lazy::new(|| role_map(&super::get_config().unwrap().hint));

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> super::super::Config {
        toml::from_str(include_str!("../../config_macos.toml")).unwrap()
    }

    fn legacy() -> super::super::Config {
        let mut config = defaults();
        for (name, ids) in LEGACY_DEFAULTS {
            let kind = config.hint.types.get_mut(*name).unwrap();
            kind.element_ax_roles = None;
            kind.element_control_types = ids.to_vec();
        }
        config
    }

    fn group<'a>(hint: &'a HintConfig, role: &str) -> Option<&'a str> {
        let map = role_map(hint);
        let index = map.get(role)?.0;
        Some(hint.types.get_index(index)?.0)
    }

    #[test]
    fn native_defaults_cover_existing_roles_once_and_separate_tabs_text_and_trees() {
        let hint = defaults().hint;
        let roles: Vec<_> = hint
            .types
            .values()
            .flat_map(|kind| kind.element_ax_roles.iter().flatten())
            .collect();
        assert_eq!(
            roles.len(),
            role_map(&hint).len(),
            "no duplicate default roles"
        );
        assert_eq!(roles.len(), LEGACY_AX_ROLES.len());
        for (role, _) in LEGACY_AX_ROLES {
            assert!(group(&hint, role).is_some(), "{role}");
        }
        for role in ["AXTab", "AXTabButton", "AXTabGroup"] {
            assert_eq!(group(&hint, role), Some("tab"));
        }
        for role in [
            "AXStaticText",
            "AXTextField",
            "AXTextArea",
            "AXSecureTextField",
        ] {
            assert_eq!(group(&hint, role), Some("text"));
        }
        assert_eq!(group(&hint, "AXOutline"), Some("pane"));
        assert_eq!(group(&hint, "AXButton"), Some("button"));
        assert_eq!(group(&hint, "AXWindow"), Some("window"));
        assert_eq!(group(&hint, "AXUnknown"), None);
    }

    #[test]
    fn stock_migration_updates_roles_but_preserves_styles_order_and_other_preferences() {
        let mut config = legacy();
        config.hint.types["tab"].style = "custom color".into();
        config.hint.types["text"].z_index = 17;
        config.keybinding.hotkey_buoy = "Alt+Q".into();
        let original = config.clone();
        let serialized = toml::to_string(&config).unwrap();
        let migrated = super::super::parse_config(&serialized).unwrap();
        assert_eq!(group(&migrated.hint, "AXTab"), Some("tab"));
        assert_eq!(group(&migrated.hint, "AXStaticText"), Some("text"));
        assert_eq!(migrated.keybinding.hotkey_buoy, "Alt+Q");
        for (name, kind) in &original.hint.types {
            assert_eq!(migrated.hint.types[name].style, kind.style);
            assert_eq!(migrated.hint.types[name].z_index, kind.z_index);
        }
        assert_eq!(
            original.hint.types.keys().collect::<Vec<_>>(),
            migrated.hint.types.keys().collect::<Vec<_>>()
        );
        let saved = toml::to_string(&migrated).unwrap();
        let reloaded = super::super::parse_config(&saved).unwrap();
        assert_eq!(
            serde_json::to_value(migrated).unwrap(),
            serde_json::to_value(reloaded).unwrap()
        );
    }

    #[test]
    fn customized_legacy_membership_preserves_fallback_priority_and_disabled_types() {
        let mut hint = legacy().hint;
        hint.types["button"]
            .element_control_types
            .retain(|id| *id != 50021);
        hint.types["text"].element_control_types = vec![50021];
        hint.types["pane"].element_control_types.clear();
        migrate_legacy_roles(&mut hint);
        assert_eq!(group(&hint, "AXButton"), Some("text")); // old first-candidate precedence
        assert_eq!(group(&hint, "AXToolbar"), Some("button")); // 50020 precedes 50021
        assert_eq!(group(&hint, "AXStaticText"), Some("button")); // old fallback to 50020
        assert_eq!(group(&hint, "AXOutline"), Some("tab")); // preserve custom layout
        assert_eq!(group(&hint, "AXTab"), None);
        assert_eq!(group(&hint, "AXGroup"), None);
    }

    #[test]
    fn native_edits_and_empty_lists_survive_save_reload_without_numeric_fallback() {
        let mut config = legacy();
        migrate_legacy_roles(&mut config.hint);
        config.hint.types["tab"].element_ax_roles = Some(vec![]);
        config.hint.types["text"].element_ax_roles = Some(vec!["AXCustomWidget".into()]);
        let saved = toml::to_string(&config).unwrap();
        let reloaded = super::super::parse_config(&saved).unwrap();
        assert_eq!(group(&reloaded.hint, "AXTab"), None);
        assert_eq!(group(&reloaded.hint, "AXTextField"), None);
        assert_eq!(group(&reloaded.hint, "AXCustomWidget"), Some("text"));
        assert_eq!(
            serde_json::to_value(config).unwrap(),
            serde_json::to_value(reloaded).unwrap()
        );
    }

    #[test]
    fn explicit_native_roles_take_precedence_in_a_partially_migrated_config() {
        let mut hint = legacy().hint;
        hint.types["default"].element_ax_roles = Some(vec!["AXButton".into()]);
        hint.types["tab"].element_ax_roles = Some(vec![]);
        migrate_legacy_roles(&mut hint);
        assert_eq!(group(&hint, "AXButton"), Some("default"));
        assert!(!hint.types["button"]
            .element_ax_roles
            .as_ref()
            .unwrap()
            .contains(&"AXButton".into()));
        assert_eq!(group(&hint, "AXOutline"), None);
        assert_eq!(group(&hint, "AXTab"), Some("pane"));
    }
}
