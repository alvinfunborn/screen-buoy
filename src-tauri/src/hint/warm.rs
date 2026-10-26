//! Continuously maintain filtered coordinates for the next hint activation.
//! Displayed hints are session-owned; removals here never retract those hints.
use super::{generator::{HintLayout, HintsGenerator}, WorkQueue};
use crate::{element::{self, element::Snapshot, UIElement}, window::WindowElement};
use once_cell::sync::Lazy;
use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex},
    time::{Duration, Instant},
};

struct CheckedSource {
    raw_revision: u64,
    points: HashMap<(i32, i32), UIElement>,
    snapshot: Snapshot,
}

#[derive(Default)]
struct VisibleCache {
    windows: Vec<WindowElement>,
    sources: HashMap<i64, CheckedSource>,
    revision: u64,
}

impl VisibleCache {
    fn align(&mut self, windows: &[WindowElement]) -> bool {
        let before = self.sources.len();
        self.sources.retain(|id, source| windows.iter().any(|window|
            window.window_handle == *id && window.owner_pid == source.snapshot.window.owner_pid));
        self.windows = windows.to_vec();
        self.sources.len() != before
    }

    fn sync_source(&mut self, source: &Snapshot) -> bool {
        let entry = self.sources.entry(source.window.window_handle).or_insert_with(|| CheckedSource {
            raw_revision: source.revision,
            points: HashMap::new(),
            snapshot: Snapshot { elements: Arc::new(Vec::new()), revision: 0, ..source.clone() },
        });
        if entry.raw_revision == source.revision {
            return false;
        }
        entry.raw_revision = source.revision;
        let present: HashSet<_> = source.elements.iter().map(|e| (e.x, e.y)).collect();
        let before = entry.points.len();
        entry.points.retain(|point, _| present.contains(point));
        if entry.points.len() == before {
            return false;
        }
        Self::publish(entry, source, &mut self.revision);
        true
    }

    fn apply(&mut self, source: &Snapshot, checked: &[UIElement], visible: Vec<UIElement>) -> bool {
        let Some(entry) = self.sources.get_mut(&source.window.window_handle) else { return false; };
        if entry.raw_revision != source.revision {
            return false;
        }
        let accepted: HashMap<_, _> = visible.into_iter().map(|e| ((e.x, e.y), e)).collect();
        let mut changed = false;
        for element in checked {
            let point = (element.x, element.y);
            if let Some(element) = accepted.get(&point) {
                if entry.points.get(&point) != Some(element) {
                    entry.points.insert(point, element.clone());
                    changed = true;
                }
            } else {
                changed |= entry.points.remove(&point).is_some();
            }
        }
        if changed {
            Self::publish(entry, source, &mut self.revision);
        }
        changed
    }

    fn publish(entry: &mut CheckedSource, source: &Snapshot, revision: &mut u64) {
        *revision += 1;
        let mut elements: Vec<_> = entry.points.values().cloned().collect();
        elements.sort_by_key(|e| (e.y, e.x));
        entry.snapshot = Snapshot {
            elements: Arc::new(elements), revision: *revision, scanned_at: Instant::now(),
            ..source.clone()
        };
    }

    fn snapshots_since(&self, seen: &mut HashMap<i64, u64>) -> (Vec<WindowElement>, Vec<Snapshot>) {
        let snapshots = self.windows.iter().filter_map(|window| {
            let source = self.sources.get(&window.window_handle)?;
            if seen.get(&window.window_handle) == Some(&source.snapshot.revision) {
                return None;
            }
            seen.insert(window.window_handle, source.snapshot.revision);
            Some(Snapshot { window: window.clone(), ..source.snapshot.clone() })
        }).collect();
        (self.windows.clone(), snapshots)
    }
}

