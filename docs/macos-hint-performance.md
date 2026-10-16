# macOS hint occlusion and performance

The macOS generator uses window rectangle intersections as a cheap first pass.
An on-screen element outside all intersections requires no native query. An
element inside an intersection is a candidate for a point hit test, even when
other window rectangles cover the whole target window.

Before showing overlays, AppKit's public
[`NSWindow.windowNumber(at:belowWindowWithWindowNumber:)`](https://developer.apple.com/documentation/appkit/nswindow/windownumber(at:belowwindowwithwindownumber:))
identifies the window that would receive a mouse-down at each suspect point.
It skips windows that ignore mouse events or are transparent at that point.
AX / CG coordinates are converted to AppKit points using the primary display's
logical height; negative coordinates on displays above/left are retained.

Results, including failed queries, are cached by global logical `(x, y)` for
one activation. A hidden element cannot reserve a duplicate coordinate ahead
of its visible owner. No hit-test result is reused after reactivation, window
movement or click-through changes. An unavailable result falls back to the
rectangle rule. This policy and its native module compile only on macOS;
Windows keeps its existing rectangle pruning, deduplication and hint flow.

## Scheduling

Planning and hint generation run on the existing async worker. Native queries
run on the AppKit main thread in sequential tasks of at most 64 points, stopping
after a cooperative 2 ms budget. The next task is queued only after the previous
one completes. One system query can exceed the budget; the actual longest task
is measured, rather than treating 2 ms as a guaranteed ceiling. Escape or a new
activation cancels an unfinished scan before it can publish stale hints.

## Metrics

At INFO logging level, each completed activation emits `[hint_perf macos]`:

| Field | Meaning |
| --- | --- |
| `activation` | Activation identifier, also invalidated on hide |
| `candidates`, `offscreen` | Cached UI elements belonging to enumerated windows, and candidates outside all monitors |
| `rect_fast`, `rect_suspect` | On-screen candidates passing rectangles directly, or requiring point checks; includes duplicate elements |
| `native_queries` | Actual native calls, once per unique suspect coordinate |
| `cache_hits` | Repeated suspect coordinates avoided by the cache |
| `failures`, `unresolved` | Calls with no receiver; unique coordinates without a usable result, including dispatch failures |
| `hints`, `grid_hints`, `control_hints` | Actual generated labels after visibility, deduplication and capacity checks |
| `total_ms` | From activation setup through both hint batches being emitted and stored; excludes background AX collection and frontend paint |
| `hit_test_ms` | Point-check wall time, including main-thread dispatch waits |
| `query_ms` | Sum of native query durations |
| `main_total_ms`, `main_max_ms`, `batches` | Total and longest occupancy of this change's main-thread query tasks, and completed task count |

To measure, build a release candidate, use INFO logging, keep the target browser
and a transparent overlay in place, then activate and dismiss hints several
times. Preserve user configuration and record executable path, desktop layout,
counts and timing for each completed activation. Verify both a visible composer
under a click-through host and a genuinely occluded control. Separately check
Escape during preparation and movement between activations. Unit tests and
cross-compilation do not establish these desktop runtime results.

## Validation recorded on 2026-10-08

| Check | Result |
| --- | --- |
| `cargo test --locked --lib` on Apple Silicon | PASS, 19 tests |
| `cargo check --locked` on macOS | PASS |
| `npm run check:version` | PASS, 1.2.2 |
| Release frontend and aarch64 macOS `.app` build | PASS |
| `cargo check --locked --target x86_64-pc-windows-msvc` | BLOCKED in Tauri's resource build: this Mac has no `llvm-rc`; Windows source compilation and runtime are not accepted |
| Desktop composer recovery and real performance measurements | PENDING: native desktop automation could not locate the running Edge / Activity Monitor windows; the browser page itself remained readable |

The candidate is in
`src-tauri/target/aarch64-apple-darwin/release/bundle/macos/screen-buoy.app`.
The installed `/Applications/screen-buoy.app` was not replaced. No measured
native query timings or desktop recovery result are claimed by the checks above.

## Follow-up acceptance

The user reported the hint visibility fix works in local use. The candidate was
installed in `/Applications/screen-buoy.app` during the October 14–16 iteration.
The table above records the earlier validation snapshot; no measured real-hint
query counts or timing benchmark has been collected for this release.
