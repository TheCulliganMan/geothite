# Joined lighthouse masonry

Three original unit-scale volumes provide dressed ashlar, a single recessed
window and paired recessed windows. Each has staggered bevelled courses, pale
plinth/crown bands, continuous corner quoins and a closed underside. Window
reveals have physical depth, recessed blue-green glazing and bronze mullions.
The art is authored from geometry and semantic colors; it contains no source
images, sampled sprite pixels or copied game geometry.

```sh
blender -b --threads 2 --python tools/build-lighthouse-masonry.py -- target/lighthouse-masonry
python3 tools/check-lighthouse-masonry.py
cargo test -p crystal-voxel-view lighthouse
```

`--skip-preview` skips only the two rendered views. Standard Python with
`--runtime-only` reproduces the exact runtime documents without Blender. Source
collections contain named editable stone courses, quoins, reveals and panes.
The source manifest stores the complete Blender file as hash-verified gzip and
bounded base64 chunks, using the repository's ordinary source reconstruction.

## Native ownership and physical space

The resolver is restricted to the six named Olivine lighthouse floors. It
accepts only complete 4x4 drawings from native blocks 29, 3c, 3d and 3e, checking
every cell's atlas, block identity, exact tile motif and source phase. A changed,
clipped, shifted or mixed drawing is rejected as a whole. The native block-27
floor supplies the underlay and must be a zero-height source surface.

All four collision quadrants in 3c and 3e are WALL. Block 3d has one WINDOW
quadrant, and block 29 has two WINDOW quadrants; their other quadrants are WALL.
No candidate contains reachable backing floor. A volume therefore stays inside
its original four-source-cell-wide and four-source-cell-deep wall footprint.
Stairs, pits, entry threshold, plain flooring, room furnishings, the 6F counter
and all actor/warp footprints remain outside those claims. These meshes do not
read or write authoritative collision or navigation state. Appending retains
native floor height and leaves the existing footing vector unchanged.

The assembler omits a cardinal side only when the complete neighbor also passes
ownership. This composes straight runs, independent ends, outside/inside corners,
T junctions and crossings with the same exact boundary coordinates. It does not
bridge openings. Windows face the adjacent room floor; the exact native blank
pocket can orient 1F's southwest window. Missing context cannot guess a window
orientation. North/east quarter turns preserve single-window handedness by
reflecting the model with corrected winding.

## Cost and verification

The compact cached files total about 30 KB. Complete isolated shells contain
684, 808 and 932 triangles. A plain exposed cardinal face is 164 triangles;
shared faces are omitted before batching. The crown and underside together are
28 triangles. Everything appends to the existing solid terrain batch without
textures, material entities or per-wall scene entities.

The checker verifies finite unit bounds, opaque colors, normal/winding
agreement, nondegenerate triangles, positive volume and a watertight shell with
exactly two oppositely oriented uses of every welded edge. The source tests
cover all six floors, all source cells, the complete sixteen exposure masks,
other-map rejection, claimed neighbors, cropped/shifted/mixed art, missing
native floor, four window orientations, unknown context, reflected winding,
physical bounds and unchanged footing.

Read-only inspection of the ignored base pack identifies 301 complete blocks:
4,176 plain masonry cells plus 640 window masonry cells, or 4,816 total. That is
3,696 new cells beyond the previous 1,120: the old 3,504-cell facade residual
plus 192 previously flat-classified 6F double-window cells. The production 388-map audit confirms these exact source claims. Native
captures of all six floors preserve the stairs, entrance, central gaps and 6F
chamber layout.

## Player reveal and floor finish

Successful masonry appends now mark only their solid vertex ranges for a
camera-aware player reveal. A projected capsule removes marked fragments in
front of the player, with a fixed stippled boundary; walls behind the player
remain opaque. The original 32px wall geometry, shadow geometry, collision and
footing remain intact. Moving the player changes uniform data rather than
rebuilding wall meshes. Missing player/support data disables the reveal.

The [slate floor finish](lighthouse-floor.md) uses exact source guards and keeps
floor fragments outside the cutaway mask. Current native 4F captures verify the
reveal and quieter floor; 6F captures retain the original central checker zone
and furnishings. Additional orbit/movement and floor-by-floor checks remain,
and 6F furniture/machinery still needs its own bounded treatment. These changes
do not certify every lighthouse scene as finished.
