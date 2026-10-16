# Remaining source-family review

This is an implementation and visual-review checklist. Counts are residual
8-pixel source cells from inspected base maps, not objects, triangles or a
percentage of finished scenes. Plot counts can include backing ground. Dynamic
map states and individual species art remain separate work in the broader
conversion checklist.

## Game Corner, Gym, store and tower expansion

Focused Rust mesh regressions passed on the Bevy 0.20 verification executable:
`store_basement`, `cerulean_bridge`, `enhanced_room_floor_tests`, `vermilion_`,
`sprout`, `blackthorn`, `mahogany`, `fuchsia_wall`,
`source_phase_rejects_rocks` and `entrance_carpet`. The executable includes the
latest Sprout carpet exclusion and statue, Olivine floor, Fuchsia wall and
Blackthorn channel tests. All selected tests passed; filter counts overlap.
This resolves the focused mesh-test execution items below. Pack-backed native
consumption audits, rendered room appearance and puzzle/animation playthrough
checks remain separate requirements; these unit results do not complete them.

The first targeted pass adds 22 store stair drawings (88 source cells) and all
eight Sprout Tower ladder drawings (32 source cells). Store source checks cover
both stores on 1F–6F; the tower checks include northeast corner variants. Native
keyboard routes verify Goldenrod 1F→2F and Sprout Tower 1F→2F. Room-floor and
intentional tower-depth finishes preserve support and earn no object credit.

The requested complete-room conversion remains active.

The current production audit example passed a pack-backed base-state audit of
all 39 named rooms with zero mesher errors and finite footing for every source
cell. It was linked directly against the matching finalized native libraries;
the temporary executable and report were removed afterward. This confirms
the earlier temporary harness result with the ordinary production shell's
tile-bank resolver. It consumed all 648 basement fixture cells,
Cerulean's 160 bridge cells, Vermilion's 64 north-panel and 16 guardian cells,
all three 24-cell Sprout pillars, Mahogany's 80 wall and 72 partition cells,
Blackthorn 1F's 222 lava, 62 lip, eight stair and eight boulder cells, Fuchsia's
40 wall cells and Olivine's 216 boulder cells. Native floor finishes are not
object coverage. Violet still has 40 facade cells; Sprout's residual timber
and altar architecture still require review. Source classification is not
proof that every residual is intentional. This audit does not certify actual
palette, visual quality, animation or gameplay, and it does not close the goal.
The fresh Goldenrod Game Corner window capture was correctly rejected by the
production tool as a blank/uniform frame (one color, zero luma range). It is
not visual approval or evidence that the room renders correctly. The engine
upgrade owner independently reproduced black window captures in a minimal
Bevy app while explicit image-target captures worked; room-specific offscreen
visual verification therefore uses the explicit image target mode.
An explicit one-shot image-target verification mode now preserves the same
production camera targets' physical dimensions, viewport and scale. The
matching scale must be retained in the screenshot's normalized target as
well; using the scale-1 convenience constructor with a scale-2 camera was
reproduced as a blank readback and corrected. An actual GPU test of the
production capture module passes at 640×576 with scale 2, correct projected
color samples and the original short warmup. Three target-ownership tests
pass, covering viewport/order/scale preservation, resize isolation and the
unchanged ordinary window mode. Temporary test binaries and images were
removed. The exact production example subsequently captured paired source and
3D views of Goldenrod/Celadon Game Corner, the prize room, Goldenrod store 1F,
Cerulean, Cianwood, Violet and Olivine Gyms, and Sprout Tower 3F. All nine image
readbacks passed the production visible-frame validator and were inspected.
This confirms those camera/render paths; it does not certify gameplay or every
room. Review found stools too wide, overly heavy arcade floor borders, lost
blue in Violet's floor, teal water replacing Cerulean's blue pool, and remaining
flat Olivine stage/posts and Sprout altar courses. The next refinement reduces
arcade stool diameter from ten to eight source pixels without changing its
seven-pixel height or footing, and gives Violet's exact verified floor cells a
blue brick finish. Arcade floor borders are thinner and their pale/gray
diagonal inset motif is restored from the native design. The external pack
floor/mutation checks pass on fifteen
maps; fresh native captures of these refinements remain required.

The remaining Violet north facade and Sprout 3F framed altar now have closed
wall bodies, cap/base trims and their actual packed artwork on the front faces.
Source guards verify all 40 Violet wall cells and eight altar cells against the
external pack, rejecting changed art and custom namespaces. New renderer tests
require complete drawings, unchanged actor footing, physical back/side faces,
and rejection of incomplete crops and other maps. Execution and fresh native
review are still required. Cerulean's source-proven water keeps the existing
geometry/animation path with a map-scoped blue finish and blue highlights;
other maps/materials retain their palette. Its scope regression is also pending
execution. None of these source checks alone closes the visual review.

The remaining thirty paired baseline captures also completed successfully,
so all 39 named rooms now have an inspected source/3D comparison. These are
baseline images from the finalized library set before the latest refinements,
not approval of current source. Review confirms broad desaturation, partial
flat/finished floor seams in Fuchsia, lost blue in Blackthorn 2F and Viridian,
plain roof/elevator finishes, remaining Sprout timber cards, and store plants
reading as shelving. Camera framing crops some entrance fixtures; they require
alternate positions. The 5F counter's reveal transparency also requires review.
The user's latest feedback accepts sizing/variety and prioritizes color and
texture. Keep those dimensions stable. Exact tower floor identities now use
richer amber timber and partitioned, coplanar grain bands; source-scoped garden
floors and rose ceramic have stronger green/pink colors. Store ceramic now
has small coplanar pigment variations, and the verified store upholstery uses
a richer rose color without changing its geometry. External-pack floor
and mutation checks pass on fifteen maps after these palette edits. New captures
and the existing floor regressions remain required for this material pass.
The preceding four new mesh regressions passed in the freshly compiled test
binary, along with all nine enhanced-floor tests and the terrain geometry/
footing preservation regression. The subsequent amber/green/pink compile also passes all nine enhanced-floor
regressions and five Gym-floor regressions. Store pigment/rose upholstery
changes came afterward and still require fresh compilation/review. Blackthorn
2F had a separate fingerprinted special-room stone binding overriding its blue
floor finish. That exact binding now uses a blue brick surface; the existing
floor-area regression also covers its coplanar mortar/brick partition. The
fresh compiled test executable passes that exact-area regression, its actor
datum regression, and the new complete wall/altar source regression. Native
review still remains required for the latest material changes.

The final production examples have now rebuilt successfully after these
material changes. A fresh ownership audit verifies all 39 base room meshes,
zero mesher errors and one footing entry per source cell. Violet consumes 40
new north-wall cells (280 modeled total); Sprout 3F consumes eight altar cells
(164 modeled total); Blackthorn 2F's exact blue-brick binding consumes 476 floor
cells within its existing 720 modeled cells. The report was temporary and removed.
Fresh full-room GPU review initially stopped explicitly when the strict character
loader required the absent canonical `bruno.png` painting. The character owner
completed the required set; all 75 paintings are present, and the same production
capture then completed eight paired rooms without a texture fallback. The
comparisons in `target/room-material-review/` have been inspected: Goldenrod 1F,
Goldenrod Game Corner, Goldenrod Gym, Sprout 3F, Violet, Cerulean, Blackthorn 2F
and Cianwood. Blue floors/pool, framed wall/altar volume and stronger timber are
visible; furnishings and timber walls still appear too pale and flat. The broader
complete-room goal stays open.

The next narrowly scoped material pass restores honey-gold store ceramic and
warm orange grout. Verified store tile and amber timber faces carry explicit
indoor paint coordinates, outside atlas/water/fountain/outdoor/cave ranges.
The shader applies stationary glazed pigment/speckles or continuous wood grain,
with derivative filtering on the grain to reduce shimmer. Tile grout and walking
geometry retain their original partitions. Ordinary oak and unrelated material
UVs are untouched. The world-phase/exact-floor-area regression passes. Both store 1F and Sprout 3F
captured successfully through the production GPU shader and were inspected.
The store reads warmer; the first wood grain was too fine at game-camera scale,
so its frequency/contrast has been adjusted for another capture. Four existing
Gym foliage models now use deeper greens and warm brown planter wood, with
positions, normals, indices and bounds unchanged. Their canonical builder palette
is updated too; all twenty Gym model geometry checks still pass (364 closed
parts, 6,220 triangles). The targeted production examples rebuilt successfully afterward. Five fresh
paired native captures in `target/room-glazed-material-review/` passed the visible
GPU frame validator and were inspected: Goldenrod 1F, Sprout 3F, Goldenrod Gym,
Azalea Gym and Celadon Gym. The tower grain now reads at camera scale and the
Gym greens/planter wood are visibly richer. This approves neither the whole room
families nor their remaining textures: wall timber, counter finishes, plain
pink/green Gym floor areas and Celadon's beige path still need material work.
Store plants still resolve as shelving and require their exact drawing/model
binding correction.

The local pack's complete named review set contains 39 maps: seventeen actual
Gym rooms, sixteen department-store maps, three Game Corner rooms and three
Sprout Tower floors. Store coverage includes both stores' 1F–6F and elevators,
plus Goldenrod's basement and roof. There is no separate Celadon store roof
map in this pack. The Game Corner set is GoldenrodGameCorner,
CeladonGameCorner and CeladonGameCornerPrizeRoom. Each map requires its own
consumption audit and native appearance review; representative room views do
not close the full set.

The external pack contains seventeen actual Gym rooms. The final native review
must cover each one, including Seafoam and both Blackthorn floors; similarly
named badge/speech houses are separate house interiors.

| Region | Required Gym room IDs |
| --- | --- |
| Johto | AzaleaGym, BlackthornGym1F, BlackthornGym2F, CianwoodGym, EcruteakGym, GoldenrodGym, MahoganyGym, OlivineGym, VioletGym |
| Kanto | CeladonGym, CeruleanGym, FuchsiaGym, PewterGym, SaffronGym, SeafoamGym, VermilionGym, ViridianGym |

Remaining work:

- Game Corner presentation polish and interaction review after the new complete-source
  north service architecture, counter corners, payout equipment, outward-facing
  slot banks and resized stools; base-map residuals are checker floor/entry carpet
- Sprout Tower's new animated central pillar needs native animation review;
  altar/guardian drawings, ladder-side
  architectural courses and remaining native timber strips; related tower
  interiors need their own source and dynamic-state review
- Remaining Gym wall courses, specialty floor/ice/lava/water materials and
  puzzle fixtures, with their original events and battle approaches retained
- Olivine's north structure is Jasmine's raised platform with a south stair
  flight and rocky side faces; its native floor selection does not convert
  those faces or stairs. Violet's block 15 upper course remains 40 facade
  cells. Sprout 3F's inner halves of blocks 2e/2f contain an eight-cell timber
  wall course with a central framed ornament, separate from its guardians.
  Source-image inspection establishes these shapes; their full-volume
  conversion and actual color/appearance review remain outstanding.
- Department-store remaining service fixtures and basement architecture; eight
  service-counter ends now have complete geometry, including four registers.
  Both stores’ 2F U-shaped counters and 4F L-shaped returns now use the same
  furniture kit, with clerk floors and adjacent shelves preserved.
  Preserve native charm and color throughout the requested rooms: source
  palettes, bright accents and playful details are part of the conversion.
  Twelve source-guarded
  floor lift fronts now have open 3D frames and metal thresholds (96 cells).
  Cabins and Goldenrod's roof already have special-room models

The timber Gym follow-up covers Ecruteak's 96 perimeter cells and both rear
statues. Native Ecruteak and Cianwood views verify joined standing wall strips;
Ecruteak's remaining native identities are floor boards and intentional puzzle
darkness. The invisible-path collision/event data is unchanged.

The previous requested-room production audit identified 2F/4F service runs;
those now have guarded counter geometry. The final four-layout audit consumes
all 120 new service cells with flat native residuals and no mesher errors;
Goldenrod 2F and Celadon 4F native views inspect the opaque joined fixtures. The 5F relief remainder is its genuine staff carpet inset, confirmed in a
native comparison. Basement wall geometry now has source guards for all 492
wall cells; native consumption and presentation verification remain pending.
Basement lift/stair geometry now also has complete source selectors and packed
warp checks; its native consumption and presentation remain pending. Cerulean's
144 pink brick deck cells and 16 wooden face cells now have map-scoped geometry
selectors. Pack inspection distinguishes walkable deck from water-border faces;
deck geometry retains the original zero-height footing. Cropped-cell regressions
are written, but Rust execution and native consumption/presentation remain
pending the shared verification build. Cerulean's perimeter and Vermilion's
gates remain in the review queue. These are
remaining drawing families, not completion percentages. Blackthorn's two floors,
Mahogany's ice puzzle, Violet's facade strips and Sprout's animated pillar still
need their own native/dynamic-state review. Static coverage does not certify
interaction or animation.

Sprout's three central pillars now use a complete 4×6 drawing guard and a
modeled timber body. The external-pack check accepts all 72 native pillar cells
and rejects native floor, altered drawings and custom namespaces. The body bends
from the existing live animation-frame handles, keeps its base anchored and
adjusts normals for the bend. Initial model ownership is retained in the built
terrain cache; animation updates only those selected pillars, preserving custom
overrides. The update path also rejects pillar anchors on other maps. Regressions
now exercise valid source with no selected owner, a selected owner, and a stale
owner on Tin Tower; the focused ownership and pose regressions now pass. Source
selection passes; full pose/footing regressions and native
views of all three floors remain pending. This does not close the remaining
tower architecture or certify animation appearance.

Cerulean's gray perimeter now has exact map/block/phase guards and a ceramic
finish with warm grout. Its map-specific profile removes the old shallow relief
while retaining the same zero-height actor footing. The production floor
selector passes the external-pack harness for all 56 distinct gray floor
identities and rejects altered tile/block/namespace identities. Floors earn no
object coverage. The native appearance and full renderer footing regression
remain pending; the north boundary still needs its own architecture review.
The wooden bridge face now requires an exact native water source, including
block and subtile phase. Its previous plain blue substitute is removed. A crop
without that sample retains native presentation and earns no model coverage;
the brick deck uses its own material and does not need a water sample. The new
renderer rejection regression now passes.

Mahogany's five north wall modules now have shallow closed panel geometry in
`dungeon_extension.rs`, selected only by complete native block `27` drawings.
The external-pack source harness proves all 80 wall cells and excludes ice,
floor, custom namespaces and changed artwork. The renderer regression covers
cropped groups, adjacent ice, volume and unchanged footing; execution and native
scale/color review remain pending. Its 18 native stone partition drawings (72
cells) now use the existing GymStonePartition model, with complete 2×2 source
guards. The source harness excludes ice, floor and altered art. Sliding ice and
stop pads also have source-scoped coplanar finishes: continuous blue cloud bands
and distinct dark ribbed pads. All 80 distinct ice identities and 56 ordinary
floor/pad identities pass the external-pack material check. The seam, override,
footing and native appearance regressions remain pending; these source results
do not certify the room as fully modeled or playable.

Blackthorn 1F no longer uses the legacy boulder conversion for its two block
`29` stair drawings. Exact source selectors distinguish stair flights (`29`),
the two standing rocks (`30`), and filled crossing artwork from callback blocks
`3a/3b`. Crossings have volume below their unchanged walking datum; standing
rocks use the existing CaveBoulder kit and stairs use StairFlight. The external
pack harness verifies all three source families and the crossing FLOOR
permission. Native geometry consumption, stair orientation/scale, callback
transitions and full room review remain pending. No scripts, warps or actor
support were changed.
Blackthorn's channels now have closed depth below the unchanged walking datum,
with exact source guards for 222 lava and 62 platform-lip cells in the base room.
The animated lava tiles retain their own production atlas slot rather than a
replacement animation clock or a generic liquid sample. Both floors also have
pale blue ceramic finishes scoped to exact native FLOOR identities (84 distinct
identities on 1F and 64 on 2F). Source checks pass; full mesh regressions, native
animation, callback transitions and room-scale appearance remain pending.

Fuchsia's five north wall courses now have closed panel/cap geometry, selected
only by complete block `0d` upper-half drawings. Its invisible maze `37`, floor,
statues and entry carpet are excluded. The renderer regression checks complete
drawing rejection, volume and unchanged footing; execution and native appearance
remain pending. Olivine rock selectors also now require their native quadrant
masks, and both rooms' statue selectors reject out-of-range subtile metadata.
This prevents altered floor drawings from silently matching standing rocks.

Olivine's native image confirms that blocks `1c..22` form Jasmine's platform,
including floor and steps, rather than a north wall. Its physical platform and
height transition still need native review. The surrounding tan tile floor,
platform's flat floor cells and orange/cream checker strip now have exact
source-scoped ceramic finishes. All 136 distinct floor identities pass the
external-pack harness; rocks, blocked stage edges, steps, entry carpet and
changed art are excluded. These surfaces preserve the walking datum and earn
no object coverage. Native appearance and renderer regressions remain pending.

The legacy tower `26` statue conversion was a source misclassification: its
eight lower-half cells are entrance warp carpet. Removed the authored matcher,
legacy grouped card path and generic prop-card branch, so none can replace the
carpets in Cianwood, Ecruteak or Pewter with statue geometry. Actual Sprout
statues (`30/31` and altar-side `2e/2f`) now use complete 2×4 guards and the
TowerGuardian kit, scoped to Sprout's three floors. The external pack proves
56 true statue cells and 24 excluded Gym carpet/warp cells. Renderer regressions
and native views remain pending; the altar's other architectural courses still
need review.
The packed Sprout 3F layout places `2e/2f` adjacent at block coordinates
`(4,0)/(5,0)`. Their inner halves form one remaining 4×2 architectural course
between the altar guardians: `11/48/49/11` above `21/58/59/21`. This eight-cell
course is a separate conversion/review item; statue geometry does not certify
it. The requested room completion remains unproven until its native appearance
and the whole altar assembly are inspected.

Vermilion's fifteen puzzle bins already use authored bin models. The two
entrance guardian drawings were mislabeled as trash-can banks by the legacy
profile comments. Exact 2×4 selectors now cover both eight-cell drawings with
the existing stone guardian model, leaving their neighboring floor and original
events alone. A complete-drawing/mutation regression is written. Require sixteen
consumed guardian cells and a native entrance view before closing this pass;
the north target/gate drawings now have closed panel geometry, with native review
pending. Direct inspection of the packed 2bpp drawing identifies a dark upper
inset, pale lower wall and repeated plinth. Exact 8×4 guards preserve both complete
panels and the central passage. The source-extracted panel selector passes a
temporary Rust check for all 64 packed wall cells, excluding the central passage,
entrance warps, altered art and custom namespaces. Actual mesh consumption,
scale and camera visibility remain pending native review. The local
pack has no block-change commands or callbacks for Vermilion, and its trash
interaction is text-only. Preserve that authored static behavior; do not invent
a switch puzzle or certify nonexistent open/closed script states.
The source-extracted production selector also passes a temporary Rust check
against the external pack: all sixteen guardian identities are accepted, while
floor, bin, gate and altered drawings are rejected. This is source-selection
evidence; the full renderer regression and native appearance are still pending.

Targeted audits and native views establish this pass only. The older full-map
checkpoint below has not been recalculated for concurrent work or this pass.

The latest production audit verifies all 388 base maps without mesher errors
and records 256,311 cells consumed by authored geometry, 120 above the preceding
256,191 checkpoint. Complete Fast Ship B1F room fronts replace 36 old partial
claims with 156 source-specific architecture cells; their entrances and four
real corner voids remain untouched. All 818 voxel tests pass, with two existing
benchmarks ignored. This does not complete per-map art or gameplay review.
Further validation is tracked in [the conversion checkpoint](3d-coverage-checkpoint.md).

Earlier representative native review covers the four Gyms, stores, sign
families, Cable rooms, keeper room, corrected ground bindings and exterior
player reveal. Those results do not certify every map. A further 48 native views cover the
five latest kits, including reverse views of the captain/chair, bulkheads,
stage, rail openings, reception counters and train shell. Static review found
no additional blocking kit geometry defect. Twenty-eight more views inspect the
stool/block36-bed corrections and close park placements from opposite sides.
Eight final views also verify the four joined block38/39 beds. Train travel remains
a separate gameplay blocker; static review does not certify whole-room art,
animation or per-map gameplay.

## Former ten-entry queue: integrated source reconciliation

The five latest kits address the ten remaining entries from the earlier
seventeen-entry checklist. Retained selector cells below include legitimate
surfaces and specific review exceptions. Representative static views support
the new volumes and openings; actor animation, moving-camera reveal and
relevant controller routes require their own checks. Additional packages remain
open, so closing this selected queue is not whole-world completion.

| Family | Prior selector → retained cells | Current source result |
| --- | ---: | --- |
| Facility instrument cabinets | 208 → 0 | 26 complete paired apparatus drawings integrated across five maps |
| Facility workstation plots | 152 → 40 | 112 newly consumed cells and 40 native floor cells; the complete 120-cell bench ownership also replaces eight old LabCounter cells |
| Radio desks/counter plots | 380 → 240 | 212 floor + 16 entrance carpet + 12 studio side-wall cells retained; the wall edges remain an architectural review item |
| Ship table/captain-desk plots | 272 → 40 | 232 old-query furniture cells consumed; 40 checker-floor cells retained |
| Timber divider screens | 40 → 0 | Six complete Wise Trio panels integrated |
| Barn stall rails | 24 → 0 | Three complete panels integrated |
| Theater stage/backdrop | 240 → 168 | 48 backdrop + 24 fascia object cells; 168 finished stage-floor cells intentionally receive no object credit |
| Magnet Train body/boarding plots | 128 → 32 | 96 old-query body cells consumed; 16 door + 16 rail/backing cells retained |
| Park small round props | 44 → 0 | Five litter bins (20 cells) and four pedestal fountains (24 cells) integrated |
| Park fountain rim/basin | 48 → 44 | Four old-query basin cells consumed; 28 live water/spray and 16 beta-fragment cells retained |

Complete drawings extend beyond several old selectors:

- Facility/Radio owns 502 cells, replacing 12 prior model cells for a net 490.
  The radio portion owns 174: 140 newly consumed old-query cells, 30 adjacent
  new cells and four replaced BroadcastConsole cells. RadioTower2F correctly
  has no desk placement; its eight studio edge cells are still architecture
- Ship furniture owns 386: 232 old-query cells, 88 complete table-header cells,
  and 66 previously omitted cup-table/chair cells. The 208-cell lower-deck
  divider network is also integrated, producing the full 594-cell increase.
  See [ship-room notes](art/ship-rooms.md)
- Both complete train plots own 120 body cells and retain 16 live door plus
  24 rail/backing cells. The center blocks omitted by the old selector supply
  24 of those body and eight of those backing cells. Source door apertures,
  platform approaches and travel behavior remain separate controller gates.
  See [train-station notes](art/train-station.md)
- Both complete pond groups own eight static basin cells, including four
  east-half cells absent from the old selector. Their 40 water and 16 spray
  cells remain live. Safari beta has no complete joined fountain: its three
  disconnected fragments retain 48 cells (30 water, 12 spray, six static
  fragments). Preserve that source topology; do not invent a missing half over
  grass. This remains an explicit fidelity/appearance exception, not another
  complete basin awaiting a generic model. See [park notes](art/park-scenery.md)

## Additional bounded work still open

| Package | Residual cells / maps | Source evidence and required result |
| --- | --- | --- |
| Café tables and service counters | 64 table + 86 counter / 3 | `cafe.rs` exact table/counter selectors; complete round tables and connected C-shaped counters, preserving the operator bay |
| Game Corner rear architecture and counters | 288 rear-course + 88 counter / 2 | `casino.rs`; source-specific walls, payout equipment, pillars, doorways and connected counters |
| Power Plant equipment/track courses | 192 plot cells / 1 | Six block `12` plus six `16` plots; reconstruct connected equipment and track surfaces. Only 48 cells match the existing B2F west-half track drawing exactly |
| Facility divider networks | 948 / 3 | PowerPlant 116, TeamRocketBaseB3F 670, TrainerHouseB1F 162; `facility_divider.rs`; replace/refine the existing closed 16-pixel network |
| Radio studio side-wall edges | 24 / 3 | Complete `3d`/`20`/`3e` cap runs: RadioTower2F 10, RadioTower5F 10, LavRadioTower1F 4. Native ceiling-height glass follows the actual cap strip; the fifth-floor entrance stays open. Three entrance walks and six paired east/west views verified. |
| Underground switch-room walls/gates | 904 / 1 | `elite_four_room.rs`, blocks `2a/37/3d/3e/3f`; validate all script-controlled gate and switch states |
| Warehouse side walls | 288 / 2 | GoldenrodDeptStoreB1F and GoldenrodUndergroundWarehouse; `underground_boundary.rs`, block `0c` west halves and `0e` east halves |
| Center lobby counter ends and PC headers | 44 counter + 44 header / 22 | `pokecenter.rs`; narrow block `03` right-column `0f/25` return and two block `08` wall cells above each PC |
| Flower Shop display assemblies | 44 / 1 | `flower_shop.rs::display_shape`; finish source-specific bent/corner planters and display tables |
| Cerulean Gym bridges | 160 / 1 | `dungeon_extension.rs`, art `05/0b`; 144 brick deck and 16 wood face selectors implemented, native review pending; gray floor `0a` and foam `22` excluded |
| Vermilion Gym north panels | 64 / 1 | Exact `20/21` source guards and closed recessed wall/plinth geometry implemented; native consumption and appearance pending; preserve the central passage and static authored behavior |

Native views revealed ordinary facility tables/chairs, radio/ship stools and
partial bed bodies outside the previous object selectors. The facility tables
and chairs are now modeled as exact source-complete forms. The targeted stool
and all nineteen bed-foot bindings are corrected, as detailed below, including
four source-complete compound berths. Several chairs and
stools sit beneath authored actors on FLOOR cells; actor animation, all required
facings and approaches must remain clear without changing footing, collision,
object positions or script behavior.

These are source-family counts, not additive object totals. Some families already
have legacy dimensional geometry. For example, the 948 facility cells are called
“plane” by the coverage classifier but already render as a closed divider
network. The Cerulean count is not 176 barrels. Whole-object reconstruction and
native appearance remain prerequisites for closing these packages.

## Integrated source families and corrections

The facility furniture kit completes 172 table and 88 chair cells across five
maps with four meshes: square document table, shallow drawer desk, joined
meeting table and square-backed chair. Native views and 32,832 pose checks
support the placement; see [the source and rig notes](art/facility-tables.md).

Both Rocket Base B1F plants now reuse the cached radial-frond sculpture with
whole-block guards and native underlays. The base-map audit has zero remaining
cutout cells, while other source-art categories remain. Paired 3D/classic views
and one ordinary keyboard approach are verified; see [plant scope](art/rocket-room-plants.md).

