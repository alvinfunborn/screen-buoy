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
    pid: u32,
    #[serde(default)]
    native_id: u64,
    port: u16,
    token: String,
    #[serde(default = "default_action")]
    action: String,
    #[serde(default)]
    held_keys: Vec<u32>,
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
        protocol: 2,
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
    if r.version != 2
        || r.held_keys.len() > 32
        || r.held_keys.iter().any(|code| *code > 255)
        || r.port == 0
        || r.token.len() != 36
        || !r.token.bytes().all(|c| c.is_ascii_hexdigit() || c == b'-')
        || !["hints", "status", "settings"].contains(&r.action.as_str())
        || (r.action == "hints" && (r.pid == 0 || r.pid > i32::MAX as u32 || r.native_id == 0))
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
        return reply(&mut stream, request,
            Some("Screen Buoy hint pages are not ready".into()));
    }
    let target = get_all_windows()
        .into_iter()
        .find(|w| w.window_handle as u64 == request.native_id && w.owner_pid as u32 == request.pid);
    let Some(target) = target.filter(focused) else {
        return reply(
            &mut stream,
            request,
            Some("目标窗口已失效或焦点已改变 / Target window is no longer focused".into()),
        );
    };
    let window = app
        .get_webview_window("main")
        .ok_or("Screen Buoy window unavailable")?;
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let held_keys = request.held_keys.clone();
    tauri::async_runtime::spawn(async move {
        hint::show_window_hints(window, target, held_keys, sender).await;
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
            string::CFString,
        };
        #[link(name = "ApplicationServices", kind = "framework")]
        extern "C" {
            fn AXUIElementCreateSystemWide() -> CFTypeRef;
            fn AXUIElementCopyAttributeValue(
                element: CFTypeRef,
                attr: core_foundation::string::CFStringRef,
                out: *mut CFTypeRef,
            ) -> i32;
            fn AXUIElementGetPid(element: CFTypeRef, pid: *mut i32) -> i32;
            fn _AXUIElementGetWindow(element: CFTypeRef, window: *mut u32) -> i32;
            fn AXUIElementSetMessagingTimeout(element: CFTypeRef, timeout: f32) -> i32;
        }
        let system = AXUIElementCreateSystemWide();
        if system.is_null() {
            return false;
        }
        AXUIElementSetMessagingTimeout(system, 0.05);
        let mut app = std::ptr::null();
        let mut window = std::ptr::null();
        let mut pid = 0;
        let mut id = 0;
        let valid = AXUIElementCopyAttributeValue(
            system,
            CFString::new("AXFocusedApplication").as_concrete_TypeRef(),
            &mut app,
        ) == 0
            && !app.is_null()
            && AXUIElementGetPid(app, &mut pid) == 0
            && pid == target.owner_pid
            && {
                AXUIElementSetMessagingTimeout(app, 0.05);
                true
            }
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
        if !app.is_null() {
            CFRelease(app);
        }
        CFRelease(system);
        valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_incompatible_or_malformed_handoffs() {
        let args = |s: &str| vec!["sb".into(), FLAG.into(), s.into()];
        let valid = r#"{"version":2,"pid":23,"nativeId":42,"port":12222,"token":"12345678-1234-1234-1234-123456789abc"}"#;
        assert!(parse(&args(valid)).is_ok());
        for action in ["status", "settings"] {
            let setup = format!(
                r#"{{"version":2,"pid":0,"nativeId":0,"action":"{action}","port":12222,"token":"12345678-1234-1234-1234-123456789abc"}}"#
            );
            assert!(parse(&args(&setup)).is_ok());
        }
        assert!(parse(&args(&valid.replace("\"version\":2", "\"version\":1"))).is_err());
        assert!(parse(&args(&valid.replace("\"pid\":23", "\"pid\":0"))).is_err());
        let with_keys = valid.replace("\"version\":2", "\"version\":2,\"heldKeys\":[1,2,55]");
        assert_eq!(parse(&args(&with_keys)).unwrap().held_keys, vec![1, 2, 55]);
        assert!(parse(&args(&with_keys.replace("[1,2,55]", "[256]"))).is_err());
        assert!(parse(&args(&with_keys.replace("[1,2,55]", "[-1]"))).is_err());
        assert!(parse(&args(&with_keys.replace("[1,2,55]", &serde_json::to_string(&vec![1; 33]).unwrap()))).is_err());
        assert!(parse(&args("{}")).is_err());
        assert!(parse(&vec![FLAG.into()]).is_err());
    }
}
