use log::{debug, error, info};

use crate::{
    config,
    hint::{filter_hints, hide_hints, move_hints, hint::get_hint_position_by_text},
};

use super::{keyboard::KeyboardState, mouse};

pub struct Executor<'a> {
    app_handle: &'a tauri::AppHandle,
    state: &'a mut KeyboardState,
    config: &'a config::Config,
}

impl<'a> Executor<'a> {
    pub fn new(
        app_handle: &'a tauri::AppHandle,
        config: &'a config::Config,
        state: &'a mut KeyboardState,
    ) -> Self {
        Self {
            app_handle,
            config,
            state,
        }
    }

    pub fn execute(&mut self, cmd: Option<&str>) -> bool {
        match cmd {
            Some(config::keybinding::MOVE_TO_HINT_CMD) => self.execute_move_to_hint(false),
            Some(config::keybinding::MOVE_TO_HINT_EXIT_CMD) => self.execute_move_to_hint(true),
            Some(config::keybinding::LEFT_CLICK_CMD) => self.execute_quick_click(false),
            Some(config::keybinding::LEFT_CLICK_EXIT_CMD) => self.execute_quick_click(true),
            Some(config::keybinding::HOLD_AT_HINT_CMD) => self.execute_hold_at_hint(),
            Some(config::keybinding::EXIT_CMD) => self.execute_exit(),
            Some(config::keybinding::TRANSLATE_UP_CMD) => {
                self.execute_move_hints(&self.config.keybinding.global.translate)
            }
            Some(config::keybinding::TRANSLATE_DOWN_CMD) => {
                self.execute_move_hints(&self.config.keybinding.global.translate)
            }
            Some(config::keybinding::TRANSLATE_LEFT_CMD) => {
                self.execute_move_hints(&self.config.keybinding.global.translate)
            }
            Some(config::keybinding::TRANSLATE_RIGHT_CMD) => {
                self.execute_move_hints(&self.config.keybinding.global.translate)
            }
            _ => false,
        }
    }

    pub fn execute_at_hint(&mut self, cmd: Option<&str>) -> bool {
        match cmd {
            Some(config::keybinding::HOLD_AT_HINT_CMD) => {
                // 拦截hint_key, 保持按住final_hint_key, 不传播按键
                true
            }
            Some(config::keybinding::RIGHT_CLICK_CMD) => self.execute_right_click(false),
            Some(config::keybinding::RIGHT_CLICK_EXIT_CMD) => self.execute_right_click(true),
            Some(config::keybinding::DOUBLE_CLICK_CMD) => self.execute_double_click(false),
            Some(config::keybinding::DOUBLE_CLICK_EXIT_CMD) => self.execute_double_click(true),
            Some(config::keybinding::LEFT_CLICK_CMD) => self.execute_left_click(false),
            Some(config::keybinding::LEFT_CLICK_EXIT_CMD) => self.execute_left_click(true),
            Some(config::keybinding::MIDDLE_CLICK_CMD) => self.execute_middle_click(false),
            Some(config::keybinding::MIDDLE_CLICK_EXIT_CMD) => self.execute_middle_click(true),
            Some(config::keybinding::EXIT_CMD) => self.execute_exit(),
            Some(config::keybinding::TRANSLATE_UP_CMD) => {
                self.execute_move_hints(&self.config.keybinding.at_hint.translate)
            }
            Some(config::keybinding::TRANSLATE_DOWN_CMD) => {
                self.execute_move_hints(&self.config.keybinding.at_hint.translate)
            }
            Some(config::keybinding::TRANSLATE_LEFT_CMD) => {
                self.execute_move_hints(&self.config.keybinding.at_hint.translate)
            }
            Some(config::keybinding::TRANSLATE_RIGHT_CMD) => {
                self.execute_move_hints(&self.config.keybinding.at_hint.translate)
            }
            Some(config::keybinding::SCROLL_UP_CMD) => {
                self.execute_scroll_hints(&self.config.keybinding.at_hint.scroll)
            }
            Some(config::keybinding::SCROLL_DOWN_CMD) => {
                self.execute_scroll_hints(&self.config.keybinding.at_hint.scroll)
            }
            Some(config::keybinding::SCROLL_LEFT_CMD) => {
                self.execute_scroll_hints(&self.config.keybinding.at_hint.scroll)
            }
            Some(config::keybinding::SCROLL_RIGHT_CMD) => {
                self.execute_scroll_hints(&self.config.keybinding.at_hint.scroll)
            }
            Some(config::keybinding::DRAG_UP_CMD) => {
                self.execute_drag_hints(&self.config.keybinding.at_hint.drag)
            }
            Some(config::keybinding::DRAG_DOWN_CMD) => {
                self.execute_drag_hints(&self.config.keybinding.at_hint.drag)
            }
            Some(config::keybinding::DRAG_LEFT_CMD) => {
                self.execute_drag_hints(&self.config.keybinding.at_hint.drag)
            }
            Some(config::keybinding::DRAG_RIGHT_CMD) => {
                self.execute_drag_hints(&self.config.keybinding.at_hint.drag)
            }
            _ => false,
        }
    }

