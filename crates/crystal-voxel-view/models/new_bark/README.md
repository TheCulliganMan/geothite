# Authored New Bark model kit

Original low-poly geometry for the optional New Bark 3D presentation. These
assets contain no ROM, source sprites, game scripts, map export, or collision
data. The game content pack remains external. Blender is an authoring tool only;
the Rust renderer embeds and reads these meshes without a runtime exporter.

## Assets and coordinate convention

- `house`: warm plaster cottage, red tiled roof, timber, recessed windows and door
- `player_house`: two-storey cottage with blue shutters and modeled window boxes
- `lab`: wider teal-roofed research building
- `tree`: faceted canopy, branches and trunk
- `flowers`: five modeled flowers with stems, leaves and petals
- `trainer`, `teacher`, `scientist`: preserved original static character art; the
  active connected-map characters now use the articulated `../johto_characters/` kit

All exports are right-handed, +Y up, front +Z, with an origin at floor center.
House and lab footprint bounds are respectively 4 × 3 and 6 × 4 model units.
Their `door_anchor` is the centered ground-level entrance threshold on the south
edge; the visible door leaf is recessed behind it. Colors are Blender's linear
material values, not sRGB bytes.

Each JSON has per-material primitives with flat XYZ `positions` and `normals`,
triangle `indices`, and RGBA `base_color`. Names, bounds and counts are authoring
metadata. The importer validates buffers, derives actual bounds and retains
vertex colors. Terrain fits bake gentle facet shading. The separate articulated-character
importer uses unshaded colors for Bevy PBR.

## Source-aware placement

`src/mesh/new_bark.rs` activates on the connected New Bark, Route 29,
Cherrygrove, Route 30, Route 31, and Violet slice. The additional civic buildings,
traditional architecture, and grass live in `../johto/`. Buildings require a
complete known Johto metatile drawing and matching 2 × 2 door artwork. The
existing `BuildingPlacement` supplies the full plot. Both plot edges and the
source doorway are preserved by an off-center horizontal fit; no collision,
warp, script, movement or actor-footing value is changed.

Complete source trees replace their prior live-profile cards. A missing group
or ground sample leaves the existing renderer path available. Flowers use the
same model in both the initial and animated-terrain refresh paths. Unknown or
clipped drawings, Route 31's east-entry gate, interiors, and maps outside this
verified slice retain their shared source-art renderer. Read `../johto/README.md`
for the added landmarks and native doorway constraints.

Tests cover buffer validity, source identities, rejection of incomplete or
altered door art, map scoping, plot bounds, tree ownership, flowers, and unchanged
footing. Full gameplay and visual acceptance still require the external pack.
