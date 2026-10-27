# Runtime input processing

The runtime now borrows the verified pack's music-ID membership table from its
existing immutable catalog cache. Previously, every input transaction rebuilt a
`BTreeSet<String>`, cloning every track ID. Input, script, spawn, and map-transition
paths share the cached set. The public APIs that return owned sets retain their
behavior. Audio programs and content packs are unchanged.

## Production-controller measurement

The `input_timing` example constructs `VisibleShellController`, completes the
production new-game flow, and sends every input through `press_buttons`. It
checks that turning changes facing and walking changes the authoritative tile.
It opens no window and uses no audio device.

Run from this checkout with the completed external TUI pack:

```sh
cargo run --locked -p crystal-bevy --example input_timing -- \
  content-packs/text-tui.crystalpack 1024
```

The optional frame count must be a positive multiple of 64. Each scenario runs
eight batches and discards the first for warmup. The JSON output contains seven
batch averages in nanoseconds per input, their median, and the final game state.
Pack loading, new-game scripts, and the benchmark's snapshots are outside the
timed batches. Internal controller work remains inside them.

On macOS arm64 with Rust 1.99.0 and the workspace's optimized development profile
(`opt-level = 2`), the following comparison used unchanged runtime code from
`194c7b94` and the same benchmark. Runs were sequential in candidate, baseline,
baseline, candidate order, after all owned compilers and tests completed.

Times below are median batch averages, in microseconds per input:

| Scenario | Baseline 1 | Candidate 1 | Baseline 2 | Candidate 2 |
| --- | ---: | ---: | ---: | ---: |
| Idle | 155.876 | 156.383 | 156.721 | 155.261 |
| Turning | 198.142 | 192.808 | 199.548 | 190.774 |
| Walking | 205.347 | 201.584 | 207.286 | 198.439 |

Turning processing time fell 2.7% and 4.4%; walking fell 1.8% and 4.3%. Idle
results varied by less than 1%. Every run ended in the same map, position,
facing, and overworld frame for its scenario, with no pending script work.
Walking ended at `(7, 3)` in `PlayersHouse2F`; turning retained `(3, 3)` and
changed facing to right.

These music-membership measurements cover CPU controller processing in the
development build. The subsequent settling change below has a separate release
comparison. Dense-outdoor 3D throughput still requires separate work.

## Settling snapshots in release builds

The production controller's settling loop also used a full integrity snapshot
on every iteration. That validates the saved-state graph and computes two
whole-state checksums. Its branch decisions only read presentation fields, so
the loop now uses the existing `presentation_snapshot` API. Initialization,
explicit integrity snapshots, save validation, command checksums, and replay
boundaries retain their existing checks. The settle-limit diagnostic still
uses a full integrity snapshot.

Both binaries in this comparison already contain the music-membership cache.
The only production-code difference is the settling snapshot call. Rust 1.99.0,
the workspace release profile, the same external TUI pack, and the same
1024-input batches were used. Sequential runs were baseline, candidate,
candidate, baseline; all owned compilers and tests finished before timing.

Median batch averages, in microseconds per input:

| Scenario | Baseline 1 | Candidate 1 | Baseline 2 | Candidate 2 | Time reduction |
| --- | ---: | ---: | ---: | ---: | ---: |
| Idle | 121.461 | 14.123 | 122.290 | 14.010 | 88.4–88.5% |
| Turning | 148.414 | 41.689 | 149.591 | 40.661 | 71.9–72.8% |
| Walking | 153.613 | 46.940 | 155.535 | 47.287 | 69.4–69.6% |

The reported authoritative map, tile, facing, movement mode, and frame matched
across all four runs. No run ended with pending script work. These are
controller-processing measurements; rendered or display-presented frame rates
were not measured by this benchmark.

The release benchmark was built with:

```sh
cargo build --release --locked -p crystal-bevy --example input_timing \
  --target-dir target/desktop-release-production
target/desktop-release-production/release/examples/input_timing \
  content-packs/text-tui.crystalpack 1024
```

The release text client also built successfully:

```sh
cargo build --release --locked -p geothite \
  --target-dir target/desktop-release-production
```

