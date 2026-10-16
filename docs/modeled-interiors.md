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
indices from legacy/live matching, preserving immutable original cells, shapes
and atlas slots. Production appends with the complete furnishing list so chairs
can face their adjacent tables:

```rust
modeled_interiors::append_with_tables(
    &mut mesh, map, &original_cells, &geometry, placement, &mut claims,
    world_origin, &placements,
);
```

The append returns false without changing either mesh or claims if any owned cell
is already claimed. A successful append restores the existing floor sample,
places geometry within the drawing's footprint, sets claims and records
`TerrainMeshData.authored_cells`. Explicit live-profile footing values are
preserved. Native Radio stair flights replace support only in their four owned
cells; other furnishings retain the caller's original footing. Model fitting
inverse-transforms normals and shades solid facets; native material adaptations
can sample the authenticated source atlas.

The semantic live-profile allowlist is exact. Sources are matched against the
existing checked-in canonical profiles, including maps, tilesets, complete
metatile grids, every subtile phase and every tile index. A customized matching
live profile keeps ownership and is not overridden even by a compiled matcher.
There is no copied content-pack catalog or new source-data dump.

## Room perimeter and camera cutaway

`mesh/room_perimeter.rs` continues rear wall panels around the west, east and
south edges of enclosed source maps. It reuses the room's modeled architecture
or original wall bands. Where no reusable panel is available, plain returns
use the rear drawing's field paint. Wall geometry stays outside
the playable floor; source warp positions preserve boundary exits, and authored
Radio Tower glazing keeps its perimeter opening. Outdoor rooftops stay open.

The perimeter uses the real source-map extent, never the viewport edge. Each
section records its inward-facing plane. The existing whole-object fade system
hides a section completely when the camera is outside that plane, then restores
it as the camera moves around the room. Cameras inside the room retain its
walls. This policy travels with cached and worker-built terrain and does not
change collision, scripts, warps, source cells or actor support heights.

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

The enhanced room-floor pass also finishes the complete four-tile arcade
checker pattern, Sprout Tower's three floors, and the verified timber/stone
floor strips in Cianwood, Ecruteak, Pewter and Violet Gyms. Sparse source phase
masks leave invisible paths, pits, warp carpets and animated pillar artwork
intact. The verified tower depth fields receive dark timber while remaining
horizontal at their existing datum. Custom floor profiles retain ownership. These finishes
preserve every support height and earn no modeled-object claims. Run
`python3 tools/check-enhanced-room-floor.py --pack content-packs/core-modular.browser.crystalpack`
to check native floor identities against the external pack's FLOOR quadrants
and separately verify the intentional blocked depth fields.

All eight Sprout Tower 2×2 ladder drawings, including the two northeast corner
variants, now have original timber rails and
seven rungs. Upward ladders stand at the north of their plot; downward ladders
sit in an open recessed well. Source identity, complete drawing, ground and
map scope are checked before reserving four cells. This geometry preserves
support and uses the existing warp/controller path. The production audit adds
exactly 32 ladder cells. A native 1F-to-2F ladder transition passes; further
routes remain to be checked. The central timber pillar uses a complete-source animated mesh driven by live
pack frames. Its base stays anchored while the upper geometry and normals follow
each pose; custom ownership and clipped art retain the source fallback.

Game Corner row banks fit two individual arcade cabinets back to back, with
screens and controls facing the west and east stools. The quarter-turns fit
before rotation and rotate normals too; source claims and ground stay intact.
Cianwood Gym's static tower-atlas boulders now share Pewter's complete 2×2 rock
replacement. Movable Strength boulders retain the existing actor renderer and
gameplay state; altered or partial static drawings retain the native fallback.
Casino stools now fit a 10px seat diameter inside each 16px source plot.
The north service areas use complete-source shelves, payout workstations,
connected counter pieces and wall panels, including the Celadon Prize Room.
Cianwood and Ecruteak perimeter strips use standing timber walls, matching their
partition kit; both rear guardian statues and their adjacent wall halves are
modeled in each Gym. Ecruteak’s rear corner strips retain the invisible-path
puzzle’s exact floor drawings and support. Reserved source cells, partial
strips and altered source artwork fall back without a partial replacement.
The four targeted Gym regressions and packed room-floor checks pass; native
views verify both timber Gym layouts. Ecruteak’s audit records 96 perimeter
cells and only floor/dark-puzzle identities left native.
All replacements leave support and custom profile ownership intact.
The targeted production audit leaves only floor/entry-carpet identities in
Goldenrod Game Corner, Celadon Game Corner and its Prize Room, and Cianwood
Gym. Native views verify these base layouts; this is not a dynamic-state or
whole-game gameplay certification. The remaining Gym and tower queue stays open.

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

