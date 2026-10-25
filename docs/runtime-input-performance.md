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