Its stdio MCP session started CHRIS at `(3, 3)` in `PlayersHouse2F`, moved to
`(4, 3)` through the production movement tool, and exposed PACK and SAVE after
Start. Stderr remained byte-for-byte empty. No desktop app was replaced.

## Correctness checks

- Both music-membership regressions passed: cache sharing/owned-result isolation, and exact
  state/checksum parity between live and recorded input transactions.
- Nine production-controller regressions passed, including Mom/New Bark's
  gate and a battle turn with authoritative PP mutation. One existing controller
  test remained explicitly ignored.
- The new controller regression passed both before and after the settling
  change. It compares every presentation field against the full integrity view
  after movement and menu input, checks authoritative checksums, and proves
  malformed state is still rejected by integrity snapshots, checksumming, and
  saving without overwriting an existing save.
- The additional legacy synthetic-fixture replay probe fails identically on a
  fresh unchanged baseline and the candidate. Startup validation rejects its
  missing `EVENT_TEMPORARY_UNTIL_MAP_RELOAD_1` declaration before settling. This
  probe is not a passing replay result.
- Both terminal-client unit tests and the spawned stdio regression passed,
  including MCP movement/Start-menu ownership and empty stderr during dialogue.
- A fresh full runtime baseline had 202 passes, 52 failures, and one ignored
  test. The candidate had 204 passes, exactly the same 52 failing test names,
  and the same ignored test. Existing failures include stale fixture paths
  outside this checkout and gameplay/save fixture mismatches.
- The required runtime Clippy command stopped at the existing
  `clippy::bad_bit_mask` error in `crystal-core/src/multiplayer/mod.rs:1332`.
  Runtime-only linting with `--no-deps` completed with existing warnings and no
  diagnostics in the new cache module. Bevy's all-target check with `--no-deps`
  also encounters an existing `clippy::never_loop` error in
  `crystal-bevy/src/bevy_shell/title_menu.rs:8660`. The focused benchmark check
  completed with `--no-deps -- -A clippy::never_loop`, with no diagnostics in the
  new example. The subsequent all-target Bevy-only check also completed with
  that command-line allowance and no diagnostics in the new controller test.
  The full checks remain blocked by those existing errors.
- Workspace formatting checks still report existing differences. The new
  Rust files pass standalone formatting checks, and the patch passes
  `git diff --check`.

## Core 2D frame processing

The source compositor now caches immutable collision-derived priority and
foreground flags inside each loaded tileset's art. Previously, every subtile
in the fullscreen terrain margin looked up collision strings and classified
the same four quadrants again. Animation frames, textures, palette selection,
border rules, terrain dimensions, and gameplay collision remain unchanged.
Map callbacks still select their live block IDs before looking up these flags.

Two presentation copies were also removed. An active map whose blocks equal
the pack's blocks shares its original catalog entry. Live map writes retain
precedence over saved overrides, and neighboring maps retain their saved
writes. Fullscreen NPC expansion now checks for missing distant objects before
making a shared snapshot mutable. Expansion still preserves the gameplay
object roster and the last loaded pose of distant actors.

The existing `interactive_bevy_schedule_performance_benchmark` probe exercises
the production input, timer, 2D compositor, and UI systems in a headless Bevy
schedule. Its repository-root lookup was corrected for the current crate
layout. To rerun it with the external pack:

```sh
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
  cargo test --release --locked -p crystal-bevy --lib \
  interactive_bevy_schedule_performance_benchmark -- \
  --ignored --nocapture --test-threads=1
```

The comparison used macOS arm64, Rust 1.99.0, the workspace release profile,
and default features including fullscreen scaling. The baseline was
`9f40b434` with the same probe-root correction. Preserved baseline and candidate
test executables ran sequentially after all compilers and correctness tests
finished, in baseline/candidate/candidate/baseline/candidate/baseline/baseline/
candidate order. Each scenario contains 120 updates; initial app setup is
outside measurement. Values below are medians of four run averages, in
microseconds per CPU frame:

| Scenario | Baseline | Candidate | Time reduction | Baseline p95 | Candidate p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Idle | 241.265 | 211.765 | 12.2% | 1956.960 | 1761.815 |
| Held right | 455.760 | 425.475 | 6.6% | 2140.270 | 1933.520 |
| Walking and turning | 524.740 | 484.295 | 7.7% | 2163.730 | 1964.480 |

