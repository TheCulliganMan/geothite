# R09 current-story publication runbook

Run from the repository root with the exact R10 parent and reviewed source
hashes in the README. All packs and render outputs stay ignored; keep previous
versions. No Python tool writes or patches a pack.

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo build --profile wasm-dev --locked -p crystal-assets \
  --bin pack_current_r09_mine \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0

RUST_MIN_STACK=67108864 target/runtime-fps/wasm-dev/pack_current_r09_mine \
  content-packs/minnesota-r10-rock-gallery-v01.crystalpack \
  planning/minnesota/caves/johto-quality-v01/MnR09Cave1F.previous.json \
  planning/minnesota/caves/johto-quality-v01/MnR09Cave1F.json \
  planning/minnesota/caves/current-r09-v01/native.contract.json \
  01d6f0f59ab69a73fce679cfcf6587426690469da64a91bd3cb8d5a44eb688f3 \
  content-packs/minnesota-r09-rock-mine-replay-v02.crystalpack

cmp content-packs/minnesota-r09-rock-mine-v01.crystalpack \
  content-packs/minnesota-r09-rock-mine-replay-v02.crystalpack
```

The fresh guarded publisher build and replay passed; complete files have the
same SHA256 `b8178a3de63a7aab68280e6ed3646bbfb28e1c168315139aeeca5f2ff0ede14d`.
The publisher requires a fresh ignored/untracked destination, validates the
parent file and complete native identity, and strictly reads back the result.

## Source, preservation, assets and audio

```sh
export PYTHONPATH="$PWD/.local/ely-backroute-python${PYTHONPATH:+:$PYTHONPATH}"
python3 planning/minnesota/caves/current-r09-v01/audit.py \
  --pack content-packs/minnesota-r09-rock-mine-v01.crystalpack \
  --output planning/minnesota/renders/r09-current-mine-v01/published-v01

CRYSTAL_CURRENT_R09_PARENT="$PWD/content-packs/minnesota-r10-rock-gallery-v01.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo test --profile wasm-dev --locked -p crystal-assets --features test-fixtures \
  --test current_r09_mine \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  -- --ignored --test-threads=1

target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-r09-rock-mine-v01.crystalpack --audio --desktop
```

All passed. The successful asset run is `.local/r09-current-assets-tests-v02.log`
(3passed,155.25s); audio checks377 programs. The read-only audit compares the
entire decoded native result against the parent with only the declared room
terrain/border/mirrors/identity substituted. This includes current native
dialogue, teams/moves and split payload indexes, rather than only geometry.

## Production input and native presentation

The completed shared test build is
`target/route-grass/wasm-dev/deps/crystal_bevy-8ddf5a560e93701b`. It was consumed
read-only after its owner confirmed completion; its test source includes the
current R09 single-room case and exact regional-party expectations. No other
owner's cache or active compiler was modified. Rebuild with the pinned
toolchain when needed and use Cargo's actual reported executable path.

```sh
CRYSTAL_CAVE_QUALITY_PACK="$PWD/content-packs/minnesota-r09-rock-mine-v01.crystalpack" \
CRYSTAL_CAVE_QUALITY_SAVES="$PWD/planning/minnesota/renders/r09-current-mine-v01/reached-v01" \
RUST_MIN_STACK=67108864 \
target/route-grass/wasm-dev/deps/crystal_bevy-8ddf5a560e93701b \
  cave_quality_controller_current_r09_cameras_stairs_drops_mouth_and_continue \
  --ignored --nocapture --test-threads=1

CRYSTAL_IRON_RANGE_STORY_TEST_PACK="$PWD/content-packs/minnesota-r09-rock-mine-v01.crystalpack" \
CRYSTAL_IRON_RANGE_STORY_SAVE_DIR="$PWD/planning/minnesota/renders/r09-current-mine-v01/story-reached-v01" \
RUST_MIN_STACK=67108864 \
target/route-grass/wasm-dev/deps/crystal_bevy-8ddf5a560e93701b \
  bevy_shell::tests::iron_range_story::iron_range_story_controller_ \
  --ignored --nocapture --test-threads=1

python3 planning/minnesota/tools/review_cave_quality_native.py \
  --pack content-packs/minnesota-r09-rock-mine-v01.crystalpack \
  --binary target/runtime-fps/wasm-dev/examples/render_at_location \
  --room MnR09Cave1F \
  --output planning/minnesota/renders/r09-current-mine-v01/native-v01
