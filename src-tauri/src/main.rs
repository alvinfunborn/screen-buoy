#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use log::{error, info};
use screen_buoy::config;
use screen_buoy::element;
use screen_buoy::hint;
use screen_buoy::hint::create_overlay_windows;
use screen_buoy::init_plugins;
use screen_buoy::input;
use screen_buoy::monitor::monitor;
use screen_buoy::setup_shortcut;
use screen_buoy::setup_tray;
use tauri::Manager;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};

fn main() {
    // Initialize COM
    unsafe {
        let result = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if result.is_err() {
            error!("COM 初始化失败: {:?}", result.message());
        } else {
            info!("COM 初始化成功 (APARTMENTTHREADED)");
        }
    }

    // Initialize config first
    config::init_config().expect("Failed to initialize config");
    let config = config::get_config().expect("Failed to get config");
    let config_for_manage = config.clone();

    let mut builder = screen_buoy::create_app_builder();
    // Setup application
    builder = builder.setup(move |app| {
        info!("=== 应用程序启动 ===");
        info!("调试模式: {}", cfg!(debug_assertions));

        let app_handle = app.handle();

        // 初始化插件
        init_plugins(&app_handle).expect("Failed to initialize plugins");

        // Setup system tray
        setup_tray(&app_handle, &config).expect("Failed to setup system tray");

        // Setup main window
        let main_window = app_handle.get_webview_window("main").unwrap();

        if config.system.start_at_login {
            info!("[i] 开机自启动功能在 v2 中可能需要 tauri-plugin-autostart (待验证)");
        }

        #[cfg(debug_assertions)]
        {
            main_window.open_devtools();
            info!("[✓] 主窗口开发工具已打开");
        }

        // Handle window visibility
        if config.system.start_in_tray {
            if let Err(e) = main_window.hide() {
                error!("[✗] 启动时隐藏窗口失败: {}", e);
            }
            info!("[✓] 已最小化到托盘 (如果 show_tray_icon 为 true)");
        } else {
            if let Err(e) = main_window.show() {
                error!("[✗] 启动时显示窗口失败: {}", e);
            }
        }

        // Initialize input hook
        input::hook::init(app_handle.clone());

        // Initialize hints
        hint::init_hint_text_list_storage();
        info!("[✓] hints初始化成功");

        monitor::init_monitors(&main_window);
        info!("[✓] 显示器信息初始化成功");

        // Setup UI collection
        element::setup_ui_collection(&config);
        info!("[✓] UI元素收集线程已启动");

        // Create overlay windows
        match create_overlay_windows(&app_handle) {
            Ok(_) => info!("[✓] 遮罩层窗口创建成功"),
            Err(e) => error!("[✗] 创建overlay窗口失败: {}", e),
        }

        // Setup shortcuts
        setup_shortcut(&app_handle, &config, main_window.clone())
            .expect("Failed to setup shortcuts");

        info!("=== 应用程序初始化完成 ===");
        Ok(())
    });

    // Build and run application
    let app = builder
        .build(tauri::generate_context!("Tauri.toml"))
        .expect("error while building tauri application");

    app.manage(config_for_manage);

    app.run(|_app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            info!("应用程序正在退出，清理资源...");
            input::hook::cleanup();
            info!("[✓] 键盘钩子已清理");

            unsafe {
                CoUninitialize();
                info!("[✓] COM 已卸载");
            }
        }
    });
}
