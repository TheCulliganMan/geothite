# Cave 3D fidelity handoff

Checkpoint: 2026-10-12, branch `feat/connected-johto-3d`.
Implementation checkpoint: `f055f28` (Improve cave crowns, stair joins, and sculpted rock footing).

## Goal and scope

The original goal is **high fidelity 3D**, not merely passing geometry tests. This
chat owns cave presentation; other chats own trees, outdoor scenery, actors, and
building interiors. The goal remains unfinished. Ryan requested a transfer
checkpoint and deferred further visual improvements to the new host.

The requested cave direction is sculpted, contiguous rock with tall enclosing
walls; credible elevated crowns, ramps, and stairs; merged rocky boulder fields;
and coherent floor heights. Use the supplied ice-cavern reference's broad rocky
platforms, continuous mineral sides, and blended joins. Preserve the original
2D map topology and the full tile between the player and Victory Road's stair.
Players stand on crowns rather than in their sides. Ladders rise above their
correct supporting floor. Wall cutaways must fade gradually over a broad area,
with the front wall mostly transparent when obstructing the player, while
retaining surrounding rock rather than exposing outdoor green/empty terrain.
Dim actual cavern lights; do not disguise incorrect geometry with darkness.

Named areas needing attention include Union Cave, Ice Path, Mount Silver,
Whirl Islands, Diglett's Cave, and Victory Road. Existing work across those
areas is not a verified completion audit for all caverns.

## Current implementation and last action

* `cave_crowns` proves bounded raised walking crowns from authored cap courses
  and adjoining plain dust. It currently enables **VictoryRoad and
  SilverCaveRoom2 only**. Proven crowns and their rim resolve to 16 source
  pixels; enclosing geology retains its separate 40/48-pixel datum. Unbounded
  or incomplete evidence does not lift ordinary floors.
* Complete source stair pictures in metatiles `$12/$30` become descents only
  when both landings are proven. Six full-width stone treads replace the
  two-by-two source picture, with beveled noses and no middle divider. No long
  approach trench is inserted.
* `RockCorners` uses shared, map-stable contour fields. Stair sockets blend
  adjoining rock gradually into exact edges. Walking cap relief uses the
  resolved support plane, preventing the rock surface from burying feet and
  knees; taller enclosing rock retains sculpted relief.
* The final stair cleanup now subtracts the exact opening from crossing
  triangles, interpolating attributes and preserving cutaway ownership. The
  previous triangle-center classification could leave fragments or holes.
* Cave boulder-field grouping and geological caps/sides are procedural source
  geometry, not optional placeholder asset fallbacks. Victory Road's palette
  pass is scoped to Victory Road; do not gray all cave palettes indiscriminately.

Last actions: committed the 13 cave-owned files as `f055f28`, inspected fresh
native Mount Silver and Victory Road renders, removed owned temporary test and
preview executables, and stopped implementation edits for transfer. This note
is the requested durable restart record, not another improvement cycle.

## Ownership and shared integration

The implementation commit owns:

```
crates/crystal-voxel-view/src/cave.rs
crates/crystal-voxel-view/src/mesh/cave_cliff.rs
crates/crystal-voxel-view/src/mesh/cave_corners.rs
crates/crystal-voxel-view/src/mesh/cave_enclosure.rs
crates/crystal-voxel-view/src/mesh/cave_finish.rs
crates/crystal-voxel-view/src/mesh/cave_hatch.rs
crates/crystal-voxel-view/src/mesh/cave_shore.rs
crates/crystal-voxel-view/src/mesh/cave_sill.rs
crates/crystal-voxel-view/src/mesh/cave_stones.rs
crates/crystal-voxel-view/src/mesh/cave_boulder_fields.rs
crates/crystal-voxel-view/src/mesh/cave_crowns.rs
crates/crystal-voxel-view/src/mesh/kanto_cave_palette.rs
```

It also deletes obsolete `mesh/cave_flight.rs`.

The tested **complete worktree** also needs shared integrations in `mesh.rs`,
`mesh/build_scene.rs`, `mesh/modeled_dungeons.rs`, `mesh/new_bark.rs`,
`voxel_surface.wesl`, the renderer's light/cutaway paths, and related profile
and footing code. These contain other workers' edits. They were deliberately
left for the coordinator/owners to commit. Do not assume `f055f28` alone is the
tested transferable state. Transfer the final coordinated branch tip and
confirm these integrations are committed before building on the new host.

In particular, `build_scene` must call crown resolution before footing is
finalized and the stair finish after earlier bank joins; `modeled_dungeons`
must construct `RockCorners::new_with_shapes` using resolved shapes. Keep
other workers' water, outdoor, actor, and interior changes intact.

## Verified checks and limitations

The following native build completed successfully against the full current
worktree. It emitted warnings but no build errors:

```sh
CRYSTAL_PREVIEW_MUTE=1 \
CARGO_TARGET_DIR="$PWD/target-bevy20-verify" \
CARGO_INCREMENTAL=0 CARGO_PROFILE_WASM_DEV_DEBUG=0 CARGO_BUILD_JOBS=2 \
cargo build --locked --profile wasm-dev \
  -p geothite -p crystal-bevy -p crystal-voxel-view -p crystal-render-api \
  --all-features --example new_bark_3d
```

