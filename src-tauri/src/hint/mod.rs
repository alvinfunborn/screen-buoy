mod generator;
pub mod hint;
pub mod overlay;

use crate::config;
use crate::hint::generator::HintsGenerator;
use crate::input;
use log::{debug, error, info};
use overlay::ensure_all_overlays_topmost;
use serde_json::json;
use std::collections::HashSet;
use hint::clear_hints;
use hint::save_hints;
use hint::update_hints_offset;
use tauri::Emitter;
use tauri::{Manager, WebviewWindow};

pub use generator::init_hint_text_list_storage;
pub use overlay::create_overlay_windows;
pub use overlay::OVERLAY_WINDOW_PREFIX;

pub async fn show_hints(window: WebviewWindow) {
    // 清空之前的 hints 数据
    clear_hints();

    // 发送hints到对应的overlay窗口
    let app_handle = window.app_handle();
    let hints_generator = HintsGenerator::new();
    let mut position_set = HashSet::new();
    let mut hints_count = 0;

    // 设置键盘状态为监听
    input::keyboard::switch_keyboard_ctrl(true, Some(&app_handle));
    debug!("[show_hints] switch keyboard ctrl to true");
    let monitor_grid_hints = hints_generator.generate_hints_grid(&mut hints_count);
    let mut monitor_hints = hints_generator.generate_hints_batch1(&mut position_set, &mut hints_count);
    for (window_label, grid_hints) in &monitor_grid_hints {
        if let Some(hints) = monitor_hints.get_mut(window_label) {
            hints.extend(grid_hints.clone());
        } else {
            monitor_hints.insert(window_label.clone(), grid_hints.clone());
        }
    }
    for (window_label, hints) in &monitor_hints {
        if let Some(overlay_window) = app_handle.get_webview_window(window_label) {
            if let Err(e) = overlay_window.emit(
                "show-hints",
                json!({
                    "windowLabel": window_label,
                    "hints": hints
                }),
            ) {
                error!("[show_hints] show-hints failed: {}", e);
            }
            debug!("[show_hints] show {} hints to overlay window: {}", hints.len(), window_label);
        }
        // 保存 hints 到存储
        save_hints(window_label.clone(), hints.clone()).await;
    }
    if !config::get_config().unwrap().system.debug_mode {
        ensure_all_overlays_topmost();
    }

    let monitor_hints = hints_generator.generate_hints_batch2(&mut position_set, &mut hints_count);
    for (window_label, hints) in &monitor_hints {
        if let Some(overlay_window) = app_handle.get_webview_window(window_label) {
            if let Err(e) = overlay_window.emit(
                "show-hints2",
                json!({
                    "windowLabel": window_label,
                    "hints": hints
                }),
            ) {
                error!("[show_hints] show-hints2 failed: {}", e);
            }
            debug!("[show_hints] show {} hints2 to overlay window: {}", hints.len(), window_label);
        }
        // 保存 hints 到存储
        save_hints(window_label.clone(), hints.clone()).await;
    }
    if !config::get_config().unwrap().system.debug_mode {
        ensure_all_overlays_topmost();
    }
}

pub async fn hide_hints(app_handle: tauri::AppHandle) {
    // 获取所有overlay窗口并发送hide-hints事件
    let window = app_handle.get_webview_window("main").unwrap();
    if let Err(e) = window.emit("hide-hints", ()) {
        error!("[hide_hints] hide-hints failed: {}", e);
    }

    // 设置键盘状态为不可见
    input::keyboard::switch_keyboard_ctrl(false, Some(&app_handle));
    debug!("[hide_hints] switch keyboard ctrl to false");
    // 清空 hints 数据
    clear_hints();
}

pub async fn move_hints(app_handle: tauri::AppHandle, move_direction: (i32, i32)) {
    update_hints_offset(move_direction.0, move_direction.1);
    let window = app_handle.get_webview_window("main").unwrap();
    let json = json!({
        "x": move_direction.0,
        "y": move_direction.1
    });
    if let Err(e) = window.emit("move-hints", json) {
        error!("[move_hints] move-hints failed: {}", e);
    }
}

pub async fn filter_hints(app_handle: tauri::AppHandle, letters: String) {
    let window = app_handle.get_webview_window("main").unwrap();
    if let Err(e) = window.emit("filter-hints", letters.clone()) {
        error!("[filter_hints] filter-hints failed: {}", e);
    }
}
