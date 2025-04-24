use crate::{
    config,
    element::{UIElement, WINDOWS_UI_ELEMENTS_MAP_STORAGE},
    monitor::{MonitorInfo, MONITORS_STORAGE},
    utils::Rect,
    window::{window::calculate_covered_areas, WindowElement},
};
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use super::overlay::OVERLAY_WINDOW_PREFIX;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hint {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub scale: f64,
    // hint_type: 0-default, 1-window, 2-pane, 3-tab, 4-button, 5-scrollbar
    pub hint_type: usize,
}

static HINT_TEXT_LIST_STORAGE: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn init_hint_text_list_storage() {
    let configs = config::get_config().expect("配置未初始化");
    let hint_config = &configs.hint;
    let mut list = Vec::new();
    generate_n_digit_hints(&hint_config.charsets, String::new(), 0, &mut list);
    // 如果有额外字符集，生成n+1位hints
    if *config::hint::HAS_EXTRA_CHARSET {
        for &extra_char in hint_config.charset_extra.iter() {
            let mut prefix = String::new();
            prefix.push(extra_char);
            generate_n_digit_hints(&hint_config.charsets, prefix, 0, &mut list);
        }
    }
    *HINT_TEXT_LIST_STORAGE.lock().unwrap() = list;
}

// 生成n位hints
fn generate_n_digit_hints(
    charsets: &[Vec<char>],
    current: String,
    depth: usize,
    result: &mut Vec<String>,
) {
    if depth == charsets.len() {
        result.push(current);
        return;
    }

    for &c in charsets[depth].iter() {
        let mut new_current = current.clone();
        new_current.push(c);
        generate_n_digit_hints(charsets, new_current, depth + 1, result);
    }
}

pub struct HintsGenerator {
    monitors: Vec<MonitorInfo>,
    uncovered_windows: HashSet<WindowElement>,
    windows_covered_areas: IndexMap<WindowElement, Vec<Rect>>,
    ui_elements: HashMap<WindowElement, Vec<UIElement>>,
}

impl HintsGenerator {
    pub fn new() -> Self {
        let monitors = MONITORS_STORAGE.lock().unwrap().clone();
        let (uncovered_windows, windows_covered_areas) = calculate_covered_areas();
        let ui_elements = WINDOWS_UI_ELEMENTS_MAP_STORAGE.lock().unwrap().clone();

        Self {
            monitors,
            uncovered_windows,
            windows_covered_areas,
            ui_elements,
        }
    }

    pub fn generate_hints_batch1(
        &self,
        position_set: &mut HashSet<(i32, i32)>,
        hints_count: &mut i32,
    ) -> HashMap<String, Vec<Hint>> {
        let mut monitor_hints = HashMap::new();

        for window_element in self.uncovered_windows.iter() {
            if let Some(elements) = self.ui_elements.get(window_element) {
                self.do_generate_hints(
                    &mut monitor_hints,
                    window_element,
                    elements,
                    &Vec::new(),
                    position_set,
                    hints_count,
                );
            }
        }
        monitor_hints
    }

    pub fn generate_hints_batch2(
        &self,
        position_set: &mut HashSet<(i32, i32)>,
        hints_count: &mut i32,
    ) -> HashMap<String, Vec<Hint>> {
        let mut monitor_hints = HashMap::new();
        for (window_element, areas) in &self.windows_covered_areas {
            if let Some(elements) = self.ui_elements.get(window_element) {
                self.do_generate_hints(
                    &mut monitor_hints,
                    window_element,
                    elements,
                    areas,
                    position_set,
                    hints_count,
                );
            }
        }
        monitor_hints
    }

    fn do_generate_hints(
        &self,
        monitor_hints: &mut HashMap<String, Vec<Hint>>,
        window_element: &WindowElement,
        ui_elements: &Vec<UIElement>,
        covered_areas: &Vec<Rect>,
        position_set: &mut HashSet<(i32, i32)>,
        hints_count: &mut i32,
    ) {
        for hint in ui_elements {
            if !position_set.insert((hint.x, hint.y)) {
                println!(
                    "window:{}, hint:{}:({},{}) 已存在，跳过",
                    window_element.title, hint.text, hint.x, hint.y
                );
                continue;
            }
            // 找到hint所在的显示器
            for (index, monitor) in self.monitors.iter().enumerate() {
                if hint.x < monitor.x
                    || hint.x >= monitor.x + monitor.width
                    || hint.y < monitor.y
                    || hint.y >= monitor.y + monitor.height
                {
                    continue;
                }
                // 检查hint是否在窗口的可见区域内
                let mut is_covered = false;
                for area in covered_areas {
                    if area.contains_point(hint.x, hint.y) {
                        is_covered = true;
                        // println!(
                        //     "window:{}, hint:{}:({},{}) 在遮挡区域:{:?}内，跳过",
                        //     window_element.title, hint.text, hint.x, hint.y, area
                        // );
                        break;
                    }
                }

                if !is_covered {
                    // 检查是否超出范围
                    if *hints_count >= HINT_TEXT_LIST_STORAGE.lock().unwrap().len() as i32 {
                        return;
                    }

                    // 转换为相对于显示器的坐标
                    let mut hint = hint.clone();
                    hint.x -= monitor.x;
                    hint.y -= monitor.y;
                    hint.x = (hint.x as f64 / monitor.scale_factor) as i32;
                    hint.y = (hint.y as f64 / monitor.scale_factor) as i32;
                    let hint_letter =
                        HINT_TEXT_LIST_STORAGE.lock().unwrap()[*hints_count as usize].clone();
                    let hint_type = hint.element_type;
                    // println!("window:{}, NO.{}hint:{},type:{},ctrl_type:{},pos:({},{}):{} 在显示器{}内，添加到hints",
                    //     window_element.title, *hints_count, hint_letter, hint_type, hint.control_type, hint.x, hint.y, hint.text, monitor.id);
                    let hint = Hint {
                        text: hint_letter,
                        x: hint.x,
                        y: hint.y,
                        z: hint.z,
                        scale: monitor.scale_factor,
                        hint_type,
                    };
                    let window_label = format!("{}{}", OVERLAY_WINDOW_PREFIX, index);
                    monitor_hints
                        .entry(window_label.clone())
                        .or_default()
                        .push(hint);
                    *hints_count += 1;
                }
                break;
            }
        }
    }
}
