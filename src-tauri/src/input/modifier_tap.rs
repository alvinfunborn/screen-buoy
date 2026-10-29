//! A tap is committed on release, only if no other input joined the gesture.
use std::time::{Duration, Instant};

const MAX_TAP_DURATION: Duration = Duration::from_millis(300);

#[derive(Default)]
pub struct ModifierTap {
    down: bool,
    candidate: Option<(Instant, u64)>,
}

impl ModifierTap {
    pub fn press(&mut self, now: Instant, eligible: bool, context: Option<u64>) {
        // A duplicate down must not restart the clock or revive a canceled tap.
        self.candidate = if !self.down && eligible {
            context.map(|context| (now, context))
        } else {
            None
        };
        self.down = true;
    }

    pub fn release(&mut self, now: Instant, eligible: bool, context: Option<u64>) -> bool {
        self.down = false;
        self.candidate.take().is_some_and(|(start, original)| {
            eligible && context == Some(original)
                && now.saturating_duration_since(start) <= MAX_TAP_DURATION
        })
    }

    pub fn cancel(&mut self) {
        self.candidate = None;
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_tap_fires_once_on_release_without_a_timer() {
        let mut tap = ModifierTap::default();
        let start = Instant::now();
        tap.press(start, true, Some(1));
        assert!(tap.release(start + Duration::from_millis(80), true, Some(1)));
        assert!(!tap.release(start + Duration::from_millis(90), true, Some(1)));
    }

    #[test]
    fn another_key_click_scroll_or_drag_cancels_even_if_it_ends_first() {
        let mut tap = ModifierTap::default();
        let start = Instant::now();
        tap.press(start, true, Some(1));
        tap.cancel();
        assert!(!tap.release(start + Duration::from_millis(80), true, Some(1)));
        tap.press(start, true, Some(1));
        assert!(tap.release(start + Duration::from_millis(80), true, Some(1)));
    }

    #[test]
    fn held_inputs_and_unknown_or_changed_foreground_cancel() {
        let start = Instant::now();
        for (eligible_down, eligible_up, before, after) in [
            (false, true, Some(1), Some(1)),
            (true, false, Some(1), Some(1)),
            (true, true, Some(1), Some(2)),
            (true, true, None, None),
            (true, true, Some(1), None),
        ] {
            let mut tap = ModifierTap::default();
            tap.press(start, eligible_down, before);
            assert!(!tap.release(start + Duration::from_millis(80), eligible_up, after));
        }
    }

    #[test]
    fn long_holds_and_duplicate_downs_do_not_trigger_or_rearm() {
        let start = Instant::now();
        let mut tap = ModifierTap::default();
        tap.press(start, true, Some(1));
        assert!(!tap.release(start + Duration::from_millis(301), true, Some(1)));
        tap.press(start, true, Some(1));
        tap.press(start + Duration::from_millis(250), true, Some(1));
        assert!(!tap.release(start + Duration::from_millis(280), true, Some(1)));
    }

    #[test]
    fn interrupted_capture_and_hint_sessions_discard_pending_input() {
        let start = Instant::now();
        let mut tap = ModifierTap::default();
        tap.press(start, true, Some(1));
        tap.reset();
        assert!(!tap.release(start + Duration::from_millis(80), true, Some(1)));
        tap.press(start, true, Some(1));
        assert!(tap.release(start + Duration::from_millis(80), true, Some(1)));
    }
}
