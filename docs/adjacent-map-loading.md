# Adjacent maps in 3D

The optional renderer now resolves the outdoor connection graph in active-map
coordinates. Each cardinal edge uses the pack's metatile dimensions and signed
offset. Breadth-first traversal retains the first placement of each map, so
reciprocal edges and inconsistent longer cycles cannot repeatedly move or load
the same map. This includes the New Bark / Route 29 / Cherrygrove chain and
New Bark / Route 27 / Route 26, with Route 26's northward offset retained.

Camera selection tests each map's source patches against the perspective
frustum, including horizon/near-plane crossings and the height occupied by
scenery. Patches are 32 source cells wide with a 16-cell source margin for
complete models. Zoom, pitch, orbit, first person and window aspect can change
selection without a gameplay tick. The active atlas keeps its normal fixed
halo; distant maps no longer enlarge one mostly empty rectangular atlas.

Selected neighbors publish independent immutable `VisualWorldFrame`s through
`VisualWorldNeighborhood`. Each frame carries its own map ID, source coordinates,
map bounds, warp/ledge evidence, tileset art and effective palette period. The
voxel consumer uses the ordinary terrain builder/cache for each map, preserving
map-specific models, materials, water and animation families. Placement is
separate from immutable source grids, so walking or a small orbit can retain
both the atlas and geometry. Preparation starts at most one heavy neighbor
build at a time; two recently hidden maps can stay resident for camera reversals.
Other offscreen scenes retire their roots and assets normally.

Only the optional presentation receives these scenes. The classic image,
priority image, collision lookups and battle-transition tiles retain their
original direct connection sources. Saved block overrides come from runtime
map snapshots. No neighbor scripts, actors or gameplay sessions are run.
Terrain identity includes connected maps' current blocks; retained map identity
includes its actual source blocks, art key and crop. Sparse authenticated
animation phases update the owning retained atlas, and water lookups follow
that map's placement. Exterior window emitters also use owning-map transforms.

The active optional grid marks foreign map rectangles with a sentinel so they
cannot be drawn again with the active map's profiles. Its backdrop also leaves
these rectangles open. Unknown distant cells retain the separate flat-backdrop
sentinel. The original close border halo remains intact.

## Harbor sightlines

Vermilion and Olivine ports have no cardinal edge: gameplay reaches them through
an underground passage. Their two named city/port pairs have a presentation
seam at the city's south edge, aligned horizontally by their entrance warps.
The city, port tileset, passage and both reciprocal warp IDs must agree with
the pack before this seam is available. Warp constants and resolved map names
are both accepted. The folded underground corridor is not placed outdoors.
This is an authored visual arrangement; it does not add a traversable edge or
change the passage/ship boarding flow.

## Verification

Use the completed external pack; generated content stays ignored:

```sh
export CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack"
cargo test --locked -p crystal-bevy --features location-tester --lib \
  map_neighborhood -- --include-ignored
cargo test --locked -p crystal-bevy --features location-tester --lib \
  visual_tile_grid_requires_one_row_major_cell_per_terrain_position
cargo test --locked -p crystal-bevy -p crystal-voxel-view \
  --features crystal-bevy/location-tester --lib \
  terrain_demand_follows_perspective_pitch_zoom_and_window_aspect
```

The external checks cover transitive positions, cycles, interior exclusion,
harbor placement in both directions, distant block invalidation, actual ship
source cells from Vermilion City, identical classic base/priority pixels and
unchanged gameplay checksums. The publication check also requires an active
frame before and after changing only the camera.

## Work still requiring acceptance

Pack/source checks do not establish visual acceptance. Inspect paired production
captures at the same position, time, facing and state, including looking south
from Vermilion, west from Route 29, and east from New Bark. Check shoreline
joins, complete buildings and ships, elevation and material continuity.

Independent scenes now carry map-scoped ownership. The new path still needs
fresh paired production acceptance for harbor scenery, New Bark/route joins
and Cherrygrove, followed by memory and camera/connection transition checks.
These are outstanding verification gates, not a completed visual acceptance
claim. Prepared runtime/browser bundles have not yet been replaced.

### October 15 verification and rejected wide-atlas experiment

The two external-pack checks, eighteen camera checks, six backdrop checks,
grid-publication check and five production controller regressions passed
(32 total). The expanded grid was published as an active frame, and changing
only the camera retained classic pixels and the gameplay checksum.

Paired production image-target captures from Vermilion City `(19, 31)` and
Vermilion Port `(9, 5)` preserved their state, facing and noon clock across 2D
and 3D. The city view includes the port, but uses blue source-art fallback
structures where the active-port reference uses modeled gray structures.
This verifies the need to carry neighboring map ownership through meshing.
The boat sightline has not passed visual acceptance. A separate window-target
attempt produced uniform black images and was rejected; it is not render proof.

The uncapped 20-degree, zoom-0 city view requested a `436 x 434` grid. The two
wide city/port runs reported peak process footprints of 7.55/11.26 GB and took
about three minutes each under concurrent build/graphics work. Those are
contended observations, not isolated performance measurements. This rejects
using a larger combined atlas as the final design. That prototype was replaced
by independently retained, frustum-selected map patches with map-scoped
profile/material ownership. No prepared runtime or
browser bundle has been replaced with this experimental implementation.

### Flat-water correction

Pools, lakes and seas now use a level normal with static depth tint and a quiet
shore edge. Rolling wave normals, moving color bands and crest glints were
removed from the shared pool shader, including the modeled park basin.
Authored waterfall flow and swimmer contact remain separate from that surface.
The embedded-shader native build passed. A fresh noon Vermilion City pair at
`(19, 31)`, zoom 0, rotation 4 and pitch 35 showed flat water and preserved the
classic image and gameplay state. This verifies the water correction; the
neighboring port's source-art fallback remains an adjacent-map defect.
