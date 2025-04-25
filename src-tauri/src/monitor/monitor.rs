use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::WebviewWindow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub id: usize,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub scale_factor: f64,
}

pub static MONITORS_STORAGE: Lazy<Mutex<Vec<MonitorInfo>>> = Lazy::new(|| Mutex::new(Vec::new()));

// 获取所有显示器信息，按照x坐标排序
fn get_sorted_monitors(window: &WebviewWindow) -> Result<Vec<MonitorInfo>, String> {
    let monitors = window.available_monitors();
    if let Err(e) = monitors {
        panic!("[get_sorted_monitors] get available monitors failed: {}", e);
    }
    let mut monitors = monitors
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(index, monitor)| {
            let position = monitor.position();
            let size = monitor.size();
            MonitorInfo {
                id: index,
                x: position.x,
                y: position.y,
                width: size.width as i32,
                height: size.height as i32,
                scale_factor: monitor.scale_factor(),
            }
        })
        .collect::<Vec<_>>();

    // 首先按y坐标排序，然后按x坐标排序
    monitors.sort_by(|a, b| {
        // 优先按y坐标排序
        match a.y.cmp(&b.y) {
            std::cmp::Ordering::Equal => a.x.cmp(&b.x), // y相同时按x排序
            other => other,
        }
    });

    for monitor in &monitors {
        info!(
            "[get_sorted_monitors] monitor: {}, position: ({}, {}), size: {}x{}, scale_factor: {}",
            monitor.id, monitor.x, monitor.y, monitor.width, monitor.height, monitor.scale_factor
        );
    }

    Ok(monitors)
}

pub fn init_monitors(window: &WebviewWindow) {
    // 获取显示器信息
    let monitors = get_sorted_monitors(window).unwrap_or_else(|e| {
        error!("[init_monitors] get sorted monitors failed: {}", e);
        Vec::new()
    });
    if let Ok(mut cached_monitors) = MONITORS_STORAGE.lock() {
        *cached_monitors = monitors;
    }
}
