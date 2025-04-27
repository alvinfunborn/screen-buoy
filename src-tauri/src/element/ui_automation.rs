use log::{debug, error, info};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use windows::Win32::{Foundation::*, System::Com::*, UI::Accessibility::*};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config;
use crate::window::WindowElement;

pub struct UIAutomationRequest {
    pub automation: IUIAutomation,
    pub condition: IUIAutomationCondition,
}

#[derive(Clone)]
pub struct UIElement {
    pub text: String,
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

static ELEMENTS_CACHE_WITH_EXPIRATION: Lazy<Mutex<HashMap<i64, (Vec<UIElement>, u128)>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

impl UIAutomationRequest {
    pub fn new() -> Self {
        unsafe {
            let automation = CoCreateInstance::<_, IUIAutomation>(&CUIAutomation, None, CLSCTX_ALL)
                .expect("Failed to create UI Automation");
            let condition = automation.CreateTrueCondition()
                .expect("Failed to create condition");
            UIAutomationRequest {
                automation, condition
            }
        }
    }

    pub fn get_elements_for_window(&self, window: &WindowElement) -> Option<Vec<UIElement>> {
        unsafe {
            let root_element = self.automation.ElementFromHandle(HWND(window.window_handle as *mut _)).ok()?;
            match root_element.FindAll(TreeScope_Subtree, &self.condition) {
                Ok(elements) => {
                let expire_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() + config::get_config().unwrap().ui_automation.cache_ttl as u128;
                let window_handle = window.window_handle;
                let elements = convert_ui_automation(elements, window_handle);
                ELEMENTS_CACHE_WITH_EXPIRATION.lock().unwrap().insert(window_handle, (elements.clone(), expire_at));
                debug!("[get_elements_for_window] get {} elements for window: {}", elements.len(), window_handle);
                Some(elements)
            }
                Err(e) => {
                    error!("[get_elements_for_window] find all elements failed: {}", e);
                    None
                }
            }
        }
    }
    
    pub fn get_cached_elements_for_window(&self, window: &WindowElement) -> Option<Vec<UIElement>> {
        let window_handle = window.window_handle;
        if let Some((elements, expire_at)) = ELEMENTS_CACHE_WITH_EXPIRATION.lock().unwrap().get(&window_handle) {
            if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() <= *expire_at {
                debug!("[get_cached_elements_for_window] get cached {} elements for window: {}", elements.len(), window_handle);
                return Some(elements.clone());
            }
            ELEMENTS_CACHE_WITH_EXPIRATION.lock().unwrap().remove(&window_handle);
            debug!("[get_cached_elements_for_window] remove expired elements for window: {}", window_handle);
        }
        self.get_elements_for_window(window)
    }
}

pub fn get_cached_elements_for_window(window: &WindowElement) -> Option<Vec<UIElement>> {
    let request = UIAutomationRequest::new();
    request.get_cached_elements_for_window(window)
}

unsafe fn convert_ui_automation(all_elements: IUIAutomationElementArray, window_handle: i64) -> Vec<UIElement> {
    let mut position_map: HashMap<(i32, i32), UIElement> = HashMap::new();
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
            Err(_) => continue
        };

        // 检查可见性和启用状态
        let (is_enabled, is_offscreen) = match (
            element.CurrentIsEnabled(),
            element.CurrentIsOffscreen(),
        ) {
            (Ok(e), Ok(o)) => (
                e.as_bool(),
                o.as_bool(),
            ),
            _ => continue
        };
        if !is_enabled || is_offscreen {
            continue;
        }

        let control_type_id = match element.CurrentControlType() {
            Ok(id) => id,
            Err(_) => continue,
        };
        // 获取元素类型和z_index
        let (element_type, z_index) =
            match config::hint::HINT_CONTROL_TYPES_ID_Z_MAP.get(&control_type_id.0) {
                Some((element_type, z_index)) => (element_type, z_index),
                None => continue,
            };

        let rect = match element.CurrentBoundingRectangle() {
            Ok(r) => r,
            Err(_) => continue,
        };

        debug!("[convert_ui_automation] get element with control_type: {}, element_type: {}, z_index: {}, rect: {:?}, window_handle: {}", 
            control_type_id.0, element_type, z_index, rect, window_handle);
        let ui_element = UIElement {
            text: "".to_string(),
            x: (rect.right + rect.left) / 2,
            y: (rect.bottom + rect.top) / 2,
            z: *z_index,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
            window_handle: window_handle,
            control_type: control_type_id.0,
            element_type: *element_type,
        };

        let position = (rect.left, rect.top);
        match position_map.get(&position) {
            Some(old_element) => {
                if old_element.z < *z_index as i32 {
                    debug!("[convert_ui_automation] overwrite element at z:{} from window:{} with z_index:{}", 
                        old_element.z, old_element.window_handle, *z_index);
                    position_map.insert(position, ui_element);
                }
            }
            None => {
                position_map.insert(position, ui_element);
            }
        }
    }
    position_map.values().cloned().collect()
}

// 定时清理过期key的方法
pub fn clean_expired_cache() {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let mut cache = ELEMENTS_CACHE_WITH_EXPIRATION.lock().unwrap();
    cache.retain(|_, (_, expire_at)| *expire_at > now);
}
