# macOS Equivalent Implementation Plan

## 1) Goal and Scope

Build a macOS implementation that is behaviorally equivalent to the current Windows-first Screen Buoy core:

- Global hint navigation across monitors
- Hint generation from UI elements and blank regions
- Keyboard-driven interactions (move, click, double-click, drag, scroll)
- Overlay rendering and topmost behavior
- Tray, autostart, and global shortcut workflow

Out of scope for the first release:

- Perfect 1:1 control-type mapping with Windows UI Automation IDs
- Cross-platform refactor of frontend UX (unless required by backend capability differences)

## 2) Current Gap Summary

Current backend core relies on Windows-only APIs:

- UI element discovery: Windows UI Automation COM
- Window enumeration / z-order / occlusion helpers: Win32 APIs
- Global keyboard hook and key translation: Win32 low-level hook
- Mouse simulation: Win32 mouse APIs
- Overlay native adjustments: Win32 window style and DWM attributes

So macOS needs an equivalent native capability layer, not just build config changes.

## 3) Target Architecture

Introduce a platform abstraction layer in Rust:

- `platform/mod.rs` (trait definitions)
- `platform/windows/*` (existing logic moved with minimal behavior change)
- `platform/macos/*` (new implementation)

Suggested trait boundaries:

- `UiElementProvider`: enumerate actionable UI elements
- `WindowProvider`: enumerate windows, geometry, z-order, and occlusion data
- `InputHookProvider`: global keyboard capture and event normalization
- `MouseProvider`: pointer move/click/drag/scroll synthesis
- `OverlayProvider`: topmost, click-through, and window style hooks

Business modules (`hint`, `input::executor`, `element`) should depend on traits, not OS APIs.

## 4) macOS Equivalence Mapping

| Capability | Windows Source | macOS Candidate |
|---|---|---|
| UI element tree | UI Automation COM | Accessibility API (`AXUIElement`) |
| Window list + bounds | Win32 window APIs | CoreGraphics window list (`CGWindowList`) |
| Global keyboard capture | `WH_KEYBOARD_LL` hook | Event tap (`CGEventTap`) |
| Mouse simulation | Win32 mouse APIs | `CGEvent` synthesis |
| Overlay topmost/click-through | Win32 style + DWM | AppKit/NSWindow level + ignore mouse events |
| Multi-monitor geometry | Tauri monitor API | Tauri monitor API (keep) |
| Autostart | plugin + launcher | plugin launch agent (keep) |

## 5) Delivery Plan (Priority + Progress)

Progress baseline:

- P0 0/4, 0%
- P1 0/3, 0%
- P2 0/3, 0%

### P0 (Must Have): Run core flow on macOS

1. Platform abstraction extraction
   - Move existing Windows code behind platform traits
   - Keep current Windows behavior unchanged via `platform/windows`
2. macOS permission and bootstrap
   - Detect and surface required permissions:
     - Accessibility
     - Input Monitoring (if needed by capture strategy)
   - Add startup checks and user guidance
3. macOS input pipeline
   - Implement global key capture and key normalization
   - Preserve existing keybinding semantics (short/long press and combos)
4. macOS mouse actions
   - Implement move/click/double-click/drag/scroll
   - Keep timing and state behavior compatible with current executor logic

Acceptance for P0:

- App starts on macOS
- Global hotkey triggers hint mode
- Hold mode actions work on at least common desktop apps

### P1 (Should Have): UI discovery and hint quality

1. Window enumeration and ordering
   - Build window snapshot model for visible windows
   - Define z-order approximation strategy if exact order is unavailable
2. AX-based element discovery
   - Collect actionable elements and map to internal hint types
   - Add filtering and caching strategy similar to current behavior
3. Occlusion and candidate merge
   - Rebuild occlusion logic using window geometry and visibility rules
   - Merge UI-derived candidates with blank-area grid candidates

Acceptance for P1:

- Hint coverage and hit accuracy are usable on mainstream apps
- No severe lag during repeated hint activation

### P2 (Nice to Have): Stabilization and packaging

1. Overlay behavior tuning
   - Ensure non-intrusive topmost rendering and click-through semantics
2. Packaging and distribution readiness
   - macOS bundle metadata, signing/notarization preparation
3. Regression and compatibility matrix
   - Validate behavior across macOS versions and Intel/Apple Silicon

Acceptance for P2:

- Reproducible build artifacts
- Installation and launch path documented for end users

## 6) Risk Register

1. Permission-denied behavior
   - Trigger: user has not granted Accessibility/Input permissions
   - Impact: core capture or automation unavailable
   - Mitigation: startup diagnostics + guided permission flow + degraded-mode messaging

2. AX tree inconsistency across apps
   - Trigger: app-specific Accessibility tree quality differs
   - Impact: hint density and accuracy fluctuate
   - Mitigation: fallback grid hints, app-specific filters, telemetry-free local debug logs

3. Event tap instability or throttling
   - Trigger: high-frequency events or OS constraints
   - Impact: missed keyboard sequences
   - Mitigation: bounded processing pipeline, queue backpressure, watchdog restart

4. Window ordering mismatch
   - Trigger: macOS APIs expose partial ordering metadata
   - Impact: occlusion inference errors
   - Mitigation: heuristic ordering + confidence scoring + fallback to non-occlusion mode

## 7) Verification Plan

Minimum executable checks per milestone:

1. Build and startup
   - `npm run tauri dev` starts on macOS without backend panic
2. Permission checks
   - Startup clearly reports missing permission and recovery action
3. Hotkey and mode transitions
   - Global hotkey -> hints visible -> hold mode -> exit path
4. Input actions
   - Left/right/double click, drag, scroll, and cursor move verified
5. UI-derived hints
   - Hints appear on common controls in Finder, browser, and editor
6. Multi-monitor behavior
   - Overlay coverage and coordinate correctness across monitors

Definition of done:

- All P0 acceptance items passed
- P1 hint quality meets baseline in target app set
- No blocker-level crash in 30-minute exploratory session

## 8) Suggested Implementation Sequence

1. Extract platform traits and wrap current Windows code (no behavior change).
2. Add macOS permissions and key/mouse providers first (to unblock interactive loop).
3. Implement window + AX providers and integrate hint generation.
4. Tune occlusion and overlay behavior.
5. Add packaging readiness and compatibility validation.

This sequence minimizes risk by enabling an early end-to-end loop on macOS before deeper hint-quality optimizations.
