# Source-aware structural extension

This adds 17 original, editable structures to the existing 42-model exterior
catalog. All bindings are renderer-only. No simulation, collision, warp, map,
palette, or content-pack files are changed.

## Diagnosed gaps

The prior audit's “building” category included complete non-building drawings:

- Blackthorn: a closed 12x8 rocky mesa
- Ecruteak: two 8x4 tower forecourts, whose northern two rows are flush decks
- Route 2: the 16x8 Diglett's Cave formation with a real recessed entrance
- Ice Path: complete variable-width two-block shelves, including left/right
  stairs, concave shoulders, side closures, and uninterrupted cap continuations

The other missing buildings already had complete recognized drawings. Lake of
Rage and Route 27 contain lawn but no preferred ordinary path tile; Lavender
and Route 19 contain their native paving but no preferred Kanto lawn sample.
The underlay resolver now accepts these explicitly identified same-tileset
surfaces. It rejects an unrelated metatile reusing that tile number, a connected
map with a different tileset, and an incompatible requested ground metatile.
Lavender's four-storey radio building and Route 19's gateway receive distinct
architectural meshes rather than another town's landmark or an ordinary house.

## Matching and geometry

New formations require a complete footprint, consistent block/subtile phase,
and every expected source tile. Missing, modified, unknown and clipped drawings
remain on the existing fallback path. An edited live profile retains its own
renderer. The model catalog includes original faceted rock, beveled cornices,
recessed glazing, real arch/pier apertures, individual planks, open forecourt
rails, and reusable handed ice shelf sections. No game art is loaded by the
Blender generator.

The cave opening aligns with native column 5 of its 16-cell plot. Its modeled
arch has air before the recessed interior; no hidden box seals the entry.
The forecourt has the original two-cell entrance gap at columns 2..4 and a zero
walking datum. Source stairs retain left/right handedness and north-high rise.
Grid-based ice modules preserve their logical footprints rather than stretching
a smaller physical notch or stair cap to fill a rectangle. A flush authored cap
or deck does not receive a duplicate coplanar underlay.

Footing follows exactly the existing complete-rock-formation rule: six top
source rows are supported at 16 pixels where the legacy renderer already did so.
In particular, legacy eight-cell-wide Ice Path shelves remain on their existing
zero datum. This extension does not silently correct or reinterpret that rule.

## Sources and verification

- Authoring: `tools/build-structure-extensions.py`
- Editable Blender kit: `structure-extensions.blend`, stored through the common
  chunked art-source manifest
- Runtime: `crates/crystal-voxel-view/models/world_exteriors/`
- Binding: `src/mesh/structure_extensions.rs` through `modeled_exteriors.rs`
- Geometry check: `python tools/check-structure-extensions.py`

The geometry checker verifies triangle winding, finite unit normals, bounds,
per-model budgets, the cave and forecourt apertures, and all three stair variants.
Rust tests cover exact matching, mutation/phase/crop rejection, no partial or
repeat claims, unchanged legacy support, and safe underlay selection. Re-run the
full terrain tests, full-pack audit, and actual native map captures after
integration. The offline Blender previews demonstrate the assets themselves;
they do not establish in-game coverage, movement, or camera behavior.
