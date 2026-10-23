# Gameplay CPU/GPU audit

The auditor runs the production graphical shell offscreen, using the same
accepted keyboard snapshot and input-owner dispatcher as normal native play.
It does not implement another gameplay dispatcher. Each scenario starts a
fresh disposable session and cannot write a player save. Game content stays
in the external, ignored pack.

Build once from the repository root with the pinned Rust toolchain:

```sh
CARGO_TARGET_DIR=target/performance-audit cargo build --release --locked \
  -p crystal-bevy --example performance_audit \
  --features performance-audit,operation-trace
```

Run the scenario suite against the local native pack:

```sh
python3 tools/performance_audit.py \
  --binary target/performance-audit/release/examples/performance_audit \
  --pack content-packs/core-modular.crystalpack \
  --output .local/performance-audit/baseline --repeats 3 --timings-only
```

Open `.local/performance-audit/baseline/index.html`. It is a standalone offline
report with scenario/repeat selection, frame timings, slow-frame inspection,
controller versus render-settling latency, CPU operation attribution with detailed
tracing, native GPU stage waterfalls and operation quantiles, optimization leads, repeat ranges, and
baseline regressions. A sampled presentation-state timeline shows terrain/actor
pending flags, warp fade phases, dialogue/battle state and final verification on
the CPU observation clock. Its boundaries have frame-level precision; a cleared
pending flag alone does not prove that the destination is ready. Hover a frame to follow its source ID into the detailed
CPU thread waterfall. For a full CPU waterfall, re-run a selected scenario without
`--timings-only` into a fresh output directory
(`--scenario outdoor-connection --repeats 1`).
Raw traces also open in Perfetto. Trace readers stream large captures; the
detailed HTML includes the measured window rather than retaining startup spans.

The checked-in recipes cover outdoor/indoor idle and movement, land and Surf
movement, house exit/entry, outdoor map connections, a region boundary, ordinary
scripted battle entry, battle idle, and an actual battle move. Each movement/transition step requires the
specified authoritative map/tile/mode and a settled renderer. The battle move
requires actual PP consumption and resulting battle dialogue, then continues
ordinary A presses for five seconds to exercise the attack animation and result
pages. Depending on production damage and fainting, those pages can return to
commands or leave battle; this is an animated action sample, not a promise that
both battlers survive. Prepared battle
scenarios render the real Route36 field first, enter the Weird Tree interaction
with ordinary A/Yes, then wait for the production command menu. Battle idle and
turn use a legal level-30 Cyndaquil / Swift fixture against the source-scripted
Sudowoodo; battle entry retains the ordinary field-review lead. Encounter level,
accuracy, damage and fainting remain production
behavior; captured HP and phase identify those branches. A blocked route,
unexpected encounter, script or wrong pack fails with the last observed state; it is never accepted as a
fast result. Surf preparation uses the existing legal HM/badge developer
fixture, then mounts through ordinary A/Yes before warmup and measurement.
Separate `land-to-water` and `water-to-land` recipes time the mount and dismount
themselves and require the expected map, tile, mode and completed rendering.
The performance-only Surf setup adds a legal level-100 lead and uses ordinary
Max Repel through validated mutations to exclude random level-20/25 encounters.
The source encounter table and RNG remain unchanged. Field movement assertions
also require that no battle is active.

Filter a run with repeated `--scenario NAME` arguments. Output directories must
be empty. Each repeat runs in a separate process; repeats cycle over the suite
rather than collecting all repeats of one scenario consecutively. Production
RNG and autonomous actors remain active: recipes repeat the intended action
and assert its result, rather than claiming a bit-for-bit gameplay replay.
Directions are held until their assertion matches; ordinary buttons pulse at
5 Hz with a 50% duty cycle on the wall clock. Response checkpoints time that
requested input sequence, including its cadence. A dialogue advance can therefore
shift by a 200 ms pulse period; this is not a single physical-key latency test.
Changes to Rust readiness, input cadence or GPU boundaries require a new
`measurement_revision` and fresh captures. Offline analyzer changes can instead
reprocess retained raw captures, with their source hashes recorded.

