//! Versioned, on-demand handoff from BeaconHop. No listener or polling service.
use crate::{
    hint, permissions,
    window::{window::get_all_windows, WindowElement},
};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use tauri::Manager;

static READY: AtomicBool = AtomicBool::new(false);
pub const FLAG: &str = "--beaconhop-request";
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    version: u8,
    #[serde(default)]
    target: Option<Target>,
    port: u16,
    token: String,
    #[serde(default = "default_action")]
    action: String,
    #[serde(default)]
    held_keys: Vec<u32>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(
    tag = "scope",
    rename_all = "lowercase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum Target {
    Window {
        pid: u32,
        native_id: u64,
    },
    Desktop {
        desktop_id: String,
        display_id: String,
    },
}
impl Target {
    fn valid(&self) -> bool {
        match self {
            Self::Window { pid, native_id } => {
                *pid > 0 && *pid <= i32::MAX as u32 && *native_id != 0
            }
            Self::Desktop {
                desktop_id,
                display_id,
            } => {
                !desktop_id.is_empty()
                    && desktop_id.len() <= 128
                    && display_id.len() <= 128
                    && !desktop_id.contains('\0')
                    && !display_id.contains('\0')
            }
        }
    }
    fn resolve(&self) -> Result<hint::HintScope, String> {
        match self {
            Self::Window { pid, native_id } => get_all_windows()
                .into_iter()
                .find(|w| w.window_handle as u64 == *native_id && w.owner_pid as u32 == *pid)
                .filter(focused)
                .map(hint::HintScope::Window)
                .ok_or_else(|| {
                    "目标窗口已失效或焦点已改变 / Target window is no longer focused".into()
                }),
            Self::Desktop {
                desktop_id,
                display_id,
            } => crate::desktop::DesktopScope::capture(desktop_id, display_id)
                .map(hint::HintScope::Desktop),
        }
    }
}
fn default_action() -> String {
    "hints".into()
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Reply<'a> {
    token: &'a str,
    error: Option<String>,
    version: &'static str,
    protocol: u8,
    permissions: permissions::PermissionStatus,
}
fn reply(stream: &mut TcpStream, request: &Request, error: Option<String>) -> Result<(), String> {
    let message = Reply {
        token: &request.token,
        error,
        version: env!("CARGO_PKG_VERSION"),
        protocol: 3,
        permissions: permissions::get_permission_status(),
    };
    writeln!(stream, "{}", serde_json::to_string(&message).unwrap()).map_err(|e| e.to_string())
}

pub fn is_requested(args: &[String]) -> bool {
    args.iter().any(|a| a == FLAG)
}
fn parse(args: &[String]) -> Result<Request, String> {
    let raw = args
        .windows(2)
        .find(|a| a[0] == FLAG)
        .ok_or("Missing handoff request")?;
    if raw[1].len() > 2048 {
        return Err("Oversized handoff request".into());
    }
    let r: Request = serde_json::from_str(&raw[1]).map_err(|e| e.to_string())?;
    if r.version != 3
        || r.held_keys.len() > 32
        || r.held_keys.iter().any(|code| *code > 255)
        || r.port == 0
        || r.token.len() != 36
        || !r.token.bytes().all(|c| c.is_ascii_hexdigit() || c == b'-')
        || !["hints", "status", "settings"].contains(&r.action.as_str())
        || (r.action == "hints" && !r.target.as_ref().is_some_and(Target::valid))
        || (r.action != "hints" && (r.target.is_some() || !r.held_keys.is_empty()))
    {
        return Err("Invalid BeaconHop request".into());
    }
    Ok(r)
}

pub fn initialized(app: &tauri::AppHandle) {
    READY.store(true, Ordering::Release);
    handle(app, &std::env::args().collect::<Vec<_>>());
}
pub fn handle(app: &tauri::AppHandle, args: &[String]) -> bool {
    if !is_requested(args) {
        return false;
    }
    let request = match parse(args) {
        Ok(r) => r,
        Err(error) => {
            log::warn!("[beaconhop] {error}");
            return true;
        }
    };
    let app = app.clone();
    std::thread::spawn(move || {
        if let Err(error) = receive(&app, &request) {
            log::warn!("[beaconhop] {error}");
        }
    });
    true
}