```

Geometry1passed in7.47s; story3passed in42.97s. Eight source/native cameras
passed with zero pixel differences. Their fixture is initially empty-party,
south-facing at noon, with the room's forced native NITE palette. Actual
reached captures instead load the retained gameplay saves:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576 RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/examples/render_at_location \
  --pack content-packs/minnesota-r09-rock-mine-v01.crystalpack \
  --save planning/minnesota/renders/r09-current-mine-v01/reached-v01/MnR09Cave1F-shaft-turn.crystalsave \
  --view 2d \
  --screenshot planning/minnesota/renders/r09-current-mine-v01/native-reached-v01/shaft-turn.png
```

The same reached command for `fitted-bench` also passed. Whole room, all eight
native cameras and both reached frames were visually inspected. These are
2D and explicit local fixtures; no earned campaign or final3D claim follows.

The latest worktree was then independently rebuilt, including the current
runtime changes from other threads:

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo test --profile wasm-dev --locked -p crystal-bevy --lib --no-run \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.codegen-units=256
```

It completed successfully in3m42s and produced
`target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652`, SHA256
`66ce9b82b9d101262f8308f20e157075bd620a6b54bf0e823d90b8168d27b6a8`.
Repeating the same geometry and story commands with this executable and fresh
`reached-v02` / `story-reached-v02` directories passed1case in16.17s and3cases
in44.85s. The bound receipt records these latest test logs and their hashes.
The earlier saves and screenshots remain unchanged.

The production `target/northstar-story/wasm-dev/geothite mcp` client loaded the
latest `reached-v02/MnR09Cave1F-shaft-turn.crystalsave`, opened the actual
Start/Pack surface, refused three westward inputs into block20's WALL quadrant,
then walked(9,18)→(10,18)→(9,18). It exited0 with byte-empty stderr. Its pack,
client and reached-save hashes are bound in `mcp-v02/receipt.json`; the autosave
is retained alongside it. This proves local input through the production
controller, not earned campaign progression.

## Retained failures and check limits

The original `revise_r09_mine` correctly rejected the current faction-text
preimage with `Q06 route/story text differs: MnIronRangeForemanSeenText`.
It lowers a historical fixed team and story, so its guards were not loosened
to overwrite the current regional mine. The current-only wrapper instead
preserves the exact installed story and team. The legacy certifier and geometry
checks remain intact; only two helpers became `pub(crate)` for safe reuse.

The shared guard was initially placed at the existing migration validator's
filename, producing a missing `validation::validate` compile error. That
collision was corrected: `compact_superior_validation.rs` is restored unchanged;
the new public guard is `compact_superior_geography_validation.rs`. The first
asset test then caught the new placement guard traversing original Kanto
Route14/13. Its scope now explicitly selects Minnesota outdoor maps, matching
the native atlas. The second complete asset run passes. Both failures are
retained and are not described as unrelated baseline failures.

The old straight-shaft R09 helper does not cover this revised lower turn. The
generic cave quality course now handles a native stair's side exit, and the
new single-room wrapper runs that same course without weakening all-nine
coverage. Story tests accept only the exact original team or exact current
regional team and ordered levels; the loss check still requires the installed
first opponent and rejects opponents outside that team.

The first stdio probe expected x8,y18 to be floor. The actual revised block20
has a WALL there; production correctly stayed at x9,y18. The rejected probe's
save and explanation remain in `mcp-v01`. The successful probe checks that
denial, then walks on the actual east floor. No terrain or movement rule was
changed to satisfy the mistaken probe.

Changed Rust leaves pass rustfmt. The required workspace `cargo fmt --all --
--check` was executed and still fails first on existing differences in
`offset-allocator/src/lib.rs:408`; unrelated files were not reformatted.
Required affected-crate all-target Clippy with `crystal-bevy/location-tester`
was executed and fails on existing core `bad_bit_mask` and three voxel errors.
The scoped affected-crate `--no-deps --lib --tests` check with the existing
`-A clippy::never_loop` allowance passes in1m52s. It reports no diagnostic in
the current mine publisher, shared detailed guard, asset regression, cave
quality course or foreman story test. These scoped checks do not make the
whole workspace lint-clean.
Logs and rejected renders remain under `.local/r09-current-*` and the versioned
render directories. Wider map quality and integration remain ongoing.