The p95 columns are medians of each run's p95. Individual candidate run
averages ranged from 199.080–241.290 microseconds idle, 419.950–430.770 with
right held, and 478.140–491.110 while walking and turning. A separate
snapshot-only/candidate/candidate/snapshot-only comparison also reduced
walking-and-turning time from 510.920 to 485.415 microseconds, supporting the
priority cache's contribution independently of the snapshot changes.

These initial measurements cover CPU engine processing. They exclude GPU
rendering, display presentation, and frame pacing; they do not establish a
displayed FPS increase. Neither baseline nor candidate missed the Game Boy frame
budget in these runs. The native GPU follow-up below measures the complete
offscreen renderer.

Verification for this change:

- Four new regressions passed: map sharing and write isolation; shared NPC
  expansion and idempotence; priority parity across every external-pack
  collision quadrant; and preservation of malformed-row errors.
- The final combined controller/rendering selection had 42 passes, three
  failures, and one existing ignored test. All nine active production-controller
  regressions passed. The three fullscreen failures match the preserved
  baseline exactly: slot-art retention, title-menu setup, and title-to-field
  handoff. Eight selected map/block rendering checks passed; a separate Cut
  probe fails on both binaries because this pack lacks `Script_Cut`.
- The full runtime suite had 205 passes, 52 failures, and one ignored test.
  Its failing names exactly match the earlier 204-pass runtime baseline;
  the new map regression accounts for the additional pass.
- The classic 2D library also passed a release check with
  `--no-default-features --features bevy-shell`.
- Required all-target Clippy encountered the existing
  `crystal-core/src/multiplayer/mod.rs:1332` bad-bit-mask error. Affected-crate
  all-target Clippy completed with `--no-deps -- -A clippy::never_loop`, with
  existing warnings and no diagnostics in the new modules. Workspace formatting
  still reports existing differences; the three new leaf modules pass standalone
  formatting checks, and `git diff --check` passes.

## Native source 2D CPU/GPU follow-up

The production sprite extractor now omits exactly zero-alpha tints. Its blended
pipeline does not write depth, so these draws contribute no pixels. Persistent
fade and poison entities still publish on their next nonzero frame, including
alpha `1/255`. Sprite synchronization also avoids assigning unchanged flip,
color, and visibility values. This prevents Bevy's `Changed<Sprite>` material
conversion from rebuilding unchanged presentation state every host frame.

Native source-view `Update` and `PostUpdate` schedules now execute on one thread.
A detailed CPU trace showed worker dispatch dominating these small systems.
The policy changes between frames, preserves deferred commands and system-local
state, and restores the multithreaded executor when 3D is enabled. The render
sub-app and its thread remain unchanged; browser scheduling is unchanged.

The audit runner now accepts `--view 2d`; its default remains 3D. Both modes
require actual GPU completion, verified warmup pixels, and authoritative
gameplay checkpoints. See [the audit runbook](../tools/PERFORMANCE_AUDIT.md) for
the reproducible sustained-walking recipe and command. All captures were
offscreen and opened no windows.

The initial cache-only candidate reduced the CPU microbenchmark above but did
not materially increase complete-renderer average throughput. A subsequent
baseline/candidate/candidate/baseline comparison of the zero-alpha and sprite
change guards reduced GPU time from approximately 0.513 to 0.336 ms (34.4%),
while completed-frame throughput improved only about 2%. The CPU trace then
motivated the native source-view scheduling policy.

The final comparison used Apple M5 / Metal, macOS arm64, Rust 1.99.0, the
workspace release profile, and identical audit features and native pack.
The baseline was `9f40b434` with the same two audit instrumentation files.
Both executables rendered 960×864 at noon. Four fresh-process captures ran
baseline/candidate/candidate/baseline, each after the quiet-host gate. The route
alternated between New Bark `(13, 6)` and `(11, 6)` for sixteen legs using normal
game timing. Measured host frames were unthrottled; simulation speed was not
increased. Values are averages of the two per-run means:

| Metric | Baseline | Final candidate | Improvement |
| --- | ---: | ---: | ---: |
| Main CPU time per frame | 0.99049 ms | 0.54395 ms | 45.1% less |
| Native GPU render-graph time | 0.51338 ms | 0.33391 ms | 35.0% less |
| Completed-frame interval | 1.16617 ms | 0.94917 ms | 18.6% less |
| Offscreen frame throughput | 857.5 FPS | 1053.6 FPS | 22.9% higher |