Ship cabin carpet, mess lino, corridor panels and border trim now finish 2,424
native floor cells without changing heights, navigation art or custom profiles.
Thirty-one new furniture/floor views pass static review. The B1F front bands now have complete connected wall geometry, while the small
PC backing patch belongs to its own drawing rather than the floor mask. Both
room entrances and reciprocal ladders pass production-controller routes.
Trashcan text lifetime and repeated-A release exposed separate baseline
controller limitations, so whole-room gameplay is not yet signed off. See
[the room-front notes](art/ship-fronts.md).


The follow-up furniture correction reuses canonical assets for six additional
Radio Tower stools (24 cells), fourteen additional Fast Ship stools (56 cells)
and nineteen complete ship bed-foot sections (76 cells). The radio/ship stools
keep FLOOR collision and are fitted north of the unchanged actor foot anchor.
The ship's block `36` berth is guarded as one complete 2×4 drawing. Four other
berths span block `38` and block `39`; those joined drawings are now guarded
atomically, including their 16 previously flat foot cells. These binding
corrections add no mesh assets. Exact source guards, custom-profile
fallback and sampled rest-pose rig clearance do not prove animated/native
clearance. The final production audit confirms the 156-cell increase. Twenty-eight
fresh native views inspect the first corrections, ship cabin variants and close
park fixtures; implemented geometry passes static review. Eight further views
verify the four complete joined beds. Static views do not establish animated
clearance or execute the bed's sleep/heal script.
See [ship-room and furniture-binding notes](art/ship-rooms.md).

Seven former entries now consume their intended drawings or deliberately retain
the correct live surface: Gym planters (720 cells), Azalea's central tree (9),
Viridian's maze (356), Celadon's hedges (168), outdoor signs (348), department
wall courses and department U displays. Representative native views now cover
all seven families. Movement/interaction and additional per-map compositions
remain a separate closure requirement.

The department kit owns 520 wall and 112 U-display cells. Of the previous
704-cell wall selector, 184 cells intentionally retain elevator/threshold (96)
and stair/escalator (88) art. The 112-cell U display includes 64 lower display
cells plus 48 connected counter cells; its staff-floor inset stays open.
The 416 previously recognized window-pattern cells were available prototypes,
not previous runtime ownership: the old floor selector rejected these twelve
maps. Do not count them as 416 completed old walls or 416 new objects.
See [department-store notes](art/department-store.md).

The Cable Club follow-up is integrated and production-audited across all eleven
upstairs rooms: 44 safe rear-wall, 154 link-console and 132 open-doorway cells,
plus the existing complete PC/sign assets now bound in ten beta rooms. Its net
increase is 462: 60 beta PC cells + 80 beta sign cells - 8 incorrectly owned
main-room picture cells + 330 rear-fixture cells. Each sign keeps its own frame;
the real adjoining door has an open frame with its live artwork retained.
The 66 rear stripe cells and 22 upper console-tether cells deliberately remain
native within mobile-entry footprints. The two source-negative corners in each
room also remain untouched. They are not missing solid backwalls.
See [Cable rear-fixture notes](art/cable-rear-fixtures.md). Final main/beta
views and a real beta-room stair transition pass; link service interactions
and further entry routes remain in the controller review queue.

The native ground correction is integrated and production-audited: Route10South
adds 8 land-rock cells, Route19 adds 32, and RuinsOfAlphOutside adds 48 tall-grass
cells. These reuse existing meshes on proven same-atlas native ground, adding no
new asset family. Land-boundary rock ownership is now 5,520 cells; shore and pale
path ownership remain 2,488 and 1,168. Targeted native views cover all three maps.
Route19 review caught paving squares under offshore rocks; they now use verified
block `43` water at its native datum, with both corrected groups reviewed.
See [ground-binding notes](art/native-ground-bindings.md).

Other established kits remain integrated:

- Kanto capped posts consume 1,904 cells across 21 maps; gate counters/phones
  consume 1,534 counter and 96 phone cells across 28 maps
- Cable Club partitions consume 968 cells across eleven rooms and the capsule
  consumes 88; the existing ceramic finish adds no object-coverage credit
- Lighthouse masonry consumes 4,176 plain and 640 window cells across six
  floors. The 6F tea table, cot and stool now consume 24, 8 and 4 cells; its
  checker floor is a surface finish. The southwest long fixture is the tea table;
  additional camera and movement arrangements still need contextual review.
  See [chamber notes](art/lighthouse-chamber.md)
- Route40/RuinsOfAlphOutside tree bindings (704 cells), department-store 4F
  shelf/fridge bindings (192), Kanto Center/Mart roles, house block-08 roles and
  Rocket Base B2F retired-divider sampling were corrected previously

Native review still shows live Gym flowerbed surfaces and small source-floor
strips beneath older statues. Those are material/detail refinements, not proof
that the new planter or hedge kits failed.

The quiet Azalea, Goldenrod and Celadon Gym materials finish 2,566 cells,
including 897 backing cells beneath already-counted scenery. They stay coplanar
and add no object ownership. Authored face lighting and player-only exterior
reveal change presentation, not coverage. See [Gym floor notes](art/gym-floor.md).

## Surface and backing review

The bright red/black bands near the facility instruments are 64 native block
`0b` floor cells (MrPokemonsHouse 16, PowerPlant 48), alternating art `0c/0d` in
indoor red palette slot 1. All collision quadrants are FLOOR. They are separate
from cabinet ownership and need source-scoped material review, not pipe volume.
The bank underlays are correct: native block `07`/art `01/26` checker for Mr
Pokémon's room, and block `1b`/art `1c` diamond floor for Power Plant. The checker
receives the existing stone finish; the blue diamond and striped courses retain
live art. Broader room wall/floor skins remain visible and unfinished.

The two Rocket Base B1F plants now consume the former 12 cutout-classified
cells, leaving zero base-map cutout residuals. Wise Trio screen geometry owns
its 40 former cells, and 176 continuous tatami cells remain correctly flat.
Current facade, plane and raised totals are 2,382, 10,212 and 9,446.
Those classifier totals are not missing-object totals.

All 176 traditional-house block `04` cells in Kurt's House and Dance Theater
now receive the strict continuous zero-height tatami treatment, including the
44 formerly retained `46/56` edge cells. The incorrect cushion grouping is
explicitly disabled in those maps; a future ground sample cannot activate
invented pads. The theater's 168 interior stage-floor cells retain the existing
8-pixel datum and earn no object credit. Native front/reverse theater views
show the five dancers meeting the retained stage datum and both access strips remaining readable. Older audience floor,
cushion and fancy-panel surfaces still need material and complete-room review.
See [traditional-room notes](art/traditional-room.md).

Ordinary-house plain wall-art `00` remains on 258 cells across 56 maps, but
`append_known_room_backing` already supplies room wall geometry. Review source
skins and seams against that geometry before suppressing anything. This is not
evidence of 258 missing walls.

The 10,212 residual “plane” cells include tower depth/background (3,820), buoy
courses (3,148), facility dividers (948), Elite Four boundaries/invisible fields
(1,016), ruins boundary strips (512), Center landings (390), ship caps (146)
and cave depth fields (286). Of 9,446 “raised” cells, 8,294 are existing outdoor
cliff/bank terrain. Those terrain edge strips remain a material/coherence review
queue; blanket extrusion or replacement would lose source semantics.

The 656 waterfall cells already have geometry and live animation; the buoy
course intentionally remains at water height. Floors, water, voids, inscriptions,
framed pictures, sign lettering, carpets, source doors and live markings may
legitimately remain surfaces. Their visual quality still needs review.

Completion requires complete source guards, correct sign/door roles, faithful
side/back silhouettes, unchanged footing and warp/actor approaches, relevant
dynamic-state fixtures and native views of each family/variant. Aggregate reports
do not expose final per-cell UV/primitive ownership, so neither an authored map
count nor a zero scalar residual proves every retained skin is intentional or
every scene is finished. Local source reports stay under ignored `target/` or
external scratch storage; packs, extracted source art and generated audit
manifests must not enter the repository or shipped bundle.

The store west plant correction replaces the catch-all rack-to-shelf assignment
with explicit roles for all currently verified Mart display drawings. Exact
Mart `$0b` drawings select `PottedPlant`, `$21` selects vending machines, and
only the enumerated merchandise drawings select shelving. Unclassified drawings
return no model role. The ignored pack confirms eight plants across both stores'
1F/6F maps; basement `$0b` uses a different atlas and is excluded. The regression
checks both 1F maps, all eight changed-art rejections and unchanged actor support.
A temporary Rust check of the exact production selectors passes for all 64
packed plant cells, changed-art rejection and wrong-atlas rejection; it leaves
no copied content or test binary. The whole-drawing renderer regression now passes on the freshly compiled
Bevy 0.20 test executable: both 1F maps select one complete plant, every
changed cell rejects the drawing, and actor support is unchanged. Fresh
west-side native captures remain queued on the shared build cache.

Store service counters now have an opt-in finish path that retains exact named
`reception_counter` primitive ranges. Only explicitly named timber/enamel
materials carry object-registered paint coordinates; unknown materials and
ordinary model placement stay unchanged. The source-fingerprinted store counter
network enables this detail without changing colors, positions, normals, indices
or support. Reserved UVs avoid floor, fountain, outdoor stone and cave tags, and
are explicitly excluded from the live atlas-water mask. Geometry/unknown-material
regression execution and GPU review are pending after the live queued build.

Four production west-side captures at `(3,4)` now verify all eight corrected
store plants as physical plants, with source comparisons inspected in
`target/store-plant-west-review/`. Earlier batch captures used nearest-center
positions even when x/y were passed; these west views use individual `--map`
commands. Plant leaves remain too pale for final color approval. Both 6F right
seat drawings also incorrectly reached shelving through the old catch-all;
exact `$07/$20` source drawings now select seating with the existing 12px
height/depth. Their regression and fresh render remain pending.

Celadon 6F's wall stayed flat because its authored entry callback replaces the
unused rooftop stair block at `(12,0)` with `$03`, invalidating the base full-row
fingerprint. A separate Celadon-only complete runtime binding now owns 40 wall
cells, including the two-bay window over the removed roof flight. It does not
relax the base fingerprint or create a roof warp. The pack verifier authenticates
that exact callback and resulting fingerprint, all wall collision cells, all
unchanged base bindings and 1,900 basement crops; it passes. Its previous loose
basement hash extraction accidentally included later water masks; the extraction
is now restricted to the actual basement matcher. Native callback-wall review
and the renderer's scoped/mutation regressions remain pending.

The current compiled renderer test executable passes the named counter-finish
preservation regression, all eighteen tests selected by `department_store`,
the 6F right-seat regression, and the Celadon callback-wall regression. The
store-wide checks include every changed source field, cropped/missing guards,
custom/reserved profile priority, atomic claims, all 31 sparse layouts, original
footing and geometry bounds. Filter overlap is intentional; these are not an
additive test count. Fresh native review of the 6F seat/wall changes remains open.

Olivine's remaining stage is an exact three-by-two-block drawing ($1c/$1d/$1e
over $20/$21/$22), with forty $51 terrace cells and eight $54/$55 stair cells.
Its other frame cells are retaining-stone art, with two ground-art corners.
A physical stage will need coherent stair/deck support for actors; merely
extruding the border or retaining a zero-height terrace cannot close its 3D
review. The pink outer perimeter remains native flat art as well.

Fresh native source comparisons now verify both 6F right-seat rows as seating,
plus Celadon 6F's callback wall as closed window architecture, in
`target/store-seat-wall-review/`. Four 1F/4F counter comparisons were inspected
in `target/store-counter-material-review/`; their finish compiles and renders,
but the large floor and pale plant paint still read too uniform. The store-only
potted-plant pass now retains exact named primitive ranges, applies deeper green
and terracotta pigment after fitting, and preserves geometry, shading, alpha,
unknown materials and ordinary house/gate plant paint. Stronger stationary
amber-glaze pooling and per-tile firing variation are limited to the existing
source-guarded StoreCeramic UV range. Their new regression/build/native review
are in progress; these edits do not constitute final room approval.

The seat pass removes the former blue-versus-red color inference. Exact named
`purple`/`linen_shadow` primitive ranges retain the approved store palette and
carry fixed upholstery weave coordinates (the existing prop range, y=2500).
Unrecognized material names remain untouched even if their colors match old
upholstery. The new regression checks that case, dimensions, normals, indices
and tagged coordinates; the frame and ordinary floor cushions remain unchanged.

The production render example rebuilt successfully with these finishes. Five
new native source comparisons were inspected in `target/store-rich-finish-review/`:
both 1F stores at (6,4), both 6F stores at (3,4), and Celadon 6F at (12,4)
for the closed callback window bays. All ten 2560x1600 captures passed readback
verification with no shader errors; temporary capture logs were removed. Deep
green leaves/terracotta, stationary amber glaze and woven pink upholstery are
visible at game scale. Seven current `interior_models::tests` pass, including
unknown upholstery material rejection from the opt-in finish and preservation
of support, geometry, normals and indices. The complete store pack verifier
also passes (including 1,900 basement crops and the exact callback variant).
These are a verified store material increment; large counters/window panels,
roof/elevator materials, Game Corner, gym architecture and the other documented
residuals remain open. Arcade palette painting still contains a color comparison
that should be replaced with named material ownership in its oriented append
helpers; this is source evidence for the next pass, not a completed correction.

The next Game Corner pass replaces the cabinet RGB comparison with exact named
`purple` enamel primitive ownership. Screens, controls and unknown materials
keep their paint; finish UVs are registered before each outward-facing rotation.
Source-bound Game Corner reception counters enable the existing wood/enamel
finish, and their potted plants opt into the verified deeper green/terracotta
palette. The Prize Room audit revealed that the generic threshold family owned
its red entrance carpet. That eight-cell rule was removed and replaced by a
complete 4x2 drawing guard (hash `5ff63366c65ce6f9`), shared through three separate
map-scoped bindings with the two main Game Corners. The native border rows were
checked against the ignored pack in memory. Sixteen closed below-datum cloth
strips preserve the original floor, red/pink selvedge and exact warp height;
geometry/mutation tests and fresh native review are queued. Existing generic
thresholds remain untouched. The original base audits and all three before
comparisons in `target/arcade-finish-before/` were inspected; the Prize Room
audit scratch report was removed.

Game Corner verification now passes eight arcade tests, eleven room-geometry/
source tests and ten interior-model tests (filters overlap). The actual resolver
check covers all three named maps, exact whole-mat ownership, an altered source
cell and a reserved cell with no partial replacement. Source-ground selection
uses the existing authenticated casino floor predicate only on the three maps.
The full room suite exposed a stale assertion demanding raised grip strips on
blue-brick paving; its full floor-area/footing checks remain, with the intentional
level paint and sealed body below datum now checked separately.

A fresh production ownership audit confirms eight closed mat cells in each Game
Corner and finite footing throughout, with no mesher errors; its temporary report
was removed. Five comparisons in `target/arcade-material-review/` were inspected,
including the two main entrances. The subsequent casino-only counter palette
restores green fronts and golden enamel tops. Four further comparisons in
`target/arcade-green-gold-review/` verify all three rooms and a Goldenrod 1F store
control view. Store paint, approved sizes and outward machine facing remain.
Named palette metadata retains source pigment and handles valid zero color
channels without rejecting an otherwise valid model; a black named cabinet and
black screen regression passes, with unknown paint and screen controls untouched.
All capture readbacks passed with no shader errors.

The Prize Room now uses a dedicated original `prize_booth` model for its exact
12x4 source course (`ed6b02ffaf1309d1`): a blue wall, two service displays, a
framed sign and a green/gold counter. The sign and displays use the live packed
atlas; no game artwork is copied into the model. Its standard-library generator
matches the canonical JSON exactly. All 37 closed parts (444 triangles) have
positive volume, outward normals and valid bounds. Named enamel/timber parts
receive the existing scoped room finishes; brass, paper and display paint keep
their authored materials.

The booth resolves after customized live-profile ownership and before compiled
interior claims. Both the drawing-signature and casino-block passes exclude the
incorrect Prize Room house-window substitution. A fresh executable passes both
booth regressions, including 240 altered source identities through the actual
resolver, all 48 reserved cells, cropping, exact live sign/display quads,
positive solid volume and unchanged footing. Eight arcade regressions also
pass, retaining machine facing and stool dimensions. The production native comparison in `target/prize-booth-review/` was inspected:
blue wall, both service displays, live sign and green/gold counter are visible,
with no shader errors. Its fresh production ownership audit confirms 48 booth
cells, eight entry-mat cells, finite footing and zero mesher errors; the scratch
report was removed. The Game Corner set is still incomplete: beige wall/payout furniture in the larger rooms, floor motif
review and interaction states remain open, alongside the documented gym/store
architecture and materials.

The Prize Room review exposed a separate floor-material error: its fingerprinted
`Surface::Ceramic` cells used the generic four-triangle faceted lid, overriding
the source's framed cream/gray tiles. Those exact map-scoped cell fingerprints
now select `ArcadeCeramic`. Its body keeps the sides and underside but removes
that faceted lid; the shared arcade tile painter supplies one coplanar complete
floor partition using native subtile phase. Arcade tile paint now opts into the
existing ceramic glaze coordinates. Nine current arcade tests pass, including the new palette/UV/source check.
Expanded complete-floor-area and unchanged-footing tests also pass. In-memory
pack review confirms the 24 exact identities own 88 native floor cells and
rejects 528 changed identity bytes, with no prop tiles claimed. All three fresh native comparisons in `target/arcade-tile-finish-review/`
were inspected. Framed cream/gray tiles and stationary glaze are visible; the
Prize Room has no triangular floor facets, and its booth remains intact. All
six visible readbacks passed with no shader errors. The current production
audit confirms 88 arcade-ceramic floor cells, 48 booth cells and eight entrance
mat cells in the Prize Room, eight entrance-mat cells in each larger room,
finite footing and zero mesher errors across the set; the scratch report was
removed. This verifies the floor/material increment, not the remaining Game
Corner wall/payout, interactions or broader gym/store/tower scope.

The next Olivine stage geometry pass has in-memory pack evidence for the whole
12x8 course (`7282c18080447b71`), containing 40 terrace-floor and eight stair
cells with no scripted block changes. It still needs a physical terrace,
coherent stair support and retained source barriers; this source proof alone
is not a completed stage.

The Game Corner back-wall correction replaces only the casino pass's source-guarded
acoustic-panel substitutions with an original `arcade_wall` kit. It has four
recessed ochre panels, brown timber rails/stiles and gold beads/coping, all in
20 closed parts (240 triangles). Vertical finishes use source height as well as
width, preserving geometry and unnamed shadow materials. Accepted depth/rise,
footing and source reservations remain unchanged. Exact generator, positive-volume,
normal and bounds checks pass. Eleven current arcade regressions pass in the
fresh shared-cache test executable, including the new complete-course guard
and vertical-finish tests. The first native review in `target/arcade-wall-review/` verifies the gold/brown
paint in both larger rooms and the unchanged Prize Room control. Its audit
confirms 40/48 wall cells, no acoustic panels, unchanged other family counts,
finite footing and zero errors; the scratch report was removed. However the
close-up exposed staggered wall planes inherited from the one-/two-row source
rectangles and duplicate lower courses. The renderer now anchors those guarded
wall pieces (and the corresponding Celadon poster) to the same native north
course. Lower 2f/30 bands retain source claims/underlays without duplicate
volumes. A new regression covers all eight wall drawings, joined-plane bounds,
unit counts and unchanged footing. All twelve arcade regressions pass in the newly built executable. The three
second native comparisons in `target/arcade-wall-aligned-review/` were inspected:
wall planes join without the staggered posts, and all six visible readbacks
pass with no shader errors. The production audit retains 40/48 wall cells,
all other family counts, finite footing and zero errors; its scratch report
was removed. This review then exposed missing wall backing below Celadon's
posters. That exact guarded poster placement now adds a complete closed wall
unit behind the frame, with 0.2px front relief. A new production-append
regression checks the complete backing, relief, retained four frame/four wall
source claims and unchanged footing. All thirteen arcade regressions pass in the fresh executable. All three
muted native comparisons in `target/arcade-wall-backed-review/` were inspected;
Celadon now has continuous backing below its posters with no holes or z-fighting.
The production audit retains 40/48 wall cells, eight Celadon frame cells,
unchanged other families, finite footing and zero mesher errors. Its scratch
report was removed. Payout furniture and interactions remain beyond this wall
increment.

The department-store blue tint heuristic has now been removed. Its original
generator names glass, reflections, glints and shadow paint explicitly. Closed
diagonal reflection/glint facets replace the flat rectangular highlight; the
cream frames, gold trim and accepted bounds are retained. All eight canonical
JSON models were regenerated without game artwork or pack exports. The full
source/geometry checker passes: 178 closed positive-volume parts, 4,184 triangles,
1,900 accepted native crops with changed visible drawings rejected, and intact
callback/stair/lift/counter/island source ownership. Non-window geometry and
paint match the earlier generator after excluding new material-name metadata.
A production-append regression checks that all five window-course sizes retain
authored paint and unchanged footing through the real store mesher. All nineteen department-store regressions pass in the fresh executable.
All twelve muted native floor comparisons in `target/store-blue-glass-review/`
were inspected: the blue glass, diagonal reflections and frame joins are
consistent across both stores, including lift and stair openings. All 24 visible
readbacks passed without shader errors. The production audit retains the base
object totals and 520 architectural wall cells across the twelve floors, finite
footing and zero mesher errors; its scratch report was removed. This verifies
the window finish. The fifth-floor display transparency still looks washed out
and needs a separate presentation correction.

The Game Corner payout display now has an original authored
`arcade_prize_rack` mesh and a dedicated model binding. Only the existing complete
casino block 04 source guard selects it; department-store bookcases remain
unchanged. It retains the reviewed 32px width, 10px depth and 28px rise, with a
closed dark cabinet/back, metal uprights/grille, brass shelves/drawer pulls and
beveled colored prize packages. All 55 closed parts (932 triangles) have positive
volume, valid winding and finite bounds. Drawer fronts/pulls stand beyond the
closed lower body. A new production regression exercises both rooms, unchanged
support/claims, all 160 altered-source cases and reserved cells. The fresh renderer test executable passes all fourteen arcade tests, all six
casino architecture tests and the complete interior-model finite/full-volume
regression, including the new case. Native verification remains pending in the
shared build cache; this is not yet a visually verified payout display.

The fifth-floor display island is now excluded from wall-reveal ranges through
its exact `mart/department_store_closed_u` network label, alongside the existing
waist-high counters. The production-append regression requires its solid and
textured geometry to remain outside those ranges while preserving claims,
footing and owned-cell bounds. The fresh test executable contains the renamed opaque-display assertion and
passes all nineteen department-store regressions. Native review remains pending;
the earlier washed-out image is not evidence of this correction.

The Olivine source recheck distinguishes 32 ordinary deck-floor cells and eight
UP_WALL deck cells, eight floor stair cells, 38 WALL retaining cells and eight
UP_WALL retaining cells. Its two ground-art corners have WALL collision. These
aggregate roles were checked in memory from the external pack without exporting
source catalogs. A stage implementation must preserve directional barriers as
well as its complete drawing guard; the current stage remains flat.

The existing full-map native Goldenrod roof comparison was re-inspected. It
shows physical parapets, a planter trough, viewing instruments and wooden slat
seats, so describing these as wholly flat would overstate the geometry gap.
The quality gap remains substantial: pale faceted floor, sparse stick-like
planter foliage, wooden seating that drops the original pink accent, and the
central enclosure still dominated by native atlas panels. The exact planter
uses `RoofPlanter` (4x2 plot, 13px rise, 12px depth); roof seating alone uses
`StationBench` through a complete 2x4 drawing guard. Preserve those source
guards and avoid changing shared station furniture when correcting roof paint.
No roof correction or fresh roof verification is claimed by this inspection.

Fresh muted native verification in `target/arcade-prize-rack-review/` completed
for all three Game Corners, Goldenrod 1F and both stores' 5F. All six comparisons
were inspected, with twelve visible readbacks and no shader errors. Dedicated
prize cases show dark lower cabinets, metal uprights/grilles, brass shelf lips
and colored packages in both larger rooms. The Prize Room booth and 1F store
window/seating control remain intact. The current production audit confirms
24/36 dedicated prize-case cells, no retail-shelf replacements in the larger
Game Corners, unchanged 40/48 wall cells, and the Prize Room's 48 booth/88
ceramic/eight mat cells. Store object totals remain 140, with each 5F island
owning 56 cells. Footing is finite and all six maps have zero mesher errors;
the scratch report was removed. The fifth-floor opacity correction is visually
verified, but it exposes a broad blank cream countertop and dull display glass.
Those finishes/details still need refinement before the display can be called
complete. This verifies the prize-case increment, not Game Corner interactions
or the full gym/store/tower goal.

The fifth-floor finish refinement now uses explicit warmer countertop enamel,
blue lid/front glazing, closed diagonal sky reflections/glints and thin copper
countertop beads. The inset cream tops end below the copper beads, preventing
coplanar overlaps. Accepted 64x64x8 bounds, all four original sealed bodies,
body paint, floor inset and exact source binding are retained. Only named
`countertop_enamel` primitives in `closed_u_display` receive the stationary
enamel finish, through the actual store append path; other models/materials
are not guessed or tinted. Two added regressions cover production finish,
unchanged mesh paint/geometry/support/56 claims and unknown material/model
exclusion. The source/geometry checker passes all 1,900 crops and intact
fixture guards: eight models, 192 closed parts and 4,416 triangles. All twenty-one department-store regressions pass in the fresh test executable,
including both new finish exclusions and production paint/support checks.
Native visual review remains pending in the shared cache. The archived
Blender scene still predates these finishes and has not been resaved.

An original authoring generator, `tools/build-olivine-stage.py`, now defines a
96x64 terrace at an 8px deck height, with 40 warm paving tiles, 46 golden
retaining stones, closed foundations and north/south stair levels at 8/4px.
The standard geometry validator passes all 2,528 triangles in the in-memory
prototype; temporary model output was removed. No canonical stage mesh has
been emitted or bound yet. Integration still requires the complete native
stage fingerprint, retained source ownership/barriers, coherent actor support,
crop/reservation rejection and native traversal/appearance verification. The
current flat stage is not converted by this authoring step alone.

The new fifth-floor finish is now inspected in fresh muted native comparisons
for both stores in `target/store-display-glass-review/`, alongside Goldenrod 1F
and Celadon Game Corner controls. All four comparisons were inspected; all
eight visible readbacks passed without shader errors. Blue lid/front glazing,
diagonal reflections, copper beads and the warmer opaque countertop are visible
without coplanar flicker. The staff inset remains open; controls retain the
verified window, seating, arcade wall and prize-case presentations. The current
production audit confirms 140 object cells on each store floor, 56 island cells
on each 5F, and unchanged Celadon Game Corner wall/prize-case counts, finite
footing and zero mesher errors; its scratch report was removed. All twenty-one
store regressions pass in the completed union test run. This verifies this
finish increment; the broad island design still has limited merchandise detail
and is not a claim that every requested store/gym/tower room is complete.

