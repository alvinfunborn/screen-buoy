use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU16, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use core_foundation::base::TCFType;
use core_foundation::mach_port::CFMachPort;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::EventField;
use log::{debug, error, info, warn};
use once_cell::sync::Lazy;

use crate::config;
use crate::input::keyboard::{handle_keyboard_event, KEYBOARD_STATE};
use crate::input::modifier_tap::ModifierTap;

type CGEventRef = *const c_void;
type CGEventTapProxy = *const c_void;
type CGKeyCode = u16;

type CGEventTapCallBackInternal = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    etype: u32,
    event: CGEventRef,
    user_info: *const c_void,
) -> CGEventRef;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: CGEventTapCallBackInternal,
        user_info: *const c_void,
    ) -> core_foundation::mach_port::CFMachPortRef;

    fn CGEventTapEnable(tap: core_foundation::mach_port::CFMachPortRef, enable: bool);
    fn CGEventTapIsEnabled(tap: core_foundation::mach_port::CFMachPortRef) -> bool;
    fn CGEventGetFlags(event: CGEventRef) -> u64;
    fn CGEventGetLocation(event: CGEventRef) -> core_graphics::geometry::CGPoint;
    fn CGEventSourceKeyState(state_id: u32, key_code: CGKeyCode) -> bool;
    fn CGEventSourceButtonState(state_id: u32, button: u32) -> bool;
    fn CGEventKeyboardGetUnicodeString(
        event: CGEventRef,
        max_string_length: libc::c_ulong,
        actual_string_length: *mut libc::c_ulong,
        unicode_string: *mut u16,
    );
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
}

#[repr(C)]
#[derive(Default)]
struct ProcessSerialNumber { high: u32, low: u32 }

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn GetFrontProcess(process: *mut ProcessSerialNumber) -> i32;
}

fn foreground_context() -> Option<u64> {
    let mut process = ProcessSerialNumber::default();
    (unsafe { GetFrontProcess(&mut process) } == 0)
        .then_some(((process.high as u64) << 32) | process.low as u64)
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRunLoopRun();
    fn CFRunLoopStop(rl: *const c_void);
}

static TAP_RUNLOOP: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static RUNNING: AtomicBool = AtomicBool::new(false);
static READY: AtomicBool = AtomicBool::new(false);
// Captured at startup, matching the settings UI's restart-to-apply contract.
static SINGLE_KEY_ENABLED: AtomicBool = AtomicBool::new(false);
static SINGLE_KEY_CODE: AtomicU16 = AtomicU16::new(58);

static SHORTCUT_NAME: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static SHORTCUT_CODE: AtomicU16 = AtomicU16::new(u16::MAX);
static SHORTCUT_MODIFIERS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static SHORTCUT_HOLD: Lazy<Mutex<ShortcutHold>> = Lazy::new(|| Mutex::new(ShortcutHold::default()));

#[derive(Default)]
struct ShortcutHold {
    armed: bool,
    key_down: bool,
    modifiers: bool,
}

impl ShortcutHold {
    fn observe(&mut self, trigger: bool, down: bool, modifiers: bool, other_down: bool) {
        if trigger {
            if down && modifiers { self.armed = true; }
            self.key_down = down;
            if !down { self.armed = false; }
        } else if other_down { self.armed = false; }
        self.modifiers = modifiers;
    }
    fn take_ready(&mut self, active: bool) -> bool {
        if active && self.armed && self.key_down && !self.modifiers {
            self.armed = false;
            true
        } else { false }
    }
}

pub(crate) fn activate_shortcut_hold(app: &tauri::AppHandle) {
    let active = KEYBOARD_STATE.lock().unwrap().in_ctrl_session;
    if SHORTCUT_HOLD.lock().unwrap().take_ready(active) {
        let key = SHORTCUT_NAME.lock().unwrap().clone();
        crate::input::keyboard::hold_at_cursor(app, key);
    }
}

