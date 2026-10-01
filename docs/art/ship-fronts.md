# Fast Ship lower-deck room fronts

The two complete six-block drawings at source `(12,12)` and `(36,12)` now have
original steel cap/return shells and the existing cached plain/porthole wall
faces. This closes the architectural drawing between the corridor and both
rooms. It does not generalize WALL collision into geometry.

Each source guard covers a 24×4-cell rectangle: the whole wall drawing, its
four-cell-wide FLOOR entrance, and two genuine corner void cells. Exact map,
atlas, world/crop origin, metatile, both subtile coordinates and tile art must
match. The two different source arrangements have independent fingerprints.
The five exact unchanged bundled flat partition profiles are superseded. Any
renamed or edited profile, other current live profile, or prior authored
reservation within the guarded rectangle prevents the whole replacement,
including a profile in an entrance or corner void. Canonical profiles are
filtered before custom resolution so their order cannot hide a custom edit. Append revalidates the complete drawing before changing any
mesh or ownership buffer.

Only 78 architecture cells per room are claimed. The 32 entrance cells and
four art01 void cells remain native, with no wall or new floor emitted there.
The two replacements supersede 36 previously modeled partial face cells and
finish 120 residual architectural cells. Those counts describe this selected
increment; they are not a claim that the entire ship is remodeled.

## Physical form and shared components

The cap/return shells follow the source silhouette and share a B1F-only
28px display height with the U walls and north-corridor panels. Each consists of
two separated wings around its full 32px entrance. Closed steel bodies, dark
deck skirting, narrow ivory shoulders and inset cool steel crowns surround
the real corner holes. Each outer
return reaches the existing U-wall seam at the original 32px south edge. The
middle face course ends at z29 so the existing closed wall models occupy the
remaining 3px. The native porthole drawing uses the canonical 16px-wide
`PortholeBulkhead`; plain face and jamb columns use `ShipBulkhead` through the
existing shared caches. No duplicate plain or porthole model is added.

Corridor-facing recessed navy panels, narrow stiles and brass fasteners match
the existing restrained papercraft hull. Back, end, underside and hole faces
remain complete. Only proven internal shared edges are omitted by the authoring
generator; no potentially visible face is removed based on a camera assumption.

The same quiet mess-floor material supplies a zero-height backing under claimed
wall cells. That surface is emitted before the cutaway range. Only the closed
shells and shared wall-face model vertices join the existing local player
reveal. Native floors, furniture, collision and actor support heights remain
unchanged. The four genuine void cells never acquire backing or object credit.

## Editable source and rebuild

`tools/build-ship-fronts.py` authors two shells, with 31 individually editable
closed components and 2,020 triangles each. It uses the existing lighthouse
authoring helper for material and compact runtime serialization. Runtime
outputs contain only original geometry/materials. The source is
`ship-fronts.blend`, stored as two hash-verified gzip/base64 chunks in the
existing art-source manifest. Plain and porthole faces retain their existing
editable source and shared runtime model identities.

```sh
python3 tools/build-ship-fronts.py target/ship-fronts --runtime-only
python3 tools/check-ship-fronts.py --pack content-packs/realtime-clock.browser.crystalpack
blender -b --threads 2 --python tools/build-ship-fronts.py -- target/ship-fronts --skip-preview
cargo test -p crystal-voxel-view ship_fronts
```

The lightweight checker validates deterministic runtime reproduction, opaque
colors, finite coordinates, unit normals, consistent closed topology, positive
component volume and the real sparse footprint at vertices, edge midpoints
and triangle centroids. It reads the external pack without copying it and
checks both source fingerprints, the exact entrances/voids, every native
event and actor footprint, and unchanged pack bytes. Its proof does not execute
the Rust adapter or gameplay.

The adapter tests exercise every identity field in every guarded cell,
changed/cropped viewport origins, custom ownership at walls/voids/entrances,
atomic append rejection, repeated append, exact shared face model fitting,
opaque backing, cutaway scope and unchanged support heights. The standalone
test harness uses the production resolver/model fitting and profile matching
with minimal shell types and a floor-only shape-classifier stand-in. A full
Cargo/integrated renderer check is still necessary.

## Native acceptance gate

Inspect both room fronts from corridor and room sides, then both shallow
diagonals. The former black/blue strips must read as connected steel/ivory
caps and returns. Check the face seam, open mouths, joins into both existing
U walls, porthole placement, reverse faces and the four intentionally retained
corner voids. Move the camera through both front/back views and compare the
whole-wall translucency: the blocking source-owned wall must remain faintly
visible across its complete shape, with the player and other actors/props
visible through it. Floor and furniture stay opaque. Unblocked walls restore
solid depth rendering without a capsule hole or a lighting change.