The cave test harness was compiled directly from the actual Rust crate source
with Rust 2024, `--test`, `operation-trace` and `people-review`, and the native
build's dependency rlibs. It did not substitute shim implementations. Actual
successful harness invocations were:

```sh
target/cave-review/stair-face-trim-tests cave --test-threads=1
# 129 passed, 0 failed
target/cave-review/stair-face-trim-final-tests mesh::cave_crowns::tests --test-threads=1
# 6 passed, 0 failed against the final source
```

These temporary executables were removed. Old-host ignored logs are
`target/cave-review/stair-face-trim-tests.log`,
`stair-face-trim-final-tests.log`, and
`stair-face-trim-native-build-fixed.log`. They are evidence, not deliverables.
The checks cover cap/support agreement, stair footprint and approach gap,
landing continuity, winding, exact socket subtraction and retained area,
cutaway ownership, and preservation of enclosing rock.

Fresh native captures were viewed, not merely checked for file existence:

* `VictoryRoad-stair-face-trim.png`: feet visibly planted, six continuous
  treads, source approach gap preserved.
* `SilverCaveRoom2-stair-face-trim.png`: both shoes visible above the walking
  surface; **an adjoining rock/stair-side seam remains visibly unresolved**.

Those PNGs and matching logs under `target/cave-review/` are ignored old-host
evidence and are not guaranteed to transfer. Earlier classic captures include
`SilverCaveRoom2-fidelity-audit-2d.png` and
`VictoryRoad-short-stairs-gap-2d.png`; recreate them on the new host.

An initial native compile failed because the clipping code lacked explicit
`Vec2/Vec4` imports; that was fixed and the native build then passed. A final
direct test compile encountered a newly added concurrent test's
`crystal_assets` dependency; adding that existing dependency to the harness
invocation resolved it and the six final checks passed. These are resolved
verification failures, not active blockers.

No complete all-map visual audit, browser cave review, or controller gameplay
proof was performed for this checkpoint. Geometry tests and screenshots do
not establish TUI/MCP playability. The overall fidelity goal is not complete.

## Resume on the new host

Follow [the transfer guide](../cold-start-3d.md) and use the coordinator's
`scripts/cold-start-3d.sh`; do not create a competing cold-start script.
Restore the final Git branch, real Git LFS model/texture objects, and the
separately supplied ignored content packs. The cave reviews used
`content-packs/core-modular.browser.crystalpack`; the cold-start script's
default pack may differ, so explicitly select this reviewed pack when
reproducing evidence.

```sh
git switch feat/connected-johto-3d
git status --short
git lfs checkout
CRYSTAL_PACK="$PWD/content-packs/core-modular.browser.crystalpack" \
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
scripts/cold-start-3d.sh check

# Portable source-test route to run on the new host; this exact Cargo
# invocation was not the old-host direct-rustc harness invocation above.
CRYSTAL_PREVIEW_MUTE=1 cargo test --locked -p crystal-voxel-view \
  --all-features cave -- --test-threads=1
```

Recreate the same views through the preview script:

```sh
mkdir -p target/cave-review
CRYSTAL_PACK="$PWD/content-packs/core-modular.browser.crystalpack" \
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
CRYSTAL_BUILD_PROFILE=wasm-dev CRYSTAL_PREVIEW_MUTE=1 \
CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
scripts/cold-start-3d.sh preview --map SilverCaveRoom2 --x 15 --y 18 \
  --zoom 2 --orbit 0 --size 1280x800 --seconds 2 \
  --screenshot target/cave-review/silver-resume.png

CRYSTAL_PACK="$PWD/content-packs/core-modular.browser.crystalpack" \
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
CRYSTAL_BUILD_PROFILE=wasm-dev CRYSTAL_PREVIEW_MUTE=1 \
CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
scripts/cold-start-3d.sh preview --map VictoryRoad --x 5 --y 35 \
  --zoom 2 --orbit 2 --size 1280x800 --seconds 2 \
  --screenshot target/cave-review/victory-resume.png
```

Add `--classic` with the **same coordinates** for the matching 2D comparison.
`CRYSTAL_CAVE_SURFACE_TRACE=1` enables bounded source/support diagnostics;
`CRYSTAL_CAVE_RENDER_TRACE=1` diagnoses surface finishing. Keep diagnostics
opt-in; never add unconditional output to fullscreen gameplay paths.

Next action: authenticate the source cells and resolved shapes at the visible
SilverRoom2 stair-side seam before changing more heights. Exact triangle
trimming did not eliminate that visible seam; do not claim it did. Check
whether adjoining diagonal geometry still uses a tall enclosing datum or the
visible source motif was not admitted by the descent proof. These are
**hypotheses**, not established causes. Fix the owning geometry, then check
both classic topology and native views while keeping actor support exact.
After that, review the other named caverns individually and improve rocky
field forms and material blending. Do not blindly enable crown resolution
for every cave or flatten enclosing rock to make tests pass.

Preview audio must use `CRYSTAL_PREVIEW_MUTE=1`. **Never change the computer's
system volume.** Packs, generated PCM, build products, scratch renders, and
audit dumps must remain ignored and unstaged. No external exporter is required
for the procedural cave work; the compatible user-supplied pack and real
canonical model assets are required for the production preview.
