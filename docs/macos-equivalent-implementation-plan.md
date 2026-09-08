# macOS implementation status

Updated 2026-09-08 for version 1.2.1. Published downloads and release dates are listed on [GitHub Releases](https://github.com/alvinfunborn/screen-buoy/releases).

## Architecture and scope

The native macOS layer is implemented using conditionally compiled modules alongside the Windows modules. Shared hint generation, keyboard state, action dispatch, and settings remain in the existing modules; the originally proposed platform traits were not needed for this delivery.

| Area | Implementation | Verification boundary |
| --- | --- | --- |
| Global input | Dedicated CGEventTap run loop, modifier synchronization, repeat handling, recovery and readiness status | Requires Accessibility and a working event tap |
| Mouse actions | CGEvent move, left/right/middle/double click, drag and two-axis scroll | Cross-app interaction requires runtime checks |
| UI discovery | AX traversal, role mapping, window matching, cache, enhanced UI request | Coverage depends on each application's AX tree |
| Window occlusion | Preserve CGWindowList front-to-back order; exclude own overlays, zero-alpha windows and known display-sized system desktop surfaces | Regression tests cover floating, same-level, partially and fully covered windows |
| Coordinates | AX/CG window bounds use global logical points; monitor geometry converted from Tauri physical coordinates | Negative-coordinate matching has regression coverage; mixed-DPI runtime remains in the matrix |
| Overlay | Per-monitor transparent windows, click-through, topmost level, join-all-Spaces | Fullscreen/Spaces must be checked on target hardware |
| Permissions | Bilingual settings status, system-settings links, explicit grant actions, readiness gate | No unresponsive Hint mode while the hook is unavailable |
| Configuration | Standalone release app creates writable user config; sidecar config migrates once | Default parsing, migration, and preservation tests |
| Distribution | 1.2.1 package versions synchronized; universal architecture build and archive checks | Developer ID signing and notarization need credentials |
| Cursor | Mouse pointer stays visible on both platforms | Removed unused hide/show placeholders; no focus stealing to hide the pointer |

## Changes from 1.2.0

- Missing permissions are visible in the settings UI, including the exact running executable and recovery instructions. Permission checks themselves do not prompt; users explicitly choose the permission buttons.
- Hint activation requires Accessibility and an installed keyboard hook. Input Monitoring preflight is displayed separately; the installed active tap is the authority for keyboard availability.
- A floating window can no longer be incorrectly placed behind a normal window by layer sorting. Transparent Screen Buoy overlays cannot mask all target hints.
- Display-sized transparent Dock/Notification Center hosts are identified by executable path and bounds, so localized owner names do not hide every hint. Smaller panels retain occlusion.
- Modifier changes read the event flags, including left/right device flags, instead of a potentially stale session snapshot. Normal typing outside Hint mode is forwarded without decoding or logging its text.
- AX window matching no longer stretches every candidate to the target window's size before comparing geometry.
- The release app works without a sidecar config and preserves preferences across app replacement.
- PR/push CI checks both platforms; release CI verifies versions and both macOS architectures.

## Acceptance

See [macos-validation.md](macos-validation.md) for the current evidence and remaining runtime matrix. Build success is separate from interaction acceptance, and a universal executable does not prove Intel hardware compatibility.

Remaining external requirements:

- Developer ID Application certificate and Apple notarization credentials for a verified public download.
- Physical Intel Mac and older macOS versions for compatibility acceptance.
- Fullscreen/Spaces, permission revocation, multi-monitor hotplug, and a 30-minute exploration on the intended app set.

The previous P0/P1/P2 0% progress table described the Windows-only baseline and is obsolete. Core implementation exists; release acceptance is tracked by explicit checks instead of inferred percentages.
