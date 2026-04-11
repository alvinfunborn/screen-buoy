use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Mutex;

use core_foundation::base::TCFType;
use core_foundation::mach_port::CFMachPort;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::EventField;
use log::{debug, error, info, warn};
use once_cell::sync::Lazy;

use crate::config;
use crate::input::keyboard::{handle_keyboard_event, KEYBOARD_STATE};

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
    fn CGEventSourceKeyState(state_id: u32, key_code: CGKeyCode) -> bool;
    fn CGEventKeyboardGetUnicodeString(
        event: CGEventRef,
        max_string_length: libc::c_ulong,
        actual_string_length: *mut libc::c_ulong,
        unicode_string: *mut u16,
    );
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRunLoopRun();
    fn CFRunLoopStop(rl: *const c_void);
}

static TAP_RUNLOOP: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

const CG_KEY_DOWN: u32 = 10;
const CG_KEY_UP: u32 = 11;
const CG_FLAGS_CHANGED: u32 = 12;
const CG_TAP_DISABLED_TIMEOUT: u32 = 0xFFFF_FFFE;
const CG_TAP_DISABLED_USER: u32 = 0xFFFF_FFFF;

static APP_HANDLE: Lazy<Mutex<Option<tauri::AppHandle>>> = Lazy::new(|| Mutex::new(None));

thread_local! {
    static TAP_STATE: RefCell<Option<TapState>> = RefCell::new(None);
}

struct TapState {
    mach_port: CFMachPort,
    loop_source: core_foundation::runloop::CFRunLoopSource,
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

unsafe extern "C" fn tap_callback(
    _proxy: CGEventTapProxy,
    etype: u32,
    event_ref: CGEventRef,
    _user_info: *const c_void,
) -> CGEventRef {
    if event_ref.is_null() {
        return event_ref;
    }
    match etype {
        CG_TAP_DISABLED_TIMEOUT | CG_TAP_DISABLED_USER => {
            TAP_STATE.with(|c| {
                if let Some(state) = c.borrow().as_ref() {
                    CGEventTapEnable(state.mach_port.as_concrete_TypeRef(), true);
                    warn!("[mac tap] re-enabled after disable (etype={})", etype);
                }
            });
            return event_ref;
        }
        _ => {}
    }

    let keycode = unsafe {
        CGEventGetIntegerValueField(event_ref, EventField::KEYBOARD_EVENT_KEYCODE) as u16
    };

    let (key_opt, is_down) = match etype {
        CG_KEY_DOWN => (event_to_key_string(event_ref, keycode), true),
        CG_KEY_UP => (event_to_key_string(event_ref, keycode), false),
        CG_FLAGS_CHANGED => {
            let name = keycode_to_name(keycode);
            let down = unsafe { CGEventSourceKeyState(0, keycode) };
            (name, down)
        }
        _ => return event_ref,
    };

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

    let is_autorepeat = etype == CG_KEY_DOWN
        && unsafe { CGEventGetIntegerValueField(event_ref, EventField::KEYBOARD_EVENT_AUTOREPEAT) }
            != 0;
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
                let pressed = unsafe { CGEventSourceKeyState(0, *code) };
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
    *APP_HANDLE.lock().unwrap() = Some(app_handle);

    if let Ok(p) = std::env::current_exe() {
        info!("[mac tap] installing for executable: {}", p.display());
    }

    std::thread::Builder::new()
        .name("keyboard-tap".into())
        .spawn(|| {
            let mask =
                (1u64 << CG_KEY_DOWN) | (1u64 << CG_KEY_UP) | (1u64 << CG_FLAGS_CHANGED);

            let port_ref;
            loop {
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

                if crate::macos_access::is_accessibility_trusted() {
                    warn!(
                        "[mac tap] CGEventTapCreate failed — Accessibility is granted but Input Monitoring is missing. \
                         Please enable in: System Settings > Privacy & Security > Input Monitoring. executable: {}",
                        exe
                    );
                } else {
                    warn!(
                        "[mac tap] CGEventTapCreate failed — missing Accessibility & Input Monitoring permissions. executable: {}",
                        exe
                    );
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
                    loop_source,
                });
            });

            info!("[mac tap] CGEventTap installed on dedicated thread");
            unsafe { CFRunLoopRun(); }
            info!("[mac tap] dedicated thread exiting");
        })
        .expect("failed to spawn keyboard-tap thread");
}

pub fn cleanup() {
    let rl_ptr = TAP_RUNLOOP.swap(std::ptr::null_mut(), Ordering::AcqRel);
    if !rl_ptr.is_null() {
        unsafe { CFRunLoopStop(rl_ptr); }
    }
    *APP_HANDLE.lock().unwrap() = None;
}
