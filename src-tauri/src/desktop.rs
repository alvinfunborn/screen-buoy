//! Read-only desktop identity and geometry for a BeaconHop-scoped hint session.
use crate::{utils::Rect, window::WindowElement};

#[derive(Clone, Debug)]
pub struct DesktopScope {
    pub id: String,
    pub display_id: String,
    pub bounds: Vec<Rect>,
}

impl DesktopScope {
    pub fn capture(id: &str, display_id: &str) -> Result<Self, String> {
        let bounds = current_bounds(id, display_id)
            .filter(|bounds| !bounds.is_empty())
            .ok_or(
                "目标桌面已改变或显示器不可用 / Target desktop changed or display is unavailable",
            )?;
        Ok(Self {
            id: id.into(),
            display_id: display_id.into(),
            bounds,
        })
    }

    pub fn is_current(&self) -> bool {
        current_bounds(&self.id, &self.display_id).is_some_and(|bounds| {
            bounds.len() == self.bounds.len()
                && bounds
                    .iter()
                    .zip(&self.bounds)
                    .all(|(a, b)| (a.x, a.y, a.width, a.height) == (b.x, b.y, b.width, b.height))
        })
    }

    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        self.bounds.iter().any(|bounds| bounds.contains_point(x, y))
    }

    pub fn windows(&self) -> Vec<WindowElement> {
        crate::window::window::get_all_windows()
            .into_iter()
            .filter(|window| {
                self.bounds.iter().any(|bounds| {
                    bounds.intersects(&Rect::new(window.x, window.y, window.width, window.height))
                }) && visible_on_desktop(window)
            })
            .collect()
    }
}

#[cfg(target_os = "macos")]
fn visible_on_desktop(_: &WindowElement) -> bool {
    // macOS get_all_windows already uses kCGWindowListOptionOnScreenOnly.
    true
}

#[cfg(target_os = "windows")]
fn visible_on_desktop(window: &WindowElement) -> bool {
    use windows::Win32::{
        Foundation::HWND,
        Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED},
    };
    let mut cloaked = 0u32;
    unsafe {
        DwmGetWindowAttribute(
            HWND(window.window_handle as *mut _),
            DWMWA_CLOAKED,
            &mut cloaked as *mut _ as *mut _,
            std::mem::size_of_val(&cloaked) as u32,
        )
        .is_ok()
            && cloaked == 0
    }
}

#[cfg(target_os = "windows")]
fn current_bounds(id: &str, display_id: &str) -> Option<Vec<Rect>> {
    use windows::core::BOOL;
    use windows::Win32::{
        Foundation::{LPARAM, RECT},
        Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR},
    };
    // A Windows virtual desktop spans all displays.
    if !display_id.is_empty()
        || format!(
            "{:?}",
            winvd::get_current_desktop().and_then(|d| d.get_id()).ok()?
        ) != id
    {
        return None;
    }
    unsafe extern "system" fn collect(
        _: HMONITOR,
        _: HDC,
        bounds: *mut RECT,
        data: LPARAM,
    ) -> BOOL {
        let bounds = &*bounds;
        (&mut *(data.0 as *mut Vec<Rect>)).push(Rect::new(
            bounds.left,
            bounds.top,
            bounds.right - bounds.left,
            bounds.bottom - bounds.top,
        ));
        BOOL(1)
    }
    let mut bounds: Vec<Rect> = Vec::new();
    unsafe {
        EnumDisplayMonitors(
            None,
            None,
            Some(collect),
            LPARAM(&mut bounds as *mut _ as isize),
        )
        .ok()
        .ok()?;
    }
    bounds.sort_by_key(|b| (b.x, b.y));
    Some(bounds)
}

#[cfg(target_os = "macos")]
fn macos_display_bounds(display: u32, shared: bool, displays: &[(u32, Rect)]) -> Option<Vec<Rect>> {
    if !displays.iter().any(|(id, _)| *id == display) {
        return None;
    }
    let mut bounds: Vec<_> = displays
        .iter()
        .filter(|(id, _)| shared || *id == display)
        .map(|(_, bounds)| bounds.clone())
        .collect();
    // Keep scope validation independent of native display enumeration order.
    bounds.sort_by_key(|b| (b.x, b.y, b.width, b.height));
    Some(bounds)
}