unsafe fn observe_shortcut_hold(etype: u32, event: CGEventRef, active: bool) -> bool {
    if !matches!(etype, CG_KEY_DOWN | CG_KEY_UP | CG_FLAGS_CHANGED) { return false; }
    let code = CGEventGetIntegerValueField(event, EventField::KEYBOARD_EVENT_KEYCODE) as u16;
    let trigger = code == SHORTCUT_CODE.load(Ordering::Relaxed);
    let flags = CGEventGetFlags(event) & (0x20000 | 0x40000 | 0x80000 | 0x100000 | 0x800000);
    let repeat = etype == CG_KEY_DOWN && CGEventGetIntegerValueField(event, EventField::KEYBOARD_EVENT_AUTOREPEAT) != 0;
    {
        let mut hold = SHORTCUT_HOLD.lock().unwrap();
        // Arm only on the configured chord, never from an ordinary held key.
        if !active && trigger && etype == CG_KEY_DOWN && !repeat {
            hold.observe(true, true, flags != 0, false);
            hold.armed = flags == SHORTCUT_MODIFIERS.load(Ordering::Relaxed);
        } else if trigger && etype == CG_KEY_UP {
            hold.observe(true, false, flags != 0, false);
        } else if etype == CG_FLAGS_CHANGED {
            hold.modifiers = flags != 0;
            // A newly pressed modifier cancels a pending chord handoff.
            if modifier_down_from_flags(code, CGEventGetFlags(event)) == Some(true) { hold.armed = false; }
        } else if etype == CG_KEY_DOWN && !trigger { hold.armed = false; }
    }
    let app = APP_HANDLE.lock().unwrap().clone();
    if let Some(app) = app { activate_shortcut_hold(&app); }
    // Keep shortcut repeats from being interpreted as hint characters while
    // waiting for its modifiers to be released.
    trigger && etype == CG_KEY_DOWN && repeat && active
        && SHORTCUT_HOLD.lock().unwrap().armed
}

fn shortcut_keycode(code: tauri_plugin_global_shortcut::Code) -> Option<u16> {
    use tauri_plugin_global_shortcut::Code::*;
    Some(match code {
        KeyA => 0, KeyS => 1, KeyD => 2, KeyF => 3, KeyH => 4, KeyG => 5,
        KeyZ => 6, KeyX => 7, KeyC => 8, KeyV => 9, KeyB => 11, KeyQ => 12,
        KeyW => 13, KeyE => 14, KeyR => 15, KeyY => 16, KeyT => 17,
        Digit1 => 18, Digit2 => 19, Digit3 => 20, Digit4 => 21, Digit6 => 22, Digit5 => 23,
        Equal => 24, Digit9 => 25, Digit7 => 26, Minus => 27, Digit8 => 28, Digit0 => 29,
        BracketRight => 30, KeyO => 31, KeyU => 32, BracketLeft => 33, KeyI => 34, KeyP => 35,
        Enter => 36, KeyL => 37, KeyJ => 38, Quote => 39, KeyK => 40, Semicolon => 41,
        Backslash => 42, Comma => 43, Slash => 44, KeyN => 45, KeyM => 46, Period => 47,
        Backquote => 50,
        _ => return None,
    })
}

pub fn is_ready() -> bool {
    READY.load(Ordering::Acquire)
}

const CG_KEY_DOWN: u32 = 10;
const CG_KEY_UP: u32 = 11;
const CG_FLAGS_CHANGED: u32 = 12;
const CG_TAP_DISABLED_TIMEOUT: u32 = 0xFFFF_FFFE;
const CG_TAP_DISABLED_USER: u32 = 0xFFFF_FFFF;

static APP_HANDLE: Lazy<Mutex<Option<tauri::AppHandle>>> = Lazy::new(|| Mutex::new(None));

thread_local! {
    static TAP_STATE: RefCell<Option<TapState>> = RefCell::new(None);
    static MODIFIER_TAP: RefCell<ModifierTap> = RefCell::new(ModifierTap::default());
}

