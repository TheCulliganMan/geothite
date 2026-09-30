# Full-volume interior furnishings

This is an incremental authored-art kit, not a claim that all indoor assets are
converted. It supplies 68 original full-volume mesh variants with wood joinery,
rounded cushions, appliance controls, separate books/stock, plants, clinical
sockets, window frames and other role-specific details. No artwork is extruded
from game pixels and no collision data is used to invent objects.

## Runtime integration contract

Register `interior_models` in the crate and `mesh/modeled_interiors.rs` under mesh.
Before legacy source masking, call:

```rust
let placements = modeled_interiors::resolve(map, &original_cells, &geometry, profiles);
```

Each `Placement::indices(width)` identifies only a complete verified drawing.
`Placement::label()` supplies an exact `interior/<asset>` label. Exclude those
indices from legacy/live matching, preserving immutable original cells, shapes,
atlas slots and footing calculations. Append with:

```rust
modeled_interiors::append(&mut mesh, &original_cells, &geometry, placement, &mut claims);
```

`append` returns false without changing either mesh or claims if any owned cell
is already claimed. A successful append restores the existing floor sample,
places geometry within the drawing's footprint, sets claims and records
`TerrainMeshData.authored_cells`. Explicit live-profile footing values are
preserved; other actor footing remains the caller's original result. Model
fitting inverse-transforms normals and shades solid facets without textures.

The semantic live-profile allowlist is exact. Sources are matched against the
existing checked-in canonical profiles, including maps, tilesets, complete
metatile grids, every subtile phase and every tile index. A customized matching
live profile keeps ownership and is not overridden even by a compiled matcher.
There is no copied content-pack catalog or new source-data dump.

## Implemented matching

- Original live roles for lab cabinets/workstations/restoration apparatus, table
  assemblies, house plants, domestic tables, Center PCs/healing machines/seats/
  dividers, Mart shelving/refrigerators, facility shelves/instruments/plants,
  station seating, arcade machines/stools, radio PCs/machine, traditional house
  clock/cabinets/walls/cushions, gate doors/terminal/walls/planters, flower stands,
  and memorial units
- Existing compiled complete groups for household cupboards/bookshelves/TV/
  radio/stove/sink/refrigerator, player beds and consoles, traditional gift
  shelves/cushions, open books, stools, dining tables, Center treatment console,
  Mart racks/vending machines, PokeCom workstation/chairs/plants, station gates
  and planters, casino cabinets/plants and complete bedroom computer assembly
- Source-verified Center and Mart native top/front counter segments

## Deliberate remaining boundaries

- Most continuous house/player-house/mansion north-wall courses, floor finishes,
  doors and stair flights still use existing architectural geometry. Only the
  explicitly listed complete wall/window/door profiles are converted
- Dynamic bedroom posters, rugs, plant and console variants beyond the existing
  compiled matching remain for the decoration/actor lane
- Laboratory side apparatus, counter corners, office/radio studio layouts,
  Battle Tower fixtures, escalators/elevators and specialty service fixtures
  need additional complete source signatures. A broadcasting mixer and café
  table asset are authored but not yet bound to a source drawing
- Mixed or cropped source drawings, missing ground samples, and unknown or
  intentionally customized live profiles remain legacy rendering
- Full-game coverage is established by the integrated per-cell audit, not by
  the asset count. Integration captures and crate tests are owned by the main
  task; this lane does not run concurrent shared Cargo builds

## Validation

`python3 tools/check-interior-models.py` verifies all 68 exports, material
variation, finite positions, unit normals, indices, full-volume bounds, triangle
counts and embedded references. Rust tests cover complete-source claims, original
ground restoration, phase/art corruption, missing ground, clipped drawings,
overlap rejection, customized profile ownership, fitted bounds and valid normals.
The Blender studio overview was rendered and visually inspected.

## Second source-complete pass

The kit now has 68 full-volume assets. `mesh/interior_signatures.rs` adds 128
individually inspected source drawings, with explicit source cells and subtile
phase. These are selected semantic matcher signatures, not an exported tileset
catalog. New model roles include domestic wainscoting/wallpaper, traditional
exposed timber, clinical dado panels, acoustic treatment, stairs with individual
nosings/stringers/handrails, a thick picture frame and an office telephone.

Bedroom block changes select distinct feathery, pink, polkadot and geometric
Pikachu quilts; magna, tropical and jumbo plants have distinct leaf arrangements
and silhouettes. The existing resolved-actor path separately owns the four game
consoles, 23 ornaments and three large dolls. Rugs remain original horizontal
floor finishes. Picture frames preserve the exact live poster subject as a
recessed textured surface; this is intentionally source-preserving mixed media.

Additional exact wall/window bands cover ordinary homes, player homes/bedroom,
traditional homes, mansion rooms, Centers, Marts and gates. Radio rooms gain
acoustic panels, shelves, control consoles and seats; the Battle Tower gets
source-bound reception and waiting-room fixtures. A paired source window or
bookcase draws two independently fitted 16px-wide units rather than stretching
one object over both. Complete existing house staircase signatures draw closed
stairs, retaining original per-cell footing. Descending stairs restore the well
floor below the room datum, so a zero-height floor cannot hide the flight.

Remaining boundaries still include irregular connected wall/divider networks,
service-specific escalators/elevators, some radio/lab side equipment, complete
multi-part desk feet and untouched mixed source drawings. The integrated
388-map audit is the authority for remaining cells. The second studio overview
was rendered and visually inspected after all 66 assets were exported.

## Source-proven surface finish and room backing

Call `modeled_interiors::finish_surfaces(mesh, map, original_cells, geometry,
world_grid_origin)` at the end of terrain assembly. It returns the number of
finished native-floor quads. It intentionally leaves `authored_cells` untouched:
planks, tiles and tatami are surface finishing, not added modeled objects.

The finishing pass accepts only horizontal, one-native-cell quads whose UV bounds
exactly reference a whitelisted flat-floor source tile. It preserves their
existing height and all actor footing. New oak boards have restrained variation,
sparse staggered end joints and lower-contrast seams; clinical ceramic, civic
stone, mansion marble and traditional tatami retain distinct materials. Pattern
phase comes from the map-space grid origin, so scrolling cannot move joints or
change board colors. Removed source vertices are compacted out of the textured
mesh. No unknown prop strip, original poster, upright picture or stair tread is
covered over. The few wallpaper samples previously used beneath complete modeled
fixtures are interpreted as floor backing only where a model was actually drawn.

Complete source-verified north-wall courses give ordinary houses, player homes,
the player bedroom and traditional houses continuous modeled backing and a
shared cornice. Exact staircase columns remain open. The complete bedroom
blueprint additionally has a shallow below-floor cutaway edge. No side wall is
inferred from collision, and a cropped viewport never becomes a new room edge.
The first actual in-game bedroom capture exposed disconnected panels and harsh
native stripes; this pass addresses those specific issues. Its final appearance
must still be checked in the integrated native capture.

Two source-reviewed civic refinements complete the 68-asset checkpoint: a radio
equipment rack with tape reels, scope display and controls; and a Battle Tower
reception counter with a workstation and visitor writing pad. Center wall
displays preserve their actual contents in separately modeled frames.