Measurement revision 9 uses `continuous_movement: true` for directional field
traversals. These recipes immediately chain the next direction at the requested
authoritative tile, preserving fractional movement, including reversals. They do not wait for terrain or
GPU settlement between legs. Their checkpoints record `renderer_waited: false`
and zero `render_settle_ms`; the analyzer labels them as movement legs and
excludes their travel durations from response-latency comparisons. Ordinary
checkpoints retain the post-result GPU verification requirement. Rebuild both
comparison binaries with the same measurement revision.

Use `--require-after-continuity` with `performance_audit_pairs.py` when validating
seamless movement. For `-continuous` scenarios it stops further collection with
exit status 3 if the candidate loses 3D, has unknown camera observations, moves
the camera while the player remains fixed, or records a presentation interval
above 45 ms. It preserves the failed application outcome separately
from recorder failures. An unavailable baseline remains a measured outcome;
its raw CPU interval cannot be presented as a fast rendered crossing.

For narrow native ownership diagnostics, combine `CRYSTAL_LOADING_TRACE` with
`CRYSTAL_LOADING_TRACE_PREFIX=world.`. This records window submission, readiness,
promotion, retention and retirement without installing the detailed render-graph
barriers used by a full loading trace. Such diagnostics remain separate from
primary timing captures.

The acceptance ceiling is 45 ms for the worst measured frame interval. Normal
frames should fit within 16.67 ms for 60 Hz presentation; 45 ms is a maximum
hitch threshold, not the target frame time. A good p95 or average cannot excuse
a single interval above that ceiling. Paired
collection defaults to `--max-frame-ms 45` for every candidate scenario and
stops with exit status 3 on a breach, preserving `frame-budget-failure.json`
and the raw run. Slow baseline outcomes remain available for comparison.
Browser analysis reports the same ceiling in `frame_budget`. These checks
use observed frame intervals and do not claim physical display latency.
The standalone native runner also exits with status 3 on a ceiling breach.
Use `browser_world_stream.py OUTPUT --require-frame-budget` to enforce it for
a browser candidate; baseline analysis can retain a slower valid outcome.

For static PNG/SVG graphs and a CSV of the plotted numbers, use the optional
matplotlib exporter (`python3 -m pip install matplotlib` in your plotting environment):

```sh
python3 tools/performance_audit_charts.py \
  .local/performance-audit/baseline/summary.json \
  --output .local/performance-audit/baseline/graphs
```

Connected-map continuity also has dedicated held-movement recipes:
`outdoor-connection-continuous`, `region-boundary-continuous`, and
`north-south-connection-continuous`. They keep walking several cells into the
destination before reversing, so map publication alone cannot satisfy the
movement assertion. Fresh raw captures include simulation ticks, remaining
tile-tween ticks, rendered player feet, the actual active world camera transform,
and reciprocal source-map translations. A nonzero free-movement offset alone
does not indicate whether the player is moving.

Plot one matched pair's raw presentation samples with:

```sh
python3 tools/performance_audit_motion.py \
  --before .local/performance-audit/before/outdoor-connection-continuous-1/audit.json \
  --after .local/performance-audit/after/outdoor-connection-continuous-1/audit.json \
  --output .local/performance-audit/motion
```

The exporter writes PNG, SVG, CSV and crossing receipts. It translates coordinates
only when the production source regions prove the reciprocal placement; door
warps start separate segments. Samples describe CPU-side presentation positions,
not physical display latency. Camera plots use the actual 3D camera's ground-plane
position, not the clamped classic viewport center.

Motion graphs also show camera height. Each analyzed run records
`stationary_camera`: observed adjacent samples with a fixed player position,
maximum horizontal/vertical camera corrections, and source-frame IDs for
corrections larger than 0.0001 gameplay cells. No qualifying observations stays
unknown. This exposes grid-alignment or late terrain corrections that a plot
of ground-plane motion would miss; it does not infer input-to-display latency.

Route checkpoints include travel to and beyond the crossing. Measure the
crossing itself separately; a one-second route is not a one-second boundary
stall. The boundary exporter accepts matched, successful held-movement runs:

