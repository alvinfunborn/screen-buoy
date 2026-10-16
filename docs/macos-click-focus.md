# macOS focus before clicking

Implemented on 2026-10-13 and installed on 2026-10-14. Desktop acceptance
found a key-release regression: `hold_at_hint` hid the session before the
asynchronous `left_click_exit`, causing `hint session ended` cancellations.
The macOS release dispatcher now schedules short clicks before hold cleanup;
long releases retain hold/drag cleanup. The user confirmed the corrected installed candidate works on 2026-10-14.
The broader manual matrix below remains unverified.

The macOS click path now captures the live window and AX element under the
pointer. For an ordinary or floating app window, it checks both `AXFrontmost`
and the exact `AXFocusedWindow`. If necessary, it requests `AXMain`, `AXRaise`
and `AXFrontmost`, then polls for confirmation. An accepted activation request
alone does not authorize a click.

- Already-focused targets take the path without activation or a polling delay.
- AX calls and polling run in a blocking worker, not on the AppKit main thread.
  The main thread only resolves the native window at the point.
- Poll interval is 10 ms, with a 350 ms cooperative operation deadline and
  50 ms per-object AX IPC timeout. A native call cannot be interrupted; no click
  is posted if the deadline has passed when control returns.
- Before posting input, the code checks the original pointer position, window
  ID/PID/bounds, AX element identity/bounds, and confirmed window focus again.
  It cancels on a missing target, changed geometry, lost focus or timeout.
- Menu items, Dock items and desktop surfaces retain a direct click path with
  live window/coordinate validation. A positive window layer alone is not an
  exemption: floating app windows still require focus confirmation.
- A single-click command posts one down/up pair with click count 1. There is no
  compensating click or retry. An explicit double-click command retains its
  two-pair semantics.
- Escape, a new hint session, a newer click request, or pointer movement cancels
  pending input. Requests reserve their cancellation ticket before being
  spawned; stale tasks do not dismiss a newer hint session.
- Quick-click key release keeps macOS hint input active until the click finishes,
  allowing Escape during the wait. Move-only, scroll and drag do not activate
  target windows. Windows retains its existing executor and mouse behavior.

`[click_focus macos]` logs completion time or a cancellation reason. A completion
log means input was posted, not that the target application accepted the action.

## Validation

Run `cargo test --locked --lib` and `cargo check --locked` in `src-tauri`.
Regression tests exercise immediate focus, delayed focus, unconfirmed activation,
changed targets, cancellation during polling, focus loss during revalidation,
negative coordinates and floating-window versus menu/Dock classification.

Manual acceptance with a disposable button-counter page:

1. Focus another app; select a button hint in the background browser. Verify one
   increment with one command. Repeat with an already-focused target.
2. Repeat between two windows of the same browser, and on a floating app window.
3. Repeat on the display above/left of the primary and with AutoRaise disabled.
4. While focus is pending, press Escape or move/resize/close the target window;
   verify no stale click or unexpected increment.
5. Check menu/Dock, right/middle click, explicit double click, move-only and drag.

No desktop success or Windows runtime result is inferred from the Rust tests.
