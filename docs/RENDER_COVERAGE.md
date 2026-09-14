# Render coverage

Use breadth-first passes across maps before local polishing. These are sampled
views, not claims of complete map or gameplay coverage. Current pass uses the
144-object profile document and water-backed profile renderer. No builds/tests.

| Map and position | Inspected artifact in /tmp | Finding / next check |
| --- | --- | --- |
| VioletCity (22,18) | coverage-VioletCity-compare.png | Gym facade and trees coherent; distant tower base needs closer verification. |
| AzaleaTown (10,10) | coverage-AzaleaTown-compare.png | Upright trees, signs, house and fences; southern gym cropped. |
| GoldenrodCity (20,23) | coverage-Goldenrod-road-compare.png | Road, building fronts and railings readable; sample from street replaces invalid building spawn (16,20). |
| Route30 (10,40) | coverage-Route30-compare.png | House, cutout trees and water banks readable; foreground sign cropped. |
| VioletPokecenter1F (5,6) | coverage-VioletPokecenter1F-compare.png | PC/healing machine remain thin; matching furniture profiles are a candidate for shared improvement. |

Next breadth pass: additional towns, a cave, a gate, and upstairs/interior types.
Prioritize recurring defects across maps over repeated single-city captures.

## Second breadth pass

All five paired captures completed with the existing executable and 144 profiles;
images inspected, no builds or tests. Paths below are under /tmp.

| Map and position | Artifact | Finding / next check |
| --- | --- | --- |
| OlivineCity (16,20) | coverage-OlivineCity-compare.png | Center/Mart/Gym facades and fences read coherently; lighthouse and port not sampled. |
| MahoganyTown (9,8) | coverage-MahoganyTown-compare.png | Foreground gym roof occludes player, revealing them through transparency; inspect roof projection along road. Northern low building needs full framing. |
| UnionCave1F (5,5) | coverage-UnionCave1F-compare.png | Water banks and rocks have depth; raised wall tops show detached-looking end faces near upper-left passage. Prioritize cave junction geometry. |
| Route29Route46Gate (4,3) | coverage-Route29Route46Gate-compare.png | Booths and actors readable; north doors remain low against floor compared with raised wall. Review shared gate doorway folding. |
| PlayersHouse2F (3,3) | coverage-PlayersHouse2F-compare.png | Table/bed/equipment stand above floor; stair opening visible. No movement or side-angle review. |

Highest-value next fixes from coverage: shared gate doors, cave wall junctions,
and common Center furniture. Continue sampling additional maps alongside fixes.

## Shared Center furniture fix

Added four tileset-scoped fallback profiles for exact PC, healing-machine and
seat drawings. Existing map-specific profiles retain precedence. Current
profile count148. Inspected paired renders at (5,6) in VioletPokecenter1F,
OlivinePokecenter1F and AzaleaPokecenter1F: equipment has closed depth, seat
sides are solid, and actors remain visible. Artifacts:
`/tmp/shared-furniture-<map>-compare.png`. No build/tests.

GoldenrodPokecenter1F could not render: runtime callback initialization places
GOLDENRODPOKECENTER1F_PCC_TRADE_CORNER_RECEPTIONIST at (16,8), outside the loaded
10x8 room. This is a coverage blocker for that map, not a furniture render
result. Track the map/callback mismatch separately.

## Third breadth pass

148 profiles, existing executable, paired images inspected in /tmp; no builds/tests.

| Map and position | Artifact | Finding / next check |
| --- | --- | --- |
| BlackthornCity (18,20) | coverage-BlackthornCity-compare.png | Cliff stairs have clear elevation, but nearby terrace edges show thin vertical slivers; prioritize shared cliff junctions alongside cave ends. |
| CianwoodCity (10,20) | coverage-CianwoodCity-compare.png | House, shore and isolated cliff rocks readable; no full coast or Gym review. |
| IlexForest (1,37) | coverage-IlexForest-compare.png | Distinct forest tree art remains upright; dense canopy/ground texture is visually busy. Gate roof cropped. Initial (10,20) was unwalkable and produced no render. |
| OlivineMart (3,5) | coverage-OlivineMart-compare.png | Refrigerators still lie flat; common Mart furniture is another shared-profile candidate. Shelves and counter otherwise readable in this sample. |

## Shared Mart furniture fix

Added five exact-pattern tileset fallbacks from the reviewed Ecruteak Mart
profiles, preserving earlier map-specific precedence. 153 profiles total.
Inspected paired OlivineMart, VioletMart and AzaleaMart at (3,5), artifacts
`/tmp/shared-mart-<map>-compare.png`: refrigerators now upright with solid depth,
matching shelves have closed sides, and actors remain visible in the sampled
views. Existing red-display shelf rendering remains. No rebuild/tests; all
three captures exited cleanly. Walking aisles and alternate angles remain
separate checks.

## Kanto and Gym pass

153 profiles, existing executable. All paired captures completed and were
inspected; no rebuild/tests. Artifacts under /tmp.

| Map and position | Artifact | Finding / next check |
| --- | --- | --- |
| PewterCity (20,18) | coverage-PewterCity-compare.png | Gym and Mart raised; posts upright; rock row partially cropped. |
| CeruleanCity (18,20) | coverage-CeruleanCity-compare.png | Several complete-looking blue-roof house drawings remain flat while Center rises. High-priority Kanto building-template coverage gap. Player starts on building footprint, so actor overlap is not a walking result. |
| VermilionCity (18,16) | coverage-VermilionCity-compare.png | Center/Mart and banks coherent; right blue-roof house edge has awkward roof join, requiring full framing. |
| VioletGym (4,13) | coverage-VioletGym-compare.png | Raised winding path, entrance statues and trainers readable; wall fixtures are thin. No traversal reviewed. |

Next shared building investigation: Kanto blue-roof house templates, using
Cerulean and Vermilion together rather than polishing a single city.

## Kanto compact houses: middle spans recognized

Extended append_kanto_building_placements to recognize $02/$30, zero or more
$09 middle blocks, and $03 end cap, widths2–6. Previously only width2 matched.
Built native renderer once. Inspected `/tmp/kanto-houses-CeruleanCity-compare.png`
and `/tmp/kanto-houses-VermilionCity-compare.png` at the prior coverage positions.
Cerulean's wider houses now have upright facades and raised roofs; Vermilion's
existing compact house remains raised. Roof-end padding still produces dark
notches beside sloped blue cap art, a shared meshing issue now easier to see.
No tests; both captures exited cleanly. This fixes detection, not the roof-cap
silhouette. Existing Cerulean spawn overlaps a building and is unsuitable for
actor/collision validation.

## Additional Kanto towns after house detection fix

153 profiles, current executable; all four paired captures inspected and exited
cleanly without rebuild/tests. Artifacts under /tmp.

| Map and position | Artifact | Finding / next check |
| --- | --- | --- |
| PalletTown (8,8) | coverage-PalletTown-compare.png | Two-storey houses raised, but broad dark roof padding is visible behind blue roofs. Confirms roof-padding issue extends beyond compact houses. |
| ViridianCity (18,18) | coverage-ViridianCity-compare.png | Trees, posts, sign and distant houses readable; this view does not fully frame major buildings. |
| LavenderTown (8,8) | coverage-LavenderTown-compare.png | Complete foreground compact house clearly exposes dark side/back roof padding; useful reference for shared roof fix. Center/Mart facades coherent. |
| FuchsiaCity (18,18) | coverage-FuchsiaCity-compare.png | Garden trees and signs upright; rock barriers remain thin cards. No Gym or Safari-area review. |

Use Lavender compact house and Pallet two-storey house together when fixing
roof padding; avoid solving only one signature.

## Kanto house roofs follow source outline

Enabled existing plan-shaped roof meshing for Kanto $02/$30 compact houses and
$38 two-storey houses, alongside existing $0c/$20 buildings. Built once and
inspected `/tmp/roof-outline-LavenderTown-compare.png` and
`/tmp/roof-outline-PalletTown-compare.png`. Broad dark backing behind Pallet's
roofs is removed; Lavender's roof cap follows its blue outline more closely.
Small dark/gray remnants outside Lavender's roof ends remain, likely involving
remaining edge/floor treatment; do not consider the entire padding issue closed.
Facades remain upright. Both paired captures exited cleanly; no tests.

## Large-city and distinct-interior coverage

Current renderer,153 profiles,no builds/tests. Paired images inspected.

| Map and position | Artifact under /tmp | Finding / next check |
| --- | --- | --- |
| SaffronCity (18,23) | coverage-road-SaffronCity-compare.png | Silph entrance facade raised and road player framed; upper landmark cropped. Replaces building-overlap spawn (18,20). |
| CeladonCity (20,21) | coverage-road-CeladonCity-compare.png | House fronts, fence and banks readable; chosen position intersects fence, so no actor/collision conclusion. Earlier (20,18) overlapped facade. |
| MrPokemonsHouse (3,5) | coverage-MrPokemonsHouse-compare.png | Table and right cabinets have depth; left cabinets and blue seat remain low/flat. Distinct interior furniture gap. |
| SproutTower1F (10,12) | coverage-road-SproutTower1F-compare.png | Central pillar appears as a low box rather than tall support; entrance statues also flat. Room perimeter and actors readable. Initial (4,7) unwalkable, no capture. |

Improve coverage coordinate selection from source walkability before future
batch runs; rendering an actor over an impassable source drawing is not useful
movement evidence. Sprout pillar and shared tower statues join the fix queue.

## Tower statue profile pass

Added two exact tileset fallbacks for the reviewed red-orb tower statue source
patterns; map-specific Ecruteak profiles retain precedence. EcruteakGym paired
render remains coherent (`/tmp/shared-statues-EcruteakGym-compare.png`).
Attempted separate golden-statue profiles for Sprout $30/$31, but wider live
captures showed incorrect gold-head presentation both with and without ground
masking. Removed those two experimental profiles; golden statues remain an
unresolved source/meshing investigation. Current count155. No build/tests.
`/tmp/statue-wide-2.5d.png` is rejected trial evidence, not current result.

## Lab, school and traditional home

155 profiles,current executable,no builds/tests. Closed the completed Sprout
live renderer before running these one-shot captures. All exited cleanly and
paired images were inspected.

| Map and position | Artifact under /tmp | Finding / next check |
| --- | --- | --- |
| ElmsLab (5,8) | coverage-ElmsLab-compare.png | Existing desk, bookcases and equipment profiles retain solid depth; actor feet/readability coherent. North-center blue cabinet still low. |
| EarlsPokemonAcademy (3,5) | coverage-EarlsPokemonAcademy-compare.png | Desks and chairs visible, but north bookcases and chalkboard remain floor-oriented; high-value school furniture/wall coverage gap. |
| KurtsHouse (3,5) | coverage-KurtsHouse-compare.png | Table, cabinetry and wall readable; cushions remain floor-level as expected for seating mats. Small threshold details lie flat. |

The academy makes the missing upright wall/cabinet treatment especially clear;
use its source-pattern matches alongside MrPokemonsHouse when considering a
shared interior fix. Do not generalize by color alone.

## Paired bookcases in lab/facility interiors

Added exact tileset profiles for lab $14 and facility $06 paired bookcases,
with four roof pixels and eight pixels of depth. Their source patterns differ,
so each uses its own tile matrix and floor ID. Current count157. Inspected
`/tmp/bookcase-EarlsPokemonAcademy-compare.png` and
`/tmp/bookcase-MrPokemonsHouse-compare.png`: previously flat northern bookcases
now stand upright with complete faces and solid sides. Desk/actor presentation
remains readable. Chalkboard, blue seat and other wall gaps remain separate.
Both renders exited cleanly, no build/tests.

## Route and park pass

157 profiles,current executable,no builds/tests. Inspected paired captures:

| Map and position | Artifact under /tmp | Finding / next check |
| --- | --- | --- |
| Route31 (10,8) | coverage-Route31-compare.png | Trees and sign upright; thin vertical ledge-end face visible at grass break. Shared ledge-end fix remains relevant beyond Route29. |
| Route32 (10,10) | coverage-Route32-compare.png | Raised cliffs and tree cutouts readable; blocky cliff corner joins remain. |
| NationalPark (10,20) | coverage-NationalPark-compare.png | Park sample captured successfully; see image for vegetation and path layout. |

Route34 (10,20) failed to produce a capture: required day_care_mon_1 PNG could
not be decoded. The runtime repeated the error continuously; stopped exact
capture PID26774 after confirming its command, allowing the batch to continue.
This map needs dynamic Day Care sprite resolution reviewed. No claim of Route34
visual coverage. Failed-render loops should terminate promptly in the toolkit.

## Route34 dynamic Day Care sprites

Resolved SPRITE_DAY_CARE_MON_1/2 from the snapshot's man/lady resident species
before asset lookup; inactive/empty slots are omitted. Uses existing species
menu-icon fallback. Built native renderer (corrected species.id field after an
initial compile error). Same previously failing Route34 (10,20) now renders
both views and exits cleanly in1.83s. Inspected
`/tmp/route34-resolved-compare.png`: Day Care facade, fence, banks and actors
visible. This proves empty-slot capture recovery; occupied resident sprite and
animation behavior still require a populated-state review. No tests.

## Routes35–37 and Lake of Rage

157 profiles,current executable. All paired captures exited cleanly and were
inspected, no rebuild/tests.

| Map and position | Artifact under /tmp | Finding / next check |
| --- | --- | --- |
| Route35 (8,20) | coverage-Route35-compare.png | Fences, trees and several actors readable on the broad path. |
| Route36 (10,8) | coverage-Route36-compare.png | Spawn lies within source tree artwork; confirms cutouts remain upright but provides poor route-layout coverage. Replace coordinate before movement review. |
| Route37 (7,10) | coverage-Route37-compare.png | Apricorn trees upright; thin ledge-end faces repeat at northern grass break. Spawn overlaps tree art, unsuitable for movement conclusions. |
| LakeOfRage (10,20) | coverage-LakeOfRage-compare.png | Tree-filled shoreline sampled, player overlaps tree artwork. Need valid shoreline or surf starting point for meaningful traversal coverage. |