```sh
python3 tools/performance_audit_boundary.py \
  --before .local/performance-audit/paired/before/summary.json \
  --after .local/performance-audit/paired/after/summary.json \
  --output .local/performance-audit/paired/boundaries
```

It writes PNG/SVG graphs and JSON/CSV receipts for every crossing frame, its
main-app wall time, the next observed player movement, and remaining landing
ticks. Only continuous recipes qualify: stopping an input at the boundary
would mix the benchmark's checkpoint settling with production movement.
Missing subsequent movement remains unknown. These CPU presentation samples
and frame-start intervals do not measure physical display latency.

Also inspect the entire journey. A renderer can cross a seam successfully and
exhaust its resident terrain a second later. Every analyzed run includes
`world_residency`: observed unavailable milliseconds, frame counts, episodes,
and unknown final intervals. Missing observations remain unknown; authored
battle and warp presentation still requires scenario-specific interpretation.
Plot the raw captures, including unavailable periods away from seams, with:

```sh
python3 tools/performance_audit_residency.py \
  --before .local/performance-audit/before/run/audit.json \
  --after .local/performance-audit/after/run/audit.json \
  --output .local/performance-audit/residency
```

The PNG/SVG graphs mark unavailable 3D periods in red and logical map changes
with dashed lines. JSON preserves each episode and any censored interval.
These are CPU observations of renderer availability, not display latency.
Diagnostic runs with tracing or readbacks must remain separate from primary
timing comparisons.

For a separate visual inspection run, set `CRYSTAL_AUDIT_FRAME_CAPTURE=1` when
launching the Rust example directly. Each measured simulation tick requests one
PNG in `motion-frames/`; filenames contain request source-frame IDs and the raw
report records every readback completion or failure. Readbacks perturb timing.
The benchmark analyzer explicitly rejects these captures as timing evidence.
Leave that variable unset for all CPU/GPU comparisons. Captures remain offscreen
and require the usual headless/image environment settings.

The response-latency chart excludes duration-only sampling steps such as the
battle animation observation window. It retains authored dialogue and animation
in actual response checkpoints.
Add `--baseline .local/performance-audit/baseline/summary.json` when exporting a
candidate to include before/after graphs and `comparison.csv`. Incompatible or
failed captures are refused instead of plotted as performance improvements.
Native graphs show both the GPU activity envelope and active-time union. The
envelope can include submission gaps or work from other source frames; the
union counts the correlated activity itself. They are separate distributions,
so subtracting their p95 values does not produce a p95 gap duration.

Transition percentiles also depend on how many source and destination frames
were sampled. Export a separate map/mode/readiness breakdown when investigating
an aggregate change:

```sh
python3 tools/performance_audit_scenes.py \
  .local/performance-audit/candidate/summary.json \
  --baseline .local/performance-audit/baseline/summary.json \
  --output .local/performance-audit/candidate/scene-costs.csv
```

The CSV keeps population sizes and missing groups visible. It suppresses
percentage comparisons unless every repeat contains at least 30 samples of
that metric in the matching group. Renderer readiness does not imply idle
gameplay. This diagnostic supplements the original scenario gates.
`natural_offset_nonzero` records whether the player's natural-movement offset
from its authoritative tile center is nonzero. It can remain nonzero at rest,
so it does not classify movement or animation. Missing offset data stays
unavailable.

## Compare changes

After building the changed executable, preserve the first report and run:

```sh
python3 tools/performance_audit.py \
  --binary target/performance-audit/release/examples/performance_audit \
  --output .local/performance-audit/candidate --repeats 3 --timings-only \
  --baseline .local/performance-audit/baseline/summary.json
```

The gate compares median **per-run p95** durations for frame intervals,
main-app CPU work and GPU envelopes, plus median checkpoint latency. Defaults
require both a 10% increase and a 0.2 ms increase to flag a regression; use
`--threshold-percent` and `--threshold-ms` to set project-specific budgets.
These are practical gates, not statistical significance claims. Inspect the
repeat range before acting on small changes; increase repeats for noisy runs.
The report warns when the range of valid repeat p95 values exceeds 20% of their
median. This is a noise indicator, not a rejection threshold or a confidence
interval. For a claimed improvement near the observed spread, alternate frozen
baseline and candidate binaries in close pairs with the same configuration.

