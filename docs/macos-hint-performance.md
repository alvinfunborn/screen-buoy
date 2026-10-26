# Hint collection, live updates, and macOS occlusion

## Collection and cache

A single scheduler owns the visible-window list, per-window scan deadlines and
in-flight jobs. Two persistent workers perform native scans. The front normal
window has a reserved lane; other visible windows share the other lane in due
order. A floating window overlapping the front app no longer puts that app
behind a 60-second cache TTL. There is at most one in-flight scan per window,
even when it moves between lanes. Windows workers initialize their own MTA COM
apartment and create/drop UIA objects on that same thread.

Window topology is checked every 250 ms, including while hints are closed.
On macOS this reads CGWindow metadata, not the application's AX tree. Dock AX
bounds are collected by its scan worker; enumeration only reads those bounds.
The hotkey and mouse releases wake the scheduler immediately. Mouse hooks only
queue the click coordinates; native enumeration and AX work stay off the hook.

There is one UI-element cache per window. **A valid completed cache skips native
scanning**, including the foreground window while idle. `cache_ttl` controls this
reuse and the age of raw source updates used by visibility processing. A separate
filtered-coordinate cache supplies the initial display while a refresh is pending.
Recommended defaults are `collect_interval = 250` and `cache_ttl = 60000`.
The interval is a delay after scan completion, not a fixed-rate schedule;
a longer valid cache wins. The installed user's setting was changed live from
1000 to 250 ms after the scheduler measurements below; those measurements used
the earlier 1000 ms configuration.

Refreshes are targeted:

- Switching the front normal window refreshes the new front immediately.
- A click refreshes the front and the windows at that coordinate (including the
  normal window underneath a potentially transparent floating host).
- A new/moved/changed window invalidates only its own source cache. Closing a
  window drops only that entry. Reordering windows replays cached points for
  visibility checking without invalidating every AX tree.
- Opening hints refreshes the front immediately, keeps an already-running valid
  walk, and enables continuous updates of the current visible-window set.
- Clicks cancel superseded walks only for their target windows. Lane changes,
  removed windows and changed window identities also cancel obsolete work.

| Mode | Related/front window | Other windows |
| --- | --- | --- |
| Hints closed, valid completed cache | Skip until TTL expires | Skip until TTL expires |
| Switch/click while hints are closed | One immediate pass, then one coalesced pass after the transition | Keep cached results |
| Hints remain open | Front: 250 ms pause | Current visible-window set: 1 second pause |
| Failed scan | 1–60 second exponential retry backoff | Same |

Every pause starts **after scan completion**; slow native calls cannot consume
an interval and immediately loop again. These are scheduling pauses, not latency
guarantees. Explicit targeted input bypasses cache TTL and retry backoff. Once
hints close, active-mode polling stops. A switch/click queues at most one trailing
pass, due 250 ms after the latest interaction and at least 250 ms after scan
completion. Repeated input coalesces that pass and cannot restart a walk more
than every 250 ms; explicit hint activation bypasses that cooldown. A scan
starting after the transition consumes the trailing request itself. Config
changes are read during scheduling.

### Bounded interaction refresh measurement (2026-10-23)

The previous 1.5-second/100-ms interaction burst could repeat a small window
walk 15 times and a roughly 3,150-node browser walk four times after a click.
The bounded replacement removes that burst while keeping immediate hint
refresh and the existing 250-ms completion delay for active hints.

The installed replacement (`e5a6cb954335d8cb6aa8af6abe7c420c32cc8088d87ab502883d01b95c177ad0`)
was measured with Accessibility and the event tap working, normal `info`
logging, `collect_interval = 250`, and `cache_ttl = 60000`. Over 90 seconds
with hints closed, six 15-second CPU-time samples were **1.53%, 0.53%, 0.87%,
2.00%, 0.33%, and 0.67%**: **0.99% of one core on average**. RSS was
73.6–77.1 MiB. The observation includes a cache-expiry period; desktop topology
changed during most intervals, so this is an observed background workload,
not a controlled idle benchmark or a CPU ceiling.

Earlier 60-second observations of the previous implementation averaged 10.83%
and 6.20%. These were different real-use intervals and must not be reported as
a controlled before/after reduction. The attempted matched-click comparison
was interrupted by user interaction and is inconclusive. Info-level sampling
does not emit `[ui_scan]`, so a zero debug-scan count does not mean zero walks.
Raw samples, diagnostic logs, build/test output and the installation backup
record are under ignored `logs/investigations/cache-scheduler-20261023/`.

Validation: 48 Rust tests, four overlay tests, version consistency and release
app build passed. The new regressions cover bounded one-click refresh,
coalescing with immediate hint bypass, and consuming a trailing request when
the worker starts after the transition.

