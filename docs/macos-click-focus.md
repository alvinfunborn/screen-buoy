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
- Some custom-drawn content (observed in WeChat chat images) does not implement
  AX point hit testing. Only `kAXErrorNotImplemented` (-25208) and
  `kAXErrorNoValue` (-25212) may use a window target instead: `AXWindows` must
  contain exactly one matching frame, the exact AX window must gain focus,
  and its identity/frame plus the native receiver ID/PID/frame are rechecked
  before posting. This validates the coordinate and window, not a semantic
  control inside the window. IPC failure, disabled AX, ambiguous matches and
  missing window geometry still cancel; there is no application-name exception.
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
Failed AX hits also log their numeric status, PID, window ID and coordinates.
Apple documents the distinction between a missing/unsupported AX element and
communication errors in
[AXUIElementCopyElementAtPosition](https://developer.apple.com/documentation/applicationservices/1462077-axuielementcopyelementatposition).

## Validation

Run `cargo test --locked --lib` and `cargo check --locked` in `src-tauri`.
Regression tests exercise immediate focus, delayed focus, unconfirmed activation,
changed targets, cancellation during polling, focus loss during revalidation,
negative coordinates and floating-window versus menu/Dock classification.
They also cover missing-AX fallback eligibility and unique window matching.

Manual acceptance with a disposable button-counter page:

1. Focus another app; select a button hint in the background browser. Verify one
   increment with one command. Repeat with an already-focused target.
2. Repeat between two windows of the same browser, and on a floating app window.
3. Repeat on the display above/left of the primary and with AutoRaise disabled.
4. While focus is pending, press Escape or move/resize/close the target window;
   verify no stale click or unexpected increment.
5. Check menu/Dock, right/middle click, explicit double click, move-only and drag.
6. Run `docs/fixtures/macos-no-ax-hit.swift` as a disposable native app. Its
   application intentionally returns nil from AX hit testing (macOS reports
   -25208), while window/focus attributes remain available. Use a grid hint or
   Space-at-pointer and Enter: each command must add two mouse downs and one
   double click. Repeat with the window initially in the background, and cancel
   while focus is pending. Then verify a WeChat chat image opens as expected.

No desktop success or Windows runtime result is inferred from the Rust tests.