struct TapState {
    mach_port: CFMachPort,
    _loop_source: core_foundation::runloop::CFRunLoopSource,
}

fn keycode_to_name(keycode: u16) -> Option<String> {
    let map = config::keyboard::VIRTUAL_KEY_MAP.lock().unwrap();
    map.get(&keycode).cloned()
}

fn unicode_from_event(event: CGEventRef) -> Option<String> {
    let mut buf = [0u16; 16];
    let mut len: libc::c_ulong = 0;
    unsafe {
        CGEventKeyboardGetUnicodeString(
            event,
            buf.len() as libc::c_ulong,
            &mut len,
            buf.as_mut_ptr(),
        );
    }
    if len == 0 {
        return None;
    }
    let n = (len as usize).min(buf.len());
    String::from_utf16(&buf[..n]).ok()
}

fn event_to_key_string(event: CGEventRef, keycode: u16) -> Option<String> {
    // Prefer named key from VIRTUAL_KEY_MAP (Space, Enter, Esc, etc.)
    if let Some(name) = keycode_to_name(keycode) {
        return Some(name);
    }
    // Fall back to unicode for regular character keys
    if let Some(mut s) = unicode_from_event(event) {
        s.retain(|c| !c.is_control());
        if let Some(c) = s.chars().next() {
            if s.chars().count() == 1 {
                if c.is_ascii_alphabetic() {
                    return Some(c.to_uppercase().collect());
                }
                return Some(c.to_string());
            }
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    None
}

// Decode the event being handled. Querying session key state inside the tap
// can still report the previous state, especially for remapped/synthetic keys.
fn modifier_down_from_flags(keycode: u16, flags: u64) -> Option<bool> {
    let (generic, left, right, own) = match keycode {
        56 => (0x20000, 0x2, 0x4, 0x2),
        60 => (0x20000, 0x2, 0x4, 0x4),
        59 => (0x40000, 0x1, 0x2000, 0x1),
        62 => (0x40000, 0x1, 0x2000, 0x2000),
        58 => (0x80000, 0x20, 0x40, 0x20),
        61 => (0x80000, 0x20, 0x40, 0x40),
        55 => (0x100000, 0x8, 0x10, 0x8),
        54 => (0x100000, 0x8, 0x10, 0x10),
        57 => return Some(flags & 0x10000 != 0),
        _ => return None,
    };
    Some(if flags & (left | right) != 0 { flags & own != 0 } else { flags & generic != 0 })
}

fn tap_has_other_modifiers(keycode: u16, flags: u64) -> bool {
    let (own_group, opposite) = match keycode {
        56 => (0x20000, 0x4), 60 => (0x20000, 0x2),
        59 => (0x40000, 0x2000), 62 => (0x40000, 0x1),
        58 => (0x80000, 0x40), 61 => (0x80000, 0x20),
        55 => (0x100000, 0x10), 54 => (0x100000, 0x8),
        _ => return true,
    };
    // Every other modifier (including the opposite-side key and Fn) cancels.
    // Caps Lock may remain on without preventing a tap.
    flags & (((0x20000 | 0x40000 | 0x80000 | 0x100000 | 0x800000) & !own_group) | opposite) != 0
}

unsafe fn other_input_is_held(keycode: u16, flags: u64) -> bool {
    tap_has_other_modifiers(keycode, flags)
        || (0..128).any(|code| code != keycode && code != 57 && CGEventSourceKeyState(0, code))
        || (0..32).any(|button| CGEventSourceButtonState(0, button))
}

unsafe fn observe_modifier_tap(etype: u32, event: CGEventRef, in_hint: bool) -> bool {
    MODIFIER_TAP.with(|state| {
        let mut tap = state.borrow_mut();
        if in_hint || !SINGLE_KEY_ENABLED.load(Ordering::Relaxed) {
            tap.reset();
            return false;
        }
        let keycode = SINGLE_KEY_CODE.load(Ordering::Relaxed);
        if etype == CG_FLAGS_CHANGED
            && CGEventGetIntegerValueField(event, EventField::KEYBOARD_EVENT_KEYCODE) == i64::from(keycode)
        {
            let flags = CGEventGetFlags(event);
            let eligible = !other_input_is_held(keycode, flags);
            let context = foreground_context();
            if modifier_down_from_flags(keycode, flags) == Some(true) {
                tap.press(Instant::now(), eligible, context);
            } else {
                return tap.release(Instant::now(), eligible, context);
            }
        } else {
            // Keyboard, modifiers, mouse buttons, drag and scroll all cancel.
            // Only key codes/flags are observed here; typed text is never decoded.
            tap.cancel();
        }
        false
    })
}

unsafe extern "C" fn tap_callback(
    _proxy: CGEventTapProxy,
    etype: u32,
    event_ref: CGEventRef,
    _user_info: *const c_void,
) -> CGEventRef {
    match etype {
        CG_TAP_DISABLED_TIMEOUT | CG_TAP_DISABLED_USER => {
            MODIFIER_TAP.with(|state| state.borrow_mut().reset());
            *SHORTCUT_HOLD.lock().unwrap() = ShortcutHold::default();
            READY.store(false, Ordering::Release);
            TAP_STATE.with(|c| {
                if let Some(state) = c.borrow().as_ref() {
                    CGEventTapEnable(state.mach_port.as_concrete_TypeRef(), true);
                    READY.store(CGEventTapIsEnabled(state.mach_port.as_concrete_TypeRef()), Ordering::Release);
                    warn!("[mac tap] re-enabled after disable (etype={})", etype);
                }
            });
            return event_ref;
        }
        _ => {}
    }
    if event_ref.is_null() {
        return event_ref;
    }

    let in_hint = KEYBOARD_STATE.lock().map(|state| state.in_ctrl_session).unwrap_or(false);
    if observe_shortcut_hold(etype, event_ref, in_hint) { return std::ptr::null(); }
    if observe_modifier_tap(etype, event_ref, in_hint) {
        // Forward both modifier edges unchanged so the foreground app never sees
        // a stuck modifier. Start Hint only after the release has been recognized.
        let app = APP_HANDLE.lock().ok().and_then(|guard| guard.clone());
        if let Some(app) = app {
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                tauri::async_runtime::spawn(async move {
                    crate::hint::show_hints(window).await;
                });
            }
        }
        return event_ref;
    }

    // Observe releases without intercepting the mouse or decoding ordinary typing.
    // Both physical and injected clicks use this path, including Dock/Space changes.
    if matches!(etype, 2 | 4 | 26) {
        let point = unsafe { CGEventGetLocation(event_ref) };
        crate::element::request_refresh_at(point.x as i32, point.y as i32);
        return event_ref;
    }

    // Outside Hint mode, forward input without decoding or logging typed text.
    // Tap-disable notifications above still recover the capture connection.
    if !in_hint || !matches!(etype, CG_KEY_DOWN | CG_KEY_UP | CG_FLAGS_CHANGED) {
        return event_ref;
    }

    let keycode = unsafe {
        CGEventGetIntegerValueField(event_ref, EventField::KEYBOARD_EVENT_KEYCODE) as u16
    };

    let (key_opt, is_down) = match etype {
        CG_KEY_DOWN => (event_to_key_string(event_ref, keycode), true),
        CG_KEY_UP => (event_to_key_string(event_ref, keycode), false),
        CG_FLAGS_CHANGED => {
            let name = keycode_to_name(keycode);
            let down = modifier_down_from_flags(keycode, CGEventGetFlags(event_ref))
                .unwrap_or_else(|| CGEventSourceKeyState(0, keycode));
            (name, down)
        }
        _ => return event_ref,
    };

    let is_autorepeat = etype == CG_KEY_DOWN
        && unsafe { CGEventGetIntegerValueField(event_ref, EventField::KEYBOARD_EVENT_AUTOREPEAT) } != 0;
    if crate::input::keyboard::consume_handoff_key(keycode as u32, is_down, is_autorepeat) {
        // BH also needs the release to clear its consumed-key bookkeeping when
        // this tap precedes BH's tap. SB alone must ignore the old release.
        return if is_down { std::ptr::null() } else { event_ref };
    }

    let key_opt = if keycode == SHORTCUT_CODE.load(Ordering::Relaxed)
        && KEYBOARD_STATE.lock().unwrap().cursor_hold_key.is_some() {
        Some(SHORTCUT_NAME.lock().unwrap().clone())
    } else { key_opt };
    let Some(key) = key_opt else {
        debug!(
            "[mac tap] skip (no key string) etype={} keycode={}",
            etype, keycode
        );
        return event_ref;
    };

    debug!(
        "[mac tap] etype={} keycode={} key={} is_down={}",
        etype, keycode, key, is_down
    );

    if is_autorepeat {
        if let Ok(s) = KEYBOARD_STATE.lock() {
            if !s.in_ctrl_session {
                return event_ref;
            }
            // In ctrl session: fall through to handle_keyboard_event
            // so held keys (drag/scroll/translate) repeat continuously
        } else {
            return event_ref;
        }
    }

    // For non-modifier key events, sync actual modifier state from OS
    // to fix stale hold_keys caused by missed FLAGS_CHANGED events
    // (e.g., after global shortcut Alt+H consumes the Alt release)
    if etype == CG_KEY_DOWN || etype == CG_KEY_UP {
        if let Ok(mut state) = KEYBOARD_STATE.lock() {
            let modifier_codes = config::keyboard::get_modifier_keycodes();
            for (code, name) in &modifier_codes {
                let pressed = !state.handoff_keys.contains(*code as u32)
                    && unsafe { CGEventSourceKeyState(0, *code) };
                let old = state.hold_keys.get(name.as_str()).copied().unwrap_or(false);
                if old != pressed {
                    debug!(
                        "[mac tap] sync modifier {}={} (was {})",
                        name, pressed, old
                    );
                    state.hold_keys.insert(name.clone(), pressed);
                }
            }
        }
    }

    if let Ok(guard) = APP_HANDLE.lock() {
        if let Some(app_handle) = guard.as_ref() {
            let swallowed = handle_keyboard_event(app_handle, &key, is_down);
            debug!(
                "[mac tap] handle_keyboard_event key={} -> swallowed={}",
                key, swallowed
            );
            if swallowed {
                // Carbon must see the shortcut key's release even when hold
                // consumes it. Otherwise its pressed state survives an Alt-up
                // handoff and the next chord can be lost. A key-up cannot type
                // a hint or trigger the target application's key-down action.
                if etype == CG_KEY_UP && keycode == SHORTCUT_CODE.load(Ordering::Relaxed) {
                    return event_ref;
                }
                return std::ptr::null();
            }
        } else {
            debug!("[mac tap] APP_HANDLE empty, not dispatching");
        }
    } else {
        warn!("[mac tap] APP_HANDLE lock poisoned");
    }

    event_ref
}

pub fn init(app_handle: tauri::AppHandle) {
    if RUNNING.swap(true, Ordering::AcqRel) {
        return;
    }
    if let Some(config) = config::get_config() {
        SINGLE_KEY_CODE.store(config.keybinding.single_key.keycode(), Ordering::Relaxed);
        SINGLE_KEY_ENABLED.store(config.keybinding.single_key_enabled, Ordering::Relaxed);
        if config.keybinding.hotkey_enabled {
            use std::str::FromStr;
            use tauri_plugin_global_shortcut::{Shortcut, Modifiers};
            if let Ok(shortcut) = Shortcut::from_str(&config.keybinding.hotkey_buoy) {
                let code = shortcut_keycode(shortcut.key).or_else(|| config.keyboard.available_key.iter().find_map(|(name, code)|
                    Shortcut::from_str(name).ok().filter(|key| key.key == shortcut.key).map(|_| *code)));
                if let Some(code) = code {
                    let mut flags = 0;
                    for (modifier, flag) in [(Modifiers::SHIFT, 0x20000), (Modifiers::CONTROL, 0x40000),
                        (Modifiers::ALT, 0x80000), (Modifiers::SUPER, 0x100000)] {
                        if shortcut.mods.contains(modifier) { flags |= flag; }
                    }
                    let name = config.keyboard.available_key.iter().find(|(_, value)| **value == code)
                        .map(|(name, _)| name.clone()).unwrap_or_else(|| {
                            let key = shortcut.key.to_string();
                            key.strip_prefix("Key").or_else(|| key.strip_prefix("Digit")).unwrap_or(&key).to_string()
                        });
                    *SHORTCUT_NAME.lock().unwrap() = name;
                    SHORTCUT_CODE.store(code, Ordering::Relaxed);
                    SHORTCUT_MODIFIERS.store(flags, Ordering::Relaxed);
                }
            }
        }

    }
    *APP_HANDLE.lock().unwrap() = Some(app_handle);

    if let Ok(p) = std::env::current_exe() {
        info!("[mac tap] installing for executable: {}", p.display());
    }

    std::thread::Builder::new()
        .name("keyboard-tap".into())
        .spawn(|| {
            let mut mask =
                (1u64 << CG_KEY_DOWN) | (1u64 << CG_KEY_UP) | (1u64 << CG_FLAGS_CHANGED)
                | (1u64 << 2) | (1u64 << 4) | (1u64 << 26);
            if SINGLE_KEY_ENABLED.load(Ordering::Relaxed) {
                mask |= (1u64 << 1) | (1u64 << 3) | (1u64 << 6) | (1u64 << 7)
                    | (1u64 << 22) | (1u64 << 25) | (1u64 << 27);
            }

            let port_ref;
            let mut recovery_shown = false;
            loop {
                if !RUNNING.load(Ordering::Acquire) {
                    return;
                }
                let p = unsafe {
                    CGEventTapCreate(0, 0, 0, mask, tap_callback, std::ptr::null())
                };
                if !p.is_null() {
                    port_ref = p;
                    break;
                }
                let exe = std::env::current_exe()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();

                warn!("[mac tap] installation failed: accessibility={}, input_monitoring={}, executable={}",
                    crate::macos_access::is_accessibility_trusted(),
                    crate::macos_access::has_input_monitoring_access(), exe);
                if !recovery_shown {
                    if let Some(app) = APP_HANDLE.lock().unwrap().as_ref() {
                        use tauri::Manager;
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                        }
                    }
                    recovery_shown = true;
                }
                std::thread::sleep(std::time::Duration::from_secs(3));
            }

            let mach_port = unsafe { CFMachPort::wrap_under_create_rule(port_ref) };
            let loop_source = match mach_port.create_runloop_source(0) {
                Ok(s) => s,
                Err(()) => {
                    error!("[mac tap] create_runloop_source failed");
                    return;
                }
            };

            let rl = CFRunLoop::get_current();
            rl.add_source(&loop_source, unsafe { kCFRunLoopCommonModes });
            unsafe {
                CGEventTapEnable(mach_port.as_concrete_TypeRef(), true);
            }

            TAP_RUNLOOP.store(
                rl.as_concrete_TypeRef() as *mut c_void,
                Ordering::Release,
            );

            TAP_STATE.with(|c| {
                *c.borrow_mut() = Some(TapState {
                    mach_port,
                    _loop_source: loop_source,
                });
            });

            info!("[mac tap] CGEventTap installed on dedicated thread");
            READY.store(true, Ordering::Release);
            unsafe { CFRunLoopRun(); }
            READY.store(false, Ordering::Release);
            TAP_RUNLOOP.store(std::ptr::null_mut(), Ordering::Release);
            TAP_STATE.with(|c| {
                if let Some(state) = c.borrow_mut().take() {
                    unsafe { CGEventTapEnable(state.mach_port.as_concrete_TypeRef(), false); }
                }
            });
            info!("[mac tap] dedicated thread exiting");
        })
        .expect("failed to spawn keyboard-tap thread");
}

