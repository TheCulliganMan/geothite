# R03 native publication and verification

Run from the repository root. Outputs below are immutable; reruns require fresh
ignored pack, render and save paths. Rust owns all pack publication. The Python
tools only audit data, render installed art or drive production stdio input.

```sh
RUST_MIN_STACK=67108864 target/runtime-fps/wasm-dev/revise_authored_cave \
  content-packs/minnesota-northstar-current-story-v03.crystalpack \
  planning/minnesota/caves/johto-quality-v01/MnR03Cave1F.previous.json \
  planning/minnesota/caves/johto-quality-v01/MnR03Cave1F.json \
  content-packs/minnesota-r03-bank-seep-v02.crystalpack

export PYTHONPATH="$PWD/.local/ely-backroute-python${PYTHONPATH:+:$PYTHONPATH}"
python3 planning/minnesota/tools/audit_current_cave.py \
  --contract planning/minnesota/caves/current-r03-v01/native.contract.json \
  --pack content-packs/minnesota-r03-bank-seep-v02.crystalpack \
  --output planning/minnesota/renders/r03-current-bank-v01/published-v02

python3 planning/minnesota/tools/review_cave_quality_native.py \
  --pack content-packs/minnesota-r03-bank-seep-v02.crystalpack \
  --binary target/runtime-fps/wasm-dev/examples/render_at_location \
  --room MnR03Cave1F \
  --output planning/minnesota/renders/r03-current-bank-v01/native-v02

target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-r03-bank-seep-v02.crystalpack --audio --desktop
```

These pass. File SHA256:
`c6fdc6bdad16f49b5a0cc8ea0efcf7f3500d2fa61e9494e69492d4b540e6a96a`;
native identity:
`6c54eeb1451cdec23133a7b397542ec24485d34b5d2c7404d9893c1f266ec8a9`.
The fresh `minnesota-r03-bank-seep-replay-v02.crystalpack` is byte-identical.
The strict publisher validates compact geography before publication and again
after native readback. Independent whole-pack equality permits only the
declared cave blocks/border/mirrors/identity. All703othermaps and every
unowned native program/catalog/runtime file/audio field remain exact.

## Original interaction regression

```sh
CRYSTAL_R03_BANK_CONTROLLER_PACK="$PWD/content-packs/minnesota-r03-bank-seep-v02.crystalpack" \
CRYSTAL_R03_BANK_SOURCE="$PWD/planning/minnesota/caves/johto-quality-v01/MnR03Cave1F.json" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  r03_bank_cave_controller_native_roof_walker_three_pages_great_ball_once_and_continue \
  --ignored --nocapture --test-threads=1
```

Pass:3.67seconds before the combined-test rebuild. No original R03 test or
engine behavior is changed. Initial empty party, money1000, local position and
deterministic RNG are explicit setup; all later movement/dialogue/pickup/
Continue use the production controller. Regional access is not earned here.
The rejected v01failure is retained; the fix moves the water, preserving the
original dry talking spot and all original assertions.
The unchanged case passed again on the final executable in3.09seconds.

## Combined cave build and checks

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo test --profile wasm-dev --locked -p crystal-bevy --lib --no-run \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.codegen-units=256
```

The combined test explicitly recognizes the native current-story mine's
prerequisite shaft. The prior failed all-nine run correctly rejected that
one-cell overlay rather than silently accepting terrain drift. The original
mine source stays unchanged; exact closed-block/callback/warp checks belong to
the composed story profile. Opening and battling through the lair remain the
separate story chapter's responsibility.

The final rebuild finished in2m22s. These final-build checks passed:

```sh
CRYSTAL_CAVE_QUALITY_PACK="$PWD/content-packs/minnesota-r03-bank-seep-v02.crystalpack" \
CRYSTAL_CAVE_QUALITY_SAVES="$PWD/planning/minnesota/renders/r03-current-bank-v01/reached-v03" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  cave_quality_controller_all_nine_cameras_stairs_drops_mouths_and_continue \
  --ignored --nocapture --test-threads=1

CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-r03-bank-seep-v02.crystalpack" \
CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR="$PWD/planning/minnesota/renders/r03-current-bank-v01/earned-opening-v02" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  story_events_earned_reed_battle_ivy_quest_and_real_egg_decline_retry_continue \
  --ignored --nocapture --test-threads=1
