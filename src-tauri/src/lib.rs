#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod config;
pub mod element;
pub mod hint;
pub mod input;
pub mod monitor;
mod utils;
pub mod window;

use std::str::FromStr;

use config::{get_config_for_frontend, get_hint_styles, save_config_for_frontend};
use hint::show_hints;
use log::{error, info};
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_log::{Target, TargetKind};

pub fn init_plugins(app_handle: &AppHandle,) -> Result<(), Box<dyn std::error::Error>> {
    app_handle.plugin(tauri_plugin_process::init())?;
    app_handle.plugin(tauri_plugin_log::Builder::new().build())?;
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
                        ShortcutState::Released => {
                        }
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
