//! One cache and two persistent scan lanes. Native calls never run on the scheduler.
use super::ui_automation::{ScanControl, UIAutomationRequest, UIElement};
use crate::{config, window::WindowElement};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc, Arc, Mutex,
};
use std::time::{Duration, Instant};

const TOPOLOGY_INTERVAL: Duration = Duration::from_millis(250);
const INTERACTION_SETTLE: Duration = Duration::from_millis(250);

#[derive(Clone)]
pub struct Snapshot {
    pub window: WindowElement,
    pub elements: Arc<Vec<UIElement>>,
    pub revision: u64,
    // Progressive publications of one native walk share this ID.
    pub scan_id: u64,
    pub scanned_at: Instant,
}

struct Entry {
    snapshot: Option<Snapshot>,
    next_scan: Instant,
    job: u64,
    completed_at: Option<Instant>,
    urgent: bool,
    follow_up: Option<Instant>,
    last_started: Option<Instant>,
    failures: u32,
}

#[derive(Default)]
struct Cache {
    windows: Vec<WindowElement>,
    entries: HashMap<i64, Entry>,
    revision: u64,
    topology_request: u64,
}

pub(crate) fn same_window(a: &WindowElement, b: &WindowElement) -> bool {
    a.window_handle == b.window_handle
        && a.owner_pid == b.owner_pid
        && (a.x, a.y, a.width, a.height) == (b.x, b.y, b.width, b.height)
        && a.title == b.title
        && a.class_name == b.class_name
}

impl Cache {
    fn reconcile(&mut self, windows: Vec<WindowElement>, now: Instant) -> bool {
        let old_front = foreground(&self.windows);
        let changed = self.windows.len() != windows.len()
            || self
                .windows
                .iter()
                .zip(&windows)
                .any(|(a, b)| !same_window(a, b) || a.z_index != b.z_index);
        self.entries
            .retain(|id, _| windows.iter().any(|w| w.window_handle == *id));
        for window in &windows {
            let old = self
                .windows
                .iter()
                .find(|w| w.window_handle == window.window_handle);
            if old.map_or(true, |old| !same_window(old, window)) {
                self.entries.insert(
                    window.window_handle,
                    Entry {
                        snapshot: None,
                        next_scan: now,
                        job: 0,
                        completed_at: None,
                        urgent: true,
                        follow_up: None,
                        last_started: None,
                        failures: 0,
                    },
                );
            } else if changed {
                // Recheck occlusion using warm points, without traversing every AX tree.
                if let Some(snapshot) = &mut self
                    .entries
                    .get_mut(&window.window_handle)
                    .unwrap()
                    .snapshot
                {
                    self.revision += 1;
                    snapshot.revision = self.revision;
                }
            }
        }
        self.windows = windows;
        if old_front.is_some() && old_front != foreground(&self.windows) {
            if let Some(id) = foreground(&self.windows) {
                self.refresh_target(id, &[], false, true, now);
            }
        }
        changed
    }

    fn cancel_obsolete(&self, slots: &[Option<Job>; 2]) {
        let front = foreground(&self.windows);
        for (lane, job) in slots
            .iter()
            .enumerate()
            .filter_map(|(lane, job)| job.as_ref().map(|j| (lane, j)))
        {
            if !self.windows.iter().any(|w| same_window(w, &job.window))
                || (lane == 0) != (Some(job.window.window_handle) == front)
            {
                job.cancelled.store(true, Ordering::Release);
            }
        }
    }

    fn refresh_target(
        &mut self,
        id: i64,
        slots: &[Option<Job>],
        cancel: bool,
        interaction: bool,
        now: Instant,
    ) {
        let Some(entry) = self.entries.get_mut(&id) else {
            return;
        };
        let mut running = false;
        for job in slots
            .iter()
            .flatten()
            .filter(|j| j.window.window_handle == id)
        {
            running = !job.cancelled.load(Ordering::Acquire);
            if cancel {
                job.cancelled.store(true, Ordering::Release);
            }
        }
        // A hint can consume an already-running walk. A click needs a new walk.
        entry.urgent |= cancel || !running;
        if interaction {
            // One immediate refresh and one trailing refresh cover a transition.
            // Repeated input coalesces the trailing pass, rather than extending
            // a 100 ms polling burst indefinitely while hints are closed.
            entry.follow_up = Some(now + INTERACTION_SETTLE);
            entry.next_scan = entry.last_started
                .map(|started| (started + INTERACTION_SETTLE).max(now))
                .unwrap_or(now);
        } else {
            // Explicit hint activation always bypasses background throttling.
            entry.next_scan = now;
        }
        entry.failures = 0;
    }

