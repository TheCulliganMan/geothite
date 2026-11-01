# Source-aware dungeon and terrain kit

This is original geometric art. The checked-in Python authoring source uses
Blender primitives and individually named parts. It never reads a ROM, content
pack, source sprite, atlas, source image or game model. It produces editable
unjoined Blender collections and compact untextured runtime triangle meshes.

## Rebuild and validate

```sh
blender -b --python tools/build-dungeon-models.py -- target/dungeon-kit
python3 tools/check-dungeon-models.py
cargo test -p crystal-voxel-view dungeon_models
cargo test -p crystal-voxel-view modeled_dungeons
```

`--skip-preview` omits only the studio render. The generated `.blend` remains
editable and uncompressed, with one collection per model. `target/` output is
scratch output; the repository's common source-package workflow distributes the
editable scene. Only the model JSON exports belong under `models/dungeons/`.
The contact-sheet render is QA output, not a game texture.

## Authored vocabulary and actual bindings

Nineteen model exports are present. Seventeen have exact runtime source bindings:

- Light cave, dark cave and ice boulders, plus full ice masses
- Carved tower/gym guardians and warm Alph guardians
- Globe-topped League podiums, distinct from the guardian sculptures
- Gym plaques and the open bins in Vermilion's puzzle
- Independent framed warehouse crates
- Ship barrels, stools, linen racks, bunks and porthole bulkheads
- Tower board-and-beam wall courses
- Rocket warning beacons

The stone-tablet and timber-column exports are **unbound art reserves**. They do
not count as runtime coverage. In particular, a static column must not replace
Sprout Tower's animated pillar, and an abstract inscription must not replace
readable puzzle clues. Those sources retain the existing presentation.

Source-classified procedural volumes supplement the imported meshes:

- Connected Rocket Base steel wall networks, with all sixteen cardinal
  exposure configurations and no internal faces between matching cells
- Complete lighthouse straight side-wall drawings, with faceted masonry;
  mixed corners, windows and doorway variants remain fallback
- Mirrored complete cave diagonal corner volumes
- Continuous cave, dark-cave, ice and verified outdoor mountain-bank surfaces

## Placement and topology contract

`mesh::modeled_dungeons::resolve` runs on immutable source cells before live
profiles mask them. Existing source-local classifiers supply exact whole-drawing
boundaries. A second phase check verifies subtile coordinates and block-local
consistency. Grounds must be present, same-tileset and genuinely flat/water;
missing ground, clipped drawings or changed source cells never reserve a plot.
Live-profile candidates reuse the existing resolver and additional semantic
checks, rather than embed a second exported source catalog.

Every placement reports consumed source indices and its authored kind label.
The parent pipeline owns the single suppression/coverage mask. The append step
only builds visible geometry and repaints the vacated plot with its source ground
sample. It never writes collision, event state, actor support, object visibility,
warps or stairs. Source spacing is retained; face-on furniture collapses only
into its declared physical depth, anchored at the original south seam.

The rock-terrain pass receives final resolved mountain/rock tiers. Caps keep the
same support height, all face endpoints remain fixed, and internal faces are
culled against neighbor heights. A sloped neighbor contributes its exact touching
edge, including partial-height triangle terminations. The rock faces receive
recessed hand-cut facets whose shared endpoints never jitter. Viewport clipping
does not invent a perimeter wall. Ladders, ramps, unknown planes and unsupported
bank vocabularies remain on their established render path.

## Verification boundaries

The standalone checker validates every export's dimensions, finite coordinates,
triangle indices and winding, unit normals, opacity, semantic materials and
triangle budget. Rust regressions cover complete/clipped/corrupt/phase-shifted
sources, missing ground, independent crate grouping, map scope, exact source
footing preservation, wall exposure permutations, tiered terrain, viewport
clipping and ramp-adjacent seams. Pack-driven scene audit and screenshots are
still required at integration time. This kit alone does not establish complete
coverage of every dungeon, glyph, effect or puzzle state.