Only successful complete walks extend the background cache lifetime. Partial
snapshots and cancelled/failed walks cannot keep it fresh indefinitely.
Consumers share immutable vectors through `Arc`; no extra platform TTL cache
or per-lookup threads are needed. Source-cache cleanup does not remove a
coordinate shortcut from an already-open hint session.

During an AX walk, results can be published at 50 ms checkpoints. UIA does the
same during conversion after its native `FindAll` returns. A partial result
merges into the warm snapshot; only completion replaces it, so rescanning does
not temporarily hide the rest of the warm cache. Identical results keep their
revision and allocation. Completion refreshes their timestamp and wakes any
subscriber that had skipped an expired snapshot, without repainting unchanged
hints. A coalescing watch notification lets a slow subscriber consume the latest
snapshot without accumulating a queue of all intermediate trees.

## Hint session

The hotkey subscribes and captures the existing **filtered-coordinate cache**
before requesting a refresh. After resetting the overlays, it publishes the
grid and those cached on-screen coordinates without waiting for another window
enumeration, AX/UIA scan or native occlusion check. Raw, unfiltered source points
are never replayed. Background filtering also runs with hints closed, so the
first activation after prewarming can use this cache.

One visibility worker listens for source and window-layout changes. Every hint
activation also schedules a fresh visibility pass, even if the raw coordinates
have not changed. The worker processes all candidates, including coordinates
already displayed in the active session. A newly occluded point is removed from
the reusable cache, and a point removed from a source is pruned there too. New
accepted points are published progressively. Visibility results are independent
of hint-code capacity or whether the user has closed the overlay.

This keeps two intentionally different views: the current session only appends,
while the reusable cache adds and removes points. For example, if cached point
A is now covered and new point B is visible, the current session keeps A and
appends B; the next activation gets B without A after that check completes.
Until a recheck finishes, the prior filtered coordinates remain available for
immediate replay. Closed windows and replaced window owners are pruned from the
cache; unrelated layout changes do not flush all filtered points.

Offscreen points and duplicate coordinates are skipped, and code capacity is
respected. Codes use the current display mapping and start with zero movement
offset each session. An empty cold cache shows grids first and appends controls
as background filtering finishes.

A session owns its used coordinates and next unused code. New coordinates are
appended. Existing codes and coordinates are never reassigned, moved or deleted
by a scan; they remain screen-position shortcuts even if the underlying UI
changes. Explicit user movement still moves hints; later additions compensate
for that offset to stay aligned with their discovered screen positions. Once
codes run out, the session stops appending rather than recycling existing codes.

Visibility work is split into groups of at most 128 candidates, with foreground
updates considered between groups. Progressive replacement preserves traversal
progress so repeated snapshots cannot starve controls later in the tree.
Offscreen coordinates are filtered before cloning or entering this queue;
already displayed coordinates still need checking for the next session's cache.
The prefilter uses the same logical display bounds as hint
rendering on macOS, including negative coordinates and mixed display scales;
Windows keeps its physical-coordinate bounds. Window intersections and display
bounds are shared across batches until the window layout changes.

Completed work retains its visited coordinates for later progressive snapshots
of the same native walk. A changed snapshot from a new walk permits rejected
points to be checked again after the existing queue has drained. An unfinished
queue keeps its traversal progress across walks so frequent scans cannot keep
restarting at element zero. A topology change clears both pending and completed
work and reconsiders all candidates. This state is bounded by the current visible
windows and survives overlay closure. The worker sleeps when no source/layout
change or new activation needs work. It pauses 20 ms between candidate batches
with hints closed and 1 ms with hints open; these are cooperative pauses, not a
hard native-call time limit. Identical filtered results do not republish cache
vectors or wake the frontend.

Storage and overlay publication are serialized with session transitions. Every
begin/append/hide/filter/move event carries a session epoch, and deferred exits
retain their original epoch. Escape and rapid reopening reject late results
both in Rust and in the overlay. Keyboard filtering uses the original full code
and full prefix, including for late additions. No first-character freeze or
AX-element identity tracking is involved.

## macOS occlusion

Rectangle intersections identify suspect points. An on-screen element outside
all intersections needs no native query. For a suspect point, the existing
AppKit `NSWindow.windowNumber(at:belowWindowWithWindowNumber:)` check identifies
the mouse receiver, including transparent/click-through window behavior. Own
production overlays ignore mouse events. AX/CG coordinates are converted using
the primary display's logical height, preserving negative display coordinates.

Native queries run on the AppKit main thread in sequential tasks of at most 64
points, stopping after a cooperative 2 ms budget. Query results are scoped to
that candidate batch. Initial filtered-cache replay avoids repeating these checks
on the activation path; raw native hit-test results are not shared. Failed queries use
the conservative rectangle rule. A rejected point does not reserve its
coordinate before a visible owner can claim it. Windows uses rectangle pruning.