    fn refresh_requests(
        &mut self,
        requests: &RefreshRequests,
        slots: &[Option<Job>; 2],
        now: Instant,
    ) {
        if requests.hint || !requests.clicks.is_empty() {
            if let Some(id) = foreground(&self.windows) {
                self.refresh_target(
                    id,
                    slots,
                    !requests.clicks.is_empty(),
                    !requests.clicks.is_empty(),
                    now,
                );
            }
        }
        for &(x, y) in &requests.clicks {
            // Include the first normal window as well as a possible popup/Dock.
            // A transparent floating host must not hide the app underneath it.
            let contains = |w: &&WindowElement| {
                x as i64 >= w.x as i64
                    && y as i64 >= w.y as i64
                    && (x as i64) < w.x as i64 + w.width as i64
                    && (y as i64) < w.y as i64 + w.height as i64
            };
            let top = self.windows.iter().find(contains).map(|w| w.window_handle);
            let normal = self
                .windows
                .iter()
                .filter(|w| normal_window(w))
                .find(contains)
                .map(|w| w.window_handle);
            for id in [top, normal].into_iter().flatten() {
                self.refresh_target(id, slots, true, true, now);
            }
        }
    }

    fn finish(&mut self, job: &Job, success: bool, policy: Policy, now: Instant) {
        if job.cancelled.load(Ordering::Acquire) {
            return;
        }
        let Some(entry) = self.entries.get_mut(&job.window.window_handle) else {
            return;
        };
        if entry.job != job.id {
            return;
        }
        if success {
            entry.completed_at = Some(now);
            entry.failures = 0;
            entry.next_scan = now;
        } else {
            // Failed/unsupported windows have no successful cache to skip.
            // Back off those too, including while hints remain open.
            entry.failures = entry.failures.saturating_add(1);
            let delay = policy.idle_interval.as_millis().max(1000) as u64;
            entry.next_scan = now
                + Duration::from_millis(
                    delay
                        .saturating_mul(1 << (entry.failures - 1).min(6))
                        .min(60_000),
                );
        }
    }

    fn publish(&mut self, job: &Job, mut elements: Vec<UIElement>, complete: bool) -> bool {
        if job.cancelled.load(Ordering::Acquire) {
            return false;
        }
        let Some(entry) = self.entries.get_mut(&job.window.window_handle) else {
            return false;
        };
        if entry.job != job.id {
            return false;
        }
        if let Some(previous) = &mut entry.snapshot {
            if !complete {
                // Keep warm points while a rescan is still walking. A partial
                // scan must not shrink the only cache used by a new activation.
                let changed = elements.iter().any(|element| {
                    previous
                        .elements
                        .binary_search_by_key(&(element.y, element.x), |e| (e.y, e.x))
                        .map_or(true, |i| previous.elements[i] != *element)
                });
                if !changed {
                    return false;
                }
                let mut merged: HashMap<_, _> = previous
                    .elements
                    .iter()
                    .map(|e| ((e.y, e.x), e.clone()))
                    .collect();
                merged.extend(elements.into_iter().map(|e| ((e.y, e.x), e)));
                elements = merged.into_values().collect();
                elements.sort_by_key(|e| (e.y, e.x));
            } else if *previous.elements == elements {
                previous.scanned_at = Instant::now();
                return false;
            }
        }
        self.revision += 1;
        entry.snapshot = Some(Snapshot {
            window: job.window.clone(),
            elements: Arc::new(elements),
            revision: self.revision,
            scan_id: job.id,
            scanned_at: Instant::now(),
        });
        true
    }
}

#[derive(Clone)]
struct Job {
    id: u64,
    window: WindowElement,
    cancelled: Arc<AtomicBool>,
}

enum Event {
    Wake,
    Done(usize, u64, bool),
}

#[derive(Default)]
struct RefreshRequests {
    hint: bool,
    clicks: Vec<(i32, i32)>,
}

struct Collection {
    cache: Mutex<Cache>,
    sender: Mutex<Option<mpsc::Sender<Event>>>,
    wake_pending: AtomicBool,
    hint_active: AtomicBool,
    requested: AtomicU64,
    pending: Mutex<RefreshRequests>,
    changed: tokio::sync::watch::Sender<u64>,
}

static COLLECTION: Lazy<Collection> = Lazy::new(|| Collection {
    cache: Mutex::new(Cache::default()),
    sender: Mutex::new(None),
    wake_pending: AtomicBool::new(false),
    hint_active: AtomicBool::new(false),
    changed: tokio::sync::watch::channel(0).0,
    requested: AtomicU64::new(0),
    pending: Mutex::new(RefreshRequests::default()),
});

