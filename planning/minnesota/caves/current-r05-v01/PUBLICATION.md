# Executed Rock Cut publication

Use the exact completed R09 parent and both source hashes in the README.
Every output below is an immutable executed epoch; use fresh ignored names
for another run. Native publication uses Rust. Python only reads packs,
audits their preserved values and renders existing native source art.

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo build --profile wasm-dev --locked -p crystal-assets \
  --bin revise_authored_cave \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0

RUST_MIN_STACK=67108864 target/runtime-fps/wasm-dev/revise_authored_cave \
  content-packs/minnesota-r09-rock-mine-v01.crystalpack \
  planning/minnesota/caves/johto-quality-v01/MnR05RockCut.previous.json \
  planning/minnesota/caves/johto-quality-v01/MnR05RockCut.json \
  content-packs/minnesota-r05-rock-cut-replay-v01.crystalpack

cmp content-packs/minnesota-r05-rock-cut-v01.crystalpack \
  content-packs/minnesota-r05-rock-cut-replay-v01.crystalpack

export PYTHONPATH="$PWD/.local/ely-backroute-python${PYTHONPATH:+:$PYTHONPATH}"
python3 planning/minnesota/caves/current-r05-v01/audit.py \
  --parent content-packs/minnesota-r09-rock-mine-v01.crystalpack \
  --pack content-packs/minnesota-r05-rock-cut-v01.crystalpack \
  --output planning/minnesota/renders/r05-current-rockcut-v01/published-v01

target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-r05-rock-cut-v01.crystalpack --audio --desktop
```

The initial publication used the existing bounded cave publisher. Its fresh
build after adding ignored/fresh output checks and the detailed compact guard
produced the same entire file bytes. Complete normalized native equality,
both actual mouth pairings, solid perimeter and all377 audio checks pass.
The API certifies the retained native programs and uncached source payloads
before changing terrain; no replacement healer or item script is lowered.

## Fresh controller build and actual input

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo test --profile wasm-dev --locked -p crystal-bevy --lib --no-run \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.codegen-units=256

CRYSTAL_CAVE_QUALITY_PACK="$PWD/content-packs/minnesota-r05-rock-cut-v01.crystalpack" \
CRYSTAL_CAVE_QUALITY_SAVES="$PWD/planning/minnesota/renders/r05-current-rockcut-v01/reached-v01" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  cave_quality_controller_current_r05_cameras_stair_mouths_pickup_and_continue \
  --ignored --nocapture --test-threads=1

CRYSTAL_R05_ROCKCUT_CONTROLLER_PACK="$PWD/content-packs/minnesota-r05-rock-cut-v01.crystalpack" \
CRYSTAL_R05_ROCKCUT_SOURCE="$PWD/planning/minnesota/caves/johto-quality-v01/MnR05RockCut.json" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  bevy_shell::tests::r05_rockcut::r05_rockcut_controller_ \
  --ignored --nocapture --test-threads=1
```

The fresh build completed in2m47s. Geometry1passed in7.00s and the original
service/effect2cases passed in7.91s. An earlier read-only execution of the
completed shared route-grass test binary also passed both services, retained
separately as `.local/r05-current-services-controller-v01.log`. The latest
results above are bound to the own fresh executable and source.

The new single-room wrapper invokes the same generic cave course as the
all-nine case. All-nine coverage remains intact; the wrapper does not claim
that six other caves already contain their source revisions on this branch.
The original service test consumes the canonical source through its existing
environment override, rather than silently checking the older frozen source.

## Native 2D presentation

```sh
python3 planning/minnesota/tools/review_cave_quality_native.py \
  --pack content-packs/minnesota-r05-rock-cut-v01.crystalpack \
  --binary target/runtime-fps/wasm-dev/examples/render_at_location \
  --room MnR05RockCut \
  --output planning/minnesota/renders/r05-current-rockcut-v01/native-v01

CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576 RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/examples/render_at_location \
  --pack content-packs/minnesota-r05-rock-cut-v01.crystalpack \
  --save planning/minnesota/renders/r05-current-rockcut-v01/reached-v01/MnR05RockCut-bent-neck.crystalsave \
  --view 2d \
  --screenshot planning/minnesota/renders/r05-current-rockcut-v01/native-reached-v01/bent-neck.png
```

All six native frames match the entire source640×576 view with zero differing
pixels. The room is forced NITE; a daytime source swatch is not substituted.
The same reached command for `north-mouth` also passes. Whole room, camera
sheets and both reached native frames were visually inspected. The actual
images were not edited to obtain agreement.

## Production stdio replay

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo build --profile wasm-dev --locked -p geothite \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0

python3 planning/minnesota/caves/current-r05-v01/check_mcp.py \
  --pack content-packs/minnesota-r05-rock-cut-v01.crystalpack \
  --save planning/minnesota/renders/r05-current-rockcut-v01/reached-v01/MnR05RockCut-north-mouth.crystalsave \
  --output planning/minnesota/renders/r05-current-rockcut-v01/mcp-v02
```

The client build completed in2m33s. The fresh client exits/reenters both actual
mouths and walks the18-input ordinary-floor/stair course between clear arrivals.
It exposes the production Start/Pack menu and exits0 with byte-empty stderr.
Its pack, client and actual reached-save hashes are retained in the receipt.
Reentry lands at native ladder(11,5) or(5,17), followed by an ordinary Up step
to the clear floor. The proof never rewrites position or substitutes passive
clear arrivals for the real target warp coordinates.

The first probe used the older06:43 executable and reached the north ladder
without exiting immediately. A diagnostic floor circuit then exited correctly
on that same old client. The fresh current client immediately takes the warp
from the same reached save. Both diagnostics remain retained; this was resolved
by rebuilding the actual supported client, without changing geometry, warps or
the controller implementation. The replay also corrected its own expected
reentry coordinate from clear floor to the actual native ladder. The initial
rejected probe and all saved outputs remain in `mcp-v01`.

## Publication guards and check scope

The command-line publisher now rejects existing outputs before loading source,
requires an ignored/untracked destination, checks
`validate_compact_superior_geography` before/after revision and after strict
readback, and requires the written pack to equal the built pack. The library
terrain revision API remains unchanged for historical pack/fixture use.
The old nine-cave commands in the historical Johto-quality review describe a
completed old branch; the current CLI intentionally rejects its retired coast.

All six actual negative cases pass and leave no new pack: retain existing
publication; reject unignored output; reject the older Voyageurs pack's retired
northern geometry; reject walkable border; reject changed Repel quantity; reject
an extra unpaired ladder. Temporary source mutations were removed with their
temporary directory. The small bound rejection receipt is retained in
`guard-cases-v01/receipt.json`.

Changed Rust leaves pass rustfmt. Required workspace formatting still fails
first on existing `offset-allocator/src/lib.rs:408` differences. Normal
affected-crate all-target Clippy with `crystal-bevy/location-tester` still
fails on the existing core `bad_bit_mask` and voxel approximate-constant errors.
No unrelated formatting or core/voxel implementation was changed.
The affected-crate scoped `--no-deps --lib --tests --bin revise_authored_cave`
Clippy check passes in1m05s with the existing `never_loop` allowance. It reports
no diagnostic in the changed cave publisher or single-room controller case.
This does not make the full workspace lint-clean.

These are explicit local fixtures, not a new earned campaign playthrough or
final3D acceptance. Further cave candidates stay separate from the shared
region integration until their own preservation, visual and gameplay checks
are complete. The active world pointer is not changed by these commands.
