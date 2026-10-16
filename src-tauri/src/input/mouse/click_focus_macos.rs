//! Focus-before-click for macOS only. AX IPC and waits stay off the UI thread.
use super::{get_cursor_position, post_mouse_click, CGPoint};
use core_foundation::{
    base::{CFType, CFTypeRef, TCFType},
    boolean::CFBoolean,
    dictionary::{CFDictionary, CFDictionaryRef},
    number::CFNumber,
    string::{CFString, CFStringRef},
};
use core_graphics::{
    display::{kCGWindowListOptionIncludingWindow, CGDisplay},
    geometry::CGSize,
};
use std::{
    ffi::c_void,
    ptr,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

const CLICK_TIMEOUT: Duration = Duration::from_millis(350);
const IPC_TIMEOUT: Duration = Duration::from_millis(50);
const POLL_INTERVAL: Duration = Duration::from_millis(10);
static GENERATION: AtomicU64 = AtomicU64::new(0);
static REQUEST: AtomicU64 = AtomicU64::new(0);
type Result<T> = std::result::Result<T, &'static str>;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> CFTypeRef;
    fn AXUIElementCopyElementAtPosition(
        app: CFTypeRef,
        x: f32,
        y: f32,
        value: *mut CFTypeRef,
    ) -> i32;
    fn AXUIElementCopyAttributeValue(
        element: CFTypeRef,
        attr: CFStringRef,
        value: *mut CFTypeRef,
    ) -> i32;
    fn AXUIElementSetAttributeValue(element: CFTypeRef, attr: CFStringRef, value: CFTypeRef)
        -> i32;
    fn AXUIElementPerformAction(element: CFTypeRef, action: CFStringRef) -> i32;
    fn AXUIElementSetMessagingTimeout(element: CFTypeRef, seconds: f32) -> i32;
    fn AXValueGetType(value: CFTypeRef) -> u32;
    fn AXValueGetValue(value: CFTypeRef, kind: u32, value: *mut c_void) -> u8;
}

pub fn cancel_pending_clicks() {
    REQUEST.fetch_add(1, Ordering::SeqCst);
    pointer_moved();
}

pub(super) fn pointer_moved() {
    GENERATION.fetch_add(1, Ordering::SeqCst);
}

// Reserve before spawning an executor task, so Escape or a new hint session
// also cancels clicks that have not started running yet.
pub fn click_ticket() -> u64 {
    REQUEST.fetch_add(1, Ordering::SeqCst) + 1
}
pub fn click_ticket_current(ticket: u64) -> bool {
    REQUEST.load(Ordering::SeqCst) == ticket
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Click {
    Left,
    Right,
    Middle,
    Double,
}

pub(super) async fn click(kind: Click, ticket: u64) -> bool {
    if !click_ticket_current(ticket) {
        return false;
    }
    let point = get_cursor_position();
    let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let started = Instant::now();
    // CF/AX objects are created and released within this worker, never sent
    // across threads. No sleeping or cross-process AX calls on the main thread.
    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut target = NativeClick::capture(point, generation, ticket, started, kind)?;
        complete_click(&mut target)
    })
    .await;
    match result {
        Ok(Ok(())) => log::info!(
            "[click_focus macos] {:?} complete in {:.1}ms",
            kind,
            started.elapsed().as_secs_f64() * 1000.0
        ),
        Ok(Err(reason)) => log::warn!(
            "[click_focus macos] {:?} cancelled: {} ({:.1}ms)",
            kind,
            reason,
            started.elapsed().as_secs_f64() * 1000.0
        ),
        Err(error) => log::error!("[click_focus macos] worker failed: {}", error),
    }
    // A superseded task must not dismiss hints belonging to a newer session.
    click_ticket_current(ticket) && GENERATION.load(Ordering::SeqCst) == generation
}

// Keep the sequencing testable without synthesizing input on the user's desktop.
trait ClickTarget {
    fn check_pending(&self) -> Result<()>;
    fn is_focused(&self) -> Result<bool>;
    fn request_focus(&mut self) -> Result<()>;
    fn wait(&mut self);
    fn revalidate(&self) -> Result<()>;
    fn send(&mut self) -> Result<()>;
}