/// Safe in the input hook: coalesces requests, does no AX/UIA or window enumeration.
fn request_refresh(point: Option<(i32, i32)>) -> u64 {
    let mut pending = COLLECTION.pending.lock().unwrap();
    if let Some(point) = point {
        if !pending.clicks.contains(&point) {
            if pending.clicks.len() == 8 {
                pending.clicks.remove(0);
            }
            pending.clicks.push(point);
        }
    } else {
        pending.hint = true;
    }
    let request = COLLECTION.requested.fetch_add(1, Ordering::AcqRel) + 1;
    drop(pending);
    if !COLLECTION.wake_pending.swap(true, Ordering::AcqRel) {
        if let Some(sender) = COLLECTION.sender.lock().unwrap().as_ref() {
            let _ = sender.send(Event::Wake);
        }
    }
    request
}

pub fn request_refresh_at(x: i32, y: i32) {
    request_refresh(Some((x, y)));
}

pub fn set_hint_active(active: bool) -> u64 {
    COLLECTION.hint_active.store(active, Ordering::Release);
    COLLECTION
        .changed
        .send_modify(|revision| *revision = revision.wrapping_add(1));
    if active {
        request_refresh(None)
    } else {
        COLLECTION.requested.load(Ordering::Acquire)
    }
}

pub fn subscribe() -> tokio::sync::watch::Receiver<u64> {
    COLLECTION.changed.subscribe()
}

pub fn snapshots_since(
    seen: &mut HashMap<i64, u64>,
    request: u64,
) -> Option<(Vec<WindowElement>, Vec<Snapshot>)> {
    let ttl = Duration::from_millis(
        config::CONFIG
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .ui_automation
            .cache_ttl
            .max(100),
    );
    let cache = COLLECTION.cache.lock().unwrap();
    if cache.topology_request < request {
        return None;
    }
    let snapshots = cache
        .windows
        .iter()
        .filter_map(|window| {
            let snapshot = cache
                .entries
                .get(&window.window_handle)?
                .snapshot
                .as_ref()?;
            if snapshot.scanned_at.elapsed() > ttl
                || seen.get(&window.window_handle) == Some(&snapshot.revision)
            {
                return None;
            }
            seen.insert(window.window_handle, snapshot.revision);
            let mut snapshot = snapshot.clone();
            snapshot.window = window.clone();
            Some(snapshot)
        })
        .collect();
    Some((cache.windows.clone(), snapshots))
}

// Reserve a lane for the front normal window, even when a floating host overlaps it.
fn normal_window(window: &WindowElement) -> bool {
    #[cfg(target_os = "macos")]
    return !window.is_task_bar && window.z_index == 0;
    #[cfg(not(target_os = "macos"))]
    return !window.is_task_bar;
}

pub(crate) fn foreground(windows: &[WindowElement]) -> Option<i64> {
    let front = windows.iter().find(|w| normal_window(w));
    front.or_else(|| windows.first()).map(|w| w.window_handle)
}

#[derive(Clone, Copy)]
struct Policy {
    idle_interval: Duration,
    ttl: Duration,
    hint_active: bool,
}

impl Policy {
    fn current() -> Self {
        let config = config::CONFIG.lock().unwrap();
        let config = &config.as_ref().unwrap().ui_automation;
        Self {
            idle_interval: Duration::from_millis(config.collect_interval.max(100)),
            ttl: Duration::from_millis(config.cache_ttl.max(100)),
            hint_active: COLLECTION.hint_active.load(Ordering::Acquire),
        }
    }
}

impl Entry {
    fn due_at(&self, foreground: bool, policy: Policy, _now: Instant) -> Instant {
        if self.urgent || self.failures > 0 {
            return self.next_scan;
        }
        let Some(completed) = self.completed_at else {
            return self.next_scan;
        };
        let pause = if policy.hint_active {
            Duration::from_millis(if foreground { 250 } else { 1000 })
        } else {
            // A valid completed cache really skips native traversal, as in legacy.
            policy.ttl.max(policy.idle_interval)
        };
        // Always pause AFTER completion. Slow AX walks cannot run back to back.
        let normal = completed + pause;
        let due = self.follow_up
            .map(|follow_up| normal.min(follow_up.max(completed + INTERACTION_SETTLE)))
            .unwrap_or(normal);
        due.max(self.next_scan)
    }

    fn start(&mut self, id: u64, now: Instant) {
        self.job = id;
        self.urgent = false;
        self.last_started = Some(now);
        if self.follow_up.is_some_and(|follow_up| now >= follow_up) {
            self.follow_up = None;
        }
    }
}