The authored Olivine terrace is now emitted as `olivine_stage.mesh.json` and
bound through a dedicated whole-course resolver at native (4,4), fingerprint
`7282c18080447b71`. It owns 94 cells, leaving both north ground-art corners
unclaimed. Custom live objects and reserved cells retain priority. The model
replaces flat stage art with closed paving, golden retaining stones and stair
treads; rendered support rises to 8px on the deck/north stair and 4px on the
south stair, while original collision/scripts are unchanged. Named stone
finishes remain scoped to this model. `tools/check-olivine-stage.py` validates
96 closed original parts/2,528 triangles and the actual pack guard, rejecting
480 altered sources and 96 incomplete courses without retaining fixtures or
exporting game catalogs. Two new renderer regressions cover guard mutations,
reservations, custom objects, atomic append, sparse ownership and nonuniform
render scale/support. Fresh Rust/native/traversal verification is pending;
the stage is not yet a verified runtime conversion.

The Olivine stair traversal review also exposed that moving screenshots still
used a window target during explicit image-target runs. The movement capture
now requests the same prepared image target and scale as the final capture;
it returns no capture while that explicit target is unavailable, without a
window fallback. A regression covers unprepared/ready image targets, scale 2
and the unchanged window mode. This change remains pending fresh Rust/native
verification along with the terrace. It enables real intermediate stair
images; movement logs or a final floor screenshot alone will not certify actor
contact with the raised deck.

The fresh voxel test executable now passes both `olivine_stage` regressions,
including custom-profile refusal, all source/reservation cases, atomic append
and scaled support levels. Its full interior-model finite/material/full-volume
regression also passes with the terrace included. Native consumption, stair
contact during actual movement and the moving-capture regression remain pending.

All four fresh Bevy image-target regressions now pass, including moving
captures using the exact prepared scale-2 target with no window fallback and
unchanged viewport/other-camera ownership. Native traversal/readbacks remain
pending the current example build.

Fresh muted native static review in `target/olivine-stage-review/` covers
Olivine, Pewter, Goldenrod 5F and Celadon Game Corner. All four comparisons
were inspected; eight visible readbacks passed without shader errors. Olivine
now has a raised closed terrace and two physical stair treads. Its production
audit confirms 94 terrace cells, unchanged 216 boulder and 16 guardian cells,
finite footing and zero mesher errors across all four maps; the scratch report
was removed. Store and arcade controls retain their verified presentation.
The Pewter control exposes two entrance guardian drawings still reading as
flat atlas art; the surrounding gym rocks remain gray instead of native gold.
Neither issue is resolved by this terrace change.

The first native Olivine `UUDD` traversal from (4,6) failed the movement-QA
assertion: no player translation, one grid origin and zero additional builds
across 128 frames. The trace records the application window as unfocused
throughout, and the first moving screenshot has no blocking dialogue. Explicit
image-target moving readback succeeds (four PNGs), but this run proves no stair
traversal or actor/deck contact. The next check must focus the actual preview
window and wait for focus before driving ordinary keyboard input; do not
fabricate focus or treat the static stage as movement proof.

The user explicitly rejected gray gym rocks. Olivine and Pewter now select a
separate palette treatment only after their existing complete 2x2 source
guards pass. The canonical cave-boulder geometry retains its named `stone`
and `stone_light` ownership; these two gym treatments sample live body/highlight
texels (8,6)/(4,4) from the guarded drawing rather than repainting by RGB or
changing the shared cave palette. Both current native 2D drawings confirm
ochre (132,107,24) and pale gold (214,198,140) at those anchors. Geometry,
collision, footing and cached cave assets are preserved. New regressions cover
named ownership, live UV mapping, cached geometry/color preservation, map
exclusions and incomplete drawings. Fresh Rust tests/native palette review
are pending in the shared union build queue; this is not yet verified visually.

The movement-QA driver now pauses and releases direction keys whenever the
real primary window is unfocused. It does not consume a route step or falsify
window focus. A fresh example build and focused Olivine traversal remain
pending; the previous zero-translation stair run remains a failed check.

The static-rock palette scope also includes Cianwood's authenticated tower
rock drawing: its reviewed native source uses the same gold/ochre treatment.
Sprout and unrelated rooms remain excluded. Cianwood's movable Strength
boulders use a separate actor path and still require their own palette check.
The first fresh build exposed a separate planter model constructor missing the
new optional semantic field; it now explicitly has no gym-stone ownership.
The corrected union test/build runs are in progress.

The freshly linked voxel executable now passes both `gym_stone_` regressions:
complete Olivine/Pewter/Cianwood drawings select live palette anchors, altered
drawings and unrelated maps do not; cached cave colors and all fitted geometry
remain untouched. All five dungeon-model tests also pass, including finite
full-volume assets, budgets, exact bounds, normalized fitted normals and
single-pass face shading. Native palette inspection remains pending the
current render/audit example build.

The completed fresh union example build now supplies muted native comparisons
for Olivine, Pewter, Cianwood and Dark Cave in `target/gym-gold-rock-review/`.
All four comparisons were inspected, and all eight visible 2560x1600 readbacks
passed without shader errors. Static gym rocks now read as ochre/gold with
pale golden caps; the cave control retains its mineral palette. Cianwood's
four movable boulders remain gray in that verified run.

The separate Cianwood movable-boulder correction now retains explicit named
stone material ownership in its original canonical model and Blender exporter.
Only metadata was added to the canonical asset: an exact comparison against
the original verifies every position, normal, index, color and primitive order
unchanged. The importer applies verified native Cianwood sprite body/highlight/
shadow shades (165,123,24)/(198,148,57)/(57,57,57), converted to linear PBR
colors, solely to named boulder materials; unknown names/assets reject this
paint path. Runtime selection is Cianwood+object+boulder only, with distinct
cache keys and instances so movement reuses geometry and paint cannot leak
into other rooms. Geometry/palette and actor movement/map-transition
regressions are authored; fresh Rust/native checks are pending. The global
actor asset checker currently fails on unrelated `icon_clefairy` triangle
budget (38,760 >25,000); no passing full-kit claim is made and its limit was
not relaxed. No Blender archive resave is claimed.

The focused Olivine `UUDD` route now completes successfully from (4,6). A
temporary ignored local app wrapper gave the native preview real window focus;
the route had already finished when later UI raise calls reported no active
window. Its 128-frame trace records `focused=true` throughout, five grid
origins, zero additional terrain builds, median 12.02ms and p95 26.09ms.
All four moving images and the final image in
`target/olivine-stage-walk-focused/` were inspected: the player ascends onto
the raised terrace, stands/walks on its deck and returns to the lower aisle.
No blocking dialogue or shader errors were observed. Individual 4/8px tread
support remains covered by the previously passing mesher regression; the
periodic captures do not isolate every intermediate foot contact. This is
actual keyboard traversal evidence, not merely a static room render. The
wrapper and scratch movement CSV were removed after review; no own preview
process remains.

The fresh Cianwood named-material palette regression now passes, including
unchanged geometry/normal/index/UV attributes, unchanged ordinary boulder
cache and rejection of unknown material or asset names. The newer actor
movement/map-cache regression needs the queued fresh test executable;
production-native movable colors remain pending the fresh example build.

The broader freshly linked dungeon-placement suite runs 29 tests: 28 pass,
and the existing Cianwood static-rock regression still asserts the old solid
mesh channel. It now checks the palette-sampled textured channel for the same
>8px physical volume, retaining unchanged footing and complete-source checks.
The fresh suite rerun is queued; the failure is not represented as a pass.

Pewter's two native entrance drawings are independently verified in the local
pack at source anchors (4,20)/(14,20): tower blocks 19/1a, head rows22/23 and
32/33, pedestal26/27 and36/37. A map-scoped complete-eight-cell binding now
uses the original volumetric guardian model, claiming only each statue's2x4
plot. Tests cover both handed block layouts,80 altered source fields, other
map exclusions, neighboring floor and unchanged support. Fresh Rust/native
verification is pending; the reused guardian's head palette/detail still
needs native review against the gold head/gray pedestal source.

The completed fresh Cianwood movable-palette native batch in
`target/cianwood-gold-movable-review/` now confirms all four moving boulder
actors use the verified warm ochre paint instead of gray. Cianwood, Pewter
and Dark Cave comparisons were all inspected; six visible readbacks pass
without shader errors, and the other rooms retain their source treatments.
The current four-map production audit has zero mesher errors and unchanged
claims: Cianwood320 cells/static rocks16, Pewter184 rocks, Olivine326 cells
including94 terrace/216rocks/16guardians, Dark Cave2582 modeled cells. Its
scratch report was removed. This audit/build predates the new Pewter guardian
binding and cannot verify its16 additional claimed cells. The newer actor
cache/movement regression and guardian regressions remain in the live union
queue; this render verifies paint, not Strength puzzle gameplay.

Pewter's guardian adapter now also preserves its native gold figure/gray
pedestal distinction. The original guardian exporter explicitly labels head
assemblies by authored object names and pedestal assemblies separately. Only
region metadata was added to the canonical mesh: every original position,
normal, color, index and order is verified unchanged against the original.
Its16 closed authored assemblies retain complete valid region ranges. The
importer consumes this ownership only for the named guardian; Pewter's exact
2x4 drawing then samples separate live gold/highlight/shadow and pedestal
gray/cream/shadow texels. Other maps continue using the original cached
colors. Unknown names/regions, incomplete or overlapping ranges cannot
acquire this palette treatment; no RGB or generic object-shape recolor is
used. The planter occlusion importer preserves the new semantic arrays using
its existing vertex remap, alongside its concurrent stone-array preservation.
New regressions cover named assembly/color UV anchors, unchanged geometry,
cache preservation and unknown ownership. Fresh Rust/native checks are queued,
and the old Blender archive is not claimed to be resaved.

The completed union Cianwood actor run passes both regressions, including
physical movement with the same cached object/mesh and a map transition that
restores the ordinary boulder palette at the same position. The native
movable-rock preview and source audit were already verified above; Strength
puzzle gameplay remains separate. The guardian geometry checker now confirms
416 unchanged triangles and complete valid named head/pedestal vertex ranges.
Its new Rust palette/drawing tests and native review remain queued.

The roof review confirms that its native pink panel/coral-edge seat drawing
was still rendered as brown station timber. A dedicated `RoofSeat` adapter
now retains the existing exact2x4 fingerprint8dc9d24b7ddbc743 and opts into
named `station_bench` timber-panel palettes only. It uses the inspected native
pink(255,156,198)/coral(247,82,49) shades converted to linear render colors,
retaining authored grain, positions, normals, indices, UVs, metal frame and
footing. The ordinary station adapter remains unchanged. New tests assert
this source/map-specific route, preserved geometry/metal/cache and exclusion
of unknown model/material ownership. Fresh tests/native preview remain
pending; no roof quality-completion claim is made. The old white faceted floor
and stiff rod foliage remain visible quality gaps in the reviewed roof.

The guardian palette regression additionally checks missing/incomplete region
ranges and incorrect starts/ends; invalid ownership cannot receive the sampled
head/pedestal paint. These assertions remain pending the same fresh test queue.

The fresh voxel test executable now passes both Pewter guardian regressions
and all30 dungeon-placement regressions, including Cianwood's corrected
textured rock volume check and unchanged cave/stair support cases. The source
guard rejects all80 altered guardian fields and keeps adjacent floor intact.
Native guardian review remains pending the active example build.

The roof source review confirms its native floor is gray square paving, while
the present special-room `Surface::Ceramic` emitter produces pale triangle
facets. Its exact roof cell signatures are already isolated. A correction
must cover both those owned floor cells and the underlays beneath roof
fixtures; merely recoloring free floor would leave pale patches under seats
and planters. The special-room placement currently carries no room identity
into append, so that roof-floor ownership must be retained explicitly from
the resolver. Do not infer a roof palette from shared mart source IDs or
recolor the generic ceramic/stone families.


Fresh muted native review in `target/gym-palette-final-review` confirms
Pewter and Cianwood static rocks remain ochre/pale gold and Cianwood moving
boulders retain darker gold. The centered Pewter camera clipped the entrance,
so a second real render at (5,12) verified both physical guardian figures:
gold heads/bodies and pale stone pedestals, matching separate native regions.
All eight reviewed gym/roof readbacks were visible with no shader errors;
the first window-target blank capture was rejected and rerun using explicit
image targets. The production source audit reports Cianwood320 and Pewter200
modeled cells (including16 guardian cells). Its scratch JSON was removed.
The roof preview includes the new pink/coral seats but still predates the
new paving emitter; its white faceted floor remains an unverified correction.

Roof placements now retain explicit roof ownership from the resolver into
append. The paving emitter supplies a physical slab with sealed sides and
underside and one live source-textured lid, removing the old coplanar solid
lid. Free floor retains each verified cell UV; fixture underlays use the
verified floor donor instead of stretching fixture artwork into the floor.
The added regression checks both routes, no duplicate upward solid faces,
and unchanged footing. Fresh union roof-floor tests and examples are queued;
no successful build or native roof-paving result is claimed yet.

The newly rebuilt test executable contains the roof-seat regression and
passes it directly:1 test, unchanged geometry/metal/cache with named pink
paint. Both guardian regressions also pass in that executable. Parent Cargo
runs remain active; the roof-paving literal is not yet present, so paving
verification and the next fresh native example build remain pending.


The fresh roof-paving regression now passes in the rebuilt voxel executable:
free floor keeps its live cell UV and fixtures use the verified floor donor;
solid upward lids are removed, sides/underside remain and support stays zero.
The production example build is still queued, so native roof-paving appearance
is not yet claimed.

The original roof planter has been replaced by `tools/roof_planter_sculpt.py`:
48 sealed pointed leaves in three layered crowns, closed orange trough,
cream rolled rim, dark earth and four physical cream/yellow ornaments.
The source screenshot confirms seven semantic anchors within its verified
32x16 drawing: leaf123/165/8 at(4,2), light148/198/74 at(2,1),
dark74/107/0 at(9,2), cream247/231/214 at(2,9), orange255/132/8
at(2,12), earth57 at(0,0), yellow255/255/57 at(4,13).
No source image/export is stored in the model. Runtime uses the existing exact
4x2 hash72eaa4fc6fc41433 and samples these anchors into the current source atlas.
The new geometry is680 triangles and the closed-geometry checker passes all23
special-room models. The Blender builder imports the same original named
parts; no Blender archive resave is claimed. Rust planter-volume/live-UV and
native appearance checks remain pending the active union builds.


All13 special-room regressions pass in the current rebuilt executable,
including roof paving and complete source/drawing guards. This executable
still lacks the new planter regression, so those13 do not prove the new
planter append path. The Goldenrod-owned parapet now uses native neutral
107/156/57 grays converted to linear colors instead of the previous pale
cyan paint. The retained explicit roof flag limits this change to the
Goldenrod terrace; mansion/mobile rail adapters retain their authored paint.
Fresh roof tests and the native example build remain live in the shared union
cache. No roof completion claim is made before appearance verification.


The refreshed voxel executable now contains the planter regression and passes
all19 roof-filter tests, including live planter UVs, 13-pixel leafy volume,
sealed undersides, zero support changes, source floor donors and pink seat
cache preservation. This is Rust evidence, not native appearance proof.

The source pack confirms Sprout Tower side bands use the same complete tower
04/06/08/0a/0c/0e drawings already owned by timber gym side-wall meshes.
The existing complete-strip resolver now explicitly accepts all three Sprout
floors for those six drawings only. Gym guardian joins and invisible-field
planes remain separate. The regression adds all three floors and rejects
alterations to each source field or reservation on every constituent cell.
A fresh union test and production preview remain queued. No full tower or
39-room completion claim is made.


Viridian's reviewed native drawing has blue patterned paths and green maze
blocks; generic stone finishes had turned both gray. All16 original maze
variants now use explicit named wall/coping/trim paint in the native
74/107/0,123/165/8,148/198/74 greens and57 shadow, converted to linear colors.
`tools/gym_maze_palette.py` is shared with the Blender authoring builder.
Each canonical mesh was compared to HEAD: exact geometry, normals, indices,
part ownership, triangle order and join datums are unchanged. The model
checker asserts the named palettes and passes20 meshes/364 parts/6220
triangles. Maze use remains inside Viridian's complete network source guards.
No Blender archive resave is claimed.

The floor finisher now leaves Viridian's native train-station tile3d on its
existing physical floor rather than replacing its blue motif with generic
stone. This also preserves the verified floor donor beneath maze sections.
The regression checks retained full mesh/UVs and ordinary station stone paint.
Its new union test and native appearance review remain pending. Previous
roof19 tests and the completed parent roof-seat Cargo test pass; Sprout's
new regression literal is now present in the rebuilt executable.


The fresh Sprout perimeter regression passes directly in the rebuilt voxel
executable, including all three floors, every guarded cell's five source
fields/reservation, unchanged support and preserved surrounding floor.
Read-only pack checks count80/64/88 side-strip cells on1F/2F/3F respectively,
with12/11/14 complete strips and no overlap with native warp or actor cells.
The examples build remains live; these checks do not replace native review.


Blackthorn's source-scoped `BlackthornTile` finish now restores blue brick
courses instead of pale cyan square tiles. Its palette uses native blue
140/156/255 converted to linear render color, with cream mortar. Courses
partition the floor plane into nonoverlapping brick/mortar rectangles and
retain the existing world phase and support height. A new Rust regression
checks total floor area, zero rectangle overlap, unchanged walking datum,
and upward normals at three scales. That union test remains queued.
The read-only enhanced-floor source checker passes15 rooms, including both
Blackthorn floors and all exact identity mutations; it proves source scope,
not the new emitter's native appearance. Fresh preview still pending.


The newly rebuilt voxel executable contains and passes Viridian's live-blue
floor preservation regression. The station control still receives its ordinary
stone finish, and native geometry/UVs remain byte-for-byte unchanged in the
Viridian test. Its roof19 tests also pass in this refreshed executable.
Blackthorn's new partition regression is not present yet and remains queued;
all native appearance review still waits on the exact live example build.


Store elevator cabin review found the source gold floor/blue glazing had been
replaced with gray ceramic and gray walls. Both cabins now have an explicit
6x2 glazing object guard904830824255ead1, reusing three original store window
prototypes. The old12 per-cell Steel wall signatures per cabin were removed;
an incomplete or altered glazing plot cannot still receive gray partial walls.
The special-room placement retains typed Default/Roof/StoreLift floor ownership
from exact map resolution, replacing the earlier roof-only boolean. Owned
cabin paving and threshold cells keep their own live tile UVs, preserving gold
floor and red exit carpet on physical closed slabs. Fixture underlays use the
verified floor donor, avoiding fixture-art floor paint. Ordinary steel rooms
keep their existing emitter. New floor/glazing/footing tests and a fresh native
cabin build are queued; completion is not claimed.

The prior production example build now completed successfully. Its muted
image-target batch reviews roof/Sprout/Viridian/Blackthorn revisions, and it
predates the new typed-floor/cabin revision. A subsequent example build is
queued specifically for that newer production revision.


The muted native batch completed successfully for roof, all3 Sprout floors,
Viridian and both Blackthorn floors:14 visible2560x1600 readbacks, no shader
errors. All7 comparisons in `target/room-palette-volume-review` were inspected.
Roof paving is now gray square source texture, seats pink/coral and planters
sealed green leafy crowns/orange troughs rather than pale rods. Sprout's side
bands now form physical continuous timber perimeters while floor, ladders,
actor spacing and intentional depth fields remain distinct. Viridian visibly
restores green walls/blue patterned paths. Blackthorn1F shows source-blue
staggered bricks around unchanged lava, and2F retains its guarded blue floor.

These previews also identify remaining work: Sprout pillar paint/capital is
plain beige and its3F guardian figures are still gray despite gold source
heads. Blackthorn2F movable boulders are still gray despite the source's gold
sprite. Roof foreground parapet/access/stair finishes remain mixed, and the
central inset still contains native artwork. None of those rooms is declared
fully finished. The store cabin revision and typed floor ownership postdate
this renderer and remain pending fresh native/tests.


The next palette correction extends the dedicated named movable-boulder gold
finish to BlackthornGym2F as well as CianwoodGym. The source sprites in both
rooms use ochre 165/123/24 and gold 198/148/57, with 57/57/57 shadow. The
actor regression checks entity reuse while moving inside each gym, shared
gold mesh reuse across map transitions, and ordinary paint outside those maps.
The canonical boulder geometry and ordinary cached palette are unchanged.

Sprout guardian figures now use live source anchors independently of their
pedestals. Both verified head variants use gold at [8,6], highlight [8,1],
and shadow [6,0]; pedestal anchors are [8,24], [2,26], [0,24]. These adapt
the existing validated semantic head/pedestal ranges without changing the
closed guardian geometry or Pewter's own anchors. The placement remains an
exact complete eight-cell drawing. New cache/geometry/anchor tests and a
fresh muted renderer build are running; these changes have not yet received
fresh native verification. An intermediate undeclared Model dispatch compile
error was repaired with explicit calls before the current rebuild.


Muted native review in `target/live-indoor-palette-review` completed for
Cianwood, Ecruteak, Sprout3F, Blackthorn2F and both store elevators:12 visible
2560x1600 readbacks, all6 comparisons inspected, no shader errors. It verifies
gold movable Blackthorn rocks and Sprout gold guardian figures with gray
pedestals. Both elevators now show three physical blue glazed panels and the
red exit carpet. Their floor pattern still appeared as plain gold ceramic: the
later general surface pass was repainting the live slab. An explicit mart01
exception on only the two elevator maps now preserves the live lid, with a
new final-surface-pass regression. That final correction is queued for fresh
tests/native verification; neither elevator is declared finished.

Tower timber side strips and horizontal partitions now sample each complete
owned drawing's live main/highlight/shadow/iron shades from named timber-wall
materials. Both source11 and41 anchors were checked against pack pixels in
memory:170/255/85/0. Geometry, ground support, complete source guards and
ordinary cached timber paint stay unchanged. Cianwood/Ecruteak/Sprout native
views confirm the richer gold/brown wall palette. A subsequent refinement
retains bounded model-local grain on those live wood colors, excluding iron
pegs; that refinement postdates this preview and awaits the queued build.
Named palette ownership also survives the concurrent occlusion remap.
The next review still needs Sprout's beige pillar/capital finish and animation,
remaining gym objects and complete store/game-corner interactions.

The earlier movable-rock regression failed because it incorrectly expected
entity identity across a real map transition. It now verifies a new entity
with the same cached gold mesh after transition, motion reuse inside both
gyms, and the ordinary finish on leaving them. The corrected run is queued.
Direct refreshed-binary tests for cabin floor/glazing, Blackthorn nonoverlapping
brick area/support, and Sprout guardian cached geometry/live anchors pass.


The refreshed voxel test binary now passes the corrected gold boulder palette
and actor movement/cache tests (2), live timber palette/geometry tests (2),
and perimeter complete-source guards (1). A later refreshed binary includes
and passes both store-lift tests, including the final surface-pass exception,
and9 Sprout-filter regressions covering the live guardian anchors, pillar
source ownership, sway, base, normals, support, ladders and perimeter.

Sprout's pillar now uses live dark timber and gold carving anchors on the
existing canonical column, with its foot sampled from the dark source base.
The original geometry and48px rise remain unchanged: source inspection found
its width consistent with the drawing. Both initial terrain and animation
refresh put it in the textured animated mesh, preserving its selected source
anchors through native-frame motion. Named timber palette ownership accepts
only canonical timber_wall/timber_column and known material roles; ordinary
cached paint remains unchanged. Three pillar source anchors were checked
in memory against the pack: [12,6] dark0, [6,26] gold170, [4,42] dark0.
No source pixels were exported. Native appearance remains pending.

Muted review of all3 Game Corners and Goldenrod1F completed successfully:
8 visible2560x1600 readbacks, all4 comparisons inspected in
`target/current-arcade-store-review`. Slot faces remain directed toward stools,
rear service areas retain distinct machinery/prize volumes, and the prize
room has its full physical blue service wall/windows. Goldenrod1F's blue
glazing, lift/access, reception and pink seats are materially improved from
the rejected earlier capture; its warm ceramic floor still read as bland.

Store paving now replaces the washed-out uniform honey tiles with physical
lid regions in a clear angular yellow/amber motif and cream corner accents,
using native source palette colors converted to linear render color. Existing
glaze shading and world registration remain, while ten nonoverlapping regions
partition each floor cell without changing support height. The exact existing
store/mart floor scope is retained; elevator live lids, roof paving, basement
and other rooms remain under their own owners. A new multi-scale area,
nonoverlap, color-contrast and walking-height test and fresh production
example build are queued. This paving revision has not yet received native
review and is not counted as finished.


The latest refreshed voxel test binary now includes and passes the store
angular-paver color contrast, area/nonoverlap and walking-height regression,
as well as the existing every-store-cell/reservation test (2). Its world-phase
and floor-area regression also passes (1). Native example rebuilds remain
live; their prior captures still show the earlier uniform honey floor and
must not be used as evidence for the new paving or pillar paint.

Violet's small2x2 marker drawings are gray carved stone in the native view,
but the canonical gym_plaque was brown timber/brass. Its four named roles now
use shared original authoring/checker colors: gray156, pale247/231/214,
shadow107 and ink57, converted to linear render color. Positions, normals,
indices, bounds and264 authored triangles are unchanged. Blender authoring
uses dedicated stone material names and retains individual carved strokes;
no Blender archive resave is claimed. The19-mesh checker passes7973 triangles
and verifies all four marker material colors. Violet's exact complete-source
selector remains the only owner of this small marker asset. Native palette
review is still pending the fresh example build.

Further inspection identifies a separate color gap: the league podium's
closed orb is still dark stone, though gym entrances use red/cream orbs. The
canonical loose parts are intact and contiguous: stone_dark0..132 is the
dedication plate,132..372 the orb; stone_light0..1296 is pedestal/cornice,
1296..1632 equatorial band,1632..1692 front seal. These were established by
read-only closed-component topology/bounds inspection, not guessed from
paint. The next correction should retain explicit authored head/pedestal
ownership and verify live source anchors before changing that orb finish.


The entrance-orb correction now retains named orb/pedestal ranges on the
canonical league_podium. The asset's positions, normals, indices, bounds,
ordinary paint and732 triangles are unchanged. Its authoring exporter records
orb, equatorial band and front seal from explicit object names; pedestal and
dedication parts keep separate ownership. The importer validates continuous
complete region ranges and remaps them through the retained occlusion mesh.
No new broad palette fallback is introduced.

The scoped live-color path covers complete elite_four_room statue drawings
in Violet, Blackthorn1F, Mahogany, Azalea and Goldenrod, and complete tower
entrance statues in Pewter, Cianwood and Ecruteak. Existing whole-drawing
selectors and exact source-profile conversions remain authoritative; other
map/atlas combinations retain ordinary cached paint. Both native atlas
drawings share validated anchors: orb [4,4] shade85, band/seal [7,3]255,
base [8,18]170, pale frame [8,16]255, dedication shadow [4,20]85. An in-memory
match against the native Violet2D preview confirms all512 source pixels at
scale6: head red247/82/49 and pink255/156/198 with pale247/231/214, pedestal
gray156/shadow107, outline57. No pixels or pack content were exported.

