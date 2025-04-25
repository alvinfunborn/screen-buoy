#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use log::{error, info};
use screen_buoy::config;
use screen_buoy::element;
use screen_buoy::hint;
use screen_buoy::hint::create_overlay_windows;
use screen_buoy::input;
use screen_buoy::monitor::monitor;
use screen_buoy::set_auto_start;
use screen_buoy::setup_panic_handler;
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
    config::init_config();
    let config = config::get_config().unwrap();
    let config_for_manage = config.clone();

    let mut builder = screen_buoy::create_app_builder();
    // Setup application
    builder = builder.setup(move |app| {
        info!("=== application started ===");
        info!("debug mode: {}", cfg!(debug_assertions));

        let app_handle = app.handle();

        // Setup system tray
        setup_tray(&app_handle, &config).expect("Failed to setup system tray");

        // Setup main window
        let main_window = app_handle.get_webview_window("main").unwrap();

        // Handle window visibility
        if config.system.start_in_tray {
            if let Err(e) = main_window.hide() {
                error!("[✗] hide main window failed: {}", e);
            }
            info!("[✓] minimized to tray (if show_tray_icon is true)");
        } else {
            if let Err(e) = main_window.show() {
                error!("[✗] show main window failed: {}", e);
            }
        }

        // Initialize panic handler
        setup_panic_handler(app_handle.clone());

        // Initialize input hook
        input::hook::init(app_handle.clone());

        // Initialize hints
        hint::init_hint_text_list_storage();
        info!("[✓] hints initialized successfully");

        monitor::init_monitors(&main_window);
        info!("[✓] monitors initialized successfully");

        // Setup UI collection
        element::setup_ui_collection(&config);
        info!("[✓] UI elements collection thread started");

        // Create overlay windows
        create_overlay_windows(&app_handle);
        info!("[✓] overlay windows created successfully");

        // Setup shortcuts
        setup_shortcut(&app_handle, &config, main_window.clone())
            .expect("Failed to setup shortcuts");
        info!("[✓] shortcuts setup successfully");

        // set autostart
        set_auto_start(&app_handle, &config).expect("Failed to setup auto start");
        info!("[✓] auto start setup successfully");

        info!("=== application initialized successfully ===");
        Ok(())
    });

    // Build and run application
    let app = builder
        .build(tauri::generate_context!("Tauri.toml"))
        .expect("error while building tauri application");

    app.manage(config_for_manage);

    app.run(|_app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            info!("application is exiting, cleaning up resources...");
            input::hook::cleanup();
            info!("[✓] keyboard hook cleaned up");

            unsafe {
                CoUninitialize();
                info!("[✓] COM uninitialized");
            }
        }
    });
}
