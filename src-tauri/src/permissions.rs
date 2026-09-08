use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionStatus {
    pub required: bool,
    pub accessibility: bool,
    pub input_monitoring: bool,
    pub keyboard_hook: bool,
    pub ready: bool,
    pub app_path: String,
}

#[tauri::command]
pub fn get_permission_status() -> PermissionStatus {
    #[cfg(target_os = "macos")]
    {
        let accessibility = crate::macos_access::is_accessibility_trusted();
        let input_monitoring = crate::macos_access::has_input_monitoring_access();
        let keyboard_hook = crate::input::hook::is_ready();
        PermissionStatus {
            required: true,
            accessibility,
            input_monitoring,
            keyboard_hook,
            // A working active tap is the authority: macOS can allow it through
            // Accessibility even when the listen-only preflight returns false.
            ready: accessibility && keyboard_hook,
            app_path: std::env::current_exe()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
        }
    }
    #[cfg(not(target_os = "macos"))]
    PermissionStatus {
        required: false,
        accessibility: true,
        input_monitoring: true,
        keyboard_hook: true,
        ready: true,
        app_path: String::new(),
    }
}

#[tauri::command]
pub fn open_permission_settings(permission: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let pane = match permission {
            "accessibility" => {
                crate::macos_access::request_accessibility();
                "Privacy_Accessibility"
            }
            "inputMonitoring" => {
                crate::macos_access::request_input_monitoring();
                "Privacy_ListenEvent"
            }
            _ => return Err("Unknown permission".into()),
        };
        let status = std::process::Command::new("/usr/bin/open")
            .arg(format!("x-apple.systempreferences:com.apple.preference.security?{pane}"))
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err("Could not open System Settings".into())
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = permission;
        Err("This permission is only required on macOS".into())
    }
}
