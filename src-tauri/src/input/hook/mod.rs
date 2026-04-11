#[cfg(target_os = "windows")]
mod hook_windows;
#[cfg(target_os = "windows")]
pub use hook_windows::*;

#[cfg(target_os = "macos")]
mod hook_macos;
#[cfg(target_os = "macos")]
pub use hook_macos::*;