```

All-nine course:19.27seconds, nine rooms and68actual camera checkpoints. It
walks every declared viewpoint, tests all native stairs both ways, all authored
short hops/reverse blocks/ordinary returns, original item quantities/events/
input locks, Continue and every original outdoor mouth. In the current mine it
also proves the sealed shaft from its real upper approach(19,9); the earlier
test-course typo(19,12) pointed at rock and correctly failed. Both failed
all-nine epochs and all reached saves remain retained.

The earned opening passed in27.69seconds on the immediately preceding rebuild
(only the geometry test's probe coordinate changed afterward). It starts from
New Game, earns the Reed win and delivery, then verifies real egg refusal,
retry, once-only grant and Continue. It makes no party/progress/RNG/position
writes. This is an opening, not an uninterrupted full regional campaign.

## Actual stdio and reached native views

```sh
PYTHONPATH="$PWD/.local/ely-backroute-python" python3 \
  planning/minnesota/caves/current-r03-v01/check_mcp.py \
  --pack content-packs/minnesota-r03-bank-seep-v02.crystalpack \
  --save planning/minnesota/renders/r03-current-bank-v01/reached-v02/MnR03Cave1F-mouth.crystalsave \
  --output planning/minnesota/renders/r03-current-bank-v01/mcp-v02

CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576 RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/examples/render_at_location \
  --pack content-packs/minnesota-r03-bank-seep-v02.crystalpack \
  --save planning/minnesota/renders/r03-current-bank-v01/mcp-v02/seep-bank.crystalsave \
  --view 2d \
  --screenshot planning/minnesota/renders/r03-current-bank-v01/native-reached-v02/seep-bank.png

PYTHONPATH="$PWD/.local/ely-backroute-python" python3 \
  planning/minnesota/tools/review_cave_collection.py \
  --pack content-packs/minnesota-r03-bank-seep-v02.crystalpack \
  --output planning/minnesota/renders/r03-current-bank-v01/all-nine-whole-v03

PYTHONPATH="$PWD/.local/ely-backroute-python" python3 \
  planning/minnesota/tools/audit_outdoor_connections.py \
  --pack content-packs/minnesota-r03-bank-seep-v02.crystalpack \
  --output planning/minnesota/renders/r03-current-bank-v01/atlas-v02 --scale 8
```

Both stdio processes exit0with byte-empty stderr. Actual inputs verify original
walker pages/movement lock, both roof cells/blocked north, original item notice/
body removal, passive exterior revisit, reentry, Start and Continue on the
collected tile. The quantity/event assertions are separately controller-proven.
No MCP status field is invented to imply an inventory effect.

The reached bank and blind-bend captures load actual same-pack MCP saves with
no position, facing or clock overrides. Both were visually inspected, as were
the whole before/after room, all six native reference cameras and all-nine
whole-room collection. The first collection's undersized card clipped the
larger R10room; it remains retained as rejected v02. The reusable collection
renderer sizes cards from actual room dimensions; v03keeps every room uncropped
at32pixels per metatile. No terrain was resized or regenerated for that sheet.

The actual outdoor atlas still has97maps in one component, zero overlaps and
zero inconsistent placement cycles. Four planned hubs remain absent from this
selected lineage: Detroit Lakes, Moorhead, International Falls and Mille Lacs.
This is static seam evidence, not a newly earned regional Surf journey.

## Existing baseline limits

`cargo fmt --all -- --check` reports unrelated `offset-allocator` differences.
The touched cave test is independently formatted. The old generic Winona stdio
smoke script stops when the new authored rival battle takes input; it is not
an applicable complete-opening test for this story parent. Preserve that failed
log and use the existing story-aware earned-opening regression instead.

Normal all-target Bevy Clippy stops on the pre-existing
`crystal-core/src/multiplayer/mod.rs:1332` bad-bit-mask error. Supplemental
focused `--lib --tests --no-deps` Clippy passes in1m06s with the existing
`-A clippy::never_loop` allowance; no warning names the changed cave test.
Log hashes remain in the final pack-bound receipt. The supplemental command is:

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo clippy --locked --profile wasm-dev -p crystal-bevy --lib --tests --no-deps \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.codegen-units=256 \
  -- -A clippy::never_loop
```

No unrelated formatting or lint cleanup is included.

`world.json` now selects `r03_bank_seep_revision`. The selector update checks
the exact704-map story parent and complete world preimage immediately before
atomic replacement, retaining all sibling fields including R06's current audit.
The preserved preimage and final verification receipt are in
`renders/r03-current-bank-v01/verified-v02/`. No packs, builds or render outputs
are staged or committed.