    fn execute_move_to_hint(&self, exit: bool) -> bool {
        let app_handle_clone = self.app_handle.clone();
        if let Some((monitor_id, x, y)) =
            get_hint_position_by_text(&self.state.pressed_hint_keys.clone().unwrap())
        {
            tauri::async_runtime::spawn(async move {
                mouse::mouse_move(monitor_id, x, y).await;
                if exit {
                    hide_hints(app_handle_clone).await;
                }
            });
            return true;
        }
        false
    }

    fn execute_quick_click(&mut self, exit: bool) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let hold_duration = now - self.state.final_hint_key_hold_start;

        let app_handle_clone = self.app_handle.clone();
        if hold_duration < 300 {
            debug!("[execute_quick_click] execute quick click since hold_duration: {} < 300", hold_duration);
            tauri::async_runtime::spawn(async move {
                mouse::mouse_click_left().await;
                if exit {
                    hide_hints(app_handle_clone).await;
                }
            });
            return true;
        }
        false
    }

    fn execute_hold_at_hint(&mut self) -> bool {
        if !self.state.final_hint_key_hold {
            debug!("[execute_hold_at_hint] enter hold state");
            self.state.final_hint_key_hold = true;
            self.state.final_hint_key_hold_start = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            let app_handle_clone = self.app_handle.clone();
            if self.state.final_hint_key.clone().unwrap().is_empty() {
                // 未找到末位hint, 提前进入hold状态
                debug!("[execute_hold_at_hint] no final hint, directly to hold state, filter hints");
                tauri::async_runtime::spawn(async move {
                    filter_hints(app_handle_clone, "_removeAllHints".to_string()).await;
                });
            }
        } else {
            let is_dragging = self.state.is_dragging;
            debug!("[execute_hold_at_hint] already in hold state, hide hints, currently is_dragging: {}", is_dragging);
            let app_handle_clone = self.app_handle.clone();
            tauri::async_runtime::spawn(async move {
                if is_dragging {
                    mouse::mouse_drag_end().await;
                }
                hide_hints(app_handle_clone).await;
            });
        }
        true
    }

    fn execute_left_click(&self, exit: bool) -> bool {
        // 执行右键点击
        let pressed_hint_keys = self.state.pressed_hint_keys.clone().unwrap();
        let app_handle_clone = self.app_handle.clone();
        let is_dragging = self.state.is_dragging;
        tauri::async_runtime::spawn(async move {
            if is_dragging {
                mouse::mouse_drag_end().await;
            }
            if let Some((monitor_id, x, y)) = get_hint_position_by_text(&pressed_hint_keys) {
                mouse::mouse_move(monitor_id, x, y).await;
            }
            mouse::mouse_click_left().await;
            if exit {
                hide_hints(app_handle_clone).await;
            }
        });
        true
    }

    fn execute_right_click(&self, exit: bool) -> bool {
        // 执行右键点击
        let pressed_hint_keys = self.state.pressed_hint_keys.clone().unwrap();
        let is_dragging = self.state.is_dragging;
        let app_handle_clone = self.app_handle.clone();
        tauri::async_runtime::spawn(async move {
            if is_dragging {
                mouse::mouse_drag_end().await;
            }
            if let Some((monitor_id, x, y)) = get_hint_position_by_text(&pressed_hint_keys) {
                mouse::mouse_move(monitor_id, x, y).await;
            }
            mouse::mouse_click_right().await;
            if exit {
                hide_hints(app_handle_clone).await;
            }
        });
        true
    }

    fn execute_middle_click(&self, exit: bool) -> bool {
        // 执行中键点击
        let pressed_hint_keys = self.state.pressed_hint_keys.clone().unwrap();
        let is_dragging = self.state.is_dragging;
        let app_handle_clone = self.app_handle.clone();
        tauri::async_runtime::spawn(async move {
            if is_dragging {
                let _ = mouse::mouse_drag_end().await;
            }
            if let Some((monitor_id, x, y)) = get_hint_position_by_text(&pressed_hint_keys) {
                mouse::mouse_move(monitor_id, x, y).await;
            }
            mouse::mouse_click_middle().await;
            if exit {
                hide_hints(app_handle_clone).await;
            }
        });
        true
    }

    fn execute_double_click(&mut self, exit: bool) -> bool {
        // 执行双击
        let pressed_hint_keys = self.state.pressed_hint_keys.clone().unwrap();
        let is_dragging = self.state.is_dragging;
        let app_handle_clone = self.app_handle.clone();
        self.state.double_click_key_hold = true;
        tauri::async_runtime::spawn(async move {
            if is_dragging {
                mouse::mouse_drag_end().await;
            }
            if let Some((monitor_id, x, y)) = get_hint_position_by_text(&pressed_hint_keys) {
                mouse::mouse_move(monitor_id, x, y).await;
            }
            mouse::mouse_double_click().await;
            if exit {
                hide_hints(app_handle_clone).await;
            }
        });
        true
    }

    fn execute_exit(&self) -> bool {
        let app_handle_clone = self.app_handle.clone();
        let is_dragging = self.state.is_dragging;
        tauri::async_runtime::spawn(async move {
            if is_dragging {
                debug!("[execute_exit] end dragging and exit");
                mouse::mouse_drag_end().await;
            }
            hide_hints(app_handle_clone).await;
        });
        true
    }

    fn execute_move_hints(
        &self,
        key_binddings: &config::keybinding::DirectionKeybindingsConfig,
    ) -> bool {
        let hold_keys_map = &self.state.hold_keys;
        let mut hold_keys: Vec<String> = Vec::new();
        if hold_keys_map.len() > 0 {
            for (k, v) in hold_keys_map.iter() {
                if *v {
                    hold_keys.push(k.clone());
                }
            }
        }
        let mouse_step = self.config.mouse.get_translate_step(&hold_keys);
        let (dx, dy) =
            calculate_direction_delta(key_binddings, self.state, mouse_step.x, mouse_step.y);
        if dx != 0 || dy != 0 {
            let app_handle_clone = self.app_handle.clone();
            // 发送事件到前端更新显示
            tauri::async_runtime::spawn(async move {
                move_hints(app_handle_clone, (dx, dy)).await;
            });
            return true;
        }
        false
    }

    fn execute_scroll_hints(
        &self,
        key_binddings: &config::keybinding::DirectionKeybindingsConfig,
    ) -> bool {
        let hold_keys_map = &self.state.hold_keys;
        let mut hold_keys: Vec<String> = Vec::new();
        if hold_keys_map.len() > 0 {
            for (k, v) in hold_keys_map.iter() {
                if *v {
                    hold_keys.push(k.clone());
                }
            }
        }
        let mouse_step = self.config.mouse.get_scroll_step(&hold_keys);
        let (dx, dy) =
            calculate_direction_delta(key_binddings, self.state, mouse_step.x, mouse_step.y);
        if dx != 0 || dy != 0 {
            tauri::async_runtime::spawn(async move {
                mouse::mouse_wheel_move(dx, dy).await;
            });
            return true;
        }
        false
    }

    fn execute_drag_hints(
        &mut self,
        key_binddings: &config::keybinding::DirectionKeybindingsConfig,
    ) -> bool {
        let hold_keys_map = &self.state.hold_keys;
        let mut hold_keys: Vec<String> = Vec::new();
        if hold_keys_map.len() > 0 {
            for (k, v) in hold_keys_map.iter() {
                if *v {
                    hold_keys.push(k.clone());
                }
            }
        }
        let mouse_step = self.config.mouse.get_drag_step(&hold_keys);
        let (dx, dy) =
            calculate_direction_delta(key_binddings, self.state, mouse_step.x, mouse_step.y);
        if dx != 0 || dy != 0 {
            let prefix_keys = self.state.pressed_hint_keys.clone().unwrap();
            let is_dragging: bool = self.state.is_dragging;
            let start_dragging = !is_dragging;
            if start_dragging {
                self.state.is_dragging = true;
            }
            tauri::async_runtime::spawn(async move {
                if start_dragging {
                    if let Some((monitor_id, x, y)) = get_hint_position_by_text(&prefix_keys) {
                        mouse::mouse_move(monitor_id, x, y).await;
                    }
                    mouse::mouse_drag_start().await;
                }
                mouse::mouse_move_relative(dx, dy).await;
            });
            return true;
        }
        false
    }
}

pub fn calculate_direction_delta(
    key_binddings: &config::keybinding::DirectionKeybindingsConfig,
    state: &KeyboardState,
    step_x: i32,
    step_y: i32,
) -> (i32, i32) {
    let mut dx = 0;
    let mut dy = 0;
    key_binddings.up.iter().for_each(|key| {
        if let Some(is_pressed) = state.hold_keys.get(key) {
            if *is_pressed {
                dy = -step_y;
            }
        }
    });
    key_binddings.down.iter().for_each(|key| {
        if let Some(is_pressed) = state.hold_keys.get(key) {
            if *is_pressed {
                dy = step_y;
            }
        }
    });
    key_binddings.left.iter().for_each(|key| {
        if let Some(is_pressed) = state.hold_keys.get(key) {
            if *is_pressed {
                dx = -step_x;
            }
        }
    });
    key_binddings.right.iter().for_each(|key| {
        if let Some(is_pressed) = state.hold_keys.get(key) {
            if *is_pressed {
                dx = step_x;
            }
        }
    });
    debug!("[calculate_direction_delta] dx: {}, dy: {}", dx, dy);
    (dx, dy)
}