static CACHE: Lazy<Mutex<VisibleCache>> = Lazy::new(|| Mutex::new(VisibleCache::default()));
static CHANGED: Lazy<tokio::sync::watch::Sender<u64>> = Lazy::new(|| tokio::sync::watch::channel(0).0);
static STARTED: AtomicBool = AtomicBool::new(false);

pub(super) fn notify() {
    CHANGED.send_modify(|revision| *revision = revision.wrapping_add(1));
}


pub(super) fn subscribe() -> tokio::sync::watch::Receiver<u64> {
    CHANGED.subscribe()
}

pub(super) fn snapshots_since(seen: &mut HashMap<i64, u64>) -> (Vec<WindowElement>, Vec<Snapshot>) {
    CACHE.lock().unwrap().snapshots_since(seen)
}

pub fn setup_visibility_cache(app: tauri::AppHandle) {
    if STARTED.swap(true, Ordering::AcqRel) { return; }
    tauri::async_runtime::spawn(async move {
        let mut updates = element::subscribe();
        let mut seen = HashMap::new();
        let mut windows = Vec::new();
        let mut layout = Arc::new(HintLayout::new(&[]));
        let mut queue = WorkQueue::default();
        let mut activation = 0;
        // Visibility must never skip a point just because the current UI uses it.
        let excluded = HashSet::new();
        loop {
            updates.borrow_and_update();
            let current = super::current_activation();
            if current != activation && current % 2 == 1 {
                seen.clear();
                queue.clear();
            }
            activation = current;
            let Some((next_windows, sources)) = element::snapshots_since(&mut seen, 0) else { continue; };
            let changed_layout = windows.len() != next_windows.len() || windows.iter().zip(&next_windows)
                .any(|(a, b)| !element::element::same_window(a, b) || a.z_index != b.z_index);
            if changed_layout {
                queue.clear();
                layout = Arc::new(HintLayout::new(&next_windows));
                windows = next_windows;
            }
            let foreground = element::element::foreground(&windows);
            let mut changed = false;
            {
                let mut cache = CACHE.lock().unwrap();
                changed |= cache.align(&windows);
                for source in &sources { changed |= cache.sync_source(source); }
            }
            if changed { notify(); }
            for source in sources { queue.update(source, foreground, &layout, &excluded); }
            let Some((source, elements)) = queue.next(&excluded) else {
                if updates.changed().await.is_err() { break; }
                continue;
            };
            if elements.is_empty() { continue; }
            let started = Instant::now();
            let elements = Arc::new(elements);
            let generator = HintsGenerator::new(layout.clone(),
                HashMap::from([(source.window.clone(), elements.clone())]));
            #[cfg(target_os = "macos")]
            let generator = generator.prepare_visibility(&app).await;
            let visible = generator.visible_elements(&source.window);
            let accepted = visible.len();
            let changed = CACHE.lock().unwrap().apply(&source, &elements, visible);
            if changed { notify(); }
            log::debug!("[visibility_cache] epoch={} background={} window={} checked={} accepted={} changed={}",
                current, current % 2 == 0, source.window.window_handle, elements.len(), accepted, changed);
            #[cfg(target_os = "macos")]
            generator.log_performance(current, accepted, started.elapsed());
            // One worker and one native task at a time. Closed hints still warm
            // the cache, with breathing room between batches for the desktop.
            tokio::time::sleep(Duration::from_millis(if super::current_activation() % 2 == 1 { 1 } else { 20 })).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(revision: u64, xs: &[i32]) -> Snapshot {
        Snapshot {
            window: WindowElement {
                window_handle: 1, owner_pid: 1, x: 0, y: 0, width: 800, height: 600,
                title: String::new(), class_name: String::new(), z_index: 0,
                visible: true, is_task_bar: false,
            },
            elements: Arc::new(xs.iter().map(|&x| UIElement {
                text: String::new(), x, y: 10, z: 1, width: 5, height: 5,
                window_handle: 1, control_type: 0, element_type: 0,
            }).collect()),
            revision, scan_id: revision, scanned_at: Instant::now(),
        }
    }

    fn xs(snapshot: &Snapshot) -> Vec<i32> {
        snapshot.elements.iter().map(|element| element.x).collect()
    }

    #[test]
    fn raw_sources_never_leak_before_background_visibility_finishes() {
        let source = source(1, &[10, 20]);
        let mut cache = VisibleCache::default();
        cache.align(&[source.window.clone()]);
        cache.sync_source(&source);
        let mut seen = HashMap::new();
        assert!(cache.snapshots_since(&mut seen).1[0].elements.is_empty());
        // Only 20 passed the first background check. Even the first completed
        // publication must have a distinct revision from the empty placeholder.
        assert!(cache.apply(&source, &source.elements, vec![source.elements[1].clone()]));
        assert_eq!(xs(&cache.snapshots_since(&mut seen).1[0]), vec![20]);
    }

    #[test]
    fn a_newly_hidden_displayed_point_is_removed_only_from_the_next_cache() {
        let source = source(10, &[10, 20, 30]);
        let mut cache = VisibleCache::default();
        cache.align(&[source.window.clone()]);
        cache.sync_source(&source);
        cache.apply(&source, &source.elements[..2], source.elements[..2].to_vec());
        let displayed = cache.snapshots_since(&mut HashMap::new()).1.remove(0);
        assert_eq!(xs(&displayed), vec![10, 20]);

        // Recheck all points, including 10 which has already been displayed.
        // It is now covered, while newly discovered 30 is visible.
        cache.apply(&source, &source.elements, source.elements[1..].to_vec());
        let next = cache.snapshots_since(&mut HashMap::new()).1.remove(0);
        assert_eq!(xs(&next), vec![20, 30]);
        assert_eq!(xs(&displayed), vec![10, 20]);
        assert!(!Arc::ptr_eq(&displayed.elements, &next.elements));
    }

    #[test]
    fn new_source_prunes_removed_points_and_rejects_late_old_results() {
        let old = source(1, &[10, 20]);
        let mut cache = VisibleCache::default();
        cache.align(&[old.window.clone()]);
        cache.sync_source(&old);
        cache.apply(&old, &old.elements, old.elements.as_ref().clone());
        let new = source(2, &[20, 30]);
        assert!(cache.sync_source(&new));
        assert_eq!(xs(&cache.snapshots_since(&mut HashMap::new()).1[0]), vec![20]);
        assert!(!cache.apply(&old, &old.elements, old.elements.as_ref().clone()));
        assert_eq!(xs(&cache.snapshots_since(&mut HashMap::new()).1[0]), vec![20]);
        cache.apply(&new, &new.elements, new.elements.as_ref().clone());
        assert_eq!(xs(&cache.snapshots_since(&mut HashMap::new()).1[0]), vec![20, 30]);
    }

    #[test]
    fn topology_changes_keep_filtered_points_until_rechecked_but_closed_windows_drop() {
        let source = source(1, &[10]);
        let mut cache = VisibleCache::default();
        cache.align(&[source.window.clone()]);
        cache.sync_source(&source);
        cache.apply(&source, &source.elements, source.elements.as_ref().clone());
        let floating = WindowElement { window_handle: 2, owner_pid: 2, ..source.window.clone() };
        cache.align(&[floating, source.window.clone()]);
        assert_eq!(xs(&cache.snapshots_since(&mut HashMap::new()).1[0]), vec![10]);
        // Once checked, the covered point disappears from the reusable cache.
        cache.apply(&source, &source.elements, Vec::new());
        assert!(cache.snapshots_since(&mut HashMap::new()).1[0].elements.is_empty());
        cache.align(&[]);
        assert!(!cache.apply(&source, &source.elements, source.elements.as_ref().clone()));
        assert!(cache.snapshots_since(&mut HashMap::new()).1.is_empty());
    }
}
