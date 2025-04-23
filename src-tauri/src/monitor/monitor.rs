use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::Window;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub scale_factor: f64,
}

pub static MONITORS_STORAGE: Lazy<Mutex<Vec<MonitorInfo>>> = Lazy::new(|| Mutex::new(Vec::new()));

// 获取所有显示器信息，按照x坐标排序
fn get_sorted_monitors(window: &Window) -> Result<Vec<MonitorInfo>, String> {
    let mut monitors = window
        .available_monitors()
        .map_err(|e| format!("获取显示器信息失败: {}", e))?
        .into_iter()
        .enumerate()
        .map(|(index, monitor)| {
            let position = monitor.position();
            let size = monitor.size();
            MonitorInfo {
                id: format!("monitor_{}", index),
                x: position.x,
                y: position.y,
                width: size.width as i32,
                height: size.height as i32,
                scale_factor: monitor.scale_factor(),
            }
        })
        .collect::<Vec<_>>();

    // 按x坐标排序，确保显示器顺序稳定
    monitors.sort_by_key(|m| m.x);

    // 重新分配id
    for (index, monitor) in monitors.iter_mut().enumerate() {
        monitor.id = format!("monitor_{}", index);
    }

    // println!("显示器信息:");
    // for monitor in &monitors {
    //     println!(
    //         "  {}: 位置({}, {}), 大小{}x{}, 缩放{}",
    //         monitor.id, monitor.x, monitor.y, monitor.width, monitor.height, monitor.scale_factor
    //     );
    // }

    Ok(monitors)
}

pub fn init_monitors(window: &Window) {
    // 获取显示器信息
    let monitors = get_sorted_monitors(window).unwrap_or_else(|e| {
        eprintln!("获取显示器信息失败: {}", e);
        Vec::new()
    });
    if let Ok(mut cached_monitors) = MONITORS_STORAGE.lock() {
        *cached_monitors = monitors;
    }
}
