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

A follow-up side review exposed false doorway gaps behind edge-adjacent stairs.
Tin Tower stair and teleport warps now retain the perimeter backing; only the
two first-floor exterior door tiles open the shell. The pack-backed wall test
checks this distinction against every warp destination on all nine floors.
Four fresh paired views cover fourth-floor east/west, the eighth-floor left
wall and the first-floor entrance. Production inputs verify the fourth-to-fifth
floor stair warp and the first-floor exit through to Ecruteak (37, 8). Floors,
collision and warp behavior remain authoritative and unchanged.

A full-span regression now checks both side walls on every Tin Tower floor,
so separate panels at each end cannot conceal an intervening stair-sized gap.
The check passes against the external pack; fresh fourth- and eighth-floor
left-side paired views confirm the continuous backing at the stair edges.

The PokéCom admin-office map retains its native blue patterned floor and shaded
fixture aprons at the original texture scale and walking height. Exact source
phases distinguish those surfaces from equipment and the solid-black void
between its two rooms; the void no longer receives generic gray paving.
Custom floor profiles retain ownership. Paired lobby/office views and an
ordinary left/right walk verify this floor correction.

Its twelve lobby seats now use independent compact upholstered cushions with
native orange top drawings, small gray feet and a seven-pixel seated height.
The three office computers retain selected geometry with native gray electronics
paint. Their palette samples exclude the red counter lip above the monitor;
complete source drawings and material-role guards preserve fallback behavior.
Ten basic lobby counter courses now have closed volumes, an eight-pixel top
height and depth, and their original cream tops and red front drawings.
Each complete eight-cell source course retains independent claims; the lower
floor apron and all walking support remain separate. Altered or reserved source
cells reject the whole course. The two short service-counter returns now meet
the transverse course with cream caps and red sides. Their blue corner controls
sit on the countertop as closed tilted volumes, preserving the complete source
glyph. Complete return/control/footer drawings resolve together; missing or
reserved cells retain the source fallback. The return pigment uses the source
tile's bank-one mapping. Equipment and broader room reconstruction remain open.

Final PokéCom verification passed nine focused tests, the neighboring lounge
and seating-resolver controls, and the production geometry audit. Three paired
native views cover the lobby, office and lobby side; ordinary left/right lobby
and down/up office walks preserve the source state and furniture clearances.
The first computer render exposed red paint from a countertop sample; the
final bindings use the gray electronics tiles and palette bank, verified again
in the native renderer. Source claims and walking support remain unchanged.

The counter pass adds one pack-backed regression covering all ten courses,
closed outward faces, every damaged-cell and reservation rejection, untouched
aprons and unchanged support. All ten PokéCom controls pass. Four paired native
views cover the lobby, close counter, side and office control; a production
approach stops on the original floor tile before the blocked counter. The
targeted geometry audit passes with 158 authored source cells, including the
80 new counter cells. This verifies the basic courses, not the complete service
enclosure or its countertop items.

The short-return pass brings the focused PokéCom suite to eleven passing tests
and the targeted geometry audit to 166 authored source cells. Four paired native
views verify both corners and east/west joins. Ordinary two-step walks along
both outer aisles reach their original destination tiles; paired final views
and four movement frames verify clearance. The return joins have no overlapping
front face, and walking support is unchanged. These checks cover two returns
and two controls, not the longer enclosure or the remaining flat equipment.

The two long returns now join their rear caps and front courses at the same
eight-pixel height. The right corner has a raised blue control using the same
complete source glyph treatment as the short returns. Each return, rear cap
and optional control resolves as one source-guarded assembly; damaged or
reserved cells reject the whole assembly. The front course must already be
resolved. The new geometry adds 31 source claims without changing support.
All twelve focused PokéCom tests pass, and the targeted production audit passes
with 197 authored cells. Three paired native front/east/west views and ordinary
two-step walks along both outside aisles verify joins and clearance; both final
views and four movement frames were inspected. The enclosed equipment remains
flat and needs its own complete source geometry.

The western clinic wing now has a native recessed healing enclosure, public
PC, and the two-part blue cabinet/console. Its ceiling-height rear wall joins
behind every appliance and the mounted picture; three source lamps sit on the
same wall. Complete source drawings and local wall-paint donors guard the
models. The regression checks all 24 wall columns at four heights, source
damage/reservation rejection, appliance heights and unchanged floor support.
Mounted art bypasses the generic picture overlay, which otherwise duplicated
the lamps above the wall. PokéCom's separated nonrectangular rooms no longer
seed a generic rectangular wrapper through their black separation and alcoves;
their remaining boundaries require source-aligned geometry.

Final verification passes thirteen PokéCom checks, six neighboring Kanto clinic
controls and the room-wall donor control. The targeted production audit passes
with 259 authored cells (62 added by clinic appliances, wall and mounted art).
Four paired native views cover the clinic front/east/west and unchanged office.
The ordinary counter approach stops at (5, 9), while the PC-side aisle walk
reaches (10, 7); both final comparisons and four movement samples were viewed.
Earlier disconnected appliance backing, overextended room wrappers and duplicate
lamp overlays were rejected and corrected before this verification. This is
western clinic geometry acceptance, not a service-scene or whole-map claim.

PokéCom's stepped lobby and separate office now have source-aligned side and
south walls. The black separation stays open, both red exit mats retain their
openings, and outside corner thicknesses overlap without entering the room
floor. The boundaries use native wall paint and production camera cutaway
metadata. Edited room/void topology rejects this map-specific enclosure;
support and source ownership are unchanged by the perimeter.

Four office and ten upper-lobby window courses now stand on ceiling-height
backing. The office keeps its full three-row window drawing above the base
band. The lobby uses complete four-row courses; where machinery overlaps the
lower half, a matching authenticated course continues behind it while the
machinery cells remain unclaimed. Folded wall-height rows emit no horizontal
floor underlay behind the windows. Window source claims add 184 cells, bringing
the targeted production audit to 443 authored cells with zero mesher errors.

Final fourteen PokéCom checks and the room-wall donor control pass. Seven paired
native views cover the clinic, upper lobby and office from front and side
angles. Three ordinary input routes preserve the west aisle and office aisle,
and take the office exit through (1, 31) to GoldenrodPokecenter1F (0, 6).
All final comparisons and six movement samples were inspected. The earlier
flat lobby window course and the floor strip behind the office windows were
corrected before this final review. This establishes these walls/window joins
and routes, not all equipment, service scenes or the full interior scope.

The office north bar, its short left return and the three south counter
courses now have closed eight-pixel volumes with native cream tops and red
fronts. North courses own their visible front strips without consuming the
adjacent computer drawings; a reserved front rejects the whole course. The
left return meets the bar at its front edge. The two office plants now use
their own bank-one green foliage and bank-zero gold pot pigments, with dense
leaf clusters and one pot per source plant. Their existing height and footprint
remain unchanged. Counter claims add 48 cells; all support remains authoritative.

Fifteen focused PokéCom checks pass and the targeted production audit passes
with 491 authored cells. Four paired native views cover office front/east/west
and the unchanged clinic. Three ordinary input routes verify the office aisle,
the exit to GoldenrodPokecenter1F (0, 6), and the left PC approach to (3, 27).
Pressing A there displays the original administration-office email's first
dialogue page in both views. All final comparisons and nine movement samples
were inspected. The small gray patterned fixtures beside the PCs retain their
source presentation while their object identity is under review; the broader
office/equipment and service acceptance is still incomplete.

Remaining interior work retains the full 311-map scope and the original 16
Kanto block combinations. PokéCom's central communications machinery, remaining
equipment and other counter configurations still need geometry; the office
side fixtures and complete room composition need room-level review. Other
outstanding requirements include clinic restoration
and service scenes, remaining League traversal/materials and native Hall
ceremony animation review, house/lab/facility
furniture and materials, cave/gym/port views, script-variant reachability and
native variant review. Timed station boarding remains deferred following the
request to work beyond the station. Geometry audits and these scoped furniture
checks do not establish whole-scope visual or gameplay acceptance.

Hall-of-Fame acceptance follow-up: ordinary production Up/Down inputs move
from (4, 4) onto (4, 3) and return to (4, 4). The paired final view preserves
state; both moving frames were inspected. These establish a native stair route,
but the room still needs its rear/side walls and full floor/material review.
The remaining flat rear drawing and largely uniform blue platform prevent
whole-room visual acceptance.

The earned Lance test now has a separate full-ceremony gate,
`kanto_lance_controller_completes_earned_ceremony_and_continues_home`. It wins
the actual battle before continuing the original dialogue and machine approach;
it never seeds the victory flags or script cursor. Its intended remaining
assertions cover recording, every ceremony phase, THE END, title return and
Continue to New Bark. The established battle-to-Hall gate still passes.
Initially the new gate failed with both then-current text and native packs at
`credits.rs::load_visible_credits_program`: zero `credits_source_data` operations.
The failure happens after the native slow step from (4, 4) to (4, 3), when
`script_callbacks.rs` opens credits for `HallOfFameRequested`. At that point full ceremony,
recording/Continue acceptance was unproved. The source/export and controller
repairs below address this gameplay blocker; the scene must never be skipped.

Receipts: `target/kanto-resume/receipts/lance-entry-and-full-ceremony-gates.log`
(one existing gate passes, the full-ceremony gate fails against the native pack),
`lance-earned-complete-ceremony.log` (text-pack failure), and paired native
`target/kanto-resume/hall-stair-walk-review/`. This failure led to investigation of the Rust
`pack_core/source_title_fragment.rs`/`source_title_screen.rs` title-export path.
The subsequent source-backed repair and rebuild results are recorded below;
no export was attempted during that initial failure-discovery pass.

Credits repair is implemented in Rust. `pack_core/source_title_credits.rs`
adds the missing operation from the pinned external credits script, constants
and strings. Exact source sequences certify the skip threshold, music fade,
menu state and palette-clear behavior. The focused external-source test passes,
including rejection of changed skip/fade/menu instructions. The initializer
now requires the earned Lance-to-ceremony gate before pack publication.

The gate also exposed a renderer-neutral clock/input defect: credits input
was dispatched without ticking the presentation, and unsupported directions or
Start could reach the underlying map. `VisibleShellController::press_buttons`
now advances the shared production credits ticker once and retains only A/B
while credits own input; `wait_frames` uses that same path. The regression
checks a walkable Down direction and Start remain blocked while time advances.
The final Continue check uses the title menu's production save transaction,
not raw session restore. Test saves clean themselves up on ordinary unwinding.

The rebuilt external candidate passes the complete earned ceremony gate in
14.28 seconds with the local shell compiled at opt-level zero and
`RUST_MIN_STACK=16777216`. Its default test-thread stack overflows during the
longer path at that optimization setting. The ordinary optimized initializer
gate passes without a stack override in 12.55 seconds. Ten first-phase frames were inspected in
`target/kanto-resume/earned-hall-ceremony-review/`; the entry scroll phases begin
offscreen, and the member/portrait/rating frames retain native artwork. This
is functional ceremony verification and source-frame review, not a native
window recording of every animation frame or whole-room visual acceptance.

The completed rebuild certified all 28 intro scenes and verified all
377 audio programs' PCM hashes, frame counts and loop ranges. Candidate and published-pack
fingerprints match the prior pack for all non-title game data, all 4,915 runtime
assets, audio manifests, compression metadata and compiled browser audio.
Receipts are `credits-source-lowering.log`, `credits-earned-ceremony-large-stack.log`,
`credits-preservation-check.txt` and `credits-pack-rebuild.log` under
`target/kanto-resume/receipts/`. The initializer completed with exit zero and
published all three canonical ignored packs after its optimized battle,
Mom/New Bark, Research Center, earned ceremony, MCP and clean-stderr gates
passed. The TUI idle loop now advances six controller frames per 100 ms poll,
so credits animate without repeated input while preserving their A/B waits.
The final production client was launched at PlayersHouse2F (3, 3); Enter opened
the production Start menu and Ctrl-C exited cleanly. The verified rebuild
command is `CRYSTAL_PACK_BUILD_PROFILE=wasm-dev CRYSTAL_PACK_BUILD_DIR="$PWD/target/kanto-resume" CARGO_BUILD_JOBS=1 bash tools/build-crystalpack.sh --jobs 1`.
No generated packs or source exports belong in tracked source.

Hall of Fame now has a native rear course and connected ceiling-height
perimeter in `mesh/hall_room.rs`. The rear face stands behind the recording
body at the map edge, with 32-pixel closed panels and native 82/83 artwork on
both faces. Side returns meet the north/south corners outside the floor.
The south opening spans both source door tiles. The complete native header,
map dimensions, source phases, independent floor donor and door coordinates
guard this enclosure; an edited/cropped drawing or claimed wall field rejects
it. Only 34 plain wall source cells fold into the wall. The six interleaved
machine cells, collision and resolved support remain independent. Folded cells
receive the original checkerboard floor, without duplicate horizontal wall art.

The platform wings and exposed landing now retain their spatial native border
pixels, replacing the earlier uniform top paint. Their dimensions, support,
source ownership and existing stair treads are unchanged. Regression coverage
checks the full 1,280-pixel native platform area and UV placement, closed-panel
winding, continuous sides/rear, door clearance, source rejection, exact wall
claims, preserved machine triangles and unchanged contact heights. Eleven
selected Hall/source-control checks pass; the room-wall donor control also
passes. The final native build succeeds and the targeted audit reports
86/560 modeled source cells with zero mesher errors (34 newly claimed wall
cells; existing platform and terminal claims are unchanged).

Final paired front/east/west/entrance views and the actual earned Lance
dialogue save were inspected. All preserve state. The floor scale and native
wall/platform details are accepted for these reviewed views. Four production
input routes also preserve state: stairs Up/Down return to (4, 4), Down from
(4, 12) approaches the door at (4, 13), and each side aisle moves from y=10 to
y=8 at x=0 or x=9. All final comparisons and seven moving frames were viewed.
An additional reverse-warp preview was rejected because its unearned Lance
room still has a closed destination door at (4, 1); that is not a verified
return-warp route. No victory flags or destination collision were changed to
force it. The earned save must be captured with its matching full native pack,
not the separately identified browser pack.

Final evidence is in `target/kanto-resume/hall-room-final-review/`,
`hall-room-final-earned-review/`, `hall-room-final-walk-review/` and
`hall-room-final-audit.json`. Receipts under `target/kanto-resume/receipts/`
include `hall-native-room-and-platform-tests.log`, `hall-wall-room-controls.log`,
`hall-native-final-build.log`, `hall-room-final-audit.log`,
`hall-room-final-earned-capture.log` and the rejected-preview diagnostic.
Earlier uniform-platform captures were superseded and removed. This closes
the scoped rear/perimeter/floor-underlay and platform-finish correction. Full
native ceremony-animation review and the remaining League/interior families
still require acceptance. The full 311 interiors and original 16 Kanto
variants remain the goal; no pack generation or audio work was needed here.

Koga's six complete 3x3 tree drawings now reuse the closed Azalea broadleaf
geometry at a 24x24-pixel footprint and 28-pixel height. Their original source
anchors are (0,12), (16,12), (0,20), (16,20), (4,24) and (12,24).
The named foliage materials sample the native green palette bank; bark samples
the separate native brown bank. This corrects the rejected bright-green trial
without changing the existing Azalea paint. Complete source/phase and ground
donor guards protect all 54 tree source cells. Altered drawings, reserved cells
and custom profiles keep their original ownership. Collision, scripts and
resolved ground support are unchanged.

The external-pack six-tree regression, all 15 scenery controls, the model-volume
control and the native-material guard pass. The latter independently renames
each of the six required material roles, rejects incomplete bindings without
partial writes, and verifies cached geometry/paint are unchanged. The native
build succeeds. The targeted audit reports 274/720 modeled source cells and
zero mesher errors; this count includes the room's existing fixtures.

Front/east/west/lower paired comparisons and a Karen control were individually
viewed at noon with preserved source states. The Koga tree scale, volume and
palette are accepted in these views. Four production-input routes also pass
with preserved paired states: center (4,9) through (4,11) and back, lower aisle
(4,13) through (4,11) and back, and side aisles at x=2 and x=7 from y=8 through
y=10 and back. All four final comparisons and one middle moving sample per
route were inspected. These checks establish clearance on the reviewed paths,
not battle completion or whole-League acceptance.

Final evidence is in `target/kanto-resume/koga-native-tree-review/`,
`koga-native-tree-walk-review/` and `koga-native-tree-audit.json`. Receipts under
`target/kanto-resume/receipts/` include `koga-native-tree-source-test.log`,
`koga-native-final-gym-controls.log`, `koga-native-model-controls.log`,
`koga-native-material-guards.log`, `koga-native-paint-build.log` and
`koga-native-tree-audit.log`. Rejected flat/bright-green trial captures were
removed. Karen's platform/void-edge composition and League terrace material
seams still need source review; the Koga terrace's dark base strip is not
accepted by this tree-only correction. Remaining furniture, PokéCom machinery,
native ceremony animation and all 311 interiors/original 16 Kanto variants
remain active requirements. No pack or audio changes were needed.

The following terrace review resolves the previously pending dark base strip.
`league_terraces::finish` had removed the horizontal source row at y=10 along
with the stair drawing, leaving a 128x8-pixel hole outside the flight. It now
replaces only the actual stair drawing, preserving that paving row in Will,
Koga and Bruno. Original horizontal normals, full native texture phase and
zero floor support remain unchanged; preserved paving acquires no object
claim. The existing terrace faces, stair dimensions and actor height sampling
are unchanged. Karen never used the deleted-row branch and retains its source
layout.

All three terrace checks and both League floor/barrier controls pass. The new
regression checks the complete 1,024-pixel exposed row, native UVs, floor support
and ownership in all three affected rooms. The native build succeeds; the
four-room audit has zero mesher errors. Final Koga front/east, Will front and
Bruno front comparisons were individually viewed, as were both walking
comparisons and a middle moving sample from each route. All six captures
preserve their paired source states. The repaired row now meets the base of
the terrace without the former black band. Production inputs cross Koga's
row from (3,5) through (1,5) and back; a separate Up/Down route traverses
(4,6), (4,5), (4,4), (4,3) and returns to (4,6).

Evidence: `target/kanto-resume/league-terrace-floor-review/`,
`league-terrace-floor-audit.json`, and receipts
`league-terrace-floor-continuity-tests.log`, `league-terrace-floor-controls.log`,
`league-terrace-floor-native-build.log` and `league-terrace-floor-audit.log`.
The source-tile inspection image was temporary and removed. This closes the
missing paving strip only. Karen's native void lips remain flat and require
source-guarded depth; Will/Bruno's water/lava presentation, terrace face finishes,
earned battle/door progression, native ceremony animation and the broader
311-interior/original-16-variant acceptance still remain. No generated content
or unrelated changes were modified.

Karen's previously flat void lips now have source-guarded native depth in
`mesh/league_recess.rs`. The shared Violet/Karen matcher proves complete $17
void and $19 lip drawings; only the blocked lower half of each lip is owned.
Karen's native adapter requires both blue course donors ($13/$12) and a proven
black void donor. Missing donors or custom ownership preserve the source
drawing. Two native eight-pixel courses fold down from the existing platform,
then native black closes the cavity at 24-pixel depth. Narrow mitered coping
stays inside the blocked cells. The platform's native floor, actors, support,
collision, scripts and stairs are unchanged. Violet keeps its previous
geometry and finish.

All 22 recess-related controls pass, including the external Karen regression.
It proves exactly 160 owned cells, unchanged support, inward face winding,
10,240 pixels of bottom area, narrow blocked-cell coping, source-only texture
bindings, donor mutation rejection, custom-void fallback and whole-lip
reservation. The native build succeeds. The production Karen/Violet audit has
zero mesher errors: Karen reports 588/720 modeled cells, including all 160
`league:karen-recess-depth` cells; Violet remains 280/640 with no Karen claims.
These are source ownership counts, not whole-room acceptance scores.

Front/east/west/lower Karen comparisons and a Violet entrance control were
individually inspected at noon. All preserve paired state. Three production
walking routes also pass: each Karen outer platform edge at x=2 or x=7 from
y=8 through y=10 and back, and the narrow bridge at x=4 from y=11 through
y=13 and back. Their final comparisons and one middle moving sample per route
were viewed. Recess depth, edge paint and clearance on these routes are
accepted. The first Violet control position (4,9) was blocked and never
rendered; the completed control uses its walkable entrance aisle at (4,14).

Evidence is in `target/kanto-resume/karen-native-recess-review/`,
`karen-native-recess-audit.json`, and receipts `karen-native-recess-tests.log`,
`karen-native-recess-build.log` and `karen-native-recess-audit.log`.
This supersedes the earlier pending Karen void-depth item. Northern terrace
backing/material composition, Will/Bruno liquid presentation, earned League
battle/door progression, full native Hall ceremony animation, remaining
PokéCom machinery/shared furniture and all 311 interiors/original 16 Kanto
variants remain active. No content-pack or audio changes were needed.

The subsequent close review found Karen's upper-landing black band was an
uncapped strip behind the lower wall. The authored wall's ordinary backing
floor is complete, as a new geometry regression verifies for all four League
rooms; its compact body occupies the front five pixels of the sixteen-pixel
source drawing. Karen's blue landing previously ended before that body, leaving
its backing exposed at the lower datum. The terrace finisher now adds a native
blue cap at height 16, ending at the existing model's measured top shoulder.
It retains the body's front/top finish and closes the corresponding stair
side faces to the same landing height. Floor samples come from the native
upper landing. Altered wall art and custom overrides reject the assembly.
Source ownership, blocked-cell support and stair tread sampling are unchanged.

All eight League controls pass with the absolute external pack path, including
the new full-backing and cap/shoulder/stair-side/source-guard checks. The native
build succeeds. The four-room audit has zero mesher errors and unchanged
modeled-cell counts (Karen 588/720, Koga 274/720, Will 224/720, Bruno 220/720).
Fresh Karen front/upper/east, Koga upper-control and two walking comparisons
were individually inspected; all six preserve paired states. Upper-aisle
inputs traverse (2,3) through (4,3) and back. A separate stair route traverses
(4,6), (4,5), (4,4), (4,3) and returns to (4,6). Three moving samples cover
the upper aisle, ascent and descent. The cap closes the black trench and its
stair join is accepted in these views. Earlier uncapped diagnostic captures
were superseded and removed.

Evidence: `target/kanto-resume/karen-landing-cap-review/`,
`karen-landing-cap-audit.json`, and receipts `karen-landing-cap-tests.log`,
`karen-landing-cap-native-build.log`, `karen-landing-cap-audit.log` and
`league-upper-landing-backing-test.log`. This closes the scoped landing
composition defect, while League wall material fidelity, Will/Bruno liquids,
earned battle/door progression, native Hall ceremony animation, other shared
furniture/PokéCom equipment and full 311-interior/original-16 acceptance remain
open. No content packs, audio or unrelated source were changed.

`mesh/league_liquids.rs` now finishes Will's water and Bruno's lava as shallow
native basins. Complete $1f/$24 and $2b/$2c drawings prove each source phase;
custom profiles or existing claims protect the whole affected drawing. Only
the liquid rows move to -2 pixels. Existing atlas coordinates and animation
slots remain bit-identical. Closed boundaries sample their adjoining native
paint, and bottoms sit at -4 pixels. Floor support, collision, actors, scripts
and object ownership remain unchanged. This is a terrain finish and earns no
additional modeled-object claims.

The external-pack regression proves 144 Will cells and 160 Bruno cells,
unchanged original UVs/support/ownership, finite inward boundary winding,
complete bottoms, altered-drawing rejection, whole-drawing reservations,
custom-profile protection and Koga isolation. All nine League controls pass;
the native build and four-room audit succeed with zero mesher errors and
unchanged modeled-source counts. Both rooms' front/east comparisons and both
bank walks were individually inspected against matching noon 2D states; all
six preserve state. Each ordinary route stays at x=2 and moves y=8,9,8,7,8.
The final walking comparisons and one middle moving sample per room were
viewed. Shallow depth, native palette, and clearance are accepted here.

Two same-camera earlier/later 3D pairs were also viewed. Bruno's native bubbles
advance: 20,705 of 234,084 sampled red pool pixels change in the fixed left-pool
region. Will's 248,038 sampled blue pool pixels stay identical. The external
pinned `data/tileset_anims.asm` Elite Four routine lists only the two lava
bubble updates; the production `bitmap_font.rs` likewise animates $5b/$38.
Will's water is intentionally static in this source. No replacement clock or
invented water animation was added. These temporal pairs advance presentation
and are separate from the six state-preserved 2D/3D comparisons.

Evidence: `target/kanto-resume/league-native-pool-review/`,
`league-native-pool-audit.json` and receipts `league-native-pool-source-tests.log`,
`league-native-pool-controls.log`, `league-native-pool-build.log`,
`league-native-pool-audit.log` and `league-native-pool-temporal-check.txt`.
The scoped liquid-depth/paint/animation item is verified. The blue bank
drawings still need folded architecture and whole-room finish review;
League wall materials, earned battle/door progression, native Hall ceremony
animation, other shared furniture/PokéCom equipment and the full 311 interiors
and original 16 Kanto variants remain active. No generated content, audio or
unrelated work was changed.

Both rooms' blue banks now have closed native wall bodies in
`mesh/league_banks.rs`. The existing full-pool source matcher proves each
complete drawing. Four original 4x2 bank faces stand on 32x4-pixel bodies at
16-pixel height; each face retains its two source rows exactly once. Bounded
native samples finish their caps, backs, ends and undersides. The folded
horizontal blue art is removed, and proven native green floor fills the whole
original footprint underneath. Each body is one cutaway group; floor backing
remains opaque. The liquid, Will's wave apron, animation UVs, collision and
resolved support remain independent. Changed donors and custom profiles keep
their source presentation.

All ten League controls pass. The new external regression proves four exact
anchors per room, 32 owned source cells, closed positive volume and 512 pixels
of native front area per bank, unchanged support/retained UVs, source/donor
mutation rejection and complete custom override protection. It also verifies
that removing all four cutaway groups leaves 2,048 pixels of opaque native
floor under the banks, preventing a black strip during a fade. The native
build succeeds. The four-room audit has zero mesher errors and confirms 32
`league:native-pool-bank` cells in each room (Will 256/720, Bruno 252/720;
Karen 588/720 and Koga 274/720 remain unchanged).

Fresh front/east and low-angle behind-bank walking comparisons for both rooms
were individually inspected; all six preserve paired state. Each ordinary
walk stays at y=5 and traverses x=0,1,2,1,0. Their final comparisons and one
moving sample per room were viewed. A fresh earlier/later Bruno 3D comparison
also shows the native bubbles advancing after the bank conversion. Wall depth,
native material placement, backing and clearance are accepted in these views.
The temporary source-tile inspection image was removed.

Evidence: `target/kanto-resume/league-native-bank-review/`,
`league-native-bank-audit.json` and receipts `league-native-bank-tests.log`,
`league-native-bank-final-tests.log`, `league-native-bank-build.log` and
`league-native-bank-audit.log`. This closes the scoped flat-bank item. A fresh
full 311-map/declared-state geometry audit completed against the current
production example: 311/311 maps, 225 substitution states and 298 jump samples
pass with zero mesher errors. The variant identities and edits match the prior
full alternative-state audit. Coverage remains 162 exhaustive small-room
combinations and 63 bounded declared script prefixes; these are not all
reachable game states. Report/receipts are `full311-league-native-bank-audit.json`,
`receipts/full311-league-native-bank-audit.log` and
`receipts/full311-league-native-bank-summary.txt`. League wall
finish fidelity, earned battle/door progression, full native Hall animation,
remaining shared furniture/PokéCom equipment and all-interior/original-variant
visual and reachability acceptance remain active. No pack, audio or unrelated
source changes were needed.

League wall panels and compound south barriers now use live neutral paint from
their original $10 source face, replacing the generic gray/gold finish.
The named `brass`, `dark_rock`, `stone` and `stone_light` model regions bind to
native white, dark gray and mid-gray pixels. Material-role guards require all
four names; the paint helper rejects unknown roles without partial writes.
Cached positions, normals, triangles, colors and UVs remain unchanged. The
production adapter uses textured native paint, including the perimeter's
existing copied wall panels. No collision, support or source ownership changes.

The compound barrier's top now uses its complete original 4x2 cap drawing.
Eight native quads replace the uniform top paint on the same closed slab;
its 32x28-pixel top area and 16-pixel rise remain unchanged. The slab and fitted
wall stay in one textured cutaway group, while all sixteen underlying native
floor cells remain opaque. The first neutral-paint trial still had plain tops;
the final source-panelled caps supersede that trial. Final comparison sheets
also include the preceding native view for reference; the updated native view
is on the right.

All twelve League checks pass, including cached-model/material-role rejection,
external source-gray/palette proof, complete cap area/cell coverage and compound
fade/floor preservation. The room-wall donor control also passes. The final
native build succeeds; the four-room/twelve-substitution-state audit has zero
mesher errors, unchanged modeled-source counts and finite unchanged support.
Seven final matched noon comparisons were individually inspected: all four
rooms, Karen from the east, Koga's south barrier, and a low-angle Koga barrier
walk. All preserve paired state. The walk moves (0,13) to (1,13) and back;
both moving samples were viewed and show the compound fading together over
complete opaque blue floor. Native grays, patterned caps, joins and readability
are accepted in these reviewed views.

Evidence is in `target/kanto-resume/league-native-wall-paint-review/`,
`league-native-wall-paint-audit.json`, and receipts
`league-native-wall-paint-tests.log`, `league-native-wall-room-controls.log`,
`league-native-wall-paint-final-build.log` and `league-native-wall-paint-audit.log`.
The temporary cap-join crop was removed after inspection. The prior full311
geometry report predates this paint/cap subdivision; the current focused audit
covers every affected room and its declared substitutions. Upper terrace-front
material review, dimensional doorway/open-state presentation, earned League
battles/doors, full native Hall animation, other shared furniture/PokéCom
equipment and all311/original16 visual/reachability acceptance remain open.
No generated packs, audio, external source or unrelated changes were modified.


Will, Koga and Bruno's upper terrace fronts now keep the two native blue $39
wall courses vertical. The prior face incorrectly stretched green floor art
onto that wall. A proven blue landing donor caps the 16-pixel terrace; row-ten
green paving remains at ground level. Closed backs/ends use native wall
pigment, and the stair joins retain their existing gold side faces without
coplanar blue overlays. Exactly 32 source wall cells per room are owned as
`league:native-terrace-front`. Existing claims, altered donors and custom
profiles reject the entire conversion before any mesh mutation. Karen retains
its separate wall/cap treatment. Collision, stair heights and support remain
unchanged.

All thirteen League checks pass, including native face/cap area and source
phase, guards and preserved support. The native build succeeds; the four-room,
twelve-declared-state audit has zero mesher errors. Modeled counts are Will
288/720, Bruno 284/720, Koga 306/720 and Karen 588/720. Eight matched noon
comparisons were individually inspected: all four rooms, Koga from the east,
its upper landing, a front walk and a stair walk. All preserve paired state.
The front route traverses (3,5),(2,5),(1,5),(2,5),(3,5); the stair route traverses
(4,6) to (4,3) and back. Three moving samples covering front clearance, ascent
and descent were also inspected. Blue wall/paving placement and stair joins
are accepted in these views. Evidence: `target/kanto-resume/league-native-terrace-front-review/`,
`league-native-terrace-front-audit.json` and receipts
`league-native-terrace-front-tests.log`, `league-native-terrace-front-build.log`
and `league-native-terrace-front-audit.log`.

This closes the scoped upper-front material correction. Dimensional north
doors and earned League battle/door progression, full native Hall animation,
remaining shared furniture/PokeCom equipment and all311/original16 visual and
reachability acceptance remain open. The full311 audit predates this change;
the current focused audit covers all affected rooms and declared substitutions.
No packs, audio or unrelated source were changed. Fresh Tin Tower fourth- and
eighth-floor west comparisons also reconfirm continuous stair backing, with
paired states preserved; the 255-room donor control passes including full
side spans on all nine Tin Tower floors. Those confirmations are in
`target/kanto-resume/tin-left-wall-confirmation/`.


The four Elite Four north doors now fold their original $22 two-row drawing
onto an upright 32x16-pixel face at the blue landing's 16-pixel datum. A closed
one-pixel slab supplies native-painted top, rear and jambs; blue donor paving
replaces the removed horizontal door art beneath it. The complete $16 open
block omits that slab and keeps its authored passage. Eight closed-door cells
per room are owned as `league:native-north-door`. Complete-block, existing
ownership and whole-assembly custom guards run before any mutation. Support,
collision, source scripts and wall dimensions remain unchanged.

Fourteen League checks pass, including all four closed/open drawings, exact
512-pixel face and backing areas, closed 512-cubic-pixel volume, native UV
phases, existing-owner rejection and unchanged landing support. The production
examples build successfully. The four-map/twelve-declared-state audit has no
mesher errors; base modeled counts are Will 296/720, Koga 314/720, Bruno 292/720
and Karen 596/720. Open substitutions contain no native-door claim. Five fresh
closed-door comparisons (all four rooms plus Koga east) and four earned open
exit-approach comparisons were individually inspected. All nine preserve
paired state. The door meets the adjoining wall and disappears for the actual
open state, with blue paving and no leftover slab across the passage.

The existing production-controller battle regression now also walks from each
victory to its north exit and presses Up through the real warp. It passes all
four routes: Will to Koga, Koga to Bruno, Bruno to Karen, and Karen to Lance.
These are independent entry fixtures with a level-100 Mew to avoid grinding;
victory/door flags are never seeded. The test observes real battles, PP use,
script-driven exit flags and live block replacement before walking onward.
It saves each earned approach for the paired captures. This proves those four
earned exits; it does not prove a continuous gauntlet or a complete animated
native walkthrough. The earlier battle-only test also passed before the new
traversal assertions were added.

Evidence: `target/kanto-resume/league-native-door-review/`,
`league-native-door-earned-review/`, `league-native-door-audit.json`, and receipts
`league-native-door-tests.log`, `league-native-door-build.log`,
`league-native-door-audit.log`, `league-native-door-earned-test.log` and
`league-native-door-earned-warp-test.log`. Full native Hall animation,
continuous League traversal, remaining shared furniture/PokeCom equipment,
and all311/original16 visual/reachability acceptance remain open. The full311
report predates these focused geometry corrections. No packs, audio, external
sources or unrelated changes were modified; no task-owned process remains live.


PokéCom's complete 10x6-cell communications display now stands upright with its
original 80x48-pixel screen/frame drawing, a four-pixel closed housing and an
eight-pixel shallow plinth. Native blue casing paint comes from tile $86,
palette bank three; the gray screen retains its own atlas pixels. The 60 cells
are owned as `pokecom:native-communications-display`. Complete map/block/phase
and donor guards, existing claims and custom profiles protect the whole
assembly before any writes. Other maps reject the conversion. Collision,
actor support, console/control-strip art and the surrounding furniture remain
unchanged. The pinned PokeComCenter animation program contains only wait/done
commands, so no new screen animation clock is invented.

The first native rear review exposed an oversized plinth extending through the
rear wall. That trial is superseded: the final plinth is only eight pixels deep
under the screen, and ground backing is limited to the room side of source row
four. No new physical body or floor projects outside the rear wall. Sixteen
PokéCom checks pass, including all sixty source-cell mutation rejections,
complete source ownership, unchanged support, custom/existing-owner rejection,
native blue pigment/palette proof, outward winding, 3,840 pixels of face area,
20,480 cubic pixels of closed volume and 2,560 pixels of room-side floor. The
screen and base share one cutaway group; ground stays outside it.

The final native build succeeds. The focused audit has zero mesher errors and
551/4096 modeled cells, including exactly sixty display cells. Eight fresh
matched noon comparisons were individually inspected: front, east, west,
zoomed-out overview, rear, office control and both aisle walks. All preserve
paired state. The left route is (23,5),(23,4),(23,5),(23,4),(23,5); the right route
is (30,5),(30,4),(30,3),(30,4),(30,5). Both final comparisons and a moving sample
per route were reviewed. The initially held-key walking trial was superseded
by these explicit one-step routes. Display dimensions, source finish, complete
rear/edge surfaces, the shortened base and aisle clearance are accepted in
these final views. Temporary source inspection images and superseded trial
renders were removed.

Evidence: `target/kanto-resume/pokecom-native-display-final-review/`,
`pokecom-native-display-final-audit.json` and receipts
`pokecom-native-display-tests.log`, `pokecom-native-display-final-build.log`
and `pokecom-native-display-final-audit.log`. This closes the display assembly,
not the whole communications machine: the long console/body, its side/front
appliances and service-state presentation still need complete source-owned
geometry and native review. Other shared furniture, full native Hall animation,
continuous League traversal and all311/original16 visual/reachability acceptance
remain active. The previous full311 audit predates this change; this focused
report covers the only matched map. No packs, audio, external sources or
unrelated changes were modified, and no task-owned process remains live.


PokéCom's long communications console now has a closed native body instead of
horizontal projected art. Its original top deck stands at sixteen pixels, the
16x72-pixel electronics channel sits four pixels below it, and the two native
side columns form beveled cheeks down to the original ground footprint. A blue
shoulder bridges the folded front-course footprint. Both source rows of top
controls lie on the sloping front panel; both source rows of slots face forward
at the native southern boundary. No control rows were discarded. Native atlas
UVs preserve all of those drawings. The old projected shadow/apron ink is
replaced with blue floor at the unchanged walking datum, outside the console's
single compound cutaway group. Only the 192 original blocked body cells earn
`pokecom:native-communications-console` ownership; the 24 floor-apron cells do
not. The pack's twelve body blocks all have WALL permissions, while all three
apron blocks have FLOOR permissions.

The first narrow-body trial left too much empty space between the model and
its native collision edge. The final beveled casing fills that footprint and
puts the front panel at source row twenty-four. The first source-art inspection
also needed the PNG's VRAM-bank offset: both casing pigment tests now resolve
bank-one source tiles before checking the gray sample and blue palette. Runtime
paint already uses the bank-aware production atlas. The display plinth rises
from eight to sixteen pixels to meet the deck; its screen keeps its native
80x48 aspect, now from height sixteen to sixty-four. Its shallow depth and
room-side floor boundary remain unchanged. These assembled-console captures
supersede the earlier standalone-display height receipts.

All seventeen PokéCom checks pass, including the complete 216-cell drawing and
source mutation rejection, custom/donor/existing-owner protection, the original
flat-source classification, exact native front UV phases, closed positive
volume and outward winding. They verify 512 pixels of forward slot area,
64*sqrt(320) of sloped control area, 1,152 pixels of recessed electronics,
6,016 pixels of upper deck/shoulder and 13,824 pixels of opaque zero-height
floor. Display and console retain separate complete cutaway groups and all
support values. Combined ownership is sixty display cells plus 192 console
cells. The final native build succeeds; the focused audit has zero mesher
errors and 743/4096 modeled cells.

Nine fresh matched noon comparisons were individually inspected: front,
overview, east, west, rear, office control and three walking routes. All preserve
paired state. The left walk traverses (23,5),(23,4),(23,5),(23,4),(23,5), and the
right walk traverses (30,5),(30,4),(30,3),(30,4),(30,5). The front route moves
(26,13) to (26,12), rejects another Up into the native console, and returns to
(26,13). All final comparisons and a moving sample per route were reviewed.
Deck/recess depth, native finish, complete beveled casing, front-panel placement,
display join and approach clearance are accepted in these views. This is scoped
machinery-form acceptance, not whole-room/service-state or character-rig approval.

Evidence: `target/kanto-resume/pokecom-native-console-final-review/`,
`pokecom-native-console-final-audit.json` and receipts
`pokecom-native-console-tests.log`, `pokecom-native-console-final-build.log`
and `pokecom-native-console-final-audit.log`. Temporary bank/source inspection
images and the superseded narrow-console renders were removed. Remaining
PokéCom work includes the left communications kiosks/equipment, lobby planting
and other furniture/configurations, service-state visibility and full room
composition. Broader Kanto/shared furniture, full native Hall animation,
continuous League traversal and all311/original16 visual/reachability acceptance
remain open. The full311 audit predates this focused change. No packs, audio,
external sources or unrelated changes were modified. No task-owned process
remains live.


### Native floor triangle alignment (2026-10-16)

The native floor finisher now advances unmatched source geometry one triangle
at a time. Imported models can contain an odd triangle count; the previous
six-index stride then skipped valid floor quads, visibly leaving Red’s house
with a small modeled wood patch amid flat stripes. A regression inserts odd
triangles before, between and after floor cells, checks excluded cells, and
preserves their attributes, fade ownership and footing. It failed with two
finished cells instead of four before the fix. The floor-filter suite passes
233 checks with two ignored, and the native examples build successfully.

Fourteen matched production comparisons were individually inspected: six
Kanto/Johto homes, six floor-family controls and two Red-house walking routes.
All fourteen retain paired state. The two movement traces reach their expected
four-step paths and return to their starts; one moving sample per route was
also inspected. Red’s downstairs floor now finishes consistently. These are
floor checks, not whole-room acceptance: café chairs/poster duplication,
Cerulean’s black outer wall shapes, and remaining house stair backing still
need review. The current full audit covers 311 maps, 225 declared variants
with identities unchanged from the earlier bank audit, and 298 jump samples,
with no mesher errors. Declared combinations/prefixes do not prove all
reachable states. Receipts and images are ignored under
`target/kanto-resume/`, including `full311-floor-triangle-parity-audit.json`.


### Goldenrod ticket boxes and player-home wall returns (2026-10-16)

Goldenrod’s blue cases now reuse the station’s native ticket-box body at its
20-pixel height, elongated to the authored 24-pixel store length. They retain
flat blue sides, recessed slots and native pale metal paint. The complete
existing blue-case matchers retain their source-cell ownership and floor
backing; pink seating and sixth-floor drink machines keep their own models.
This correction is scoped to Goldenrod’s department-store floors.

PlayersHouse1F side returns now authenticate the complete four-by-two upper
plank drawing in block $05, then reuse a two-column section at the original
texel scale. The resulting 32-pixel walls match the rear plank course instead
of repeating the projecting divider or kitchen trim. The complete source guard
rejects a damaged drawing before emitting the template. Room bounds, door gaps,
source footing and the separate divider remain intact.

The native build and seven-map audit pass with no mesher errors. Five final
paired comparisons were individually inspected: the house from front/east/west
and the blue cases from front/west. First- and sixth-floor controls add two
inspected comparisons for actual seating and drink machines. All seven preserve
paired state. Final images are in the ignored
`target/kanto-resume/goldenrod-ticket-player-wall-accepted-review/`; the two
controls remain in `goldenrod-ticket-player-wall-final-review/`. The current
scoped audit is `goldenrod-ticket-player-wall-accepted-audit.json`.

Concurrent source edits invalidated intermediate builds, so final verification
used a temporary snapshot of the current source and the shared external pack.
The three changed renderer source files were byte-identical to the working
checkout. Local optimization overrides were used for verification; this is not
a release-client certification. Broader Kanto/shared-furniture work, Hall
animation, continuous League traversal and full311/original16 visual and
reachable-state acceptance remain open.

The final focused checks pass: the authenticated wall-donor regression, the
actual native retail-fixture regression (both blue-case layouts), and all 26
department-store controls, for 28 passing checks. The wall regression also
requires native rear-plank UVs and rejects every mutated cell of its complete
source guard. Tests reused the native feature graph in the temporary snapshot;
the unrelated Bevy harness matched zero tests and is not gameplay certification.
Receipts are `player-house-authenticated-rear-wall-snapshot-tests.log`,
`goldenrod-ticket-box-accepted-tests.log` and
`department-store-ticket-accepted-tests.log`. The snapshot and superseded trial
captures were removed after verification; no packs or audio were changed.


### Café floor motifs and single Celadon poster (2026-10-16)

Celadon’s wall poster no longer repeats down the ceiling-height wall. Its
complete block $28 drawing authenticates the timber half and four poster tiles;
only timber repeats over the wall height. The original sixteen-pixel poster
appears once on the upper course, over a closed wall backing. A regression
requires exactly 256 square pixels of poster art above height sixteen. The
shared domestic-wall helper retains its original behavior for other drawings.

Celadon and Olivine now retain the native four-quadrant café floor pattern,
including its inset rectangular border and two opposing gray panels. The
former two-color generic ceramic erased these motifs in Celadon; Olivine mixed
wood patches with the raw source pattern. The new finish uses live source UV
samples and exact source phases. A complete 16x16 tile has 52 frame pixels,
50 gray-panel pixels and 154 field pixels. Tests verify all quarters, three
world origins and three scales, plus unmodified footing/claims, reservation
exclusion and rejection of unproven source quadrants. The casino’s existing
ArcadeChecker finish remains separate.

The native build passes. Twelve café checks and two shared wall controls pass
(14 total). The two-room audit has no mesher errors; existing object ownership
and footing remain identical to the before audit: Celadon 204 claimed cells,
Olivine eight. These counts are not room-completion percentages. Five paired
comparisons were individually inspected: both rooms, a Celadon close view,
Olivine from the west and a Celadon walking view. All preserve paired state.
One moving frame was also inspected. Ordinary RRLL input follows
Celadon (4,5)->(5,5)->(6,5)->(5,5)->(4,5), with the original walking height.
Existing Celadon stools have modeled seats/legs and seated actors; close
inspection corrected the earlier impression that they were flat floor cards.

Final evidence is ignored under `target/kanto-resume/`:
`cafe-native-fidelity-final-review/`, `cafe-native-fidelity-final-audit.json`,
`receipts/cafe-native-fidelity-build.log`,
`receipts/cafe-native-fidelity-final-tests.log`,
`receipts/cafe-house-wall-control.log` and `receipts/cafe-room-wall-controls.log`.
Verification used a temporary stable source snapshot because other goals were
editing game code concurrently; both changed renderer files matched the live
checkout byte for byte. The snapshot was removed after verification.

This closes the shared floor motifs and Celadon poster, not whole-café
acceptance. The fresh Olivine front/west views show a planar table/plant drawing
and a rear wall still drawn as a horizontal strip, with plain side returns.
Those fixtures and connected ceiling-height rear architecture are next.
Celadon display-cabinet identity/materials and remaining whole-room composition
still need review. Other active findings include Cerulean outer wall shapes,
house stair backing, broader shared furniture, native Hall animation,
continuous League traversal and full311/original16 native visual and reachable
state acceptance. No new external blocker was found.


### Olivine Café furniture and connected timber architecture (2026-10-16)

Olivine now resolves its complete sixteen-cell pink table drawing and both
eight-cell plant drawings into modeled furniture with native source paint.
The plain table has no Celadon basket. Six complete gray stool drawings now
use the shared rectangular seat/leg geometry; both seated sailors use the
five-pixel seat height. The full table/plant source guards reject every changed
cell and preserve original floor support and custom-document priority.

The rear timber course is now upright at ceiling height and its two-column
perimeter donor retains native texel density. The service counter uses an
exact 48-cell source fingerprint and a connected green deck/wooden body,
claiming 26 cells while leaving the staff bay clear. Tests mutate every source
cell, including the unclaimed halo, and verify reservation exclusion and
unchanged footing. Café furniture backing uses the authored four-quadrant
floor phase. Wall source rows no longer create false floor outside the folded
rear wall, and the counter cap begins below the rear timber course.

Rear review also caught a false floor strip behind Celadon's north display
cabinet. A complete twelve-cell display guard now removes only the two upper
elevation rows when the cabinet is on the map's rear course. Its indoor foot
row remains; the interior cabinet retains all backing. The regression checks
both cases, map isolation, all source mutations and unchanged support.
The corrected rear comparison was inspected again and has no outside strip.

The final native build passes, as do 14 café checks and the pack-backed wall
donor regression. The existing shared house-wall control also passes. The
final two-map audit has zero mesher errors and unchanged zero-height support:
Celadon retains 204 claimed cells; Olivine now has 112 (30 timber, 16 table,
16 plant, 24 stool and 26 counter cells). These counts describe ownership,
not whole-room acceptance percentages.

Nine final paired comparisons were individually inspected: both fronts,
Olivine east/west/rear/detail, corrected Celadon rear, Olivine walking and the
exit arrival. Every pair preserves state. An interior moving sample and the
exit approach were also inspected. Ordinary DDUU input follows
OlivineCafe (4,4)->(4,5)->(4,6)->(4,5)->(4,4). Ordinary DD from (3,6)
passes (3,7), warps to OlivineCity (7,21), and completes the authored arrival
step at (7,22), with the city terrain active and settled.

The first exit capture overflowed a Compute Task Pool stack on the transition
to the outdoor map in this local build with renderer/game package optimization
set to zero. Repeating with RUST_MIN_STACK=16777216 completed. This is a
verification limitation requiring normal-profile/default-stack follow-up;
it is not evidence that the default-stack client transition passes.

Evidence remains ignored under target/kanto-resume/: olivine-cafe-forms-review/
(the superseded uncorrected Celadon rear is not final),
olivine-cafe-final-audit.json, receipts/olivine-cafe-final-tests.log,
receipts/olivine-cafe-final-wall-controls.log,
receipts/olivine-cafe-house-wall-control.log and
receipts/olivine-cafe-final-build.log. All six changed renderer files matched
the stable verification snapshot byte for byte; the temporary snapshot was
removed after verification. No pack or audio was changed.

This closes Olivine's previously flat furniture and rear architecture at the
reviewed poses. Whole-café composition and Celadon display identity/materials
remain open, along with the default-stack exit follow-up, Cerulean outer wall
shapes, house stair backing, broader Kanto/shared furniture, native Hall
animation, continuous League traversal and full311/original16 native visual
and reachable-state acceptance. No external blocker prevents further work.


### Celadon Café source-facing cabinets; default-stack exit verified (2026-10-16)

The two Celadon café cabinets no longer use the generic colorful prize-package
case. Their complete block $04 drawing describes an eight-pixel horizontal
grille and a sixteen-pixel front elevation. A source-painted, closed cabinet
now places those tiles on the top and front respectively, at 16-pixel height
and 8-pixel depth, with native gray side/rear paint. The café-only source guard
leaves the casino's shared prize-case model unchanged. Source ownership and
walking support remain unchanged. A complete native timber donor also closes
the ceiling-height rear wall behind the northern cabinet. Interior cabinets
do not create an extra rear wall, and the previous false-floor fix remains.

The first render caught inward-wound front overlays that were culled. Their
winding was corrected, an outward-normal triangle assertion was added, and
all affected final views were regenerated and individually inspected. Tests
also require all twelve source tiles on the correct horizontal/vertical face,
complete-source rejection, correct dimensions, the rear-only floor omission,
unchanged support and source-authenticated rear timber backing.

Final verification uses the repository's normal wasm-dev profile (inherited
opt-level 2), with none of the earlier package opt-level=0 overrides. Both native
examples build. Fourteen café checks and the external room-wall donor check
pass. The final four-map audit has zero mesher errors: CeladonCafe 204 claimed
cells, OlivineCafe 112, CeladonGameCorner 542 and GoldenrodGameCorner 494, with
unchanged support: café heights remain zero; casino ranges remain 0..8 pixels.
Casino counts remain controls, not
whole-room visual acceptance.

Six final paired comparisons preserve state and were individually inspected:
Celadon front/detail/east/rear, Olivine front control and the Olivine exit
arrival. The exit approach moving sample was also inspected. With RUST_MIN_STACK
explicitly absent, ordinary DD follows OlivineCafe (3,6)->(3,7), warps to
OlivineCity (7,21), and finishes the authored step at (7,22). City terrain is
active and settled. This closes the previous normal-profile/default-stack
exit follow-up. It does not make the earlier zero-optimization failure pass.

Final ignored evidence is under target/kanto-resume/:
celadon-native-display-final-review/, celadon-native-display-final-audit.json,
receipts/interior-default-profile-build.log,
receipts/celadon-native-display-final-audit-build.log,
receipts/celadon-native-display-final-tests.log and
receipts/celadon-native-display-final-wall-tests.log. Both changed renderer
files matched the verification snapshot; it and superseded trial images were
removed afterward. No pack, audio, model catalog or unrelated source changed
as part of this correction.

A fresh normal-profile CeruleanGym paired comparison still shows black outer
walls. Its north block $2b begins with blank black tile $0f rows; the current
fallback samples that north-row paint. The visible gray course needs its own
source-authenticated wall treatment rather than this blank-row donor. The
fresh state-preserving comparison is cerulean-native-wall-next-review/front-compare.png.
This, house stair backing, remaining Kanto/shared furniture and whole-room
reviews, native Hall animation, continuous League traversal and full311/original16
native visual and reachable-state acceptance remain open. The broader goal
is not complete, and no external blocker prevents further implementation.


### Cerulean Gym gray enclosure and clear walkable rim (2026-10-16)

Cerulean's black side walls came from the fallback sampling blank tile $0f
in the first north row. The native enclosure now samples the authenticated
gray tile $0a instead. It repeats the original tile at native texel density
over closed, ceiling-height walls with finished caps and both front/back
faces. The northern wall stands at the inner edge of the two blank source
rows. The gray rim is FLOOR in the pack's collision data and remains intact;
side walls stand outside it. The southern wall retains the two-tile entrance.
All four sides support camera cutaway, including their caps and backs.

The new cerulean_room.rs binding is limited to CeruleanGym/port, the original
10x16 core-tile bounds and both authored exit warps. It validates every visible
boundary source cell, including its block, tile and subtile phase, and refuses
unproven or custom-owned gray/blank boundary drawings. Interior bridges and
pool cells are not part of the boundary guard. Walls do not claim source cells,
change support heights or dispatch game input.

The pack-backed wall regression passes with the final normal-profile build.
It independently mutates all 256 boundary cells, rejects another map and an
altered warp list, checks 32-pixel height, outward winding, complete cutaway
registration, north placement, the open exit and unchanged support/ownership.
The first attempted test compilation used pop on the immutable warp list;
the fixture now replaces that list, and the final regression passes. The Bevy
harness matched zero tests and is not gameplay certification.

Both native examples build under the normal wasm-dev profile without package
optimization overrides or a larger stack. The final four-map audit has no
mesher errors. Family ownership, modeled-cell totals, footing counts and
footing extrema match the before audit exactly for CeruleanGym (176 cells),
PewterGym (200), VermilionPort (728) and CeladonCafe (204). Counts describe
existing source ownership, not completion percentages; the enclosure adds
presentation geometry without new furniture claims.

Nine final paired comparisons preserve state and were individually inspected:
Cerulean front/east/west/rear/overview, two wide rim walks, the wide exit
arrival and a Celadon café control. One moving sample from each final walk
was inspected. Initial side-camera walking views cropped the player at the
bottom; wider front views supersede those trials and show the player fully
on the clear gray rim. Ordinary UUDD follows (0,5)->(0,4)->(0,3)->(0,4)->(0,5)
and the same y sequence at x=9. These are isolated walkable-position fixtures,
not a claim of story access to the rim. Ordinary DD from (4,14) passes (4,15),
warps to CeruleanCity (30,23) and completes the authored step at (30,24).
All three final traces settle with the correct active terrain and no pending
terrain build. The wide exit approach also shows the complete player and a
clear doorway.

Ignored evidence is under target/kanto-resume/: cerulean-native-wall-review/,
cerulean-native-wall-before-audit.json, cerulean-native-wall-final-audit.json,
receipts/cerulean-native-wall-build.log and
receipts/cerulean-native-wall-final-tests.log. Both changed renderer files
matched the stable verification snapshot, which was removed afterward, along
with superseded camera trials. Obsolete executables were removed only from
this task's own cache; current binaries and other goals' caches were retained.
No packs, audio or unrelated source were changed by this correction.

The wall correction is verified at these poses. Whole-Cerulean acceptance is
still open: east/west and entrance views expose two raw brick rectangles in
the floor backing beneath the entrance statues, contrasting with the finished
pink deck. Correct those next, then continue house stair backing, remaining
Kanto/shared furniture and whole-room reviews, native Hall animation,
continuous League traversal and full311/original16 native visual and
reachable-state acceptance. There is no external blocker; the full goal
remains active.


### Cerulean entrance-statue floor now shares the deck (2026-10-16)

The two orange/raw brick patches beneath Cerulean's entrance statues now use
the same modeled deck builder as the adjacent pink walkway. The previous
backing repeated an unprocessed atlas sample. The shared builder retains its
existing brick courses, grout, palette, three-pixel thickness below zero and
zero-height upper surface; its existing bridge caller is unchanged in behavior.
Only complete native Cerulean guardian drawings receive the new backing.

The new guard requires NativeGymGuardian form, the complete two-by-four
port/$32 drawing and all eight exact tile/subtile identities. The regression
compares the statue backing with the shared adjacent-deck geometry, requires
the statue body to remain, mutates all five source-identity fields in every
owned cell, checks another port map, preserves support/ownership and confirms
that no backing vertex rises above the original surface.

Both native examples build with the normal wasm-dev profile and default
stack. All eight Cerulean checks pass, covering the new integration as well
as existing bridge/deck, water, gray-rim, palette and atlas-isolation controls.
A direct focused invocation with manually selected dependency features began
building a different dependency graph; that task-owned attempt was stopped.
Final tests used the established crystal-voxel-view + crystal-bevy/location-tester
graph. The Bevy harness matched zero tests and is not gameplay certification.

The final four-map audit has no mesher errors. Existing families, source
ownership and footing extrema match the previous audit: Cerulean 176 modeled
cells, Pewter 200, VermilionPort 728 and CeladonCafe 204. The three controls'
solid triangle counts are unchanged. Cerulean's added modeled backing is the
only solid geometry increase in that comparison; its authored source claims
and walking heights remain unchanged. No packs or audio were modified.

Five final Cerulean paired comparisons were individually inspected: entrance,
east, west, rear and walking. Both former patches now join the pink deck
without a material boundary, while the gray rim, statues, walls and entrance
remain intact. Every pair preserves state. One moving sample was inspected.
Ordinary UD follows (4,13)->(4,12)->(4,13), with the correct terrain settled.
These are scoped floor checks, not whole-room or story-state acceptance.

A sixth inspected/state-preserving pair freshly checks RedsHouse1F at (5,4)
from the east. It confirms missing rear wall backing at the rising stair
flight. Close that stair recess and its joins while retaining the approach and
warp; verify the actual up/down route. Do not place a wall across the flight.
This fresh source/render evidence replaces reliance on older house images.

Ignored evidence is under target/kanto-resume/: cerulean-statue-floor-review/,
cerulean-statue-floor-audit.json, receipts/cerulean-statue-floor-build.log and
receipts/cerulean-statue-floor-final-tests.log. Both changed renderer files
matched the stable verification snapshot, which was removed afterward.
Obsolete test executables were removed only from this task's cache. Unrelated
working-tree changes remain intact.

Next is the rear stair backing, followed by remaining Kanto/shared furniture
and whole-room reviews, native Hall animation, continuous League traversal
and full311/original16 native visual and reachable-state acceptance. The full
goal remains active, with no external blocker.


The later pier correction supersedes the earlier flat Cerulean-deck assumption.
Cerulean's pink walkways and the corresponding outdoor brick piers now have a
four-source-pixel deck height, continuous shallow edge beams and pilings into
the water. Decks remain level through terrace fitting. Native brick joints and
palettes are retained. Cerulean's gray rim and north shoreline timber remain at
ground level; its guardian feet share the raised deck. Actor and sign support
uses the same height field as the geometry.

Shore approaches spread the rise over four source cells (32 native pixels).
A shared vertex graph carries that grade around L-shaped turns, including the
Route 12 corner near the Super Rod house. The approach to Bill's raised paving
samples the actual terrace height instead of introducing a step. Fascia follows
the sloped edge. Route 12's flower patches are closed soil islands with native
3D blooms and continuous grass color, including animation refresh. Shore plants
retain their roots at the level water surface; lower shoreline backing closes
the gap beneath existing banks.

Verification used the ignored completed browser and TUI packs, the normal
optimized wasm-dev profile, default thread stack, and this task's Cargo cache.
The native render/audit build passed. The five elevated_pier regressions pass,
including the external 11-map corner/join/support/source-guard test and the
raised-paving, native-brick, sign and rooted-water checks. Additional direct
viewer test filters pass: cerulean (7), port::tests (4), mesh::terraces::tests
(21), mesh::kanto_paving::tests (11), mesh::outdoor_signs::tests (7),
mesh::shore_ecology::tests (3), and the persistent-flower refresh check (1).
These filter counts overlap the five focused checks. The Bevy harness selected
zero tests for elevated_pier; it is not gameplay certification.

The fresh 14-map audit has no mesher errors. Unaffected controls match the prior
statue-floor audit in family ownership, modeled-cell counts and zero footing:
PewterGym 200, VermilionPort 728, CeladonCafe 204. Native/2D comparisons cover
CeruleanGym, CeruleanCity/Route24, Routes 12, 13, 20, 21, 25, 26, 27 and 32,
including both Cerulean side views, the entry ramp, soil/flower closeups, the
Route 12 bend and shore, Bill's approach, and Johto pier approaches. All 21
paired captures preserve state. Deck/ramp/soil/support changes were inspected;
this is not acceptance of the entire surrounding landscape. Route 24's low
side view hides the player and bridge behind foreground terrain. A separate
60-degree overhead view shows the intact bridge and player, but the occluded
low view remains an open camera/terrain visibility defect.

Five ordinary seeded walking fixtures settle on the authoritative map with
terrain active, no pending replacement and no warp phase: CeruleanGym DD
(4,14) to CeruleanCity (30,24); CeruleanCity UUUU (21,1) to Route24 (9,15);
Route12 DDUU (5,34) around the bend; Route12 DDUU (9,1) across the shore; and
Route32 UUDD (13,60) across its approach. These are scoped walking fixtures,
not proof of story reachability or full-room acceptance.

Ignored evidence: target/kanto-resume/piers-smooth-review/ (21 paired views,
five movement CSVs), piers-smooth-audit.json, and receipts/piers-smooth-build.log,
piers-smooth-tests.log, piers-smooth-controls-tests.log and the per-view logs.
All changed pier renderer files matched the verified snapshot before cleanup.
A concurrent outdoor-mountain change differs from that frozen snapshot and
has its own review scope. The obsolete pier review directories and temporary
verification source were removed after every job completed. Packs and generated
content remain ignored and were not rebuilt, staged or committed.

The broader goal remains active: resolve the Route 24 low-angle visibility,
house stair backing and remaining Kanto/shared furniture and whole-room work,
native Hall animation, continuous League traversal, and the original full
variant/reachability acceptance. This scoped pier correction does not close
those requirements.


The follow-up request for slower ramps extends outdoor pier shore approaches
from four to eight source cells (32 to 64 native pixels, or four walking
tiles). The shared deck graph and player support use the same longer grade,
including bends and joins to raised native paving. Deck elevation, shoreline
contacts and the Cerulean Gym entrance remain unchanged. The existing synthetic
raised-paving regression now spans the full longer approach and checks its
halfway height, end height, shared vertices and matching actor support.

The normal optimized native build passes. Fresh same-state native/2D views
of Bill's raised landing, the Route 12 bend and Route 32 shore were inspected;
all three preserve paired state. Route 12 DDUU and Route 32 UUDD complete and
return to their starting positions with matching authoritative/terrain maps,
active terrain, no pending terrain and no warp phase. A Route 32 moving frame
also confirms support on the slope. All five focused pier regressions pass,
including the external eleven-map source/join/footing check and the expanded
full-length raised-paving regression. The Bevy harness selected zero tests;
the two native movement fixtures provide the scoped walking evidence. The
ramp changes match the verification snapshot and pass rustfmt checking. A
concurrent water_under_deck helper was added afterward to the live file; it is
preserved and belongs to the separate water-surface verification scope.
Evidence is ignored under target/kanto-resume/piers-gentler-review/ and
receipts/piers-gentler-{build,tests,raised-paving,corner,shore}.log. This verifies
the requested ramp adjustment; the broader interior review remains active.


House stair closure is now verified. Exact complete native stair drawings in
both floors of Red's, Copycat's and the player home receive a rear wall and
side returns outside the treads, with an open south approach. The returns meet
the perimeter's full two-source-cell opening. The two Johto homes retain their
whole-course source/profile guard. Red's first-floor block0a and Copycat's
block23 each had four unclaimed plank wall cells beside the ascent; their exact
bindings now close the black strip with the existing ceiling-height wall model.
Copycat's upstairs east boundary excludes the verified four-cell black padding.
No stair tread, source opening or authoritative collision was changed.

Final normal-profile native build and both home_stair regressions pass.
Controls pass: expanded native house wall fragments (1), external wall donors
and Copycat padding (1), enhanced room floors (12), and map-aware QA step
release (1). All 22 fresh comparisons in home-stair-shoulder-review were
individually inspected, including east/west views of all six floors, all six
walking flights and four close/front views. Each preserves paired state. Six
UU fixtures cross once to the other floor and once back, return to their start,
and finish with matching terrain/runtime maps, terrain active, no pending
replacement and no warp phase. These seeded fixtures prove the stair paths;
they do not certify earned story access or whole-room/model fidelity. Player
bedroom exterior shadow artifacts and Copycat decoration identity remain open.

The full311/225-state final audit has no mesher errors. Relative to the initial
house-recess audit, only RedsHouse1F and CopycatsHouse1F change family ownership:
four additional domestic-wall cells each, modeled-cell count 76 to 80. All
map footing counts and extrema remain unchanged; all 225 variant records,
including the original 16 Kanto structural alternatives, are identical.
The full311 native base capture set completed without failures; six house
entries were refreshed after the shoulder correction. Thirteen dark fixtures
use disposable Flash. These images still need family and spatial coverage
review. The earlier 41-map LCD coverage estimate compared walking dimensions
to visual dimensions. Correct units are a 10x9 walking-tile LCD footprint;
177 of the 311 maps exceed it. This correction does not itself prove which
areas were visible in the wider paired render.

Ignored evidence: target/kanto-resume/home-stair-shoulder-review/ (22 pairs),
home-stair-shoulder-full311-audit.json, full311-native-current-review/,
receipts/home-stair-shoulder-{build,tests,controls-tests}.log and per-view logs.
Verified house source files match the stable snapshot. Packs and audio remain
unchanged. Next: Viridian maze-wall quality, remaining whole-room/shared
furniture review, native Hall animation, continuous earned League traversal,
and the full original variant/reachability acceptance. The goal remains active.


The current full311 base capture review now additionally includes 24 room
pairs: TinTower1F, SproutTower3F, RadioTower1F, OlivineLighthouse6F, SaffronGym,
FuchsiaGym, VermilionGym, CeladonGym, BrunosRoom, KarensRoom, KogasRoom,
LancesRoom, WillsRoom, HallOfFame, TeamRocketBaseB1F, GoldenrodDeptStore2F,
CeladonCafe, CeladonDeptStore6F, GoldenrodDeptStore6F, OlivineCafe,
EarlsPokemonAcademy, KurtsHouse, ElmsLab and OaksLab. The previous two base
reviews were PewterGym and ViridianGym. This is a record of inspected views,
not whole-map/variant acceptance. Tall rooms and the Rocket base have clipped
or occluded areas that require other positions/angles. The League base views
do not establish ceremony animation, earned traversal or gate-state fidelity.

Fresh defects from this review: SproutTower3F has detached outer wall runs
above the side lattice; investigate perimeter placement against the actual
source void/room outline. Earl's Academy repeats the rear blackboard on side
walls; the perimeter must continue the plain striped wall instead. Player
bedroom exterior shadows remain visible in side views. Copycat's doll/model
identity remains under review. Viridian's cap construction is being improved
in the existing canonical kit; its native review is still pending. Radio's
glass placement needs a side-angle check, rather than inferring absence from
the transparent frontal view. The cafes, classroom, labs and department-store
furniture were compared for scale/orientation, but their shown base poses do
not substitute for all service/state and off-camera checks.


Verified classroom, tower perimeter and Viridian coping follow-up:

- Earl's Academy side-wall donor authenticates the complete native panel, then
  uses only its plain striped tile at full ceiling height. The original rear
  blackboard, desks, chairs and lectern retain their ownership and footprint.
  Three inspected front/east/west pairs preserve state; the classroom regression
  passes.
- Sprout's exact source padding/lattice guards place the walls at the room
  edge; internal ladders retain backing. Sprout 1–3 and Tin 1–9 now prioritize
  the source-complete rear-facing TowerTimber donor. Reusing a side-facing donor
  had rotated/stretched the template and left visible joins. The expanded
  external_room_wall_donors regression passes, including exact Sprout 3F bounds
  x61..259. All 40 final tower pairs were inspected and preserve state. Both
  stair fixtures, the UUDD column-area route and DD ground-floor exit settle;
  no script flags or progression bypasses were used. Pillar animation controls
  remain passing. Upper Tin north grating/void edges still require full-room
  source interpretation and coverage review.
- The existing 16 Viridian maze assets and authored Blender source now share
  continuous beveled coping with subtle stone variation. Two earlier native
  candidates were rejected. Four final pairs preserve state; UUUUDDDD returns
  to the start and settles. All 38 gym tests pass, including 144 neighboring
  joins/1440 contained interface triangles. The 20-model preflight validates
  364 closed parts and 6732 triangles. The four unrelated authored Blender
  collections and their shipped JSONs were preserved. Cap finish acceptance
  does not establish whole-room massing or all gameplay states.
- Eight Radio 1F/2F/5F side/entrance pairs were inspected and preserve state.
  Radio 2F RRRLLL and 5F DDUU return and settle with entrances clear. Radio 2F's
  flat west foliage strip needs source inspection across blocks 0x30/0x1f;
  complete plant tests do not cover that cropped composition.

Receipts live under ignored target/kanto-resume/: tower-rear-donor-tests.log
and tower-rear-donor-controls-tests.log in receipts/, tower-rear-donor-review/,
maze-stone-review/, maze-school-sprout-review/ (classroom views), and
radio-side-current-review/. The final tower-rear-donor-full311-audit.json is
identical to maze-stone-full311-audit.json: 311 verified maps, 225 script-state
variants, zero mesher errors, unchanged ownership and footing. Fourteen current
front gallery entries were refreshed with provenance in
full311-native-current-review/refreshed-school-maze-towers.json. Seeded capture
coverage remains distinct from earned traversal. Full original review scope,
Hall ceremony, League progression and service/state requirements remain open.


Next native review pass inspected BlackthornGym1F, BlackthornGym2F,
DragonsDenB1F and DanceTheater in the current full311 gallery. Blackthorn's
first-floor stair profile needs a close side view; its second-floor cream
maze cap/massing needs a source-material review. Dragon's Den's wide cave,
water and shrine composition needs additional poses beyond the current crop.
Dance Theater's raised stage and tatami composition are visible, but this
base view does not prove stage traversal, services or all room angles.

Radio 2F west foliage investigation confirmed a single tall topiary drawing:
block 0x30 supplies four upper leaf rows and adjacent block 0x1f supplies the
lower crown and sole pot. A guarded two-by-eight-cell resolver and selected
model crown stacking are implemented; production build and native review are
pending. No acceptance is claimed yet. The build is tracked in
receipts/radio-topiary-build.log under target/kanto-resume/.

The next four inspected base pairs were GoldenrodUnderground, GoldenrodBikeShop,
GoldenrodNameRater and GoldenrodFlowerShop. Flower Shop has genuine rear-wall
holes above the flower stands/window fragments; backing behind those fixtures
must be restored. Bike Shop's entry has a raised gray stair/black underside
that needs source-floor comparison at a low angle. Underground's center void
and long corridor require more positions; the start-room crop does not prove
coverage. Name Rater furniture scale/orientation is visible, with services and
side/rear coverage still outstanding. These findings are not acceptance.

Also inspected CherrygroveMart, VioletPokecenter1F, AzaleaPokecenter1F and
MahoganyMart1F base pairs. Mart/Pokecenter floor simplification loses much of
the source pattern; assess material fidelity rather than treating geometry
checks as floor acceptance. PokeCenter's front-left stairs need the same close
profile review as Blackthorn. Mahogany's authored shelves, tatami, clock and
rear cabinetry are visible; its service/secret-entrance states remain unproven
by this base pose. No whole-room acceptance follows from these inspections.

FightingDojo and ViridianNicknameSpeechHouse base comparisons were inspected.
Dojo has visible rear-course holes behind its two northern statues; apply the
required continuous statue backing there next. Viridian's room scale and table
seating are visible; actor/species models belong to the separate active model
work and this capture does not prove dialogue/services.

MrPsychicsHouse and SafariZoneWardensHome were then inspected at their actual
map names. Warden's home has a substantial missing rear wall above the two
notice/poster fields, and its central ornament needs source identity/shape
review. Mr Psychic's domestic composition is visible, but neither this pose
nor the other domestic base captures establish item-gift/service behavior.
Sixteen additional base room pairs have thus been inspected in this pass;
the newly found gaps and material defects remain explicit work items.

Topiary native review caught and corrected an unpadded-frame assumption;
production can add surrounding cells to a short map. The new regression
explicitly shifts the complete drawing into a padded frame. The resulting
three palm tiers were visibly too sparse and were rejected. The current
candidate instead combines one selected pot with three overlapping canonical
rounded hedge volumes, sampled from native foliage colors. A module-path
compile error was corrected before restarting the dense candidate build.
Goldenrod Flower Shop's missing plain house block 0x32 wall fragment was also
bound to the existing domestic wall builder, with the window-role regression
extended. Both final changes remain under verification, tracked by
receipts/radio-topiary-dense-final-build.log. Earlier rejected captures and
initial audit are diagnostic evidence, not final acceptance.


Final topiary/Flower Shop native verification:

- Source-bound Radio topiary detection survives padded/cropped camera frames
  containing its complete two-block drawing. One pot supports three overlapping
  existing rounded hedge volumes; upper animation slots are not used as paint
  donors. The canonical hedge and plant assets were reused without new exports.
- Both complete four-cell halves of house block 0x32 now select their existing
  wall/window builders. The native Flower Shop rear course is continuous at
  ceiling height, with windows near the ceiling and displays kept in front.
- receipts/radio-topiary-finished-build.log confirms the normal optimized native
  build. All eight paired images in radio-topiary-finished-review/ were inspected,
  preserve state and exit successfully. Both moving-01 frames were also viewed.
  Radio UUDD follows (2,5)→(2,4)→(2,3)→(2,4)→(2,5); Flower Shop follows
  (4,3)→(4,2)→(4,1)→(4,2)→(4,3). Both end with matching runtime/terrain maps,
  active terrain, no pending build, zero fade and no warp phase.
- radio-topiary-finished-full311-audit.json verifies 311 maps, 225 variants and
  an empty mesher_errors list. Compared to tower-rear-donor-full311-audit.json,
  only Flower Shop changes modeled coverage 52→68 and Radio 2F 200→208.
  All footing fields and all 225 script variant records match. The first local
  comparison script incorrectly expected numeric zero instead of an empty error
  list; its corrected completed comparison confirms these results.
- The source-bound regression command is still compiling under the same
  full-feature snapshot. Track the existing test session and
  receipts/radio-topiary-finished-tests.log; do not restart solely for a timeout.
  The runner then executes the topiary one-pot/bounds/cache regression, domestic
  window roles, perimeter donors, home stairs, floor and dense-plant controls.
  Do not call those tests passed until their terminal results are inspected.
  Refresh the two full311 base entries only after that final check passes.

This is scoped wall/plant verification. Remaining Radio equipment and Flower
Shop display volume/material fidelity still require whole-room review, alongside
newly identified Dojo/Warden backing, stair/floor issues, Hall animation,
continuous earned League traversal and the full original service/state scope.


Radio/Flower Shop follow-up is now closed for its scoped wall/plant changes:
29 focused checks pass (11 native_radio, one topiary, one window role, one
perimeter donor, two house stairs, twelve floor and one dense-plant control).
The first test link failed with LLVM "No space left on device" and left a
1440-byte non-executable output; it did not run tests. After removing obsolete
voxel/Bevy/assets variants only from target/kanto-resume, that corrupt output
was removed and the same regression command rebuilt and passed. The final
receipt is receipts/radio-topiary-finished-tests.log. Radio 2F and Flower Shop
base gallery entries are refreshed, with provenance in
full311-native-current-review/refreshed-radio-flower.json. This does not close
whole-room fidelity, service/state or the broader original goal.

Next implementation under verification: Dojo source rows fold into a thin
3px, ceiling-height 32px brick course, with source-authenticated backing behind
both northern guardians and both central notices. Interior guardians remain
free-standing. Notice art is mounted once at native size near the ceiling.
Domestic house block 0x29 receives a complete guarded wall course with its
single 16x8 notice above plain wall; Warden's home and Pokémon Fan Club share
that actual map block data. New regressions cover source mutations, folded
bounds, single notice ownership, custom ownership and unchanged footing.
The snapshot build is tracked in receipts/dojo-notice-build.log. Native review,
full audit and final focused regression results remain pending for this batch.


Dojo/notice native and audit results (regression gate still pending):
The optimized build completed successfully. All 17 pairs in dojo-notice-review/
were individually inspected, with state preserved and successful captures.
The completed original 16-case statuses are in batch-results.json; the extra
corrected Fan Club walk has its own final result. Dojo UUDD returns (5,6),
Warden UUDD returns (7,4), and Trainer RRLL returns (5,8). The initial Fan Club
UUDD at (7,4) met its NPC and ended (7,6), so it does not prove return traversal.
The corrected UUDD route at (8,4) reaches (8,2) and returns (8,4), with matching
runtime/terrain maps, active terrain, no pending build/fade/warp. No script
flags or progression bypasses were used.

dojo-notice-full311-audit.json verifies 311 maps, 225 variants and no mesher
errors. Only four map records differ from radio-topiary-finished-full311-audit:
Dojo converts its old generic plaque geometry to native textured surfaces
(solid triangles 792→0, source ownership unchanged); Pokémon Fan Club and
Warden each gain coverage 128→144, Trainer 134→150. All footing fields and
all 225 script variant records remain identical. The running source-bound
regression receipt is receipts/dojo-notice-tests.log. Do not claim those tests
passed or refresh these four base entries until their terminal result and the
new domestic notice/window/perimeter/floor controls have been inspected.

New separate source-bound defects: Trainer's rear PC course still lacks six
plain wall cells in house block 0x1c. DayCare shares that source omission and
was newly inspected in the full311 base gallery. Its room width includes a
four-cell east black-padding band, house block 0x0a/tile 0x10, which currently
leaves its east wall detached from the floor. These are next corrections,
not covered by the completed notice-course native checks. Whole-room ornament,
furniture and service/state acceptance remains unproven, as do Hall animation,
continuous earned League traversal and the original full coverage requirements.


Dojo/notice regression gate is now verified: both dojo tests pass, including
all guardian/donor mutation rejection and thin full-height backing. All thirty
related controls pass: domestic notice single-art/backing/custom ownership,
domestic window roles, native house windows, perimeter donors, twelve floor,
two house stair, eleven native Radio and one topiary test. Receipts are
receipts/dojo-notice-tests.log and receipts/dojo-notice-controls-tests.log.
All three changed source files match the compiled verification snapshot.
Four full311 base entries are refreshed, with provenance in
full311-native-current-review/refreshed-dojo-notices.json. This closes only the
scoped Dojo and shared notice wall corrections. The separately observed rear
PC gaps in Trainer/Day Care, Day Care's detached east boundary, furnishing and
ornament fidelity, wider room/state/service acceptance, Hall animation and
continuous earned League traversal remain active requirements.


Domestic header/boundary implementation follow-up:
House 0x1c now has a four-cell left shoulder and two-cell upper PC margin.
The one-row margin folds to the same 16px wall plane as its two-row neighbor,
using the native domestic wall at 32px ceiling height. House 0x1d/0x1e likewise
have two-cell plain margins above their appliances; these were missing across
53 houses in the full311 source inventory. Complete source identity and custom
ownership guards cover each new fragment. The PC remains independently owned.
Day Care's 20x16 source bounds now authenticate the east 0x0a/tile0x10 padding
band before insetting its wall by four cells. A changed band, wrong dimensions
or different map rejects that inset. New regressions cover these cases, shared
wall planes, source mutations, independent PC ownership and unchanged footing.

The initial PC-only native build and eight comparisons completed; all eight
were inspected and the Day Care wall now meets its floor. The expanded shared
header build is tracked at receipts/house-header-wall-build.log; 68 comparisons
are prepared for all 53 candidate houses plus extra side/low views and three
walks. That broader build/review, the final full audit and focused tests must
finish before claiming the shared header milestone verified or refreshing its
base gallery entries. No pack export, audio, collision or progression changes
were made. Whole-room ornament/material, service/state, Hall/League and all
remaining original requirements stay open.


Shared domestic appliance-header milestone verified:

- The final optimized native build completed (receipts/house-header-wall-build.log).
  All six domestic tests pass, including new complete-margin wall-plane,
  independent PC ownership, mutation and custom-ownership checks. All 31
  related controls pass: full external perimeter donors including Day Care's
  inset/dimension/mutation guard, twelve floor tests, house windows/bookcases,
  two house stairs, two Dojo, eleven Radio and chair orientation. Receipts are
  house-header-wall-tests.log and house-header-wall-controls-tests.log.
- All 68 batch comparisons and the extra corrected Trainer walk in
  house-header-wall-review/ were individually inspected, with successful exits
  and all 69 paired state checks preserved. Coverage includes every affected
  house, east/west views of five layouts and both low side views of Bill's.
  DayCare UUDD follows (6,4)→(6,2)→(6,4), GoldenrodNameRater UUDD follows
  (6,4)→(6,2)→(6,4). Trainer's initial RRLL met its moving NPC on the last
  return step and ended (6,8); it is not complete return evidence. The corrected
  lower-aisle UD follows (4,11)→(4,10)→(4,11). All three successful routes end
  with matching runtime/terrain maps, active terrain, no pending build/fade/warp.
- house-header-wall-full311-audit.json verifies all 311 maps/225 script states
  and no mesher errors. Compared with dojo-notice-full311-audit.json, exactly
  the 53 candidate house records change, adding 214 modeled wall cells (2/4/6
  per map; Day Care and Trainer each gain six). All footing fields and all 225
  variant records remain identical. The four changed source files match the
  final native/test snapshot.
- All 53 current full311 base entries were refreshed, with provenance in
  full311-native-current-review/refreshed-house-headers.json. The initial
  eight-view PC-only gallery is superseded by this final batch. Temporary
  scripts and obsolete capture experiments were removed; final evidence and
  the stable build snapshot remain ignored. No packs/audio/collision/progression
  were changed or published.

Scoped wall joins are verified; this does not establish whole-room acceptance.
The wider house views show a remaining picture-frame mounting-height review.
Continue source/geometry/material and service/state coverage, the earlier
floor/stair/ornament issues, player-bedroom exterior shadows, native Hall
animation, continuous earned League traversal and every original full-scope
acceptance requirement. The active goal remains incomplete.


Mounted pictures under verification:
The source-complete house 0x0e/0x1e and mansion 0x17 two-by-two drawings now
reuse their existing closed frame geometry with native border paints and a
16px vertical lift onto the upper wall. Three recognized authored materials
are required; unknown frame roles retain the source fallback. Frame topology,
normals and cached geometry remain unchanged. Other picture source families
retain their existing paths pending source-specific review. Radio's native
wall/sign renderer already paints its full artwork, so the generic post-pass
is now suppressed for those exact drawings. The new regression counts the
source artwork separately from constant frame-color UVs, catching duplicate
Radio overlays. On-map tests check high placement, one complete artwork,
source mutation, custom ownership and unchanged footing.

The affected inventory is 59 maps (52 house, Celadon Mansion 1F and all six
Radio Tower rooms). A normal optimized build is tracked in
receipts/mounted-picture-build.log; six critical front comparisons are prepared
before wider validation. The native build, visual results, full311/225 audit
and final focused tests remain pending for this batch. Existing completed
header/wall milestones remain verified; whole-room and all original wider
service/state, Hall/League and coverage requirements remain active.


Mounted house/mansion pictures and Radio overlays verified:

- Normal optimized native build completed in receipts/mounted-picture-build.log.
  The source and verification snapshot match for all three changed files.
  Six viewer `picture` checks, eleven native Radio checks and 26 related
  domestic/perimeter/floor/stair/Dojo/window/bookcase/chair controls pass.
  The initial controls used two filters matching zero tests; the actual
  enhanced_room_floor (12) and chairs_face_aligned (1) checks were then run
  and passed. These 43 checks preserve topology, material-role guards,
  complete-source/custom ownership, single artwork and independent footing.
- The broad Cargo Bevy `picture` filter ran eight checks: six passed; two
  failed separately from the viewer change. active_pokemon_picture attempted
  the absent legacy apps/web/assets/data/content-packs/core-modular.crystalpack
  relative path; new_game_name_menu_and_preset_picture_motion compared
  " NAME " to "NAME". receipts/mounted-picture-tests.log retains the failures.
  They were not masked or treated as passing furniture checks.
- All six initial critical pairs, all 59 final front pairs, twelve east/west
  pairs and three return-walk pairs were individually inspected. All 80
  paired-state receipts preserve state and the successful native processes
  exit zero. Galleries are mounted-picture-critical-review/,
  mounted-picture-all-review/, mounted-picture-side-review/ and
  mounted-picture-walk-review/. The high frame retains one recessed artwork,
  source border colors and visible wall attachment from either side.
  Radio signs no longer receive a second generic artwork layer.
- GoldenrodFlowerShop UUDD follows (4,3)→(4,1)→(4,3), RadioTower2F follows
  (2,5)→(2,3)→(2,5), and CeladonMansion1F follows (7,6)→(7,4)→(7,6).
  Final runtime and terrain maps match, terrain is active, no build remains
  pending, fade is zero and warp phase is None. The first mansion setup at
  (9,6) was outside its 8x10 gameplay bounds and did not launch; it is not
  movement evidence. The corrected corridor route supplies that evidence.
- mounted-picture-full311-audit.json verifies 311 maps and 225 variants with
  mesher_errors=[]. Against house-header-wall-full311-audit.json, exactly
  53 records change only their solid vertex/triangle totals: each selected
  frame's 904 vertices/396 triangles now receive textured native paint.
  All ownership, modeled-cell, footing and 225 variant records are identical.
  Radio artwork changes are textured and not measured by these solid totals;
  its single-art regression and native comparisons provide that evidence.
- All 59 final base gallery entries are refreshed, with ignored provenance
  in full311-native-current-review/refreshed-mounted-pictures.json. No pack,
  audio, collision or progression change was made. Finished scratch runners
  are removed; accepted evidence remains ignored.

This closes only the house/mansion frame mounting/paint and native Radio
single-art correction. Other source families require their own review:
Battle Tower frames, Oak's lab, Game Corner, Pokecom office and eleven
Pokecenter second-floor maps still have frame ownership in the full inventory.
Players-room decorations and traditional-house drawings may use other native
paths and must be checked directly, not inferred from the family inventory.
Remaining furniture/material/ornament/stair/floor issues, all original map and
structural-state/service acceptance, native Hall animation and continuous
earned League traversal remain active. Whole-room acceptance is not established.


Oak lab notice follow-up under verification:
The current full311 Oak comparison shows two near-blank brass outlines where
the source has legible lab notices. The generic 0..16px frame/art was embedded
inside the restored 32px rear backing. Both authenticated lab 0x05 origins
(0,0 and 2,0) now reuse the native mounted-frame path: 16..32px vertical extent,
source white/dark/gray frame pigment samples and front relief beyond the wall.
The selected frame mesh and backing are preserved. The mounting regression
now covers both lab drawings and requires each nondegenerate artwork tile to
appear exactly once beyond the physical wall plane. Unknown/mutated/custom
sources retain existing guards. Test and native build receipts are
lab-mounted-picture-tests.log and lab-mounted-picture-build.log; pending final
renders/audit, this is implementation progress and not visual acceptance.

The other-family base review inspected Oak, BattleTower1F/Hallway, Pokecenter2F,
PokecomCenterAdminOfficeMobile and CeladonGameCorner. Their remaining issues
must not be hidden by frame-only acceptance: Battle Tower still uses disconnected
rear fixtures and a plain gray floor where the source has a pink pattern;
Pokecenter2F likewise loses its source floor motif. Other native Pokecom and
arcade signs already mount high, but complete room/service/state review remains.


Oak notice verification progress:
- lab-mounted-picture-build.log completed normally. Six comparison pairs in
  lab-mounted-picture-review/ were individually inspected: Oak front/east/west,
  Elm and Academy front controls, and Oak's UUDD route. Every process exits zero
  and every paired-state receipt preserves state. The route follows (4,7)→
  (4,5)→(4,7), with matching runtime/terrain maps, active terrain, no pending
  build/fade/warp. The notices are visible above the restored backing from all
  three Oak angles; control rooms retain their layouts.
- lab-mounted-picture-full311-audit.json verifies 311 maps/225 variants with
  no mesher errors. Against mounted-picture-full311-audit.json only OaksLab
  changes: solid triangles 9328→8536 and vertices 20280→18472, the expected
  two frames moving to source texture paint. All other fields and all 225
  variant records remain identical.
- The first expanded fixture used house floor tile1 in a lab source drawing;
  the resolver rejected it. Correcting test data to lab floor0x10 passes the
  expanded high-mount/single-artwork regression in
  receipts/lab-mounted-picture-corrected-tests.log. This changed only test data
  after the native build, not rendered production logic.
- Five selected controls passed before external_native_cabinets_and_tvs failed
  its legacy artwork assertion. Native-painted frame rear vertices are now
  textured too; the old assertion incorrectly required them all strictly in
  front of the source wall plane. It now separately requires the closed frame
  on/in front of that plane and exactly eight nondegenerate-UV artwork triangles
  strictly in front. The real Oak source layout joins its existing house and
  mansion pack coverage. The updated backing test is compiling in own cache
  (receipt lab-mounted-picture-backing-tests.log); final control verification
  remains pending. The earlier failure remains in
  lab-mounted-picture-final-controls.log. The base gallery is not yet refreshed.

Resume by polling the current backing-test job, then run the prepared ignored
run-lab-mounted-picture-controls.py runner and the expanded mounting test through
the completed viewer harness. Record actual counts/results, refresh only Oak's
accepted base capture, and remove finished scratch runners. Native captures and
audit already cover unchanged production code; subsequent edits only refine
fixtures and assertions. Keep the full goal active, including the newly confirmed
Battle Tower rear-wall/floor and Pokémon Center floor motif issues and the
remaining Hall/League/full coverage requirements.


Oak notice milestone verified:
The final source-pack backing test passes in
receipts/lab-mounted-picture-backing-tests.log. All twenty related controls
pass in lab-mounted-picture-final-controls.log, and the expanded mounting
regression passed in lab-mounted-picture-corrected-tests.log (21 checks total).
The earlier fixture/assertion failures are retained as diagnostic history; they
are not unresolved failures. The native build, all six inspected paired views,
settled UUDD route and full311/225 audit recorded above remain valid: subsequent
lab changes were test-only. Oak's accepted base entry is refreshed, with ignored
provenance in refreshed-lab-notices.json. Finished lab scratch runners are
removed. This closes only the notice mounting/paint issue, not full room or
original-goal acceptance.

Patterned facility floor candidate implemented:
A separate source matcher authenticates tile0x11 in the 33 known Center rooms
and four Battle Tower rooms. Exact metatile phase masks reject appliance/wall/
stair fragments and malformed metadata. Valid cells retain full source UVs and
use the existing shallow bevel; no large ceramic grid replaces the motif.
The darker Center service apron keeps its separate existing finish. Native-room
custom-profile exclusion now covers the newly included Pokecenter2F and Battle
Tower floors. Existing Center checks now require complete source UV mapping,
while separate apron guards remain intact. New checks cover full external-pack
phase identity, negative world/grid origins, cell bounds, unchanged support and
claims, and user floor overrides. A precompile comparison caught and corrected
one mask grouping for Center block0x30; all 2,048 pinned source phases now match.
Rust tests, production native build and 37-room visual review remain pending;
no patterned-floor acceptance or gallery refresh is claimed. Battle Tower's
rear architecture remains a separate repair. All original wider requirements
remain active.


Patterned tile0x11 floor milestone verified:
- The optimized native renderer/auditor and production binary build succeeds
  (receipts/patterned-facility-floor-build.log). Two new patterned_facility
  regressions pass against all external-pack phases and check complete motif
  UVs, negative grid/world coordinates, cell bounds, independent footing/claims
  and custom floor ownership. Twenty relevant controls pass, covering Center
  floors/aprons, shared room floors/custom overrides, mounted pictures, actual
  pack backing, twelve enhanced floors and chair orientation. Receipts are
  patterned-facility-floor-tests.log and patterned-facility-floor-controls.log.
- All 55 pairs in critical-review (7), all-review (37) and extra-review (11)
  were individually inspected. All preserve paired state and successful
  native processes exit zero. Fifty-one are accepted for the tile0x11 scope:
  35 final front rooms, five initial critical duplicates, six side views, two
  close/low views and three walking pairs. Four pairs instead confirm that
  BattleTowerBattleRoom and BattleTowerElevator are unchanged gray special-room
  renderings; they are diagnostic failures of wider fidelity, not acceptance.
- Violet UUDD returns (5,6)→(5,4)→(5,6); Tower1F UUDD returns (8,5)→(8,3)→
  (8,5). The first Pokecenter2F setup at (8,6) is a blocked source tile and
  did not launch. The corrected RRLL route follows (8,4)→(10,4)→(8,4).
  All three successful routes settle with matching runtime/terrain maps,
  active terrain, no pending build, zero fade and no warp phase.
- patterned-facility-floor-full311-audit.json verifies 311 maps/225 variants,
  mesher_errors=[]. Against lab-mounted-picture-full311-audit.json only three
  records change, solely solid vertex/triangle totals: Tower1F 29568→27824
  triangles and 63344→59856 vertices; hallway 10892→9880 and 23424→21400;
  Pokecenter2F 25072→23286 and 59808→56236. Other Center floors were already
  textured; their motif change is proved by UV tests/native views, not these
  solid totals. All coverage/ownership/footing fields and all 225 variants
  remain identical.
- All 35 accepted base entries are refreshed, with ignored provenance in
  refreshed-patterned-facility-floors.json. No pack/audio/collision/progression
  change was made. Finished capture/audit runners are removed. Broader full-room
  geometry, materials, service/state and original Hall/League/coverage gates
  remain active.

The initial 37-room candidate count included two rooms outside tile0x11's
actual source family. Special-room bindings already own every cell of the
battle room and elevator: Surface::Ceramic paints blue source floors gray and
Detail::Arena replaces the source red/white arena with a generic gray circle.
These are next repairs in special_rooms.rs, not reasons to accept fewer of
the original full-goal rooms. Their object/floor fingerprints already provide
source identity guards. The hallway also needs real door geometry and a
contiguous rear wall: the pinned BattleTowerHallway.asm warp events lead from
its northern door positions to the battle room. Generic GateTerminal models
misread those drawings. These findings remain explicit unaccepted work.


Battle Tower special-room floor correction implemented, under verification:
FloorStyle::TowerArena/TowerLift identify the two exact room bindings. The
source-complete 8x8 arena and individually authenticated Ceramic/Threshold
cells now retain one coplanar source quad per tile, rather than gray ceramic
or a raised invented circular arena. Existing fingerprints authenticate every
identity in the source drawing. Other room objects keep their model geometry
and receive the authenticated blue floor donor underneath. The new pack
regression expects battle-room floor counts [arena64,threshold16,ceramic116]
and elevator [0,8,36], checks UV/position correspondence, unchanged support,
every single-cell mutation, unknown map rejection and existing ownership.
It is compiling in the stable snapshot; receipt is
battle-tower-special-floor-tests.log. Build/native views/audit remain pending.
This does not close source wall colors, rear connectivity, door interpretation,
props/services or any broader original requirement.


Special-room follow-up and current Hall gate:
- The initial special-floor pack regression passes in
  receipts/battle-tower-special-floor-tests.log; the initial native build also
  passes in battle-tower-special-floor-build.log. Visual acceptance was not
  claimed. Source review then identified the two TowerRoundFixture drawings
  as floor start marks: both complete 2x2 plots use metatile0x23, whose four
  gameplay quadrants are FLOOR in the pinned collision source. The existing
  native renderer's raised cylinders are a misinterpretation. Those exact
  fingerprints now join the coplanar source floor path; the expanded test adds
  eight marker cells and checks their UVs/support/mutations with the arena.
  Other model shapes remain unchanged. Expanded tests/new native views/audit
  are pending; no floor acceptance is inferred from the initial narrower test.
- The current earned Lance ceremony gate passes on text-tui.crystalpack in
  12.79s and on core-modular.browser.crystalpack in 9.69s, without a stack
  override. Both run the ignored regression explicitly with --ignored and
  retain real battle PP use, victory/script continuation, ceremony ownership,
  title and production Continue checks. Receipts are native-hall-earned-current-
  pack.log and native-hall-earned-browser-pack.log. The initial invocation
  without --ignored only skipped the test and is not a passing gate.
- The old Hall save's core-modular identity no longer matches the shared
  modded packs. A fresh TUI fixture also correctly fails graphical identity
  because it includes +text-tui. The graphical fixture was regenerated through
  the battle/controller gate, not by editing the save or bypassing validation.
  Fresh source phase frames and earned saves are ignored under
  native-hall-animation-review/ and its browser-pack/ subdirectory. The owned
  native client then launches from a copied compatible save; UI inspection
  cannot continue while the Mac is locked. cua.listApps reported that automatic
  unlock failed and asked for manual unlock. An asynchronous unlock request
  is pending while independent floor work continues. Native full ceremony
  animation review, continuous earned League traversal and all original wider
  requirements remain unproven.


Battle Tower special-floor milestone verified:
- The expanded marker/floor regression passes in
  receipts/battle-tower-special-floor-marker-tests.log. All 23 room-family
  checks pass, including that regression, native shrine, Dojo, geometry normals,
  unknown drawing and support controls (battle-tower-special-floor-controls.log).
  Four more checks pass: two patterned facility source/UV/custom floor tests,
  the full external perimeter donor control, and mounted source pictures
  (battle-tower-special-floor-extra-controls.log). Total: 27 viewer checks.
- The normal optimized final native build passes in
  battle-tower-special-floor-final-build.log. Both changed source files match
  the stable build snapshot. No source export, audio, collision or progression
  change was made. The initial narrower build/test receipts remain diagnostic
  history; the final version also restores the eight start-mark cells as flat
  source artwork rather than raised blue cylinders.
- All thirteen final pairs in battle-tower-special-floor-review/ were inspected,
  with successful process exits and preserved paired states. Eleven pass the
  floor palette/inlay review: front, east, close and walk views for each room,
  battle-room west and marker-walk, plus the final wider elevator west view.
  The initial elevator west and west-fit views crop the floor too tightly and
  are retained as rejected framing diagnostics. The final west-wide camera
  (-2,6,60) shows the full cabin. A moving marker-crossing frame was inspected
  as well: the player's feet follow the flat floor, not a raised model cap.
- Battle room UUDD returns (4,4)→(4,2)→(4,4); elevator UD returns (2,2)→
  (2,1)→(2,2). The marker route RLLR follows (1,5)→(2,5)→(1,5)→(0,5)→
  (1,5). All settle with matching runtime/terrain maps, active terrain, no
  pending build, no fade and no warp phase.
- battle-tower-special-floor-full311-audit.json verifies 311 maps and 225
  variants with mesher_errors=[]. Only the two expected records change versus
  patterned-facility-floor-full311-audit.json, solely solid mesh totals:
  battle room triangles 25268→10380, vertices 75520→30856; elevator triangles
  8552→4904, vertices 25656→14712. All ownership/coverage/footing fields and
  all 225 variant records remain identical. Both rooms retain min/max footing0.
- Both accepted front base entries are refreshed, with ignored provenance in
  refreshed-battle-tower-special-floors.json. Finished capture runners are removed.
  This closes only floor palette/arena/mark/mat representation. Whole-room
  wall height/material/continuity, doors, equipment, service/state and all
  original wider acceptance remain open.

Battle Tower door source clarification for the next structural pass:
Metatile0x04 collision is WALL,DOOR,FLOOR,FLOOR. Its left 2x2 GateTerminal drawing
is on the wall quadrant; the right 2x2 PictureFrame drawing is the actual door.
The hallway's odd-X north warp positions align with the right half. Repair the
pair with a wall/control detail on the left and a real doorway on the right;
do not backfill the door opening, and do not turn the left monitor model into
the door. Also inspect the offset central elevator entry at warp(11,1) rather
than guessing it follows the same north-plane source drawing. Native source
wall colors and full room closure remain unaccepted.

Native Hall inspection status:
The separate graphical earned-entry gate now passes in 3.34s, with a fresh
entry save under native-hall-animation-review/graphical-entry/. The complete
ceremony gates and their valid packed source frames remain evidence only for
controller/software rendering. A new owned native process loads a copied entry
save and remains open for UI inspection. The earlier compatible process was
closed after regenerating this explicit entry fixture. The source Hall map
uses MUSIC_NEW_BARK_TOWN deliberately; queued music is not proof of its map.
Mac unlock is still pending, so no native Hall animation acceptance is claimed.
Continuous earned League traversal and every original full-scope requirement
remain active.


Further pier ramp refinement (six walking tiles)

The renewed request for slower ramps extends the outdoor shore grade from
eight to twelve source cells: 64 to 96 native pixels, or four to six walking
tiles. The ordinary shore slope is one third shallower. Geometry and actor
support continue to share the same vertex heights. The raised-garden contact
on Route12 gets a short height constraint propagated along the deck graph,
preventing the longer low approach from creating a lip at the fixed soil edge.
The first five-check run exposed that lip; after the correction all five
focused pier regressions pass. The expanded raised-paving fixture checks the
full twelve-cell approach, halfway height, deck endpoint and shared vertices.
An independent Route12 source-map check also found zero overly steep cell
edges and zero raised shore contacts.

The optimized native capture build passes. Three fresh same-state native/2D
pairs were individually inspected: Route25 at (47,9), Route12 at (5,34), and
Route32 at (13,60). All preserve paired state. Route12 DDUU reaches (5,36) and
returns to (5,34); Route32 UUDD reaches (13,58) and returns to (13,60). Both
settle at zero presentation offset with matching runtime/terrain maps, active
terrain, no pending terrain and no warp. A Route32 moving frame was inspected
for foot contact. This accepts the gentler shore grade and joins in these
views; unrelated scenery and full-room acceptance remain open. Evidence is
ignored under target/kanto-resume/piers-six-tile-review/ and the five-check
and native-build receipts are piers-six-tile-tests.log and
piers-six-tile-build.log.


Battle Tower native wall/door and projected-canopy milestone — 2026-10-17

Metatile04's left 2x2 WALL/control drawing now resolves as WallPanel; its right
2x2 DOOR drawing resolves as GateDoorFrame. The source-bound native builder
replaces the former terminal and floor-mounted picture interpretation in
BattleTowerHallway and BattleTower1F. Controls retain their native 16px detail
scale on 32px walls, and door leaves sit behind clear approaches. Hallway side
returns and the south enclosure connect at ceiling height, with the renderer's
cutaway ranges intact. Control-bearing panels are deliberately excluded from
generic perimeter donors; they cannot repeat accessories around the lobby.

The hallway's separate metatile25 drawing includes a projected top row. Its
WALL,FLOOR,FLOOR,DOOR collision quadrants make the lower-left floor a real
walkable strip. Initial geometry consumed that strip: the additional native
pose at (10,1) showed the player intersecting the surround. The corrected
builder folds its control and door onto the main wall, maps all four cap cells
onto a thin horizontal canopy at 32px, and leaves the ground/body space free.
The external regression now checks that floor/doorstep strip and all eight
nondegenerate source-cap triangles on top, as well as source bounds, every
cell mutation, unknown maps, prior claims, door approaches and unchanged support.

Verified evidence:
- tower-native-architecture-final-build.log confirms the final normal optimized
  native build. All four changed source files match the stable verification
  snapshot. A disk-full archive attempt was recovered by pruning 1518 MiB of
  obsolete files in this goal's own cache; no other goal's cache was removed.
- tower-native-architecture-final-tests.log passes the expanded external
  wall/door/canopy/floor regression. tower-native-architecture-final-controls.log
  passes nine controls: complete external room-wall donors, bounded signatures,
  two patterned-facility floor checks, Tower special-floor source/support,
  mounted pictures and three Hall enclosure/source-guard checks. Ten checks
  total; the Bevy filter selected zero and is not counted as a passing gate.
- All fifteen pairs in tower-native-architecture-clear-floor-review/ were
  individually inspected, exit successfully and preserve paired state. They
  include hallway front, east/west center, four east/west end poses, regular
  door and canopy close views, corridor walk, both canopy-side walks, and
  lobby front/door/approach. The central side frames crop the ends; the extra
  end poses cover those joins instead of claiming one side image shows all.
  A moving frame on the left canopy-side route was inspected for body and foot
  clearance. Representative rejected frames and the unused-door traces remain
  in tower-native-architecture-diagnostics/; superseded full batches and the
  finished runner are removed. The rejected frames are not acceptance.
- Hallway RRLL returns (4,2)→(6,2)→(4,2). Left and right UD routes return
  (10,2)→(10,1)→(10,2) and (12,2)→(12,1)→(12,2). Lobby UD returns
  (7,2)→(7,1)→(7,2). All finish at zero presentation offset, matching runtime/
  terrain maps, active terrain, no pending terrain, no fade and no warp phase.
  Separate unused-door diagnostics bounce through (1,1)/(1,0); those are not
  counted as clean return-path or service/warp evidence.
- tower-native-architecture-full311-audit.json passes 311 maps and 225 variants
  with mesher_errors=[]. Only BattleTower1F and BattleTowerHallway records change
  from battle-tower-special-floor-full311-audit.json. Lobby solid triangles
  change 27824→26836 and vertices 59856→57716; hallway 9880→0 and 21400→0.
  The replacement architecture uses textured geometry, so zero solid totals
  do not mean no wall geometry. Hallway modeled cells increase 80→92 for its
  newly recognized twelve-cell canopy drawing. Existing terminal/picture
  labels become wall/door labels. All footing fields across all maps and all
  225 variant records remain identical. Visual/clearance evidence, not these
  solid-only totals, proves the canopy folding and door/wall finish.
- Both front base entries are refreshed with scoped provenance in
  full311-native-current-review/refreshed-tower-native-architecture.json.

Whole-room Tower acceptance remains open: the lobby still has missing rear
backing around equipment, clinical placeholder wall colors, mismatched waiting
seats and unreviewed screen/counter representation. Compare its source03/1b/
1d/1e control/cap drawings with the current TowerReception/desktop interpretation
before choosing replacements. Battle-room and elevator walls/materials need
their own review. Full service/state/reachability, all311/original16 coverage,
native Hall animation and continuous earned League traversal are still required.

Mac lock was freshly confirmed by the UI inventory tool. Independently, the
existing render_saved_session utility supports --walk A... through the normal
production input owner; its fresh optimized build passes in
native-hall-saved-session-build.log. A copied earned save and owned quick-save
path must be used when evaluating complete Hall animation and Continue. That
route has not yet been executed or accepted; static ceremony/source frames
and independent Lance fixtures remain insufficient for the outstanding gates.


### Hall native route diagnostics — 2026-10-17

The current saved-session utility was synchronized into the verification
snapshot and rebuilt with a refreshed mtime; the first attempt used a stale
parser and failed with unexpected extra arguments. Fresh A64 and alternating
AB64 routes on separate copies of the earned graphical Hall entry save both
exit zero and capture 2048 production frames. Their final player tile remains
(4,4); dialogue closes but the machine approach and ceremony never start. The
alternating route final native frame was inspected. Neither run proves native
ceremony animation. Receipts: native-hall-api-route-old-snapshot.log,
native-hall-saved-session-build.log, native-hall-api-route-diagnostic.log and
native-hall-api-alternating-diagnostic.log under target/kanto-resume/receipts/.

The saved checkpoint was taken during Lance's Hall speech. LoadSave initializes
active_script_cursor to None, and restore_visible_loaded_runtime_state restores
navigation/music without recreating that authored cursor. The source Hall
script still has the slow machine step, HealMachineAnim and halloffame after
that speech. This evidence points to the unsuitable mid-script checkpoint; it
does not justify seeding a cursor or champion flags for visual acceptance.
A quiescent pre-battle checkpoint/reload branch is being added to the earned
Lance gate so the native review can run the complete continuation in one
process. The full311/original16, native animation, continuous League and
whole-room/service/state/reachability requirements remain open.

The new pre-battle checkpoint branch passes the graphical-pack earned complete
ceremony gate: native-hall-reloaded-prebattle-gate.log, one ignored external
regression explicitly selected, 11.13s. It asserts a quiescent cursor and closed
dialogue before saving, reloads with LoadSave, verifies the same position and
unearned champion flag, then executes the existing real battle, PP mutation,
Hall approach, every ceremony phase, credits, autosave and Continue assertions.
The output checkpoint is LancesRoom-before-battle.crystalsave in the ignored
prebattle-entry/ review directory. These controller/source-frame checks still
do not substitute for the native animation review.

render_saved_session now accepts optional --quick-save PATH, wired to the
production config without changing the default. The normal optimized example
build passes in native-hall-quicksave-utility-build.log. The outside-render goal
was notified before the additive shared-example edit. The native checkpoint
route uses a separate copied input and ceremony-autosave path; it does not
overwrite the source fixture or a user's game save.

Native U18+A256 on the copied pre-battle checkpoint executes all 8768 input
frames, with 18 grid origins and three additional terrain builds (median
16.74ms, p95 21.71ms). It wins Lance, enters Hall, advances to (4,3), animates
the machine, records the ceremony and starts credits. Native moving frames
20/38/135/195/205/210/215/220/225/230/240/260/273 were inspected, covering
intro/battle/victory/Hall speech/machine effect/white transition/Pokémon and
trainer presentation/rating/early credits. This is sampled visual evidence,
not acceptance of every ceremony phase or the complete credits program. The
battle close view also exposes large foreground statue occlusion; broader
battle composition remains unaccepted.

The native-created autosave passes a Rust load/readback in
native-hall-autosave-probe.log: Hall count 1, recorded Mew, earned Elite Four
flag, stopped timer and pending post-champion Continue spawn. The short route
exhausted its A inputs during credits. render_capture readiness requires active
world/battle geometry, so it cannot take its final pair there. The owned process
was explicitly stopped after that limitation was confirmed (renderer exit1);
no final paired-state receipt or complete native Continue proof is claimed.
Representative early frames and the full Hall/early-credit segment remain in
native-prebattle-route/, with redundant early frames pruned.

A longer U18+A520 route is running from another copy to keep input available
through title/Continue and return to the active world. An initial timeout600
option was rejected before startup because the supported range is 60..300;
that failure is retained in native-hall-complete-route-timeout-option.log. The
actual longer run uses the supported 300-second bound. Both the normal current
example and the modified Lance regression match their verification snapshots.
No gameplay flags, collision, scripts, audio or content packs were changed.

The longer route completes all 17216 frames (18 grid origins, three extra
terrain builds, median16.75ms/p95 20.67ms), displays the credits and reaches the
Continue summary. It remains there through repeated A presses, so the renderer
exits1 at its supported timeout while waiting for an active world. Frames
480/500/520 show the same summary; the temporary PlayersHouse2F backing state
in the trace is the title shell, not evidence of a completed Continue. A
50-frame contact sheet of native frames200..249 was inspected for the machine,
white transition, Pokémon back/front/animation/hold, trainer back/front/rating
and closing fade into credits. Native frame410 shows later PAAD testing
credits. All frames remain ignored in native-complete-route/. No final pair or
return-to-game acceptance is claimed.

The actual defect is in press_visible_title_confirm_button: it checks exported
main-menu readiness before handling an already-open Continue summary, even
though Continue has dispatched out of that loop. The focused fix routes A to
the existing summary transaction first, preserving its authored delay. The
earned Lance regression now uses actual title A dispatch/confirmation and
asserts that early A neither bypasses nor consumes the twenty-frame delay; it
no longer calls the save loader directly. The outside goal was notified before
the shared title-menu edit. This strengthened gate is compiling. A separate
legacy cancel-loop test was attempted before the fix but fails earlier, before
opening its summary; native-hall-continue-before-fix.log is not a passing gate
or an isolated confirmation regression.

The strengthened confirmation gate passes on the current graphical pack in
10.20s and TUI pack in11.03s (native-hall-continue-confirm-gate.log and
native-hall-continue-confirm-tui-gate.log). It drives A to open the real summary,
asserts its twenty-frame delay and ignored early A, supplies the production
presentation clock, then drives A again and observes NewBarkTown with Hall
count1, consumed post-champion spawn and resumed timer. The production handler
change is limited to summary A ownership before exported main-menu readiness.

The renderer_neutral_controller_ control filter passes six checks: Mom/New Bark
gate, actual battle move/PP, research-center callback, running/menu ownership,
Link Cable through Pack, and both department-store clerks. One selected rematch
check fails before runtime creation because its separate default
trainer-rematches-tui.crystalpack path is missing (trainer_rematches.rs:8);
CRYSTAL_RENDER_TEST_PACK does not override its CRYSTAL_REMATCH_TEST_PACK lookup.
That broad result is six passed/one failed, not a green suite. Receipt:
native-hall-continue-controller-controls.log.

The native credits/title contact sheet (47 sampled frames250..480) was also
inspected: all four border Pokémon/color phases, names, the retained THE END,
return through black and the Continue summary are visible. This strengthens
ceremony/credit visual evidence but does not turn the stuck pre-fix summary
into a successful end-to-end run. The fixed native utility build is ongoing.

The final normal optimized saved-session utility build passes in1m39s
(native-hall-continue-fixed-native-build.log). title_menu.rs, the earned Lance
regression and render_saved_session.rs were explicitly byte-matched with the
verification snapshot. Native U18+A500 is now running from a new copied
quiescent checkpoint, with separate ceremony-autosave output under
native-fixed-complete-route/. Live owned tool session25628; receipt
native-hall-fixed-complete-route.log. It must be revalidated and its terminal
result/final pair inspected before claiming native end-to-end acceptance.
No other owned build or earlier native capture remains live. Both temporary
Rust autosave-probe source and binary were removed after the successful
readbacks; packs and users' saves were not modified.


### Native Continue return and continuous League gate — 2026-10-17

The fixed native U18+A500 replay reaches NewBarkTown (13,6) through actual
credits/title/Continue input. Frame480 was inspected at the home entrance. Its
last row has matching runtime/terrain map, active terrain, no pending terrain,
zero presentation offsets and fade, and no warp phase. All16576 inputs finish
(19 grid origins, four added terrain builds, median16.77ms/p95 41.95ms). The
process exits1 because the global300s timer is checked as soon as the long
route settles; capture timeout reports inactive_reason=None. No final pair or
paired-state receipt is claimed. This identifies a capture-deadline problem,
not a failed Continue. The log also retains a nonfatal stale-entity command
warning during the title→world transition and an empty-mesh warning.

render_capture's comment says ordinary walking owns its duration, but the old
check charges all elapsed route time to final readback. The outside goal was
notified before a focused timeout correction: keep startup bounded, exclude a
finite running input route, and start a fresh bounded readback window when it
settles. Two clock regressions cover an overlong route, bounded readback and
static/unstarted startup. They are compiling; acceptance remains pending.

A new ignored external regression uses one controller through Will→Koga→
Bruno→Karen→Lance→Hall and the same ceremony/title/Continue helper. Only the
initial legal level100 Mew party with Surf/Ice Beam/Psychic/Flamethrower is
supplied. It selects moves through ordinary menu inputs, spends carried PP,
earns each exit/champion flag and walks the production doorways. No per-room
controller reset, healing injection, victory flags or teleports are allowed.
The first compile caught the typed enemy species id; the first execution
caught Psychic's native move id PSYCHIC_M. Both fixture issues were corrected
(continuous-league-compile-diagnostic.log and
continuous-league-move-id-diagnostic.log); no continuous traversal pass is
claimed yet. Ceremony completion was factored into a shared helper, with
distinct temporary save names and review subdirectories for Lance/League.


### Earned native Hall return accepted; continuous audio failure — 2026-10-17

The final native U18+A500 run exits0 (native-hall-final-complete-route.log).
Both full-size final images were inspected at NewBarkTown (13,6), and its
paired-state receipt confirms state_preserved=true, four actor sources and
identical authoritative checksum/LCD frame. The last route row is settled:
matching active runtime/terrain map, no pending terrain, zero offsets/fade and
no warp. All16576 input frames execute, with19 grid origins/four extra terrain
builds, median16.77ms/p95 45.42ms. Final readbacks finish at332.07s/334.02s;
this directly exercises the corrected post-route deadline after the old300s
limit. Performance/whole-town art acceptance is not inferred from these timings.

The final52-frame ceremony contact sheet and48 sampled credits/title frames
were inspected. They cover the machine approach/effect, white transition,
Pokémon back/front/animation/hold, trainer back/front, rating, fade, all four
credits border/color phases, THE END, title and return to gameplay. A Rust
readback of this run's actual autosave passes in
native-hall-final-autosave-probe.log: count1, Mew, earned Lance/Elite Four flags,
stopped recorded timer and pending post-champion spawn. The temporary probe
source/binary were removed. This is one earned one-member fixture, not a full
story playthrough, every team variant or a continuous League traversal.

A closer machine pair was attempted with the existing HOLD_HEALING_BALLS=1
control. It fails honestly when the route exhausts: healing effects are only
published as world actors for native clinics, while Hall retains the drawn
HealMachineWorldSprite source overlay. Its failed receipt is
native-hall-machine-pair.log; sampled frames remain in native-machine-pair/.
The QA hold is being extended narrowly to require Hall kind2 and the actual
rendered ball marker. Clinic world-actor checks remain intact. Machine-effect
placement is not yet accepted as a same-state comparison. Redundant frames
from superseded failed batches were pruned; representative diagnostics,
contact sheets, traces and final successful replay remain ignored.

The continuous gate now reaches and wins Koga after ordinary trainer-facing
Up input. Arrival from the left lane can leave pathfinding's last step facing
right; A then does not talk to the trainer. The harness now turns through Up
and only navigates a move cursor when battle messages do not own input. No
production movement, scene or collision changes were needed for that fixture
correction. The next failure is real: Bruno's move animation requests selector2
for Mew, and CRY_MON_MEW is absent. Receipt:
continuous-league-cry-lookup-diagnostic.log. Continuous League remains unproven.

The pinned BattleAnimCmd_Cry loads species base cry registers and adds the
source .CryData length words [0xC0,0x40,0,0], with16-bit wrapping. The Rust source
builder exports base CRY_* programs, not a species alias catalog. The repair
queues the supplied base program with those exact registers, reads its embedded
source even for PCM/gzip playback storage, and validates canonical PCM before
derived synthesis. PlaySlowCry has the same obsolete alias and is corrected
through its existing -0x140/+0x60 register path. Ordinary unparameterized audio
still uses its original source. No pack, exporter, MIDI/source catalog or PCM
files are regenerated. The external decode regression, source word-offset
control, Miltank's existing exact PCM golden, full pack hash/frame/loop audit,
and continuous gate must pass before accepting this audio repair.

Current own build: session82683 compiles the external base animation-cry decode
regression in native-base-animation-cry-test.log. The final capture deadline
suite already passes four checks in native-hall-capture-deadline-final-tests.log;
its live-request timestamp resets are retained. Native deadline-fixed build
passes in1m32s. All affected live sources were byte-matched before compiling.


### Pack-backed cry repair and sparse weather controls — 2026-10-17

The external animation-cry decode check passes on graphical MIDI storage
(0.80s), TUI storage (0.84s) and native PCM storage (4.66s). It queues Mew's real
base cry with each source selector's register offsets, decodes nonempty
nonlooping PCM, verifies duration ordering/selectors2=3 and preserves the
canonical source. Receipts: native-base-animation-cry-test.log,
native-base-animation-cry-tui-test.log and
native-base-animation-cry-pcm-test.log. The source word-offset control passes
(native-cry-word-offset-control.log). Miltank's existing exact slow-cry golden
passes unchanged:19198 frames/hash db796201, bad canonical hash rejection,
ordinary-byte preservation, distinct cache keys and WaitSFX ownership
(native-base-slow-cry-golden.log). No speculative alias or duplicate audio
catalog was added. Derived cries read the supplied embedded source even when
ordinary playback uses PCM/gzip. Unparameterized playback keeps its old source.

The repository's Rust verify_crystalpack source was compiled against the
updated libraries and run with --audio on all three shared packs. Each passes
all377 PCM hashes, frame counts and loop ranges: cry-base-pcm-audit-
core-modular.browser.log, cry-base-pcm-audit-text-tui.log and
cry-base-pcm-audit-core-modular.log. No pack/audio export or PCM deliverable was
written; the temporary verifier binary was removed. The full optimized native
cry/Hall-hold build passes in1m31s (native-cry-hall-hold-native-build.log).

The continuous League gate advances further but fails in Bruno with
MissingWeatherModifier(Sandstorm,WATER), retained in
continuous-league-sandstorm-diagnostic.log. Pinned DoWeatherModifiers scans its
sparse exception table and leaves unlisted weather/type pairs neutral; the
old Rust function rejected an absent weather row before its sparse-type
fallback. The correction applies only declared multipliers and otherwise
keeps damage unchanged, while retaining malformed-token validation. Explicit
Sandstorm mod overrides and the original rain/sun modifiers still apply.
The competitive-battle goal was notified before the shared core edit.

continuous-league-weather-controls.log passes five weather checks, including
all17 native types under unlisted Sandstorm and explicit Sandstorm/rain/sun
multipliers. continuous-league-damage-controls.log passes all37 damage tests.
The actual continuous gate must still run successfully; these controls alone
do not prove a completed gauntlet. Current owned build session49195 is
rebuilding it in continuous-league-earned-gate.log with the graphical pack.
All11 scoped live source files were byte-matched with the stable snapshot
before that compile. No other owned capture/build is live. After success,
rerun the continuous gate on TUI, verify relevant controller/cry controls after
the core rebuild, and build the normal native utility with the core correction.

The Hall-specific live source-sprite hold now produces a successful closer
front pair at camera(zoom1,yaw0,pitch45): native-hall-machine-final-pair.log,
6432 ordinary input frames,18 origins/three added terrain builds,
median16.82ms/p95 27.59ms, readbacks127.18s/127.42s. Both full-size images were
inspected. The first ball lies on the lower central recording console in the
native and source drawings; the player stands at(4,3), and paired-state is
preserved. This accepts that front placement, not camera-rotation projection
or all six party positions. The original clinic-only hold failure remains a
diagnostic. The QA change accepts an actual Hall kind2 HealMachineWorldSprite;
clinic holds still require their published world effects. No production
healing geometry/projection was changed. Those wider views remain open.


Continuous League diagnostic checkpoint — 2026-10-17

The carried-party controller now acknowledges arrival scenes whose phase is
Text even when the generic window flag is false, and faces trainers through
ordinary Up after left-lane approaches. Its initial five Full Restores are used
through Start/Pack/USE/party input; TUI verification observed 212→340 HP and
stock5→4 with spent PP unchanged. One graphical full League/Hall/Continue run
passes in39.74s. The TUI run fails in39.85s: Will212/340 HP with72PP; healed
Koga340/340 with67PP; Bruno340/340 with59PP; Karen then ends with MewHP0 and
an opened exit flag. This is not accepted as a successful gauntlet. A bounded
24-observation battle history is being collected to distinguish defeat handling
from fixture strategy. The pinned HandleEnemyMonFaint and HandlePlayerMonFaint
both check for a usable player party before awarding trainer victory.

Post-fix isolated controls pass: earned Lance/Hall16.54s, Mom1.22s, actual battle
move/PP1.15s. The optimized native build with the weather correction passes
in2m11s. These results supersede the earlier rebuilding status; they do not
prove continuous TUI or full interior acceptance. Evidence is ignored under
target/kanto-resume/receipts/continuous-league-*.log.


League failure reproduced and scoped correction drafted — 2026-10-17

The diagnostic build passes once (36.41s), followed by two passing TUI repeats.
The third repeat fails (17.58s). Its bounded history proves actual Destiny Bond:
confused Mew217HP, Gengar122HP using DESTINY_BOND, then critical Surf makes
both faint. The controller queues proper out-of-Pokémon/whiteout narration,
then incorrectly resumes KarenScript_KarenDefeatText and sets the exit flag.
This is a controller whiteout-settling defect, not a successful battle.

The production screen-fade body is factored into advance_visible_screen_fade;
the graphical Time wrapper still calls it, and renderer-neutral settling now
advances the same fade/white-hold/recovery/fade-in before script continuation.
The new ignored external regression carries a level5 Magikarp through an actual
Karen loss and requires an unopened exit, recovery warp/healing, no battle,
no retained cursor/messages/fade, and loss result1. It is rebuilding.

Separately, lobby source03/1b/1d/1e rear control drawings are drafted as native
wall panels. Native full-height backing retains source controls; three-row
drawings fold their top row into a ceiling-height canopy above the FLOOR
strip. Generic perimeter donors exclude these control-bearing drawings.
The extended source-authenticity/aperture fixture now includes the five lobby
banks and their projected-floor clearance. No visual acceptance is claimed.
The initial viewer-only compile was interrupted to avoid a duplicate dependency
feature graph; combined Bevy/viewer checks are the next verification command.

Verification integration note: copying the live Bevy shell registration also
carried another goal's new publish_pokemon_dimensions scheduling references
without its matching frozen dependency graph. Compilation reported exactly
two E0425 errors for that symbol. Those two unrelated scheduling additions
are excluded only from the frozen verification copy; live source is preserved
and the Pokémon goal was notified. The whiteout implementation remains exact
between live source and verification.

The carried Mew fixture now has legal RECOVER instead of FLAMETHROWER. Its
normal menu strategy waits out publicly narrated Gengar Destiny Bond with
Recover, uses Psychic against Gengar/Vileplume and Ice Beam against Murkrow,
and heals below half HP. No RNG roll or mid-run state is replaced. This avoids
intentionally attacking into the reproduced legitimate loss condition while
the separate losing-Magikarp regression guards correct defeat handling.


Shared whiteout correction and continuous earned route verified — 2026-10-17

The real level5 Magikarp/Karen losing regression passes on TUI (1.76s) and
graphical (1.20s). It observes an actual trainer battle and loss, never allows
the exit flag, then checks recovery warp, full HP, loss result1, and cleared
battle/cursor/messages/fade. Existing controls pass: four whiteout cases
(4.33s), two blackout cases (1.97s), two screen-fade timing cases (0.00s).
The production clock wrapper delegates unchanged behavior to the shared
advance_visible_screen_fade body. The renderer-neutral controller advances
that same sequence before any abandoned trainer script can resume.

The complete carried-party Will→Koga→Bruno→Karen→Lance→Hall→credits→title/
Continue→NewBark controller route passes with the updated legal move strategy
on both packs. TUI52.26s carries HP322/340 after Will, uses a real Full Restore
(stock5→4), then retains340HP and57PP after Lance. Graphical60.30s carries178HP
after Will, uses Full Restore5→4, carries252HP after Karen, uses Full Restore
4→3, then retains340HP/56PP after Lance. No mid-run flags, HP, moves, positions
or RNG are supplied. Each battle spends carried PP and earns its own door
flag, ordinary walking crosses the doors, and the shared Hall helper verifies
ceremony/credits/autosave and actual title/Continue confirmation. Receipts:
continuous-league-whiteout-controller-test.log, continuous-league-fixed-
{whiteout,blackout,fade}-controls.log, continuous-league-fixed-gauntlet-tui.log,
and continuous-league-fixed-browser-kanto_controller_*.log.

This closes the reproduced renderer-neutral whiteout continuation defect and
the carried-controller route gate. It does not accept all sixteen native room
states, every party/rotation variant, native full League replay, or all311
interiors. Lobby wall-bank geometry and visual review remain in progress.


Lobby rear-bank geometry checkpoint — 2026-10-17

The combined Bevy/viewer build passes (4m04s). Two actual external-pack checks
execute: Tower native architecture and separate arena/elevator floor control.
The architecture fixture recognizes six lobby wall banks plus the real door,
retains the hallway's ten door/wall pairs and elevator, checks all edited/cropped
source guards, unchanged footing, and clear FLOOR strips under folded caps.
Six perimeter controls also pass. The native capture/audit examples are building;
rear-bank visual acceptance is still pending.


First native lobby wall-bank review and correction — 2026-10-17

The normal capture/audit build passes in1m38s. Three front/east/west pairs
exit0; the front image confirms desks are replaced by source control panels,
but rejects the remaining rear gaps above both counter returns. The native
wall course must continue behind them. The first full311/225 audit has zero
mesher errors, changes only BattleTower1F, retains every footing field and all
225 script variants. This is intermediate evidence, not final wall acceptance.

The revision extends the authenticated lobby perimeter with plain native brown
paint and adds full-height rear backing on both sides of the exact source door.
Changed door drawings disable this inference. A ray/triangle control checks
every rear cell above counter height except the actual doorway, and all source
warp centers remain body-clear. Folded canopy tops now have finished one-pixel
side/underside thickness. The combined external geometry check is rebuilding.

The first rear RRLL walk timed out before capture with None scene evidence.
Its invocation omitted CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic; the QA source
defaults to focus-dependent keyboard input and returns while unfocused. This
failed attempt is not movement evidence. The corrected invocation must use
the existing explicit native synthetic input path and inspect its trace.


Rear-backing entrance safeguards

The revised lobby perimeter uses actual boundary warps to leave both south
entrance tiles (7,9 and8,9) open through the wall thickness. It preserves the
authenticated north door (7,0); source04's right half is the door. The expanded
fixture supplies the pack's real warp coordinates, requires four side/south
wall ranges in the lobby, checks the two south apertures through their depth,
and checks continuous rear backing at every non-door source cell above the
counters. Missing entrance evidence or an altered door suppresses inferred
lobby backing. This revision is still rebuilding and has no visual acceptance.

The next furniture corrections remain concrete: source02 is the front service
counter at map block row3/column4, not a ceiling-height clinical wall; source05
is the blue PC at row3/column3; source06 is the staff seat; source07 is the gold
waiting-seat course. Long counter returns use14/15/19/28/29/2a/2b. They still
appear as flat artwork or unrelated furniture in the rejected room views and
need native volumes after the rear join is verified. These identities come
from the pinned map blocks and same-position 2D comparison.


Live verification handoff for the connected-backing revision

Cargo session30228 was confirmed running by write_stdin at this checkpoint;
poll that handle again rather than relying on this note. Its log is ignored
under target/kanto-resume/receipts/tower-native-connected-backing-test.log.
This is the combined external Tower architecture/floor command in the frozen
verification checkout. All current geometry bytes match that checkout; the
Bevy shell keeps only the documented unrelated dimensions-registration
exception. No source synchronization is pending.

After success, rerun the six perimeter and nine signature controls, rebuild
the render_at_location and audit_authored_assets examples with the same
features/profile/cache, and capture a new review directory. Use explicit
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic and CRYSTAL_RENDER_TEST_WALK_STEPS=1.
Use the verified LLRR rear route from(8,1), reaching(6,1) and returning.
Correction: source2b marks(10,1) FLOOR and(11,1) WALL; the earlier note that
RRLL would hit a wall at(10,1) was a misread and is withdrawn. A separate south entrance DUU route
from(7,8) can verify the actual outward/return warp. Inspect all pair states
and movement traces. Repeat the311-map/225-variant audit against
tower-native-architecture-full311-audit.json; the intermediate desk-to-panel
audit changed only BattleTower1F and preserved all footing and variants.
The current native examples and control-bank review images predate the new
connected backing; do not use them to accept that revision.

Completed handles: controller/cry/weather checks and the two new loss/gauntlet
pack runs are terminal. Intermediate native capture session19969 ended1 due
to its unfocused keyboard rear walk; audit session3508 ended0. The earlier
backing builds were explicitly interrupted to finish canopy thickness and
south entrance apertures; only session30228 owns current compilation.


Battle Tower lobby rear-bank and connected-backing acceptance — 2026-10-17

The expanded external architecture and arena/floor controls pass after an
8m51s combined build. Six perimeter controls and nine signature controls also
pass, for17 focused checks. The optimized production examples build in1m38s.
Five fresh paired views were inspected individually against their own 2D
reference: front, east, west, rear-walk and entrance-return. A moving rear
frame was also inspected for foot contact. All five receipts preserve state.
Rear gaps above both counter returns are closed at ceiling height, side walls
use the source brown field, the native control/display art replaces office
desks, and folded caps have finished sides/undersides. The north elevator and
two south entrance tiles remain open. This accepts connectivity, aperture and
rear-bank role for these views; side-wall ornament, furniture and whole-room
fidelity are not accepted.

Rear LLRR follows(8,1)→(7,1)→(6,1)→(7,1)→(8,1). The first DUU entrance route
only reaches the carpet and ends(7,7); it is not a return-warp proof. Corrected
DDUU follows Tower1F(7,8)→(7,9)→Outside(8,9), forced arrival step to(8,10),
then Tower1F(7,9)→(7,8). Both accepted return walks settle with runtime and
terrain maps matching, active terrain, no pending load, zero offsets/fade and
no warp phase. The entrance return adds two terrain builds and three origins.
The initial rear focus timeout remains recorded as a failed attempt.

The final311/225 audit has zero mesher errors; only BattleTower1F changes,
with every footing field and all225 script variants identical to the prior
verified architecture audit. The base lobby gallery is refreshed with the
front comparison, and its scoped refresh record lists the remaining furniture
and ornament defects. Evidence lives under target/kanto-resume/
tower-native-connected-backing-review and receipts/tower-native-connected-
backing-*.log. Audit: tower-native-connected-backing-full311-audit.json.
All own handles are terminal:30228 and87182 builds exit0;61570 capture batch,
42029 audit and90731 corrected entrance capture exit0. No compiler is live.

The exact source collision is2a=WALL,WALL,WALL,FLOOR and
2b=WALL,WALL,FLOOR,WALL. Thus(10,1) is FLOOR and(11,1) is WALL. The earlier
route note assigning WALL to(10,1) was incorrect and is superseded here.
The verified LLRR rear route remains valid. Keep the full original objective
active: all311 interiors and original16 states, services/state/reachability,
shared furniture, Hall party/rotation fidelity and native full League replay.


Native Tower furniture implementation, pending verification — 2026-10-17

A new map-scoped Rust native_tower_furniture module owns the complete native
U service counter, source05 blue rules kiosk, source19/28 counter PC and six
source07 gold stools. Counter02 now has ReceptionCounter semantics instead
of WallClinical, and07 has Stool semantics instead of LinkSeat. The actual
blue staff chair06 remains separate. The source map identifies the blue kiosk
at(6,6) as the rules sign; the gray terminal at(11,6) retains actual PC
collision. No game collision, event, text, inventory or service behavior changes.

The counter builder keeps white cap rows horizontal and blue fascia rows
vertical. Front courses are anchored at the source front edge; the left elbow
has a real deep corner, and the terminal return meets its authenticated
keyboard/footer course. That footer must remain complete and unclaimed before
the extension can be selected. All long returns have closed blue bodies and
finished tops. The kiosk is an integrated blue cabinet with its own complete
face; the counter terminal has a closed gray screen housing above its real
keyboard cap. New wall signatures2a/2b restore the original control courses
behind the two counter returns, using the existing native wall builder.

Each gold waiting stool has four grounded feet, a closed eight-sided seat with
a chamfered rim, and source-chart-clipped native top art. Its engraved/button
detail remains on top; lower feet pixels are not stretched across the seat.
Native seat metadata is7 pixels, matching the mesh, with no office-chair back
or arms. This code and its source guards are implemented, not yet visually
accepted. Cropped/altered/custom-claimed drawings retain the existing path.

The new external regression resolves the actual pack map, requires all16
pieces (three front courses, five ordinary returns, one terminal return, one
kiosk, six stools), checks ownership, unchanged walking support, closed-face
winding, bounded heights, grounded stool feet/chamfers and source/override
rejection. It also samples the whole U countertop for physical gaps. The first
build was interrupted to finish front-edge anchoring and the coupled terminal
footer; session16798 now owns the current combined build and was confirmed
live by write_stdin. Log: receipts/tower-native-furniture-test.log. All five
modified furniture/architecture source files match verification exactly.
No snapshot synchronization is pending; do not modify it while Cargo is live.

Next: poll16798, inspect the result rather than counting compilation as a pass,
then rerun the external architecture test (now eight wall banks), six perimeter
and nine signature controls. Rebuild production render_at_location and
audit_authored_assets examples. Native images currently available only prove
the prior connected-backing revision; fresh front/side/close furniture pairs
and return walks are required. Repeat the full311/225 audit against the
connected-backing report, and test the rules kiosk/PC/reception source services
where needed for the original whole-room acceptance. Keep every wider original
requirement active. No whole-room or full-goal completion is claimed.


Battle Tower furniture verification and source-text correction — 2026-10-17

The native Tower furniture regression and seventeen architecture/signature/
perimeter controls pass; the combined unit build took8m45s and native examples
1m39s. Thirteen paired captures in tower-native-furniture-review all preserve
state. Front, both sides, counter/kiosk/PC close views, stool close view, all
three final walks, and rules/PC/reception service pairs were inspected. The
counter is continuous and low, with blue vertical fascia and white horizontal
cap; the gray PC has a real keyboard surface and the blue rules cabinet has
finished flat sides/top. Six chamfered gold stools replace office chairs.
Static stool-close exposes a follower seed on blocked stool(3,8), so that
whole pose is not accepted. The Pokemon goal was notified; no follower edits
were made. Rear LLRR and entrance DDUU retain the prior clear routes; stool
LR returns from(12,9) through(11,9). All settle with active matching terrain,
no pending load, offset/fade0 and no warp. Full311/225 audit changes only
Tower1F; all footing fields and all225 variant records remain identical.

The battle_tower controller filter returns9pass/4fail. Three challenge paths
fail on missing _BTGreetingF10Text/M9Text/M23Text. Source routines correctly
select the labels, but source_asm_text.rs omitted data/battle_tower/trainer_text.asm.
The exporter now reads that external pinned file and validates all120 greeting/
win/loss messages for25 male and15 female trainers. No copied text catalogs
were added to source. The fourth failure is the outside roof test's absent
repo vendor PNG; the outside goal was notified. Rules native capture proves
only its first page, PC capture proves its root menu, and reception capture
proves dialogue, not a completed challenge.

The first fixed-tool build failed because the frozen verification source lacked
pret-source.lock.json (include_str! in pack_core.rs:261). The real repository
file was byte-copied into that checkout; rebuild session22057 is live. No
external source export or shared pack change has happened. Challenge services,
side-wall ornament, follower placement and whole-room acceptance remain open.


Battle Tower source-text candidates — verified build and content comparison

The fixed Rust tools build succeeded. pack_core against the external pinned
export-workspace passed --export-asm-text and --pack-prepared-workspace.
Existing pack_all_pokemon then pack_default_battles compose native/browser
candidates; pack_text_tui derives the TUI candidate. The competitive goal
confirmed no competing publication and preservation of All Pokemon/default
battle composition. All three candidates add exactly120 selected Tower
messages, preserve every old text entry and every other game-data field, and
have identical runtime files and embedded audio payloads. A temporary Rust
comparison program first used audio_manifest() incorrectly on compressed native
storage; the corrected comparison checks stored payloads plus all data fields
and exact text additions. Existing/native candidates both verify normally.
No production audio defect was found. All377 programs on all three candidates
pass PCM hash/frame/loop validation; desktop parameters pass native/browser.

The candidate fixes all three missing-text failures: twelve Tower controller
checks pass8.77s. Seven neutral controls pass9.02s after explicitly selecting
the existing trainer-rematches fixture (its frozen default path was absent).
Earned Lance→Hall→credits→title/Continue passes browser18.54s/native26.03s.
An old Oct16 terminal binary fails on the current competitive_battles schema;
its old unit binary also selected zero MCP tests. Neither is passing evidence.
A fresh current frozen-source terminal/test build is live as session41337.
Publish only after real MCP movement/menu and clean-stderr checks pass.
Shared packs remain unchanged. Receipts use tower-source-text-* names.

Furniture moving frames were also inspected for rear approach, entrance return
and stool lane; the lane retains a clear follower trail, while static blocked
seed placement remains open. Side-wall diagnostic source blocks16/18 contain
two-column wall fields and17/1a their corners. Current perimeter backing is
at the outside map edge, leaving those native fields flat at the floor.
A future source-authenticated inward-facing wall course should preserve
collision boundaries and corner floor cells rather than stretching the flat
source painting or moving walkable tiles. No side-wall geometry edit yet.

The deeper native rules interaction (UUAAAA from6,8) now reaches “All three
must be different.” on both presentations; its fresh candidate-pack pair was
inspected and state_preserved=true. This proves progression beyond the rules
intro, not every rules page or a native completed challenge. Browser/native
earned Hall checks and all three exact-content comparisons are terminal passes.
The terminal build remains live as41337; it is rebuilding the unified
Ratatui/Bevy dependency graph, not waiting on a stale lock. Do not restart it
solely because compilation is lengthy. No shared publication yet.


Blocked initial follower seed — implementation pending integration/native QA

Main party_follower.rs now queries collision/occupied tiles with a copied player
and the production step predicate only when initializing/resetting a trail.
If the trailing tile is blocked, no invented segment is seeded and no follower
actor is published until a full gap has been walked. Clear ordinary seeds,
actual path history, mount, palette and dimensions behavior are preserved.
The isolated actual trail implementation passes its blocked/short/full/corner/
warp/clear-seed regression. A second external-pack test probes Tower(3,9)'s
stool, Tower(8,5)'s clear floor, then an occupied floor and unchanged session
state. This second test has not run. No native result is accepted yet.

The Pokemon goal was notified of the two source/test hunks. The frozen source
is deliberately untouched while terminal build41337 remains live. After it
finishes, copy party_follower.rs and append only the two new functions from
tests/party_follower.rs; do not overwrite that frozen test file with unrelated
Pokemon dimensions tests. Then run integration and native stool/ordinary
trail checks. Current fresh-pack terminal checks/publication still come first.

Three existing gallery comparisons were additionally inspected: Lugia speech
house, Itemfinder house and player home1F. The traditional rear horizontal
rail field remains flat behind the TV, and the player's broad plank geometry
loses the source's narrow horizontal floor bands. These are follow-up native
fidelity defects, not new accepted rooms. Both traditional house tatami fields
retain their actual green pattern. No floor/wall edit was made in this turn.


Tower source text published; follower integration build running

The fresh terminal main test binary e58d1801cf5511b1 runs the actual MCP
movement/Start test:1pass1.02s. Fresh stdio_clean-84af7208262f8f68 passes1.72s
and fresh geothite dump succeeds. The aggregate no-run wrapper41337 ended101
after building these binaries because verify_train_station imported
bevy::render::asset::RenderAssetUsages. Its import is now bevy::asset; no
behavior changed. That example's rebuilt verification is still pending.

Repeated exact comparisons immediately before publication pass for all three
packs; canonical native/browser/TUI are now replaced atomically per file.
Only the120 new Tower messages differ in data; old texts, runtime files and
embedded audio remain exact. Published content hashes are native5474b146…,
browserd344850e…, TUI969fe6e8…. Other goals were notified; exporter ownership
is released. No save files migrated. Matching current graphical fixtures
were freshly earned by four Elite victories11.48s and five entrance-lock
scenes1.96s, in current-text-pack-earned-states. Full native state coverage
remains open. README/game-content document the verified source stages.

After terminal41337 was confirmed terminal, the follower production file and
only the two new test functions were synced into the frozen source. The station
import was synced too. Focused party_follower_ Cargo test73087 is live with
Bevy/geothite and location-tester, preserving the unified dependency graph.
No follower integration/native acceptance yet. No other own Cargo job is live.


Follower integration compile repair and rotated Hall capture

Integration73087 ended101 because two inline tests at the bottom of the
production follower module still used its old return type/signature. Eight
clear-seed calls now pass true and unwrap the expected visible point. All
three isolated actual-trail tests pass, including old corners/warp and
10,000-step bounded history controls plus the new blocked-seed case.
Replacement integration96694 is live; the frozen source remains untouched
while it compiles. Main/frozen follower production files match.

The current graphical pack's earned Lance checkpoint/Hall/Continue fixture
replay passes10.78s, writing a matching LancesRoom-before-battle save. Native
rotated-machine capture28069 replays U18+A500 from that quiescent save through
ordinary production input; HOLD_HEALING_BALLS=1 waits for the actually drawn
Hall source ball, then freezes the pair at camera1/1/45. Output is
native-hall-animation-review/current-text-pack-machine-east. No rotated
visual acceptance yet. The station verifier import fix remains unbuilt.


Follower checks pass; actual rotated Hall attachment defect reproduced

Follower retry96694 failed with LLVM “No space left on device,” not a test
assertion. The obsolete owned206MiB test binary and186 unneeded moving-frame
PNG scratch images from the just-completed Hall diagnostic were removed;
final paired failure evidence and selected route frames remain. Retry86037
passes all five Bevy follower checks1.30s (geothite filter has zero tests).
The source-owned production helper rejects the real stool and occupied floor
and leaves persistent state unchanged. Surf rider filter selected zero tests,
so it is not evidence. The12 Surf-anchor checks initially failed their explicit
unchanged-source-water fixture prerequisite on the modded pack. With the
prepared vanilla-source browser fixture all12 pass9.66s. Native follower
acceptance remains pending.

Rotated native replay28069 exits0 after6441frames,18 grid origins,3 terrain
builds, median16.66/p9524.96ms; final pair state_preserved=true. Both full images
were inspected. The recording ball visibly floats west of the machine. This
is a rejected attachment view; the earlier front alignment was insufficient.
The Hall OAM sprite was never published as a native effect because render_mod
only exported clinic effects. It therefore retained an unrotated 2D overlay.

Main/frozen now publish Hall balls too, require the exact complete native
machine and built-in ownership, and fold all six original OAM centers onto
the physical tray at10.15px. The bottom-pivot quad offset is included. The
guard reuses the native machine's complete drawing and custom-profile overlap
check; source/custom changes cannot acquire guessed placement. Existing clinic
code is unchanged. New tests cover all six centers, horizontal orientation,
scroll, each machine cell mutation, custom ownership and duplicate cells.
Build27801 was intentionally interrupted after discovering the bottom-pivot
correction was needed; replacement23311 is live. No new projection acceptance
yet. The Hall helper's optional KANTO_HALL_REVIEW_PARTY_SIZE=1..6 legal initial
party extension and exact recorded member count are synced too but untested.

Only own Cargo23311 is live. After it passes, run the relevant Hall/clinic/
follower controls, build the native examples and corrected station verifier
with Bevy+geothite unified features, inspect fresh blocked-stool static/walk
pairs and fresh Hall front/rotated six-ball pairs. Preserve all original311/
16-state/service/reachability/party-variant requirements.


Native follower accepted; Hall map-space correction and station failures

Hall tray test23311 passes1 viewer test after9m18s (other package filters0).
Four Hall geometry, three clinic and five follower controls pass; real nurse
publication/consumed-overlay test passes1.67s. Native examples plus the corrected
station verifier build successfully35.50s. Follower batch56577 produces three
inspected same-state native/2D pairs, all preserved: stool-static, stool-walk,
ordinary-clear. No head appears through the stool. LR follows(3,9)→(2,9)→(3,9),
settles with matching active terrain, no pending build, fade0/warpNone. Full
resolution ordinary-clear shows Totodile partly occluded behind the player;
it is present and the normal seed is preserved. This is scoped seed acceptance.

Both tray-corrected-front/east captures exit0 and preserve state but are
rejected: no native ball is visible, while2D shows it. The actual published
effect exists at center(-12,160), size32, in expanded116x114 grid with origin
(-48,-48). Converting that publication gives actual map position(77,32), not
raw OAM center(77,48). Source camera translation already shifted it16px north.
The projection now authenticates six actual map-space slot centers and places
them4px south on the tray; its test includes the captured full expanded grid
and expected world center(-12,40.6,-144). Build31312 is live. No fresh native
acceptance yet. The earlier front-only alignment and the first tray patch
are not evidence of correct3D attachment.

Six-party helper initially stalled by confirming Shift and repeatedly selecting
the active Mew. Ordinary B now declines the actually presented Shift choice;
no cursor or progression is assigned. Rebuilt gate41548 passes13.38s through
earned Lance victory, all six recorded members, credits and Continue. Matching
party6 fixtures are in current-text-pack-earned-states/party6. Native A-only
replay still needs an explicit ordinary-input keep-battler QA option; the
render_walk owner was notified, no new flag implemented.

Station verifier90859 runs the current pack and returns6pass/4fail, reporting
both controller round trips and both mesh apertures. Power/pass denials and
west approaches pass. Trip failure is auto-A exceeding256steps while a visible
train owns the clock. Main deterministic_session.rs adds visible_magnet_train
and visible_heal_machine to that boundary list, allowing outer frame settling.
The geometry predicate rejects the model's intentional continuous roof; the
train's existing portal test explicitly permits that roof at21px. Main verifier
now applies the same finite opening height rather than demanding an infinitely
tall empty column. Both fixes are pending; do not claim station acceptance.
These two main files are not yet copied into frozen source because31312 is
live. Preserve its compiler. After terminal, sync only these pending changes,
rerun production verifier and relevant controller/animation regressions.

Old owned pre-TUI library cache entries and obsolete test binaries were removed
to keep disk space available. Current graph is viewer efcc5ae / Bevy be534569;
unit binaries are viewer4d2b9e3368c8c563 and Bevy26c49de673e475b3. Do not use
a binary until the corresponding current build is complete. No other own
Cargo/native capture process remains live besides31312. All canonical packs
remain the published120-text-only correction. Original311/16-state, room,
service, native ceremony/party-variant and continuous reachability scope remains.


Hall six-slot native projection and station controller follow-up — 2026-10-17

The corrected map-space Hall test31312 passes. The optimized native build18276
passes4m23s, including the projection, taller train, clock-owned input boundary
and opt-in keep-battler QA routing. Native map-space-one-east, six-front and
six-east captures were inspected against their same-state2D frames. The balls
sit on the machine tray in the front and rotated views; all three preserve
state. Both six-member captures publish all six healing actors and log160
KeyX frames from declining actual Shift prompts. These views accept tray
attachment/rotation, not all animation phases, party variants or whole-room
quality. Both six-member native replays exit0.

The new eight-frame-observation station verifier still passes8/10. Geometry,
power/PASS denials and west approaches pass. Both round trips retain an active
MagnetTrain at (6,5), despite1024 empty controller frames. The renderer-neutral
settling loop advances healing but omitted the train sequence entirely. Main
now advances the same production train function there and adds a regression
for both directions, pending synchronization after the live headroom build.
The headroom test's Vec2 qualification is now synced. No production watchdog
limit was raised. A fresh native taller-train view is in progress.

The two new taller-train pairs have now completed and were inspected:
Goldenrod at(6,6), orbit1, and Saffron at(11,6), orbit0. Both preserve state.
The rounded roof and roof vents remain finished; player/door scale now clears
the trainer. The Goldenrod view is close and clips part of the train, so the
Saffron front view supplies whole-train composition evidence. This accepts
these size/clearance views only. Evidence: station-taller-native-review/.
The first attempted capture used unsupported --tile-x, then an incorrectly
comma-separated camera request; both failed before producing accepted images.
The successful commands use --x/--y and a space-separated camera triplet.

The next house-floor defect is confirmed in interior_surfaces.rs:
native_house_material_floor_cell samples source pixel(0,0) for wood, then
native_house_wood_floor_field substitutes a four-source-cell plank pattern and
sinusoidal grain. This discards the native horizontal bands. Carpet already
retains its complete native UV quad. The wood-family correction should retain
that full native drawing while preserving the existing floor/footing datum;
no change is implemented or visually accepted yet.

The qualified headroom build completes successfully in9m48s. All five
train_station_scenery tests pass, including actual male/female player clearance,
continuous roof, source guards, custom ownership and retained door/rail/footing.
Zero-test package results are not counted. The controller train branch and
new two-direction regression are now synced into frozen source; the unrelated
Pokemon publication scheduler exception is retained. Focused magnet_train
checks are live as session3140, receipt station-controller-settling-tests.log.
After they complete, rebuild verify_train_station and rerun its actual public-
controller round trips. Do not accept travel from headroom tests alone.

Wood floor source-band correction, implementation pending verification

Main now retains complete native wood UVs at the exact existing floor datum.
The replacement four-cell planks and sinusoidal grain are removed from this
wood path. The same helper handles player/home/gift-shop wood, traditional
room tile50 and floor donors beneath traditional wooden walls, so those joins
cannot retain the former single-pixel field. The player-home regression now
requires full source UV correspondence and unchanged footing; the material
regression checks level joins, complete source patterns, upward faces and
negative viewport phases. Carpet behavior is unchanged. These two files are
not yet synced while the station-controller test build3140 is live:
mesh/interior_surfaces.rs and mesh/modeled_interiors.rs. No native acceptance
is claimed until the new build and same-state comparisons complete.

A before-change player-home1F pair at(6,5), orbit1, was inspected and preserves
state. It confirms broad replacement planks versus the fine source bands.
The first EcruteakItemfinderHouse fixture(3,4) is blocked and failed startup;
the default(4,4) is also blocked. The next explicit fixture uses(3,6);
prior successful gallery evidence provides(5,4) if another position is needed.

Train transition failure isolated; wood build started — 2026-10-17

Station-controller test3140 completes4m07s: the new two-direction controller
completion regression and two source animation/music controls pass. Three
old art tests fail: two read raw assets through an erroneous root, and the
frame test retains title presentation state. Their fixtures now use mounted
pack artwork and an actual station field session. This is pending verification,
not three accepted tests. Verifier build41651 passes, but both round trips
now expose a later error: warpcheck commits the destination before executing
the departure script's newloadmap, causing a map mismatch at index13.

execute_visible_pending_script_warp now recognizes a following newloadmap,
stages that actual compiled opcode while still on the departure map, arms its
real next cursor and leaves the existing map-load consumer to commit the
warp/setup. A new source-authenticated regression checks both stations retain
the departure map with pending destination warp and MAPSETUP_TRAIN together.
No map validation was relaxed or interpreter duplicated.

The two floor files, battle_results.rs and audio_and_battle_ui.rs are synced.
The Pokemon scheduler exception remains unchanged. Session99975 is compiling
focused train tests with the full wood correction. After terminal success,
run the native-room floor unit filter, rebuild native examples, rerun all10
station verifier cases and inspect fresh same-state wood pairs.

The EcruteakItemfinderHouse before-change pair at(3,6), orbit1, now passes and
was inspected. It preserves state and shows the broad wood field around
unaltered green tatami. Initial(3,4) and default(4,4) were blocked fixtures;
neither is accepted evidence.

Train source continuation remains a fidelity audit item. Exact pinned source
maps place warpcheck before newloadmap MAPSETUP_TRAIN, then applymovement
PLAYER .MovementBoardTheTrain (turn_head DOWN), wait20 and end. The existing
map-load consumer resets navigation/cursors, so passing the pending-warp staging
regression or reaching the other station alone must not be presented as proof
that this post-load continuation ran. Check this sequence and arrival facing
before whole-trip/source-fidelity acceptance. The current verifier checks actual
arrival dialogue, officer return, PASS retention and ordinary return movement,
but does not yet assert that turn-head/wait continuation.

Audit evidence clarification: audit_authored_assets currently reports footing
cell counts and min/max heights, not every cell value or a whole-field digest.
Comparing all of those reported fields proves summary equality only. The
checkpoint now calls them footing summary fields. Keep that distinct from
focused tests that explicitly compare full fixture footing vectors, and from
source review showing a finish helper never mutates the footing vector. The
new full audit must not be cited as per-cell equality by itself.

Combined train/wood build99975 passes9m07s. All seven focused train tests pass
in5.12s: the three former art failures, source music/sprite-order controls,
controller completion in both directions, and pending warp/newloadmap staging
for both stations. The three fixed art fixtures now exercise the external pack
rather than relying on missing raw assets or title state. All33 native-room
floor tests pass0.43s from the fresh viewer binary, including full source wood
patterns, player-home floor datum, source/custom guards and contiguous backing.
The native examples/client build is live as session5104; no new source changes
are pending synchronization. Actual round trips and after-change wood captures
remain pending, as does the post-load source-continuation audit.

Wood source-band and reciprocal station verification milestone — 2026-10-17

The optimized native build passes37.75s. The refreshed public-controller station
verifier passes all10 cases. Real reciprocal trips now reach the other station,
show its arrival dialogue, return the officer to(9,9), retain one PASS and allow
ordinary movement afterwards. First destination snapshots are at(11,5) facing
Down with no pending script. Both production mesh/footing checks, power/PASS
ordering cases and west-door approaches pass. The post-newloadmap source
continuation remains unproven as documented above; no whole-trip/source-fidelity
acceptance is inferred from this verifier alone. Native boarding/door animation
review also remains open.

All33 native-room floor checks pass0.43s. All thirteen new after-change native/
2D pairs were inspected individually and preserve state. They cover both floors
of player/Red/Copycat homes, EcruteakItemfinderHouse, KurtsHouse, MountMoonGiftShop,
MahoganyMart1F, a player-home closeup and two ordinary return walks. Full native
bands/speckles are retained, carpets/tatami keep their own patterns, and wall-
foot donor strips use the same native finish. Home DU visits(6,6) and returns
(6,5) over64 logged frames. Traditional RRLL visits(4,6)/(5,6) and returns(3,6)
over128 frames. Both final traces show matching runtime/terrain maps, active
terrain, no pending terrain, zero center offset/fade and no warp. One moving
frame from each walk was inspected for foot contact. The unfocused keyboard
attempt timed out with no movement capture; the successful reruns explicitly
set CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic and WALK_STEPS=1, retaining the
production input dispatcher. The final after-results.json records the failed
first home-walk attempt and both successful final walks.

The full311-map/225-variant audit passes with no mesh errors. Every map audit
record, reported footing summary and variant record is identical to the prior
Tower-furniture audit. The audit's summary-field limitations still apply.
Ten inspected base gallery entries are refreshed with their matching classic
pairs, identities and paired-state records. The disposable completed source-
text comparison example is removed from frozen source. No build, native capture
or source exporter is left running; no code changes remain unsynced. No staging
or committing occurred. These are scoped floor/return-walk and station-controller
milestones; original full311, service/state, continuous-native-League, Hall
party/phase variants and all broader fidelity requirements remain active.

Post-newloadmap source continuation implementation — 2026-10-17

Exact pinned Script_newloadmap sets MAPSTATUS_ENTER and StopScript, which
clears only the per-frame SCRIPT_RUNNING flag. It does not end the interpreter
stream. The current take_visible_pending_map_load instead reset the cursor and
armed the destination scene, dropping the source arrival movement, wait20 and
end. Main now retains a newloadmap return cursor and resumes it on the live
destination before new scene input, like the existing reload continuation.

Source-program ownership and live target ownership remain distinct. The asset
movement path loads the actual source program; PLAYER targets the current live
trainer across maps. Named objects retain source/live-map equality checks.
Only pause/wait timing opcodes may cross that equality check in the runtime
command path; other map-state commands remain guarded. Existing control/text
continuations already permit source-owned streams after warps.

The expanded two-station regression requires pending warp and newloadmap before
commit, actual destination player movement, wait20's native120-frame delay,
the source end opcode, released source cursor, and immutable rejection of a
departure NPC movement at the destination. The asset/credits/test files are
synced and compiling as23830. An earlier synchronization assertion failed before
copying; a stale-snapshot test command98945 completed but is excluded from all
new-code evidence. The corrected sync/build23830 is the authoritative attempt.

Disk pressure was addressed only in the owned cache:207221021 bytes of obsolete
old-core libraries and their dependent graph were removed. Fingerprint evidence
shows current Bevy depends on core82e65efe91ebd1ff, which was retained. No other
agent cache or shared pack was deleted. No source exporter is active. The new
post-load change is not accepted until its regression, neutral/reload controls,
actual train verifier and native presentation checks pass.

Post-load regression evidence and control fixtures — 2026-10-17

Build23830 completes9m23s: six train controls pass, while the expanded regression
fails only its test probe for an end event still pending in state. Source
movement and120-frame wait ran. Normal continuation drains control events and
consumes the real end, so the assertion now checks the bounded production
end-state consumption record with exact script/index. Build63526 passes2m22s;
all seven train checks pass5.05s, including both complete arrival tails and
immutable rejected departure NPC commands. The test-only movement stderr log is
cfg(test); production logging was not changed.

The current production-code unit binary passes eight neutral controls6.15s,
all twelve Tower gameplay controls7.76s, and fourteen reload controls20.63s.
The first neutral attempts lacked the fixture's separate rematch path; the
successful run sets CRYSTAL_REMATCH_TEST_PACK to the existing absolute
content-packs/trainer-rematches-tui.crystalpack. The broad Tower filter also
selects an outside roof-art test, which still fails on a missing raw artwork
file at tests/overworld_rendering.rs:569; outside owner notified, unchanged.

The legacy visible_newloadmap fixture fails before executing the handler:
its hardcoded .FlyScript@FlyFunction label is missing from the current pack.
The fixture now takes the actual newloadmap MAPSETUP_FLY source/index/setup
from the external pack's global typed command catalog. This test-only follow-up
is synced and compiling as31183 (train-post-load-actual-fly-fixture-tests.log).
No new source code is pending synchronization.

The BattleTower1F gallery entry was found to predate the accepted native
furniture revision; it still showed the former blue seats and old counter.
The accepted tower-native-furniture-review/front pair confirms the six gold
stools and U counter. A fresh unchanged-renderer baseline at(8,6), orbit0 is
running as2723 before updating the gallery and examining the side-wall fields.
Native post-load train replay and full original acceptance scope remain open.

Authored map-load priority gate and Tower gallery refresh — 2026-10-17

The global-catalog Fly fixture attempt31183 fails its explicit prerequisite:
there is no matching newloadmap MAPSETUP_FLY entry in this pack's global typed
catalog. No Fly implementation or pack-build claim follows from that result.
The generic dispatcher-priority fixture now stages warpcheck and newloadmap
from the actual Goldenrod train program, retaining the actual source cursor.
It verifies the map-load consumer wins over the ordinary warp consumer and
installs the authored destination while clearing both requests. Build43258
passes; that one real test passes0.75s. This covers priority, not Fly behavior.
The native client/examples rebuild is now live in train-post-load-native-build.log.

The fresh Tower front fixture first rejected blocked(8,6). The recorded review
identity establishes(8,5), hour12; the rerun26442 passes and preserves state.
Its current native/classic pair was inspected and the base Tower1F gallery is
now refreshed, removing the stale pre-furniture image. Gold stools, low U counter,
source PC/rules kiosk and rear banks match the previously accepted furniture
milestone. Side-wall source fields are still flat beside outer backing and
remain a real geometry correction to implement. This capture uses the unchanged
renderer before the script-continuation native rebuild; it does not establish
new train runtime behavior.

Native reciprocal rides and destination-scene lifetime follow-up — 2026-10-17

The source-continuation native build71554 passes35.01s. Its station verifier
passes all10 cases; fresh MCP movement/menu passes0.91s and clean-stderr passes
1.40s. Native replay61289 exits0 after63.57s through3424 logged frames, with
Gold -> train LCD -> Saffron -> train LCD -> Gold transitions. Both arrival
phases reach(9,10) before the deliberate Down segments; those held directions
then walk to(9,13). The final Gold(9,13) pair preserves state with matching
runtime/terrain, active terrain, no pending load, zero offset/fade and no warp.
Final native/classic images and moving frames03/08 were inspected. Moving03
shows boarding text;08 shows the faithful LCD ride surface. These moving frames
have no same-state2D counterparts, so they are diagnostic rather than complete
boarding/door-animation visual acceptance. Both final frames were inspected.

The new native view also confirms a palette discrepancy: Gold's3D waiting-area
floor is gray while its matched classic floor is blue. The explicit native
station floor adapter is Saffron-only in the current finish pass. No floor fix
is implemented here yet. The stationary train adapter deliberately leaves door
cells to the ordinary source pass; the requested3D slide-out door presentation
has not been proven. Whole-station art acceptance remains open.

A second source-lifetime audit found destination scenes could be skipped after
the retained newloadmap source stream finished. Main now marks that handoff
with the existing deferred_script_warp_arrival_scripts flag. Normal End uses
the existing take_visible_script_end_state path, which resumes the caller or
arms the deferred destination scene; EndCallback retains its consume-only
behavior. No extra callback replay is added. The expanded train gate requires
the exact source-end consumption before destination-scene start and end, and
checks the defer flag/cursor are released. These credits/battle-results/test
changes are synced and compiling as71947 in train-deferred-arrival-scene-tests.log.
Previous passing controls and the native replay predate this last change;
repeat relevant controls/native verification before accepting it. No exporter
or native capture remains active; no code is pending synchronization.

Deferred destination-scene gate passes — 2026-10-17

Build71947 completes3m48s; all seven train checks pass5.29s, including exact
source-end -> destination-scene-start -> destination-scene-end order,120-frame
wait, source movement and rejected departure NPC mutation. Fresh final-code
controls pass: eight neutral6.38s, twelve Tower gameplay8.78s, fourteen reload
22.13s and one authored map-load priority0.88s. The known outside raw-art test
is explicitly outside this Tower gameplay filter and remains unfixed.

Native build29094 fails at linking render_at_location-69d426d8e7bc0bcd with
errno28 (No space left on device), not a timeout or code diagnostic. The owned
superseded reciprocal replay's107 moving frames consumed336629132 bytes.
Pruning to24 diagnostic frames frees260760849 bytes; final paired images,
identity/state records and full3424-frame trace remain. Completed loose compiler
objects add2264920 bytes reclaimed. Current native roots remain the preceding
version until the retry confirms success. The same native command is retried
in train-deferred-scene-native-build-retry.log. No exporter is running and no
source is unsynced. Relevant controls pass, but final native/MCP/stdio checks
must be rerun after this last scene-handoff change.

Final scene-handoff verification and native artifact repair — 2026-10-17

The first native retry98715 exits0 with a nonfatal rust-objcopy disk-full warning,
but file inspection finds render_at_location is data, not Mach-O. Its cached
hash69d426d8e7bc0bcd and root copy are both1296 bytes with magic319d4f57.
Deleting the root alone only recopies the corrupt cache and yields another
apparent success; that attempt is not accepted. Removing both corrupt artifacts
forces relink21692, which passes11.53s. Every native root (client, render_at_location,
render_saved_session, station verifier and asset audit) then has a valid native
Mach-O header. A real render_at_location Tower1F(8,5), hour12 pair23755 exits0
in6.19s, was inspected and preserves state. It refreshes the base Tower gallery.
This verifies actual capture execution as well as the header check.

Final-code station verification37320 passes all10 cases. Fresh MCP movement/
Start menu passes1.17s and clean-stderr passes1.54s (supervisor51785 exits0).
Across final train/neutral/Tower-gameplay/reload/priority/MCP/stdio receipts,
43 distinct nonzero passing test names are counted, eliminating the duplicate
renderer-neutral train check. The known outside-roof raw-art fixture is excluded
from the twelve Tower gameplay cases and remains a real unfulfilled prerequisite.
No broad full-suite-green claim is made.

Native final-scene replay70689 exits0 after57.58s with route A40 U A40 U from
the existing matching Goldenrod power/PASS fixture. It records2624 frames and
Gold -> rideLCD -> Saffron -> rideLCD -> Gold. The source routes complete ordinary
boarding, travel, arrival and receipt/dialogue handling. The final position is
Gold(9,10), facing the gate after the last blocked Up, with matching runtime/
terrain, active terrain, no pending load, zero offset/fade and no warp. The
paired checksum is StateChecksum {frame:1856, hash:2294327793}; state_preserved
is true. Both final native/classic images were inspected; moving20 (Saffron
arrival/door clearance) and44 (return boarding text) were inspected. Those moving
frames are diagnostic, without matching classic frames. The final source/native
pair continues to expose Goldenrod's gray-vs-blue waiting-floor discrepancy;
Saffron's waiting area is blue in moving20/44. Full slide-out-door animation is
still unproven. Source-tail/destination-scene continuation is accepted for this
route, not whole-station art.

The final replay supervisor prunes129812779 bytes of unneeded intermediate
scratch while retaining32 selected/recent moving images, the full trace and
final paired evidence. No exporter, owned Cargo job or native capture remains
live; no task code is unsynced. The verified frozen source still retains its
recorded Pokemon scheduler exception, so full main integration remains open.
Shared packs, audio and unrelated assets were not changed or staged. The next
visual geometry task is the existing source-authenticated Tower side-wall strip,
while station floor/door fidelity and all original311/state/service/League/Hall
requirements remain active.


Battle Tower lobby side-course correction — 2026-10-17

The current before pair confirmed that the lower side courses were still flat
beside the full-height shell. Pinned source blocks18/16 contain left/right WALL
half-blocks; 1a/17 contain only lower WALL corner quadrants. Their upper halves
and neighboring inner quadrants are FLOOR. The lobby-only adapter authenticates
these drawings, replaces their source footprint with floor plus a connected
32px-high body, and folds the inner ornamental strip onto the lower inward face.
The outer strip's pale perspective pixels remain floor rather than wall paint.
Gold comes from source shade2 in BROWN tiles; corner cap tiles40/3b use RED, so
their lower2e/2d wall edge supplies the actual gold material. Corner ends join
the body. Each course shares its inward face's camera cutaway plane. The adapter
changes no support heights or gameplay collision.

All16 focused checks pass on the final source: the expanded real-pack wall/door
regression, service furniture, special Tower floor, bounded source signatures
and12 room-floor controls. The final test build passes6m20s, with the wall check
passing0.27s. It covers full height, clear adjacent/corner FLOOR, face winding,
complete source/ownership guards, unchanged support and rejection of the known
RED/floor paint donors. Native tools build successfully3m55s; all five launchers
have valid ARM64 Mach-O headers and the capture/audit executables actually run.

Six final native/2D pairs preserve state and were individually inspected:
front/east/west, both side approaches and the entrance. DDDUUU walks left(1,5)
via(1,8) and right(14,5) via(14,8), returning in192 frames each. DDUU leaves
through the actual south doorway to BattleTowerOutside and returns to(7,8) in
128 frames. All final walk records have matching runtime/terrain maps, active
terrain, no pending load, zero offset/fade and no warp. Moving02 on each side
was inspected for foot contact; these diagnostics have no paired2D counterpart.
The base lobby gallery is refreshed from the final front pair. This accepts
these wall courses and joins; whole-room and native challenge fidelity remain
open.

The final full311/225-state audit has zero mesh errors. Only BattleTower1F's
modeled-family/source-classification records change against the wood-band audit;
every reported footing count/min/max and every variant record is identical.
This is not a per-cell footing digest or whole-room acceptance. Evidence is
ignored under target/kanto-resume/tower-native-side-wall-review/final-*,
tower-side-course-material-full311-audit.json and tower-side-course-* receipts.
Fresh whole-file SHA256 values are recorded separately from the older pack
content identities, which hash the CBOR data/audio-manifest/runtime/report tuple.
No packs were written or published in this renderer correction.

The first compile was stopped before tests after source-pixel inspection found
floor pixels in the proposed vertical texture. Three initial after-* pairs
preserved state but were rejected for pink corner bodies. The first capture
command omitted --pack and failed before producing an image; its receipt is
retained separately. These failures are not passing evidence. Sixty redundant
moving diagnostics from three completed, rejected older Hall captures were
removed (123672409 bytes); their final pairs, full CSVs and representative
frames remain. No task-generated non-ignored scratch was added.

No owned build, capture or exporter remains live, and both changed renderer
files match the frozen verification tree. Its recorded Pokemon scheduler
exception remains, so full main integration is still open. The existing
BattleTowerBattleRoom gallery shows generic pale ornament in place of the
source's gray wall panels (blocks0b/10/1f/20). That room, native challenge
presentation, station floor/door fidelity and every original311/state/service/
League/Hall requirement remain active. No unrelated changes were staged.


Battle Tower battle-room native wall correction in progress — 2026-10-17

Fresh front/east/west pairs reproduce pale generic emblem walls and a framed
picture replacing the actual opponent staging entrance. The special-room path,
not the lobby adapter, owns these drawings. Its panel fingerprints remain;
the staging binding now has a specific TowerStagingDoor detail/label. Native
source GRAY supplies the connected32px bodies and exact gray panel faces. The
blue projected base folds onto the wall, leaving its FLOOR row open. The door
has clear space below a26px lintel, with source leaves behind the opponent's
starting center. The two actual south warps remain open. Source blue corner
caps belong on top. No collision, source ownership or footing data is changed.

The first native build passes4m18s. Three after-* pairs preserve state and show
the correct gray/blue material and panel drawing, but are rejected for coplanar
side ornament. The first full311/225-state audit has zero mesh errors, changes
only BattleTowerBattleRoom's family/geometry records, and retains every reported
footing count/min/max and every variant. The initial test compile was stopped
before tests to correct the observed flicker. Ornament now sits0.02px inward
of its backing; the staging approach also has exactly one native floor plane.
That native rebuild is running. The new real-pack regression covers gray pigment,
source mutation/fallback, panel/body height, clear rear FLOOR, staging/south
entrances, trim clearance and padded-grid placement. It has not run yet.

A separate external-pack controller regression is prepared in the existing
Kanto state tests: a legal three-starter level10 fixture enters through actual
reception choices, source save prompt and escort using ordinary buttons.
It can retain a native-compatible initial save for replay, without seeding
challenge flags, directly arming a script or replacing a battle result. That
new test has not run yet. Broader room and challenge acceptance remains open.
Evidence: tower-battle-native-wall-review/, tower-battle-wall-full311-audit.json
and tower-battle-wall-* receipts. Shared packs were not written.

The corrected wall regression passes0.26s, and26 related renderer controls pass.
The final full311/225 audit repeats the scoped family/geometry changes with no
errors and unchanged reported footing summaries/variants. Five final pairs
preserve state and were inspected: front/east/west, RRLL at(2,1) via(4,1), and
UD at(4,1) via the staging DOOR(4,0). Walks settle with matching runtime/terrain,
active terrain, no pending load, zero offset/fade and no warp. The staging walk's
moving00 was inspected for foot contact and the aperture; it has no paired2D
counterpart. The battle-room base gallery is refreshed for the wall correction.

The offset build initially exited0 after rust-objcopy reported disk-full and
left corrupt1840/1936-byte capture binaries. Header checks rejected both before
execution. Both launcher/cache copies were removed and a true9.87s relink
restored valid ARM64 Mach-O executables. Those repaired tools generated the
five final comparisons. This is the actual repair, not acceptance of the
build's earlier apparent success.

The new full reception regression fails before reaching battle (native5.04s,
bounded-trace rerun4.85s). Its first diagnostic accidentally included the huge
runtime/catalog snapshot; that line was removed and replaced with a bounded
cursor/menu trace. The trace proves the real defect: an empty core ActiveMenu
marker is auto-closed while the visible challenge cursor still exists, running
the source default cancel branch into Text_WeHopeToServeYouAgain before the
actual CHALLENGE response. The isolated Explanation/cancel tests bypass this
normal opening path and therefore did not catch it.

The pending fix adds Tower challenge/room ownership guards to
close_visible_noninteractive_runtime_surface in deterministic_session.rs.
VisibleShellController::snapshot projects those same production choices even
while the source textbox remains open; graphical and text labels share the
challenge option helper. No new dispatcher or authoritative cursor is added.
The legal-party regression now requires both menus, the original ifequal cursor
while choosing, unchanged state when cleanup declines the marker, actual saved
explanation state and the loaded canonical opponent. Its revised build is
running in target/kanto-resume (tower-battle-reception-owned-menu-tests.log).
The shared bevy_shell.rs snapshot hunk was synced separately, preserving the
recorded frozen Pokemon scheduler exception. Other goals were notified.
No native challenge replay has passed yet; full room/service acceptance stays
open. No pack was written or published.


Tower reception ownership and first native challenge entry verified — 2026-10-17

The menu-marker fix passes the complete legal-party reception regression on the
canonical native pack4.90s and TUI pack1.19s. The authoritative challenge cursor
stays on its original ifequal successor until the player chooses; cleanup
returns false with an identical state checksum. Controller snapshots expose
CHALLENGE and the actual level picker through the existing production cursors.
The source save replaces the initial unexplained fixture, and ordinary A input
runs the escort and loads the canonical opponent/battle. No script cursor,
challenge flag, victory result or mid-route state was seeded. Only the legal
initial three-starter level10 party replaces grinding. The retained native save
is tower-battle-native-controller-fixtures/BattleTower-reception-before-challenge.crystalsave.

All67 distinct focused tests pass:27 renderer checks, this full reception test
(on both packs),12 existing Tower controls,10 neutral controls, the separate
water-denial source control,14 reload controls, actual MCP movement/Start and
clean stderr. The latter two pass1.00s/1.52s. The initial broad neutral run on
the modded pack fails the water preview's explicit unchanged-slot/rate
prerequisite, not an input assertion. The required external vanilla-source
browser pack at ~/.local/share/geothite/export-workspace/content-packs/
core-modular.browser.crystalpack passes2.43s. That control is not reported as
passing on the modded pack. The known outside raw-art roof fixture remains
excluded from Tower gameplay controls and is not counted passing.

The final native build passes44.82s. Actual native replay then follows
BattleTower1F(7,7) -> Elevator(1,3) -> Hallway(11,1) -> BattleRoom(3,7), ending at
the first FIGHT move menu with the player(3,3). It exits0 in53.49s after1537
logged frames, eight grid origins and three additional terrain builds.
The final3D/2D pair was fully inspected and preserves state (frame426,
hash897025709, LCD2257), with CYNDAQUIL/CORSOLA modeled and fallback=None.
World terrain is intentionally inactive in the final battle view; runtime and
retained terrain map are BattleTowerBattleRoom, no pending load, offset/fade
zero and no warp. The ordinary A route stops only once the real move menu is
presented; it does not inject an encounter or resolve a battle result.

Moving26-29 were inspected: entry, the opponent traversing the staging opening,
the two trainers in position and the actual intro text. These moving frames have
no matched2D counterpart. Additional5/10/15/20 samples show the ordinary source
intro/entry text, not complete visual acceptance of every menu frame. The wall
scope and first entry route are accepted; all-seven-battle completion, complete
challenge state/service coverage and whole-room/UI fidelity remain open.

Two concrete visual limits remain: first-move-menu-{3d,2d}.png has cramped/
overlapping move-header/metadata text, reported to the competitive UI owner.
Moving26 has a Cyndaquil follower overlapping Chris and large relative to him;
the frozen verification build still omits the newer dimension publisher and
its dependent scheduling edge. The Pokemon owner confirms that this does not
verify the current publisher and that full integration readiness is unconfirmed.
Its dependencies/edge remain preserved in main. The next integration gate is
the actual current main publisher (surf_mount.rs), not acceptance of this old
follower pose. Every original311/state/service/League/Hall requirement remains
active. The two review documents retain that full scope.

The three canonical whole-file SHA256 values match the preceding checkpoint;
no packs/audio were changed, written, staged or published. No owned Cargo job,
capture or exporter remains live. All owned code matches the frozen source,
with only its previously recorded main scheduler exception. Source export
ownership remains released. Evidence is ignored under tower-native-reception-
entry-review/, tower-battle-native-wall-review/final-* and tower-battle-
reception-* receipts. The earlier failed full-flow tests, rejected coplanar
views and disk-full artifact failures remain documented as failures.


Current-main integration cache correction — 2026-10-17

The first actual-main native integration build returned101 with three missing
VisualPokemonDimensions errors in new_bark_actors.rs. Inspection confirmed the
resource, plugin registration and scheduler publisher already exist in main.
The owned target directory reused the frozen source tree's render-API artifact:
its dep-info uses relative crates/... paths, and main source timestamps predate
the cached frozen compilation. This was a stale artifact failure, not evidence
that the resource implementation was absent. No production code was removed
or patched to bypass it.

After resolving the current main workspace with cargo metadata, cargo clean
--profile wasm-dev -p was applied only to the fourteen game packages in the
owned target/kanto-resume cache, removing9350 files/3.3GiB. Other caches and
external dependencies were preserved. The fresh actual-main native build is
live as session74194 with receipt current-main-native-integration-fresh-build.log.
Do not reuse this cache for the frozen tree without invalidating its local
crates again. Current-main follower size, Tower replay, battle typography and
controller checks remain unverified until the fresh build actually runs.


Current-main native integration evidence — 2026-10-17

The fresh actual-main build completes in7m37s, including both dimension
publication scheduler dependencies. All five native executables have valid
Mach-O headers and were not accepted merely from Cargo status. The actual
main audit covers all311 prior interior maps and225 declared variants with
zero mesher errors. Every reported footing field and every variant matches
the frozen baseline; only IlexForest solid_vertices/solid_triangles differ.
The separate --kanto-interiors flag covers120 maps and was retained under
current-main-kanto120-audit, not misrepresented as the full311 report.
Receipts: receipts/current-main-full311-audit.{json,log} and
receipts/current-main-full311-diff.json.

The first actual-main ordinary Tower replay reaches battle but exits1 after
222.38s: CYNDAQUIL/KANGASKHAN cannot obtain an admitted 3D camera. The timeout
prints the inactive overworld reason, while a separate explicitly enabled
source-fallback UI diagnostic identifies the actual battle reason as
'no readable higher shot on checked ground'. Its native/2D pair preserves
state, but both images are source presentation, not 3D acceptance. Both were
inspected: current main separates TACKLE from /HIT and SMOKESC. from PRI; the
earlier overlapping move-menu text is corrected. Evidence is retained in
tower-current-main-reception-review/ and
tower-current-main-readiness-diagnostic/ (exit0,40.37s). The legal source
trainer selection differs between runs; do not treat these as identical
opponent encounters.

A third traced ordinary reception route selects MISDREAVUS and actually
produces modeled CYNDAQUIL/MISDREAVUS, source=[], fallback=None. Its pair
passes state preservation and both final PNGs were inspected (exit0,39.55s).
The source-fallback opt-in was present for diagnostics but unused in this
actual 3D pair, as shown by the presentation receipt and visible models.
Moving28 of the first main route shows a smaller, separated Cyndaquil
follower in the Hallway. This is an unpaired diagnostic, not complete gait
acceptance. Main therefore builds and runs its dimension publication path;
there is no justification for retaining the frozen scheduler exception as
an integration workaround. Complete main focused tests are still building
as session61554. Tower/Kangaskhan camera admission and the broader original
interior/League/Hall scope remain open. No source geometry, physical sizing,
or source ownership was weakened to make these captures pass.


Superseded traditional rear-rail diagnosis (withdrawn after collision check)

Three fresh actual-main same-state pairs of EcruteakItemfinderHouse(3,6),
hour12, rotations0/2/6 were captured and individually inspected. All preserve
state. The side views confirm the horizontal wood rail drawing lies on the
ground strip in front of the tall timber backing, behind the TV/radio.
Evidence: traditional-rear-rail-review/before-{front,east,west}-*.

The concrete cause is modeled_interiors.rs in the traditional_wood_wall::source
ground branch: it uses geometry.uv(p.column+2,p.row+2), which is native wall
tile0x50, as the complete floor UV. Retaining full UVs in the wood-band fix
makes that earlier mistaken donor visibly legible as rails on the floor.
traditional_wood_wall.rs currently builds generic timber from the upper wall
source sample and never folds those lower source rail rows onto the upright
face. The next correction must use the genuine ground donor under the wall,
move the authenticated lower rail drawing onto the connected32px wall face,
and preserve the separate radio, row-two seam, source/custom exclusions and
footing. No rail correction is implemented or accepted by these before views.


Current-main Tower wall reinspection and controlled walks

Five fresh main wall-view pairs were captured and individually inspected:
front/east/west at(4,4), rear-step-walk from(2,1), and staging-step-walk
from(4,1). All five preserve state. The ordinary held-input versions were
retained as additional diagnostics; they travel farther because each route
character holds a key for32 frames, and are not claimed as exact return walks.
The step-mode RRLL path is exactly(2,1)->(3,1)->(4,1)->(3,1)->(2,1),128frames.
The step-mode UD path crosses(4,0) and returns to(4,1) twice within64frames;
the CSV is retained rather than describing it as only one crossing. Both
final states have matching runtime/terrain BattleTowerBattleRoom, active
terrain, no pending load, zero offset/fade and no warp. The gray connected
walls/blue trim and clear rear lane/staging opening hold with the main actor
scheduler and physical dimensions. Evidence: tower-current-main-wall-review/.
The base BattleTowerBattleRoom gallery PNGs, comparison, identity and paired
receipt were refreshed from the main front capture. These are scoped wall
and approach checks, not complete Tower battle/camera acceptance.

Current-main controller/renderer verification so far has65 distinct passing
test names:32 ordinary actor controls plus all five separately run supplied
rig/painting controls, source Tower walls, special-room/native-Tower and pier
checks, real reception, core battle/menu and Mom gate, and the original Kanto
entry/victory/Lance-Hall/rooftop callbacks. MCP movement/menu and clean stderr
subsequently pass as well (two further tests). Zero-test measurement/follower/
Surf filters from the first default-feature test build are explicitly excluded;
the location-tester build is running as45841 to exercise those gated tests.
Receipts use current-main-* names. All three whole-file pack hashes match
their prior verified values, recorded in current-main-pack-hashes.json.

Two exterior-owner Rust files changed after the native executables were built:
new_bark_models.rs and battle_entry_progress.rs (exterior roof/gate composition).
They are preserved. Native screenshots verify the main compiled checkpoint at
08:34 local time, while the later test builds compile the newer shared source.
Do not claim the screenshots verify every subsequent concurrent exterior edit.


Current-main integration gate completed for the scoped verification

The location-tester build completes in4m59s. Its previously excluded filters
now actually execute and pass: two dimension-conversion tests, five follower
trail/seed/production-publication tests, and two real Surf-user tests. Together
with the other current-main receipts there are76 distinct passing test names,
deduplicated in receipts/current-main-passing-test-names.json. The three
zero-test default-feature filters remain excluded. Both MCP movement/menu
and the spawned-client Mom-dialogue clean-stderr regression pass. The native
verify_train_station executable also runs successfully: all10 JSON case
records pass, including two genuine mesh aperture/footing checks and eight
controller denial/travel/approach cases. This does not add new train animation
or floor-color acceptance. Station output is current-main-train-controller-review/
and receipts/current-main-train-controller.log. Own generated .bak files were
removed; packs/audio/source inputs remain unchanged.

The actual-main dimension publisher and ordering edge have therefore been
built, exercised in native 3D and checked in the feature-specific tests. Retire
the frozen scheduler exception as a verification workaround. The frozen tree
remains only historical evidence; do not rebuild it in this task's current
main Cargo cache. The latest feature-test executable is timestamped09:01:35
local and no Rust file is newer at this checkpoint. Native screenshots retain
the separately stated08:34 source checkpoint qualification for concurrent
exterior changes. No owned Cargo/capture/controller process remains live.

This completes the current-main integration gate for the recorded checks,
not the original full goal. Traditional rail/floor ownership, the larger
Tower/Kangaskhan camera, full-room/service/reachability coverage, full native
League continuity and Hall party/rotation/animation acceptance still require
work. All original scope is preserved.


Rejected traditional rear-rail implementation — withdrawn before native build

traditional_wood_wall.rs now folds authenticated tile0x50 onto the lower16px
of the existing32px upright wall. Its two repeated eight-pixel rows retain
full native UVs and shallow recessed gaps between horizontal laths. One
closed backing spans the complete fixture width, avoiding internal end faces
through those recessed courses; the upper timber field and other backing/
perimeter templates retain their existing behavior. The upper/lower seam
closes at the same front plane, and the separate radio is unchanged.
modeled_interiors.rs now uses the resolved genuine ground UV under the
fixture instead of reusing its lower rail drawing as floor.

The existing twelve-fixture/six-map regression additionally requires real
ground UVs/corners at the original datum, upright full-row rail UVs, actual
relief, ceiling height, preserved radio bounds, unchanged footing and the
existing source mutation/custom-profile exclusions. This is not yet accepted:
the joint main-feature test build is live as7048, followed by native/2D views
and a full ownership/footing audit. Two earlier narrow test commands were
intentionally stopped before running tests (exit130) because their different
feature unions began rebuilding graphics dependencies. Receipts are retained;
neither counts as a test result. Use the joint crystal-bevy/crystal-voxel-view
command with crystal-bevy/location-tester for this cache rather than manually
trying to reconstruct the full transitive production feature union.


Source semantics correction and actual station floor work — 2026-10-17

The proposed traditional rear rails were a mistaken interpretation. Before
accepting that mesh, the pinned original traditional_house_collision.asm was
read: block01 is WALL,WALL,RADIO,FLOOR; block08 is WALL,WALL,FLOOR,FLOOR.
The lower tile0x50 strip is genuinely walkable wooden floor, consistent with
house.rs::TRADITIONAL_HOUSE_FLOOR_TILE. The prior diagnosis and attempted
rail-wall solution above are explicitly withdrawn. Only this turn's unverified
production edits were reversed; traditional_wood_wall.rs was checked byte-for-
byte against its prior source before adding a preservation comment/regression.
No native executable or image was built with the rejected rail geometry.
The interrupted joint test build7048 exited130 and supplies no passing evidence.
The original flat floor UVs, wall height/seam, separate radio and footing remain.
The regression now requires the lower wood rows at their exact floor corners
with full tile0x50 UVs, including in the radio fixture, across six room maps.

A separate real discrepancy is confirmed in the paired station screenshots:
Goldenrod's waiting floor is gray although its source is blue/white. The
native service-floor map selector and station branch had included only Saffron;
Goldenrod fell through to generic Stone. Both stations now use the source
floor adapter. Its full live tile3d UVs retain the white inset and blue field
instead of reducing the drawing to two material swatches. A shallow bevel
stays below the unchanged walking datum. Rail/platform source tiles and train
geometry are untouched. The service-floor regression now covers both stations,
complete UV correspondence and the bevel datum, while retaining unknown-room
and malformed-source exclusions. Joint tests are live as80771. Before pairs
are station-native-floor-review/before-{goldenrod,saffron}-*. No after-view
or whole-station acceptance is claimed until the native review completes.


Station floor focused checks complete; native review pending

The correct joint feature build80771 completes successfully in8m19s. Its
service-floor regression passes; the zero-test Bevy package is excluded.
The resulting current-main viewer binary then passes all33 room-floor tests,
the two traditional wood wall/backing tests (including the new flat strip
UV/corner preservation assertions), and all five train scenery/headroom/source-
claim controls. There are40 distinct passing test names, no ignored cases in
these runs. Receipts: station-native-floor-{kanto_room_floor_tests,
traditional_wood_wall,train_station_scenery}.log and test-results.json.

Both freshly captured before pairs were inspected and preserve state. They
show Goldenrod's incorrect gray waiting floor and Saffron's blue field without
the small source white inset motif. The native executables are rebuilding as
36597 with receipt station-native-floor-build.log. After views, a normal floor
walk, native train-controller/mesh verification and the full311/225 ownership/
footing audit still need to run before this floor change is accepted.

Next full-room boundary review: the existing TinTower9F gallery shows rear
corner grating/voids between its rear statue wall and side courses. Its pinned
source first block row begins/ends with black padding01 beside grating0b;
these are WALL collision areas, unlike the traditional-house floor strip.
room_perimeter.rs repeats West/East/South courses but has no general north
join. Fresh current-main side views and complete source-outline validation
are still needed before deciding the correct stepped rear enclosure. Do not
fill internal grating or turn blank source padding into walkable floor.


Station floor source fidelity verified — 2026-10-17

The native build succeeds in4m02s. All five executable headers are valid;
render_at_location, audit_authored_assets and verify_train_station actually
run with the new floor code. Six after pairs were individually inspected and
preserve state: Goldenrod and Saffron front at(9,10), Goldenrod east, Saffron
west, Goldenrod DDUU and EcruteakItemfinderHouse RRLL. Both prior front pairs
were also inspected. Full native Goldenrod detail confirms blue field, white
insets, unchanged gray platform/rail and a clean ticket-gate seam. The pattern
continues beneath chairs and the blue cases. This accepts the floor in these
views, not full station/door-animation fidelity.

Goldenrod DDUU is exactly(9,10)->(9,11)->(9,12)->(9,11)->(9,10),128frames.
The retained wood-strip RRLL is(3,1)->(4,1)->(5,1)->(4,1)->(3,1),128frames.
Both finish with matching runtime/terrain maps, terrain active, no pending
load, zero offset/fade and no warp. Moving01 on each route was inspected for
foot contact; these moving images are unpaired diagnostics, not complete gait
acceptance. The wood-strip walk directly confirms the withdrawn rail diagnosis
would have turned a real walking surface into a wall. The original geometry
remains and its full UV/corner floor-preservation regression passes.

All40 focused tests and all10 train verifier case records pass. The full
311-map/225-state actual-main audit has zero errors. Against the prior main
audit, only GoldenrodMagnetTrainStation solid_vertices/solid_triangles change.
All reported footing fields and every variant remain identical. This report
does not measure the Saffron UV change; its full source motif is proven by
unit UV correspondence and inspected paired renders. Report/diff receipts are
station-native-floor-full311-audit.json and station-native-floor-audit-diff.json.
Both station base gallery PNGs/comparisons/identity/pair receipts are refreshed.
All three canonical whole-file pack hashes remain unchanged; no audio/pack
publication occurred. Own generated controller .bak files were removed.

Three fresh main TinTower9F before pairs at(10,9),hour12, rotations0/2/6
were individually inspected and preserve state (6.20/6.04/6.08s). They confirm
the still-open rear corner joins seen from east and west; no Tin geometry was
changed in this phase. The source first row combines black padding01 and
grating0b outside its narrower rear statue course. Match the complete source
outline before adding a join; preserve internal grating, item/stair warps,
column movement and source/custom ownership. Evidence is
tin9-north-join-review/before-*. The larger Tower/Kangaskhan camera, continuous
native League and Hall party/rotation/animation and the full original room/
service/reachability review remain active. No owned process is live.


Tin outer north joins implemented, pending verification — 2026-10-17

The new tin_north_outline.rs authenticates the complete first four source
rows (160 cells) for all nine20x18-core-tile floors. Exact compact hashes
include block/art/subtile identity and blank padding. It distinguishes the
existing folded timber/guardian courses from missing outer boundary sections.
Only2F/3F/4F/5F/7F/9F have gaps;1F/6F/8F retain their old geometry. Missing
caps stand at the outer north edge, behind the existing grating or walkway;
short returns meet the back of existing16px folded courses at13px, avoiding
coplanar overlap with the native front/top. Side courses continue to that
outer boundary only where these authenticated gaps exist. Internal maze
walls, grating, jumps and stair/teleport openings retain their source owners.

room_perimeter.rs gains a North copy orientation and shares the existing
cutaway/camera-range registration between old/new sections. build_scene.rs
only passes its existing profiles argument to that perimeter call. Custom
source matches and old footprints of edited canonical objects suppress the
new inference. The real-pack perimeter regression is expanded for all nine
layouts, every first-row art mutation, cropped/translated source grids,
unknown maps/dimensions, custom ownership, continuous north spans/returns,
unchanged source claims/support, and retained animated-pillar buffers. The
existing real nine-floor wall/ladder/animation controls will also be rerun.

Joint tests are live as27299, receipt tin-north-join-tests.log. Native before
front/east/west pairs for2F/3F/4F/5F/7F are captured under tin-north-join-review/;
9F's three prior main pairs are under tin9-north-join-review/. The new before
front views were inspected:2F has outer corner gaps,3F/4F have rear grating
gaps,5F is missing its outer north boundary behind a real walking lane, and
7F has a partial rear course beside the lane. All these need after-view review
before acceptance. No native executable has yet been rebuilt with the joins.


Further pier ramp refinement (eight walking tiles), verification pending — 2026-10-17

The renewed slower-ramp request extends SHORE_APPROACH_CELLS from12 to16,
96 to128 native pixels, six to eight walking tiles. The ordinary shore grade
is25% shallower. Shared graph propagation and height interpolation continue
to drive both geometry and actor support. The raised-paving regression was
extended to20 source columns to check shore, halfway, endpoint, shared edges
and support at the full new distance. Joint focused tests are queued behind
the existing Tin boundary test in the same owned cache; no competing exporter
or content-pack edits are involved. Native review is pending.

Tin verification correction: the first external_room_wall_donors run failed
on BattleTowerBattleRoom before reaching Tin. Its native source-art trim is
intentionally0.02px inward from structural backing to avoid flicker. The
shared boundary assertion now identifies varying-UV source triangles and
permits only that existing relief on BattleTowerBattleRoom, retaining strict
body and other-room bounds. The rerun is tin-north-join-boundary-tests.log;
no Tin pass or native acceptance is claimed yet.

Pier progress: the freshly compiled viewer test binary passes all six
elevated_pier checks (piers-eight-tile-direct-tests.log), including the full
new16-cell raised-paving fixture and external11-map support/ownership test.
The queued joint run and native build remain live. The Tin boundary rerun
now reaches Tin, failing an old west-wall assertion on a new short internal
return (256px versus32px). No Tin acceptance is claimed; that assertion and
its side-coverage selection still need refinement.


Eight-tile pier ramps verified — 2026-10-17

The six focused elevated_pier checks pass in the freshly compiled viewer
binary (piers-eight-tile-direct-tests.log). The joint Cargo module run also
passes all three module tests (piers-eight-tile-tests.log). The optimized
native build passes (piers-eight-tile-build.log), and all five native tool
headers are valid. The production render_at_location executable was exercised
for three fresh same-state native/2D pairs: Route25 (47,9), Route12 (5,34),
and Route32 (13,60), hour 12, with the previous review cameras. Each comparison
was individually inspected. The longer grade stays joined to raised paving,
through the bend and to shore. Both moving-01 frames were inspected for foot
contact; they are temporal diagnostics, not paired 2D comparisons.

Route12 DDUU walks (5,34) to (5,36) and back. Route32 UUDD walks (13,60) to
(13,58) and back. Each records 128 frames and ends with zero presentation
offset, matching runtime/terrain maps, active terrain, no pending terrain,
zero fade and no warp. All three paired receipts preserve state. Evidence
and checks.json are ignored under target/kanto-resume/piers-eight-tile-review/.
The unchanged native pack SHA256 is recorded in
receipts/piers-eight-tile-pack-hash.json. No pack was rebuilt or modified.

This accepts the sixteen-source-cell/eight-walking-tile ordinary shore grade
(25% shallower than the preceding twelve-cell version) and its tested joins.
It does not accept unrelated scenery or the whole outstanding interior goal.
The build also includes the pending Tin north implementation, but Tin after
views have not been reviewed and its old side-wall test selection still
needs correction for internal returns. No owned build or capture remains live.


Tin outer north joins verified — 2026-10-17

The preceding goal turn was progress: it implemented and verified the renewed
eight-tile pier ramp request. This turn corrects the perimeter regression's
selection of physical outer side walls and completes the pending Tin join
verification. Both were concrete progress; no blocker status is warranted.
The deleted handoff remains deleted; these two canonical review documents
continue to carry the complete scope and evidence.

The old Tin assertion matched every west-facing plane, including new short
internal returns. It now identifies each outer side by its complete plane,
requires an actual wall at that source padding edge, and checks uninterrupted
coverage from north zero on affected floors or the existing 13px rear backing
on unchanged floors, through the south corner. Separate assertions still
check every short return meets native backing at 13px, all north spans are
continuous, source/grating/floor claims and footing are unchanged, and moving
pillar buffers are retained. Both outer edges are now checked explicitly.
The earlier Battle Tower relief repair remains restricted to varying-UV
source-art trim, with strict wall-body bounds; its dedicated native wall
regression passes too. The first failures are retained as diagnostic history.

The joint room_perimeter suite passes six tests after an 8m22s build
(receipts/tin-north-join-outer-plane-tests.log). Thirteen additional distinct
controls pass: three tin_tower, four sparse_pillar, three sprout_pillar, one
native Battle Tower wall and two tower_timber tests. Total: 19 unique tests.
Control receipts and exact results are tin-north-join-control-*. The native
executable is the successful piers-eight-tile build, which already includes
this Tin production geometry; this turn changes only test selection. No new
production mutation was made after that build.

Eighteen after pairs were individually inspected: front/east/west on floors
2/3/4/5/7/9, all at (10,9), noon, pitch 45. Missing outer boundaries now meet
the side/guardian courses. The real horizontal grating remains horizontal;
internal maze walls, ladders, floor lanes and teleport pads retain ownership.
Three front controls on floors 1/6/8 retain their existing outline. Two further
paired walks were inspected: Tin5 (3,0) RRLL to (5,0) and back, and Tin9
(11,9) UUDD to (11,7) and back. Each records 128 frames and ends at zero
presentation offset, active matching terrain/runtime map, no pending terrain,
zero fade and no warp. All 23 paired-state receipts preserve state.

Both moving-01 diagnostics were inspected. Tin9 keeps visible foot contact;
Tin5's middle position is hidden by an existing internal timber panel, with
only Chris's hat visible. The lane is physically clear and traversal returns
correctly, but this is an open player-readability defect. Do not claim full
Tin5 or whole-room visual acceptance. The separate column-temporal 3d-start/
3d-end images were inspected and visibly change the source-driven pillar pose.
They deliberately advance time and are not frozen-state 2D evidence.

The explicit complete 311-map/225-state audit exits zero with no mesh errors.
Every reported map record and every script variant matches the preceding
station-native-floor-full311-audit.json exactly. This audit omits textured
vertex counts, so the identical records do not mean the added walls are
absent; the geometry assertions and native views are the direct evidence.
Six base Tin gallery entries were refreshed from reviewed front pairs, with
matching images, identities and paired-state receipts. All three shared pack
SHA256 values remain unchanged (tin-north-join-pack-hashes.json). No exporter
or pack publication ran. No owned build or capture remains live.

This closes the scoped missing outer north joins on the six affected floors.
The new Tin5 internal-panel visibility defect, larger Tower battle camera,
requested 3D train slide-out doors, continuous earned native League, Hall
party variants/rotation/animation and the original complete interior/state/
service/reachability fidelity remain active. Full-room acceptance is not
inferred from the audit or gallery count.


Tin timber reveal implementation, pending verification — 2026-10-17

The previous goal turn was progress: completed scoped north-wall verification,
corrected the physical side-plane test selection, refreshed six Tin gallery
entries and recorded the new north-lane visibility defect. The current source
trace finds no cutaway registration in Form::TowerTimber or its model append.
The panel at source column8/row2 in Tin5 is exactly the body in front of the
(4,0) walking position. The existing common occluder system already supports
player-ray-driven whole-object reveal; this body simply never opted in.

modeled_dungeons.rs now records a body-only textured range immediately after
appending each TowerTimber model. Floor caps are emitted before that range.
Neighboring panels remain distinct. This does not change their geometry,
source UVs, support, collision, authored ownership or outer camera-wall planes.
The existing real nine-floor regression adds exact Tin5-panel coverage and
checks every timber body produces one actual prepared reveal group, retaining
all floor triangles in the opaque static mesh and all footing values. No
common reveal API or renderer cursor/gameplay state is modified.

Joint tests are live as37260 (tin-panel-reveal-tests.log), native build queued
as41923 (tin-panel-reveal-build.log), solely in target/kanto-resume. Native
before front/east/west pairs at Tin5 (4,0), noon, pitch45 are captured and
individually inspected under tin-panel-reveal-review/. The front reproduces
hat-only visibility; both side views show the whole player. These pairs use
the prior verified native build. Native after checks are pending; no visual
acceptance or full-goal completion is claimed. No exporter/pack write ran.

Tin reveal test progress: the joint nine-floor regression passes after an
8m04s build. Nineteen additional distinct controls pass: six room_perimeter,
four sparse_pillar, three sprout_pillar, two tower_timber, and four source
partition/fade restore/camera-orbit/current-camera ECS checks. Total20 unique
checks. Logs are tin-panel-reveal-control-* and the results JSON. Native
build41923 remains live; no after-view acceptance is claimed yet.


Tin timber reveal verified — 2026-10-17

The joint actual nine-floor test passes after8m04s; nineteen additional
focused checks pass, total20 distinct. They cover source panel preparation,
static opaque floors, exact Tin5 column8/row2 panel, source/custom ownership,
perimeter joins, timber palette/geometry, sparse pillar motion, independent
source partitioning, fade restoration, near-wall orbit and current-camera/
root ECS behavior. The native build passes (tin-panel-reveal-build.log,
11m07s wall time including waiting for the owned cache). All five executable
headers are valid, and render_at_location/audit_authored_assets were executed.
No common fade-system code, audio, gameplay or pack publication changed.

Three fresh before pairs at Tin5 (4,0) confirm the initial problem: only the
hat survives the front occlusion, while side views show Chris. Nine after
pairs were individually inspected: the same three views, Tin5 (3,0) RRLL,
Tin5 wide (10,9), Sprout3 (11,8), EcruteakGym (5,9), CianwoodGym (5,9), and
BurnedTower1F (9,9), all noon/pitch45. Every after pair preserves state; all
three before receipts do too. The front and moving-01 now show Chris's whole
body and feet through the blocking panel. Side views retain its opaque body;
the wide view retains unrelated opaque timber. The existing roof-height
outline remains intact. A fresh separate Tin9 temporal native pair was
inspected and still changes pillar pose; it intentionally advances time and
is not a same-state 2D comparison.

The north-lane RRLL route is exactly (3,0)->(4,0)->(5,0)->(4,0)->(3,0),
128 frames. It ends with zero presentation offset, active matching terrain/
runtime maps, no pending terrain, zero fade and no warp. Both primary player
and follower are visible in the inspected moving diagnostic. The actual
body range excludes every floor cap; reveal changes only visibility, not
walking clearance, source art, UVs, wall geometry, support or authored claims.

The explicit full311/225 audit exits0 with no errors. Every reported map
record and script variant matches tin-north-join-full311-audit.json exactly.
These coarse counters do not measure reveal groups; the real preparation test
and native occlusion views prove the change. All three canonical pack SHA256
values remain unchanged, recorded in tin-panel-reveal-pack-hashes.json. Five
base gallery entries were refreshed from the reviewed wide/control pairs,
including matching image, identity and paired-state receipts. No owned build
or capture remains live. This goal turn is concrete progress.

Further evidence: the gym control views show black backing slots behind the
north statues in EcruteakGym and CianwoodGym. The current
cianwood_walls.rs::guardian_wall_bounds guard explicitly allows only Tin and
Burned, despite append_cianwood_walls handling both gyms. This requires a
source-checked backing extension, without stealing guardian/floor ownership,
and native side/front review. It was not introduced or fixed by the body-reveal
patch and prevents whole-room acceptance. Sprout/Burned controls likewise
accept only the retained timber/reveal behavior, not every room detail.

The original full interior and shared furniture scope, all state/service/
reachability requirements, larger Tower battle camera, requested 3D train
door animation, continuous earned native League and Hall party variants/
rotation/animation remain active. No completion claim is made for that scope.


Cianwood/Ecruteak guardian backing implementation — 2026-10-17

The preceding goal turn was concrete progress: source-owned timber panels now
register body-only player reveal, with20 passing checks and nine inspected
after pairs. This turn tackles the separate guardian slots found in its controls.
Pinned source map constants identify both gyms as5x9 blocks. Their complete
north block row is the same, and both full guardian half-drawings match the
existing helper exactly. The source floor route (4,2)..(6,2) is clear of
trainer sight, objects and warp events; north-stage RRLL checks are planned.

Only the production map restriction in guardian_wall_bounds changes: it now
allows both gyms alongside Tin/Burned. The full eight-cell guardian drawing
still authenticates the extension, and no source claims or footing are added.
The existing guardian test now covers both gyms as well as all nine Tin floors
and Burned, every guardian cell/identity/art mutation, clipped drawings and an
unknown map. A new external-pack test builds the actual source north courses
and both guardian backings, verifies contiguous upper-body spans across the
full160px room width, keeps all statue cells unclaimed and support unchanged.
It includes the beveled crown rather than mistaking its inset top face for
a full-width structural gap. Existing incomplete-field tests remain strict.

Own joint cianwood_wall_tests suite is live as53605, native build queued as8103,
receipts gym-guardian-backing-{tests,build}.log in target/kanto-resume/. Six
before paired front/east/west views at (5,2), noon/pitch45 were captured and
individually inspected under gym-guardian-backing-review/. Both north statue
slots are visible. Native after review remains pending; no full-room or full
goal completion claim. No exporter/pack publication ran.

Guardian backing test progress: all five cianwood_wall_tests pass after an
8m14s joint build. Seventeen additional distinct controls pass: six perimeter,
one real nine-floor Tin, four sparse pillar, three Sprout pillar, two native
timber and one actual Burned Tower perimeter/fall-opening test. Total22 unique
checks. The native build8103 remains live; after images are still pending.
The altered body widths are not measured by the coarse audit vertex counters;
the actual rear-span test and forthcoming native views are the direct proof.


Cianwood/Ecruteak guardian backing verified — 2026-10-17

This goal turn is concrete progress. The existing source-authenticated backing
helper now includes CianwoodGym and EcruteakGym, closing the two missing statue
slots in each room. The pinned first block rows and complete guardian drawings
match the established Tin/Burned cases. Only the production map eligibility
changes; the source matrix, ceiling height, depth, paint and support semantics
are retained. Statue source cells remain unclaimed by the widened wall.

All five cianwood_wall_tests pass after8m14s. Seventeen additional distinct
controls pass, total22: six room_perimeter, one actual nine-floor Tin, four
sparse pillar, three Sprout pillar, two native timber and one actual Burned
perimeter/fall-opening test. Exact receipts are gym-guardian-backing-tests.log
and gym-guardian-backing-control-*. The new actual-pack geometry test assembles
native rear courses plus both guardian backings and checks contiguous upper
wall spans across all160px, excluding floor triangles. It retains statue
ownership and all sentinel footing values. Negative tests mutate every guardian
cell's identity/art, reject clipped drawings and unknown maps, and preserve
incomplete source fields. The native build passes (12m12s elapsed including
waiting for the owned cache); all five executable headers are valid.

Six before pairs were captured and individually inspected, reproducing the
slots from front/east/west at(5,2). Thirteen after pairs were individually
inspected, all noon/pitch45: each gym's front/east/west at(5,2), RRLL stage
walk from(4,2), wide at(5,9), north-facing at(3,2), plus Sprout3 at(11,8).
They show connected gold timber behind the statues from both sides and the
front, with the stage, source puzzle field and other wall courses retained.
The two north-facing checks also confirm the joined outside silhouette;
this is not a claim that every camera/actor occlusion in the rooms is solved.
Every paired-state receipt preserves state (19 total including the six before).

Each RRLL route records exactly128 frames and coordinates
(4,2)->(5,2)->(6,2)->(5,2)->(4,2), then settles with zero presentation offset,
active matching terrain/runtime map, no pending terrain, zero fade and no warp.
Both moving-01 diagnostics were inspected for visible player foot contact.
These are local stage traversals, not proof of complete gym reachability or
Strength/invisible-path puzzle completion. The separate Tin9 native temporal
comparison was inspected and still changes the column pose; it advances time
and is not frozen-state2D evidence.

The explicit full311/225 audit exits zero with no errors. Every reported map
record and script variant matches the prior tin-panel-reveal audit exactly.
The audit counts vertices but does not record the altered backing widths;
identical coarse records do not mean identical geometry. The actual span test
and native images prove the change. Both gym base gallery entries were
refreshed from the reviewed wide pairs with matching identities/receipts.
All three shared pack SHA256 values remain unchanged, recorded in
gym-guardian-backing-pack-hashes.json. No exporter or pack publication ran.
All owned builds/captures finished.

This accepts the source-owned north statue backing in these two gyms. Whole
room fidelity, remaining service/state/reachability review across the original
scope, larger Tower battle framing, requested 3D train door animation,
continuous earned native League and Hall party variants/rotation/animation
remain active. The deleted handoff remains deleted; these canonical notes
continue to carry the original scope and verified progress.

Train plug/slide doors implemented, verification pending — 2026-10-17

The native train model now separates authenticated door leaves from its static
body. Each leaf plugs outward before sliding sideways; closing reverses that
path. The renderer observes the published station officer route and does not
change source movement, controller timing, travel presentation or audio. The
selected-door flag travels through native preparation and worker serialization,
so a custom override that rejects the train cannot independently add doors.
Sparse pillar/flora refreshes retain selected doors in the animated domain.
New checks cover component partition, rigid motion and interpolated officer
coordinates in both stations, including cropped/scaled frames and unknown
actors. The focused joint test build is running; native animation, worker
roundtrips and movement/cached-mesh checks remain pending. No visual acceptance
is claimed. Evidence will be under target/kanto-resume/train-doors-review/ and
receipts/train-doors-*. The full interior objective remains active.


Player-home back wall correction implemented, verification pending — 2026-10-17

The user reports the player-house back wall as a regression. Fresh before
captures confirm mismatched wall treatments: repeated bitmap grain on exposed
courses, enlarged bright grain marks behind fixtures, and separate finishes on
returns/recesses. A shared native_home_timber producer now gives both player
home floors a quiet physical plank surface using the live source wood pigment,
small bevels and subtle grain. The existing source guards still authenticate
backing, exposed walls, side returns and the stair recess; other homes retain
their existing finish. The rear course no longer adds its separate raised cap.
Source footprints, furniture, openings, authored labels and support are not
changed. A new geometry check requires one complete projected front, outward
winding, closed bounds and no bitmap-grain UV repetition. Existing backing
checks retain coverage, source/custom guards and unchanged footing. Native
build/test and fresh after front/east/west/movement views remain required; no
visual acceptance is claimed. User wall correction takes priority over the
pending train-door and rounded-furniture acceptance work.

Nine furniture source models were regenerated from the shared interior author:
stool, arcade_stool, link_seat, cushion and five bed variants. Their complete
bounds/material identities remain exact. Interior exporters now preserve
Blender corner normals; the Blender regression passes smooth sides, flat caps,
hard corners and inverse-transpose scaling. The 68-asset validator passes.
Feathery bed output was held back because its bounds changed. The first train
build passed eight distinct source/geometry checks. The next joint test build
failed in concurrent render_quality SMAA tests requiring Debug; the owning
source now uses plain assertions. Renderer/worker checks need rerunning.
Evidence: target/kanto-resume/rounded-interior-review/, train-doors-review/
and receipts/rounded-interior-*, interior-corner-normals-*, train-doors-*.
The earlier native build33372 and queued home test1086 were deliberately
stopped (exit130) because they predated the wall correction. The corrected
native build97123 failed on a missing architecture_box import, corrected in
native_home_timber.rs. Retry72771 is live, logged in
player-home-timber-native-build-retry.log. It must finish before after captures
and a renewed joint home test run.


Player-home contact refinement pending — 2026-10-17

The corrected wall's first native build passed (9m18s). Front, east, west,
bedroom and a 128-frame RRLL return walk were inspected against matched 2D.
The quieter finish is continuous; the walk settles at(4,6) with active matching
terrain, no offset, pending load, fade or warp. The first front capture drifted
to yaw1 and was replaced by after-front-exact at yaw0. The eye fixture at(4,4)
coincided with the neighbor NPC and is excluded; after-eye-clear at(2,4) was
inspected and is usable. Every retained pair preserves source state.

Close moving-frame review caught the new front bevel receding at its outer
floor boundary. native_home_timber now fades the recess to zero on the whole
front rectangle's outer boundary, matching the remaining five body faces
exactly. The geometry regression now checks floor/crown/end contacts explicitly.
This second refinement is not yet accepted: build2617 is live in
receipts/player-home-timber-contact-native-build.log; the old home test61440
was deliberately stopped(exit130) because it predates the contact correction.
Final native captures, renewed home/house/perimeter/model tests and the audit
remain required. Train doors and rounded-furniture acceptance remain pending.


Player-home floor contact correction pending — 2026-10-17

The recessed upper backing is at1.8 source rows while the floor begins at2.
It now has a one-source-pixel timber foot reaching the actual floor boundary.
The upper body starts above that foot and remains behind appliances/trim;
projected front area remains exactly width×ceiling height. This complements
the flush outer-boundary bevel fix. Existing backing checks now explicitly
require the foot at source row2. Build2617 was stopped(exit130) because it
predated this additional contact correction. Native build45562 is live in
receipts/player-home-timber-foot-native-build.log. Earlier rendered wall views
are intermediate evidence, not final acceptance; renewed renders and tests
are still required. No pack, source footprint, support, collision or script
changes were made.


Player-house back wall verified — 2026-10-17

Both player-house floors now use one physical timber finish across the rear
backing, exposed plain panels, side returns and stair recess. Live source
pigment supplies the wood color; repeated bitmap grain and stretched bright
marks no longer fight between sections. Shallow plank bevels meet the outer
body boundary exactly. A low continuous timber foot reaches the actual floor
boundary while upper backing stays recessed behind fixtures. The separate
raised rear cap is removed. Existing source/custom guards, openings, furniture
plots and walking support remain intact.

The final production native build passes (8m32s). Six final paired views were
individually inspected: downstairs front/east/west, bedroom, eye level from
clear floor at (2,4), and the downstairs return walk. All preserve paired state
and the requested camera pose. A final moving frame was also inspected. The
128-frame walk reaches (4,6)->(5,6)->(6,6)->(5,6)->(4,6), with one terrain build,
active matching terrain, zero final offset and no pending load, fade or warp.
The dark strip seen during the intermediate contact review is covered by the
joined timber foot in the final views. The earlier eye fixture at (4,4) sat on
the neighbor NPC and is excluded; it was not a player-visibility defect.

The corrected home suite passes 11 renderer checks plus one controller Hall
save/Continue check; the two ignored continuous-League checks are not passes.
Together with house/perimeter/model/animation/train controls, 202 distinct
checks pass. The initial backing assertion incorrectly included the fixture's
existing floor UVs; its test-only correction now checks appended wall UVs and
also verifies the original floor positions/UVs/indices remain unchanged.
Production wall code did not change after the final native build/captures.

The full 311-map/225-state audit has zero mesher errors. Family/source ownership,
modeled/unmodeled classifications, jump samples and reported footing summaries
are unchanged, and all 225 variant records remain identical. Only solid geometry
counts differ on seven maps using the newly rounded shared furniture. Audit
counts do not prove wall shape; explicit boundary checks and native views do.
Both house gallery entries are refreshed. All three pack hashes are unchanged.
Evidence: target/kanto-resume/player-home-timber-review/final-* and
receipts/player-home-timber-foot-*. This accepts the back-wall/contact
correction, not whole-house or full-interior fidelity. First-person ceiling/
roof presentation and the remaining furniture/scenery review remain open.

Parallel goal progress retained: Blender corner-normal and 68-model validation
pass; nine furniture assets have denser curves with unchanged bounds/materials.
Feathery-bed fitting still needs review before publishing its new output.
Train native source/model/animation controls and all 10 controller/mesh verifier
cases pass. Both idle station pairs were inspected and show closed doors. A
2240-frame native A-button fixture replay goes Goldenrod->train LCD->Saffron,
ends at(9,10) with settled matching terrain and a state-preserved final pair.
Moving frames 03/04/18/19 were inspected as temporal diagnostics, including
arrival door opening; those frames have no same-state 2D partners. Reciprocal
native animation/precise partial poses and worker metadata remain unaccepted.
The first worker probe matched 0 tests because terrain-worker was disabled;
it is not a pass. Feature-enabled CPU worker test build is live as session 26788, logged in
receipts/train-doors-worker-feature-tests.log. Continue with that exact build,
then inspect actual one-test/roundtrip results, including both station flags
through borrowed/owned and paged restore paths.

The original all311/original16 room/state/service/reachability acceptance,
larger Battle Tower camera work, continuous earned native League and Hall party
variants/rotation/animation remain active. No exporter or pack publication ran.



Rounded furniture visual review and train transport verification — 2026-10-17

Fresh paired native captures inspect Earl's Academy at (1,9), elevated and
at eye level, plus Kurt's house elevated and from the side at (10,4). Rounded
classroom seat corners and tubular supports remain readable, with seats facing
their desks. The clear-aisle UUDD walk spans 128 frames and positions
(1,9)->(1,8)->(1,7)->(1,8)->(1,9), ending with matching active terrain,
zero offset and no pending load/fade/warp. Its moving01 frame was inspected.
The cushion side view confirms the padded edge and clear table relationship.
The initial north-facing cushion close view has a follower over its center;
it does not establish the top finish. All six paired captures preserve state.
These accept the selected chair/cushion silhouettes, not whole-room fidelity:
Kurt's visible timber joins and first-person open ceilings still need review.
Evidence: target/kanto-resume/rounded-furniture-review/, review-state-checks.json.

check-room-furniture-bindings.py now checks native fitted extents and transformed
normals for the shared curved seats, floor cushion and six upholstered beds,
in addition to its ship furniture/source checks. It passes. The feathery-bed
candidate retains material roles and the exact fitted extent at three scales;
its slightly wider raw authored bounds do not alter the native plot because
these models use independent-axis fitting. The candidate is now published.
All 68 model checks pass (114560 triangles,188531 exported vertices), and the
expanded fitting check passes. A new runtime regression requires both curved
corner normals and hard faces, finite unit fitted normals and native extents
at three scales on all ten refreshed furniture models. Runtime suite75237 is
live in receipts/rounded-furniture-ten-runtime-tests.log; it and a fresh native
build/bed capture remain required before accepting the new feathery model.
The earlier production binary predates that newly published model.

The feature-enabled CPU worker test finished successfully: exactly one viewer
test passes, exercising six external maps including both train stations.
Borrowed/owned response bytes and direct/restored/paged prepared outputs match;
the selected train-door flag is required on both stations. The Bevy invocation
matches zero tests and is not additional coverage. Evidence:
receipts/train-doors-worker-feature-tests.log (17m09s build,4.26s test).
This proves CPU worker transport; no browser-worker runtime claim is made.

Both native travel directions now run for2240 frames each: Goldenrod->travel
LCD->Saffron and Saffron->travel LCD->Goldenrod, ending at(9,10). Final native
and source images from both rides were individually inspected and preserve
paired state. Both traces end with matching active terrain,zero offset and
no pending load/fade/warp. The trace's map field is empty during the LCD travel
scene; its(6,5) entry must not be described as an active station platform pose.
Evidence: train-doors-review/boarding-native-*,reciprocal-native-*,
reciprocal-state-checks.json. Exact partial plug/slide/closing animation poses
still lack native visual acceptance; existing32-frame captures are too sparse
for that claim. Broad train fidelity and full311/original16 acceptance remain
open. No pack/export work or unrelated ownership changes were made.

Additional round-stool review — 2026-10-17

Celadon Game Corner and Elm's Lab now have inspected elevated and unobscured
close native/source pairs. The arcade side view at(9,7) clearly shows rounded
padded rims and round legs; the lab view at(9,2) shows the round timber stool
beside the table with floor contact and no table overlap. The first arcade eye
view has a follower on the nearest stool; the first lab eye view misses the
stool. Those two are excluded from close-view acceptance and replaced by
celadon-arcade-eye-side and lab-stools-eye-clear. All pairs preserve source
state; their exact cameras are in rounded-furniture-review/review-state-checks.json.
This accepts the selected stool geometry, not arcade machine/whole-room fidelity.
The newly published feathery bed is not in these pre-rebuild stool captures.

Kurt timber-join inspection and bed rebuild queue — 2026-10-17

Two additional matched native views of Kurt's house, rotated east/west at(9,5),
were individually inspected. Both preserve source state. The rear backing meets
the side returns; the large south-side opening corresponds to the entrance.
The thin dark stripes are sampled timber-joint paint on the closed wall fronts,
not missing vertical wall panels. traditional_wood_wall.rs builds a five-face
closed backing and fills its front with contiguous patches; its outer front
boundary stays at the body plane. No speculative wall relocation or closure was
made. The timber finish may still need aesthetic refinement; these diagnostic
views do not accept whole-room materials. Evidence:
rounded-furniture-review/kurt-wall-{east,west}-compare.png.

The ten-model runtime test build remains live as75237. Native build33604 is
queued behind it in the same owned Cargo cache, building render_at_location and
audit_authored_assets. It is logged in receipts/rounded-furniture-ten-native-build.log;
its current output explicitly reports waiting for the build-directory lock.
Do not start a duplicate build or treat the queue as failure. After both finish,
inspect actual runtime results, render the refreshed feathery bed with paired
state at elevated/eye level and walking, then run the full311/225-state audit.
The last completed native examples and audit predate this newly published bed.
All original interior/service/state/Hall/League gates remain in scope.

Ten-model runtime checks passed — 2026-10-17

Runtime build75237 completed successfully in11m11s. All55 selected viewer
interior_models tests pass, including rounded_furniture_keeps_curved_corner_normals_hard_faces_and_native_extents
and the native home-fixture material-role/ownership regression covering the new
feathery bed. The Bevy interior_models filter matches zero tests; it adds no
coverage. Native build33604 has acquired the Cargo lock and is now compiling;
it is no longer merely queued. Production visual acceptance and the renewed
full311 audit remain pending on that build. No furniture or wall implementation changed after the test
build began. The full interior goal remains active.

Celadon sixth-floor callback control — 2026-10-17

The freshly built Bevy libtest runs the external-pack
kanto_department_store_sixth_floor_controller_applies_rooftop_stair_callback
regression successfully (one test,1.50s). It verifies the authored rooftop-stair
replacement in authoritative layout/collision and production Start input after
the callback. It does not prove vending purchases or whole-room visual fidelity.
Evidence: receipts/rounded-furniture-celadon-callback-control.log. Native
build33604 remains the sole active renderer build; its live compiler has been
confirmed and no duplicate build was started.

Rounded feathery bed verified in production — 2026-10-17

Native build33604 completes successfully in6m35s. Fresh paired production views
inspect the default decorated PlayersHouse2F bed from the front/east, at eye
level from its side/head, and during a bedside UUDD walk. All five comparisons
and moving01 were individually inspected. Mattress/pillow/cuff edges retain
soft curves; the frame and headboard keep hard faces, floor contact and native
orientation. The bedside lane remains clear. Exact camera poses and paired
state are verified in rounded-bed-review/review-state-checks.json. The128-frame
walk runs(1,4)->(1,3)->(1,2)->(1,3)->(1,4), with one terrain build, matching
active terrain,zero final offset and no pending load/fade/warp. The bedroom's
current gallery pair is refreshed. This accepts the rounded bed geometry;
whole-room/ceiling/remaining furniture acceptance remains separate.

The renewed full311-map/225-state audit has zero mesher errors. Every field
shared with player-home-timber-foot-full311-audit.json is unchanged on every
map; all225 variant records are identical. The current audit adds the diagnostic
unmodeled_facade_cells field on all311 records; that schema addition is
preserved. Importantly, this baseline audit does not apply bedroom decoration
state: its PlayersHouse2F family list has no bed and its variant list has no
PlayersHouse2F entry. Thus unchanged audit geometry cannot prove the new bed.
The55 runtime model checks and matched decorated-room captures provide that
evidence. Audit summary/diff and native build/capture logs are in receipts/
rounded-furniture-ten-* and rounded-bed-native-captures.log. All three pack
hashes remain unchanged (rounded-furniture-ten-pack-hashes.json).

Service acceptance is being extended with the external-pack controller test
kanto_celadon_vending_controller_buys_refuses_and_releases_input. It approaches
the real background event, selects each drink through production menu inputs,
checks exact bag/money changes, insufficient-funds and full-stack refusals,
input ownership, cancellation and production Start afterward. Only initial
fixture money/stock are supplied; source/controller paths are not altered.
Test build22856 is live, logged in receipts/celadon-vending-controller-tests.log;
its compiler is confirmed active. No passing claim is made until its actual
result is inspected. Only cfg(test) code changed after the native build began;
the production furniture/wall implementation remains the tested/captured one.
The original all311/original16 service,reachable-state and native scene gates
remain active. No deleted handoff was restored and no pack/export work ran.

Celadon vending exposes qualified-menu identity bug; fix pending validation — 2026-10-17

The new external-pack controller regression compiles, then fails at drink
confirmation (0.98s): build visible menu choice frame -> menu_choice.menu_id
must be exact and untrimmed. This is a gameplay failure, not a rendering issue.
Initial inspection inferred the dotted label notation
CeladonDeptStore6FVendingMachine.MenuHeader; the later controller assertion
disproved that inference (see the exported-label correction below). game_data_scripts.rs preserves
menu.header_label as the menu identity, while MenuChoiceFrame previously applied
the generic link-token validator that forbids dots. Selection runs the source
path before recording the deterministic choice, so the transport rejection
interrupts an otherwise valid menu interaction. The failing test is evidence
of this gap, not a pass. Log: receipts/celadon-vending-controller-tests.log.

The first attempted MenuChoiceFrame validator accepted nonempty dot-separated
label components. That notation did not match the pack export and is superseded
by the exported-label correction below. It preserves the original
identity on the wire, whitespace/reserved-prefix rejection and all other link
token validation. No global token grammar or script/controller dispatcher was
relaxed. A core regression roundtrips the qualified menu, rejects empty local
components/whitespace/path separators/reserved prefixes, and verifies ordinary
link tokens still reject dots. The gameplay regression also requires the exact
qualified identity and checks three drinks across funded,zero-money and99-item
full-stack cases (nine setups), plus input ownership/cancel/Start.

Joint core/Bevy test build81625 is live in
receipts/celadon-vending-menu-identity-tests.log. After it finishes, run the
ignored vending controller test directly from the newly built Bevy libtest,
inspect all actual results and repair any further source/menu problems.
A new native build and paired service views are required for visual acceptance
of this behavior; the last completed examples predate the core identity fix.
The accepted bed geometry and its prior audit are unchanged by this transport
fix, but no current playable-vending claim is made. Full interior acceptance
remains open. No pack/export/audio content changes were made.

Exported-menu correction and Rocket statue backing implemented — 2026-10-17

Build81625 completed: three core menu-choice checks pass, with zero matching
Bevy tests. Those checks only covered the initially inferred dotted notation,
so they did not prove the actual exported vending menu. Direct gameplay test
67273 fails on the new exact-ID assertion: actual is
.MenuHeader@CeladonDeptStore6FVendingMachine, not the inferred
CeladonDeptStore6FVendingMachine.MenuHeader. The pack parser's
resolve_menu_reference explicitly emits local@parent. This is corrected at the
source validator and regression: ordinary menu tokens remain unchanged; only
qualified .local@parent identities are additionally accepted, with nonempty
components, exact whitespace and parent-token/reserved-prefix rules. The exact
original ID remains on the wire. The source-qualified core test now uses the
actual exported identity and rejects malformed/ambiguous parent separators.
Build81685 is the current corrected core/Bevy build, logged in
receipts/celadon-vending-exported-label-tests.log. It must pass, then the ignored
vending test must be rerun from its new Bevy binary. The regression now optionally
saves nine initial money/stock fixtures under KANTO_INTERIOR_STATE_REVIEW_DIR/
celadon-vending for native replay. No passing vending claim is made.

Native pre-fix evidence: approaching from(8,3) with UA yields a matched,
state-preserved vending menu with the source200/300/350 prices and3000 money.
The comparison was inspected. UAA then reproduces the real menu-choice identity
error and is correctly rejected by capture. That is a gameplay failure, not a
valid final pair. Earlier A/AA captures faced away and are excluded as service
evidence. UA from(8,2) reaches dialogue but fails the QA route's required actual
translation; its moving frame is diagnostic only. Evidence:
celadon-vending-review/pre-fix-approach-intro-* and
receipts/celadon-vending-pre-fix-approach-views.log. A fresh post-fix renderer
and purchase/refusal/cancel views remain required.

Rocket Persian review: new production before views show that the wall stops
behind the statue. The existing closes_wall_edge predicate suppresses exposed
neighbor ends but never fills that gap. The new backing requires the complete
eight-cell sensor drawing and an adjoining authenticated upper/lower horizontal
wall pair. It fills only the sensor's upper two source rows at the neighboring
32px ceiling height, using the same sampled blue body/black cap. It stays behind
the plinth, preserves eight source claims and all support, and does not infer
backing for an isolated/edited statue. New tests require complete front projected
area, confined bounds, rejection of edited source and unchanged live footing.
Body/head ellipsoids also now use24x12 curves with analytic normals under the
nonuniform native fit; pedestal/ears stay hard. Tests cover smooth/hard normals,
bounds and outward faces. No alarm/controller behavior, source claims or game content changed.
Native acceptance remains pending on a fresh build, after views and audit.

Own standalone geometry build31774 was deliberately stopped(exit130): its
standalone feature graph was rebuilding a separate GPU dependency graph, and
the core identity implementation had just changed. This was not a timeout or
passing test. Replacement joint Bevy/viewer build81341 preserves the established
feature graph and is queued behind81685, logged in
receipts/rocket-persian-smooth-backing-tests.log. Before captures are in
rocket-persian-smooth-review/. Front and eye at(3,3) were inspected; that eye
leaves the statue at the right edge, so it is not a close-shape acceptance view.
A clear quarter view at(5,2) is being captured for the actual before/after gate.
The full all311/original16 interior objective remains open.

All-five Rocket backing regression added — 2026-10-17

The backing acceptance now includes the actual external pack, not just a
synthetic wall fixture. external_rocket_persian_backing_closes_all_five_security_statues
builds immutable source cells from the supplied underground metatiles and
TeamRocketBaseB1F map, requires exactly five resolved statues and a complete
512-square-pixel ceiling-height front behind every one, then verifies original
footing across the whole map. The existing eight-cell ownership guard, the
isolated/edited-source rejection check, smooth-versus-hard normal check and
outward/bounded anatomy test remain. These checks are implemented but have not
yet executed; build81341 is still queued behind corrected identity build81685.
Both exact process handles were polled and confirmed live, not assumed stopped.

The before close pair at(5,2),camera0/7/-5/first-person, is now captured and
individually inspected. It clearly shows the missing wall behind the statue,
faceted gold anatomy and an unobstructed sculpture profile. This is the matching
position/camera for the after gate; the(3,3) eye view is peripheral and does not
replace it. No alarm-trigger coordinates were used for this close review.
Evidence: rocket-persian-smooth-review/before-eye-close-compare.png and its
paired-state file. A fresh native build, after comparisons and full-map audit
still remain required. The current native examples predate both the exported
menu-identity correction and statue/backing changes; they are before evidence
only. The original all311/original16 scope remains active.

Acceptance gates retained from current evidence — 2026-10-17

The original16 are confirmed directly in the last completed audit: three
nonempty declared block-edit combinations each for Will/Koga/Bruno/Karen/Lance,
and one Celadon Dept Store6F callback alternative. They are not simply three
chronological gameplay states. An audit combination is not proof of source
reachability; native art review and an earned controller/source-state trace
must be accounted for separately. The baseline room is additional to those
nonempty alternatives. No declared combination was removed to simplify success.

| Required scope | Evidence inspected | Outstanding acceptance |
| --- | --- | --- |
| All311 interiors | Last completed311-map/225-state geometry audit has zero mesher errors; gallery is a baseline view set | Full-room materials/composition, hidden areas, services and reachable states; refreshed audit after Rocket geometry |
| Original16 Kanto alternatives | All16 remain in the225 declared combinations, unchanged in the last audit | Native evidence for each alternative and source/controller reachability accounting; geometry counts alone are insufficient |
| Shared curved furniture |55 model checks and selected native classroom/cushion/stool/feathery-bed views pass | Remaining decoration-bed variants and broader fixture/room fidelity |
| Rocket statues | Missing wall is visible in close native before view; backing/curves and five-statue pack regression implemented | Run pending geometry checks; fresh native after views and full311 audit; alarm/support behavior must stay intact |
| Celadon vending | Actual controller/native confirmation fails on exported.local@parent identity; exact-format fix implemented | Corrected core tests, all nine controller setups, native purchase/refusal/cancel views |
| League/Hall | Earlier earned controller and scoped Hall evidence remains in detailed history | Continuous native gauntlet, complete ceremony phases/party variants/rotation and native animation fidelity |
| Large Battle Tower camera | Earlier source fallback diagnoses unreadable Cyndaquil/Kangaskhan compositions | Readable native views of actual poses; a source fallback is not native acceptance |
| Specialty equipment/services | Scoped clinic/PokéCom/counter work is recorded in detailed history | Remaining equipment identity/geometry and whole-room service-state review, including the small gray office fixtures |

The current native examples predate the corrected menu identity and Rocket
changes. Build81685 is still live; geometry build81341 is queued behind it.
This table records open proof obligations, not a narrowed completion target.

Rocket source gates passed; vending notification ownership under verification — 2026-10-17

Corrected exported-label build81685 completed successfully: three core
menu-choice tests pass, including the actual .MenuHeader@Parent grammar and
unchanged ordinary link-token validation. This supersedes the first dotted-label
attempt. Direct gameplay test2291 now confirms the first funded Fresh Water
purchase has exactly one added item and a200 deduction, then fails the cancel
assertion. It is not a full vending pass. Evidence:
receipts/celadon-vending-exported-label-{tests,controller-tests}.log.

The current test now requires the exact source menu with item-notification
flags/notices fully settled before cancelling. Inspection also found that B's
generic-source-menu branch preceded field/PC notice handling, whereas A already
lets those notices own input. B now defers that branch while either notice is
present and uses the existing modal handlers; no second dispatcher or new menu
state was introduced. Funded cases advance pocket notices with B and require
that, while the notice remains active, the underlying source menu/result is not
cancelled. Once the notice dismisses, authored continuation may legitimately
change the accumulator (PlaceMoneyTopRight publishes the current balance), so
the test does not freeze that authored result. Exact quantities/charges,
refusals, cancellation and Start remain required across all nine setups.
Build53249 is live in receipts/celadon-vending-notification-settle-tests.log.
It includes the notice-readiness and B-priority correction; inspect its actual
result before claiming success or starting final service captures.

The opt-in CRYSTAL_SCRIPT_TRACE diagnostic was attempted on the earlier test.
Its extra mid-loadmenu snapshot rejects the transient unresolved .MenuHeader
before verticalmenu qualifies it. That diagnostic failure is excluded as
cancellation evidence and was not reported as a passing gameplay test. The
normal production test's original failure remains the cancel assertion.
No trace logging or unrelated diagnostic paths were changed.

Rocket geometry build81341 completes successfully (22m39s including its queue).
All five selected viewer tests pass in0.43s: actual external-map backing for all
five statues, synthetic complete-course/rejection/footing checks, analytic
nonuniform curve normals, bounded outward sculpture faces and original complete
sensor drawing/support guards. Bevy matches zero Rocket tests and adds no
coverage. Evidence: receipts/rocket-persian-smooth-backing-tests.log.
The native after build/captures and full311 audit remain required. Current
native examples are before-fix evidence; the full original interior objective
remains active.

Vending cancellation stranded before authored closewindow; continuation fixed — 2026-10-17

The interrupted53249 handle is missing and process inspection confirms its
Cargo/compiler is no longer live. Its log has no final test result. It did
finish an updated Bevy libtest (15:18) containing the new notification check;
that artifact was run directly as5441. The actual result is failure in0.89s,
not a completed build/test pass. Purchase amount/quantity and notification
checks complete, but B cancellation leaves menu=None, runtime/text windows
open, stale CeladonClangText and active .Start@CeladonDeptStore6FVendingMachine
cursor at command3. Evidence: receipts/celadon-vending-notification-direct-tests.log.

CloseActiveMenu intentionally takes the menu identity only. The next authored
command3 is closewindow, but continue_visible_script_after_prompt treated the
leftover runtime window as an input boundary before that closer could run.
The continuation now permits exactly that authored closewindow when no menu
is active; all text waits, notices, battles and other modal owners still block.
It executes the existing source-step path, then the source comparisons,
closetext and end normally; no manual window/script-state reset or alternate
dispatcher was added. This complements the qualified-label and notice-priority
fixes. Current joint vending build24427 is live in
receipts/celadon-vending-source-closewindow-tests.log. Its exact process was
polled; the viewer dependency completed and Bevy compilation is active. Run all
nine actual cases and inspect the result before accepting service behavior.

All five Rocket before comparisons are now individually inspected with matched
source state and camera0/7/-5/first-person: (5,2),(23,2),(23,6),(21,16),(7,16).
They show the missing backing at all security statues and provide same-pose
before/after gates. All paired state is preserved in
rocket-persian-smooth-review/all-five-before-state-checks.json. The first
south-east capture lost its screenshot channel; retrying the exact same pose
succeeded. That was a readback failure, not a map/collision blocker, and the
failed attempt is not visual acceptance. Logs:
receipts/rocket-persian-{all-five,south}-before-captures.log.

The three core identity and five Rocket geometry checks remain completed
scoped evidence. Fresh native examples, after views, relevant controller
controls and the post-Rocket full311 audit still remain. No source/export/pack
publication ran. The original all311/original16 goal remains active.

Vending source cancellation now progresses; refusal fixture corrected — 2026-10-17

The source-closewindow Bevy executable was freshly produced at16:24 and run
directly as71561 while the joint Cargo run24427 was still finishing its viewer
test target. Funded Fresh Water and zero-money refusal both pass their exact
item/balance checks, cancellation and production Start. This proves the authored
closewindow continuation resolves the previously stranded .Start cursor. The
third setup then fails quantity100 versus expected99. This is a fixture error,
not an inventory regression: MAX_ITEM_STACK is99, but ordered pocket inventory
correctly appends a second stack when capacity remains. The original ReceiveItem
ASM explicitly scans to the terminator and allows a new slot, and the existing
core regression documents that behavior. No bag implementation was changed.
Evidence: receipts/celadon-vending-source-closewindow-direct-tests.log.

The full-bag setup now supplies MAX_ITEM_STACK*ITEM_POCKET_CAPACITY (1980 units
in20 full stacks) before interacting, then requires refusal with all quantity
and money unchanged. Fixture filenames use full-pocket, and the obsolete
owned FRESH_WATER-full-stack save/backup were removed. All three drinks still
require funded,zero-money and full-pocket cases (nine real controller setups),
notice ownership, cancellation and Start. New build50557 is queued in
receipts/celadon-vending-full-pocket-tests.log. The prior24427 command remains
live and must be polled rather than restarted; its old99-unit test is expected
to fail and cannot count as the corrected full-pocket gate.

The native examples still predate the controller identity/cancellation and
Rocket geometry changes. After all nine corrected setups pass, build fresh
render_at_location,render_saved_session and audit_authored_assets; replay the
owned initial vending fixtures and inspect the matched statue after views at
all five recorded positions. The full311 geometry/support audit and relevant
Mom/battle/nurse controller controls remain required. The overall all311/
original16 objective is not complete. No content pack or export ran.

Public notice presentation aligned with the controller owner — 2026-10-17

VisibleShellController.snapshot now projects active field/PC notices using the
same visible_field_dialog_pages and printer page index as graphical Bevy. It
hides an underlying source menu, exposes the current notice page with Text
ownership, and reports the remaining visible notice pages rather than stale
writetext history. Battle presentation remains on its existing path. Only the
returned presentation copy changes; source menu identity, game state, script
cursor and input dispatch are not replaced or advanced by observation.
The vending regression now explicitly requires the public controller to hide
menus under pocket notification and expose nonempty current notice text, in
addition to the existing actual B/input, money, quantity and cancellation gates.

Own partial build50557 was deliberately stopped(exit130) after this public-view
change: its Bevy test compiler had already parsed the old presentation. This
was a stale compile, not a timeout or passing result. Replacement joint
build86864 includes the full-pocket fixture and public presentation assertions,
logged in receipts/celadon-vending-public-notice-tests.log. It is the only
current owned test build and must be followed through its actual result. No
other agent's compiler/cache was stopped or modified.

All nine corrected setups still need to pass before final native rebuild and
service acceptance. The three core identity and five Rocket geometry checks
remain completed scoped evidence; native examples still predate the changes.
The all311/original16 objective remains active. No pack/export publication ran.


Vending price scope defect reproduced and repaired — 2026-10-17

The fresh public-notice Bevy executable was run directly while the joint
Cargo command98762 finished its viewer target. All three Fresh Water setups
pass (funded, zero money, full1980-unit pocket), including exact quantity/money,
notice ownership, cancellation and Start. Soda Pop then fails at its source
checkmoney: UnknownCurrencyConstant CELADONDEPTSTORE6F_SODA_POP_PRICE. This is
an actual service defect, not a successful nine-case run. The map's source DEFs
are already in story_event_script_constants; execution previously consulted
only currency_constants, although text rendering already uses scoped DEFs.

GameDataSet.apply_script_economy_command now resolves amount operands from the
owning map's constants, then global source constants, before retaining the
existing core economy evaluator and authoritative currency caps. It does not
merge other maps or rewrite exported programs, catalogs, or packs. A new assets
regression exercises local/global expression operands, foreign-map rejection,
negative/oversized rejection without mutation, and an unchanged global money
cap even when a map defines MAX_MONEY. Its build13268 is queued behind98762;
no passing result is claimed yet. The conditional native driver must reject
the old nine-case run and must not publish stale native acceptance.

The same fresh controller executable passes four explicit controls: Mom/New
Bark gate, actual battle turn/PP, nurse HP/PP healing, and native nurse live art/
overlay ownership (one matched test each). Receipts:
celadon-vending-public-notice-direct-tests.log and
vending-public-notice-controller-controls.log. Re-run the full vending matrix
and relevant controls after the scoped-price fix; native after views and full311
audit remain required. The full goal remains active. The deleted handoff stays
deleted; these canonical notes retain the remaining work.

Obsolete joint build stopped after reproduced service failure — 2026-10-17

Owned joint Cargo98762/session86864 was deliberately interrupted(exit130)
after its fresh Bevy executable reproduced the Soda Pop price-scope error.
It was compiling a viewer target with zero matching vending tests; continuing
that obsolete target could not verify the repaired assets execution path.
Only that owned Cargo was signaled. Driver36785 exits1 on its test-result gate;
no native build started. Replacement assets regression13268/Cargo822 is live,
compiling shared core dependencies in target/kanto-resume after acquiring the
released cache. Follow its actual terminal result before running the repaired
nine-case Bevy matrix. This is not a timeout-triggered restart or a test pass.

Scoped-price verification pipeline queued — 2026-10-17

Assets regression13268/Cargo822 remains live and has reached compilation of
crystal-assets with the scoped amount-operand fix and its complete new test.
Driver98204 (target/kanto-resume/run-scoped-vending-review.py) waits for that
exact Cargo process, requires its one matched test to pass, then runs the full
nine-case Bevy vending regression (crystal-bevy/location-tester only; no empty
viewer target). It requires all nine explicit case receipts plus a passing test
footer before building render_at_location, render_saved_session and
audit_authored_assets. Logs: scoped-vending-review-driver.log,
scoped-script-economy-tests.log, celadon-vending-scoped-prices-tests.log, and
statue-vending-scoped-native-build.log under receipts/. No duplicate build or
source export was launched. At this checkpoint the assets compile is still
running; neither the repaired vending matrix nor the native build is a pass.
After native exit0, inspect all five exact-pose Rocket after comparisons and
funded/refused vending routes, then perform the full311/225-state audit and
repeat the four controller controls from the repaired Bevy binary.

Scoped source-price regression passes — 2026-10-17

Assets build13268 exits0 after8m11s; the one matched
script_economy_resolves_own_map_prices_without_leaking_scopes_or_caps test
passes. It verifies own-map PRICE plus a global fee, foreign/negative/oversized
amount refusal with unchanged state, and both debit/credit preserving the
pack's MAX_MONEY despite a conflicting local definition. Log:
receipts/scoped-script-economy-tests.log. This is scoped unit evidence; the
nine real source vending cases are not yet accepted.

Driver98204/Python1619 has advanced to fresh Bevy vending Cargo2754, logged in
receipts/celadon-vending-scoped-prices-tests.log. The native example build is
still conditional on all nine explicit passing case receipts and a passing
test footer. Separate driver2768/Python2747 waits for1619 to finish, requires
NATIVE_BUILD_EXIT=0 and a native executable header, then captures all five
Rocket after poses at the exact before camera and runs all311 maps with declared
script-block alternatives. Its scripts/logs stay ignored under target/kanto-resume.
After capture, each actual native comparison must still be visually inspected;
state/file existence and a geometry audit do not prove visual acceptance.

A reread of the saved Tower Cyndaquil/Misdreavus native move-menu image confirms
that pair is readable. It is not the separate Cyndaquil/Kangaskhan case and
does not resolve that outstanding higher-shot failure. No camera threshold,
body dimension or admission gate was weakened. The obsolete99-unit full-stack
vending saves/backups were removed; full-pocket fixtures use1980 units.
The full interior/furniture/earned-state goal remains active.

Remaining bedroom variants now have a production-input fixture gate — 2026-10-17

Added ignored external-pack controller test
kanto_owned_bed_variants_use_pc_input_and_publish_review_fixtures. Each of
Pink, Polkadot and Pikachu beds starts in an explicitly owned-decoration visual
fixture. It opens the real bedroom PC, selects Decoration/Bed and the source
entry through VisibleShellController.press, dismisses setup notices, verifies
wDecoBed, closes the nested menus and walks from the PC to(1,4). Optional final
saves go to celadon-vending-controller-review/owned-bed-variants. Source flags
are seeded only for initial ownership; this is not a Mystery Gift acquisition
or earned-state proof. No bed state, source block or room transition is directly
mutated after initial setup. Native multi-angle variant review remains required.

The test was added while vending Cargo2754 was still compiling crystal-assets,
before its Bevy test compiler started. Companion driver12220 waits for that
exact Cargo, requires all nine vending case receipts and a one-test pass, then
runs the four prior gameplay controls and this owned-bed test from the actual
new libtest path printed by Cargo. It rejects zero matches; no parallel Cargo
build is added. Log: receipts/repaired-controller-controls-driver.log, with
individual *-scoped-prices.log control receipts. No result is claimed yet.
The existing native/statue/audit pipelines remain live behind their gates.

Original16 structural alternatives: source path accounting clarified — 2026-10-17

The completed225-state audit was inspected directly. For each of Will/Koga/
Bruno/Karen/Lance, block_mask1 is the northern exit-open edit, mask2 the southern
entrance-wall edit, and mask3 both. Celadon6F contributes its unconditional
roof-stair wall callback, completing16 nonempty alternatives. These masks must
all retain geometry/native review; baseline is additional, not one of the16.

The pinned external source maps were read directly. Indigo's NEWMAP callback
resets all five door scenes and both door flags. Each room's arrival scene
closes the southern doorway before returning normal input. The four Elite Four
victory scripts open the northern exit and set EXIT_OPEN while retaining the
closed entrance. Lance opens the northern block directly but sets ENTRANCE_CLOSED
again; it does not set EXIT_OPEN. The existing external controller regression
already explicitly asserts that source quirk and the actual block override;
do not normalize Lance into the other four flag sequences.

| Alternative | Source interpretation | Remaining proof |
| --- | --- | --- |
| Five mask2 entrance-only states | Normal locked arrival before victory | Reuse verified real entry states; exact native variant/joins still need accounting |
| Five mask3 combined states | Victory opens exit while the entrance remains closed; Lance uses a direct block edit | Continuous earned native gauntlet and same-state source pairs remain required |
| Five mask1 exit-only states | Declared independent edit combinations; no normal reset-to-entry-to-victory path demonstrated | Keep all five geometry/native fixtures, explicitly label synthetic combinations; do not claim earned reachability |
| Celadon6F mask1 | TILES callback unconditionally hides rooftop stairs | Real callback is already exercised; native service/room proof still pending |

This is source-path analysis, not a new runtime pass or proof that every possible
engine path was exhaustively excluded. No variant was dropped and no script,
flag, controller state or audit mask was changed. The repaired vending Cargo2754
remains live compiling the viewer library; the new bed test is not yet executed.

Exit-only native variant review queued without changing source behavior — 2026-10-17

Existing render_at_location --preview-flag was inspected: it permits only a
fresh disposable runtime-tile session without saving, settles the initial
callback, installs the verified flag, and reruns production map setup. This
supports each of the five mask1 exit-only combinations without adding a block
insertion path or changing Lance's actual script. Driver16059 waits for the
statue/audit driver2747 and requires its actual full311 exit0, then captures
front/east pairs for all five rooms. Each identity must include its exact
EXIT_OPEN fixture flag and each paired receipt must preserve state. The ignored
capture-provenance.json labels all ten views synthetic and explicitly denies
earned reachability. Visual review remains required after capture. Log:
receipts/exit-only-variant-review-driver.log. Native jobs are serialized.

A live-binding review also confirms the player bedroom has precisely four
source bed forms: Feathery1b, Pink1c, Polkadot1d and Pikachu1e. BedBlue and
BedGreen have canonical mesh/normal/bounds validation but no current production
placement selector; the full311 baseline contains only interior/bed_red and
cannot establish decoration-bed appearance. Do not invent source bindings to
make unused palette assets appear in the native gallery. The three pending
owned-bed PC fixtures cover the remaining live rounded variants; their actual
controller and multi-angle native results are still required. Cargo2754 has
reached Bevy test compilation and includes that new fixture test.

All nine vending cases pass; owned-bed controller reload isolated — 2026-10-17

Fresh Bevy build2754 completes successfully in13m10s. The one matched vending
regression passes in5.01s and logs all nine exact cases: Fresh Water200, Soda
Pop300 and Lemonade350 each buy exactly one with1000 initial money, refuse at
zero money, and refuse a full1980-unit pocket without charge/item changes.
Every case also verifies source menu identity, modal input/notice presentation,
B notice handling, source cancellation and production Start. Log:
receipts/celadon-vending-scoped-prices-tests.log. The scoped amount-resolution
fix therefore has actual service coverage, not just a unit pass.

All four controls pass again from that exact new Bevy libtest: Mom/New Bark
3.97s, actual battle turn1.67s, nurse healing1.91s, native nurse presentation
2.56s (one matched test each). Driver12220 then correctly fails on the new owned
bed test. Pink selection and wDecoBed pass, both decoration/PC menus close, but
Down/Down/Left leaves the controller at(2,2) rather than(1,4). No bed save was
accepted. The existing integrated graphical owned-Pink-PC test passes separately
in4.93s from the same external pack and executable, including its script-cursor/
text closure and movement checks. This separates a renderer-neutral settling
problem from the graphical setup path; do not label the failed new test passed.

An opt-in source trace reproduces the same failure (no additional trace error):
PlayersHousePCScript branches through .Warp and stages the authored bad-warp
map reload. The new test now asserts script/text closure before walking and
prints UI,cursor,special,load,warp,fade/action state on failure. Diagnostic Cargo
session24610 is queued behind the running native example build. No production
controller reload patch has been guessed. Logs: owned-bed-integrated-comparison,
owned-bed-controller-script-trace, owned-bed-controller-reload-diagnostic-tests.

Driver98204 advanced to the actual native example build, logged in
statue-vending-scoped-native-build.log. It has not completed yet. Statue/audit
2768 and exit-only16059 remain behind their gates. Native vending/bed visual
review, full311 audit and original16/earned-state/Hall acceptance remain open.
No pack/export publication ran; the full goal remains active.

Non-disruptive capture requirement — user correction, 2026-10-17

The user explicitly requires no render windows. The owned exit-only capture
driver5294 was terminated; process inspection confirms no owned windowed
render/capture driver remains. A child finished during termination; no batch
was restarted. All future captures must explicitly use
CRYSTAL_RENDER_TEST_HEADLESS=1 and CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image.
The existing feature-gated native implementation disables WinitPlugin, skips
native-window retention and uses ScheduleRunnerPlugin with the image target.
The compiled render_at_location contains its headless admission guard. Owned
statue/exit-only driver scripts now set HEADLESS=1; do not rerun windowed commands
from older notes, and never fall back to windowed mode if headless capture fails.

A headless test at(6,4) was rejected before app startup because that source tile
is unwalkable; no image or headless rendering pass is claimed for it. Replacement
56593 uses the already verified(5,2) statue pose and is in progress, logged in
receipts/rocket-persian-headless-eye.log. The PC reload diagnostic24610 remains
non-windowed and was not stopped. Native build98204 passed6m33s; the five statue
side after comparisons and all311 audit completed before the user correction.
All five side pairs were individually inspected: backing is continuous and
curves are smoother, but frontal Persian anatomy/overall fidelity still needs
review. The audit covers311 maps/225 variants with no mesher errors. Shared map
fields are identical to the prior baseline; added flat_house_native_window_cells
and variant original_block diagnostics require schema-aware comparison before
claiming identical variant records. The exit-only batch is interrupted and not
a completed ten-view set. The full goal remains active under the no-window rule.

Headless paired capture56593 completed successfully(exit0,5.71s) at the verified statue pose, with both images validated and state_preserved=true. This verifies the non-windowed capture path for subsequent visual work; no windowed fallback is needed.

PC reload defect identified; schema-aware full audit verified — 2026-10-17

Diagnostic24610 completes(exit101,8m02s build) and exposes the exact post-PC
state: menu=None,text=None,window_open=false,text_window_open=true, with no
script cursor, special, pending load/warp, fade or text wait. The bad-warp room
reload has completed; its empty retained textbox incorrectly owns input in the
renderer-neutral controller. The source PC changed-decor path uses warp NONE,
0,0 with MAPSETUP_BADWARP and no following closetext. take_visible_pending_map_load
now retires that inherited textbox through the existing close_text_window API
before bad-warp map callbacks run, preserving any new callback text. Other map
setup paths and authored command streams are unchanged. External owned-bed
regression15923/Cargo10049 is compiling the repair; no pass is claimed yet.

The full311 audit's shared map fields are identical to the rounded-furniture
baseline, with zero mesher errors and all225 variant identities/order retained.
All225 records are equivalent after removing only the newly added original_block
metadata from block_edits. The added flat_house_native_window_cells field exists
on311 records; it is diagnostic schema, not changed ownership/footing. Exact raw
and normalized diffs are ignored receipts/statue-vending-full311-*-diff*.json.
The report counted only mesh.solid, whereas Persian/backing append to
mesh.textured; unchanged solid counts do not prove that textured geometry is
unchanged. Added explicit textured_vertices/textured_triangles to the source
audit example. Its updated executable/audit still need building/running; do not
claim textured-count verification from the earlier report.

Exit-only headless recapture49171 is active in a fresh directory,
original16-exit-only-headless-review, so it cannot silently reuse the interrupted
windowed images. It explicitly enables HEADLESS=1, image capture and preview
mute; native window creation remains disabled. It must finish all ten pairs,
then each actual comparison still needs inspection. The full all311/original16,
shared furniture, native earned League/Hall and room-fidelity objective remains
open. No source/export/pack publication ran.

Headless original-variant review exposes Lance north-wall defect — 2026-10-17

Headless/muted batch49171 completes(exit0) with ten new paired views in
original16-exit-only-headless-review; no windowed image cache was reused. All
ten comparisons were individually inspected. Will/Koga/Bruno/Karen show their
northern exit-only openings and joined source courses from front/east. This
accepts only that declared geometry scope, not whole-room fidelity or earned
reachability. Lance's center views crop its northern doorway, so two additional
headless pairs73570 at(4,3), north-front/north-east, were captured and inspected.
All12 preserve source state and requested camera; capture-provenance.json keeps
the synthetic EXIT_OPEN flags and earned_reachability=false labels.

The closer Lance east view exposes an actual defect: the north doorway and gold
ornamental columns remain flattened source artwork on the raised platform.
The source callback opens the correct block, but this is not acceptable upright
3D architecture. Do not count the Lance mask1 art complete. A complete guarded
north arch/column volume is required for both original2e and open0b drawings,
with correct front/east views, source/custom/truncation guards, unchanged footing
and both warp approaches clear. Current lance_terraces lifts the authenticated
upper12 source rows and broad stairs; retain that platform/footing and original
sixteen alternatives while fixing the north drawing. No speculative geometry
was added during diagnosis. This defect also affects baseline/combined Lance
states and must be covered there, not hidden by a favorable camera.

Owned-bed repair15923 builds in4m05s. The new UI/cursor closure assertion passes
and actual walking now reaches(2,3), proving the stale textbox correction works;
the three-frame fixture had incorrectly expected two tiles down and one left.
The test now uses bounded ordinary directional frames until(2,4), then(1,4),
retaining exact target/state assertions. Replacement15719/Cargo11241 is live;
all three bed variants still need to pass before saves/native appearance are
accepted. No fixture coordinates are directly mutated after setup.

From the exact bad-warp-fixed libtest, all nine vending cases pass again6.19s,
plus Mom/NewBark2.51s, actual battle turn2.59s, nurse healing2.02s and native nurse
presentation3.13s (one matched test each). Driver23483 exits0; receipt
badwarp-close-controller-controls.log. Native examples still predate the
bad-warp fix and new textured audit count fields; rebuild after the controller
fixture passes. The full goal remains open and all future captures remain
explicitly headless/muted, with no windowed fallback.

Owned-bed controller gate passes; upright Lance portal implemented — 2026-10-17

Bounded-walk build15719 exits0 after6m50s. The one matched external-pack test
passes1.22s and logs all three actual PC setups: DECO_PINK_BED,
DECO_POLKADOT_BED and DECO_PIKACHU_BED. Each verifies source wDecoBed, bad-warp
reload/text/script closure and ordinary directional input to(2,4),then(1,4),
without post-setup coordinate mutation. Final review saves exist under
celadon-vending-controller-review/owned-bed-variants. Ownership was explicitly
seeded before input; no Mystery Gift acquisition is claimed. Native appearance
of these variants is still pending. The earlier five bad-warp-fixed control
passes (including all nine vending cases) remain scoped evidence.

Implemented mesh/lance_north_portal.rs, called only after lance_terraces has
authenticated the complete upper12-row champion drawing, exact map extent and
absence of conflicting custom placements. It removes the old horizontal north
art at the raised platform datum and builds ceiling-height side-wall volumes,
source-painted round gold columns and an elliptical arch/soffit. Closed2e keeps
its painted door panel; open0b has a real aperture across both source warp lanes.
Ground through the opening is separate from architectural cutaway. The existing
platform/stair finish, collision, source scripts, door flags and authoritative
footing are retained. No new game-content pack or copied model is introduced.

The existing terrace winding/cutaway test now expects the retained owner plus
the new architectural owner. A new geometry test covers both source door forms,
all80 north-cell claims, exact whole-fixture footing preservation, ceiling/
platform bounds, unchanged source frame and both open/closed warp-lane sections.
Existing all-source-cell/custom/map-extent rejection checks remain. Joint
geometry25964/Cargo13496 is live compiling the viewer with these changes in the
owned cache. No test/native acceptance is claimed yet. Logs:
receipts/lance-north-portal-geometry-tests.log and
receipts/owned-bed-controller-bounded-walk-tests.log.

After those geometry checks pass, rebuild native render_at_location,
render_saved_session and the audit example (including new textured counts),
then inspect headless/muted Lance closed/open front/east views at(4,3), actual
warp/entry/controller controls, all three bed variants and the fresh full311/
225-state audit. Keep the five synthetic exit-only labels and all original16
alternatives; continuous native earned League/Hall, Persian frontal anatomy,
remaining services/furniture and whole-room311 acceptance stay open.

Native-pack bed fixtures and six headless material diagnostics — 2026-10-17

The native save loader correctly rejects TUI saves because their identity adds
+text-tui. No header was edited and no compatibility guard bypassed. The same
owned-bed production-PC regression was rerun directly against the completed
native core pack: all three variants pass10.11s and save matching native fixtures
in native-owned-bed-controller-review/owned-bed-variants. Receipt:
owned-bed-native-pack-controller-tests.log. Initial ownership remains seeded,
not a claimed Mystery Gift acquisition.

Six headless/muted captures complete: each Pink/Polkadot/Pikachu bed has an
above-front and unobstructed eye-side view, with matched source screenshots.
All six state/camera receipts pass; source/native images and all three close
native side views were individually inspected under owned-bed-headless-review.
The rounded geometry reads clearly, but appearance is NOT accepted: all three
frames are beige wood where source rails/posts are gray; Pink is too muted;
Polkadot inverts the source's pale cover/blue dots into blue cover/white dots;
Pikachu substitutes a small geometric face for the source's large printed
figure. These are concrete remaining shared-furniture defects, not gameplay
failures. The current native tools predate the pending portal implementation;
these captures diagnose unchanged bed assets and save/camera setup only.

Source pixel checks from the actual Pink paired screenshot confirm frame gray
(156,156,156), dark(57,57,57), ivory(247,231,214), coral(247,82,49), pink
(255,156,198). Polkadot uses pale ivory and blue/periwinkle; Pikachu uses yellow,
orange and dark source art. Keep live source sampling; do not commit extracted
prints or another catalog. The existing Feathery native finish already samples
its source, while the three variant meshes still use generic material colors.
Their canonical blanket decorations in build-johto-interiors.py explain the
inverted dots and invented mascot face. Correct native frame/cover/print binding
and preserve rounded geometry, exact extents, claims, footing and custom guards;
source-accurate printed cloth must not become another flat bed card.

Portal joint test25964 exits101 before geometry tests because species_rig's
concurrent Electabuzz GLB addition arrived after the build script's asset scan;
its .gz/.meta outputs were missing in the owned cache. The gate45471 correctly
exits1 and starts no native build. No source/catalog was reverted or hand-copied,
and no cache was cleaned. Replacement37480/Cargo15691 reruns the existing build
path; the expected Electabuzz generated outputs now exist(17:40), and compilation
is live. New native-build gate91550 waits for15691 and the same three explicit
portal test passes before rebuilding tools. Logs:
lance-north-portal-generated-assets-tests.log and
lance-portal-generated-assets-native-driver.log. Portal tests/native after views
and the updated textured-count full audit are still unverified. The complete
all311/original16/shared-furniture/earned-state goal remains active; all captures
must remain explicitly headless/muted with no windowed fallback.

Live decoration-bed prints implemented; portal geometry gates pass — 2026-10-17

The regenerated-asset retry completes all three Lance portal tests: upright
closed/open doorway coverage in both warp lanes, three broad platform steps with
retained cutaway ownership, and rejection of incomplete/customized upper source
art. This proves geometry invariants, not visual acceptance. Receipt:
target/kanto-resume/receipts/lance-north-portal-generated-assets-tests.log.

Pink, Polkadot and Pikachu beds now bind their complete live blanket art onto
the same rounded canonical frame/mattress geometry, preserving fitted extents,
source claims and support. Frame/linen colors sample the live source rather
than generic beige material. Admission requires the correct map and all eight
cells of that variant's own drawing. No printed assets were extracted or added
to the repository. Two new tests cover geometry preservation, print bounds,
source ownership and unchanged support; Cargo17334 is currently compiling them.
Their result and native appearance remain unverified.

The replacement verification driver requires both new bed checks and reruns all
three portal checks from that same freshly compiled viewer test binary before
building render_at_location, render_saved_session and audit_authored_assets.
It replaces the old wait-only native driver; no duplicate build is running.
Receipts: native-decor-bed-print-tests.log,
lance-portal-current-print-binary-tests.log and
verified-interior-native-build-driver.log under target/kanto-resume/receipts.

All further captures must explicitly set CRYSTAL_RENDER_TEST_HEADLESS=1,
CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image and CRYSTAL_PREVIEW_MUTE=1. Never open
review windows or use a windowed fallback. Paired offscreen views and the fresh
311-map/225-variant audit with textured geometry counts remain pending. The
complete interior/shared-furniture/earned-state scope remains unfinished.

Native vending fixtures pass; bed build retry after concurrent inputs — 2026-10-17

The existing production-controller vending regression passes against the native
core pack: all nine combinations (three drinks, funded/no-money/full-pocket)
verify exact item/money changes or refusal, notice ownership, cancel closure and
Start input. One matched test passes12.95s. Native-compatible initial review
saves are in target/kanto-resume/native-vending-controller-review/celadon-vending;
receipt: receipts/native-vending-controller-fixtures.log. This is controller
proof, not yet native service-menu visual acceptance.

Bed build87527 exits101 before executing the new bed checks: concurrently added
Slowbro/Slowking include_model references lacked generated .gz/.meta files and
test-local constants at compile time. Both GLBs and both constants now exist in
the current sources. Retry62091/Cargo19280 runs the same normal build path;
generated Slowbro/Slowking outputs now exist in this task's cache. No unrelated
source was changed or reverted and no cache was removed. The prior gates
correctly stopped without building or capturing. Replacement driver24205 waits
for the two actual bed passes, then reruns three portal checks and builds the
native tools. Headless capture/audit driver76473 waits for that native build.
Logs: native-decor-bed-generated-assets-tests.log,
verified-interior-generated-assets-native-driver.log and
printed-bed-portal-generated-assets-headless-driver.log under receipts.

Continuous native-pack League/controller ceremony passes — 2026-10-17

Direct production-controller regression
kanto_controller_continuously_earns_league_and_continues_home passes one matched
test72.61s against core-modular.crystalpack. It seeds only the initial level100
Mew party/moves and five Full Restores, then carries one controller through
Will, Koga, Bruno, Karen and Lance. Every battle actually runs, spends PP,
earns its victory flag and settles; movement follows collision paths and real
warps. Remaining total PP81→76→70→59→53; two Full Restores are used through Pack,
with stock5→4→3. No victory flags or post-setup coordinates are assigned.
Receipt: target/kanto-resume/receipts/native-continuous-league-controller.log.

The same run enters HallOfFame, slow-steps to the machine at(4,3), verifies
ceremony input ownership and one recorded real party member, traverses all
required Pokémon/player/Oak/closing phases and the credits to THE END, drives
the production title/Continue summary including its20-frame clock, and returns
to NewBarkTown with one Hall record and resumed game timer. The setup party is
a fixture, not a claimed earned acquisition or a full story playthrough.

Eight required phase-entry images under native-continuous-league-controller-
review/continuous-league were individually inspected: Pokémon animation/hold,
Oak rating and closing have the correct source sprites/text; Pokémon/player
back/front entry frames are blank transition starts. These are source-renderer
phase-entry diagnostics, not native3D or whole-phase animation acceptance.
Native continuous walking/battle/ceremony captures and party-size/rotation
variants remain required. This completes native-pack controller progression
proof only; all311 room fidelity, source alternatives and shared furniture
remain within the unfinished objective.

Nine headless vending service pairs inspected; fixture identity tightened — 2026-10-17

Using native-compatible initial controller saves and the existing native tool
that already includes the vending fixes, nine headless/muted pairs were captured
and individually inspected. Every state/camera receipt passes. Actual observed
states include source prices200/300/350 and wallet1000; insufficient-money
refusal; full-pocket refusal followed by the returned menu; purchase Clang;
two real item-pocket notice pages with the underlying menu hidden; and cancel
with both menu/text fully closed. Shorter funded/full-pocket AA routes still
show the initial menu and are not mislabeled purchase/refusal evidence. The
native machines have finished top vents and the surrounding wall remains
continuous in these views. These service views do not certify the whole room.
Artifacts: target/kanto-resume/native-vending-headless-review; exact observed
states and pre-bed/portal binary provenance are in service-state-provenance.json.

Inspection also catches unnamed trainer setup in this regression's initial
fixtures: the source notice prints "put the" without a trainer name. The
production notice already uses snapshot.trainer.player_name. The fixture now
sets CHRIS explicitly and checks the complete trainer/drink/pocket text during
all three successful purchase cases. No production notice code was changed.
Named regression18830/Cargo21375 is queued in this task's cache; its result,
native-compatible named saves and replacement notice views remain pending.
The old unnamed notice is diagnostic evidence, not final text acceptance.

Both bed-print guards and all three current portal gates pass — 2026-10-17

Retry62091/Cargo19280 exits0 after15m06s. The two actual viewer bed tests pass
in0.01s: native_decor_bed_print_keeps_rounded_geometry_and_live_orientation and
native_decor_bed_variants_require_own_art_and_preserve_support_claims. Bevy
matches zero and adds no coverage. These establish preserved rounded geometry,
source print bounds/orientation, neutralized generic tint, own-art admission,
claims and support. Receipt: native-decor-bed-generated-assets-tests.log.

The gated driver reruns all three kanto_lance tests directly from that same
fresh viewer test executable; all three pass0.00s, with no ignored matches.
Receipt: lance-portal-current-generated-assets-tests.log. The bed/portal native
appearance remains unverified. Native tool build22002 is now submitted and
waiting for named vending Cargo21375 in this task's cache; Cargo21375 is live
compiling updated viewer dependencies. No build was restarted on a timeout.
Driver24205 still owns the native build, and headless driver76473 still waits
for its successful result before ten paired captures and the full311/225 audit.
Logs: verified-interior-generated-assets-native-{driver,build}.log and
printed-bed-portal-generated-assets-headless-driver.log under receipts.

The nine earlier service pairs and full native-pack continuous controller
League/Hall pass remain scoped evidence. Named vending regression/native notice
refresh, new bed/portal visual inspection, current textured audit, all311 room
fidelity and the remaining shared-furniture/earned native scene obligations
remain open. All rendering stays headless, image-targeted and muted; never open
windows or use a windowed fallback.

Frontal Persian diagnosis and anatomy correction implemented — 2026-10-17

Two additional headless/muted native pairs at TeamRocketBaseB1F(5,3) and(6,3)
were individually inspected against their same-state source. Both receipts pass,
with cameras0/7/0/1 and0/0/0/1. The front view exposes a round head, two bulbous
muzzle balls and a cylindrical neck: generic cat anatomy, not accepted Persian
fidelity. Artifacts: target/kanto-resume/rocket-persian-frontal-headless-review.
These are initial-location/no-input fixtures, not alarm progression evidence;
standing initially on a sensor does not prove its movement-trigger behavior.

rocket_persian.rs now blends wider shoulders into the seated haunches, lowers
and broadens the skull, reduces the muzzle and adds contour-attached almond
lids, a chiselled nose, faceted forehead jewel and six attached tapered whisker
ridges. Existing live source gold/shadow paint, pedestal,16x8 footprint,28-pixel
height, backing ownership and source/controller behavior are retained. Eyes and
jewel follow the skull surface; the whiskers begin inside the cheeks. No new
asset catalog, pack, Blender file or species model was introduced. This new
anatomy is implemented but not yet tested or visually accepted.

Own queued native Cargo22002 was confirmed to have no compiler child, then
stopped along with its wait driver19369 to include the new anatomy gate. No
other owner's compiler/cache was touched. Its old capture driver correctly
rejects the missing native-success result and starts no captures. Replacement
native driver21912/Python23240 waits for named vending Cargo21375, then runs the
two bed, three portal and five Rocket tests from that fresh viewer executable
before building tools. Capture driver52046 waits for that build, then captures
six beds, four door states/views, two frontal statue views and three named
vending states, all headless/muted, followed by the full311/225 textured audit.
Receipts: current-anatomy-native-driver.log, current-anatomy-native-build.log,
current-anatomy-headless-driver.log and current-anatomy-*-tests.log.

Rocket script control passes; real-input sensor regression added — 2026-10-17

Existing rocket_camera_runs_two_grunts_then_switch_disables_every_camera passes
one matched test16.86s. It checks both Grunt identities, completion/event order
and the secret switch disabling all eight source camera entry scripts. Its
helper starts scripts and supplies battle completion, so this is scoped source
continuation proof, not ordinary movement or real-battle proof. Receipt:
target/kanto-resume/receipts/persian-anatomy-rocket-script-control.log.

Added ignored external-pack regression
kanto_rocket_statue_sensors_use_walk_input_and_earn_both_grunt_victories. Across
all eight actual source entry tiles (both lanes of cameras1/2/3 plus cameras4/5),
it supplies only initial identity/party, walks from the neighboring tile via
VisibleShellController::press, observes GRUNTM_20 then GRUNTM_21, fights with
ordinary A input, requires PP consumption and the earned cleared-camera event,
walks off and re-enters the same sensor, and verifies released Start ownership.
It injects no camera flags, coordinates, source scripts or victory outcomes.
Optional native-compatible earned review saves are written before Start.

This new test is implemented but not compiled/executed yet. Driver20388 waits
for the current verified native build, then runs only the Bevy test target and
this exact external regression against the native pack, using this task's cache.
Receipts: native-earned-rocket-controller-driver.log and
native-earned-rocket-controller-tests.log. Named vending Cargo21375 remains
live in viewer libtest compilation; no timeout was treated as terminal and no
competing native build is running. The complete interior/shared-furniture scope
remains open; all native captures stay headless/muted.

Named vending and ten current geometry gates pass — 2026-10-17

Cargo21375 exits0 after18m58s compile. The one actual named vending regression
passes12.70s and all nine funded/no-money/full-pocket drink cases pass, including
complete CHRIS/drink/item-pocket notification text. Native-compatible named
fixtures are saved under native-named-vending-controller-review/celadon-vending.
The viewer filter matches zero vending tests and adds no coverage. Receipt:
native-named-vending-controller-tests.log.

The fresh viewer binary then passes both bed checks0.02s, all three Lance portal
checks0.00s and all five current Persian checks8.90s against the native pack.
The Persian results include the new anatomy's bounded/outward geometry and
analytic curves plus all-five-statue backing/source/support guards. Receipts:
current-anatomy-{native_decor_bed,kanto_lance,rocket_persian}-tests.log. All52
current interior_models::tests also pass0.15s, including complete catalog parse,
material-role and fitted-bound controls; current-bed-model-catalog-controls.log.
These checks establish scoped geometry/source invariants, not visual acceptance.

An offscreen matched named-pocket pair from the existing vending-fixed native
tool was individually inspected: CHRIS and FRESH WATER are fully readable,
with no underlying menu on either view. State/camera receipt passes. Artifacts:
native-named-vending-headless-review/named-pocket-{2d,3d}.png. This closes the
unnamed fixture diagnostic; it does not claim the new anatomy/bed/portal native
build is finished. Native Cargo25365 is live compiling the viewer library in
the owned cache. Driver23240 still owns that build; captures and the new earned
sensor-input regression remain behind its success gate. No windows opened.

Native batch/audit and earned sensor walks verified; finish defects corrected in code — 2026-10-17

Native build25365 exits0 in4m52s. All15 headless/muted pairs complete with
preserved state and exact requested camera, and source/native images were
individually inspected. Six decoration-bed views now show correct gray rails,
Pink marks, pale/blue Polkadot art and the large Pikachu print on rounded cloth.
Four Lance views establish an upright closed/open arch and round source-gold
posts. Two Persian front/three-quarter views show the revised face/shoulders and
continuous backing. Three named vending views show the correct menu and fully
readable CHRIS/FRESH WATER notice with the menu hidden. Both longer named routes
still stop at notice page0; an extra6A diagnostic returns Clang, so none is
misreported as named page1 evidence. Detailed observed states are recorded in
printed-bed-portal-headless-review/capture-provenance.json.

The full311/225 audit exits0 with311 verified maps, zero mesher errors and all16
original Kanto alternatives retained. All map source/support invariants and all
225 variant edits/masks/support match the prior baseline. Only Lance's common
map fields change:80 formerly flat north cells gain lance:north-portal. Only
its mask1/3 edited-cell classifications change, with16 north-door cells gaining
that family in each. All311 maps now report textured vertices/triangles; Rocket
B1F currently has168040 textured vertices/58160 triangles. New textured fields
had no prior baseline, so they do not prove unaffected textured counts.
Receipts: printed-bed-portal-full311-audit.{json,log},
printed-bed-portal-full311-invariant-summary.json and common-diff.json.

The new actual-input alarm regression also exits0: one test14.44s proves all
eight entry tiles, both real Grunt victories per entry, consumed PP, earned
camera flags, cleared re-entry and production Start release. Native-compatible
earned saves exist for all eight. Five corresponding native LR walkthrough
pairs complete and were individually inspected; traces witness both neighbor
and sensor coordinates with map unchanged. All five backing joins remain
contiguous. However several default north-camera poses obscure the player
behind foreground walls (camera2 fully): these are not camera-readability
acceptance. RocketWall currently does not opt into the authored cutaway ranges
used by the existing occluder system. Keep ceiling-height connected geometry;
foreground reveal/readability needs a proper scoped fix. Artifacts:
rocket-earned-native-walk-review; receipts/native-earned-rocket-controller-
tests.log. Initial party is seeded; victories and camera flags are earned.

Two defects found during these reviews are corrected in source, still awaiting
new native views: blanket drawing outlines must not stretch down the duvet
thickness as a black band, and the portal floor must not project out behind the
north wall as a pale shelf. Printed upper cloth now retains full live art while
rounded sides sample their own live cloth pigment at5,10; geometry is unchanged.
The closed threshold ends at wall depth; the open recess includes the actual
warp tile plus dark source-painted returns/ceiling/backing behind both lanes.
The bed checks now require plain cloth sides and printed top; portal checks
require no rear shelf, clear warp-tile centers and backing behind those centers.
Both bed tests pass0.02s, all three portal tests pass0.00s and all52 catalog
checks pass0.24s. Receipt: bed-cloth-side-portal-recess-tests.log and
cloth-recess-{kanto_lance,interior_models-tests}-tests.log.

Added opt-in CRYSTAL_RENDER_TEST_HOLD_FIELD_NOTICE to the QA input route. It
releases keys only when the live notice's exact requested page is fully
revealed, freezes paired presentation like the existing healing capture, and
writes observed page/text/wallet evidence. A missing target fails the capture;
it never changes source text, menu selection or economy. This new QA path must
still compile and pass actual native captures. Driver34799/Python28440 owns
native build28883; driver88097 waits for success, then captures six beds, four
portal views and two held notice pages requiring wallet800 after one purchase,
followed by another full311/225 audit. New scripts and receipts use cloth-recess
prefixes. All captures remain explicitly headless/image-targeted/muted.
The complete all311/original16/shared-furniture/earned native scene objective
remains unfinished; neither scoped counts nor selected camera angles replace it.

Enclosed portal and both exact notice pages verified; final cloth UV seam pending — 2026-10-17

Native build28883 exits0 in6m22s. All12 offscreen/muted pairs and the full311/225
audit complete successfully. The four same-state portal comparisons were
individually inspected: the closed threshold no longer protrudes behind the
wall; the open passage has dark enclosed returns and a real clear threshold.
This accepts the scoped architectural finish, not native earned League scene
coverage. All six bed source PNGs are byte-identical to the already inspected
references. Their new native front/side views were inspected individually.

Both opt-in notice holds succeed on the actual first purchase. Page0 is fully
revealed28 characters (CHRIS put the / FRESH WATER in); page1 is fully revealed31
characters (FRESH WATER in / the ITEM POCKET.). Both observed-wallet receipts
are800, proving one200 purchase from the funded1000 save. Both source/native
pages were individually inspected with no menu underneath, and paired-state /
camera checks pass. This proves the new QA hold compiles and catches the real
production presentation rather than relabeling timed captures. Artifacts:
cloth-recess-headless-review/vending-held-notice-page{0,1}-{2d,3d}.png and each
3d.notice-hold.txt. Whole-room service acceptance remains separate.

The refreshed audit has311 verified maps,225 variants, zero mesher errors and
all map/variant records exactly equal to the prior completed textured audit.
Count equality does not capture closed-threshold relocation or UV changes;
those are established by explicit geometry checks and native views. Open-recess
added faces are not reported as full variant geometry counts. Evidence:
cloth-recess-full311-{audit,invariant-summary}.json and audit.log.

Close bed inspection catches thin UV interpolation streaks at the printed /
plain transition, plus the source frame's black outline still painted on the
upper cloth edge. New implementation splits attribute vertices only across
that UV seam and assigns a whole triangle to printed or plain cloth, preventing
an atlas sweep across a transition triangle. It excludes the two outer source
frame columns from the cloth print; the separate native rails/posts retain
those pigments. Every fitted triangle corner and smooth normal is preserved.
The regression now compares exact indexed triangle corners/normals, requires
constant pigment for every plain triangle and checks west/east/head/foot print
orientation explicitly at all three scales. Only index/UV seam duplication may
change, not the rendered surface, source claims or support. These latest seam
changes are not yet executed or visually accepted.

Cargo30447/session42259 is live running the updated bed tests; receipt:
bed-uv-seam-cloth-tests.log. Native driver70819/Python30927 waits for both bed
passes, then runs three portal and52 catalog controls from the fresh viewer
binary before building the tools. Capture driver23572 waits for that build,
then captures only the six changed bed views and refreshes the full311/225
audit. Receipts use bed-seam prefixes. Other completed source/service/earned
sensor evidence remains scoped. Foreground Rocket wall occlusion and the full
all311/original16/shared-furniture/native earned-scene objective remain open.
All rendering stays headless, image-targeted and muted, with no windowed fallback.

Final bed seam verified offscreen; Rocket wall reveal in implementation — 2026-10-17

The updated bed tests pass (two viewer tests; Bevy has no matching test), as do
three Lance portal and52 model-catalog controls from the fresh test executable.
The native tool build exits0 in4m10s. All six Pink/Polkadot/Pikachu front/side
source/native pairs complete headless, image-targeted and muted. Each source
and native image was inspected individually: rounded plain cloth sides retain
their own pigment, the former black source-frame stripe is absent from the
cloth, and no interpolated atlas streak crosses the print/plain seam. The
correct source motifs and gray frame remain. This accepts the scoped bed-print
finish, not the bedroom's entire composition or earned decoration acquisition.
Initial ownership is seeded; actual PC selection and room reload use production
input. Evidence: bed-seam-headless-review/{pink,polkadot,pikachu}-print-{front,side}
images and paired-state receipts; bed-uv-seam-cloth-tests.log,
bed-seam-native-{driver,build}.log. All311 map records and225 source-block
variant records exactly match the prior cloth-recess audit, with311 verified
maps and zero mesher errors. UV appearance is established by image inspection,
not those equal counts. Evidence: bed-seam-full311-{audit,invariant-summary}.json.

Earned Rocket LR walks previously exposed default-camera player occlusion.
New rocket_wall_reveal.rs partitions only verified native RocketWall source
ownership into closed rectangular courses, retains every original floor outside
reveal ranges, and keeps buried course ends capped. Whole Persian statues and
their wall backings receive separate reveal ranges. Existing camera-ray fade
logic owns presentation; collision, source tiles, authored claims, ceiling
height and earned sensor flags are untouched. Non-Rocket placements delegate
to the existing mesher. No shared renderer, AA or texture-filtering code is
changed. Three new geometry/source tests are compiling under Cargo33456 in
target/kanto-resume; their execution and a new native build/capture are still
pending. Do not claim the foreground-wall defect fixed until same-camera earned
LR routes and frontal backing controls have been inspected. The full all311 /
original16 / shared furniture / earned native scene scope remains open.
All future automated captures remain offscreen and muted with no window fallback.


Rocket reveal validation cache retry — 2026-10-17

The first wall-course test build exits101 before running tests: generated
mr__mime.glb metadata/payload were absent from this goal's Cargo output while
the canonical source model was being added by another goal. The source file is
now present; no species source, shared cache, or other owner's files were
changed. The failed log is preserved as rocket-wall-course-reveal-tests-attempt1.log.
Cargo34705 rebuilds in target/kanto-resume and is confirmed live. Native driver
34794 waits for all three exact wall-course passes and five Persian/source
controls before building tools; capture driver34837 waits for that success.
Seven headless/muted pairs are prepared: the five identical earned LR routes
at camera0/0/45/0 plus two front statue/backing controls. The route verifier
requires identical authoritative position sets to the prior captures. Full311 /
225-state auditing follows. These new wall results are still pending; previous
bed/portal/service/controller acceptance remains unchanged.


PokéCom office service controller coverage added — 2026-10-17

New ignored external-pack regression
kanto_pokecom_office_computers_serve_all_choices_and_release_input drives all
seven source branches: both information choices and explicit Quit at computers1
and2, plus computer3's email. Each fixture starts at the actual customer-side
background event, faces the terminal through input and saves a quiescent native
review checkpoint if requested. The test requires each own qualified source
menu, each branch's own text label, every fully revealed authored printer page,
directional input ownership, B cancellation after help, explicit Quit, Start
and ordinary walking afterward. No menu, source cursor, choice or text is
injected. Initial room placement is a fixture, not story-earned arrival.

The new file is included from the existing Kanto interior regression file;
production behavior is unchanged. Execution is pending. Driver waits for the
current seven Rocket pairs and311/225 audit to finish before starting Cargo in
the same owned cache. Receipt: native-pokecom-office-controller-tests.log and
pokecom-office-controller-driver.log. Native terminal/service appearance still
requires paired capture; geometry identity of the small gray office fixtures
and whole-room acceptance remain open. All captures remain offscreen and muted.


Rocket wall-course and Persian source checks pass; native build running — 2026-10-17

Cargo34705 exits0 after11m18s. All three new rocket_wall_reveal tests pass in
3.72s against the native pack, including the actual-map triangle multiset
preservation check, unchanged source/support and closed independent wall-course
ends with all floor triangles outside reveal ownership. The Bevy executable
has no matching tests and earns no controller coverage here. All five Persian
source/backing/anatomy controls also pass from the fresh viewer executable in
4.92s. Receipts: rocket-wall-course-reveal-tests.log and
rocket-reveal-persian-controls.log. The earlier cache-generation failure is
superseded by this actual execution; no source model was replaced.

Native Cargo36848 is confirmed live building the three tools in the owned
cache. Driver34837 will then run the five exact earned LR routes, two frontal
backing controls and the full311/225 audit, all headless/image-targeted/muted.
Native foreground-wall readability remains unaccepted until those views are
inspected. The PokéCom service regression is implemented and queued behind
that pipeline; it is not included in the already-completed Bevy executable.
All311 room/state/service, original16 alternative/reachability and broader
shared-furniture/native earned-scene requirements remain open.


Rocket foreground reveal verified in earned native walks — 2026-10-17

Native Cargo36848 exits0 after6m47s. All seven paired captures and the full311 /
225-state audit complete headless/image-targeted/muted, with no visible window.
All14 source/native final images and10 native movement images were inspected
individually. In all five earned LR routes at the original camera0/0/45/0,
player head/body/feet remain readable; camera2 no longer hides the entire
trainer. Course and whole statue/backing fade applies only to foreground
occluders. Both front controls retain a continuous opaque full-height wall
behind the statue; no missing end is exposed beside a fading course. Scoped
foreground readability/backing acceptance is now established. Ceiling/open
black roof presentation and first-person companion composition are still open.

The routes use the same earned saves and ordinary LR input as the before set;
all authoritative position sets match. Each new source/native pair preserves
its own game state and requested camera. All five new final source PNGs are
byte-identical to the before PNGs. Warm-up timing differs by a few engine/LCD
frames between the old and new runs, so their raw cross-run checksums are not
equal; do not claim exact cross-run frame identity. These are recorded explicitly
in rocket-reveal-headless-review/before-after-state-comparison.json. Initial
party is seeded; both camera Grunt victories in these saves were earned by
the already-passing real-input controller regression. No flag/script injection
is used in these native walks.

The audit has311 verified maps,225 variants and zero mesher errors. All variant
records and all other map records are exactly equal to the final bed-seam
baseline. B1F changes only textured vertices168040→169336 and triangles58160→58592,
from the newly retained buried ends of independent wall courses. All source
claims/classification and support counts/extrema remain unchanged. Evidence:
rocket-reveal-headless-review/, rocket-reveal-full311-{audit,invariant-summary}.json,
rocket-reveal-native-{driver,build}.log and the three wall/five Persian test logs.

The broader all311 room/state/service, original16 alternative/reachability,
shared-furniture and earned native League/Hall scene acceptance remains open.
Cargo38504 is confirmed live compiling the seven-branch PokéCom office service
regression next, in the same owned cache after the Rocket pipeline finished.
No pack, audio, model source owned by another goal, shared AA/filtering code,
collision or source scripts were changed by this reveal work.


Fresh mansion/Warden views expose stool UV and shape defect; fix implemented — 2026-10-17

Four new headless/muted pairs complete with preserved state and requested
cameras: CeladonMansion1F overview and clock view at(4,5), WardensHome overview
at(7,4) and stool/table close-up at(8,3). All eight source/native images were
inspected individually. The mansion's clock/bookshelf/blue-wall/gray-cap and
floor composition is visible; clock relative height against its flanking
bookcases needs source-based review before whole-room acceptance. First-person
black-roof presentation and actor/companion composition remain open.

Warden's source shows compact round cushioned stools. Native close-up instead
shows a square pink U-shaped cap with a dark gap. Actual house tiles02/03 have
background rows0..4; their oval cushion is rows5..11, followed by feet. The old
rectangular helper incorrectly painted rows1..7 over the cap, including those
background pixels. New native_house_stool.rs emits a complete32-segment round
beveled cushion with its highlighted center, main upholstery, side and dark
feet sampled from actual seat/foot pixels. Four feet overlap the underside and
fit within it. No sprite background or foot outline is projected over the cap.
Only the already source-complete reviewed `house` stool branch changes; player
home round wood stools and cafe gray rectangular seats retain their paths.
Claims, support heights, collision, object positions and scripts are unchanged.

The prior rectangular-house expectation is updated to require a complete
projected disc and rounded sides. New checks cover outward/closed geometry and
constant own-pigment triangles at three scales, actual packed source pigments,
and all four Warden stools' independent claims/support. These tests are not yet
executed. Driver waits for the running office-controller Cargo38504, requires
its actual pass, then compiles/runs the three native_house_stool checks including
the ignored pack case. Evidence: mansion-warden-headless-review/ (before views),
house-stool-test-driver.log and pending house-stool-round-source-tests.log.
A new native build and same-pose after views plus311/225 audit are required;
this new stool finish is not visually accepted yet. Full original scope remains
active. No windowed fallback is allowed.


Office email exposes renderer-neutral jumptext acknowledgement bug; repair compiling — 2026-10-17

The new office regression executes after12m51s compilation. All six choices
for computers1/2 pass: both complete help streams, explicit Quit, B cancellation,
Start and walking. Computer3 fails because its own source text label is already
absent immediately after A. This is not accepted as a passing seven-case test.
The failed result is preserved in native-pokecom-office-controller-tests-before-jumptext-fix.log.
The first stool-only gate correctly stops rather than bypassing this failure.

A production stdio-MCP replay from the actual computer3 checkpoint exits0 with
empty stderr but falsely reports no dialogue after A; an opt-in dialogue trace
shows the email's first page created and immediately lost. The actual graphical
path instead retains the correct first page. Its headless/muted source/native
pair was inspected individually: The ADMINISTRATION / OFFICE received an is
fully visible in both. Receipt: pokecom-email-graphical-baseline/ and its log.
This baseline uses the prior Rocket native binary and is not evidence that the
new controller repair or stool geometry has compiled.

Tracing the production controller finds its settling loop automatically takes
a pending TextLabel before respecting jumptext/farjumptext's paired closing
button wait. That also acknowledges the wait and closes the entire document.
The renderer already retains that same boundary. New guard returns input for
a closing text wait when automatic input is disabled; only real A/B page
acknowledgements may finish it. No script/text/menu state is injected and no
second dispatcher is added. The existing explicit auto-input smoke path is
unchanged. Production diagnostics remain opt-in; no unconditional logs added.

Cargo42485 is confirmed live rebuilding both test executables with this guard
and the new round house stool. New driver waits for actual executable receipts,
then requires all seven office branches, three stool checks, player wood and
two cafe controls, Mom/NewBark and battle-command controls. It then requires
actual stdio-MCP movement/Start and clean-stderr regressions before a fresh
native build. Logs use house-office and house-stool prefixes. The prior
stool-only driver is terminal after its gate assertion, not a live build.
All new controller/stool results remain pending; full all311/original16/shared
furniture/native scene scope remains intact. No visible windows may be opened.


Exact source-menu capture hold added; house/office verification queued — 2026-10-17

RenderWalk gains opt-in CRYSTAL_RENDER_TEST_HOLD_MENU_ID. After ordinary QA
input reaches that exact live ScriptDefinition menu with selectable source
options and no notice/item overlay, it releases keys and freezes only the
paired presentation. The receipt records the observed menu ID, map, tile and
options. It does not open a menu, select a row, set a cursor or mutate source
state. Exhausting the route without the target fails. Normal gameplay never
installs this render-QA input resource. This addition still needs actual build
and native-capture execution; it is not included in the test executable that
was already compiling when this late QA-only change was written.

Driver42857 still waits for actual combined test-build receipts, then runs the
office/stool/source/controller and stdio controls before building native tools.
Capture driver44503 waits for its actual success and prepares five paired
views: both identical Warden before poses, office source menus1/2 through the
new exact hold, and the email's first page. It then runs the full311/225 audit
and replays the rebuilt stdio MCP against computer3: text visible after A,
directions blocked, ten pages acknowledged, Start released, stderr empty.
These results are pending. Relevant logs: house-office-{native,headless}-driver.log,
house-stool-office-jumptext-build-tests.log, house-office-full311-audit.json and
house-office-email-stdio.jsonl. The active full original objective is unchanged.
Every capture explicitly uses headless1, image target and mute1; no windows.


Round house stool/source checks pass; office reader condition corrected — 2026-10-17

Combined build42485 completes successfully after9m26s. Direct execution from
its actual fresh viewer binary passes all three native_house_stool checks in
4.12s: full packed source pigments/four Warden claims, outward complete disc at
three scales and integration's round cap/unchanged low height. Player-home wood
and both gray cafe source controls pass. The actual native-pack Mom/NewBark
control passes5.08s and battle-command/turn/PP control passes5.01s. These results
verify the unchanged stool and controller-guard code; they do not establish a
new native stool finish. Logs use house-stool-{round-source-tests,player-wood-control,
cafe-gray-control,cafe-front-control} and house-office-{mom-gate,battle-command}-control.

The repaired closing-text wait now retains the email's first fully revealed
page instead of losing its text label. The seven-case reader then stops after
that first page because its own completion predicate checked only generic
ui.window_open, which jumptext need not set. The authoritative text window is
still open. Corrected predicate also requires !ui.text_window_open; all ten
expected pages and independent source passages remain mandatory. No acceptance
assertion is relaxed. The partial run is preserved in
native-pokecom-office-after-jumptext-guard-before-reader-fix.log; it is not a
passing email test. Both old build/capture drivers are terminal at their gates.

Cargo46399 is confirmed live rebuilding the corrected reader and late exact
source-menu QA hold. Driver46583 requires its executable receipts and complete
seven-case pass; the already executed unchanged stool/source/controller controls
are reused, not gratuitously rerun. It then requires the real CLI MCP/stdout
controls and builds tools. A new headless driver waits on46583 and retains the
same five paired views,311/225 audit and email stdio checks. Receipts now use
house-office-menu-hold-build-tests.log and house-office-{native,headless}-driver.log.
Full original acceptance remains open, including actual native stool/office
views. Source investigation also traced the three gray fixtures beside office
monitors to complete C6/C7/D6/D7 drawings in VRAM bank1 (192-tile sheet uses its
96-tile half offset). Their geometry remains unmodeled pending that source
assembly review; do not silently classify them as benches. No windows opened.


All seven office controller branches pass; CLI/native verification running — 2026-10-17

Cargo46399 finishes successfully after4m42s and compiles the exact menu-hold QA
addition. The corrected seven-case office regression passes5.18s from the fresh
Bevy test executable. Both help choices and explicit Quit for each menu, B
cancellation after help, all ten fully revealed email pages with independent
source passages, directional ownership, Start and ordinary walking all pass.
The closing-text wait repair is now exercised; the prior partial page result
is superseded by complete execution, not by accepting fewer pages.

The already-executed three stool tests and five unchanged wood/cafe/Mom/battle
controls remain passing and are reused. CLI Cargo47567 is confirmed live testing
actual production MCP movement/Start; stderr-clean regression follows, then a
fresh native build. Capture driver remains gated on that actual success and
will inspect five paired views, the full311/225 audit and rebuilt email stdio.
Native stool/office finish acceptance and the full original goal remain open.
Evidence: house-office-menu-hold-build-tests.log,
native-pokecom-office-controller-tests.log and house-office-{native,headless}-driver.log.
All rendering stays offscreen, image-targeted and muted.


Production CLI movement passes; actual TUI-pack stderr/movement also verified — 2026-10-17

The rebuilt geothite MCP movement/production-Start regression passes4.34s with
the native pack after4m55s compilation. That same Cargo invocation also builds
the actual stdio-clean executable (its first filter selected zero tests there;
that zero-test result is not a stderr pass). Directly executing the fresh
main-test and stdio-clean executables with the completed text-tui.crystalpack
now passes each real one-test regression; the actual Mom-dialogue stdio check
passes2.98s and requires byte-for-byte empty stderr. No fullscreen UI/window is
opened. Logs: house-office-mcp-control.log,
house-office-text-tui-mcp-control.log and house-office-text-tui-stdio-clean.log.

Native-pack stdio Cargo48436 remains confirmed live compiling; concurrent
species-model source updates invalidate that renderer dependency. Do not stop
another owner's compiler or revert those models. Driver46583 continues to gate
the native build on its actual one-test pass, and offscreen captures remain
queued behind that build. The stool/office visual finish and full original
all311/original16/shared-furniture/native-scene acceptance remain open.


Round house stool native finish accepted; office blank-label defect repaired in code — 2026-10-17

Native Cargo49309 exits0 after6m29s. All five headless/muted pairs, the311/225
audit and actual rebuilt email stdio checks complete successfully. Every one
of the ten final source/native images was inspected individually. Both Warden
views now show a solid round padded seat, its source cream/pink/red pigments,
and connected dark feet. The former square notch/open cap is gone. Both source
PNGs are byte-identical to the before views. This accepts the scoped stool
finish; roof/companion/actor composition and whole-room acceptance remain open.
Evidence: house-office-headless-review/warden-{overview,stool-table}.

The audit verifies311 maps and225 variants with zero mesher errors. All source
claims/classifications, support counts/extrema and all225 variant records are
unchanged. Only textured counts change, across56 house-stool maps, by exactly
306 triangles per source stool (four stools per changed map); the new closed
round geometry explains each delta. Evidence: house-office-full311-{audit,
invariant-summary}.json. This is shared Kanto/Johto source-family proof, not
whole-map visual acceptance for all56 houses.

The exact source-menu hold works for both office menus and records their actual
raw choices. Native and2D menus both show blank option text because these two
English-office source headers retain Japanese labels unsupported by the English
bitmap font. These menu images are explicitly rejected for readability; the
passing seven-case gameplay test is not a visual pass. The current native email
first page is fully readable, and all ten pages separately pass through actual
MCP with zero stderr, correct directional ownership and released Start.

New shared visible_script_menu_option_label in crystal-bevy presents these six
exact scoped source choices in readable English, fitting the authored13-cell
column. The two qualified menu IDs, owning ScriptDefinition map and exact raw
option strings all gate the labels. Raw choices, source program, cursor/index,
branch values and pack bytes stay unchanged. Unknown/foreign options retain
their original text. Both graphical paths and the TUI use this one formatter.
The label contract tests source scope/font width; the actual seven-case
controller regression also requires each selected menu's display labels while
still executing its original branches. These changes are not yet executed.

Cargo51970 is confirmed live building all three lib test executables. Driver52312
requires the label/unit and complete source-controller passes plus actual CLI
stderr control before building tools. A new offscreen driver then captures only
the two changed menus and verifies both English label sets and B→Start through
actual stdio; the unchanged terrain audit/stool/email proof is reused. Receipts
use office-visible-label prefixes. The full all311/original16/shared-furniture /
earned native-scene objective remains open. Gray office-unit clarification is
pending from the user; independent verification continues. No windows opened.


Mansion bookcase/clock proportions corrected in source; verification queued — 2026-10-17

The individually inspected saved mansion clock pair confirms a source/native
scale mismatch. Mansion block0f bookcases occupy three source rows (24 pixels),
while the adjacent clock occupies four (32 pixels). Native generic shelves at29
pixels nearly reach the30-pixel clock, erasing the source's height difference.
Only complete source-matched mansion corridor Bookcase placements now use22
pixels, retaining the same two-pixel allowance as the30-pixel clock. Other
bookcase families, cabinet depth, source claims, footing and scripts are unchanged.
The existing external mansion regression now checks all matched shelf heights
and actual emitted geometry, while retaining source corruption, neighbor,
whole-object reveal and opaque-floor controls. This change is not yet tested
or visually accepted. The sequential mansion-height review driver waits for the
currently live office pipeline, then runs the real-pack regression, builds
native tools, captures two offscreen/muted source/native pairs and refreshes
all311/225 audit records. Receipts use mansion-height prefixes. No windows or
application navigation are used; the deleted handoff remains deleted.

Lighthouse hidden-area review queued behind verified native build — 2026-10-17

The saved Lighthouse6F source/native pair was inspected individually again.
Its central furnishings and moss courses are visible, but large foreground
wall caps obscure surrounding corridors. That center pose cannot establish
whole-room acceptance or wall joins from other sides. Existing production
render_at_location --perimeter selects eight boundary targets and deduplicates
nearest standable non-warp/non-ledge tiles using the runtime collision data.
A sequential offscreen/muted review job now waits for the mansion source/native
and311 audit gates, then captures these perimeter poses on all six lighthouse
floors at hour12 and camera0/0/45/0. It checks each actual paired-state receipt
and records manual review as pending. These are disposable initial-room visual
fixtures, not earned arrival or quest-state evidence. No new compiler/exporter
is started by this queued capture job. Evidence will use lighthouse-perimeter
prefixes under ignored target/kanto-resume. All311/original16/shared-furniture
and earned native-scene obligations remain open.

Office display/controller and mansion height regressions pass — 2026-10-17

The current three-package test build exits successfully after15m40s. Its actual
fresh Bevy executable passes the office label width/scope contract and all seven
real-pack office branches in8.87s, including all ten email pages, source choice
semantics, text input ownership, cancel, Start and walking. The fresh viewer test
executable passes external_mansion_corridor_furniture_reveal_owns_whole_objects_but_not_floor_or_neighbors
in7.75s: all four source-matched mansion shelves use22-pixel geometry, the clock
retains30 pixels, floor/footing and complete-object reveal/source guards remain
valid. Evidence: office-visible-label-{unit,controller}-tests.log and
mansion-height-current-source-test.log. The rebuilt native-pack CLI also passes
its actual one-test clean-stderr gameplay regression in4.72s after6m37s
compilation (office-visible-label-stdio-clean.log). Native tools are rebuilding
against the current render API; menu readability and mansion proportions are
not yet visually accepted. The existing sequential capture drivers remain live.

The original16 alternatives were rechecked against the completed311/225 audit
and the pinned external source. Audit masks describe block combinations, not
necessarily persisted flag combinations or chronological states:

| Maps | Mask1: exit only | Mask2: entrance closed | Mask3: both block edits |
| --- | --- | --- | --- |
| Will, Koga, Bruno, Karen | Disposable visual fixture; regular entry closes the entrance before a victory opens the exit | Authored entry scene, exercised by earned controller traversal | Authored victory opens exit while retaining closed entrance; four real wins and exit traversals already pass |
| Lance | Disposable visual fixture; no source setter for EVENT_LANCES_ROOM_EXIT_OPEN | Authored entry scene | Victory directly opens the north block while retaining entrance closure; the source sets ENTRANCE_CLOSED again rather than EXIT_OPEN, so callback reconstruction from flags differs |

These are fifteen declared combinations. The sixteenth is CeladonDeptStore6F's
single tiles-callback changeblock12,0,$03, applied unconditionally to hide the
rooftop stairs. The baseline room is additional. The four ordinary rooms use
exit block$16 and entrance block$2a; Lance uses$0b and$34. The audit's visual-cell
coordinates are twice those source changeblock coordinates. Indigo Plateau's
entry reset clears all ten door flags; it does not create an earned exit-only
case. Source inspection does not replace native evidence for every alternative.
The prior actual continuous five-win/Hall/Continue controller result remains
verified at72.61s; full native earned-scene coverage and all311 fidelity remain
open. Do not add Lance's missing exit flag or remove synthetic audit states to
make accounting simpler.

Actual office CLI label presentation and cancel→Start verified — 2026-10-17

The rebuilt geothite mcp client loads each current office checkpoint and reaches
its menu through three ordinary A inputs. Actual rendered text shows all three
English options in each menu: POKECOM CLUB / MOBILE CENTER / QUIT and USING PHONE /
NO CONNECTION / QUIT. B produces menu=null, dialogue=false and mode=overworld;
Start then produces the production visible-shell:text-frontend menu with
mode=menu. Both positions remain unchanged at(6,27)/(6,29), and both processes
exit successfully with zero stderr bytes. These checks inspect the actual text
frontend, beyond the formatter/controller contract. Evidence:
office-visible-label-current-cli-menus.json. Native 2D/3D label appearance is
still pending; its existing headless driver is waiting for the live native
rebuild. Full all311/original16/shared-furniture and earned-scene scope stays open.

Office menu labels and mansion bookcase/clock height visually accepted — 2026-10-17

Native tools finish successfully in3m59s. Both new held-menu source/native pairs
complete offscreen and muted, preserving each paired state at camera0/0/45/0.
All four final images were inspected individually. Each menu displays three
readable English labels, its first-row cursor and the dialogue below, with no
border overflow. This accepts the scoped office menu-label repair, superseding
the earlier rejected blank-label images. Raw source choices and branches remain
unchanged. Actual stdio English-label/B→Start checks pass for both; the freshly
built stdio-clean executable also passes its real one-test text-tui-pack check
in0.76s. Evidence: office-visible-label-headless-review/ and
office-visible-label-{native-driver,headless-driver,text-tui-stdio-clean}.log.
Gray monitor-side fixture identity/geometry and whole-room fidelity remain open.

The current real-pack mansion height/source regression already passes7.75s.
The queued standalone Cargo invocation would rebuild Bevy with a different
feature set to repeat that same check. Only that owned redundant compile and
its dependent waiting drivers were stopped, preserving its log as
mansion-height-redundant-feature-build-stopped.log. No compiler failure or
external blocker is inferred. A revised driver requires the actual successful
source-test, native-build and office/TUI receipts, confirms the native tool is
newer than the changed geometry source, and reuses it for both mansion pairs
and the full audit; no build cache or other goal's process is touched.

All four new mansion source/native images were inspected individually. In both
the overview and close view the30-pixel clock now rises clearly above the22-pixel
bookcases, matching the source's four-row/three-row drawings; fixtures remain
grounded. This accepts the scoped height correction. First-person black roof,
actor/companion composition and whole-room fidelity remain open. Evidence:
mansion-height-headless-review/ and its updated capture-provenance.json.

The actual new all311/225 audit succeeds with zero mesher errors. Every root
field, map record and variant record is exactly equal to the completed
house-office audit. This preserves reported source ownership, classifications,
geometry counts and footing summaries; count equality alone is not height
proof, which comes from the actual geometry regression and inspected views.
Evidence: mansion-height-full311-{audit,invariant-summary}.json. The restarted
lighthouse-perimeter driver is confirmed live at PID59619, consuming these
completed tools for all six floors. Full all311/original16/shared-furniture and
earned native-scene acceptance remain open. No windows opened.

Lighthouse1F eight perimeter pairs individually inspected — 2026-10-17

All sixteen actual source/native images for northwest, north, northeast, east,
southeast, south, southwest and west were inspected individually. Each pair
preserves its own source state at camera0/0/45/0. The player remains readable
in every checked pose; checked wall joins remain continuous and green moss
courses are present. The south entrance remains visible and the southwest
stair geometry is seen at its adjacent pose. Foreground wall caps partly hide
the northern NPC and the stair approach from some farther poses. Those need
side-angle and actual traversal review before whole-room acceptance; do not
change collision or shrink architecture from these static images alone.
Provenance marks only the scoped wall/moss/player review and explicitly keeps
whole_room_accepted=false. Evidence: lighthouse-perimeter-headless-review/
OlivineLighthouse1F-* and its capture-provenance.json. Floors2–6 continue in the
same serial offscreen/muted job; no all-floor visual pass is claimed.

Lighthouse2F perimeter review exposes nearby trainer occlusion; side/warp checks running — 2026-10-17

All sixteen actual source/native images for the eight2F perimeter poses were
inspected individually. Each preserves its paired state. Player readability,
checked joins and moss courses remain present. The northern inner cap can
completely hide the nearby trainer at the north pose; stair/pit approaches also
need side views. Whole-room acceptance remains false. Static occlusion alone
does not justify changing collision, lowering ceiling-height walls or reversing
stairs. The source1F stair at(3,11) is block$3a leading to2F, consistent with the
positive-rise native flight. Independently reading all six pinned source maps
finds fourteen stair warps; block$31 always leads down a floor and$3a up a floor.

A new serial offscreen/muted driver waits for the existing six-floor perimeter
job. It captures west/east side views of the1F stair,2F northern trainer and2F
pit, then starts immediately west of each of the fourteen source stairs and
uses ordinary Right input. Actual movement CSV runtime map/tile must equal the
source warp's destination and landing tile; no coordinates, flags or warp
outcomes are injected after initial disposable room placement. It records
initial arrival as unearned and subsequent input-driven destination arrival
separately. Every resulting image still needs individual visual review. No new
compiler/exporter is started. Evidence uses lighthouse-stair-input prefixes;
2F provenance records only the scoped wall/moss/player inspection. Floors3–6
and full all311/original16/shared-furniture/earned-scene scope remain open.

Fourteen native stair warps pass; side views confirm unmodeled flat pit — 2026-10-17

The serial offscreen/muted job completes all three side pairs and all fourteen
ordinary-Right stair cases successfully. Each case starts immediately west of
one authored stair as a disposable room fixture; the subsequent native keyboard
input actually crosses its warp. Every CSV final runtime map and tile matches
the pinned source destination and landing coordinate, and each final source/native
pair preserves its own state. Initial arrival remains unearned; this is fourteen
independent real warp crossings, not one continuous earned six-floor ascent.
The fourteen final movement pairs still require individual visual inspection.
Evidence: lighthouse-stair-input-headless-driver.log and its capture-provenance.
All48 perimeter pairs across six floors have also completed; only1F/2F's sixteen
pairs have been individually inspected so far. Floors3–6 remain pending review.

All ten images in the five static side pairs were inspected individually,
including opposite rotation6 views of the1F stair and2F pit. Rotation6 makes
the previously hidden2F northern trainer and player readable, accepting only
that scoped actor-view result. The1F stair is source-correct in rise and its
actual crossing works, but side cheeks/caps still dominate the checked side
views; no full stair appearance pass is claimed.

The2F pit is rejected for missing readable3D depth. The current audit confirms
all sixteen source cells of block$28 remain unmodeled flat; the lower eight
black tile$01 cells are the pit. The floor matcher deliberately excludes that
lower half, and the masonry matcher excludes pits entirely. A proper next fix
needs a recessed native shaft/opening with strict complete-block admission:
upper eight cells alternate$2e/$2f source floor, lower eight are$01. Claim only
the lower eight, preserve the upper source floor, all support/collision and
fall-warp behavior, and reject altered/clipped/foreign drawings. Do not fill
it with a floor or classify it as a wall. Add actual pit/fall input checks and
inspect the same two side cameras after geometry changes. No production pit
change is implemented or accepted yet. Full all311/original16/shared-furniture
and earned native-scene obligations remain open. No windows opened.

Native lighthouse pit implementation added; source/full-scene tests compiling — 2026-10-17

New lighthouse_pit.rs replaces only the lower eight black cells of an exact
source-complete block$28 with an open beveled mouth, inward-facing stone shaft
and black bottom16 pixels below the floor. Upper source floor cells remain
unclaimed. Its three pigments come from this same drawing: tile$2e's mid/dark
floor pixels and the pit's black tile$01. All vertices remain at or below the
original floor; support, collision, source events and warp handling are untouched.
The pit is not a room wall or fading owner, and cuts_foundation describes its
open topology. Every one of the block's sixteen tile identities, tileset,
metatile and phase must match; clipped/changed/foreign drawings retain fallback.
Pinned source inspection finds six instances across2F/3F/4F/5F/6F (1/1/2/1/1).

Four focused regressions cover complete-source admission/negative controls and
eight-cell ownership, inward winding and closed lower topology at three scales,
the full production mesher's lack of any floor bridge, and all six openings in
the actual external pack with untouched footing and verified source pigments.
The unified three-package/location-tester Cargo63829 is confirmed live compiling
the viewer test executable; these tests are not yet reported passed. Diff checks
pass. Driver64765 requires all four actual passes, then fresh stair source,
masonry source and authoritative stair-controller controls before building the
native tools. Its dependent headless driver captures all six new pits (eight
static pairs), crosses all twelve pitfall events using ordinary Down input from
the northern source floor, and requires actual expected destination map/tile,
matching active terrain, no pending load/warp and zero fade. It then audits all
311 maps/225 alternatives. The twelve cases start from disposable room fixtures;
only their subsequent input-driven fall arrivals can be earned. No source pack
or event/coordinate outcomes are injected. Logs use lighthouse-pit prefixes.

Native pit appearance and fall behavior remain unverified until these gates
and individual image review complete. Earlier rejected flat-pit evidence is
preserved. Upper-floor perimeter and fourteen stair-arrival images still need
individual review; full all311/original16/shared-furniture and native earned
scene acceptance remains open. All rendering is explicitly offscreen/muted.

Pit geometry/source and twelve native falls pass; default-view visibility still open — 2026-10-17

All four actual pit regressions pass6.78s, including all six packed openings,
closed/inward-wound shaft topology, complete-block guards, untouched footing
and the production mesher's no-floor-bridge check. Fresh stair-source,
masonry-source and authoritative stair-controller controls each pass. Native
examples build successfully in5m17s. The first capture attempt correctly rejects
the4F close-up at(7,3), a wall tile. Its exact error and attempt logs are retained;
only that fixture moves to the original floor at(8,2), north of the same pit.
Four completed pairs are reused rather than rerendered. Geometry/collision and
expected fall outcomes are not weakened to repair the harness.

The resumed job completes eight static pit pairs and all twelve actual Down
pitfall inputs. Every destination map/tile matches source, active terrain matches
the destination, pending terrain is false, warp phase isNone and fade is zero.
Each pair preserves its state. All20 explicit initial fixture positions were
also checked against source actor positions. These are independent disposable
starts followed by real input-driven falls, not a continuous earned ascent.
The twelve fall-arrival pairs still need individual visual inspection.

The311-map/225-variant audit has zero mesher errors. Exactly five maps change:
2F/3F/4F/5F/6F contain1/1/2/1/1 pits. Each adds eight modeled cells, two net
textured triangles and four net vertices; total48/12/24. Only the lower$28
black cells leave unmodeled-flat classification. Upper source floor cells,
all other source keys/families/bounds and support summaries are unchanged;
306 map records and all225 variant records are identical. Root totals differ
only by48 modeled and minus48 flat cells. Exact bounds and deltas are verified
in lighthouse-pit-full311-invariant-summary.json, alongside the actual audit.

All six2F static source/native images (rotations0/2/6) were inspected individually.
The lining/recess is present, but foreground caps still hide the pit at the
default rotation0 and partly at rotation2. Do not accept pit readability from
source/topology/input passes or the side view alone. Follow-up must make the
nearby fall opening readable while preserving ceiling-height masonry, source
navigation and wall continuity. Other five static pairs still need individual
review. No broad pit visual acceptance is claimed.

The3F review also found the old north fixture at(9,2) on the original sailor.
render_at_location's batch sampler now excludes authored runtime object positions,
including startup-unloaded actors, in addition to its existing collision/warp/
ledge checks. The actual external-pack example regression passes3.82s and selects
(9,3). Both corrected source/native images were inspected: player and sailor
now occupy distinct tiles; the original sailor/item/architecture stay intact.
Old north evidence is explicitly rejected, and only this corrected fixture is
scoped accepted. NW/NE/east3F before-pit pairs have been inspected; the other
four perimeter poses, floors4–6, fourteen stair-arrival pairs and twelve fall
pairs remain pending individual review. Full all311/original16/shared-furniture
and native earned-scene scope remains open. No windows opened.

Nearby lighthouse pit reveal added; body-only ray limitation addressed in code — 2026-10-17

The remaining default-view pit occlusion comes from whole-object fade selection:
its nine rays end on the player capsule, so an adjacent pit can be blocked by a
foreground cap that does not cover the player. New source-scoped floor targets
reuse lighthouse_pit.rs's exact complete-block guard. Only an opening within two
walking tiles contributes its mouth center and four inset corners. Coordinates
include the actual visual-grid world offset; malformed/duplicate/clipped/altered
or foreign sources and distant openings contribute nothing. Camera-behind
points are removed. The player's existing capsule/radius remains unchanged.

The fade system consumes these extra targets only with an active camera/player
and an authored built map matching the current map. Actual retained triangle
rays still decide blockers. Existing camera-plane outside-room behavior remains;
pit rays independently detect blockers for those walls too. Existing reveal
amounts, timing, shadow geometry, source meshes, ceiling heights, support,
collision, scripts and warp outcomes stay unchanged. The pit lining/floor have
no fade ownership, so they remain opaque. Previous target lists participate in
change detection, letting blockers restore when the player leaves or the map
changes. Other maps/unknown drawings retain the body-only path.

Two new regressions exercise complete-source/near-distance/world-offset targets
(including array reorder, corruption, duplicate and foreign-map controls), and
an actual triangle cap that blocks the pit but none of the player's original
rays. The unified build now passes all six pit/reveal tests in5.41s. The actual
fresh viewer executable also passes all41 occluder-fade tests in0.43s, including
real ECS camera/map replacement and gateway/concavity controls, plus both
unchanged capsule controls. Driver70558 is confirmed live building native tools;
these source/behavior passes do not establish rendered pit readability.
The queued headless job then captures all six pits in the default view, both
2F side views, a far-from-pit lighthouse control and the Rocket backing control.
It also performs one actual fall/map transition with settled terrain/zero fade
checks and refreshes all311/225 audit records. Prior twelve fall inputs remain
valid behavioral evidence; new native opacity/appearance is still unverified.

No visual acceptance is inferred from implementation or tests. Individually
inspect every new pair for pit readability, connected fading/opaque joins,
retained masonry/moss and player composition. Full all311/original16/shared
furniture, upper-floor and earned native-scene obligations remain open. All
renders stay explicitly offscreen/muted; shared AA files are untouched.

Lighthouse pit reveal visually checked; perimeter fixtures corrected — 2026-10-17

The offscreen/muted native build completed in3m50s, followed by all eleven
matched source/native pairs, a real Down-driven2F→1F fall and the full311-map/
225-variant audit. All22 images were individually inspected. Each of the six
openings now has a visible mouth/recess in the default view;2F rotation2 also
shows depth. Rotation6 shows the lining behind the player but the body partly
overlaps its projection, so broader alternate-camera readability remains open.
Checked wall/moss joins remain connected, the adjacent floor remains opaque,
and player scale/capsule and ceiling heights are unchanged. The distant-from-pit
lighthouse control retains opaque masonry. The Rocket control retains continuous
opaque blue backing behind the Persian. Existing faceted wall lighting and
first-person ceiling/companion composition remain open.

The new fall pair visibly lands on solid1F floor with a readable player and
sailor. Its movement receipt confirms the actual source destination(16,13),
matching active terrain, no pending terrain/warp and zero fade. This is one
input-earned fall after disposable placement, not a continuous earned ascent.
Every parsed audit field is exactly equal to lighthouse-pit-full311-audit.json:
all311 map records,225 variant records, source ownership, support, geometry and
root totals; zero errors. The reveal change affects runtime opacity selection.
The equality receipt is lighthouse-pit-reveal-full311-invariant-summary.json.
Capture provenance now records these scoped findings, never whole-room approval.

All third-floor perimeter pairs are now individually reviewed, with the old
north(9,2) evidence rejected and its checked replacement(9,3) retained. All eight
fourth-floor pairs have also been individually reviewed: checked player and
masonry/moss joins are readable/continuous. They predate pit geometry and do not
prove the new pits. The northeast view partly hides its northern NPC; the south
view exposes floor-like source wall-base remnants during foreground reveal.
These remain composition/refinement work rather than broad room acceptance.

Source actor-position checking found another invalid old fixture:5F southwest
(2,13). It is marked rejected. The expanded external-pack example regression
passes in4.14s and independently covers both old actor positions, selecting3F
(9,3) and5F(3,13) while retaining all authored actors. The fresh location-tool
build passes in6.97s and the corrected5F source/native pair completes offscreen.
Both images are inspected: the item ball west of the player is separate, and
the sailor remains in the central aisle. Its paired state is preserved. The
fixture is valid, but the wall appearance is rejected: independently fading
masonry exposes an uncapped neighbor end beneath an opaque top. Source review
confirms lighthouse blocks cull buried neighbor sides even though each block
owns a separate cutaway range. Next repair must keep those independently
revealed boundaries closed without altering collision or source ownership.
The production sampler change remains confined to disposable QA placement.

Floors5–6 perimeter images, fourteen stair-arrival pairs and the remaining old
fall-arrival images still need individual review. Distant NPC occlusion, room
composition, all311/original16 alternatives, shared furniture, earned native
League/Hall sequences and the broader acceptance table remain open. Rendering
uses HEADLESS=1, CAPTURE_TARGET=image and mute, with no visible-window fallback.
Shared antialiasing files and external content packs remain untouched.

Lighthouse independent wall-end repair added; native proof pending — 2026-10-17

The corrected5F southwest pair exposes a hollow end when an adjacent masonry
block fades. Lighthouse masonry now retains buried model faces because every
block owns its own reveal range. The source-complete guard, floor restoration,
ceiling height, source pigment, orientation, rubble ownership, source collision
and events are unchanged. Only rubble skirts still cull shared sides. New
regression independently ray-tests emitted outward-facing body triangles from
all four sides, three samples per side, every native wall/window type and all
four rotations/reflections after neighboring bodies disappear. It also requires
the restored floor to stay outside the reveal range.

Unified Cargo79815 is confirmed live compiling the lighthouse/source/controller
tests; no pass claimed yet. Its dependent driver requires the actual new ray
regression plus external-pack masonry proof, then builds native location/audit
tools, captures six source/native pairs explicitly offscreen/muted (the5F defect
at rotations0/2/6,4F south and2F pit rotations0/2), and audits all311/225 records.
Current closed-end appearance, geometry deltas and all new checks remain pending.
All seven valid old5F perimeter pairs are individually inspected, plus the
corrected southwest fixture. Their player/checked wall courses are readable;
the northern trainer stays hidden by a cap, and foreground base remnants remain
open. The rejected old southwest pose is retained as invalid evidence. Floor6
poses and the full acceptance table remain open. No windows
or focus changes; external packs/shared AA files remain untouched.

Closed lighthouse wall ends verified; stair support defect identified — 2026-10-17

The unified lighthouse build completes in11m07s:36 viewer tests pass10.94s
(one separate ignored test), including the new outward-ray closure test and
actual-pack masonry/stair/pit guards; the authoritative stair-arrival/reentry
controller test passes4.39s. Fresh native location/audit tools build in3m37s.
All six source/native pairs complete explicitly offscreen/muted, and all twelve
images are individually inspected. The failed5F southwest view now has an
opaque masonry end under its top instead of a hollow opening. Both side views
retain closed/connected courses.4F south's adjacent opaque end is closed;2F
pit mouth/lining stays visible at rotations0/2. Player and neighboring item ball
remain separate. Scoped closure accepted; cap dominance, foreground base-floor
remnants and full composition remain open.

The311-map/225-variant audit has zero errors. Only six lighthouse map records
change, only textured triangle/vertex counts (buried body faces retained).
Triangle deltas for floors1–6 are35096/36408/36736/38048/35752/32800; vertex
deltas are exactly three times those counts. All other fields in those maps,
305 other complete map records, all root/non-map records and all225 alternatives
are exactly equal. The actual comparison is recorded in
lighthouse-closed-ends-full311-invariant-summary.json. This is closed original
masonry geometry, not changed source ownership/support/collision or game events.

All eight6F perimeter pairs are now individually reviewed. Together with prior
floors and the corrected3F/5F actor-clear fixtures, all48 intended perimeter poses
have individually reviewed replacements. The two old overlapping poses remain
rejected. These before-pit captures prove only their checked player/wall/moss
scope. Chamber/actor fidelity, foreground base remnants and overall composition
remain open; new pit/end views provide their own separate evidence.

Five old native stair-arrival pairs have now been individually inspected:
1F3,11→2F;2F3,11→1F;2F5,3→3F;3F13,3→4F;3F5,3→2F.
The two arrivals onto upward stairs hide the player's lower body. Source code
confirms that the new flights rise16 source pixels while visual footing remains
at the original floor height. Downward arrivals also need actual tread support
rather than unchanged floor height. The next repair must sample only complete
accepted stair geometry for visual footing, preserving authored navigation,
warp destinations and original support arrays. Remaining nine stair-arrival
pairs, old fall images, all311/original16/shared-furniture/earned native scenes
and the full acceptance table remain open. No windows or app focus changes.

Lighthouse visual tread support implemented; verification running — 2026-10-17

lighthouse_stairs.rs now supplies rendered actor footing for complete accepted
up/down flights. It reuses the mesher's native floor/phase/source admission,
orders the actual frame grid and rejects missing/duplicate/clipped/foreign art,
nonfinite coordinates and incompatible resolved support planes. Ordinary floor
queries return before dense resolution. Contact samples the actual six tread
heights and closed cheeks at the frame's real world offset/scale. Stored source
support arrays, flight geometry, source cells, collision and warp scripts remain
unchanged. The common resolved-footing path consumes this scoped visual sample;
complete lighthouse flights keep walking gait while authored ledge jumps still
retain their arc. Other maps and rejected drawings retain existing behavior.

Two new regressions test actual emitted horizontal triangle contact for all six
steps, both directions, scales0.5/1/2, reordered grids/world offsets, source
corruption/clipping/duplicates/foreign maps/missing ground and altered support.
A gait test distinguishes ordinary stair walking from an authored ledge jump.
The existing external-pack stair guard now independently checks all fourteen
packed flights against emitted tread tops as well as source warp identity,
paint and untouched support arrays. No pass is claimed until the actual unified
source run finishes: Cargo87532 is confirmed live compiling. The dependent
source/native driver requires those passes and the authoritative reentry test,
then existing footing/radio controls, a native build, all fourteen ordinary-Right
stair arrivals with settled terrain/zero fade, two extra1F camera pairs and the
full311/225 audit. Every resulting image still requires individual review.

All6F perimeter pairs were individually inspected and their scoped records are
updated. All intended six-floor perimeter poses now have individually reviewed
valid fixtures; the old3F north and5F southwest overlapping fixtures remain
rejected. The full room, source-alternative, shared-furniture, earned native
League/Hall and other acceptance obligations remain open. Render work remains
explicitly offscreen/muted, with no visible-window fallback or focus changes.

Stair tread/source controls pass; thirteen gallery fixtures rejected — 2026-10-17

The unified build completes in8m43s. All38 selected viewer tests pass10.73s,
including both new tread/gait regressions and all fourteen packed flights'
rendered contact checks. The authoritative stair-arrival/reentry controller
passes4.42s. Fresh executable controls pass eight footing tests and five Radio
stair tests (56.41s), retaining ordinary-ground, authored ledge and existing
Radio behavior. Native tools are rebuilding; new actor height/readability and
all fourteen native arrivals are still unverified until the queued capture job
and individual review finish. No visual approval is inferred from these passes.

Inspection of the original CeruleanPoliceStation pair found the player on its
Pokefan NPC. An exact player/object-center comparison across all311 original
paired receipts identifies thirteen invalid original gallery fixtures:
BattleTowerHallway, BillsFamilysHouse, CeladonDeptStore5F,
CeruleanPoliceStation, Colosseum, CopycatsHouse1F, EarlsPokemonAcademy,
EcruteakLugiaSpeechHouse, ElmsHouse, GoldenrodDeptStore5F, MahoganyGym,
Route39Farmhouse and ViridianNicknameSpeechHouse. Their original capture exit
codes remain preserved, but fixture validity/whole-room approval is explicitly
rejected in results.json and actor-overlap-review.json. This exact overlap test
is not a proof that every other room/fixture is adequate. Existing newer scoped
reviews remain separately identified; original invalid evidence is not deleted.

The corrected batch sampler already avoids authored runtime/start positions,
collision, warps and ledges. A serial follow-up driver is queued after the
current stair source/native/audit process87698, reusing its fresh tools to
replace all thirteen original fixtures. It requires preserved paired state and
distinct actually captured player/object centers. Every replacement still needs
individual image review; these disposable starts do not prove earned arrivals.
No gameplay object position or script is changed. All311/original16/shared
furniture/earned native-scene and broader composition obligations stay open.
All native captures remain explicitly offscreen/muted with no window fallback.

Native lighthouse tread support verified; gallery replacements captured — 2026-10-17

Fresh native tools build in4m05s. All fourteen ordinary-Right stair crossings
reach their original expected map/tile, with matching active terrain, no pending
terrain/warp and zero fade. Two additional1F upward-flight views at rotations0/6
complete too. All sixteen paired state/camera receipts are checked and all32
source/native images are individually inspected. The upward-flight arrivals now
show the player supported above the steps instead of buried at the original
flat floor. Descending arrivals follow recessed treads; lower legs are naturally
obscured by the shaft, with head/body still identifiable. Checked masonry/moss
and independent block ends remain connected. Scoped tread support/arrival
readability accepted. Cap dominance, foreground floor-like wall remnants and
whole-room/actor fidelity remain open. Each crossing follows a disposable
initial fixture; these are14 input-earned arrivals, not a continuous ascent.

The new311-map/225-variant audit has zero errors and is exactly equal in every
parsed field to lighthouse-closed-ends-full311-audit.json, including all map,
source ownership, geometry, stored support and variant records. The equality
receipt is lighthouse-stair-support-full311-invariant-summary.json. Actual
rendered contact is proved by the source/triangle regressions and native images,
not inferred from this unchanged mesh audit. Source test results remain38 viewer
passes plus the authoritative reentry pass, eight footing and five Radio stair
controls; no all-tests or whole-goal claim is made.

The serial gallery job completes all thirteen actor-clear paired replacements.
Every receipt preserves state and the requested default camera, and every
actually captured player center differs from every source object center. Old
invalid originals stay rejected and link to their replacement evidence. The
CeruleanPoliceStation pair is individually inspected: player and seated Pokefan
are separate, pink stools have finished round seats, the table is grounded and
checked wall courses are continuous. The fishing guru still appears in a blue
uniform, and Diglett is unusually small relative to its source drawing, so
actor fidelity/whole-room approval stay open. The other twelve replacement pairs
remain pending individual inspection. No object positions, source scripts or
pack data were changed for these fixtures.

Full all311 room composition/materials/services and reachable states, original16
alternatives, shared furniture and earned native League/Hall/animation acceptance
remain active. Old fall-arrival image review and moving-column/other animation
obligations remain separate. All captures ran offscreen/muted; no windows or app
focus changes, external pack exports, shared AA edits or commits were made.

All thirteen replacement fixtures reviewed; native link-room finishes added — 2026-10-18

Every source/native pair in gallery-actor-clear-headless-review has now been
individually inspected, with specific findings in capture-provenance.json. All
thirteen replacement poses keep player and authored objects separate; original
overlapping gallery evidence remains rejected. Checked house stools/tables/wall
courses and school benches/chair alignment are readable in these views, but
this does not close whole-room, service, alternate-camera or actor fidelity.
Colosseum loses its blue wall, green apparatus and native entry/floor colors to
white/steel; the same loss is visible in Time Capsule's saved pair. Elm's house
has much paler timber than source. Mahogany has conspicuous black/white outer
wall strips and pale barrier tops. Battle Tower hallway door/control fronts
still require close volume/material review. These are explicit remaining
appearance issues; small classic species icons alone do not establish correct
physical species scale, so small silhouettes need actual readability review.

The link-room repair covers Colosseum, TimeCapsule and TradeCenter. The last
two share the authenticated trade drawing; all three previously have full320
source-cell ownership, which is retained. New native_link_rooms.rs uses existing
whole-plot/cell fingerprints and a dedicated LinkRoom style. It replaces generic
ceramic/grating paint with the native checker, gray waist strip and red entry;
restores blue wall pigments and gray skirt from this room's own atlas; paints
the original octagon's gray/beige courses and original apparatus with their own
green pigments. Time Capsule/Trade Center's receiver uses its own blue/white
source drawing. Model parse metadata requires complete named link materials;
unknown names/missing roles and altered drawings retain the existing fallback.
Model geometry is not regenerated, and logical support/collision/scripts/actors
are unchanged. Procedural floor/entry tops stay at the original zero datum.

New regressions compare actual source palette banks/pixels for all three packed
rooms, require source coverage/unchanged stored support, compare every upright
triangle against the prior room kit, reject every altered platform source cell,
and reject unsupported model roles/names without writes. The first owned two-map
compile was stopped before tests to include TradeCenter; its log is preserved,
with no pass/failure or gameplay-blocker inference. Only owned Cargo96728 and
rustc97086 were stopped. Current unified Cargo97627 is confirmed live compiling
the three-room source/model tests. No new link source pass or appearance claimed
yet. Its serial driver requires actual passes, existing special-room/sample/
perimeter controls, a fresh native build, all three rooms at rotations0/2/6,
three ordinary LLRR lower-platform routes and the complete311/225 audit. Every
new image and actual audit delta still needs inspection before acceptance.

All311/original16/shared-furniture/earned native-scene obligations remain active.
No windows, focus changes, external pack export, generated model artifact or
shared antialiasing edit was made. Deleted handoff remains deleted; these two
canonical review documents retain the work and outstanding acceptance scope.

Offscreen-only verification and complete link header — 2026-10-18

Automated native captures remain headless image targets with audio muted; no
visible fallback, window launch, application switch or focus change is permitted.
The full source wall header consists of three accepted Wall rows, including the
gray lower band. The native finish guard now admits and validates all three rows,
with blue body/cap sampling from row zero and the gray skirt from qualified floor
paint. This corrects the previous two-row guard before source tests execute.
Owned Cargo97627/rustc98632 and dependent Python98201 were stopped before their
test/source gate; their receipts are preserved as header-attempt-stopped and
header-driver-stopped. No pass or gameplay blocker is inferred from cancellation.
Fresh unified Cargo99553 is running the three-room source/model checks. The
serial review driver waits for its actual result before controls, native build,
offscreen paired capture, ordinary walking and full311/225 audit. New visual
acceptance remains pending; the wider interior obligations remain open.

Additional individual gallery review — 2026-10-18

PowerPlant and ViridianGym's existing matched source/native images were each
inspected individually while the link-room source build runs. Their paired
receipts preserve state; PowerPlant's actual player/object centers are distinct.
PowerPlant's office divider is connected and furniture is grounded, but several
large central/southern machinery banks remain flat source paintings, unlike the
upright northern bank. Red/white connecting runs also remain low; their exact
structural identity must come from source before inventing machinery. Its office
furniture is paler than native gold. Viridian's maze joins are continuous with
an open central aisle, but large smooth cap islands and repetitive side panels
still read as manufactured blocks rather than the mottled native green drawing.
Those surfaces remain rejected for final appearance; full northern coverage,
entrance-statue framing, services and earned leader scenes are not proven.
Findings are recorded in ignored full311-native-current-review/
individual-room-review.json. This review does not close either room or replace
current source-backed geometry/footing checks with screenshot inference.

Native link-room source and offscreen-default checks pass — 2026-10-18

Unified Cargo99553 completed compilation in8m55s, then all14 Bevy link controls
passed60.49s and all6 selected viewer checks passed4.92s. The latter includes
the new actual-pack Colosseum/TimeCapsule/TradeCenter palette/coverage/upright
geometry/support checks and named-model-role rejection/volume test. The source
header now validates all three Wall rows. Fresh executable controls also pass:
10 special-room checks, one source-sampled asset control4.30s, one perimeter
control and the one automated-native-offscreen-default regression. No visual
or whole-room acceptance follows from these tests. The serial driver is now
running owned native Cargo1583, then requires nine static matched pairs, three
ordinary four-step routes and the complete311/225 audit. All new rendering is
explicitly headless image capture with mute enabled and no visible fallback.
A second serial driver waits for that review process before rechecking current
PowerPlant/ViridianGym/GoldenrodGameCorner images, avoiding concurrent renders.

Link checker-floor regression found in native review; final-pass repair pending — 2026-10-18

The first offscreen three-room build passed its source/control gates and built
in3m57s. Nine static pairs at rotations0/2/6 and all three ordinary LLRR routes
completed with expected ordered positions5,6→4,6→3,6→4,6→5,6, active settled
terrain, no warp/fade and paired state preserved. The full311/225 comparison
has zero errors, only the three link-room geometry fields changed, all308 other
maps and all225 variants are exactly equal, and ownership/support fields remain
unchanged. These receipts are preserved with initial-finish names.

The first Colosseum source/native pair was individually inspected and rejected:
blue walls, green apparatus and red entry were restored, but the later generic
floor finisher still erased native diamonds to gray tiles. Remaining first-pass
pairs are retained as superseded/unaccepted, not individually accepted. The
native_link_room append-only tests did not exercise that final finishing stage.
interior_surfaces.rs now excludes the exact three-room gate01 floor sources
(complete17 or16's last row with valid phases) from generic repainting, retaining
the existing full live drawing, zero floor datum and source/collision ownership.
The new link_room_native_checker_survives_final_floor_finishing regression runs
the actual final profile-aware floor pass over all accepted source rows in all
three maps and checks unchanged positions, normals, complete UVs, colors,
indices, stored support and ownership; unrelated maps/altered sources reject.
Fresh unified Cargo2951 is confirmed running; its serial driver waits for all
three named source tests, controls and fresh native build before recapturing
all12 pairs/routes and the full311/225 audit. The final floor repair has no
passing-test or native appearance claim yet.

Fresh current-build PowerPlant/ViridianGym/GoldenrodGameCorner pairs have also
been individually inspected, all with preserved state and distinct actual
player/object centers. They confirm the older-gallery issues: large flat Power
Plant machinery plots, smooth/manufactured Viridian maze caps/sides, and Game
Corner cabinet fronts southward despite west/east stools plus black rear-wall
gaps behind equipment. Native arcade paint is selected before the older
outward-pair branch, so outward-facing source code elsewhere does not prove
current cabinet orientation. These remain concrete next repairs; no whole-room,
service or earned-state acceptance follows. All captures were headless image
targets with mute enabled, no windows or focus changes. No packs or model assets
were regenerated; unrelated work and the deleted handoff remain untouched.

Game Corner orientation cause traced; link floor build still live — 2026-10-18

The current renderer's native_arcade_source branch invokes append_native_arcade
before the older ArcadePair outward adapter. The pair export explicitly faces
+Z. Inspection of all nine material primitives proves no triangle crosses the
unit midline and the nearest vertex is0.08 model units away, so these are two
independent cabinets, not one connected body. A future source-qualified pair
adapter can rotate each closed unit west/east toward its existing stool aisle,
retaining live palette roles, indices, heights and source-cell ownership; the
single rear machines must remain south-facing. This is cause/implementation
preparation, not an executed fix or visual pass. Rear-wall joins still require
source-qualified backing behind equipment and normalized short header planes;
Celadon's authored poster text and southern city exits must remain accessible.

Owned Cargo2951 remains confirmed live compiling the link checker final-pass
regression; it has not been restarted for observation latency. The serial driver
continues waiting on this exact process before tests/build/captures. No final
floor pass or whole-room acceptance is claimed yet. No visible render launched.

Complete packed-room floor gate added after entry-corner defect — 2026-10-18

Cargo2951 completed11m30s; all14 Bevy link controls59.12s, all7 viewer checks
3.71s, offscreen-default/existing-room controls and all34 floor controls11.19s
passed. The native build completed3m47s. Nine static pairs, all three complete
LLRR walks and the311/225 audit completed. Its comparison again changes only
three maps' geometry fields; all308 other maps, all225 variants, ownership and
support fields are exactly unchanged, with zero errors. These receipts are
preserved as partial-checker; they do not prove final visual acceptance.

Individual inspection of the new Colosseum source/native pair shows restored
diamonds over most of the floor, but gray patches remain beside the entry.
Actual source blocks1f/20 also contain the same gate01 diamond floor outside
the platform. Restricting final-pass protection to blocks16/17 missed those
physical floor cells; the isolated final-pass fixture was too narrow. Remaining
partial-checker pairs are retained as superseded/unaccepted, with no invented
individual-review claim. The finish exclusion now recognizes that unambiguous
live gate01 motif with valid phases in only the three link maps, including all
platform-corner floor cells. The existing complete plot/cell geometry guards
are retained. The actual-pack regression now calls the real final profile-aware
finisher on the complete generated room mesh and requires zero rewriting and
full TerrainMeshData equality for each map, covering every cell/UV/index/range,
not just plain-floor fixtures. The isolated fixture also covers corner blocks
and invalid phases/atlas/tile/map. Fresh unified Cargo5995 is confirmed live;
its serial driver requires these actual tests, all34 floor controls, native
recapture, actual walking and full311/225 comparison. This last repair has no
passing-test or native appearance claim yet.

The first checked link view also retains very broad, low wall caps and a much
smaller Object(0) Chris than the player, despite identical source IDs/sizes in
the receipt. Geometry/actor fidelity remains open beyond the finish repair;
unchanged old geometry is not evidence of acceptable room scale or ceiling
height. Current arcade trace likewise confirms south-facing native pair models
bypass the older outward branch and yellow body sampling loses pink panel
paint. Actual Crystal CeladonGameCorner.asm has no callbacks/changeblock or
hidden-stair warp: both posters jumptext 'nothing' and its two exits go south
to Celadon City. The earlier speculative hidden-stair note was corrected;
source interactions, not remembered first-generation behavior, govern repair.

All311/original16/shared furniture/service/earned-native-scene obligations remain
active. No window, focus change, model regeneration, pack export or cross-chat
message occurred; unrelated changes and the deleted handoff remain preserved.

Current verification wait and actor-scale qualification — 2026-10-18

Owned Cargo5995 and viewer-test compiler7006 are confirmed live; the serial
review driver6050 waits on that exact Cargo process. No source changes were
made while compiling, and no observation timeout was treated as termination.
The complete packed-room final-floor regression and new native appearance are
still pending, with earlier initial/partial-checker captures retained rejected.

Read-only actor-source inspection qualifies the earlier link-room scale finding:
new_bark_actors.rs explicitly applies PLAYER_PRESENTATION_SCALE1.25 only to the
player identity; NPCs sharing Chris's source retain authored stature. Therefore
same source IDs/64px sprite sizes do not prove equal physical height. The small
other trainer still needs seated-pose/readability review; do not normalize its
height solely to match uniform source icons or undo the requested player scale.
No actor files were edited. Hall source-frame capture code was also inspected:
the continuous controller saves render_visible_hall_of_fame_screen images at
phase entry, so those source frames still do not prove native GPU ceremony
presentation or its mid-phase animation. The full native-scene gate stays open.

Complete packed-room floor regression passes; original16 native-state review queued — 2026-10-18

Cargo5995 completed compilation8m34s. All14 Bevy link controls pass64.89s and
all7 selected viewer checks pass3.64s, now including full TerrainMeshData equality
through the real final finisher on each complete packed link-room mesh. All34
floor controls also pass11.62s; existing special-room/sample/perimeter and
one offscreen-default control pass. The exact native gate01 motif now remains
native across plain-floor and platform-corner source blocks; no floor-block
allowlist omits the entry corners. These tests still do not establish appearance.
Owned native Cargo8212 is confirmed live; driver6050 will recapture all12 link
pairs/three ordinary four-step walks and perform the full311/225 comparison.
No final native link-room acceptance is claimed before those fresh images.

A second serial driver8104 now waits for the completed link source/native/audit
gates before extending the original16 evidence. It runs fresh actual-pack
controller entry-lock and four-victory tests, publishes their five entry/four
battle saves, builds the saved-session renderer and captures all nine original
alternatives from front/east as18 matched pairs. Entries use the authored
arrival transaction from an initial map fixture; victories seed the initial
party only and require real battle/PP/script changes. Neither flags nor block
edits are seeded, and this is not continuous League traversal. The driver
requires actual passes, same-map/state/camera receipts, and explicitly leaves
all individual image review pending. Lance's third mask, Celadon's unconditional
callback case, native continuous League/Hall scenes and all311 room quality
remain separate required work. All rendering stays headless/image/muted.

Link finish repair visually verified; nine owned door alternatives and rooftop callback reviewed — 2026-10-18

Native Cargo8212 completed4m03s. All12 final link-room pairs (24 images) were
individually inspected: each of the three rooms at rotations0/2/6 plus its
ordinary LLRR lower-platform route. Complete native diamonds now extend through
both entry corners; red entries, gray/beige octagons, green apparatus, cream
round stools and blue/white trade receivers retain source pigment. This scoped
floor/pigment repair is accepted. Every pair preserves state and actual
player/object centers are distinct. Each route proves all ordered positions
5,6→4,6→3,6→4,6→5,6 with settled active terrain, no warp/fade, and readable final
player/companion footing. No network battle/trade service follows from walking.
Broad low blue wall caps, lower gray-band/wall-face proportions, apparatus and
actor/companion fidelity remain open; the full rooms are not accepted.

The fresh complete311/225 audit comparison was actually executed. It changes
only the three link rooms' four geometry fields; all308 other maps, all225
variants, all source ownership/support fields and all other root/non-geometry
totals are exactly equal, with zero errors. The ignored invariant summary records
Colosseum solid triangles−27936/textured+12640 and TimeCapsule/TradeCenter each
solid−28316/textured+13020, with corresponding actual vertex deltas. No palette
or visual acceptance was inferred from these counts. This follows the current
7 source checks/full packed-room final-pass proof,34 floor controls and existing
room/offscreen controls; rejected initial and partial-checker evidence remains
explicitly superseded.

The fresh actual-pack entry-lock controller passes5.18s across all five rooms;
four actual victory/PP/exit controller cases pass14.12s. Saved-session renderer
build passes4.40s. All18 source/native pairs from their nine alternatives were
individually inspected from front/east. Closed southern barriers are clear in
side views despite front player cutaway; four northern exit openings are visible
after actual wins. Some side views crop far ends, so they do not establish full
coverage individually. Guardian spheres lose red/white source accents, and Koga/
Karen retain generic blue human outfits; room/actor fidelity remains open.
Lance's lower entry pair does not cover its north gate, so two additional actual
ten-step Up walks reach5,9 from5,19. Both final pairs were individually inspected:
the closed north gate remains visible, source/native state matches and terrain
settles. Their20 intermediate images are not reviewed; no middle-step animation
acceptance is inferred. Lance's side courses remain largely flat blue/gray art
against black perimeter, so its wall height/material/ornament work is open.

Two additional Celadon6F callback pairs were individually inspected. Actual
source aliases DeptStore6F.blk; its base block6,0 is2f, and the unconditional
MAPCALLBACK_TILES changes it to03. Runtime-tile initialization explicitly
finishes this authoritative callback. Both source and native show glazing
instead of roof stairs at that course, with no preview flags or synthetic block
edits. Window/perimeter heights remain too low and backing is missing behind
the vending bank, clearly exposed from east; full room architecture is rejected.
All22 final pairs in original16-owned-state-headless-review preserve state and
actual player/object centers are distinct. These ten original alternatives plus
the five previously reviewed synthetic exit-only cases give native evidence for
15 of the original16; Lance's third mask still requires appropriate native
state/scene capture. The old lance-victory-saves load was attempted and rejected
by the client because its core-modular identity differs from the current pack.
Its ID was not rewritten. The compatible current-text-pack-earned-states Lance
victory save is authored after entering HallOfFame, as its controller helper
explicitly asserts, so it cannot prove the temporary Lance-room combination.

Full all311 composition/materials/services/reachable states, all16 native/earned
state obligations, continuous native League/Hall animation and shared furniture
remain active. Next concrete quality repairs include Game Corner cabinet
orientation/pink paint/rear joins, low retail walls and vending backing, and
remaining PowerPlant/Viridian/link-room volumes. All captures were offscreen,
image targets and muted; no windows, focus changes, cross-chat messages, pack
export, generated model assets or unrelated edits occurred. Deleted handoff
remains deleted; these canonical documents retain current evidence.

Game Corner cabinet orientation/native pink and rear-wall repair implemented — 2026-10-18

New native_arcade_pairs.rs adapts the existing closed, beveled two-unit model.
It validates separate halves/no crossing triangles, turns the original west and
east units toward their stool aisles, centers each on its actual owned half,
and rejects geometry outside the unchanged source footprint before any write.
Height, source UVs, indices and each triangle's rigid edge lengths are preserved;
normals/neutral face shading turn with the units and cached geometry is immutable.
Only the existing complete native ArcadePair source guard uses this adapter;
rear/front-facing single machines and unsupported-source fallbacks are retained.
Pair body/trim swatches now select the native pink field and yellow edge rather
than making nearly the entire body yellow.

New native_arcade_backing.rs admits complete north-row prize racks and both
complete rear-cabinet halves in only Celadon/Goldenrod Game Corner. It reuses
the existing cabinet guards, rejects altered/clipped drawings and custom-owned
source, joins spans and copies the authenticated room-wall volume behind their
original occupied cells. Source claims, flooring, support and all events remain
unchanged; no wall is invented behind interior banks. Existing camera-wall and
cutaway ownership is retained. Source arcade-specific wall pieces/poster backing
were28px while rear machines reach29px, despite the shared wall kit's32px datum.
Resolved wall/window courses in these two rooms and poster backing now share32px;
prize racks/cabinets keep their original sizes. Updated wall tests require the
actual ceiling datum, and the new backing ray proof now reaches31px above the
machines, as well as lower/middle wall bands.

First source Cargo14564 compiled9m01s and ran five tests: four passed, covering
rigid outward-facing detail, immutable cached models, whole-source/footing guards
and actual packed north-wall backing rays. The pigment test passed pink but
failed its expected yellow bank number, reporting actual4 versus expected6.
Pinned source tileset_constants.asm explicitly defines PAL_BG_YELLOW4 and
PAL_BG_ROOF6; the assertion was corrected to that authoritative constant, without
changing the native swatch to fit a wrong expectation. That failed receipt and
dependent gate failure are preserved as initial-wall-attempt. No native build
or visual acceptance occurred from that attempt. The additional32px ceiling
repair was added before the fresh run recorded below.

Fresh unified Cargo16581 finished compilation in10m44s and all five selected
viewer tests passed in8.08s, including actual pink/yellow pigments, rigid outward
cabinet faces and complete north-wall rays through31px. No Bevy gameplay test
was selected by this filter. The same fresh viewer test binary also passed21
arcade,23 casino and one packed room-donor control; selections overlap and are
not additive distinct test counts. This verifies the32px ceiling/source repair
at the mesh level, not visual acceptance. Serial driver16736 then started native
Cargo18348 for render_at_location and audit_authored_assets. Both rooms at0/2/6,
actor-clear perimeter poses, two ordinary UD aisle walks and the complete311/225
audit remain queued. A strict audit comparator is prepared to allow only the two
Game Corners' geometry fields to differ; all309 other maps,225 variants, source
ownership/support and other totals must remain exact. All actual new images and
source/geometry/support deltas remain pending inspection. Every capture explicitly
uses headless image target and mute, with
no visible fallback or focus change. No model assets/packs were regenerated.
Full all311/original16/shared furniture/service/earned-native-scene and remaining
wall/actor obligations remain active. Deleted handoff and unrelated work remain
untouched; no staging, commit, PR, cross-chat message or export was performed.

Game Corner first native review rejects cream bodies; pink-midtone correction — 2026-10-18

Native Cargo18348 completed in3m57s. All six front/east/west paired views of
Celadon/Goldenrod Game Corner were individually inspected (12 images). Cabinet
fronts now face the stool aisles and the north walls/ceiling remain connected
behind cabinets and prize racks. However, the bodies remain cream: [10,1] used
RED's255 highlight, so the previous bank-only assertion accepted the wrong
visible pigment. This is rejected material evidence, not room acceptance.
Existing actual source-color traces identify [25,1] as pink RGBA255,156,198;
the source swatch now uses that pixel, and the pack test requires RED's170
midtone instead of255. Yellow trim remains its independent YELLOW170 sample.
No size, ownership, support, topology, UV layout or gameplay change accompanies
this one-swatch correction. Fresh source Cargo19216 is live; the serial
native-arcade-pink-midtone driver gates rebuild and complete recapture/audit on
its actual passes. The original quality driver16736 completed its16 perimeter captures, both
ordinary UD routes and full311/225 audit using its already-built pre-correction
executable. Both walks prove10,7→10,6→10,7 with active same-map terrain, no warp
and zero fade. Their two final pairs were also individually inspected: the player
is grounded/visible, but the companion is partly behind the player in this front
view; no full actor readability acceptance follows. Together with the six static
pairs, eight pairs/16 images have been individually reviewed. All16 perimeter
pairs remain manually unreviewed. Strict audit comparison actually passed: only
Celadon solid triangles+1380/vertices+2760 and Goldenrod+1840/+3680 change; all309
other maps,225 variants, source ownership/support, textured geometry and all
other non-geometry/root fields remain exact, with zero mesher errors. These
receipts must not be presented as proof of the corrected pink paint. All rendering stays offscreen/image/muted.
Full room/service/actor and all311/original16 obligations remain open.

Game Corner pink/orientation/rear-wall repair verified; Viridian stone relief in tests — 2026-10-18

Pink correction Cargo19216 compiled9m49s and all five actual viewer tests passed
in8.04s; the Bevy filter selected no gameplay tests. Fresh controls passed21
arcade,23 casino and one packed wall-donor selection (overlapping selections).
Native build20828 completed5m11s. All24 final pairs/48 full-size images in
native-arcade-pink-midtone-headless-review were individually inspected: both
Game Corners at rotations0/2/6, all16 actor-clear perimeter poses and both
ordinary UD aisle walks. Pink bodies/yellow edges, outward fronts toward stool
rows, connected rear wall above prize racks/rear equipment and original clear
floor aisles are accepted for this scoped repair. Every pair preserves state
and actual Player/Object centers are distinct. Both actual routes prove
10,7→10,6→10,7 with active same-map terrain, no warp and zero fade. Companion
occlusion in front views and player/stool pose/readability at some perimeter
fixtures still require full actor/service review; no blanket room pass follows.

The raw full311 audit equality assertion initially failed. Investigation found
exactly one additive schema field: audit_authored_assets.rs now records
coastal_terrace_field = frame.terrace_datum.is_some(); it was absent in the prior
receipt and is false for every one of these311 interiors. No preexisting field
differs. The executed strict comparison explicitly admits only this reviewed
absent→false addition and proves all preexisting map/geometry/ownership/support,
root/totals and all225 variant fields exactly equal, with zero mesher errors.
This does not claim byte-for-byte JSON equality. The earlier orientation/wall
audit already proved changes limited to the two Game Corners' solid geometry.

New viridian_maze_stone.rs now replaces the old pigment-only cap finish under
the same complete Viridian network admission. A shared continuous height field
weathers the existing closed stone parts down inside their original footprint
and height envelope; its cubic ramp fixes floor contact and its positive vertical
derivative preserves closed/overlapping parts. Normals follow the inverse
transpose and native green pigments retain mild weathering. Topology, UVs,
buried join ordinals, source claims, collision, floor and fade groups are retained.
Existing bevel proof now checks genuine uneven geometry rather than requiring
flat tops. New tests cover all16 shapes at three scales/three offsets, coincident
hard-edge copies, positive winding, unit normals, cached-model immutability,
fixed floor/XZ bounds and exact neighboring buried-face point sets. Source
Cargo21689 compiled8m27s; all three selected maze tests passed0.02s. The fresh
17-test gym_scenery control selection passed3.82s; all41 fresh occluder_fade
controls passed0.41s. Serial driver23214 started native Cargo23217 after those
actual passes. No native acceptance yet: front/side/perimeter/ordinary UD and
full311/225 audit remain pending; its strict comparator is prepared to require
complete count/ownership/support audit equality with the final pink baseline. Every
capture is offscreen/image/muted. No window, focus switch, model export, pack
regeneration, staging, commit or cross-chat message occurred. Deleted handoff
stays deleted; all311/original16/shared-furniture/service obligations remain open.

Viridian first relief rejected visually; irregular cap refinement implemented — 2026-10-18

Native Cargo23217 completed1m34s; the offscreen driver generated12 pairs (three
rotations/eight perimeter poses/one UD walk) and completed its311/225 audit.
Only rotation0 and rotation2 pairs have been individually reviewed so far
(four full-size images). Joins stay connected and the top is genuinely uneven,
but its regular concentric tessellation plus a height frequency near one cycle
per8px cell produces a repeating quilt-like pattern. This fails the requested
improvement over manufactured/block-like green maze walls. No visual acceptance
is granted. Rotation6/perimeter/walk images remain manually unreviewed. The
actual UD route proved5,9→5,8→5,9 with settled active same-map terrain, no warp
and zero fade. The executed strict311/225 comparison is exactly equal to the
final pink audit for every field; it proves counts/claims/support/variants,
not unchanged vertex positions or satisfactory appearance.

The refinement in viridian_maze_stone.rs keeps section boundary XZ coordinates
fixed while moving interior cap vertices with two smooth, bounded one-axis
fields. Their windows vanish exactly at every boundary and at floor height;
their positive derivative bounds keep closed parts inside the same source plot.
Normals compose the inverse transpose of both horizontal transforms and the
existing vertical relief. Height frequencies no longer align near one cell
period. Existing source ownership, UVs, index topology, buried join ordinals,
floor datum and fade ownership remain unchanged. Bounds tests now require the
same source plot and exact section edges rather than requiring every interior
vertex's XZ to remain flat/regular. New source Cargo24359 is live; fresh
viridian-irregular-stone driver is queued behind its actual three maze passes,
then fresh gym/fade controls, rebuild, all12 paired captures, actual UD and full
311/225 equality audit. Its new geometry/pigments and appearance remain unverified.
All captures stay offscreen/image/muted. Full all311/original16/shared-furniture,
actor and service obligations remain active. No unrelated changes or deleted
handoff were restored; no model export, pack build, staging or commit occurred.

Viridian irregular relief checked; green-wall fidelity still open; native Lance checkpoint repair — 2026-10-18

Fresh source Cargo24359 compiled8m31s. All three mesh tests passed0.01s, the
17 fresh gym controls passed3.93s and41 fade controls passed0.44s. Native
Cargo25955 completed4m02s. All12 final pairs/24 full-size images in
viridian-irregular-stone-headless-review were individually inspected: rotations
0/2/6, all eight perimeter fixtures and the ordinary UD route. The repeating
quilt pattern is removed; section edges and all source maze joins remain closed,
floor/entry space stays intact and the player remains grounded. This accepts
only the irregular-relief/closed-join improvement. Broad cap proportions,
repeated side-panel/material fidelity, guardian shapes and companion readability
still prevent full green-maze/room acceptance. Some northern/middle fixtures
hide the companion, so distinct actor centers alone do not prove readability.
The actual route proves5,9→5,8→5,9 with active same-map terrain, no warp and zero
fade. Strict full311/225 comparison actually passed exact JSON equality with the
final pink baseline: all counts/claims/support/variants/root fields are intact;
this count audit does not prove vertex positions are unchanged.

To pursue the remaining original16 Lance state without synthetic door flags,
the exact pinned source was re-read: the actual victory sequence opens north
block0b while the entrance remains closed, then displays Mary's first page.
The existing ordinary-input notice hold can capture that authored state. Fresh
saved-session renderer build completed3m37s. First attempted native replay from
current-text-pack-earned-states/LancesRoom-before-battle.crystalsave was rejected
before rendering: its actual embedded pack hash is the browser11f9e17c… baseline,
not nativea0153af8…. The earlier assumption that this checkpoint was native-
compatible was wrong. All five available similarly named checkpoints were
inspected: two use that browser hash and three an older8086b150… hash. No identity
or hash was rewritten. The failed load/log and original bytes are retained.
The existing ignored actual-pack Lance controller regression was explicitly
executed against the current native pack and passed6.70s, including real battle,
PP mutation, authored exit opening and Hall continuation. Its newly created
quiescent checkpoint embeds the correct nativea0153af8… hash and was loaded by
that regression before fighting. The failed browser input/logs were preserved
under rejected-browser-checkpoint names. The ordinary native replay was retried
from the genuinely native checkpoint; fresh saved-renderer Cargo28308 is live
before its inputs/captures. Native earned third-mask capture remains unproven;
no flags/pack IDs/hashes were rewritten. All work stays offscreen/image/muted;
no visible windows, pack export, staging, commits or cross-chat messages occur.
Full all311/original16/shared-furniture/service/earned-scene obligations remain
active, and deleted handoff/unrelated work remain untouched.

Lance replay exposed source-dialogue QA gap; read-only observer repair implemented — 2026-10-18

The native-compatible checkpoint was actually replayed with ordinary U18+A500
inputs after a3m55s saved-renderer rebuild. It won and continued through Hall,
credits and home instead of holding Mary's page. Once NewBarkTown was observed,
owned renderer28827 was deliberately terminated: the desired page had already
passed, so more A inputs could not satisfy this capture. The failed run and its
moving frames are retained separately in lance-earned-third-mask-source-hook-miss;
no final pair or original16 acceptance follows. Individual moving frames80/120
show actual battle, and140/141/142/160 show Mary's text, Oak's arrival and later
continuation with the north portal open. Those six selected images were
individually inspected; the other hundreds of intermediate images are not
reviewed. Battle composition is poor (tiny Mew against oversized wall/stair
scenery); this is not a battle-rendering acceptance. Lance's flat blue side
courses/black backing and whole-room/actor work remain open.

A production stdio-MCP replay from the same native checkpoint reached Mary's
actual first page after73 ordinary A presses following Up input; it retained
byte-empty stderr. Its exact page is MARY: Oh, no!\nIt's all over!, matching the
requested target. This rules out the guessed text as the cause. The old QA hold
only consulted field_notice, while ordinary compiled writetext uses the shared
visible_field_dialog_pages surface with field_notice=None. New
render_walk::revealed_dialogue_page reads that actual renderer page and its
matching complete printer. It rejects other/unfinished pages and stale retained
text after closetext. Only QA key release/freeze changes; the production input,
dialogue dispatcher, flags and script continuation are unchanged.

The hold now additionally records a read-only dialogue-hold.json witness with
actual source label/window/page/checksum, party move PP, current-map overrides
and three observed Lance flags. This makes native earned-state evidence checkable
without flag injection or save identity edits. The existing real-pack Lance
regression now requires observation of the earned ordinary Mary page, verifies
field_notice=None, rejects a closed-window copy and wrong page, checks observer
checksum immutability and actual victory/entrance/exit flags/PP depletion.
Fresh source Cargo30959 is live; the queued replay driver waits for its actual
pass, then rebuilds and checks both held pairs against actual north0b/south34
blocks, earned champion/closed entrance/unset synthetic exit flag, spent Ice Beam
PP and matching paired checksum. All new observer/build/native results remain
pending. No original16 or full-room completion is claimed. Captures remain
headless/image/muted; no windows, source exports, pack builds, staging, commits
or cross-chat messages occurred. Full all311/original16/shared-furniture and
service/earned-scene requirements remain active.

Ordinary source-dialogue observer regression passes; native Lance replay resumed — 2026-10-18

Initial source Cargo30959 compiled6m48s and reached Mary's ordinary page, proving
the observer admission worked, but its new checksum assertion compared a cheap
presentation snapshot (frame15/hash0) with a full snapshot (frame15/hash155502283).
The primary runtime implementation explicitly makes presentation_snapshot omit
integrity hashing. This was a mismatched verification API, not evidence that the
observer mutated gameplay. That actual failed receipt/driver gate are retained
as initial-checksum-view-mismatch. Neither native build nor scene acceptance
followed that failed gate.

The regression now compares state_checksum() before/after observation, and the
QA witness uses the same authoritative checksum API as paired capture. Fresh
Cargo32505 compiled2m20s and the real native-pack Lance regression passed6.79s:
ordinary page admission, stale closed-window/wrong-page rejection, read-only
checksum, actual win/PP/flags and Hall continuation all pass. Fresh existing
printer controls pass2/8.85s; new-page printer identity passes1/4.51s; same-tile
warp route release passes1/0.00s. No zero-test filter is treated as a pass.

Serial driver32621 passed its source gate and completed saved-renderer rebuild;
its actual U18+A500 native front replay is now live, with an east-wide replay
queued only after actual front-page/block/flag/PP/checksum assertions pass.
Final pairs, source witnesses, actual portals and original16 acceptance remain
pending inspection. The code changes observe QA text and release QA keys only;
production controller, source scripts, flags and input dispatch are retained.
All captures are headless/image/muted, and all311/shared-furniture/full-room/
service/native-earned-scene obligations remain active.

Lance earned third mask verified offscreen; all16 alternatives have native state evidence — 2026-10-18

Three ordinary-input native replays now reach and hold Mary's actual source page
in LancesRoom after defeating Lance. All six final source/native images were
individually inspected. Each pair preserves its own authoritative checksum;
frames/checksums differ across independent runs. The read-only witnesses record
BEAT_CHAMPION_LANCE=true, ENTRANCE_CLOSED=true, EXIT_OPEN=false and actual map
blocks north$0b/south$34. No door/victory flags or block edits were seeded.
The initial legal party/location is a fixture, not proof of full-story arrival.
A production stdio move menu verifies Ice Beam starts at16PP; all three native
victories end at10PP. Both source-page and initial-menu stdio probes retain
byte-empty stderr. The native pack/checkpoint identity was preserved.

The final front, east90-degree and east110-degree pairs and witnesses are in
target/kanto-resume/lance-earned-third-mask-headless-review. The wider110-degree
view includes the room footprint, but the south entrance is too small for close
geometry acceptance; the90-degree view clips that end. Blue side courses remain
flat, guardians/actors and whole-room materials remain open. No battle visual
or continuous native League/Hall acceptance is inferred.

The actual original16 evidence checker passes:16 image-supported physical-state
alternatives,37 native pairs, all225 declared audit alternatives retained.
This covers five synthetic exit-only states, ten controller-owned entrance or
victory states (including Lance's new third mask), and the6F callback. Inventory
coverage is not full visual acceptance. Receipts are original16-native-evidence-
inventory.json and lance-earned-third-mask-verification-summary.json under the
ignored target/kanto-resume/receipts directory. All captures used headless image
targets and mute; no windows or focus changes. Full all311 interiors, original16
art review, shared furniture, services and earned native scene obligations remain
active. The deleted handoff stays deleted; these two canonical guides retain
current acceptance and history.

Lance blue side courses raised and reviewed offscreen — 2026-10-18

New mesh/lance_side_walls.rs folds the complete native west/east blue courses
into closed ceiling-level walls joining the north course. The full native
north drawing and both44-row side-block strips must authenticate; altered,
clipped or customized courses yield without partial wall construction. Both
source wall columns are owned (168 cells); carpet, statues, south door and all
walking heights remain unchanged. Each side has one coherent cutaway group.
No core scripts, flags, pack inputs, furniture assets or audio were changed.

The initial84-cell revision passed geometry and the311/225 invariant check,
but all four native/source pairs were individually reviewed and rejected:
it incorrectly enlarged the narrow outer-edge art into dark striped faces
and sampled alternating cap colors. Those rejected images and receipts remain
separate under lance-blue-side-wall-headless-review. The final revision uses
the adjacent native brick-face tile93 and authenticated north field paint for
continuous caps/backs, folding both columns rather than leaving a flat brick
course beside the new wall. This corrects the observed error, not a source
palette/export change.

Final source build4m13s; both new wall tests and all three existing Lance
geometry tests pass (the lance_ filter also selects one unrelated Hitmontop
balance test; six total,9.92s). The wall tests verify every inner-face height
along both courses, closed signed volume/outward faces, original support,
source immutability and edited/clipped/custom-source rejection. Fresh43 fade
controls pass0.63s. Native build4m45s. All five final pairs/ten full images and
both captured moving images are individually inspected: north join, east/west
middle walls, south ends and an ordinary UD aisle walk. The actual route is
(4,12)->(4,11)->(4,12), with active settled terrain, no warp and zero fade.
The64-frame movement log is functional evidence, not64 reviewed native images.
Each final source/native pair retains its own authoritative state.

Final full311 audit has zero mesher errors. Strict comparison admits only
Lance's intended168 wall claims, +1448 textured triangles/+3568 vertices and
corresponding flat-to-modeled accounting; every other field,310 other map
records, all support and all225 declared alternatives are unchanged. Receipts:
lance-blue-brick-wall-{tests,fade-controls,native-build,full311-invariant-summary}
under target/kanto-resume/receipts; final images/provenance/individual-room-review
under target/kanto-resume/lance-blue-brick-wall-headless-review. All captures
are headless/image/muted; no windows, tabs, focus changes or exports occurred.

Continuous blue brick faces/closed backing are scoped accepted. Broad foreground
cap composition, flat floor-edge fragments, guardian/actor fidelity, remaining
south-barrier art and full-room presentation remain open. These visual fixtures
do not earn a victory or continuous League traversal. The earlier earned Lance
flags/blocks/PP evidence is separate and predates the side-wall change. Full
all311/original16/shared-furniture/service/earned-native-scene work remains active.

Power Plant machinery modeled; source finish reviewed; foreground reveal repair in verification — 2026-10-18

New mesh/power_plant_equipment.rs resolves all twelve complete native facility
$12/$16 equipment plots in PowerPlant (six each,192 cells). Exact full4x4 art
and subtile guards, prior ownership and live custom profiles determine each
whole placement. Other maps and partial/edited/customized plots yield. Low
closed machine beds occupy the existing blocked plots; chamfered housings rise
to20 source pixels overall. The upper two source rows wrap the roofs, and the
lower two become upright controls. Original footing, source cells, scripts,
collision, actors and adjacent floor/conduit drawings are not mutated. Primary
source collision confirms both equipment blocks are WALL in all quadrants,
while red/white conduit block$0b is FLOOR in all four. This resolves the full
192-cell equipment selector, beyond the48 cells matching an old B2F drawing.
Integration is a private facility_radio child and one authored-landmark call.

The first compiled render fixture9,10 was rejected as nonwalkable before any
image; its receipt is retained under rejected-unwalkable-fixture. The previously
verified9,9 office pose was used instead, without rebuilding unchanged code.
The first five pairs/ten full images were individually inspected: roof/front
roles and grounded housings are visible, but pale bed/end paint was rejected.
The next dark-field revision was inspected at two pairs/four images and rejected
as nearly black; its other three pairs are not manually reviewed. The final
midtone selector chooses a real opaque neutral source-control pixel, excluding
white highlights, dark ink, blue and gold. It copies no palette or artwork and
changes no model assets. All five midtone static pairs plus its conduit final
pair (twelve full images) and both captured moving images were individually
inspected. Roof/control orientation, machine scale and the revised gray field
are scoped accepted; whole machinery/room acceptance is withheld for reveal.

Midtone source build4m58s: three equipment tests pass7.55s (twelve plots, exact
roof/front areas and outward normals, immutable support, edited/custom ownership
and the field selector). Fresh eight facility controls pass3.81s and43 fade
controls pass0.48s. Native build3m47s. Actual ordinary UD conduit route is
(13,11)->(13,10)->(13,11), with64 logged frames, settled active terrain, no warp
and no screen fade. The64-frame log is not64 visually reviewed images. Every
final source/native pair retains its own state. The walk exposes closed faces
blending through each other in the foreground unit; this is a real remaining
visual defect despite the passing movement/fade controls.

The depth-writing change passed all three source tests (Cargo48842,3m46s build,
7.07s tests), but its native build failed at terrace_datum.rs with E0063: a shared
VisualTerraceDatum change had not yet supplied water_barriers. The current
constructor was inspected and already has its actual water_barriers value;
this thread did not alter that unrelated work. No new native reveal images
followed the failed build.

The current refinement keeps the closed shallow beds opaque and assigns only
the complete housing/panel ranges to source-cabinet depth-writing reveal,
alongside their ordinary cutaway ranges. This retains a solid foundation beneath
revealing hardware. Tests require all twelve coverage ranges plus bed vertices
outside every reveal range. Actual source Cargo49770 and its gated opaque-bed
offscreen driver are running; fresh source/native reveal/walk results remain
pending. No earlier images are evidence for the refined reveal.

All three completed geometry/finish311-map audits have zero mesher errors.
Strict comparison permits only PowerPlant's192 claims, +384 textured triangles,
+1536 vertices and flat-to-modeled accounting (276->468 modeled,1070->878 flat).
Every other field,310 other map records, all support and all225 declared states
are unchanged. The finish-only audits are exactly equal to the first machinery
audit; counts do not prove appearance or reveal. Receipts/provenance are under
ignored target/kanto-resume with power-plant-equipment, power-plant-metal-equipment,
power-plant-midtone-equipment, source-only power-plant-depth-equipment and pending
power-plant-opaque-bed-equipment prefixes.
All captures use headless image targets and mute. No windows, tabs, focus changes,
source exports, packs, assets, staging, commits or cross-chat messages occurred.

PowerPlant's office wood/material palette, rear fixtures, actor/companion
composition and authored services/quest scenes remain open. Full all311,
original16 art/earned-native scenes and shared-furniture obligations remain
active; the deleted handoff stays deleted and these canonical guides retain
current acceptance/history.

Power Plant opaque beds and housing reveal verified offscreen — 2026-10-18

Fresh source Cargo49770 finished5m01s; all three equipment tests pass16.06s,
including the twelve housing coverage ranges and opaque beds outside all reveal
ranges. Eight fresh facility controls pass4.47s and43 fade controls pass0.61s.
The native build succeeds3m51s after the shared terrace constructor was completed
by its owning work. No unrelated terrain changes were made here.

Four fresh static pairs and the actual UD conduit final pair (ten full images),
plus both captured moving images, were individually inspected. Office overview,
front/east/west conduit views and movement confirm roofs/controls, source gray,
closed solid beds and the existing nearest-surface coverage reveal. Foreground
housing faces no longer ordinary-blend through one another; beds stay opaque.
Coverage grain is still visible and is not accepted as full visual polish.
The actual authoritative route remains (13,11)->(13,10)->(13,11),64 logged frames,
active settled terrain, no warp/screen fade and no extra terrain builds. Each
paired source/native capture preserves its own state. These are initial room
fixtures, not earned quest arrivals or service completion.

The final311/225 audit passes the strict original equipment comparison: only
PowerPlant's192 machine claims, +384 textured triangles/+1536 vertices and the
corresponding flat-to-modeled accounting differ from the Lance baseline. All
310 other map records, every other field/support and all225 declared states are
unchanged, with zero mesher errors. It is also exactly equal to the first
machinery geometry audit, as expected for finish/reveal metadata changes.
Counts do not prove appearance. Actual receipts, provenance and individual-room-
review are under ignored target/kanto-resume with power-plant-opaque-bed-equipment
prefixes. All captures remain headless/image/muted; no windows or focus changes.

The machinery's source shape/roof/front/grounded support and reveal ordering are
scoped verified. Visible coverage grain, broad bed detailing, office materials,
rear fixtures, actor/companion composition and authored quest/service scenes
remain open. A source trace also identifies the next shared furniture defect:
facility bookcase$06 still uses generic pale materials because facility is
excluded from native_bookcase_source. Actual audited ownership is112 cells in
four rooms: MrPokemonsHouse16, PowerPlant16, RuinsOfAlphResearchCenter16 and
TeamRocketBaseB3F64. The source's left cupboard ends$2a/$2b; the right unit ends
with open-book tiles$28/$29. These need distinct source finishes/base treatment,
not a blanket recolor or two identical cupboard bases. No bookcase change or
acceptance is claimed yet. Full all311/original16/shared-furniture work remains
active, and the deleted handoff remains deleted.

Facility paired bookcases: source finish and distinct bases verified offscreen — 2026-10-18

The exact complete facility $06 drawing now selects source-derived wood and
book materials in MrPokemonsHouse, PowerPlant, RuinsOfAlphResearchCenter and
TeamRocketBaseB3F. Seven paired assemblies retain the source distinction:
left $2a/$2b cupboard bases replace the lowest book row; right $28/$29 units
retain all three book rows. The source guard rejects edited, partial and other
room drawings. Original placement, scale, support, source art and cached models
are unchanged. No exported model, pack or generated content was added.

Actual source build succeeds in3m55s; the two new external-pack tests pass7.04s.
The fresh bookcase filter passes15 tests13.94s, including those same two tests;
it is not17 distinct tests. All43 fade controls pass0.41s. The native examples
build succeeds3m43s. Fourteen paired static captures, covering all seven source
assemblies from front and side, were individually reviewed as28 full images.
Each pair preserves state and has distinct actual player/object centers. These
are room fixtures, not earned arrivals or dynamic foreground-reveal proof.
All captures use explicit headless/image/muted settings. No windows, tabs or
focus changes were used. Ignored receipts and reviewed capture provenance are
under target/kanto-resume with facility-bookcase-native-finish prefixes.

The strict311-map/225-state comparison passes with zero mesher errors. All307
other map records and every nongeometry field, claim and support value are
exactly unchanged. Only the four intended solid/textured geometry fields differ
in the four selected rooms; replacing the left lower book rows reduces combined
triangles by826 overall. Counts alone do not establish appearance.

Source wood/books and the distinct cupboard/open-book bases are scoped verified.
The views still expose missing source checker floors in MrPokemonsHouse, the
Ruins research center and Rocket B3F, plus insufficient rear backing behind
fixtures and generic table/chair materials. These are visible outstanding
defects, not established regressions from this furniture change. Whole-room
composition, actors, services and dynamic bookcase reveal remain open. Full
all311/original16/shared-furniture work remains active; the deleted handoff
remains deleted.

Johto facility checker floors restored and verified offscreen — 2026-10-18

New johto_facility_floor.rs authenticates the blue/cream floor fragments in
MrPokemonsHouse, RuinsOfAlphResearchCenter, TeamRocketBaseB2F and TeamRocketBaseB3F.
The final surface pass now retains live source color fields and small ceramic
seams instead of generic gray paint. Exact metatile/phase/art masks exclude
walls, stairs, carpet, equipment and edited drawings; live custom profiles keep
first refusal. Source art, gameplay, support and authored ownership are unchanged.
The existing PowerPlant/Silph/TrainerHouse finish paths remain unchanged.

B2F needed a second repair: its authored Surface::Facility slabs painted opaque
gray lids before the final finish. Their 1,032 exact source floor fragments now
supply native textured tops, allowing the same live ceramic finish to survive.
The external-pack regression resolves all actual B2F placements and checks that
these lids have no solid gray overlay, retain their source UVs and preserve
support. No model, source export or pack regeneration was required.

Final source Cargo58668 succeeds in3m43s; all three focused tests pass7.01s.
The first selector test run exposed a wrong test expectation for custom ownership:
one exact source phase is reserved, so15 of16 fixture cells should finish, not8.
That expectation was corrected and the retained custom quad is asserted. The
source-phase test passed even in that first run. Current tests additionally
check triangle-local color donors and the actual B2F authored floor path.
Fresh34 room-floor controls pass10.43s,43 fade controls pass0.40s and the native
examples build succeeds3m46s. Repeated earlier selector tests are not extra
unique test cases.

Twelve static pairs cover both small rooms and two positions each in B2F/B3F
from front and side cameras. Those24 full images, the native RL final pair and
both moving captures were individually inspected (26 paired images plus two
moving images). Every pair preserves its own game state and distinct actual
player/object centers. Ordinary controller input moves B2F(4,10)->(5,10)->(4,10)
across64 logged frames, with active settled terrain, no warp/screen fade and no
additional terrain build. Logged frames do not establish blanket animation
acceptance. These are initial fixtures, not earned quest/service arrivals.
All captures use headless/image/muted settings; no windows, tabs, focus changes
or application switching occurred.

The latest strict311-map/225-state audit passes with zero mesher errors. Only
four intended solid/textured geometry fields differ in the four target maps;
all307 other map records, every nongeometry field/claim/support value and all225
declared alternatives remain exact. B2F's1,032 authored floor claims survive.
Receipts, strict comparison, per-pair provenance and individual-room review are
under ignored target/kanto-resume with johto-facility-checker prefixes. Neither
audit counts nor passing tests substitute for the inspected native views.

The blue/cream layout and grounded ceramic finish are scoped verified. Visible
remaining work includes full-height rear backing in the two small rooms,
unmodeled blue bench relief, B2F generic gray walls/broad caps, B3F backing behind
cabinets, table/chair materials, actor composition and services. A source trace
confirms the divider builder currently excludes both small rooms. Floor/source
boundaries must be retained when repairing those walls. Full all311/original16,
shared-furniture, room composition and earned-scene obligations remain active.
The deleted handoff remains deleted; these canonical guides retain the work.

Small facility walls connected; exterior wall-art remnants removed and verified offscreen — 2026-10-18

New mesh/native_facility_room_shell.rs supplies continuous32-source-pixel green
walls in MrPokemonsHouse and RuinsOfAlphResearchCenter. The shell uses the actual
rear tile$22 field/light/ink palette, closes behind every rear furniture column,
joins both sides and keeps source exits(2,7)/(3,7) open. All64 cells of each
complete north drawing authenticate it; edited, clipped, unknown or custom-owned
north drawings yield. Padded grids cannot shift physical boundaries. No source
art, actors, support, collision or source ownership are changed.

A broad perimeter test rejected the first protruding trim. Recessed courses now
stay behind the physical boundary. The first native build then raced the shared
Electrode JSON-to-GLB replacement: build.rs:172 failed reading the removed JSON.
The replacement GLB was present afterward, and a fresh build succeeded without
restoring or changing that separate asset work. Six static pairs and a Ruins LR
final pair (14 full images), plus both moving images, were individually reviewed.
Those first views exposed an old flat green tile$22 band outside the rear wall;
they are recorded as the rejected ground-art stage, not final acceptance.

The final refinement removes only original horizontal, zero-height wall-art
faces at their own north source-cell bounds:16 cells in MrPokemon's room and14
in the Ruins room. Real checker floors, raised surfaces, borrowed color donors,
existing vertex/range identities and all support/ownership survive. New source
Cargo65239 succeeds4m14s: three focused tests pass11.29s; ten perimeter controls
including those same three pass37.35s;43 fade controls pass0.48s. The native
examples build succeeds5m57s. Repeated earlier focused/control runs are not
additional unique tests.

All seven final pairs (14 full images) and both captured moving images were
individually reviewed again. Front/east/west views show green backing above and
behind the cabinets, connected side joins, the open entrance and removal of the
exterior green ground band. Every pair preserves its own source/native state
and distinct actual player/object centers. Ordinary Ruins LR movement follows
(6,6)->(5,6)->(6,6) across64 logged frames, with active settled terrain, no warp,
no screen fade and no additional terrain build. The64 rows are not64 reviewed
images or blanket animation acceptance. MrPokemon's authored introduction was
not bypassed to manufacture another walking result. These remain initial room
fixtures, not earned arrivals or authored service completion.

The final strict311-map/225-state audit has zero mesher errors. Compared with
the verified facility-floor baseline, only the two intended room geometry
records differ: MrPokemon +3688 textured triangles/+7440 vertices and Ruins
+3692/+7440. The previous wall revision had+3720 triangles in each; removal of
the original wall ground faces accounts exactly for the32/28-triangle cleanup.
All309 other map records, every nongeometry field/claim/support value and all225
declared alternatives are exact. Counts alone do not establish appearance.
Ignored receipts, provenance and individual-room review are under
target/kanto-resume with native-facility-room-shell (earlier stage) and
native-facility-wall-ground-cleanup (final) prefixes. All work remains
headless/image/muted, without windows, tabs, focus changes, source export,
pack regeneration, staging or commits.

Closed backing, connected sides, the source-green finish, exact open exits and
removal of the exterior wall-art band are scoped verified. Fine trim aliasing,
exterior furniture-underlay corners, generic table/chair materials, actor
composition and authored arrival/service scenes remain open. The flat blue
fixture is the same sofa art already modeled for Silph: MrPokemon block$19
lower rows match Silph$2d upper rows. Its missing binding needs the correct
source-row offset and native floor guards; no sofa fix is claimed yet. Full
all311/original16/shared-furniture and whole-room obligations remain active.
The deleted handoff stays deleted; these canonical guides retain current proof.

Three missing facility sofas modeled at their correct source offsets — 2026-10-18

Two new facility_tables_bindings rules reuse the existing native blue sofa for
MrPokemonsHouse and PowerPlant. The complete guarded plot begins in block$19's
lower two rows and includes the following block$07 checker rows. Only the eight
sofa cells are owned; floor guards remain unclaimed and unraised. This restores
one sofa at source(4,2) in MrPokemon's room and two at(12,2)/(16,2) in PowerPlant.
No new mesh assets, model dimensions, palettes, source exports or packs were
introduced. The three existing Silph$2d sofas retain their original geometry.

Actual source Cargo68444 succeeds4m25s; both focused external-pack tests pass
14.69s. They resolve the three missing plots under canonical profiles and the
three existing Silph plots, verify grounded11-pixel bodies/closed winding,
separate opaque floor and body reveal, unchanged support/source art, all sixteen
body/floor guard edits, custom ownership, reserved body cells, clipped plots
and unknown-room rejection. The first harness compile used the wrapper's wrong
accessor and omitted the profile import; both were corrected before these tests
ran. No runtime or visual result is attributed to that failed compile.

Eight fresh facility-furniture tests (including those same two) pass15.16s;
ten perimeter controls pass49.63s and43 fade controls pass0.49s. These repeated
focused tests are not ten distinct furniture tests. The native examples build
succeeds4m07s. Nine static native/source pairs cover the new sofas from front
and both sides, an extra PowerPlant pose and both Silph control poses. Those
18 full images, the RL final pair and both moving captures were individually
reviewed:20 paired images plus two moving images overall. Scale, cushions,
arms, feet, source-blue finish and source positions are scoped verified, with
clear native floor in front. Every pair preserves its own source/native state
and distinct actual player/object centers. All three Silph sofas are visible
in the control views.

Ordinary controller input follows PowerPlant(7,3)->(8,3)->(7,3),64 logged frames,
with active settled terrain, no warp/screen fade and no additional terrain
build. This checks the aisle in front of the sofas; it is not an earned quest
arrival or blanket animation/reveal acceptance. MrPokemon's introduction was
not bypassed to manufacture movement proof. Sofa-body reveal ranges and their
independent opaque floor have focused test coverage; broader dynamic poses
remain part of full-room acceptance.

The strict311-map/225-state audit passes with zero mesher errors. Exactly24
flat source cells become sofa-owned: MrPokemon8 and PowerPlant16. Each body's
source bounds are pinned, all guard floors/support/other claims are unchanged,
and only intended furniture geometry/accounting differs. Geometry deltas are
+420 textured triangles/+968 vertices in MrPokemon and+840/+1936 in PowerPlant.
All309 other map records (including Silph), all225 alternatives and every other
field remain exact. Global sofa coverage is now48 cells:24 new and24 existing,
six sofas across three rooms. Counts do not establish visual fidelity alone.

Ignored receipts, strict comparison, reviewed provenance and individual-room
review are under target/kanto-resume with facility-wall-sofa prefixes. All
captures stay headless/image/muted, with no windows, tabs, focus changes,
exports, staging or commits. The deleted handoff stays deleted; these two
canonical guides retain the evidence and remaining full scope.

The views still show facility chairs whose facing needs table-aware review,
generic table/chair materials, and missing rear-wall backing in PowerPlant and
Silph. PowerPlant's fine floor pattern also aliases. Whole-room composition,
actor/companion placement, services and earned arrivals remain open. In
facility_tables::append, the chair path still fits its model without choosing
a facing from adjacent table/counter geometry; the sofa binding does not fix
that requirement. Full all311/original16/shared-furniture work remains active.

Facility chairs face their serving surfaces; verified without native windows — 2026-10-18

New mesh/facility_chair_facing.rs selects the nearest authenticated resolved
square/meeting table, side desk or actual L-counter course within32 source
pixels. The two reception courses are shared with the counter builder, so its
empty staff area is not treated as a filled rectangle. Twenty of22 chairs have
a serving target: two in MrPokemonsHouse, four in PowerPlant, six in SilphCo1F
and eight in TeamRocketBaseB3F. Ten bodies rotate: MrPokemon's two west chairs
face east and eight south-side meeting chairs face north. Twelve existing
south-facing bodies remain byte-exact, including the two PowerPlant waiting
chairs whose distant tables are across a wall. This does not orient actors.

Only newly appended solid chair vertices rotate, within their original16x16
owned plots and clear of the actor foot. Source floor, support, UVs, indices,
alpha and cached models remain untouched. Normals account for nonuniform
fitting, and their baked directional response is recomputed. A custom/edited
serving surface cannot supply a facing; append rechecks the target witness.
No model asset, content pack or source export changed.

Source Cargo succeeds3m46s; three focused tests pass7.40s against the actual
external pack. Tests pin all22 headings/20 targets/10 rotations, each four-cell
claim and unchanged floor/support; all four quarter turns at five fitting
scales check real backrest direction, owned bounds, outward winding, normals,
material alpha and cached-model immutability. Edited/custom table and stale
target rejection are covered. An initial harness compile failed on the token
< followed immediately by -; it was corrected before this successful run.
Eleven furniture controls (including those three) pass18.95s, ten perimeter
controls37.01s and43 fade controls0.52s. Native examples build succeeds4m38s.

Twelve static native/source pairs cover MrPokemon front/both sides, PowerPlant
front/side and waiting-area control, Silph front/side, and both Rocket tables
front/side. A PowerPlant RL final pair makes13 pairs/26 full images; every
image and both moving captures were individually reviewed. Serving directions
are scoped verified in these views. Silph's front view includes both staff
chairs; its side view crops one. Generic cream table/chair materials, actor
alignment, rear backing in PowerPlant/Silph, PowerPlant floor aliasing, broad
Rocket wall caps and whole-room composition remain open. Static location
fixtures are not earned arrivals or authored-service proofs.

Ordinary controller input follows PowerPlant(7,15)->(8,15)->(7,15) across64
logged frames: terrain active/settled, no warp or screen fade, one terrain
build throughout. This checks the waiting-area aisle, not every chair approach
or dynamic reveal pose. The complete311-map/225-state audit is parsed exactly
equal to the preceding verified sofa baseline:311 identical map records,
all alternatives/support/claims/geometry counts exact and zero mesher errors.
Chair coverage remains88 cells and sofa coverage48 cells. Orientation changes
do not change these counts; visual review supplies the direction evidence.

All captures explicitly use HEADLESS=1, CAPTURE_TARGET=image and PREVIEW_MUTE=1
with the CRYSTAL_RENDER_TEST_/CRYSTAL_ prefixes. Winit and macOS window keepalive
are disabled; ScheduleRunner drives image readback. No windows, app navigation,
focus changes, staging, commits or exports were used. Ignored receipts, source
compile failure, strict audit comparison and reviewed provenance remain under
target/kanto-resume with facility-chair-facing prefixes. The deleted handoff
stays deleted. Full all311/original16/shared-furniture acceptance remains open.


Facility table/chair wood now uses its live source finish — 2026-10-18

New native_facility_furniture_materials.rs authenticates the four original
facility kits by export and part names, rejecting unknown parts/materials or
missing gold/brown/light/ink roles. Model::append_native_facility_furniture
retains fitted geometry and maps those roles onto samples from the complete
source drawing. Tabletop/wood, brown aprons/insets, light edges/pages/cups and
dark ferrules/binding now follow the live atlas. The small native printed page
face is unchanged. No fixed source palette copy, model asset or pack changed.

Facility append revalidates this material contract before appending any floor
or body. Existing source/custom/reserved guards retain priority. Thirty-one
original bodies change finish: one square table, five document desks, three
meeting tables and22 chairs across MrPokemonsHouse, PowerPlant,
RuinsOfAlphResearchCenter, SilphCo1F and TeamRocketBaseB3F. Silph's selected
source-gold meeting model is unchanged, making32 total objects/260 source cells.
Chair rotation now operates on the newly appended textured body; its ten
corrected headings, twelve unchanged headings, owned plots and actor clearance
remain verified. Source floors/support stay independent and unchanged.

Source Cargo succeeds5m09s. Two focused tests pass4.22s against the actual pack:
all32 objects retain exact positions/normals/indices and support, all four live
material roles are present, the Silph selected-table control is exact, cached
models remain immutable across three fitting scales, and unknown part/material
contracts reject atomically. Existing chair/body/page tests now inspect native
textured vertices rather than vacuously inspecting an empty solid mesh. An
initial test compile incorrectly passed a compressed Source wrapper to the
JSON string parser; it was corrected using bounded decode_bytes before this
successful run. Thirteen furniture controls (including these two) pass22.77s,
ten perimeter controls34.91s,43 fade controls0.41s and52 shared model controls
0.14s. Native examples build succeeds5m09s.

Fifteen static native/source pairs include front/side views of the square
table, PowerPlant meeting table and both desks, Ruins desk, both Rocket tables
and both Rocket desks, plus Silph and PowerPlant waiting-chair controls. The
RL final pair makes16 pairs/32 full images, all individually inspected along
with both moving captures. Gold bodies, brown lower trim and light details
are scoped visually verified. The source-native small document print, cup
opening/handle, drawer and leg forms remain readable. Each pair preserves its
own state with distinct actual player/object centers. These location fixtures
are not earned arrivals or service proofs. Ordinary PowerPlant input follows
(7,15)->(8,15)->(7,15),64 active/settled terrain frames, no warp/fade and one
terrain build throughout. It checks the waiting aisle, not every approach.

The strict311-map/225-state audit has zero mesher errors. Exactly17,732
triangles/53,196 vertices transfer from solid colors to live textured finish,
with no added/removed geometry: MrPokemon1,756 triangles, PowerPlant5,332,
Ruins972, Silph2,856 and RocketB3F6,816. All five rooms' other fields, all306
other map records, source/support/ownership counts, totals and all225
alternatives remain exact against the preceding chair-facing baseline. These
counts supplement the actual visual review; they do not establish full-room
fidelity alone.

All captures explicitly stay HEADLESS=1/CAPTURE_TARGET=image/PREVIEW_MUTE=1,
without native windows, app navigation or focus changes. Ignored receipts,
strict comparison and reviewed provenance use facility-native-finish prefixes
under target/kanto-resume. No export, staging or commit occurred. The deleted
handoff remains deleted; these canonical guides retain the full objective.

Remaining observed defects include fixed pale instrument-workbench bodies
beside the now-gold desks, missing PowerPlant/Silph rear backing, PowerPlant
floor aliasing, broad Rocket wall caps, actor/chair and companion alignment,
fine trim, broader dynamic reveal, composition and authored services. The
instrument workbench path is a separate source family; this finish does not
close it. Full all311/original16/shared-furniture acceptance remains active.


Facility workbench finish verified; slot-bank correction reopened by source layout — 2026-10-18

Ten facility_workbench bodies now use live gold/brown/light/ink from their own
complete block28 drawings. native_facility_furniture_materials authenticates
all named bench, casing, key, trim and recess parts before any floor/body
append. The current geometry, live eleven-pixel display, operator floor,
actor positions and support are unchanged. The four maps contain MrPokemon1,
PowerPlant3, Ruins1 and RocketB3F5 benches (120 owned cells). No assets, palettes,
pack or exports changed. New facility_workbench_finish_tests pins every body
against the existing fitted model and the real pack's four grayscale pigments,
all claims/support, cached geometry at three scales and unknown/missing roles.
Old readout tests now inspect the actual display tail and native body, not an
empty solid pass.

Source Cargo succeeds9m22s; two focused tests pass15.05s. Thirteen workbench/
radio checks pass131.11s,13 table/chair94.59s,52 shared-model0.42s,ten perimeter
85.66s and43 fade1.48s. The focused two are included in the13 workbench checks,
not additional unique cases. Native examples build succeeds13m58s. Nine static
front/side source/native pairs show all ten benches, including Rocket's boss
room control. The RL final pair makes ten pairs/20 full images; every image
and both moving captures was individually reviewed. The gold bodies and
readable keys/recesses/readouts are scoped finish acceptance. Workbench form,
whole-room composition, remaining walls, floor aliasing, broad Rocket caps,
actor alignment, broader dynamic reveal and services remain open.

Ordinary PowerPlant input follows(6,9)->(7,9)->(6,9),64 active/settled frames,
no warp/fade and one terrain build. Each pair preserves its own state with
distinct player/object centers. Fixtures are not earned arrivals. All captures
explicitly remain headless/image/muted; no windows or app/focus changes.

The311-map/225-state audit has zero mesher errors and the exact13,440-triangle/
40,320-vertex transfer from solid to textured benches: MrPokemon1,344 triangles,
PowerPlant4,032, Ruins1,344 and Rocket6,720. Every other field in these rooms,
all source/support/claim/totals and all225 alternatives stay exact.304 other
map records are entirely identical. Three additional maps have only external
solid-geometry deltas observed during concurrent pine work: Dragon'sDen
+1,184 triangles/+840 vertices, Ilex+42,756/+25,212 and SafariBeta-260/-780.
The shared foliage/pine.rs generator changed after the preceding native build;
its source and geometry were inspected, not edited here. The first strict
comparison rejected those deltas; the final bounded comparison explicitly pins
all six external values and keeps their visual acceptance false. This is not
an assertion that all307 other map geometry records are unchanged. Receipts,
comparison and reviewed provenance use facility-workbench-finish prefixes
under ignored target/kanto-resume. No staging/commit occurred.

Human clarification reopens the arcade row interpretation: slots are two
source tiles tall, stools one; Celadon has four two-wide banks, six positions
long. Fronts must face their stools, including back/end units facing outward.
Existing source interaction events pin rows6..11. Current rendering treats the
projected row5 header as separate fronts and leaves the final stool row's end
art unaligned. A new native_arcade_banks.rs has been drafted but is not yet
integrated or tested at this checkpoint; its full-bank source guard and six
row placement must now be wired, tested and reviewed. Prior pair-only approval
does not close this human-corrected bank layout. No completion is claimed.

The human also did not recognize the vague PokéCom gray-unit question. An
existing source/native pair (pokecom-office-walk-review/office-aisle) shows rows
of keys in the three rectangles beside the monitors; keyboard trays are the
working source interpretation. The generic computer places keys below its
screen and leaves these adjacent drawings flat. Investigate that actual layout
without requiring the human to label an ambiguously described fixture.
Full all311/original16/shared-furniture acceptance remains active; the deleted
handoff remains deleted and both canonical guides retain the objective.


Arcade banks integrated; revised visible-front/stool tests compiling — 2026-10-18

Human correction is authoritative: slot drawings are two tiles tall beside
one-tile stools; Celadon has four two-wide banks, six positions long. Back/end
units must face outward. New native_arcade_banks.rs is now wired before the
individual profiles in modeled_interiors. It authenticates the complete56-cell
header/middle/end drawing and builds six pairs on source rows6..11 at32-pixel
height. Row5's projected header no longer creates unpaired fronts. Both ends
and all intermediate rows face the outer aisles through the existing rigid
pair adapter. Custom-owned or tile-edited complete banks yield as a whole;
stale source append fails before geometry. Unknown model contracts retain
source rendering. No collision, support, actors, scripts, packs or assets change.

First source build succeeds4m44s, but one of two tests fails: the test incorrectly
requires the hidden rear faces of the closed screen blocks to point outward.
The stale/source/custom/atomic test passes. The corrected test identifies the
actual forward display faces using the original model's +Z screen faces, adds
an explicit full32-pixel height gate and matches all36 Celadon/30 Goldenrod
bank stools. Cargo93689/session73634 is confirmed live compiling that revision,
logged in receipts/native-arcade-banks-tests.log; the original failure is kept
as native-arcade-banks-closed-screen-test-failure.log. Do not claim test or visual
acceptance yet. The headless driver run-native-arcade-banks-review.py is prepared
but has not started. It gates fresh arcade/casino/perimeter/fade controls and a
fresh native build, then ten static pairs and two UD final pairs/moving samples.
Use the completed facility-workbench-finish311/225 audit as the new baseline,
including its explicitly observed external pine changes. Full slot-bank visual
verification and full all311/original16/shared-furniture acceptance remain open.


Slot banks verified facing outward toward stools — 2026-10-18

Complete Celadon and Goldenrod banks now render six pairs at source gameplay
rows6..11, with full32-pixel cabinet height. Celadon retains four two-wide banks;
Goldenrod retains three. Left fronts face west and right fronts east, including
back/end units. All36 Celadon and30 Goldenrod bank stools match a single facing
station. Goldenrod's two table stools remain separate. The projected header no
longer creates unpaired fronts; the final machine row replaces the old flat end
art. Existing source collision, support, actors, scripts and pack are unchanged.

Two focused external-pack tests pass20.76s after the corrected visible-front
assertion;23 arcade,23 casino,ten perimeter and43 fade controls pass (overlapping
selections, not a unique sum). Native examples build successfully5m37s. Twelve
headless native/source pairs (24 full images) and four moving samples were
individually reviewed at matching noon/position/state. Both UD walks prove
(10,7)→(10,6)→(10,7),64 settled frames each, one terrain build and no warp/fade.
No windows were opened. Receipts and reviewed provenance are under ignored
target/kanto-resume/native-arcade-banks-headless-review and receipts/native-arcade-banks-*.

The strict311/225 comparison against the completed workbench audit confirms
exactly56 former bank-end cutout cells become modeled, with total geometry
counts unchanged in both game corners. All225 alternatives and support remain
exact.307 other map records are exact; DragonsDenB1F and IlexForest have separate
concurrent pine solid-geometry changes, explicitly pinned in the comparison
and not visually accepted by this work. No assets or packs were regenerated.
Scoped bank count, height, alignment and outward orientation are accepted;
whole-room finish/composition, seating actors, PokéCom keyboard layout and the
remaining all311/original16/shared-furniture objective stay open.

PokéCom complete side-keyboard workstation repair in verification — 2026-10-18

The prior slot-bank turn completed source/layout tests, individual headless
native/source review and strict311/225 comparison; it is concrete progress.
The wider interior objective remains active. The deleted handoff file remains
deleted; these canonical guides retain current evidence and open work.

Current source adds station_local for the complete4x3 monitor/side-keyboard
source drawing. Three guarded stations claim36 cells instead of leaving side
keyboard drawings behind. The selected closed computer kit is adapted at runtime:
its complete3072-vertex keyboard moves right, its tabletop/legs span both halves,
and the monitor/other electronics stay in the left half. No model assets or packs
are exported. Existing source collision/support/controllers remain unchanged.
The legacy monitor-only placement is removed, and stale station append rejects
before geometry. Unknown model roles/keyboard topology retain source rendering.
Counter header candidates keep all three physical red backing courses while
six former no-body face cells become station-owned. Office tests now distinguish
this ownership from removal of the backing.

Initial Cargo session59248 terminates successfully: compile18m16s, one
external-pack complete station/support/source test passes14.08s. Its actual test
binary also contains the new three-scale keyboard/monitor/cache/unknown-contract
test, which passes0.02s when invoked directly. Both results are preserved as
pokecom-side-keyboard-intermediate-tests.log and intermediate-geometry.log.
Counter-header ownership was refined during that compile, so neither result
alone proves final-revision acceptance. The pipeline PID3592/session93710 has
now launched fresh final-revision Cargo5424 (pokecom selection), verified live,
logging receipts/pokecom-side-keyboard-tests.log. It next gates perimeter/fade
controls, native rebuild, five static pairs plus one RL final pair/moving samples,
and a strict311/225 comparison against the completed slot baseline. Follow
receipts/pokecom-side-keyboard-review-driver.log and the same live handle; do not
restart on an observation timeout. No new native visual acceptance is claimed.
Whole-room office materials/composition/actors and the full all311/original16/
shared-furniture objective remain open.

Power Plant backing diagnosis — 2026-10-18

The current workbench headless pair at(6,9) was inspected again: the full rear
backing is absent behind the two sofas and equipment, while generic side
walls borrow gold furniture paint. native_facility_room_shell::source currently
admits only the two8x8 small facilities. The external pinned source confirms
PowerPlant is20x18 gameplay tiles and has exactly the(2,17)/(3,17) entrance pair.
Its complete north drawing includes cupboards, native wall22, both sofas, the
divider join and equipment. Next repair must authenticate that complete course,
keep backing behind every fixture, retain the internal divider and exit, and
replace generic perimeter generation only when the native contract succeeds.
No Power Plant source change or new visual acceptance is made at this checkpoint.

PokéCom backing ownership corrected; lower station gets a real counter — 2026-10-18

Fresh pokecom Cargo5424 terminates after11m39s compile:17 pass,one fail54.30s.
The failure is the office test's assumed seven counter placements; actualnine
are correct. Exact pinned source inspection shows the complete stations take
four of the old six no-body red counter-face cells, leaving two between the
upper desks. The lower station additionally owns two formerly flat red cells.
Tests now expect nine counter placements, three unchanged physical headers,
two remaining face cells and44 counter-owned cells. The failed result is kept
in receipts/pokecom-side-keyboard-counter-ownership-failure.log.

That inspection also exposes the lower station's red band as counter backing,
not floor. station_backing_donor authenticates its complete monitor/keyboard
and the original upper floor apron, then chooses a complete straight header
from this same room for live countertop/front material. append_station_backing
adds the matching eight-pixel-high closed course behind this lower desk only;
the three existing upper headers remain intact. The source test compares this
actual appended tail and checks height/placement; electronics retain their own
five gray paints. No actor, collision, support, script, pack or asset changes.

Final revised pipeline PID9709/session15220 is verified live with Cargo9712
(--lib pokecom), logging receipts/pokecom-side-keyboard-tests.log and
pokecom-side-keyboard-final-review-driver.log. Earlier pipeline session93710 is
terminal after the expected test-count failure; do not restart it. The current
pipeline gates all18 pokecom checks, perimeter/fade, fresh native examples,
five static pairs plus one RL pair and a full311/225 audit. The RL fixture now
uses clear(1,29)→(2,29)→(1,29): source scientist(4,28) walks vertically, so the
old(3,29)→(4,29) aisle could be occupied. Both standing scientists are confirmed
at(7,27)/(7,29), directly in front of the right-hand keyboards. Native pack SHA
remains a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.

The prepared strict comparison now requires12 keyboard cells plus two red
cells become modeled, four former counter-face cells transfer to stations,
and exactly24 triangles/56 vertices for the new lower backing. This expected
comparison is not yet executed against a new audit. No final native visual
acceptance is claimed. Power Plant backing is the next diagnosed repair; the
full all311/original16/shared-furniture objective remains active.

Power Plant divider must reach the ceiling — human correction, 2026-10-18

Human explicitly requires the Power Plant divider to be full height. Inspection
finds a shared legacy-network defect: WALL_HEIGHT/caps are32px, but both exposed
side bands use tile_height and stop at16px with the normal8px source grid. The
mesher now divides the actual wall height into two continuous side bands. This
closes every outward side to its existing cap without changing footprints,
openings, source cells, support, UVs, normals or triangle/vertex counts. The same
floor-to-cap invariant applies to this existing facility fallback vocabulary;
fully authored Rocket walls still retire that network as before.

New mesh/facility_divider_height_tests.rs checks actual external Power Plant
sources: every side segment has exact[0,16]/[16,32] spans, every occupied cell
retains its ceiling cap, claims match all existing horizontal/vertical/end-cap
placements and every other source cell/opening remains unclaimed. Support and
original source metadata are exact; an unrelated map emits nothing. The queued
run-power-plant-divider-height-review.py/session22734 uses the same owned Cargo
cache after the active native build, then requires divider, retired-facility,
Rocket ownership, perimeter/fade checks and a fresh native build. Seven static
pairs and two UD pairs/moving samples cover the room from opposite sides and
along the partition from both camera sides. All captures are explicitly headless/image.
Source collision confirms(11,9) is floor beside the divider; the earlier(10,9)
probe is rejected by the authoritative shell as a wall, not bypassed.

Five before pairs from the completed slot native binary (PowerPlant code then
unchanged) were produced and individually reviewed: (9,9) rotations0/2/6,
(7,3) and(13,11) front. Provenance is marked reviewed; they are diagnostic before
views only. They also confirm the separate missing north backing/incorrect
gold outer boundary remain outstanding. Do not treat this divider correction
as completion of that outer-wall repair or whole-room fidelity.

The revised PokéCom tests now pass18/18 in50.95s. Its native build is still
verified live as Cargo11649 beneath pipeline9709/session15220; final paired
images and audit have not yet been accepted. No windows, model exports, pack
regeneration, commits or other goals' cache changes were made. The full
all311/original16/shared-furniture objective remains active.

PokéCom side-keyboard layout visually verified; divider height tests pass — 2026-10-18

Revised PokéCom source passes18 checks50.95s; native examples build11m25s.
All six native/source pairs (12 full images) and two moving samples were
individually inspected offscreen. Each of the three desks has one monitor and
one keyboard beside it; the old duplicate below-screen keyboards and flat side
keyboard drawings are gone. Upper counter backing stays connected, the lower
red band is a matching real8px counter, and the entrance/aisle stays clear.
This accepts layout/backing only: CRT roof highlight, broader fixture finish,
floor/wall materials, composition/actors and hidden areas remain open.

The strict311/225 comparison passes after inspecting exact differences. Twelve
keyboard cells and two red cells become modeled; four counter-face cells
transfer to complete stations. The lower backing adds24 indexed triangles;
whole-room net textured vertices rise48 (raw backing builder56), rather than
the initially predicted56. All source/support/225 alternatives are exact.
308 other map records are exact; two concurrent forest solid-geometry changes
are explicitly pinned: DragonsDenB1F(+33,858tri/+28,786v) and IlexForest
(+22,164,516tri/+17,019,176v). Current pine/forest_instances/foliage/understory
changes were inspected and not edited; their visual acceptance is not claimed.
The invariant receipt is pokecom-side-keyboard-full311-invariant-summary.json.
Ordinary RL input proves(1,29)→(2,29)→(1,29),64 active/settled frames, one build,
no warp/fade; paired states and actual Player/Object centers stay distinct.
Pipeline session15220 is terminal0. Reviewed provenance and scoped critique
are in pokecom-side-keyboard-headless-review.

The human's Power Plant correction remains the immediate active work. All six
facility_divider checks pass6.17s after the queued source build11m30s (including
waiting behind the prior native build). Retired facility/Rocket ownership,
ten perimeter43.07s and43 fade controls pass. Pipeline13208/session22734 is
confirmed live building fresh native examples; before views are not after
acceptance. The prepared nine after pairs include opposite divider sides and
UD movement. Separate missing Power Plant north/outer backing remains open.
The full all311/original16/shared-furniture objective remains active.

Power Plant divider height verified offscreen — 2026-10-18

The divider side bands now span 0–16–32 source pixels and meet the existing
ceiling cap continuously. The long laboratory separator, entrance partition,
ends and joins retain their source footprints and doorway openings. Six
facility-divider regressions, retired-facility/Rocket ownership controls,
ten perimeter checks and 43 fade checks pass. Fresh native examples built
in 12m11s; pipeline session22734 is terminal0.

Individually inspected nine matching native/source pairs (18 full images)
and four moving samples, including opposite sides/cameras. Upper faces close
against caps and the player remains readable. Both ordinary UD walks follow
(9,9)→(9,8)→(9,9), 64 active/settled frames per camera, one build, no warp/fade.
The strict audit is entirely equal to the completed PokéCom baseline: all311
map records, all225 script alternatives, source claims, support and geometry
counts remain exact. See power-plant-divider-height-full311-invariant-summary.json
and reviewed provenance in power-plant-divider-height-headless-review.

This accepts the divider-height correction only. Missing north/outer backing,
floor aliasing, fixture finish and broader room composition remain open;
fixture arrivals are not earned gameplay progress. The all311/original16
shared-furniture objective remains active.


Power Plant outer backing implementation in verification — 2026-10-18

The prior divider correction was concrete progress: height tests, native
opposite-side/walking inspection and an exact311/225 comparison completed.
The next diagnosed defect is the room boundary: generic gold cabinet paint
and flat green rear drawing. Extended the existing native facility room shell
to the actual20×18 core-tile Power Plant with its full ten-metatile rear
course, including cupboard/sofa/divider/machine source drawings. The source
wall's green field/light/ink samples now define continuous32px backing and
returns; both real(2,17)/(3,17) exit tiles remain open. The existing two small
facility rooms retain their exact dimensions and source contracts.

A new external-pack regression checks every rear column through31px height,
both side joins, front openings, actual wall-paint donors, unchanged support
and source, all160 rear-source edits rejecting atomically, and exactly38 old
flat wall cells removed while floor/support remain. Source tests are running
in session45711/Cargo22133; pipeline64348 waits for that exact process, then
runs divider/perimeter/fade controls, builds native examples, captures nine
paired views plus moving samples offscreen, and audits all311/225 states.
This is implementation awaiting evidence, not visual acceptance. Remaining
floor aliasing, fixtures, composition and the full interior goal stay open.


Power Plant outer-backing source proof — 2026-10-18

Four shell tests pass21.48s (source build7m19s); six divider,11 perimeter
39.41s and43 fade0.40s controls pass. Session45711 is terminal0. Pipeline
64348/Python22297 and native Cargo24283 are confirmed live. Native paired
inspection and exact311/225 comparison remain required; no visual acceptance
is claimed from source tests alone. Power Plant is also excluded from the
existing live blue/cream checker finish, explaining its remaining microdither;
that source-authenticated material extension follows backing verification.

Power Plant backing visually verified; checker finish in verification — 2026-10-18

Closed the missing rear/outer wall with the existing source-green facility
shell, behind cupboards, sofas and machines and through both corner returns.
Four shell tests pass21.48s; six divider,11 perimeter39.41s and43 fade0.40s
controls pass. Native examples built4m04s, pipeline64348 is terminal0. Nine
paired native/source views (18 full images) plus four moving samples were
individually inspected offscreen. Both UD walks settle(9,9)→(9,8)→(9,9),64
frames each, one build, no warp/fade. The wall/returns stay continuous, actual
exits stay open and the player remains readable. Backing scope accepted;
whole-room composition, fixtures and machinery-area floor finish remain open.

The strict311/225 comparison permits only PowerPlant's +8,604 textured
triangles/+17,360 vertices. All other fields, source/support, all310 other map
records and all225 alternatives remain exact. See the reviewed provenance and
individual-room-review.json in power-plant-outer-backing-headless-review and
receipts/power-plant-outer-backing-full311-invariant-summary.json. This native
build predates the separate checker-floor source change below.

Extended the existing authenticated blue/cream ceramic finish to PowerPlant.
The real room has234 blue+234 cream source cells across15 metatile patterns.
The new real-pack regression requires468 finished cells and preserves every
non-floor corner/UV, source and support. Existing complete phase, edited
source and custom-ownership tests now include PowerPlant; the broad service
floor fixture now uses actual all-blue block30 rather than machine block00.
Six facility-floor tests pass14.30s after a4m25s source build; source session
14030 is terminal0. All34 shared-floor checks10.44s, six divider3.64s,
11 perimeter38.34s and43 fade0.39s controls pass. Pipeline6607/Python26861 is
live with fresh native Cargo27520; matching views/walking samples and the
full311/225 comparison follow. Checker finish is not yet visually accepted.
Silph's existing diagnostic native/source pair confirms the same flat green
rear-course defect. Its actual eight source blocks and two real south exits
were inspected externally; the north stair artwork at source columns26–27
requires an opening rather than a sealed continuation. Repair follows Power
floor verification. The full interior goal remains active.


Production divider scale corrected; Power checker finish visually verified — 2026-10-18

Reopened the prior divider-height acceptance. The earlier face-band correction
closed faces only at the unscaled32-world-unit cap; source tests used8-unit
tiles, but the native shell publishes32-unit tiles (bevy_shell::TILE_SIZE).
The outer shell ceiling is128 world units there, so the divider was still too
low. Earlier height acceptance in the ignored critique/invariant receipts is
withdrawn. This is a concrete missed scale conversion, not a lighting issue.

append_facility_divider_network now converts32 source pixels by tile_height/8,
matching the room ceiling at4/8/16/32-unit tile scales. The actual-pack test
checks continuous side bands and caps at all four scales, unchanged source,
claims/openings/support, and wall-only reveal ranges. Each cap/side section
uses the existing depth-writing player reveal; its floor stays opaque. Six
divider tests pass4.08s after6m34s including the wait behind Silph tests.
Production height/readability still requires the new native captures.

Power Plant checker finish is scoped visually accepted: six floor tests14.30s,
34 shared-floor10.44s, six divider3.64s,11 perimeter38.34s and43 fade0.39s
controls pass; native build3m40s. All nine pairs (18 full images) and four
moving samples were individually inspected; blue/cream layout is retained
with quiet ceramic fields, floors remain below their walking datum, and
machine/conduit/carpets retain their source drawings. Both UD walks settle
64 frames each, one build, no warp/fade. Its strict audit is entirely equal
to the backing baseline: all311 map records, all225 alternatives and support
unchanged. Pipeline6607 is terminal0. See the reviewed checker-finish provenance
and power-plant-checker-finish-full311-invariant-summary.json. This native
build predates the divider scale and Silph changes; it proves floor finish,
not corrected production divider height.

Silph backing now authenticates the full128-cell rear course, restores58 flat
wall-art cells to a source-green32px wall and connected side/front returns,
and leaves north source columns26–27 open for the stair artwork and both real
south exits open. Five shell checks pass25.07s after4m55s source build; current
fresh scale-test binary is also rerunning them. New test covers all rear
columns/heights, side joins, exits, all128 source edits, preserved stair art,
custom ownership and unchanged source/support. It is not yet visually accepted.

Combined pipeline17970/Python30330 is live. The current binary passes five
shell24.97s, six facility-floor17.40s,34 shared-floor12.12s, both retired
ownership controls,12 perimeter56.58s and43 fade0.66s checks. Native Cargo30886
is confirmed compiling fresh examples. It then captures18 native/source pairs
across PowerPlant, SilphCo1F and TrainerHouseB1F plus three ordinary walks and
audits all311/225 states. All renders remain headless. Fresh full-height
player visibility and Silph backing appearance remain unverified. Broader
room/material/actor, service/reachable-state and original16 obligations keep
the full goal active.


Silph backing verified; full-height divider reveal refinement — 2026-10-18

Fresh native examples built9m42s; combined pipeline17970 is terminal0. Seven
Power Plant static pairs were individually inspected and confirm the divider
now reaches the actual128-world-unit ceiling at32-unit native tiles, with
continuous joins and the original openings. Its cabinet depth-coverage reveal
is visually rejected: the grainy mask tints the occluded player gold. Height
is correct, but this is not full divider/readability acceptance. The two Power
walk pairs and three TrainerHouseB1F pairs in that artifact are superseded and
not individually reviewed; provenance explicitly records that limit.

Replaced only the divider's exact wall/cap reveal ranges with the existing
smooth layered-alpha policy used by native timber dividers. Floor remains
opaque and all source geometry, indices, UVs, footprints and support remain.
The four-scale regression now requires exact layered ranges, no cabinet pixel
mask, complete cap/side coverage and an opaque floor. All six divider tests pass
3.49s after6m14s source build; session14530 is terminal0. Pipeline26137/
Python35722 is live; compaction and five shell17.32s controls pass, with the
remaining facility/shared-floor/ownership/perimeter/fade checks underway.
It then builds fresh native examples, captures12 PowerPlant/TrainerHouseB1F
pairs and two walks, and audits311/225. New smooth reveal is not yet visually
accepted; the prior grainy reveal remains rejected.

Silph's separate backing scope passes visual review: six paired native/source
views (12 full images) and two moving samples were individually inspected.
Green rear backing is continuous behind the reception chairs and seating;
side/front returns connect, north source stair columns26–27 stay open and
both real south exits stay clear. Player/companion readability holds in the
reception, stair, aisle and walking poses. Ordinary RL settles(7,5)→(8,5)→(7,5)
in64 active/settled frames, one build, no warp/fade. Fixtures/composition,
actors and broader services remain open; fixture arrival is not earned travel.

Strict audit passes after accounting for the inspected external schema
addition open_water_level=None on all311 interiors (derived from terrace
open-water datum in audit_authored_assets.rs). Only Silph's wall geometry
changes: +5,444 textured triangles/+11,120 vertices. All310 other map records'
preexisting fields, all225 alternatives, source claims and support remain
exact. See facility-wall-scale-and-silph-full311-invariant-summary.json and
the scoped critique/reviewed provenance. The compaction regression also passes
0.01s, preserving complete reveal owners and opaque floors. Full all311,
original16, shared-furniture, native service and earned-state goals stay active.


Power Plant full-height/smooth reveal verified; Trainer basement backing underway — 2026-10-18

Fresh smooth-reveal native build passed7m08s; pipeline26137 is terminal0.
All nine PowerPlant native/source pairs (18 full images) and four moving
samples were individually inspected. The divider reaches the actual128-world
ceiling at32-unit native tile scale; joins and source openings are intact.
The smooth translucent reveal replaces cabinet pixel grain and keeps player/
companion silhouettes readable on both sides and while moving; floor stays
opaque. Both UD walks settle(9,9)→(9,8)→(9,9),64 active/settled frames each,
one build, no warp/fade. Scoped height/reveal accepted; machinery-area floor,
fixture finish and whole-room composition remain open.

Six source divider tests3.49s, one compaction, five shell17.32s, six facility
floor13.87s,34 shared floor10.61s, both ownership controls,12 perimeter107.93s
and43 fade1.09s checks pass. Strict311/225 audit equals the completed Silph
baseline entirely: all map records, alternatives, claims/support/counts exact.
See facility-divider-smooth-reveal-full311-invariant-summary.json and reviewed
Power-only provenance. Its three TrainerHouseB1F pairs remain unreviewed and
are superseded for reception-counter/backing work; this is not whole-family
or whole-room visual acceptance. This native binary predates the changes below.

TrainerHouseB1F's actual10×16 core-tile room has one east warp(9,4). Extended
native facility backing through its full rear drawing and upper returns only;
returns join the existing gold arena at source row12, leave source stair
rows8–9 open, and add no second green lower enclosure or invented south exit.
Source authentication covers80 rear cells plus48 stair/arena-join cells; the
paint donor is its actual green tile22 at source(4,0), not a machine cap.
Six shell checks pass22.38s; new regression covers source/native scales,
every rear column and side join, the landing, all128 edits rejecting, unchanged
source/support and22 old flat wall cells removed while stair artwork survives.

The receptionist at(7,1) stands inside a counter, distinct from arena walls.
Its unique source$21 return anchors a connected22-cell counter component;
only that component stays at8-source-pixel desk height. Arena walls remain
at32-source-pixel ceiling height, without guessed viewport rows. New source/
production-scale and padded-grid tests verify both heights and unchanged
support/claims. The first run passed six existing tests but its new count
assertion expected production162 cells in a direct raw-source fixture; the
fixture also includes eight cabinet caps retired in production. It actually
has22 counter+148 wall/cap cells=170, including exactly eight raw cabinet caps.
The expectation is corrected and explicitly tests that distinction; failure
receipt is preserved. All seven divider/counter checks pass10.58s after5m47s;
source session71449 is terminal0. Current six shell28.78s, six facility-floor
14.02s,34 shared-floor10.49s, compaction and both ownership controls also pass.

Pipeline71203 stopped terminal1 on the broad perimeter check (12 pass/one
fail52.17s): it expected an inferred green front wall even where the authored
gold arena already supplies the enclosure. The check now requires upper
returns to stop at their arena join and separately verifies actual source
arena front-face coverage at every source column, preserving the front-wall
gate without adding a duplicate green enclosure. Failure receipt is retained.
That rerun stopped terminal101 (12 pass/one fail70.33s); pipeline4056 is
terminal1. Its stricter front check assumed a flat exterior at source row32.
The actual$3e/$3f corner drawings end in gray footings: corner wall bodies end
at row31, while the middle$34 course reaches row32. The test now verifies the
front-facing solid at every column along that exact stepped source outline,
rather than forcing the arena into a different rectangular footprint. Both
failed perimeter receipts remain preserved. No runtime geometry was changed
for this assertion correction.

The corrected source-outline run passes all13 perimeter checks48.98s after
5m21s compile; source session79459 is terminal0. Pipeline63365/Python44335 is
live and passes fresh seven divider/counter7.06s, six shell27.25s, six facility
floor18.58s,34 shared floor14.45s,43 fade0.51s, compaction and both ownership
controls. Native Cargo45304 is compiling fresh examples. Thirteen paired
views plus two walks and311/225 comparison follow. Reception fixture(6,3)
avoids the actual(7,3) battle-prompt coordinate event; actual warp(9,4) is not
used as a spawn. Trainer backing/counter and receptionist visibility await
native review.

The original16 matrix still correctly leaves Lance's south barrier open:
the inspected enclosed portal checkpoint concerns the north door only. A
fresh saved-session capture tool is queued in session18578/Cargo45695 after
the active native build. Pipeline51709 waits for that exact build and captures
three offscreen pairs from the existing controller-owned entry-locked save,
with state preservation and unchanged save hash checks, to diagnose the
actual south barrier in current production geometry. It does not seed door
flags or claim a continuous native League run. No Lance code has changed;
old entry-locked/blue-wall images are diagnosis history, not fresh acceptance.
The full all311/original16/shared-furniture goal remains active.


Power Plant divider ceiling height verified; Trainer basement backing verified — 2026-10-18

Power Plant's divider cap and every side band now scale to the room ceiling:
128 world units at the production32-unit tile scale. Nine paired source/native
views and four moving samples were inspected offscreen, including opposite
sides. Joins and openings remain intact; smooth wall transparency keeps the
player readable. The311-map/225-variant audit retains exact source claims,
support and geometry counts. Height/reveal scope is accepted; broader room
materials and composition remain open.

Trainer basement pipeline63365 and saved-session build18578 are terminal0;
fresh native examples built7m16s. All13 pairs/26 full images and four moving
samples were inspected: continuous green backing, upper returns joined to the
gold arena, open east stairs, low reception counter and visible receptionist.
Two64-frame walks settle along(5,3)→(6,3)→(5,3) and(5,8)→(5,7)→(5,8).
Seven divider/counter, six shell, six facility-floor,34 shared-floor,13 perimeter,
43 fade, compaction and ownership checks pass. Strict311/225 comparison changes
only Trainer geometry (+2166 textured triangles/+4420 vertices);310 other map
records, all alternatives, claims and support are exact. See the smooth-reveal
and Trainer connected-backing invariant receipts under target/kanto-resume/receipts.
Reception service/battle and whole-room visual acceptance remain open.

Lance diagnostic pipeline51709 is terminal0. Three paired views from the
unchanged controller-owned entry-locked save confirm its south barrier remains
flat floor artwork; implementation and native continuous League proof remain
open. This does not close the broader interiors goal.


Lance south barrier geometry verified offscreen — 2026-10-18

Added mesh/lance_south_barrier.rs and integrated authenticated perimeter
ownership. Actual base$31 keeps both entrance warp lanes open; scripted$34
builds a closed gray course at the existing48-source-pixel blue-wall ceiling,
with cap, sole, back and ends. It joins both sides at source row46. All80
south source cells and the north/side drawings authenticate before suppressing
the duplicate inferred black exterior. Source state, collision, flags, warps
and support remain unchanged. Custom/edited/partial drawings retain fallback.

The provisional$33 interpretation was rejected after reading the actual
collision table: it blocks its north half and is not an authored entrance
alternative. The final source regression explicitly rejects it. An initial
test-helper path compile error was repaired; failed receipt is preserved.
Final source session15929/Cargo51438 is terminal0:8 selected tests59.70s after
6m40s compile/queue (seven Lance checks plus one unrelated balance-name match).
These cover both authored south states/either north state, source8/native32
scale, complete face/volume/winding, every80-cell edit, custom ownership and
unchanged open floor/support. Current-binary perimeter session94359 is also
terminal0:13 checks53.59s. Compaction1 and fade43/0.66s pass.

Pipeline37845/Python50664 and native Cargo51905 are terminal0; fresh native
examples built8m07s including their queue. All eight pairs/16 full images and
eight moving samples were individually inspected offscreen. Both side views
show the closed gray course joined to the blue ceiling with finished caps and
ends; the base state clears both entrance lanes. Front view uses the existing
outside-wall cutaway policy. The player stays readable at the wall. The192-frame
closed walk runs(5,19)→(5,22), rejects the fourth Down, then returns to(5,20);
the64-frame open walk runs(4,21)→(4,20)→(4,21). Both stay active/settled with
one build and no warp/fade.

Strict311/225 audit passes:310 other map records and222 non-Lance alternatives
are exact. Lance gains32 base wall claims and eight closed-alternative claims;
all source/support and other ownership fields are exact. The native course
adds116 triangles/232 vertices, removes64 flat-source triangles and retires580
triangles/1160 vertices of inferred exterior: net−528 textured triangles/−928
vertices; solid geometry is exact. Reviewed counts are asserted explicitly.
Receipt: lance-south-barrier-full311-invariant-summary.json. Shared native pack
SHA remains a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Barrier geometry/joins/opening scope accepted. Whole-room fidelity, remaining
original16 room/material/actor obligations and continuous native League remain
open; no broad completion is claimed.

Read-only next-task diagnosis: Power Plant has296 noisy machinery-floor tile
$1c cells in blocks$1a/$1b/$1c/$2c/$2e, distinct from the finished$01/$26 checker.
A separate source-authenticated finish must preserve their partial block masks
and collision drawings. No Power floor edit was made during this Lance work.


Power Plant aggregate floor verified offscreen — 2026-10-18

Added a separate source-authenticated machinery-floor finish for the296 actual
facility$1c cells in blocks$1a/$1b/$1c/$2c/$2e. The existing468-cell office
checker contract is separate. The new surface keeps the blue field and all
five light/seven dark fleck centers from the original8×8 drawing, with live
source pigments and shallow faceted insets. It introduces no slab grid and
stays below the original walking datum; machinery/conduit/wall fragments
retain their source art. Changed/custom drawings yield to their original owner.

New tests cover every external source phase, actual floor counts, protected
non-floor art, source/native8/32 scales, full projected coverage, outward
normals, exact light/dark areas, bounds, support and custom ownership. The
first source build was intentionally interrupted terminal130 after a winding
error was found by inspection, not from a timeout; the ring triangulation is
corrected. Source session46413/Cargo54799 is terminal0: three checks3.95s
after4m16s compile. Six facility-floor17.38s,34 shared-floor12.75s,13 perimeter
46.96s, seven divider7.06s, compaction1 and43 fade0.46s controls pass. Fresh
native examples built3m50s; pipeline49127/Python54968 is terminal0.

All13 source/native pairs (26 full images) and eight moving samples were
individually inspected offscreen. Two extra direct-aggregate walks (session
52298 terminal0) prove(11,9)→(11,8)→(11,9), with tile$1c at both endpoints;
the original two office controls prove(9,9)→(9,8)→(9,9). Each64-frame run has
one active/settled build and no warp/fade. The blue aggregate is finer and
continuous around equipment and partial floor fragments; office checker and
full-height divider remain unchanged. The original floordrawing fleck centers
and live pigments remain, with faceted insets below the walking datum.

Strict311/225 audit passes:309 other map records and all225 alternatives exact.
Power source/support/claims are exact. Its296 actual source cells plus112
existing paired-cabinet ground quads are finished:408 surfaces ×(174−2)
triangles/(522−4) vertices =+70176 textured triangles/+211344 vertices. The
cabinet producer already appends those112 Ground::Diamond quads at height0;
no cabinet body or walkable datum was changed.

The only concurrent external delta is Dragon's Den's existing4-cell notice at
[36,48,37,49], Johto$45 art$4e/$4f/$5e/$5f. The current outdoor-sign binding
transfers world-exterior/route_sign to outdoor-sign/modern-frame. Inspected old
model252 triangles/576 vertices and new1468/3568 account exactly for+1216
solid triangles/+2992 vertices; its live inscription adds2 textured triangles/
4 vertices. All remaining Dragon fields/source/support are exact. These files
were read only; no Dragon visual acceptance is claimed. The checker asserts
these exact exceptions rather than dropping fields. Receipt:
power-plant-aggregate-full311-invariant-summary.json.

Aggregate-floor scope accepted. The machine-housing reveal still leaves pixel
grain over the player/companion at(13,11) and during(11,9) walks; a matched old
(13,11) render confirms it predates this change. Fix that housing reveal next,
keeping its shallow bed opaque. Red/black floor drawings still need source
identity/finish review; machine fixtures, broader room composition, original16
and native continuous League/Hall remain open.


Power Plant smooth housing reveal verified offscreen — 2026-10-18

Replaced only the twelve machine housings' depth-coverage ranges with the
existing layered-alpha policy. The renderer already separates depth writing
from pixel coverage: this uses smooth forward alpha with depth ordering, not
ordinary multilayer blending. The closed4-source-pixel bed and floor remain
outside every fade range. Geometry, UVs, source claims and support are unchanged.

Source session19224/Cargo58970 is terminal0: three equipment checks4.41s after
4m25s build; exact layered ranges, no coverage mask and opaque beds verified.
Three aggregate-floor4.60s, seven divider, compaction1,43 fade0.58s and one
smooth-depth forward-pipeline controls pass. Fresh native examples built6m25s;
pipeline4850/Python59176 is terminal0.

All12 pairs/24 full images and eight moving samples were individually inspected
offscreen. Matched(13,11) front views replace the rejected grain with smooth
housing ghosts; (11,9) moving views no longer stipple either character. Checked
poses show no hidden-face clutter. Low beds stay opaque; clear housings retain
their solid source roofs and controls. Aggregate/office floor, full-height
stripe divider and backing controls remain intact. Four64-frame walks settle
(13,11)→(13,10)→(13,11) in cameras0/2 and(11,9)→(11,8)→(11,9) in cameras2/6,
one active/settled build each, no warp/fade.

The whole311/225 audit equals the completed aggregate baseline exactly: every
map record, source/support/claim, geometry count, variant and global field is
unchanged. No new external exception is needed. Receipt:
power-plant-smooth-housing-full311-invariant-summary.json. Housing-reveal scope
accepted; whole-room acceptance remains open. Red/black utility-floor drawings
still need source identity/3D finish review, alongside fixture/material details
and broader composition. Native continuous earned League/Hall, the original16
room/material/actor obligations and full interiors/shared-furniture goal remain
active; this does not close those requirements.



Native continuous League capture gates corrected — 2026-10-18

Added the opt-in location-tester example review_native_league and an adaptive
RenderWalk policy. It begins outside Will in Indigo Plateau with one initial
legal level100 Mew, four legal moves/max PP and five Full Restores. Later
progression uses ordinary keyboard edges through the existing native input
owner. Routing probes clones only; it never installs their state. Victory
flags, spent/carried PP, Pack healing stock, actual exits, ceremony, credits
and the normal Continue summary are checked rather than supplied.

The driver captures matched 3D/2D checkpoints within the same running App,
pausing presentation/input during readback and then resuming. Battle evidence
requires actual modeled species and room scenery; GPU readback owns captures.
Hall entrance samples wait until visible scroll poses; animation frames and
fully revealed Oak rating pages get separate captures. Blank phase entries
cannot satisfy these pose checks. Each League room requires arrival, actual
walking, battle and earned checkpoints. Art acceptance still requires viewing
every image; generated completion metadata deliberately leaves that false.

Native example builds passed4m45s,4m18s and6m16s. Pipeline18877 is terminal1:
its Python3.9 hash helper failed after compilation and before any native input;
the ignored helper was corrected and the completed build reused. Resume66126
ran the ordinary paired Power Plant UD control successfully11.45s: all two
full images/two moving samples individually inspected, source checksum exact,
64 frames(9,9)→(9,8)→(9,9), one terrain build, no warp/fade. Existing capture
input still works with the added opt-in policy.

The first native League attempt stopped before startup: guessed Indigo tile
(14,4) is not walkable in this actual pack. No battles, flags or saves were
created. The example now derives Will's actual compiled warp and probes all
four adjacent approaches on cloned sessions, requiring a real step/warp before
choosing the initial tile. The native policy follows that actual direction;
no south-facing stair assumption remains. Warp build43585/Cargo73603 passed
5m30s. Adaptive-direction build82195 passed6m16s and its v2 run entered Will
through the real compiled warp(14,3), approaching from(15,3) with ordinary Left.
It walked to(5,8), completed the trainer dialogue and reached the actual
Mew–Xatu battle without spending PP or earning a win yet.

All four v2 field images were individually inspected and rejected: they still
show Indigo Plateau while the core is WillsRoom. A preserved paused state is
insufficient when the rendered room is stale. The v2 run then stopped at the
reviewer's erroneous unconditional row-cache gate. Actual native status was
active PBR/native, modeled MEW/XATU, actual WillsRoom scenery, no source species
or scenery fallback; row-cache receipts are unused for this normal scene draw.
No battle image or native model-quality acceptance is claimed from that status.
The rejection and hash-zero metadata diagnosis are recorded in v2's
individual-room-review.json; pipeline82195 is terminal1.

New capture/input gates require the actual rendered map and installed terrain
to match the live room. Walking additionally requires a committed in-room step
from a QA arrow, excluding an inherited warp stride. Earned rooms are held
before later dialogue/exit input. Row receipts are required only for an image
composite; normal native scenes still require modeled species, actual scenery
and GPU screenshot readback. Receipts now report the real authoritative hash
rather than presentation_snapshot's intentional zero placeholder, plus actual
source event flags and block overrides. The paired comparator already used
full integrity snapshots; that comparison was not replaced or weakened.
Map-gate build63531 passed4m01s. Final v3 pipeline95488/Python78363/Cargo78366 is
live, compiling the latest complete gate changes before a fresh session. Logs,
rejected images and pending v3 evidence stay under target/kanto-resume. Native
League acceptance remains pending; no five-win/ceremony completion is claimed.
Keep the full original16, all311/shared-furniture, party-variant, animation and
whole-room obligations open. No geometry, gameplay dispatcher, source flags,
packs or audio changed for this harness. The deleted
docs/handoff/kanto-interiors.md remains deleted; this checkpoint and
docs/modeled-interiors.md retain the current handoff.

Bruno lava-platform correction — 2026-10-18

Human clarified that the blue courses support the green platform over lava.
The previous four freestanding blue walls above the green floor were wrong.
Bruno now keeps the green approach and central platform at the entrance datum,
with the lava sixteen source pixels below it. The original blue courses face
outward below the green caps, and continuous blue edge supports close the
walkway sides. Native stairs, walking support and source collision are retained.
Will’s pool and bank heights are unchanged. Native render build passed in
8m38s. Three matched 2D/native pairs (production-default front, east and west)
and both moving samples were individually inspected offscreen. Blue faces now
sit under the green platform; both approach edges connect down to lava without
exposed gaps. All paired receipts preserve state. The 64-frame UD pass walks
(4,12) to (4,11) and back in Bruno’s room. All three focused external-pack
regressions passed on the final source (4.09s after compilation). Pack size/hash remain
582005378/a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Evidence: target/kanto-resume/bruno-support-headless-review and receipts/
bruno-support-*.log. This is scoped platform acceptance, not whole-room or
all311 acceptance; no new global asset audit was run.

Continuous native League status from the preceding work is incomplete: v4
actually earned Will/Koga/Bruno/Karen wins, then Lance failed native camera
admission (no readable higher shot on checked ground). No native five-win,
Hall ceremony or Continue completion is claimed. The HUD readability focused
regression passed, with native visual verification still pending. The old v3
live-build descriptions below are historical and superseded by this status.

Lance native camera diagnosis — 2026-10-18 (in progress)

Reproduced the saved native-pack Mew/Gyarados battle offscreen with ordinary
U18+A160 and opt-in encounter tracing. The renderer rejects all shots: 398
leave authentic retained bounds, 208 exceed checked source ground, 174 cannot
fit complete battlers inside authentic terrain, 14 are statically occluded and
6 have scenery dividing the composition. Only four stage pairs pass the floor
check (348 rejected) in each elevation tier. The resulting source-art move
menu is diagnostic fallback and is rejected as native acceptance.

A hypothesis that trainer ground radius three was starving this scene was
tested, built and disproved: Lance binds ScriptedGround, already radius five;
the stage counts and rejection reasons were identical. That radius/API change
and its unneeded tests were removed, with no camera admission gate relaxed.
The new opt-in checked-floor trace reports actual retained cells/elevations.
Build session69440/Cargo91813/rustc94369 writes receipts/lance-floor-trace-build.log;
last confirmed active at 13m50s of the Bevy compiler. Pipeline20619 waits on that
exact Cargo and then automatically runs the saved diagnostic into
lance-floor-trace-headless-review with receipts/lance-floor-trace-native.log.
It checks that the input save remains unchanged and rejects shader errors.
Revalidate these actual handles; do not start a competing diagnostic/build.
The shader source correction predates this tracing build and is embedded by
load_internal_asset; shader source edits cannot repair an already-built binary.
An initial producer-test build was explicitly stopped and superseded, then
stopped when the tested hypothesis was rejected; no test pass is claimed.

All36 images of native continuous League v4 have now been individually viewed;
individual-room-review.json records that review. Arrival player crop, old tiny
native HUD, obsolete Bruno freestanding walls, and Lance fallback prevent full
acceptance. Fresh Bruno support review supersedes only those blue banks. The
full original16/all311/shared furniture and continuous native Hall gates remain
open. The human-deleted docs/handoff/kanto-interiors.md remains absent.

Fresh continuous v5 (session39235/PID95495) was explicitly stopped after actual
voxel_surface.wesl shader parse errors. Its Will battle pair was individually
viewed: the bigger HUD is legible, but terrain is completely absent while scene
metadata misleadingly reports ready. The entire v5 visual run is rejected;
individual-room-review.json records that failure. The current shader already
has the parenthesized XOR expression; it belongs to other concurrent work and
was not changed here. V5 cannot prove HUD/room acceptance despite the clear
font improvement, and is not continuous native League completion.

Interior arrival framing and Lance stage separation — 2026-10-18 (verification pending)

Previous turn made progress: the complete v4 image set was reviewed and the
saved native failure reproduced; its final wait was on confirmed live Cargo.
Tracing build69440 finished successfully in28m43s. Pipeline20619 then completed
the diagnostic: source save unchanged, zero terrain shader errors, source-art
fallback still rejected. The new trace proves ScriptedGround radius5 with164
retained cells and multiple actual terrace/stair heights. The old radius
hypothesis was removed; it did not fix Lance.

Replaying those exact floor cells and full recorded body envelopes through
production layout/ground methods reproduces four supported stages and zero
geometrically admissible shots. Adding a 1.5-core-tile gap between the existing
1/2-core choices yields15 supported stages and13 shots satisfying complete-body
floor, authentic bounds and far-plane proof. Production now searches that gap;
it does not enlarge floor evidence, shrink bodies or relax camera checks.
The landing regression proves one tile overlaps, two crosses the real edge,
and 1.5 fits. Its isolated compilation passed. Full native static-occlusion
and GPU acceptance are still required. Existing staging tests are preserved.

The confirmed default-camera arrival crop is also addressed: interior orbit
cameras pan in their own up-plane enough to contain the modeled player's
actual foot and full stature with a visible margin. Existing yaw, pitch, FOV,
distance, body size and depth stay fixed. Outdoor clearance and first-person
logic are retained. The actual projection regression passed in isolation at
four yaw angles; full-crate and fresh native image checks remain pending.
Isolated source/test scratch binaries were removed after recording scoped
results in receipts/interior-camera-stage-isolated-results.json.

Live work to revalidate, not restart: camera Cargo test session11829 writes
receipts/interior-arrival-camera-test.log (voxel test compiler99862 was live
at9m59s). Production examples build session60837/Cargo1612 is queued behind it,
writing receipts/lance-half-tile-stage-build.log. Pipeline12123 waits on that
exact Cargo and then captures a native Lance move-menu pair WITHOUT permitting
source fallback, rejects shader errors, checks the original save hash, and
captures production-default Will(4,17), Lance(4,23), home(7,7) arrival pairs.
Outputs go to lance-half-tile-headless-review. Inspect each full image before
acceptance. The full continuous native League/Hall/Continue session is still
required after this isolated native gate passes. All311, shared furniture,
original16 and whole-room composition/material obligations remain open.

Full-scope artifact reconciliation — 2026-10-18

The prior goal turn was a verified wait on the still-live camera test compiler;
no timeout was treated as a failed build. The saved scope metadata audit lists
311 non-Town/Route rooms and225 declared alternatives across22 maps. The
original16 requirement remains the three declared states in each of the five
League rooms plus CeladonDeptStore6F’s sale callback, not all225 states.

A filesystem provenance inventory finds754 canonical-core-pack identity records
covering68 rooms with source/native pair files (46 Johto/22 Kanto);2451 identity
records name the browser pack. This is artifact provenance, not proof of visual
acceptance or a semantic difference between those packs.243 rooms lack a
canonical-native pair identity in this format and need reconciliation with
saved-session/continuous captures before calling them uncaptured. No old
browser-pack baseline is promoted to current-source visual acceptance.
The scoped records themselves leave whole-room acceptance false.

The inventory is target/kanto-resume/receipts/interior-native-artifact-coverage.json.
It explicitly does not certify current geometry, shader readiness, whole-room
composition, hidden areas, scripted states or reachability. After the pending
native camera checks, refresh the full311 native baseline and prioritize actual
defects across the full scope; camera/League work must not replace shared
furniture, Johto, unused rooms, original16 or actual service obligations.


Native arrival, move-board and broad review checkpoint — 2026-10-18

The camera Cargo regression and all19 camera tests passed. Optimized native
Will/Lance/home arrival pairs were individually inspected with preserved source
state and complete player framing. The actual Mew/Gyarados gap admission passes
with default stack and no source fallback; whole battle composition stays open.
The revised expanded native move board clears Gyarados, hides duplicate HP and
retains readable16px board/24px HUD text; its matched source image is unchanged.
Two actual-source layout tests pass. See the current checkpoint at the top of
3d-remaining-families.md for build limitations and exact receipts.

Runtime-reuse headless batch validation completed ten rooms, all individually
viewed with matching source state and no shader/logger errors. Findings remain
explicit in native-runtime-reuse-ten-room-headless-review/individual-room-review.json;
center crops do not prove whole-room acceptance. A fresh311-map native baseline
is running (session36009, native-full311-runtime-reuse.log).

Every native action/move command is now gated, exposing an actual Bruno/Onix
camera failure in v6 (exit101), after earned Will/Koga wins. Ordinary source
battle text explains Koga's Ariados-to-Forretress switch; no species mapping bug
is established. A new traced ordinary-input run (session20223,
native-continuous-league-v7-trace.log) will diagnose Onix's checked floor.
All311/shared furniture/original16 and continuous native Hall/Continue remain
open. The deleted handoff was not restored; no content-pack/audio changes.

The next reviewer revision compiles with verified unchanged-source provenance:
separate species-labeled command captures, rendered/presented species matching,
and authoritative enemy IDs in receipts. V8 is live (session18174,
native-continuous-league-v8-opponent.log); its runtime gate is not yet accepted.
Eight fresh Kanto center comparisons and three v6 battle pairs were individually
viewed, with narrow findings recorded beside their images. The full311 center
batch had77 pairs at that checkpoint; hidden areas/full-room states remain open.


Ecruteak hall and Onix admission — 2026-10-18 (verification pending)

Fresh baseline review found EcruteakTinTowerEntrance omitted from native timber
and guardian-backing guards. A separate architecture selector now covers its
existing source drawings without changing Tin Tower perimeter/warp rules.
Focused external-pack test61280 and optimized renderer/perimeter pipeline6376
are running/queued. The hall's actual four guardians and three ladders are
included in the regression. No finished visual acceptance is claimed.

V7/V8 terminated at Bruno's Onix. Actual trace/replay finds28 supported stages,
zero shots at old angles and ten geometric shots at yaw180/elevation75.
A final high-angle end-on search tier is implemented after the entire original
search, with all physical/admission checks retained. Recorded-floor test54731
and optimized native build/run16639 remain pending. The revised review also
labels opponent party slots, preserving distinct evidence for Will's two Xatu.
Earlier v8 Xatu/Jynx matched pairs and species gates were individually verified.

The center baseline stopped after186 comparisons at genuinely unlit
RockTunnelB1F; both dark images were inspected and rejected for room-art review.
Flash-enabled evidence remains required. Remaining124 centers resume in
session87576/PID24476 (native-full311-runtime-reuse-remaining.log). All311 stay in
scope; the baseline predates the newest source fixes. See the top checkpoint
in 3d-remaining-families.md for exact evidence, handles and remaining gates.


Refreshed311 baseline and verified native Onix — 2026-10-18

All311 selected center pairs are now present with preserved state:298 ordinary
plus13 separate disposable Flash fixtures. The thirteen Flash comparison images
were individually inspected. The coverage record explicitly retains full-room
acceptance false and identifies the frozen build; latest Ecruteak/Onix changes
are separate. Explicit --maps batches now forward the existing preview flag to
fresh shells, verified by the completed thirteen-room batch. Johto dark-cave
fill and generic teal water remain real defects; no repair is claimed yet.

Ecruteak's actual external-pack/nine-floor regression passed7.76s. The new Onix
regression passed0.06s in the same fresh test binary; redundant queued Cargo
was cancelled after that success. Optimized renderer build38m34s and all24
perimeter comparisons completed. Comparisons were individually viewed, with
full original source/native Sprout south frames additionally checked. Internal
wood joins/backing improve, but Ecruteak's left-hall east component edge remains
open; do not bridge its black separation or close real exits. Tin player reveal,
beast composition, Sprout foreground/reverse views and movement remain open.

Optimized native v9 earned all five League wins and natively admitted Onix.
The inspected complete Onix pair has preserved state, matching IDs, real floor/
lava/supports and no source fallback, but tiny Mew/steep framing prevent whole
composition acceptance. V9 stopped at the Hall save because its example used
.save.json; the filename is corrected to .crystalsave. V10 is live in
session32322/PID36458, native-continuous-league-v10-crystalsave.log, linked
against the exact completed optimized production graph. Hall/Continue is still
unproven. The current checkpoint at the top of 3d-remaining-families.md records
all receipts, source limitations and remaining full-scope obligations.

Tin Tower summit inspection — 2026-10-18

The requested Ho-Oh tower peak is TinTowerRoof. It is already modeled with
pitched roof volumes and curved ridge tiles; it is not an absent rooftop.
Headless production captures at (9,9) were inspected in full from the default,
east and reverse cameras, against the full default 2D source. Paired states
are preserved. Evidence: target/kanto-resume/ho-oh-tower-peak-headless-review/
individual-review.json. No windows were opened. This is inspection, not a new
model implementation or whole-roof acceptance: source-art end/ladder details
remain flat, the center walkway has a wood finish unlike the gold reference,
and the roof reads as a repeated broad sheet at distance. Roof refinement
remains open alongside the existing interiors work.


Tin Tower summit sky and village backdrop — 2026-10-18

Implemented a summit-only lower-atmosphere sky treatment and a separate native
distant Ecruteak vista in summit_vista.rs: pitched/timber houses, theater, gym,
Mart, Center, pond, ruined Burned Tower shell, gardens, broad-canopy trees and
continuous surrounding hills. It is an authored distant interpretation, not
another playable/source-exported map. It creates no terrain support, collisions
or actors. Geometry is built lazily only on entering the verified tower-atlas
TinTowerRoof; it stays anchored to source-map coordinates as the render grid
moves, hides on departure, and follows the actual clock. Night energy was
corrected after the first capture made the valley glow blue. Other-map sky
behavior retains the existing zero-valued summit parameter.

Actual-source direct Rust compilation succeeded for the full voxel library,
Bevy root and location example using the completed dependency graph; these
are opt-level=0 visual QA builds, not performance acceptance. Three new geometry/
anchor/lifecycle-clock tests and eight existing sky controls passed. Final
full original day/night native frames were inspected, with preserved paired
source states; the day source image is pixel-identical to the prior rooftop
source. Default/east views were also inspected before the final night-only tint
correction. Evidence: target/kanto-resume/summit-vista-verified/review.json.
The shared overview uses camera "0.75 1 25 0"; production default camera
behavior is unchanged. No windows opened. Roof-end flat details, walkway finish,
whole-roof art acceptance and the broader interior/furniture goals remain open.

Separately, the previously running native League v10 reached Hall and wrote
earned-champion.crystalsave, then failed its review assertion "continuous native
review missed WillsRoom battle". No completed Hall/credits/Continue claim is
made; that prior goal remains open.


Actual Ecruteak and all surrounding summit routes — 2026-10-18

Supersedes the authored village approximation below: it has been fully removed.
TinTowerRoof now displays the actual pack maps EcruteakCity, Route36, Route37,
Route38, Route39, Route42 and MahoganyTown, placed by the existing production
connection graph. Each full map keeps its own identity, blocks, source atlas,
current palette, animation families, connected terrace evidence, native models
and normal TILE_SIZE=32 scale. No second player, NPC session or dispatcher is
created. The summit attaches to the actual Tin Tower exterior model's footprint
center and measured roof height, including its terrace, rather than the western
entrance hall. Scroll and sub-tile camera movement retain the source anchor.

The ordinary repeated map apron is disabled only for distant finite meshes;
repeating it over adjacent maps previously covered their terrain and water.
Unmapped interstitial cells instead receive the actual Ecruteak/Johto border
metatile 5. A separate real city lawn sample lets the unchanged native grove
builder recognize its complete trees; that sample column is removed from the
visible ground. Shared native trees and mesh handles cover only source-border
plots outside all seven map rectangles. Flat green source-card strips from the
first border attempt were rejected and replaced. The background creates no
footing/collision data in the live world. Summit-only haze, lower sky and far
depth retain the actual clock and ordinary camera pose.

Full actual voxel, Bevy root and location-example compilation succeeded against
the completed dependency graph. These are opt-level=0 visual QA builds, not
optimized-load/performance acceptance. Seventeen voxel regressions pass (four
summit, two finite-map/apron/attachment, three separate entrance-perimeter and
eight sky controls). The external-pack host regression passes: exactly all five
routes plus the two towns, every actual map tile/atlas at native scale, complete
native border-tree ownership, stable extraction cache and unchanged authoritative
session checksum. Pack SHA256 remains
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.

Current visual evidence is in target/kanto-resume/actual-summit-native-border/;
full southwest day and south night native/source originals have been inspected,
with preserved paired state. All captures explicitly use the headless image target and mute;
no windows are opened. Whole-roof finishes, the broader all311/interior/furniture
review and earned Hall/credits/Continue acceptance remain separate open work.
The preceding native League v10 assertion failure is not resolved by this change.

Café dining height and native League completion check — 2026-10-18

Fresh full source/native originals for CeladonCafe, EarlsPokemonAcademy and
TeamRocketBaseB3F were inspected in interior-resume-current. The academy has
its compact source-blue chairs facing the desks; the Rocket facility has the
ceiling-height connected divider network. These center views do not prove all
facings, room edges, interactions or whole-room acceptance. The café still
showed its tables near the characters' shoulders: both authentic café drawings
were fitted to16-source-pixel height, twice the normal8-pixel dining datum.
Celadon and Olivine tables now use that8-pixel dining height. Table footprints,
source/palette bindings and seats stay intact; Celadon's closed serving basket
and contents retain their original heights relative to the new tabletop.
The complete source selectors and custom-profile ownership are preserved.

The full actual voxel/root/location sources compiled against the completed
native dependency graph (opt0 QA, not optimized performance acceptance).
All14 café regressions pass, including corrected geometry bounds, unchanged
footing/claims, complete closed basket, source edits/crops/custom ownership,
shared gray seats, display cabinets and connected counter controls. Final full
Celadon/Olivine source/native originals and the final aisle pair were inspected
in target/kanto-resume/cafe-dining-height-native. Every paired state is
preserved. The64-frame ordinary synthetic UD route reaches authoritative café
rows1 through5 at x6, keeping the current native map active with no pending
terrain or extra terrain builds. Logs are receipts/cafe-height-{build,tests}.log
and receipts/cafe-dining-height-{native,aisle}.log. This accepts the bounded
height correction, not café completion: the bin still visibly has a closed cap
and needs the human-requested open-can appearance; actor seating/interactions
and other full-room details remain open.

Native League v10 is terminal, not waiting: its history contains five earned
wins and species/party-slot battle pairs, Hall phases, credits and Continue,
but its final check still expected the removed generic room-battle label.
The final room validator now accepts the actual species-and-known-party-slot
capture format while retaining every room's arrival, walking and earned gates.
Three new regressions pass: distinct same-species slots, removal of each room/
field phase, and rejection of generic, unknown-slot and malformed battle labels.
This does not claim all opponents or animation are covered; those visual gates
remain separate. Full actual Bevy root/reviewer compilation passed.

A fresh native v11 run uses the explicit headless image target, synthetic keys,
muted audio and a fresh save/output directory. Its live session64395 was
revalidated after the café checks; history shows Will/Koga earned wins and entry into
Bruno's room with carried PP. Output is
native-continuous-league-v11-checkpoint-headless-review/continued-home-3d.league;
log is receipts/native-continuous-league-v11-checkpoint.log. Poll this same
handle and inspect current history before considering any restart. Hall/
credits/Continue final acceptance is still unproven until this run completes
and its images are inspected. No windows, pack/audio exports, staging or
commits. The deleted handoff remains absent; these checkpoints retain the full
Kanto120/all311/shared-furniture/original16 scope and its remaining requirements.

Open shared cans and supported Will platform — 2026-10-18

Fresh café/gym originals proved the shared canonical gym_bin still had a
light cap above its intended dark opening. Its original Blender authoring now
uses an uncapped tapered outer body, inward dark walls and a deeply recessed
floor. Only gym_bin.mesh.json was published from the temporary authoring kit;
all unrelated asset exports and the temporary blend were removed. Bounds remain
exactly min[-.475,.015,-.46], max[.475,.94,.46], and the same ink/metal_light/steel
roles retain each source drawing's palette. No new bin or duplicate catalog was
added. The shared geometry/Arc binding is preserved.

A new native-mesh regression casts five downward sight lines through the mouth
and requires the first visible surface to be the source-dark floor below one
quarter of the fitted height; the old light cap fails this condition. Thirty-six
distinct focused regressions pass (37 executions across trashcan, all19 dungeon
models, puzzle-source and café filters), including unchanged footprint/footing,
complete-art guards and shared native geometry. The focused Python asset
validator passes324 finite, correctly wound triangles with exact bounds. The
broader Python dungeon checker separately fails at the pre-existing gym_plaque
missing material key; this is recorded in open-bin-existing-dungeon-checker-
failure.log, not misreported as a successful full checker run. Actual full
voxel/root/location compilation passed; these are opt0 QA builds.

Full fresh CeladonCafe and VermilionGym native/source originals were inspected
in target/kanto-resume/open-bin-native. The café mouth is now visibly open and
dark; the existing procedural open gym bins retain their appearance. Paired
states are preserved. This does not accept the whole rooms: Vermilion's rear
wall/return continuity still needs source-topology work.

V11's actual Will arrival view exposed a second source interpretation error:
blue courses stood16 pixels above the green walkway. Will now shares Bruno's
underside construction: every authenticated bank is a closed32×16×16-source-
pixel supporting body below the unchanged green datum, without an upright
cutaway obstacle. Its water and original foam row are lowered16 pixels, and
side-course paint comes from the actual blue source. Bruno's construction is
preserved. Two external-pack regressions pass for all four supports in both
rooms: volumes/normals/blue faces, complete source/custom-owner guards, untouched
UV/animation bindings, opaque backing, exact footing and unchanged claims.

Fresh full default Will/Bruno native/source originals, Will's full east
native/source pair and the final walk pair were inspected in
will-supported-platform-native. All paired states are preserved. The64-frame
ordinary UD bridge route reaches authoritative Will rows8 through12 at x5;
every frame retains the correct active native map, no pending terrain and one
terrain build. Side views still expose rear perimeter/cap fragments, and water
finish/whole-room/actor approval remains open. Logs are receipts/will-platform-
{build,tests}.log and will-supported-platform-{native,east,walk}.log. These
bounded repairs do not certify all original16 alternatives or all311 rooms.

The same native League v11 session64395 is now terminal134. It earned all five
wins, captured Hall room/ceremony phases and wrote earned-champion.crystalsave,
then aborted on a Compute Task Pool stack overflow after credits, before
Continue completion. The macOS crash report identifies bincode/serde
RawPokemonStorage/PokemonStorage deserialization in save loading. The Bevy
worker builder leaves stack size unspecified, so the unoptimized QA root uses
Rust's default worker stack. No save-format/gameplay mutation or broad core
rewrite was made. Full originals viewed this turn: Will arrival, Bruno/Onix
command, Hall arrival, PokemonHold and PlayerFront pairs. Hall room is native;
the ceremony panels are existing faithful source UI in both rendering modes,
not evidence of new3D ceremony models. Onix's tiny Mew/overhead composition
remains unaccepted; all other v11 images are not implicitly approved.

A fresh v12 reviewer was linked from the current compiled source (including
open cans and corrected Will supports) and launched with RUST_MIN_STACK=16777216,
explicit headless image capture, muted audio and ordinary synthetic input.
It uses a fresh output/save directory and leaves v11's earned save intact.
Session29443 is confirmed live; output is
native-continuous-league-v12-worker-stack-headless-review/continued-home-3d.league;
log receipts/native-continuous-league-v12-worker-stack.log. Poll this exact
handle; the larger QA stack is still a hypothesis until Continue succeeds.
No window opened. Read-only coordination confirmed the Pokemon-model goal
active and outdoor goal idle; their source/rig/landscape work was preserved.
No pack/audio export, staging, restored deleted handoff or global completion.
Full Kanto120/all311/shared furniture/original16/services/reachability work
remains active in these checkpoints.

Vermilion stepped walls and completed continuous League QA — 2026-10-18

VermilionGym's actual20×36-source-cell plan has an inset leader alcove and a
wider main room. The former rectangular perimeter left its rear blue bar flat,
continued returns outside that alcove, and retained floating cap/floor fragments.
The new source-scoped vermilion_room.rs authenticates the complete three header
bands, source phases/art, finite bounds and both real exit warps. It closes the
alcove's north/east/west walls and joins them to the native threshold panels and
main-room returns at the32-source-pixel ceiling height. The central passage and
both southern doors stay open. Changed/cropped/foreign layouts and live custom
owners decline the inferred shell. Header ownership is46 nonblank source cells;
footing/collision/object positions and the original panels/guardians/cans remain
unchanged.

The perimeter samples its own rear blue field and light cap rather than copying
the internal partition's black upper panels. The two original illustrated wall
footprints no longer leave gray floor pads outside the room. Their required
inside backing continues the actual four checker-floor source phases; removed
source indices and backing remain independent of game support. All11 focused
regressions pass, including the three new stepped-outline/source/custom-owner/
external-pack/translated-origin tests, palette provenance and existing panel,
guardian and puzzle-bin controls. Actual full voxel/root/location compilation
passed against the completed dependency graph (opt0 QA, not optimized runtime
performance acceptance). Logs: receipts/gym-outline-{build,tests}.log.

Final full native originals were inspected in
 target/kanto-resume/vermilion-blue-wall-clipped/{default,east,west}/.
Their2D source PNGs are byte-identical; the full matched source was inspected.
The final passage native/source pair was also inspected. All paired states are
preserved. The64-frame ordinary UD route from(5,6) traverses authoritative
rows3..6 through the central opening, with three grid origins, one terrain build
and the correct active native map throughout. An earlier probe from(5,8) was
correctly blocked between two cans and failed the movement gate; it was not
counted as a pass or used to weaken that gate. The ordinary south-door control
from(5,16) reaches VermilionCity(10,19); its full final source/native pair was
inspected in vermilion-blue-wall-final/exit-*.png. That exit control precedes
only the final floor-backing cleanup. Room/actor/furniture/service and full
variant acceptance remain separate open requirements.

The same native League v12 process/session29443 is now terminal0, not live.
It finished in1061s using RUST_MIN_STACK=16777216 for the opt0 worker stack.
There are1039 ordinary key decisions, five earned wins,57 preserved paired
checkpoints, a22,647-byte earned-champion.crystalsave, Hall count1, credits,
Continue summary and an authoritative NewBarkTown return at(13,6). The completion
path also requires the pending champion spawn to clear and the game timer to
resume. Earned PP progresses [24,9,16,32] → [24,9,11,32] → [18,9,11,32] →
[16,8,9,32] → [15,4,8,32], with Full Restore stock5→4; no victories are seeded.
Hall's normal healing then restores the party. This is the explicit fresh QA
fixture's legal initial MEW100 party, not a claim of a main-story playthrough.
The larger QA stack resolves the concrete save-deserialization overflow seen
in v11; no production save schema or gameplay dispatcher was rewritten.

All25 paired battle receipts report native figures, exact presented/modeled
species equality, matching room scenery and no source-art/scenery fallback.
The final New Bark, credits-end and Continue-summary full source/native pairs
were individually inspected; the final pair preserves state. The remaining
checkpoint images are not implicitly visually approved. Koga slot3 has no
stable paired-command capture, and native battle composition/animation/all-party
coverage remains open. The ceremony panels retain the existing faithful source
UI in both rendering modes; this is not new3D ceremony-model evidence. Two
nonfatal stale-entity warnings occurred during Continue; the run still exited
successfully with the final captures. No optimized performance claim is made.
Evidence: native-continuous-league-v12-worker-stack-headless-review/review.json,
continued-home-3d.league/{history,completed}.json and
receipts/native-continuous-league-v12-worker-stack.log. The frozen v12 build
includes the earlier open cans/Will supports; Vermilion's later repair does not
affect that League route. Do not restart a completed run merely to re-observe it.

All work stayed headless and muted. The canonical native pack hash remains
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Unrelated source/rig/landscape work was preserved; no pack/audio export, staging,
commit or restored deleted handoff. Full Kanto120/all311/shared-furniture,
original16 source alternatives, services/reachability and individual whole-room
visual acceptance remain active. The completed continuous controller gate does
not replace those requirements.

All connected summit routes — 2026-10-19

Expanded TinTowerRoof's real-map panorama from a two-connection neighborhood
(seven maps/five routes) to the complete connected outdoor component: 32 maps,
20 routes (26, 27 and 29–46), eleven towns/landmarks and BattleTowerOutside.
The existing authored connection offsets place every map at its actual scale;
no replacement city layout, gameplay connection, actor session or collision
surface is introduced. A breadth-first traversal admits each distant map once
and terminates reciprocal/cyclic connections. Nearby scenery builds first.
Ordinary field rendering retains its two-hop scope and overlap precedence.

Every panorama map keeps the same local two-hop terrace evidence used during
native play. Extending the panorama no longer runs a whole-region terrace solve
for every route. Actual finite map rectangles also exclude the shared border
forest, so additional roads, towns, coasts and water replace the earlier filler
where their source maps belong. Native TILE_SIZE=32 and actual tower attachment
are preserved.

Seven distinct topology/source/session regressions pass in the final eight test
executions: the real pack contains exactly all 32 expected maps; every authored
connection meets at its actual offset; each map retains every source tile and
atlas; native border admission, idle cache and authoritative checksum are
preserved; synthetic cycles terminate; and the five existing bounded-neighbor,
landmark/gate and runtime-write identity controls remain green.

Evidence: target/kanto-resume/all-surrounding-routes-native/review.json. All four
full native/source originals were inspected: southwest daylight at camera
(0,3,45,0), and opposite northeast night at (0,7,20,0), with the same roof player
position (9,9) within each pair. Both paired-state receipts preserve the actual
session. Both runs publish all 32 actual native maps and the source border
before capture. Every capture explicitly uses HEADLESS=1, CAPTURE_TARGET=image
and mute; no windows are opened. Pack SHA256 remains
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.

The initial opt-level=0 capture hit its 300-second limit before completing the
last maps; its failure receipt is retained. Actual host/voxel opt-level=1 QA
builds complete both paired captures (about 142–151 seconds before shutdown),
but loading/memory cost and optimized production performance remain open.
This establishes complete connected source coverage and two reviewed summit
views, not individual art acceptance for every route, every roof finish, or the
broader interiors/furniture work. No completed League QA is restarted.

Flash cave pigment, complete pools and bank closure — 2026-10-19

Finished the renderer-only cave work that was in progress after the surrounding
summit routes. Published Flash uses the source Night palette; lit Johto caves no
longer receive a second darkness multiplier. Genuine Dark state retains the
existing concealment. Cave water now reads its own live opaque blue source
swatches, including Johto cave families. Missing, black or transparent paint
clears stale colors on primary and cloned materials without advancing phases.
The HDR world camera uses Linear transfer in native caves so PBR Neutral does
not subtract the weak green channel from dark mineral pigment. HDR, bloom,
dithering and color grading remain; exiting, F3, inactive frames and missing
resources restore the exact previous camera curve. UI cameras stay independent.

The mineral foundation now excludes authenticated water and wet shore cells
from the original source, including cells covered by native models. Real source
water sheets extend beneath those owners; ladder shafts remain cuts. Shore
triangles borrowing pool UVs keep the pool level while their rock rim continues
to follow dry land. New source-colored mineral faces close the drop below raised
banks without repainting already-finished caps or changing upper sculpture,
collision, support heights or authored movement. The rejected shader UV
experiment was removed. A stale cutaway test expectation was corrected from
Z=0 to the existing actor-derived Z=-8 after reproducing the same failure in the
older binary; production actor positioning was not changed.

Actual current voxel library, host and location example compiled successfully
(opt-level=1 QA). All84 focused regressions pass: Flash/source state,17 water
checks,10 quality checks,36 terrace checks, source mineral finish, foundation,
shore, cutaway, hatches and the old outdoor/palette controls. Final receipts are
receipts/flash-cave-palette-linear-tests-{0..15}.log. Earlier failed hypothesis
and stale-filter receipts remain diagnostic, not passing evidence.

All30 full native/source originals were individually inspected in
 target/kanto-resume/flash-cave-palette-native/verified-linear/:
Dark Cave's two entrances, Rock Tunnel's two floors, all eight Whirl Island
rooms, SilverCaveRoom1, UnionCave1F and TohjoFalls. These are13 Flash-dark rooms
and two normal cave controls, high quality at hour12 with the explicit Flash
preview flag. All15 paired states are preserved; no tone/dither warnings remain.
Blackthorn's full lake now shows source-blue water instead of foundation or
black gaps, with rock closing the bank bases. review.json records each exact
position and the inspected originals. The ordinary UD controller walk from
DarkCaveVioletEntrance(20,18) covers64 frames and authoritative rows17..18,
keeps the correct active terrain and returns to(20,18). Its final source/native
pair was inspected and preserves state. Every capture is explicitly headless,
image-targeted and muted; no windows were opened. Pack/audio/game data remain
unchanged; SHA256 is
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.

This verifies pigment, pool visibility and bank backing at the captured views.
The small rim-like formations in Whirl Island B1F/SE were initially flagged
as hollow columns. A subsequent direct native-pack check confirms block31
explicitly surrounds four lower-floor tile16 cells with cap/face art; filling
those centers would erase authored topology. Preserve those openings. Broader
rock finish, opposite sides, every approach, Surf, battle backdrop contrast and
the wider311-room/furniture/service review are not certified here. The connected32-map summit coverage above remains
complete; its loading/performance limit remains open. Completed League QA was
not restarted. The deleted handoff document stays deleted.

Shared CRT screen/bezel correction and source-rim interpretation — 2026-10-19

The preceding turn completed the Flash/pool repair and verified the connected
summit routes; that was concrete progress. AGENTS.md was reread. The deleted
handoff remains absent; these two canonical guides retain the complete scope.
A direct bounded native-pack probe confirms Whirl Island's small block31
formations explicitly surround lower tile16 floor with cap/face art. The earlier
hollow-column diagnosis was incorrect for those rings; their centers must stay
open. The prior checkpoint and ignored visual receipt now state that evidence.
No cave geometry or support changed in this turn.

Fresh full native/source centers were individually inspected for BluesHouse,
CeladonMansion2F, CeladonDeptStore4F and CeruleanPokecenter1F. Blue's TV still
showed a pale CRT inside a blue bezel despite the source blue glass. The selected
canonical television mesh uses `metal` for its curved inset and `metalMedium`
for its narrow surround. Their existing source-swatch roles were reversed.
interior_models.rs now assigns the glass to the existing live screen pigment
and the surround to the existing pale pigment. Cabinet, body and control roles,
selected closed geometry, dimensions, source matching, floor and support remain
unchanged. No model assets, palettes or packs were regenerated.

A new regression traces nine sight lines through the actual curved front and
one through its narrow bezel, rather than merely requiring every color to occur
somewhere. All54 interior-model tests and the separate source-placement test
pass (55 unique checks; the four-TV selection overlaps three model checks).
Existing controls preserve exact selected positions/normals/indices/cache,
screen-only geometry, dimensions, complete source guards and unknown-contract
rejection. Actual current voxel/host/location compilation succeeds at opt-level1
against completed dependencies. The first helper attempts lacked a build-script
dependency or a private library search path; those tool setup errors were fixed
and are retained in receipts, not counted as passing builds.

All24 full source/native originals from12 preserved pairs were individually
inspected in target/kanto-resume/tv-pigment-native/: six centers (Blue, both Red
floors, both player floors and GuideGentsHouse), four approach/rotated views,
one closer Blue view, and the final ordinary RL walk. Blue's reference PNG is
byte-identical to the pre-fix center. The initial six-name batch completed five
valid maps before rejecting the misspelled guide map; GuideGentsHouse then
completed separately with the correct batch arguments. That initial batch error
is retained and is not represented as a successful six-map process. The walk
traverses(3,2)->(4,2)->(3,2),64 frames, with active settled BluesHouse terrain,
no extra terrain build and a preserved final pair. High-quality captures all
use explicit headless image targets and mute, without windows or focus changes.

Two fresh audits use the same current example source against the before/after
actual voxel/host libraries. All311 map records and all225 declared variants
are exactly equal, with no mesher errors: geometry, source coverage, ownership,
support and every reported field are retained. The material correction is
established by the ray test and native images, not inferred from those counts.
Receipts use tv-pigment prefixes; tv-pigment-native/review.json records exact
poses and individually inspected images. Pack SHA256 remains
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Task scratch helpers were removed; ignored build caches and evidence remain.

This accepts the bounded screen/bezel correction. TV highlight detail, broader
appliance finish, the reviewed mansion's stair/door approaches, department-store
counter/edge stock beyond the center crop, clinic services and full camera/room
coverage remain open. Center views and unchanged audit counts do not certify
those requirements. All311/original16/shared furniture, reachable states and
broader scene/animation fidelity remain active; completed League QA was not
restarted. No staging or commit occurred.

Clinic service coverage, PC snapshot ownership and remaining Indigo stock — 2026-10-19

The preceding TV turn made verified progress. The handoff remains deleted;
these maintained guides retain the full all311/original16/shared-furniture goal.
Current source adds tests/clinic_services.rs and broadens the existing nurse
regression from two Kanto rooms to all22 authored clinic lobbies (11 Kanto,
11 Johto). Exact discovered names are pinned so missing rooms cannot silently
reduce coverage. Initial locations, CHRIS identity and parties/HP/PP are explicit
disposable fixtures, not earned arrivals or a main-story playthrough.

Nurse input now proves visible dialogue rejects movement, HP1 and PP1 restore
to source maxima, Start returns and ordinary walking resumes in each lobby.
The public-PC case authenticates each actual block08 approach, boots through
PCScript, verifies all five Bill choices, deposits Cyndaquil into real storage,
withdraws it, preserves both original Pokémon exactly, closes the PC and walks.
The first broad run exposed a real controller presentation bug: snapshot()
returned the suspended core chooser instead of the menu accepting input.
VisibleShellController now projects the current boot/notice page above it and
publishes the controller-owned Bill/menu/list surface when that printer closes.
No second dispatcher, game-state mutation or storage implementation was added.

All seven named controller regressions pass in eight final executions, including
the PC case in the default renderer-neutral build as well as the native build.
These exercise44 distinct nurse/PC room cases, plus22 repeated default-PC cases.
Existing six-ball/four-palette nurse publication, three owned-bed variants,
Mom/New Bark gate, actual battle-turn/PP and integrated PC controls remain green.
Three actual-pack clinic geometry controls also pass (stairs/support, complete
terminal and rear backing). Final logs are receipts/clinic-final-verified-{0..7}
and clinic-voxel-controls.log. Early failures are retained: Violet uses its
actual VioletPokecenterNurse label; returning from the party list selects
WITHDRAW rather than retaining DEPOSIT; named fixtures use the existing smoke
setup in both feature sets. The first default helper selected a stale dependency
graph; the complete current graph then compiled both feature configurations.

Actual current host and native wrappers compile at opt-level1 against the
completed dependencies; this is QA, not release-performance certification.
The read-only native review hold now observes actual pc:hub/pc:bill-actions
owners and their production rows, excluding boot text and transfer notices.
It freezes an already reached menu after ordinary A input without setting a
cursor or selecting a service. Its rows agree with the controller snapshot in
the external-pack PC case. Default-feature code does not reference this QA hook.

All44 full originals from22 selected paired captures were individually inspected:
all11 Kanto centers, seven Indigo perimeter positions, Pewter's actual six-ball
nurse sequence, named PC boot, chooser and Bill actions. All pairs preserve
state. Location captures use noon; saved PC captures retain their saved clock
within each pair. Explicit HEADLESS=1/CAPTURE_TARGET=image/mute/high quality are
used throughout, without windows or focus changes. review.json under
 target/kanto-resume/clinic-current-native records exact poses and limitations.
The final boot and chooser commands pass. Bill's final native/source PNGs show
all five legible rows and preserve state, but the generic screenshot verifier
rejects the faithful three-color source menu (exit1, luma range255). That is
recorded as a failed command, not a green gate; the verifier was not weakened.
Its receipt is receipts/clinic-final-native-pc-bill-actions.log.

The larger views expose real unfinished furniture: Indigo's source block2c at
block(7,3), source-cell plot(28,12) size4x4, contains two stock shelves whose art
is still laid across a shallow block. All16 cells are unmodeled in the latest
full audit. Build complete upright, source-owned cabinets there next; do not
call Indigo finished from its working nurse/PC or continuous main-room walls.
Actor fidelity, full camera/room and upstairs coverage, PC rating/box/mail/item/
release branches and the wider311/original16/furniture requirements remain open.
No geometry, support, pack, audio or gameplay dispatch changed in this turn;
no fresh CLI/stdio/playability or global art acceptance is claimed. Pack SHA256
remains a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Task scratch helpers were removed; ignored caches/receipts remain. No staging,
commit, pack export or restart of completed League QA occurred.

Indigo stock cabinets and continuous backing — 2026-10-19

The preceding route turn rechecked existing coverage; this turn returns to the
remaining interior work and makes a verified furniture repair. AGENTS.md was
read, and the deleted handoff remains absent. These two maintained guides retain
the full Kanto/shared-furniture, all311, original16 and reachable-state scope.

IndigoPlateauPokecenter1F block2c at block(7,3), source plot(28,12) size4x4,
now produces two upright stock cabinets. Each uses its own upper source course
on the roof and its three lower courses on the recessed vertical shelves, with
closed sides, back and underside. Nominal source width16, face height24 and
depth8 preserve the drawing's proportions. The complete original16-cell block,
including tileset, metatile, local phase and every tile index, authenticates both
units together. A custom claim anywhere prevents the whole replacement; an
actual custom-profile resolver test also verifies ownership. Ground remains
source-derived and original footing heights remain untouched.

The first native pair exposed a missing rear wall beyond the old stock cards.
The repair continues the verified adjacent plain wall across the two cabinets,
using the same source row12/course ending at row14. It stops at the actual stair
corridor. The higher stair-alcove wall cannot donate the backing position. This
is scoped to this source-complete stock family; other equipment keeps its
existing wall behavior. No new model export, game dispatch, collision, pack or
audio change was introduced.

Thirteen distinct focused regressions pass on the final source: two new source/
custom-owner/proportion/support tests, one external-pack cabinet/backing seam
case, six existing clinic controls, three external clinic terminal/stair/rear
wall controls and the existing native retail finish control. Actual voxel and
host libraries and native wrappers compile at opt-level1 against the completed
dependency graph; tests compile at opt-level0. This is local QA rather than a
release-performance claim. Logs: target/kanto-resume/receipts/indigo-stock-*
(final-tests, clinic-controls-final, external-clinic-final and retail-control).

The final full audit uses exactly the existing311-map inventory and225 scripted
block variants. No mesher errors occur. Indigo is the only changed map report:
modeled cells148->164, exactly16 retail-shelf cells at(28,12)-(31,15), with no
remaining block2c drawings. All225 variant reports are exactly unchanged, as
are every other map, actor-source and battle-species report. Evidence:
receipts/indigo-stock-full311-final.json and indigo-stock-invariants.json under
 target/kanto-resume. An initial intermediate audit included all388 maps and is
retained under indigo-stock-intermediate-all-maps.json; it is not the311 gate.

Five final matched pairs were captured without windows or focus changes: noon
east(16,7), southeast(15,11), rear(15,2) at opposite orbit, west(12,7) at side
orbit, and night(15,11) after ordinary RL input. All ten full originals were
individually inspected. Cabinet fronts, finished side/top, rear wall continuity
and corridor clearance hold in these views; the rear wall correctly occludes
furniture from behind. All commands exit0 and every paired-state receipt says
state_preserved=true. The night walk records64 frames through actual tiles
(13..17,11), one terrain origin and no additional terrain builds. Initial visual
positions are disposable fixtures, not earned arrivals. Explicit HEADLESS=1,
CAPTURE_TARGET=image, mute and high quality are used. The first final-capture
batch used an invalid comma-separated camera request and failed before capture;
the corrected whitespace-separated batch passes. Receipt details and source
hashes: target/kanto-resume/indigo-stock-native/review.json.

This closes the exposed stock-family defect, not whole-room or whole-game art
acceptance. Upstairs/link rooms, full room/actor review and PC rating/box/mail/
item/release branches still require their broader checks. The earlier faithful
three-color Bill-menu screenshot-verifier rejection remains recorded and open;
its threshold has not been weakened. Full all311/original16/shared-furniture and
reachable-service requirements remain active. No completed League run was
restarted. Pack SHA256 remains
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Task scratch build helper was removed; ignored QA binaries/receipts remain.
No staging, commit or unrelated model/outdoor changes occurred in this repair.

Clinic CHANGE BOX persistence, native ownership and source paging — 2026-10-19

The Indigo stock turn made verified progress. This continuation read AGENTS.md
and the maintained scope, confirmed that the deleted handoff remains absent,
and broadened real PC services instead of certifying the wider goal from
furniture coverage. The full all311/original16/shared-furniture and reachable
service requirements remain active in these maintained guides.

Real input exposed several defects that manually seeded menu tests missed.
CHANGE BOX could refuse its file write because PCScript retained a core text
window, discard that refusal, switch boxes and play the saved sound anyway.
The Bill save policy now distinguishes that suspended window from a live boot/
notice printer; a refused write propagates failure. Native automatic script
advancement now recognizes both the box list and its action menu as player
boundaries, preventing a256-step confirmation loop. Native save dialogs take
priority over the suspended source chooser, matching controller presentation.

Source ChangeBoxSaveText has a CONT acknowledgement before YesNoBox. Its first
two lines now use the existing field printer; the final scrolled pair includes
"will be saved. OK?" before YES/NO appears. No alternate input dispatcher or
replacement text catalog was added. Returning from the full-LCD box list also
requires one complete field compositor pass: the text-only fast path must not
retain the old modal canvas/frame. The final native introduction shows the
actual room and both source lines without an early YES/NO box.

Reload verification found that the earlier file retained an invisible core PC
menu that blocked Start. Bill's actual PC saves now close only transient menu,
window and textbox surfaces on a cloned persistence shell through existing
runtime mutation APIs. The live PC stays open. Party, boxes, map, variables and
progression are preserved. Save-before-switch ordering matches the pinned
external source3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6 in pret-source.lock.json:
ChangeBoxSaveGame calls SaveBox before assigning wCurBox. The live box changes
0->1; loading correctly restores saved box0, with both original Pokémon.
Start and ordinary walking then work on the loaded controller.

The shared source-PC fixture authenticates block08 in each of all22 authored
lobbies, names CHRIS and supplies two explicit initial Pokémon. Service actions
then use ordinary VisibleShellController input. In every lobby, the new case
reaches both box menus, checks native auto-advance waits without mutating state,
prints/acknowledges the source introduction, declines without writing, accepts
a real save, loads it, opens/closes Start and walks. It also cancels RELEASE
without mutation, accepts RELEASE, verifies exact party compaction, refuses the
last remaining Pokémon with the production wrong sound, closes the PC and
walks. Last-party refusal text is a timed transient drained by controller
settling; no current-page text assertion is substituted for that sequence.

Four service regressions pass in both native and default renderer-neutral
builds: the existing22-lobby deposit/withdraw flow, new22-lobby box/save/reload/
release flow, live-boot/pending-map-work save protection and a real write-error
case that keeps box0, preserves the obstructing file and queues no saved sound.
Four existing box controls, save-policy, Mom/New Bark gate, actual battle turn/
PP and Elm dialogue/world-retention controls also pass. Total12 distinct tests,
16 executions. The Elm test's old ../../../apps input location was corrected
to this repository and the explicit CRYSTAL_RENDER_TEST_PACK; its first missing-
file failure is retained. Final records: target/kanto-resume/receipts/
clinic-branches-final-verification.json and clinic-branches-verified-*.log.
Actual host/wrapper builds use opt-level1 against completed dependencies; test
builds use opt-level0. These are local QA builds, not release certification.

All eight full originals from four final matched pairs were inspected:
introduction, scrolled question, saved confirmation, and a64-frame DU walk
loaded from that native-written file. All four commands exit0 and preserve
paired state. The reload walk visits(9,2),(9,3),(9,2), with no extra terrain
build. Saved confirmation corresponds to an actual file; introduction/question
holds create none. Captures explicitly use HEADLESS=1, CAPTURE_TARGET=image,
mute and high quality, without windows or focus changes. The read-only save
review hooks freeze already reached production surfaces; they do not seed
cursors or save results. Evidence and exact source hashes:
 target/kanto-resume/clinic-branches-native/review.json.

Earlier failures remain evidence, not accepted gates: a too-short replay route,
the native256-step loop, a hidden prompt, a clipped question, a stale fullscreen
intro and the invisible owner after reload. The deliberately obstructed native
write-error capture was rejected by the runtime-error gate in the earlier
pre-pagination build; no accepted write-error image is claimed. Current API
error behavior passes in both builds. The earlier faithful three-color Bill
menu screenshot-verifier rejection also remains open; no verifier was weakened.

Next verify the actual MOVE W/O MAIL save/move path and its separate CONT text;
this source paging change is specific to CHANGE BOX. Mailbox/items/Oak rating,
box naming/printing through real input, timed release/refusal native frames,
upstairs/link rooms and whole-room/actor review remain broader obligations.
No new voxel/model/collision edit or full311/225 audit occurred in this turn;
the preceding Indigo audit is not promoted to global visual acceptance. No
completed League run was restarted. Pack/audio are unchanged; SHA256 remains
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Task build helpers and failed-test temporary saves were removed; ignored QA
binaries, fixtures and receipts remain. No staging or commit occurred.

Clinic MOVE W/O MAIL, authored Mail pages and hidden-cursor ownership — 2026-10-19

The preceding CHANGE BOX continuation made verified progress. AGENTS.md and the
current implementation were inspected; the deleted handoff remains absent.
The maintained guides retain the complete Kanto/shared-furniture, all311,
original16 and reachable-service goal. This turn broadens the actual MOVE
service and its source text rather than treating the prior PC checks as whole-
room or whole-service acceptance.

Before images and a real-input regression exposed MOVE's clipped save prompt:
YES/NO appeared beside its first two lines while "will be saved. OK?" was hidden.
MOVE now shares the source CONT/scrolling helper with CHANGE BOX. Its initial
page prints through the existing field printer; acknowledgement then exposes
the final two source lines and YES/NO. No replacement dispatcher or text catalog
was added. The source comparison uses the external pinned revision
3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6, matching pret-source.lock.json.

The Mail gate also discarded its second paragraph after A. It now reads the
actual _PCMonHoldingMailText pages from the verified pack, preserving both
"holding MAIL." and the subsequent removal instruction. The held-item check
follows IsAnyMonHoldingMail's party-item scan instead of scanning unrelated
stores or detached message payloads. Initial Mail fixtures use either actual
party slot, with complete Mail data; no impossible boxed-Mail fixture is used.

The new text revealed another input defect: directions changed Bill's hidden
cursor while the Mail page owned the screen. The guard belongs in the shared
primary direction helpers, used by both controller and keyboard paths. A first
attempt in the native dispatcher alone did not protect controller input; that
failed test is retained and the incomplete guard was removed. All four
directions are now checked on both Mail pages without moving the hidden choice.
Hall-of-Fame browsing remains outside this field-printer owner.

The source-PC MOVE case runs in every one of the22 authored lobbies through
ordinary VisibleShellController input after explicit initial trainer/party/room
fixtures. It checks source paging, decline without a file or storage mutation,
entry save, pickup cancellation without rewriting the save, party->box0,
box0->box1, box1->party, exact original Pokémon preservation and actual storage
contents in every per-insertion save. Browsing does not change the authoritative
current-box selector. The final file loads, exposes Start, and PC closure
releases walking. Mail cases cover either party slot in all22 lobbies:44 cases
per feature build, preserving storage and writing no save. Thus the two new
regressions add66 positive cases per feature configuration.

All six clinic service regressions pass in native and renderer-neutral default
builds. Seven existing MOVE controls, four box controls, save-policy, Mom/New
Bark gate, real battle turn/PP and Elm dialog/world retention also pass. Total:
21 distinct regressions,27 executions, with no zero-test result counted. Logs
and process outcomes: target/kanto-resume/receipts/clinic-move-final-verification.json
and clinic-move-verified-*.log. Actual host/wrapper builds use opt-level1 against
completed dependencies; tests use opt-level0. This is local QA, not a release-
performance, browser, CLI/stdio or full playable-game certification.

All fourteen full originals from seven final paired captures were inspected:
MOVE introduction, final question, saved confirmation, both Mail pages, carried-
Pokémon selector and a64-frame DU walk loaded from the real party->box autosave.
All seven commands exit0 and preserve paired state. Introduction/question/Mail
holds create no files; saved confirmation corresponds to a real write. The
carried selector uses the original source LCD intentionally; its800x720 LCD
pixels are exactly equal in both modes. It is UI evidence, not3D scenery/model
acceptance. The only inactive-world admission added is the named pc:move-carry
review, requiring an actual carried source, fullscreen storage owner, settled
ordinary input and frozen state. Ordinary model/scenery and screenshot-color
gates are unchanged. A real per-move autosave supplies its boxed Pokémon;
neither boxed data nor a menu cursor is seeded for rendering. Initial fixtures
are not earned arrivals or a main-story run. Captures explicitly use HEADLESS=1,
CAPTURE_TARGET=image, mute/high quality, with no windows or focus changes.
Evidence and exact source hashes: target/kanto-resume/clinic-move-native/review.json.

Next audit per-insertion write-error recovery; entry-save error tests do not
cover failure after an insertion has begun. The20/24-frame insertion-save
animation still needs native temporal coverage. Mailbox/items/Oak rating,
real-input box naming/printing, wider party/species variants, upstairs/link
rooms, whole-room/actor fidelity and the full all311/original16 obligations
remain open. Earlier expected-error capture rejection and faithful three-color
Bill-menu verifier rejection remain recorded; neither is relabeled as passed.
No voxel/model/collision edit or fresh311/225 audit occurred. The completed
League run was not restarted. Pack/audio remain unchanged; SHA256 is
 a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Task helpers and failed-test temporary directories were removed; ignored QA
binaries, fixtures and receipts remain. No staging or commit occurred.

Clinic MOVE insertion write failure: rollback and deliberate retry — 2026-10-19

Revalidated the current implementation after the route-only coverage check and
continued the pending service audit. AGENTS.md was read; the deleted handoff
remains absent. No existing room/model work was reverted.

An ordinary-input Pewter PC regression reproduced a distinct insertion failure:
MOVE's entry save succeeded, then a filesystem obstruction made the insertion
save fail after party storage had already changed. BeforeMove remained armed.
The retained before-storage.log proves the storage mismatch, rather than merely
inferring the defect from code. The obstruction changes only the filesystem;
all PC entry, pickup, insertion, acknowledgement and retry use
VisibleShellController::press/wait_frames after the initial room/party fixture.

The insertion now retains the pre-move runtime shell until saving succeeds.
On write failure it restores that shell, clears the timed commit, retains the
original pickup and its cancellation scroll, leaves the destination cursor in
place, and presents SAVE FAILED / Try again through the existing PC notice
owner. No success sound is queued. Sixty-four further controller frames leave
storage unchanged. Restoring the filesystem, acknowledging the notice and
explicitly retrying moves exactly the original Pokemon; the resulting file
loads with exactly that storage. The prior entry file and obstruction bytes
remain unchanged during failure. This is application-level recovery for the
verified write failure, not a new filesystem crash-durability guarantee.

The actual reached notice was rendered offscreen through the production LCD
sprite compositor and inspected. Its first wording wrapped into three lines;
the final wording fits two. This CPU UI evidence does not certify native room
geometry, GPU composition, or the20/24-frame animation. Evidence, exact source
hashes and logs: target/kanto-resume/pc-move-recovery-build/review.json.

The seven clinic regressions pass with native/location features and again with
renderer-neutral default features; existing positive cases still cover all22
lobbies. Seven MOVE controls pass, and the final recovery/render case is checked
separately in both builds. These are actual-source QA compilations against the
completed dependency cache, not production release/Cargo-build certification.
The pack remains a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.

Remaining: native temporal save evidence, mailbox/items/Oak/naming/printing,
wider party variants, upstairs/link rooms, whole-room/actor and all311/original16
visual acceptance. Return to the broader room/furniture review after this
bounded service fix; the whole objective is still open. No geometry, audio or
pack was edited, no League run restarted, and no staging/commit occurred.

Cable Club rear-wall joins and shared gold-seat/blue-counter finishes — 2026-10-19

The preceding insertion-save turn made verified progress. This continuation
returns to room/furniture work, reads AGENTS.md and current implementation,
and keeps the deleted handoff absent. The full all311/original16/shared-furniture
and reachable-service requirements remain active.

An offscreen native/source comparison of PewterPokecenter2FBeta exposed the
rear-wall break behind both signs and their adjoining doors. Each sign now has
24px clinical backing, with its original frame and live inscription raised 4px
and moved 2.25px forward so neither embeds in the wall. Each 18px doorway frame
gets a header through 24px, with the lower aperture clear. Complete existing
sign/door/mobile-corner source identities and actual map anchors guard the
backing. The mobile entries at source x12..14 and20..22, rows0..2, remain open;
console negative corners and all existing source ownership are preserved.

The main Pokecenter2F view exposed two related missing bindings: blue generic
seats instead of source gold, and peach counter fronts instead of source blue.
The main room now shares the existing native source-sampled seat and counter
finishes with the beta rooms. Counter coverage uses its actual blocks05/38;
lobby-only06/07 fixtures retain their original classifier scope. No alternate
furniture model or replacement palette catalog was introduced.

Twenty Cable Club regressions, the broadened main/beta seat control and the
broadened main/Kanto counter control pass: 22 distinct focused checks. New tests
cover backing height, all five source identity fields, map/anchor rejection,
live sign clearance, header height, open portal/mobile entry, unchanged source
claims and unchanged walking support. The source-only fixture initially tried
a lobby-only block06 in the main room; its failed log is retained and the final
main fixture follows that room's actual block plan. Actual Rust source and a
fresh generated model inventory were built against completed dependencies;
this is local QA, not a normal Cargo or production release certification.

Four final native/source pairs are reviewed at matching position (8,4), clock,
facing and state: main front/east daylight, beta front daylight and beta west
night with a 64-frame ordinary LR walk. All eight full originals were inspected,
all commands exit0, and every paired receipt preserves state. The walk retains
an active native terrain owner and visits multiple actual positions. Captures
explicitly use HEADLESS=1, CAPTURE_TARGET=image, mute/high quality. No window or
focus changes occur. Earlier before/wall-only/seat-only diagnoses are retained.
Evidence: target/kanto-resume/clinic-upper-review/review.json and final/.

The final full 311/225 audit has no mesher errors. Only the eleven declared
upstairs maps change geometry fields; all other 300 map reports, all 225 variant
reports, actor-source and battle-species reports remain exact. Source/model
ownership, family counts and footing reports are unchanged. This is invariant
coverage, not whole-room/variant visual acceptance. Pack SHA256 remains
 a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.

Next concrete room defect: the three source-green link receptionists still
render in pink uniforms. Actor palette/fidelity, main/beta services and mobile
entry routes, wider room/furniture compositions, native save-animation timing,
remaining PC branches and full all311/original16 acceptance remain open. No
pack/audio edit, League restart, staging or commit occurred. Task scratch
builder was removed; ignored receipts, binaries and images remain.

Cable Club source uniform palettes and isolated GPU variants — 2026-10-19

The prior wall/furniture turn made verified progress. Current implementation
and AGENTS.md were inspected; the deleted handoff remains absent. This continues
the same complete interior/shared-furniture and all311/original16 objective.

The three actual Pokecenter2F link receptionists used a fixed pink external
painting despite their source-green object palettes. Native presentation now
reads the green/blue uniform color from each actor's already-paletted source
sprite. Only LinkReceptionist qualifies. Transparent pixels, skin, paper and
outline do not supply a uniform color. The existing source ID, map object,
texture and gameplay session are never rewritten.

A per-instance GPU variant keeps the canonical body, head, bind poses and bounds.
Only pink cloth in documented clothing UV islands is recolored; the collar,
badge, seams, hair, face, cap and alpha remain intact. Costume colors use the
same resolved color with unchanged geometry/joints/weights. Variants cache by
rig, kind and actual source color, so simultaneous green/blue uniforms cannot
alias. A source color change refreshes only the affected actor; unchanged
animation frames keep their existing entity. Region retirement drops unused
variant handles. External paintings remain immutable and no new raster asset,
model, content catalog or render-API field was added.

Four new checks pass: source-color/alpha filtering, pixel-island preservation,
actual body/costume GPU-cache separation and geometry identity, and production
actor refresh without mutating the frame or rebuilding the player. Three
existing controls also pass: asynchronous/direct publication byte identity,
independent costume animation/joints, and region asset retirement. The costume
control requires its documented explicit CRYSTAL_OPEN_MODEL_ROOT; the missing-
variable failure is retained separately, then the actual supplied root passes.
The first pixel assertion compared an entire appended mip chain with mip zero;
that diagnostic is retained without its huge pixel arrays. Final checks verify
unchanged mip-zero pixels through the normal filtered upload path.

Three final paired captures, six full originals, were inspected: actual main
room front daylight, main east at night with a64-frame ordinary LR walk, and
an unaffected Cinnabar nurse/control room. Both main views show green uniforms
with the existing white caps and faces; the nurse is not recolored. Commands
exit0, all paired-state receipts preserve the actual session, and the walk
retains active native terrain. Captures explicitly use HEADLESS=1,
CAPTURE_TARGET=image, mute/high quality. No window/focus changes occur.
Evidence: target/kanto-resume/clinic-uniform-review/review.json and final/.

The blue Goldenrod receptionist is declared at(16,8), outside this pack's base
10x8 clinic. An attempted base-room capture at(16,10) correctly fails its bounds
check and produces no accepted image. Blue material/cache behavior is checked
with an explicit presentation fixture, not claimed as an earned visible GS Ball
scene or an arrival/service proof. Keep that native scene as remaining work.

Fresh full311/225 reports are exactly equal to the preceding final audit:
all map geometry/source/footing/family reports, all225 variants, actor-source
and battle-species reports remain unchanged; no mesher errors. This proves
preservation, not whole-room or all-actor visual acceptance. Pack SHA256 stays
 a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Actual source/current model inventory QA builds use completed dependencies;
normal Cargo/release/browser certification is not implied.

Next concrete visual findings: Cinnabar's fisher still wears blue versus source
green, and cooltrainer_f's clothing also differs from source. Its clinic rear
apparatus/materials require a fuller room review despite positive family counts.
These are remaining fidelity work, not newly accepted rooms. Link/mobile/GS Ball
services, wider furniture/room/actor review, native save-animation timing,
remaining PC branches and all311/original16 acceptance remain open. No audio or
pack edit, League restart, staging or commit occurred. The task scratch builder
was removed; ignored QA images, logs and binaries remain.

Shared clinic apparatus relief and six-ball clearance — 2026-10-19

The uniform turn made verified progress. Current implementation and AGENTS.md
were inspected; the deleted handoff remains absent. The complete Kanto/shared-
furniture, all311/original16 and reachable-service objective remains active.

The Cinnabar before comparison showed that the authored machine still read as
a mostly flat front on a shallow enclosure. Its empty bay now recedes4px rather
than1.5px. Four thin chamfered bezel rails stay outside the aperture; the four
source side controls gain metal rims and cream push surfaces, and the source
red cross gains shallow relief. Fourteen closed chamfered pieces add448
triangles per apparatus. Colors come from the same actual source-art pixels,
not a replacement palette. The original front artwork, source claims, floor,
height, selected-model guard and healing-animation coordinates are retained.
No idle capsules or another opaque surface are placed in the bay.

The expanded enclosure regression checks all face winding, exact source-front
area, every source course, the4px cavity, actual control relief and clear bay,
plus unchanged claims/support and damaged-source rejection. Three healing-
effect controls, four actual-pack clinic terminal/stair/backing/stock controls,
and the actual-pack PokéCom apparatus control also pass. Two current-source
controller regressions pass: real HP/PP recovery through the nurse in all22
lobbies, and production publication/overlay consumption of all six healing
balls. Total:11 distinct focused regressions. Actual Rust source and a fresh
model inventory were compiled against completed dependencies; this is local
QA, not normal Cargo/release/browser certification.

Four final native/source pairs, eight full originals, were inspected: Cinnabar
quarter daylight, Pewter's actual six-ball healing scene, PokéCom's clinic
apparatus, and a Violet night UD walk. All commands exit0 and paired receipts
preserve the session. The six-ball case starts with an explicit disposable
six-member party, then uses ordinary native synthetic Up/A input and freezes
only after the sixth real effect publishes. All six drawn effects remain
visible with the new bezel. It is a service-presentation fixture, not an earned
arrival/main-story claim. The Violet walk retains active native terrain and
visits actual positions. Captures explicitly use HEADLESS=1,
CAPTURE_TARGET=image and mute/high quality, with no windows/focus changes.
Evidence: target/kanto-resume/clinic-relief-review/review.json and final/.

The first six-ball attempt used keyboard input and exhausted its route; its
rejected log is retained. The successful run uses the existing documented
WALK_INPUT=synthetic path. PokéCom's default center(16,16) is blocked; the rejected
start is retained, and the final view uses the previously verified walkable
clinic approach(5,10). Neither failure is relabeled as accepted evidence.

The full311/225 audit found the actual shared scope:22 clinic lobbies plus
PokéCom's existing source-identical apparatus. Only their textured vertex/
triangle fields change, each by448 triangles. All other288 map reports and
all225 variant reports remain exact, as do source/model family ownership,
footing, actor-source and battle-species reports. No mesher errors. The initial
comparison expected only22 maps; tracing the additional map confirmed its
existing shared helper, so its source test and native view were added rather
than suppressing the shared improvement. Pack SHA256 remains
 a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.

Whole-room wall/material and other equipment fidelity remain open, alongside
Cinnabar fisher/cooltrainer_f palettes, live link/mobile/GS Ball scenes, wider
room/furniture review, native PC save-animation timing, remaining PC branches
and all311/original16 acceptance. This apparatus repair does not certify those
rooms complete. No pack/audio edit, League restart, staging or commit occurred.
The task scratch builder was removed; ignored QA receipts/images/binaries remain.

Cinnabar fisher/cooltrainer source palette roles — 2026-10-19

The preceding route turn rechecked existing evidence and made no new interior
implementation progress. Current source and AGENTS.md were revalidated; the
deleted handoff remains absent. This closes the pending clinic palette repair
within the unchanged Kanto/shared-furniture and all311/original16 objective.

The fisher now takes his shirt and cap color from the actual paletted actor
sprite; cooltrainer_f takes her shirt accents and hair color from her sprite.
The existing LinkReceptionist per-instance palette cache supports these two
reviewed families. Blue shirt pixels and brown hair pixels are recolored only
inside their respective UV islands. Fisher hair, both dark lower garments,
faces, skin, badges, white details and alpha remain intact. A rigid head color
variant preserves every non-color attribute and index; body meshes, bounds,
binds, costume geometry and live independent joints remain shared/unchanged.
Canonical external paintings are immutable. No source actor/session rewrite,
new raster asset, content catalog or pack edit was introduced.

All17 tests selected by uniform_ pass, including seven source-palette, pixel-
island, cache/geometry-isolation and actor-refresh checks, plus ten existing
controls. Actual Rust source and fresh model inventory were compiled against
completed local dependencies for QA, not normal Cargo/release certification.
The full311-map/225-variant audit JSON exactly matches the preceding apparatus
baseline, including ownership, footing, geometry, actor-source and battle
reports; no mesher errors.

Three final native/source pairs, six full originals, were inspected: Cinnabar
quarter daylight, Cinnabar opposite night with an ordinary64-frame UD walk,
and the existing shared Cable Club green-uniform control. All paired receipts
preserve the actual session. The night walk reaches(5,3) through(5,7), retains
active Cinnabar terrain throughout, and requires no additional terrain build.
Explicit headless image capture, mute and high quality keep verification
non-disruptive. Evidence: target/kanto-resume/clinic-palette-review/review.json
and final/. Pack SHA256 remains
 a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.

Only LinkReceptionist, Fisher and CooltrainerF are supported by this palette
path. Warm/brown and other NPC palettes remain unreviewed; this is not global
actor fidelity acceptance. Whole-room wall/material/equipment review, live
link/mobile/GS Ball scenes, wider shared furniture, native PC save timing and
remaining PC branches, and all311/original16 acceptance remain open. No League
restart, staging or commit occurred. The task scratch builder was removed;
ignored QA evidence and caches remain.

Department-store ceiling-height glazing and continuous equipment backing — 2026-10-19

The preceding turn completed the pending clinic palette visual/evidence gate.
AGENTS.md and current worktree were revalidated; the deleted handoff remains
absent. The complete Kanto/shared-furniture and all311/original16 objective
remains active, with unrelated renderer and model changes preserved.

Fresh Celadon6F front/east source comparisons exposed16px glazing and side
walls below24px vending machines, plus missing wall behind the bank. The
shared architectural courses and lift fronts now fit32px room height, with
unchanged3px depth and original pane widths/materials. Directory and call-panel
inscriptions keep their original size and move8px upward. Furniture, stair
flights, thresholds, floor underlays and source claims keep their own datums.

The wider floor review then found the same missing backing behind2F equipment
shelves and4F refrigerators. The complete authenticated north-wall networks
now continue the same glazing behind actual blocks13/14/21 on2F/4F/6F in both
stores. These sections add no machine/floor claims or support heights. Full
network identity, map/floor/anchor, block and subtile phase remain required;
actual stair and lift openings are not filled. No new asset, pack or event
catalog is introduced.

All27 selected store geometry/material/source/retirement checks pass, including
an actual-pack test of all12 floors plus Celadon's post-callback layout. It
checks ceiling-height rear faces, backing rays through the full authored final
finisher, exact original claims/support and rejection of every hidden equipment
cell mutation. Existing stair/lift aperture, live inscription, source/crop,
custom-profile and counter checks remain green. Two current controller tests
also pass: the actual sixth-floor callback and both department-store clerks'
shop entry/exit. Total29 distinct focused checks.

Fourteen final native/source pairs,28 full originals, were inspected: every
floor1F–6F of both stores, plus Celadon6F east and Goldenrod6F opposite night
with an ordinary64-frame UD walk. All commands exit0 and all paired receipts
preserve state. The walk visits(8,3) through(8,7), retains active native terrain
and needs no additional terrain build. All captures explicitly use headless
image targets, mute and high quality. Source-correct openings and complete
rear/side height are visible; foreground furniture crops are not accepted as
whole-room coverage. Evidence: target/kanto-resume/retail-shell-review/review.json.

The full311/225 audit has no mesher errors. Only six map reports change solid
geometry counts:2F gains984 triangles per store,4F/6F gain656 each. The other305
map reports and all other report sections, including225 variants, ownership,
footing, actor sources and battle species, are exact. Height-only changes do
not change counts; the geometry tests and twelve-floor native review provide
that separate evidence. Pack SHA256 remains
 a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.

The first final-finisher fixture incorrectly used a geometry-only path and
uncentered coordinates; the retained failure is superseded by explicit neutral
opaque samples through the authored path at actual centered coordinates. The
initial blocked Celadon6F(5,4) and Goldenrod4F(8,4) starts were rejected; accepted
positions are(8,4) and(8,3). Intermediate wall-height-only images are retained
separately. Concurrent streamed-image API work required private current API
rebuilds; no concurrent source was reverted. Actual source was compiled for
local QA against completed dependencies, not normal Cargo/release certification.

Equipment face depth/materials, source seating interpretation, varied NPC
palettes, further room poses/services and all311/original16 acceptance remain
open, alongside native PC timing/remaining branches and link/mobile/GS Ball
scenes. No League restart, staging or commit occurred. The task scratch builder
was removed; ignored QA receipts, images and caches remain.

Mailbox services, native input ownership and source notice layout — 2026-10-19

AGENTS.md and the worktree were revalidated; the deleted handoff remains absent.
The complete Kanto/shared-furniture and all311/original16 objective stays active.
The preceding summit coverage check also passed against the unchanged pack.

Two actual-pack service regressions now cover all22 clinic lobbies. Ordinary
PCScript input opens the player PC and Mailbox, reads the second letter, closes
it, cancels and accepts PUT IN PACK, cancels party selection, attaches the exact
remaining MailData to the selected Pokemon, acknowledges the notice, handles
the empty list and logs off before real one-tile walking resumes. The negative
fixture checks Eggs, an already-held item and a full item pocket, preserving
the full message, party and bag on each refusal. Initial parties, letters and
bag contents are explicit disposable fixtures; subsequent actions use the
production controller rather than writing cursors, progression or outcomes.

The reader now owns directional, Start and Select input as well as A/B, so
reading cannot move its hidden Mailbox cursor. Expected attachment/full-bag
errors are recognized from their actual root causes and show the exported
source refusal text instead of failing with the generic mutation context.
Party attachment notices use the source20x6 PrintText window in rows12..17,
with two authored baselines; the oversized nested frame is gone. Empty-Mailbox
text is now drawn above the retained player-PC chooser. Other PC notice
layouts retain their existing behavior.

Native keyboard review exposed two further defects missed by direct service
tests. Idle/release frames previously lost their modal input budget to the
overworld loop, leaving A recorded as held and preventing READ MAIL. Mailbox
now retains that budget. The parent player's-PC confirmation also carries its
physical button history into the list, preventing held opening A from selecting
the first letter. A real-PCScript integration test now holds/releases ordinary
keys through source WaitBGMap, opens the list without selecting, opens and
closes the reader, reopens the submenu and changes its selection. It preserves
Mail and the player's actual position. Both native and default configurations
pass all8 focused Mailbox tests, including the two all22 service tests, five
existing control checks and this native keyboard regression. Earlier two
stationery/reader source-art checks and seven other clinic service checks also
have passing receipts; these are distinct scopes, not whole-room acceptance.

Eight full source-UI captures were inspected: list, Surf Mail reader, warning,
attachment acknowledgement, empty Mailbox, Egg/item refusals and full pack.
Their eight default-renderer counterparts are byte-identical. Four full
native/source originals were inspected for the PC hub and selected Surf Mail
reader. The final513-frame native route walks in the actual room, uses ordinary
keyboard input, selects the second letter and reaches CD's MEET IN KANTO at
PewterPokecenter1F(9,2). Its receipt preserves the paired session; all800x720
pixels of the displayed Mail LCD match between3D and2D. The initial first-letter
capture is retained as superseded evidence, not the selected-letter acceptance.
The QA policy only observes live owners/cursors and emits bounded keys; its
capture gate asserts the selected index and complete second MailData.

Evidence: target/kanto-resume/clinic-mail-build/review.json. All GPU captures
explicitly use HEADLESS=1, CAPTURE_TARGET=image and mute; no window is opened.
The pack remains ignored and SHA256 is unchanged:
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Current Rust source was compiled against the completed native dependency cache
in private QA outputs, including a private current voxel rebuild after a
concurrent outdoor-water API change. This is local QA, not Cargo/release or
browser certification; unrelated source/model work was preserved.

The independent mailbox_windows_match_original_rom_lcd comparison still fails
because its external nine-image fixture directory is absent; the first missing
reference is /Users/ryanculligan/GitHub/tools/asm-oracle/fixtures/mailbox/rom-mailbox-0.png.
No golden was fabricated or verifier weakened. That independent ROM-pixel gate,
parent PC palette/spacing review, remaining item/Oak/naming/printing branches,
native PC save-animation timing, link/mobile/GS Ball scenes, wider furniture
and all311/original16 visual acceptance remain open. No League restart, export,
staging or commit occurred. Task scratch builders were removed; ignored QA
receipts, images and caches remain.

Recessed retail roof vents across Kanto and Johto — 2026-10-19

Continued the full Kanto/shared-furniture and all311/original16 objective after
the Mailbox fixes. AGENTS.md and current source were revalidated; the deleted
handoff remains absent. No unrelated source/model changes were reverted.

The actual Mart0c/0d header draws two four-by-one-pixel vent slots. These now
recess0.35 source pixels into the roof, with closed inner walls, rather than
remaining flat paint. Only complete authenticated retail cabinet adapters use
the finish. Source UVs remain continuous, the old cap datum stays fixed, and
door/display attributes, placement, source claims and player support are exact.
Blue ticket boxes, long lids, electronics headers, clinic stock and existing
sixth-floor vending vents retain their separate treatments.

All28 focused store/retail checks pass. The new geometry test casts rays into
both openings and surrounding cap, rejects any covering plane, checks outward
triangle winding and unchanged height/claims/support, and compares every
original body/front attribute exactly across fridge/shelf fixtures in Kanto
and Johto. The old normal-only door assertion incorrectly included the new
inner roof walls; it now guards the actual front plane. Its initial failure
is retained alongside the final passing suite.

Fresh before/after audits cover all311 established interior maps and225 block
variants. Both have no mesher errors. Only textured geometry counts change in
fifteen maps: thirteen marts gain364 triangles/728 vertices each; Celadon4F
and Goldenrod4F gain624/1248 each. This is115 cabinets at52 triangles per cap.
The other296 complete map reports and every other report section are exact,
including source ownership, support, actors, battle species and all225 variants.
The initial Kanto-only120-map reports remain distinctly named before120/after120.

Four final headless native/source pairs, eight full originals, were inspected:
Celadon4F front, Goldenrod4F east/high angle, PewterMart quarter daylight and
EcruteakMart opposite night with an ordinary64-frame LR walk. All paired-state
receipts preserve the session. The walk visits(5,3) and(6,3), keeps native
terrain active and requires no additional terrain build. All captures use
HEADLESS=1, CAPTURE_TARGET=image, mute and high quality; no windows are opened.
A comma-separated camera argument was rejected; final requests use whitespace.
Both the default mart center(6,4) and guessed(4,4) were blocked; accepted(6,3)
starts come from prior source-checked map receipts.

Evidence: target/kanto-resume/retail-vent-review/review.json and
target/kanto-resume/retail-vent-build/invariants.json. Pack SHA256 is unchanged:
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Current Rust mesh code, tests and examples were built privately against the
completed native dependency cache for local QA, not Cargo/release or browser
certification. No pack, source-content export, new asset or gameplay change
was introduced. The task scratch builder was removed; ignored QA evidence and
caches remain. No staging, commit or League restart occurred.

This accepts the scoped roof finish, not whole-room art. The oblique mart
views leave checkout continuity/register detail and source floor hatching open;
trace counter geometry and cutaway ownership before changing the split-looking
tops. Foreground crops, further equipment faces, seating, actors, other poses,
remaining PC/item/Oak/naming/printing/link/mobile/GS Ball services and complete
all311/original16 visual acceptance remain unfinished.

Connected Mart checkouts and actual department-store counter joins — 2026-10-19

Continued the full Kanto/shared-furniture and all311/original16 objective.
AGENTS.md and current implementation were inspected. The deleted handoff stays
absent; these two maintained guides retain completion evidence and open work.
The preceding summit turn rechecked all 32 connected maps/20 surrounding routes:
both source/topology regressions pass, day/night originals were reviewed and
the actual pack hash is unchanged. That coverage does not accept every route's art.

All thirteen standard Marts now use their actual complete return drawing rather
than a separate register cabinet and fragmented legacy sides. Twenty source
cells authenticate the back-wall join and the complete drawing; sixteen counter
cells are owned, with the source south face folded upright. A continuous cap
runs from the wall to the front end at height eight. The existing mechanical
register is seated on that cap and faces west toward the actual clerk. Its
four finishes sample the source register; no new asset/content export is added.
All thirteen real source shop scripts open and browse their authored inventory,
close without mutating bag or money, and return to actual one-tile walking.
Initial room placement is a disposable fixture; interactions use the production
VisibleShellController, not direct cursor or progression mutation.

A shared loader defect was also repaired: body-only counter indices were saved
before surface refinement replaced a primitive's vertices and triangles. They
now come from the final refined material range. The regression compares the
exact material triangle mask and checks winding/normals and immutable cache.
This fixes malformed body faces in eight department-store floors as well.

Actual-map review found a further gap in both stores' 1F and 3F front strips.
Their two-row source drawings previously occupied one physical row, and the
standalone inset top still left a slit after enlarging the footprint. These
four maps now use their full sixteen-pixel depth and the same separate seven-
pixel body/eight-pixel continuous cap as the returns. A final-finisher test
loads all four actual pack maps and sweeps both return/front joins; it failed
at source depth32.25 before this fix and now passes. Source ownership, floor
underlay, support, shop scripts and unrelated clinic counters stay unchanged.

All 32 focused retail/store checks and four actual shop/controller checks pass.
The final audit covers 311 established interior maps and all 225 block variants,
with no mesher errors. Only thirteen Marts and eight store-floor reports change;
the other 290 complete reports are exact. Each Mart gains twelve modeled cells,
296 textured triangles and 768 vertices. Store 1F/3F each lose228 triangles and
288 vertices with the continuous cap; 2F gains1080 vertices and 4F gains540,
with their triangle counts unchanged. Actor sources, battle species, every
script variant and all audited support fields are exact. Counts establish
bounded scope; geometry tests and native views establish the counter joins.

Twenty-three final native/source pairs, 46 full originals, were inspected:
all thirteen Marts in daylight, both stores' floors1F–4F, and extra EcruteakMart
and Goldenrod4F night/high-angle LR walks. Fresh four-map join-final captures
supersede the earlier rejected 1F/3F gap views. The unaffected Mart/2F/4F pairs
retain the same audited meshes. All paired-state receipts preserve the session.
Each 64-frame walk visits two adjacent source positions, keeps terrain active
and stays at one terrain build. All captures are headless image targets, muted
and high quality; no windows or application focus changes occur.

Evidence: target/kanto-resume/mart-checkout-review/review.json and
mart-checkout-build/invariants.json, final-full311.json, final-focused-tests.log
and final-services.log under the same ignored QA root. The source hashes are
recorded there. Pack SHA256 remains
 a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Current Rust libraries/tests/examples were built privately against the completed
native dependency cache for local QA; this is not Cargo/release/browser
certification. The task scratch builder and probe source were removed. Ignored
QA evidence/caches remain; no pack export, staging, commit or League restart.

Scoped acceptance covers connected counter geometry/register placement, not
whole-room fidelity. Next source comparisons must address the dark counter
fascia versus the source's gold horizontal bands, Mart green floor hatching,
store pink seating, remaining equipment faces and NPC palette differences.
Foreground crops, other poses/reachability/services, remaining PC/item/Oak/
naming/printing/link/mobile/GS Ball branches, missing independent Mailbox ROM
goldens and complete all311/original16 visual acceptance remain open. The full
goal remains active; it is not redefined around this bounded repair.

Source gold courses on Mart and department-store counter fronts — 2026-10-19

Continued the complete Kanto/shared-furniture and all311/original16 objective.
AGENTS.md and current source were revalidated. The deleted handoff remains
absent; both maintained guides carry this identical checkpoint. The previous
turn made concrete source/geometry/service progress rather than a status-only
wait. Unrelated source, models, caches and other goals' work are preserved.

The thirteen Mart returns and both stores' floors1F–4F now sample the complete
actual tile$1a front strip by world height across vertical panels, framing and
bevels. The old adapter stretched a single dark edge sample across inset panels,
creating black rectangles where the source has gold bands and dark separators.
The new paint uses the existing room atlas; no second asset, image or catalog is
added. Horizontal body plates, caps and registers retain their finishes. Mesh
positions, indices, normals, colors, claims, support and interaction paths are
unchanged. Unknown/missing front art retains the previous whole-object finish.

Initial production review caught an incomplete lookup: floors2F/4F carry this
same source tile in different metatiles, so the narrow $0c–$0e/$29 filter left
their counters dark. The final lookup recognizes tile$1a within the already
authenticated native counter layouts. A new actual-pack regression requires
the real front strip in all eight store maps and rejects a foreign tileset.
The initial2F/4F captures remain separately named; stores-final supersedes them.
The original1F/3F views use exactly the same first source sample after that fix.

All34 focused store/retail tests pass. The thirteen-Marts source test now casts
independent rays through exposed overlapping body panels, interpolates visible
UVs and verifies the six inner source courses against the actual pack PNG.
It detects the old solid-dark-panel finish. A separate paint test compares
geometry, indices, normals and colors exactly and preserves cap/horizontal UVs
and missing-source behavior. The existing final-finisher join and support tests
remain green. No gameplay code changed; the preceding four shop/controller
regressions remain their separately recorded service evidence, not new runs.

Final headless inspection covers22 native/source pairs,44 full originals:
all thirteen Marts at a high quarter angle, both stores' floors1F–4F, and an
opposite/lower PewterMart night view exposing the long return face. Every pair
preserves the actual session. Its64-frame LR walk visits(3,3) through(7,3), keeps
terrain active and stays at one terrain build. Foreground store crops and Mart
views dominated by tops are explicit limits; the opposite view plus source rays
establish the return's striped finish. No windows or focus changes occur.

The final311-map/225-variant audit is exactly equal to the preceding checkout
report, with zero mesher errors. This proves unchanged report fields, including
coverage/support/actors/battle species/variants; the report does not fingerprint
UVs. Geometry-invariance tests and matched native views are additional required
evidence for this UV-only change. Pack SHA256 is unchanged:
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Evidence: target/kanto-resume/mart-fascia-review/review.json and
mart-fascia-build/invariants.json, final-full311.json and final-focused-tests.log
under that ignored QA root. Source/image hashes are recorded there. Current
Rust libraries/tests/examples were built privately against the completed native
dependency cache for local QA, not Cargo/release/browser certification. The
scratch builder was removed. No pack export, staging, commit or League restart.

This accepts the scoped gold-course finish, not every counter silhouette or
whole-room art. Next address the large plain green Mart pavers versus the source
hatching. Pink store seating, equipment backs/remaining faces, NPC palettes,
other poses, reachability and remaining PC/item/Oak/naming/printing/link/mobile/
GS Ball services remain open. Missing independent Mailbox ROM goldens and full
all311/original16 visual acceptance still prevent completion. The complete goal
remains active without narrowing its requirements.

Native green Mart floor crosshatch at the flat source datum — 2026-10-19

Continued the full Kanto/shared-furniture and all311/original16 objective.
AGENTS.md and current implementation were inspected; the deleted handoff stays
absent. The previous turn completed a source-backed counter finish with tests
and renders, so this is continued concrete progress rather than a blocked wait.
Unrelated work remains intact. Both maintained progress guides carry this same
checkpoint; no separate replacement handoff is created.

The thirteen standard Marts now retain the complete native green tile$48
crosshatch at its original scale and phase. The former two-swatch ceramic pass
removed the source lattice and introduced large green pavers, narrow grout and
shallow beveled seams. The finish now uses the full live atlas drawing on the
original flat floor plane, including existing furniture floor underlays. No new
texture, content catalog, furnishing claim, support height or gameplay behavior
is introduced. Department-store yellow floors and other room finishes retain
their own adapters.

All35 focused store/retail checks pass. The two old Mart tests asserted the
invented two-swatch paver treatment; they now require the complete UV rectangle,
flat height, exact area/winding, negative world origins and unequal tile axes,
and compare floor geometry/UVs/support/claims against the original source
surface in all thirteen rooms. A new actual-pack final-finisher regression
checks every plain tile$48 cell in every Mart: each is covered exactly once,
every floor vertex samples its original atlas position, walking support stays
zero and no floor is credited as a furnishing. Its opaque geometry fixture is
explicit; the actual pack PNG confirms a real motif and native/source captures
verify palette and appearance. All existing counter, roof, cabinet, wall and
shared store controls remain green. No service/controller code changed.

The complete311-map/225-variant audit has no mesher errors. Only thirteen Mart
reports change, each solely by4984 fewer textured triangles and15664 fewer
vertices. The other298 whole map reports are exact. All coverage/source bounds,
support, family ownership, actors, battle species, totals and225 block variants
are exact. This is bounded geometry evidence; it does not replace UV or visual
verification. No performance benchmark or production release claim is inferred
from the reduced triangle counts.

Fourteen headless native/source pairs,28 full originals, were inspected: all
thirteen Marts in daylight and an additional PewterMart oblique night LR walk.
Both native movement stills were also inspected. The crosshatch remains aligned
under furniture and walking actors; no large tile seams remain. Every pair
preserves the actual session. The64-frame walk visits(3,3) through(7,3), keeps
terrain active and remains at one terrain build. All captures use headless image
targets, mute and high quality; no windows or application focus changes occur.

Evidence: target/kanto-resume/mart-floor-review/review.json and
mart-floor-build/invariants.json, full311.json, actual-source-floors.log and
focused-tests.log under that ignored QA root. Source/image hashes are recorded.
Pack SHA256 is unchanged:
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Current Rust libraries/tests/examples were built privately against the completed
native dependency cache for local QA, not Cargo/release/browser certification.
The scratch builder was removed; ignored QA evidence and caches remain. No
pack export, staging, commit or League restart occurred.

This accepts the scoped Mart floor finish, not whole rooms. Next examine the
pink department-store seating against its source drawings. Equipment backs and
remaining faces, NPC palettes/occlusion, other poses and reachable services
remain open. Remaining PC/item/Oak/naming/printing/link/mobile/GS Ball branches,
missing independent Mailbox ROM goldens and full all311/original16 visual
acceptance still prevent goal completion. The full objective remains active.

Source-complete two-cushion department-store benches — 2026-10-19

Continued the full Kanto/shared-furniture and all311/original16 objective.
AGENTS.md, current source and fresh actual-pack drawings were inspected. The
previous turn completed the Mart floor repair with source/geometry/native QA;
this turn makes further authoritative progress. The deleted handoff remains
absent. Both maintained progress guides carry this identical checkpoint, and
unrelated source/assets/caches and other goals' work remain intact.

The pink store drawing is two connected cushions on one base: its upper half
already begins the next cushion, while the final six source pixels draw the
front apron/feet. The old adapters split the halves into separate puffy stools;
the6F right-hand complete drawing instead became only one pad. Both stores'
1F/5F/6F now resolve20 complete two-by-four-cell benches, with40 pads, one
continuous base and four grounded feet per bench. They reuse the canonical
Cushion mesh with the unsupported central button omitted, thinner padding and
live source upholstery/front/foot colors. The source frame's orange/red shade
is sampled from its actual front row, not invented generic metal paint.

The physical depth is26 source pixels after folding the drawn front upright;
seat tops remain at the established seven-pixel datum. Each complete eight-cell
guard retains the old total ownership and its floor underlay. Reservations and
altered/incomplete drawings never become partial native stools. Actor seating
exposes two separate pad rectangles ending before the folded apron, so merging
the source claim does not collapse the sitting positions to the center seam.
This is presentation geometry/targeting; authoritative collision, position,
scripts and input ownership remain unchanged.

There are49 distinct passing checks: the focused suite executes47 and initially
skips two supplied-rig checks; those two were then run explicitly and both pass.
New actual-pack checks cover all six rooms and every bench, both seven-pixel pad
tops, the closed common seam, the vacated front-floor row, four foot-contact
areas, triangle winding, unchanged support and eight claims. Production
seating_surfaces emits40 targets; each target agrees with its actual cushion
center/height. Guard checks mutate every source identity field in every owned
cell, exercise both left/right drawings, unknown maps, missing ground, each
reserved cell and unequal-axis/negative-origin geometry. Existing candidate,
NPC occupancy/cache, sitting transitions, supplied-rig hip/foot and all prior
retail/floor/counter controls remain green. The old synthetic half-stool test
now supplies the complete drawing and asserts a supported bench.

The311-map/225-variant audit has no mesher errors. Only the six store-floor
reports change, solely in textured geometry counts; the other305 complete
reports and all other fields/sections are exact, including ownership, support,
source bounds, actors, battle species and all225 variants. Each1F loses240
triangles/480 vertices and each5F loses120/240 by sharing bases/feet. Each6F
gains4160 triangles/2160 vertices because its formerly incomplete right-hand
assemblies now include both source pads. Counts alone did not detect that old
missing-pad defect; actual source/geometry/seating/native evidence is required.

Twelve headless native/source pairs,24 full originals, were inspected: all six
rooms in daylight, additional left/closer1F and5F views in both cities, and
opposite/lower6F night LR walks in both cities. The default5F poses crop the
benches; the closer pairs supply their actual inspection. Four movement stills
were also inspected, showing the existing renderer's seated presentation on a
cushion followed by walking away. Both64-frame walks keep their authoritative
approach outside the blocked source bench, visit(6,4) through(10,4), retain
active terrain and stay at one terrain build. Every final pair preserves its
actual session. No visible windows or application focus changes occur.

Evidence: target/kanto-resume/store-bench-review/review.json and
store-bench-build/invariants.json, full311.json, focused-tests.log and
supplied-rig-seating.log under that ignored QA root. Source/image hashes are
recorded. Pack SHA256 remains
 a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Current Rust libraries/tests/examples were built privately against the completed
native dependency cache for local QA, not Cargo/release/browser certification.
The scratch builder was removed. No asset/content export, staging, commit or
League restart occurred; ignored QA evidence/caches remain.

This accepts the scoped paired-bench geometry and sitting targets, not every
actor/party/animation or whole-room state. Fresh5F views still expose clerk/NPC
palette differences: trace their actual source object palettes next, especially
the left Goldenrod clerk and both Celadon clerks. Equipment backs/remaining
faces, NPC occlusion, other poses/reachability and remaining PC/item/Oak/naming/
printing/link/mobile/GS Ball services remain open. Missing independent Mailbox
ROM goldens and full all311/original16 visual acceptance still prevent overall
completion. The full goal remains active without narrowing its requirements.

Actual source colors for store clerks and receptionists — 2026-10-19

Continued the full Kanto/shared-furniture and all311/original16 objective.
AGENTS.md and current implementation were inspected. The previous summit
recheck supplied no new interior implementation; this turn takes the available
store-palette repair forward. The deleted handoff remains absent, and both
maintained progress guides carry this identical checkpoint. Unrelated changes
and shared caches remain intact.

Celadon5F's two SPRITE_CLERK objects explicitly use blue palettes, but the native
models retained green shirts. Goldenrod5F's left booth person is a distinct
SPRITE_RECEPTIONIST with a red palette, not another clerk; her blue suit/skirt
also ignored the source. Both roles now use the existing per-instance source
sprite color and GPU variant cache throughout the game. This covers29 Clerk and
18 Receptionist objects. Clerk shirts change while dark trousers and the cream
apron remain fixed. Receptionist shirt/trouser UV clothing islands change, and
her fitted skirt now receives the same explicit pigment instead of resampling
the immutable default blue painting. Skin, hair, collar, badge, ink, alpha,
head accessories, rigs, binds, positions and non-color garment attributes stay
unchanged. Existing LinkReceptionist/Fisher/CooltrainerF behavior is preserved.
No runtime script, collision, actor position or source pack changes are made.

All22 distinct uniform checks pass. The initial broad selection passes20 and
fails two supplied-rig controls solely because their explicit asset-root
variable is absent; rerunning those two with the existing assets/open-models
root passes both. New external-pack checks enumerate every actual object and
its resolved source palette, verify all six sprite frames and four lighting
banks for each used role/palette (144 combinations), and preserve source-style
border-connected transparency. Pixel-role checks protect non-clothing regions.
Supplied-rig GPU checks verify immutable base paintings, variant reuse and
cleanup, separate color materials, shared body/binds/head, unchanged garment
geometry and the cream Clerk apron; Receptionist skirt colors differ as
required. Prior link, Fisher and Cooltrainer tests remain green.

Ten headless source/native pairs,20 full originals, plus two movement stills
were inspected: daylight Celadon/Goldenrod1F and5F, TrainerHouse1F,
RadioTower1F and SilphCo1F; opposite55-degree5F views in both cities at21:00;
and a quarter-turn50-degree PewterMart night LR walk. The reviewed blue/red/
green clothing follows the actual source, including the green standard Mart
clerk and green Trainer House receptionist. The64-frame Mart walk visits
(3,3) through(7,3), retains active terrain and one terrain build. All ten paired
receipts preserve their actual sessions. Some views crop or shadow actors;
these establish scoped clothing acceptance, not every actor/face/room angle.
No visible window, focus change, asset export, staging or commit occurs.

The fresh311-map/225-variant terrain audit has no errors and is exactly equal
to the complete preceding bench report in every section. That audit does not
serialize actor pigment, so the actual sprite/GPU/image checks above are
necessary. Evidence: target/kanto-resume/service-uniform-review/review.json and
service-uniform-build/{invariants.json,full311.json,uniform-tests.log,
supplied-uniform-tests.log}. Current private opt-level1 Rust libraries and
examples use the completed dependency cache; Cargo/release/browser production
certification is not claimed. Scratch builders were removed; ignored QA
receipts/caches remain. Pack SHA256 stays
 a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.

A separate current shared foliage test remains failing:
occluder_fade::tests::opaque_sections_preserve_complete_triangles_collision_and_shared_fade_owner
fails leaf-bound containment. Its two Vec3A-to-Vec3 test conversions were
required for the current suite to compile; the actual containment assertion is
preserved and its failure receipt is service-uniform-build/occluder-test.log.
This is not hidden as a passing check. Equipment backs/remaining faces, other
NPC palettes and occlusion, remaining poses/reachability, PC/item/Oak/naming/
printing/link/mobile/GS Ball services, missing independent Mailbox ROM goldens
and complete all311/original16 visual acceptance remain open. The first
external Mailbox golden is still absent at tools/asm-oracle/fixtures/mailbox/
rom-mailbox-0.png outside this checkout. No completed League QA is restarted.
The full goal remains active.

Actual retail cabinet backs and exposed ends reviewed — 2026-10-19

Continued the full Kanto/shared-furniture and all311/original16 objective.
AGENTS.md and current source were reread. The previous staff-palette repair
was concrete progress; this turn completes another bounded acceptance check.
The deleted handoff stays absent, and this identical checkpoint is recorded in
both maintained progress guides. Other goals' changes and shared caches remain
intact.

The source layouts resolve299 complete native retail cabinet shells across19
rooms: nineteen in each of the thirteen standard Marts; six per department-store
2F, four per3F and sixteen per4F. Of these,115 use the previously repaired
vented roof and184 use their original unvented source roof. This cohort is
larger than the initial17-room estimate because both2F rooms also contain the
same native source display. They are not all electronics: the same source
adapter also supplies ordinary Mart stock displays.

The new actual-pack regression runs the production source resolver and append
path for every shell, with12x6 world pixels and translated origin(-31,17).
It checks full projected area on rear, front and exposed ends, authenticated
removal of buried joined ends, positive triangle winding/finite normals,
floor-to-cap contact, each cabinet's live casing UV sample, eight owned cells,
unchanged actor footing and no extra roof-header height. All299 pass. Three
existing display/source-guard, roof-recess/body-invariance and electronics-volume
controls also pass. The initial regression's geometry checks passed, but its
room-count assumption failed; actual enumeration established the exact19-room,
299-cabinet cohort retained by the final passing check. Both receipts remain.

Twenty-three fresh offscreen source/native pairs,46 full originals, were
inspected: all19 rooms from the reverse50-degree daylight camera, then
PewterMart, FuchsiaMart, CeladonDeptStore3F and GoldenrodDeptStore4F from the
side45-degree camera at21:00. The reviewed freestanding rear panels and ends
are closed, grounded and painted; the existing plain casing finish is retained.
No cabinet repair or added rear detail was necessary for these surfaces.
Some wall-backed displays are hidden by the opaque room wall in reverse views;
the geometry regression covers them, while these camera views establish only
the surfaces actually visible. All23 paired-state receipts preserve the actual
session. No window or focus change occurs.

Evidence: target/kanto-resume/retail-shell-faces-review/review.json,
retail-shell-faces-build/shell-tests-final.log and existing-display-controls.log.
This turn adds the regression and its test-only module registration; production
geometry, source content, input, actors, collision and support are unchanged.
Current Rust libraries/examples were rebuilt privately against the completed
dependency cache. A fresh311 audit was not needed for this test-only change;
the latest complete311/225 baseline remains service-uniform-build/full311.json.
No Cargo/release/browser certification is implied. Scratch builders were
removed; ignored QA receipts/caches remain. No export, staging, commit or League
restart occurs. Pack SHA256 remains
 a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.

This closes the scoped shell/visible-back review, not whole-room acceptance.
Blue aisle cases/ticket boxes, clinic stock and sixth-floor vending machines
use separate adapters and are not included in these299 shells. NPC readability
behind the tall stock rows remains visibly incomplete; source2D still shows
actors that some native angles obscure. Other NPC palettes, every remaining
pose/reachable service, PC/item/Oak/naming/printing/link/mobile/GS Ball work,
independent Mailbox ROM goldens and full all311/original16 visual acceptance
remain open. The previously recorded shared foliage containment failure was
not rerun or silently reclassified in this unrelated surface review. Continue
with the actual occluded actors/remaining palette roles and remaining services.
The full goal remains active.

Indoor NPC sight lines through source furniture — 2026-10-19

Continued the full Kanto/shared-furniture and all311/original16 objective.
AGENTS.md and current implementation were reread. The preceding cabinet-shell
review supplied new acceptance evidence; this turn fixes its concrete NPC
occlusion defect. The deleted handoff remains absent. Both maintained guides
carry this identical checkpoint, preserving unrelated work and other goals'
shared source changes.

The existing nearby-NPC reveal covered outdoor buildings only, while most
native stock cabinets were never registered as whole-object occluders. All299
reviewed source-complete cabinets in19 Mart/store rooms now register their
complete roof/front/back/ends with the existing depth-coverage furniture fade.
Their floor underlays remain opaque. Geometry, source ownership, support,
collision, shadows, scripts and authoritative actor state remain unchanged.

Source-backed indoor Object actors can now reveal explicitly registered
furniture that actually obstructs their lower torso or head. Targets use each
modeled character's stature and resolved terrain support; props, remote players,
invalid/out-of-grid actors and outdoor maps are excluded. Selection is stable
by source object ID and bounded to16; the external pack confirms every original
map fits that budget. The outdoor two-partner policy and ordinary/camera wall
rules retain their existing behavior. Cached mismatched-room footing cannot
supply these targets.

Each furniture piece uses the existing body-aperture shader with a union of its
actually blocked player/NPC bodies, so retained casing pixels do not stipple
characters or jump between two staff sharing a bay. Furniture restores its
original opacity after the line clears. The first visual pass exposed NPCs but
speckled their bodies; adding the aperture fixed that. A subsequent pass exposed
legs through low counters; NPC obstruction now tests torso/head instead. A
chest-only ray still grazed above a cabinet in Goldenrod2F, so the lower-torso
sample is40% of actual stature, with a dedicated grazing-top regression. The
low-counter regression remains green. Shader edits document the existing
aperture's additional caller; its fragment algorithm is unchanged.

All56 distinct focused checks pass: the current full52-test occluder suite and
four cabinet/display controls. New checks cover the real ECS fade/material
update with player rays clear, opaque restoration after NPC movement, wall and
stale-room exclusions, source support/stature, props/outdoor exclusions, stable
crowd limits against the actual pack, two-body aperture containment, low-counter
leg occlusion and the grazing cabinet edge. The actual299-cabinet regression
also requires one complete reveal owner, retained depth policy and opaque floor
underlays, alongside its geometry/claim/support checks. Existing outdoor, room
camera, cache, collision, source partition, packed-geometry and display controls
remain green. Shared packed-normal changes were preserved. The historical
leaf-containment check is absent from current shared source; it is not reported
as the same test rerun and passing.

Nine final headless source/native pairs,18 full originals, and four movement
stills were inspected: reverse50-degree daylight views in Azalea/Pewter Marts
and both stores'2F/4F, side45-degree21:00 LR walks in PewterMart and
Goldenrod4F, plus a PlayersHouse1F wall/furniture control. The affected NPCs now
read clearly through the tall stock rows while low service counters remain
opaque. The house control retains its connected full-height wall/divider and
ordinary furniture. Both64-frame walks retain active terrain and one terrain
build; Pewter visits(3,3)..(7,3), Goldenrod4F visits(5,3)..(9,3). Their measured
medians are20.77/20.27ms and p95 values43.90/28.08ms in these private QA builds;
this is not production performance certification. Every final pair preserves
its actual session. Rejected/intermediate visual passes remain separately
identified as evidence, not accepted final views. No windows or focus changes
occur.

The fresh311-map/225-variant static audit has no errors and is exactly equal
to service-uniform-build/full311.json in every section. That audit does not
serialize reveal eligibility/apertures; the actual source-owner and live
ECS/material checks above prove those separately. Evidence:
target/kanto-resume/interior-npc-review/review.json and
interior-npc-build/{occluder-tests-final.log,cabinet-tests-final.log,
full311-final.json,invariants.json}. Current Rust libraries/tests/examples were
built privately against the completed dependency cache. The scratch builder
was removed; ignored QA receipts/caches remain. Pack SHA256 stays
 a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
No content/asset export, staging, commit or completed League restart occurs.

This accepts the scoped source-furniture NPC repair and reviewed poses, not
all actor/party/camera states or every room. Separate blue aisle/ticket-box,
clinic and vending adapters still need their remaining registration/reveal
review where applicable; ordinary-wall occlusion remains a separate policy.
Other NPC palettes, remaining reachable services (PC/item/Oak/naming/printing/
link/mobile/GS Ball), independent Mailbox ROM goldens and full all311/original16
visual acceptance remain open. The first external Mailbox golden is still
absent. The full goal remains active without narrowing its requirements.

Shared blue cases, ticket boxes and vending services — 2026-10-19

Continued the full Kanto/shared-furniture and all311/original16 objective.
AGENTS.md and current implementation were inspected. The preceding summit
verification confirmed coverage but did not advance the remaining interiors;
this turn repairs a concrete missing furniture family and finishes the pending
shared-furniture ownership and vending checks. The deleted handoff stays absent.
Other goals' source/model changes remain intact.

Ordinary Marts used blue-case blocks $23/$28 rather than department-store $12.
Their eight furniture cells are identical; only the west-side floor is $48.
The complete-drawing matcher now admits both variants and the existing native
case adapter consumes them: two grounded cases in each of13 Marts,26 previously
flat/unmodeled units. They retain the established24px long lid,8px body, flat
sides and0.6px top-only ridges. Goldenrod's separately authored taller ticket
boxes retain their existing model and orientation. Every changed-cell mutation
in all26 actual Mart cases rejects the complete native placement; the floor
half never becomes furniture.

All48 blue cases, eight beverage machines, two Indigo stock cabinets and four
station ticket boxes now have complete furniture reveal owners across21 rooms.
The four station units are independent owners, not one fading bank. Vending
roof vents share their own machine's owner. Indigo's rear wall backing is
appended before the furniture range and remains opaque; floor underlays also
stay opaque. Actual source ownership, support sentinels, complete indexed
triangles, separate ranges and real prepare_linked grouping are checked against
the verified external pack. Existing collision and scripts are unchanged.

Goldenrod and Celadon6F now share production-controller vending acceptance:
all three drinks at their source prices, funded/no-money/full-pocket cases,
exactly one item and charge on success, unchanged inventory/money on refusal,
item-notice input ownership, the original qualified source menu, cancellation,
real departure movement and production Start access. All18 scenarios pass.
The first movement assertion was too strict about the first input frame;
the final check permits at most eight actual Down presses, requires the real
customer tile to change, and checks no runtime error. No movement or menu state
is assigned to bypass the controller.

All74 focused checks pass:52 occluder tests,18 Mart controls, one actual62-owner
regression, the existing299-cabinet control and two vending-controller tests.
The final311-map/225-variant audit has no errors. Only13 Marts change: each gains
16 modeled cells,92 triangles and184 vertices, and only the sixteen blue-case
cells leave unmodeled coverage. All298 other maps and225 variants remain exact;
actor/battle sources and all other top-level audit sections remain exact apart
from expected family/geometry totals. Static audits do not encode reveal policy;
the source/range/grouping and live ECS controls cover that separately.

Twenty-four headless native/source pairs,48 full originals, plus four movement
stills were inspected: reverse daylight in the eight shared-adapter rooms,
front daylight in all13 Marts and Indigo, and21:00 side/aisle walks in Pewter
and Goldenrod6F. All paired states are preserved. Both64-frame walks retain
active terrain and one build; Pewter visits(3,3)..(7,3), Goldenrod(9,2)..(9,6).
The initial Goldenrod lateral walk was blocked by the actual neighboring NPCs;
its failure is retained and the accepted walk uses the open aisle. Indigo's
reverse view correctly keeps the backing wall opaque and cannot certify player
readability through that architecture; the front view verifies the stock.
Other NPC palette mismatches remain visible and require separate work.
No window, app switch or focus change occurred.

Evidence: target/kanto-resume/shared-furniture-review/review.json and
shared-furniture-build/{shared-owners-final.log,occluder-final.log,
mart-controls-final.log,cabinet-control.log,vending-controller-final.log,
full311-final.json,invariants.json}. Pack SHA256 remains
 a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
The private scratch builder was removed; ignored QA receipts/caches remain.
No content export, staging, commit or completed League restart occurs.

This completes the scoped missing-case/owner/vending repair. Ordinary-wall
occlusion, remaining actor palettes and camera/party poses, reachable PC/item/
Oak/naming/printing/link/mobile/GS Ball services, independent Mailbox ROM goldens
and full all311/original16 visual acceptance remain open. The first external
Mailbox golden is still absent. The original full goal remains active.

Source clothing for Lasses, Youngsters, Cooltrainers and Super Nerds — 2026-10-19

Continued the full Kanto/shared-furniture and all311/original16 objective.
AGENTS.md and current implementation were inspected. The preceding turn repaired
26 missing Mart cases and verified shared furniture/service ownership; this turn
addresses the visible fixed-color NPC clothing left in those reviews. The
deleted handoff remains absent. Shared files and unrelated model work are
preserved; no other goal's build cache or generated assets were changed.

The source sprite confirms that female Cooltrainer hair deliberately shares its
palette accent. That existing behavior is preserved, correcting the initial
suspicion that all colored hair was erroneous. The actual mismatches were fixed
red/blue clothing on Lass, Youngster, male Cooltrainer and Super Nerd models.
Their per-instance clothing now uses the already paletted source image, including
actual brown variants. Hair, faces, skin, white details, ink and alpha remain
unchanged. Lass skirts and male Cooltrainer trousers follow the source accent;
Youngster/Super Nerd dark trousers retain their existing dark role. The first
pass still left male Cooltrainer trousers blue; that source-role defect was
fixed and all final views were captured again. No actor identity, physical
scale, source position, controller input, rig, topology or collision changes.

The actual external-pack corpus covers63 Lasses,124 Youngsters,38 male
Cooltrainers and61 Super Nerds:286 newly supported objects. Every actual palette
across all six sprite frames and four palette banks passes,360 new combinations.
With the existing29 Clerks and18 Receptionists, the same regression checks333
objects and504 palette/frame combinations. It uses pack PNGs and palette data,
including border-connected transparency, rather than a duplicate source catalog.

All23 distinct uniform/role/cache/geometry controls pass. The GPU variant check
now covers all six staff/community types: shared body/bind/head meshes and
physical bounds, distinct source-colored materials, immutable base paintings,
alpha and protected UV roles, actual nonempty repainted cloth, preserved Clerk
aprons, source-colored Lass/Receptionist skirts and equal costume geometry.
The live actor-refresh regression was subsequently expanded and passes for the
four new source types plus Link Receptionist: only the affected Object actor
refreshes when its palette changes, the player stays intact, old entities are
removed, steady frames reuse the variant, and the VisualWorldFrame is exact.
Existing Fisher/cap, female Cooltrainer/hair, link, uniform-cap and detail
controls remain green. No test is skipped in the23-check run.

Eleven final headless native/source pairs,22 full originals, plus four moving
stills were inspected: daylight in Pewter/Cerulean/Saffron/Viridian Marts, both
6F stores, TrainerHouse1F, RadioTower1F and FuchsiaMart;21:00 side Pewter and
reverse Goldenrod6F walks. They show source blue/red/green/brown cloth with
retained faces, hair and accessories. Fuchsia retains the previous correctly
source-colored female Cooltrainer/Fisher behavior. Both64-frame walks preserve
active terrain and one build: Pewter visits(3,3)..(7,3), Goldenrod(9,2)..(9,6).
Every paired state is preserved. Superseded first-pass captures are separately
identified; only their Saffron/Goldenrod originals were inspected and they are
not counted as accepted final views. No windows or focus changes occur.

The fresh311-map/225-variant static audit is exactly equal to
shared-furniture-build/full311-final.json in every section, with no errors.
It cannot certify actor painting; the actual source, GPU and live-instance
checks provide that evidence. Receipts:
target/kanto-resume/community-colors-review/review.json and
community-colors-build/{uniform-tests-final.log,live-refresh-final.log,
full311-final.json,invariants.json}. Pack SHA256 remains
 a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Current Rust libraries/tests/examples were privately built against the completed
dependency cache. Temporary source enlargements and the scratch builder were
removed; ignored production QA receipts/caches remain. No content export,
staging, commit or completed League restart occurs.

This accepts these four clothing roles and the reviewed views, not every one
of286 actors or all poses. Other unsupported NPC palettes/details, ordinary-wall
occlusion, remaining camera/party states and reachable PC/item/Oak/naming/
printing/link/mobile/GS Ball services remain open. Independent Mailbox ROM
goldens are still absent, including the first required image. Full
all311/original16 visual acceptance remains incomplete. The original full goal
stays active without narrowing its scope.

Surrounding routes coverage rechecked — 2026-10-19

The Tin Tower roof panorama uses the complete connected source component:
32 actual maps, including all 20 routes 26, 27 and 29–46, their connected
settlements and BattleTowerOutside. Existing authored offsets and native scale
are retained. Re-ran both current production-host summit regressions: complete
pack coverage/source tiles/seams/session preservation and cyclic traversal;
2 passed, 0 failed. Log: target/kanto-resume/all-surrounding-routes-native/current-route-check.log.
Re-inspected the existing four full day/night native/source originals from
all-surrounding-routes-native; no visible windows were opened. This confirms
panorama coverage, not individual route art acceptance. The recorded panorama
loading cost remains unresolved. No geometry or gameplay changes were needed
for this recheck.

Item PC notices and Oak's complete PC question flow — 2026-10-19

Continued the full Kanto/shared-furniture/all311/original16 objective after the
surrounding-route coverage check. AGENTS.md and current source were inspected;
the deleted docs/handoff/kanto-interiors.md remains absent. Checkpoints continue
here and in modeled-interiors.md. Shared and unrelated changes are preserved;
no pack export, source catalog, staged content, or geometry change was made.

The preceding item-PC work is now recorded: actual PCScript entry in all22
clinic lobbies, withdrawal/deposit/cancellation, all four source pockets,
protected single items, full-PC/full-Pack/empty refusals, and safe Toss NO/YES.
Notices now use the pack's authored two-line text instead of clipped invented
sentences; the older pack's retained decimal slot is resolved for quantities.
Toss waits for its question to print, ignores horizontal input, clamps YES/NO,
and draws the source6x5 box with two-row baselines. The retained LCD render key
now includes notice reveal, quantity, confirmation and actual cursor, fixing
blank native quantity/Toss windows. The narrow three-color screenshot admission
requires live frozen fullscreen-PC ownership; ordinary scene checks remain.
Prior evidence: target/kanto-resume/item-pc-review/review.json,33 passing checks,
three matched pairs/six inspected originals plus ten production-UI rasters.
Earlier blank/clipped/rejected captures are explicitly superseded.

Oak's PC previously bypassed ProfOaksPC's question and decline path. It now
prints the actual access text, asks the source rating question, and waits for
printing before accepting YES, NO or B. YES invokes the existing authoritative
rating and source fanfare; NO/B skip both. All branches print _OakPCText4 before
returning to the hub. Oak lab assessments retain their separate script return.
Question geometry/input uses the source PC YesNoBox. Access, rating, question
and shutdown use the actual map text palette, including its yellow background.

Native QA caught a real defect in the first implementation: reusing item-window
battle markers suppressed the clinic behind the field question. A field-owned
palette window now retains the actual room; the live-render regression rejects
those world-suppressing markers and verifies printed text/cursor repainting.
The initial blackout/timed-out native run is retained as a failure, not accepted.

Final-source checks: two new Oak tests pass, including all22 lobbies times
YES/NO/B (66 actual PCScript visits), nonzero recorded seen/owned counts,
unchanged party/storage/bag/money/Pokedex, pre-print input ownership, and return
to walking and production Start. Five existing Oak checks pass: all19 source
ratings, complete lab text, original lab script continuation, all19 fanfare
waits and acknowledgements, and pack PCM hash/frame/loop integrity. All seven
item-PC integration/UI tests pass again on final source (14 distinct final-source
checks total). Twenty available item controls also passed before the scoped
field-window refinements. Full original PC-item acceptance is not claimed:
its three missing independent ROM fixtures still fail, as recorded below.

Final evidence: target/kanto-resume/oak-pc-review/review.json. All six full
native/source originals were inspected: question/YES, B-decline shutdown, and
actual3-seen/2-owned totals at Pewter's real PC. Every pair preserves session
state and its textbox pixels match exactly; the question's YesNo pixels also
match. Three final production-UI rasters show YES, NO and shutdown. These nine
images are current-source evidence, not independent ROM screenshots. All renders
use HEADLESS=1 and CAPTURE_TARGET=image, with no windows or focus changes.
Pack SHA256 remains a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007;
source revision remains3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6. Scratch builders
and generated save backups were removed; verification artifacts remain ignored.
No fresh311 geometry audit was needed/run for these controller/UI-only changes.

Remaining: source YesNoBox's post-answer15-frame pause and independent complete
ROM timing/pixel acceptance; generic YesNo geometry outside these source PC
boxes; absent tools/asm-oracle/fixtures/pc-items-initial.png,
pc-items-scroll.json and pc-items-move.json, plus the independent Mailbox ROM
goldens. Other reachable naming/printing/link/mobile/GS Ball services, remaining
actor palettes/details/poses, ordinary-wall occlusion, and full all311/original16
visual acceptance remain open. The overall objective remains active.

Shared field YES/NO layout and live Save acceptance — 2026-10-19

Continued the full Kanto/shared-furniture/all311/original16 objective. AGENTS.md
is unchanged (SHA1 7e2e3189e2cfed182daa4c841955e9b8a4b3a5ed); current code and
remaining checkpoints were inspected. The deleted handoff remains absent.
Unrelated/shared changes are preserved. This resolves the preceding checkpoint's
open generic YesNo geometry issue, without claiming the wider objective complete.

The original home/menu.asm sets inclusive borders(14,7)..(19,11), cursor column15,
labels from16, and YES/NO rows8/10. The shared field presenter incorrectly used
6x4, adjacent answer rows and glyphs starting on the left border. Its initial
draw and retained glyph update now both use the source6x5 geometry and inset
labels. The complete speech-plus-prompt frame has66 perimeter tiles. Separate
source item/Oak/nickname presenters retain their existing five-row geometry.
No room, furniture, collision, actor, script, pack or audio source was changed.
Maintenance reference: docs/art/field-yes-no.md.

Three focused tests pass: eight phone/day/PC/Save cursor-owner/answer layout
fixtures authenticate all18 literal source border tiles and seven glyph slots;
a retained cursor update preserves all frame entities and is pixel-identical
to a fresh draw; and44 production Save visits in all22 clinic lobbies decline
with NO/B, preserve party/storage/bag/money, write no save and return walking.
The layout fixtures are explicitly isolated renderer phases, not claims of
end-to-end acceptance for every corresponding service.

All18 distinct selected checks pass, including those three, shared cursor and
orphan-frame controls, actual title-to-Save writing/reloading, the production
Mom/New Bark gate controller, seven item-PC checks and both Oak PC checks.
The full Mom-to-starter keyboard test also passes: all Mom dialogue/YesNo and
weekday/DST presentation, input release, house exit/re-entry with one regular
Mom sprite, route to Elm and actual starter selection. Its existing input and
leak thresholds were retained. Previously dead test setup now loads the verified
external pack and correct repository root; title setup finishes the actual boot
splash; live entity checks use Bevy0.20 World.entity_count rather than allocated
index capacity. The geometric warp planner now recognizes source directional
door coordinates/facing and the outward input, including arrival on the door.
Every planned step/warp is still replayed through production keyboard input.

Native QA additionally exposed an old route alias: S sent Select/ShiftRight and
cycled the camera. It now sends the production Start key, Enter. A field-Save
observer admits only the real Start-origin prompt and cursor; it does not seed
service state or bypass world/capture gates. A menu-only replay was correctly
rejected by movement QA; the final DU approach retains the ordinary requirement.

Evidence: target/kanto-resume/shared-yes-no-review/review.json. Both full final
native/source Save originals plus four production UI rasters were inspected
(six images). The pair preserves the authoritative state, and both its textbox
and YesNo pixels match exactly. The room remains visible, labels fit, and no
save file is written by the review. Every native render uses HEADLESS=1 and
CAPTURE_TARGET=image; no windows/focus changes. Failures are retained separately
and not accepted. Scratch builder and temporary instruction-read file removed.
Pack SHA256 remains a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007;
pinned external source remains3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6. No pack
publication or fresh311 geometry audit was needed/run for these UI/test changes.

Remaining: source post-answer15-frame timing, generic field palette fidelity,
custom two-option caller layouts and independent ROM pixel/frame acceptance;
the three item-list/scroll/move fixtures and Mailbox goldens remain absent.
Other reachable naming/printing/link/mobile/GS Ball services, remaining actor
palettes/details/poses, ordinary-wall occlusion and full all311/original16 visual
acceptance remain open. The full objective remains active.

Body-aware smooth partitions and source objects — 2026-10-19

Continued the full Kanto/shared-furniture/all311/original16 objective after the
shared dialog work. AGENTS.md remains unchanged; current implementation and
remaining checkpoints were inspected. The deleted handoff remains absent.
Scoped edits preserve shared/unrelated work and the existing pack.

The ordinary wall review found a narrower concrete defect: the indoor people
reveal required coverage rendering even for an already registered depth-aware
source object. Smooth WiseTriosRoom partitions tinted the player, and smooth
PowerPlant partitions hid workers clear of the player's rays. Coverage is a
blend choice, not source ownership. Eligibility now admits both registered
smooth and coverage owners, retaining residual opacity0.20 and excluding
camera-wall planes. The existing per-owner union clears only its obstructed
player/NPC bodies. Source model stature/footing and torso/head obstruction stay
authoritative; low counters hiding shoes still stay opaque. No mesh, range,
wall height, join, opening, source floor/support, collision or shadow change.
Unregistered ordinary walls retain their previous policy and remain open.
Maintenance reference: docs/art/interior-people-reveal.md.

All69 distinct selected checks pass:54 occluder/people/fade/cache controls,
12 PowerPlant source/finish/ownership controls and three additional unique
partition/table/Rocket controls. The smooth and coverage live-system cases now
both prove NPC-only obstruction with player rays clear, player-only obstruction,
fully opaque restoration, unchanged blend recipe, stable mesh/material counts,
camera-wall exclusion and map/cache identity. The source fixtures retain
ceiling-height sides, original openings, table bodies, floor paint and support.
A concurrent packed-grove pigment test appeared in the shared fade file; that
work is preserved and included in the final54-check pass. Shared-file
coordination was sent under the original objective. No exporter was run.

Fresh static audit in target/kanto-resume/wall-reveal-build/full311.json is
exact against community-colors-build/full311-final.json in every section:
all311 map records,225 script-block variants, source claims, support, family
bounds and geometry counts, with zero mesher errors. This is static verification,
not whole-room art acceptance or certification of concurrent grove optimization.

Evidence: target/kanto-resume/wall-reveal-review/review.json. Eight final matched
native/source pairs (16 full originals) plus four moving stills were inspected:
WiseTriosRoom front/east/west, PowerPlant east and opposite night walk,
TeamRocketBaseB1F ordinary-wall control, and MrPokemonsHouse wide-table control.
The final20 images show clear body clothing through the affected partitions,
retained surrounding form and floors, connected full-height wall geometry and
unaffected controls. Six diagnostic before pairs (12 further originals) were
also inspected. Independent runs can differ in idle NPC facing; each individual
2D/3D pair preserves its exact authoritative state and presentation phase.

Both ordinary UD walks retain64 active/settled frames, one terrain build and no
warp/fade: WiseTriosRoom(4,4)->(4,3)->(4,4), PowerPlant(9,9)->(9,8)->(9,9).
All captures explicitly use HEADLESS=1/CAPTURE_TARGET=image and mute; no windows
or focus changes. The default MrPokemon center(4,4) was correctly rejected as
an unwalkable source table. Its accepted source entrance fixture is(3,7);
the failed attempts are retained and not accepted. Scratch builder removed;
verification binaries/images/reports remain ignored. Pack SHA256 remains
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.

Remaining: unregistered ordinary walls and additional camera/body poses; visible
NPC palette/detail inaccuracies (including PowerPlant staff), other room finish
and service acceptance; post-answer timing, generic field palette/custom-menu
checks and absent independent item/Mailbox ROM fixtures; reachable naming,
printing/link/mobile/GS Ball services and full all311/original16 visual review.
The full objective remains active.

Officer source uniforms and fitted collars — 2026-10-19

Continued the full Kanto/shared-furniture/all311/original16 objective after the
surrounding-route coverage recheck. Current implementation and AGENTS.md were
inspected; AGENTS.md SHA1 remains 7e2e3189e2cfed182daa4c841955e9b8a4b3a5ed.
The deleted handoff remains absent. Shared/unrelated changes are preserved;
localized renderer edits were coordinated with the active performance goal.

PowerPlant's two green source Officers were fixed-blue native models. Officer
instances now derive their cloth pigment from the already paletted sprite,
including jacket and trousers, fitted cap crown and raised collar. The first
native review caught the separate collar still blue; the final implementation
also colors that mesh. Faces, hair, hands, shoes, white trim, transparent pixels,
gold badges, dark visors/bands and seams retain their roles. Body dimensions,
geometry, rig/joints/weights and gameplay remain unchanged. Default accessory
callers retain their existing appearance. Reference: docs/art/officer-source-colors.md.

All13 distinct selected checks pass: seven source/pigment/painting controls,
three cap geometry/fit/color checks, actual-rig GPU variant isolation, animated
uniform-detail fit and live source-color refresh. The pack corpus covers all33
Officer objects, their three used palettes, four palette banks and six sprite
frames (72 combinations). Actual paints change both shirt and trouser islands
while retaining protected pixels and alpha. The live-system test now includes
Officer: source palette changes replace only that actor, preserve the player
and authoritative frame, and ordinary animation retains the resulting entity.
The initial cap-fit invocation omitted its required model-root variable;
rerunning with the supplied artwork passes. Its failed receipt is retained.

Evidence: target/kanto-resume/officer-colors-review/review.json. Seven final
matched pairs (14 full originals) and two walking stills were inspected:
PowerPlant east/day, west/night and ordinary UD walk; Lavender radio Officer
front/east; blue Route15FuchsiaGate and red Route31VioletGate. Both PowerPlant
Officers and Lavender's Officer now use green cloth, and red/blue gate Officers
follow their own source palettes. Two additional diagnostic pairs (four full
originals) were inspected. Exact state preservation is proved within every
2D/3D pair; independent runs can differ in idle NPC facing. The Lavender14,3
start was correctly rejected as unwalkable. Its entrance3,6 diagnostic leaves
the Officer offscreen; final walkable15,2 views expose the actual Officer.

The final PowerPlant walk has64 active settled frames, one terrain build,
no fade/warp and actual(9,9)->(9,8)->(9,9) movement. Captures explicitly use
HEADLESS=1/CAPTURE_TARGET=image and mute, with no windows/focus changes.
Fresh final static audit officer-colors-build/full311-final.json exactly equals
wall-reveal-build/full311.json in every section:311 maps,225 script variants,
source/support/family bounds and geometry counts, zero mesher errors. This
proves static invariants, not whole-room or concurrent-performance acceptance.
Pack SHA256 remains a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
No exporter ran. Scratch builder/capture scripts were removed; evidence stays
ignored. No generated pack, audio, source catalog or build artifact was staged.

Remaining: unregistered ordinary walls and additional camera/body poses; other
NPC details, room finishes and reachable services; post-answer timing, generic
field palettes/custom menus and absent independent item/Mailbox ROM fixtures;
naming, printing/link/mobile/GS Ball services and full all311/original16 visual
acceptance. This scoped uniform correction is complete; the full goal remains
active.

Transparent radio studio glazing and clear doorway — 2026-10-19

Continued the full Kanto/shared-furniture/all311/original16 objective after the
Officer correction. That preceding turn made verified source/visual progress.
AGENTS.md and current implementation were inspected; SHA1 remains
7e2e3189e2cfed182daa4c841955e9b8a4b3a5ed. The deleted handoff remains absent.
Shared renderer work is preserved and localized changes were coordinated with
the active performance goal; no exporter or competing Cargo cache was used.

The radio studio partitions were connected but their pane bodies and reflection
faces were opaque textured geometry. Explicit glass roles now publish through
the existing blended scenery path using an atlas assembled from the immutable,
already paletted live tile samples. Source UVs/colors remain authoritative;
there is no replacement palette or exported content. A faint body tint and
stronger reflection surface reveal floors, people and apparatus behind the
panes. Header caps, bottom rails, dark bands, side mullions and grille equipment
stay opaque. Physical front end stiles retain the pane silhouette independently
of reflection paint. Connected east returns and the fifth-floor opening retain
their authored placement. Vertex ownership ranges remain stable for existing
fade grouping; only referenced glass triangles move from the opaque draw.
Missing/invalid pixels retain an opaque source drawing and ordinary rooms
retain their previous policy. Reference: docs/art/radio-glazing.md.

All37 distinct radio-related checks pass, including two new live-atlas/blend
and missing-data/ordinary-room controls, complete source pane/table/grille/claim
controls, stairs/support/warp endpoints, equipment/material identity and existing
exterior/house-radio controls. The blend test verifies exact base source atlas
pixels (before generated mip levels), unchanged framing geometry/ranges and
support, immutable source alpha and a separate double-sided blended pane mesh.
The first fixture compared generated mip bytes with base pixels; it was corrected
to verify the exact base bytes while allowing normal mip generation.

Fresh full audit: radio-glazing-build/full311.json against
officer-colors-build/full311-final.json. All311 maps retain exact source claims,
floor/support, family bounds and other non-geometry fields. All225 script-block
variants and every other report section are exact; mesher errors remain empty.
Only four opaque geometry records differ, as glass moves to scenery and front
stiles are added: LavRadioTower1F -144 triangles/-288 vertices, RadioTower2F
-216/-432, RadioTower4F -84/-168 and RadioTower5F -132/-264. The remaining307 map
records are exact, including RadioTower3F's ordinary office control.

Evidence: target/kanto-resume/radio-glazing-review/review.json. Ten final matched
pairs (20 full originals) plus six walking stills were inspected: Radio2 front
and west, Radio3 control, Radio4, Radio5 front/east, Lavender front/east, and two
Radio5 west walks. Two additional diagnostic before pairs (four originals) were
also inspected. Panes read as glass with visible background and retained caps,
frames and continuous returns. Each native/2D pair preserves its authoritative
state. Independent before/after runs can differ in idle NPC facing.

The Radio5 night LR walk has64 frames and moves(10,7)->(9,7)->(10,7). The ordinary
DDUU doorway walk has128 frames, crossing(16,2)->(16,3)->(16,4) and back through
the authored opening. Both retain one build, active terrain with no pending
state, zero fade and no warp on every frame. All captures explicitly use
HEADLESS=1/CAPTURE_TARGET=image and mute, with no windows or focus changes.
Pack SHA256 remains a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Scratch build/capture/audit scripts removed; verification artifacts stay ignored.
No pack, audio content, generated audit catalog or compiled artifact was staged.

Remaining: unregistered ordinary walls and further camera/body poses; other NPC
and room finishes; source post-answer timing, generic field palette/custom-menu
checks and missing independent item/Mailbox ROM fixtures; reachable naming,
printing/link/mobile/GS Ball services and full all311/original16 visual/service
acceptance. This completes the scoped glazing correction, not the full goal.

PC source YES/NO closing delay and retained drawing — 2026-10-19

Continued the full Kanto/shared-furniture/all311/original16 objective after the
verified radio-glazing correction. AGENTS.md and current implementation were
inspected; its SHA1 remains 7e2e3189e2cfed182daa4c841955e9b8a4b3a5ed. The deleted
handoff remains absent. Shared/unrelated work is preserved and the localized
service edits were coordinated with the active performance goal.

The pinned source home/menu.asm InterpretTwoOptionMenu calls VerticalMenu,
DelayFrames($f), then CloseWindow. Mailbox confirmations already implemented
this wait; Toss and Oak's rating question resolved immediately. Those three PC
source YES/NO owners now share the existing PC joypad wait and result field.
The selected window/cursor stay visible for15 VBlanks; item removal, Oak rating
and fanfare, or shutdown occur only afterward. B returns NO without moving the
displayed cursor; A+B gives B priority. During closing, direct A/B, directions,
Start and Select cannot bypass the phase. The next menu keeps its four-frame
background-upload fence. Renderer-neutral VisibleShellController settling now
crosses this same noninteractive wait before exposing another input boundary.
The existing production confirmation dispatcher remains authoritative.

Drawing readiness is separate from input readiness: inspection caught that
using the input guard to draw the box would hide it during closing. Printed
questions now retain the frame, choice arrow and YES/NO while input is blocked.
The live Oak UI regression verifies retained glyphs, then shutdown repainting.
Its immediate post-direction B fixture now crosses the existing four-frame
source upload fence before pressing B. Reference: docs/art/pc-yes-no-timing.md.

All19 distinct selected checks pass: the new eight-case frame-by-frame
Toss/Oak YES/NO/B/A+B regression; both Oak PC integration/render checks; seven
item PC checks including all22 public-PC lobbies; the existing15-frame Mailbox
control, both Mailbox clinic cases and its native keyboard-release control;
and five Oak source-text/fanfare/acknowledgement/lab checks. The new test retains
exact core state, question and cursor through the first14 frames, blocks all
eight direct controls, and proves mutation/branching only on frame15. Actual
PCScript entry and YES/NO/B decisions pass in every one of the22 lobbies, with
real input release/walking and inventory/party invariants. Audio metadata
checks still pass against the unchanged external pack.

Evidence: target/kanto-resume/pc-yes-no-timing-review/review.json. Three final
native/2D pairs (six full originals) were inspected: Oak's selected question
frozen with15 VBlanks remaining, Toss's selected question at that same actual
closing boundary, and Oak's subsequent shutdown. New read-only closing
observers reach these owners through ordinary input; they never create a
confirmation or choose its result. All paired-state receipts preserve the
session. Question and YES/NO regions are pixel-exact within each pair; Toss's
complete source LCD region is exact. The Oak room stays visible and Toss
intentionally owns the original fullscreen item LCD. No save files were written.

The first replay omitted single-step movement and overshot the PC approach;
its rejected route logs remain. A valid Toss closing capture was initially
rejected by the three-color scenery validator. The existing narrow PC LCD
proof now recognizes the actual frozen Toss closing owner too; general scene
color/contrast and blank-frame checks remain unchanged, and the final capture
passes with three source colors. These calibration failures are not accepted
captures. A broader Oak test run also exposed two unrelated failures, both
reproduced in the preceding test binary: oak_intro_renders_real_oak_wooper_and_player_portraits
and title_new_game_opens_gender_then_oak_clock_intro_before_name_input. Their
baseline logs are retained; this is not a claim of a fully green library suite.

All native captures explicitly use HEADLESS=1/CAPTURE_TARGET=image and mute;
no windows or focus changes. Host/controller code was built against the
completed radio-glazing voxel QA library; no terrain or pack exporter changed
in this turn and no fresh static geometry acceptance is claimed. Pack SHA256
remains a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
Scratch builder/capture scripts removed; verification artifacts remain ignored.
No generated content, audio catalog, pack or compiled artifact was staged.

Remaining: source closing timing for other field YES/NO owners and generic
field palettes/custom menus; the two baseline intro-test failures; independent
item/Mailbox ROM fixtures; unregistered ordinary walls, further camera/body
poses and other NPC/room finishes; reachable naming, printing/link/mobile/GS
Ball services and full all311/original16 visual/service acceptance. The scoped
PC closing behavior is verified; the complete objective remains active.

Independent PC and Mailbox ROM references — 2026-10-19

Resumed the full Kanto/shared-furniture/all311/original16 objective. The preceding
route-only recheck repeated existing summit evidence and did not advance the
interior objective. This batch closes the pending independent PC/Mailbox source
reference gates. AGENTS.md and current implementation were inspected; the deleted
handoff remains absent. Shared and unrelated changes are preserved.

Added tools/capture-pc-menu-oracle.py, an optional isolated QA capture tool.
PyBoy2.8.1 executes the pinned external assembled ROM with a null window, audio
emulation disabled and save writes disabled. Only initial fixture RAM and a CPU
entry boundary are supplied. Original ROM font/palette/screen/menu routines and
ordinary joypad input produce the references; no Geothite renderer or rendered
menu/font/palette pixels are used. ROM, symbols, emulator environment, PNGs,
traces and provenance remain external. This adds no pack builder, audio exporter,
Cargo/runtime/bundle dependency or committed generated content. The tool pins
ROM SHA1 f4cd194bdee0d04ca4eac29e09b8e4e9d818c133 and emulator version, refuses
repository-local output, and records source/symbol/constants/generator hashes.

Generated38 independent source PNGs: initial item list1, item move/select/cancel8,
withdraw quantities4, deposit quantities4, deposit scrolling12 and Mailbox9.
Source cursor/scroll/switch/quantity traces accompany them. The Mailbox test now
uses the existing GEOTHITE_ORACLE_FIXTURES runtime helper instead of a hardcoded
sibling-checkout path. Three existing PC tests gained opt-in CPU LCD PNG output;
comparison assertions and production rendering/gameplay behavior are unchanged.

All31 distinct selected checks pass: pc_item_23, pc_deposit_6, withdrawal-quantity1
and Mailbox1. This includes the formerly missing initial/scroll/move PC gates,
source quantity controls, deposit Cancel clamping/reentry and all9Mailbox frames.
The first deposit trace matched its images but ended on the wrong row for the
existing reentry contract. Its input sequence was corrected to initial +8Down
+3Up; no expected renderer state/assertion was relaxed. Final tests ran only
after capture completed. Early rejected/stale diagnostics remain ignored.

All38 source/production pairs (76 full originals) were inspected, including the
final9Mailbox pairs in small batches after an oversized tool result was
truncated. An independent whole-image check finds zero differing RGB5 pixels
for every pixel of every4x production LCD, with opaque production alpha. This
also verifies the initial list's palette beyond its existing ink-only assertion.
Two completed reference regenerations produce identical hashes for all38PNGs.
Evidence: target/kanto-resume/menu-oracle-review/review.json and comparison.json;
source provenance: external menu-oracle-fixtures/menu-oracle-provenance.json.
The reproducible optional workflow, fixture contracts and exact verification
scope are documented in docs/menu-oracle.md, linked from README and content guide.

No visible windows, focus changes, pack export, audio change or save writes.
Pack SHA256 remains a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007;
the external ROM hash is unchanged. Scratch build/probe scripts removed; QA logs,
images and binaries remain ignored. Nothing was staged. No fresh GPU room,
whole311 geometry or continuous original-ROM playthrough acceptance is claimed.

Remaining: generic field YES/NO closing timing and palettes/custom menus; the
two reproduced baseline intro-test failures; other oracle families; unregistered
ordinary walls and further room/actor poses/finishes; reachable naming,
printing/link/mobile/GS Ball services and complete all311/original16 shared
furniture/visual/service acceptance. The full goal remains active.

Mounted classroom blackboard and source-sized ledge — 2026-10-19

Continued the full Kanto/shared-furniture/all311/original16 objective after the
independent PC/Mailbox reference gates were completed. That turn made concrete
acceptance progress. AGENTS.md and current source were inspected; the deleted
handoff stays absent and unrelated/shared changes are preserved. Localized work
was coordinated with the active performance goal; no competing exporter ran.

Current headless front/east native comparisons showed Earl's blackboard still
painted flush onto the wall. native_classroom.rs now gives its actual48x16-source-
pixel drawing a shallow mounted body, closed sides/back and a lower ledge.
All twelve board source cells keep their original frame and chalk lettering on
a front plane0.75source pixels forward; other wall cells retain their plane.
The ledge projects2source pixels and is0.5pixel thick, using the source charcoal
frame swatch. The32pixel ceiling height, joined plain side returns, furniture
layout, source ownership and floor support are preserved. This exact complete
classroom drawing remains map/source/custom-owner guarded. No generic wall or
other classroom furniture model changes. Guide: docs/art/classroom-furniture.md.

All18 distinct selected checks pass in a private current-source voxel test
build: classroom2, seating11, opt-in seated-body/footing2 and dining-table3.
The external-pack classroom check verifies all eight desks/sixteen chairs/the
lectern/rear course, every source face UV and its actual wall/board depth,
outward winding, unchanged support, edited/cropped/foreign-map rejection and
custom ownership. Existing chair direction and seated body controls pass.

Fresh production-host paired captures cover front(4,7), east(4,3), and west(4,3).
All six final native/source originals and four before originals were individually
inspected. Every final paired-state receipt preserves authoritative state;
front/east before-versus-after2D images are pixel-identical. The board now has
readable shallow edges and a ledge while its source lettering stays intact;
chairs continue to face desks and seated bodies remain supported. The low side
views crop some entrance geometry, so they do not establish whole-room acceptance.
Every capture is explicitly HEADLESS=1/CAPTURE_TARGET=image and mute, with no
windows/focus changes. An initial comma-separated camera request was rejected
before capture; the corrected whitespace request produced the accepted pair.

Fresh full audit completes388maps and288script-state variants with no mesher
errors. It includes all311maps/225variants from radio-glazing-build/full311.json.
Comparison by map identity finds only Earl's textured geometry changed:
18860->18882triangles,45264->45308vertices. All other310map records are exact;
all nongeometry fields for Earl and all225original state records are exact.
Actor-source/species/format/limitations sections are also exact. The additional
77maps/63variants are newly audited, not a claim of prior-to-current geometry
identity or visual acceptance. Evidence: school-board-build/full388.json and
audit-delta.json, school-room-review/review.json. Private host/voxel builds use
opt-level1 QA libraries; no production performance certification is inferred.

Pack SHA256 remains a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
No pack, audio, gameplay, save or external source content changed. No generated
content/artifact was staged; build/capture/audit output remains ignored and no
scratch script remains. git diff --check passes.

Remaining: broader classroom interactions/room composition; remaining room and
actor finishes/poses, ordinary-wall occlusion, generic field YES/NO timing and
palettes/custom menus, the two baseline intro-test failures, other independent
oracle families, reachable naming/printing/link/mobile/GS Ball services and full
all311/original16 shared-furniture/visual/service acceptance. The goal is active.

Name Rater normal entry, source pages and shared editing — 2026-10-19

Continued the full Kanto/shared-furniture/all311/original16 objective after the
verified classroom finish. That preceding turn made concrete model/visual
progress. AGENTS.md and current implementation were inspected. The deleted
handoff remains absent; shared/unrelated work is preserved and service changes
were coordinated with the active performance goal. No exporter ran.

A new real-input regression initially failed when talking to Lavender's actual
NPC: run_compiled_script_until_boundary executed LavenderNameRater command2,
special NameRater, before any party selection, reporting missing _party_slot.
Existing tests opened the interceptor directly and missed normal entry. The
runtime now exposes NameRaterPrompt before that opcode, leaving its existing
VisibleShellController interceptor authoritative. It stops even when a prior
service left selection variables, so revisits cannot silently reuse them.

Further real-input review found that pending choices could consume A/B while
introduction/better-name pages were still active, and that the editor prefilled
the old nickname instead of the original NamingScreen_InitNameEntry's empty
entry. Those source pages now retain A/B until acknowledged. Only the printed
question enters the existing source YES/NO sampler, border layout and shared
15-VBlank closing phase. Naming starts empty; blank/unchanged/new names preserve
their separate original completion pages. Egg/foreign-OT refusals stay separate.

The renderer-neutral controller also lacked naming-grid dispatch. Its ordinary
direction/A/B/Start buttons did not edit the screen. It now adapts all eight
buttons to the existing native apply_visible_name_input_keys handler. Cursor,
case, deletion, END and nickname completion remain owned by that same handler;
no second frontend editor/dispatcher or authoritative renderer cursor was added.

Native review then caught the generic heading touching the right border.
Presentation now uses the existing selected-species two-line heading in both
LCD and text observation, preserving the logical completion owner. General
heading placement keeps both border columns intact and preserves the old
short-label offset. Guide: docs/art/name-rater-services.md.

All13 distinct selected checks pass against current production code: seven
Name Rater checks, the Mom/New Bark controller gate, existing eight-case PC
15-frame control, live Oak question repaint, and three naming asset/layout/
opaque-LCD controls. The new production-controller regression proves26 cases
across Lavender and Goldenrod plus two repeat visits: renaming, blank/same-name
submission, ten-character limit, B deletion/Select case, egg, foreign ID/name,
NO/B at both questions, party cancel, exact storage/inventory/money/Pokedex
invariants, walking and Start-menu return. Only initial room/party data are
fixtures; no post-start service cursor, choice, nickname or script variable is
seeded. Source heading conversion preserves the logical input and checksum.

Both final native/source naming pairs (four full originals) were individually
inspected. Ordinary DU movement and released A pulses reach the actual editor;
the read-only service:name-rater-entry observer freezes its existing owner.
Each receipt records LEN0/10, the actual species heading and preserved session.
Complete1280x800 RGBA images are byte-exact within each pair; both strict visible
frame validators pass with four colors. A Lavender baseline room pair and the
earlier clipped-heading pair were also inspected as diagnosis, not whole-room
acceptance. All captures explicitly use HEADLESS=1/CAPTURE_TARGET=image and
mute; no visible windows or focus changes. Native transitions still emit
deferred UI entity-teardown warnings; their cleanup remains open.

The asset-render regression initially tried the missing sibling raw-assets path
/Users/ryanculligan/GitHub/apps/web/assets/gfx/sgb/predef.pal. It now materializes
the supplied verified pack's runtime assets, and two naming test roots correctly
point to this repository. Assertions are unchanged. Early route/input diagnostics
are rejected: a wrong capture input variable, held A without release, a bump-only
route lacking required movement, and a revisit that turned before walking back.
Their corrected final routes prove real movement. An obsolete cursor-loop test
was sampled and stopped once its missing input handler was identified; cursor
loops are now bounded. A private example's missing dependency search path was
also repaired. None of those failures counts as an accepted render/test.

Evidence: target/kanto-resume/name-rater-review/review.json and the individual
*-accepted/-final logs in name-rater-build. Private runtime/host builds use
opt-level1 QA libraries and the unchanged school-board voxel library; no new
terrain or static geometry acceptance is claimed. Pack SHA256 remains
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007. No pack/audio
content, user save or compiled artifact was staged. Only ignored disposable
fixture saves were written; backups, obsolete moving captures and scratch
sampling output were removed. git diff --check passes.

Remaining: naming-screen independent ROM pixel comparison, Pokemon icon and
animation, deferred UI teardown warnings, other party-selection specials and
printing/link/mobile/GS Ball services; generic field YES/NO timing/palettes,
the two baseline intro-test failures, ordinary-wall/actor/room finishes and
complete all311/original16 shared-furniture/visual/service acceptance. Name Rater
normal entry/editing is scoped verified; the full goal remains active.

Fullscreen presenter cleanup and field return — 2026-10-19

Continued the full Kanto/shared-furniture/all311/original16 objective. The
preceding route-only recheck repeated existing evidence and did not advance
interior acceptance. AGENTS.md and current source were inspected. The deleted
handoff remains absent; unrelated changes and the completed pack are preserved.

Native Name Rater backtraces identified two ownership removals queued against
a fullscreen presenter already retired earlier in render_playfield. The intro
query still contained that entity until deferred commands applied. Retirement
now records its ID in the render pass's existing queued-despawn set; later intro
cleanup skips those IDs. Live ownership cleanup executes as one EntityCommand
with the configured error handler. No warning suppression or try_remove was
introduced. An experimental strict-capture fallback option was removed because
Bevy's original component removals explicitly selected the warning handler.

The new real CommandQueue regression fails before the retired-ID guard and
passes afterward, testing both retained and cleared live intro ownership.
Production-renderer phase fixtures also verify party/naming/Name Rater returns,
actual presenter removal, reuse of the LCD image, preservation of both field
layers and absence of shell errors. These fixtures do not replace real-input
service reachability tests from the preceding checkpoint.

Seven distinct selected checks pass: the two new lifecycle checks, complete
naming LCD, heading layout, Name Rater presentation, live party/stats retention,
and cold fullscreen-to-field handoff. The cold fixture initially failed in both
current and prior binaries because it cleared intro/title but left the newer
boot splash active. It now clears that splash as part of its explicit phase
fixture; its map-layer and delayed-retirement assertions remain unchanged.
The two separately recorded baseline intro failures remain open.

Both headless native Name Rater transitions complete without Entity despawned
or Encountered an error in command messages. All four final native/source
originals were inspected. Complete RGBA bytes match within each pair and the
previous accepted captures; both paired receipts preserve the actual session.
Other empty-mesh/optional-renderer warnings remain. No windows or focus changes
were used. Evidence: target/kanto-resume/lcd-lifecycle-review/review.json and
lcd-lifecycle-build logs, including atomic-before.log, strict-native-before.log,
the selected final logs and cold-handoff-corrected.log. Private QA builds are
not a full Cargo/release or full-library acceptance claim.

Guide: docs/art/name-rater-services.md. Pack SHA256 remains
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007. No exporter,
terrain/model, audio or user-save changes were needed. Disposable moving
captures were removed; accepted originals and diagnostic logs remain ignored.

Remaining: independent naming ROM pixels/icon animation, other party specials,
printing/link/mobile/GS Ball services, generic field YES/NO timing/palettes,
baseline intro failures and the full all311/original16 room/furniture/service
acceptance. This closes the specific stale-presenter defect; the goal is active.

Independent nickname pixels, source icon/gender and idle animation — 2026-10-19

Continued the full Kanto/shared-furniture/all311/original16 objective after the
fullscreen cleanup was verified and recorded. That turn made concrete lifecycle
progress. AGENTS.md/current implementation were inspected. The deleted handoff
stays absent; shared changes were coordinated with the active performance goal,
and unrelated work is preserved. No exporter ran.

Extended the existing optional headless ROM reference tool with --family naming.
The pinned external ROM's actual NamingScreen draws male/female Cyndaquil and
genderless Magnemite from initial party/DV fixtures; ordinary joypad input then
enters/deletes letters, changes case and selects END. No Geothite pixels or
screen data are supplied. The corpus has 120 source PNGs: 32 idle and 8 editing
frames per case. A second generation repeats every PNG hash. All 38 existing
PC/Mailbox reference PNG hashes remain unchanged. ROM/references/traces stay
external. See docs/menu-oracle.md and docs/art/name-rater-services.md.

Independent comparison exposed the missing Pokémon icon/gender marker and a
static naming cursor. Name Rater presentation now uses its actual selected
species' pack icon mapping, existing party icon decoder/palette and source LCD
position (16,12). Gender combines Attack and Speed DVs and uses the source's
inclusive ratio threshold, with fixed-gender/genderless rules. Icon frames last
18 VBlanks; cursor visibility alternates every 4. These bounded phases enter the
naming art/cache key without changing the logical service owner or shared input
editor. Temporary icon-decoder images are removed after composition.

An actual idle-render check then exposed the snapshot-revision fast path keeping
the new phases static. Native clock advancement now marks presentation dirty
only when naming crosses a cursor/icon phase boundary, including catch-up
ranges. The final regression runs 74 ordinary clock updates without keys and
observes all 4 actual LCD combinations, one retained entity/texture and 4 cached
compositions. Party/bag/money/Pokedex/location remain unchanged. Initial fixture
assertions incorrectly expected the entire checksum to survive the original
VBlank GameTimer; that test was corrected to distinguish timer advancement from
gameplay mutations. The pixel comparison's initially detected 32-pixel cursor
mismatches and 94-pixel END mismatch remain documented in diagnostic logs.

All 21 distinct selected current-source checks pass: 9 Name Rater, 2 fullscreen
cleanup/return, complete naming LCD, party/stats, cold handoff, 3 player naming,
Pokemon heading, Mom/New Bark, live Oak and source PC 15-frame controls. The real
service regression retains 26 cases and 2 revisits. All 120 complete CPU LCD
comparisons are RGB5-exact at matching source icon/cursor phases, without masked
regions. Three source/production comparison sheets cover 4 idle variants and all 8
editing states per case and were inspected.

Both final headless native/source pairs (four originals) were individually
inspected. Each whole pair is RGBA-byte-identical and its receipt preserves the
actual session. Each native LCD also matches the independent ROM at every pixel
of its 5× expansion: Lavender phase 3, Goldenrod phase 15. Both production screenshot
validators pass 1280×800 / 6 colors; both input receipts prove 449 frames, 2 grid origins
and 0 additional terrain builds. Neither transition emits stale-entity errors;
other optional-renderer/empty-mesh warnings remain. No visible window or focus
change was used. Evidence: target/kanto-resume/naming-pixels-review/review.json,
comparison.json, naming-pixels-native/ and naming-pixels-build logs/commands.

Private host/runtime QA libraries are not a full Cargo/release-performance claim.
The voxel library and pack are unchanged; pack SHA256 remains
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007. No model/audio,
content export or user-save changes were needed. Disposable moving captures were
removed; selected originals, comparisons and logs remain ignored. git diff
--check passes. No generated content or compiled product was staged.

Remaining: exact source naming entry-clock/joypad-subframe synchronization,
other naming owners/species and party specials, printing/link/mobile/GS Ball,
generic field YES/NO timing/palettes, previously recorded intro issues not
re-audited here, ordinary-wall/actor/room finishes and full all311/original16
visual/furniture/service acceptance. The full goal remains active.

Cerulean raised walkway source material — 2026-10-19

Continued the full Kanto/shared-furniture/all311/original16 objective after the
independent Name Rater pixel/icon work was completed. That turn made concrete
production and acceptance progress. AGENTS.md/current source were inspected;
the deleted handoff remains absent. Shared files were coordinated with the
active performance goal and unrelated changes preserved. No exporter ran.

Fresh headless middle and entrance comparisons confirmed CeruleanGym's pink
deck still used generic color triangles, losing the original brick print and
introducing diagonal facet patches. dungeon_extension.rs now partitions only
exposed upward deck/grout caps into the textured buffer with live source UVs
and uniform source shading. Every original triangle/normal, shallow relief,
suspended body, post and footing remains. There is no additional coplanar sheet.
The same finish under the two complete guardians borrows their authenticated
adjacent deck donor. The existing entry grade transforms both material buffers
so the cap follows its body; source admission and custom owners are unchanged.

All 34 distinct selected checks pass: eight Cerulean, ten elevated-pier, thirteen
seating and three dining-table controls. The new regression compares complete
triangle/normal sets before and after material partition at two scales and all
four source rows, checks live UV placement/uniform shading, and preserves
footing/claims. The guardian control retains source-mutation and support gates.
An exact [0,1,0] assertion initially rejected an existing normalized float
[0,0.99999994,0]; its upward-direction tolerance was corrected while the separate
bit-exact original triangle/normal comparison remains unchanged.

Four final native/source pairs (eight originals) were individually inspected:
middle, entrance, east-side view and ordinary UUDD input. Both before pairs
(four originals) and two actual moving originals were inspected as well. All
six paired receipts preserve the actual session. Complete before/after classic
RGBA images are byte-identical at middle and entrance. The walk records 128
frames, nine authoritative tile positions and six viewport origins with zero
additional terrain builds. Entry support and the deck beneath guardians remain
coherent. Every capture explicitly uses HEADLESS=1/CAPTURE_TARGET=image and
mute; no window or focus change was used. See docs/art/cerulean-walkways.md.

The final full audit passes 388 maps and 288 scripted states with zero mesher
errors, including all 120 Kanto interiors and 16 states. Only CeruleanGym's
four material-buffer counts differ from the preceding full audit: 2,560 triangles
and 7,680 vertices move from solid to textured. Its total 27,088 triangles and
75,408 vertices are unchanged. Source/support/claim/family fields, all other
387 map records and all 288 scripted-state records match exactly. Evidence:
target/kanto-resume/cerulean-deck-build/full388.json and audit-delta.json;
cerulean-deck-review/review.json, original images and paired/movement receipts.

The first example compile lacked the newer dependency search paths, and the
first voxel-test invocation omitted OUT_DIR. Both private build commands were
corrected using existing generated caches (all 613 model metadata files agree
between the two used caches). No source/model data was regenerated. The first
full audit hit its 240-second wrapper limit and was confirmed terminal; that
failure is retained. Its completed replacement uses the current runtime and a
600-second limit without concurrent captures. These opt0 test/opt1 native QA
builds are not a full Cargo/release performance claim. No build is left running.

Pack SHA256 remains
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007. No pack/audio,
model asset, gameplay dispatcher, collision, user save or exported content was
changed or staged. Unreviewed disposable moving images were removed; selected
verification evidence stays ignored. git diff --check passes. The current bridge
queue row now distinguishes accepted caps from its remaining timber-face work.

Remaining: Cerulean's vertical deck/timber finishes, water/wall/guardian/actor
and room/quest acceptance; other rooms/shared furniture and full all311/original16
visual/service acceptance; remaining naming owners/timing, party specials,
printing/link/mobile/GS Ball and generic field YES/NO work. The full goal stays
active; this closes the demonstrated deck-cap material defect only.

Cerulean gray-bank identity and live timber paint — 2026-10-19

Continued the full Kanto/shared-furniture/all311/original16 objective after the
raised-deck cap repair passed its visual and full-audit gates. That preceding
turn made concrete production/acceptance progress. AGENTS.md and current source
were inspected; the deleted handoff stays absent. Shared files were coordinated
with the active performance goal and unrelated work preserved. No exporter ran.

Primary source/pack review corrected a real identity error in the remaining
queue: port tile0b is flat GRAY bank/shadow art, not timber. The actual wooden
support drawing is tile22 with the BROWN palette. The prior “foam22” wording was
also incorrect for this tileset. A new external-pack regression verifies both
palette IDs, the flat bank bitmap and the actual beam/post/highlight/underside
source samples. No graphic/catalog copy or source-program export was added.

Sixteen former CeruleanBridgeFace bindings now use CeruleanBankFace and the
special:cerulean-gray-bank label. Their continuous closed body keeps its original
bounds and samples its own gray source; two invented timber posts per bank cell
are removed. Exact column guards distinguish real bank art from the gray rim.
The existing authenticated water backing, source ownership and footing remain.

Real deck bodies, posts and tile22 edge frames now use live source swatches:
beam sides, lighter post fronts, upper highlights and undersides remain distinct.
Thin brick risers use their own tile05 pink while the accepted cap print/UVs stay
unchanged. A timber donor must be an actual support drawing below a genuine
deck; gray banks, unrelated port art, corrupt fields and missing decks reject it.
Missing donors retain the prior body paint. Other maps keep their pier treatment.
The same finish reaches the deck beneath both complete guardians. Existing grade
transforms apply to both material buffers; deck/beam/post geometry and normals,
footing, collision, claims and walking joins are preserved.

All 39 distinct focused checks pass in 43 executions: thirteen Cerulean, fourteen
elevated-pier, thirteen seating and three dining-table executions overlap on four
new timber checks. Controls cover source identity, donor mutations, real edge
consumption, exact beam/post geometry, preserved brick UVs and existing support.
The original external piling/ramp test initially queried only the solid buffer.
It now verifies Cerulean's actual textured timber, requiring vertical pilings
below -5.9 source pixels and a raised beam face. Exact ramp-height tolerance and
all other routes' original assertions remain. Initial failures are retained.

Five final native/source pairs (ten full originals) were individually inspected:
middle, entrance, east, north bank and ordinary UUDD input. Two actual moving
native originals were also inspected. All paired receipts preserve their actual
held session. Middle and entrance classic before/after images are full-RGBA
byte-exact. The east before/after differs in 13,968 pixels, exclusively source
blue/cream water-animation swaps; no whole-image equality is claimed there.
Its current pair still holds the same actual state. The walk records 128 frames,
six viewport origins and zero additional terrain builds. All captures explicitly
use HEADLESS=1/CAPTURE_TARGET=image and mute; no window or focus change was used.
Guide/current queue: docs/art/cerulean-walkways.md.

The final full audit passes 388 maps / 288 scripted states, including Kanto120/16,
with zero mesher errors. Only CeruleanGym's record changes: the intentional
16-cell bank identity rename and material/geometry counts. Removing 32 invented
bank posts removes 768 triangles / 2,304 vertices, leaving 26,320 triangles /
73,104 vertices. After normalizing that one bank label, all source/support/
coverage/family fields are exact. Other 387 map records and all 288 scripted-state
records match the preceding accepted cap-stage audit. Evidence:
target/kanto-resume/cerulean-timber-build/full388.json and audit-delta.json;
cerulean-timber-review/review.json, original captures and paired/walk receipts.

The first builds encountered the other goal's new VisualWorldFrame.view_center
and anchor_to_map API with an older cached render-api rlib. Both failed attempts
were terminal. Current crystal-render-api was rebuilt privately and all dependent
arguments refreshed; the shared renderer changes were preserved. The final QA
libraries/examples and tests use that current API. These opt0 test/opt1 native
builds are not a full Cargo/release performance claim. All jobs are terminal.

Pack SHA256 remains
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007. No pack/audio,
model asset, gameplay dispatcher, user save or source content was regenerated
or staged. Unreviewed disposable moving captures were removed; selected ignored
evidence remains. git diff --check passes. The queue now names gray banks and
real timber correctly instead of carrying the old interpretation forward.

Remaining: Cerulean water/wall/guardian/actor/quest and whole-room acceptance;
other rooms/shared furniture and complete all311/original16 visual/service
acceptance; other naming owners/timing, party specials, printing/link/mobile/
GS Ball and generic field YES/NO work. These demonstrated identity/material
repairs are verified; the full goal remains active.

Oak/Elm workstation reconstruction and missing starter balls — 2026-10-19

The previous summit turn repeated completed route coverage and did not advance
interior work. Revalidated AGENTS.md, current source and the active queue; the
deleted handoff remains absent. Coordinated shared files with the performance
goal. Preserved unrelated renderer, camera and actor work; no exporter ran.

Fresh paired source reviews showed a concrete workstation form defect in both
labs: one wide generic monitor replaced a compact left CRT, separate centre
unit and tall right enclosure. A source-authenticated Rust assembly now supplies
all three closed chamfered bodies, shallow controls and keyboard, on a low gold
workbench. Existing 32px source width / 12px depth / 22px maximum height remain;
worktop is 8.5px, left CRT 19px, centre 16.5px and right enclosure 22px. All paint
samples the live atlas. The selected material contract, complete twelve-cell
source guards, custom ownership, floor and support remain. The generic computer
asset and mansion use are unchanged. No generated model artifact was added.

Broader Elm testing reproduced the user's missing starter balls: the source
showed three while both modeled props and fallback cards explicitly suppressed
all Elm PokeBall objects. Removed that unconditional room-name suppression.
Ordinary source actor visibility and existing real-table support now render all
three. The existing actor regression failed with zero props before the repair
and passes afterward, checks all three at the actual 10.04px root, and verifies
that removing one authoritative actor retires its model. No static decorative
balls, game-state edits, collision or script-dispatch changes were introduced.

Ten distinct focused checks pass in eleven final executions: workstation five,
Elm five (one overlap), ordinary item-ball one. A supplied-coat test remains
explicitly ignored for its external rig. Closed shells/outward winding and
bounds pass at three scales; immutable generic geometry, source mutations,
custom profiles, claims, floor/support and actual pack placement are covered.
An initial unqualified pack-path test failed canonical-path validation; the
same selection passes with the absolute CRYSTAL_RENDER_TEST_PACK path.

The full terrain audit passes 388 maps / 288 scripted states with no errors.
Only Oak and Elm geometry counts change, each -1,332 textured triangles and
-3,996 vertices. All source/family/support fields, the other 386 map records
and all scripted-state records are exact. This terrain audit preceded the actor
suppression removal; it does not prove actor visibility. That repair has the
separate failing-then-passing regression and two final production visual pairs.

Four new native/source pairs (eight full originals) were individually inspected:
Oak and Elm workstations, then final restored Elm balls from the default view
and an opposite side approach. Every paired receipt preserves its held actual
session. Before/after captures have different idle NPC phases, so whole-image
pixel identity is not claimed. The side approach retains the existing table
reveal around the player; the three balls remain visible at tabletop height.
All captures use HEADLESS=1, CAPTURE_TARGET=image and mute; no windows opened.
Guide: docs/art/lab-workstations.md. Evidence:
target/kanto-resume/lab-assembly-build/{full388.json,audit-delta.json,*tests*.log};
target/kanto-resume/lab-followup-review/review.json and original paired captures.

QA rebuilt actual Rust voxel/host/render sources against the completed local
dependency graph (tests opt0, native opt1), preserving concurrent camera changes.
This is not a full Cargo/release performance or installed-app update claim.
Pack SHA256 remains
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
No pack, audio, user save, source content or compiled deliverable was staged.
Disposable analysis crop removed; selected evidence remains ignored. Jobs are
terminal and git diff --check passes.

Remaining: lab wall/window and whole-room/actor/service acceptance; other rooms,
shared furniture and full all311/original16 visual/service acceptance; other
naming owners/timing, party specials, printing/link/mobile/GS Ball and generic
field YES/NO work. The full goal remains active.

Oak/Elm native wall, window and perimeter finish — 2026-10-19

Continued after the verified three-unit workstation and restored Elm starter
balls, which made concrete production/acceptance progress. Read AGENTS.md and
current implementation/queue; the deleted handoff stays absent. Coordinated
shared files with the performance goal and preserved unrelated actor/camera/API
changes. No exporter ran and no window or application focus was changed.

Fresh source comparison showed broad generic clinical blue panels and aqua
crossbar windows in place of the actual blue checker course and two pane
patterns. Native Rust now authenticates both labs' complete wall08/0d/17 and
window09/20 drawings, including both two-column panes and the complete four-
column profile. A closed 32px wall retains one 16px source course near its crown,
with a plain 16px cream course underneath. The actual panes keep source-sized
14×9px framed faces, 0.4px relief and finished edges. The front is partitioned
rather than overlaid. Atlas/block/phase/tile, crop, raised/custom ownership and
foreign-map guards remain strict; other lab-tileset rooms keep their treatment.

The same source course continues behind authenticated rear bookcases/pictures
and around both side/front returns. Freestanding shelves receive no backing.
Initial native review caught the generic perimeter stretching a four-column
donor into two-column sections. The lab now supplies a two-column donor; short
aperture/corner runs crop its continuous source interval instead of squeezing
the checker. World-to-atlas pitch is pinned at 3px, 11px and 16px section widths.
Walls remain ceiling height and contiguous; existing warp openings, cutaway
behavior, source/family ownership, floor and support remain authoritative.

Sixteen distinct focused checks pass in seventeen final executions: source/
geometry two, all 255 perimeter donors one, Elm five, cabinet/TV backing one,
workstation five and Vermilion three (one shared Elm/workstation test overlaps).
A supplied-coat test remains explicitly ignored for its external rig. Tests
cover every source-field mutation/custom owner, outward surfaces and bounds at
three scales, one complete front, frame relief, native checker pitch, rear
backing and existing workstation/three-ball support and source visibility.

The broad perimeter control initially failed identically with the preceding
lab-assembly executable: its old zero-source-ownership assumption contradicts
Vermilion's already-authored stepped header. The corrected test requires the
exact 46 cap/return cells, no void/floor claims, and exactly 32 permanent checker
floor quads at the actual inset threshold. Every other wall vertex must remain
in a cutaway section. The existing three dedicated topology/source/shifted-
origin controls pass. No Vermilion production code changed; its audit record is
identical. Initial failures and baseline reproduction remain diagnostic evidence.

All three final native/source pairs (six full originals) were individually
inspected: Elm (4,5), Oak (4,6), opposite-side Elm approach (7,4). Each pair holds
the same actual session. The source pane patterns, source-colored trim, full
wall joins and native checker scale are readable; the three starter balls and
accepted workstation remain intact. Initial three pairs were also inspected
before the pitch repair. Different idle NPC phases prevent a whole-image
before/after identity claim. All captures explicitly use HEADLESS=1,
CAPTURE_TARGET=image and mute. Guide: docs/art/lab-workstations.md.

The final audit passes 388 maps / 288 scripted states, no mesher errors. Only
Oak and Elm geometry counts change: Elm -8,476 triangles/-18,616 vertices and
Oak -7,424 triangles/-16,304 vertices. Their source coverage/family bounds and
support fields are exact; the other 386 map records, all 288 scripted-state
records and every other top-level audit field are identical to the accepted
lab-assembly baseline. Evidence: target/kanto-resume/lab-walls-build/
{full388.json,audit-delta.json,*final.log}; lab-walls-review/review.json and
final original paired captures/receipts. The first wall-stage audit is retained
as full388-initial.json; the final audit was rerun after the verified pitch fix.

Actual Rust voxel/host/test/example sources were rebuilt against the completed
local dependency graph: tests opt0, native opt1. No full Cargo/release/FPS or
installed-app update is claimed. Final jobs are terminal; git diff --check and
new-module rustfmt check pass. The pack checksum remains
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007.
No pack/audio/model asset, gameplay state, user save or compiled deliverable was
regenerated or staged. Selected verification evidence stays ignored.

Remaining: whole-lab actor/service and broader room acceptance; remaining rooms,
shared furniture and full all311/original16 visual/service acceptance; other
naming owners/timing, party specials, printing/link/mobile/GS Ball and generic
field YES/NO work. The full goal remains active.
