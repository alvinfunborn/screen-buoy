use std::sync::Mutex;

use once_cell::sync::Lazy;
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::{
        Input::KeyboardAndMouse::{
            GetKeyboardState, ToUnicode, VK_BACK, VK_DELETE, VK_DOWN, VK_ESCAPE, VK_LCONTROL,
            VK_LEFT, VK_LMENU, VK_LSHIFT, VK_OEM_2, VK_OEM_3, VK_OEM_4, VK_OEM_5, VK_OEM_6,
            VK_OEM_7, VK_OEM_8, VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS,
            VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_SPACE, VK_TAB, VK_UP,
        },
        WindowsAndMessaging::{
            CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
            WH_KEYBOARD_LL, WM_KEYDOWN,
        },
    },
};

use crate::configs;

use super::keyboard::handle_keyboard_event;

// 包装 HHOOK
struct HookHandle(HHOOK);
unsafe impl Send for HookHandle {}

// 设置全局键盘钩子
pub unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let key_info = *(lparam.0 as *const KBDLLHOOKSTRUCT);
    let is_down = wparam.0 == WM_KEYDOWN as usize;
    let vk_code = key_info.vkCode as u16;

    let config = configs::get_config().expect("配置未初始化");
    let keyboard_config = &config.keyboard;
    let key: String;
    if let Some(key_config) = keyboard_config.get_key_by_virtual_key(vk_code) {
        key = key_config.key.clone();
    } else {
        // 转换虚拟键码到字符串
        key = match vk_code {
            // 数字键 (0-9)
            0x30..=0x39 => {
                let mut key_state = [0u8; 256];
                let _ = GetKeyboardState(&mut key_state);
                let mut result = [0u16; 2];
                let chars = ToUnicode(
                    vk_code as u32,
                    key_info.scanCode as u32,
                    Some(&key_state),
                    &mut result,
                    0,
                );
                if chars > 0 {
                    if let Some(c) = char::from_u32(result[0] as u32) {
                        c.to_string()
                    } else {
                        return CallNextHookEx(None, code, wparam, lparam);
                    }
                } else {
                    return CallNextHookEx(None, code, wparam, lparam);
                }
            }
            // 字母键 (A-Z)
            0x41..=0x5A => {
                let mut key_state = [0u8; 256];
                let _ = GetKeyboardState(&mut key_state);
                let mut result = [0u16; 2];
                let chars = ToUnicode(
                    vk_code as u32,
                    key_info.scanCode as u32,
                    Some(&key_state),
                    &mut result,
                    0,
                );
                if chars > 0 {
                    if let Some(c) = char::from_u32(result[0] as u32) {
                        c.to_string().to_uppercase()
                    } else {
                        return CallNextHookEx(None, code, wparam, lparam);
                    }
                } else {
                    return CallNextHookEx(None, code, wparam, lparam);
                }
            }
            x if x == VK_OEM_PLUS.0 => "=".to_string(),
            x if x == VK_OEM_MINUS.0 => "-".to_string(),
            x if x == VK_OEM_COMMA.0 => ",".to_string(),
            x if x == VK_OEM_PERIOD.0 => ".".to_string(),
            x if x == VK_OEM_2.0 => "/".to_string(),
            x if x == VK_OEM_3.0 => "`".to_string(),
            x if x == VK_OEM_4.0 => "[".to_string(),
            x if x == VK_OEM_5.0 => "\\".to_string(),
            x if x == VK_OEM_6.0 => "]".to_string(),
            x if x == VK_OEM_7.0 => "'".to_string(),
            x if x == VK_OEM_8.0 => ";".to_string(),
            _ => return CallNextHookEx(None, code, wparam, lparam),
        };
    }
    if let Ok(app_handle_lock) = APP_HANDLE.lock() {
        if let Some(app_handle) = app_handle_lock.as_ref() {
            if handle_keyboard_event(app_handle, &key, is_down) {
                println!("[键盘钩子] 事件已处理，不传递");
                return LRESULT(1);
            }
        } else {
            println!("[键盘钩子] APP_HANDLE 为 None");
        }
    } else {
        println!("[键盘钩子] 无法获取 APP_HANDLE 锁");
    }

    CallNextHookEx(None, code, wparam, lparam)
}

// 全局钩子ID
static HOOK_ID: Lazy<Mutex<Option<HookHandle>>> = Lazy::new(|| Mutex::new(None));
static APP_HANDLE: Lazy<Mutex<Option<tauri::AppHandle>>> = Lazy::new(|| Mutex::new(None));

pub fn init(app_handle: tauri::AppHandle) {
    println!("[键盘钩子] 初始化键盘钩子");
    *APP_HANDLE.lock().unwrap() = Some(app_handle);

    unsafe {
        if let Ok(hook) = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0) {
            *HOOK_ID.lock().unwrap() = Some(HookHandle(hook));
            println!("[键盘钩子] 键盘钩子设置成功");
        } else {
            println!("[键盘钩子] 键盘钩子设置失败");
        }
    }
}

pub fn cleanup() {
    println!("[键盘钩子] 清理键盘钩子");
    if let Ok(mut hook_id) = HOOK_ID.lock() {
        if let Some(hook) = hook_id.take() {
            unsafe {
                if let Ok(_) = UnhookWindowsHookEx(hook.0) {
                    println!("[键盘钩子] 键盘钩子清理成功");
                } else {
                    println!("[键盘钩子] 键盘钩子清理失败");
                }
            }
        } else {
            println!("[键盘钩子] 没有找到键盘钩子");
        }
    }
}