New Rust checks cover named cached geometry/material ownership and complete
source drawings, mutated identity fields, all8 map scopes, neighboring floor
and support preservation. They are queued alongside the fresh native build.
The card classifier now rejects out-of-range subtile coordinates before
indexing source art, so an altered drawing cannot panic or acquire a partial
statue. The19-mesh checker passes7973 triangles and verifies full semantic
region coverage. Native3D orb appearance remains pending.

The refreshed voxel binary now contains and passes the named gym-orb
assembly/cache regression. The broader source-scope regression is not present
yet and remains queued. The corrected gold-boulder union Cargo run also
completed successfully (2 tests). No fresh3D appearance claim follows from
those narrower tests; the live production example rebuild continues.


Fresh production captures in ignored target/indoor-source-palette-review
now visually verify all six paired rooms: CianwoodGym, BlackthornGym2F,
GoldenrodDeptStore1F, GoldenrodDeptStoreElevator, SproutTower3F and VioletGym.
The movable gym rocks remain ochre/gold, not gray; the store angular paving
now visibly matches its bright yellow/orange/cream source pattern. The cabin
floor retains that source pattern after final surface finishing. Sprout's
pillar now shows the dark shaft and gold trim, with gold guardian heads and
gray pedestals. Violet's small markers retain gray/pale stone coloring.
These camera positions do not fully expose the new entrance-orb assembly,
so its appearance still needs a focused capture.

Direct Rust regressions pass: two gold-gym boulder tests, two store-floor
tests, nine sprout tests, and one named gym-orb assembly test. The broader
gym-orb source-scope test remains pending in Cargo session36169; the newest
example build remains live in32703. Earlier queued jobs failed on a transient
footwear syntax error that is now fixed in the shared checkout. Successful
example10540 contains both the live pillar and named gym-orb palette paths.
These targeted checks and captures do not complete the full39-map goal.


A fresh focused VioletGym entrance capture now exposes both entrance orbs:
red heads, pale band/seal, gray dedication and pale/gray stone pedestal,
matching the source color separation. The small neighboring markers are
gray stone rather than brown plaques. The broader source mutation regression
remains live in Cargo session36169.

Mahogany's complete north-wall module now samples four live source shades
from its validated tile34 drawing: [7,2] dark outline, [1,1] pale trim,
[1,2] blue highlight and [0,0] blue inset. Closed slab geometry, panel
recesses,24-unit rise, source ownership, normals and footing are retained;
only that exact wall changes from fixed washed-out colors to source UVs
with neutral face shading. No unrelated atlas rooms gain this finish.
The wall regression now checks all four palette anchors as well as the
existing full-drawing rejection, adjacent ice and support checks. Test96275
and production example16135 are live and queued; native wall appearance
remains pending. Mahogany's cyan ice still needs its source-blue pattern.


Mahogany's ice finish now retains the complete live source UV motif and
palette on original procedural beveled slabs. Each16-pixel slab perimeter
has a shallow0.006-tile depression; internal8-pixel quadrant boundaries
meet continuously. The authoritative sliding height, grip-pad identity,
source exclusion/override logic and entry carpet remain unchanged. The old
generic cyan cloud finish has been removed; source-backed ice cannot enter
the generic painted-floor path. No source pixels were exported or stored.

The production geometry helper and its actual three-scale regression were
compiled in an isolated temporary Rust harness against the existing Bevy
math dependency and passed source-UV continuity, projected coverage exactly
once, unit/upward normals and bounded depth. The scratch harness was removed.
Both edited files pass Rust syntax parsing and the focused diff whitespace
check. This is narrower than the full room integration test: Cargo99811
mahogany_ and production example71643 remain live/queued, alongside earlier
wall-only96275/example16135 and orb source-scope36169. Native ice appearance
still needs a fresh production capture before claiming it verified.


The newly available gym-orb source-scope test exposed an incorrect fixture:
it asked the profile-free resolver to select tower12/13 and elite25/26
forms, although these are explicitly authored live-profile drawings. The
regression now supplies an exact scoped Document through the production
profile boundary, uses valid adjacent ground blocks and reports map/block
on a missing selection. All intended eight map scopes and per-cell mutation
checks remain present. The named assembly test still passes; the corrected
source-scope run is queued in74862. No production selector was broadened to
make the fixture pass.


Source review confirms every resolved PlanterLow in Azalea and Goldenrod
uses the same complete floral drawing07/08 over17/18, while PlanterRound
uses3e/3f crowns. The former previously rendered only stems/leaves. Its
existing authored planter now gains four original closed five-petal heads
(160 additional triangles) within its existing14-pixel envelope. Colors
sample its live flower drawing: pale[4,2],pink[3,1],shadow[2,1], with neutral
face shading. No broad animated-flower or palette fallback was added; full
fingerprint/reservation/profile checks remain authoritative, round shrubs
retain their own geometry, and the floor/footing are unchanged.

The production petal helper and actual three-scale regression passed in an
isolated Rust harness: each petal closes all12 edges exactly twice, source
UV shades are present, normals are unit length, and all geometry fits its
plot/height. The scratch harness was removed. The updated existing complete
planter regression distinguishes native floor backing from the new flowers;
stale source checks still reject both geometry layers atomically. Cargo36330
is live for gym_planter_ and production example15306 is live/queued. Native
flower appearance is pending and must be reviewed before calling this done.


The current voxel test executable now passes both gym-orb tests, including
the corrected live-profile source/mutation fixture. It also passes all three
Mahogany tests; binary freshness was explicitly verified from the new
source-motif quadrant-continuity and source-panel-shade assertion strings.
This closes the pending Rust integration checks for Mahogany ice/walls and
orb assembly/scope. The example still lacks the live-ice guard string, so
native ice/wall verification is not claimed from that stale build.

Arcade targeted runs pass14 arcade_ tests and7 casino_ tests (some overlap),
including east/west screen-to-seat orientation, stool plot sizing, whole
wall/prize-rack guards, opt-in material scope and custom-profile priority.
The existing Goldenrod Game Corner comparison was visually inspected again:
faces align with seats, cabinets are pink/cyan/gold, and source layout is
retained. This is static geometry evidence, not full interaction completion.
The pure current-source gym checker also passes Azalea, Goldenrod, Celadon
and Viridian mutation/atlas/phase/clipping/padding/native-gap guards.
The full blossom geometry regression is not in this executable yet; its
prior isolated check remains the only direct new-petal geometry evidence.
Cargo36330 and production15306 remain pending alongside other live queues.


The union Cargo gym-orb run36169 now finishes successfully with both actual
voxel tests; zero-match results from other crates are not counted as tests.
Current native renderer still lacks the new live-ice guard string. Production
example16135/71643/15306 and blossom36330 handles remain live, so they are
being awaited rather than restarted.

Saffron's actual source review still shows remaining visual gaps: washed-out
wall bands/floor pattern and entrance fixture finish need authored palette
work. During that review, malformed subtile phases were found to reach the
statue/pad index expressions. Both exact drawing selectors now reject all
coordinates outside0..4 before indexing. The native Saffron wall role and
planter-local selector likewise reject impossible columns/rows; valid wall
rows and complete2x4 drawing behavior remain unchanged. The three actual
saffron_gym module tests pass in an isolated Rust harness using the existing
render API type and a minimal presentation-shape declaration, and its
scratch files were removed. This is source-selector evidence only; full
union saffron_ session34803 remains live/queued. Both edited files parse
and pass the focused whitespace diff check. No broad fallback was added.


Saffron's exact horizontal09 wall module now resolves as its own complete
4x2 source drawing (04 upper row,14 lower row). It is selected before the
existing corridor network so only verified straight runs gain the finish;
other maps/junctions retain their existing behavior. The closed16-unit
volume retains the live dark upper panel, pale line and blue bands as one
complete front drawing, with a separate pale physical cornice. The facade
ends at the cornice rather than overlapping its front plane. Colors are
sampled from live source cells, without baked palette guesses. Footing,
teleport pads, statue drawings and neighboring floor stay unchanged.

A new full-drawing test checks8-cell ownership, live UV extents, physical
height/footing, wrong-room exclusion and all five source identity mutations
in every owned cell. Both files parse and pass the focused whitespace check.
The existing queued saffron_34803 and renderer15306 must be checked for
freshness when complete; native appearance remains unverified. This only
addresses complete straight wall runs; junction palettes and the floor
diamond pattern still need review.

The updated actual saffron_gym module now passes four isolated source tests,
including all eight valid blue-band phases, altered tiles, other block
identities and malformed coordinates. The temporary harness is removed.
Full geometry integration and fresh native wall appearance are still pending.


Fresh muted production capture target/indoor-source-palette-review/
Seafoam-current-compare.png confirms the SeafoamGym palette gap is current,
not merely an old screenshot: native gold/ochre floor, rock rim and boulders
still become gray/brown cave material in3D. Both2560x1600 native readbacks
were visible and inspected. The existing closed irregular cave geometry
should be retained; the next correction needs a Seafoam-scoped source
palette across its rocks/floor/enclosure rather than changing all caves or
replacing those masses with generic gym facets. This remains incomplete.
The capture used app-only preview mute and completed normally.


SeafoamGym now has a narrowly scoped live mineral-palette finishing pass.
It reads three shade roles from the actual opaque tile01 image in reviewed
plain shelf block24 (shadow[1,1],body[0,0],highlight[3,0]); source identity,
phase bounds, image size and alpha are validated. No native source pixels
or palette catalog are exported/stored. Its current2D render supports
ochre132/107/24,173/140/57 and pale214/198/140 rather than the old gray paint.

Only existing positive cave mineral tags2(enclosure),4(floor) and6(complete
low rocks) receive the source palette, with existing vertex luminance
retaining facet/weathering variation. Closed meshes, positions, normals,
indices, map-space texture coordinates and authoritative support stay
unchanged. The existing mineral shader/texture is preserved. Other rooms,
frozen tags, water, outdoor domains, actors and source-art UVs are untouched.
Absent/unverified palette images do not create a guessed replacement.
The production pigment helper passes an isolated Rust regression proving
three affected domains, preserved geometry/coordinates/alpha and untouched
frozen domains; its scratch harness is removed. The full integration test
also checks exact room/image/source scope and support, but remains queued
in67392. Production example95102 is live/queued; visual confirmation of
Seafoam's corrected3D palette remains pending.


The current voxel executable now contains and passes the full Saffron
wall/phase guards (8 saffron_ matches including unrelated Silph checks),
all3 Mahogany integration tests, both gym-orb tests and all4 gym_scenery
tests. The latter now proves new closed/live-color blossoms coexist with
original native ground and reject stale drawings/custom-profile claims.
Freshness is confirmed from the new Saffron label, live ice assertion and
closed-petal regression names/strings in the tested executable.

The earlier Mahogany wall queue96275 failed on a transient Seafoam test
fixture tuple error; that exact line had already been corrected in source
before later queues and the direct fresh Mahogany tests all pass. The
production example16135 now completes successfully (32m21) and includes
the live-ice and Saffron blue-band paths. A new five-map paired capture is
running for Mahogany,Saffron,Azalea,GoldenrodGym and Seafoam. No appearance
claim is made until those actual images are inspected. The additional
Seafoam image-role regression now covers changed live colors, missing
images, invalid image size and transparent role anchors; Cargo67392 is
still pending for full Seafoam integration.


All five new paired production captures completed and were inspected
(10 visible2560x1600 readbacks, no shader failures). Mahogany now visibly
retains its source blue diagonal ice motifs and blue/pale recessed wall
panels. Saffron's complete straight runs show the source dark/pale/blue
bands; pale junction walls and the floor diamond motif are still unfinished,
and small dark marks on the cornice need another close inspection. Seafoam's
inner walls/floor/rocks now render ochre, preserving the irregular cave
geometry. Its separate cave_enclosure surface remained gray, exposing an
additional owned material domain. The same scoped live-palette pass now
includes that enclosure and tagged cave vertices in the textured surface;
source-art UVs remain excluded. The integration regression covers those
extra surfaces and a genuine source-UV control; native recheck is pending.

The Azalea/Goldenrod captures contradicted the earlier floral assumption:
07/08 is the leafy family, while3e/3f carries the pale/pink/red flower heads.
The prior addition sampled green leaf colors, so it is not accepted as a
completed flower result. Leafy planters are restored unchanged. Complete
3e/3f planters now use the original named wooden container parts, with
clipped shrub stems/crowns excluded through validated semantic names. The
ordinary cache and original asset are unchanged, and the container keeps
its original full-assembly fit so it cannot stretch to plant height. Three
new closed stem/five-petal assemblies (144 triangles) use verified floral
anchors pale[4,4],pink[5,5],red-shadow[3,4],dark-stem[2,3]. The actual helper
and updated three-scale regression pass closedness, normals, bounds and
all four live shade anchors in an isolated Rust harness; scratch removed.
The exact source checker still passes all four affected plant/maze gyms.
New full integration/cache/ownership checks are queued in20839, and the
corrected production example50269 is live/queued. Fresh flower/enclosure
appearance is required before these corrections are accepted as complete.


The latest Seafoam live-palette integration executable passes both regressions,
including the separate cave_enclosure and tagged textured surfaces, genuine
source-UV exclusion, live image colors, alpha rejection and unchanged support.
Both gold_gym_boulder regressions also pass: named palette/geometry and moving
actor identity/map isolation. These are actual matching tests, not zero-test
Cargo targets.

Production example15306 finished successfully and contains the corrected named
flower container. A muted, explicit-image three-map paired capture completed
for Seafoam, Azalea and GoldenrodGym (six visible 2560x1600 images). All three
comparisons were inspected. Seafoam now has ochre outer enclosure as well as
inner walls/floor/rocks; the smaller rock silhouettes and size still need
quality work. Azalea and Goldenrod now show pale/pink flower heads on the
correct 3e/3f family while 07/08 remains leafy. Native rendering exercises the
validated filtered container successfully. Flower display density and the
missing native floor motifs remain appearance gaps, so these gyms are not
considered complete. New full flower cache/ownership tests remain queued;
existing test executable does not yet contain their new test names.


Azalea/Goldenrod rose floors and Azalea/Celadon turf now retain their exact
live source UVs through the existing floor source/phase/ownership/donor guards.
The generic green color fields and invented rose checker are removed from
the production finish path. Rose ceramic has shallow recessed module joints
(0.006 tile depth), wholly below the unchanged walking datum; green turf
retains its native pattern at that datum. Custom profiles, altered/cropped
plant backing, neighbor-UV permission and warp exclusion remain guarded.
The actual native-floor helper and three-scale/eight-phase geometry regression
pass in an isolated Rust harness: projected area, texture mapping, upward
normals, height bounds and shared crop edges. Scratch removed. Existing full
floor regressions now account for retained source artwork and unchanged
rejected cells. Full union test29660 is confirmed queued; native render
verification of the floor change remains pending. Prior captures are evidence
of the missing patterns, not proof of the revised floor appearance.


The fresh voxel executable now contains both corrected flower/cache test
names and passes all five gym_planter_ matches (four directly relevant
flower/container/ground checks plus the existing unrelated Saffron flat-card
check). The new floor geometry test is not present in that executable yet;
its full integration and render checks remain pending. Generic GymRose and
GymGreen palette substitutes are explicitly removed, so an accidental call
through the old solid finish path cannot silently restore the bland paint.


Saffron's dark cornice marks have a concrete geometry cause: both the body
and pale cornice emitted top faces at height16. The hidden body now ends
at15.75, matching the cornice underside, so the exterior top belongs only
to the pale cap. No source palette, footprint, drawing guard or footing
changes. The actual production slab/faceted-face helper and wall cap loop
pass a temporary Rust test at scales0.5/1/1.5: projected top area is exactly
one cornice surface and every top UV is its pale anchor. Scratch removed.
The full Saffron integration regression now checks that same area/UV property.
Full executable freshness and native visual confirmation remain pending.
Olivine's reviewed older capture still exposes flat pink perimeter artwork;
its current typed rock path already uses the live gold gym palette, so the
old gray-rock capture is not treated as evidence of current palette state.


Full union Cargo job36330 now exits0 (44m38), with five actual matching
gym_planter_ tests passing in the voxel target. The other union targets
correctly have zero matches and are not counted as additional verification.
The current executable still lacks the new live-floor/cornice markers,
confirming that their integration checks are pending rather than passing.
Floor job29660 and production examples95102/50269 remain confirmed live.


Olivine's perimeter now has an original closed rounded profile rather than
flat pink strip art. The exact source helper recognizes only 24/25 west and
26/27 east half-blocks, all four native rows, and the distinct 1a/1b terminal
row. Complete 2x4 groups use the existing coherent-phase, real-ground, custom
reservation and atomic ownership pipeline. Each section has36 triangles,
closed end caps and a rounded thirteen-pixel crown, wholly inside its owned
plot. All faces sample the live strip atlas with half-pixel border inset;
no palette substitute is added. Native floor backing and footing remain
unchanged. Full integration includes per-cell five-field mutations, wrong
room, crop/custom exclusion and unchanged support.

Actual source-helper tests (four total including three existing rock/statue
checks) pass against the cached render API. The actual production perimeter
helper and geometry regression pass at three scales: closed edges, outward
winding/positive volume, bounded plot/height, unit normals and owned UVs.
A read-only pack audit runs the actual helper across Olivine's source cells
and confirms16 complete strips /128 native WALL cells, with no NPC/warp
overlap. All temporary Rust harnesses are removed. Full union integration
job56724 is confirmed queued; native appearance remains pending.

Saffron job34803 exited101 on the newly added regression's reference to
near(), which belongs to a sibling interior module. The test now uses its
own direct tolerance comparison; production wall geometry was unaffected
by that compile error. Later union jobs remain live; no passing full Saffron
claim is made for the revised cornice until the current marker is present.


The refreshed full voxel test executable contains the live-floor and revised
Saffron cornice markers. All six gym_floor_ integration regressions pass,
including source masks, model backing, custom ownership, neighbor UV rejection,
crop/world phase, full motif coverage and unchanged support. The corrected
Saffron blue-wall regression also passes its one-visible-top/pale-UV check.
These are actual current production-module tests; their new native captures
remain pending until the production example is refreshed.

A fresh muted explicit-image Goldenrod roof capture was completed and
inspected (two visible2560x1600 readbacks). It confirms the live gray paving
and updated leafy/gold planters, and exposes beige generic stairs which
discard their native gray drawing. The current source audit reports512/512
modeled cells, but that is not appearance proof; its temporary JSON is removed.
Roof stairs now use live_source_slab for every closed step, preserving their
complete source plot/texture across both2x2 and2x4 flights. The non-roof
stair path is unchanged. No drawing fingerprints/ownership/warps/footing
are modified. The actual production stair branch, slab and source-UV helpers
pass a temporary geometry regression for both flight sizes at three scales:
full projected source area, five tread heights, exact UV/position mapping,
bounded recess and unchanged support. Scratch removed. Full integration82104
and fresh roof appearance remain pending.


Saffron's generic ceramic checker is replaced by exact native diamond
paving on closed slabs. Source masks cover only the nine reviewed block
phases (08/09/0a/0c/0d/0e/0f/24/36); pad/carpet/art cells are excluded.
Unverified phases no longer receive a generic corridor ceramic substitute.
The source atlas closes each floor lid at the original zero-height datum;
sides/underside use an owned background texel. The actual primitive and
three-scale test pass closed edges, signed volume, exactly one lid's area,
UV-position mapping and depth bounds. Actual source-helper tests and a
read-only live pack audit pass all1440 cells, selecting exactly944 ordinary
FLOOR cells and excluding every warp trigger. Scratch removed. Full
integration5067 remains queued; native appearance is pending.

The fresh full Olivine perimeter tests pass source/geometry, but fail their
custom-profile ownership assertion. This revealed a real priority gap: the
general dungeon resolver does not reserve all arbitrary live overrides.
The new families now explicitly receive the profile document and protect
resolving live plots. A protected cell rejects the complete Olivine strip;
Saffron paving also yields at protected cells without a ceramic substitute.
This is scoped to these new families; unrelated dungeon priority stays
unchanged. Olivine's integration now exercises override at every one of its
eight source cells. A passing corrected full ownership run is still required.
Seafoam union67392 finishes successfully (46m14) with both actual live-palette
regressions passing. Production example95102 is confirmed compiling, not
finished; no restart is needed.


Production example95102 finishes successfully (48m47). Its actual binary
contains the live Gym motif path, Saffron diamond paving and Olivine perimeter
labels, and application mute support is checked before launch. A muted
explicit-image six-map capture completes for Azalea, GoldenrodGym, CeladonGym,
SaffronGym, OlivineGym and GoldenrodDeptStoreRoof. All twelve2560x1600 readbacks
are visible; all six paired comparisons are inspected. Azalea/Goldenrod now
visibly retain the native green/pink motifs; Celadon's green turf is restored.
Saffron has the gray/pale diamond floor and its cornice no longer exhibits
the dark overlapping-top marks. Olivine's coral/pale rounded perimeter is
now volumetric, and its rocks are visibly gold. The roof stairs now retain
their native gray artwork instead of beige paint.

These captures verify appearance of the new paths in ordinary pack state,
not the later custom-priority fix or all room interactions. Corrected full
custom-owner integration remains pending. Remaining visible gaps include
Celadon's bland generic timber walkway and flat blue flowerbeds, Saffron's
pale vertical/junction walls, sparse Azalea/Goldenrod flower displays, and
Olivine's generic checker/stage finishes. Full39-map completion is still
unproven; static authored counts alone cannot establish it.


All six remaining old queue handles29660/20839/50269/56724/82104/5067
are terminal101 on the same shared park_flowerbed unresolved park_flowers
module reference; no observation timeout is treated as a terminal build.
The necessary module connection is repaired through a narrow owning-module
wrapper: modeled_exteriors::append_park_underplanting delegates to the
existing private producer, and park_flowerbed calls that wrapper. Park
geometry/data/ownership are unchanged. Union test62960 and example89352
are newly confirmed live after those terminal failures; integration remains
pending until the current corrected executable is available.

Celadon's native timber walkway now retains its live source grain through
the existing exact Gym floor guards and destination/donor ownership checks.
Staggered eight-cell boards / two-cell courses have shallow recessed joints,
entirely beneath unchanged support; rose ceramic and green turf retain
their proven paths. The old generic GymTimber paint/partition implementation
and palette substitute are removed. The actual helper and updated native
geometry test pass three scales, all board phases/crop joins, area, UV
mapping, normals and height bounds in a temporary Rust harness. A read-only
pack check invokes the actual source classifier across Celadon and proves
all216 selected timber cells are ordinary FLOOR, with no additional source
cells admitted. Scratch removed. New full integration verifies live grain,
unchanged support and custom floor exclusion; full/native confirmation is
pending. Current native helper marker is now “native Gym motifs and timber
require live source UVs”, preventing stale pre-timber renders from being
accepted as evidence.

Current rebuilt voxel test executable confirms seven Gym floor tests,
three Olivine perimeter tests (including custom ownership), the roof stair
test and three Saffron native floor tests all pass. Two movable gold Gym
boulder tests also pass for geometry, object movement and map scoping.
Cianwood's reviewed native comparison shows gold/ochre static and movable
rocks rather than a generic gray stone finish. The existing movable-boulder
test now additionally checks the exact decoded source sprite colors
(165,123,24), (198,148,57), and charcoal outline (57,57,57), including every
vertex and every named shade. That new assertion awaits union rebuild77477;
the preceding two-test result does not verify the newly added assertions.
Example89352 and union62960 remain live. Broader indoor completion remains
active, with the appearance and interaction gaps listed above.

Celadon's previously flat blue flowerbeds now have an original closed raised
planter, gold body/cream rim courses, inset soil, broad paired green leaves,
and dense five-petal blue blooms with pale centers. All colors sample the
matching live blue/green/gold source cells; no generic planter palette is
used. Six complete source bindings cover the long and short native drawings.
The current renderer's common tile animations exclude train_station; these
beds use its ordinary live atlas rather than an invented animation path.
A read-only actual-pack check invokes the production scenery resolver and
finds all20 complete beds /120 owned WALL cells. Every field mutation and
reserved-cell override rejects the affected whole bed; another atlas-sharing
map resolves none. The actual geometry helper plus actual face bake pass at
three scales, including closed/outward per-part geometry, bounded footprint,
height and source palette samples. Temporary harnesses removed. New full
integration tests check unchanged support, authored claims, stale append
rejection, and actual live-profile priority. Union76168 remains queued and
native appearance still requires a fresh binary; do not count the geometry
or pack audit as visual completion.

Saffron's reviewed comparison still had generic cream masonry on vertical
partitions and junctions despite the native dark panels/pale outlines.
These now resolve a dedicated live-source partition finish: closed dark
volumes, original top/front source UVs, and separate source-colored west/east
edge strips. Body/trim meet without overlapping top area. Complete16-cell
native block fingerprints protect every source field and custom ownership,
including the horizontal blue wall modules; cropped or changed blocks do
not receive substitute cream masonry. Existing UndergroundPath and store
basement finishes are preserved. The actual helper/face bake/closed slab
test passes three scales for source UVs, total volume/top area, footprint,
height and shading. Full source/priority/support tests are queued75960;
native appearance awaits a fresh binary with label
special:saffron-dark-partitions. Scratch geometry harness removed.

The actual-pack Saffron block guard audit now passes all42 complete wall
blocks /360 WALL cells and every per-cell source-field/custom-reservation
mutation. Temporary source harness removed. Example89352 has finished
successfully; fresh native flower review is being captured separately from
the new Saffron partition finish, whose renderer build remains to be checked.

The reviewed Goldenrod Gym native comparison still showed nearly empty
flower-planter soil. Azalea/Goldenrod round flower planters now carry six
larger blooms in staggered paired rows, replacing the sparse three tiny
heads. Petals retain the same live cream/pink/red samples; planter assembly,
source guards, foliage-only planter family, claims and support are unchanged.
The actual updated bloom helper and face bake pass the existing closed-volume,
normal, UV and footprint/height tests at three scales in a removed temporary
Rust harness. Each planter now has288 flower triangles; full integration
assertions are updated. Native appearance and rebuilt full assertions remain
pending, and the old144-triangle run is not evidence for this density change.

Native flower-density review completed successfully for Celadon, Goldenrod
and Azalea, with six visible2560x1600 screenshots and all three comparisons
inspected. Celadon now visibly has blue flower volumes, green leaves, gold/
cream raised pots and its live brown timber grain; the flat flower art and
beige walkway are gone. Goldenrod/Azalea visibly have fuller pink/cream
flower clusters while retaining leafy planter variety and native floors.
These are static appearance checks, not proof of puzzle/gameplay completion.
The example used predates Saffron's new dark partition finish. New example
61440 is queued after terminal successful89352; native session4803 finished0.

