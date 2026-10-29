# Executed current Gunflint shelter publication

Use the exact accepted region-polish-final-v03 parent and both source hashes in the README.
Choose fresh ignored destinations on reruns; the commands below name immutable
executed outputs. Rust publishes the pack. Python only audits, renders native
source art and sends production MCP inputs.

```sh
RUST_MIN_STACK=67108864 target/runtime-fps/wasm-dev/revise_authored_cave \
  content-packs/minnesota-region-polish-final-v03.crystalpack \
  planning/minnesota/caves/johto-quality-v01/MnR12Cave1F.previous.json \
  planning/minnesota/caves/johto-quality-v01/MnR12Cave1F.json \
  content-packs/minnesota-region-polish-r12-v01.crystalpack

export PYTHONPATH="$PWD/.local/ely-backroute-python${PYTHONPATH:+:$PYTHONPATH}"
python3 planning/minnesota/caves/current-r12-v01/audit.py \
  --parent content-packs/minnesota-region-polish-final-v03.crystalpack \
  --pack content-packs/minnesota-region-polish-r12-v01.crystalpack \
  --output planning/minnesota/renders/r12-current-shelter-v01/region-published-v02

target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-region-polish-r12-v01.crystalpack --audio --desktop
```

All pass. The existing bounded publisher was freshly built in the preceding
Rock Cut work and includes current compact-geography pre/post/readback checks,
fresh ignored/untracked output requirements, native preimage certification and
exact strict readback. The same command with fresh output
`minnesota-region-polish-r12-replay-v01.crystalpack` reproduces the entire file byte
for byte. The independent audit compares the complete result to the exact
parent except the declared room blocks/border/mirrors/identity. It checks all32
water positions, the original mouth aperture and actual compact warp pairing.

## Production-controller checks

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo test --profile wasm-dev --locked -p crystal-bevy --lib --no-run \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.codegen-units=256

CRYSTAL_CAVE_QUALITY_PACK="$PWD/content-packs/minnesota-region-polish-r12-v01.crystalpack" \
CRYSTAL_CAVE_QUALITY_SAVES="$PWD/planning/minnesota/renders/r12-current-shelter-v01/region-reached-v01" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  cave_quality_controller_current_r12_cameras_stairs_drops_mouth_pickup_and_continue \
  --ignored --nocapture --test-threads=1

CRYSTAL_R12_SHELTER_CONTROLLER_PACK="$PWD/content-packs/minnesota-region-polish-r12-v01.crystalpack" \
CRYSTAL_R12_SHELTER_SOURCE="$PWD/planning/minnesota/caves/johto-quality-v01/MnR12Cave1F.json" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  bevy_shell::tests::r12_shelter::r12_shelter_controller_ \
  --skip r12_shelter_controller_original_mouth_three_stairs_two_hop_lanes_and_ordinary_returns \
  --ignored --nocapture --test-threads=1
```

The initial build completed in3m37s; an additional current-worktree incremental
build completed in3m21s. On the final-parent child, the complete-room geometry
case passes in4.19s and both effect/dialogue cases pass in4.67s. The earlier
R05-parent candidate passed the same cases in9.46s and10.45s respectively. It does not run the
historical fixed-probe geometry case against a source without that old metadata.
The new generic course exercises every actual final viewpoint, all stairs,
both lips/reverse/returns and the real mouth. Its all-nine caller is retained.

The two effects cases now resolve the current exterior from native warp data,
then require one of two exact reviewed profiles: compact MnR12C00R00(40,15/16)
or historical MnR12C01R00(2,23/24). Both verify the exact active/return indices,
reciprocal target, CAVE entrance and safe FLOOR. The code does not accept an
arbitrary newly moved door. It retains the native active-return behavior,
bounded arrival phases, stable Floor checks and all original item/dialogue/
PP/Continue assertions. No production dispatcher or gameplay rule changes.

## Native2D and stdio replay

```sh
python3 planning/minnesota/tools/review_cave_quality_native.py \
  --pack content-packs/minnesota-region-polish-r12-v01.crystalpack \
  --binary target/runtime-fps/wasm-dev/examples/render_at_location \
  --room MnR12Cave1F \
  --output planning/minnesota/renders/r12-current-shelter-v01/region-native-v01

CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo build --profile wasm-dev --locked -p geothite \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0

python3 planning/minnesota/caves/current-r12-v01/check_mcp.py \
  --pack content-packs/minnesota-region-polish-r12-v01.crystalpack \
  --save planning/minnesota/renders/r12-current-shelter-v01/region-reached-v01/MnR12Cave1F-mouth.crystalsave \
  --output planning/minnesota/renders/r12-current-shelter-v01/region-mcp-v01
