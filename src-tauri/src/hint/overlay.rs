use crate::monitor::MONITORS_STORAGE;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Manager;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
};

// 静态变量，保存所有overlay窗口的句柄
static OVERLAY_HANDLES_STORAGE: Lazy<Mutex<HashMap<String, i64>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub const OVERLAY_WINDOW_PREFIX: &str = "overlay_";

pub fn get_overlay_window_label(monitor_index: usize) -> String {
    format!("{}{}", OVERLAY_WINDOW_PREFIX, monitor_index)
}

pub fn get_overlay_monitor_id(window_label: &str) -> usize {
    window_label
        .replace(OVERLAY_WINDOW_PREFIX, "")
        .parse::<usize>()
        .unwrap()
}

pub async fn create_overlay_windows(app_handle: tauri::AppHandle) -> Result<(), String> {
    let monitors = MONITORS_STORAGE.lock().unwrap();

    println!("开始为每个显示器创建overlay窗口");

    // 清空之前保存的窗口句柄
    if let Ok(mut handles) = OVERLAY_HANDLES_STORAGE.lock() {
        handles.clear();
    }

    // 为每个显示器创建一个overlay窗口
    for (index, monitor) in monitors.iter().enumerate() {
        let window_label = get_overlay_window_label(index);

        // 如果已存在，先关闭
        if let Some(existing_window) = app_handle.get_webview_window(&window_label) {
            println!("关闭已存在的overlay窗口: {}", window_label);
            existing_window.close().map_err(|e| e.to_string())?;
        }

        println!(
            "创建overlay窗口 {} 在显示器 {}: 位置({}, {}), 大小{}x{}",
            window_label, monitor.id, monitor.x, monitor.y, monitor.width, monitor.height
        );

        let window = tauri::WebviewWindow::builder(
            &app_handle,
            window_label.clone(),
            tauri::WebviewUrl::App("overlay.html".into()),
        )
        .title(window_label.clone())
        .transparent(true)
        .decorations(false)
        // .skip_taskbar(true)
        .always_on_top(true)
        .position(monitor.x as f64, monitor.y as f64)
        .inner_size(monitor.width as f64, monitor.height as f64)
        .focused(false)
        .build()
        .map_err(|e| format!("创建overlay窗口失败: {}", e))?;

        // 设置窗口为鼠标穿透并确保在最顶层
        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{
                GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT,
            };
            if let Ok(hwnd) = window.hwnd() {
                let hwnd_raw = hwnd.0;
                let style = GetWindowLongW(HWND(hwnd_raw as *mut _), GWL_EXSTYLE);
                SetWindowLongW(
                    HWND(hwnd_raw as *mut _),
                    GWL_EXSTYLE,
                    style | (WS_EX_TRANSPARENT.0 | WS_EX_LAYERED.0) as i32,
                );

                // 保存窗口句柄
                if let Ok(mut handles) = OVERLAY_HANDLES_STORAGE.lock() {
                    handles.insert(window_label, hwnd_raw as i64);
                }
            }
        }

        #[cfg(debug_assertions)] {
            window.open_devtools();
        }
    }

    println!("所有overlay窗口创建完成");
    Ok(())
}

// 确保所有overlay窗口都在最顶层，直接使用保存的窗口句柄
pub fn ensure_all_overlays_topmost() {
    if let Ok(handles) = OVERLAY_HANDLES_STORAGE.lock() {
        for (_, &hwnd_raw) in handles.iter() {
            #[cfg(target_os = "windows")]
            unsafe {
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