The tool's successful startup is not enough to establish a useful capture
position: these outdoor impassable-art spawns can still render. Coordinate
selection needs an explicit walkability/source-occupancy check in the toolkit.

## Clear-position route coverage, Routes36–42

Inspected six paired renders with the existing executable and 157 profiles;
no build or tests. These are stationary samples, not traversal validation.

| Map and position | Artifact under /tmp | Finding |
| --- | --- | --- |
| Route36 (44,9) | coverage-clear-Route36-compare.png | Clear path, upright sign and trees; flower ground patches visibly checkerboard. |
| Route37 (5,10) | coverage-clear-Route37-compare.png | Clear path; thin vertical ledge ends remain at northern grass opening. |
| LakeOfRage (22,29) | coverage-clear-LakeOfRage-compare.png | Clear shoreline grass; lake, actors and signs visible, shoreline corner joins need polish. |
| Route38 (30,8) | coverage-Route38-compare.png | Path junction, raised fences, sign and upright trees readable. |
| Route39 (6,6) | coverage-Route39-compare.png | Farm buildings, doors, roofs and fence openings readable from clear approach. |
| Route42 (47,9) | coverage-Route42-compare.png | Cave approach and raised mountain visible; narrow cliff side seams recur. |

The first three replace the earlier source-tree-overlap positions. Preserve
2.5D tree cutouts. Prioritize recurring ledge/cliff seams across these maps
rather than returning to single-city decoration.

## Shared Johto ledge taper

Removed Route29-only dispatch from the existing exact johto $57 east-edge
ramp rule. Route31 and Route37 use the same authored block at grass openings.
One native build required for mesh code; no tests. Inspected paired renders
`/tmp/shared-ledge-Route31-compare.png` (10,8),
`/tmp/shared-ledge-Route37-compare.png` (5,10), and
`/tmp/shared-ledge-Route39-compare.png` (6,6).
Route37's left bank now eases down at its east end; the opposite bank's west
end still has a vertical edge. Route31 also retains an edge at its opening,
so this is a partial shared improvement, not a complete seam fix. Route39's
farm facades and fences remain visually coherent. A mirrored west-end rule
needs separate neighbor/source review before applying it globally.

## Mirrored ledge end

Added west-end taper to johto $57, restricted to its first source column and
an adjacent flat cell. One native build; no tests. Inspected
`/tmp/paired-ledge-Route37-compare.png` (5,10): the previously upright west
edge of the right bank now slopes into the opening, matching the left bank.
`/tmp/paired-ledge-Route31-compare.png` (10,8) still shows a thin edge beside
its different $4b opening. Keep that separate from the verified $57 correction;
this does not establish that all ledge seams are fixed.

## Lighthouse, gym, shop and house coverage

Existing executable, no build/tests. Inspected paired captures at these clear
floor positions; stationary samples only.

| Map and position | Artifact under /tmp | Finding |
| --- | --- | --- |
| OlivineLighthouse1F (12,14) | interior-coverage-OlivineLighthouse1F-compare.png | Floor and actors readable; stepped wall sections appear thin/disconnected around dark interior void. Needs dedicated wall topology work. |
| CianwoodGym (4,14) | interior-coverage-CianwoodGym-compare.png | Shared red-orb statue profiles verified outside Ecruteak; both entrance statues upright with volume. Wooden partitions and upper boulders visible; traversal not reviewed. |
| GoldenrodBikeShop (4,5) | interior-coverage-GoldenrodBikeShop-compare.png | Counter has depth, but bicycle displays and rear shelf remain flat. Prioritize this tileset's missing object coverage. |
| OlivineTimsHouse (1,5) | interior-coverage-OlivineTimsHouse-compare.png | Table, stools, rear fixtures and plants raised; left foreground plant cropped by framing. |
| CianwoodPharmacy (1,5) | interior-coverage-CianwoodPharmacy-compare.png | Same house layout renders with raised furniture and wall fixtures; seated actor overlaps stool naturally in this view. |

## Bike shop rear display

Added map-specific champions_room $0a full 4x4 display profile, floor32,
top4/depth8. Count158. Inspected `/tmp/bike-display-compare.png` at (4,5):
rear cabinet now upright with complete shelves and closed sides. No build/tests.
Tried zero-depth ground-masked bicycle cards at $04/$03; inspected
`/tmp/bike-upright-compare.png` and rejected them because the mask lost bicycle
pixels and produced broken silhouettes. Removed all three trial profiles;
retained only the verified rear cabinet. Bicycle art needs a dedicated mask.

## Station, department store and research center

No build/tests. Inspected `/tmp/coverage-interiors2-{Map}-compare.png`:
GoldenrodMagnetTrainStation (9,14) has readable platform/entrance and actors,
but seats are thin upright cards and train remains low relief.
GoldenrodDeptStore1F (8,5) has raised counter/walls, but pink seat surfaces
stand as boards and need proper seat/back geometry.
RuinsOfAlphResearchCenter (1,3) has raised rear bookcases and desks; rear
instrument cabinet was flat. Added exact facility $3b origin(2,1) three-row
profile, ground1/top4/depth6, map-specific. Count159. Inspected
`/tmp/research-cabinet-compare.png`: cabinet now upright, full face visible
above the nearby scientist. Northern wall remains low, plant geometry thin.
PewterMuseum1F is absent from this content pack; no museum coverage claimed.

## Shared department-store cushions

Added two exact mart $07 profiles for the upper/lower 2x2 cushion source
patterns. Each folds twelve source pixels onto a twelve-pixel-deep seat,
with six-pixel height and closed sides. Count161. No build/tests.
Inspected `/tmp/store-seats-GoldenrodDeptStore1F-compare.png` and
`/tmp/store-seats-CeladonDeptStore1F-compare.png`, both at (8,5): pink surfaces
now lie horizontally as raised cushions rather than standing like boards.
Actors, counter and clear aisle remain readable in both sampled views.
These stationary captures do not verify collision or orbit behavior.

## Shared station seats

Added four exact train_station profiles: $01 paired row and $02 paired column,
2x2 tile pattern64/65/66/67, floor61, top/depth10, height6. Count165.
No build/tests. Inspected Goldenrod at (9,14),
`/tmp/station-seats-GoldenrodMagnetTrainStation-compare.png`, and final Saffron
same coordinates, `/tmp/station-seats-saffron-complete-compare.png`.
Both row and column arrangements now have raised cushion tops and closed
sides. Aisles remain readable. Planters still contain flat foliage; train and
platform geometry require further coverage. No traversal/orbit claim.

## Game corners and Center regression sample

Inspected `/tmp/coverage-casino-{Map}-compare.png`: GoldenrodGameCorner (3,5),
CeladonGameCorner (9,5), CinnabarPokecenter1F (5,6). Center machines and seats
remain raised. Both game corners expose thin/open gaming-row geometry and
flat rear fixtures. Added two shared game_corner $0e rear machine profiles,
top4/depth8/floor1, exact 2x4 source patterns. Count167. No build/tests.
Inspected `/tmp/casino-machines-GoldenrodGameCorner-compare.png`: four rear
machines now have complete upright faces and closed sides.
Celadon equivalent capture at (9,5) only shows a cropped machine at far right;
it establishes room context, not full machine validation. Rear shelving and
main gaming rows remain unresolved and visibly need work.

## Game-row cabinet volume

Added three exact shared profiles for game_corner $07 rows0/2 and $0b row0,
32x16 source patches. Top/depth12, height12 closed volumes. Count170.
Inspected `/tmp/casino-row-volume-compare.png` (Celadon9,5) and
`/tmp/casino-row-volume-goldenrod-compare.png` (Goldenrod3,5).
The colored gaming rows now have tops and closed sides rather than open
upright panels. Nearby stools remain thin and cabinet color bands stretch
on the vertical faces; finer machine detail and reverse views remain work.
No build/tests, both paired captures exited successfully.

## Game-corner stool seats

Four exact shared game_corner $05/$06 profiles cover both stool rows.
Source10/11/26/27, floor1, top/depth10, height5. Count174.
Inspected `/tmp/casino-stools-GoldenrodGameCorner-compare.png` (3,5) and
`/tmp/casino-stools-CeladonGameCorner-compare.png` (9,5). Seats now have
horizontal tops and closed shallow sides instead of upright panels.
Occupied seats remain visible beneath actors in these views. No build/tests.
Main cabinets still have stretched side bands; reverse-angle and movement
validation remain pending. Continue coverage in other tilesets next.

## Port, ship and radio tower

Inspected `/tmp/coverage-port-{Map}-compare.png`: OlivinePort (10,14),
FastShip1F (24,6), RadioTower1F (4,3). No build/tests.
Port piers and pilings remain mostly flat, requiring substantial tileset work.
Ship walls rise but portholes sit on low bands and upper wall backing looks
oversized. Radio Tower counter rises, plants are low, PC was flat.
Added exact shared radio_tower $12 PC origin(0,1), floor1/top4/depth6.
Count175. Inspected `/tmp/radio-pc-compare.png`: complete upright PC and
closed sides visible. Only RadioTower1F instance verified this pass.

## Shared port piling depth

Added28 exact matches of port tile matrix1/2/17/18 across its metatiles.
Water ground20, top10/depth12, height8. Count203. No build/tests.
Inspected `/tmp/port-pilings-OlivinePort-compare.png` (10,14) and
`/tmp/port-pilings-VermilionPort-compare.png` (8,11): repeated pilings now
have raised tops and closed sides along both dock arrangements. Walkways
remain readable. Pale source-art lower rims remain conspicuous, and piers
are still flat; this is piling coverage only, not finished port geometry.

## Ice Path, Dragon's Den and Power Plant

Existing executable,203 profiles,no build/tests. Inspected paired captures:

| Map and position | Artifact under /tmp | Finding |
| --- | --- | --- |
| IcePath1F (20,8) | coverage-special-IcePath1F-compare.png | Clear floor at stairs. Major stretched icy wall textures and large exposed vertical faces; isolated rocks raised. Needs wall topology/UV work, not decorative profiles. |
| DragonsDen1F (6,14) | coverage-special-DragonsDen1F-compare.png | Clear lower-room floor. Detached wall sections around corners and exposed floor outside room; ladder appears on floor. Shared cave geometry is a priority. |
| PowerPlant (7,7) | coverage-special-PowerPlant-compare.png | Desks, cases and machinery raised; meeting-table chairs flat and northern wall low. |

Prioritize the cave/ice structural defects over further isolated furniture
profiles. Static captures establish visible geometry issues only; no movement,
collision, or ladder-transition validation in this pass.

## Cave straight boundary grouping

Native cave rule now groups $08 west-facing and $0a east-facing repeated
source columns into one two-band wall plane at height16. Previously each
column became an independent six-pixel ledge at a different plane.
One native build, no tests. Inspected `/tmp/cave-wall-pair-compare.png`,
DragonsDen1F (6,14): straight room sides connect vertically now. Detached
corner pieces and exposed exterior floor persist; mixed corner blocks need
separate topology work. This sample does not prove other caves complete.

## Shared cave-wall follow-up

No build/tests. Inspected `/tmp/cave-shared-UnionCave1F-compare.png` (5,5):
long straight side wall now continuous; diagonal joins still leave triangular
separations, and nearby rock/floor transitions remain uneven. This extends
straight-wall evidence beyond Dragon's Den, not overall cave approval.
DarkCaveVioletEntrance (3,5) produced an unlit black scene with visible actors;
paired capture rejected the nearly uniform 2D frame. Inspected raw 2.5D PNG,
`/tmp/cave-shared-DarkCaveVioletEntrance-2.5d.png`; no wall-quality claim.
Dark-cave geometry review requires a lit scene/state. Do not disable the
uniform-frame check just to label this capture successful.

## Rejected diagonal endpoint trial

Tried zero bevel in append_diagonal_cave_corner, one native build, inspected
`/tmp/cave-corner-aligned-compare.png` at UnionCave1F (5,5). Visible triangular
corner separations remained. Reverted the source change: no demonstrated
visual improvement. Root cause needs corner placement/neighbor connectivity
review, not only bevel adjustment. No tests. Rebuilt restored source so the
reusable executable matches the retained implementation.

## Cave side-view diagnosis

Verified shipped cave $10/$11/$35 source matrices match diagonal recognition;
no source-ID mismatch found. Live render of DragonsDen1F (6,14), request
zoom0/orbit2, inspected `/tmp/cave-side-review-2.5d.png`. No build/tests.
Side view exposes thin backs of small rock drawings, disconnected outer top
surfaces, and striped corner side faces. A front-view-only correction is
insufficient. Require front and side views for the next cave topology change.
Closed the task-owned live render after capture to preserve iteration speed.

## Cave rock side volume

Added31 exact cave rock profiles for12/13/28/29; count234. Initial cave plus
 dark_cave trial exceeded256-object limit and was rejected. Removed dark_cave
trial; corrected bevel profile to full16px top as required by validation.
Final profile uses16x16 footprint, height10, bevel3, ground22. Live reload
accepted; inspected DragonsDen1F(6,14),zoom0/orbit2 saved at
`/tmp/cave-rock-volume-side.png`, then orbit0 at
`/tmp/cave-rock-volume-2.5d.png`. Rocks retain volume from the side instead of
vanishing into panels. Source textures stretch over bevels; front silhouettes
are more angular. Wall corners remain unresolved. No build/tests. Closed live
renderer. Dark-cave variants are not included or verified.

## Cave rock breadth check