fn receive(app: &tauri::AppHandle, request: &Request) -> Result<(), String> {
    let started = Instant::now();
    while !READY.load(Ordering::Acquire) {
        if started.elapsed() > Duration::from_secs(4) {
            return Err("Screen Buoy is not initialized".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    // A cold launch installs its hook asynchronously. Do not report a transient
    // false negative when Accessibility is already authorized.
    while permissions::get_permission_status().accessibility
        && !permissions::get_permission_status().ready
        && started.elapsed() < Duration::from_secs(2)
    {
        std::thread::sleep(Duration::from_millis(20));
    }
    // A native WebviewWindow existing does not mean its JavaScript listeners
    // exist yet. Wait before the hello/begin handshake so its short ack timeout
    // is reserved for activating the requested hint session.
    let overlays_ready = request.action != "hints"
        || !permissions::get_permission_status().ready
        || hint::overlay::wait_until_ready(app, Duration::from_secs(2));
    let mut stream = TcpStream::connect_timeout(
        &([127, 0, 0, 1], request.port).into(),
        Duration::from_secs(1),
    )
    .map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(1)))
        .map_err(|e| e.to_string())?;
    reply(&mut stream, request, None)?;
    let mut command = String::new();
    BufReader::new((&mut stream).take(32))
        .read_line(&mut command)
        .map_err(|e| e.to_string())?;
    if command != "begin\n" {
        return Ok(());
    }
    if request.action == "settings" {
        crate::show_settings_window(app);
    }
    if request.action != "hints" {
        return reply(&mut stream, request, None);
    }
    if !permissions::get_permission_status().ready {
        return reply(
            &mut stream,
            request,
            Some("请在 Screen Buoy 中完成辅助功能授权 / Complete Screen Buoy access setup".into()),
        );
    }
    if !overlays_ready {
        return reply(
            &mut stream,
            request,
            Some("Screen Buoy hint pages are not ready".into()),
        );
    }
    let target = match request
        .target
        .as_ref()
        .ok_or("Missing hint target")?
        .resolve()
    {
        Ok(target) => target,
        Err(error) => return reply(&mut stream, request, Some(error)),
    };
    let window = app
        .get_webview_window("main")
        .ok_or("Screen Buoy window unavailable")?;
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let held_keys = request.held_keys.clone();
    tauri::async_runtime::spawn(async move {
        hint::show_scoped_hints(window, target, held_keys, sender).await;
    });
    match receiver.recv_timeout(Duration::from_secs(2)) {
        Ok(result) => reply(&mut stream, request, result.err()),
        Err(_) => reply(
            &mut stream,
            request,
            Some("Screen Buoy hint setup timed out".into()),
        ),
    }
}

pub fn focused(target: &WindowElement) -> bool {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowThreadProcessId,
        };
        let window = GetForegroundWindow();
        let mut pid = 0;
        GetWindowThreadProcessId(window, Some(&mut pid));
        window.0 as i64 == target.window_handle && pid == target.owner_pid as u32
    }
    #[cfg(target_os = "macos")]
    unsafe {
        use core_foundation::{
            base::{CFRelease, CFTypeRef, TCFType},
            boolean::CFBoolean,
            string::CFString,
        };
        #[link(name = "ApplicationServices", kind = "framework")]
        extern "C" {
            fn AXUIElementCreateApplication(pid: i32) -> CFTypeRef;
            fn AXUIElementCopyAttributeValue(
                element: CFTypeRef,
                attr: core_foundation::string::CFStringRef,
                out: *mut CFTypeRef,
            ) -> i32;
            fn _AXUIElementGetWindow(element: CFTypeRef, window: *mut u32) -> i32;
            fn AXUIElementSetMessagingTimeout(element: CFTypeRef, timeout: f32) -> i32;
        }
        // System-wide AXFocusedApplication can return AXErrorNoValue on macOS
        // even when the app's AXFrontmost/AXFocusedWindow are available. Resolve
        // the requested PID directly, then verify both foreground and window ID.
        let app = AXUIElementCreateApplication(target.owner_pid);
        if app.is_null() {
            return false;
        }
        AXUIElementSetMessagingTimeout(app, 0.05);
        let mut frontmost = std::ptr::null();
        let mut window = std::ptr::null();
        let mut id = 0;
        let valid = AXUIElementCopyAttributeValue(
            app,
            CFString::new("AXFrontmost").as_concrete_TypeRef(),
            &mut frontmost,
        ) == 0
            && frontmost == CFBoolean::true_value().as_CFTypeRef()
            && AXUIElementCopyAttributeValue(
                app,
                CFString::new("AXFocusedWindow").as_concrete_TypeRef(),
                &mut window,
            ) == 0
            && !window.is_null()
            && _AXUIElementGetWindow(window, &mut id) == 0
            && id as i64 == target.window_handle;
        if !window.is_null() {
            CFRelease(window);
        }
        if !frontmost.is_null() {
            CFRelease(frontmost);
        }
        CFRelease(app);
        valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_incompatible_or_malformed_handoffs() {
        let args = |s: &str| vec!["sb".into(), FLAG.into(), s.into()];
        let valid = r#"{"version":3,"target":{"scope":"window","pid":23,"nativeId":42},"port":12222,"token":"12345678-1234-1234-1234-123456789abc"}"#;
        assert!(parse(&args(valid)).is_ok());
        for action in ["status", "settings"] {
            let setup = format!(
                r#"{{"version":3,"action":"{action}","port":12222,"token":"12345678-1234-1234-1234-123456789abc"}}"#
            );
            assert!(parse(&args(&setup)).is_ok());
        }
        assert!(parse(&args(&valid.replace("\"version\":3", "\"version\":2"))).is_err());
        assert!(parse(&args(&valid.replace("\"pid\":23", "\"pid\":0"))).is_err());
        let with_keys = valid.replace("\"version\":3", "\"version\":3,\"heldKeys\":[1,2,55]");
        assert_eq!(parse(&args(&with_keys)).unwrap().held_keys, vec![1, 2, 55]);
        assert!(parse(&args(&with_keys.replace("[1,2,55]", "[256]"))).is_err());
        assert!(parse(&args(&with_keys.replace("[1,2,55]", "[-1]"))).is_err());
        assert!(parse(&args(
            &with_keys.replace("[1,2,55]", &serde_json::to_string(&vec![1; 33]).unwrap())
        ))
        .is_err());
        assert!(parse(&args("{}")).is_err());
        assert!(parse(&vec![FLAG.into()]).is_err());
    }
    #[test]
    fn desktop_targets_need_no_window_and_cannot_be_confused_with_window_targets() {
        let mut value = serde_json::json!({"version":3,"target":{"scope":"desktop","desktopId":"space-1","displayId":"display-2"},"port":12222,"token":"12345678-1234-1234-1234-123456789abc"});
        let parse_value = |value: &serde_json::Value| parse(&[FLAG.into(), value.to_string()]);
        assert!(matches!(
            parse_value(&value).unwrap().target,
            Some(Target::Desktop { .. })
        ));
        value["target"]["desktopId"] = "".into();
        assert!(parse_value(&value).is_err());
        value["target"] = serde_json::json!({"scope":"window","desktopId":"d","displayId":"m"});
        assert!(parse_value(&value).is_err());
        value["target"] =
            serde_json::json!({"scope":"desktop","desktopId":"d","displayId":"m","nativeId":42});
        assert!(parse_value(&value).is_err());
        value["target"] = serde_json::Value::Null;
        assert!(parse_value(&value).is_err());
    }
}
