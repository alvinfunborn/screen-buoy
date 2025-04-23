#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod element;
mod hint;
mod input;
mod monitor;
mod utils;
mod window;

use element::element::collect_ui_elements;
use hint::{create_overlay_windows, show_hints};
use log::{error, info};
use monitor::monitor::init_monitors;
use std::time::Duration;
use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowEvent,
    GlobalShortcutManager,
};
use tauri_plugin_log::{Builder as LogBuilder, LogTarget};
use windows::Win32::System::Com::*;
use config::hint::get_hint_styles;

fn main() {
    // 初始化配置
    if let Err(e) = config::init_config() {
        eprintln!("配置初始化失败: {}", e);
        return;
    }

    // 获取配置用于状态管理
    let config = config::get_config().expect("配置已初始化");

    unsafe {
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if hr.is_err() {
            error!("COM初始化失败: {:?}", hr);
        }
    }

    // 创建系统托盘菜单
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show = CustomMenuItem::new("show".to_string(), "Settings");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(quit);
    let tray = SystemTray::new().with_menu(tray_menu);

    let builder = tauri::Builder::default()
        .plugin(
            LogBuilder::default()
                .targets([LogTarget::Stdout, LogTarget::Webview, LogTarget::LogDir])
                .level(log::LevelFilter::Info)
                .build(),
        )
        .manage(config.clone())
        .setup(move |app| {
            info!("=== 应用程序启动 ===");
            info!("调试模式: {}", cfg!(debug_assertions));

            let app_handle = app.handle();
            
            // 设置开机自启动
            if config.system.start_at_login {
                info!("[i] 开机自启动功能需要系统权限支持");
            }

            // 初始化键盘钩子
            input::hook::init(app_handle.clone());
            info!("[✓] 键盘钩子初始化成功");

            // 初始化hints
            hint::init_hint_text_list_storage();
            info!("[✓] hints初始化成功");

            // 初始化显示器信息
            let main_window = app_handle.get_window("main").unwrap();
            let main_window_clone = main_window.clone();
            init_monitors(&main_window_clone);
            info!("[✓] 显示器信息初始化成功");

            // 根据配置决定是否启动时最小化到托盘
            if config.system.start_in_tray {
                main_window.hide().unwrap();
                info!("[✓] 已最小化到托盘");
            }

            // 启动后台线程更新 UI 元素
            std::thread::spawn(move || {
                info!("[✓] UI元素收集线程已启动");
                loop {
                    std::thread::sleep(Duration::from_millis(1000));
                    collect_ui_elements();
                }
            });

            // 启动时创建 overlay 窗口
            info!("正在创建遮罩层窗口...");
            match tauri::async_runtime::block_on(create_overlay_windows(app_handle.clone())) {
                Ok(_) => info!("[✓] 遮罩层窗口创建成功"),
                Err(e) => error!("[✗] 创建overlay窗口失败: {}", e),
            }

            let main_window_clone = main_window.clone();
            let hotkey_buoy = config.keybinding.hotkey_buoy.clone();
            match app_handle
                .global_shortcut_manager()
                .register(hotkey_buoy.as_str(), move || {
                    let main_window_clone = main_window_clone.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = show_hints(main_window_clone).await {
                            error!("[✗] 显示hints失败: {}", e);
                        } else {
                            info!("[✓] 显示hints成功");
                        }
                    });
                }) {
                Ok(_) => info!("[✓] 全局快捷键注册成功"),
                Err(e) => error!("[✗] 全局快捷键注册失败: {}", e),
            }

            info!("=== 应用程序初始化完成 ===");
            Ok(())
        })
        // 处理窗口事件
        .on_window_event(|event| {
            if let WindowEvent::CloseRequested { api, .. } = event.event() {
                event.window().hide().unwrap();
                api.prevent_close();
            }
        })
        // 处理系统托盘事件
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        let window = app.get_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}
        });

    // 根据配置决定是否显示托盘图标
    let builder = if config.system.show_tray_icon {
        builder.system_tray(tray)
    } else {
        builder
    };

    // 运行应用
    match builder
        .invoke_handler(tauri::generate_handler![
            get_hint_styles,
        ])
        .run(tauri::generate_context!())
    {
        Ok(_) => info!("应用程序正常退出"),
        Err(e) => error!("应用程序异常退出: {}", e),
    }

    // 清理键盘钩子
    input::hook::cleanup();
    info!("[✓] 键盘钩子已清理");

    unsafe {
        CoUninitialize();
    }
}
