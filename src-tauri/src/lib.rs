#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod config;
pub mod element;
pub mod hint;
pub mod input;
pub mod monitor;
mod utils;
pub mod window;

use config::{get_config_for_frontend, get_hint_styles, save_config_for_frontend};
use hint::{overlay::OVERLAY_HANDLES_STORAGE, show_hints};
use log::{error, info};
use std::str::FromStr;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_log::{Target, TargetKind};
use windows::Win32::Foundation::HWND;

pub fn init_plugins(app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    app_handle.plugin(tauri_plugin_process::init())?;
    app_handle.plugin(
        tauri_plugin_log::Builder::new()
            .targets([
                Target::new(TargetKind::Stdout),
                Target::new(TargetKind::Webview),
            ])
            .build(),
    )?;
    app_handle.plugin(tauri_plugin_positioner::init())?;
    Ok(())
}

pub fn setup_tray(
    app_handle: &AppHandle,
    config: &config::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    if !config.system.show_tray_icon {
        info!("[i] 系统托盘未启用");
        return Ok(());
    }

    let exit_item = MenuItemBuilder::with_id("exit", "Exit").build(app_handle)?;
    let restart_item = MenuItemBuilder::with_id("restart", "Restart").build(app_handle)?;
    let settings_item = MenuItemBuilder::with_id("settings", "Settings").build(app_handle)?;

    let tray_menu = MenuBuilder::new(app_handle)
        .item(&settings_item)
        .item(&restart_item)
        .item(&exit_item)
        .build()?;

    let tray_icon = Image::from_bytes(include_bytes!("../icons/icon.ico"))?;

    let _tray_icon = TrayIconBuilder::new()
        .menu(&tray_menu)
        .on_menu_event(move |tray_handle, event| {
            let app_handle = tray_handle.app_handle();
            match event.id.as_ref() {
                "exit" => {
                    app_handle.exit(0);
                }
                "settings" => {
                    let window = app_handle.get_webview_window("main").unwrap();
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "restart" => {
                    app_handle.restart();
                }
                _ => {}
            }
        })
        .icon(tray_icon)
        .on_tray_icon_event(move |tray_handle, event| {
            let app_handle = tray_handle.app_handle();
            match event {
                TrayIconEvent::DoubleClick { .. } => {
                    let window = app_handle.get_webview_window("main").unwrap();
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                _ => {}
            }
        })
        .build(app_handle)?;

    info!("[✓] 系统托盘已设置");
    Ok(())
}

pub fn setup_shortcut(
    app_handle: &AppHandle,
    config: &config::Config,
    main_window: tauri::WebviewWindow,
) -> Result<(), Box<dyn std::error::Error>> {
    let hotkey_buoy = config.keybinding.hotkey_buoy.clone();
    let main_shortcut: Shortcut = FromStr::from_str(&hotkey_buoy)?;
    let main_window_clone = main_window.clone();

    app_handle.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app, shortcut, event| {
                if shortcut == &main_shortcut {
                    match event.state() {
                        ShortcutState::Pressed => {
                            let window_clone = main_window_clone.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Err(e) = show_hints(window_clone).await {
                                    error!("[✗] 显示hints失败: {}", e);
                                }
                            });
                        }
                        ShortcutState::Released => {}
                    }
                }
            })
            .build(),
    )?;

    if let Err(e) = app_handle.global_shortcut().register(main_shortcut) {
        error!("[✗] 注册快捷键失败: {}", e);
        return Err(e.into());
    }
    info!("[✓] 快捷键已注册: {}", hotkey_buoy);

    Ok(())
}

pub fn create_app_builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_hint_styles,
            get_config_for_frontend,
            save_config_for_frontend,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
}

pub fn create_overlay_window(
    app_handle: &AppHandle,
    window_label: &str,
    monitor: &monitor::MonitorInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    // 如果已存在，先关闭
    if let Some(existing_window) = app_handle.get_webview_window(&window_label) {
        info!("关闭已存在的overlay窗口: {}", window_label);
        existing_window.close().map_err(|e| e.to_string())?;
    }

    info!(
        "创建overlay窗口 {} 在显示器 {}: 位置({}, {}), 大小{}x{}",
        window_label, monitor.id, monitor.x, monitor.y, monitor.width, monitor.height
    );

    let window = tauri::WebviewWindow::builder(
        app_handle,
        window_label,
        tauri::WebviewUrl::App("overlay.html".into()),
    )
    .title(window_label)
    .transparent(true)
    .decorations(false)
    // .skip_taskbar(true)
    .always_on_top(true)
    .title(window_label)
    .inner_size(monitor.width as f64, monitor.height as f64)
    .focused(false)
    .build()
    .map_err(|e| format!("创建overlay窗口失败: {}", e))?;

    window
        .set_position(tauri::PhysicalPosition::new(monitor.x, monitor.y))
        .map_err(|e| e.to_string())?;

    info!("overlay窗口创建成功，准备设置窗口属性");

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

            info!("设置窗口样式成功");

            // 保存窗口句柄
            if let Ok(mut handles) = OVERLAY_HANDLES_STORAGE.lock() {
                handles.insert(window_label.to_string(), hwnd_raw as i64);
                info!("窗口句柄保存成功");
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        info!("开发模式：打开devtools");
        window.open_devtools();
    }

    info!("overlay窗口创建完成");
    Ok(())
}
