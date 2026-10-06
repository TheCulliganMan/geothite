# Lighthouse keeper room

The native 6F chamber contains a narrow cot, a low round stool, and a long
tea table with a small cup and larger handled teapot. Read-only inspection of
the native atlas drawings, complete layout, collision quadrants and current
3D capture established these identities before authoring. There is no source
beacon console or machinery in this furniture plot.

Three original papercraft assemblies retain those roles. The table has a
bevelled walnut frame, recessed apron, linen runner, faceted glazed cup and
teapot, open handles and a rising pouring spout. The cot has raised end rails,
slatted mattress support, ivory linen, a pillow and folded sage blanket. The
stool has short wood legs, stretchers and a low terracotta cushion. Each of the
59 separately named closed components remains editable in the Blender source.
No model contains copied pixels, textures or source-derived geometry.

The chamber floor uses broad 2×2 source-cell checks in muted sage and chalk,
with a thin warm-stone band at verified room boundaries. It sits at the native
surface elevation and keeps the surrounding slate. Absolute grid coordinates
control its phase; an absent viewport neighbor cannot invent a border.

## Exact source ownership

All new ownership requires `OlivineLighthouse6F`, the exact `lighthouse` tileset,
original metatile, every tile sample, correct subtile phase and an intact whole
picture. The models also require a proven zero-height checker-floor sample.
Changed, clipped, phase-shifted or already claimed pictures remain on the native
renderer. No source pixel sampling influences the authored materials.

- Tea table: the south two rows of `$06` joined to the complete `$2d` below,
  a 4×6 source-cell picture at native source `(12, 18)`; all 24 cells are WALL
- Cot: the east half of `$36`, a 2×4 picture at `(22, 12)`; all 8 cells are WALL
- Stool: the southwest quadrant of `$08`, a 2×2 picture at `(16, 18)`; its four
  native cells are FLOOR and retain native pass-through behavior
- Checker: exact alternating `$0d`/`$1d` samples in all of `$0b`, the north
  half of `$06`, the three non-stool quadrants of `$08`, and the west half of
  `$36`; these 108 cells all have FLOOR collision

The furnishing footprints overlap no actor or warp footprint. All stairs,
pits, slate cells, masonry and source boundaries remain outside these claims.
Successful furnishing labels permit checker backing beneath only their exact
source fragments; an unrelated model label or floor UV does not. The surface
pass now verifies all four actual atlas corners, closing the case where one
corner is clipped but the remaining UV bounds still look complete.

Geometry stays strictly inside each original picture's X/Z footprint. Runtime
heights are 17 pixels for the tea table including its teapot, 12 for the cot and
6 for the stool. These heights do not create actor support: authoritative
collision, navigation and the existing footing vector are unchanged. Furniture
and checker fragments do not join the masonry cutaway mask. The floor is a
surface finish and is deliberately not counted as object coverage.

## Rebuilding and checking

```sh
python3 tools/build-lighthouse-chamber.py target/lighthouse-chamber --runtime-only
python3 tools/check-lighthouse-chamber.py target/lighthouse-chamber
blender -b --threads 2 --python tools/build-lighthouse-chamber.py -- target/lighthouse-chamber
cargo test -p crystal-voxel-view lighthouse_chamber
```

The compact runtime files total about 61 KiB. The table has 1,736 triangles,
the cot 1,440 and the stool 564, appended to the existing solid terrain batch
with no textures, extra material entities or per-frame rebuild. Runtime-only
and Blender generation use the same named component geometry. The editable
source is distributed as hash-verified gzip and bounded base64 chunks in the
existing art-source format.

The checker validates each component separately: finite bounds and colors,
opaque materials, nondegenerate triangles, unit normals and outward winding,
positive volume and exactly two opposite edge uses. Five focused Rust tests
cover each source cell and identity mutation, incorrect maps, missing floor,
claims and crops, unchanged footing/cutaway ranges, physical bounds, floor
masks, donor/ownership guards, distorted UV rejection, coplanar area and checker
phase. A standalone rustc harness runs these exact staged implementations plus
three existing storage tests using minimal surrounding types; it is focused
verification, not an integrated voxel-suite run.

## Native review

Integrated voxel tests and the complete 388-map audit pass. The native 6F
chamber capture verifies the cup/teapot table, cot, low stool, checker floor,
Amphy and Jasmine; 4F and FastShip1F were also captured as controls. Review the cup/teapot silhouette at gameplay zoom,
the cot beside its existing masonry, Amphy/Jasmine hierarchy and room trim
under camera motion. Walk around the table and cot and through the native
passable stool tile: it must preserve its FLOOR support even though the low
seat remains visible. The source game permits that overlap; this finish does
not add collision or relocate furniture to hide it. Also compare 4F and a Fast
Ship room as unchanged negative controls.