234 profiles,current executable,no build/tests. Inspected
`/tmp/rock-coverage-UnionCave1F-compare.png` (5,5) and
`/tmp/rock-coverage-SlowpokeWellB1F-compare.png` (14,13).
The profiles apply to isolated rocks and repeated rock rows in both layouts.
Volume persists, but front texture reads as dark striped/tent-like forms;
this is a material/art-mapping defect, not finished rock quality. Slowpoke
Well also shows the same separated corner geometry and uneven ledge joins.
MountMortar1FOutside uses dark_cave, outside the current rock-profile scope;
its grid was inspected but no rendered coverage claimed this pass.

## Cave rock source-face correction

Replaced full-art bevel mapping on31 cave rock profiles with separate8px
source cap/front courses, depth10 and height8. Count234 unchanged.
Inspected `/tmp/rock-face-UnionCave1F-compare.png` (5,5) and
`/tmp/rock-face-SlowpokeWellB1F-compare.png` (14,13): striped tent-like fronts
removed and original light cap/dark face restored. Side review in Dragon's
Den (6,14),zoom0/orbit2, `/tmp/rock-face-side-2.5d.png` confirms retained
closed depth. Shapes are still boxy and lower than the original cutouts;
further silhouette work remains. No rebuild/tests. Closed live renderer.

## Azalea, Blackthorn and Saffron gym pass

Inspected `/tmp/coverage-gyms-{Map}-compare.png`: AzaleaGym(4,12),
BlackthornGym1F(4,15),SaffronGym(9,15). No build/tests.
Azalea entrance statues were flat. Added two exact elite_four_room $25/$26
profiles, floor3/top4/depth8/ground mask. Count236. Inspected
`/tmp/azalea-statues-compare.png`: both statues now upright with closed sides.
Blackthorn lava/platform boundaries have shallow depth and boulders are
raised; no traversal claim. Saffron partition walls rise, warp pads remain
floor-parallel, but wall junctions and statue silhouette need refinement.
Azalea garden perimeter fixtures remain thin in this angle.

## Pewter, Cerulean and Vermilion gyms

No rebuild/tests. Inspected `/tmp/coverage-kanto-gyms-{Map}-compare.png`:
PewterGym(4,10),CeruleanGym(4,12),VermilionGym(4,14).
Pewter rocks raised but statues flat. Cerulean pool paths readable, statues
thin; swimmers' full standing sprites remain visible above water and need
presentation review. Vermilion cans flat and entrance statues thin.
Added map-specific game_corner $1f origin(2,2) trash-can profile, floor1,
top/depth8,height8. Count237. Inspected `/tmp/vermilion-cans-compare.png`:
all15 cans have raised lids and closed fronts in this view. They remain
boxy; puzzle interaction and reverse angles not validated.

## Rejected Pewter traditional-statue profiles

Tried tower $19/$1a full statue profiles at PewterGym(4,10), inspected
`/tmp/pewter-statues-compare.png`: raised but head silhouette corrupted.
Split head/base trial `/tmp/pewter-statues-split-compare.png` separated head
from pedestal in depth as well as losing mask pixels. Removed both profiles;
count237 restored. No build/tests. Need explicit head mask and aligned part
offsets; do not generalize the red-orb statue treatment to this artwork.

## Celadon, Fuchsia and Fighting Dojo

237 profiles,current executable,no build/tests. Inspected
`/tmp/coverage-gym-final-CeladonGym-compare.png` (4,14),
`/tmp/coverage-gym-final-FuchsiaGym-compare.png` (4,14), and
`/tmp/coverage-gym-final-FightingDojo-compare.png` (4,8).
Celadon retains upright2.5D trees and raised planter bases; traditional statues
remain thin. Fuchsia actors and clear floor visible; invisible barriers must
remain invisible, so do not infer missing geometry from collision. Its statues
share the silhouette problem. Dojo has eight traditional statues, several flat
and others thin, establishing this as a shared high-impact artwork issue.
Resolve mask/part alignment centrally before creating more tileset variants.
No movement, warp or collision verification in this static pass.

## Statue alignment isolated from masking

Read live part placement: front plane includes source rect y+h, so upper
16px part needs offset_pixels[1]=16 to align with lower16px pedestal.
Pewter trial `/tmp/statue-align-compare.png` confirms head/base alignment,
but head still appears striped even with mask none. Thus ground masking alone
does not explain the head artifact; inspect source sampling/UVs before adding
an explicit mask. Removed trial profiles,count237. No build/tests.

## Grouped prop ownership guard

Built-in grouped prop replacement now returns when any source cell is already
claimed; explicit overlay mode remains allowed. This prevents a second grouped
replacement drawing over live-profile geometry. One native build,no tests.
Pewter statue trial `/tmp/statue-ownership-compare.png` still showed striped
heads, so ownership alone is not the statue fix. Removed trial profiles,
count237. Inspected `/tmp/ownership-trees-compare.png` at Route37(5,10):
upright2.5D trees and apricorns preserved. Retain ownership guard but investigate
prepared source texture changes before attributing head defects to UV math.

## Statue texture-path inspection

Inspected lib.rs terrain build: frame.map_texture is cloned unchanged into an
immutable mesh-owned atlas; TerrainImageSamples captures the same frame.
Host visual-world tile handles are copied from resolved tile handles before
composition. No statue-specific texture rewrite found in this path.
Live profile UVs use grid cell bounds and subdivide each tile into8 texels.
No demonstrated UV defect from code inspection. Prior claim that the head
artifact points to source sampling remains a hypothesis, not an established
bug. Next diagnostic should use a close-up camera and compare source/detail;
avoid another native geometry change without isolating the cause.

## Close-up traditional statue evidence

Live CeladonGym(4,14),zoom2/orbit0, inspected
`/tmp/statue-close-2.5d.png`. Zoom4 clipped statues and was replaced by zoom2.
Visible floor-colored holes pass through heads at large pixel scale, so this
is not merely distant minification. Built-in celadon_statue_placements uses
outline_mask=true; append_grouped_tree_scaled_inner uses darker_palette_mask
then boundary connectivity. This is the concrete built-in mask path to inspect
next. It does not prove the earlier live-profile trials had the same cause.
No build/tests. Closed task-owned live renderer after evidence capture.

## Rejected statue row-fill trial

Tried row-wise silhouette filling for recognized tower/Celadon statue
placements. Built and inspected `/tmp/statue-mask-CeladonGym-compare.png`
(4,14) and `/tmp/statue-mask-PewterGym-compare.png` (4,10). No demonstrated
improvement; removed the trial. This weakens the palette-mask-only diagnosis.
Need direct generated-pixel/face inspection rather than further mask guesses.
No tests. Restored executable from retained source after reverting.

## Route37 movement sample

Current renderer/profiles, UUDD from(5,10),128frames,9grid origins,
zero additional terrain builds, median16.66ms,p95 17.15ms,total5.21s.
Inspected `/tmp/route37-motion-moving-01.png` and `-moving-03.png`:
player back/front facing changes correctly for north/south input, stays
upright, trees/apricorns remain source-art cutouts, ledge end slopes remain
visible during scroll. North movement bumps at the ledge; this is not proof
of traversal through the central opening. No cargo/tests.
Statue face-code inspection did not isolate a defect; no new mask edit kept.

## Interior and port movement coverage

Reused the existing executable and 237-profile document; no build or tests.
Ran RRLL for 128 frames at each location:

| Location | Start | Grid origins | Additional terrain builds | Median / p95 |
| --- | --- | --- | --- | --- |
| GoldenrodDeptStore1F | (8,5) | 5 | 0 | 16.66 / 17.67 ms |
| OlivinePort | (10,14) | 8 | 0 | 16.68 / 17.14 ms |

Inspected `/tmp/motion-coverage-GoldenrodDeptStore1F-moving-03.png`:
raised cushions and counters remain visible during scrolling; actors stay
upright in this camera view. Seat bases retain conspicuous vertical stripes.
Inspected `/tmp/motion-coverage-OlivinePort-moving-01.png`: piling tops and
closed sides remain visible; pale rims and repeated side patterns are still
conspicuous, and the pier surface remains flat. Both logs include bump sounds,
so these samples do not establish unobstructed traversal of the full paths.
Logs, movement CSVs, final screenshots, and four movement screenshots per map
use `/tmp/motion-coverage-{Map}` prefixes. Total run times were 5.17 and 5.21s.
These extend Route37 movement coverage into an interior and a waterfront;
fixed-camera movement does not validate camera-orbit facing behavior.

## Kanto shared-prop orbit coverage

Reused current executable without build/tests. CeladonDeptStore1F(8,5),
zoom0/orbit4 (180 degrees), `/tmp/celadon-seat-orbit-2.5d.png`:
seat tops and closed backs remain visible. Player presents back art and stays
upright; actors remain upright. Counter back exposes a broad flat gray face;
wall fixture backs retain front artwork. These are unresolved reverse-view
art limitations, not a claim of complete interior quality.
VermilionPort(8,11),zoom0/orbit2 (90 degrees),
`/tmp/vermilion-piling-orbit-2.5d.png`: closed piling depth survives side view,
but broad blank blue faces and bright rims are more obvious than at default
angle. Prioritize source-art side treatment over adding more piling height.
Actor cards remain upright in the sampled view. Captures refreshed in0.90s
and0.89s; both task-owned live renderers closed after review.

## Shared port piling silhouette refinement

Changed all28 shared port piling profiles from8px vertical boxes to6px rise
with3px bevel and16px footprint. The entire16x16 source image maps once over
the cap/slopes, removing the broad blank blue extruded side seen at90degrees.
Retained after inspecting VermilionPort(8,11),zoom0/orbit2,
`/tmp/port-bevel-2.5d.png`, and OlivinePort(10,14),defaultcamera,
`/tmp/port-bevel-olivine-2.5d.png`. Shape is less box-like from both views.
White source-art edge bands remain conspicuous, particularly in Olivine;
this change improves silhouette/side coverage but does not resolve that art
limitation. Profiles remain237; no Rust rebuild or tests. Vermilion live
capture refreshed in0.89s; live process closed after review.

## Mahogany town and shop coverage

Current237 profiles, existing executable, no build/tests. MahoganyTown(10,8),
`/tmp/coverage-mahogany-compare.png`,1.78s: building fronts/roofs raised,
source-art trees retained in the sampled edge, ledge opening has visible
uneven cut faces. Foreground roofs occlude portions of the town at this angle.
MahoganyMart1F automatic walkable location,
`/tmp/mahogany-auto/MahoganyMart1F-compare.png`,1.73s: shelves raised, but
north wall, clock/cabinet, radio and blue fixture remain flat. This interior
needs traditional_house wall/fixture coverage; raised shelves alone are not
sufficient. Manual(3,5) and(1,4) positions were rejected as nonwalkable.
Existing --maps selection recovered by choosing a compiled walkable tile.

## Traditional-house fixtures raised

Added three exact shared traditional_house profiles: metatile13 clock at0,0
(2x4tiles), cabinet at2,1(2x3), and metatile01 radio at0,2(2x2).
Preserved source fronts with shallow4/4/3px depth and2px cap strips.
Inspected MahoganyMart1F(4,4), `/tmp/mahogany-fixtures-compare.png` then
`/tmp/mahogany-radio-2.5d.png`: clock, cabinet and radio now stand upright
rather than lying on the north floor. North wall and blue fixture remain
flat; radio cap includes source trim and needs refinement with wall geometry.
Accepted240 profiles confirmed in log. Final paired render1.74s, no build
or tests. Shared matching is exact; other traditional-house maps have not yet
been visually checked for these additions.

## Traditional wall trial rejected

EcruteakItemfinderHouse automatic sample confirms shared clock/cabinet/radio
placements also match a second interior: initial artifact
`/tmp/traditional-wall/EcruteakItemfinderHouse-compare.png` includes a trial
raised plain wall. Extending the wall to radio metatile01 removed all usable
trim80 ground samples in MahoganyMart1F, causing MissingGroundSample tile80
and inactive2.5D. Removed both wall profiles and restored standalone radio;
240 profiles retained. No build/tests. Need ground sample capture independent
of replacement ownership before raising the complete wall. Do not retain a
partial raised wall that conceals this dependency. Trial process stopped.

## Ground sample identity fix and traditional north walls

Root cause confirmed in mesh.rs: live override masking replaced both metatile
and tile identities with u16::MAX, hiding atlas ground samples from compiled
props. Preserve tile identity only for source Flat/Water cells while still
masking metatile identity and retaining ownership. Non-ground identities
remain masked. One native build required and completed; no tests.

Restored plain traditional_house08 wall and multipart01 wall/radio profiles.
Accepted241 profiles, inspected paired captures in
`/tmp/ground-sampling-fixed/`: MahoganyMart1F north wall now continuous and
upright with shelves still present (previous MissingGroundSample80 failure
resolved); EcruteakItemfinderHouse matching wall sections and clock/radio
upright, alternate right-hand cabinets still flat. Route37 comparison keeps
source-art upright trees/apricorns and ledges; automatic spawn is close to a
tree, so this is geometry coverage rather than a traversal validation.
No inactive-renderer warning in completed three-map batch. Source art stays
in the immutable atlas; the fix changes metadata availability, not pixels.

## Alternate traditional cabinets and another house

Added two exact traditional_house29 profiles: low cabinet at0,1 and tall
four-drawer cabinet at2,0, preserving full source fronts with2px cap/4px depth.
EcruteakItemfinderHouse(5,4),
`/tmp/cabinet-coverage/EcruteakItemfinderHouse-compare.png`: both previously
flat right-hand cabinets now upright and meet the modeled wall frontage.
243 profiles accepted; paired capture1.66s, no build/tests.

