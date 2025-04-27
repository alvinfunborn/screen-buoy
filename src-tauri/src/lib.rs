#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod config;
pub mod element;
pub mod hint;
pub mod input;
pub mod monitor;
mod utils;
pub mod window;

use config::{get_config_for_frontend, get_hint_types_styles, hint::get_hint_default_style, save_config_for_frontend};
use hint::{ overlay::OVERLAY_HANDLES_STORAGE, show_hints};
use log::{error, info};
use std::{panic, str::FromStr};
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_log::{Target, TargetKind};
use windows::Win32::{Foundation::HWND, Graphics::Dwm::DWMWINDOWATTRIBUTE};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_LAYERED, 
    WS_EX_TRANSPARENT, 
};
use time;
use windows::Win32::Graphics::Dwm::DwmSetWindowAttribute;

pub fn setup_tray(
    app_handle: &AppHandle,
    config: &config::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    if !config.system.show_tray_icon {
        info!("[setup_tray] tray icon is not enabled");
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
                                show_hints(window_clone).await;
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
    Ok(())
}

pub fn set_auto_start(
    app_handle: &AppHandle,
    config: &config::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let auto_start = config.system.start_at_login;
    let autostart_manager = app_handle.autolaunch();
    if auto_start {
        let _ = autostart_manager.enable();
    } else {
        let _ = autostart_manager.disable();
    }
    Ok(())
}

pub fn setup_panic_handler(app_handle: tauri::AppHandle) {
    panic::set_hook(Box::new(move |panic_info| {
        let location = panic_info
            .location()
            .unwrap_or_else(|| panic::Location::caller());
        let message = match panic_info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match panic_info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        let error_info = format!(
            "program panic:\nlocation: {}:{}\nerror: {}",
            location.file(),
            location.line(),
            message
        );

        error!("{}", error_info);

        // 发送错误到前端
        let window = app_handle.get_webview_window("main").unwrap();
        window.emit("rust-panic", error_info).unwrap_or_else(|e| {
            error!(
                "[setup_panic_handler] send panic info to frontend failed: {}",
                e
            );
        });
    }));
}

pub fn create_app_builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                ])
                .format(|out, message, record| {
                    let time = time::OffsetDateTime::now_local()
                        .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
                    out.finish(format_args!(
                        "[{}][{:02}:{:02}:{:02}.{:03}][{}][{}] {}",
                        time.date(),
                        time.hour(),
                        time.minute(),
                        time.second(),
                        time.millisecond(),
                        record.target(),
                        record.level(),
                        message
                    ))
                })
                .build(),
        )
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .invoke_handler(tauri::generate_handler![
            get_hint_default_style,
            get_hint_types_styles,
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
) {
    // 如果已存在，先关闭
    if let Some(existing_window) = app_handle.get_webview_window(&window_label) {
        if let Err(e) = existing_window.close() {
            error!(
                "[create_overlay_window] close existing window failed: {}",
                e
            );
        }
    }

    let width = monitor.width as f64 / monitor.scale_factor;
    let height = monitor.height as f64 / monitor.scale_factor;
    let position_x = monitor.x;
    let position_y = monitor.y;
    info!(
        "[create_overlay_window] create overlay window {}: position({}, {}), size{}x{}",
        window_label, position_x, position_y, width, height
    );
    let window = tauri::WebviewWindowBuilder::new(
        app_handle,
        window_label,
        tauri::WebviewUrl::App(format!("overlay.html?window_label={}", window_label).into()),
    )
    .title(window_label)
    .transparent(true)
    .decorations(false)
    // must disable shadow, otherwise the window will be offset
    .shadow(false)
    .resizable(true)
    .inner_size(width, height)
    .focused(false)
    .build();

    if let Err(e) = window {
        panic!(
            "[create_overlay_window] create overlay window failed: {}",
            e
        );
    }
    
    let window = window.unwrap();
    if let Err(e) = window.set_position(tauri::PhysicalPosition::new(position_x, position_y)) {
        error!("[create_overlay_window] set position failed: {}", e);
    }
    // 确保窗口位置正确
    if let Ok(hwnd) = window.hwnd() {
        let hwnd_raw = hwnd.0;
        const DWMWA_WINDOW_CORNER_PREFERENCE: DWMWINDOWATTRIBUTE = DWMWINDOWATTRIBUTE(33);
        const DWMWCP_DONOTROUND: u32 = 1;
        let preference: u32 = DWMWCP_DONOTROUND;
        unsafe {
            // 去掉 Windows 11 圆角
            let _ = DwmSetWindowAttribute(
                HWND(hwnd_raw as *mut _),
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &preference as *const _ as _,
                std::mem::size_of_val(&preference) as u32,
            );
            set_window_transparent_style(&window, hwnd_raw as i64);
        }
        if let Ok(mut handles) = OVERLAY_HANDLES_STORAGE.lock() {
            handles.insert(window_label.to_string(), hwnd_raw as i64);
        }
    }

}

fn set_window_transparent_style(window: &tauri::WebviewWindow, hwnd_raw: i64) {
    // 设置无任务栏图标并确保在最顶层
    if let Err(e) = window.set_skip_taskbar(true) {
        error!("[set_overlay_style] set skip taskbar failed: {}", e);
    }
    if let Err(e) = window.set_always_on_top(true) {
        error!("[set_overlay_style] set always on top failed: {}", e);
    }

    // 设置扩展窗口样式
    unsafe {
        let style = GetWindowLongW(HWND(hwnd_raw as *mut _), GWL_EXSTYLE);
        // 确保WS_EX_TRANSPARENT样式被正确设置
        SetWindowLongW(
            HWND(hwnd_raw as *mut _),
            GWL_EXSTYLE,
            style | (WS_EX_TRANSPARENT.0 | WS_EX_LAYERED.0) as i32,
        );
    }
}
