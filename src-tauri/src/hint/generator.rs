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
    let list = super::codes::generate(&hint_config.charsets, &hint_config.charset_extra);
    info!("[init_hint_text_list_storage] init hint text list storage: {}", list.len());
    *HINT_TEXT_LIST_STORAGE.lock().unwrap() = list;
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
    ) -> Self {
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
        self.visibility.prepare(app).await;
        self
    }

    #[cfg(target_os = "macos")]
    pub fn log_performance(&self, epoch: u64, visible: usize, total: std::time::Duration) {
        self.visibility.log_performance(epoch, visible, total);
    }

    fn point_visible(&self, window: &WindowElement, element: &UIElement, areas: &[Rect]) -> bool {
        if !self.layout.contains_point(element.x, element.y) {
            return false;
        }
        let covered = areas.iter().any(|area| area.contains_point(element.x, element.y));
        #[cfg(target_os = "macos")]
        return self.visibility.visible(window.window_handle, (element.x, element.y), covered);
        #[cfg(target_os = "windows")]
        return !covered;
    }

    // Filtering is independent of assigned hint codes. Recheck already displayed
    // points too, so a newly hidden point is absent from the next activation.
    pub(super) fn visible_elements(&self, window: &WindowElement) -> Vec<UIElement> {
        let areas = if self.layout.uncovered_windows.contains(window) {
            &[][..]
        } else if let Some(areas) = self.layout.windows_covered_areas.get(window) {
            areas.as_slice()
        } else {
            return Vec::new();
        };
        self.ui_elements.get(window).into_iter().flat_map(|elements| elements.iter())
            .filter(|element| self.point_visible(window, element, areas)).cloned().collect()
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

    // Replay previously filtered coordinates immediately, even if visibility is stale.
    // The empty activation layout needs only current display bounds; no window
    // intersections or native visibility checks are needed on this path.
    pub(super) fn generate_cached_hints(
        &self,
        windows: &[WindowElement],
        position_set: &mut HashSet<(i32, i32)>,
        hints_count: &mut i32,
    ) -> HashMap<String, Vec<Hint>> {
        let mut hints = HashMap::new();
        for window in windows {
            if let Some(elements) = self.ui_elements.get(window) {
                self.do_generate_hints(&mut hints, window, elements, position_set, hints_count);
            }
        }
        hints
    }

    fn do_generate_hints(
        &self,
        monitor_hints: &mut HashMap<String, Vec<Hint>>,
        window_element: &WindowElement,
        ui_elements: &Vec<UIElement>,
        position_set: &mut HashSet<(i32, i32)>,
        hints_count: &mut i32,
    ) {
        for hint in ui_elements {
            for (index, (monitor, bounds)) in self.layout.monitors.iter()
                .zip(&self.layout.monitor_bounds).enumerate() {
                if !bounds.contains_point(hint.x, hint.y) {
                    continue;
                }

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
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static CODE_TEST_LOCK: Mutex<()> = Mutex::new(());

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
        let _guard = CODE_TEST_LOCK.lock().unwrap();
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
        let first = generator.generate_cached_hints(std::slice::from_ref(&window), &mut assigned, &mut count);
        let first = first.values().next().unwrap();
        assert_eq!((&first[0].text, first[0].x), (&"AA".to_owned(), 100));
        assert_eq!((&first[1].text, first[1].x), (&"AS".to_owned(), 200));
        // The first point disappeared, another moved, and the second is duplicated.
        generator.ui_elements.insert(window.clone(), Arc::new(vec![element(200), element(300), element(400)]));
        let appended = generator.generate_cached_hints(std::slice::from_ref(&window), &mut assigned, &mut count);
        let appended = appended.values().next().unwrap();
        assert_eq!(appended.len(), 1); // Capacity exhaustion never recycles AA/AS.
        assert_eq!((&appended[0].text, appended[0].x), (&"AD".to_owned(), 300));
        assert_eq!(first[0].x, 100);
        assert_eq!(count, 3);
    }

    #[test]
    fn previously_filtered_points_replay_without_rechecking_then_new_points_append() {
        let _guard = CODE_TEST_LOCK.lock().unwrap();
        *HINT_TEXT_LIST_STORAGE.lock().unwrap() = vec!["AA".into(), "AS".into(), "AD".into()];
        let front = WindowElement {
            window_handle: 1, owner_pid: 1, x: 0, y: 0, width: 800, height: 600,
            title: String::new(), class_name: String::new(), z_index: 0,
            visible: true, is_task_bar: false,
        };
        let behind = WindowElement { window_handle: 2, owner_pid: 2, ..front.clone() };
        let windows = vec![front.clone(), behind.clone()];
        let layout = Arc::new(HintLayout::with_monitors(&windows, vec![
            MonitorInfo { id: 0, x: 0, y: 0, width: 800, height: 600, scale_factor: 1.0 },
        ]));
        let element = |x| UIElement { text: String::new(), x, y: 100, z: 1, width: 10,
            height: 10, window_handle: 2, control_type: 0, element_type: 0 };
        let mut generator = HintsGenerator::new(layout, HashMap::from([
            (behind, Arc::new(vec![element(-100), element(100), element(100), element(200)])),
        ]));
        let mut assigned = HashSet::new();
        let mut count = 0;
        // Both points were visible earlier but are now behind the front window.
        // A new background check rejects them; replay of the old cache is allowed.
        assert!(generator.visible_elements(&windows[1]).is_empty());
        let initial = generator.generate_cached_hints(&windows, &mut assigned, &mut count);
        let initial = initial.values().next().unwrap();
        assert_eq!(initial.iter().map(|h| (h.text.as_str(), h.x)).collect::<Vec<_>>(),
            vec![("AA", 100), ("AS", 200)]);
        assert_eq!(assigned, HashSet::from([(100, 100), (200, 100)]));

        // A fresh foreground scan adds one point, retaining both cached codes.
        generator.ui_elements = HashMap::from([(front, Arc::new(vec![element(200), element(300)]))]);
        let appended = generator.generate_cached_hints(&windows, &mut assigned, &mut count);
        let appended = appended.values().next().unwrap();
        assert_eq!(appended.len(), 1);
        assert_eq!((appended[0].text.as_str(), appended[0].x), ("AD", 300));
        assert_eq!((initial[0].text.as_str(), initial[0].x), ("AA", 100));
        assert_eq!(count, 3);
    }
}