MahoganyRedGyaradosSpeechHouse(5,4),
`/tmp/cabinet-coverage/MahoganyRedGyaradosSpeechHouse-compare.png`,1.78s:
uses house tileset, so separate coverage rather than proof of new cabinet
matching. Existing rear cabinets, TV/radio, table and seats raised; rear wall
upright. Left foreground plant clipped by framing; right plant preserved as
source-art upright cutout. Seats have conspicuous striped sides. Automatic
spawn stands near a stool; no movement or camera-orbit claim.

## Cherrygrove/Violet coverage and traditional cushions

Inspected `/tmp/small-house-coverage/GuideGentsHouse-compare.png`: existing
house wall/furniture depth retained; stools show striped sides, left plant
clipped. Correct map ID is GuideGentsHouse (not CherrygroveGuideGentsHouse).
VioletNicknameSpeechHouse confirms shared traditional walls/cabinets in another
town. Added shallow2px beveled blue cushion profiles for traditional_house
10(2,0),1b(0,2),19(2,0), exact tiles02/03/12/13(hex),16px footprint.
Initial document rejected because object top_pixels equaled full height;
corrected default top to8, while explicit beveled part uses16. Rejected-run
screenshots are not final validation. Accepted246 profiles and inspected
`/tmp/traditional-cushions-fixed/{VioletNicknameSpeechHouse,MahoganyMart1F}-compare.png`:
blue cushions now slightly raised with preserved white center art. Actors
partially obscure Violet seats; Mahogany gives a clear unobstructed sample.
No builds/tests. Existing wall fix remains visible in accepted captures.

## Route32 bridge and Route43 vegetation coverage

Accepted current246 profiles, existing executable; no build/tests.
`/tmp/route-breadth/Route32-compare.png` at(10,45),1.79s: bridge remains flat
beside raised banks; cliff faces are conspicuously stretched.
`/tmp/route-breadth/Route43-compare.png` at(10,27),1.94s: source-art trees
upright, dense grass obscures path/actor lower bodies, shoreline joins visible.

Route32 UUDD movement from(10,45), `/tmp/route32-bridge-motion` prefix:
128frames,9grid origins,0additionalterrainbuilds,median16.63ms,p95 17.17ms,
total5.17s. Inspected moving01 and03: player remains upright and changes
north/back to south/front presentation. North sample reaches the grass bank;
large cliff wall dominates the view and hides portions of the scene behind.
Bridge has no visible deck thickness. These are sampled movement/geometry
findings, not full-route traversal or completion.

## Route32 bridge implementation diagnosis

Inspected source metatiles: bridge06 is a4x4 repetition of tile07, also used
for ordinary paths; nearby35 is water14(hex),43 is edge49 above water14.
A global tile07 extrusion would incorrectly lift ordinary paths. Any authored
bridge placement must be map/metatile scoped. Current Part validation only
allows base_pixels0..64, and live append emits the replacement ground at0.
A positive deck rise changes visual footing without a matching actor surface
rule; a negative-base deck would require schema/ground emission work and
would still be submerged by adjacent water at0. Therefore a profile-only
height tweak cannot provide visible deck thickness while keeping the current
walking/water relationship correct. No bridge edit retained. Next bridge
implementation needs an explicit deck/water elevation relationship and actor
surface handling, then movement validation at bridge/shore transitions.

## Shared gatehouse north doors

Covered Route35GoldenrodGate and Route43MahoganyGate using automatic walkable
positions. Baselines `/tmp/gate-coverage/{Map}-compare.png` expose flat north
doors between upright wall sections. Added exact gate04 4x3tile door/trim
profile, ground01,2px cap/4px depth. Accepted247 profiles and inspected both
`/tmp/gate-doors/{Map}-compare.png`: doors now upright and align with adjacent
wall frontage. Counters remain low relief with striped fronts; no counter
change here. Doorway traversal/reverse view not validated by these static
captures. No build/tests; same executable used throughout.

## Gate north exit movement validation

Current247 profiles, UUU from(4,3) in Route35GoldenrodGate and
Route43MahoganyGate. Inspected `/tmp/gate-transition-moving-02.png` and
`/tmp/mahogany-gate-transition-moving-02.png`: outdoor scenes and ROUTE35 /
ROUTE43 banners confirm north warps occurred. Player remains upright with
north/back artwork after exit; outdoor trees retain source-art cutouts.
Each96-frame run traversed8grid origins and built terrain once for map change.
Goldenrod median16.67ms,p95 17.52ms; Mahogany median16.69ms,p95 17.20ms.
Both total4.63s. No build/tests. Does not validate reverse entry or arbitrary
orbit; this closes the previously missing north-exit check for both doors.

## National Park gate layouts and terminals

Covered Route35NationalParkGate and Route36NationalParkGate, distinct north/
south and east/west room layouts. Baselines in `/tmp/park-gate-coverage/`
show flat northeast terminals and stepped/flat north-wall variants; Route35
uses alternate door metatile2e so the earlier04 door profile does not apply.
Added exact shared gate2c terminal at(2,1),2x3tiles,ground01,2px cap/4px depth.
Accepted248 profiles; inspected both `/tmp/park-terminals/{Map}-compare.png`:
terminal screen/keyboard now upright, room floor and counters retained.
Walls, alternate doors and stools remain incomplete. No build/tests; same
executable, static coverage only (no terminal interaction or gate traversal).

## Park gate alternate door/wall profiles

Added exact shared gate29,2e,2d upper3-row profiles, ground01,2px cap/4px
depth. Accepted251 profiles. Inspected both park gates in
`/tmp/park-wall-variants/{Map}-compare.png`: Route35 alternate double doorway
and wall sign now upright; Route36 sign/wall section raised. Remaining low
sections at terminal metatile2c and counter junction13 are still apparent.
Do not claim a continuous complete wall yet. Same executable, no build/tests.

## Park terminal adjacent wall join

Added exact gate2c left2x3 wall profile, keeping it disjoint from terminal
cells. Accepted252 profiles. Inspected both park gate comparisons in
`/tmp/park-wall-join/`: former low segment beside terminal now aligns with
raised sign/door wall. Route35 frontage is continuous through that segment;
Route36 still has low counter junction13. Terminal stands slightly forward
as authored. No rebuild/tests; default-angle static captures only.

## Counter junction completed and National Park sample

Added disjoint gate13 left wall and upper-right wall-strip profiles. The
strip uses base14,height8,offsetz16 to meet the wall plane while preserving
compiled counter cells below. Accepted254 profiles; inspected
`/tmp/gate-counter-join/Route36NationalParkGate-compare.png`: wall top now
continuous over counter junction; source light stripe has a small alignment
variation. Counter and terminal remain visible.1.75s, no build/tests.

NationalPark(20,27), `/tmp/national-park-coverage/NationalPark-compare.png`,
1.80s: central fountain/pool border still flat, center fitting low relief;
grass forms conspicuous parallel strips and conceals actor lower bodies.
This sample contains fountain/grass rather than the outer tree boundary.
Next park improvement should address fountain rim or grass presentation.
Profile document now254/256 slots; consolidate exact duplicate definitions
or extend format deliberately before adding many more profiles.

## Exact profile consolidation

Removed24 map-scoped definitions whose entire payload (excluding name/map)
is identical to an already-present shared definition. No new shared matching
introduced. Kept every map-specific definition without an exact shared twin.
254->230 objects, freeing24 slots without schema changes or rebuild.
Accepted230 profiles; inspected CherrygroveMart and CinnabarPokecenter1F in
`/tmp/profile-consolidation/{Map}-compare.png`: refrigerator/shelves and
healing machine/PC/seats remain raised. This is representative visual coverage,
not exhaustive equality across all affected maps or ordering interactions.
No tests. Existing shared tower statues still need their separate art fix.

## Contest map and Route38 gate planters

Covered NationalParkBugContest automatic(20,27): same flat fountain rim and
parallel grass strips as normal park; no contest progression claim.
Route38EcruteakGate shows a different counter layout and flat four planters.
Added two exact gate39 bottom-row planter profiles (origins0,2 and2,2),
2x2tiles,ground01,top0,depth0,ground mask. Accepted232 profiles and inspected
`/tmp/gate-planters/Route38EcruteakGate-compare.png`: all four plants upright
with source-art foliage retained as thin cutouts. Northern wall heights still
step across planter/counter variants; no wall fix in this pass.
Baselines in `/tmp/contest-gate-coverage/`. No build/tests.

## Coastal routes expose a terrain failure

Automatic Route40 paired capture stalls after its2D image: terrain sync
MissingGroundSample{column:62,row:1,tile_index:5},2.5D inactive. Stopped owned
process after confirming failure; no Route40 rendered-quality claim.
Log `/tmp/coastal-coverage.log`,2D `/tmp/coastal-coverage/Route40-2d.png`.
Batch stops at the failed map, so rendered Route41 separately rather than
waiting on the known inactive renderer. Route41 paired capture completed1.78s,
accepted232 profiles; `/tmp/coastal-coverage/Route41-compare.png` inspected.
No rebuild/tests. Route40 requires missing-ground handling or valid source
sampling; capture mode should also terminate on inactive renderer instead of
waiting indefinitely. This is distinct from rejected profile syntax.

## Route40 failure narrowed by position

Existing232 profiles: Route40(10,4) renders successfully in1.90s;
`/tmp/route40-north-compare.png` inspected. Previous(10,18) fails at mesh
column62,row1 requesting ground05, so failure is position/source-window
dependent rather than every Route40 view. Ground05 caller not yet isolated;
connected-map halo is a hypothesis, not proven source attribution.
Inspected source identity masking and ground lookup: no new native fix kept
without establishing the actual failing call path. No build/tests.
Northern sample player overlaps building frontage in2D/2.5D; despite successful
capture it is not a valid actor-footing reference. Use it only to establish
position-dependent renderer failure and building/tree rendering.

## Route40 terrain failure recovered

Opt-in CRYSTAL_VOXEL_TRACE_GROUND backtrace identified
append_grouped_tree_scaled_inner -> authored_surface_cell. Added diagnostic
for ground/surface/relief sample lookup, disabled unless env var is set.
Grouped props now return without claiming/drawing when no required ground
sample exists, retaining source drawing rather than aborting whole terrain.
This generalizes the existing shoreline missing-sample fallback; it is a
resilience fix, not replacement artwork for the unresolved prop.
Two native builds (diagnostic then fix), no tests. Re-ran exact failing
Route40(10,18); `/tmp/route40-recovered-compare.png` now completes and was
inspected. Water/beach/actors render; coastal cliffs still have detached and
stretched faces. Broader fallback regression coverage remains to be done.

## Grouped-prop fallback rendered regression samples

Reused fixed executable and232 profiles, no build/tests. Inspected
`/tmp/fallback-route37-compare.png` at(5,10): trees/apricorns retain upright
source-art cutouts; ledge slopes and actors remain visible.
`/tmp/fallback-mahogany-compare.png` at(4,4): north wall,clock,cabinet,radio,
cushion and shelves retain their modeled forms. Both completed without
inactive-renderer warnings. These cover normal successful grouped/profile
paths after Route40 resilience fix; they do not prove every missing-ground
fallback drawing is visually ideal or every map unaffected.

## Whirl Islands darkness and Cianwood framing correction

Existing232 profiles, no build/tests. WhirlIslandNW automatic capture is dark
in both2D/2.5D except actor/entrance: no cave geometry-quality claim.
Cianwood automatic sample landed in water; inspected but insufficient town
coverage. Chose source-map beach tile(14,42) and inspected
`/tmp/cianwood-beach-compare.png`: town buildings raised, beach paths readable;
cliffs retain stretched faces. Baselines `/tmp/island-coverage/`.
Keep dark captures distinct from usable cave coverage and water samples
distinct from town street coverage when using automatic spawn selection.

## Cianwood photo studio and Lugia house

Inspected `/tmp/cianwood-interiors/CianwoodPhotoStudio-compare.png` and
`CianwoodLugiaSpeechHouse-compare.png`. Both use the shared house layout:
rear wall,cabinets,TV,radio and table raised; stools retain conspicuous
floor-colored/striped tops and sides. Foreground left plant is clipped;
right plant visible as upright source-art cutout. Actor placement near stools
limits seat visibility. No special photo-studio fixtures are present in the
source scene, so none invented. Existing232 profiles, no build/tests.
These repeat the house-stool defect across additional maps: next house pass
should fix the shared stool mapping rather than add more identical rooms.

## House stool fold trial rejected

Tried exact house01/02/0c/0d stool profiles with8px cap,12px depth,5px rise.
GuideGentsHouse renders in `/tmp/house-stool-fold/` restore pink seat tops but
lose readable bases. Removing ground mask in `/tmp/house-stool-solid/` does
not repair bases, disproving the mask-only explanation. Removed all four
trial profiles;232 retained. No native build/tests. Need source-pixel/UV
inspection for the complete stool before another fold adjustment. Do not
trade away recognizable stool legs just to remove the striped top artifact.

## Shared house stool interior mask fixed

Read original house.png stool pixels02/03/12/13: rows5..10 contain seat,
11..15 legs, confirming existing fold ranges. Defect was per-pixel floor
comparison deleting enclosed same-color seat texels. Changed native stool
mesher to flood only boundary-connected floor candidates across full16x16
drawing. Keeps interior seat colors and source legs, still removes exterior
floor. Built once, no tests. Inspected both
`/tmp/stool-boundary/{GuideGentsHouse,CianwoodPhotoStudio}-compare.png`:
pink seat surfaces now continuous, base/leg fronts retained. Side geometry
remains thin; not a claim of finished reverse-angle stools.232 profiles.

## Player-house stool mask and side coverage

PlayersHouse1F(4,4), `/tmp/player-house-stools/PlayersHouse1F-compare.png`,
1.68s: shared native stool mask also preserves tan player-house seat art.
Live zoom0/orbit2, `/tmp/player-house-side-2.5d.png`: seats retain surfaces,
but bases lack closed side geometry (current mesher emits front only).
Table depth/partition remain, thin rear wall and fixture sides apparent.
This validates a second tileset while identifying the next stool geometry
requirement. No rebuild/tests; closed task-owned live renderer after capture.

