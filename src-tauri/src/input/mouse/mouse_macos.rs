use crate::monitor::MONITORS_STORAGE;
use log::error;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};

type CGEventRef = *const c_void;
type CGEventSourceRef = *const c_void;

#[repr(C)]
#[derive(Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

const CG_EVENT_LEFT_MOUSE_DOWN: u32 = 1;
const CG_EVENT_LEFT_MOUSE_UP: u32 = 2;
const CG_EVENT_RIGHT_MOUSE_DOWN: u32 = 3;
const CG_EVENT_RIGHT_MOUSE_UP: u32 = 4;
const CG_EVENT_MOUSE_MOVED: u32 = 5;
const CG_EVENT_LEFT_MOUSE_DRAGGED: u32 = 6;
const CG_EVENT_OTHER_MOUSE_DOWN: u32 = 25;
const CG_EVENT_OTHER_MOUSE_UP: u32 = 26;

const CG_MOUSE_BUTTON_LEFT: u32 = 0;
const CG_MOUSE_BUTTON_RIGHT: u32 = 1;
const CG_MOUSE_BUTTON_CENTER: u32 = 2;

const CG_HID_EVENT_TAP: u32 = 0;
const CG_SCROLL_EVENT_UNIT_PIXEL: u32 = 0;
const CG_MOUSE_EVENT_CLICK_STATE: u32 = 1;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventCreateMouseEvent(
        source: CGEventSourceRef,
        mouse_type: u32,
        mouse_cursor_position: CGPoint,
        mouse_button: u32,
    ) -> CGEventRef;
    fn CGEventCreate(source: CGEventSourceRef) -> CGEventRef;
    fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
    fn CGEventPost(tap: u32, event: CGEventRef);
    fn CGEventSetIntegerValueField(event: CGEventRef, field: u32, value: i64);
    fn CGWarpMouseCursorPosition(new_cursor_position: CGPoint) -> i32;
    fn CGEventCreateScrollWheelEvent(
        source: CGEventSourceRef,
        units: u32,
        wheel_count: u32,
        wheel1: i32,
        ...
    ) -> CGEventRef;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
}

static DRAGGING: AtomicBool = AtomicBool::new(false);

fn get_cursor_position() -> CGPoint {
    unsafe {
        let event = CGEventCreate(std::ptr::null());
        let point = CGEventGetLocation(event);
        CFRelease(event);
        point
    }
}

fn post_mouse(etype: u32, point: CGPoint, button: u32) {
    unsafe {
        let event = CGEventCreateMouseEvent(std::ptr::null(), etype, point, button);
        if !event.is_null() {
            CGEventPost(CG_HID_EVENT_TAP, event);
            CFRelease(event);
        }
    }
}

fn post_mouse_click(etype: u32, point: CGPoint, button: u32, click_count: i64) {
    unsafe {
        let event = CGEventCreateMouseEvent(std::ptr::null(), etype, point, button);
        if !event.is_null() {
            CGEventSetIntegerValueField(event, CG_MOUSE_EVENT_CLICK_STATE, click_count);
            CGEventPost(CG_HID_EVENT_TAP, event);
            CFRelease(event);
        }
    }
}

pub async fn mouse_move(monitor: usize, x: i32, y: i32) {
    if let Ok(monitors) = MONITORS_STORAGE.lock() {
        if let Some(m) = monitors.get(monitor) {
            let gx = (m.x as f64 / m.scale_factor) + x as f64;
            let gy = (m.y as f64 / m.scale_factor) + y as f64;
            unsafe {
                CGWarpMouseCursorPosition(CGPoint { x: gx, y: gy });
            }
        } else {
            error!("[mouse_macos] monitor not found: {}", monitor);
        }
    }
}

