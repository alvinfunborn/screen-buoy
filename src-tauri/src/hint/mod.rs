pub mod generator;
pub mod overlay;
pub mod storage;

use crate::hint::generator::HintsGenerator;
use crate::input;
use overlay::ensure_all_overlays_topmost;
use serde_json::json;
use std::collections::HashSet;
use storage::clear_hints;
use storage::save_hints;
use storage::update_hints_offset;
use tauri::Emitter;
use tauri::{Manager, WebviewWindow};

pub use generator::init_hint_text_list_storage;
pub use overlay::create_overlay_windows;
pub use overlay::OVERLAY_WINDOW_PREFIX;

pub async fn show_hints(window: WebviewWindow) -> Result<(), String> {
    // 清空之前的 hints 数据
    clear_hints();

    // 发送hints到对应的overlay窗口
    let app_handle = window.app_handle();
    let hints_generator = HintsGenerator::new();
    let mut position_set = HashSet::new();
    let mut hints_count = 0;

    // 设置键盘状态为可见
    input::keyboard::switch_keyboard_ctrl(true, Some(&app_handle));

    let monitor_hints = hints_generator.generate_hints_batch1(&mut position_set, &mut hints_count);
    for (window_label, hints) in &monitor_hints {
        if let Some(overlay_window) = app_handle.get_webview_window(window_label) {
            overlay_window
                .emit("show-hints", hints)
                .map_err(|e| e.to_string())?;
        }
        // 保存 hints 到存储
        println!("保存第一批 hints 到存储: {} 个", hints.len());
        save_hints(window_label.clone(), hints.clone()).await;
    }
    ensure_all_overlays_topmost();

    let monitor_hints = hints_generator.generate_hints_batch2(&mut position_set, &mut hints_count);
    for (window_label, hints) in &monitor_hints {
        if let Some(overlay_window) = app_handle.get_webview_window(window_label) {
            overlay_window
                .emit("show-hints2", hints)
                .map_err(|e| e.to_string())?;
        }
        // 保存 hints 到存储
        println!("保存第二批 hints 到存储: {} 个", hints.len());
        save_hints(window_label.clone(), hints.clone()).await;
    }
    ensure_all_overlays_topmost();

    Ok(())
}

pub async fn hide_hints(app_handle: tauri::AppHandle) -> Result<(), String> {
    // 获取所有overlay窗口并发送hide-hints事件
    let windows = app_handle.webview_windows();

    for (label, window) in windows {
        if label.starts_with(OVERLAY_WINDOW_PREFIX) {
            window.emit("hide-hints", ()).map_err(|e| e.to_string())?;
        }
    }

    // 设置键盘状态为不可见
    input::keyboard::switch_keyboard_ctrl(false, Some(&app_handle));

    // 清空 hints 数据
    clear_hints();
    Ok(())
}

pub async fn move_hints(
    app_handle: tauri::AppHandle,
    move_direction: (i32, i32),
) -> Result<(), String> {
    update_hints_offset(move_direction.0, move_direction.1);
    let windows = app_handle.webview_windows();
    for (label, window) in windows {
        if label.starts_with(OVERLAY_WINDOW_PREFIX) {
            println!("发送move-hints事件到窗口: {}", label);
            let json = json!({
                "x": move_direction.0,
                "y": move_direction.1
            });
            window.emit("move-hints", json).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub async fn filter_hints(app_handle: tauri::AppHandle, letters: String) -> Result<(), String> {
    let windows = app_handle.webview_windows();
    for (label, window) in windows {
        if label.starts_with(OVERLAY_WINDOW_PREFIX) {
            println!("发送filter-hints事件到窗口: {}", label);
            window
                .emit("filter-hints", letters.clone())
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