## Stool support depth

Native shared stool mesher now repeats the authored front-leg silhouette on
front/back2px-deep supports with side faces. Seat mask/height unchanged.
Built once; no tests. PlayersHouse1F(4,4) live zoom0 orbit2 then4 inspected:
shallow bases remain visible from side and rear instead of front plane only.
Final `/tmp/stool-depth-side-2.5d.png` is orbit4 (overwrites side capture).
Support silhouette remains blocky; no claim of polished all-angle furniture.
Closed live renderer. Pink house tileset needs post-depth render coverage.

## Pink stool rear verification

GuideGentsHouse(5,4),zoom0/orbit4, inspected
`/tmp/pink-stool-rear-2.5d.png`: all four pink seats retain uninterrupted art
and rear support fronts/leg gaps. This closes the pending house-tileset
post-depth sample. Existing232 profiles, no build/tests. Reverse view also
exposes unrelated cabinet backs as open shells and rear wall culling; these
remain broader interior quality issues. Both foreground plant cutouts visible
from this direction. Closed live renderer after capture.

## Saffron gate coverage

Inspected Route5SaffronGate,Route6SaffronGate and Route7SaffronGate paired
captures in `/tmp/kanto-gates/`, current232 profiles, existing executable.
North/south layout shared doors now upright at Routes5/6. East/west Route7
layout provides additional counter/wall junction coverage. Counters remain
low relief; these static samples do not validate gate traversal or reverse
camera views. No build/tests. Avoid counting the two identical north/south
layouts as distinct geometry implementations.

## Mirrored gate wall coverage

Added shared gate0e right wall/left upper strip and gate39 upper wall above
planters. Exact disjoint source rectangles preserve counters/plant cutouts.
Accepted235 profiles, inspected Route7SaffronGate and Route38EcruteakGate in
`/tmp/gate-mirrored-wall/`: former low north-wall spans now share top height.
Light stripe still shifts slightly at upper-only strips; small gaps behind
counter tops remain. This improves silhouette continuity, not all junction
art or reverse views. No build/tests; existing executable.

## Fuchsia and Route15 coverage

Current235 profiles, no rebuild/tests. Inspected paired views
`/tmp/fuchsia-route-coverage/FuchsiaCity-compare.png` at(20,18) and
`Route15-compare.png` at(20,9). Fuchsia buildings/trees raised; lower hedge
fragments remain shallow/broken. Route15 paths readable, sign upright,
flowers/grass form regular rows and boundary rocks boxy.
Ran Route15 RRLL128frames, inspected `/tmp/route15-motion-moving-03.png`:
player upright with side artwork; ground and ledge geometry retain during
scroll. Metrics in `/tmp/route15-motion.log` and movement CSV. This is local
movement coverage, not full-route traversal or a new geometry fix.

## Fuchsia hedge corner diagnosis

Source kanto60 left bottom2x2 is2d/2e/3d/3e; repeated bushes on the right use
40/41/50/51. Trial exact upright corner profile withground2c produced no
visible improvement in `/tmp/fuchsia-corner-compare.png` at(20,18); removed.
Do not treat this distinct corner art as the normal repeated hedge. Ground
sample availability or matching needs inspection before another profile
trial; no cause established from accepted-document status alone.235 profiles
restored, no build/tests.

## Hedge source/matcher inspection

Read Kanto2d/2e/3d/3e source pixels directly: full16x16 drawing includes
sparse foliage/trunk mixed with checker-pattern background; it is not a
solid repeat of the neighboring40/41/50/51 bush. Native tree index catalog
includes both drawings. Live resolver additionally requires a Flat/Water
sample matching ground2c before it considers placements; some2c cells are
raised ledge tops. Neither fact alone proves why the prior profile had no
visible effect. Avoid replacing this source drawing with an invented solid
bush. No code/profile edit or rebuild; source inspection narrows next work
to placement diagnostics and mask behavior, not tile-number guessing.

## Lavender and Route8 coverage

Current235 profiles, existing executable; inspected paired captures in
`/tmp/lavender-route-coverage/`. LavenderTown buildings/landmark raised,
roof-edge joins rough and some foreground roofs clipped by framing. Route8
sample shows broad pale boundary structure flat while rocks/cliffs have depth;
vegetation remains repetitive rows. Treat boundary art as authored structure,
not permission to derive new walls from collision. No build/tests, no movement
or orbit claim. These samples identify roof joins and Route8 boundary shaping
as further work; they do not establish full-map quality.

## Cross-region breadth pass

Existing executable, 236 external profiles accepted; no build or tests.
Paired captures in `/tmp/breadth-review/`, visually inspected:

- Route1 (10,18): source vegetation retained; raised ledges expose thin vertical
  seams at ends and crossing detail. Not a complete route traversal.
- CherrygroveCity (20,9): raised buildings, signs and source-art 2.5D trees
  retained. Shore edge partially clipped by framing.
- GoldenrodUnderground automatic (19,25): isolated room/stair sample, not the
  main corridor. Follow-up (4,16), `/tmp/underground-corridor-compare.png`,
  covers vendors and counters. Planter foliage and stools remain flat;
  counters have low relief. This layout needs distinct furniture treatment.
- RuinsOfAlphResearchCenter (4,3): upright cabinets and actors; plants remain
  flat, desks low relief.

Batch paired renders finished in 1.67–1.80 seconds per map. These are static
coverage samples; no camera orbit or movement claim for this pass.

LavRadioTower1F glass-machine profile accepted at 236 objects, paired capture
`/tmp/lavender-machine/LavRadioTower1F-compare.png`: one targeted machine at
right raised, adjacent glass panels still flat. Partial prop coverage only;
north seating and wall remain low. No rebuild required.

## Underground planter correction

238 external profiles accepted; existing executable, no build/tests.
Gate38 long planters now retain their horizontal footprint as low beds:
full source-art top, four-pixel height, two-pixel bevel. Rejected an upright
full-height trial because it turned elongated beds into tall panels.
Gate33 small corridor planter uses an upright zero-depth source-art card,
matching the previously supported gate39 plants. No tree profile changed.
Visually inspected `/tmp/underground-beds-compare.png` at(4,16): bed fronts
now visible and small plants raised; stools remain flat. Regression capture
`/tmp/planter-regression/Route38EcruteakGate-compare.png` preserves existing
plants and walls. Paired captures took1.76s and1.62s. Static views only.

## Facility plant coverage and lab review

240 external profiles accepted. Facility0d/0e potted plants now use upright
zero-depth source-art cards. Research center default framing clips the left
plant; changing player position does not widen this room's framing. Live
zoom0/orbit0 `/tmp/research-zoom-2.5d.png` confirms both upright plants, with
checker-like gaps in foliage/pot masks still visible. Mask quality remains
unfinished. Camera-only recapture took0.89s; owned session closed.
Paired `/tmp/facility-plants/` covers research center, OaksLab and ElmsLab.
Labs retain bookcases and actors; round stools remain flat, Oak's table has
little height, Elm's desk has visible supports. Labs do not show this plant
art, so these are wider regression samples, not evidence of plant reuse.
No build/tests. Paired runs1.60–1.78s. Tree profiles untouched.

## Facility plant stripe isolation

Correction to preceding diagnosis: disabling Ground mask on both facility
plant profiles produced the same visible striped artwork in accepted240-object
render `/tmp/plant-mask-none-compare.png` (1.73s). Restored Ground mask.
This does not support blaming flood-fill removal for these stripes.
Live mesher already uses whole-part boundary-connected masking. Grid UVs span
whole tile slots and pixel subdivisions use eighths; no obvious inset error
found. Next diagnosis should distinguish source sampling from overlapping
surfaces, with a controlled depth offset or source texel inspection before
changing shared native masking. No native edits/build/tests this pass.

## Plant depth isolation and source evidence

Moved the live plant card back two source pixels; accepted render
`/tmp/plant-offset-compare.png` retains the pattern while moving the plant.
Removed diagnostic offset. Together with the mask-none comparison this
weakens the coplanar-overlap and mask-removal hypotheses; neither is proven.
Read source facility tiles2c/2d/3c/3d/2e/2f/3e/3f directly: foliage contains
mixed dark/light texels and pot rows include repeating patterns. Do not call
all apparent stripes missing geometry without comparing exact texels.
Native material is opaque; surface shader also applies directional shadows,
so source pattern versus shading remains to distinguish. No native changes,
no build/tests. Current240 profiles restored.

## Shared lab stool

Added exact lab07 origin(2,2), tiles0e/0f/1e/1f, ground10 profile.
Final fold preserves first12 source rows on a12-pixel-deep seat with5-pixel
rise, remaining4 rows on the front, Ground mask. Initial8-row cap trial
made an awkward dark raised edge; revised before retaining.
Inspected both `/tmp/lab-stool-seat/OaksLab-compare.png` and
`/tmp/lab-stool-seat/ElmsLab-compare.png`: white seat front now raised beside
table, subtle at default framing. Rear/side supports not yet inspected;
this does not establish a finished stool from all angles.
241 profiles accepted; runs1.71/1.78s, no build/tests. Labs' existing desks,
bookcases and actors remain visible. No tree changes.

## Lab orbit review and shared workstation

Elm live zoom0 orbit2 `/tmp/lab-side-2.5d.png`, orbit4
`/tmp/lab-orbit-2.5d.png`: stool cap and rear base visible, side remains thin.
Player stays upright, side sprite at90 and back at180. Elm partly occluded by
his desk, so this is not a complete NPC-facing audit. Bookcase reverse faces
repeat front artwork; machine backs repeat controls. These remain quality
limitations. Camera recapture0.89s; owned live session closed.
Removed Elm-only restriction from exact lab08/0c computer-workstation profile;
renamed Shared lab computer workstation. Both rooms use this source drawing.
`/tmp/shared-lab-workstation/` pairs show Oak's previously flat computer now
upright and Elm's retained.241 profiles accepted,1.70/1.78s, no build/tests.

## West Kanto exterior sweep

241 profiles, existing executable. Inspected paired `/tmp/west-kanto-coverage/`:
PalletTown(10,9) automatic spawn visually overlaps lab roof; do not use as a
footing sample. Follow-up(8,12) `/tmp/pallet-road-compare.png` places player on
clear street and shows raised fences/signs, houses and shoreline. House roof
joins remain rough. ViridianCity(20,18) shows shared buildings and dense
source-art tree cards. Route2(10,27) lies among dense hedges with conspicuous
short end pieces; this is not a clean corridor sample.
Route2(4,27) UUDD `/tmp/route2-walk-moving-03.png`: local movement128frames,
2 grid origins,0 additional terrain builds, median16.68ms,p9517.20ms. Bumps
limit travel; not a route traversal. Paired city runs1.79–1.84s; movement5.28s.
No build/tests or tree edits. These captures extend coverage and identify
better Pallet framing, not new geometry fixes.

## Route2 hedge source and explicit-card trial

Kanto6c/6d/6e/6f contain the same40/41/50/51 two-by-two hedge drawing,
arranged in different quadrants beside ground2c. Added eight exact zero-depth
card placements temporarily.249-object document accepted, but paired Route2
and Viridian renders in `/tmp/kanto-hedge-cards/` show no visible improvement.
Removed all eight trial profiles;241 restored. Do not infer successful
placements from accepted-document count. Resolver's Flat/Water ground-sample
requirement remains a candidate explanation; source-only inspection cannot
prove this. Next work should instrument actual placements/skips, not change
height blindly. No native edits, builds or tests; original2.5D trees retained.

## Placement diagnostic resolves hedge uncertainty

Added opt-in CRYSTAL_VOXEL_TRACE_PROFILES substring filter in native live
resolver. One necessary Rust build succeeded; no tests. Temporary242-profile
file `/tmp/hedge-trace-profiles.json` adds one exact Kanto6c hedge placement.
Route2(10,27) log proves13 matches using ground-cell58. Thus missing ground
sample is NOT the explanation for this sample's unchanged appearance.
Inspected `/tmp/hedge-trace-compare.png`; same short hedge end appearance.
Oak workstation diagnostic proves1 match, ground-cell2640. Runs1.85/1.76s.
Tracked profile file remains241 objects; temporary diagnostic card not retained.
Toolkit now distinguishes accepted documents from matched placements.

## Trainer-house and Pewter interior coverage

Reviewed TrainerHouse1F, TrainerHouseB1F, PewterNidoranSpeechHouse and
PewterSnoozeSpeechHouse. Bundled map list contains no PewterMuseum1F; initial
batch stopped after TrainerHouse1F and remaining valid rooms ran separately.
Basement confirms shared facility plant profiles apply in another room;
railings raised, stair geometry rough. Pewter houses preserve seat supports
and cabinets; plant rendering remains coarse.

TrainerHouse1F large-table stools used house21/22/23/24 variants absent from
native furniture matcher. Added those exact variants to existing stool
resolution, retaining exact2x2 tile validation and existing seat/leg geometry.
One necessary native build succeeded, no tests. Paired captures inspected in
`/tmp/trainer-seats-fixed/`: trainer-house seats now show raised caps and legs;
PewterNidoranSpeechHouse existing stools retained.241 profiles unchanged.
Other evidence `/tmp/west-kanto-interiors/`, `/tmp/interior-coverage-next/`.

## Mount Moon area coverage and terrain priority