fn next_window(
    cache: &Cache,
    slots: &[Option<Job>; 2],
    lane: usize,
    policy: Policy,
    now: Instant,
) -> Option<WindowElement> {
    let front = foreground(&cache.windows);
    cache
        .windows
        .iter()
        .filter(|w| (Some(w.window_handle) == front) == (lane == 0))
        .filter(|w| {
            !slots
                .iter()
                .flatten()
                .any(|job| job.window.window_handle == w.window_handle)
        })
        .filter(|w| cache.entries[&w.window_handle].due_at(lane == 0, policy, now) <= now)
        .min_by_key(|w| {
            let entry = &cache.entries[&w.window_handle];
            (!entry.urgent, entry.due_at(lane == 0, policy, now))
        })
        .cloned()
}

fn worker(lane: usize, jobs: mpsc::Receiver<Job>, events: mpsc::Sender<Event>) {
    #[cfg(target_os = "windows")]
    let mut com_ready = false;
    for job in jobs {
        #[cfg(target_os = "windows")]
        if !com_ready {
            use windows::Win32::System::Com::*;
            match unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.ok() {
                Ok(()) => com_ready = true,
                Err(error) => {
                    log::error!("[ui_scan] COM initialization failed: {error}");
                    let _ = events.send(Event::Done(lane, job.id, false));
                    continue;
                }
            }
        }
        let started = Instant::now();
        let mut control = ScanControl::new(job.cancelled.clone(), |elements, complete| {
            let mut cache = COLLECTION.cache.lock().unwrap();
            if cache.publish(&job, elements, complete) {
                COLLECTION.changed.send_replace(cache.revision);
            }
        });
        // Requests and all native references are created/dropped on this worker.
        let result = UIAutomationRequest::new()
            .and_then(|request| request.get_elements_for_window(&job.window, &mut control));
        let success = result.is_some();
        if let Some(elements) = result {
            control.finish(elements);
            // An activation may have skipped an expired snapshot. Completing
            // an unchanged scan refreshes its timestamp, so wake that subscriber
            // too; revision filtering still prevents redundant frontend work.
            COLLECTION
                .changed
                .send_modify(|revision| *revision = revision.wrapping_add(1));
        }
        log::debug!(
            "[ui_scan] window={} lane={} elapsed_ms={:.1} cancelled={}",
            job.window.window_handle,
            lane,
            started.elapsed().as_secs_f64() * 1000.0,
            job.cancelled.load(Ordering::Acquire)
        );
        let _ = events.send(Event::Done(lane, job.id, success));
    }
    #[cfg(target_os = "windows")]
    if com_ready {
        unsafe {
            windows::Win32::System::Com::CoUninitialize();
        }
    }
}

