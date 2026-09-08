# macOS validation matrix

Candidate: 1.2.1, branch `fix/macos-readiness`. Date: 2026-09-08.

## Recorded checks

| Check | Result |
| --- | --- |
| TypeScript and frontend production build | PASS |
| Package / Cargo / bundle version agreement | PASS |
| Rust regression suite on Apple Silicon | PASS, 12 tests |
| Universal release bundle | PASS: arm64 + x86_64; bundle 1.2.1; ad-hoc signature verifies |
| Installed runtime flow | Core Safari flows passed; see recorded runtime below |
| Intel instruction-set regression suite | PASS, 12 tests via Rosetta; not physical Intel acceptance |
| Config migration and app replacement | PASS: original preferences preserved byte-for-byte |
| Login startup configuration | PASS: launch agent points to `/Applications/screen-buoy.app`; login cycle not tested |
| Developer ID / notarization | BLOCKED: no Developer ID Application identity available locally |
| Physical Intel runtime | Not tested |

Tests cover window occlusion (floating, same-level, aggregate coverage), negative screen coordinates, AX window matching, transparent system desktop hosts, left/right modifier flags, rectangle subtraction, default config creation, migration and preservation. They do not synthesize input or change system permissions.

## Recorded runtime

Host: macOS 15.6.1 on Apple Silicon, two 2x displays. The external display lies above/left of the primary (negative global coordinates). Existing user settings retained `Alt+Q`, login startup enabled, and debug logging; the default for new installs remains `Alt+H`.

- Replaced the two previous app copies with the local universal candidate in `/Applications`. Archived the previous installations and migrated config before replacement.
- Confirmed the settings permission warning, executable path, system-settings links, recovery after re-adding the updated ad-hoc app, and the ready state. Accessibility alone enabled the active event tap on this machine; Input Monitoring preflight stayed false.
- Used a disposable local Safari page ([fixture](fixtures/macos-input-qa.html)) and explicitly authorized CGEvent test input. The driver checks the foreground app and exact fixture window title before input.
- Global hotkey displayed hints with Safari retaining keyboard focus. AX readback placed button hints within about one point of their target centers.
- Hint-label clicks incremented the page counter on both displays, including a target at `(-222, -1316)`.
- Space at the existing pointer position incremented the single-click counter. Space held with Enter incremented the double-click counter.
- Space held with K scrolled the inner panel by 120 points. Space held with Right moved a draggable item into its drop target; releasing Space completed the drop.
- After the final modifier-event fix, Space + Right Shift incremented the right-click counter. Space click, Enter double-click, K scroll and Right-arrow drag were repeated on the installed final binary. Final counters: 4 clicks, 2 double clicks, 1 right click, 2 drops, scroll offset 240.
- After exiting Hint mode, native synthesized key events entered `123` in the fixture text field. The overlay AX trees contained no hints and both mouse buttons read released.
- Readback confirmed the installed executable matches the final universal build, bundle version is 1.2.1, the login target is correct, and the original config hash is unchanged.

During testing, display-sized transparent Dock/Notification Center windows initially suppressed every Safari control hint; the filtering regression was fixed and the same button flow passed afterward. Synthetic Right Shift originally read as released because session state lagged the event; event-flag decoding was added with regression tests. These are distinct from generic application AX coverage.

Not yet accepted: physical Intel hardware, older macOS, mixed display scale factors, fullscreen/Spaces, monitor hotplug, permission revocation during an active gesture, 30-minute uninterrupted exploration, and Windows runtime. No Developer ID/notarization credentials are available; these runtime results describe the local ad-hoc build. GitHub release artifacts have separate CI and archive verification.

## Runtime procedure

Use a disposable Finder folder, browser page, and editor document. Keep unsaved work out of the test surface.

1. Without permissions, launch the app: settings must explain recovery; the configured hotkey must not leave an unresponsive overlay.
2. Grant Accessibility for the exact installed app. If capture is unavailable, also grant Input Monitoring. Refresh/restart as directed; check the keyboard capture status.
3. In each target app, press the configured hotkey (Option+H by default): labels appear without changing the focused app. Type a control's label; verify the actual control receives the click.
4. Hold the last hint key; verify right click, double click, arrow-key drag, ESDF/IJKL scrolling, then Escape. Verify mouse button state is released and normal typing resumes.
5. Repeat on each display, including a monitor left/above the primary and mixed scaling. Verify the pointer lands on the selected control.
6. Overlap normal and floating windows. Hidden controls must not receive labels; visible uncovered portions should remain usable.
7. Switch Spaces and fullscreen apps; check visibility, click-through and focus preservation. Disconnect/reconnect a monitor and record whether restart is required.
8. Revoke permission during use and verify the recovery path; repeated toggling and a 30-minute exploration must not crash or leave input captured.
9. Restart the installed app and replace it with a new build; settings must persist. Check login startup separately when enabled.

Record runtime results separately from compile/test results. Do not mark a hardware or OS combination passed using only a cross-compiled artifact.
