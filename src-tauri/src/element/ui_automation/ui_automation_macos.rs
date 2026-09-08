use core_foundation::array::{CFArray, CFArrayRef};
use core_foundation::base::{CFIndex, CFType, CFTypeRef, TCFType};
use core_foundation::string::{CFString, CFStringRef};
use core_graphics::geometry::{CGPoint, CGSize};

use core_foundation::boolean::CFBoolean;
use log::{debug, info, warn};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::ptr;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config;
use crate::config::hint::HINT_CONTROL_TYPES_ID_Z_MAP;
use crate::window::WindowElement;

pub struct UIAutomationRequest;

#[derive(Clone)]
pub struct UIElement {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub width: i32,
    pub height: i32,
    pub window_handle: i64,
    pub control_type: i32,
    pub element_type: usize,
}

static ELEMENTS_CACHE_WITH_EXPIRATION: Lazy<Mutex<HashMap<i64, (Vec<UIElement>, u128)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static ENHANCED_UI_PIDS: Lazy<Mutex<HashSet<i32>>> = Lazy::new(|| Mutex::new(HashSet::new()));

type AXUIElementRef = CFTypeRef;

const AX_SUCCESS: i32 = 0;
const MAX_DEPTH: usize = 28;
const MAX_NODES: usize = 6000;

fn leak_cfstring(s: &str) -> CFStringRef {
    let c = CFString::new(s);
    let r = c.as_concrete_TypeRef();
    std::mem::forget(c);
    r
}

#[derive(Clone, Copy)]
struct AxAttr(CFStringRef);
unsafe impl Send for AxAttr {}
unsafe impl Sync for AxAttr {}

impl AxAttr {
    fn as_cf(&self) -> CFStringRef {
        self.0
    }
}

static AX_WINDOWS: Lazy<AxAttr> = Lazy::new(|| AxAttr(leak_cfstring("AXWindows")));
static AX_CHILDREN: Lazy<AxAttr> = Lazy::new(|| AxAttr(leak_cfstring("AXChildren")));
static AX_ROLE: Lazy<AxAttr> = Lazy::new(|| AxAttr(leak_cfstring("AXRole")));
static AX_TITLE: Lazy<AxAttr> = Lazy::new(|| AxAttr(leak_cfstring("AXTitle")));
static AX_POSITION: Lazy<AxAttr> = Lazy::new(|| AxAttr(leak_cfstring("AXPosition")));
static AX_SIZE: Lazy<AxAttr> = Lazy::new(|| AxAttr(leak_cfstring("AXSize")));
static AX_ENHANCED_UI: Lazy<AxAttr> =
    Lazy::new(|| AxAttr(leak_cfstring("AXEnhancedUserInterface")));

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> u8;
    fn AXUIElementCreateApplication(pid: libc::pid_t) -> CFTypeRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> i32;
    fn AXUIElementSetAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: CFTypeRef,
    ) -> i32;
    fn AXValueGetType(value: CFTypeRef) -> u32;
    fn AXValueGetValue(value: CFTypeRef, the_type: u32, ptr: *mut c_void) -> u8;
}

const K_AXVALUE_CGPOINT: u32 = 1;
const K_AXVALUE_CGSIZE: u32 = 2;

impl UIAutomationRequest {
    pub fn new() -> Self {
        UIAutomationRequest
    }

    pub fn get_elements_for_window(&self, window: &WindowElement) -> Option<Vec<UIElement>> {
        get_elements_for_window_inner(window)
    }

    pub fn get_cached_elements_for_window(&self, window: &WindowElement) -> Option<Vec<UIElement>> {
        let window_handle = window.window_handle;
        let expired = {
            let cache = ELEMENTS_CACHE_WITH_EXPIRATION.lock().unwrap();
            if let Some((elements, expire_at)) = cache.get(&window_handle) {
                if SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    <= *expire_at
                {
                    debug!(
                        "[get_cached_elements_for_window] macos cached {} for {}",
                        elements.len(),
                        window_handle
                    );
                    return Some(elements.clone());
                }
                true
            } else {
                false
            }
        };
        if expired {
            let mut cache = ELEMENTS_CACHE_WITH_EXPIRATION.lock().unwrap();
            cache.remove(&window_handle);
        }
        self.get_elements_for_window(window)
    }
}

pub fn get_cached_elements_for_window(window: &WindowElement) -> Option<Vec<UIElement>> {
    let request = UIAutomationRequest::new();
    request.get_cached_elements_for_window(window)
}