pub fn cleanup() {
    *SHORTCUT_HOLD.lock().unwrap() = ShortcutHold::default();
    RUNNING.store(false, Ordering::Release);
    READY.store(false, Ordering::Release);
    let rl_ptr = TAP_RUNLOOP.swap(std::ptr::null_mut(), Ordering::AcqRel);
    if !rl_ptr.is_null() {
        unsafe { CFRunLoopStop(rl_ptr); }
    }
    *APP_HANDLE.lock().unwrap() = None;
}

#[cfg(test)]
mod tests {
    use super::{modifier_down_from_flags, tap_has_other_modifiers};

    #[test]
    fn shortcut_hold_requires_modifier_release_with_trigger_still_down() {
        let mut hold = super::ShortcutHold::default();
        hold.observe(true, true, true, false);
        assert!(!hold.take_ready(true));
        hold.observe(false, false, false, false);
        assert!(!hold.take_ready(false)); // Async session startup has not finished.
        assert!(hold.take_ready(true));
        assert!(!hold.take_ready(true)); // Enter exactly once.

        hold.observe(true, true, true, false);
        hold.observe(true, false, true, false); // Q released before Alt.
        hold.observe(false, false, false, false);
        assert!(!hold.take_ready(true));

        hold.observe(true, true, true, false);
        hold.observe(false, true, true, true); // Another key cancels the handoff.
        hold.observe(false, false, false, false);
        assert!(!hold.take_ready(true));
        assert_eq!(super::shortcut_keycode(tauri_plugin_global_shortcut::Code::KeyQ), Some(12));
    }

