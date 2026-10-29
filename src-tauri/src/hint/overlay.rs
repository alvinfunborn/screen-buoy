use crate::create_overlay_window;
use crate::monitor::MONITORS_STORAGE;
#[cfg(target_os = "windows")]
use log::debug;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::{Condvar, Mutex};
use std::time::Duration;
use tauri::Manager;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
};

// 静态变量，保存所有overlay窗口的句柄
pub static OVERLAY_HANDLES_STORAGE: Lazy<Mutex<HashMap<String, i64>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub const OVERLAY_WINDOW_PREFIX: &str = "overlay_";

#[derive(Default)]
struct OverlayReadiness {
    labels: Mutex<HashSet<String>>,
    changed: Condvar,
}

impl OverlayReadiness {
    fn mark(&self, label: String) {
        self.labels.lock().unwrap().insert(label);
        self.changed.notify_all();
    }

    fn wait(&self, expected: &[String], timeout: Duration) -> bool {
        if expected.is_empty() {
            return false;
        }
        let (labels, _) = self.changed.wait_timeout_while(
            self.labels.lock().unwrap(), timeout,
            |ready| expected.iter().any(|label| !ready.contains(label)),
        ).unwrap();
        expected.iter().all(|label| labels.contains(label))
    }
}

static OVERLAY_READINESS: Lazy<OverlayReadiness> = Lazy::new(OverlayReadiness::default);

/// The page acknowledges only after styles and every hint listener are ready.
#[tauri::command]
pub fn overlay_ready(window: tauri::WebviewWindow) {
    if window.label().starts_with(OVERLAY_WINDOW_PREFIX) {
        OVERLAY_READINESS.mark(window.label().to_owned());
        log::info!("[overlay] {} listeners ready", window.label());
    }
}

pub(crate) fn wait_until_ready(app: &tauri::AppHandle, timeout: Duration) -> bool {
    let labels: Vec<_> = app.webview_windows().into_keys()
        .filter(|label| label.starts_with(OVERLAY_WINDOW_PREFIX)).collect();
    OVERLAY_READINESS.wait(&labels, timeout)
}

pub fn get_overlay_window_label(monitor_index: usize) -> String {
    format!("{}{}", OVERLAY_WINDOW_PREFIX, monitor_index)
}

pub fn get_overlay_monitor_id(window_label: &str) -> usize {
    window_label
        .replace(OVERLAY_WINDOW_PREFIX, "")
        .parse::<usize>()
        .unwrap()
}

pub fn create_overlay_windows(app_handle: &tauri::AppHandle) {
    OVERLAY_READINESS.labels.lock().unwrap().clear();
    let monitors = MONITORS_STORAGE.lock().unwrap();
    // 清空之前保存的窗口句柄
    if let Ok(mut handles) = OVERLAY_HANDLES_STORAGE.lock() {
        handles.clear();
    }
    // 为每个显示器创建一个overlay窗口
    for (index, monitor) in monitors.iter().enumerate() {
        let window_label = get_overlay_window_label(index);
        create_overlay_window(app_handle, &window_label, &monitor);
    }
}

// 确保所有overlay窗口都在最顶层，直接使用保存的窗口句柄
pub fn ensure_all_overlays_topmost(_app_handle: &tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    sync_overlay_visibility(_app_handle);
    #[cfg(target_os = "windows")]
    if let Ok(handles) = OVERLAY_HANDLES_STORAGE.lock() {
        for (_label, &hwnd_raw) in handles.iter() {
            unsafe {
                debug!("[ensure_all_overlays_topmost] set overlay window topmost: {}", hwnd_raw);
                let _ = SetWindowPos(
                    HWND(hwnd_raw as *mut _),
                    Some(HWND(HWND_TOPMOST.0)),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub fn sync_overlay_visibility(app_handle: &tauri::AppHandle) {
    for (label, window) in app_handle.webview_windows() {
        if !label.starts_with(OVERLAY_WINDOW_PREFIX) {
            continue;
        }
        if let Err(e) = window.with_webview(|webview| {
            // Read the current state on the main thread. A slow second hint
            // batch must not resurrect the native overlay after Escape.
            let visible = crate::input::keyboard::KEYBOARD_STATE.lock().unwrap().in_ctrl_session;
            unsafe { crate::macos_window::set_overlay_visible(webview.ns_window(), visible); }
        }) {
            log::error!("[overlay] update native visibility failed: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn handoff_waits_for_every_page_listener() {
        let ready = Arc::new(OverlayReadiness::default());
        let pages: Vec<String> = vec!["overlay_0".into(), "overlay_1".into()];
        ready.mark(pages[0].clone());
        assert!(!ready.wait(&pages, Duration::ZERO));
        let loader = ready.clone();
        let thread = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            loader.mark("overlay_1".into());
        });
        assert!(ready.wait(&pages, Duration::from_secs(1)));
        thread.join().unwrap();
        // An already warm page does not wait for another notification.
        assert!(ready.wait(&pages, Duration::ZERO));
    }

    #[test]
    fn missing_or_recreated_pages_do_not_report_ready() {
        let ready = OverlayReadiness::default();
        assert!(!ready.wait(&[], Duration::ZERO));
        ready.mark("overlay_0".into());
        let pages = vec!["overlay_0".into()];
        ready.labels.lock().unwrap().clear();
        assert!(!ready.wait(&pages, Duration::from_millis(5)));
    }
}
