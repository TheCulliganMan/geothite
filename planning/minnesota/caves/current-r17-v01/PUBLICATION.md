# Executed current R17 talus-den publication

All paths are relative to the repository root. Choose fresh output names on
reruns. Packs/renders/checkpoints remain ignored; authored source and plans
remain under `planning/minnesota/`. No content export workspace is modified.

```sh
RUST_MIN_STACK=67108864 target/runtime-fps/wasm-dev/revise_authored_cave \
  content-packs/minnesota-r13-spring-shelter-v01.crystalpack \
  planning/minnesota/caves/johto-quality-v01/MnR17Cave1F.previous.json \
  planning/minnesota/caves/johto-quality-v01/MnR17Cave1F.json \
  content-packs/minnesota-r17-talus-den-v01.crystalpack

export PYTHONPATH="$PWD/.local/ely-backroute-python${PYTHONPATH:+:$PYTHONPATH}"
python3 planning/minnesota/tools/audit_current_cave.py \
  --contract planning/minnesota/caves/current-r17-v01/native.contract.json \
  --pack content-packs/minnesota-r17-talus-den-v01.crystalpack \
  --output planning/minnesota/renders/r17-current-den-v01/published-v01

target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-r17-talus-den-v01.crystalpack --audio --desktop
```

All pass. The bounded Rust publisher checks fresh ignored/untracked output,
compact geography before/after/readback, exact source preimages and complete
native readback. Repeating it with fresh output
`minnesota-r17-talus-den-replay-v01.crystalpack` produces a byte-identical pack.
Independent normalized equality permits only the declared terrain/border/
mirrors/identity delta; all690other maps remain exact. All377 audio PCM hashes,
frame counts and loop ranges pass.

## Controller and stdio inputs

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo test --profile wasm-dev --locked -p crystal-bevy --lib --no-run \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.codegen-units=256

CRYSTAL_CAVE_QUALITY_PACK="$PWD/content-packs/minnesota-r17-talus-den-v01.crystalpack" \
CRYSTAL_CAVE_QUALITY_SAVES="$PWD/planning/minnesota/renders/r17-current-den-v01/reached-v02" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  cave_quality_controller_current_r17_cameras_mouth_pickup_and_continue \
  --ignored --nocapture --test-threads=1

CRYSTAL_R17_TOR_CONTROLLER_PACK="$PWD/content-packs/minnesota-r17-talus-den-v01.crystalpack" \
CRYSTAL_R17_TOR_SOURCE="$PWD/planning/minnesota/caves/johto-quality-v01/MnR17Cave1F.json" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  r17_tor_cave_controller_bent_route_directional_roof_revive_once_and_continue \
  --ignored --nocapture --test-threads=1

python3 planning/minnesota/caves/current-r17-v01/check_mcp.py \
  --pack content-packs/minnesota-r17-talus-den-v01.crystalpack \
  --save planning/minnesota/renders/r17-current-den-v01/reached-v01/MnR17Cave1F-mouth.crystalsave \
  --output planning/minnesota/renders/r17-current-den-v01/mcp-v01
```

The new single-room wrapper retains the original all-nine caller and runs the
same complete-room course. Initial whole-room geometry passes; the first effect
case exposes the obsolete uniform-forest RouteGrid assumption. The corrected
fixture validates exact native mouth fields and actual installed collision,
then uses the same real Up/Down inputs. It retains the ten adjacent approach
steps, all three roof probes, found-item payload, both printed pocket pages,
movement lock, quantity/flags, reentry and Save/Continue assertions.

The intermediate rebuild fails on a mistaken Result/Option comparison in this
new test helper and an independently edited R04 journey's borrowed runtime.
Both precise compile fixes are preserved; no runtime movement, clock, reward,
map or validation rule is changed to satisfy tests. Rejected logs remain
`.local/r17-current-{effect-v01,controller-build-v02}.log`.

The stdio replay independently completes all six viewpoints, all three roof
cells, blocked Up, original item notice/input lock/body removal, actual passive
mouth safety and same-mouth reentry. A second real client loads the saved
collected state and walks onto the former item tile. Both processes exit0with
byte-empty stderr. These empty-party local placements are explicit setup;
they do not earn regional access. Controller checks own exact quantity/flag
and complete resource assertions rather than inferring those from map text.

The final own rebuild completes in2m15s. Its whole-room case passes in7.86s
and its original directional-roof/reward/persistence case passes in7.63s.
Those final executions are retained as `geometry-v02` and `effect-v02`, with
fresh viewpoint saves in `reached-v02/`. The initial generic course and its
actual reached saves remain separately bound to the first successful build.

## Native presentation and connected atlas

```sh
python3 planning/minnesota/tools/review_cave_quality_native.py \
  --pack content-packs/minnesota-r17-talus-den-v01.crystalpack \
  --binary target/runtime-fps/wasm-dev/examples/render_at_location \
  --room MnR17Cave1F \
  --output planning/minnesota/renders/r17-current-den-v01/native-v01

CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576 RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/examples/render_at_location \
  --pack content-packs/minnesota-r17-talus-den-v01.crystalpack \
  --save planning/minnesota/renders/r17-current-den-v01/reached-v01/MnR17Cave1F-stone-toe.crystalsave \
  --view 2d \
  --screenshot planning/minnesota/renders/r17-current-den-v01/native-reached-v01/stone-toe.png

python3 planning/minnesota/tools/audit_outdoor_connections.py \
  --pack content-packs/minnesota-r17-talus-den-v01.crystalpack \
  --output planning/minnesota/renders/r17-current-den-v01/atlas-v01 --scale 8

python3 planning/minnesota/tools/check_winona_mcp.py \
  --pack content-packs/minnesota-r17-talus-den-v01.crystalpack \
  --client target/runtime-fps/wasm-dev/geothite
```

All six complete640×576frames match native source with zero differences across
2,211,840pixels. Actual night palette and unchanged native pixels are used.
Whole room/source/native camera sheets were inspected; all prior originals,
conversions and rejected passes remain retained. The whole exterior atlas is
static connection evidence, not an earned Surf/campaign playthrough. Parent
regional/grass/city/river logs retain their original pack bindings.
The actually reached stone-toe frame and complete97-map atlas were inspected
as well. A fresh whole native Union Cave1F terrain reference was rendered and
viewed under `reference-v01/`, retaining its forced NITE palette. The comparison
uses the same original rock edges, attached stone and native floor vocabulary;
R17 remains a short den rather than copying the larger reference layout.

Execution logs use `.local/r17-current-*.log`; the final bound verification
receipt records the actual source, executable and log hashes. Whole-workspace
format/Clippy results and supplemental scoped results are recorded separately.

Required workspace formatting fails first on unrelated
`offset-allocator/src/lib.rs:408` differences. Required normal Bevy all-target
Clippy fails on the existing core `bad_bit_mask` denial in
`crystal-core/src/multiplayer/mod.rs:1332`. Both failed logs remain retained.
The two changed Rust leaves pass scoped rustfmt; no unrelated source is
reformatted to conceal those failures.
Supplemental Bevy `--no-deps --lib --tests` Clippy passes in1m51s with only the
existing `never_loop` denial allowed and no diagnostic in either changed test
leaf. It does not replace the failed required all-target run.
