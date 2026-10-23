//! Reuse recently displayed controls after validating the current window layout.
//! This stores accepted coordinates, not another native-element scan/cache.
use crate::{
    element::element::{same_window, Snapshot},
    window::WindowElement,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Default)]
pub(super) struct WarmControls {
    activation: u64,
    windows: Vec<WindowElement>,
    points: HashMap<i64, HashMap<(i32, i32), Instant>>,
}

impl WarmControls {
    fn align(&mut self, activation: u64, windows: &[WindowElement]) -> bool {
        // A cancelled activation may finish its native query after a new one.
        if activation < self.activation {
            return false;
        }
        self.activation = activation;
        if self.windows.len() != windows.len()
            || self.windows.iter().zip(windows).any(|(a, b)| {
                !same_window(a, b)
                    || a.z_index != b.z_index
                    || a.visible != b.visible
                    || a.is_task_bar != b.is_task_bar
            })
        {
            self.points.clear();
            self.windows = windows.to_vec();
        }
        true
    }

    pub(super) fn restore(
        &mut self,
        activation: u64,
        windows: &[WindowElement],
        snapshots: &[Snapshot],
        now: Instant,
        ttl: Duration,
    ) -> HashMap<WindowElement, Arc<Vec<crate::element::UIElement>>> {
        if !self.align(activation, windows) {
            return HashMap::new();
        }
        self.points.retain(|_, points| {
            points.retain(|_, checked| now.saturating_duration_since(*checked) <= ttl);
            !points.is_empty()
        });
        // Only points still present in a fresh source snapshot may be replayed.
        // A restored point does not extend its visibility-check lifetime.
        snapshots
            .iter()
            .filter_map(|snapshot| {
                let points = self.points.get(&snapshot.window.window_handle)?;
                let elements: Vec<_> = snapshot
                    .elements
                    .iter()
                    .filter(|e| points.contains_key(&(e.x, e.y)))
                    .cloned()
                    .collect();
                (!elements.is_empty()).then(|| (snapshot.window.clone(), Arc::new(elements)))
            })
            .collect()
    }

    pub(super) fn record(
        &mut self,
        activation: u64,
        windows: &[WindowElement],
        source: i64,
        points: impl Iterator<Item = (i32, i32)>,
        now: Instant,
    ) {
        if self.align(activation, windows) {
            self.points
                .entry(source)
                .or_default()
                .extend(points.map(|p| (p, now)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(id: i64, xs: &[i32]) -> Snapshot {
        Snapshot {
            window: WindowElement {
                window_handle: id,
                owner_pid: id as i32,
                x: 0,
                y: 0,
                width: 800,
                height: 600,
                title: String::new(),
                class_name: String::new(),
                z_index: 0,
                visible: true,
                is_task_bar: false,
            },
            elements: Arc::new(
                xs.iter()
                    .map(|&x| crate::element::UIElement {
                        text: String::new(),
                        x,
                        y: 10,
                        z: 1,
                        width: 5,
                        height: 5,
                        window_handle: id,
                        control_type: 0,
                        element_type: 0,
                    })
                    .collect(),
            ),
            revision: 1,
            scan_id: 1,
            scanned_at: Instant::now(),
        }
    }

    #[test]
    fn reopening_restores_only_verified_points_still_in_the_latest_source() {
        let now = Instant::now();
        let source = snapshot(1, &[10, 30]); // Old point 20 was removed; 30 is new.
        let windows = vec![source.window.clone()];
        let mut warm = WarmControls::default();
        warm.record(1, &windows, 1, [(10, 10), (20, 10)].into_iter(), now);
        let restored = warm.restore(3, &windows, &[source.clone()], now, Duration::from_secs(60));
        assert_eq!(
            restored[&source.window]
                .iter()
                .map(|e| e.x)
                .collect::<Vec<_>>(),
            vec![10]
        );
        // Old native work cannot repopulate or reset the newer activation.
        warm.record(1, &[], 1, [(30, 10)].into_iter(), now);
        assert_eq!(
            warm.restore(5, &windows, &[source], now, Duration::from_secs(60))[&windows[0]].len(),
            1
        );
    }

    #[test]
    fn replay_does_not_keep_visibility_fresh_forever() {
        let now = Instant::now();
        let source = snapshot(1, &[10]);
        let windows = vec![source.window.clone()];
        let mut warm = WarmControls::default();
        warm.record(1, &windows, 1, [(10, 10)].into_iter(), now);
        assert_eq!(
            warm.restore(
                3,
                &windows,
                &[source.clone()],
                now + Duration::from_secs(59),
                Duration::from_secs(60)
            )
            .len(),
            1
        );
        assert!(warm
            .restore(
                5,
                &windows,
                &[source],
                now + Duration::from_secs(61),
                Duration::from_secs(60)
            )
            .is_empty());
    }

    #[test]
    fn geometry_order_or_owner_changes_require_new_visibility_checks() {
        let now = Instant::now();
        let sources = vec![snapshot(1, &[10]), snapshot(2, &[20])];
        let windows: Vec<_> = sources.iter().map(|s| s.window.clone()).collect();
        for changed in 0..4 {
            let mut warm = WarmControls::default();
            warm.record(1, &windows, 1, [(10, 10)].into_iter(), now);
            let mut current = windows.clone();
            match changed {
                0 => current.reverse(),
                1 => current[1].x += 20,
                2 => current[1].z_index += 1,
                _ => current[0].owner_pid += 1,
            }
            assert!(warm
                .restore(3, &current, &sources, now, Duration::from_secs(60))
                .is_empty());
        }
    }
}
