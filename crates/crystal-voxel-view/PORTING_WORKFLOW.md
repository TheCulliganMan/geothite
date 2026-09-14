# 2.5D Porting and Visual-Audit Workflow

This renderer is an optional visual mod. `crystal-core` remains the gameplay
authority, and disabling the `voxel-view` feature leaves the normal 2D path in
place. Geometry must never change collision, movement, scripts, warps, or
event state.

## Source and reference boundary

Crystal's own exported map data is the source of truth for object identity:

- tileset id;
- metatile id;
- subtile column and row;
- tile index;
- current live 8x8 texture, including palette and animation state.

The recovered DramaticShape Gen 1 mod is used only to study rendering
behavior: authored shape profiles, grouping a complete drawing before
meshing, separating top art from upright art, native-size facade bands, and
bounded caches. The recovered mod has no
project-level license, so its code, tables, identifiers, and assets are not
copied. Gen 2 profiles are independently derived from `vendor/pokecrystal`
and the exported Crystal assets.

Collision is not geometry. It may help locate a suspicious unprofiled cell in
the audit report, but it never promotes a tile into a wall, cliff, tree, or
building.

## Object loop

Every object is handled with the same small loop:

1. Find a bad object in a paired 2D/2.5D render.
2. Record its exact Crystal source identity from the map block and metatile
   layout.
3. Find the equivalent behavior in the reference mod, when one exists.
4. Classify the complete Crystal drawing, not isolated 8x8 cells.
5. Recreate that behavior in a focused Rust module.
6. Capture a fixed-angle reference and an oblique view in the live renderer.
7. Render the same location in 2D and 2.5D and inspect both images.
8. Keep iterating while topology, footprint, palette, or occlusion differs.

Unknown or incomplete drawings remain faithful flat 2D art. A flat correct
cell is preferable to invented volume.

The unit of work is an authored object, not a map name. Once a tree, sign,
fence, ledge, or cliff rule is correct, it is selected by source identity and
reused everywhere that drawing occurs. Map-specific matching is reserved for
genuinely unique assembled landmarks whose shape depends on several placed
metatiles.

## Evidence recorded for each object

Before adding geometry, record:

- a paired screenshot and the player's exact map coordinate;
- map id, tileset id, metatile id, subtile coordinate, and tile index;
- the complete metatile group occupied by the drawing;
- which pixels are top, front, side, transparent background, or unchanged
  ground;
- the object's authored foot line and gameplay support height;
- adjacent variants used for corners, ends, transitions, and animation;
- the closest behavioral analogue in DramaticShape, if one exists.

The Rust profile matcher consumes those facts. It must not infer them from a
blocked collision value, a palette color, or the object's screen position.

## Geometry vocabulary

- **Plane**: ordinary map art at its authored datum.
- **Flat-standing card**: trees, signs, flowers, movable boulders, and other
  face-on drawings. The complete drawing is background-masked once and stood
  at its foot line. No rounded voxel hull is generated.
- **Ledge course**: one native source band, normally 6px high. Courses stack
  only when the 2D map topology stacks them.
- **Cliff or mound**: one connected raised region with a modest trapezoidal
  exterior. Only exterior boundaries receive faces; neighboring raised cells
  do not create internal walls.
- **Building**: roof pixels stay top-facing, facade rows fold upright exactly
  once, and sidewalls come from authored side evidence or narrow live edge
  strips. A facade source cell is not also painted on the ground.
- **Water**: a plane or authored recess. Shore rock and waterfall art are
  separate roles and are never inferred from an adjacent ledge.

Textured vertical faces are split into native 8px bands. A source tile is not
stretched over an arbitrary wall height.

## Persistent visual review

Use the existing native tester binary directly. Keep the process open for
visual checks: neither Cargo nor a fresh game/GPU startup belongs in the
capture loop. The tester is built with `location-tester` when its code changes;
camera, movement, and repeated capture checks need no rebuild.

From the repository root:

```sh
target/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map ElmsLab --x 4 --y 8 --view both --live \
  --screenshot /tmp/elm-review.png
```

The session writes the 2D source reference and 2.5D render using the same
runtime and GPU. It then stays open in 2.5D. Move normally and press **F6**
to capture again. **F3** still toggles the presentation for direct inspection.

An external reviewer can trigger the same running session without keyboard
focus. Write the request file printed by the tester:

```sh
# Capture the current location and camera again.
touch /tmp/elm-review-2d.request

# Set zoom and orbit before capturing: orbit 1 = 45 degrees.
printf '1.0 0.5' > /tmp/elm-review-2d.request
```

