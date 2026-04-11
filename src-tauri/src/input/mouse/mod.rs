#[cfg(target_os = "windows")]
mod mouse_win;
#[cfg(target_os = "windows")]
pub use mouse_win::*;

#[cfg(target_os = "macos")]
mod mouse_macos;
#[cfg(target_os = "macos")]
pub use mouse_macos::*;
