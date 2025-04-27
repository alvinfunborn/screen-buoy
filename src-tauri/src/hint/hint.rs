use crate::hint::generator::Hint;
use crate::hint::overlay::get_overlay_monitor_id;
use log::{debug, error, info};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

// 存储所有显示的 hints 信息
pub static ACTIVE_HINTS_STORAGE: Lazy<Mutex<HashMap<String, HashMap<String, Hint>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// 存储 hints 的移动偏移量
pub static HINTS_OFFSET_STORAGE: Lazy<Mutex<(i32, i32)>> = Lazy::new(|| Mutex::new((0, 0)));

// 根据完整的 hint 文本获取 hint 的位置
pub fn get_hint_position_by_text(hint_text: &str) -> Option<(usize, i32, i32)> {
    // 获取偏移量
    let offset = if let Ok(offset) = HINTS_OFFSET_STORAGE.lock() {
        *offset
    } else {
        (0, 0)
    };
    if let Ok(hints_map) = ACTIVE_HINTS_STORAGE.lock() {
        // 遍历所有 hints，查找匹配的文本
        for (window_label, hints) in hints_map.iter() {
            if let Some(hint) = hints.get(hint_text) {
                debug!("[get_hint_position_by_text] get hint position: ({},{}) with offset: ({},{})",
                    hint.x, hint.y, offset.0, offset.1);
                return Some((
                    get_overlay_monitor_id(window_label),
                    hint.x + offset.0,
                    hint.y + offset.1,
                ));
            }
        }
    } else {
        error!("[get_hint_position_by_text] failed to get ACTIVE_HINTS_STORAGE lock");
    }

    None
}

// 更新 hints 的偏移量
pub fn update_hints_offset(dx: i32, dy: i32) {
    if let Ok(mut offset) = HINTS_OFFSET_STORAGE.lock() {
        debug!("[update_hints_offset] update hints offset: ({},{}) with dx: {}, dy: {}",
            offset.0, offset.1, dx, dy);
        offset.0 += dx;
        offset.1 += dy;
    }
}

// 重置 hints 的偏移量
fn reset_hints_offset() {
    if let Ok(mut offset) = HINTS_OFFSET_STORAGE.lock() {
        debug!("[reset_hints_offset] reset hints offset: ({},{})", offset.0, offset.1);
        *offset = (0, 0);
    }
}

// 保存 hints 信息
pub async fn save_hints(window_label: String, hints: Vec<Hint>) {
    if let Ok(mut hints_map) = ACTIVE_HINTS_STORAGE.lock() {
        debug!("[save_hints] save {} hints to ACTIVE_HINTS_STORAGE: {}", hints.len(), window_label);
        if hints_map.contains_key(&window_label) {
            hints.iter().for_each(|hint| {
                hints_map
                    .get_mut(&window_label)
                    .unwrap()
                    .insert(hint.text.clone(), hint.clone());
            });
        } else {
            hints_map.insert(
                window_label,
                hints
                    .into_iter()
                    .map(|hint| (hint.text.clone(), hint))
                    .collect(),
            );
        }
    } else {
        error!("[save_hints] failed to get ACTIVE_HINTS_STORAGE lock");
    }
}

// 清空所有 hints 信息
pub fn clear_hints() {
    if let Ok(mut hints_map) = ACTIVE_HINTS_STORAGE.lock() {
        debug!("[clear_hints] clear ACTIVE_HINTS_STORAGE");
        hints_map.clear();
    } else {
        error!("[clear_hints] failed to get ACTIVE_HINTS_STORAGE lock");
    }
    reset_hints_offset();
}