    #[test]
    fn option_tap_allows_caps_lock_but_rejects_every_other_modifier() {
        assert!(!tap_has_other_modifiers(58, 0x80020));
        assert!(!tap_has_other_modifiers(58, 0x90020));
        for other in [0x20000, 0x40000, 0x100000, 0x800000, 0x40] {
            assert!(tap_has_other_modifiers(58, 0x80020 | other));
        }
        assert_eq!(modifier_down_from_flags(58, 0x80040), Some(false));
        assert_eq!(modifier_down_from_flags(58, 0x80060), Some(true));
    }

    #[test]
    fn every_selectable_modifier_distinguishes_its_opposite_side() {
        use crate::config::keybinding::SingleModifierKey::*;
        for (key, flags, opposite) in [
            (LeftOption, 0x80020, 0x40), (RightOption, 0x80040, 0x20),
            (LeftCommand, 0x100008, 0x10), (RightCommand, 0x100010, 0x8),
            (LeftControl, 0x40001, 0x2000), (RightControl, 0x42000, 0x1),
            (LeftShift, 0x20002, 0x4), (RightShift, 0x20004, 0x2),
        ] {
            assert_eq!(modifier_down_from_flags(key.keycode(), flags), Some(true));
            assert_eq!(modifier_down_from_flags(key.keycode(), 0), Some(false));
            assert!(!tap_has_other_modifiers(key.keycode(), flags));
            assert!(tap_has_other_modifiers(key.keycode(), flags | opposite));
        }
    }

    #[test]
    fn modifier_events_use_their_own_flags_even_before_session_state_updates() {
        assert_eq!(modifier_down_from_flags(60, 0x20000), Some(true));
        assert_eq!(modifier_down_from_flags(60, 0), Some(false));
        assert_eq!(modifier_down_from_flags(58, 0x80000), Some(true));
        assert_eq!(modifier_down_from_flags(58, 0), Some(false));
    }

    #[test]
    fn releasing_one_shift_preserves_the_other_shift() {
        assert_eq!(modifier_down_from_flags(56, 0x20004), Some(false));
        assert_eq!(modifier_down_from_flags(60, 0x20004), Some(true));
        assert_eq!(modifier_down_from_flags(56, 0x20002), Some(true));
        assert_eq!(modifier_down_from_flags(60, 0x20002), Some(false));
    }
}