Full-scope native review session19652 finished0:35 additional map comparisons,
70 visible2560x1600 readbacks, every comparison inspected. Combined with
the three flower Gym comparisons and fresh Saffron session54445 (finished0),
every map in the39-map scope now has inspected static evidence. This does
not establish full completion: the captures identified further material
errors, and the new Fuchsia/Blackthorn corrections below are not in them.
Saffron's new dark partitions/pale outlines and blue junctions are visibly
present; generic cream partition masonry is gone. Example61440 finished0.
Union62960 passed7 Gym floor tests;77477 passed2 movable-boulder tests with
the new exact RGB assertions;76168 passed2 Celadon flowerbed tests;75960
passed2 Saffron partition tests. Fresh direct gym_planter filter passed5
matches, including the enlarged288-triangle clusters.

Blackthorn2F's partition mesh now samples its exact gray inset/body/pale
coping shades from the authenticated2x2 native drawing, replacing cream
paint. All220 original triangles, positions, normals, bounds and the six
whole-source object bindings are retained. Semantic source anchors are
recorded in the original editable asset and authoring script. The expanded
special-room model checker passes all23 meshes /28472 triangles. New full
dispatch regression verifies native UVs, cached geometry and unchanged
support; rebuilt integration/native confirmation remains pending.

Fuchsia's broad lab Oak finish was visibly producing beige patches against
gray concealed-barrier artwork. Fuchsia now has a dedicated source-only
gray board finish for both ordinary tile10 and concealed tile37. Exact
block/phase masks exclude north walls, statue drawings and entry warps.
Both use the same shallow native board geometry under the original datum;
neither collision nor guessed floor colors selects presentation. The lab
palette fallback is explicitly excluded in this map, and custom profiles,
raised surfaces and redirected donors cannot acquire this finish. An actual
pack audit invokes the exact source helper:656 gray source cells, including
144 concealed-barrier cells, foreign atlas/block/tile and invalid phase all
rejected. Scratch removed. Full gray-floor/support/reservation test62153
is live; latest example3560 queued. Neither older full-scope screenshot nor
the source audit verifies the pending rendered finish.

A temporary older store coverage audit found no unmodeled object source
cells on2F/3F/4F/6F (only floor), while5F still has8 relief source cells.
Some apparent flat store displays in the native screenshots therefore need
an ownership/reveal/backing investigation rather than an assumed new model
binding. The temporary report was removed. Remaining review gaps include
store5F central display/materials, reveal/backing appearance, Seafoam's small
rock silhouettes, Olivine stage/runner finishes, additional guardian/fixture
palettes and dynamic interactions/animation verification. Goal remains active.

### Palette review: native floor and partition evidence

The fresh native image captures in ignored `target/native-palette-review/`
show Fuchsia's gray native boards throughout the concealed maze, without the
previous generic wood patches. Its full Cargo regression passed. Blackthorn
2F keeps its gold movable rocks, blue paving, gray partition insets and pale
source trim. These are static render checks, not interaction proofs.

The two Cianwood gold-boulder palette regressions passed against the current
library. The full-scope source/render comparison also shows gold static rocks;
the static rock silhouette remains a quality gap. The Blackthorn sampled-model
regression initially incorrectly expected an empty solid mesh despite the
existing stone floor underlay. It now compares that underlay separately from
the sampled partition geometry; its corrected full Cargo run is pending.
Store display appearance and the broader remaining work above stay open.

### Seafoam boulder volume and department-store display identity

The previous palette checks passed, but the reviewed Seafoam render still
showed narrow fractured wedges in place of broad source boulders. Seafoam-only
complete 2x2 cave-rock placements now use the authored boulder volume at 12
pixels high and 14 pixels deep. Native rock material anchors feed the textured
model, while its existing floor datum, mesh joins and player support remain
unchanged. The full regression and new native render are pending.

A focused, app-muted Goldenrod 3F capture at (13,5) proves the red/gray display
was already volumetric but incorrectly replaced by book merchandise. The
exact mart $13 display path now emits closed cabinets with their separate
native fronts, rather than retail bookshelves. A new regression checks both
Goldenrod and Celadon bindings, complete native front UV coverage, outward
faces, enclosed volume and full front area. Full tests and fresh captures are
pending; cabinet contents and material quality still require visual review.

The corrected Blackthorn partition regression passed in full Cargo. The
original failure came from its incorrect expectation about the unchanged
stone floor underlay, not from a partition material failure.

### Native evidence and Mahogany source finishes

Seafoam's full boulder regression passed. Fresh app-muted native comparisons
in ignored `target/store-rock-volume-review/` were inspected for Seafoam,
Goldenrod 3F and Celadon 3F. Seafoam's eight rocks now read as broad ochre
boulders instead of spikes. Their repeated silhouette and the nearby cave
boundary cutaway still need quality review. A focused Goldenrod capture at
(13,5), `target/store-cabinet-native-front-compare.png`, confirms the red and
gray fronts on closed cabinets. These simpler casings preserve identity but
are not proof that merchandise details are fully modeled.

Mahogany's grip squares previously discarded gray source speckles in favor
of blue ceramic pad geometry. They now retain their own native UVs over small
recessed facets, with no geometry above the sliding/stopping datum. Redirected,
raised and customized source samples are excluded; the generic blue grip
finish was removed. Mahogany partitions now sample their three materials from
the complete gray/pale native source drawing on the unchanged authored volume.
Existing regressions now check native sampled geometry/material anchors and
source coverage. Full Mahogany tests and fresh native capture remain pending.

The newly linked voxel library's `store_electronics_preserve_both_native_fronts_on_closed_cabinets`
regression passed directly (both store maps, full native fronts, closed volume).
The union Cargo invocation is still finishing the other package builds. The
23-model special-room source checker also passed (28472 original triangles).

### Cabinet relief geometry

The focused cabinet render established native color/identity but still had
plain box fronts. The exact mart $13 path now has four recessed bays, shelf
lips and a beveled frame. Shared front knots meet the unchanged casing edges;
geometry remains within its original footprint and height. An independent
Rust check extracted the actual emission/shading helpers and verified both
source halves: positive closed volume 4541.797, projected native front area
479.9998, outward faces. Mahogany grip geometry passed source-area and maximum
height checks at scales 0.5, 1 and 1.5. Temporary harness files were removed.

The earlier store-cabinet regression finished successfully in full Cargo.
The revised cabinet regression, full Mahogany regressions and current native
example build are still pending; fresh visual review is required for relief.

The full union Cargo Mahogany run subsequently passed all three regressions:
source-scoped slide/stop/warp identity and geometry, custom override/entry
carpet/footing preservation, and native wall/partition material geometry.
Fresh native visual verification still depends on the live example build.

### Cabinet relief / Mahogany visual review; Olivine floor correction

The revised cabinet regression passed in full Cargo. Fresh app-muted native
captures `target/store-cabinet-relief-compare.png` and
`target/mahogany-native-materials-compare.png` were inspected. The cabinets keep
red/gray fronts on recessed shelves; Mahogany keeps native gray speckled stop
patches, blue/pale ice and source-sampled partition insets. These checks prove
static appearance, not the unresolved full interaction/render-state audit.

Olivine's native gold rough floor and bright runner had been replaced by muted
ceramic squares/checker geometry. Its exact source-floor path now uses native
UVs on recessed rough ground and beveled runner surfaces. Generic Olivine floor
palettes and checker emission were removed. Changed/raised/redirected source
samples are excluded; player footing remains untouched. The new regression
covers native UV mapping, entry-carpet exclusion, custom ownership, redirected
paint and raised geometry. Full Olivine tests/native render remain pending.
The raised Olivine stage still needs its distinct native material pass.

### Olivine stage native materials

The prior Olivine floor jobs no longer had available process handles; a process
inspection found no matching Cargo process. The newly linked voxel library
passed all 12 Olivine regressions directly, including the new native-floor
scope/UV/override checks. A fresh app-muted floor comparison at
`target/olivine-native-floor-compare.png` was inspected. Its native bright
runner and rough gold floor are restored; the stage in this capture still uses
the preceding generic material implementation.

The separate whole-drawing stage emitter now resolves all seven named model
materials against its own authenticated 12x8 source drawing. Deck stones keep
live repeating native grain, retaining stones use native gold/ochre/outline
texels, and steps/lips use native gray/pale texels. Original geometry, normals,
2528 triangles, footprint, claim masks and 4/8-pixel support heights remain.
The metadata applies only to the named stage and requires all seven roles;
unknown/missing roles reject sampled emission before mutating the mesh.
Tests now compare the complete native emission to the original model and
check material anchors, cache immutability and rejection of unrelated models.
Full stage tests and native stage render remain pending.

### Game Corner native floor surfaces

The three Game Corner rooms still used a generic pale/gray checker finish in
the full-scope render despite the source's boxed-tile motif. Their exact
floor identities now retain live source UVs on beveled physical surfaces.
The generic arcade palette and checker emission were removed. Prize-room
floor underlays use the same native surface helper and retain their existing
closed side/underside geometry. Raised, redirected and customized floor
samples are excluded; slot orientation, stool sizes, ownership and footing
are unchanged. Regression changes check native UV/area/height plus all three
maps' custom ownership and source guards. Full arcade tests/native review are
pending.

The latest Olivine voxel library passed all 13 relevant tests, including the
new seven-role stage material/cache/geometry check. The union Cargo command
stopped earlier on the unrelated Battle Tower roof-art test, which loads a
missing legacy `vendor/pokecrystal` PNG. No legacy assets were restored or
added. Current native example build remains live; stage appearance is still
unverified.

A read-only actual-pack check compiled the current native floor classifier and
passed for 602 Goldenrod Game Corner, 546 Celadon Game Corner and 88 prize-room
floor cells. Every selected cell had native FLOOR permission; altered atlas,
metatile, tile and invalid phases were rejected. Temporary source fixtures and
executables were removed. This proves identity scope, not rendered completion.

### Native arcade/stage review and tower rock highlights

The current native example build completed. Fresh app-muted captures in
`target/arcade-stage-native-review/` were inspected for Olivine Gym and all
three Game Corner rooms. Olivine now keeps the gold patterned stage, ochre
retaining stones and gray/pale steps. Arcade floors retain the complete native
boxed motif. These are static visual checks; the full interaction audit and
remaining store material work are still open. The arcade Cargo regression
job remains live.

The fixed tower-atlas Gym rocks previously spread the source's palest cream
glint across broad highlight facets. They now sample source shade 1 at [8,1]
for those facets, preserving shade 2 at [8,6] for their bodies. This matches
Cianwood's lighter gold rather than washing the rocks out. Selection remains
the authenticated complete Gym drawing; cave, Seafoam and Olivine sampling
are unchanged. A read-only actual-pack check confirmed both anchors' shades
and four complete Cianwood drawings. The existing source-scope regression
now checks emitted highlight/body UVs and unchanged footing in all three
Gym rooms. Syntax and diff checks pass; full tests and fresh rock rendering
are queued behind the shared Cargo build lock.

The freshly linked pre-highlight-change voxel test executable passed all 15
arcade regressions and the complete-floor-area regression. Its two movable
gold-boulder palette/actor regressions also pass. The source-scope test in
that executable predates the fixed-rock highlight edit and is not evidence
for the new assertions; their Cargo build remains pending.

The voxel test executable linked after the highlight source edit passed the
updated `gym_stone_palette_requires_complete_room_scoped_drawings` regression,
including emitted body/highlight UVs, omission of the cream highlight,
source/map guards and preserved footing. Fresh native rendering remains
pending the queued example build.

The full union Cargo arcade command completed successfully: all 15 relevant
voxel regressions passed, with no matching tests in the other selected
packages. The updated rock scope/UV regression passed in the newly linked
voxel executable as recorded above. The standalone native example rebuild
is still compiling; no screenshot of the new highlight has been claimed.

### Rock highlights reviewed; fifth-floor display native material pass

The full rock source-scope Cargo regression passed. Fresh app-muted native
comparisons at `target/gold-rock-native-review/CianwoodGym-compare.png` and
`PewterGym-compare.png` were inspected: the fixed rocks retain ochre bodies
and lighter gold facets, with no broad cream patches. Movable Cianwood
boulders keep their existing gold palette. This is static appearance evidence;
push-script completion remains part of the wider interaction audit.

Both fifth-floor department-store display islands still used generic cream,
copper and brown paint despite the source's gray cases and blue glazing.
The complete authenticated island emitter now sends the original model to
the textured mesh and samples its own live source drawing for all ten named
material roles. Case/drawer gray, pale enamel, dark trim/pulls/plinth, blue
glass, pale reflections and recessed glass rims have distinct verified
anchors. Geometry, fitted normals, indices, source claims and clerk floor
insets remain. Generic display glaze application was removed. Named-material
metadata is restricted to the canonical display and requires every role;
unknown or missing roles reject sampled emission without mesh mutation.
Regressions compare canonical geometry, live anchor UVs, normal shading,
opacity, footing, cache immutability and source/missing-material rejection.
The broader atomic island test also inspects the textured geometry so its
floor-inset and fitted-bounds checks cannot become vacuous.

The actual-pack department-store source/geometry checker passed, including
both complete 5F islands (112 owned cells, all 16 floor-inset cells preserved).
A separate read-only pixel audit verified every new anchor's shade in both
Goldenrod and Celadon 5F. Syntax/diff checks pass. Full display tests and
fresh native case rendering are queued; these materials are not yet claimed
visually verified.

A focused actual-pack check of the eight central 5F inset cells per room
found tile 3a in blocks 1a/1b, with FLOOR permission in every cell. These
are the deliberately preserved clerk-floor opening, not evidence of eight
missing furniture cells. The older generic coverage report is therefore
insufficient to justify filling this opening with geometry. Source masks and
script access remain authoritative.

The voxel executable linked after the store material and atomic-test edits
passed both new display regressions and the full atomic ownership/bounds/
inset test. Its complete `department_store` filter passed all 21 tests,
covering exact drawings, crops, custom/reserved ownership, windows, lift
apertures, stairs, floors, seating and the display changes. Current fresh
native display appearance remains pending.

### Basement wall native surfaces

The basement comparison still showed broad cream masonry caps and generic
blue trim above the source's dark chrome/blue wall drawings. The exact
basement passage-wall emitter now retains its original masonry/front model
geometry but transfers it to the textured mesh. Caps sample their own dark
source texel; exposed faces keep their own complete live native panel
artwork. The invented cream paint and additional generic blue trim were
removed. Other passage rooms retain their existing emission. Complete wall
source guards, crops, claims, underlays and footing are unchanged.

A new regression compares emitted positions/normals/indices to the original
wall geometry, checks own-cell UV bounds and cap anchors, native white face
shading and unchanged footing for all eight wall cells of a complete block.
Syntax/diff checks pass. Full basement tests and native appearance remain
pending; the active fifth-floor native build and display test are still live.

A read-only actual-pack audit checked all 492 basement wall source cells
across the eight exact wall tile identities. Each new cap anchor is native
shade 3 on palette 3; the source's dark/blue wall palette is retained for
every selected tile. No source catalog or raster copy was written. This
verifies the native anchors, not the pending rendered appearance.

The voxel test executable linked after the basement edit passed all three
`store_basement` regressions: complete/cropped source guard behavior, distinct
lift/stair geometry and the new native wall geometry/palette/footing check.
The union Cargo command remains live, as does the queued native example
build. Fresh basement and fifth-floor appearance still require review.

### Native store material review and complete-scope audit

The full display Cargo filter passed both regressions; the full basement
filter passed all three. Fresh app-muted comparisons in
`target/store-native-material-review/` were inspected for both 5F rooms and
Goldenrod B1F. Native gray/blue case materials and dark/blue basement walls
are visible. The 5F cases still had broad plain lower tops, motivating the
separate four-section glazing refinement below.

The freshly built production coverage audit passed all 39 scoped maps with
zero mesh/footing errors. It counted 11,323 authored cells; remaining legacy
classifications were 15,267 flat, 440 plane, 24 relief and 194 water cells.
The 5F relief residuals are the previously verified eight clerk-floor cells
per store. Celadon Gym's eight relief cells are its exact 4x2 native exit
carpet at (8..11,34..35), all WARP_CARPET_DOWN and aligned with its two city
warps. Cerulean's 194 water cells and the tower/Violet plane classifications
are presentation surface evidence, not new prop coverage. The audit itself
requires dynamic-state fixtures and actual-palette visual review; it does
not certify completed rooms. Its temporary reports were removed.

The canonical 5F lower display now has four closed recessed glass lids,
with separate diagonal reflections and glints, matching the repeated native
fronts. The supporting body, overall 8px height, footprint and clerk inset
remain. The model grew from 1,212 to 1,452 triangles. The deterministic
geometry/source checker passes: 204 closed positive-volume kit parts, 4,656
triangles, complete native bindings and all 16 inset cells preserved.
The newly linked voxel library passed all 21 department-store regressions
with the four-section model. Editable Blender-source regeneration and a
fresh native render of this geometry are pending.

The four-section display's full Cargo filter passed both production/native-
metadata regressions. The editable department-store Blender source was
regenerated headlessly and stored through the existing source manifest. Its
eight embedded runtime SHA-256 identities match the canonical mesh JSONs;
four recessed lid objects were verified in the saved scene. The full kit
geometry/actual-pack/source checker passes. Temporary authoring runtime,
installer, mount and verification outputs were removed after verified eject.
The revised native example build completed; its two 5F captures are live.

The external pack declares scripted block changes in four scoped maps:
Blackthorn Gym 1F (filled boulder gaps), Celadon store 6F (roof-stair callback),
Goldenrod B1F (cleared crate layouts) and Goldenrod roof (sale setup). Existing
base-map audit evidence does not verify these state combinations. B1F's
replacement block 0d is all native FLOOR; the roof replacements and
Blackthorn replacements require a separate production mesh/state audit.
No scripted progression or interaction result is claimed from these source
metadata reads.

The revised four-section display captures completed and were inspected for
Goldenrod and Celadon 5F at `target/store-glazing-native-review/`. Four blue
glass lids with native pale reflection/glint colors now break up the lower
case top and align with the repeated blue source facade. Original height,
clerk opening, source footprint and opaque support remain. These native
views verify this material/geometry increment, not the still-open scripted
block-state and wider dynamic interaction audit.

### Declared scripted-block geometry audit

The production headless audit now accepts `--script-block-variants` and
keeps its base-map reports separate from substitution combinations. It
deduplicates identical declarations, uses the core coordinate stride,
rejects conflicting alternatives/unaligned or out-of-map changes, and
rebuilds every bounded combination from the pack's own layout and palette
bank identities. Each variant runs the production mesher and legacy source
classifier, validates complete finite footing/audit vectors and reports
authored families or residual source classifications in edited cells.
No script is executed and no progress/input state is advanced; reachability,
actors and actual-palette state views remain separate evidence requirements.
Syntax/diff checks pass. The example build is live; the declared-layout
results have not yet been claimed.

The declared-layout audit example build completed successfully. The current
39 base maps and 26 non-base substitution combinations ran through the
production mesher with zero mesher/finite-footing errors. The seven Blackthorn
combinations retain authored filled-rock crossings and lava channels; Celadon's
hidden roof stair retains the authored store wall; all 15 basement variants
replace crates with the declared native floor. Goldenrod roof's three non-base
combinations have no authored claims in the edited cells (all classify flat),
so explicit rooftop sale-state coverage remains unfinished. This result is a
coverage gap, not justification for a generic fallback. Reports were temporary
and removed. Actual script execution, actors and native views of changed states
remain separate outstanding verification.

### Rooftop sale cartons

Added exact complete 3x4 and 3x2 source-object fingerprints for the two
Goldenrod rooftop sale drawings, plus their changed native rail and floor
cell signatures. Cartons have sealed six-sided depth, separate folded lids,
center seams and small front price labels. The renderer samples the complete
object's live atlas, including its VRAM bank. The stacked body uses native
brown palette shade 1; the smaller body uses native red palette shade 2;
both use native pink lids, dark seams and pale labels. These exact bank-
resolved palette/shade roles were checked directly against the external pack.
The first raw-PNG inspection missed the bank mapping; its temporary images
were removed and all final anchors use the bank-resolved drawing.

No generic crate replacement or map-wide fallback was added. Geometry stays
inside each claimed plot and does not mutate footing/collision. A multi-scale
regression checks closed six-sided cartons, bounds, all four native roles and
unchanged footing. Syntax/diff checks pass; freshly compiled geometry tests,
updated production variant coverage and a native changed-state render remain
pending. Do not count this pending increment as verified visual completion.

The location renderer now accepts `--preview-flag <verified-flag>` for one
explicit map/screenshot. Its location-tester-only configuration seeds the
existing smoke flag API before normal map callbacks settle, requires a fresh
disposable location session without a quick-save path, and includes the flag
in comparison identity. The sale fixture will use the actual pack engine flag
`ENGINE_GOLDENROD_DEPT_STORE_SALE_IS_ON`. This is a visual-state fixture, not
story progression evidence or a second input dispatcher. Both examples are
queued for a fresh build behind the live geometry test.

The final carton multi-scale geometry/support regression passed through full
Cargo. Both examples rebuilt successfully. The production roof audit now
claims every edited cell in all three substitution combinations: 18 carton
cells, 8 parapet cells and 6 floor cells when both edits apply; no residual
unmodeled classification remains. All footing vectors are complete/finite.
A muted native sale-flag fixture capture is live; it is not yet inspected.

### Gym-wall joins and furniture scale feedback

The user specifically reports gym wall and countertop connectivity, plus
bad register/chair sizing. The tower side-wall southern returns ended at
source row 2 while the adjoining horizontal facade folds onto row 4, leaving
a 13-pixel physical corner gap. Complete neighboring facade guards now let
the side geometry reach that wall. Claims remain the original four cells,
footing is unchanged, and reserved/custom neighbors prevent extension. A
regression covers the four corner drawings and every reserved lower-wall
cell; its fresh full Cargo build is live. This affects the verified timber
gym/tower scopes only, not an inferred map-wide wall fallback.

Registers now occupy 65% of their previous width/depth and 70% of their
previous height above the unchanged countertop datum. Store seating now has
a 2-pixel cushion atop a 5-pixel frame instead of a 6-pixel cushion atop a
6-pixel frame, with a centered 85% footprint. Native pink materials and
source claims remain. Placement metadata and existing seat-height regression
expectations match the actual 7-pixel seat height. Fresh furniture tests and
native size/connection views are pending. Countertop joins still require a
concrete repair and verification; this feedback has not been declared fixed.

The muted sale-flag capture completed and was inspected. Its visible west
side still shows the base seating layout; it does NOT prove the sale block
substitutions were applied. The flag-only fixture therefore remains
insufficient for native sale-state verification. Before claiming a rendered
sale result, inspect the authoritative callback/layout state and arrange the
fixture before entry callbacks. No native carton visual success is claimed.

### Continuous store service tops

The store-only counter path now selects the canonical reception cabinet's
three named body roles and omits its separate enamel top and raised timber
privacy lip. All five material roles must be present; unknown roles reject
before appending. Cached generic/arcade counter geometry remains unchanged.
The selected body's referenced height fits to 9 pixels, then a closed map-
space top spans the exact cabinet plot from 9 to 10 pixels. Straight and
rotated return bays therefore meet on the same plane, without rounded end
trenches or a privacy lip cutting across the neighboring worktop. Registers
sit at the shared 10-pixel datum. A multi-scale regression checks continuous
coverage across straight and corner bays and the smaller register's bounds.
The full department-store Cargo filter is live, not yet passed.

The newly linked voxel test binary passed the southern gym-wall return
regression and both existing store-seat regressions (native material/height,
support unchanged, sixth-floor right/left matching). The full wall-return
Cargo command still waits on shared build work; native renders of the revised
wall joins and furniture proportions remain outstanding. Syntax and diff
checks pass for the current countertop code.

The newly linked voxel binary passes all 22 department-store checks, including
continuous-top area at three scales, reduced register bounds, original clerk
openings, opaque islands, finite original support and complete source guards.
The actual-pack/model/editable-source checker also passes (1,900 basement
crop checks, all store architecture/stair/lift/counter bindings, both 5F
islands and all 16 floor-inset cells). These are current source/geometry
results, not native visual approval.

The store body adapter now rejects non-positive/invalid fitting heights and
flat selected bodies before writing. A new named-role/cache regression checks
all five missing/unknown roles and a changed fixture name, atomic rejection,
selected 9-pixel referenced body height, and unchanged cached generic model.
That latest regression has not yet appeared in the linked test binary. Full
Cargo sessions 20680/23352 and the updated two-example build 33787 remain
pending; the latest material-rejection test and native wall/counter/furniture
views must be verified before this increment is called visually complete.

Full Cargo finished successfully for the southern gym-wall return regression
(session 20680) and all 23 department-store tests (23352), including the new
atomic named-role/cache body adapter check. The current two-example renderer
build (33787) is still live and has not produced fresh native images.

The flag preview now settles the constructor's initial callbacks, then reruns
the production MAPSETUP_WARP callback path with the seeded flag. For the roof
sale fixture, initialization requires the actual saved block overrides to
match every declared sale edit from the pack before presenting the view.
It never installs a block directly. A dedicated external-pack regression
compares the base and sale sessions, verifies both real block substitutions,
and requires the sale clerk event flag. This ignored external-pack regression
is explicitly queued with the known pack; its result and native callback-
state evidence are pending. Normal starts do not enter the fixture refresh.

### Fresh native wall/counter/scale review

The current two-example build finished successfully (33787). App-muted native
2D/current-3D comparisons were captured and inspected for CianwoodGym,
EcruteakGym, both stores' 1F/2F/4F, and all three SproutTower floors in
`target/wall-counter-scale-native-review`. Timber southern returns meet the
folded facade; straight and return service tops meet at a common height.
The 1F pink seating has thinner cushions and exposed legs, and the reduced
register rests on the worktop. Native bright floor, blue wall glazing and pink
seat palettes remain. All 23 store checks and the southern-return regression
passed through full Cargo; current targeted diff checks pass.

The explicit external-pack sale-callback regression passed (69853): both real
rooftop block substitutions differ from base and the clerk event is enabled.
A fresh app-muted `target/roof-sale-callback-native-review-compare.png` was
captured and inspected. Unlike the earlier flag-only capture, its 2D side now
shows the actual sale blocks and its 3D side shows the modeled stacked sale
boxes with brown/coral bodies and pink lids. This verifies the fixture's
visible state, not story reachability.

A current production audit again verified all 39 scoped base maps plus 26
source-declared substitution combinations with no mesher errors. Counts are
unchanged: 11,323 modeled cells, 15,267 flat surface cells, 440 plane cells,
24 low relief cells and 194 water cells. These counts include deliberately
retained floors, tower voids and water surfaces; they are not blanket full-3D
completion evidence. The temporary audit report was removed automatically.
Broader dynamic gameplay/reachability review remains outstanding.

Both sixth-floor seating comparisons were also captured app-muted and
inspected. All 13 current wall/counter/furniture scope comparisons completed;
reduced seat height and centered footprint remain consistent on left/right
sixth-floor layouts. QA images stay ignored under target; no source fixture
exports, content packs or compiled build products are deliverables.

### Changing indoor-layout native review