The paired collector alternates `before → after` and `after → before` on
successive rounds. Freeze both executables first so later builds cannot replace
them during collection. For example:

```sh
python3 tools/performance_audit_pairs.py \
  --before .local/performance-audit/bin/baseline \
  --after .local/performance-audit/bin/candidate \
  --output .local/performance-audit/paired --pairs 4 \
  --scenario region-boundary --scenario indoor-floor-transition
```

Each capture still uses a fresh process and the full correctness/coverage gates.
Its child report is available immediately, and stdout prints its p95 timings.
Aggregate waterfall reports are checkpointed after each round and finalized at
the end, avoiding repeated serialization of the entire growing GPU history.
Detailed CPU spans are filtered by default; add `--detailed-cpu` when investigating
operations, or `--cpu-only` for a control without native GPU profiling. Open
`paired/after/index.html` for the comparison. Export it with the chart tool using
`paired/after/summary.json` and `--baseline paired/before/summary.json`.

The collector also exports `paired/paired-effects/effects.csv` and `pairs.csv`.
These show differences within each adjacent pair, their median and range, and
how many pairs improved or slowed. Common drift between rounds can make the
median paired effect differ from the difference of the two aggregate medians.
The graph exporter adds `paired-effects-*.png` / `.svg` plots for paired inputs:
each point is a pair, color identifies run order, and the white tick is the
median paired change.
Inspect both; the supplemental view does not replace the primary gate or claim
statistical significance. Unavailable metrics and duration-only sampling steps
are omitted. Export this view for an existing completed paired cohort with:

```sh
python3 tools/performance_audit_pair_effects.py \
  .local/performance-audit/paired/after/summary.json \
  --baseline .local/performance-audit/paired/before/summary.json \
  --output .local/performance-audit/paired/paired-effects
```

On an Apple Silicon Mac, add `--quiet-host` to either collector for repeatable
workstation captures. Before each launch it requires ten consecutive seconds
with AGX GPU utilization at most 15%, estimated host CPU utilization at most 30%,
and no recognized compiler, game-test renderer, or headless Chrome job. It waits
without stopping other processes. During the capture it checks for those jobs
every half second and fails a contaminated capture while retaining its data.
The report records the policy, prelaunch samples and detected jobs. Missing
telemetry or a failed monitor is an error, rather than evidence of an idle host.
These are measurement preconditions, not limits on game resources or trace size.
The CPU estimate uses `ps` process percentages divided by logical CPU count;
this guard is best-effort and does not provide OS isolation, lock device clocks,
certify thermal state, or detect every possible source of contention. Keep other workloads stopped as well. Run an
unchanged-binary A/A control when assessing small differences.

Exit status is 0 for valid runs within thresholds, 1 for a failed scenario or
capture, and 2 for regressions or an incompatible baseline. Configuration,
scenario, pack hash/identity, viewport, clock, instrumentation and hardware
must match. On macOS, AC/battery source is recorded before and after each
capture; a mismatch fails the run. Binary hashes and Git revision/dirty state are recorded as
provenance, while changed binaries are allowed for intentional A/B comparisons.
The Git fields identify the collection checkout; they do not prove which source
built a previously frozen executable. Retain its build receipt or source snapshot
alongside the executable hashes, especially in a checkout with concurrent edits.
The runner explicitly selects `assets/open-models` by default; use `--model-root`
for a frozen artwork copy. Its recursive filename/content hash is a compatibility
key, since those native assets live outside the game pack. Keep that directory
immutable during collection. Older reports with no artwork hash cannot certify
that aspect of a comparison.
Executable, pack and artwork hashes are checked again after each capture;
changing an input during collection fails the run while preserving its data.
A missing GPU is explicitly unavailable, never a zero-millisecond result.
Supported GPU captures with invalid measured timestamps/identities or less than
99% source-frame coverage fail. Missing native boundaries discard the whole
source-frame group and count as correlation drops; those frames never enter GPU
quantiles. CPU frames remain in the interval/CPU distribution, and coverage and
drops remain visible. Invalid warmup groups never donate work to measured frames; raw
warmup diagnostics remain visible separately.
The native profiler version and backend are part of baseline compatibility.

