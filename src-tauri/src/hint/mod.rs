mod codes;
mod generator;
pub mod hint;
#[cfg(target_os = "macos")]
mod occlusion_macos;
pub mod overlay;
mod warm;

use crate::{config, element, input};
use generator::{Hint, HintLayout, HintsGenerator};
use hint::{clear_hints, save_hints, update_hints_offset};
use once_cell::sync::Lazy;
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::Instant;
use tauri::{Emitter, Manager, WebviewWindow};

pub use generator::init_hint_text_list_storage;
pub use overlay::{create_overlay_windows, OVERLAY_WINDOW_PREFIX};
pub use warm::setup_visibility_cache;

// Odd epochs are active, even epochs are closed. Serialize storage + IPC with
// session transitions, so a late native result cannot repopulate a closed session.
static ACTIVATION: AtomicU64 = AtomicU64::new(0);
static LIFECYCLE: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
static FILTER_REVISION: AtomicU64 = AtomicU64::new(0);

pub fn current_activation() -> u64 {
    ACTIVATION.load(Ordering::Acquire)
}
pub fn is_active(activation: u64) -> bool {
    activation % 2 == 1 && current_activation() == activation
}

fn publish(
    app: &tauri::AppHandle,
    activation: u64,
    monitor_hints: HashMap<String, Vec<Hint>>,
) -> usize {
    let _guard = LIFECYCLE.lock().unwrap();
    if !is_active(activation) {
        return 0;
    }
    let mut count = 0;
    let offset = *hint::HINTS_OFFSET_STORAGE.lock().unwrap();
    for (label, mut hints) in monitor_hints {
        if hints.is_empty() {
            continue;
        }
        count += hints.len();
        for hint in &mut hints {
            hint.x -= offset.0;
            hint.y -= offset.1;
        }
        // Input can resolve a code as soon as the frontend displays it.
        save_hints(label.clone(), hints.clone());
        if let Some(window) = app.get_webview_window(&label) {
            if let Err(error) = window.emit(
                "append-hints",
                json!({
                    "session": activation, "windowLabel": label, "hints": hints,
                }),
            ) {
                log::error!("[hint] append failed: {error}");
            }
        }
    }
    count
}

// Preserve traversal progress when a newer progressive snapshot arrives.
// Restarting at element zero on every update can starve controls late in a tree.
struct Pending {
    snapshot: element::element::Snapshot,
    checked: HashSet<(i32, i32)>,
    remaining: VecDeque<element::UIElement>,
}

impl Pending {
    fn new(
        snapshot: element::element::Snapshot,
        previous: Option<Self>,
        layout: &HintLayout,
        assigned: &HashSet<(i32, i32)>,
    ) -> Self {
        // Finish pending work before retrying rejected points, even if a newer
        // walk publishes meanwhile. Once drained, reuse only within one walk.
        let checked = previous
            .filter(|p| !p.remaining.is_empty() || p.snapshot.scan_id == snapshot.scan_id)
            .map(|p| p.checked).unwrap_or_default();
        let remaining = snapshot
            .elements
            .iter()
            .filter(|e| layout.contains_point(e.x, e.y)
                && !assigned.contains(&(e.x, e.y))
                && !checked.contains(&(e.x, e.y)))
            .cloned()
            .collect();
        Self {
            snapshot,
            checked,
            remaining,
        }
    }

    fn batch(&mut self, assigned: &HashSet<(i32, i32)>) -> Vec<element::UIElement> {
        let mut batch = Vec::new();
        for _ in 0..128 {
            let Some(element) = self.remaining.pop_front() else {
                break;
            };
            self.checked.insert((element.x, element.y));
            if !assigned.contains(&(element.x, element.y)) {
                batch.push(element);
            }
        }
        batch
    }
}

#[derive(Default)]
struct WorkQueue {
    pending: VecDeque<Pending>,
    completed: HashMap<i64, Pending>,
}

impl WorkQueue {
    fn clear(&mut self) {
        self.pending.clear();
        self.completed.clear();
    }

    fn update(
        &mut self,
        snapshot: element::element::Snapshot,
        foreground: Option<i64>,
        layout: &HintLayout,
        assigned: &HashSet<(i32, i32)>,
    ) {
        let id = snapshot.window.window_handle;
        let previous = self.pending.iter()
            .position(|p| p.snapshot.window.window_handle == id)
            .and_then(|i| self.pending.remove(i))
            .or_else(|| self.completed.remove(&id));
        let work = Pending::new(snapshot, previous, layout, assigned);
        if work.remaining.is_empty() {
            self.completed.insert(id, work);
        } else if Some(id) == foreground {
            self.pending.push_front(work);
        } else {
            self.pending.push_back(work);
        }
    }

    fn next(&mut self, assigned: &HashSet<(i32, i32)>)
        -> Option<(element::element::Snapshot, Vec<element::UIElement>)>
    {
        let mut work = self.pending.pop_front()?;
        let elements = work.batch(assigned);
        let snapshot = work.snapshot.clone();
        if work.remaining.is_empty() {
            self.completed.insert(snapshot.window.window_handle, work);
        } else {
            self.pending.push_back(work);
        }
        Some((snapshot, elements))
    }
}

