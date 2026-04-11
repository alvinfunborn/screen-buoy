#[cfg(target_os = "windows")]
mod ui_automation_win;
#[cfg(target_os = "windows")]
pub use ui_automation_win::*;

#[cfg(target_os = "macos")]
mod ui_automation_macos;
#[cfg(target_os = "macos")]
pub use ui_automation_macos::*;