Fresh app-muted production callback fixtures were captured and inspected for
Blackthorn Gym 1F base plus each of its three filled crossings, and Goldenrod
B1F base plus card-key/layout-2/layout-3 flags. The basement drawings correctly
remove the relevant crates and leave native floor. Native captures are ignored
under `target/indoor-callback-state-review`. The first requested gym position
(4,9) was rejected as non-walkable; the verified positions (5,9) in the gym and
(9,9) in the basement completed. An explicit external-pack regression now
checks all six flags against the pack's declared actual block overrides; its
full Cargo result is pending, not assumed from the captures.

The gym state views exposed a remaining palette mismatch: the upper blue
stone fixtures and gold callback crossings still used generic gray material.
The complete verified 4e/4f/5e/5f boulders now enter the existing named stone
palette adapter, sampling blue body/highlight shades at (2,3)/(2,2). The 44
crossing's own palette supplies dark joints at (1,1), pale facets at (0,2)
and medium stones at (6,7). In-memory source PNG/metadata reads confirm shade
identities 3/1/2; no source image/catalog is exported. Crossing geometry stays
10 closed slabs, 360 indices, top at the walking datum and depth below it;
source guards/claims and footings stay unchanged. Extended regression checks
the textured UV roles and exact volume/depth ownership. Syntax/diff checks
pass; full Blackthorn Cargo checks and fresh palette renders are pending.

The six gym/basement callback fixtures passed through full external-pack Cargo
(73851), verifying real declared replacements and a difference from each base
session. The original gym source checker was stale after adding Celadon's
BlueFlowerBed kind. It now checks six explicit families and all 120 native
blue-bed cells, alongside 168 green hedge cells; no catch-all or skipped family
was added. All four actual-pack source cases passed, including complete,
custom, altered atlas/phase, crop, padding and maze-gap guards (41494).
Original model checks also passed: 20 gym assets / 364 closed parts / 6,220
triangles with all 16 wall datums, and 23 special-room assets / 28,472 triangles.
The Blackthorn material regression (97879) and two-example rebuild (35041)
remain live; native verification of the corrected blue/gold materials is still
required. Preserve and poll these handles; do not relaunch due to silence.

### Blackthorn native palette correction follow-up

The first rebuilt native review (`target/blackthorn-native-palette-review`)
confirms blue standing rocks. It exposed that the crossing's 44 corner has
palette 0 (gray), so the initial crossing anchors still rendered gray. The
adjacent native 45 tile has palette 5 (gold); crossing dark/light/body samples
now use global drawing coordinates (8,0)/(9,2)/(10,6), verified in memory as
bank 0, native shades 3/1/2. The boulder 4e tile is palette 6, bank 0, shades
2/1 at (2,3)/(2,2). Do not claim gold crossing visual approval from the first
review. A fresh renderer build is live (49970) for the corrected anchors.

The prior Blackthorn filter ran 11 tests: ten passed; the new regression's
360-index expectation was wrong because the existing faceted-face helper
splits each face into four triangles. It now requires the original 720
indices (10 closed boxes / 240 triangles), without any geometry reduction.
A rerun is live (44641); the final gold-anchor-specific filter is separately
queued after the newest build. Fresh palette renders and the final test
remain required. Current complete-scope audit (3225) passes all 39 base maps
and 26 declared combinations with the same residual surface categories and
no mesher errors; its temporary report was removed.

### Complete-scope current native review and remaining finish work

The first Blackthorn rerun passed all 11 targeted checks (44641). The renderer
with corrected palette-5 crossing anchors rebuilt successfully (49970), and
the final anchor-specific external Cargo filter passed (88715). All three
fresh app-muted crossings were captured and inspected in
`target/blackthorn-gold-crossing-native-review`: their materials are now gold,
while the standing rocks remain blue. The previous gray-crossing images are
historical evidence only. No support, collision, source ownership, scripted
block substitution or saved state changed.

The app-muted full native batch (92976) completed successfully. All 39 paired
2D/current-3D images in `target/indoor-complete-scope-native-review` were viewed:
17 gym maps, 16 department-store maps, three Game Corner maps and three Sprout
floors. This verifies current room presentation across the full requested
scope; it also exposes remaining finish issues, so the goal remains active.
Current source/geometry audit still covers 39 maps and 26 variants; neither
that audit nor model counts erase the following visible shortcomings.

* Goldenrod's blue north gaming table is modeled as two generic reception
  counter sections with plain tan tops. Native blocks 2f/30 lie at block
  coordinates (5,0)/(6,0); the table source plot spans cell columns 22..25,
  rows 1..3, with stools to either side. Replace the complete guarded pair
  with one framed table, gray supports and native blue playing surface. Do
  not retain a generic half-counter fallback if one native half is altered,
  reserved or clipped. Source claims and walking heights must stay intact.
* Viridian's connected maze has large plain green caps. Preserve its source
  layout and joined geometry while restoring native cap detail/variation;
  gray or generic wall substitutions are not an acceptable finish.
* Sprout 2F/3F lattice/depth regions contain broad plain brown panels despite
  their native gold/black grating. Inspect the actual depth-finish path and
  retain its intentional lower support/void semantics while improving the
  visible lattice detail. Do not fill the intentional voids with walls.

All current build/test/capture handles are terminal. There is no pending
build to restart. QA images remain ignored under target; temporary audit/source
inspection artifacts are removed. The next increment must implement the
remaining source-specific finishes and verify their native appearance.


### Gaming table and remaining native finishes

Goldenrod now resolves its complete 2f/30 source pair as one blue gaming table,
with gray frame and four legs. Both complete drawings and the twelve owned
cells are required; altered, clipped or reserved table cells retain source art
rather than a generic half-counter. Sixteen arcade regressions passed (83900),
including the new full-pair/ownership/three-scale geometry regression. Native
paired capture in `target/indoor-table-grating-review` was viewed: the blue
board, card marks, gray frame, legs and adjacent stools are visible.

Sprout 2F/3F depth drawings now use closed native gold rails over the dark
opening, with exact top datum and no footing mutation. Both actual native
paired renders were viewed (17574), and their grating matches the original
palette and lattice layout. The source matcher rejects substitutions; native
UV sampling remains in memory. Added geometry/ownership regression is pending
(14435), so compilation or screenshots alone are not recorded as a test pass.

Viridian's cap inlays are part of each original solid wall section and its
cutaway range, with the old cap recessed inside the original 12-pixel height.
Four closed olive stones provide visible grooves and palette variation.
Original model caches, triangle join ordinals and native source ownership
remain intact. All sixteen masks and three scales have a bounds/cache/closed
geometry regression queued (56673); final renderer build is queued (95819).
Actual Viridian appearance and the final 39-map/26-variant audit remain to be
checked. The 20-model closed geometry and actual four-gym source guards passed.

The current native voxel test binary (newer than the final source edits) passed
both new grating and cap-inlay regressions directly. The cap regression checks
all sixteen masks at three scales, unchanged base triangle indices, cached
prototype independence, closed parts and outward face winding. Native Sprout
captures and the gaming table are already verified; Viridian capture and the
final source audit still await the queued examples build. This is meaningful
progress, not an external-state block.


### Current full-scope audit and final motion gate

The final production examples build (95819) succeeded. App-muted native
Viridian capture (34678) was viewed: carved green cap inlays are visible,
wall connections and source routes remain continuous, and its purple floor
retains the source pattern. The whole-scope actual-pack audit (53049) verified
all 39 named rooms and all 26 declared script-block combinations with zero
mesher errors. Totals remain 11,323 authored object/architecture cells;
15,267 ordinary floor/background cells; 440 intentional planes (Sprout depth
and Violet facade); 24 floor/carpet relief cells; and 194 Cerulean water cells.
These counts are not a claim that every floor should become a standing prop.
The remaining classifications were inspected: store clerk floors, Celadon
exit carpet, Cerulean pool, Sprout grating and Violet source facade strips.

The full Cargo grating regression (14435) passed. The duplicate queued
cap-filter Cargo process (56673) was deliberately stopped after its current
compiled regression had passed; it is not reported as a Cargo pass. Neither
other chats' jobs nor the actual examples build were stopped or restarted.

The extra native Sprout motion gate exposed a capture issue, not yet proven
animation failure: visible-window readback returned blank and was rejected;
image-target temporal readbacks were visible but sampled the same pillar
pose (zero changed shaft/base pixels). Neither pair counts as motion proof.
The existing same-view capture option now supports one explicit 2.5D map,
uses 3d-start/3d-end filenames, and labels its comparison as temporal poses.
The second readback waits 40 frames rather than 12, exceeding the native
16-tick source animation cadence. Normal paired 2D/3D warmup is unchanged.
This capture-harness correction is queued in build 54631; verify native
pillar sway and stable base before closing the full goal. No room geometry,
palette, source ownership, support or production gameplay changed in this
capture increment. Static 39-map review and all four localized final finishes
remain proven. The temporary full-map audit JSON is removed after inspection.


### Sparse animation verification and Sprout timber revision

The 40-frame capture alone did not resolve the frozen pillar. The stable
terrain revision now excludes live texture handles, and the sparse animation
resource was not consumed by the cached prop renderer. `terrain_animation`
now authenticates map, complete source identity, immutable frame family and
retained atlas coordinates before updating owned atlas slots or selected
animated prop meshes. Static terrain, mesh handles, support and simulation
stay intact. Invalid authenticated images report an inactive renderer rather
than silently using a generic replacement. All four sparse regressions passed
through Cargo (5554); the current binary also passed nine related Sprout tests.
The final examples build (37762) succeeded with the 43-frame temporal capture
and empty-domain upload guard. App-muted native Sprout capture (54102) shows
actual shaft motion; native Blackthorn capture (77677) shows changing lava
patterns (40,708 changed pixels). The Cerulean pool keeps its authored quiet
blue surface; its temporal capture is not claimed as water-motion proof.
The fresh actual-pack audit (41241) again passed all 39 rooms and 26 declared
block combinations, with the same classifications and no mesher errors.

The user rejected the Sprout pillar's appearance after this verification.
The old swatch assignment made the shaft and oversized rectangular capital
black, with a bright straight stripe. The selected source-owned pillar now
uses an authored twelve-sided timber shaft with fine axial grain bands, a
small stepped shoe and restrained upper collars. Its warm finish samples the
stable native ochre swatch; the complete 24-cell source guard, 48-pixel rise,
selected ownership and source-frame sway remain. Geometry tests cover closed
courses, face winding, bounds at three scales and finish variation. This
latest visual revision is pending native inspection and Cargo tests: examples
build 64501, regression run 93541. Earlier pillar screenshots do not prove
the replacement's appearance.

The user clarified the pillar must match the 2D broad square block that
wiggles. The round-column experiment is rejected and its screenshots are not
completion evidence. Reinspection of the actual paired source shows a broad
dark top above a full-width amber front: its 48 drawn pixels combine top
depth and front height. The current replacement is a 24-by-24-by-24-pixel
square timber block with that native dark-top/ochre-side arrangement, subtle
axial grain and a mottled lower face; no decorative collar, separate shoe or
bright stripe. Original 24-cell source ownership and frame-driven bend remain.
The source footprint is an image projection, not a mandate to double the
physical rise. A closed mesh/winding/three-scale/source-swatch regression is
updated. Build 9212 is queued behind the older round experiment's test 55744;
check compiler source timing and rerun current square tests before claiming
anything passed. A missing Vec2 in a concurrently edited cave file is fixed
with qualified bevy::prelude names only; no cave design was changed by this
task. Latest square appearance still needs native inspection.

Build 9212 completed successfully after the square source revision. All three
app-muted native paired tower captures (47638) were viewed; the broad dark
top and full-width amber face now match the 2D design on each floor. The
square motion pair (67660) was also viewed: actual upper-block movement is
visible, with 79,635 changed pixels in the isolated pillar region and zero
changes in the fixed upper-wall region. Base geometry stability still awaits
the current unit regression rather than being inferred from a screenshot.
The fresh exact 39-room/26-variant audit (87043) passed with zero mesher errors
and unchanged classification totals. Current square Cargo tests remain in
progress (63777); the older round test pass (55744) does not verify this final
shape. The rejected round preview files and temporary audit JSON are removed.

The current square source was confirmed in the fresh native voxel test binary
by its new `sprout_pillar_square_matches_projected_top_and_front_and_keeps_closed_grained_faces`
name. Direct execution passed all ten related Sprout regressions and all four
sparse animation regressions. These checks prove closed outward-facing
square geometry at three scales, native dark-top/ochre-side UV ownership,
complete source rejection, stable base vertices/support, selected source-driven
mesh motion, retained mesh/atlas handles and no generic replacement for invalid
authenticated images. Current native all-three-floor comparison and actual
wiggle are verified. The union Cargo wrapper (63777) is still finishing other
crate test binaries; its terminal status is not yet claimed as a Cargo pass.
All scope-specific geometry and animation regressions have passed in its
freshly compiled voxel binary. No pack or generated audit was added to source.

The union Cargo regression (63777) subsequently finished with exit zero: all
three current square-pillar regressions passed, and the other crate filters
selected zero tests. Together with the direct current-binary ten Sprout and
four sparse passes, native review of all three final tower floors, actual
square motion capture and the fresh 39-room/26-variant audit, this completes
the requested indoor conversion and the final pillar correction. Ordinary
floors/backgrounds, source water and intentional depth/facade planes retain
their proper surface roles; classification counts are not a standing-prop
quota. No pack, compiled product or temporary audit was staged or committed.


### New goal: full Kanto interior review

The separate Kanto goal begins from current pack evidence, not the completed
39-room gym/store/tower scope. The audit example now records environment,
actual landmark region and landmark constant alongside existing geometry
evidence. The pack declares 120 Kanto maps outside TOWN/ROUTE: 100 INDOOR,
13 GATE and seven CAVE. Beta rooms, League rooms and the Safari beta map
remain in the inventory; environment alone does not prove what their source
art represents. Exact Kanto baseline audit (99026) passed all 120 base rooms
and 16 declared script-block combinations with zero mesher errors. The
whole-pack metadata audit (30902) rendered all 388 base maps but returned
exit one for nine conflicting Johto-only block-alternative declarations;
that is not recorded as a whole-pack variant pass or a Kanto blocker.

App-muted native paired baseline (54350) completed, and all nine comparisons
were viewed: Oak's Lab, Blue's House, both Red's House floors, Celadon Mansion
1F, Celadon Cafe, Power Plant, Fighting Dojo and Saffron Magnet Train Station.
They show actual palette loss: pink residential flooring becomes beige wood,
blue domestic walls become cream, Oak's gray striped flooring becomes beige,
and its ochre shelving becomes clinical white. Power Plant and Saffron's
station lose blue floor motifs; the dojo becomes beige stone with gray
statues instead of native timber, blue masonry and gold guardians. Some cafe
service surfaces still carry enlarged flat artwork and need proper authored
geometry. None of these baseline captures proves completion.

First implementation restores genuine atlas flooring in 38 exact Kanto
residential/mansion/lab rooms. Recognized native floor sources remain
textured on shallow beveled horizontal surfaces, with their actual palette
and motif, rather than a generic Oak/Marble finish. Existing authenticated
object backing that used a wallpaper donor requires a real same-tileset floor
donor. Custom profiles keep ownership; source phase/art checks reject stale
floor art, and actor footing/object coverage is unchanged. Two regressions
cover atlas-coordinate preservation, datum/claims, changed phases/art and
custom ownership. Production build 73587 and Cargo regression 91491 are
currently live; final native floor appearance remains unverified. This is
a first repair within the entire 120-map goal, not a reduced completion
criterion. Remaining work includes full Kanto native room review, walls and
connectivity, source palettes for furnishings, proportions, service rooms,
stations, labs, gates, Centers/Marts and specialty/League/cave interiors.

The audit command also gains `--kanto-interiors`, selecting the current pack's
actual KANTO landmark mapping and all non-TOWN/non-ROUTE environments before
meshing. Missing landmark/environment data fails the requested scope instead
of silently dropping rooms. This avoids repeating a whole-pack audit or
maintaining a generated inventory catalog. The new example flag was edited
while build 73587 was compiling the library; inspect its final binary/mtime
before using it. Both 73587 and 91491 were polled and remain live. First floor
native verification and the full Kanto goal remain active and unproven.

Production example build 73587 passed. Native first-floor review 86421
completed and all five comparisons were inspected: Blue's House, both Red's
House floors, Oak's Lab and Celadon Mansion 1F. Pink checks, ochre bands,
gray lab stripes and mansion corridor motifs now retain actual source
texture. Furniture and walls remain visibly unfinished. Full Kanto audit
25746 used the new --kanto-interiors flag successfully: 120/120 rooms,
16 script-block variants and zero mesher errors. Its temporary JSON is
removed after recording these results.

Direct execution of the newly compiled voxel regression found one real
first-floor failure: custom profile destinations were still repainted
because the final exclusion guard omitted Kanto rooms. The guard now
includes the same exact room scope as resolution. Do not claim the original
two-test run passed: one passed and one failed; new run 58989 is pending.

The next furnishing repair retains named structural material ownership
through vertex refinement in both bookcase models. Exact source-authenticated
Kanto residential/lab/mansion cabinets opt into their own live drawing
swatches for shadow, ochre structure and lighter shelves. Book colors retain
authored variety, ordinary non-Kanto placement retains its paint, and model
cache, geometry, normals and support remain unchanged. Unknown material
roles cannot acquire a guessed palette. Native appearance remains pending
a fresh example build; full Kanto completion is unproven.

The original floor Cargo wrapper 91491 ended with the same one-pass/one-fail
result seen in the direct binary. First cabinet test build 58989 then found
a moved-value compile error in material metadata construction; metadata now
finishes before the surface is moved into Model. Replacement regression
66706 is pending. Production build 44910 began before the latest material
routing revision, so it alone cannot prove that final source.

Blue house wall drawings now select a closed 16-pixel native course, carrying
the original horizontal blue bands instead of generic cream vertical weave.
The course meets the adjacent native window height; repeated segments share
identical seam coordinates, and the front replaces rather than overlays the
closed body face. Selection requires the complete already-authenticated
house wall drawing in the exact Kanto room scope. Red's gold wall drawing
is not repainted blue. Closed winding/seams/source-height regression added.

Cabinet source UVs require the textured terrain material. Structural
triangles now route to that atlas-backed surface, while named book materials
route to the untextured surface; this uses retained material ownership,
not RGB/UV guessing. Integration regression verifies triangle conservation,
unchanged support, non-Kanto preservation, custom ownership and altered-art
rejection. Fresh final test names and production capture are still required.

The final cabinet palette retains bounded model-local timber grain as
neutral brightness modulation of the live native swatches. All current
canonical material names were inspected and are covered by the explicit
structural/book ownership table. Integration/source-routing test added after
build 44910 began, so new build 84321 and regression 66706 are the active
verification handles. Both were confirmed live; no older render is evidence
for the current cabinet routing, grain or final wall source. No new pack,
source-art extraction or generated catalog was written.

Fresh direct voxel binary from 66706 contains all five new Kanto regression
names. Four passed, including the repaired custom floor exclusion; cabinet
integration failed because newly named custom profiles were not reserved in
Kanto rooms. The resolver's existing custom-priority rule now includes the
exact Kanto residential room scope. The earlier five-test run is a failure,
not a pass; a fresh replacement is required.

Native paired Fighting Dojo baseline reinspection confirms ochre timber,
blue masonry and gold heads over gray bases. Its exact fingerprint-selected
placements now carry FloorStyle::Dojo. Authenticated timber floors and
fixture backing use genuine atlas UVs on shallow bevels below the original
datum; floor selection inspection against the external pack proves every
Tatami identity uses tile 56, wall identity tile 30, threshold identities
0e/1e. No catalog or pixels were exported. Closed wall sides carry native
brick courses instead of generic timber rails; threshold source paint stays
at its original datum. Guardian head and pedestal named roles use live
texel swatches separately, on the original closed fitted model. Native
source swatches were inspected in memory before selecting this path.
A regression covers native UV/datum, unchanged support, closed wall winding,
head/pedestal atlas roles and altered drawing rejection. New source changes
require a fresh build/capture/audit; full120-map completion remains unproven.

Native app-muted service baseline batch 66649 finished successfully. All
nine paired comparisons were inspected: Pewter Center/Mart, Route5 gate,
Silph 1F, Trainer House B1F, Lavender Radio 1F, Celadon Cafe, Will and Hall
of Fame. Center pink/Mart green/checkered blue facility/gate diamond/radio
pink floors are lost to generic gray; yellow seats become blue, green gate
counters become brown, and blue/gray walls become cream. Cafe tables retain
large folded source decoration, with bland stool backing patches, and its
service furniture still needs fully authored geometry. Hall of Fame keeps
its dark floor but its main chair/platform remains flat and its computer is
a generic white cabinet. Will retains native color and intentional water
surface roles, while rear enclosure shape still needs review. These baseline
captures do not verify the new source changes.

Existing example inventory audit 86794 passed 120/120 maps and 16 variants,
with zero mesher errors. This binary predates final dojo/service-floor
changes, so it supplies scope/unchanged baseline evidence only. Its temporary
JSON was removed after recording results. Current metadata identifies 21
Kanto Center maps, seven ordinary Marts, thirteen gate rooms, three facility
rooms, one radio floor and one station. The native atlas floor treatment now
opts these exact 46 service maps in, alongside the earlier 38 rooms. Floor
sources still require their recognized tile and zero-height/flat role; phase
checks, native UV motif, shallow below-datum bevel and custom exclusion remain.
Department stores, previously reviewed gym floors and unrelated Johto maps
retain their dedicated paths. New service regression covers each source
family, genuine UV/datum, phase rejection and exact scope. All newly authored
Kanto native rooms now reserve new custom profile names before builtin models.
The fresh seven-test suite and native examples are still pending; complete
120-map fidelity and native evidence remain required.

Cargo wrapper 66706 ended with the already-known four-pass/one-fail result
in its pre-custom-priority binary. The fix remains pending in replacement
60633. Production build 84321 is now compiling after the latest source
changes; confirm freshness and run native comparisons before calling its
appearance verified. Do not restart either merely because a poll timed out.


Production build 84321 passed. Native muted 11-map palette batch produced all
paired images and its completed log ends at Saffron Station. All eleven
comparisons were inspected: Blue/Red 2F, Oak, Mansion 1F, Dojo, Pewter
Center/Mart, Route5 gate, Silph 1F, Lavender Radio 1F and Saffron Station.
Native residential and service floors now preserve gray stripes, pink checks,
green motifs, blue facility checks and gate diamonds. Cabinets retain native
gold structure and authored multicolor books; Dojo has blue masonry, ochre
floor and gold guardians. This does not prove complete room fidelity: Center
seats/counters, gate green counters/blue walls, lab equipment, mansion
architecture, radio walls and station orange seating still need correction.
Blue/Red previews exposed duplicate tall cream backing behind the native
wall course. Exact Kanto residential rooms now omit that weak-metadata
scaffold, and Red's authenticated wall art uses the native course. Pure black
players_house framing is rejected as a domestic wall within those rooms.
Fresh direct current-source nine Kanto regressions (89717) passed, including
both new wall/framing checks. Production replacement build 59001 was queued
and then confirmed compiling; current native images predate those wall fixes.
Full 120-map/16-variant audit 13774 passed with zero mesher errors; its ignored
temporary JSON was removed after this receipt. It predates the latest walls.

Pewter native comparison reveals lobby source seats are low yellow cushions,
not blue chairs with tall backs. Actual local pack lounge drawing was read
in memory: upper yellow upholstery/light motif, ochre sides, dark foot row.
No source art was exported. Exact authenticated Kanto lobby seats now use
closed low cushions and four short feet with original upper native pattern
UVs; physical rise seven pixels, unchanged source ownership/support. Link
Club and Johto treatments retain their existing paths. Source geometry,
palette routing, unchanged support and altered-art rejection regression added.
Fresh ten-test direct run 9571 is pending; native final seats and walls need
new production capture. The full Kanto goal remains active.

Current-source focused Rust run 9571 finished successfully: all ten Kanto
native regressions passed, including low-seat height/native textured routing,
unchanged support/claims, Johto preservation and changed-art rejection. Earlier
Cargo wrapper 60633 also finished successfully with seven selected Kanto tests;
it predates the two wall/framing tests and low seat change. Production build
59001 remains live after compiling voxel and Bevy code. No latest native
capture or completed production build is claimed yet. The focused ignored
binary was removed after recording its result; no compiled artifact, pack or
source image was staged. Fresh house/Center visuals are the next gate.


Production build 59001 finished successfully. App-muted native batch 72232
passed and all three paired images were inspected. Blue and Red 2F no longer
have the duplicate tall cream backing: native blue/gold wall courses are
visible. Center still shows blue backed chairs, contradicting visual
completion of the low-seat change despite its direct source regression pass.
The actual local pack was re-read in memory: all four seat anchors use the
expected 2e/2f phases and 48/49/58/59 drawing. The next fresh production binary
must settle compilation freshness versus a production routing issue before
claiming seats verified. These images are intermediate evidence only.

Kanto's connected gate modules now retain primitive ownership from the
exporter's explicit material `name` field. All sixteen modules validate their
known timber/top/trim/panel roles; unknown models cannot acquire this palette.
No geometry, cached model paint or unrelated primitive parsing is changed.
The exact eleven Kanto gate maps with counter networks select native top and
side color donors from within each complete fingerprint-authenticated sparse
network. All counter triangles use the native atlas material; telephone
geometry keeps its separate model treatment. Missing/forged donors or unknown
model roles reject before mutating ground, claims or geometry. Johto and
underground counter treatments remain unchanged. Original model-local grain
is a small neutral brightness modulation of the native color samples.

Fresh current-source direct Rust run 44394 passed all eleven Kanto native
regressions and all ten gate-network regressions. The new gate check covers
all sixteen modules' unchanged fitted vertices/normals/indices and cached
paint, atlas routing, unchanged support/claims, and atomic forged-donor
rejection. Actual pack full-network donor check 43790 passed: twenty-two
fingerprint-matching networks across all eleven selected maps each contain
both required native color sources. Its source/binary scratch files were
removed; no pack data or pixels were exported. Production replacement build
33804 is confirmed live compiling voxel and Bevy; new green-counter native
previews and a fresh full120/16 audit still remain. Current goal is active.


Production replacement build 33804 passed. App-muted native batch 17465 and
full-scope audit 57635 passed: 120/120 maps, sixteen variants, zero mesher
errors. All four paired captures were inspected. Pewter now displays four low
yellow lounge seats with native cushion motifs and dark feet: earlier blue
render was superseded by this fresh binary. Route5/Route15/Victory counters
are native green, retaining connected geometry, operator bays, sparse service
openings and separate phone geometry. Gray diamond floors remain genuine
source UVs. Gate walls/door frames still cream; planters still generic red.
The temporary full audit JSON was removed after deriving the remaining-map
capture selection. No source art or pack was exported.

