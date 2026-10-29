# Executed R10 publication

Run from the repository root with the exact completed waterfront parent,
SHA256 `3862c34b044a456721b5f7ee5464b42dd123f07e3bd12d16e7e503bec972ffa2`.
Check the two source hashes in the README before another publication. Use
fresh ignored output paths; existing packs and render epochs are retained.

```sh
export PYTHONPATH="$PWD/.local/ely-backroute-python${PYTHONPATH:+:$PYTHONPATH}"
python3 planning/minnesota/tools/review_cave_quality.py \
  --pack content-packs/minnesota-waterfront-frontages-v01.crystalpack \
  --room MnR10Cave1F \
  --output planning/minnesota/renders/iron-cave-quality-v01/source-v02

CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo build --profile wasm-dev --locked -p crystal-assets --bin revise_authored_cave \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0

RUST_MIN_STACK=67108864 target/runtime-fps/wasm-dev/revise_authored_cave \
  content-packs/minnesota-waterfront-frontages-v01.crystalpack \
  planning/minnesota/caves/johto-quality-v01/MnR10Cave1F.previous.json \
  planning/minnesota/caves/johto-quality-v01/MnR10Cave1F.json \
  content-packs/minnesota-r10-rock-gallery-v01.crystalpack

python3 planning/minnesota/caves/current-iron-v01/audit.py \
  --pack content-packs/minnesota-r10-rock-gallery-v01.crystalpack \
  --output planning/minnesota/renders/iron-cave-quality-v01/published-v02

target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-r10-rock-gallery-v01.crystalpack --audio --desktop
```

The initial publication used the existing Rust publisher. After rebuilding it
from current source, the same command with fresh
`minnesota-r10-rock-gallery-replay-v01.crystalpack` produced identical complete
file bytes. Rust verifies the native source preimage, validates revised cave
geometry and strictly reads back the written pack. The independent read-only
audit proves the full result equals the parent except this exact room's
terrain/border/mirrors/identity. No Python tool writes or patches a pack.

## Production input

The fresh test build used the pinned toolchain:

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo test --profile wasm-dev --locked -p crystal-bevy --lib --no-run \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.codegen-units=256

export RUST_MIN_STACK=67108864
export CRYSTAL_CAVE_QUALITY_PACK="$PWD/content-packs/minnesota-r10-rock-gallery-v01.crystalpack"
export CRYSTAL_R10_GALLERY_CONTROLLER_PACK="$CRYSTAL_CAVE_QUALITY_PACK"
export CRYSTAL_CAVE_QUALITY_SAVES="$PWD/planning/minnesota/renders/iron-cave-quality-v01/reached-v01"

target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  cave_quality_controller_r10_cameras_stairs_drops_mouth_and_continue \
  --ignored --nocapture --test-threads=1
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  bevy_shell::tests::r10_gallery::r10_gallery_controller_ \
  --ignored --nocapture --test-threads=1
```

The new single-room case passes6.53s, and the three original effects/traversal
cases pass6.80s. The existing all-nine geometry test still calls the same
course with all nine explicit room IDs; its scope was not reduced. The new
single-room case adds optional immutable reached-camera saves. No production
gameplay code or permission guard is changed.

## Native visual review

```sh
python3 planning/minnesota/tools/review_cave_quality_native.py \
  --pack content-packs/minnesota-r10-rock-gallery-v01.crystalpack \
  --binary target/runtime-fps/wasm-dev/examples/render_at_location \
  --room MnR10Cave1F \
  --output planning/minnesota/renders/iron-cave-quality-v01/native-v01
```

All nine complete640×576 frames match original native art with one coherent
water phase per frame. The production pixels are never edited. The actual
room's NITE, camera clamp, facing, native border and object priority are used
for comparison. Whole room, nine-camera sheet, critical full frames and three
actually reached native frames were inspected. The reached captures use:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576 RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/examples/render_at_location \
  --pack content-packs/minnesota-r10-rock-gallery-v01.crystalpack \
  --save planning/minnesota/renders/iron-cave-quality-v01/reached-v01/MnR10Cave1F-spring-bank.crystalsave \
  --view 2d \
  --screenshot planning/minnesota/renders/iron-cave-quality-v01/native-reached-v01/spring-bank.png
```

The gallery's source and forced native palette are unchanged from the earlier
reviewed branch. Only R10 is composed into the current waterfront parent here;
the other eight native caves are not silently imported by this command.

The existing production `target/northstar-story/wasm-dev/geothite mcp` client
was launched with this pack and `reached-v01/MnR10Cave1F-widest-fork.crystalsave`.
Its real Start menu exposes Pack; native movement goes(11,9)→(10,9)→(11,9).
The session exits0 with byte-empty stderr. Its autosave and receipt are retained
in `renders/iron-cave-quality-v01/mcp-v01/`; this is an explicit reached local
fixture, not a New Game campaign claim.

## Retained attempts and check limits

The older single-cave preview helper expected obsolete `loop_waypoints_core`
metadata and rejected the current source before drawing. The current cave
review tool now supports selecting one complete room. The first new audit
incorrectly assumed the spring extent stayed fixed. The actual reviewed source
retains40 old water cells and deliberately adds24; the corrected audit pins
those exact coordinates and both changed old wet-bank metatiles, then proves
the complete rock perimeter. It does not relax a native publisher guard or
change the reviewed geometry to satisfy that assumption.

The first fresh controller build encountered a transient shared-source
`installed_chunks` visibility error in two other asset modules. The function
was already `pub(crate)` when inspected; a fresh build against the current
worktree passed. No unrelated visibility or validation code was edited here.

The changed Rust leaf passes rustfmt and diff checks. Required workspace
formatting still fails on existing unrelated differences, beginning in
`offset-allocator/src/lib.rs:408`. Normal affected-crate all-target Clippy with
`location-tester` still fails on existing core `bad_bit_mask` and voxel
`approx_constant` errors. The scoped `--no-deps --lib --tests` check with the
existing `never_loop` allowance passes, with no diagnostic in the changed
cave test. Logs are retained under `.local/iron-cave-*`.

The local native candidate is verified for this room, its named interactions,
preservation and 2D presentation. These fixtures do not earn regional access,
complete the campaign or establish final3D acceptance.