Walk both room mouths, the perimeter corridor and the two native ladder
warps `(5,11)` and `(31,13)`. Exercise both sailor coordinate events
`(30,7)` and `(31,7)` and the relevant sailor scene/exit routes. The trashcan
read at `(27,9)` must stay reachable. Check the integrated scene for z-fighting
and actor clearance under movement, then record native captures and a
performance comparison. Source/geometry checks do not certify those behaviors.

## Cap contrast, shared wall height and editable reproduction

The broad cap uses dedicated dark cool steel with a narrow ivory perimeter
shoulder. The shared `ship::B1F_VISUAL_WALL_HEIGHT` is 28px. The U, both complete
front shells, legacy plain faces and north-corridor compound cap/faces all use
that height in FastShipB1F. Other ship maps retain their original 16px visual
wall fitting. The native shape classifier, source footprints, actor support,
entrances/voids and cutaway vertex scope are unchanged.

The original normalized asset documents and editable Blender sources remain
the same dark-cap revision. Their nominal authoring Y extent is 16px; runtime
fitting multiplies Y by 28/16 while retaining X/Z. The authored 15.2px shoulder
therefore displays at 26.6px and the crown at 28px. To reproduce the U/front
shell display in Blender, select only those asset collections and scale their
mesh Y-up coordinate by 1.75 (Blender Z). Do not scale source footprints, floors,
actors or unrelated furniture. No additional asset or source archive is needed
for this parameterized display fit.

The canonical porthole panel uses a separate vertical fit to preserve the
window's existing proportions. First fit the original panel to its current
16px height and original width/depth. For native fitted height y, use:

- y below 5px: y28 = 2.2 × y
- y from 5 through 14px: y28 = y + 6
- y above 14px: y28 = 28 − 4 × (16 − y)

The brass ring, glass and all eight rivets lie wholly within the unchanged
central band. All their dimensions and relative positions are retained; the
assembly moves upward by 6px. The lower and upper plain wall extend to meet
zero and 28px. The fit retains the cached vertex/index topology and source
asset, inverse-transforms normals for each local Y scale, and applies the
existing face-light bake once. Requests for the original 16px fit use the old
append path byte-for-byte. The test covers native and tall fits at three tile
scales, exact central-band translation, unchanged X/Z, indices and UVs, unit
normals and the unchanged original-height result.

The shared crowns keep the same 1.25–14.65px and 177.35–190.75px return
cross-sections at their joins. Same-camera native review must verify increased
vertical presence, no seam step, open entrances, retained reverse faces and
correct local player reveal. Static fit tests do not certify those visual or
gameplay results.


## Integrated validation

The full 805-test voxel suite passes, with two existing benchmarks ignored.
All 27 Python model/source CI commands and all 70 editable source archives pass
hash verification. Both inspected external packs pass the exact source checks.
The production 388-map audit is clean and records the expected 120-cell net
increase. Optimized native and WebAssembly builds pass.

Thirteen actual native frames cover both complete fronts, room/corridor sides,
shallow angles, local player reveal, both entrances under ordinary held input,
and unchanged cabin/first-floor controls. A same-camera comparison confirms
that the taller shaded panels read more clearly as walls. Floor/backing remains
opaque; closed ends and crown joins are visible. Both entrance walks rebuild
terrain zero times; the east route subsequently reaches a trainer sight event
and displays its authored dialogue.

A separate production-controller fixture harness passes both two-lane entrances
in both directions, all adjacent wall rejections, both reciprocal ladder warps,
and both released sailor gate orientations. It also exposes two existing
interaction limitations: farjumptext's TrashCanText is opened and immediately
consumed during controller settling, and an immediate second A after dialogue
can retain the original key edge. Those do not arise from this geometry change
and remain controller work. The first sailor request and both source sidesteps
execute. The harness and reports remain external staging until the interaction
repair is validated; no passing full-ship gameplay claim is made here.

The 28px static B1F sample measures 94.75ms median and 110.77ms p95 on the cloud
software renderer. The preceding floor/furniture checkpoint measured 81.97/
91.06ms under the same setup. This heavier scene is not fluid; static terrain
submission and culling remain priority work. See the performance notes for
the controlled 16px/28px comparison and its limits.