Gate wall profiles were re-read directly: upper fragments specify base14,
height8 and a16px forward offset; planter fragments specify base8, height14
and8px offset. The generic clinical model path ignored these authored offsets,
generating staggered walls in cream. Exact source-selected Kanto gate clinical
walls now use a common22px crest, original blue/gray front drawing, closed
volume/caps and those raised fragment bases/offsets. The complete source guard
and custom-profile priority remain unchanged. Door frames still need native
palette review; this clinical-wall correction does not claim them complete.
Fresh direct current-source run85881 passed all twelve Kanto native tests.
The new wall check covers full/upper/planter courses at one front plane/crest,
native atlas routing, unchanged support/claims, Johto preservation, canonical
profile ownership and changed-art rejection. Fresh production build queued
for this latest wall code; native appearance is still pending.

A fixed, app-muted production executable snapshot from33804 is capturing the
104 remaining Kanto maps in batch8216, to target/kanto-complete-room-review.
The thirteen gate rooms and the three latest house/Center captures are
excluded because they have separate current/pending visual gates. This is
scope-wide evidence gathering, not a narrower completion definition. The
snapshot is an ignored hard link and its wrapper removes it on termination.
Batch completion and actual review of every room remain required; current
visual evidence does not yet cover all120 room designs.

Latest gate-wall production build91596 is confirmed live compiling. Remaining
room batch8216 was confirmed live, and its first three comparisons were
inspected: Bill/Bill's older sister retain pink floor/gold books/blue native
wall but table/stools, windows, computer/radio and plant pots still use
mismatched generic palettes. Bruno retains green/blue/lava source surfaces;
rear gray masonry is painted cream and podium/enclosure depth needs closer
review. Intentional lava planes are not missing standing geometry. These
findings are incomplete-room evidence, not completion marks. Other completed
batch captures still require inspection. No source changes after the twelve
passing tests were made before build91596.


Gate-wall production build91596 passed. Its native verification waits for
batch8216 to terminate so native GPU captures remain serial. That fixed
104-map batch was confirmed live; fifty paired images exist so far, not all
reviewed. Newly inspected Celadon Cafe and department floors1F/2F/3F expose
remaining shortcomings. Store native yellow/red floors, window course and
closed counter shapes retain their patterns; ordinary blue display racks
still use pale shelves, reception bodies remain generic cream/red, and plant
shapes/pots need correction. Cafe's giant table decoration was pixel relief,
with generic brown stool/furniture floor patches and coral round stools where
native seating is gray. The service appliances/counter still need authored
shape/color review. Earlier all-standing-object classification was not proof
of complete native palette fidelity.

Celadon Cafe now resolves its complete actual32px table drawing atomically,
with exact16-cell source identities and ground donor. Source inspection came
only from the local pack in memory. Edited/cropped blocks and live custom
profiles cannot acquire a new table. The metatile-only Celadon pedestal/
pixel-relief fallback is removed: unsupported source stays on its original
plane. Other Cafe maps retain their current paths. A fitted original CafeTable
model now has native pink cloth and native dark stand/trim through explicit
named linen/metal/timber roles; grain/weave is model-local neutral brightness.
Its source-projected depth is24px and rise16px, rather than a32px slab. A
closed oval woven basket and six closed raised contents carry its own gold
source palette. No source basket picture is copied onto a flat tabletop.
Celadon's genuine geometric floor motif now extends under furniture through
the same source-atlas floor path, removing the generic oak patch treatment.
The exact native floor allowlist is now85 maps plus separate Dojo treatment.

Fresh current-source direct Rust run62883 passed all thirteen Kanto native
regressions and all four Cafe scope tests. New test checks complete ownership,
low fitted bounds, native textured routing, raised basket, closed basket/
contents edge counts and outward winding, unchanged support/claims, custom
priority, unrelated map preservation and changed-source rejection. Canonical
CafeTable retains explicit named material ownership after refinement; no RGB
material guessing was added. Cafe native capture and latest full-scope audit
are still pending a fresh production build. The fixed104-map batch predates
Cafe changes and latest gate-wall changes; those maps need separate latest
captures. No audit JSON, source art, pack or compiled product was committed.

Cafe production build17569 is queued/live after the latest thirteen passing
checks; its latest native appearance is not yet verified. Additional104-batch
images inspected: CeladonDeptStore4F/5F/6F/Elevator. 4F has incorrect pale book/
goods racks where native black electronics/blue glass occur, despite the
source-owned electronics subset keeping its native front. 5F's closed U
window display has native lower glass and a coherent volume/open service bay;
its cream counter is closer to this source than the ordinary red reception
body. 6F has pink raised seats and native floor/window course, but vending
machines are teal instead of native gray, and tall native planters become
generic pots. Elevator preserves its blue glass/yellow floor motif and a
modeled control panel. No full store-family completion is claimed from these
comparisons. The exact eleven reviewed104-batch rooms so far are Bill/Bill's
older sister/Bruno/Cafe and all seven Celadon department floors/elevator.


Cafe production build17569 completed successfully. The original104-map native
batch8216 stopped after64 complete pairs at RockTunnelB1F: its dark2D image
failed the nonuniform-frame check. No full-batch success is claimed; its
snapshot executable was removed. Fresh audit99123 covers the original120
Kanto maps and16 declared block variants with zero mesher errors. Its temporary
JSON was removed after deriving the exact39 uncaptured rooms (Safari beta gate
was already included in the separate gate review).

Native batch96096 completed all14 latest Cafe/gate comparisons, and every pair
was inspected. Cafe now has raised native pink cloth/gold basket and native
floor under furniture; coral round stools still conflict with gray square
source seats. All gate blue wall courses now preserve native band colors and
placement. Most green counters retain connected operator bays, but both
underground entrances still paint their source-green enclosures beige. White
open door frames replace native blue doors, terminals remain generic teal,
and potted plants retain generic red pots instead of native ochre planters.
VictoryRoadGate has remaining north-door course/connectivity work. This review
is incomplete-family evidence, not a completion declaration. CeruleanGym and
FightingDojo baseline pairs were also viewed: Dojo source palettes/guardian
volumes remain coherent; Cerulean's platform/pool has depth but gray perimeter
and gold plaque treatment still require further review.

Kanto retail displays now route complete blue aisle, glass-case and black
equipment drawings through closed cases with recessed native fronts rather
than generic cream merchandise. The new test initially failed because its
ordinary-Mart fixture supplied tile48 instead of the genuine department tile01
ground donor; the fixture now supplies both. Its next failure caught missing
native row-corner boundaries on the glass case; added half/three-quarter front
knots preserve those boundaries without changing the glass recess. Fresh
current-source direct run9025 passed all14 Kanto native checks plus the existing
closed-electronics winding/volume/source-front check. Scratch test binary was
removed. Fresh production examples build55082 is live; latest retail appearance
still requires native inspection after it completes.

Remaining-room capture83874 runs39 exact uncaptured rooms sequentially using
an ignored fixed executable snapshot; individual failures are recorded without
stopping later rooms. RockTunnelB1F passed with STATUSFLAGS_FLASH set only in
a fresh disposable preview fixture. Its pair was inspected: modeled rock and
cliff volumes exist but the source purple cave palette becomes gray, so cave
palette fidelity remains incomplete. The batch is confirmed live after that
first success; it is not yet a39-room capture pass. No pack, source-art export,
audit JSON, compiled deliverable, staging or commit was added.

Capture83874 has now exposed a wrapper error for several small houses: single
--map defaults to the map center (4,4), which is blocked. These are failed
preview launches, not map-render passes. Continue the confirmed-live batch;
retry its recorded failures only after terminal status, using --maps with one
name and --output-dir so the existing nearest_walkable_tile selector supplies
a legal native start. RockTunnel and both Route10 Center floors succeeded.
The source pack and controller collision were correctly preserved.


Production retail build55082 passed. Native batch7763 completed all four
CeladonDeptStore2F/3F/4F and PewterMart pairs; every pair was inspected. Blue
source aisle cases now remain blue, refrigerator/glass fronts retain the
native blue/white design, and black equipment shelves no longer become pale
book/goods racks. Cases have sealed volume and recessed fronts. Reception
counters remain generic cream/red, and appliances still need further shape
review; this is not completion of all store fixtures. The latest120-map/
16-variant audit13745 passed with zero mesher errors and its JSON was removed.
The remaining39-room batch83874 terminated:18 success,21 blocked-center start
failures. Its fixed snapshot was removed. Union inspection against all120
actual map identities found one additional previously uncaptured GATE map,
UndergroundPath. Capture53300 now runs those22 missing rooms using the existing
nearest_walkable_tile selector through --maps (one map at a time), so no
collision or pack change was used to bypass blocked starts. First nine small
house/station retries succeeded; the live batch is not yet a22-map pass.

Source corrections now include both underground entrances in the exact Kanto
native green-counter palette path. Actual-pack in-memory check16595 verified
both original sparse fingerprinted networks on both maps, each with three
own top07 and six own side24/25 donors. Its helper source/binary were removed.
Kanto double doors now use their complete native blue drawing on a closed
22px wall course aligned with adjacent walls, rather than a white open portal.
The new test checks complete front area, source UV ownership, unchanged
claims/support, custom priority and edited-source rejection; unrelated Johto
portal treatment is retained. Current-source run76257 passed15 Kanto tests and
all10 gate-network tests. No generated art or pack was introduced.

The user called out poor cave readability. A new Kanto cave finish reads the
live authenticated tile16 floor field color, then separates lighter walkable
shelf tops, lower paths, dark cliff faces and enclosing caps. Existing shapes,
indices, normals and authoritative support remain unchanged. Distinct mineral
UV domains retain the same map coordinates, preserve cutaway only on enclosing
rock, and use quieter monochrome grain so painted bedrock cannot wash native
purple back into gray. This applies to the six actual cave/dark_cave Kanto
maps: DiglettsCave, MountMoon, RockTunnel1F/B1F, TohjoFalls and VictoryRoad.
The seventh CAVE-environment map, SafariZoneBeta, uses park art and does not
inherit this treatment. Source atlas, live water, frozen caves, ladders,
actors and unrelated maps keep their material domains.

Current-source run84407 passed all16 Kanto tests plus both Seafoam native-palette
tests. New cave regression verifies distinct shelf/floor/cliff values, live
purple hue, unchanged topology/support/alpha, preserved unrelated domains and
invalid-source rejection. Scratch test binary was removed. Production build
66979 remains confirmed live waiting on the shared Cargo lock; at last log
inspection compilation had not started, so it should consume these final
cave/door/counter sources when it starts. Compiler timing and embedded shader
must be checked before claiming the latest native appearance. Both cave
readability and corrected gate doors/underground counters still require fresh
muted native comparisons, followed by the full120/16 audit. No completion
claim follows from unit tests or the previous cave screenshots.

New baseline pairs inspected this turn: SaffronGym retains blue/pale divider
faces and warp motifs but thick cream caps remain; SeafoamGym keeps ochre
rock volumes; TrainerHouseB1F preserves blue floor/gold boundaries but green
rear wall is flat and plants generic; SoulHouse's native gold memorials become
gray and windows turquoise; ViridianGym's green partitions become broad
checker caps and guardian presentation needs review; WillsRoom retains native
floor/water but rear gray structure becomes generic cream; VictoryRoad's
purple-to-gray substitution and weak scarp contrast prompted the cave finish.
These findings remain open in the original120-room scope.


Latest native cave review: production build66979 completed successfully. The
muted image-target batch89270 completed all five comparisons: RockTunnelB1F
(with the disposable Flash preview flag), VictoryRoad, Route5SaffronGate and
both Route5/6 underground entrances. All five pairs were inspected. Blue
gate doors now close the source wall course and both underground counter
networks retain their native green. Teal telephone and other unrelated prop
palettes remain open. The cave finish restores purple but the current cave
images still fail readability: raised shelves merge with lower floor and
dark edges can read as holes. This is an open visual issue, not a completed
cave delivery. The capture snapshot was removed.

The missing-room batch53300 completed all22 pairs successfully. The union
of the complete-room, cafe/gate-wall, green-counter and wall-seat review
folders contains120 unique map comparisons. Their existence is not evidence
that every room has passed visual review. The fresh geometry audit54952
completed successfully and its temporary JSON was removed.

League floor source correction preserves Karen's blue stone and Lance's red
runner using their native atlas instead of generic cream/purple floors. The
external pack's complete Lance metatile31 was checked in memory: each row
is 3b,3c,3c,3d. Scratch source and binary were removed. Current-source run14555
passed all17 Kanto regressions and the existing floor-volume regression.
Production build32197 remains in progress; League native verification is
pending. Koga's flat trees and Hall of Fame console remain open.

Following the rejected cave previews, the native cave finish now samples
the exact01 raised-shelf drawing separately from the16 lower floor. Missing
or invalid shelf samples leave that surface unchanged rather than inventing
a pale substitute. Geometry and support are unchanged. Source tests and a
fresh native preview are pending; the preceding five-map captures predate
this additional shelf correction.

The subsequent cave correction also authenticates26 enclosing-rock cap
art independently from16 floor and01 shelf. Pale source ceiling masses
therefore no longer inherit the darkened lower-floor pigment. Absent donors
leave the original surface intact. Run4108 passed all17 Kanto tests plus the
floor-volume regression, including exact distinct donor colors, preserved
support/topology, and missing-donor rejection. Scratch binary removed. The
first shelf-only run71601 exposed an incorrect test expectation for the
existing bounded brightness modulation; that expectation was corrected
before the passing full run. Build32197 completed successfully and the
Karen/Lance native batch4153 is running. A fresh optimized cave-pigment
build is running under its own log; the five-map cave review above predates
this latest correction. The original goal remains incomplete.

League native batch4153 completed both room comparisons and both were
inspected. Karen's walkway now preserves the blue/white stone pattern;
Lance's runner retains its native orange-red color and edge drawing. Rear
League structures still have cream framing; Lance's gray guardian drawings
still become cream spiked figures. These remain open. The League snapshot
was removed. Cave-pigment production build34170 is still waiting on the
shared build-directory lock; do not restart it or claim native verification
of01/26 pigment changes from earlier captures.

Soul House's memorial source colors are now routed through the live atlas.
The exact six house28/2a/2b two-by-two variants retain separate claims and
existing geometry; named stone, stone_dark and stone_light materials map
to verified own-tile37 body/recess/trim pixels [1,1], [4,3], [4,1]. The
external pack's grayscale drawing was inspected in memory, not exported;
helper source/binary removed. Unknown or incomplete model material roles
reject the native treatment without appending a substitute. Customized
source profiles and changed drawings retain ownership. Run68717 caught a
test comparison including source underlay vertices on only one side; the
comparison was corrected to verify underlay and model independently. Run8965
passed all19 Kanto native tests and the floor-volume regression. Scratch
test binary removed. Native memorial appearance remains pending.

Production build34170 is confirmed live and compiling; its voxel source
compilation began12:43:55, after the cave and memorial implementation edits.
The subsequent memorial changes were test-only. Once it completes, capture
all six Kanto cave/dark-cave maps and SoulHouse with the verified muted
image target, using separate disposable Flash previews for both RockTunnel
floors. Review those pairs and run the fresh full120/16 audit. Do not restart
the live build or infer completion from source tests.

Four additional baseline comparisons were inspected: VermilionGym's source
small gray bins become oversized pale blue cylinders and its electric-wall
barriers become generic cream display panels; TohjoFalls uses gray rock
instead of native purple (latest cave source fix awaits preview) and waterfall
water shifts native blue to teal; UndergroundPath replaces native diamond
floor with generic blue ceramic and cream walls; TrainerHouse1F has generic
cream round stools, red plant pots, pale/turquoise PC and counter worktop
mismatches while retaining its gold table. UndergroundPath's explicit
passage Ceramic substitution in dungeon_extension is a confirmed next
source-fidelity correction. Cafe's coral round stools still mismatch gray
rectangular source stools. These are open in the full120-map goal.

Production build34170 completed successfully. Native batch28717 completed
all seven pairs, and every pair was inspected: both RockTunnel floors with
disposable Flash previews, VictoryRoad, DiglettsCave, MountMoon, TohjoFalls,
and SoulHouse. Memorial gold/brown/pale trim now matches its source palette;
its rectangular silhouette and turquoise windows remain open. The native
cave pixels expose a remaining cap problem: dense dark hatch pigment wins
a histogram even where the source drawing represents a pale rock top.
Some enclosing caps therefore look like brown holes above purple faces.
RockTunnel remains very dim; disconnected small outcrops, huge upright
warp ladders and teal waterfall water also require further visual review.
No full cave readability approval follows from this batch. Snapshot removed.

Audit64612 completed the actual full120 maps and16 script-block variants,
verified120, with zero mesher_errors. The report was inspected and its
temporary JSON removed. This establishes geometry/ownership coverage for
that build, not visual fidelity and not the later floor/color corrections.

UndergroundPath's exact underground tile10 now uses NativeCeramic with its
own atlas and the original ceramic family label. Its volume stays below
the actor datum. Invalid tileset/art/phases cannot fall through to generic
Ceramic. Custom live-profile claims protect the selected floor; the same
protection is applied to the exact native League floors. New source tests
cover native diamond UVs, unchanged support, other-map isolation, invalid
phase rejection and custom ownership. The initial run85387 exposed the
missing custom-floor protection; implementation has been corrected.

Cave cap/shelf sampling now selects their actual light mineral swatch rather
than the dominant dark hatch. Dust retains its source field. Small outcrops
now sample their own authenticated0c rock drawing, instead of borrowing
walking dust color; missing rock/cap/shelf donors leave those surfaces intact.
The cap regression includes dominant dark hatch with a minority native
light mineral swatch. Exact separate rock donor color is verified as well.
The current source test run66166 is in progress; production41458 is confirmed
live waiting on the shared build-directory lock and should consume these
final edits when compilation starts. Do not restart either from timeout.

Direct test dependencies had to be refreshed from current crystal-render-api
after concurrent Pokemon model-palette work. Earlier compile37014 used a
stale cached API. API attempt52854 lacked the proc-macro manifest environment;
30479 then encountered a concurrent duplicate Vec2 import, already repaired
when the source was re-inspected. No unrelated source was reverted. Fresh
API/test scratch files are removed by the current wrapper on completion.

Further cave source correction gives small rocks their own exact0c pigment,
with authenticated complete-rock local phase and no dust-color substitution.
Native cap selection uses the actual brightest visible mineral pigment from
its own source drawing; the minority-light hatch test covers the rejected
brown-cap case. The custom-floor ownership failure in85387 was a real gap
in the shared semantic-floor loop and is now protected for UndergroundPath
and the exact native League floors.

Run66166 encountered concurrent duplicate append_all declarations during
cave-field integration. Current-source inspection confirms that the other
work resolved this to a single cave_boulder_fields re-export; no edit or
revert was made here. Run75417 recompiles the current render API and tests
against it, with the required manifest environment, and is confirmed live.
Production41458 has started compiling after the final floor/cap/rock edits;
no native preview from that build is yet available. Current source now also
contains the concurrent connected-boulder field renderer, so fresh cave
comparisons must verify its integration as well as the native palette.

Run75417 still failed the custom corridor-floor assertion. Inspection found
two issues: the protection vector was only populated for Olivine/Saffron
and the custom fixture named block00 while the shared fixture creates25.
Protection population now includes UndergroundPath, the exact five League
rooms and Lance. The custom regression now names block25 (decimal37) and
first asserts that its live override actually resolves. Run56609 is confirmed
live compiling this corrected current source, including the hatch/own-rock
palette changes. Build41458 remains live; it started before this final
custom-profile population/fixture correction. Its stock room pixels can
verify the floor/cap/rock work, but a subsequent production build is required
to include the final override protection. No current20-test pass is claimed
until56609 finishes. Cave readability remains unapproved.

Run56609 completed successfully: all20 Kanto native regressions and the
existing floor-volume regression passed against the freshly compiled current
render API. This includes the corrected resolving override fixture, true
protected native floors, minority-light rock-cap sampling and the separate
small-rock donor. Both scratch test binary and API rlib were removed.
Native preview approval for the latest floor/cap/rock changes remains
pending production41458 and its follow-up final-protection build.

Build41458 completed successfully in12m09s. The native seven-map batch71478
terminated on its first map: RockTunnelB1F's2D capture succeeded but the
3D capture hit the startup deadline. Enhanced-only run62386 also exited
with the explicit capture timeout after geometry preparation, so neither
run provides a new3D approval. Both snapshots were removed. The executable
literal mute guard and native image target were verified; window readback
was never substituted. Earlier seven-map review28717 remains the last
complete cave comparison batch and predates the latest cap/rock finish.

Native Kanto house stools and CeladonCafe now use a compact rectangular
closed seat with four dark feet, height5 source pixels. Only complete house
02/03/12/13 furniture drawings route to it; upper01..07 seat rows are cropped
without stretching the lower feet over upholstery. Own-source02[4,2] supplies
the gray side and12[6,1] the dark feet, checked in memory in the external pack.
Palette handles remain live. Other tilesets and Johto retain their treatment;
custom source ownership and changed art reject replacement. No new source
art or model file is exported. Scratch source checker/binary removed.
Run15326 passed all21 Kanto native tests and the existing floor-volume test.
The new stool regression verifies rectangular closed top area, maximum
height5, four original claims, source cap UVs, support preservation, custom
priority, source edits and unrelated-room isolation. Sizing still requires
native review beside the Cafe table and current actor models.

Preview verification now accepts an explicit bounded capture timeout via
CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS (60..300). The default remains60;
invalid input is rejected before any capture output is removed. This applies
only to an installed automated capture and does not change the native render
target, image scale, warmup, scene lighting or game presentation. The existing
image-target test fixture uses the default60. Syntax check passed; production
compilation/native verification is pending. New build is running under
kanto-stool-capture-production-build.log and includes final floor override
protection, native stools and this bounded capture option. Next captures
should verify literal mute/timeout settings in the new executable, then use
CRYSTAL_PREVIEW_MUTE=1, CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image and timeout240
for the original-quality native images. No quality or visual fallback.


Kanto cave readability follow-up: Tohjo Falls now registers authenticated
animated cave water as role3. Its live atlas blue/foam colors remain in the
shader with continuous flow modulation, instead of fixed teal substitution.
Mask, phase and map-space guards remain unchanged; unanimated and unrelated
tiles cannot enter this path. A new regression covers live phase changes,
cascade/pool roles, stable registration, unrelated maps and missing animation.
The initial mask-alpha expectation was corrected:255 encodes zero impact,
not opacity. Run52599 passed22 Kanto tests,9 cave-water tests and1 floor-volume
test; receipt:kanto-native-water-test-receipt.log. Scratch API/test binaries
were removed. Prior94861 output was unavailable after context truncation,
so it is not counted as an independently observed passing receipt.

Build65157 finished successfully (15m40s), including native house stools,
final floor override guards and the bounded capture deadline. It compiled
before the latest Tohjo registration edit, so it does not verify that water
change. Follow-up build73180 is running/queued under
kanto-native-water-production-build.log.

Native run69894 passed all four image-target comparison pairs using explicit
timeout240 and the verified muted executable:CeladonCafe,BillsHouse,
TrainerHouse1F,UndergroundPath. All four comparisons were viewed. House
stools now use compact pink rectangular source seats; proportions beside
tables still need refinement. The Cafe retains coral round arcade stools:
its actual tileset is game_corner, whereas the new test's synthetic house
fixture does not represent this room. That fixture is not Cafe fidelity
evidence. UndergroundPath now retains its native diamond floor; generic
cream enclosing walls remain visually wrong. No blanket visual approval.

Serial native cave review93493 is running for both RockTunnel maps with
disposable Flash, then VictoryRoad,DiglettsCave,MountMoon. It uses image
capture at original resolution/scale and timeout240; no quality fallback.
The full120-map goal remains incomplete.


Run93493 finished successfully for all five cave pairs; all were viewed.
The current lighter caps and connected outcrops are present, but both Rock
Tunnel images remain too dark and the large upright yellow warp ladders
obscure native source drawings. These caves are not visually approved.
Own snapshot was removed.

The six exact Kanto cave maps with cave/dark_cave sources now retain full
ambient fill and reduced direct energy (80 ambient,1950 direct). Their live
source palette already supplies darkness; this avoids dimming it a second
time. Existing face shading, shadows and other maps' lighting remain intact.
The existing light regression now covers all six maps, a wrong-tileset guard,
and restoration through a Johto dark cave and exterior. Syntax passed.
Run29834 compiles these current-source checks; receipt pending. Production
73180 started its voxel compile before this lighting edit, so a subsequent
production build/native comparison is required for this final lighting.
The synthetic house-stool regression now names BillsHouse/TrainerHouse1F
only; it no longer misrepresents actual game_corner Cafe art.

Light regression29834 passed22 Kanto+9 water+1 floor-volume, then failed
the synthetic wrong-tileset expectation. A house frame is not classified
as a cave at all; its correct existing energies are80/3000, not48/1440.
The expected baseline was corrected, with map/tileset assertion diagnostics.
Run85322 rechecks current source (also including the corrected house-only
stool fixture) and writes kanto-native-water-light-test-receipt.log. It must
finish before a passing lighting receipt is reported. No production/capture
failure was hidden by this fixture correction.

Run85322 finished successfully:22 Kanto native tests,9 water tests,1
floor-volume regression and1 expanded cave-light regression (33 checks).
All temporary API/test binaries were removed. Production73180 remains live
and predates the final lighting edit; follow-up build and all six native
cave comparisons remain required. Oversized upright warp ladders are still
unresolved. No full-scope or cave visual completion claim.


Cave warp/readability continuation:73180 finished production successfully
(10m13s). Native run96733 passed the TohjoFalls pair with muted image capture,
original scale/resolution and timeout240. The comparison was viewed:blue
water/foam and purple cap palette are now present rather than prior teal
water/dark brown cap. This image predates final light/warp edits; it proves
the source-water shader compiles/renders, not full cave readability or motion.
Temporary water snapshot was removed.

The six exact Kanto cave maps now resolve complete authored overhead warp
drawings to NativeCaveWarp, preserving the established family label, four
claims, original16px footprint and floor0/6 datum. Its closed1.25px shallow
volume uses the four original cap tiles and its own edge swatch. Tall35px
yellow/brown ladders no longer obscure these native drawings. Johto/ice
ladder treatment remains unchanged. Missing/edited phase groups still do
not resolve. The regression covers all six map names, both cave tilesets,
six drawing variants,0/6 support, cap area/all source corners, unrelated
Johto map and edited source rejection. Pack art was read only in memory;
scratch helper source/executable were removed. No art catalog was exported.

Run43313 stopped at a missing shared box-helper import; the path was fixed
before queued production compilation. Run54652 checks the final source and
writes kanto-native-warp-test-receipt.log. Build7744 is queued/live under
kanto-native-warp-light-production-build.log and must include final cave
light and warp edits. Fresh six-map native review and120/16 audit are still
required. Neither closed source relief nor source tests alone prove a good
visual result. Full Kanto goal remains incomplete.


User clarified that water may use better continuous patterns and reported
bad Y alignment and wall colors. Tohjo water now extracts dark water/light
foam palette extrema from authenticated opaque8x8 live source images. Its
continuous pool currents, cascade ribbons and downstream foam use those
colors instead of repeating magnified source glyphs. Missing palette art
retains the actual source surface; no foreign teal is selected. Palette
bindings109/110 are installed with the image material and inherited by its
cloned batches. Animation still drives the authenticated mask phase.

