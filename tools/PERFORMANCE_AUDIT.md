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
  --output .local/performance-audit/baseline --repeats 3
```

Open `.local/performance-audit/baseline/index.html`. It is a standalone offline
report with scenario/repeat selection, frame timings, slow-frame inspection,
controller versus render-settling latency, CPU operation attribution, render
pass diagnostics, evidence-based optimization leads, repeat ranges, and
baseline regressions. Hover a frame to follow its source ID into the detailed
CPU thread waterfall. Raw traces also open in Perfetto.

The checked-in recipes cover outdoor/indoor idle and movement, land and Surf
movement, house exit/entry, outdoor map connections, a region boundary, battle
idle, and an actual battle move. Each movement/transition step requires the
specified authoritative map/tile/mode and a settled renderer. The battle move
requires actual PP consumption. A blocked route, unexpected encounter, script
or wrong pack fails with the last observed state; it is never accepted as a
fast result. Surf preparation uses the existing legal HM/badge developer
fixture, then mounts through ordinary A/Yes before warmup and measurement.

Filter a run with repeated `--scenario NAME` arguments. Output directories must
be empty. Each repeat runs in a separate process; repeats cycle over the suite
rather than collecting all repeats of one scenario consecutively.

## Compare changes

After building the changed executable, preserve the first report and run:

```sh
python3 tools/performance_audit.py \
  --binary target/performance-audit/release/examples/performance_audit \
  --output .local/performance-audit/candidate --repeats 3 \
  --baseline .local/performance-audit/baseline/summary.json
```

The gate compares median **per-run p95** durations for frame intervals,
main-app CPU work and GPU envelopes, plus median checkpoint latency. Defaults
require both a 10% increase and a 0.2 ms increase to flag a regression; use
`--threshold-percent` and `--threshold-ms` to set project-specific budgets.
These are practical gates, not statistical significance claims. Inspect the
repeat range before acting on small changes; increase repeats for noisy runs.

Exit status is 0 for valid runs within thresholds, 1 for a failed scenario or
capture, and 2 for regressions or an incompatible baseline. Configuration,
scenario, pack hash/identity, viewport, clock, instrumentation and hardware
must match. Binary hashes and Git revision/dirty state are recorded as
provenance, while changed binaries are allowed for intentional A/B comparisons.
A missing GPU is explicitly unavailable, never a zero-millisecond result.
Supported timestamp captures with errors or less than 95% frame coverage fail.

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
- **Start interval**: consecutive main-app start times, excluding the first
  measured boundary. This measures offscreen throughput, not displayed FPS.
- **GPU envelope**: timestamp markers in empty compute passes bracket the render
  graph's queued command buffers. Pass-descriptor timestamp writes support Metal
  without Bevy's disabled macOS encoder timestamps. An eight-slot readback ring
  skips rather than waiting when full, and reports skipped/error counts. Earlier
  queue uploads and work outside the graph are outside this envelope.
- **GPU clock**: independent of CPU time. Source frame IDs come through resource
  extraction; durations are compared without pretending the clocks are aligned.
  Queue completion latency includes driver scheduling and callback polling and
  must not be described as GPU execution time.
- **Render passes**: Bevy's diagnostic paths provide individual CPU durations and,
  where supported, GPU durations. Their receipt frame is recorded explicitly;
  Bevy's API does not provide their source frame. Nested durations overlap. On
  macOS Bevy's encoder timestamp writes are disabled upstream, so its GPU pass
  values are omitted; the portable GPU envelope remains available.
- **Checkpoint**: time from an input step starting to its asserted authoritative
  result, followed by renderer/pipeline settling and a completed GPU submission
  for that state. The split separates controller/script/animation time from
  subsequent render settling. This is not input-to-photon latency. Authored
  fades and movement cadence are part of the result, not automatic optimization
  targets.

Untimed preparation and at least 120 warmup frames precede a validated GPU image
readback. Thirty more frames drain the screenshot before timing. Production
asynchronous pipeline compilation remains enabled; readiness observes its real
pending/error state. Destination map/pipeline creation during a transition stays
inside the measurement. The client drains callbacks for 500 ms after capture.
There are bounded frame, duration, step and subprocess deadlines.

Each fresh process resets game/renderer caches; OS filesystem and driver caches
remain warm across runs. This suite therefore measures first-use transitions
inside a fresh game process, not a cold boot or a guaranteed cold disk/shader
cache. Startup remains visible in the raw CPU trace but is excluded from the
measured operation census. See [loading diagnostics](../docs/cold-start-3d.md)
for pack/download/startup waterfalls.

## Instrumentation controls

Detailed tracing and timestamp queries have real cost. Build a second executable
without `operation-trace` into another target directory, and use `--cpu-only` to
measure interval/main-app timing without tracing, GPU queries or per-pass
diagnostics. The readiness observer and queue-completion polling still run.
These settings intentionally make its baseline incompatible with instrumented
captures. Compare the reports manually to estimate instrumentation overhead;
do not claim traced throughput is final production FPS.

## Author a scenario

The suite is an array of recipes in `tools/performance-scenarios.json`. Inputs
are `left`, `right`, `up`, `down`, `a`, `b`, `start`, or `wait`. Directions are
held; action buttons pulse with a real release every 200 ms. Every step has a
real-seconds deadline and an `until` assertion. Supported assertions are map,
tile, movement mode, battle presence, PP consumption, and a minimum duration.
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
python3 -m unittest discover -s tools -p 'test_operation_waterfall.py'
CARGO_TARGET_DIR=target/performance-audit cargo test --locked -p crystal-bevy \
  --lib --features performance-audit,operation-trace performance_audit
CARGO_TARGET_DIR=target/performance-audit cargo clippy --locked -p crystal-bevy \
  --all-targets --features performance-audit,operation-trace
cargo fmt --all -- --check
```

Reports, raw traces, image receipts and executables are ignored local outputs.
They are not source deliverables and must not be staged.