pub fn setup_ui_collection(_config: &config::Config) {
    let (tx, rx) = mpsc::channel();
    let mut sender = COLLECTION.sender.lock().unwrap();
    if sender.is_some() {
        return;
    }
    *sender = Some(tx.clone());
    COLLECTION.wake_pending.store(false, Ordering::Release);
    drop(sender);
    std::thread::Builder::new()
        .name("ui-scheduler".into())
        .spawn(move || {
            let lanes: Vec<_> = (0..2)
                .map(|lane| {
                    let (jobs, receiver) = mpsc::channel();
                    let events = tx.clone();
                    std::thread::Builder::new()
                        .name(format!("ui-scan-{lane}"))
                        .spawn(move || worker(lane, receiver, events))
                        .unwrap();
                    jobs
                })
                .collect();
            let mut slots: [Option<Job>; 2] = [None, None];
            let mut next_topology = Instant::now();
            let mut next_id = 0;
            let mut refresh = false;
            loop {
                let now = Instant::now();
                if refresh || now >= next_topology {
                    #[cfg(target_os = "macos")]
                    let permitted = crate::macos_access::is_accessibility_trusted();
                    #[cfg(not(target_os = "macos"))]
                    let permitted = true;
                    let (requests, request) = {
                        let mut pending = COLLECTION.pending.lock().unwrap();
                        (
                            std::mem::take(&mut *pending),
                            COLLECTION.requested.load(Ordering::Acquire),
                        )
                    };
                    let windows = if permitted {
                        crate::window::window::get_all_windows()
                    } else {
                        Vec::new()
                    };
                    let mut cache = COLLECTION.cache.lock().unwrap();
                    let changed = cache.reconcile(windows, now);
                    cache.topology_request = request;
                    cache.cancel_obsolete(&slots);
                    cache.refresh_requests(&requests, &slots, now);
                    if refresh || changed {
                        cache.revision += 1;
                        COLLECTION.changed.send_replace(cache.revision);
                    }
                    next_topology = Instant::now() + TOPOLOGY_INTERVAL;
                    refresh = false;
                }
                let policy = Policy::current();
                let mut cache = COLLECTION.cache.lock().unwrap();
                for lane in 0..2 {
                    if slots[lane].is_some() {
                        continue;
                    }
                    if let Some(window) = next_window(&cache, &slots, lane, policy, Instant::now())
                    {
                        next_id += 1;
                        let entry = cache.entries.get_mut(&window.window_handle).unwrap();
                        entry.start(next_id, Instant::now());
                        let job = Job {
                            id: next_id,
                            window,
                            cancelled: Arc::new(AtomicBool::new(false)),
                        };
                        let _ = lanes[lane].send(job.clone());
                        slots[lane] = Some(job);
                    }
                }
                let front = foreground(&cache.windows);
                let next_due = cache
                    .entries
                    .iter()
                    .filter(|(id, _)| {
                        let lane = if Some(**id) == front { 0 } else { 1 };
                        slots[lane].is_none()
                            && !slots
                                .iter()
                                .flatten()
                                .any(|j| j.window.window_handle == **id)
                    })
                    .map(|(id, e)| e.due_at(Some(*id) == front, policy, Instant::now()))
                    .min()
                    .unwrap_or(next_topology)
                    .min(next_topology);
                drop(cache);
                match rx.recv_timeout(
                    next_due
                        .saturating_duration_since(Instant::now())
                        .max(Duration::from_millis(1)),
                ) {
                    Ok(Event::Wake) => {
                        COLLECTION.wake_pending.store(false, Ordering::Release);
                        refresh = true;
                    }
                    Ok(Event::Done(lane, id, success)) => {
                        if slots[lane].as_ref().map(|job| job.id) == Some(id) {
                            let job = slots[lane].take().unwrap();
                            COLLECTION.cache.lock().unwrap().finish(
                                &job,
                                success,
                                policy,
                                Instant::now(),
                            );
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        })
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(id: i64) -> WindowElement {
        WindowElement {
            window_handle: id,
            owner_pid: id as i32,
            x: 0,
            y: 0,
            width: 800,
            height: 600,
            title: format!("window-{id}"),
            class_name: String::new(),
            z_index: 0,
            visible: true,
            is_task_bar: false,
        }
    }
    fn point(x: i32) -> UIElement {
        UIElement {
            text: String::new(),
            x,
            y: 10,
            z: 1,
            width: 5,
            height: 5,
            window_handle: 1,
            control_type: 0,
            element_type: 0,
        }
    }
    fn job(cache: &mut Cache, id: i64, serial: u64) -> Job {
        cache.entries.get_mut(&id).unwrap().job = serial;
        cache.entries.get_mut(&id).unwrap().urgent = false;
        Job {
            id: serial,
            window: cache
                .windows
                .iter()
                .find(|w| w.window_handle == id)
                .unwrap()
                .clone(),
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    #[test]
    fn hotkey_bypasses_interval_and_old_scan_cannot_overwrite_new_points() {
        let now = Instant::now();
        let mut cache = Cache::default();
        cache.reconcile(vec![window(1)], now);
        let old = job(&mut cache, 1, 1);
        cache.publish(&old, vec![point(10)], true);
        cache.entries.get_mut(&1).unwrap().next_scan = now + Duration::from_secs(60);
        assert!(next_window(&cache, &[None, None], 0, idle(), now).is_none());
        cache.refresh_target(1, &[Some(old.clone()), None], true, true, now);
        assert!(old.cancelled.load(Ordering::Acquire));
        assert_eq!(
            next_window(&cache, &[None, None], 0, idle(), now)
                .unwrap()
                .window_handle,
            1
        );
        let new = job(&mut cache, 1, 2);
        assert!(cache.publish(&new, vec![point(20)], true));
        assert!(!cache.publish(&old, vec![point(99)], true));
        assert_eq!(
            *cache.entries[&1].snapshot.as_ref().unwrap().elements,
            vec![point(20)]
        );
    }

    #[test]
    fn background_work_never_occupies_foreground_lane_or_duplicates_a_window_scan() {
        let now = Instant::now();
        let mut cache = Cache::default();
        cache.reconcile(vec![window(1), window(2), window(3)], now);
        let slow = job(&mut cache, 2, 1);
        let slots = [None, Some(slow.clone())];
        assert_eq!(
            next_window(&cache, &slots, 0, idle(), now)
                .unwrap()
                .window_handle,
            1
        );
        // Switch to the window already scanned by the other lane: cancel it,
        // then wait for it to release native references before scanning again.
        cache.reconcile(vec![window(2), window(1), window(3)], now);
        cache.cancel_obsolete(&slots);
        assert!(slow.cancelled.load(Ordering::Acquire));
        assert!(next_window(&cache, &slots, 0, idle(), now).is_none());
        assert_eq!(
            next_window(&cache, &[None, None], 0, idle(), now)
                .unwrap()
                .window_handle,
            2
        );
    }

    #[test]
    fn progressive_scan_keeps_warm_points_and_unchanged_results_share_storage() {
        let now = Instant::now();
        let mut cache = Cache::default();
        cache.reconcile(vec![window(1)], now);
        let first = job(&mut cache, 1, 1);
        cache.publish(&first, vec![point(10), point(20)], true);
        let warm = cache.entries[&1]
            .snapshot
            .as_ref()
            .unwrap()
            .elements
            .clone();
        let rescan = job(&mut cache, 1, 2);
        assert!(!cache.publish(&rescan, vec![point(10)], false));
        assert!(Arc::ptr_eq(
            &warm,
            &cache.entries[&1].snapshot.as_ref().unwrap().elements
        ));
        assert!(!cache.publish(&rescan, vec![point(10), point(20)], true));
        assert!(Arc::ptr_eq(
            &warm,
            &cache.entries[&1].snapshot.as_ref().unwrap().elements
        ));
        assert!(cache.publish(&rescan, vec![point(30)], false));
        assert_eq!(cache.entries[&1].snapshot.as_ref().unwrap().scan_id, rescan.id);
        assert_eq!(
            *cache.entries[&1].snapshot.as_ref().unwrap().elements,
            vec![point(10), point(20), point(30)]
        );
        // Completion can prune the source cache; it does not delete session hints.
        assert!(cache.publish(&rescan, vec![point(30)], true));
        assert_eq!(cache.entries[&1].snapshot.as_ref().unwrap().scan_id, rescan.id);
        assert_eq!(
            *cache.entries[&1].snapshot.as_ref().unwrap().elements,
            vec![point(30)]
        );
    }

    #[test]
    fn topology_changes_recheck_visibility_and_discard_moved_or_closed_windows() {
        let now = Instant::now();
        let mut cache = Cache::default();
        cache.reconcile(vec![window(1), window(2)], now);
        let old = job(&mut cache, 1, 1);
        cache.publish(&old, vec![point(10)], true);
        let revision = cache.entries[&1].snapshot.as_ref().unwrap().revision;
        cache.reconcile(vec![window(2), window(1)], now);
        assert!(cache.entries[&1].snapshot.as_ref().unwrap().revision > revision);
        let mut moved = window(1);
        moved.x = 100;
        cache.reconcile(vec![moved], now);
        assert!(cache.entries[&1].snapshot.is_none());
        assert!(!cache.entries.contains_key(&2));
        assert!(!cache.publish(&old, vec![point(10)], true));
        cache.reconcile(vec![], now);
        assert!(cache.entries.is_empty());
        assert!(!cache.publish(&old, vec![point(10)], true));
    }

    fn warm(cache: &mut Cache, id: i64, serial: u64, now: Instant) {
        let work = job(cache, id, serial);
        cache.entries.get_mut(&id).unwrap().start(serial, now);
        cache.publish(&work, vec![point(id as i32)], true);
        cache.finish(&work, true, idle(), now);
    }

    #[test]
    fn one_click_performs_only_an_immediate_and_a_settled_refresh() {
        let start = Instant::now();
        let mut cache = Cache::default();
        cache.reconcile(vec![window(1)], start);
        warm(&mut cache, 1, 1, start + Duration::from_secs(1));
        let click = start + Duration::from_secs(5);
        cache.refresh_target(1, &[], true, true, click);
        let mut scans = 0;
        for tick in 0..200 {
            let now = click + Duration::from_millis(tick * 10);
            if next_window(&cache, &[None, None], 0, idle(), now).is_some() {
                scans += 1;
                warm(&mut cache, 1, scans + 1, now);
            }
        }
        assert_eq!(scans, 2, "background prewarming must stop after the trailing pass");
        assert!(cache.entries[&1].follow_up.is_none());
    }

    #[test]
    fn rapid_clicks_coalesce_and_hint_activation_bypasses_the_cooldown() {
        let start = Instant::now();
        let mut cache = Cache::default();
        cache.reconcile(vec![window(1)], start);
        warm(&mut cache, 1, 1, start + Duration::from_secs(1));
        let click = start + Duration::from_secs(5);
        cache.refresh_target(1, &[], true, true, click);
        warm(&mut cache, 1, 2, click);
        for ms in [10, 20, 30, 40] {
            let now = click + Duration::from_millis(ms);
            cache.refresh_target(1, &[], true, true, now);
            assert!(next_window(&cache, &[None, None], 0, idle(), now).is_none());
        }
        let now = click + Duration::from_millis(50);
        cache.refresh_target(1, &[], false, false, now);
        assert!(next_window(&cache, &[None, None], 0, idle(), now).is_some());
    }

    #[test]
    fn a_slow_scan_starting_after_the_transition_needs_no_extra_background_pass() {
        let start = Instant::now();
        let mut cache = Cache::default();
        cache.reconcile(vec![window(1)], start);
        let click = start + Duration::from_secs(5);
        cache.refresh_target(1, &[], true, true, click);
        // The worker was busy elsewhere. Its eventual scan already covers the
        // settled state and must consume the follow-up request.
        warm(&mut cache, 1, 1, click + Duration::from_millis(300));
        assert!(cache.entries[&1].follow_up.is_none());
        assert!(next_window(&cache, &[None, None], 0, idle(), click + Duration::from_secs(2)).is_none());
    }

    #[test]
    fn valid_completed_cache_skips_both_native_lanes_until_ttl() {
        let start = Instant::now();
        let mut cache = Cache::default();
        cache.reconcile(vec![window(1), window(2)], start);
        warm(&mut cache, 1, 1, start);
        warm(&mut cache, 2, 2, start);
        // Polling topology for an entire idle minute must not dispatch AX work.
        for tick in 8..240 {
            let now = start + Duration::from_millis(tick * 250);
            assert!(!cache.reconcile(vec![window(1), window(2)], now));
            for lane in 0..2 {
                assert!(next_window(&cache, &[None, None], lane, idle(), now).is_none());
            }
        }
        for lane in 0..2 {
            assert!(next_window(
                &cache,
                &[None, None],
                lane,
                idle(),
                start + Duration::from_secs(60)
            )
            .is_some());
        }
    }

    #[test]
    fn unrelated_window_changes_replay_visibility_without_invalidating_warm_trees() {
        let start = Instant::now();
        let mut cache = Cache::default();
        cache.reconcile(vec![window(1), window(2), window(3)], start);
        for id in 1..=3 {
            warm(&mut cache, id, id as u64, start);
        }
        let now = start + Duration::from_secs(5);
        let mut moved = window(2);
        moved.x = -600;
        cache.reconcile(vec![window(1), moved, window(3)], now);
        assert!(next_window(&cache, &[None, None], 0, idle(), now).is_none());
        assert_eq!(
            next_window(&cache, &[None, None], 1, idle(), now)
                .unwrap()
                .window_handle,
            2
        );
        assert_eq!(cache.entries[&3].completed_at, Some(start));
        assert_eq!(
            cache.entries[&3].due_at(false, idle(), now),
            start + Duration::from_secs(60)
        );
        // A z-order-only change among background windows is not an AX invalidation.
        warm(&mut cache, 2, 4, now);
        cache.reconcile(vec![window(1), window(3), cache.windows[1].clone()], now);
        assert!(next_window(&cache, &[None, None], 1, idle(), now).is_none());
    }

    #[test]
    fn switching_front_bypasses_its_cache_without_refreshing_the_old_front() {
        let start = Instant::now();
        let mut cache = Cache::default();
        cache.reconcile(vec![window(1), window(2), window(3)], start);
        for id in 1..=3 {
            warm(&mut cache, id, id as u64, start);
        }
        let now = start + Duration::from_secs(5);
        cache.reconcile(vec![window(2), window(1), window(3)], now);
        assert_eq!(
            next_window(&cache, &[None, None], 0, idle(), now)
                .unwrap()
                .window_handle,
            2
        );
        assert!(next_window(&cache, &[None, None], 1, idle(), now).is_none());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn click_refreshes_position_and_front_but_preserves_unrelated_scan_and_cache() {
        let start = Instant::now();
        let mut cache = Cache::default();
        let mut left = window(2);
        left.x = -900;
        let mut floating = window(4);
        floating.x = -900;
        floating.z_index = -3;
        cache.reconcile(vec![floating, window(1), left, window(3)], start);
        for id in 1..=4 {
            warm(&mut cache, id, id as u64, start);
        }
        let unrelated = job(&mut cache, 3, 5);
        let now = start + Duration::from_secs(5);
        cache.refresh_requests(
            &RefreshRequests {
                hint: false,
                clicks: vec![(-500, 100)],
            },
            &[None, Some(unrelated.clone())],
            now,
        );
        for id in [1, 2, 4] {
            assert!(cache.entries[&id].urgent);
        }
        assert!(!cache.entries[&3].urgent);
        assert!(!unrelated.cancelled.load(Ordering::Acquire));
        assert_eq!(
            cache.entries[&3].due_at(false, idle(), now),
            start + Duration::from_secs(60)
        );
    }

    #[test]
    fn hint_rescans_while_open_and_immediately_returns_to_cache_when_closed() {
        let start = Instant::now();
        let mut cache = Cache::default();
        cache.reconcile(vec![window(1), window(2)], start);
        for id in 1..=2 {
            warm(&mut cache, id, id as u64, start);
        }
        let now = start + Duration::from_secs(5);
        cache.refresh_requests(
            &RefreshRequests {
                hint: true,
                clicks: vec![],
            },
            &[None, None],
            now,
        );
        let active = Policy {
            hint_active: true,
            ..idle()
        };
        assert_eq!(
            next_window(&cache, &[None, None], 0, active, now)
                .unwrap()
                .window_handle,
            1
        );
        assert!(next_window(&cache, &[None, None], 1, active, now).is_some());
        warm(&mut cache, 1, 3, now);
        warm(&mut cache, 2, 4, now);
        assert!(next_window(
            &cache,
            &[None, None],
            0,
            active,
            now + Duration::from_millis(249)
        )
        .is_none());
        assert!(next_window(
            &cache,
            &[None, None],
            0,
            active,
            now + Duration::from_millis(250)
        )
        .is_some());
        for lane in 0..2 {
            assert!(next_window(
                &cache,
                &[None, None],
                lane,
                idle(),
                now + Duration::from_secs(2)
            )
            .is_none());
        }
    }

    #[test]
    fn slow_walks_pause_after_completion_and_failures_back_off_until_targeted_input() {
        let start = Instant::now();
        let mut cache = Cache::default();
        cache.reconcile(vec![window(1)], start);
        let active = Policy {
            hint_active: true,
            ..idle()
        };
        let work = job(&mut cache, 1, 1);
        let done = start + Duration::from_secs(5);
        cache.finish(&work, true, active, done);
        assert!(next_window(&cache, &[None, None], 0, active, done).is_none());
        assert_eq!(
            cache.entries[&1].due_at(true, active, done),
            done + Duration::from_millis(250)
        );
        let failed = job(&mut cache, 1, 2);
        cache.finish(&failed, false, active, done);
        assert_eq!(
            cache.entries[&1].due_at(true, active, done),
            done + Duration::from_secs(1)
        );
        cache.finish(&failed, false, active, done + Duration::from_secs(1));
        assert_eq!(
            cache.entries[&1].due_at(true, active, done),
            done + Duration::from_secs(3)
        );
        cache.refresh_requests(
            &RefreshRequests {
                hint: true,
                clicks: vec![],
            },
            &[None, None],
            done,
        );
        assert!(next_window(&cache, &[None, None], 0, active, done).is_some());
    }

    #[test]
    fn partial_or_cancelled_walks_cannot_extend_completed_cache_lifetime() {
        let now = Instant::now();
        let mut cache = Cache::default();
        cache.reconcile(vec![window(1)], now);
        let partial = job(&mut cache, 1, 1);
        cache.publish(&partial, vec![point(20)], false);
        assert!(cache.entries[&1].completed_at.is_none());
        partial.cancelled.store(true, Ordering::Release);
        cache.finish(&partial, true, idle(), now);
        assert!(cache.entries[&1].completed_at.is_none());
        let current = job(&mut cache, 1, 2);
        cache.publish(&current, vec![point(20)], true);
        cache.finish(&current, true, idle(), now);
        assert_eq!(cache.entries[&1].completed_at, Some(now));
    }

    #[test]
    fn opening_hint_keeps_current_scan_but_a_click_cancels_only_its_target() {
        let now = Instant::now();
        let mut cache = Cache::default();
        cache.reconcile(vec![window(1), window(2)], now);
        let front = job(&mut cache, 1, 1);
        let other = job(&mut cache, 2, 2);
        let slots = [Some(front.clone()), Some(other.clone())];
        cache.refresh_requests(
            &RefreshRequests {
                hint: true,
                clicks: vec![],
            },
            &slots,
            now,
        );
        assert!(!front.cancelled.load(Ordering::Acquire));
        assert!(!cache.entries[&1].urgent);
        cache.refresh_requests(
            &RefreshRequests {
                hint: false,
                clicks: vec![(10, 10)],
            },
            &slots,
            now,
        );
        assert!(front.cancelled.load(Ordering::Acquire));
        assert!(!other.cancelled.load(Ordering::Acquire));
        assert!(!cache.publish(&front, vec![point(30)], true));
    }

    fn idle() -> Policy {
        Policy {
            idle_interval: Duration::from_millis(1000),
            ttl: Duration::from_secs(60),
            hint_active: false,
        }
    }
}