pub async fn show_hints(window: WebviewWindow) {
    if !crate::permissions::get_permission_status().ready {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let started = Instant::now();
    let app = window.app_handle();
    // Subscribe before requesting work: a fast scan cannot be missed.
    let mut updates = warm::subscribe();
    let mut seen = HashMap::new();
    // Read the last filtered result immediately. A new check never gates replay.
    let (cached_windows, cached_snapshots) = warm::snapshots_since(&mut seen);
    let activation = {
        let _guard = LIFECYCLE.lock().unwrap();
        // Drain/reset old input before changing its session tag. Otherwise an
        // old key handler could stamp a prefix or deferred exit with the new ID.
        input::keyboard::switch_keyboard_ctrl(false);
        let old = current_activation();
        let activation = if old % 2 == 1 { old + 2 } else { old + 1 };
        ACTIVATION.store(activation, Ordering::Release);
        warm::notify();
        clear_hints();
        // Reset every monitor, including those with no cached controls.
        for (label, overlay) in app.webview_windows() {
            if label.starts_with(OVERLAY_WINDOW_PREFIX) {
                let _ = overlay.emit(
                    "show-hints",
                    json!({ "session": activation, "windowLabel": label }),
                );
            }
        }
        input::keyboard::switch_keyboard_ctrl(true);
        element::set_hint_active(true);
        if !config::get_config().unwrap().system.debug_mode {
            overlay::ensure_all_overlays_topmost(app);
        }
        activation
    };
    let mut positions = HashSet::new();
    let mut count = 0;
    let layout = Arc::new(HintLayout::new(&[]));
    let grids = HintsGenerator::new(layout.clone(), HashMap::new()).generate_hints_grid(&mut count);
    publish(app, activation, grids);
    // Only previously filtered coordinates enter this path, never raw AX data.
    // Their visibility may be stale, but the session keeps them as shortcuts.
    let sources = cached_snapshots.len();
    let cached = HintsGenerator::new(layout.clone(), cached_snapshots.into_iter()
        .map(|snapshot| (snapshot.window, snapshot.elements)).collect());
    let hints = cached.generate_cached_hints(&cached_windows, &mut positions, &mut count);
    let added = publish(app, activation, hints);
    log::info!("[hint_cache] session={} added={} sources={} activation_ms={:.1}",
        activation, added, sources, started.elapsed().as_secs_f64() * 1000.0);
    let mut first_control = added == 0;
    drop(cached);
    while is_active(activation) {
        updates.borrow_and_update();
        let (windows, snapshots) = warm::snapshots_since(&mut seen);
        for snapshot in snapshots {
            if !generator::has_capacity(count) { break; }
            let generator = HintsGenerator::new(layout.clone(),
                HashMap::from([(snapshot.window.clone(), snapshot.elements.clone())]));
            let hints = generator.generate_cached_hints(&windows, &mut positions, &mut count);
            let added = publish(app, activation, hints);
            if added > 0 {
                log::info!("[hint_update] session={} window={} revision={} added={} total={} first_control={} activation_ms={:.1} snapshot_age_ms={:.1}",
                    activation, snapshot.window.window_handle, snapshot.revision, added, count, first_control,
                    started.elapsed().as_secs_f64() * 1000.0, snapshot.scanned_at.elapsed().as_secs_f64() * 1000.0);
                first_control = false;
            }
        }
        if !is_active(activation) || updates.changed().await.is_err() {
            break;
        }
    }
}

pub async fn hide_hints_for_session(app: tauri::AppHandle, activation: u64) {
    let _guard = LIFECYCLE.lock().unwrap();
    if !is_active(activation) {
        return;
    }
    let closed = activation + 1;
    ACTIVATION.store(closed, Ordering::Release);
    warm::notify();
    input::keyboard::switch_keyboard_ctrl(false);
    element::set_hint_active(false);
    clear_hints();
    if let Err(error) = app.emit("hide-hints", json!({ "session": closed })) {
        log::error!("[hint] hide failed: {error}");
    }
    #[cfg(target_os = "macos")]
    if !config::get_config().unwrap().system.debug_mode {
        overlay::sync_overlay_visibility(&app);
    }
}

pub async fn move_hints(app: tauri::AppHandle, activation: u64, direction: (i32, i32)) {
    let _guard = LIFECYCLE.lock().unwrap();
    if !is_active(activation) {
        return;
    }
    update_hints_offset(direction.0, direction.1);
    let _ = app.emit(
        "move-hints",
        json!({ "session": activation, "x": direction.0, "y": direction.1 }),
    );
}

// Called synchronously by the input state machine, preserving prefix order.
// No lifecycle lock here: input holds KEYBOARD_STATE and show/hide reset it.
pub fn filter_hints(app: &tauri::AppHandle, letters: String) {
    let activation = current_activation();
    if !is_active(activation) {
        return;
    }
    let revision = FILTER_REVISION.fetch_add(1, Ordering::Relaxed) + 1;
    let _ = app.emit(
        "filter-hints",
        json!({ "session": activation, "revision": revision, "letters": letters }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_layout() -> HintLayout {
        HintLayout::with_monitors(&[], vec![crate::monitor::MonitorInfo {
            id: 0, x: 0, y: 0, width: 1000, height: 800, scale_factor: 1.0,
        }])
    }

    fn snapshot(count: i32, revision: u64) -> element::element::Snapshot {
        let elements = (0..count)
            .map(|x| element::UIElement {
                text: String::new(),
                x,
                y: 10,
                z: 1,
                width: 1,
                height: 1,
                window_handle: 1,
                control_type: 0,
                element_type: 0,
            })
            .collect();
        element::element::Snapshot {
            window: crate::window::WindowElement {
                window_handle: 1,
                owner_pid: 1,
                x: 0,
                y: 0,
                width: 1000,
                height: 800,
                title: String::new(),
                class_name: String::new(),
                z_index: 0,
                visible: true,
                is_task_bar: false,
            },
            elements: Arc::new(elements),
            revision,
            scan_id: 1,
            scanned_at: Instant::now(),
        }
    }

    #[test]
    fn progressive_updates_cannot_restart_visibility_work_and_starve_later_controls() {
        let layout = test_layout();
        let assigned = HashSet::new(); // Even rejected/occluded points count as visited work.
        let mut work = Pending::new(snapshot(300, 1), None, &layout, &assigned);
        assert_eq!(work.batch(&assigned).last().unwrap().x, 127);
        work = Pending::new(snapshot(310, 2), Some(work), &layout, &assigned);
        assert_eq!(work.batch(&assigned).first().unwrap().x, 128);
        work = Pending::new(snapshot(320, 3), Some(work), &layout, &assigned);
        let batch = work.batch(&assigned);
        assert_eq!(batch.first().unwrap().x, 256);
        assert_eq!(batch.last().unwrap().x, 319);
        assert!(work.remaining.is_empty());
    }

    #[test]
    fn new_walk_cannot_starve_the_tail_of_unfinished_visibility_work() {
        let layout = test_layout();
        let assigned = HashSet::new();
        let mut queue = WorkQueue::default();
        for scan in 1..=3 {
            let mut source = snapshot(300, scan);
            source.scan_id = scan;
            queue.update(source, Some(1), &layout, &assigned);
            let batch = queue.next(&assigned).unwrap().1;
            assert_eq!(batch.first().unwrap().x, (scan as i32 - 1) * 128);
            if scan == 3 { assert_eq!(batch.last().unwrap().x, 299); }
        }
        assert!(queue.next(&assigned).is_none());
    }

    #[test]
    fn drained_queue_skips_checked_points_until_new_scan_or_layout() {
        let layout = test_layout();
        let mut queue = WorkQueue::default();
        let assigned = HashSet::new(); // Simulate points rejected by visibility.
        queue.update(snapshot(100, 1), Some(1), &layout, &assigned);
        assert_eq!(queue.next(&assigned).unwrap().1.len(), 100);
        assert!(queue.next(&assigned).is_none());
        queue.update(snapshot(110, 2), Some(1), &layout, &assigned);
        let batch = queue.next(&assigned).unwrap().1;
        assert_eq!(batch.iter().map(|e| e.x).collect::<Vec<_>>(), (100..110).collect::<Vec<_>>());

        let mut refreshed = snapshot(110, 3);
        refreshed.scan_id = 2;
        queue.update(refreshed.clone(), Some(1), &layout, &assigned);
        assert_eq!(queue.next(&assigned).unwrap().1.len(), 110);
        // A changed layout can expose an old point even in the same native walk.
        queue.clear();
        queue.update(refreshed, Some(1), &layout, &assigned);
        assert_eq!(queue.next(&assigned).unwrap().1.len(), 110);
    }

    #[test]
    fn offscreen_and_restored_points_do_not_delay_new_visible_points() {
        let layout = test_layout();
        let mut source = snapshot(6000, 1);
        let elements = Arc::make_mut(&mut source.elements);
        // Put 5,999 offscreen entries before one visible target.
        for element in elements.iter_mut() { element.y = -9000; }
        elements.last_mut().unwrap().x = 50;
        elements.last_mut().unwrap().y = 50;
        let mut queue = WorkQueue::default();
        queue.update(source.clone(), Some(1), &layout, &HashSet::new());
        assert_eq!(queue.next(&HashSet::new()).unwrap().1.len(), 1);
        assert!(queue.next(&HashSet::new()).is_none());

        queue.clear();
        let assigned = HashSet::from([(50, 50)]);
        queue.update(source, Some(1), &layout, &assigned);
        assert!(queue.next(&assigned).is_none());
    }
}