The request is consumed once. Empty input keeps the camera; two finite
numbers set zoom and orbit. Invalid input leaves the session running and
prints the error. Capture time is printed after GPU readback and image
validation, rather than after a guessed sleep. The live comparison is
`/tmp/elm-review-2d.compare.png`, with columns ordered **2D reference,
previous 2.5D (when location and camera match), current 2.5D**. Full-size
individual PNGs are retained beside it. Read the refreshed image after each
`review ready` message. Close the window to finish the session.

For a single capture, omit `--live`. `--view both` then writes
`*-2d.png`, `*-2.5d.png`, and `*-compare.png`, plus a small HTML review page.
Repeated one-shot captures retain a previous panel only when the pack,
location, and hour match. Use `--maps <id,id,...> --output-dir <dir>` for
coverage across rooms; these batch runs are separate game sessions.

Start the New Bark interior review with `ElmsLab`, `PlayersHouse1F`,
`PlayersHouse2F`, `PlayersNeighborsHouse`, and `ElmsHouse`. Inspect multiple
positions around furnishings and the stair/door approaches in each room.

Do not accept a render only because it compiled. Check at minimum:

- the player's foot occupies the same map location;
- doors and paths retain their 2D alignment;
- raised regions have the same connected topology as 2D;
- props do not raise neighboring terrain;
- sprites are correctly in front of or behind objects;
- no source pixels are duplicated, stretched, or left on the vacated floor;
- viewport edges continue into the render halo rather than becoming a box;
- palette and animated tiles still come from the live frame.

For the current fast loop, use [the rendering toolkit](../../docs/RENDER_TOOLKIT.md).
It records the current session and separates rendered evidence from pending source edits.

## Full-game coverage audit

The source-coverage auditor inventories every placed metatile/subtile and
reports its current visual class:

```sh
target/debug/examples/audit_voxel_coverage \
  --pack content-packs/core-modular.browser.crystalpack \
  --output /tmp/crystal-voxel-coverage.json
```

The `suspicious_flat` field is a review queue, not permission to extrude a
collision wall. Each finding still requires an authored profile and a visual
comparison.

The JSON report is grouped by tileset and source identity. Review repeated
sources first, then unique landmarks. Coverage means that an object has been
classified; it does not mean its geometry has passed visual review.

## Rejected approaches

These failures are documented because they can compile and pass mesh tests
while still producing an obviously wrong image:

- extruding every blocked cell creates stairs, duplicate columns, and false
  cliffs;
- stretching one 8px source tile over a tall face destroys facade and cliff
  artwork;
- generating rounded voxel hulls for drawn boulders invents spikes and
  changes their silhouette;
- closing the edge of the 20x18 viewport creates a false box around the map;
- profiling a transition metatile alone creates disconnected wall chunks;
- fixed-color side triangles create visible flyaway fins;
- accepting a successful build without comparing the 2D image hides
  footprint, doorway, and topology errors.

The first Ice Path perimeter pass used independent straight-wall rules for a
small subset of its metatiles. The result was disconnected raised chunks and
an oversized sidewall, so it was removed from the runtime profile. The next
implementation must resolve the complete connected edge graph before it
emits any face.

## Cave audit contract

The reusable cave rules are:

- Johto cave and Kanto cave assets use the same heights for the same visual
  role; regions do not get different dimensions.
- free-standing 16x16 rocks and boulders are flat-standing cards;
- a boulder never raises a cliff or its surrounding cells;
- a walkable shelf uses the normal low ledge course;
- a rock wall or mound uses one cliff level per authored stacked layer;
- exposed cliff sides use a modest trapezoidal slope and live source bands;
- diagonal closures use live edge strips, never a fixed palette-colored fin;
- Ice Path perimeter rocks require one connected directional profile spanning
  its straight, corner, and transition pieces. The reviewed family includes
  `04-12` and `3e`; it cannot be implemented as unrelated `04-0e` blocks;
- water and waterfalls remain independent from cliff elevation.

The cave pass is complete only after paired renders cover ordinary cave,
dark cave, Ice Path perimeter/ice field, stacked shelves, a waterfall, and a
map-edge continuation.

## Definition of done

An object is done only when all of the following are true:

1. Its exact source variants have been inspected in live paired renders.
2. Its mesh uses native source bands without duplicated or stretched art.
3. The paired 2D and 2.5D renders preserve footprint and connected topology.
4. The player and NPCs occlude correctly at the front, back, and sides.
5. A render one step away remains in 2.5D without any automatic mode change.
6. The same rule works at a second location that reuses the source object.
7. Disabling `voxel-view` still gives the untouched normal 2D renderer.
