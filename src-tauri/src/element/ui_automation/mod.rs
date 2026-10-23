#[cfg(target_os = "windows")]
mod ui_automation_win;
#[cfg(target_os = "windows")]
pub use ui_automation_win::*;

#[cfg(target_os = "macos")]
mod ui_automation_macos;
#[cfg(target_os = "macos")]
pub use ui_automation_macos::*;

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

/// The platform owns native objects; the collector owns cancellation and publication.
/// Intermediate snapshots are coalesced to at most one per 50 ms per worker.
pub struct ScanControl<'a> {
    cancelled: Arc<AtomicBool>,
    publish: Box<dyn FnMut(Vec<UIElement>, bool) + 'a>,
    last_publish: Instant,
    last_count: usize,
}

impl<'a> ScanControl<'a> {
    pub fn new(cancelled: Arc<AtomicBool>, publish: impl FnMut(Vec<UIElement>, bool) + 'a) -> Self {
        Self {
            cancelled,
            publish: Box::new(publish),
            last_publish: Instant::now(),
            last_count: 0,
        }
    }

    pub fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub fn checkpoint(&mut self, elements: &HashMap<(i32, i32), UIElement>) {
        if !self.cancelled()
            && elements.len() != self.last_count
            && self.last_publish.elapsed() >= Duration::from_millis(50)
        {
            self.publish(elements.values().cloned().collect(), false);
        }
    }

    pub fn finish(&mut self, elements: Vec<UIElement>) {
        self.publish(elements, true);
    }

    fn publish(&mut self, mut elements: Vec<UIElement>, complete: bool) {
        if self.cancelled() {
            return;
        }
        // Deterministic coordinate order within each published batch.
        elements.sort_by_key(|e| (e.y, e.x));
        self.last_count = elements.len();
        self.last_publish = Instant::now();
        (self.publish)(elements, complete);
    }
}
