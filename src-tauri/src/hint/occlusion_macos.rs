//! Per-activation macOS visibility checks. Windows retains its rectangle policy.
use crate::{input::keyboard::KEYBOARD_STATE, macos_window};
use std::collections::{hash_map::Entry, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static ACTIVATION: AtomicU64 = AtomicU64::new(0);

pub fn begin_activation() -> u64 {
    ACTIVATION.fetch_add(1, Ordering::Relaxed) + 1
}

pub fn cancel_activation() {
    ACTIVATION.fetch_add(1, Ordering::Relaxed);
}

pub fn is_active(activation: u64) -> bool {
    ACTIVATION.load(Ordering::Relaxed) == activation
        && KEYBOARD_STATE
            .lock()
            .map(|state| state.in_ctrl_session)
            .unwrap_or(false)
}

#[derive(Default)]
pub struct PointVisibility {
    // Never shared between activations: window geometry and click-through state
    // may change even when the UI element cache still contains the same point.
    hits: HashMap<(i32, i32), Option<i64>>,
    points: Vec<(i32, i32)>,
    candidates: usize,
    offscreen: usize,
    fast_passes: usize,
    suspects: usize,
    unique_points: usize,
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
        } else if suspect {
            self.suspects += 1;
            if let Entry::Vacant(entry) = self.hits.entry(point) {
                entry.insert(None);
                self.points.push(point);
                self.unique_points += 1;
            }
        } else {
            self.fast_passes += 1;
        }
    }

    pub async fn prepare(&mut self, app: &tauri::AppHandle, activation: u64) -> bool {
        let started = Instant::now();
        let primary_height = core_graphics::display::CGDisplay::main()
            .bounds()
            .size
            .height;
        let mut offset = 0;
        while offset < self.points.len() {
            if !is_active(activation) {
                return false;
            }
            // One short main-thread task at a time; never queue the entire scan.
            let points = self.points[offset..(offset + 64).min(self.points.len())].to_vec();
            let (tx, rx) = tokio::sync::oneshot::channel();
            if let Err(error) = app.run_on_main_thread(move || {
                let main_started = Instant::now();
                if !tx.is_closed() && is_active(activation) {
                    let mut result =
                        unsafe { macos_window::hit_test_points(&points, primary_height) };
                    result.main_elapsed = main_started.elapsed();
                    let _ = tx.send(result);
                }
            }) {
                log::warn!("[hint_perf macos] main-thread dispatch failed: {error}; using rectangle fallback");
                break;
            }
            let Ok(batch) = rx.await else {
                break;
            };
            offset += batch.hits.len();
            self.queries += batch.hits.len();
            self.failures += batch.hits.iter().filter(|(_, hit)| hit.is_none()).count();
            self.hits.extend(batch.hits);
            self.batches += 1;
            self.query_elapsed += batch.query_elapsed;
            self.main_total += batch.main_elapsed;
            self.main_max = self.main_max.max(batch.main_elapsed);
        }
        self.hit_test_elapsed = started.elapsed();
        // Drop planning allocations; keep only the point result cache.
        self.points = Vec::new();
        is_active(activation)
    }

    pub fn visible(&self, window: i64, point: (i32, i32), suspect: bool) -> bool {
        match self.hits.get(&point).copied().flatten() {
            Some(receiver) => receiver == window,
            None => !suspect, // Failed/no query: retain the conservative rectangle rule.
        }
    }

    pub fn log_performance(&self, activation: u64, hints: i32, grid: i32, total: Duration) {
        log::info!(
            "[hint_perf macos] activation={} candidates={} offscreen={} rect_fast={} rect_suspect={} native_queries={} cache_hits={} failures={} unresolved={} hints={} grid_hints={} control_hints={} total_ms={:.3} hit_test_ms={:.3} query_ms={:.3} main_total_ms={:.3} main_max_ms={:.3} batches={}",
            activation, self.candidates, self.offscreen, self.fast_passes, self.suspects,
            self.queries, self.suspects.saturating_sub(self.unique_points), self.failures,
            self.unique_points.saturating_sub(self.queries) + self.failures, hints, grid, hints - grid,
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
    fn only_suspect_onscreen_points_are_queried_once() {
        let mut visibility = PointVisibility::default();
        visibility.add_candidate((-100, -200), true, true);
        visibility.add_candidate((-100, -200), true, true);
        visibility.add_candidate((500, 500), true, false);
        visibility.add_candidate((9000, 9000), false, true);
        assert_eq!(visibility.points, vec![(-100, -200)]);
        assert_eq!(visibility.suspects, 2);
        assert_eq!(visibility.fast_passes, 1);
        assert_eq!(visibility.offscreen, 1);
    }

    #[test]
    fn transparent_host_does_not_hide_target_or_claim_its_duplicate_point() {
        let mut visibility = PointVisibility::default();
        visibility.hits.insert((1665, -214), Some(62));
        assert!(visibility.visible(62, (1665, -214), true));
        assert!(!visibility.visible(1032, (1665, -214), false));
        assert!(!visibility.visible(99, (1665, -214), true));
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
