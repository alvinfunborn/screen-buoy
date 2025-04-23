#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_windows;
mod elements;
mod hints;
mod monitors;
mod utils;
mod inputs;
mod configs;

use elements::collect_ui_elements;
use hints::{create_overlay_windows, show_hints};
use monitors::monitor::init_monitors;
use std::time::Duration;
use tauri::{GlobalShortcutManager, Manager};
use windows::Win32::System::Com::*;
use tauri_plugin_log::{LogTarget, Builder as LogBuilder}; 
use log::{error, info};

fn main() {
    // 初始化配置
    if let Err(e) = configs::init_config() {
        eprintln!("配置初始化失败: {}", e);
        return;
    }

    unsafe {
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if hr.is_err() {
            error!("COM初始化失败: {:?}", hr);
        }
    }

    let builder = tauri::Builder::default()
        .plugin(LogBuilder::default()
            .targets([
                LogTarget::Stdout,
                LogTarget::Webview,
                LogTarget::LogDir,
            ])
            .level(log::LevelFilter::Info)
            .build())
        .setup(move |app| {
            info!("=== 应用程序启动 ===");
            info!("调试模式: {}", cfg!(debug_assertions));
            
            let app_handle = app.handle();

            // 初始化键盘钩子
            inputs::hook::init(app_handle.clone());
            info!("[✓] 键盘钩子初始化成功");

            // 初始化hints
            hints::init_hint_text_list_storage();
            info!("[✓] hints初始化成功");

            // 初始化显示器信息
            let main_window = app_handle.get_window("main").unwrap();
            let main_window_clone = main_window.clone();
            init_monitors(&main_window_clone);
            info!("[✓] 显示器信息初始化成功");

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
            // 注册 Alt+H 快捷键
            match app_handle.global_shortcut_manager().register("Alt+H", move || {
                info!("触发快捷键: Alt+H");
                let main_window_clone = main_window_clone.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = show_hints(main_window_clone).await {
                        error!("[✗] 显示hints失败: {}", e);
                    } else {
                        info!("[✓] 显示hints成功");
                    }
                });
            }) {
                Ok(_) => info!("[✓] Alt+H快捷键注册成功"),
                Err(e) => error!("[✗] Alt+H快捷键注册失败: {}", e),
            }

            // 开发工具相关代码...
            #[cfg(debug_assertions)]
            {
                let main_window_clone = main_window.clone();
                tauri::async_runtime::spawn(async move {
                    main_window_clone.open_devtools();
                    info!("[✓] 主窗口开发工具已打开");
                });
            }

            info!("=== 应用程序初始化完成 ===");
            Ok(())
        });

    // 运行应用
    match builder
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!()) {
            Ok(_) => info!("应用程序正常退出"),
            Err(e) => error!("应用程序异常退出: {}", e),
        }

    // 清理键盘钩子
    inputs::hook::cleanup();
    info!("[✓] 键盘钩子已清理");

    unsafe {
        CoUninitialize();
    }
}