```

All ten native frames pass with zero differences across complete640×576 images.
The room uses its forced NITE palette and native actors; one coherent water
phase is compared across each entire frame. Whole room, source/native camera
sheets and actually reached crossing/spring/exterior frames were inspected.
The production screenshots are never modified for a match.

The fresh stdio client build completes in3m14s. The replay verifies Otta's
visible dialogue/input lock, the cache approach, both actual drops and reverse
denial, ordinary uphill21/19-input returns, compact mouth exit/reentry and the
production Start/Pack surface. It exits0 with byte-empty stderr. Initial setup
is the actual reached empty-party save; all subsequent changes use native input.
The item PP effects are separately covered by the nonempty-party controller
cases, not inferred from this empty-party replay.

Reached headless captures use the save rather than location overrides:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576 RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/examples/render_at_location \
  --pack content-packs/minnesota-region-polish-r12-v01.crystalpack \
  --save planning/minnesota/renders/r12-current-shelter-v01/region-reached-v01/MnR12Cave1F-widest-traverse.crystalsave \
  --view 2d \
  --screenshot planning/minnesota/renders/r12-current-shelter-v01/region-native-reached-v01/shoulder-crossing.png
```

The same method captures `spring-bank` and the actually reached MCP exterior.
All originals, conversions, rejected passes and checkpoints remain retained.

The fresh New Game regression also passes on this exact final-parent child:

```sh
python3 planning/minnesota/tools/check_winona_mcp.py \
  --pack content-packs/minnesota-region-polish-r12-v01.crystalpack \
  --client target/runtime-fps/wasm-dev/geothite
```

It plays the guardian, starter decline/retry and nickname choice, native balls/
survey handover, repeat interaction, exterior return and Start with byte-empty
stderr. The current full exterior atlas was regenerated from this child:

```sh
python3 planning/minnesota/tools/audit_outdoor_connections.py \
  --pack content-packs/minnesota-region-polish-r12-v01.crystalpack \
  --output planning/minnesota/renders/r12-current-shelter-v01/region-atlas-v01 \
  --scale 12
```

It retains97 exterior maps in one seam component, with zero footprint overlaps
and inconsistent placement cycles. The actual whole atlas was inspected. Its
graph remains distinct from a new earned whole-region playthrough.

## Retained revisions and limits

The first current-lineage source/pass is saved under `source-v01`, with its
native pack `minnesota-r12-shelter-v01.crystalpack` and ten zero-difference
native frames under `native-v01`. Visual review then extended the attached
central shoulder. That second source is frozen in `source-v02`; its earlier
R05-parent pack is `minnesota-r12-shelter-v02.crystalpack`, SHA256
`acaba4929d5c0cf5a7dd5a2a613ba1dbe256dc0fb0d876149bc05a36bb376b65`,
with evidence under `published-v02`, `native-v02`, `reached-v02` and `mcp-v02`.

After the shared region cut was accepted, the same exact source was rebased
onto `minnesota-region-polish-final-v03.crystalpack`. All current commands and
primary review links above use that child and its new `region-*` proof epochs.
The final source audit renders both before/after directly from their actual
parent/child packs. It does not substitute the earlier R05-parent before image.
The parent city, river and Vergloom journey logs keep their original identity
bindings. Complete normalized native equality proves their entire maps,
programs and unowned data stay exact; it does not relabel those journeys as a
new earned playthrough on this child.

The first independent perimeter audit incorrectly demanded solid wall beside
the original edge ladder. Native cave07 has a retained two-tile aperture:
Floor16,23 and Ladder17,23. The corrected audit pins the exact unchanged07
source cell and both original quadrant permissions, with blocking rock on
every other edge tile. It does not permit arbitrary new edge Floor or WATER.
The rejected audit log remains `.local/r12-current-audit-v02.log`; the corrected
passing audit is `.local/r12-current-audit-v03.log`.

The old source's exterior66,23 and its2×3-chunk test grid belong to retired
Gunflint geography. Neither was republished. Current native coordinates and
their exact source provenance are explicit in `native.contract.json`.
The terrain revision preserves all events/programs and all690 other maps.
These local fixtures do not earn regional prerequisites, complete the campaign
or establish final3D visual acceptance. After the final-parent checks and visual
inspection, `world.json` selects `r12_shelter_revision` and retains the accepted
region-polish parent unchanged as a recoverable prior selection.

Changed Rust leaves pass rustfmt. Required workspace formatting still fails
first on existing `offset-allocator/src/lib.rs:408` differences. Normal Bevy
all-target Clippy with `location-tester` fails on the existing core
`bad_bit_mask` and voxel approximate-constant errors. Scoped Bevy
`--no-deps --lib --tests` Clippy passes in52.43s with the existing `never_loop`
allowance and no diagnostic in either changed test leaf. The whole workspace
is not reported lint-clean. Execution logs and exact hashes are bound in
`region-verified-v01/receipt.json`.
