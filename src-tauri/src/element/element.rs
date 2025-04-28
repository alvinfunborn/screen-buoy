use crate::window::WindowElement;
use log::{debug, error, info};
use once_cell::sync::Lazy;
use tokio::time::Instant;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use super::ui_automation::{clean_expired_cache, get_cached_elements_for_window, UIAutomationRequest, UIElement};

static PROCESSING_WINDOWS: Lazy<Mutex<HashSet<i64>>> = Lazy::new(|| Mutex::new(HashSet::new()));

pub static WINDOWS_UI_ELEMENTS_MAP_STORAGE: Lazy<Mutex<HashMap<WindowElement, Vec<UIElement>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn queue_collect_for_window(window: WindowElement) {
    let hwnd = window.window_handle;
    {
        let mut set = PROCESSING_WINDOWS.lock().unwrap();
        if set.contains(&hwnd) {
            debug!("[queue_collect_for_window] window:{:?} already has windowtask running", window);
            return; // 已有任务在跑
        }
        set.insert(hwnd);
    }
    // 派发异步任务
    std::thread::spawn(move || {
        let elements = get_cached_elements_for_window(&window);
        if let Some(elements) = elements {
            let mut map = WINDOWS_UI_ELEMENTS_MAP_STORAGE.lock().unwrap();
            map.insert(window, elements);
        }
        PROCESSING_WINDOWS.lock().unwrap().remove(&hwnd);
    });
}

fn cache_ui_elements_for_windows(windows: &[WindowElement], real_time: bool) {
    debug!("[cache_ui_elements_for_windows] cache ui elements for {} windows, real_time: {}", windows.len(), real_time);
    if real_time {
        let ui_automation = UIAutomationRequest::new();
        for window in windows.iter() {
            let elements = ui_automation.get_elements_for_window(window);
            if let Some(elements) = elements {
                let mut map = WINDOWS_UI_ELEMENTS_MAP_STORAGE.lock().unwrap();
                map.insert(window.clone(), elements);
            }
        }
    } else {
        for window in windows.iter() {
            queue_collect_for_window(window.clone());
        }
    }
}

pub fn collect_ui_elements() {
    let start_time = Instant::now();
    clean_expired_cache();
    let windows = crate::window::window::get_all_windows();
    let top_windows = crate::window::window::calculate_top_windows(&windows);

    let mut top_level_windows = Vec::new();
    let mut visible_windows = Vec::new();
    for window in windows.iter() {
        if top_windows.contains(window) { 
            // 顶层窗口获取实时元素
            top_level_windows.push(window.clone());
        } else if window.visible {
            // 可见窗口获取缓存元素由任务队列处理
            visible_windows.push(window.clone());
        }
    }
    debug!("[collect_ui_elements] windows: {:?}, top_level_windows: {:?}, visible_windows: {:?}", windows, top_level_windows, visible_windows);
    cache_ui_elements_for_windows(&top_level_windows, true);
    cache_ui_elements_for_windows(&visible_windows, false);

    debug!("[collect_ui_elements] collect ui elements time: {:?}", start_time.elapsed());
}