Baseline frame means ranged from 1.16364–1.16870 ms; candidate means ranged
from 0.94655–0.95179 ms. Baseline p99 intervals were 2.97408 and 3.02942 ms;
candidate p99 intervals were 2.17808 and 2.19683 ms. Each run passed all sixteen
authoritative waypoints, had 100% native GPU timing coverage, zero GPU/capture
errors, and no frame-budget misses. GPU time comes from correlated Metal
render-graph encoder intervals, not marker-only timing. The final candidate
also passed a separate calibration capture with the same route. Physical
display FPS, vsync behavior, and other hardware were not measured; display
refresh limits are unchanged.

Correctness verification includes:

- Matched `PlayersHouse2F (3, 3)` captures at noon were visually inspected and
  byte-identical before and after all changes. Their PNG SHA-256 is
  `cecc16aaea4938b4769f70b3258dc8d8c07c3cb8958b761d89524541163d8cfe`.
- The headless `source_sprite_regression` example checks 40 independent source
  textures and five overlay phases: zero, `1/255`, half, full, and zero again.
  Every phase passed both extraction-count and actual GPU-pixel assertions.
- A sprite synchronization regression verifies clean idle frames while walking,
  mirroring, fade, poison, and priority-background transitions still publish.
- The executor regression alternates execution policies and verifies command
  application and accumulated system-local state.
- Sixteen selected controller, sprite, fade, poison, and executor tests passed;
  one existing performance probe remains ignored. These include Mom's dialogue,
  the New Bark exit gate, production menus, warps, and an authoritative battle
  turn with PP consumption.
- A separate final-build source 2D `battle-turn` audit passed the production
  battle preparation, PP-consumption checkpoint, resulting dialogue, and
  verified warmup image. Its timings were excluded from the performance table.
  An initial capture overlapped compilation and was repeated on a quiet host.
  Both quiet baseline and candidate passed correctness but failed the 45 ms
  worst-frame gate: baseline had two misses with a 280.3 ms peak; candidate had
  one miss with a 297.0 ms peak. That first optimization left battle stalls
  unresolved. The battle-search follow-up below addresses this separate cost.
  The audit exits with status 3 for those budget failures.
- The audit analyzer's 55 Python tests passed, including 2D/3D baseline
  incompatibility and handling of source views without modeled terrain.
- The final native 3D indoor audit passed with external models and verified
  pixels. The final classic 2D all-target release check also passed with
  `--no-default-features --features bevy-shell`.
- Final all-target Clippy with `performance-audit,operation-trace` still stops
  at the existing multiplayer bad-bit-mask error described above. The affected
  crate completes with `--no-deps -- -A clippy::never_loop`, with no new module
  diagnostics. Workspace formatting remains blocked by existing differences;
  the five new Rust leaf modules and changed examples pass scoped formatting
  checks, and `git diff --check` passes.

One earlier Metal recording had a duplicated command-buffer identity. It failed
the auditor, was excluded, and was rerun successfully. Detailed CPU tracing was
used only to identify overhead, not to calculate the performance improvements.

## Battle search frame latency

The competitive planner used most of the long battle frame copying complete
battle states for damage previews. Live turns and previews now share borrowed
damage preparation. Ordinary accuracy and stat calculations also borrow their
battlers; transformations and type overrides retain their owned effective view.
Both damage endpoints share preparation, and AI predictions omit unused display
metadata.

Each decision owns bounded caches for pure damage calculations and identical
continuations. Damage equality includes every formula input, enforced by the
formula's typed input view, and the cache borrows immutable rules. Continuation
reuse requires complete combat-state and trainer-profile equality. The planner
retains all attacks, its eight first-turn samples and two continuation samples,
and its independent deterministic chance streams. The live RNG consumption order is unchanged.
Private simulations discard species TM/HM compatibility lists after constructing
the public threat set; live Pokemon and the content catalog retain those lists.

