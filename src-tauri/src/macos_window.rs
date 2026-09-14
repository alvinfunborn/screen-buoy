//! AppKit policy for passive hint windows. Call native functions on the main thread.
use core_foundation::{base::TCFType, string::CFString};
use std::ffi::{c_char, c_void};

#[link(name = "objc", kind = "dylib")]
extern "C" {
    fn objc_msgSend();
    fn objc_getClass(name: *const c_char) -> *mut c_void;
    fn sel_registerName(name: *const c_char) -> *const c_void;
}

unsafe fn selector(name: &'static [u8]) -> *const c_void {
    sel_registerName(name.as_ptr().cast())
}

unsafe fn get_object(obj: *mut c_void, name: &'static [u8]) -> *mut c_void {
    let send: unsafe extern "C" fn(*mut c_void, *const c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    send(obj, selector(name))
}

unsafe fn send_value<T>(obj: *mut c_void, name: &'static [u8], value: T) {
    let send: unsafe extern "C" fn(*mut c_void, *const c_void, T) =
        std::mem::transmute(objc_msgSend as *const ());
    send(obj, selector(name), value);
}

pub fn set_app_activation_policy_accessory() {
    unsafe {
        let cls = objc_getClass(b"NSApplication\0".as_ptr().cast());
        let app = get_object(cls, b"sharedApplication\0");
        send_value(app, b"setActivationPolicy:\0", 1_isize);
    }
}

pub unsafe fn configure_overlay(window: *mut c_void) {
    const CAN_JOIN_ALL_SPACES: usize = 1 << 0;
    const STATIONARY: usize = 1 << 4;
    const IGNORES_CYCLE: usize = 1 << 6;
    send_value(window, b"setCollectionBehavior:\0", CAN_JOIN_ALL_SPACES | STATIONARY | IGNORES_CYCLE);
    send_value(window, b"setExcludedFromWindowsMenu:\0", 1_u8);

    // IgnoresCycle only affects AppKit's own window cycle. AltTab enumerates
    // AXWindows and admits AXStandardWindow / AXDialog, even for borderless
    // accessory apps. These passive surfaces are floating auxiliary windows.
    let subrole = CFString::new("AXFloatingWindow");
    send_value(window, b"setAccessibilitySubrole:\0", subrole.as_concrete_TypeRef());

    // Tao's set_always_on_top asynchronously writes level 5, below Dock (20).
    // Use one native owner for the level so a queued setter cannot overwrite it.
    send_value(window, b"setLevel:\0", 25_isize); // NSStatusWindowLevel
}

pub unsafe fn set_overlay_visible(window: *mut c_void, visible: bool) {
    if visible {
        configure_overlay(window);
        // Tauri show() calls makeKeyAndOrderFront on this Tao version. Hints
        // must leave the target app focused, including for Space-at-pointer.
        let send: unsafe extern "C" fn(*mut c_void, *const c_void) =
            std::mem::transmute(objc_msgSend as *const ());
        send(window, selector(b"orderFrontRegardless\0"));
    } else {
        send_value(window, b"orderOut:\0", std::ptr::null::<c_void>());
    }
}