fn complete_click(target: &mut impl ClickTarget) -> Result<()> {
    target.check_pending()?;
    if !target.is_focused()? {
        target.check_pending()?;
        target.request_focus()?;
        loop {
            target.check_pending()?;
            if target.is_focused()? {
                break;
            }
            target.wait();
        }
    }
    target.revalidate()?;
    // Revalidation can itself take time or race with an app switch.
    if !target.is_focused()? {
        return Err("focus changed before click");
    }
    target.check_pending()?;
    target.send()
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Frame {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Frame {
    fn contains(self, p: CGPoint) -> bool {
        p.x >= self.x && p.y >= self.y && p.x < self.x + self.width && p.y < self.y + self.height
    }
    fn matches(self, other: Self) -> bool {
        // AX and CG can round a window frame to adjacent logical points.
        [
            self.x - other.x,
            self.y - other.y,
            self.width - other.width,
            self.height - other.height,
        ]
        .iter()
        .all(|difference| difference.abs() <= 1.0)
    }
}

#[derive(Debug, PartialEq)]
struct Window {
    id: i64,
    pid: i32,
    layer: i32,
    frame: Frame,
}

fn number(dict: &CFDictionary<CFString, CFType>, key: &str) -> Result<f64> {
    dict.find(&CFString::new(key))
        .and_then(|value| value.downcast::<CFNumber>())
        .and_then(|value| value.to_f64())
        .ok_or("window metadata unavailable")
}

fn window_at(point: CGPoint, started: Instant) -> Result<Window> {
    let (tx, rx) = mpsc::sync_channel(1);
    dispatch::Queue::main().exec_async(move || {
        let hit = unsafe {
            crate::macos_window::click_window_at(core_graphics::geometry::CGPoint::new(
                point.x, point.y,
            ))
        };
        let _ = tx.send(hit);
    });
    let remaining = CLICK_TIMEOUT.saturating_sub(started.elapsed());
    let id = rx
        .recv_timeout(IPC_TIMEOUT.min(remaining))
        .map_err(|_| "window hit test timed out")?
        .ok_or("no window at click point")?;
    let list = CGDisplay::window_list_info(kCGWindowListOptionIncludingWindow, Some(id as u32))
        .ok_or("window disappeared")?;
    for raw in list.get_all_values() {
        let dict = unsafe {
            CFDictionary::<CFString, CFType>::wrap_under_get_rule(raw as CFDictionaryRef)
        };
        if number(&dict, "kCGWindowNumber")? as i64 != id {
            continue;
        }
        let bounds = dict
            .find(&CFString::new("kCGWindowBounds"))
            .ok_or("window bounds missing")?;
        let bounds = unsafe {
            CFDictionary::<CFString, CFType>::wrap_under_get_rule(
                bounds.as_CFTypeRef() as CFDictionaryRef
            )
        };
        let frame = Frame {
            x: number(&bounds, "X")?,
            y: number(&bounds, "Y")?,
            width: number(&bounds, "Width")?,
            height: number(&bounds, "Height")?,
        };
        if !frame.contains(point) {
            return Err("point outside window");
        }
        return Ok(Window {
            id,
            pid: number(&dict, "kCGWindowOwnerPID")? as i32,
            layer: number(&dict, "kCGWindowLayer")? as i32,
            frame,
        });
    }
    Err("window disappeared")
}

struct Ax(CFType);
impl Ax {
    fn owned(raw: CFTypeRef) -> Result<Self> {
        if raw.is_null() {
            return Err("AX object unavailable");
        }
        let value = Self(unsafe { CFType::wrap_under_create_rule(raw) });
        // Object-local timeout; never change the process-wide collector timeout.
        unsafe {
            AXUIElementSetMessagingTimeout(raw, IPC_TIMEOUT.as_secs_f32());
        }
        Ok(value)
    }
    fn raw(&self) -> CFTypeRef {
        self.0.as_CFTypeRef()
    }
    fn attribute(&self, name: &str) -> Result<CFType> {
        let mut value = ptr::null();
        let status = unsafe {
            AXUIElementCopyAttributeValue(
                self.raw(),
                CFString::new(name).as_concrete_TypeRef(),
                &mut value,
            )
        };
        if status != 0 || value.is_null() {
            return Err("AX attribute unavailable");
        }
        Ok(unsafe { CFType::wrap_under_create_rule(value) })
    }
    fn child(&self, name: &str) -> Result<Ax> {
        let child = Self(self.attribute(name)?);
        unsafe {
            AXUIElementSetMessagingTimeout(child.raw(), IPC_TIMEOUT.as_secs_f32());
        }
        Ok(child)
    }
    fn at(&self, point: CGPoint) -> Result<Ax> {
        let mut value = ptr::null();
        let status = unsafe {
            AXUIElementCopyElementAtPosition(self.raw(), point.x as f32, point.y as f32, &mut value)
        };
        if status != 0 {
            return Err("AX hit test failed");
        }
        Self::owned(value)
    }
    fn frame(&self) -> Result<Frame> {
        let position = self.attribute("AXPosition")?;
        let size = self.attribute("AXSize")?;
        let mut point = CGPoint { x: 0.0, y: 0.0 };
        let mut dimensions = CGSize::new(0.0, 0.0);
        unsafe {
            if AXValueGetType(position.as_CFTypeRef()) != 1
                || AXValueGetType(size.as_CFTypeRef()) != 2
                || AXValueGetValue(
                    position.as_CFTypeRef(),
                    1,
                    (&mut point as *mut CGPoint).cast(),
                ) == 0
                || AXValueGetValue(
                    size.as_CFTypeRef(),
                    2,
                    (&mut dimensions as *mut CGSize).cast(),
                ) == 0
            {
                return Err("AX bounds unavailable");
            }
        }
        Ok(Frame {
            x: point.x,
            y: point.y,
            width: dimensions.width,
            height: dimensions.height,
        })
    }
    fn set_true(&self, name: &str) -> i32 {
        unsafe {
            AXUIElementSetAttributeValue(
                self.raw(),
                CFString::new(name).as_concrete_TypeRef(),
                CFBoolean::true_value().as_CFTypeRef(),
            )
        }
    }
}

struct Focus {
    app: Ax,
    window: Ax,
    element: Ax,
    element_frame: Frame,
}
struct NativeClick {
    point: CGPoint,
    generation: u64,
    ticket: u64,
    started: Instant,
    window: Window,
    focus: Option<Focus>,
    kind: Click,
}

fn is_nonactivating_surface(layer: i32, role: &str) -> bool {
    // Do not infer "nonactivating" from a positive layer alone: ordinary
    // floating app windows also live above layer zero and still need focus.
    layer < 0
        || matches!(
            role,
            "AXMenuBar" | "AXMenuBarItem" | "AXMenu" | "AXMenuItem" | "AXDockItem" | "AXDock"
        )
        || (layer == 20 && role == "AXList") // Dock's empty background
}

impl NativeClick {
    fn capture(
        point: CGPoint,
        generation: u64,
        ticket: u64,
        started: Instant,
        kind: Click,
    ) -> Result<Self> {
        let window = window_at(point, started)?;
        let mut target = Self {
            point,
            generation,
            ticket,
            started,
            window,
            focus: None,
            kind,
        };
        target.check_pending()?;
        // Activating a menu or Dock can dismiss the target. Ordinary and
        // floating app windows both require confirmation of window focus.
        if target.window.layer >= 0 {
            let app = Ax::owned(unsafe { AXUIElementCreateApplication(target.window.pid) })?;
            let element = app.at(point)?;
            let role = element
                .attribute("AXRole")?
                .downcast::<CFString>()
                .map(|s| s.to_string());
            if is_nonactivating_surface(target.window.layer, role.as_deref().unwrap_or("")) {
                return Ok(target);
            }
            let window = if role.as_deref() == Some("AXWindow") {
                Ax(element.0.clone())
            } else {
                element.child("AXWindow")?
            };
            if !window.frame()?.matches(target.window.frame) {
                return Err("AX window does not match click receiver");
            }
            let element_frame = element.frame()?;
            if !element_frame.contains(point) {
                return Err("point outside AX element");
            }
            target.focus = Some(Focus {
                app,
                window,
                element,
                element_frame,
            });
        }
        Ok(target)
    }
}

impl ClickTarget for NativeClick {
    fn check_pending(&self) -> Result<()> {
        if !click_ticket_current(self.ticket)
            || GENERATION.load(Ordering::SeqCst) != self.generation
        {
            return Err("superseded or cancelled");
        }
        if !crate::input::keyboard::KEYBOARD_STATE
            .lock()
            .map(|state| state.in_ctrl_session)
            .unwrap_or(false)
        {
            return Err("hint session ended");
        }
        if self.started.elapsed() >= CLICK_TIMEOUT {
            return Err("focus/click deadline exceeded");
        }
        if get_cursor_position() != self.point {
            return Err("pointer moved");
        }
        Ok(())
    }
    fn is_focused(&self) -> Result<bool> {
        let Some(focus) = &self.focus else {
            return Ok(true);
        };
        let active = focus
            .app
            .attribute("AXFrontmost")
            .ok()
            .and_then(|v| v.downcast::<CFBoolean>())
            == Some(CFBoolean::true_value());
        if !active {
            return Ok(false);
        }
        // Main window alone is insufficient, especially with multiple windows
        // belonging to the same app. Confirm the actual keyboard-focus window.
        // A temporarily missing focused window is not confirmation; keep
        // polling within the deadline, including during app activation.
        Ok(focus
            .app
            .attribute("AXFocusedWindow")
            .map(|window| window == focus.window.0)
            .unwrap_or(false))
    }
    fn request_focus(&mut self) -> Result<()> {
        if let Some(focus) = &self.focus {
            let main = focus.window.set_true("AXMain");
            self.check_pending()?;
            let raise = unsafe {
                AXUIElementPerformAction(
                    focus.window.raw(),
                    CFString::new("AXRaise").as_concrete_TypeRef(),
                )
            };
            self.check_pending()?;
            let activate = focus.app.set_true("AXFrontmost");
            if main != 0 && raise != 0 && activate != 0 {
                return Err("activation request rejected");
            }
        }
        Ok(())
    }
    fn wait(&mut self) {
        std::thread::sleep(POLL_INTERVAL);
    }
    fn revalidate(&self) -> Result<()> {
        self.check_pending()?;
        if let Some(focus) = &self.focus {
            let hit = focus.app.at(self.point)?;
            if hit.0 != focus.element.0
                || !hit.frame()?.matches(focus.element_frame)
                || !focus.window.frame()?.matches(self.window.frame)
            {
                return Err("target element or bounds changed");
            }
        }
        // Fresh WindowServer receiver after activation; never reuse hint cache.
        if window_at(self.point, self.started)? != self.window {
            return Err("click receiver or bounds changed");
        }
        self.check_pending()
    }
    fn send(&mut self) -> Result<()> {
        let (down, up, button) = match self.kind {
            Click::Left | Click::Double => (
                super::CG_EVENT_LEFT_MOUSE_DOWN,
                super::CG_EVENT_LEFT_MOUSE_UP,
                super::CG_MOUSE_BUTTON_LEFT,
            ),
            Click::Right => (
                super::CG_EVENT_RIGHT_MOUSE_DOWN,
                super::CG_EVENT_RIGHT_MOUSE_UP,
                super::CG_MOUSE_BUTTON_RIGHT,
            ),
            Click::Middle => (
                super::CG_EVENT_OTHER_MOUSE_DOWN,
                super::CG_EVENT_OTHER_MOUSE_UP,
                super::CG_MOUSE_BUTTON_CENTER,
            ),
        };
        post_mouse_click(down, self.point, button, 1);
        post_mouse_click(up, self.point, button, 1);
        // Only an explicit double-click command emits a second pair. Never
        // retry an ordinary click to compensate for unconfirmed focus.
        if matches!(self.kind, Click::Double) {
            std::thread::sleep(Duration::from_millis(50));
            self.check_pending()?;
            if !self.is_focused()? {
                return Err("focus changed during double click");
            }
            post_mouse_click(down, self.point, button, 2);
            post_mouse_click(up, self.point, button, 2);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Target {
        elapsed: Duration,
        focus_after: Duration,
        focus_requests: usize,
        clicks: usize,
        valid: bool,
        cancelled: bool,
        lost_focus: Cell<bool>,
        cancel_after: Option<Duration>,
        lose_focus_on_revalidate: bool,
    }
    impl Target {
        fn new(focus_after: Duration) -> Self {
            Self {
                elapsed: Duration::ZERO,
                focus_after,
                focus_requests: 0,
                clicks: 0,
                valid: true,
                cancelled: false,
                lost_focus: Cell::new(false),
                cancel_after: None,
                lose_focus_on_revalidate: false,
            }
        }
    }
    impl ClickTarget for Target {
        fn check_pending(&self) -> Result<()> {
            if self.elapsed >= CLICK_TIMEOUT || self.cancelled {
                Err("cancelled or timeout")
            } else {
                Ok(())
            }
        }
        fn is_focused(&self) -> Result<bool> {
            Ok(self.elapsed >= self.focus_after && !self.lost_focus.get())
        }
        fn request_focus(&mut self) -> Result<()> {
            self.focus_requests += 1;
            Ok(())
        }
        fn wait(&mut self) {
            self.elapsed += POLL_INTERVAL;
            if self
                .cancel_after
                .is_some_and(|deadline| self.elapsed >= deadline)
            {
                self.cancelled = true;
            }
        }
        fn revalidate(&self) -> Result<()> {
            if self.lose_focus_on_revalidate {
                self.lost_focus.set(true);
            }
            if self.valid {
                Ok(())
            } else {
                Err("target moved or replaced")
            }
        }
        fn send(&mut self) -> Result<()> {
            self.clicks += 1;
            Ok(())
        }
    }
    #[test]
    fn focused_window_clicks_once_without_activation_or_delay() {
        let mut target = Target::new(Duration::ZERO);
        assert!(complete_click(&mut target).is_ok());
        assert_eq!(
            (target.focus_requests, target.clicks, target.elapsed),
            (0, 1, Duration::ZERO)
        );
    }
    #[test]
    fn delayed_focus_is_confirmed_before_exactly_one_click() {
        let mut target = Target::new(Duration::from_millis(30));
        assert!(complete_click(&mut target).is_ok());
        assert_eq!(
            (
                target.focus_requests,
                target.clicks,
                target.elapsed.as_millis()
            ),
            (1, 1, 30)
        );
    }
    #[test]
    fn activation_success_without_focus_confirmation_never_clicks() {
        let mut target = Target::new(Duration::MAX);
        assert!(complete_click(&mut target).is_err());
        assert_eq!((target.focus_requests, target.clicks), (1, 0));
        assert_eq!(target.elapsed, CLICK_TIMEOUT);
    }
    #[test]
    fn changed_target_and_cancelled_requests_never_click() {
        let mut target = Target::new(Duration::from_millis(20));
        target.valid = false;
        assert!(complete_click(&mut target).is_err());
        assert_eq!(target.clicks, 0);
        target.cancelled = true;
        target.focus_requests = 0;
        assert!(complete_click(&mut target).is_err());
        assert_eq!((target.focus_requests, target.clicks), (0, 0));
    }
    #[test]
    fn negative_screen_coordinates_and_frame_changes_are_checked() {
        let frame = Frame {
            x: -1000.0,
            y: -600.0,
            width: 500.0,
            height: 400.0,
        };
        assert!(frame.contains(CGPoint {
            x: -999.0,
            y: -599.0
        }));
        assert!(!frame.contains(CGPoint {
            x: -500.0,
            y: -599.0
        }));
        assert!(!frame.matches(Frame { x: -900.0, ..frame }));
        assert!(!frame.matches(Frame {
            width: 600.0,
            ..frame
        }));
    }
    #[test]
    fn escape_during_focus_wait_never_clicks() {
        let mut target = Target::new(Duration::from_millis(100));
        target.cancel_after = Some(Duration::from_millis(20));
        assert!(complete_click(&mut target).is_err());
        assert_eq!((target.clicks, target.elapsed.as_millis()), (0, 20));
    }
    #[test]
    fn losing_focus_during_coordinate_revalidation_never_clicks() {
        let mut target = Target::new(Duration::ZERO);
        target.lose_focus_on_revalidate = true;
        assert!(complete_click(&mut target).is_err());
        assert_eq!(target.clicks, 0);
    }
    #[test]
    fn floating_windows_require_focus_but_menus_and_dock_do_not() {
        assert!(!is_nonactivating_surface(3, "AXButton"));
        assert!(!is_nonactivating_surface(25, "AXWindow"));
        assert!(is_nonactivating_surface(101, "AXMenuItem"));
        assert!(is_nonactivating_surface(20, "AXDockItem"));
        assert!(is_nonactivating_surface(-2147483623, "AXScrollArea"));
    }
}
