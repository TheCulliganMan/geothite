# Original Gym scenery kit

This kit replaces four reviewed source families with original closed geometric
art: oak display planters with inset soil and two foliage crowns, Azalea's broad
round tree with a branching trunk and buttress roots, a clipped Celadon hedge,
and connected low Viridian masonry with coping, inset panels and corner trim.
Every named Blender part is editable. No source pixels, pack files, map scripts,
collision definitions, or duplicate game catalogs are shipped with the kit.

## Exact source ownership

| Map | Complete drawings or networks | Source cells |
| --- | ---: | ---: |
| AzaleaGym | 26 leafy + 28 round planters | 216 |
| GoldenrodGym | 79 leafy + 47 round planters | 504 |
| AzaleaGym | One broad 3×3 tree | 9 |
| CeladonGym | 42 complete 2×2 hedges | 168 |
| ViridianGym | Seven sparse connected maze networks | 356 |

The four requested families total 1,253 source cells, with zero residual cells
in those exact source selectors in the inspected external base pack. This is
source-binding completion, not a claim that the four rooms are finished.
Unrelated flowerbeds, statues, wall/background art, floor, warp carpets and all
actors remain governed by their existing render paths and game semantics.

`gym_scenery_source.rs` checks the map, tileset and full drawing identity;
identity hashes include each cell's metatile, subtile phase and tile. Individual
planters and hedges are complete drawings, never independent pixel columns.
The tree requires all nine cells. Missing native backing ground rejects a kit.

Viridian uses exact sparse ownership inside seven source-anchored guard
rectangles. Guards include the native openings and a neighboring-cell halo;
viewport padding and scrolling are accounted for by `grid_origin`. A changed or
clipped network is rejected as a whole. The half-row floor cutouts in `$3e/$3f`
stay unowned even though their collision quadrant is blocked. Collision is not
used to invent geometry. All sixteen N/E/S/W exposed-face variants retain exact
unit bounds and a shared coping datum, so straight runs, returns and joins have
no independently jittered seams.

Resolving custom profiles and earlier authored reservations take precedence.
A custom profile inside a maze's guarded opening also rejects that network.
Source identity and the selected ground are checked again before any geometry
is appended; rejected/stale placements append nothing. The adapter only adds
render geometry and authored-coverage labels. It never changes collision,
footing heights, warps, events, NPC locations, movement, or controller state.
The inspected maps have no authored block-change scripts for these families.

## Geometry and visibility

The twenty cached prototypes contain 6,220 triangles, 14,326 indexed vertices
and 364 individually
closed original parts. Per-instance rises are 14px for leafy planters, 16px for
round planters, 28px for the broad tree, 15px for round hedges, and 12px for maze
walls. Models are fitted inside their verified source plots. Floor is still at
its native zero-height datum and uses a verified sample from the same atlas:
Azalea `$12:1f`, Goldenrod `$02:03`, Celadon `$19:57`, Viridian `$03:3d`.

Only successfully appended maze-wall vertices receive the existing camera-aware
player-reveal mask. The ground, neighboring scenery, and actors are not tagged.
This keeps close foreground masonry from hiding the player when the camera
orbits without changing its physical geometry. Tree/canopy proximity to Bugsy,
hedge density and maze continuity are checked in two native camera directions.
Additional player positions and interactions remain in the per-map review queue.

## Rebuild and verify

```sh
blender -b --threads 2 --python tools/build-gym-scenery-models.py -- target/gym-scenery
python tools/check-gym-scenery-models.py target/gym-scenery
python tools/model_asset_storage.py compress target/gym-scenery/*.mesh.json
python tools/johto_art_sources.py --store target/gym-scenery/gym-scenery-kit.blend
python tools/check-gym-scenery-source.py --pack content-packs/core-modular.browser.crystalpack
```

Copy the checked exports to `crates/crystal-voxel-view/models/gym_scenery/` only
when intentionally updating this kit. Runtime exports use the existing bounded
`geothite-model-gzip-v1` text format. The editable source uses the existing
hash-verified gzip/base64 Blender chunk manifest. Build outputs and contact
renders stay local under `target/` or another ignored directory.

The source checker compiles the actual production matcher with standalone
`rustc`, creates temporary identity fixtures directly from the ignored external
pack, and deletes them afterward. It checks exact ownership, unrelated-map and
atlas rejection, missing ground, changed metatiles/art/phase, stale resolution,
custom reservations, altered openings, clipped targets, connected-face masks,
and padded origins. It also detects newly introduced block-change scripts that
would require an additional dynamic-state review. Geometry checks validate
finite normalized normals, opaque material channels, nondegenerate winding,
watertight named parts and identical wall join bounds.

After integration, run the `crystal-voxel-view` tests containing `gym_`, the
production authored-source audit, and representative native captures. The source/geometry checks and corrected Blender studio contact render
were reviewed separately. Integrated voxel tests and eight native views now pass;
four ordinary south-exit controller routes also reach their destination towns.
These representative checks do not complete the longer interaction matrix below.

## Native QA checklist

- GoldenrodGym: walk from the south entry to Whitney at `(8,3)`, passing both
  dense planter rows and narrow turns; check both crown variants and source floor
- AzaleaGym: inspect the tree from the entrance and both sides; approach Bugsy
  at `(5,7)`, the twins at `(4,10)/(5,10)`, and the guide at `(7,13)`; ensure the
  roots/canopy do not obscure interactions or extend outside the source plot
- ViridianGym: walk both branches from `(4,17)/(5,17)` to Blue at `(5,3)`; inspect
  every turn and the half-row cuts, then orbit the camera while standing behind
  an exposed wall; only the foreground wall should reveal the player
- CeladonGym: approach Erika at `(5,3)` and the twins at `(4,10)/(5,10)`; check
  dense adjacent hedge crowns, floor continuity and untouched animated flowerbeds
- On every map: exercise an actual south exit warp, confirm statuary interactions
  remain available, and compare ordinary/custom-profile fallback without hidden
  residual source strips, duplicate geometry, lost ground or blocked openings

Coordinates above are native 16px movement cells. Coverage counts refer to 8px
source cells and cannot substitute for gameplay or art signoff.

To verify a fresh Blender export against the installed compressed documents, run
`python3 tools/check-gym-scenery-models.py --generated target/gym-scenery`.
The checker also rejects nonpositive signed volume for each named closed part.

## Geometry budget

A native no-readback comparison exposed the cost of repeating tiny bevels on
every planter and wall face. Cuboid bevels narrower than 0.03 model units are
now squared; broad chamfers, foliage, leaf folds, roots, coping, closed part
identities and complete bounds remain intact. The kit fell from 12,620 to
6,220 triangles. Exact per-material position/normal indexing further reduced
37,860 vertices to 14,326 without changing expanded triangle attributes, order
or hard-edge normals. The geometry checker enforces a 1,000-triangle ceiling
per prototype.

For a reproducible lossless-indexing check, generate with
`--write-unindexed-reference`, then run
`python3 tools/check-gym-scenery-models.py --generated target/gym-scenery --unindexed-reference target/gym-scenery/unindexed-reference`.
This proof checks f32 position, normal, color and runtime UV bits. Native timing
is reported separately in the performance notes; polygon savings alone do not
establish a frame-rate gain.
