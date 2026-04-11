use crate::utils::Rect;
use core_foundation::base::{CFType, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_graphics::display::{
    CGDisplay, kCGWindowListExcludeDesktopElements, kCGWindowListOptionOnScreenOnly,
};
use indexmap::IndexMap;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowElement {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub title: String,
    pub class_name: String,
    pub z_index: i32,
    pub window_handle: i64,
    pub visible: bool,
    pub is_task_bar: bool,
    pub owner_pid: i32,
}

impl Hash for WindowElement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.window_handle.hash(state);
    }
}

impl PartialEq for WindowElement {
    fn eq(&self, other: &Self) -> bool {
        self.window_handle == other.window_handle
    }
}

impl Eq for WindowElement {}

fn cf_dict_get_number(dict: &CFDictionary<CFString, CFType>, key: &str) -> Option<CFNumber> {
    let k = CFString::new(key);
    dict.find(&k)?.clone().downcast::<CFNumber>()
}

fn cf_dict_get_bool(dict: &CFDictionary<CFString, CFType>, key: &str) -> Option<bool> {
    let k = CFString::new(key);
    let v = dict.find(&k)?.clone().downcast::<CFBoolean>()?;
    Some(v == CFBoolean::true_value())
}

fn cf_dict_get_string(dict: &CFDictionary<CFString, CFType>, key: &str) -> Option<String> {
    let k = CFString::new(key);
    let s = dict.find(&k)?.clone().downcast::<CFString>()?;
    Some(s.to_string())
}

fn bounds_rect(dict: &CFDictionary<CFString, CFType>) -> Option<(i32, i32, i32, i32)> {
    let k = CFString::new("kCGWindowBounds");
    let bounds_cf = dict.find(&k)?.clone();
    let bounds = unsafe {
        CFDictionary::<CFString, CFType>::wrap_under_get_rule(
            bounds_cf.as_CFTypeRef() as CFDictionaryRef,
        )
    };
    let x = cf_dict_get_number(&bounds, "X")?.to_f64()? as i32;
    let y = cf_dict_get_number(&bounds, "Y")?.to_f64()? as i32;
    let w = cf_dict_get_number(&bounds, "Width")?.to_f64()? as i32;
    let h = cf_dict_get_number(&bounds, "Height")?.to_f64()? as i32;
    Some((x, y, w.max(0), h.max(0)))
}

pub fn get_all_windows() -> Vec<WindowElement> {
    let Some(arr) = CGDisplay::window_list_info(
        kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
        None,
    ) else {
        error!("[get_all_windows macos] window_list_info returned null");
        return Vec::new();
    };

    let mut windows: Vec<WindowElement> = Vec::new();
    for ptr in arr.get_all_values() {
        if ptr.is_null() {
            continue;
        }
        let dict = unsafe {
            CFDictionary::<CFString, CFType>::wrap_under_get_rule(ptr as core_foundation::dictionary::CFDictionaryRef)
        };

        if let Some(false) = cf_dict_get_bool(&dict, "kCGWindowIsOnscreen") {
            continue;
        }

        let layer = cf_dict_get_number(&dict, "kCGWindowLayer")
            .and_then(|n| n.to_i32())
            .unwrap_or(0);

        let owner_pid = cf_dict_get_number(&dict, "kCGWindowOwnerPID")
            .and_then(|n| n.to_i32())
            .unwrap_or(0);

        let win_num = cf_dict_get_number(&dict, "kCGWindowNumber")
            .and_then(|n| n.to_i64())
            .unwrap_or(0);

        let Some((x, y, width, height)) = bounds_rect(&dict) else {
            continue;
        };
        if width <= 0 || height <= 0 {
            continue;
        }

        let owner_name = cf_dict_get_string(&dict, "kCGWindowOwnerName").unwrap_or_default();
        let win_name = cf_dict_get_string(&dict, "kCGWindowName").unwrap_or_default();
        let title = if win_name.is_empty() {
            owner_name.clone()
        } else {
            win_name.clone()
        };

        let class_name = format!("{}:L{}", owner_name, layer);

        let is_task_bar = layer >= 24
            || title == "Menubar"
            || owner_name.contains("Dock")
            || owner_name == "Window Server"
            || owner_name == "ControlCenter";

        let z_index = -layer;

        windows.push(WindowElement {
            x,
            y,
            width,
            height,
            title,
            class_name,
            z_index,
            window_handle: win_num,
            visible: true,
            is_task_bar,
            owner_pid,
        });
    }

    windows.sort_by_key(|w: &WindowElement| -w.z_index);
    debug!("[get_all_windows macos] count={}", windows.len());
    windows
}