pub async fn mouse_move_relative(dx: i32, dy: i32) {
    let cur = get_cursor_position();
    let new_pos = CGPoint {
        x: cur.x + dx as f64,
        y: cur.y + dy as f64,
    };
    unsafe {
        CGWarpMouseCursorPosition(new_pos);
    }
    let etype = if DRAGGING.load(Ordering::Relaxed) {
        CG_EVENT_LEFT_MOUSE_DRAGGED
    } else {
        CG_EVENT_MOUSE_MOVED
    };
    post_mouse(etype, new_pos, CG_MOUSE_BUTTON_LEFT);
}

pub async fn mouse_click_left() {
    let pos = get_cursor_position();
    post_mouse(CG_EVENT_LEFT_MOUSE_DOWN, pos, CG_MOUSE_BUTTON_LEFT);
    post_mouse(CG_EVENT_LEFT_MOUSE_UP, pos, CG_MOUSE_BUTTON_LEFT);
}

pub async fn mouse_click_right() {
    let pos = get_cursor_position();
    post_mouse(CG_EVENT_RIGHT_MOUSE_DOWN, pos, CG_MOUSE_BUTTON_RIGHT);
    post_mouse(CG_EVENT_RIGHT_MOUSE_UP, pos, CG_MOUSE_BUTTON_RIGHT);
}

pub async fn mouse_click_middle() {
    let pos = get_cursor_position();
    post_mouse(CG_EVENT_OTHER_MOUSE_DOWN, pos, CG_MOUSE_BUTTON_CENTER);
    post_mouse(CG_EVENT_OTHER_MOUSE_UP, pos, CG_MOUSE_BUTTON_CENTER);
}

pub async fn mouse_double_click() {
    let pos = get_cursor_position();
    post_mouse_click(CG_EVENT_LEFT_MOUSE_DOWN, pos, CG_MOUSE_BUTTON_LEFT, 1);
    post_mouse_click(CG_EVENT_LEFT_MOUSE_UP, pos, CG_MOUSE_BUTTON_LEFT, 1);
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    post_mouse_click(CG_EVENT_LEFT_MOUSE_DOWN, pos, CG_MOUSE_BUTTON_LEFT, 2);
    post_mouse_click(CG_EVENT_LEFT_MOUSE_UP, pos, CG_MOUSE_BUTTON_LEFT, 2);
}

pub async fn mouse_drag_start() {
    DRAGGING.store(true, Ordering::Relaxed);
    let pos = get_cursor_position();
    post_mouse(CG_EVENT_LEFT_MOUSE_DOWN, pos, CG_MOUSE_BUTTON_LEFT);
}

pub async fn mouse_drag_end() {
    let pos = get_cursor_position();
    post_mouse(CG_EVENT_LEFT_MOUSE_UP, pos, CG_MOUSE_BUTTON_LEFT);
    DRAGGING.store(false, Ordering::Relaxed);
}

pub async fn mouse_wheel_move(delta_x: i32, delta_y: i32) {
    // macOS horizontal scroll direction is opposite to Windows convention
    let delta_x = -delta_x;
    unsafe {
        if delta_y != 0 && delta_x != 0 {
            let event = CGEventCreateScrollWheelEvent(
                std::ptr::null(),
                CG_SCROLL_EVENT_UNIT_PIXEL,
                2,
                delta_y,
                delta_x,
            );
            if !event.is_null() {
                CGEventPost(CG_HID_EVENT_TAP, event);
                CFRelease(event);
            }
        } else if delta_y != 0 {
            let event = CGEventCreateScrollWheelEvent(
                std::ptr::null(),
                CG_SCROLL_EVENT_UNIT_PIXEL,
                1,
                delta_y,
            );
            if !event.is_null() {
                CGEventPost(CG_HID_EVENT_TAP, event);
                CFRelease(event);
            }
        } else if delta_x != 0 {
            let event = CGEventCreateScrollWheelEvent(
                std::ptr::null(),
                CG_SCROLL_EVENT_UNIT_PIXEL,
                2,
                0i32,
                delta_x,
            );
            if !event.is_null() {
                CGEventPost(CG_HID_EVENT_TAP, event);
                CFRelease(event);
            }
        }
    }
}
