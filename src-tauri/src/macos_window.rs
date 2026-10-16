//! AppKit policy for passive hint windows. Call native functions on the main thread.
use core_foundation::{base::TCFType, string::CFString};
use core_graphics::geometry::CGPoint;
use std::ffi::{c_char, c_void};
use std::time::{Duration, Instant};

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
    send_value(
        window,
        b"setCollectionBehavior:\0",
        CAN_JOIN_ALL_SPACES | STATIONARY | IGNORES_CYCLE,
    );
    send_value(window, b"setExcludedFromWindowsMenu:\0", 1_u8);

    // IgnoresCycle only affects AppKit's own window cycle. AltTab enumerates
    // AXWindows and admits AXStandardWindow / AXDialog, even for borderless
    // accessory apps. These passive surfaces are floating auxiliary windows.
    let subrole = CFString::new("AXFloatingWindow");
    send_value(
        window,
        b"setAccessibilitySubrole:\0",
        subrole.as_concrete_TypeRef(),
    );

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

pub struct PointHitBatch {
    pub hits: Vec<((i32, i32), Option<i64>)>,
    pub main_elapsed: Duration,
    pub query_elapsed: Duration,
}

/// Live click receiver, in global CG points. Must run on the AppKit main thread.
pub unsafe fn click_window_at(point: CGPoint) -> Option<i64> {
    let class = objc_getClass(b"NSWindow\0".as_ptr().cast());
    if class.is_null() {
        return None;
    }
    let primary_height = core_graphics::display::CGDisplay::main().bounds().size.height;
    let send: unsafe extern "C" fn(*mut c_void, *const c_void, CGPoint, isize) -> isize =
        std::mem::transmute(objc_msgSend as *const ());
    let number = send(
        class,
        selector(b"windowNumberAtPoint:belowWindowWithWindowNumber:\0"),
        CGPoint::new(point.x, primary_height - point.y),
        0,
    );
    (number > 0).then_some(number as i64)
}

fn appkit_point(point: (i32, i32), primary_height: f64) -> CGPoint {
    // AX / CGWindow coordinates use the primary display's top-left origin;
    // AppKit uses its bottom-left origin, including displays above/left of it.
    CGPoint::new(point.0 as f64, primary_height - point.1 as f64)
}

/// Must run on the AppKit main thread, before showing the hint overlays.
/// The time budget is cooperative: a single system query cannot be interrupted.
pub unsafe fn hit_test_points(points: &[(i32, i32)], primary_height: f64) -> PointHitBatch {
    let started = Instant::now();
    let class = objc_getClass(b"NSWindow\0".as_ptr().cast());
    let method = selector(b"windowNumberAtPoint:belowWindowWithWindowNumber:\0");
    let send: unsafe extern "C" fn(*mut c_void, *const c_void, CGPoint, isize) -> isize =
        std::mem::transmute(objc_msgSend as *const ());
    let mut hits = Vec::with_capacity(points.len());
    let mut query_elapsed = Duration::ZERO;
    for &point in points {
        let before = Instant::now();
        // Public AppKit hit testing skips ignoresMouseEvents windows and pixels
        // transparent at this point, and includes windows owned by other apps.
        let number = if class.is_null() {
            0
        } else {
            send(class, method, appkit_point(point, primary_height), 0)
        };
        query_elapsed += before.elapsed();
        hits.push((point, (number > 0).then_some(number as i64)));
        if started.elapsed() >= Duration::from_millis(2) {
            break;
        }
    }
    PointHitBatch {
        hits,
        main_elapsed: started.elapsed(),
        query_elapsed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_test_coordinates_preserve_left_and_above_display_positions() {
        let point = appkit_point((-1665, -214), 900.0);
        assert_eq!(point.x, -1665.0);
        assert_eq!(point.y, 1114.0);
        assert_eq!(appkit_point((300, 900), 900.0).y, 0.0);
    }
}
