mod ui_automation;
#[cfg(target_os = "macos")]
pub(crate) use ui_automation::{dock_frame, DOCK_WINDOW_CLASS};
pub mod element;

pub use ui_automation::UIElement;
pub use element::{request_refresh_at, set_hint_active, setup_ui_collection, snapshots_since, subscribe};
