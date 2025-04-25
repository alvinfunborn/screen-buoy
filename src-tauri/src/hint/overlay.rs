use crate::create_overlay_window;
use crate::monitor::MONITORS_STORAGE;
use log::{error, info};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
};

// 静态变量，保存所有overlay窗口的句柄
pub static OVERLAY_HANDLES_STORAGE: Lazy<Mutex<HashMap<String, i64>>> =
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

pub fn create_overlay_windows(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let monitors = MONITORS_STORAGE.lock().unwrap();

    info!(
        "开始为每个显示器创建overlay窗口, 当前显示器数量: {}",
        monitors.len()
    );
    info!("显示器信息: {:?}", monitors);

    // 清空之前保存的窗口句柄
    if let Ok(mut handles) = OVERLAY_HANDLES_STORAGE.lock() {
        info!("清空之前的窗口句柄存储");
        handles.clear();
    }

    // 为每个显示器创建一个overlay窗口
    for (index, monitor) in monitors.iter().enumerate() {
        let window_label = get_overlay_window_label(index);
        info!("准备创建窗口 {}", window_label);
        if let Err(e) = create_overlay_window(app_handle, &window_label, &monitor) {
            error!("创建overlay窗口失败: {}", e);
        }
    }

    info!("所有overlay窗口创建完成");
    Ok(())
}

// 确保所有overlay窗口都在最顶层，直接使用保存的窗口句柄
pub fn ensure_all_overlays_topmost() {
    if let Ok(handles) = OVERLAY_HANDLES_STORAGE.lock() {
        info!("准备设置所有overlay窗口置顶，当前窗口数: {}", handles.len());
        for (label, &hwnd_raw) in handles.iter() {
            #[cfg(target_os = "windows")]
            unsafe {
                info!("设置窗口 {} 置顶", label);
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
        info!("所有窗口置顶设置完成");
    }
}
