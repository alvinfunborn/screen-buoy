use crate::{config, window::WindowElement};
use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use windows::{
    core::GUID,
    Win32::{Foundation::*, System::Com::*, UI::Accessibility::*},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIElement {
    pub text: String,
    pub is_enabled: bool,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub width: i32,
    pub height: i32,
    pub window_handle: i64,
    pub control_type: i32,
    // element_type: 0-default, 1-window, 2-pane, 3-tab, 4-button, 5-scrollbar
    pub element_type: usize,
}

pub static WINDOWS_UI_ELEMENTS_MAP_STORAGE: Lazy<Mutex<HashMap<WindowElement, Vec<UIElement>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn init_ui_automation() -> (IUIAutomation, IUIAutomationCondition) {
    let automation = unsafe {
        match CoCreateInstance::<_, IUIAutomation>(&CUIAutomation as *const GUID, None, CLSCTX_ALL)
        {
            Ok(automation) => automation,
            Err(e) => {
                panic!("[init_ui_automation] create UI Automation failed: {:?}", e);
            }
        }
    };
    let condition = unsafe {
        match automation.CreateTrueCondition() {
            Ok(condition) => condition,
            Err(e) => {
                panic!("[init_ui_automation] create condition failed: {:?}", e);
            }
        }
    };

    (automation, condition)
}

pub fn collect_ui_elements_for_window(
    automation: &IUIAutomation,
    condition: &IUIAutomationCondition,
    window: &WindowElement,
) -> Vec<UIElement> {
    let mut position_map: HashMap<(i32, i32), UIElement> = HashMap::new();
    let hwnd = HWND(window.window_handle as *mut _);

    unsafe {
        let root_element = match automation.ElementFromHandle(hwnd) {
            Ok(element) => element,
            Err(_) => {
                error!("[collect_ui_elements_for_window] element from handle failed");
                return Vec::new();
            }
        };

        let all_elements = match root_element.FindAll(TreeScope_Subtree, condition) {
            Ok(elements) => elements,
            Err(_) => {
                error!("[collect_ui_elements_for_window] find all elements failed");
                return Vec::new();
            }
        };

        let count = match all_elements.Length() {
            Ok(len) => len,
            Err(_) => {
                error!("[collect_ui_elements_for_window] get all elements length failed");
                return Vec::new();
            }
        };

        for i in 0..count {
            let element = match all_elements.GetElement(i) {
                Ok(e) => e,
                Err(_) => {
                    error!("[collect_ui_elements_for_window] get element failed");
                    continue;
                }
            };

            // 获取基本属性
            let (name, control_type_id, is_enabled, is_offscreen) = match (
                element.CurrentName(),
                element.CurrentControlType(),
                element.CurrentIsEnabled(),
                element.CurrentIsOffscreen(),
            ) {
                (Ok(n), Ok(id), Ok(e), Ok(o)) => (n, id, e, o),
                _ => {
                    error!("[collect_ui_elements_for_window] get basic properties failed");
                    continue;
                }
            };

            // 检查可见性和启用状态
            if !is_enabled.as_bool() || is_offscreen.as_bool() {
                continue;
            }

            // 获取位置和类名
            let rect = match element.CurrentBoundingRectangle() {
                Ok(r) => r,
                _ => continue,
            };

            // 检查元素是否有效大小
            let has_valid_size = (rect.right - rect.left) > 0 && (rect.bottom - rect.top) > 0;
            if !has_valid_size {
                continue;
            }

            // 获取元素类型和z_index
            let (element_type, z_index) = match config::hint::HINT_CONTROL_TYPES_ID_Z_MAP.get(&control_type_id.0) {
                Some((element_type, z_index)) => (element_type, z_index),
                None => continue,
            };

            let ui_element = UIElement {
                text: name.to_string(),
                is_enabled: true,
                x: (rect.right + rect.left) / 2,
                y: (rect.bottom + rect.top) / 2,
                z: *z_index,
                width: rect.right - rect.left,
                height: rect.bottom - rect.top,
                window_handle: window.window_handle,
                control_type: control_type_id.0,
                element_type: *element_type,
            };

            let position = (rect.left, rect.top);
            match position_map.get(&position) {
                Some(old_element) => {
                    if old_element.z < *z_index as i32 {
                        position_map.insert(position, ui_element);
                    }
                }
                None => {
                    position_map.insert(position, ui_element);
                }
            }
        }
    }
    position_map.values().cloned().collect()
}

pub fn collect_ui_elements() {
    let windows = crate::window::window::get_all_windows();
    let (automation, condition) = init_ui_automation();

    for window in windows.iter() {
        if !window.visible {
            continue;
        }
        let window_elements = collect_ui_elements_for_window(&automation, &condition, window);
        if window_elements.is_empty() {
            continue;
        }

        if let Ok(mut map) = WINDOWS_UI_ELEMENTS_MAP_STORAGE.lock() {
            map.insert(window.clone(), window_elements);
        }
    }
}