After updating the offline analyzer or dashboard, refresh retained captures
without rerunning the game:

```sh
python3 tools/performance_audit_reanalyze.py .local/performance-audit/baseline
python3 tools/performance_audit_reanalyze.py .local/performance-audit/candidate \
  --baseline .local/performance-audit/baseline/summary.json
```

Wait for capture collection to finish before refreshing its report. The first
summary is preserved as `summary-before-reanalysis.json`. Reanalysis retains
process, power and configuration failures; it cannot turn a missing native GPU
import into a successful CPU-only capture. Reports record the analysis revision
and SHA-256 hashes of the analysis sources so a comparison can be reproduced.
Reanalysis also retains the original collection-source hashes separately.
Captures with checkpoints from measurement revisions older than 7 fail the
readiness gate: those versions could reuse an earlier GPU completion. Their
frame statistics remain inspectable, but their checkpoint latency requires a
fresh capture rather than offline repair.

Do not compile, run another game capture or run GPU-heavy applications while
collecting baselines. Keep power mode, charger state and thermal conditions
consistent. The runner clears `CRYSTAL_*`, `WGPU_*` and `BEVY_*` overrides and
sets offscreen image capture explicitly. It never opens a native game window.
It records a 960×864 target, a fixed noon clock and the production real-time
simulation cadence; it does not speed up the game simulation.

## What the measurements mean

- **Main CPU**: wall duration from the main app's First schedule to Last. This
  includes blocking and the observer overhead; the render sub-app and extraction
  are outside this duration. Detailed CPU traces include those operations too.
  The observer reuses the production snapshot at its current revision; its own
  work is identified by the `performance_audit_observer` trace span.
- **Start interval**: each measured main-app start to its next start. This measures offscreen throughput, not displayed FPS.
- **Native Metal GPU** (macOS default): the headless client completes preparation,
  warmup and image validation before publishing readiness. The runner attaches
  Xcode's native `Metal Application` and `GPU` instruments (a Blank template),
  waits for its documented Darwin recording-start notification, then releases
  the client through thirty untimed profiler warmup frames. No measured frame
  precedes that handshake. It then imports actual Vertex, Fragment and
  Compute intervals for that PID. Labelled begin/end command buffers carry the
  extracted source-frame ID. Metal's commit ordinal (`frame-number`), rather
  than encoding-start timestamps, groups submitted command-buffer identities;
  GPU intervals join by those identities. Independent GPU markers can execute
  concurrently with graphics, so their execution times never bound the frame.
  The reported envelope is the earliest-to-latest actual GPU activity for the
  frame; active time is the union of its intervals, not their sum. Marker work
  is excluded. Earlier uploads and work outside the marked render-graph group are outside
  this envelope. The raw native `.trace` remains beside each run for inspection.
  `native-scratch.json` records kernel intermediates opened by the launched
  recorder. After it exits, matching temporary-file identities are retired;
  the saved `.trace` remains self-contained and reviewable. This avoids
  accumulating multi-gigabyte Instruments conversion files in system temporary
  storage and does not impose a trace-size limit.
  Bevy's redundant per-pass counter instrumentation is disabled in this mode:
  on this Metal stack it can exhaust counter sample buffers. Xcode command-line
  tools and access to Instruments are required; capture/import failures fail the
  run. Use `--gpu-backend metal` to require this backend explicitly.
- **Other GPU backends**: `--gpu-backend timestamps` uses encoder-completion
  timestamps around queued render-graph commands, requiring both `TIMESTAMP_QUERY`
  and `TIMESTAMP_QUERY_INSIDE_ENCODERS`. This is a queue-completion envelope,
  not a per-pass active-time attribution. An eight-slot asynchronous readback
  ring skips rather than waits when full. Resolves follow sampling completion;
  no CPU GPU-wait is introduced. This path has not been hardware-validated by
  this macOS run. Metal explicitly refuses this marker approximation; use its
  native backend. Unsupported GPU timing is unavailable, never zero.