241 profiles, existing executable; no build/tests. Paired renders visually
inspected in `/tmp/moon-coverage/`: Route3(30,9), MountMoon(15,8),
MountMoonSquare(15,9),1.76–1.80s. Route3 cliff joins expose thin vertical seams.
MountMoon has prominent triangular corner faces and disconnected shelf/wall
edges; these are higher-impact terrain defects than additional furniture.
MountMoonSquare shop and signs raised, cliff seams visible behind them.
`/tmp/moon-interior/MountMoonGiftShop-compare.png` confirms shared traditional
walls, cabinets and cushions; counter surface remains low and striped floor
trim is visible. No whole-map or movement quality claim.
Next terrain investigation: cave diagonal_corner_local exact10/11/35 sources
versus surrounding shelf planes, using this MountMoon framing as baseline.
Avoid treating the existing corner primitive's intended shape as evidence
that its connections to adjacent geometry are correct.

## Cave corner bevel experiment rejected

Native diagonal cave prism pushes its foot half a source tile outside its
quadrant. Trial zero bevel built and rendered MountMoon and UnionCave1F in
`/tmp/cave-corner-contained/`. Both still show detached triangular corners;
MountMoon front corner triangles became more conspicuous. Reverted trial and
rebuilt executable to match restored source. No tests or profile changes.
This rules out simply removing the foot bevel as a sufficient fix. Next
investigation must compare corner edge coordinates against neighboring ledge
planes and shelf elevations; do not retain this isolated bevel change.

## Cave corner connection coordinates

Source inspection narrows next edit: MountMoon block10 sits immediately left
of0d and above0a; mirrored11 sits right of0d and above08. All share16-pixel
rock height. Straight0d face lies at block south boundary(row4);0a at east
boundary(column4),08 at west boundary(column0). Current corner10 diagonal
runs from its quadrant northeast to southwest; its southeast vertex belongs
only to the closing faces. Corner11 mirrors this. Thus matching height alone
does not ensure the visible diagonal meets the adjacent straight faces.
Source matrices confirmed from bundled source tiles. A next experiment should
change diagonal orientation/connectivity while preserving exact boundary
endpoints, winding and UV mapping; not just shrink the foot. No native or
profile changes/build/tests in this inspection. Existing render evidence in
`/tmp/moon-coverage/` remains baseline.

## Cave orientation trial rejected

Reflected corner prism north/south with corrected winding/normals; built and
inspected MountMoon and UnionCave1F pairs in `/tmp/corner-orientation/`.
The resulting detached faces were more conspicuous, with larger dark gaps.
Restored exact pre-trial function and rebuilt successfully. No tests.
This contradicts treating diagonal orientation alone as the fix. Corner
geometry and adjacent folded strips must be reconstructed as one connected
boundary, with a local coordinate diagram before further native experiments.
No trial geometry retained;241 profiles unchanged. Existing baseline remains
visually incomplete. Avoid repeating bevel-only or reflection-only changes.

## Corner surrounding-floor trial rejected

Trial raised tile16 within corner blocks10/11 to adjacent rock height.
MountMoon and UnionCave1F `/tmp/corner-cap/` show new unwanted steps while
corner fins remain. Restored source and executable; no tests. Same tile16
identity in these blocks must not be assumed to represent raised shelf cap:
it also represents surrounding walking floor. No change retained. Further
terrain edits need collision-aware local topology evidence before another
native build; bevel, orientation alone, and broad cap raising have all failed.

## Cerulean and Route4 coverage

241 profiles, existing executable, no build/tests. Paired captures inspected
in `/tmp/cerulean-east-coverage/`: Route4 ledge boundaries uneven; house
retains raised stool supports. Cerulean automatic center visually overlaps
Pokecenter roof. Use corrected(28,18) `/tmp/cerulean-street-compare.png` for
clear street/building sample; roof texture repetition remains conspicuous.
Route4(20,9) RRLL failed movement assertion: actor only turned on a blocked
ledge tile. Do not report as movement coverage. Corrected(20,12) RRLL succeeded:
128frames,7 grid origins,0 terrain builds,median16.64ms,p9517.38ms,5.17s total.
Inspected `/tmp/route4-path-motion-moving-03.png`; actor upright on grass path,
ledge ends remain abrupt. Coordinate correction is essential for future runs.

## Repeatable coverage preset

Added RENDER_PRESETS.md with reviewed positions, known bad starts, a four-scene
command sequence and explicit status/image checks. Linked from toolkit.
TrainerHouse1F center(5,7) rejected as unwalkable during preset verification;
used actual reviewed(5,8) from successful trainer-seats-fixed log instead.
No native/profile changes, builds or tests. This avoids repeating misleading
center guesses when moving between established regression locations.

## Northern Cerulean routes

241 profiles, no build/tests. Inspected `/tmp/north-cerulean/` pairs:
Route24(10,9) automatic location overlaps a post; Route25(30,9) confirms
short hedge ends and tall repeated rows; BillsHouse(5,4) retains furniture.
Paired runs1.67–1.83s. Corrected Route24 bridge center(8,9), UUDD:
128frames,9 grid origins,0 additional terrain builds,median16.68ms,
p9517.21ms,total5.18s. `/tmp/route24-bridge-motion-moving-03.png` shows player
upright on deck; posts vary in apparent shape/height, deck remains flat,
shore faces stretched. Added bridge position to presets. Local movement only,
not full route traversal or proof of finished bridge geometry.

## Oak display table depth

Added exact lab06/07/0a/0b six-by-three drawing profile:16-row top,
24-pixel depth,6-pixel height. Diagnostic confirmed one match in both labs.
Paired `/tmp/lab-table-depth/` shows deeper Oak tabletop with raised rim;
Elm trial clipped lower Pokeball sprite pixels. Scoped retained profile to
OaksLab and renamed Oak lab display table. Verified Elm restored in
`/tmp/lab-table-elm-restored/ElmsLab-compare.png`.242 profiles, no build/tests.
Occupied display-table improvement requires coordinated object footing;
do not broaden this profile until that is implemented and rendered.
Oak side/rear supports remain unreviewed; this is a local depth improvement.

## Live profile support height fixes occupied lab display

Added optional validated footing_pixels to Rust profile schema and apply it
to the matched drawing's support cells. One native build succeeded; no tests.
Shared lab display table now sets6 pixels matching its tabletop and applies
to both labs. Inspected `/tmp/table-footing/` pairs: all three Elm Pokeballs
fully visible atop table, Oak retains deeper top/rim.242 profiles accepted.
This supersedes previous Oak-only restriction. Capability is explicit and
uniform over source footprint; do not use for mixed-height/offset drawings
without finer support handling. Live reload uses existing terrain/footing
replacement. Side/rear and occupied-table movement remain unreviewed.

## Occupied table live reload verified

Single Elm session at(5,6),zoom0/orbit4: changed table mesh/support6→10,
then10→6 source pixels. Accepted revisions2/3 each applied and recaptured in
0.91s. Images `/tmp/table-support-height10.png` and final
`/tmp/table-live-support-2.5d.png` visibly show both table and Pokeballs move
together, without clipping. Final6 restored; owned session closed. Rear view
shows repeated source front rim on table back and cabinet backs; those remain
art limitations. No build/tests. This is direct live-reload evidence beyond
the preceding fresh-process captures;242 profiles retained.

## Vermilion interior and Route11 coverage; flower stands

Reviewed PokemonFanClub, VermilionFishingSpeechHouse and Route11 pairs in
`/tmp/vermilion-coverage-next/`. Route11 boundary structure remains flat;
FanClub confirms large-table stools but small flower stands were flat.
Added four exact house30/31 quadrant profiles:2a/2b/5e/5f drawing, ground01,
8-row top/12-pixel depth. Diagnostic proves one match each in FanClub and
zero in fishing house.246 profiles, no build/tests.
`/tmp/fanclub-flower-stands/` pairs and live zoom0
`/tmp/fanclub-wide-2.5d.png` show all four raised stands and their fronts.
Side panels are plain dark and remain crude. Wide view also reveals northern
NPCs heavily occluded by the table; investigate their footing/occlusion
separately. Owned live session closed. No tree changes.

## FanClub occlusion side view

Live PokemonFanClub(7,4),zoom0/orbit2 captured in
`/tmp/fanclub-occlusion-2.5d.png`. Northern actors appear intersected/occluded
by the wall plane rather than simply standing below a tall table. Do not
apply table footing to those NPCs to conceal this. Investigate ordinary-house
north-wall course placement against house03/05/29 source row extent and NPC
positions. Side view also confirms flower stand depth and stool legs. No
code/profile changes, builds or tests; owned live session closed.

## Ordinary-house wall floor-row correction

Native north-wall course previously folded all four block rows, including
floor01 below plain two-row wall panels. For columns with floor01 in both
lower rows, now fold only upper2 rows at row2 and leave lower rows unclaimed
for normal ground rendering. Fixture columns retain four-row treatment.
One Rust build succeeded, no tests. Paired `/tmp/house-wall-floor/` inspected:
FanClub northern NPCs now visible in front of wall; Vermilion fishing house
floor remains clear and fixtures retained. Shorter plain panels reveal stepped
heights beside fixture-backed wall strips; continuity still needs refinement.
246 profiles unchanged. This is a source-row ownership correction, not an
NPC footing adjustment. Side view of final correction remains to review.

## House wall correction breadth and side validation

Inspected DayCare, MrFujisHouse and TrainerHouse1F pairs in
`/tmp/house-wall-breadth/`. Plain panels sit behind usable floor; furniture
and stools remain. MrFuji also confirms raised flower stands reused in another
layout. Stepped wall height above radio/PC remains visible in all three.
FanClub zoom0/orbit2 `/tmp/fanclub-wall-fixed-2.5d.png` directly compares with
prior `/tmp/fanclub-occlusion-2.5d.png`: wall plane now clears both northern
NPCs. Table still occludes lower bodies naturally in this view. No footing
change needed. Owned live session closed;246 profiles, no build/tests.
Next wall refinement should separate fixture-backed panels from fixtures,
not move the plain wall forward into the actors again.

## Fixture-backed wall panel alignment

Native ordinary-house wall tile00 above radios/PCs now uses same two-row
rear panel plane as adjacent plain columns. Fixture artwork retains existing
forward fold. Built once; inspected TrainerHouse1F, DayCare and Vermilion
fishing house in `/tmp/fixture-panel-alignment/`. Tall blue protrusions above
PC/radio removed; continuous upper panel edge restored across these sections.
Narrow gaps behind fixture silhouettes remain, and bookcases stand taller.
No tests;246 profiles unchanged. This preserves earlier NPC clearance fix.

## Flower shop curtain alignment and SoulHouse coverage

GoldenrodFlowerShop baseline exposed curtain tiles above stands still folded
at four-row fixture plane. Native panel classification now includes exact
curtain24/4a/34/2c tiles in upper two rows of matched house courses. One build;
paired `/tmp/curtain-panel-alignment/` confirms curtains align with plain wall
and SoulHouse wall remains consistent.246 profiles unchanged, no tests.
SoulHouse coverage shows memorial rows currently rendered as low slabs and
candle fixtures shallow; those need separate prop treatment. Flower shop stool
also remains flatter than standard supported house variants. Baselines in
`/tmp/special-house-walls/`. No tree edits.

## Soul House memorial faces

Added six map-scoped exact house28/2a/2b quadrant profiles covering24 memorials.
Placement diagnostic confirms5+5+3+3+4+4 matches. Final drawing folds wholly
upright with2-pixel depth, preserving inscription face rather than bending its
upper rows over a cap. `/tmp/soul-memorial-faces/SoulHouse-compare.png` reviewed:
memorials now upright, aisles and actors visible. Background-colored borders
remain conspicuous, and mixed candle variants still have shallow candle bases;
silhouette cleanup and side/rear review remain.252 profiles accepted, no build
or tests. Initial2-row top/4-depth trial in `/tmp/soul-memorials/` superseded.

## Regional coverage: Ecruteak through Cinnabar

Paused local memorial refinement to review six distinct maps with the existing
binary. Paired captures in `/tmp/regional-render-coverage/`, all visually read:

| Map | Position | Finding |
| --- | --- | --- |
| EcruteakCity | 20,18 | Buildings and source-art trees present; dense distant foliage and roof repetition remain conspicuous. |
| Route37 | 10,9 | Tall tree cards meet much shorter hedge silhouettes; inconsistent vegetation height is a shared outdoor priority. |
| BurnedTower1F | 10,9 | Rubble mixes raised and flat shapes; upright structural remnants remain. |
| EcruteakItemfinderHouse | 5,4 | Table, wall fixtures and cushions present; book face projects upright from tabletop. |
| CinnabarIsland | 10,9 | Automatic position overlaps roof artwork; superseded for player clearance review. |
| CinnabarPokecenter1F | 5,4 | Counter and seats raised; nurse partly hidden behind counter. |

Cinnabar recaptured at6,12: `/tmp/cinnabar-street-coverage-compare.png`.
Player clears building; coastal columns are angular and raised terrain has
visible seams. Capture timings1.72–1.89s per pair,252 profiles loaded in each.
No builds, tests, geometry changes or tree removals. These are stationary
visual reviews, not movement or full-map coverage. Shared vegetation and
terrain consistency take priority over additional SoulHouse decoration.

## Johto small-tree profile scope and height experiment

Reviewed Route37, NewBarkTown and Route29 using temporary profile documents
`/tmp/shared-johto-tree-profiles.json` and
`/tmp/shared-johto-tall-profiles.json`. First removed Route29 map restrictions
from its26 exact small-tree profiles; second changed their part heights18→28.
Neither experiment is retained in the repository.

Pairs in `/tmp/shared-johto-trees/` and `/tmp/shared-johto-tall/` show that
height28 stretches Route29's masked shapes into visibly striped spires.
Route37's conspicuous short foreground segments remain largely unchanged,
so a blanket height increase does not address the reported discontinuity.
NewBark's central street/buildings remain visible in both experiments, but
this framing is weak evidence for its boundary vegetation.

