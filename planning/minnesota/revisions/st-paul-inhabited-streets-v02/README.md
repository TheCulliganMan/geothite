# St. Paul market streets and care court

This is a bounded exterior refinement, not acceptance of the whole city. Nine
complete native facades move closer to useful shared streets. The fair's food,
berry and garden stalls share a brick market lane; the information booth faces
a shaded approach and garden display. West Seventh's bakery and Pokemon Center
share a small care court. A low wooden rail defines its southern edge, with two
ordinary openings to the river promenade. The mart and craft shop sit closer
to the street garden. Civic and eastern streets still need composition work.

`after.layout.json` and `recomposition.contract.json` are the terrain and indexed
anchor authority. `town.dossier.json` records all40 actual doors, their returns,
all23 installed objects and11 readings. Its35 expansion facades have exact native
rows and current footprints; older expansion footprints are retained as history.
Room programs, activities, trainers, rewards and story flags are inherited.
There are no new activities, transactions or story gates in this revision.

## Exact changes and connections

Coordinates in this table are32px metatiles; dimensions remain40x34.

| Complete facade | Before origin | After origin | Exterior entry index |
| --- | --- | --- | --- |
| Information booth |2,3 |5,4 |23 |
| Food hall stall |2,6 |2,5 |11 |
| Berry stall |6,6 |5,6 |27 |
| Garden stall |10,6 |8,6 |13 |
| Quiz pavilion |13,6 |12,6 |21 |
| Bakery |2,21 |2,19 |45 |
| Pokemon Center |6,21 |6,19 |3 |
| Mart |12,15 |10,15 |5 |
| Lowertown Crafts |11,20 |11,19 |53 |

The native77 rail occupies(2..3,22) and(5..8,22). Its bottom walking row is
blocking WALL, not a painted fence over floor. Openings at x4 and x9 retain
ordinary two-way passage. Native0a is urban FLOOR turf; no encounter grass or
incidental Cut tree is introduced. Existing ledges, stairs, bridge layers,
water tiles and the entire map perimeter remain exact.

The west road retains offset15 to `MnR04C00R00`, including core lanes(0,48..49)
to(55,18..19). East and south water connections retain their exact offsets14
and36 to `MnMississippiLowerWoodsC00R01` and
`MnMississippiConfluenceC03R00`. The contract explicitly inventories all eight
reciprocal water lanes. This edit does not add a State Fair admission link or
change the existing northern reservation.

## Native publication and evidence

Final local pack: `content-packs/minnesota-st-paul-inhabited-streets-v03.crystalpack`.
Its parent is the newer `duluth_inner_buoy_clearance_revision`, preserving the
accepted shoreline rocks, route fence openings, inner-buoy removal, harbor,
North Shore cafes and R10 cave. `sources.json` binds the exact pack and all
current authoring paths. Source/native drift in the older canonical St. Paul
layout was178 cells; `before.layout.json` retains the installed repairs before
applying this120-cell revision. Canonical creation files remain historical.

Evidence lives in the ignored `renders/st-paul-streets-v200/` folder:

- `whole-root-preservation-v205.json`: independently reconstructs all21 anchor
  relocations and literal event mirrors; all731 other complete maps, native art,
  audio, catalogs, scripts, trainers, spawns and publication records remain exact.
- `production-tour-v206/receipt.json`:1221 real stdio/controller calls enter and
  return from all40 buildings, walk both bridges, read both relocated service
  boards and the river record, then Save/Continue and open Start. Stderr is empty.
- `court-rail-v208/receipt.json`: continues that actual reached save; the rail
  denies entry from both sides and both openings work in both directions.
- `native-gpu-v205/`: four fresh640x576 native captures have zero mismatching
  pixels. All four, the complete city and both district originals were inspected.
- `audio-verification-v206.log`: all377 PCM hashes, frame counts and loops pass.
- `current-roster-v206.json`: both unchanged exterior opponents and both local
  fixture partners belong to the active original Minnesota roster.

The initial local party, money and placement are QA fixtures. Door round trips
do not prove room transactions, gym completion, earned access or the campaign.
Earlier source passes, unsuccessful harness runs and all original images remain
retained. The earlier harness corrections distinguished passive arrivals from
exit carpets, supplied the outward carpet step, handled native Repel expiry and
read the cliff-side river board from its actual floor approach.

## Fast source loop

This executed command loads the parent once, validates the exact anchor and
Foot graph, and saves an immutable complete-town render per edit. A measured
render/check pass took0.204seconds after pack loading. It does not publish packs.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/render_city_streets.py \
  --parent content-packs/minnesota-current-shore-rocks-open-routes-v03.crystalpack \
  --directory planning/minnesota/revisions/st-paul-inhabited-streets-v02 \
  --output planning/minnesota/renders/st-paul-streets-next --watch
```

The publication used the retained Rust `recompose_authored_city` executable
`.local/duluth-garden-cached-recompose` with these four arguments: parent pack,
`after.layout.json`, `recomposition.contract.json`, fresh output pack. The source
entry is `crates/crystal-assets/src/bin/recompose_authored_city.rs`. No new Rust
implementation was required and no fresh Cargo build is claimed. Choose a new
output filename for another publication. Before selecting any successor, resolve
`world.json` again and preserve independently accepted updates.
