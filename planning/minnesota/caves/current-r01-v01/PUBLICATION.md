# Executed R01 fitted-hollows publication

Run from the repository root. Output names below are immutable executed
epochs; reruns require fresh ignored pack, render and save directories.
Rust publishes packs. Python only audits native data, renders existing source
art and drives production stdio inputs. No source export workspace is changed.

```sh
RUST_MIN_STACK=67108864 target/runtime-fps/wasm-dev/revise_authored_cave \
  content-packs/minnesota-r20-seep-cleft-v02.crystalpack \
  planning/minnesota/caves/johto-quality-v01/MnR01Cave1F.previous.json \
  planning/minnesota/caves/johto-quality-v01/MnR01Cave1F.json \
  content-packs/minnesota-r01-limestone-hollows-v01.crystalpack

export PYTHONPATH="$PWD/.local/ely-backroute-python${PYTHONPATH:+:$PYTHONPATH}"
python3 planning/minnesota/tools/audit_current_cave.py \
  --contract planning/minnesota/caves/current-r01-v01/native.contract.json \
  --pack content-packs/minnesota-r01-limestone-hollows-v01.crystalpack \
  --output planning/minnesota/renders/r01-current-hollows-v01/published-v01

target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-r01-limestone-hollows-v01.crystalpack --audio --desktop
```

All pass. The bounded publisher checks exact source/native preimages, fresh
ignored output, compact geography before/after/readback and strict native
readback. Repeating the command with fresh output
`minnesota-r01-limestone-hollows-replay-v01.crystalpack` produces a byte-identical
pack. Independent normalized equality permits only the declared room terrain,
solid border, mirrors and identity. All690other maps/unowned fields remain
exact. All377audio PCM hashes, frame counts and loop ranges pass.

## Fresh controller build and unchanged original cases

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo test --profile wasm-dev --locked -p crystal-bevy --lib --no-run \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.codegen-units=256

CRYSTAL_CAVE_QUALITY_PACK="$PWD/content-packs/minnesota-r01-limestone-hollows-v01.crystalpack" \
CRYSTAL_CAVE_QUALITY_SAVES="$PWD/planning/minnesota/renders/r01-current-hollows-v01/reached-v01" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  cave_quality_controller_current_r01_cameras_stairs_mouth_pickup_and_continue \
  --ignored --nocapture --test-threads=1

CRYSTAL_R01_HOLLOWS_CONTROLLER_PACK="$PWD/content-packs/minnesota-r01-limestone-hollows-v01.crystalpack" \
CRYSTAL_R01_HOLLOWS_SOURCE="$PWD/planning/minnesota/caves/johto-quality-v01/MnR01Cave1F.json" \
CRYSTAL_R01_UPPER_LOOP_PLAN="$PWD/planning/minnesota/caves/current-r01-v01/input-plan.json" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  bevy_shell::tests::r01_hollows:: --ignored --nocapture --test-threads=1
```

Build3m00s; generic complete-room case11.78s; all three original cases17.34s.
All pass. The added wrapper calls the same generic all-nine course, preserving
that caller. No original R01test is changed. Current source restores its exact
stair probes and consumes a separate explicit adjacent-step upper-loop plan.
The native program/body/pickup/flag metadata remains unchanged.

The upper-loop case actually walks every edge of the26-input cycle, saves at
the upper seam, continues there, then proves both6- and18-input returns. The
medicine case observes all three original guide pages, original item payload
and notice/input ownership, native poison cure and consumption, once-only
pickup/reentry and complete party/bag/flag persistence. Its local legal
Totodile20poison setup does not claim earned starter/route progress.

## Native presentation, stdio and exterior composition

```sh
python3 planning/minnesota/tools/review_cave_quality_native.py \
  --pack content-packs/minnesota-r01-limestone-hollows-v01.crystalpack \
  --binary target/runtime-fps/wasm-dev/examples/render_at_location \
  --room MnR01Cave1F \
  --output planning/minnesota/renders/r01-current-hollows-v01/native-v01