`/tmp/route37-tree-trace.log` confirms multiple shared profiles match, with
ground-cell42; map scope and missing ground alone do not explain the result.
Next diagnosis should trace ownership of the specific visible short segments
through live-profile and native grouped-tree passes before another height edit.
Existing252 profiles preserved; no native changes, builds or tests. All owned
capture processes exited successfully. The prior regional review yielded
actionable evidence; this experiment rejects a broad but ineffective fix.

## Tree outline ownership check

Native `complete_tree_placements` already sets `outline_mask:true` on every
placement. Adding the same assignment at its call site was redundant; removed
after one unnecessary build and five-map visual review. Do not repeat this
mask hypothesis. `/tmp/tree-outline-coverage/` contains inspected Route37,
NewBarkTown, Route29, EcruteakCity and Route2 pairs with unchanged vegetation.
Current binary includes the redundant assignment but is behaviorally equivalent
to restored source; no second build warranted. No profile edits or tests.

Source inspection identifies Johto60 left and62 right quadrants as additional
small-tree drawings absent from the26 Route29 profiles. Both contain the same
`[[30,31],[62,63]]` source art in two vertically repeated quadrants. Native
grouped-tree ownership covers them. Extending existing profile map scope does
not affect these variants. Diagnose those exact source placements next.

## Flower shop stool source profile

Retained `Shared house flower shop stool`: house2e origin2,0, exact
`[[2,3],[18,19]]`, ground1, top11/depth12/height5 with boundary-connected
ground mask. Count253. Compared `/tmp/flower-stool-review/GoldenrodFlowerShop-compare.png`
against `/tmp/curtain-panel-alignment/GoldenrodFlowerShop-compare.png`:
the rectangular ground-colored base is removed; shallow seat and legs remain.
MrFujisHouse pair in the same review directory checks standard stools and
flower stands remain present. This corrects the earlier shorthand 'flat stool':
baseline already had relief, but carried a conspicuous rectangular backing.
No build or tests; reviewed temporary document's exact object copied into
the repository profile document. Side/rear view still needs review.

## Stool side review and Olivine/Blackthorn houses

Flower shop live capture at4,3, zoom0/orbit2:
`/tmp/flower-stool-side-2.5d.png`. Recapture completed in0.89s with253
profiles. Side exposes solid dark extrusion beneath the seat; front improvement
is partial, not a finished volumetric stool. Need separate leg supports rather
than the profile's whole-width side wall.4,4 is not walkable; use4,3.
Owned live session closed after review.

Inspected all four pairs in `/tmp/johto-house-coverage/`:
OlivineGoodRodHouse, OlivineTimsHouse, BlackthornDragonSpeechHouse and
BlackthornEmysHouse. All share the same room furniture arrangement, so these
add map coverage but only one distinct geometry layout. Cabinets, table,
stools and plants remain present. Northern seated NPCs overlap their stools;
no claim of correct sitting posture. Plants retain striped cutouts and fixtures
have narrow backing gaps. No builds, tests or geometry edits this pass.

## Flower shop stool uses shared native supports

Added house2e upper-right exact stool variant to `house::furniture_local`.
Removed the temporary `Shared house flower shop stool` profile (252 remain),
so the existing native stool renderer owns it: sampled seat rows5..10 and
separate support strips at depth3..5 and12..14. One native build succeeded;
no tests. This supersedes the profile's solid side-wall treatment.

Live review at4,3 zoom0: `/tmp/flower-native-stool-orbit2.png` shows the open
space beneath the seat instead of the prior continuous dark side. Rear orbit4
`/tmp/flower-native-stool-2.5d.png` shows seat and rear support intact. Camera
recaptures completed in about0.9s without rebuild; owned session closed.
Shared geometry is reused without changes to other stool variants. Flower
stand side slabs and striped plant masks remain separate unresolved issues.

## Eastern Johto terrain and Route42 movement

Reviewed all pairs in `/tmp/east-johto-terrain/` with existing binary:

| Map | Position | Finding |
| --- | --- | --- |
| Route42 | 30,9 | Automatic spawn coincides with raised cliff artwork. Source cave opening is lost in the continuous modeled face; needs source-face preservation. |
| Route44 | 30,9 | Grass patch between lakes visible; repeated tall vegetation cards obscure boundaries. |
| IcePath1F | 20,18 | Floor clear, but walls have severe stretched/repeated texture and uneven corner joins. |
| DragonsDenB1F | 20,22 | Fenced platform readable; distant shoreline blocks vary between isolated raised chunks and flat edges. |

Route42 lower path30,11 RRLL:128 frames,9 grid origins,0 additional terrain
builds, median16.69ms/p9517.41ms,5.18s total. Final
`/tmp/route42-path-motion.png` inspected: player on clear path below cliff;
walk ends near water boundary and includes bump sounds. This establishes
translation, not uninterrupted traversal of the entire route. Cave opening
loss is a higher-priority shared terrain/art issue than local decoration.
252 profiles unchanged, no builds or tests, all capture processes exited0.

## Connected cliff front-course ownership

`authored_bank_face_cell` now searches matching courses from front to back.
Previously the first northern matching band could replace a southern doorway
in the same connected run with plain rock. Existing upper-tier doorway
exclusion remains: this changes which authored course owns the exposed face.
One native build succeeded, no tests;252 profiles unchanged.

Inspected `/tmp/cliff-front-course/Route42-compare.png`: the cave opening now
appears on the south face at the source-art location, unlike the prior eastern
Johto baseline. Route45 comparison also inspected: paths and mountain sides
remain readable; narrow seams and rough height transitions remain. Route42's
automatic30,9 spawn still overlaps cliff artwork; use30,11 for path movement.
Both capture processes finished successfully. Entrance traversal itself has
not been validated by this stationary art correction.

## Route42 entrance traversal

From28,10, UUUU movement capture reaches MountMortarB1F. CSV
`/tmp/route42-entrance-walk.movement.csv` confirms map transition, not just
facing change. Reviewed initial approach `/tmp/route42-entrance-walk-moving-00.png`
and final `/tmp/route42-entrance-walk.png`: player approaches restored entrance
and appears inside the cave.128 frames,8 grid origins, median16.68ms,
p9517.25ms,5.18s total. Two additional runtime terrain meshes generated during
the run; these are scene geometry work, not compiler builds. Bumps occur after
entry. No cargo or tests. Cave interior still has detached triangular faces
and uneven ledge geometry, so successful transition does not validate its art.

## Route42 west and east entrance traversal

Existing executable, UU from west10,6 and east46,8. Both CSVs confirm
Route42→MountMortar1FOutside at frame13. Each run:64 frames,6 grid origins,
one runtime terrain mesh generation, median16.67ms, p9517.23ms west/17.20ms
east,4.09s total. Final images `/tmp/route42-west-entry.png` and
`/tmp/route42-east-entry.png` visually inspected. Together with the central
entrance run, all three Route42 mouths now have transition coverage.

Interior arrivals expose the same malformed cave corners and raised surfaces
around approaches; these remain visual failures despite functional warps.
No code/profile changes, compiler builds or tests; both owned processes exit0.

## Cave corner adjacency diagnosis

Inspected `append_diagonal_cave_corner`, `cave_shape`, and the bundled
MountMortar1FOutside block grid (`/tmp/mortar-corner-layout.txt`). Example:
block10 at block2,4 has block0e to its east and block0e immediately south.
Its lower-right drawing is `[[0a,26],[17,0a]]`; the other twelve source cells
are tile16. Current geometry raises a southeast triangular prism16px over
the lower-right quadrant while those twelve plain cells remain at datum0.
Adjacent0e folds its south and east bands at16px. Block09 (all tile16)
also stays flat, so elevating only the corner's plain cells would introduce
another isolated step, consistent with the previously rejected cap experiment.

This is an elevation/topology consistency problem across a connected region,
not evidence that a global tile16 height or reflected triangle alone is correct.
Tile16 is shared ground and must not be globally promoted. Before the next
native edit, establish which side of each authored edge owns the raised region
and carry that assignment through straight edges, corners and floor support.
No geometry change or build this pass; constructor and source rules inspected
directly rather than repeating earlier mask/height/orientation experiments.

## Fast Ship coverage and cabin wall panels

Reviewed four baseline pairs in `/tmp/fastship-coverage/`: FastShip1F,
FastShipB1F, FastShipCabins_NNW_NNE_NE and
FastShipCabins_SE_SSE_CaptainsCabin. Cabin portholes lost surrounding wall
panels, unlike the corridor treatment. Tables/chairs already have relief;
basement partitions remain flat and separate cabin rooms remain visible at
distance in the shared map layout.

Retained three shared exact lighthouse09/0a/38 profiles, upper two source
rows, ground13, top0/depth0/masknone. Complete wall strips now stand upright
with their door/porthole artwork attached.255 profiles. Inspected both cabin
pairs and FastShip1F in `/tmp/ship-wall-review/`: cabin panels restored,
corridor unchanged. Mixed cabinet block2f still has a small panel gap; not
covered by these profiles. No native build or tests. Temporary reviewed
objects copied exactly into repository profiles; all processes exited0.

## Ship cabinet-side panel and third cabin group

Retained lighthouse2f upper-left2x2 `[[2,3],[18,18]]` as a complete upright
panel, ground13/top0/depth0/masknone. `/tmp/ship-cabinet-wall/` pairs inspected:
NNW_NNE_NE cabinet-side porthole now joins its wall; SW_SSW_NW room also has
a continuous panel behind the dining tables. This extends review to all three
cabin-group maps, not every individual cabin within those maps. No builds or
tests.256 profiles accepted: document is now at its configured object limit;
consolidate redundant entries before adding more. Furniture remains unchanged.

## Exact map-list consolidation

Added optional object `maps` allowlist to Rust parser/resolver, mutually
exclusive with `map`. Consolidated exact duplicates while preserving all
prior map scopes and effective matching order:256→241 objects. Entries
involving NewBarkTown remain separate where merging changes priority.
Compared expanded ordered objects against the pre-consolidation document
for all20 named maps and unlisted maps; geometry fields and order identical.

One native parser build succeeded. Four reviewed pairs in
`/tmp/profile-map-list-review/`: NewBarkTown, Route29, PlayersNeighborsHouse,
CherrygroveEvolutionSpeechHouse. Current profiles accepted by the renderer;
existing tree/plant silhouette issues remain. No test suite run. Toolkit
documents selectors and priority preservation.15 slots available for further
art improvements without changing the256-object limit.

## Ship basement partition faces

Retained five exact lighthouse profiles:26 full width,1e right2 columns,
17 left3,1c right3,1f left2. Each folds source rows1..3 upright24px,
ground13, top0/depth0/masknone.246 profiles. Initial center-only trial created
a stepped panel; final includes matching end faces. Inspected FastShipB1F and
FastShip1F pairs in `/tmp/ship-partition-complete-review/`: basement wall now
stands with continuous horizontal trim; main corridor retains prior treatment.
Cabin SW_SSW_NW was also reviewed during center trial and remains intact.
Vertical partition strips still lie flat and wall ends need side-angle review;
this is a front-panel correction, not complete partition geometry. No builds
or tests; five reviewed objects copied to repository profile document.

## Ship partition side and lighthouse reuse review

FastShipB1F16,8 zoom0/orbit2 live capture
`/tmp/ship-partition-side-2.5d.png` reviewed,0.89s recapture. Side reveals
existing raised perimeter walls, so the earlier blanket statement that side
strips are flat was too broad. Doorway/partition joins remain incomplete and
upright front cards become edge-on; a complete continuous enclosure is not
yet demonstrated. Owned live session closed.

Reviewed OlivineLighthouse1F,3F,6F pairs in
`/tmp/lighthouse-floor-coverage/`. Shared ship profiles do not visibly replace
the lighthouse's distinct stone/green wall art. Stair-shaped perimeter still
creates disconnected tall panels; upper-floor NPCs can be obscured by those
walls.6F bed/table/chair retained, with characters visible in central room.
These are three distinct floor layouts, not full tower traversal.246 profiles,
no builds, tests or geometry edits; batch finished successfully.

## Lighthouse perimeter source ownership

Inspected native `profile.rs` lighthouse wall branch and shipped source blocks:
3c/3d/3e each fold all four rows onto their own south edge, always facing the
same direction regardless of neighboring floor.3e repeats two stone courses;
3c/3d have distinct lower green/window artwork. No plain floor rows were found
inside these exact wall blocks, so the ordinary-house floor-row correction
does not apply here.

OlivineLighthouse1F block grid shows these blocks forming west/east and stepped
inner contours, not just a northern wall. The missing side joins follow from
folding all of them southward. Next implementation must resolve perimeter
orientation against neighboring authored floor/void and preserve3c/3d lower
art; a blanket wall-height reduction or floor-row skip would not fix this.
No native change or build made from this diagnosis.

## Lighthouse straight side-wall orientation

Added native `append_lighthouse_side_walls`: complete unclaimed lighthouse3e
blocks beside a full27 walkway edge face east/west toward that edge. Blocks
with north/south walkway neighbors stay with existing corner treatment.
Preserves full source drawing,32px height and existing ground replacement;
no collision-derived walls. Mixed3c/3d blocks remain unchanged.
One renderer build succeeded; no tests;246 profiles unchanged.

Inspected three pairs in `/tmp/lighthouse-side-review/`:1F and3F now show
straight side walls along the walkway instead of detached south-facing strips.
6F central room/furniture remain intact. Stepped corners, mixed green/window
courses and wall backing still need work. This is a partial perimeter fix,
not a complete enclosure or NPC-visibility solution.

## Remaining lighthouse floors and walking review

Inspected2F/4F/5F pairs in `/tmp/lighthouse-remaining-floors/`, extending
stationary layout review to all six floors.5F shows corrected straight walls
flanking the central walkway;4F has an unresolved stone panel at a staircase
junction. Mixed horizontal wall sections still hide northern actors.