On the same Apple M5, Rust 1.99.0 release build and external native pack, eight
source 2D `battle-turn` captures passed the unchanged 45 ms worst-frame gate.
Their maximum intervals ranged from 27.496 to 28.875 ms. A subsequent capture
of the preserved executable before this battle fix still reached 280.430 ms.
All nine runs passed the authoritative PP-consumption and presentation checks.
The encounter retains normal randomness, so DVs and resulting damage vary;
deterministic regression tests separately compare prediction and search results.

These captures used `--view 2d --cpu-only --timings-only`, with no owned compiler
or competing audit running. They did not use `--quiet-host`: shared-workstation
GPU activity prevented that policy from settling. Actual offscreen Metal rendering
and readiness checks remained active; GPU timestamp queries were disabled.
The comparison measures game-loop frame intervals, not physical display refresh.

```sh
python3 tools/performance_audit.py \
  --binary target/runtime-fps/release/examples/performance_audit \
  --view 2d --cpu-only --timings-only --scenario battle-turn \
  --repeats 8 --output target/battle-audit
```

Battle correctness and lint verification:

- The core battle suite passed 763 tests, including borrowed/owned effective
  battler parity, cached/uncached prediction equality, and the live critical
  register write on a damage-validation error. Five unchanged source-oracle
  tests still fail because their external move-effect constants are absent.
- The decision-cache regression passed complete response-vector comparisons
  across all six trainer profiles, repeated choices, and compact/private versus
  original planning states. The existing competitive battle suite passed 16
  tests; its two explicitly ignored strength probes were left ignored. That
  suite was compiled directly against the same release dependency artifacts to
  avoid building unrelated exporter executables.
- Production controller battle/PP/dialogue and Mom/New Bark regressions passed.
  The real text-client MCP movement/Start and spawned stderr-clean regressions
  also passed with the completed external text pack.
- Required all-target Clippy remains blocked by the existing multiplayer
  bad-bit-mask error, absent radio-oracle fixtures, and legacy never-loop errors
  in the mutation protocol. Library-only checks complete with those existing
  lint allowances; the new search/view modules have no diagnostics. Workspace
  formatting still has pre-existing differences; new leaf modules and the patch
  pass scoped formatting and whitespace checks.

The browser release also optimizes `crystal-core` and `crystal-assets` for speed
with one codegen unit each, allowing their damage/search helpers to inline
without whole-program LTO. Eight codegen units still produced a 45.2 ms browser
frame-budget failure after speed optimization. Three consecutive final-build
captures passed the same 45 ms limit: maximum animation-frame intervals were
42.1, 40.8, and 43.9 ms; maximum callback durations were 41.9, 40.6, and 43.6 ms.

Those checks used headless Chromium with Metal on the same M5, source 2D, and
an external disposable field save. Normal Pack/SQUIRTBOTTLE input entered the
Route 36 battle; each measured first turn consumed TACKLE PP from 35 to 34,
retained the authoritative battle, and published the resulting dialogue without
JavaScript errors. No owned compiler or other browser audit ran during these
measurements. They measure this browser/runtime configuration, not physical
display refresh or performance on other devices.
A later visual-diagnostic run with console capture and extra view checks reached
46.6 ms; it was not part of that isolated timing series. These measurements do
not establish a universal upper bound.

Screenshot review also reproduced an existing intermittent 2D battle-rendering
defect on both the candidate and the previous public release: some LCD layers
can disappear while the logical HUD and dialogue remain present. Switching to
3D shows the complete battle; returning to 2D can reproduce the missing layers.
That separate rendering defect remains, and these captures do not establish
visual acceptance for 2D battles.

Distribution checks also found and fixed terminal portability issues: the
downloaded client now uses its built-in default theme and crossterm's portable
terminal poller. Continue releases completed bedroom decoration callbacks through
the shared production controller. Its new external-pack regression passes with
both the hosted and text packs, along with the existing snapshot-integrity,
Mom/New Bark, and authoritative battle regressions. Real curl/PTY checks pass on
Apple Silicon macOS and Linux x86_64, including Start, saving, a reinstall that
preserves the save byte for byte, and loaded-save MCP movement/Start. The
standard browser test command includes installer retention; all 129 tests pass.
Final Bevy all-target Clippy completes with `--no-deps -- -A clippy::never_loop`;
the workspace formatting check still reports existing differences.
