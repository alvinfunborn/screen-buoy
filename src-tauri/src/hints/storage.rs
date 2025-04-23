use crate::hints::generator::Hint;
use crate::hints::overlay::get_overlay_monitor_id;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

// 存储所有显示的 hints 信息
pub static ACTIVE_HINTS_STORAGE: Lazy<Mutex<HashMap<String, HashMap<String, Hint>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// 存储 hints 的移动偏移量
pub static HINTS_OFFSET_STORAGE: Lazy<Mutex<(i32, i32)>> =
    Lazy::new(|| Mutex::new((0, 0)));


// 根据完整的 hint 文本获取 hint 的位置
pub fn get_hint_position_by_text(hint_text: &str) -> Option<(usize, i32, i32)> {
    println!("[存储] 根据文本获取hint位置: {}", hint_text);

    // 获取偏移量
    let offset = if let Ok(offset) = HINTS_OFFSET_STORAGE.lock() {
        *offset
    } else {
        (0, 0)
    };
    println!("[存储] 当前偏移量: {:?}", offset);
    if let Ok(hints_map) = ACTIVE_HINTS_STORAGE.lock() {
        println!("[存储] 当前共有 {} 个窗口的hints", hints_map.len());
        // 遍历所有 hints，查找匹配的文本
        for (window_label, hints) in hints_map.iter() {
            if let Some(hint) = hints.get(hint_text) {
                println!("[存储] 找到匹配的hint: {}, 原始位置: ({}, {}), 偏移后位置: ({}, {})", 
                    hint.text, hint.x, hint.y, hint.x + offset.0, hint.y + offset.1);
                return Some((get_overlay_monitor_id(window_label), hint.x + offset.0, hint.y + offset.1));
            }
        }
        println!("[存储] 没有找到匹配的hint: {}", hint_text);
    } else {
        eprintln!("[存储] 无法获取 ACTIVE_HINTS_STORAGE 锁");
    }

    None
}

// 更新 hints 的偏移量
pub fn update_hints_offset(dx: i32, dy: i32) {
    if let Ok(mut offset) = HINTS_OFFSET_STORAGE.lock() {
        println!("[存储] 更新偏移量: {:?}", offset);
        offset.0 += dx;
        offset.1 += dy;
    }
}

// 重置 hints 的偏移量
fn reset_hints_offset() {
    if let Ok(mut offset) = HINTS_OFFSET_STORAGE.lock() {
        *offset = (0, 0);
    }
}

// 保存 hints 信息
pub async fn save_hints(window_label: String, hints: Vec<Hint>) {
    println!("[存储] 保存窗口 {} 的 {} 个 hints", window_label, hints.len());
    if let Ok(mut hints_map) = ACTIVE_HINTS_STORAGE.lock() {
        if hints_map.contains_key(&window_label) {
            hints.iter().for_each(|hint| {
                hints_map.get_mut(&window_label).unwrap().insert(hint.text.clone(), hint.clone());
            });
        } else {
            hints_map.insert(window_label, hints.into_iter().map(|hint| (hint.text.clone(), hint)).collect());
        }
        println!("[存储] 当前共有 {} 个窗口的 hints", hints_map.len());
    } else {
        println!("[存储] 无法获取 ACTIVE_HINTS_STORAGE 锁");
    }
}

// 清空所有 hints 信息
pub fn clear_hints() {
    println!("[存储] 清空所有 hints 信息");
    if let Ok(mut hints_map) = ACTIVE_HINTS_STORAGE.lock() {
        let count = hints_map.len();
        hints_map.clear();
        println!("[存储] 已清空 {} 个窗口的 hints", count);
    } else {
        println!("[存储] 无法获取 ACTIVE_HINTS_STORAGE 锁");
    }
    reset_hints_offset();
}
