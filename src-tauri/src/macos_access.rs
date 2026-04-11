use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::string::CFString;
use log::{info, warn};

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> u8;
}

pub fn log_executable_identity() {
    let pid = std::process::id();
    match std::env::current_exe() {
        Ok(p) => info!(
            "[macos] pid={} executable={} — 请在「辅助功能」「输入监视」中勾选与此路径完全一致的项",
            pid,
            p.display()
        ),
        Err(e) => warn!("[macos] current_exe failed: {}", e),
    }
}

pub fn log_accessibility_status() {
    if is_accessibility_trusted() {
        info!("[macos] Accessibility permission granted");
    } else {
        warn!("[macos] Accessibility not granted. Please enable in: System Settings > Privacy & Security > Accessibility");
        warn!("[macos] If already checked, remove and re-add the app (binary may have changed after reinstall)");
    }
}

/// Check accessibility without prompting. Returns true if trusted.
pub fn is_accessibility_trusted() -> bool {
    let dict: CFDictionary<CFString, CFBoolean> = CFDictionary::from_CFType_pairs(&[(
        CFString::from_static_string("AXTrustedCheckOptionPrompt"),
        CFBoolean::false_value(),
    )]);
    unsafe { AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef()) != 0 }
}
