//! Background macOS visibility checks. Windows retains its rectangle policy.
use crate::macos_window;
use std::collections::{hash_map::Entry, HashMap};
use std::time::{Duration, Instant};

// Share exact-coordinate results across batches/windows for one short interval.
// A topology change clears these immediately; failures are always retried.
#[derive(Default)]
pub(super) struct HitCache(HashMap<(i32, i32), (i64, Instant)>);

impl HitCache {
    pub fn clear(&mut self) { self.0.clear(); }
    fn prune(&mut self, now: Instant) {
        self.0.retain(|_, (_, at)| now.saturating_duration_since(*at) < Duration::from_millis(250));
    }
}

#[derive(Default)]
pub struct PointVisibility {
    // This batch owns its results; HitCache permits bounded cross-batch reuse.
    hits: HashMap<(i32, i32), Option<i64>>,
    points: Vec<(i32, i32)>,
    candidates: usize,
    offscreen: usize,
    uncovered: usize,
    suspects: usize,
    queries: usize,
    failures: usize,
    batches: usize,
    query_elapsed: Duration,
    main_total: Duration,
    main_max: Duration,
    hit_test_elapsed: Duration,
}

impl PointVisibility {
    pub fn add_candidate(&mut self, point: (i32, i32), onscreen: bool, suspect: bool) {
        self.candidates += 1;
        if !onscreen {
            self.offscreen += 1;
        } else {
            if suspect { self.suspects += 1; } else { self.uncovered += 1; }
            if let Entry::Vacant(entry) = self.hits.entry(point) {
                entry.insert(None);
                self.points.push(point);
            }
        }
    }

    pub(super) async fn prepare(&mut self, app: &tauri::AppHandle, cache: &mut HitCache) {
        cache.prune(Instant::now());
        self.points.retain(|point| {
            if let Some(&(receiver, _)) = cache.0.get(point) {
                self.hits.insert(*point, Some(receiver));
                false
            } else { true }
        });
        let started = Instant::now();
        let primary_height = core_graphics::display::CGDisplay::main()
            .bounds()
            .size
            .height;
        let mut offset = 0;
        while offset < self.points.len() {
            // One short main-thread task at a time; never queue the entire scan.
            let points = self.points[offset..(offset + 64).min(self.points.len())].to_vec();
            let (tx, rx) = tokio::sync::oneshot::channel();
            if let Err(error) = app.run_on_main_thread(move || {
                let main_started = Instant::now();
                if !tx.is_closed() {
                    let mut result =
                        unsafe { macos_window::hit_test_points(&points, primary_height) };
                    result.main_elapsed = main_started.elapsed();
                    let _ = tx.send(result);
                }
            }) {
                log::warn!("[visibility_perf macos] main-thread dispatch failed: {error}; using rectangle fallback");
                break;
            }
            let Ok(batch) = rx.await else {
                break;
            };
            offset += batch.hits.len();
            self.queries += batch.hits.len();
            self.failures += batch.hits.iter().filter(|(_, hit)| hit.is_none()).count();
            let checked_at = Instant::now();
            for (point, receiver) in &batch.hits {
                if let Some(receiver) = receiver { cache.0.insert(*point, (*receiver, checked_at)); }
            }
            self.hits.extend(batch.hits);
            self.batches += 1;
            self.query_elapsed += batch.query_elapsed;
            self.main_total += batch.main_elapsed;
            self.main_max = self.main_max.max(batch.main_elapsed);
        }
        self.hit_test_elapsed = started.elapsed();
        // Drop planning allocations; keep only the point result cache.
        self.points = Vec::new();
    }

    pub fn visible(&self, window: i64, point: (i32, i32), suspect: bool) -> bool {
        match self.hits.get(&point).copied().flatten() {
            Some(receiver) => receiver == window,
            None => !suspect, // Failed/no query: retain the conservative rectangle rule.
        }
    }

    pub fn log_performance(&self, epoch: u64, window: i64, visible: usize, total: Duration) {
        log::info!(
            "[visibility_perf macos] epoch={} window={} background={} candidates={} offscreen={} rect_uncovered={} rect_suspect={} native_queries={} cache_hits={} failures={} unresolved={} visible={} elapsed_ms={:.3} hit_test_ms={:.3} query_ms={:.3} main_total_ms={:.3} main_max_ms={:.3} batches={}",
            epoch, window, epoch % 2 == 0, self.candidates, self.offscreen, self.uncovered, self.suspects,
            self.queries, self.candidates.saturating_sub(self.offscreen + self.queries), self.failures,
            self.hits.values().filter(|hit| hit.is_none()).count(), visible,
            total.as_secs_f64() * 1000.0, self.hit_test_elapsed.as_secs_f64() * 1000.0,
            self.query_elapsed.as_secs_f64() * 1000.0, self.main_total.as_secs_f64() * 1000.0,
            self.main_max.as_secs_f64() * 1000.0, self.batches,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_hits_expire_and_topology_invalidates_them() {
        let now = Instant::now();
        let mut cache = HitCache::default();
        cache.0.insert((1, 2), (62, now));
        cache.prune(now + Duration::from_millis(249));
        assert!(cache.0.contains_key(&(1, 2)));
        cache.prune(now + Duration::from_millis(250));
        assert!(cache.0.is_empty());
        cache.0.insert((1, 2), (62, now));
        cache.clear();
        assert!(cache.0.is_empty());
    }

    #[test]
    fn all_onscreen_points_are_queried_once() {
        let mut visibility = PointVisibility::default();
        visibility.add_candidate((-100, -200), true, true);
        visibility.add_candidate((-100, -200), true, true);
        visibility.add_candidate((500, 500), true, false);
        visibility.add_candidate((9000, 9000), false, true);
        assert_eq!(visibility.points, vec![(-100, -200), (500, 500)]);
        assert_eq!(visibility.suspects, 2);
        assert_eq!(visibility.uncovered, 1);
        assert_eq!(visibility.offscreen, 1);
    }

    #[test]
    fn transparent_host_does_not_hide_target_or_claim_its_duplicate_point() {
        let mut visibility = PointVisibility::default();
        visibility.hits.insert((1665, -214), Some(62));
        assert!(visibility.visible(62, (1665, -214), true));
        assert!(!visibility.visible(1032, (1665, -214), false));
        assert!(!visibility.visible(99, (1665, -214), true));
        visibility.hits.insert((500, 500), Some(0));
        assert!(!visibility.visible(1032, (500, 500), false));
    }

    #[test]
    fn failed_queries_fall_back_and_new_activation_does_not_reuse_hits() {
        let mut visibility = PointVisibility::default();
        visibility.hits.insert((10, 20), None);
        assert!(!visibility.visible(62, (10, 20), true));
        assert!(visibility.visible(62, (10, 20), false));
        visibility.hits.insert((10, 20), Some(62));
        assert!(!PointVisibility::default().visible(62, (10, 20), true));
    }
}
