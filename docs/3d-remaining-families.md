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
| Radio studio side-wall edges | 12 / 2 | LavRadioTower1F 4 and RadioTower2F 8; block `20`, rightmost column `8a/8b`; preserve the thin wall role rather than treating these as desks or floor |
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
