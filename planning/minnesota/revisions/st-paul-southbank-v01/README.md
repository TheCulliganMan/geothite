# St. Paul south-bank neighborhood and landing

Five complete native facades move into smaller destination groups. Riverside
Supplies and the angler booth share a turning neighborhood lane with Westside
Community. A continuous bank street connects both real bridges. The boat
cooperative, inn and survey hall group around the eastern landing, with the
original trainer and pickup in a bounded river court. The40x34 city extent,
all40 rooms and previous market/civic refinements remain.

The terrain authority is `after.layout.json`; `recomposition.contract.json`
binds every original indexed anchor. `town.dossier.json` gives all40 entry and
return pairs,23 installed objects,11 readings and35 exact expansion facade
footprints. `sources.json` binds the selected native pack and inherited sources.
This is a local map refinement, not full regional/campaign acceptance.

## Deliberate geometry

Coordinates below are32px metatiles. Facade rows move intact, including their
real door quadrant and paired passive return.

| Destination | Before origin | After origin | Native extent | Entry warp |
| --- | --- | --- | --- | --- |
| Riverside Supplies |2,27 |3,28 |2x2 |69 |
| Westside Community |7,28 |8,28 |5x3 |71 |
| Angler House |2,31 |5,31 |2x1 |67 |
| Boat Cooperative |21,30 |22,30 |2x1 |75 |
| South Landing Survey |30,28 |29,28 |4x3 |79 |

The western bridge meets the two-tile street at x13, with the community and
arena on either side. The supplies frontage turns around the western buildings;
the angler threshold branches from that lane. The shared south-bank street at
y31 connects this neighborhood to the eastern boat, inn and survey frontage.
The eastern bridge street is at x28. Ground cover is native FLOOR turf and
ordinary flowers, not encounter grass.

The east visitor moves from core(59,61) to(57,61), clear of the moved survey
wall. River reading2 moves from(59,57) to the actual native79 board at(55,57),
on the left corner of the eastern landing. The old board is removed before the
survey roof is restored; no invisible reading remains inside a house.

The pine boundary at x36/y28..30 leaves the original pickup at(73,63) reachable.
Trainer8 stays at(71,61), with an ordinary challenge approach at(71,62) and
passing space around the court. The opponent, party, reward and defeat flag
are unchanged. All water, layered bridge, perimeter and existing hop cells are
exact; both river seams and all three connection offsets are retained.

## Current native publication

Final local pack:
`content-packs/minnesota-st-paul-southbank-v03.crystalpack`.
Its parent is `route_grass_mankato_town_revision`, including the newer six-cell
Mankato repair. The earlier v02 south-bank pack and its review images are kept.
`world.json` owns selection; resolve it again before publishing a successor.

The Rust publication used the existing `recompose_authored_city` entry through
the retained executable `.local/duluth-garden-cached-recompose`, with parent
pack, layout, contract and fresh output pack as its four arguments. Source entry:
`crates/crystal-assets/src/bin/recompose_authored_city.rs`. No new Rust
implementation or fresh Cargo build is claimed.

Evidence under `renders/st-paul-southbank-v221/`:

- `whole-root-preservation-v229.json`:104 changed terrain cells and12 indexed
  anchors, including the visitor and river board. Event mirrors are reconstructed
  independently; all731 other maps and unrelated native data remain exact.
- `production-tour-v229/receipt.json`:1387 production calls cover all40 actual
  building entries/returns, both bridge walks, the civic hop and denied reverse,
  its19-step ramp return, both service boards, both checked river records,
  Save/Continue and Start. Stderr is empty.
- `battle-court-v230/receipt.json`:114 more production calls continue the actual
  tour save, approach the trainer, choose native moves, win, collect the reward,
  and verify post-defeat dialogue both before and after Continue. Money changes
  from10000 to10420; repeating the interaction starts no second battle.
- `battle-court-v230/native-pp-verification.log`: the native save reader confirms
  POUCHID's selected PSYBEAM spends PP20 to19, with bag and badges unchanged.
- `audio-v229.log`: all377 PCM hashes, frame counts and loop ranges pass.
- `rebase-preservation-v231.json`: only Mankato differs between the reviewed v02
  and final v03 worlds. St. Paul, every city room/neighbor, art/audio and shared
  battle catalogs are exact. Earlier city views remain bound to their v02 pack.
- `native-connected-context-v228/all-five-views.json`: all five original views
  match. The edge view uses real two-hop neighboring scenery and the native
  immediate-neighbor camera clamp. Its initial single-map padding failure and
  every original GPU image remain retained and unmodified.
- `native-connected-final-v229/`: a fresh final-pack court capture also has zero
  pixel differences. All five views, this fresh capture and full town were
  visually inspected. The native camera fixture uses CINDORE explicitly.

The initial tour position, POUCHID45/CINDORE12, money and Repel are declared QA
setup. The battle leg continues its actual reached save and rewrites no state.
This establishes local access and one original challenge, not earned regional
progression, complete interior transactions or final user visual approval.

## Connected state render and source loop

The final pack's complete connected image and all103 full-resolution exterior
originals are retained under `renders/current-connected-after-st-paul-v229/`.
`full-state-native-current.png` uses actual native rectangles at one scale. The
audit finds one direct-seam group, no overlapping footprints, no inconsistent
offset cycles and no absent planned hubs. Its ordinary water-lane inventory is
static geometry; raw exposed-edge records and the warp graph do not establish
reachable Surf escapes, earned access or statewide controller acceptance.

The source tool loads the parent once, checks real anchors/collision and saves
a complete-town original per edit. Measured rendering/checking is about0.3s
after pack loading; it publishes no packs.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/render_city_streets.py \
  --parent content-packs/minnesota-current-mankato-native-exterior-v03.crystalpack \
  --directory planning/minnesota/revisions/st-paul-southbank-v01 \
  --output planning/minnesota/renders/st-paul-southbank-next --watch
```

To refresh the actual connected atlas with a fresh output directory:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/audit_outdoor_connections.py \
  --pack content-packs/minnesota-st-paul-southbank-v03.crystalpack \
  --output planning/minnesota/renders/connected-southbank-next
```

Keep all original/cropped/rejected images, packs, reached saves and receipts.
Generated output and content packs remain ignored and must never be committed.
