#[cfg(target_os = "windows")]
mod window_win;
#[cfg(target_os = "windows")]
pub use window_win::*;

#[cfg(target_os = "macos")]
mod window_macos;
#[cfg(target_os = "macos")]
pub use window_macos::*;
