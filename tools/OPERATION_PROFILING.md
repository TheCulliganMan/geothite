# Crystal Bevy operation profiling

Run from `rust/`. The opt-in `operation-trace` feature enables Bevy 0.14's
system, command, schedule, asset, and render tracing, including detailed engine
spans. Native entry points start capture before reading the pack, with named
spans for pack verification, runtime loading, shell initialization, audio device
opening, snapshots, background audio preparation, and terrain construction.
Ordinary builds do not enable tracing. This records CPU wall time,
including blocking; it does not measure GPU execution time.

Build once, then capture the actual renderer (one process per trace):

```sh
cargo build -p crystal-bevy --example render_at_location \
  --features location-tester,operation-trace
mkdir -p output/operation-profile
TRACE_CHROME="$PWD/output/operation-profile/overworld.json" \
RUST_LOG=info \
  target/debug/examples/render_at_location \
  --pack "$PWD/../content-packs/core-modular.crystalpack" \
  --map NewBarkTown --x 13 --y 6 --view 2.5d \
  --screenshot "$PWD/output/operation-profile/overworld.png"
python3 tools/operation_waterfall.py output/operation-profile/overworld.json \
  --output output/operation-profile/overworld.html
```

Open the generated HTML locally. It contains the full completed-span waterfall,
thread lanes, nesting, frame selection, timing inspection, and an operation
census with inclusive total, median, p95, and maximum durations. Its sibling
`.summary.json` provides machine-readable statistics. No hosted service or
JavaScript dependency is required. The raw trace also opens in Perfetto.

Use `--view 2d` for the original compositor. Add `--walk RRDLLU` for a movement
capture in 2.5D; this invokes the existing developer location-test keyboard
harness, and its movement CSV reports frame times separately. Screenshots and
warm-up are included in the trace; select steady-state frames for performance
comparisons. Always compare the same map, coordinates, clock, renderer, build
profile and hardware. Close other rendering applications and avoid compiling
while collecting comparative timings.

The capture covers operations actually executed by the selected scenario, not
unvisited battle/menu/script branches. Native entry points retain a single
logging owner through shutdown so startup and frame work share one clock.
Library callers may call `operation_trace::start()` before their own loading
and retain its owner; the shell shares that subscriber. Unfinished spans
are counted and excluded from timings. Nested totals overlap, and parallel
threads overlap, so their totals must not be interpreted as elapsed frame time.
A long render span can be a driver or presentation wait rather than CPU compute.
Use a single-map invocation for each trace file; batch and `--view both` modes
spawn child processes and must not share a `TRACE_CHROME` output path.

For production-speed measurements use `--release` both when building and when
selecting the executable (`target/release/examples/render_at_location`). Measure
without `operation-trace` too when assessing final frame latency; tracing has
runtime overhead. Do not change Game Boy simulation cadence to improve FPS.

Native audio device creation occurs during startup, before the render loop.
An unavailable audio device therefore reports its error at startup. Browser
audio retains its user-gesture initialization.

Parser tests:

```sh
python3 -m unittest discover -s tools -p 'test_operation_waterfall.py'
```

The isolated multiplayer presentation benchmark includes ordinary Bevy system
dispatch but excludes pack loading, graphics presentation, and network traffic:

```sh
cargo test -p crystal-bevy --lib --features location-tester \
  multiplayer_render_performance_benchmark -- --ignored --nocapture
```

Its speedup is a system-level result, not an overall game FPS multiplier.

Native presentation uses synchronized output with a two-frame maximum latency
hint. The previous one-frame hint can serialize CPU and GPU work in wgpu's
surface acquisition; two permits overlap but may buffer an additional frame.
Movement CSVs include the window's `focused` state so background throttling
can be distinguished from normal rendering cost.
For an A/B test using the same executable, set
`CRYSTAL_RENDER_TEST_FRAME_LATENCY=1` or `=2` on a location capture. This
override is compiled only with `location-tester`, requires a screenshot
capture, and does not change ordinary play. Other values are rejected.

Pack verification remains at the file-loading and runtime-entry boundaries.
Runtime construction verifies once and retains that verified identity rather
than revalidating and rehashing the same immutable pack internally. The
`runtime_load_` tests count this work and check rejection of corrupted packs
and incompatible saves.