fn ax_copy_attribute(element: AXUIElementRef, attr: CFStringRef) -> Option<CFType> {
    let mut out: CFTypeRef = ptr::null();
    let st = unsafe { AXUIElementCopyAttributeValue(element, attr, &mut out) };
    if st != AX_SUCCESS || out.is_null() {
        return None;
    }
    Some(unsafe { CFType::wrap_under_create_rule(out) })
}

fn ax_copy_string(element: AXUIElementRef, attr: CFStringRef) -> Option<String> {
    let v = ax_copy_attribute(element, attr)?;
    v.downcast::<CFString>().map(|s| s.to_string())
}

fn ax_point_size(element: AXUIElementRef) -> Option<(f64, f64, f64, f64)> {
    let pos_v = ax_copy_attribute(element, AX_POSITION.as_cf())?;
    let sz_v = ax_copy_attribute(element, AX_SIZE.as_cf())?;
    let mut pt = CGPoint { x: 0.0, y: 0.0 };
    let mut sz = CGSize {
        width: 0.0,
        height: 0.0,
    };
    unsafe {
        if AXValueGetType(pos_v.as_CFTypeRef()) != K_AXVALUE_CGPOINT
            || AXValueGetValue(pos_v.as_CFTypeRef(), K_AXVALUE_CGPOINT, &mut pt as *mut _ as *mut c_void)
                == 0
        {
            return None;
        }
        if AXValueGetType(sz_v.as_CFTypeRef()) != K_AXVALUE_CGSIZE
            || AXValueGetValue(sz_v.as_CFTypeRef(), K_AXVALUE_CGSIZE, &mut sz as *mut _ as *mut c_void)
                == 0
        {
            return None;
        }
    }
    Some((pt.x, pt.y, sz.width, sz.height))
}

fn ax_rect_in_screen_points(px: f64, py: f64, pw: f64, ph: f64) -> (i32, i32, i32, i32) {
    // AX and CGWindow bounds both use global logical points. Scaling every
    // candidate to the target's size can make a different window look identical.
    (
        px.round() as i32,
        py.round() as i32,
        pw.round() as i32,
        ph.round() as i32,
    )
}

fn map_ax_point_to_target_pixels(
    px: f64,
    py: f64,
    pw: f64,
    ph: f64,
    target: &WindowElement,
    rpx: f64,
    rpy: f64,
    sx: f64,
    sy: f64,
) -> (i32, i32, i32, i32) {
    let x = target.x + ((px - rpx) * sx).round() as i32;
    let y = target.y + ((py - rpy) * sy).round() as i32;
    let w = (pw * sx).round() as i32;
    let h = (ph * sy).round() as i32;
    (x, y, w, h)
}