The requested palette pass retains native room charm: department-store floors
use a continuous warm ceramic grid, windows are blue, and seating is raised pink
upholstery on a frame. Arcade checker
floors keep their cream outlines, dark border and grey inset at the original
16px phase; slot cabinet enamel is pink with gold trim. These are scoped room
finishes, preserve support and source claims, and avoid changing unrelated
rooms that share the same models. Native arcade views verify the pink/gold checker-room palette. The revised Goldenrod 1F native view confirms even ceramic spacing and
raised seats with visible frames/legs. Creative liberties are acceptable; room
quality, proportions and a coherent finished scene remain required.
Four enhanced floor and slot orientation/seat regressions pass. Full room
completion still requires the remaining architecture and interaction review.

## Radio Tower stair contact

The twelve flights on RadioTower1F through RadioTower5F use complete four-cell
source drawings, with per-floor counts 1, 2, 3, 4 and 2. Upward flights match
block 0x03's right half or 0x0c's left half; downward flights match 0x0d's right
half or 0x1d's left half. Exit carpets and Lavender Radio Tower do not select flights.
Missing ground or native materials, altered/cropped art and custom ownership
retain source fallback.

The existing selected stair model supplies the geometry, gray/white source
finishes and eight treads. Its complete rail envelope is fitted to 12 pixels
over the 16-pixel run, keeping the stone treads visible at the normal camera
angle. Descending wells have an ink-colored bottom. Support
uses the fitted stone tops and their beveled nosings, excluding rails and
stringers. Per-point contact requires the complete source and all four expected
native support samples, so a fallback drawing cannot create raised footing.

Pack-backed `radio_stair` tests cover all twelve endpoints, source paint,
ownership/fallback and contact against rendered surfaces at three tile scales.
The controller regression
`radio_tower_all_twelve_stairs_keep_authored_step_off_and_allow_reentry` covers
the authored southward step off a staircase and reentry through the production
controller. Native visual acceptance is tracked separately in
[the interior handoff](handoff/kanto-interiors.md).

Sprout Tower's longitudinal timber strips extend through the two source rows
before a verified transverse wall face. The side and cross wall retain their
own source claims and floor support; custom neighboring ownership rejects the
extension. East and west production views verify the joined corners.

Radio studio east glass returns authenticate the complete terminal pane, table,
and floor footer, then follow the connected glass run to its outer corner.
Their placement is independent of the padded render grid. Both exposed sides carry the live glass drawing with outward
triangle winding; the closed return meets the front panel at ceiling height.
On RadioTower5F, the front panel ends at a doorway: no inferred return is
placed there, including when altered or cropped side art uses source fallback.
The actual side partition follows the pale cap at source column 35.

Complete pale side-cap drawings on RadioTower2F, RadioTower5F and
LavRadioTower1F produce ceiling-height glass partitions at the marked strip.
Their 24 source cells retain white top caps, with native glass on both faces
and vertical mullions. A missing complete front-pane material donor preserves
the original source art. Confirmed glass on an outer map edge replaces the
opaque perimeter segment over the same span. Three production entrance walks
and paired east/west views verify the openings and wall placement.

Tin Tower's nine interior floors contain a complete blank block outside the
playable room on the west, east and south. Perimeter walls meet the inner edge
of that authenticated block instead of leaving a black strip beside the floor.
Altered source borders and other rooms keep their existing boundary behavior;
floor support, source ownership and the first-floor exit opening are preserved.
Native third-floor default/east/west views and the first-floor entrance view
verify these joins. Tin Tower's timber-barrier hops keep a level floor, as
specified by the production terrace model; the audit checks level endpoints
there and descending endpoints for other ledges.