Cancellation is checked between AX nodes and UIA conversions. A native IPC call
already in progress cannot be forcibly interrupted; UIA `FindAll` must return
before progressive conversion can start. Very slow or unresponsive apps can
still delay recognition. The worker bound prevents unbounded scan threads,
but does not establish a fixed worst-case recognition time.

## Metrics and validation

`[ui_scan]` logs window, lane, scan duration and cancellation at DEBUG level.
`[hint_update]` logs actual additions with session, source window/revision,
added/total counts, first-control flag, elapsed activation time and snapshot age
at INFO level. The first-control metric excludes grids but may describe a warm
control; it does **not** prove the user's newly appeared target is ready.
`[hint_cache]` reports the initial cached-control count, source count and elapsed
backend activation time, including when the cache is empty. It does not include
frontend paint. `[hint_warm]` in older logs describes the previous verified-only
replay path that required an unchanged desktop before reuse.
`[visibility_perf macos]` reports one background worker batch, including epoch,
whether hints were closed, accepted-point count, native query time and main-thread
occupancy. `elapsed_ms` is batch processing time, not activation or paint latency.
Older `[hint_perf macos]` records described visibility on the activation path.

### Hint queue work reduction (2026-10-23)

A sample of 47 real hint activations on the preceding bounded-refresh build
contained 6,615 visibility batches. Of 681,155 candidate encounters, 551,789
(81%) were offscreen; 4,402 batches contained only offscreen points. These points
already skipped native hit testing, but still caused queue allocation, generator
setup and repeated window-intersection computation. Draining a window's queue
also discarded its visited set, allowing a later partial snapshot of the same
walk to repeat rejected on-screen checks.

An isolated benchmark extracts the actual before/after queue implementations and
the macOS `UIElement` type. Both consume the same ten cumulative snapshots of one
6,000-point walk, with 81% of points offscreen and all remaining unassigned points
rejected by the simulated visibility stage. Snapshot/window/display types are
simplified. The benchmark excludes AX, native hit tests, layout construction,
IPC and rendering; its times are **queue processing only**, not application CPU
or visible activation latency.

| Same synthetic input | Before | After |
| --- | --- | --- |
| Nonempty candidate batches | 263 | 10 |
| Candidates handed to visibility | 33,000 | 1,140 |
| On-screen candidate checks, including repeats | 6,270 | 1,140 |
| Median queue processing per ten snapshots | 1.84 ms | 0.17 ms |

With roughly half of on-screen points already restored, candidates dropped from
30,030 to 600 and queue time from 2.48 to 0.19 ms. These deliberately repeat-heavy
fixtures demonstrate eliminated work, not a predicted percentage reduction in
real-use CPU. A controlled desktop before/after replay was not obtained because
the desktop was in concurrent use.

Validation: 52 Rust tests, four overlay tests, version consistency, frontend
build and arm64 release app build passed. New regressions cover offscreen tails,
warm-restored coordinates, same-walk reuse versus new-walk/layout invalidation,
unfinished traversal across new walks, and negative/mixed-scale display bounds.
Raw source snapshots, benchmark code/results and build/test output are under
ignored `logs/investigations/cache-scheduler-20261023/` with the `roi` prefix.

The replacement was installed at 15:42 with executable SHA-256
`f2ca45ad64db6b8f725c0bff3572de709d8a353c0f735266786cb786f4c359df`;
the bundle signature and installed hash were verified. The previous app and
configuration were backed up. Configuration bytes were unchanged: 250 ms
completion delay, 60-second cache TTL and `info` logging. macOS required Touch ID
to refresh the previous Accessibility grant after the ad-hoc signature changed;
the UI subsequently reported ready and the event tap installed at 15:44:07.

A subsequent 60-second observation with no hint publications used **1.63% of
one core on average**, with 15-second samples of 3.26%, 1.60%, 1.00% and 0.67%.
RSS ranged from 47.6 to 52.1 MiB. Two intervals had different start/end window
topologies. This is an observed real-desktop background interval, not a matched
before/after workload; these queue changes do not alter background scheduling.
No errors or event-tap disablements were logged after authorization recovered.

A further 60-second real-use interval containing one hint activation averaged
**2.47% of one core**: 3.67%, 5.20%, 0.53% and 0.47% per 15 seconds. The second
interval contained 47 hint updates; the final two contained none. All four
start/end topology pairs matched, but intermediate activity is not excluded.
Two actual activations and completed hint clicks were observed in total, with
9,876 visibility candidates and zero offscreen candidates. Source:
`after-roi-manual.json` and `roi-validation-summary.json` in the same ignored
investigation directory. This is not a continuous-hint-open CPU benchmark.