5F10,9 UUDD `/tmp/lighthouse5-walk.png` reviewed: player translated within
central room and encountered a trainer, ending in dialogue. This is not an
uninterrupted loop or stair-transition check. Characters and side walls remain
visible during the encounter. No native builds, profile edits or tests; all
capture processes finished successfully.246 profiles unchanged.

## Cinnabar coast coverage and Seafoam arena rock bases

Reviewed Route19/20/21 and SeafoamGym pairs in
`/tmp/cinnabar-seafoam-coverage/`.19/21 automatic capture uses surfing state;
20 captures the island shoreline. Boundary boulders are shallow, and Route21
has an isolated building-art patch on water that needs source-layout review.
Seafoam arena rocks appeared nearly flat.

Placement tracing `/tmp/seafoam-rock-trace.log` confirms eight existing cave
rock profiles match, ground-cell0. Their8px faces start at0 while the arena
shelf is6px. Retained eight SeafoamGym-only copies before generic profiles,
with part base_pixels6.254 profiles. Inspected
`/tmp/seafoam-rock-base-review/SeafoamGym-compare.png`: rock faces now clear
arena floor, central aisle/characters retained. Blocky silhouettes and cave
corner triangles remain. No native build or tests; side/rear review pending.

## Route21 source patch and Seafoam side review

Route21 source block grid itself contains paired05 blocks above paired78
blocks at columns2–3/rows7–8 and columns7–8/rows11–12. The brick-like patch
also appears in the 2D comparison; it is not new 2.5D geometry. Preserve until
its bundled source/catalog identity is resolved rather than deleting it in
an optional renderer. This revises the earlier renderer-artifact suspicion.

SeafoamGym5,4 zoom0/orbit2 `/tmp/seafoam-rock-side-2.5d.png` inspected:
rock faces stand above the arena on both sides; slab-like side strips and
stretched corner faces remain conspicuous. Base correction retained, not a
claim of finished rock silhouettes. Recapture0.90s,254 profiles accepted,
owned live session closed. No compiler builds, tests or edits to geometry.

## Surf movement and encounter validation gap

Route19 at10,18 RRLL:128 frames,9 grid origins,0 additional terrain meshes,
median16.66ms/p9517.33ms. `/tmp/route19-surf-motion.png` inspected: directional
surf sprite present over water; encounter music also occurs, so do not claim
an uninterrupted travel loop. No compiler build or tests.

Route21 at10,28 RRLL produced `/tmp/route21-surf-motion.png` but log contains
repeated `active battle snapshot is missing active player party index` errors.
Process nevertheless exited0 after screenshot validation. This is a failed
runtime validation, not a passing surf review. The location renderer's battle
fixture/error propagation needs correction before using encounter-prone walks
as clean evidence. Preserve the failing log `/tmp/route21-surf-motion.log`.
No geometry/profile edits;254 profiles retained.

## Location-render party fixture

Added explicit render_test_party config, enabled by render_at_location only,
restricted to fresh runtime-tile starts. Seeds level30 Totodile with fixed
DVs and nonempty fixture owner. Initial run rejected empty trainer name during
initialization; corrected to configured smoke name or RENDER. Two builds,
no tests. Normal new-game/save-load paths do not enable this option.

Route21 rerun10,28 RRLL completed with no logged runtime errors;128 frames,
6 grid origins,0 extra terrain meshes, median16.66ms/p9517.80ms. Longer
RRRRRRRRLLLLLLLL also error-free:512 frames,11 origins, median16.65ms,
p9517.78ms. Both final PNGs visually reviewed (`/tmp/route21-party-motion.png`,
`/tmp/route21-party-encounter.png`). Neither rerun entered battle, so this
proves fixture initialization and surfing, not the full encounter path.
Encounter-trigger validation and fatal runtime-error propagation remain open.

## West Johto cross-map review

Reused the existing executable with the current 254 profiles; no build or
 tests. Batch exited successfully. Visually inspected all seven paired renders
in `/tmp/regional-coverage-west-johto/`:

- AzaleaTown: street and storefronts readable; tree border stretches into tall
  striped curtains, with exposed brown caps. Needs shared tree ownership review.
- IlexForest: source-art 2.5D tree cards remain present; distorted silhouettes
  and detached red patches on the path require investigation.
- Route34: repeated hedge/fence strips extend across the foreground; source
  shoreline remains recognizable. Shared boundary assembly needs correction.
- GoldenrodCity: streets, storefronts and signs readable at this position;
  fences and actor contact still need other-angle coverage.
- AzaleaGym and GoldenrodGym: continuous source partitions become disconnected
  upright panels. Treat as a shared partition continuity defect.
- IlexForestAzaleaGate: counter, floor and actors readable; planter cards and
  side-facing actor silhouettes remain weak.

These are individual viewpoints, not whole-map passes. Preserve the 2.5D trees;
fix their geometry and shared rules rather than removing vegetation. Prioritize
cross-map defects and recheck multiple maps after each retained correction.

## Garden gym partition diagnosis and rejected fold

`elite_four_gym_card_placements` groups each display into an independent 2x2
flat card. Exact block inspection confirms $08/$0a each contain two displays
stacked north/south, so the gap is produced by collapsing each footprint to
an upright plane, not missing source tiles.

Profile-only trial `/tmp/gym-partition-trial.json` gave those GoldenrodGym
halves two 16px-deep, 8px-high parts with 8px source tops. Existing executable,
no build/tests; batch GoldenrodGym/AzaleaGym exited0. Inspected Goldenrod pair:
the runs connect but become low black-striped boxes with stretched red tops,
and mismatch the surrounding upright displays. Rejected; main profiles remain
254 and unchanged. Evidence `/tmp/gym-partition-trial/GoldenrodGym-compare.png`.
A retained correction must preserve source display proportions while resolving
north/south connections and corner continuity together; this simple fold does
not meet that requirement.

## Modern tree source-color preservation

Retained two exact shared johto_modern $05 profiles, origins0/2, source rows
[30,31]/[19,21]/[19,21]/[62,63], ground6, zero top/depth and ground mask.
The native outline_mask removes light boundary-connected canopy pixels; the
source-art profile preserves those bright crown pixels. This tileset's middle
rows differ from johto $05, so existing New Bark/Route29 profiles cannot simply
be reused. Profiles now256, at current parser capacity.

Visually inspected `/tmp/modern-tree-coverage/` pairs for AzaleaTown, Route34,
GoldenrodCity, NewBarkTown and Route29. Azalea crowns are more complete than the
native-mask baseline `/tmp/regional-coverage-west-johto/AzaleaTown-compare.png`.
Other viewpoints retain existing streets/buildings; this is not a whole-map
regression pass. No runtime errors in batch log; pair durations1.76–1.88s.
Azalea zoom0/orbit1 `/tmp/modern-tree-angle-2.5d.png` inspected: cards retain
zero thickness; dense border still reads as repeated curtains, exposed brown
caps persist. Retained only as source-color preservation, not finished tree
composition. Live recapture0.90s; owned session stopped. No build or tests.

## Park, connecting routes and theater coverage

Existing executable/current256 profiles, no build or tests. Batch exited0;
visually inspected all pairs in `/tmp/park-theater-coverage/`:
Route35, NationalPark, Route36, DanceTheater, Route35NationalParkGate.
Route35 confirms north/south fences collapse into separated posts while the
horizontal run stays connected. NationalPark preserves pool/fountain and tree
cards but grass reads as repeated strips. Route36 has incomplete small-tree
silhouettes and fragmented border transitions. Gate counter/terminal remain
readable; stools are shallow source art.

Initial theater viewpoint shows partial performers at the top. Follow-up
DanceTheater6,3 `/tmp/theater-stage-close-compare.png` inspected: central
performers and player have full bodies above the stage. Existing native stage
support already uses its8px surface height. Do not increase stage/actor height
based on the distant screenshot. View-dependent actor clipping or extraction
needs investigation; edge performers remain partial. Back-wall art is also
still laid horizontally. This is coverage and diagnosis, not a resolved actor
clipping claim. Both rendering sessions exited0; no geometry edits this pass.

## Correct actor support outside the classic viewport

The theater issue was support lookup, not sprite extraction: tile_at_visual_point
rejected feet outside the LCD before accounting for the wider published terrain
grid. Those visible actors consequently fell back to ground0 despite the8px stage.
Changed footing.rs to bound lookup against terrain grid coordinates including its
margin. Outside-grid fallback remains. One native build, no tests.

Visually reviewed `/tmp/actor-halo-footing-review/` DanceTheater, Route35 and
NewBarkTown pairs. Same distant theater view now shows all five performers above
the stage, including the side performers previously buried. Outdoor viewpoints
retain existing actor placement; fence/tree defects remain. Batch exited0.
This supersedes the prior sprite-clipping hypothesis. Stage geometry was unchanged.

## Actor support movement and raised-terrain follow-up

Theater6,6 is not walkable; initialization rejected it. Used verified6,3 with
DDUU instead. `/tmp/theater-footing-motion.png` and moving00/02 inspected:
performers stay above stage as viewpoint changes.128frames,2grid origins,
0additional terrain builds, median16.65ms/p9517.76ms. Repeated bump sounds and
limited travel mean this does not prove a full stage crossing. Session exited0.

Reused binary for `/tmp/raised-terrain-actor-review/`: visually inspected
TrainerHouseB1F, MountMortar1FInside, Route45 pairs. TrainerHouse dividers and
stairs readable, rear counter still occludes an NPC. Mortar has severe corner
fins and mismatched ledges; Route45 presents the raised path and player but
only at one viewpoint. Those geometry defects remain independent of halo
support. Batch exited0. No build, tests or geometry changes in this follow-up.

## North/south Johto fence rails

Added native append_johto_vertical_fences: exact johto/johto_modern tile4a in
40/42/44/46/48/4a uses live5a/59 rail courses oriented alongZ, with ground6
replacement. Three darker source levels retain rail paint. Missing any source
sample leaves the original path intact. Profiles remain256.

First build/render connected runs but exposed missing back faces. Added reverse
faces; second build. No tests. Initial batch `/tmp/vertical-fence-review/`
Route35/GoldenrodCity/EcruteakCity inspected; final Route35 live default and
zoom0/orbit2 `/tmp/vertical-fence-final-2.5d.png` inspected. Both sides of the
north/south runs now visible and continuous instead of isolated standees.
Corner offsets remain: horizontal and vertical source folds meet at different
anchors, requiring a junction correction. Keep this as straight-run improvement,
not complete fence geometry. Live session stopped; captures around0.9s.

## West-coast fence coverage and corner anchors

Reused executable, no builds/tests. Visually reviewed Route38, Route39 and
OlivineCity pairs in `/tmp/west-coast-fence-coverage/`; batch exited0.
Route38 horizontal runs preserved; Route39 north/south ranch rail now continuous
but its join is visibly offset. Olivine street/storefront readable, distant
fences thin at this angle. Tree/mountain defects remain.

Exact johto corner source blocks:
40 has top5a row0; row1 starts4a then three59; vertical4a continues rows2/3.
42 mirrors this at right.48 has vertical4a left rows0/1 then full5a/59 rows2/3;
4a mirrors at right. Horizontal north course folds to row2, south to row4,
while new vertical geometry uses each original cell's Z interval. Consequently
south corners stop two cells before their folded horizontal rail, and north
corners can overrun the rail. The current johto_fence module handles49 only;
later Johto profile handles41/42, leaving40/48/4a horizontal portions without
that shared fold. A corner fix must unify horizontal ownership and trim/extend
vertical endpoints together, including the missing lower-course corner cell.
Do not shift all straight fences to conceal the gap. No geometry edits this pass.

## Authored fence corner connection

Expanded Johto horizontal folds to exact40/41/42 north courses and48/49/4a
south courses. Native vertical mesher replaces north-corner row1 stripe with
the missing lower horizontal face at its folded plane. South-corner final
vertical cell repeats native rail segments through the two-cell folded gap;
no texture stretching. Existing assertion for48 updated to the new behavior;
no tests run. One build for geometry change.

`/tmp/fence-corners-review/` Route35/Route39/GoldenrodCity pairs inspected;
batch exited0. Route39 ranch elbow now connects, Route35 north corner face
restored. Route35 also joins straight41 to straight44 without an explicit
corner block; its overrun is outside the authored-corner correction and remains.
Horizontal half-cell overhangs and reverse-angle review remain. Profiles256,
trees unchanged. Retained as a verified improvement, not all fence joins solved.

## Ranch reverse view and interiors

Route39 zoom0/orbit3 `/tmp/ranch-corner-angle-2.5d.png` visually inspected:
repaired ranch elbow remains connected from reverse oblique view. Small rail
end overhang remains; no claim of completed junction geometry. Live recapture
0.90s, owned session stopped.

`/tmp/ranch-interior-coverage/` Route39Barn and Route39Farmhouse pairs inspected;
batch exited0. Barn wall/fence and central actors readable. Farmhouse retains
raised table/stools and appliances, but foreground plant has striped card
silhouette; center-selected player overlaps an NPC beside the table. Separate
placement from rendering diagnosis before changing actor height. No builds,
tests or geometry edits; current256 profiles.

## House plant isolation trials

Compared Route39Farmhouse and PlayersNeighborsHouse using existing executable.
Trial1 `/tmp/farmhouse-plant-trial.json` extended the existing split plant profiles
only to farmhouse. Both pairs visually inspected: pot changes, canopy remains
striped. Trial2 `/tmp/house-plant-whole.json` removes parts/depth, one whole
source-art card. Farmhouse pair inspected: canopy remains striped. Both batches
exited0. Main profiles unchanged256; no builds/tests.

This rules out the split-part offsets as the sole cause of the canopy defect.
Do not broaden that profile as a claimed canopy fix. Next investigation should
compare assembled source pixels/mask with the rendered card; both native and
profile paths share the source image/UV path. Preserve trials outside repo.
