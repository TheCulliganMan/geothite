# Remaining source-family review

This is an implementation and visual-review checklist. Counts are residual
8-pixel source cells from inspected base maps, not objects, triangles or a
percentage of finished scenes. Plot counts can include backing ground. Dynamic
map states and individual species art remain separate work in the broader
conversion checklist.

The latest production audit verifies all 388 base maps without mesher errors
and records 256,311 cells consumed by authored geometry, 120 above the preceding
256,191 checkpoint. Complete Fast Ship B1F room fronts replace 36 old partial
claims with 156 source-specific architecture cells; their entrances and four
real corner voids remain untouched. All 805 voxel tests pass, with two existing
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
| Cerulean Gym pool props/borders | 176 / 1 | `port.rs`, blocks `2e..31/36..39`, art `0a/0b/22`; establish complete silhouettes in native context, retaining the waterline |
| Vermilion Gym targets/gates | 64 / 1 | `vermilion.rs`, blocks `20/21`; preserve both puzzle states and real openings |

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

The remaining 12 cutout-classified cells are the two Rocket Base B1F plants.
The latest integration removes the 40 Wise Trio screen cells and reclassifies
176 continuous tatami cells as flat. Current facade, plane and raised totals
are 2,382, 10,266 and 9,446. Those classifier totals are not missing-object totals.

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
