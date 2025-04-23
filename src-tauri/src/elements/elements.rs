use crate::app_windows::WindowElement;
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
    // element_type: 0-default, 1-window, 2-region, 3-click2focus, 4-click2modify, 5-click2hold
    pub element_type: i32,
}

pub static WINDOWS_UI_ELEMENTS_MAP_STORAGE: Lazy<Mutex<HashMap<WindowElement, Vec<UIElement>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn init_ui_automation() -> (IUIAutomation, IUIAutomationCondition) {
    let automation = unsafe {
        match CoCreateInstance::<_, IUIAutomation>(&CUIAutomation as *const GUID, None, CLSCTX_ALL)
        {
            Ok(automation) => automation,
            Err(e) => {
                panic!("创建UI Automation失败: {:?}", e);
            }
        }
    };
    let condition = unsafe {
        match automation.CreateTrueCondition() {
            Ok(condition) => condition,
            Err(e) => {
                panic!("创建条件失败: {:?}", e);
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
    let start = Instant::now();
    let mut position_map: HashMap<(i32, i32), UIElement> = HashMap::new();
    let hwnd = HWND(window.window_handle as *mut _);

    unsafe {
        let root_element = match automation.ElementFromHandle(hwnd) {
            Ok(element) => element,
            Err(_) => return Vec::new(),
        };

        let all_elements = match root_element.FindAll(TreeScope_Subtree, condition) {
            Ok(elements) => elements,
            Err(_) => return Vec::new(),
        };

        let count = match all_elements.Length() {
            Ok(len) => len,
            Err(_) => return Vec::new(),
        };

        for i in 0..count {
            let element = match all_elements.GetElement(i) {
                Ok(e) => e,
                Err(_) => continue,
            };

            // 获取基本属性
            let (name, control_type_id, is_enabled, is_offscreen) = match (
                element.CurrentName(),
                element.CurrentControlType(),
                element.CurrentIsEnabled(),
                element.CurrentIsOffscreen(),
            ) {
                (Ok(n), Ok(id), Ok(e), Ok(o)) => (n, id, e, o),
                _ => continue,
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

            let is_window = match control_type_id {
                UIA_WindowControlTypeId => true,
                _ => false,
            };
            let is_region = match control_type_id {
                UIA_AppBarControlTypeId => true,
                UIA_HeaderControlTypeId => true,
                UIA_HeaderItemControlTypeId => true,
                UIA_PaneControlTypeId => true,
                UIA_StatusBarControlTypeId => true,
                UIA_TabControlTypeId => true,
                UIA_TitleBarControlTypeId => true,
                _ => false,
            };
            let is_click2focus = match control_type_id {
                UIA_TabItemControlTypeId => true,
                _ => false,
            };
            let is_click2modify = match control_type_id {
                UIA_ToolBarControlTypeId => true,
                UIA_GroupControlTypeId => true,

                UIA_ButtonControlTypeId => true,
                UIA_CalendarControlTypeId => true,
                UIA_CheckBoxControlTypeId => true,
                UIA_ComboBoxControlTypeId => true,
                UIA_CustomControlTypeId => true,
                UIA_DataGridControlTypeId => true,
                UIA_DataItemControlTypeId => true,
                UIA_DocumentControlTypeId => true,
                UIA_EditControlTypeId => true,
                UIA_HyperlinkControlTypeId => true,
                UIA_ImageControlTypeId => true,
                UIA_ListControlTypeId => true,
                UIA_ListItemControlTypeId => true,
                UIA_MenuBarControlTypeId => true,
                UIA_MenuControlTypeId => true,
                UIA_MenuItemControlTypeId => true,
                UIA_ProgressBarControlTypeId => true,
                UIA_RadioButtonControlTypeId => true,
                UIA_SemanticZoomControlTypeId => true,
                UIA_SplitButtonControlTypeId => true,
                UIA_TableControlTypeId => true,
                UIA_TextControlTypeId => true,
                UIA_ThumbControlTypeId => true,
                UIA_ToolTipControlTypeId => true,
                UIA_TreeControlTypeId => true,
                UIA_TreeItemControlTypeId => true,
                _ => false,
            };
            let is_click2hold = match control_type_id {
                UIA_ScrollBarControlTypeId => true,
                UIA_SeparatorControlTypeId => true,
                UIA_SliderControlTypeId => true,
                UIA_SpinnerControlTypeId => true,
                _ => false,
            };

            let element_type = if is_window {
                1
            } else if is_region {
                2
            } else if is_click2focus {
                3
            } else if is_click2modify {
                4
            } else if is_click2hold {
                5
            } else {
                0
            };
            let z = if is_window {
                5
            } else if is_region {
                1
            } else if is_click2focus {
                2
            } else if is_click2modify {
                4
            } else if is_click2hold {
                3
            } else {
                0
            };

            let ui_element = UIElement {
                text: name.to_string(),
                is_enabled: true,
                x: (rect.right + rect.left) / 2,
                y: (rect.bottom + rect.top) / 2,
                z,
                width: rect.right - rect.left,
                height: rect.bottom - rect.top,
                window_handle: window.window_handle,
                control_type: control_type_id.0,
                element_type,
            };

            let position = (rect.left, rect.top);
            match position_map.get(&position) {
                Some(old_element) => {
                    if old_element.z < z {
                        // println!("collect_ui_elements_for_window:{} 更新元素:{}:({},{})",
                        //     window.title, ui_element.text, ui_element.x, ui_element.y);
                        position_map.insert(position, ui_element);
                    }
                }
                None => {
                    // println!("collect_ui_elements_for_window:{} 添加元素:{}:({},{})",
                    //     window.title, ui_element.text, ui_element.x, ui_element.y);
                    position_map.insert(position, ui_element);
                }
            }
        }
    }
    // println!(
    //     "collect_ui_elements_for_window:{} 耗时: {:?}, 元素数量: {}",
    //     window.title,
    //     start.elapsed(),
    //     position_map.len()
    // );
    position_map.values().cloned().collect()
}

pub fn collect_ui_elements() {
    let windows = crate::app_windows::windows::get_all_windows();
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
