use crate::monitor::MONITORS_STORAGE;
use log::{error, info};
use serde::{Deserialize, Serialize};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    mouse_event, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL,
};
use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;

fn move_to(monitor: usize, x: i32, y: i32) -> windows::core::Result<()> {
    // 获取显示器信息
    if let Ok(monitors) = MONITORS_STORAGE.lock() {
        if let Some(monitor_info) = monitors.get(monitor) {
            // 检查坐标是否在显示器范围内
            let mut x = x * monitor_info.scale_factor as i32;
            let mut y = y * monitor_info.scale_factor as i32;
            if x < 0 {
                x = 0;
            } else if x > monitor_info.width {
                x = monitor_info.width;
            }
            if y < 0 {
                y = 0;
            } else if y > monitor_info.height {
                y = monitor_info.height;
            }

            // 计算全局坐标
            let global_x = monitor_info.x + x;
            let global_y = monitor_info.y + y;

            unsafe { SetCursorPos(global_x, global_y) }
        } else {
            error!("[move_to] monitor not found: {}", monitor);
            Err(windows::core::Error::from_win32())
        }
    } else {
        error!("[move_to] failed to get MONITORS_STORAGE lock");
        Err(windows::core::Error::from_win32())
    }
}

fn move_relative(delta_x: i32, delta_y: i32) -> windows::core::Result<()> {
    unsafe {
        mouse_event(MOUSEEVENTF_MOVE, delta_x, delta_y, 0, 0);
        Ok(())
    }
}

fn click_left() -> windows::core::Result<()> {
    unsafe {
        mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
        mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
        Ok(())
    }
}

fn click_right() -> windows::core::Result<()> {
    unsafe {
        mouse_event(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0, 0);
        mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0, 0);
        Ok(())
    }
}

fn click_middle() -> windows::core::Result<()> {
    unsafe {
        mouse_event(MOUSEEVENTF_MIDDLEDOWN, 0, 0, 0, 0);
        mouse_event(MOUSEEVENTF_MIDDLEUP, 0, 0, 0, 0);
        Ok(())
    }
}

fn start_drag() -> windows::core::Result<()> {
    unsafe {
        mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
        Ok(())
    }
}

fn end_drag() -> windows::core::Result<()> {
    unsafe {
        mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
        Ok(())
    }
}

fn wheel_move(delta_x: i32, delta_y: i32) -> windows::core::Result<()> {
    unsafe {
        // 垂直滚动
        if delta_y != 0 {
            mouse_event(MOUSEEVENTF_WHEEL, 0, 0, delta_y, 0);
        }
        // 水平滚动
        if delta_x != 0 {
            mouse_event(MOUSEEVENTF_HWHEEL, 0, 0, delta_x, 0);
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Point {
    x: i32,
    y: i32,
}

pub async fn mouse_click_left() {
    if let Err(e) = click_left() {
        error!("[mouse_click_left] failed: {}", e);
    }
}

pub async fn mouse_click_right() {
    if let Err(e) = click_right() {
        error!("[mouse_click_right] failed: {}", e);
    }
}

pub async fn mouse_click_middle() {
    if let Err(e) = click_middle() {
        error!("[mouse_click_middle] failed: {}", e);
    }
}

pub async fn mouse_double_click() {
    if let Err(e) = click_left() {
        error!("[mouse_double_click] failed: {}", e);
    }
    // 等待15毫秒，这是Windows默认双击间隔的一小部分
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    if let Err(e) = click_left() {
        error!("[mouse_double_click] failed: {}", e);
    }
}

pub async fn mouse_move(monitor: usize, x: i32, y: i32) {
    if let Err(e) = move_to(monitor, x, y) {
        error!("[mouse_move] failed: {}", e);
    }
}

pub async fn mouse_move_relative(delta_x: i32, delta_y: i32) {
    if let Err(e) = move_relative(delta_x, delta_y) {
        error!("[mouse_move_relative] failed: {}", e);
    }
}

pub async fn mouse_drag_start() {
    if let Err(e) = start_drag() {
        error!("[mouse_drag_start] failed: {}", e);
    }
}

pub async fn mouse_drag_end() {
    if let Err(e) = end_drag() {
        error!("[mouse_drag_end] failed: {}", e);
    }
}

pub async fn mouse_wheel_move(delta_x: i32, delta_y: i32) {
    if let Err(e) = wheel_move(delta_x, delta_y) {
        error!("[mouse_wheel_move] failed: {}", e);
    }
}

pub async fn hide_cursor() {
}

pub async fn show_cursor() {
}


