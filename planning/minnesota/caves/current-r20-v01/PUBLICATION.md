# Executed current R20 cleft publication

Run from the repository root. The paths below name immutable executed epochs;
reruns require fresh ignored output names. Rust publishes packs; Python only
reads native data, renders existing source art and drives real stdio inputs.

```sh
RUST_MIN_STACK=67108864 target/runtime-fps/wasm-dev/revise_authored_cave \
  content-packs/minnesota-current-grass-r04-v02.crystalpack \
  planning/minnesota/caves/johto-quality-v01/MnR20Cave1F.previous.json \
  planning/minnesota/caves/johto-quality-v01/MnR20Cave1F.json \
  content-packs/minnesota-r20-seep-cleft-v02.crystalpack

export PYTHONPATH="$PWD/.local/ely-backroute-python${PYTHONPATH:+:$PYTHONPATH}"
python3 planning/minnesota/tools/audit_current_cave.py \
  --contract planning/minnesota/caves/current-r20-v01/native.contract.json \
  --pack content-packs/minnesota-r20-seep-cleft-v02.crystalpack \
  --output planning/minnesota/renders/r20-current-cleft-v01/published-v02

target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-r20-seep-cleft-v02.crystalpack --audio --desktop
```

All pass. The existing bounded publisher certifies source/native preimages,
fresh ignored output, compact geography before/after/readback and exact native
readback. Its replay uses fresh output
`minnesota-r20-seep-cleft-replay-v02.crystalpack`. Full normalized equality
proves that all690other maps and every unowned native field remain exact.
All377audio PCM hashes, frame counts and loop ranges pass.

## Fresh controller build and actual input

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo test --profile wasm-dev --locked -p crystal-bevy --lib --no-run \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.codegen-units=256

CRYSTAL_CAVE_QUALITY_PACK="$PWD/content-packs/minnesota-r20-seep-cleft-v02.crystalpack" \
CRYSTAL_CAVE_QUALITY_SAVES="$PWD/planning/minnesota/renders/r20-current-cleft-v01/reached-v02" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  cave_quality_controller_current_r20_cameras_stairs_hops_mouths_pickup_and_continue \
  --ignored --nocapture --test-threads=1

CRYSTAL_R20_CLEFT_CONTROLLER_PACK="$PWD/content-packs/minnesota-r20-seep-cleft-v02.crystalpack" \
CRYSTAL_R20_CLEFT_SOURCE="$PWD/planning/minnesota/caves/johto-quality-v01/MnR20Cave1F.json" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  bevy_shell::tests::r20_cleft:: --ignored --nocapture --test-threads=1
```

The own fresh build completes in1m58s. Final-parent geometry passes in13.88s,
and the three original mouth/stair/drop/item-effect cases pass in16.65s. No
original R20 test assertion is removed or relaxed. Restored editorial probes
name the same two stair lanes, two ledges, four closed shoulders, true passive
mouths and original item. The added wrapper invokes the unchanged generic
all-nine course for this room alone, preserving the original all-nine caller.

The first R17-parent candidate passes the same four cases in12.96s/15.75s.
Its initial evidence and saves remain separate; current primary checks bind
to the later R04-parent v02pack. Explicit local position/party/RNG fixtures
do not earn regional access or complete SQ03/campaign progress.

## Native views and production stdio

```sh
python3 planning/minnesota/tools/review_cave_quality_native.py \
  --pack content-packs/minnesota-r20-seep-cleft-v02.crystalpack \
  --binary target/runtime-fps/wasm-dev/examples/render_at_location \
  --room MnR20Cave1F \
  --output planning/minnesota/renders/r20-current-cleft-v01/native-v02

CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo build --profile wasm-dev --locked -p geothite \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0

python3 planning/minnesota/caves/current-r20-v01/check_mcp.py \
  --pack content-packs/minnesota-r20-seep-cleft-v02.crystalpack \
  --save planning/minnesota/renders/r20-current-cleft-v01/reached-v02/MnR20Cave1F-west-mouth.crystalsave \
  --output planning/minnesota/renders/r20-current-cleft-v01/mcp-v02

python3 planning/minnesota/tools/check_winona_mcp.py \
  --pack content-packs/minnesota-r20-seep-cleft-v02.crystalpack \
  --client target/runtime-fps/wasm-dev/geothite

python3 planning/minnesota/tools/audit_outdoor_connections.py \
  --pack content-packs/minnesota-r20-seep-cleft-v02.crystalpack \
  --output planning/minnesota/renders/r20-current-cleft-v01/atlas-v02 --scale 8

CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576 RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/examples/render_at_location \
  --pack content-packs/minnesota-r20-seep-cleft-v02.crystalpack \
  --save planning/minnesota/renders/r20-current-cleft-v01/mcp-v02/upper-lip.crystalsave \
  --view 2d \
  --screenshot planning/minnesota/renders/r20-current-cleft-v01/native-reached-v02/upper-lip.png
```

Nine complete640×576night views compare3,317,760pixels. Native animation uses
one coherent water phase per complete frame; production pixels are not edited.
Every final-parent comparison passes with zero differing pixels. The complete
before/after room, all nine native cameras, actually reached lip/seep frames
and the complete exterior atlas were personally inspected. The reached frames
load actual controller/MCP saves without position, clock or facing overrides.
The fresh own client contains the current checked enemy-Mimic projection from
the concurrent R04 work; no stale pre-fix client is recommended for this child.
R20 does not modify that battle code. Its exact final executable and execution
logs are bound in the verification receipt.

The two-process MCP course plays all nine viewpoints, both native drops and
reverse refusals,15/13ordinary returns, each exterior passive return/clear
step/same-mouth entry, native item pages/input lock/body removal and Start.
A second real process continues the saved collected state and walks onto the
old item tile. Native quantity/HP/flags are separately asserted by the original
controller cases rather than inferred from text output.

The first-parent read-only qualification also includes native UP_WALL entry
and departure masks, both before and after item collection. Its minimum
non-hop returns remain15/13; the proof is retained in
`native-return-qualification-v01.json` and is not relabeled as a new-parent
execution. Final whole-pack preservation retains that exact room/art/collision.

All original renders, source/camera crops, conversions, checkpoints and the
unselected R17-parent pack remain retained. The actual full outdoor atlas is
static connection evidence, not an earned Surf/prerequisite playthrough.

## Repository check limits

The changed Rust test leaf passes rustfmt. Required workspace formatting
still fails first on unrelated `offset-allocator/src/lib.rs:408` differences.
Required normal Bevy all-target Clippy fails on the existing core
`bad_bit_mask` denial at `crystal-core/src/multiplayer/mod.rs:1332`.
Supplemental Bevy `--no-deps --lib --tests` Clippy passes in2m04s with the
existing `never_loop` denial allowed, with no diagnostic in the changed leaf.
These scoped results do not make the whole workspace lint-clean.

Executed logs are `.local/r20-current-*.log`. The final receipt binds actual
source, contract, binaries, capture reports and logs to this publication.
