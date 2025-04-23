use crate::utils::Rect;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ptr;
use std::time::Instant;
use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetTopWindow, GetWindow, GetWindowLongW, GetWindowRect,
    GetWindowTextW, IsIconic, IsWindowVisible, GWL_EXSTYLE, GW_HWNDNEXT, WS_EX_TOOLWINDOW,
    WS_EX_TRANSPARENT,
};

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
}

impl Hash for WindowElement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.window_handle.hash(state);
        self.class_name.hash(state);
    }
}

impl PartialEq for WindowElement {
    fn eq(&self, other: &Self) -> bool {
        self.window_handle == other.window_handle && self.class_name == other.class_name
    }
}

impl Eq for WindowElement {}

pub fn get_all_windows() -> Vec<WindowElement> {
    let mut windows = Vec::new();
    unsafe {
        EnumWindows(
            Some(enum_window_proc),
            LPARAM(&mut windows as *mut _ as isize),
        )
        .map_err(|e| format!("枚举窗口失败: {:?}", e))
        .unwrap();
    }
    windows
}

pub fn focus_to_window(x: i32, y: i32) {
    // todo: focus to window at point (x,y)
}

unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let elements = lparam.0 as *mut Vec<WindowElement>;
    let mut rect = RECT::default();

    if GetWindowRect(hwnd, &mut rect).is_ok() && IsWindowVisible(hwnd).as_bool() {
        let mut title = [0u16; 512];
        let mut class_name = [0u16; 512];

        GetWindowTextW(hwnd, &mut title);
        GetClassNameW(hwnd, &mut class_name);

        let title =
            String::from_utf16_lossy(&title[..title.iter().position(|&x| x == 0).unwrap_or(0)]);
        let class_name = String::from_utf16_lossy(
            &class_name[..class_name.iter().position(|&x| x == 0).unwrap_or(0)],
        );

        // 检查窗口样式
        let exstyle = GetWindowLongW(hwnd, GWL_EXSTYLE);
        let is_enabled = IsWindowEnabled(hwnd).as_bool();
        let is_iconic = IsIconic(hwnd).as_bool();

        // 排除特定类型的窗口
        let is_tool_window = (exstyle & WS_EX_TOOLWINDOW.0 as i32) != 0;
        let is_transparent = (exstyle & WS_EX_TRANSPARENT.0 as i32) != 0;

        // 排除系统窗口，但保留任务栏
        let is_system_window =
            class_name == "Windows.UI.Core.CoreWindow" || class_name == "Progman";

        // 特别标记任务栏窗口
        let is_taskbar = class_name == "Shell_TrayWnd" || class_name == "Shell_SecondaryTrayWnd";
        let is_tray = class_name == "TopLevelWindowForOverflowXamlIsland"
            || class_name == "NotifyIconOverflowWindow"
            || class_name == "SysPager"
            || class_name == "ToolbarWindow32"
            || class_name == "TrayNotifyWnd";

        // 检查窗口是否有效
        let has_valid_size = (rect.right - rect.left) > 0 && (rect.bottom - rect.top) > 0;

        // 打印调试信息
        // println!("窗口 - 标题:{}, 类名:{}, 有效:{}, 最小化:{}, 工具窗口:{}, 透明:{}, 系统窗口:{}, 有效尺寸:{}",
        //     title, class_name, is_enabled, is_iconic, is_tool_window, is_transparent, is_system_window, has_valid_size);

        // 如果窗口符合所有条件，则添加到列表中
        // 对任务栏窗口特殊处理，即使没有标题也允许
        if is_taskbar
            || is_tray
            || !title.is_empty()
                && is_enabled
                && !is_tool_window
                && !is_transparent
                && !is_system_window
                && has_valid_size
        {
            // 获取窗口的 Z 序，从顶层窗口开始计数
            let mut z_order = 0;
            let mut current_hwnd = GetTopWindow(None).unwrap_or(HWND(ptr::null_mut()));
            while current_hwnd.0 != ptr::null_mut() {
                if current_hwnd == hwnd {
                    break;
                }
                z_order -= 1; // 越往下 z_order 越小
                current_hwnd =
                    GetWindow(current_hwnd, GW_HWNDNEXT).unwrap_or(HWND(ptr::null_mut()));
            }

            let window_element = WindowElement {
                x: rect.left,
                y: rect.top,
                width: rect.right - rect.left,
                height: rect.bottom - rect.top,
                title: title.clone(),
                class_name: class_name.clone(),
                z_index: z_order,
                window_handle: hwnd.0 as i64,
                visible: !is_iconic,
            };

            (*elements).push(window_element);
        }
    }

    BOOL(1)
}

pub fn calculate_covered_areas() -> (HashSet<WindowElement>, IndexMap<WindowElement, Vec<Rect>>) {
    let start = Instant::now();
    let mut windows = get_all_windows();

    // 按Z序从高到低排序窗口（z_index越大越靠近顶层）
    windows = windows.iter().filter(|w| w.visible).cloned().collect();
    windows.sort_by_key(|w: &WindowElement| -w.z_index);

    let mut uncovered_windows = HashSet::new();
    let mut covered_areas: IndexMap<WindowElement, Vec<Rect>> = IndexMap::new();

    // 从顶层窗口开始遍历
    for (i, window) in windows.iter().enumerate() {
        let window_rect = Rect::new(window.x, window.y, window.width, window.height);

        // 初始化当前窗口的遮挡区域
        let mut covered = Vec::new();
        let max_covered_area = 10;
        let mut total_covered_area: Vec<Rect> = Vec::new(); // 用于追踪所有已遮挡的区域
        let mut is_fully_covered = false;

        // 检查上层窗口的遮挡
        for upper_window in windows.iter().take(i) {
            let upper_rect = Rect::new(
                upper_window.x,
                upper_window.y,
                upper_window.width,
                upper_window.height,
            );

            // 如果有重叠，添加遮挡区域
            if window_rect.intersects(&upper_rect) {
                if let Some(intersection) = window_rect.intersection(&upper_rect) {
                    // 检查这个新的遮挡区域是否与已有的遮挡区域重叠
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

                        // 计算当前总遮挡面积
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
            // 保存遮挡区域
            if covered.is_empty() {
                uncovered_windows.insert(window.clone());
            } else {
                covered_areas.insert(window.clone(), covered);
            }
        }
    }

    // println!(
    //     "未被遮挡的窗口: {:?}",
    //     uncovered_windows
    //         .iter()
    //         .map(|w| w.title.clone())
    //         .collect::<Vec<String>>()
    // );
    // for (window, areas) in covered_areas.iter() {
    //     println!("窗口:{}，遮挡区域数量:{}", window.title, areas.len());
    //     for area in areas {
    //         println!("窗口:{}，遮挡区域:{:?}", window.title, area);
    //     }
    // }
    // println!("calculate_covered_areas 耗时: {:?}", start.elapsed());
    (uncovered_windows, covered_areas)
}