python3 planning/minnesota/caves/current-r01-v01/check_mcp.py \
  --pack content-packs/minnesota-r01-limestone-hollows-v01.crystalpack \
  --save planning/minnesota/renders/r01-current-hollows-v01/reached-v01/MnR01Cave1F-mouth.crystalsave \
  --output planning/minnesota/renders/r01-current-hollows-v01/mcp-v01

CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576 RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/examples/render_at_location \
  --pack content-packs/minnesota-r01-limestone-hollows-v01.crystalpack \
  --save planning/minnesota/renders/r01-current-hollows-v01/mcp-v01/upper-cycle.crystalsave \
  --view 2d \
  --screenshot planning/minnesota/renders/r01-current-hollows-v01/native-reached-v01/upper-cycle.png

python3 planning/minnesota/tools/audit_outdoor_connections.py \
  --pack content-packs/minnesota-r01-limestone-hollows-v01.crystalpack \
  --output planning/minnesota/renders/r01-current-hollows-v01/atlas-v01 --scale 8

python3 planning/minnesota/tools/check_winona_mcp.py \
  --pack content-packs/minnesota-r01-limestone-hollows-v01.crystalpack \
  --client target/runtime-fps/wasm-dev/geothite
```

All six complete640×576native night frames compare2,211,840pixels with zero
differences. Production pixels are not edited. Stdio independently plays the
complete26-input cycle and6/18returns, original guide/pickup movement locks,
passive exterior safety and same-mouth reentry. A second client continues the
collected state and walks onto the old item tile. Both processes exit0with
byte-empty stderr. Exact medicine/quantity/flag effects remain separately
controller-asserted, rather than inferred from text output.

The actual complete before/after room and all native camera views were
inspected. The reached captures use actual controller/MCP saves without
position, clock or facing overrides. All original/pre-conversion/crop/preview
and historical source/native epochs remain retained. Parent route/city/river
test logs retain their original identity bindings; static exterior graph
evidence is not a new earned Surf/campaign journey.
The actually reached upper-cycle and rounded-toe views and the whole exterior
atlas were also inspected. The atlas retains97actual outdoor maps in one
seam component with zero overlaps and inconsistent placement cycles.

## Repository check limits

The changed Rust leaf passes rustfmt. Required workspace formatting retains
unrelated `offset-allocator/src/lib.rs:408` differences. Required normal Bevy
all-target Clippy stops at the existing core `bad_bit_mask` denial in
`crystal-core/src/multiplayer/mod.rs:1332`. Logs are retained as
`.local/r01-current-{workspace-fmt,clippy}-v01.log`; the whole checkout is not
reported lint-clean. Scoped supplemental results are recorded separately.
Supplemental Bevy `--no-deps --lib --tests` Clippy passes in2m16s with the
existing `never_loop` denial allowed and no diagnostic in the changed test
leaf. It does not replace the failed required all-target run.

All execution logs use `.local/r01-current-*.log`. The final bound receipt
records exact pack/source/contract/executable/log hashes for these actual runs.

## Later composed R05/League child

After this R01 acceptance, the grass owner caught the selector change before
publishing its prior-parent candidate. It replayed the completed League text
and exact33R05grass cells onto the accepted R01 pack, then selected the composed
v03child after its own native/preservation/MCP/audio checks. No cave was dropped
to advance the route work. The current R01 source and original c7e3e029pack
remain unchanged and recoverable.

```sh
python3 planning/minnesota/caves/current-r01-v01/check_descendant.py \
  --pack content-packs/minnesota-current-grass-r05-v03.crystalpack \
  --output planning/minnesota/renders/r01-current-hollows-v01/r05-descendant-v01
```

The independent read-only comparison passes: complete R01 module/programs/
dialogue/actors/events, native payload/attribute mirrors, runtime files,
tilesets and compiled audio/manifest remain exact. The whole-room source
comparison has zero differing pixels across57,344pixels. This is preservation
and source-render evidence, distinct from fresh production GPU/controller
execution on that later child. Prior R01 execution hashes retain their c7pack
binding; no earlier result is relabeled.
The selected world record and actual v03pack hash were independently matched
to that readback. A fresh full outdoor atlas at `r05-atlas-v01/` retains97maps
in one component with zero overlaps/cycles; it was visually inspected. This
adds current composition evidence without changing R01's historical test logs.