- **GPU clock**: independent of CPU time. Durations can be compared, but the
  graphs do not imply simultaneous CPU/GPU starts. Queue callback latency, where
  available, includes scheduling and polling; it is not GPU execution time.
- **Render passes**: native Metal rows retain source frame, command buffer,
  encoder, stage and operation. Parallel stages and nested intervals overlap.
  Elsewhere optional Bevy diagnostic rows use receipt frames because their API
  does not identify source frames. The report distinguishes those diagnostics
  from native operation timings.
- **Checkpoint**: time from an input step starting to its asserted authoritative
  result, followed by renderer/pipeline settling and a completed GPU submission
  for that state. The split separates controller/script/animation time from
  subsequent presentation/render settling, including any remaining authored warp
  fade. This is not input-to-photon latency. Authored
  fades and movement cadence are part of the result, not automatic optimization
  targets.

Measurement revision 7 re-arms GPU readiness when an assertion first becomes
true, even if map/tile/battle identity is unchanged, and when a pipeline becomes
ready. A checkpoint waits for a submission containing that state. Earlier
revision-6 checkpoint latencies do not certify completion of the new state and
must not be used as verified result-latency baselines. Frame-duration traces
remain useful historical evidence; baseline compatibility prevents mixing the
measurement revisions.

Untimed preparation and at least 120 warmup frames precede a validated GPU image
readback. Preparation/warmup has a 60 Hz ceiling to avoid flooding native captures
with incomplete startup renders; measured frames remain unthrottled. The warmup
cadence and measurement revision are baseline compatibility keys. GPU correlation markers run during warmup so their first use is untimed. Thirty more
frames drain the screenshot before timing. Production asynchronous pipeline compilation remains enabled; readiness observes its real
pending/error state. Destination map/pipeline creation during a transition stays
inside the measurement. The client drains callbacks for 500 ms after capture.
There are bounded frame, duration, step and subprocess deadlines.

Each fresh process resets game/renderer caches; OS filesystem and driver caches
remain warm across runs. This suite therefore measures first-use transitions
inside a fresh game process, not a cold boot or a guaranteed cold disk/shader
cache. Startup remains visible in the raw CPU trace but is excluded from the
measured operation census. See [loading diagnostics](../docs/cold-start-3d.md)
for pack/download/startup waterfalls.

## Browser comparisons

`tools/browser-world-stream.mjs` drives the production browser keyboard path in
a headless browser. It accepts a local game URL, a fresh output directory,
`long`, `outdoor`, `vertical`, or `house`, and `chromium` or `webkit`.
Use the corresponding preview (`world-stream`, `route29-border`,
`cherrygrove-border`, or `house-door`) and `multiplayer=off`.

The browser runner defaults to a 1280×800 CSS viewport at device scale 1.
Use `STREAM_VIEWPORT=1280x800 STREAM_DEVICE_SCALE=2` for a Retina comparison;
the report records actual canvas pixels, CSS dimensions and device pixel ratio.
Match those values between paired runs.

Set `STREAM_TRACE=0` for primary frame-cadence comparisons. This records raw
`requestAnimationFrame` intervals and worker outcomes without starting the Rust
loading trace. Run alternating before/after pairs on the same browser, backend,
pack, and hardware, with builds and other GPU verification finished first.
`python3 tools/browser_world_stream.py OUTPUT` summarizes the capture. Browser
rAF intervals measure callback cadence; they do not establish physical display
latency or prove that terrain and camera remained available.

Run a separate capture with the default detailed tracing to verify per-frame
world residency, camera continuity, connected-map motion, and worker waterfalls.
`GPU_WATERFALL=1` additionally collects supported WebGPU device timestamps;
`STREAM_SCREENSHOTS=1` enables separate visual QA and paired 3D/2D captures.
GPU query storage grows in pages through the entire measured journey. Queries
stop with movement, and readback occurs afterward. The report includes query
page counts and unframed passes; GPU records describe render submissions.
Completion callback latency includes the GPU queue and browser event-loop delivery,
so compare it with active GPU spans rather than treating it as display timing.
Keep these instrumented captures separate from profiling-off comparisons.
Worker fallback, unavailable rendering, and slow valid baseline runs remain
reported outcomes. Record the actual adapter and whether it is a fallback.

