# Dock and AltTab validation

Date: 2026-09-14. Local fix based on 1.2.1, branch `fix/macos-overlay-windows`.
Host: macOS 15.6.1, Apple Silicon, AltTab 10.7.0, two 2x displays.
The secondary display is above/left at `(-385, -1440)`; its logical size is
2560 × 1440. The primary display is 1800 × 1169.

## Confirmed causes

- Both idle overlay windows were visible `AXStandardWindow` instances. AppKit's
  `ignoresCycle` does not exclude them from AltTab's accessibility-based window
  enumeration. AltTab 10.7.0 admits standard windows/dialogs, but excludes these
  floating auxiliary windows ([source](https://github.com/lwouis/alt-tab-macos/blob/v10.7.0/src/logic/WindowDiscriminator.swift)).
- Both overlays actually had CGWindow layer 5, below Dock's layer 20. Tao 0.33.0
  queues its `set_always_on_top` level update, overwriting the native level 25.
- Dock exposes zero `AXWindows`; its application has an `AXList` of `AXDockItem`
  icons. Its CGWindow bounds cover the entire display, although the icon list
  occupies only a narrow strip at the bottom.

## Changes

- Create passive overlays hidden, configure `AXFloatingWindow` and exclusion from
  the Windows menu on the main thread, and order them forward without making them
  key windows. Keep the native level at 25 with a single owner.
- Hide native windows when hint mode exits. Queued visibility work reads the
  current input state, so late hint batches cannot reopen an exited overlay.
- Represent Dock by its actual AX list bounds, once per process, and collect its
  clickable icons directly from that list. Match the system executable rather
  than its localized name; exclude separator/unknown Dock item subroles.
- Send a movement/drag event after absolute pointer warps as well as relative
  movement, so hover-driven controls receive movement before the click.

## Recorded results

| Check | Result |
| --- | --- |
| Rust regression suite | PASS: 13 tests on arm64; 13 via x86_64/Rosetta |
| `cargo check --locked --all-targets` | PASS on macOS |
| Frontend build and version consistency | PASS |
| Universal application and ad-hoc signature | PASS: arm64 + x86_64; signature verifies |
| Installed application | Matches the universal build byte-for-byte; user config unchanged |
| Accessibility recovery | Updated existing app permission after user completed system authentication |
| Idle overlay windows | No overlays in AXWindows or the on-screen CGWindow list |
| Active overlays | Both are AXFloatingWindow at CGWindow layer 25; Safari retains focus |
| AltTab idle and active hint sessions | PASS: live thumbnail UI and AX readback contain no overlay entries |
| Dock icon hints | Safari icon receives a label at its center; above the Dock in CGWindow order |
| Dock input | Selecting Safari's Dock hint and using Right Shift opens its Dock menu, including the disposable QA window entry |
| Ordinary hint clicks | PASS on both displays, including a negative-coordinate target |
| Space at pointer | PASS: increments the existing pointer target's click counter |
| Exit and immediate cancellation | Native overlays disappear; ordinary typing resumes |

Runtime input used the disposable [Safari fixture](fixtures/macos-input-qa.html)
and a temporary CGEvent driver guarded by its exact foreground window title.
Final click counter was 3 (two hint selections and one Space click).
No AltTab preferences were changed. The mouse pointer remains visible.

Scope: local installed candidate, not a newly published GitHub release. Dock
auto-hide/magnification, left/right Dock placement, other task switchers, physical
Intel hardware, fullscreen/Spaces and Windows runtime were not accepted by these
checks. Moving the test window required waiting for the existing background AX
scan before reopening hints at its new position.
