use crate::{
    config::{self, hint::HINT_TYPE_ID_MAP},
    element::UIElement,
    monitor::{MonitorInfo, MONITORS_STORAGE},
    utils::Rect,
    window::{window::covered_areas_for_windows, WindowElement},
};
use indexmap::IndexMap;
use log::{debug, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

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

pub fn has_capacity(count: i32) -> bool {
    (count as usize) < HINT_TEXT_LIST_STORAGE.lock().unwrap().len()
}

pub fn init_hint_text_list_storage() {
    let configs = config::get_config().unwrap();
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
    info!("[init_hint_text_list_storage] init hint text list storage: {}", list.len());
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

// Geometry is shared by every candidate batch until the window layout changes.
pub(super) struct HintLayout {
    monitors: Vec<MonitorInfo>,
    monitor_bounds: Vec<Rect>,
    uncovered_windows: HashSet<WindowElement>,
    windows_covered_areas: IndexMap<WindowElement, Vec<Rect>>,
}

impl HintLayout {
    pub(super) fn new(windows: &[WindowElement]) -> Self {
        Self::with_monitors(windows, MONITORS_STORAGE.lock().unwrap().clone())
    }

    pub(super) fn with_monitors(windows: &[WindowElement], monitors: Vec<MonitorInfo>) -> Self {
        let monitor_bounds = monitors.iter().map(|monitor| {
            #[cfg(target_os = "macos")]
            return Rect::new(
                (monitor.x as f64 / monitor.scale_factor) as i32,
                (monitor.y as f64 / monitor.scale_factor) as i32,
                (monitor.width as f64 / monitor.scale_factor) as i32,
                (monitor.height as f64 / monitor.scale_factor) as i32,
            );
            #[cfg(target_os = "windows")]
            return Rect::new(monitor.x, monitor.y, monitor.width, monitor.height);
        }).collect();
        let (uncovered_windows, windows_covered_areas) = covered_areas_for_windows(windows);
        Self { monitors, monitor_bounds, uncovered_windows, windows_covered_areas }
    }

    pub(super) fn contains_point(&self, x: i32, y: i32) -> bool {
        self.monitor_bounds.iter().any(|bounds| bounds.contains_point(x, y))
    }
}

pub struct HintsGenerator {
    layout: Arc<HintLayout>,
    ui_elements: HashMap<WindowElement, Arc<Vec<UIElement>>>,
    #[cfg(target_os = "macos")]
    visibility: super::occlusion_macos::PointVisibility,
}

impl HintsGenerator {
    pub(super) fn new(layout: Arc<HintLayout>, ui_elements: HashMap<WindowElement, Arc<Vec<UIElement>>>) -> Self {
        Self {
            layout,
            ui_elements,
            #[cfg(target_os = "macos")]
            visibility: Default::default(),
        }
    }

    #[cfg(target_os = "macos")]
    pub async fn prepare_visibility(
        mut self,
        app: &tauri::AppHandle,
        activation: u64,
    ) -> Option<Self> {
        let windows = self.layout.uncovered_windows.iter().map(|window| (window, &[][..]))
            .chain(self.layout.windows_covered_areas.iter().map(|(window, areas)| (window, areas.as_slice())));
        for (window, areas) in windows {
            if let Some(elements) = self.ui_elements.get(window) {
                for element in elements.iter() {
                    self.visibility.add_candidate(
                        (element.x, element.y),
                        self.layout.contains_point(element.x, element.y),
                        areas.iter().any(|area| area.contains_point(element.x, element.y)),
                    );
                }
            }
        }
        if self.visibility.prepare(app, activation).await {
            Some(self)
        } else {
            None
        }
    }

    #[cfg(target_os = "macos")]
    pub fn log_performance(&self, activation: u64, hints: i32, grid: i32, total: std::time::Duration) {
        self.visibility.log_performance(activation, hints, grid, total);
    }

    pub fn generate_hints_grid(
        &self,
        hints_count: &mut i32,
    ) -> HashMap<String, Vec<Hint>> {
        let mut monitor_hints = HashMap::new();
        let config = config::get_config().unwrap();
        let grid_row = config.hint.grid.rows;
        let grid_column = config.hint.grid.columns;
        let show_at_row = config.hint.grid.show_at_rows;
        let show_at_column = config.hint.grid.show_at_columns;
        let hint_type = config.hint.grid.hint_type;
        let hint_z = config.hint.types.get(&hint_type).unwrap().z_index;
        for (index, monitor) in self.layout.monitors.iter().enumerate() {
            let mut hints = Vec::new();
            for row in show_at_row.iter() {
                for column in show_at_column.iter() {
                    // 检查是否超出范围
                    if *hints_count >= HINT_TEXT_LIST_STORAGE.lock().unwrap().len() as i32 {
                        return monitor_hints;
                    }
                    let hint = Hint {
                        text: HINT_TEXT_LIST_STORAGE.lock().unwrap()[*hints_count as usize].clone(),
                        x: (((*column as f64) - 0.5) * monitor.width as f64 / grid_column as f64 / monitor.scale_factor) as i32,
                        y: (((*row as f64) - 0.5) * monitor.height as f64 / grid_row as f64 / monitor.scale_factor) as i32,
                        z: hint_z,
                        scale: monitor.scale_factor,
                        hint_type: HINT_TYPE_ID_MAP.get(&hint_type).unwrap().clone(),
                    };
                    hints.push(hint);
                    *hints_count += 1;
                }
            }
            monitor_hints.insert(format!("{}{}", OVERLAY_WINDOW_PREFIX, index), hints);
        }
        debug!("[generate_hints_grid] generate {} hints: {:?}", monitor_hints.len(), monitor_hints);
        monitor_hints
    }

    pub fn generate_hints_batch1(
        &self,
        position_set: &mut HashSet<(i32, i32)>,
        hints_count: &mut i32,
    ) -> HashMap<String, Vec<Hint>> {
        let mut monitor_hints = HashMap::new();

        for window_element in self.layout.uncovered_windows.iter() {
            if let Some(elements) = self.ui_elements.get(window_element) {
                self.do_generate_hints(
                    &mut monitor_hints,
                    window_element,
                    elements,
                    &Vec::new(),
                    position_set,
                    hints_count,
                    false,
                );
            }
        }
        debug!("[generate_hints_batch1] generate hints for {} windows", monitor_hints.len());
        monitor_hints
    }

    pub fn generate_hints_batch2(
        &self,
        position_set: &mut HashSet<(i32, i32)>,
        hints_count: &mut i32,
    ) -> HashMap<String, Vec<Hint>> {
        let mut monitor_hints = HashMap::new();
        for (window_element, areas) in &self.layout.windows_covered_areas {
            if let Some(elements) = self.ui_elements.get(window_element) {
                self.do_generate_hints(
                    &mut monitor_hints,
                    window_element,
                    elements,
                    areas,
                    position_set,
                    hints_count,
                    false,
                );
            }
        }
        debug!("[generate_hints_batch2] generate hints for {} windows", monitor_hints.len());
        monitor_hints
    }

    // Call only with the intersection of recent verified coordinates and the
    // current source snapshots, after checking the complete window layout.
    pub(super) fn generate_verified_hints(
        &self,
        windows: &[WindowElement],
        position_set: &mut HashSet<(i32, i32)>,
        hints_count: &mut i32,
    ) -> HashMap<String, Vec<Hint>> {
        let mut hints = HashMap::new();
        for window in windows {
            if let Some(elements) = self.ui_elements.get(window) {
                self.do_generate_hints(&mut hints, window, elements, &[], position_set, hints_count, true);
            }
        }
        hints
    }

    fn do_generate_hints(
        &self,
        monitor_hints: &mut HashMap<String, Vec<Hint>>,
        window_element: &WindowElement,
        ui_elements: &Vec<UIElement>,
        covered_areas: &[Rect],
        position_set: &mut HashSet<(i32, i32)>,
        hints_count: &mut i32,
        verified: bool,
    ) {
        for hint in ui_elements {
            for (index, (monitor, bounds)) in self.layout.monitors.iter()
                .zip(&self.layout.monitor_bounds).enumerate() {
                if !bounds.contains_point(hint.x, hint.y) {
                    continue;
                }

                let mut is_covered = false;
                for area in covered_areas {
                    if area.contains_point(hint.x, hint.y) {
                        is_covered = true;
                        #[cfg(target_os = "windows")]
                        debug!(
                            "[generator] skip window:{}, hint:{}:({},{}) was covered by {:?}",
                            window_element.title, hint.text, hint.x, hint.y, area
                        );
                        break;
                    }
                }

                #[cfg(target_os = "macos")]
                let is_covered = !self.visibility.visible(
                    window_element.window_handle,
                    (hint.x, hint.y),
                    is_covered,
                );

                if verified || !is_covered {
                    // A rejected transparent host / hidden element must not claim
                    // the coordinate before its visible owner is considered.
                    if !position_set.insert((hint.x, hint.y)) {
                        break;
                    }
                    if *hints_count >= HINT_TEXT_LIST_STORAGE.lock().unwrap().len() as i32 {
                        debug!("[generator] skip hint:{}:({},{}) due to hint text is out of use",
                            hint.text, hint.x, hint.y);
                        return;
                    }

                    let mut hint = hint.clone();
                    #[cfg(target_os = "macos")]
                    {
                        hint.x -= bounds.x;
                        hint.y -= bounds.y;
                    }
                    #[cfg(target_os = "windows")]
                    {
                        hint.x -= monitor.x;
                        hint.y -= monitor.y;
                        hint.x = (hint.x as f64 / monitor.scale_factor) as i32;
                        hint.y = (hint.y as f64 / monitor.scale_factor) as i32;
                    }
                    let hint_letter =
                        HINT_TEXT_LIST_STORAGE.lock().unwrap()[*hints_count as usize].clone();
                    let hint_type = hint.element_type;
                    debug!("[generator] add window:{}, NO.{}hint:{},type:{},ctrl_type:{},pos:({},{}):{} to monitor:{}",
                        window_element.title, *hints_count, hint_letter, hint_type, hint.control_type, hint.x, hint.y, hint.text, monitor.id);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefilter_matches_render_bounds_for_negative_and_mixed_scale_displays() {
        let layout = HintLayout::with_monitors(&[], vec![
            MonitorInfo { id: 0, x: -2400, y: -1800, width: 2400, height: 1800, scale_factor: 2.0 },
            MonitorInfo { id: 1, x: 0, y: 0, width: 1600, height: 900, scale_factor: 1.0 },
        ]);
        #[cfg(target_os = "macos")]
        let origin = (-1200, -900);
        #[cfg(target_os = "windows")]
        let origin = (-2400, -1800);
        assert!(layout.contains_point(origin.0, origin.1));
        assert!(layout.contains_point(-1, -1));
        assert!(layout.contains_point(1599, 899));
        assert!(!layout.contains_point(origin.0 - 1, origin.1));
        assert!(!layout.contains_point(-1, 0));
        assert!(!layout.contains_point(1600, 0));
        assert!(!layout.contains_point(0, 900));
    }

    #[test]
    fn append_preserves_allocations_and_deduplicates_coordinates_across_scans() {
        *HINT_TEXT_LIST_STORAGE.lock().unwrap() = vec!["AA".into(), "AS".into(), "AD".into()];
        let window = WindowElement {
            window_handle: 1, owner_pid: 1, x: 0, y: 0, width: 800, height: 600,
            title: String::new(), class_name: String::new(), z_index: 0,
            visible: true, is_task_bar: false,
        };
        let mut generator = HintsGenerator {
            layout: Arc::new(HintLayout::with_monitors(&[window.clone()],
                vec![MonitorInfo { id: 0, x: 0, y: 0, width: 800, height: 600, scale_factor: 1.0 }])),
            ui_elements: HashMap::new(),
            #[cfg(target_os = "macos")]
            visibility: Default::default(),
        };
        let element = |x| UIElement { text: String::new(), x, y: 100, z: 1, width: 10,
            height: 10, window_handle: 1, control_type: 0, element_type: 0 };
        let mut assigned = HashSet::new();
        let mut count = 0;
        generator.ui_elements.insert(window.clone(), Arc::new(vec![element(100), element(200)]));
        let first = generator.generate_hints_batch1(&mut assigned, &mut count);
        let first = first.values().next().unwrap();
        assert_eq!((&first[0].text, first[0].x), (&"AA".to_owned(), 100));
        assert_eq!((&first[1].text, first[1].x), (&"AS".to_owned(), 200));
        // The first point disappeared, another moved, and the second is duplicated.
        generator.ui_elements.insert(window, Arc::new(vec![element(200), element(300), element(400)]));
        let appended = generator.generate_hints_batch1(&mut assigned, &mut count);
        let appended = appended.values().next().unwrap();
        assert_eq!(appended.len(), 1); // Capacity exhaustion never recycles AA/AS.
        assert_eq!((&appended[0].text, appended[0].x), (&"AD".to_owned(), 300));
        assert_eq!(first[0].x, 100);
        assert_eq!(count, 3);
    }
}