At 15:48:38 a real shortcut activation produced 54 visibility batches containing
3,497 candidates and **zero offscreen candidates**; it appended controls and
completed a hint-selected click. The first backend control publication was
1.0 ms, which is not the appearance time of a newly introduced target. The
automated app-targeted key attempt had not produced a global shortcut event.
The browser rejected navigation to the local `file://` fixture under its URL
policy, so delayed-target paint latency was not accepted in this run.

Automated regressions exercise a full idle minute with no repeated native dispatch,
targeted switch/click invalidation, hint exit returning to cache reuse, retry
backoff, completion-based pauses, hotkey deadline bypass, stale scan rejection,
foreground lane isolation, source-cache pruning, partial warm-data retention,
unchanged snapshot sharing, stable append-only allocation, progressive traversal,
late-prefix filtering, and old-session event rejection. Run:

- `cargo test --locked --lib` in `src-tauri`
- `npm run test:overlay`
- `npm run build` and `npm run check:version`

For desktop acceptance, record the executable/config and the **new target's**
coordinate/code, then measure its appearance after (1) switching windows/Spaces,
(2) clicking to navigate, and (3) a delayed asynchronous page update while hints
remain open. Repeat with a typed prefix and with Escape/reopen during a slow scan.
Check idle CPU, negative/mixed-scale displays, Dock icons, a transparent host and
a truly occluded control. Capture target publication and frontend paint separately;
backend logs alone do not measure visible end-to-end latency. The proposed
200 ms P95 goal has not been established by unit tests or compilation.

## Validation recorded on 2026-10-23, cache-aware scheduler

The follow-up restores successful-cache scan skipping and adds the warm display
path. Rust regressions: 45 passed; overlay tests: 4 passed. The arm64 release
app build and ad-hoc signature verification passed. Installed defaults are
`collect_interval = 1000`, `cache_ttl = 60000`, logging `info`.

Before the warm display follow-up, the scheduler-only installed candidate was
sampled for 120 seconds, covering cache expiry. Process CPU used a mean of
3.78% of one core, compared with 18.08% over the preceding candidate's 60-second
sample. Fifteen-second samples ranged from 0.47% to 11.86%; 6/8 sample pairs
had identical starting/ending topology, and no hint additions occurred in the
sample window. The prior legacy comparison was approximately 5–6.5% of one core.
These are separate intervals on the real desktop, not an identical workload
replay; endpoint topology equality cannot exclude intermediate activity.
Evidence: ignored `logs/investigations/cache-scheduler-20261023/`.

Repeated hint sessions on the scheduler-only build exposed a separate display
delay: the first cached controls reached backend publication in about 1–4 ms,
but many remaining cached points were republished in 128-point batches after
other windows' native checks, extending additions to roughly 0.4–0.8 seconds.
The warm display path addresses that delay without increasing idle scans.

## Validation recorded on 2026-10-23, initial live-update candidate

Branch: `perf/live-hint-collection`, version 1.2.3.

| Check | Result |
| --- | --- |
| `cargo test --locked --lib` on Apple Silicon | PASS, 35 tests |
| `npm run test:overlay` | PASS, 4 tests |
| `npm run check:version` | PASS, 1.2.3 |
| `npm run build` | PASS; existing Vite chunk-size warning remains |
| `npm run tauri build -- --bundles app` | PASS, arm64 macOS app |
| Local ad-hoc bundle signing and `codesign --verify --deep --strict` | PASS after signing the generated bundle with `codesign --force --sign -` |
| `cargo check --locked --target x86_64-pc-windows-msvc` | BLOCKED in Tauri's resource build: missing `llvm-rc`; Windows source compilation and runtime are not accepted |
| Switching windows/Spaces, click-to-new-target latency, idle CPU | NOT MEASURED on this candidate |

Current candidate:
`src-tauri/target/release/bundle/macos/screen-buoy.app`.
The candidate was installed to `/Applications/screen-buoy.app` and relaunched on
October 23 at the user's request. Installed and candidate executable SHA-256
match, and the installed bundle signature verifies. The former app and config
were backed up under the user configuration directory's `backups` folder.
The installed user's background interval is now 250 ms, coordinate cache reuse
is 3000 ms, and logging is `info`; other preferences were preserved.

The new ad-hoc binary has a different code hash. Startup initially reported
Accessibility unavailable. After macOS Touch ID authentication, the existing
Screen Buoy grant was refreshed in System Settings; the switch is enabled and
the application log confirms `CGEventTap installed on dedicated thread`.
Real hint sessions have emitted incremental `[hint_update]` records. These are
backend observations, not a target-specific frontend paint latency benchmark.
Installation evidence is in ignored `logs/installations/20261023-112855/`.

The automated checks establish scheduling, cache and append behavior; they do
not establish native recognition or paint latency. The older results below
describe earlier candidates only.

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