#[cfg(target_os = "macos")]
fn current_bounds(id: &str, display_id: &str) -> Option<Vec<Rect>> {
    use core_foundation::{
        array::{CFArray, CFArrayRef},
        base::{CFType, TCFType},
        dictionary::CFDictionary,
        number::CFNumber,
        string::CFString,
        uuid::{CFUUIDCreateString, CFUUIDRef, CFUUID},
    };
    use core_graphics::display::CGDisplay;
    use std::sync::OnceLock;
    type Connection = unsafe extern "C" fn() -> i32;
    type Managed = unsafe extern "C" fn(i32) -> CFArrayRef;
    fn dictionary(value: &CFType) -> Option<CFDictionary<CFString, CFType>> {
        let untyped = value.downcast::<CFDictionary>()?;
        Some(unsafe { CFDictionary::wrap_under_get_rule(untyped.as_concrete_TypeRef()) })
    }
    // Cache function pointers; keep the framework loaded for their lifetime.
    static FUNCTIONS: OnceLock<Option<(Connection, Managed)>> = OnceLock::new();
    let (connection, managed) = *FUNCTIONS
        .get_or_init(|| unsafe {
            let handle = libc::dlopen(
                c"/System/Library/PrivateFrameworks/SkyLight.framework/SkyLight".as_ptr(),
                libc::RTLD_LAZY,
            );
            if handle.is_null() {
                return None;
            }
            let connection = libc::dlsym(handle, c"SLSMainConnectionID".as_ptr());
            let managed = libc::dlsym(handle, c"SLSCopyManagedDisplaySpaces".as_ptr());
            if connection.is_null() || managed.is_null() {
                return None;
            }
            Some((
                std::mem::transmute::<*mut libc::c_void, Connection>(connection),
                std::mem::transmute::<*mut libc::c_void, Managed>(managed),
            ))
        })
        .as_ref()?;
    let display = display_id.parse::<u32>().ok()?;
    let active_displays = CGDisplay::active_displays().ok()?;
    if !active_displays.contains(&display) {
        return None;
    }
    let space = id.parse::<i64>().ok()?;
    if space <= 0 {
        return None;
    }
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGDisplayCreateUUIDFromDisplayID(display: u32) -> CFUUIDRef;
    }
    let identifier = unsafe {
        let raw = CGDisplayCreateUUIDFromDisplayID(display);
        if raw.is_null() {
            return None;
        }
        let uuid = CFUUID::wrap_under_create_rule(raw);
        CFString::wrap_under_create_rule(CFUUIDCreateString(
            std::ptr::null(),
            uuid.as_concrete_TypeRef(),
        ))
        .to_string()
    };
    let displays = unsafe {
        let raw = managed(connection());
        if raw.is_null() {
            return None;
        }
        CFArray::<CFType>::wrap_under_create_rule(raw)
    };
    for item in displays.iter() {
        let Some(dict) = dictionary(&item) else {
            continue;
        };
        let name = dict
            .find(&CFString::new("Display Identifier"))?
            .downcast::<CFString>()?
            .to_string();
        if name != identifier && !(name == "Main" && display == CGDisplay::main().id) {
            continue;
        }
        let current_value = dict.find(&CFString::new("Current Space"))?;
        let current = dictionary(&current_value)?;
        if current
            .find(&CFString::new("id64"))?
            .downcast::<CFNumber>()?
            .to_i64()?
            != space
        {
            return None;
        }
        // With shared Spaces, WindowServer reports one "Main" desktop group
        // spanning every active display. The BH display ID is its activation
        // anchor, not a restriction to that screen. UUID groups remain scoped
        // to their own display when each display has separate Spaces.
        let screens: Vec<_> = active_displays
            .iter()
            .map(|&id| {
                let bounds = CGDisplay::new(id).bounds();
                (
                    id,
                    Rect::new(
                        bounds.origin.x as i32,
                        bounds.origin.y as i32,
                        bounds.size.width as i32,
                        bounds.size.height as i32,
                    ),
                )
            })
            .collect();
        return macos_display_bounds(display, name == "Main", &screens);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Read-only native desktop check; requires SB_TEST_DESKTOP_ID and SB_TEST_DISPLAY_ID"]
    fn native_desktop_scope_round_trip_without_switching() {
        let id = std::env::var("SB_TEST_DESKTOP_ID").unwrap();
        let display = std::env::var("SB_TEST_DISPLAY_ID").unwrap_or_default();
        let scope = DesktopScope::capture(&id, &display).unwrap();
        assert!(scope.is_current());
        assert!(scope.bounds.iter().all(|b| b.width > 0 && b.height > 0));
        if let Ok(count) = std::env::var("SB_TEST_DISPLAY_COUNT") {
            assert_eq!(scope.bounds.len(), count.parse::<usize>().unwrap());
        }
        println!(
            "desktop={} anchor={} bounds={:?}",
            id, display, scope.bounds
        );
        assert!(DesktopScope::capture("invalid-desktop", &display).is_err());
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn shared_spaces_cover_all_displays_with_stable_geometry_order() {
        let screens = vec![
            (1, Rect::new(0, 0, 1440, 900)),
            (2, Rect::new(-1920, -180, 1920, 1080)),
        ];
        let bounds = macos_display_bounds(1, true, &screens).unwrap();
        assert_eq!(bounds.len(), 2);
        assert_eq!(
            (bounds[0].x, bounds[0].y, bounds[0].width, bounds[0].height),
            (-1920, -180, 1920, 1080)
        );
        let mut reversed = screens.clone();
        reversed.reverse();
        let reordered = macos_display_bounds(1, true, &reversed).unwrap();
        assert!(bounds
            .iter()
            .zip(reordered)
            .all(|(a, b)| (a.x, a.y, a.width, a.height) == (b.x, b.y, b.width, b.height)));
        let scope = DesktopScope {
            id: "d".into(),
            display_id: "1".into(),
            bounds,
        };
        assert!(scope.contains_point(-100, -100));
        assert!(scope.contains_point(100, 100));
        assert!(!scope.contains_point(100, -100));
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn separate_spaces_stay_on_the_selected_display_and_require_live_anchor() {
        let screens = vec![
            (1, Rect::new(0, 0, 1440, 900)),
            (2, Rect::new(-1920, 0, 1920, 1080)),
        ];
        let bounds = macos_display_bounds(2, false, &screens).unwrap();
        assert_eq!(bounds.len(), 1);
        assert_eq!(bounds[0].x, -1920);
        assert!(macos_display_bounds(3, true, &screens).is_none());
        assert!(macos_display_bounds(1, true, &[]).is_none());
    }
    #[test]
    fn desktop_bounds_include_empty_background_but_exclude_other_displays() {
        let scope = DesktopScope {
            id: "d".into(),
            display_id: "m".into(),
            bounds: vec![Rect::new(-1280, 0, 1280, 720)],
        };
        assert!(scope.contains_point(-100, 100));
        assert!(!scope.contains_point(0, 100));
        assert!(!scope.contains_point(-100, 720));
    }
}
