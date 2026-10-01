# Remaining source-family review

This is an implementation and visual-review checklist. Counts are residual
8-pixel source cells from inspected base maps, not objects, triangles or a
percentage of finished scenes. Plot counts can include backing ground. Dynamic
map states and individual species art remain separate work in the broader
conversion checklist.

The final-scenery production audit verifies all 388 base maps without a mesher
error and records 254,371 cells consumed by authored geometry, including named
surface finishes. This is +550 over the four-kit checkpoint: +462 Cable Club and
+88 native ground bindings. Representative native appearance review is complete for the four Gyms, stores,
sign families, Cable rooms, keeper room and exterior player reveal. Controller
and per-map closure are tracked separately; coverage does not provide art signoff.

The former seventeen-entry checklist now has ten open entries. The additional
packages below are also required; the original list was not an exhaustive
remaining-world denominator. Staged Facility/Radio, Ship and traditional-room
kits remain open until integrated, audited and reviewed natively.

## Ten open entries from the former checklist

| Family | Residual cells | Maps | Required result |
| --- | ---: | ---: | --- |
| Facility instrument cabinets | 208 | 5 | PowerPlant and related rooms; faithful equipment silhouettes |
| Facility workstation plots | 152 | 4 | Complete desks/equipment and their native openings; plot count |
| Radio desks/counter plots | 380 | 6 | Joined desks, consoles and counter returns; plot count |
| Ship table/captain-desk plots | 272 | 4 | Complete cabin furniture; plot count, excluding the installed Lighthouse 6F table |
| Timber divider screens | 40 | 1 | WiseTriosRoom; panels and separate returns |
| Barn stall rails | 24 | 1 | Route39Barn; three complete panels |
| Theater stage/backdrop | 240 | 1 | DanceTheater; coherent stage and backdrop over the existing raised treatment |
| Magnet Train body/boarding plots | 128 | 2 | Both stations; preserve the boarding opening; plot count |
| Park small round props | 44 | 3 | Confirm whole-object identity in native context before authoring |
| Park fountain rim/basin | 48 | 3 | Refine the existing grouped animated treatment |

## Additional bounded object and architecture packages

| Package | Residual cells / maps | Source evidence and required result |
| --- | --- | --- |
| Café tables and service counters | 64 table + 86 counter / 3 | `cafe.rs` exact table/counter selectors; complete round tables and connected C-shaped counters, preserving the operator bay |
| Game Corner rear architecture and counters | 288 rear-course + 88 counter / 2 | `casino.rs`; source-specific walls, payout equipment, pillars, doorways and connected counters |
| Facility divider networks | 948 / 3 | PowerPlant 116, TeamRocketBaseB3F 670, TrainerHouseB1F 162; `facility_divider.rs`; replace/refine the existing closed 16-pixel network |
| Underground switch-room walls/gates | 904 / 1 | `elite_four_room.rs`, blocks `2a/37/3d/3e/3f`; validate all script-controlled gate and switch states |
| Warehouse side walls | 288 / 2 | GoldenrodDeptStoreB1F and GoldenrodUndergroundWarehouse; `underground_boundary.rs`, block `0c` west halves and `0e` east halves |
| Lower-deck ship dividers | 208 / 1 | FastShipB1F; `ship.rs`, 120 vertical + 88 horizontal cells; preserve passages and berth openings |
| Center lobby counter ends and PC headers | 44 counter + 44 header / 22 | `pokecenter.rs`; narrow block `03` right-column `0f/25` return and two block `08` wall cells above each PC |
| Rocket Base B1F plants | 12 / 1 | Two complete 2×3 plants in blocks `29/2a`; `rocket_base.rs::plant_local` has no modeled binding |
| Flower Shop display assemblies | 44 / 1 | `flower_shop.rs::display_shape`; finish source-specific bent/corner planters and display tables |
| Cerulean Gym pool props/borders | 176 / 1 | `port.rs`, blocks `2e..31/36..39`, art `0a/0b/22`; establish complete silhouettes in native context, retaining the waterline |
| Vermilion Gym targets/gates | 64 / 1 | `vermilion.rs`, blocks `20/21`; preserve both puzzle states and real openings |

These are source-family counts, not additive object totals. Some families already
have legacy dimensional geometry. For example, the 948 facility cells are called
“plane” by the coverage classifier but already render as a closed divider
network. The Cerulean count is not 176 barrels. Whole-object reconstruction and
native appearance remain prerequisites for closing these packages.

## Integrated source families and corrections

Seven former entries now consume their intended drawings or deliberately retain
the correct live surface: Gym planters (720 cells), Azalea's central tree (9),
Viridian's maze (356), Celadon's hedges (168), outdoor signs (348), department
wall courses and department U displays. Representative native views now cover all seven families. Movement/interaction
and additional per-map compositions remain a separate closure requirement.

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
path ownership remain 2,488 and 1,168. Targeted native views now cover all three maps. Route19 review caught paving
squares under offshore rocks; they now use verified block `43` water and its
native datum, with both corrected groups reviewed. See [ground-binding notes](art/native-ground-bindings.md).

Other established kits remain integrated:

- Kanto capped posts consume 1,904 cells across 21 maps; gate counters/phones
  consume 1,534 counter and 96 phone cells across 28 maps
- Cable Club partitions consume 968 cells across eleven rooms and the capsule
  consumes 88; the existing ceramic finish adds no object-coverage credit
- Lighthouse masonry consumes 4,176 plain and 640 window cells across six
  floors. The 6F tea table, cot and stool now consume 24, 8 and 4 cells; its
  checker floor is a surface finish. The source long fixture is a tea table, not machinery; additional camera and
  movement arrangements still need contextual review. See [chamber notes](art/lighthouse-chamber.md)
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

The 228 remaining cutout-classified cells are fully accounted for: 176 flat
tatami cells, 40 Wise Trio screen cells and 12 Rocket Base B1F plant cells. The
88 ground-binding leftovers and 40 beta PC cutout cells are now consumed.
Facade (2,702), plane (10,266) and raised (9,614) classifications did not change
in this last integration.

The 176 traditional-house block `04` cells in Kurt's House and Dance Theater are
continuous tatami floor. They currently stay flat despite the old “cushion”
classifier: both cushion paths reject their unavailable zero-height `50`
ground sample. Of those cells, 132 receive the existing tatami finish and 44
edge cells (`46/56`) retain source art. Complete the strict continuous-floor
finish and correct the classification; do not activate raised cushions by
adding a broad ground fallback.

Ordinary-house plain wall-art `00` remains on 258 cells across 56 maps, but
`append_known_room_backing` already supplies room wall geometry. Review source
skins and seams against that geometry before suppressing anything. This is not
evidence of 258 missing walls.

The 10,266 residual “plane” cells include tower depth/background (3,820), buoy
courses (3,148), facility dividers (948), Elite Four boundaries/invisible fields
(1,016), ruins boundary strips (512), Center landings (390), ship caps (146)
and cave depth fields (286). Of 9,614 “raised” cells, 8,294 are existing outdoor
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