The previous cascade spanned16..0 while its pool stayed at-2, leaving bad
head/landing joins. Exact Tohjo upstream native-water components now render
at shelf6 minus water recess2 (=4), with cascades joining4..-2 and lower
water at-2. All walking/support arrays remain unchanged. Only complete
cascades with separated upper and lower native pools qualify; ambiguous or
clipped components cannot create an invented upper tier. Shader landing
foam is registered to-2 for native Tohjo. Other maps keep prior geometry.

Run9893 passed25 Kanto tests,10 water tests, the floor-volume regression,
expanded cave-light regression and legacy waterfall UV/slope regression.
Receipt:kanto-native-water-tiers-test-receipt.log. Temporary API/test binaries
were removed. The native tier test verifies both pool joins, exact head/foot
vertices and unchanged supports. A separate in-memory pack check confirmed
actual Tohjo60x36 layout:two complete cascades and160 upstream pool cells;
both cascade heads connect to that pool. Scratch source/executable removed.
This classification receipt is not a native appearance/motion approval.

Build7744 finished successfully (8m48s), including light/warp changes but
predating these newer water-pattern/tier edits. Native review72215 runs all
six cave pairs serially with image target, original scale, mute and timeout240.
RockTunnel uses disposable Flash. Build98890 runs final current water code
under kanto-water-pattern-tier-production-build.log. After it succeeds,
Tohjo final source-palette continuous flow and correct levels still require
GPU/native verification; broad wall colors and full120 interiors remain open.

2026-10-12 shared asset coordination and corrected water support receipt:
The prior statement that all rendering support heights stayed unchanged is
superseded. Tohjo's render-only footing heights now follow the raised upstream
pool at 4 and cascade centers down to lower water at -2, preventing Surf actors
from sitting below that surface. Authoritative gameplay coordinates, source
footprints and events stay unchanged. Run5559 passed25 Kanto tests,10 water
tests and the floor-volume, cave-light and legacy waterfall regressions.
Temporary API and test binaries were removed.

Build98890 passed in11m22s with continuous native water and tiers, before the
final actor support correction. Build94248 passed in9m30s with the later source
under target/kanto-water-render-support-production-build.log. The serial native
six-cave review72215 and Tohjo pattern/tier review44707 both completed and were
viewed. Neither is visual approval: cave cutaways expose raw pixel floor patches
and the newest Tohjo capture has almost entirely gray stone. Final actor support
still needs a matching native capture.

The user's direct instruction to coordinate all workers was verified in Redo
Outside. That task owns global tree dispatch/geometry; lighting/effects owns
shared water appearance. Interior work reported its existing role3 water palette
uniforms109/110 and pool/cascade datums, then paused independent shared shader
edits. Cave palette/desaturation and exposed floor patches were reported to the
cave owner. Future native art previews must explicitly select
CRYSTAL_OPEN_MODEL_ROOT=$PWD/assets/open-models to match browser asset selection.
Fallback previews must not motivate replacement of canonical tree geometry.

Interior and dungeon loaders now share exact immutable geometry through
shared_model_geometry.rs while retaining variant-local colors, UVs and cutaway
ranges. Canonical duplicate families are blue/green beds, red/pink beds and
cave/ice boulders. Exact equality verifies reuse after a hash lookup; unfamiliar
geometry is separate. No serialized assets were copied or rewritten. The first
sharing test run77628 failed to compile concurrent forest tests with tuple UVs;
the owner corrected those calls and current-source run26812 is pending. The
full120-map visual completion goal remains active.

Coordination follow-up: the cave owner scoped mineral_pigment desaturation to
VictoryRoad, which has its own direct user color request. Tohjo, Diglett, Mount
Moon and Rock Tunnel retain source pigments. The cave owner also accepted
exposed native floor patch cleanup after current crown/descent verification;
interior work will preserve those terrain changes. Build94248 and native44707
predate this palette correction and cannot verify its final appearance.

Run26812 completed successfully:25 Kanto tests,10 water tests, three targeted
terrain/light/waterfall regressions and all3 shared_assets tests passed. The real
canonical bed and cave/ice assets reuse the same geometry allocations while
retaining distinct rendered colors and material metadata. Temporary test/API
binaries were removed. Receipt:kanto-shared-assets-verified-test-receipt.log.

The user's later no-fallback directive was verified directly in Redo Outside:
missing 3D may remain original 2D to expose missing coverage. Interior inventory
found silent external/bundle-to-embedded substitution in open_models::load,
load_from_root and the wasm branch. The shared loader owner was notified before
editing that boundary; interior callers will need optional model coverage so
placements are not claimed when their required model is missing. Dungeon models
currently select explicit embedded canonical assets, without family substitution.
The new geometry interning cache does not introduce any missing-model fallback.

No-fallback interior boundary implemented after loader-owner agreement:
selected_model uses load_required exclusively for bookcase, dining table, red
bed, chair and television. Other intentional authored interior kinds parse their
designated catalog source directly. The five obsolete embedded alternate JSONs
were deleted and build-johto-interiors.py no longer regenerates them. Direct
model access for those five kinds requires the selected asset; production
furnishing placement uses the optional accessor before emitting any geometry.

Missing selection emits only the exact source atlas cells and reserves them
against older 3D extrusion, with no authored-model label and unchanged actor
footing. The designated external bookcase lacks the old cabinet's native wood
material roles. Native Kanto cabinet placements therefore retain source 2D art
when those required roles are missing; LabBookcase retains its authored native
roles. This visibly incomplete coverage replaces the former crash/alternate
assumption. The deleted red-bed alternate no longer participates in geometry
sharing; blue/green beds and cave/ice boulders still share canonical geometry.

Run95494 passed all targeted filters:25 Kanto,10 water,3 terrain/light/waterfall,
3 shared geometry,1 missing-source,4 shared-loader,17 interior-model,1 native
bookcase and21 furnishing-placement tests (filters overlap). Scratch test/API
binaries removed. Receipt:kanto-no-fallback-interior-test-receipt.log.
Production build5683 is running under kanto-shared-no-fallback-production-build.log
with regenerated Cargo model inventory and native examples; no new native
capture is claimed until that build finishes. Full120 visual work remains open.

Kanto bedroom PC palette correction: the exact players_room2x3 monitor/keyboard
assembly now uses its own three source swatches for named computer materials.
The generic teal/cream/timber tint is removed from this native placement.
Positions, indices and normals still come from the shared designated computer;
source guards, footprint and actor footing are unchanged. Material-role coverage
is validated before emitting geometry; unknown roles retain source2D instead of
guessing colors. Other room and lab PC placements keep their separate ownership.

Run62443 passed27 Kanto,10 water,3 targeted,3 geometry sharing,1 missing-source,
4 loader,18 interior-model,1 native bookcase and22 placement tests (overlap).
New tests verify own PC swatches, unchanged cached/fitted geometry, unknown
material rejection, complete claims and clipped-drawing rejection. Temporary
API/test binaries removed. Receipt:kanto-native-computer-test-receipt.log.
Build5683 remains live. It began before this PC correction, so a current rebuild
and canonical-root native RedsHouse2F comparison remain required. The stored
older room comparisons establish the mismatch, not the final appearance.

Selected cabinet coverage restored through canonical asset semantics:
bookcaseClosed(Clone)/wood explicitly identifies the external cabinet body;
books(Clone) with the four known book materials retains authored color variety.
The cabinet uses its source wood swatch and existing mild wood grain. No old
embedded alternate was restored. Unknown body/book names or materials reject
native-role coverage and retain source2D. New real-asset and BluesHouse complete
source fixtures prove geometry preservation, source claims, book colors and
unchanged actor footing. This supersedes the earlier blanket missing-role
status for the designated external bookcase.

CeladonCafe's four complete game_corner stool variants0a/0b/1a/1b now share
append_native_rectangular_stool with domestic seating, keeping their own gray
art and compact5px seat height. The earlier coral round seat was a poor source
match. Other arcade maps retain their intended stool family. Tests cover all
source phases, clipped drawings and unchanged footing.

Run22360 passed29 Kanto,10 water,3 targeted,3 shared geometry,1 missing-source,
4 loader,18 model,2 native-bookcase and24 furnishing tests (overlap), including
the loader owner's final exclusive-bundle fix. Scratch test/API binaries removed.
Receipt:kanto-selected-bookcase-cafe-test-receipt.log. Two obsolete test-only
EXACT_SHARED_SPECIES assertions were removed from johto_actor_props after owner
notification; dedicated model count and live species/icon routing tests remain.
Build5683 is still live through Bevy dependencies, before these latest edits.
A serial canonical-root native comparison command for RedsHouse2F,BluesHouse,
CeladonCafe is prepared and rejects executables lacking the PC patch marker;
it has not run. Full120 native appearance/proportion completion remains unproven.

Actual pack PC swatch correction: a transient Rust client read the existing
ignored browser pack and inspected only selected monitor colors. The previous
dark/medium samples both selected170 gray and the light sample selected85 gray.
Corrected source pixels[3,0],[0,0],[3,1] select0black,170gray,255white in the
players_room monitor drawing. The source helper and executable were removed;
no asset/program extracts or audit manifests were saved. Source palette still
belongs to the live tiles; no substitute RGB palette was introduced.

The same local client verified LakeOfRageHiddenPowerHouse and
LakeOfRageMagikarpHouse as actual8x8 saved map bounds for the user's cabin follow-up.
Run3333 passed the current interior/shared-loader/water filters after this PC
correction under kanto-pc-verified-swatches-test-receipt.log; scratch removed.
Build5683 remains live and its existing executable lacks the PC patch marker.
Potential reuse of another worker's broad queued build was reported to the
coordinator; no replacement build or native capture was launched from a timeout.

Build5683 completed successfully in16m22s. Native47090 then completed all three
canonical-root original2560x1600 pairs serially with mute/image target/240s bound;
all pairs were viewed and snapshot scratch removed. Gold cabinet bodies with
varied books and the cafe's compact gray rectangular seats are present. These
rooms remain unapproved: dining table readability, wall coverage, raw patterned
floors and unrelated fixture colors still require work.

The native RedsHouse2F capture revealed a real scope error: its pack tileset is
players_house, not the earlier players_room fixture. The prior PC branch did not
cover that actual room. native_computer_source now validates the exact complete
house/players_house computer signatures (including all phases and source tiles),
in addition to the bedroom monitor/keyboard matcher. A transient Rust pack check
verified black0/gray170/white255 points in the actual house40 and players_house20
monitor tiles and the separate players_house11 drawing. Source helpers removed.
The regression now uses the actual RedsHouse2F players_house10 drawing. Run40664
passed all current interior/loader/water filters; scratch API/test binary removed.

A headless audit was accidentally invoked with RedsHouse2F as its output path,
so it inspected388 maps rather than the single requested map. Its generated
scratch file was read and immediately removed. It confirms the actual room's
six computer cells and players_house tileset; it does not prove native appearance
or completion. Current audit also records30 facade source cells in RedsHouse2F,
consistent with the wall coverage debt visible in the capture.

The other queued broad build process is now absent. A current incremental
build was launched under kanto-live-pc-source-production-build.log; its new
native-computer diagnostic marker will distinguish the live-source correction
from the completed earlier capture. Matching native verification remains open.

### Native PC verification and shared wall/table placement follow-up

The live-PC production build finished successfully (9m22s; ignored receipt
`target/kanto-live-pc-source-production-build.log`). Its serial image-target
capture completed Red's bedroom, Blue's house and Celadon Cafe; all three native
comparison images were inspected in `target/kanto-live-pc-native-review`.
Red's actual `players_house` PC is now gray. This is bounded evidence: gold wall
fragments are still flat, the TV remains a poor match, and striped/checkered
floor presentation needs further work. These rooms are not visually approved.
The capture hardlink snapshot was removed after the terminal result.

The remaining exact `players_house` wall drawings beside the stairs and PC now
reuse the existing closed wall course. Upper and lower strips share a front
plane, preserve their gold source bands, and use separate vertical offsets.
PC/stair cells and custom ownership are preserved. The first current-source
regression run passed (30 Kanto tests and the associated loader/model/placement
checks; `target/kanto-connected-source-wall-test-receipt.log`). Scratch Rust
binaries were removed by the verification wrapper.

Blue's table diagnosis found a concrete placement problem: the imported full
model inherited the legacy six-pixel source-extrusion height, shrinking its
entire footprint. Complete house dining tables now use the same ten-pixel
model proportions as profile-resolved dining tables. Traditional low tables
retain their existing placement. A complete source drawing regression checks
footprint, model height and unchanged actor footing. The follow-up source tests
and production build are in progress; new wall/table native appearance has not
yet been verified. All 120 Kanto interiors remain the active review scope.

Follow-up source verification completed successfully:
`target/kanto-connected-wall-table-test-receipt.log` reports 31 Kanto and 26
placement regressions passing, alongside the shared geometry, model, loader,
water and missing-source checks. Temporary test binaries were removed. The
production build session remains live and queued for the shared build lock;
no new wall/table native visual receipt is claimed.

### Shared dining-table native palette follow-up

The selected `tableCrossCloth` geometry now supports a strict named wood/cloth
material adapter. Complete Kanto `house` dining drawings sample their own tile26
medium and dark tones (`[7,2]` and `[5,2]`) for timber and cloth, replacing the
imported pink/cream finish without changing geometry. The existing full-drawing
matcher guards placement; all source cells must belong to `house`. Missing or
unknown selected material roles display source art before any 3D geometry.

The packed grayscale source pixels were inspected in a temporary Rust helper;
helper source, executable and pixel scratch were removed. The initial palette
and immutable-geometry regression suite passed in
`target/kanto-native-table-palette-test-receipt.log`. A stronger source guard
regression is running separately. No new native palette approval is claimed;
the production build must finish, and a fresh executable must contain
`native dining table requires named wood and cloth roles` before captures can
verify this adapter. Prior native room images predate this change.

The strengthened dining source-guard verification finished successfully in
`target/kanto-native-table-source-guard-test-receipt.log`: 32 Kanto tests,
19 interior-model tests, 26 placement tests, and associated loader/shared/water
checks pass. It verifies both native source UVs, complete-model footprint,
unchanged footing, absence of the imported pink surface, immutable geometry,
and rejection of altered source drawings and unknown material roles. Scratch
verification binaries were removed. Native appearance remains pending.

The completed wall/scale build passed in 9m11s. Two serial native captures in
`target/kanto-connected-wall-scale-native-review` completed and were inspected:
Red's former flat wall fragments now visibly join the gold wall course around
PC/stairs. Blue's table has visible braces and increased size, but remains too
narrow relative to its square source drawing. Neither room is approved; their
floors and TV presentation still need work. The capture snapshot was removed.

That native evidence prompted another concrete table correction: the strict
native complete-house table adapter now fits both axes to the source square
footprint through the shared fitting implementation. Other imported furniture
retains its own fitting policy. Model regressions check full footprint and
outward triangle winding; map regressions require width/depth over 31 pixels
inside the 32-pixel drawing. Source tests are running. The capture guard now
requires `native square dining table requires named wood and cloth roles` to
reject executables predating this fix. Fresh native table verification remains
pending; full 120-map scope stays active.

Square-footprint current-source verification finished successfully in
`target/kanto-square-table-source-test-receipt.log` (32 Kanto, 19 model, 26
placement checks and associated shared/loader/water checks). Verification
scratch binaries were removed. The follow-up production build is confirmed
live, currently waiting for the shared build lock. Its handle is 5786; do not
restart it merely on observation timeout. After its terminal result, check the
square-table marker and run the serial native review guard. NativeRoom floor
presentation is explicitly preserving raw source motifs; it is not a missing
house-floor classification. Improve its wood/checker texture presentation with
source palettes after the current native table review, without claiming the
existing noisy motifs are approved.

### Kanto house floor material finish

Actual packed tile01 inspection confirmed `house` alternates medium/light every
pixel and `players_house` repeats medium/light horizontal rows. A temporary Rust
helper inspected these classes; its source and executable were removed. The
native source bevel topology is now shared through `native_floor_patches`.
Existing non-house source slabs keep their original interpolated UVs.

Exact Kanto house NativeRoom surfaces now use larger 16-pixel pink/light
ceramic squares. Exact players_house surfaces use 32-pixel staggered gold wood
planks with neutral grain tint. Both sample their own authenticated floor atlas
swatches, preserve source colors, keep bevels below the existing datum and
respect the existing custom-profile exclusions. Furniture underlays use the
same authenticated source palette. No new generic paint or alternate asset was
introduced. Other native service floor motifs retain their existing source UVs.

Current source verification passed in
`target/kanto-native-floor-material-test-receipt.log`: adjoining wood seams,
negative world-coordinate phases, neutral grain, outward winding, source
swatches, unchanged actor footing/claims, service floor UV compatibility and
custom/altered-source guards pass alongside the existing model/loader/water
checks. Temporary verification binaries were removed. The live production
build handle remains 5786; a fresh native capture must contain both square-table
and `native house material floor requires finite source bounds` markers. These
new floor visuals are not approved until that capture is inspected. The full
120 Kanto interior scope remains incomplete.

### Native square-table verification and television source finish

The square-table build finished successfully in 12m42s. It contained the square
adapter but predated the new floor and TV finishes. A guarded serial native
image capture in `target/kanto-square-gold-table-native-review` completed Blue's
house and its original comparison image was inspected. The table now fills the
square source footprint, has visible braces, and uses source gold rather than
pink cloth. This confirms the bounded table fix. The room still has the old
noisy floor and pale TV in this build; it is not approved. Snapshot removed.

The selected television asset was inspected separately: it includes a rounded
CRT screen and distinct cabinet stand. A strict named primitive/material
adapter now paints the stand from the lower drawing and the TV body, bezel,
screen and controls from their own source pixels. Complete house1e and
players_house02 television source drawings are guarded, and unknown material
roles display source art before model geometry. Both original packed sources
were inspected with a temporary Rust helper; helper source/executable removed.
No canonical geometry or alternate model was replaced.

Current source regressions passed in `target/kanto-native-tv-source-test-receipt.log`:
35 Kanto, 20 interior-model and 27 placement tests plus associated shared/loader/
water checks. They cover actual Blue's/Red's drawings, own source swatches,
unchanged geometry and footing, altered drawing rejection and unknown material
rejection. Temporary test binaries removed. Fresh floor/TV production build
handle 75230 remains live and queued; do not restart solely on observation
expiry. Native floor and television review remain pending. Full 120-map scope
is still active and incomplete.

### Kanto curtain wall source geometry

Existing native Blue's-house captures show the generic aqua window replacing
blue/light source curtains. Complete exact house/player-house window signatures
now reuse the closed domestic wall volume with their original source UVs on
shallow recessed folds. This replaces the mismatched aqua finish and keeps the
front edges at the same 16-pixel wall course. The shared floor surface helper
was renamed `native_source_surface_patches` and is also used for these folds;
non-house floor UV behavior is unchanged. Altered/clipped window source remains
outside this native adapter, with custom ownership handled by resolution.

Current source regressions passed in
`target/kanto-native-curtain-wall-source-test-receipt.log`: complete Blue's-house
and Red's-first-floor window drawings, source UV endpoints, closed wall height,
recessed relief, outward winding, unchanged footing and altered-art rejection
plus current table/TV/floor/shared/loader/water checks. Temporary test binaries
removed. Production handle 75230 is confirmed live compiling. Capture must
check `native curtain wall requires complete source bounds` in addition to the
current square-table/floor/TV markers. No new curtain/floor/TV native visual
approval is claimed until the fresh capture has been inspected. Full 120-map
scope remains active and incomplete.

### Five-room native review and actual Red drawing corrections

Production handle 75230 finished successfully in 9m06s and all current
square-table/floor/TV/curtain markers were verified. Guarded native capture
47797 completed all five maps in `target/kanto-connected-wall-table-native-review`;
all five original comparison images were inspected and snapshot removed.
Blue's/Bill's houses show larger pink/light squares, square gold tables, blue
TV bodies and source curtains. Red's gold floor no longer has noisy stripes,
but its grain is too faint. Red's first-floor table remained narrow/pink with
flat feet underneath; its first-floor TV and upstairs screen still retained
pale imported finishes. Cafe's existing raw tile motifs and generic service
fixtures remain outstanding. These rooms are not approved as complete.

Actual pack source inspection confirmed Red's table is the five-row canonical
players_house profile across blocks08/09/0c/0d, including feet1c/40/40/1d.
The native resolver now validates that complete profile and reserves all five
rows while keeping its physical tabletop square at 32 pixels. Its gold finish
uses actual top tile23 light/medium swatches. Altered or clipped feet prevent
the enhanced placement. The temporary pack-inspection helper and audit JSON
were removed.

Native TV matching now includes actual players_house11 first-floor TV+stand
and players_house13 screen-only upstairs TV. Screen-only geometry reuses the
selected CRT triangles with the cabinet excluded, through the existing shared
surface-selection helper (visibility extended to crate scope). The CRT fits
within its own physical bounds and never leaves unused below-floor vertices.
Its source console remains independently claimed. Source stand08 was inspected;
medium gold is at[1,0], correcting a first guessed black-outline sample. All
temporary Rust helper sources/executables were removed.

Wood plank seams and grain were strengthened after native evidence showed the
floor too plain. Staggered plank tint phase now follows the same world phase as
its joins. Current source checks passed in
`target/kanto-red-complete-drawing-test-receipt.log`: 38 Kanto,21 interior-model,
29 placement checks plus associated shared/loader/water checks; scratch removed.
The last changes after that test run were the verified stand UV and diagnostic
marker wording. New production handle37116 is live. Before capture, require
`native complete square dining table requires named wood and cloth roles` and
`native screen-only television requires complete CRT geometry`, plus existing
floor/TV/curtain markers. Native Red variants and stronger wood remain pending.
Full120-map Kanto scope remains active and incomplete.

### Native Red drawing review and café shared architecture

Production37116 passed in4m59s. Guarded serial native review14831 completed
all five maps in `target/kanto-red-complete-drawing-native-review`; all original
comparison images were inspected and snapshot removed. Red1F now has a square
source-gold table without flat feet below it. Red2F has a blue CRT without the
extra cabinet. Gold wood floors now show staggered planks and grain. These
bounded corrections are verified; remaining cabinet/console/bed style and wider
Kanto room coverage are not approved as complete.

The café source audit confirmed two complete block04 display drawings, exactly
matching the source used by the shared arcade prize-display cabinet. The café
now resolves those drawings to that same canonical asset, retaining its cabinet
structure and merchandise variety instead of leaving floor-facing graphics.
Gold source wall fragments from blocks02/13/28/29/2d now reuse the native closed
wall course at one connected16-pixel plane; the poster stays in its source wall.
Cafe service counter layout is unchanged. Other maps do not inherit this café
architecture resolver. A repetition guard ensures a four-cell wall is emitted
once, avoiding overlapping duplicate courses.

Current tests passed in `target/kanto-cafe-shared-source-test-receipt.log`:
complete shared display/wall drawings, no overlapping wall geometry, original
footing/claims, altered-art and map-scope guards plus current Red/table/TV/floor/
loader/shared checks. Test scratch and the café audit JSON were removed. Build
67713 is live; current café visuals are not verified yet. Native café review
must require `native cafe wall requires source course bounds`.

A temporary pack context helper confirmed the remaining round gray fixture is
`CeladonCafeTrashcan` at game coordinate(7,1). Its exact source is the lower-right
2x2 of game_corner29 (46/47/56/57). Reuse a designated canonical bin with its
source-gray palette and proper small proportions in the next fixture pass;
do not guess its role from sprite appearance or silently substitute a different
object. Helper source/executable were removed. Full120 Kanto interior scope
remains active and incomplete.

Café production67713 finished successfully in5m23s. Guarded native capture3611
completed and its original comparison image in
`target/kanto-cafe-shared-native-review` was inspected; snapshot removed. The
café now visibly has two full closed display cabinets with shelves, drawers,
grilles and package variety, plus a connected source-gold back wall retaining
the green poster. This confirms the bounded shared cabinet/wall change. Its
trashcan remains flat; service counter and remaining room detail still need
work, so the room is not approved as complete.

Fresh current production audits completed successfully across all120 actual
Kanto interiors and16 declared script-block variants, with zero mesher errors
and no maps lacking authored families. Reports were read then removed; ignored
logs are `target/kanto-current-full-interior-audit.log` and
`target/kanto-current-script-variant-audit.log`. Geometry integrity is verified,
not full visual completion: the audit still includes unmodeled source features,
and floors/other intentional flat surfaces cannot be judged complete from
family counts. All120 maps remain the active completion scope.

Café bin sharing correction: complete game_corner block29 lower-right drawing
46/47/56/57 now selects the canonical models/dungeons/gym_bin.mesh.json.
No copied asset was added. A regression proves Arc pointer equality between
the interior TrashCan and dungeon GymBin geometry; named material roles sample
black/light-gray/dark-gray pixels from the actual café source drawing. The bin
is fitted to10×10×8 pixels inside its16×16 drawing. Unknown roles retain2D.
Fresh source regressions passed (41 Kanto,22 interior-model,31 placement tests,
plus the existing shared/missing-asset and water checks); receipt:
`target/kanto-cafe-trashcan-source-test-receipt.log`. Native review pending.

Bin production40908 passed in4m54s. Guarded native capture78444 passed; the
original2560×1600 image and comparison were inspected in
`target/kanto-cafe-trashcan-native-review`. The bin is a compact ribbed closed
volume with a light cap and gray rim, fits beside the counter, and no longer
leaves the flat round glyph. Snapshot removed. This verifies the bounded bin
sharing correction; service counter finish and busy source floor remain open,
and all120 Kanto interiors remain the active scope.

Café counter correction extends the shared connected-counter kit with an
exact24×16 CeladonCafe network at(16,0), guarded by its whole source halo.
Only34 source counter cells are owned; the service bay, shelves and wall are
excluded. Existing counter00..0f modules provide joined volumes, exposed end
panels and corners. Native green worktop and gold/brown side samples come from
source0e/1e independently; height stays8 source pixels and footing is unchanged.
No new mesh asset or alternate3D substitution. Source47604 and network77390
regressions passed, including all11 connected-counter tests plus42 Kanto tests.
Receipts: `target/kanto-cafe-counter-source-test-receipt.log` and
`target/kanto-cafe-counter-network-test-receipt.log`. Native review pending.

Counter production20381 passed in5m45s; native44107 completed. Its original
image was inspected: joined gold panels and clear service bay were correct,
but green incorrectly reached some vertical gold trim. This capture is not
the final accepted paint. The source now applies green only to named upward
cap faces, deriving their datum independently of the slightly raised grain
primitive. Initial overly strict cap-datum regression failed and was repaired;
rerun12815 passed all16 module cap/trim assertions plus11 network,43 Kanto,
23 interior-model and31 placement tests. Receipt:
`target/kanto-cafe-counter-cap-source-test-receipt.log`. Corrected native
review remains pending.
