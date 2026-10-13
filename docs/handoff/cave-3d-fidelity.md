# Cave 3D fidelity handoff

Completed native fidelity audit: 2026-10-13, branch `feat/connected-johto-3d`.
Historical implementation checkpoint: `f055f28`. Completion uses the full
shared worktree and integrations below, not that commit alone.

## Goal and scope

The original goal is **high fidelity 3D**, not merely passing geometry tests. This
chat owns cave presentation; other chats own trees, outdoor scenery, actors, and
building interiors. The documented native cave-fidelity goal is complete in
the current worktree after source regressions and individual visual acceptance.

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

The completed audit covers Union Cave, Ice Path, Mount Silver, Whirl Islands,
Diglett's Cave, and Victory Road: all 22 maps in these six areas.

## Completion evidence

Fresh native and classic views were personally compared at matching positions,
facing, time, and game state for all 22 maps, plus nine focused landing, hatch,
summit, exit, and stair views. Previews used `CRYSTAL_PREVIEW_MUTE=1`, DX12,
zoom 2, two-second captures and a 1280x800 requested viewport (1920x1200 at
the host's 1.5 DPI). Dark Whirl Islands and Silver Room 1 comparisons used the
actual `STATUSFLAGS_FLASH` engine flag in both renderers.

| Area | Maps | Accepted improvements and preserved source structure |
| --- | ---: | --- |
| Union Cave | 3 | Mineral floor grain, enclosing rock, water edges and ladder support |
| Ice Path | 5 | Continuous frozen enclosure, visible feet at low ladder landings, complete ice and pit footprints, gradual platform apertures |
| Mount Silver | 4 | Original Room 2 seam removed; six chained flights, Room 3 summit/corridor/exit datums, crowns and enclosing relief preserved |
| Whirl Islands | 8 | Lower water datum and thin mineral rims, visible feet at B2F ladder, original shelves and enclosing rock |
| Diglett's Cave | 1 | Coherent mineral floor, intact shallow authored hatch footprints, supported actors and continuous tunnel walls |
| Victory Road | 1 | Full-width six-tread flights, closed stair sides, supported feet and the original full-tile approach gaps |

Final checks passed: 140 voxel cave tests (including all three canonical-source
regressions), eight Bevy cave runtime tests against the real TUI pack, and
13 terrain-batching tests. These test sets overlap; do not add their counts
as unique tests. The full native `new_bark_3d` example built successfully after
temporary investigation probes were removed. The exact source-test setup was:

```powershell
$env:CARGO_TARGET_DIR = 'target/cave-native'
$env:CRYSTAL_CAVE_EXPORT_ROOT = 'C:\Users\rrcul\Documents\Geothite-export-workspace'
$env:CRYSTAL_RENDER_TEST_PACK = (Resolve-Path 'content-packs/text-tui.crystalpack').Path
cargo test --locked --profile wasm-dev -p crystal-bevy -p crystal-voxel-view --all-features cave --lib -- --include-ignored
cargo test --locked --profile wasm-dev -p crystal-bevy -p crystal-voxel-view --all-features terrain_batches --lib
cargo build --locked --profile wasm-dev -p crystal-bevy -p crystal-voxel-view --all-features --example new_bark_3d
```

Native comparisons consumed the export owner's ignored
`content-packs/core-modular.unvalidated.crystalpack`, with unchanged SHA256
`391CE466C404C222529DD1E689856B5EC13D9205230256A66C6E2D9D3FAA04D4`.
This audit proves cave rendering and source topology; pack publication and
general gameplay acceptance remain with their respective goal owners.
Final ignored logs use the `cave-final-` prefix and captures use
`final-acceptance` or the focused `final-landing`, `final-stair-side`,
`final-summit`, `final-exit`, `final-upper-hatch`, `final-tunnel`, and
`final-middle-stair`/`final-last-stair` labels. Recreate these views after
future shared renderer changes; scratch evidence is not required source.

## Current implementation and last action

* `cave_crowns` proves bounded raised walking crowns from authored cap courses
  and adjoining plain dust. It currently enables **VictoryRoad,
  SilverCaveRoom2, and SilverCaveRoom3**. Proven upper crowns and their rim resolve to 16 source
  pixels; enclosing geology retains its separate 40/48-pixel datum. Unbounded
  or incomplete evidence does not lift ordinary floors.
* Complete source stair pictures in metatiles `$12/$30` become descents only
  when both landings are proven. Six full-width stone treads replace the
  two-by-two source picture, with beveled noses and no middle divider. No long
  approach trench is inserted. SilverCaveRoom2's unbranched three-flight chain
  uses its two complete 8-by-2 intermediate cap courses to connect the unchanged
  16-pixel crown and 6-pixel shelf at equal intervals. All six source stair
  pictures now have proven landings; unknown, branching, or incomplete chains
  retain their original geometry.
  SilverCaveRoom3's complete `$12/$36` flights connect its 16-pixel summit to
  the 6-pixel exit shelf through an 11-pixel corridor. That corridor is a
  bounded crown with exactly one incoming and one outgoing stair; it must not
  be flattened onto the summit or interpreted as an eight-by-two landing strip.
* `RockCorners` uses shared, map-stable contour fields. Stair sockets blend
  adjoining rock gradually into exact edges. Walking cap relief uses the
  resolved support plane, preventing the rock surface from burying feet and
  knees; taller enclosing rock retains sculpted relief. Both dungeon construction
  and final surface polishing must consume these same resolved shapes. Complete
  barred shelf motifs `$13/$37` at the proven 16-pixel crown also keep their cap
  on that support plane. Low stair socket sides use stone material ownership;
  enclosing walls retain their gradual reveal material. Proven low crown caps
  and closing faces also keep stone ownership, preventing enclosing-wall reveal
  from exposing a dark strip at a closed stair socket.
* The final stair cleanup now subtracts the exact opening from crossing
  triangles, interpolating attributes and preserving cutaway ownership. The
  previous triangle-center classification could leave fragments or holes.
* Cave boulder-field grouping and geological caps/sides are procedural source
  geometry, not optional placeholder asset fallbacks. Victory Road's palette
  pass is scoped to Victory Road; do not gray all cave palettes indiscriminately.
* Cave-water edge blocks `$25-$27`, like corner blocks `$3d/$3e`, retain the
  lower water datum. Their boundary glyphs create thin mineral rims rather than
  six-pixel walking shelves. In particular, `$27` must not raise a strip in
  front of the lower-floor ladder landing in WhirlIslandB2F.
* Cutaway support protection follows a nearly horizontal surface at the
  authoritative foot plane. A sloping enclosing-wall facet below the trainer's
  head is not support. Protecting those facets produced the original
  SilverCaveRoom2 blade-like seam; the fresh native view at `(15,18)`, zoom 2,
  orbit 0 now dissolves it while retaining the crown and stair approach.
  Frozen platforms use only the gradual body aperture above the supporting
  floor, retaining the rest of their complete footprint.
* Opaque crown triangles and revealing rock triangles cannot share material
  UV roles through a shared vertex. Split ownership boundaries without moving
  positions, normals, support, or authored cutaway metadata.
* The outer enclosure must not duplicate a complete frozen wall already
  supplied by the native source formation or its retained source halo.
  Incomplete or changed formations still need the enclosing cross-section.
  Static cave batching preserves source-tagged sculpted wall sides within
  their bounded relief of the real map edge. Dropping every outside triangle
  cropped that relief into jagged holes; outside floor padding still belongs
  exclusively to the separate enclosing rock.

The 2026-10-12 transfer stopped at `f055f28`. The completed continuation fixed
the seam's cutaway support guard, frozen enclosure and batching ownership,
material boundaries, hatch mesh alignment, and source-proven stair/support
datums, then inspected the final native/classic audit above.

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
and footing code. The current cave integration also uses `terrain_batches.rs`
for preserving sculpted boundary sides, and `mesh/frozen_wall.rs` for complete
frozen-wall ownership. These contain other workers' edits. They were deliberately
left for the coordinator/owners to commit. Do not assume `f055f28` alone is the
tested transferable state. Transfer the final coordinated branch tip and
confirm these integrations are committed before building on the new host.

In particular, `build_scene` must call crown resolution before footing is
finalized and the stair finish after earlier bank joins; `modeled_dungeons`
must construct `RockCorners::new_with_shapes` using resolved shapes. Keep
other workers' water, outdoor, actor, and interior changes intact.

## Historical 2026-10-12 transfer evidence

The following records the earlier transfer checkpoint. Its unresolved seam
and incomplete audit were superseded by the completion evidence above.

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

Acceptance is complete. For future renderer changes, repeat the source checks
and matched native/classic views above. The original
SilverRoom2 seam was caused by cutaway support protection reaching sloping
wall facets below the trainer's head; triangle trimming alone did not fix it.
Do not blindly enable crown resolution for every cave or flatten enclosing
rock to make tests pass. Remove temporary investigation instrumentation after
the final native verification and rerun the cave regressions; this completion
already removed the temporary probes and passed those final checks.

Preview audio must use `CRYSTAL_PREVIEW_MUTE=1`. **Never change the computer's
system volume.** Packs, generated PCM, build products, scratch renders, and
audit dumps must remain ignored and unstaged. No external exporter is required
for the procedural cave work; the compatible user-supplied pack and real
canonical model assets are required for the production preview.