fn frame_iou(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> f64 {
    let x1 = a.0.max(b.0);
    let y1 = a.1.max(b.1);
    let x2 = (a.0 + a.2).min(b.0 + b.2);
    let y2 = (a.1 + a.3).min(b.1 + b.3);
    if x2 <= x1 || y2 <= y1 {
        return 0.0;
    }
    let inter = (x2 - x1) * (y2 - y1);
    let u = a.2 * a.3 + b.2 * b.3 - inter;
    if u <= 0 {
        return 0.0;
    }
    inter as f64 / u as f64
}

fn find_ax_window(app: AXUIElementRef, target: &WindowElement) -> Option<(CFType, f64, f64, f64, f64)> {
    let Some(wins) = ax_copy_attribute(app, AX_WINDOWS.as_cf()) else {
        warn!(
            "[ui_automation macos] AXWindows missing pid={} wh={}",
            target.owner_pid, target.window_handle
        );
        return None;
    };
    let arr = unsafe { CFArray::<CFType>::wrap_under_get_rule(wins.as_CFTypeRef() as CFArrayRef) };
    let _n = arr.len();
    let tgt = (target.x, target.y, target.width, target.height);
    let mut best_i: Option<usize> = None;
    let mut best_s = -1.0_f64;
    let n = arr.len() as usize;
    for i in 0..n {
        let Some(item) = arr.get(i as CFIndex) else {
            continue;
        };
        let el = item.as_CFTypeRef();
        if el.is_null() {
            continue;
        }
        let title = ax_copy_string(el, AX_TITLE.as_cf()).unwrap_or_default();
        let Some((px, py, pw, ph)) = ax_point_size(el) else {
            continue;
        };
        let fr = ax_rect_in_screen_points(px, py, pw, ph);
        let score = frame_iou(tgt, fr);
        let title_bonus = if !target.title.is_empty() && title == target.title {
            0.15
        } else {
            0.0
        };
        let s = score + title_bonus;
        if s > 0.35 || (!target.title.is_empty() && title == target.title && score > 0.05) {
            if s > best_s {
                best_s = s;
                best_i = Some(i as usize);
            }
        }
    }
    let Some(i) = best_i else {
        debug!(
            "[ui_automation macos] no AX window matched pid={} wh={} tgt={:?} ax_windows={}",
            target.owner_pid, target.window_handle, tgt, _n
        );
        return None;
    };
    let item = arr.get(i as CFIndex)?;
    let el = item.as_CFTypeRef();
    let (rpx, rpy, rpw, rph) = ax_point_size(el)?;
    unsafe { Some((CFType::wrap_under_get_rule(el), rpx, rpy, rpw, rph)) }
}

fn control_type_for_ax_role(role: &str) -> Option<i32> {
    let m = &*HINT_CONTROL_TYPES_ID_Z_MAP;
    let try_seq = |ids: &[i32]| ids.iter().copied().find(|id| m.contains_key(id));
    match role {
        "AXButton" => try_seq(&[50021, 50000, 50026]),
        "AXRadioButton" => try_seq(&[50013]),
        "AXCheckBox" => try_seq(&[50002]),
        "AXPopUpButton" | "AXComboBox" => try_seq(&[50003]),
        "AXTextField" | "AXTextArea" | "AXSearchField" | "AXSecureTextField" => try_seq(&[50004]),
        "AXLink" => try_seq(&[50005]),
        "AXImage" => try_seq(&[50006]),
        "AXList" => try_seq(&[50008]),
        "AXMenu" => try_seq(&[50009]),
        "AXMenuBar" => try_seq(&[50010]),
        "AXMenuItem" | "AXMenuButton" => try_seq(&[50011]),
        "AXProgressIndicator" => try_seq(&[50012]),
        "AXScrollBar" => try_seq(&[50014]),
        "AXSlider" => try_seq(&[50015]),
        "AXIncrementor" => try_seq(&[50016]),
        "AXTabGroup" => try_seq(&[50017]),
        "AXTabButton" | "AXTab" => try_seq(&[50018]),
        "AXOutline" => try_seq(&[50019, 50037]),
        "AXToolbar" => try_seq(&[50020, 50021]),
        "AXWindow" | "AXSheet" | "AXDrawer" => try_seq(&[50032]),
        "AXGroup" | "AXLayoutArea" | "AXScrollArea" => try_seq(&[50033, 50035, 50040]),
        "AXSplitGroup" | "AXSplitter" => try_seq(&[50034]),
        "AXTable" | "AXCell" => try_seq(&[50036]),
        "AXStaticText" => try_seq(&[50033, 50020]),
        "AXWebArea" => try_seq(&[50033, 50035, 50004]),
        _ => None,
    }
}

fn walk_element(
    element: AXUIElementRef,
    depth: usize,
    window_handle: i64,
    target: &WindowElement,
    rpx: f64,
    rpy: f64,
    sx: f64,
    sy: f64,
    nodes: &mut usize,
    out: &mut HashMap<(i32, i32), UIElement>,
    role_stats: &mut HashMap<String, usize>,
) {
    if depth > MAX_DEPTH || *nodes > MAX_NODES {
        return;
    }
    *nodes += 1;

    let role = ax_copy_string(element, AX_ROLE.as_cf()).unwrap_or_default();
    *role_stats.entry(role.clone()).or_insert(0) += 1;

    if let Some((px, py, pw, ph)) = ax_point_size(element) {
        if pw >= 1.0 && ph >= 1.0 {
            if let Some(ctrl) = control_type_for_ax_role(&role) {
                let hint_map = &*HINT_CONTROL_TYPES_ID_Z_MAP;
                if let Some((et, zix)) = hint_map.get(&ctrl) {
                    let (x, y_tl, w, h) =
                        map_ax_point_to_target_pixels(px, py, pw, ph, target, rpx, rpy, sx, sy);
                    let cx = x + w / 2;
                    let cy = y_tl + h / 2;
                    let ui = UIElement {
                        text: String::new(),
                        x: cx,
                        y: cy,
                        z: *zix,
                        width: w,
                        height: h,
                        window_handle,
                        control_type: ctrl,
                        element_type: *et,
                    };
                    let key = (cx, cy);
                    match out.get(&key) {
                        Some(old) if old.z >= *zix => {}
                        _ => {
                            out.insert(key, ui);
                        }
                    }
                }
            }
        }
    }

    let Some(ch) = ax_copy_attribute(element, AX_CHILDREN.as_cf()) else {
        return;
    };
    let arr = unsafe { CFArray::<CFType>::wrap_under_get_rule(ch.as_CFTypeRef() as CFArrayRef) };
    let nc = arr.len() as usize;
    for i in 0..nc {
        let Some(item) = arr.get(i as CFIndex) else {
            continue;
        };
        let child = item.as_CFTypeRef();
        if !child.is_null() {
            walk_element(
                child,
                depth + 1,
                window_handle,
                target,
                rpx,
                rpy,
                sx,
                sy,
                nodes,
                out,
                role_stats,
            );
        }
    }
}

fn ensure_enhanced_ui(app: AXUIElementRef, pid: i32) {
    let mut set = ENHANCED_UI_PIDS.lock().unwrap();
    if set.contains(&pid) {
        return;
    }
    let val = CFBoolean::true_value();
    let rc = unsafe {
        AXUIElementSetAttributeValue(app, AX_ENHANCED_UI.as_cf(), val.as_CFTypeRef())
    };
    if rc == AX_SUCCESS {
        info!("[ui_automation macos] AXEnhancedUserInterface set for pid={}", pid);
    }
    set.insert(pid);
}

fn get_elements_for_window_inner(window: &WindowElement) -> Option<Vec<UIElement>> {
    if unsafe { AXIsProcessTrusted() } == 0 {
        warn!("[ui_automation macos] AXIsProcessTrusted=false — grant Accessibility in System Settings");
        return None;
    }
    if window.owner_pid <= 0 {
        return None;
    }

    let app = unsafe {
        let r = AXUIElementCreateApplication(window.owner_pid);
        if r.is_null() {
            return None;
        }
        CFType::wrap_under_create_rule(r)
    };

    ensure_enhanced_ui(app.as_CFTypeRef(), window.owner_pid);

    let (root, rpx, rpy, rpw, rph) = find_ax_window(app.as_CFTypeRef(), window)?;
    let sx = window.width as f64 / rpw.max(1.0);
    let sy = window.height as f64 / rph.max(1.0);
    let mut nodes = 0usize;
    let mut map: HashMap<(i32, i32), UIElement> = HashMap::new();
    let mut role_stats: HashMap<String, usize> = HashMap::new();
    walk_element(
        root.as_CFTypeRef(),
        0,
        window.window_handle,
        window,
        rpx,
        rpy,
        sx,
        sy,
        &mut nodes,
        &mut map,
        &mut role_stats,
    );
    drop(root);
    drop(app);

    let elements: Vec<UIElement> = map.values().cloned().collect();
    let expire_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        + config::get_config().unwrap().ui_automation.cache_ttl as u128;
    ELEMENTS_CACHE_WITH_EXPIRATION
        .lock()
        .unwrap()
        .insert(window.window_handle, (elements.clone(), expire_at));
    debug!(
        "[get_elements_for_window] macos {} elements (nodes={}) for {} ({}) roles={:?}",
        elements.len(),
        nodes,
        window.window_handle,
        window.title,
        role_stats
    );
    Some(elements)
}

pub fn clean_expired_cache() {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let mut cache = ELEMENTS_CACHE_WITH_EXPIRATION.lock().unwrap();
    cache.retain(|_, (_, expire_at)| *expire_at > now);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_smaller_window_must_not_be_scaled_to_match_the_target() {
        let target = (400, 400, 800, 600);
        let wrong_window = ax_rect_in_screen_points(200.0, 200.0, 400.0, 300.0);
        assert!(frame_iou(target, wrong_window) < 0.35);
        assert_eq!(frame_iou(target, ax_rect_in_screen_points(400.0, 400.0, 800.0, 600.0)), 1.0);
    }

    #[test]
    fn negative_screen_coordinates_are_preserved() {
        let bounds = ax_rect_in_screen_points(-1440.0, -200.0, 800.0, 600.0);
        assert_eq!(bounds, (-1440, -200, 800, 600));
        assert_eq!(frame_iou(bounds, bounds), 1.0);
        assert_eq!(frame_iou(bounds, (0, 0, 800, 600)), 0.0);
    }
}