pub fn calculate_top_windows(windows: &Vec<WindowElement>) -> HashSet<WindowElement> {
    let mut uncovered_windows = HashSet::new();

    for (i, window) in windows.iter().enumerate() {
        if window.is_task_bar {
            uncovered_windows.insert(window.clone());
            continue;
        }

        let mut has_covered = false;
        let window_rect = Rect::new(window.x, window.y, window.width, window.height);
        for upper_window in windows.iter().take(i) {
            if upper_window.is_task_bar {
                continue;
            }
            let upper_rect = Rect::new(
                upper_window.x,
                upper_window.y,
                upper_window.width,
                upper_window.height,
            );

            if window_rect.intersects(&upper_rect) {
                debug!(
                    "[calculate_top_windows] window:{}:{}:({},{},{},{}) is covered by window:{}:{}:({},{},{},{})",
                    window.title,
                    window.class_name,
                    window.x,
                    window.y,
                    window.width,
                    window.height,
                    upper_window.title,
                    upper_window.class_name,
                    upper_window.x,
                    upper_window.y,
                    upper_window.width,
                    upper_window.height
                );
                has_covered = true;
                break;
            }
        }
        if !has_covered {
            uncovered_windows.insert(window.clone());
        }
    }
    uncovered_windows
}

pub fn calculate_covered_areas() -> (HashSet<WindowElement>, IndexMap<WindowElement, Vec<Rect>>) {
    let windows = get_all_windows();
    let mut uncovered_windows = HashSet::new();
    let mut covered_areas: IndexMap<WindowElement, Vec<Rect>> = IndexMap::new();

    for (i, window) in windows.iter().enumerate() {
        let window_rect = Rect::new(window.x, window.y, window.width, window.height);

        let mut covered = Vec::new();
        let max_covered_area = 10;
        let mut total_covered_area: Vec<Rect> = Vec::new();
        let mut is_fully_covered = false;

        for upper_window in windows.iter().take(i) {
            let upper_rect = Rect::new(
                upper_window.x,
                upper_window.y,
                upper_window.width,
                upper_window.height,
            );

            if window_rect.intersects(&upper_rect) {
                if let Some(intersection) = window_rect.intersection(&upper_rect) {
                    debug!(
                        "[calculate_covered_areas] window:{}:{}:({},{},{},{}) is covered by window:{}:{}:({},{},{},{}) with intersection:{:?}",
                        window.title,
                        window.class_name,
                        window.x,
                        window.y,
                        window.width,
                        window.height,
                        upper_window.title,
                        upper_window.class_name,
                        upper_window.x,
                        upper_window.y,
                        upper_window.width,
                        upper_window.height,
                        intersection
                    );
                    let new_covered_area = intersection.clone();
                    let mut is_new_area = true;

                    for existing_area in total_covered_area.iter() {
                        if existing_area.contains(&new_covered_area) {
                            is_new_area = false;
                            break;
                        }
                    }

                    if is_new_area {
                        if covered.len() > max_covered_area {
                            is_fully_covered = true;
                            break;
                        }
                        covered.push(intersection.clone());
                        total_covered_area.push(new_covered_area);

                        let mut remaining = vec![window_rect.clone()];
                        for covered_rect in &total_covered_area {
                            let mut new_remaining = Vec::new();
                            for rem in remaining {
                                if rem.intersects(covered_rect) {
                                    new_remaining.extend(rem.subtract(covered_rect));
                                } else {
                                    new_remaining.push(rem);
                                }
                            }
                            remaining = new_remaining;
                            if remaining.is_empty() {
                                is_fully_covered = true;
                                break;
                            }
                        }
                    }
                }
            }

            if is_fully_covered {
                break;
            }
        }
        if !is_fully_covered {
            if covered.is_empty() {
                uncovered_windows.insert(window.clone());
            } else {
                covered_areas.insert(window.clone(), covered);
            }
        }
    }

    debug!(
        "[calculate_covered_areas] uncovered windows: {:?}",
        uncovered_windows
            .iter()
            .map(|w| w.title.clone())
            .collect::<Vec<String>>()
    );
    for (window, areas) in covered_areas.iter() {
        for area in areas {
            debug!(
                "[calculate_covered_areas] window:{} covered by area:{:?}",
                window.title, area
            );
        }
    }
    (uncovered_windows, covered_areas)
}
