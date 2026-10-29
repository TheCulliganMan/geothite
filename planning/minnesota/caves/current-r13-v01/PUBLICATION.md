# Executed R13 spring-shelter publication

Run from the repository root with the pinned Rust toolchain. Every output is
an immutable executed epoch; reruns require fresh pack/render/save names.
Native publication is Rust. Python only audits existing packs, renders native
art and drives the production stdio client.

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo build --profile wasm-dev --locked -p crystal-assets \
  --bin revise_authored_cave \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0

RUST_MIN_STACK=67108864 target/runtime-fps/wasm-dev/revise_authored_cave \
  content-packs/minnesota-current-grass-r03-v02.crystalpack \
  planning/minnesota/caves/johto-quality-v01/MnR13Cave1F.previous.json \
  planning/minnesota/caves/johto-quality-v01/MnR13Cave1F.json \
  content-packs/minnesota-r13-spring-shelter-v01.crystalpack

export PYTHONPATH="$PWD/.local/ely-backroute-python${PYTHONPATH:+:$PYTHONPATH}"
python3 planning/minnesota/caves/current-r13-v01/audit.py \
  --pack content-packs/minnesota-r13-spring-shelter-v01.crystalpack \
  --output planning/minnesota/renders/r13-current-shelter-v01/published-v01

target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-r13-spring-shelter-v01.crystalpack --audio --desktop
```

Publication, strict readback, compact geography and full native preservation
pass. The same publisher command with fresh output
`minnesota-r13-spring-shelter-replay-v01.crystalpack` reproduces the whole file
byte for byte (`cmp` exits0). No services, rewards or runtime commands are
reinstalled. The read-only source/render audit takes0.164s after pack loading.

## Fresh controller build and input

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo test --profile wasm-dev --locked -p crystal-bevy --lib --no-run \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.codegen-units=256

CRYSTAL_CAVE_QUALITY_PACK="$PWD/content-packs/minnesota-r13-spring-shelter-v01.crystalpack" \
CRYSTAL_CAVE_QUALITY_SAVES="$PWD/planning/minnesota/renders/r13-current-shelter-v01/reached-v01" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  cave_quality_controller_current_r13_cameras_stairs_mouth_pickup_and_continue \
  --ignored --nocapture --test-threads=1

CRYSTAL_R13_REFUGE_CONTROLLER_PACK="$PWD/content-packs/minnesota-r13-spring-shelter-v01.crystalpack" \
CRYSTAL_R13_REFUGE_SOURCE="$PWD/planning/minnesota/caves/johto-quality-v01/MnR13Cave1F.json" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  bevy_shell::tests::r13_refuge:: \
  --skip true_passive_return_and_both_useful_stairs \
  --ignored --nocapture --test-threads=1
```

Build1m21s; whole-room case5.70s; two service cases6.27s. All pass. The new
single-room wrapper calls the unchanged generic all-nine course. The all-nine
caller remains intact; the wrapper does not claim four other caves have their
source revisions installed yet. Existing service fixtures use original legal
Mew/Totodile parties as explicit setup, followed only by actual controller
input. Pack species/teams/encounters remain the parent's regional roster.

## Native frames and actual stdio

```sh
python3 planning/minnesota/tools/review_cave_quality_native.py \
  --pack content-packs/minnesota-r13-spring-shelter-v01.crystalpack \
  --binary target/runtime-fps/wasm-dev/examples/render_at_location \
  --room MnR13Cave1F \
  --output planning/minnesota/renders/r13-current-shelter-v01/native-v01

CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576 RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/examples/render_at_location \
  --pack content-packs/minnesota-r13-spring-shelter-v01.crystalpack \
  --save planning/minnesota/renders/r13-current-shelter-v01/reached-v01/MnR13Cave1F-spring-bank.crystalsave \
  --view 2d \
  --screenshot planning/minnesota/renders/r13-current-shelter-v01/native-reached-v01/spring-bank.png

python3 planning/minnesota/caves/current-r13-v01/check_mcp.py \
  --pack content-packs/minnesota-r13-spring-shelter-v01.crystalpack \
  --save planning/minnesota/renders/r13-current-shelter-v01/reached-v01/MnR13Cave1F-mouth.crystalsave \
  --output planning/minnesota/renders/r13-current-shelter-v01/mcp-v01

python3 planning/minnesota/tools/audit_outdoor_connections.py \
  --pack content-packs/minnesota-r13-spring-shelter-v01.crystalpack \
  --output planning/minnesota/renders/r13-current-shelter-v01/atlas-v01 --scale 8

python3 planning/minnesota/tools/check_winona_mcp.py \
  --pack content-packs/minnesota-r13-spring-shelter-v01.crystalpack \
  --client target/runtime-fps/wasm-dev/geothite
```

All pass. Native comparison covers2,949,120 complete pixels with zero
differences. The reached capture loads the actual controller save without
position/clock/facing overrides. Whole before/after, native camera sheet,
reached view and exterior atlas were personally inspected. All originals,
conversions and historical source epochs remain retained.
Fresh Winona New Game also passes guardian input ownership, starter
decline/retry/nickname choice, actual balls/survey handover, repeat interaction,
exterior return and Start with byte-empty stderr on this exact child.

Execution logs are `.local/r13-current-{publisher-build,publication,audit,audio,
controller-build,geometry,services,native,mcp,replay,atlas}-v01.log`.
The exact pack/source/binary/log bindings are recorded separately in the
verification receipt rather than treating older-parent tests as new runs.

## Repository check limits

The changed Rust leaf passes rustfmt. Required workspace formatting still
fails first at unrelated `offset-allocator/src/lib.rs:408`. Required normal
Bevy all-target Clippy stops on the existing core `bad_bit_mask` denial at
`crystal-core/src/multiplayer/mod.rs:1332`. These are failed workspace checks,
not a claim the checkout is lint-clean; their logs are retained as
`.local/r13-current-{workspace-fmt,clippy}-v01.log`.
Supplemental Bevy `--no-deps --lib --tests` Clippy passes in2m12s with the
existing `never_loop` denial allowed and no diagnostic in the changed test
leaf. It does not replace the failed required all-target run.