## Instrumentation controls

For repeated timing sweeps, add `--timings-only` to filter detailed CPU spans
at runtime in the same executable. Frame/GPU timings, pass diagnostics and
checkpoints remain available; the detailed CPU census/waterfall is explicitly
unavailable. This avoids very large Chrome traces during repeated sweeps.
Re-run a selected slow scenario without that flag to obtain its full CPU
waterfall. The trace filter is part of the compatibility key, so detailed and
timings-only captures cannot accidentally pass the same baseline gate.


Detailed tracing, native Instruments captures and timestamp queries have real cost. Build a second executable
without `operation-trace` into another target directory, and use `--cpu-only` to
measure interval/main-app timing without tracing, GPU queries or per-pass
diagnostics. The readiness observer and queue-completion polling still run.
These settings intentionally make its baseline incompatible with instrumented
captures. Keep separate baseline/candidate comparisons for each mode. Profiling
can change scheduling and GPU clock behavior, so subtracting durations across
modes does not isolate instrumentation overhead. Traced throughput is not
production display FPS.

## Author a scenario

The suite is an array of recipes in `tools/performance-scenarios.json`. Inputs
are `left`, `right`, `up`, `down`, `a`, `b`, `start`, or `wait`. Directions are
held; action buttons pulse with a real release every 200 ms. Every step has a
real-seconds deadline and an `until` assertion. Supported assertions are map,
tile, movement mode, battle presence/command-menu readiness, PP consumption, and a minimum duration.
All supplied assertions must hold. Unknown JSON fields and invalid keys or
bounds are rejected. Durations are integer seconds.

Inspect connection/warp metadata directly from the supplied pack:

```sh
target/performance-audit/release/examples/performance_audit \
  content-packs/core-modular.crystalpack --catalog \
  PlayersHouse1F NewBarkTown Route29 CherrygroveCity Route28 SilverCaveOutside
```

Keep recipes small and state-based. Do not replace a failing assertion with a
wall-clock sleep, teleport across a transition during measurement, suppress a
real script, or change collision/encounter data to force a passing result.
New scenarios can reuse an existing disposable fixture or load through normal
inputs; gameplay fixes belong in the production controller with regressions.

Tests:

```sh
python3 -m unittest discover -s tools -p 'test_performance_audit.py'
python3 -m unittest discover -s tools -p 'test_performance_audit_pairs.py'
python3 -m unittest discover -s tools -p 'test_performance_audit_pair_effects.py'
python3 -m unittest discover -s tools -p 'test_performance_audit_reanalyze.py'
python3 -m unittest discover -s tools -p 'test_performance_audit_scenes.py'
python3 -m unittest discover -s tools -p 'test_performance_audit_motion.py'
python3 -m unittest discover -s tools -p 'test_performance_audit_boundary.py'
python3 -m unittest discover -s tools -p 'test_performance_audit_residency.py'
node --test tools/test_performance_audit_dashboard.mjs
python3 -m unittest discover -s tools -p 'test_host_quiet.py'
python3 -m unittest discover -s tools -p 'test_operation_waterfall.py'
python3 -m unittest discover -s tools -p 'test_trace_stream.py'
python3 -m unittest discover -s tools -p 'test_metal_trace.py'
python3 -m unittest discover -s tools -p 'test_metal_capture.py'
python3 -m unittest discover -s tools -p 'test_metal_scratch.py'
CARGO_TARGET_DIR=target/performance-audit cargo test --locked -p crystal-bevy \
  --lib --features performance-audit,operation-trace performance_audit
CARGO_TARGET_DIR=target/performance-audit cargo clippy --locked -p crystal-bevy \
  --all-targets --features performance-audit,operation-trace
cargo fmt --all -- --check
```

Reports, raw traces, image receipts and executables are ignored local outputs.
They are not source deliverables and must not be staged.
