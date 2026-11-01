# Grand Rapids depot and quay

This whole-town revision brings the clinic and Survey Depot onto the main
street and the Mart onto the waterfront circuit. The discarded northern service
clearings become snow pines. The worker stands off the through street; the
optional angler stands in a shore nook instead of occupying the pier.

The candidate is `content-packs/minnesota-reviewed-maps-v05.crystalpack`, SHA256
`79e431d36fd47c02904b8c9a92d7871a4d9cead4c3e557ed6a73c98d27837e30`,
content identity `0513ba901b48e3f85c42a8ecad8b38711c31ca743484b422cfd054256618cf16`.
It is byte-identical to the standalone
`content-packs/minnesota-grand-rapids-depot-quay-v01.crystalpack`.
It is now selected for local review in `world.json`; no browser
publication, existing-save migration or complete campaign acceptance is implied.

- [Complete installed town](../../renders/grand-rapids-depot-quay-installed-v1362/after-whole-original.png)
- [Whole before/after at native scale](../../renders/grand-rapids-depot-quay-installed-v1362/before-after-original.png)
- [Explicit source rows and anchor preimages](source-v01.json)
- [Combined source recipe](../../revisions/reviewed-maps-v01/recipe-v05.json)

## Composition and traversal

The footprint remains 24×24 metatiles so both real road connections and their
checkpoint planes remain intact. It is a recomposition of the inhabited town,
not a resized render. Original native facade rectangles move as complete
assemblies: clinic 2×2 from (4,4) to (4,9), depot 3×2 from (13,5) to (10,10),
Mart 2×2 from (19,6) to (14,13). Roofs, door quadrants and original palettes are
preserved. There is no new art, ice gate, Cut tree or generic obstacle puzzle.

The main street has two clear walking lanes, y24/y25 between x4 and x43.
Both original pier lanes, x25/x26 from y34 through y38, are clear. The native
casting and Surf launch remains (26,38), facing water at (26,39). Waterfront
rails, bank positions and every metatile in rows17–23 remain exact. All original
water and encounter-grass positions remain exact, including 84 grass quadrants.
The northern pines close unused service clearings without removing habitat.

| Useful native FLOOR walk with NPC bodies | Before | After |
| --- | ---: | ---: |
| West arrival → clinic | 15 | 5 |
| West arrival → depot | 31 | 15 |
| Clinic → depot | 24 | 14 |
| Clinic → Mart | 34 | 28 |
| Pier → Mart | 35 | 11 |
| Depot → Mart | 14 | 14 |
| East arrival → Mart | 12 | 20 |

The east-arrival shop approach is eight steps longer; the shop now serves the
actual waterfront loop. These static counts describe the tradeoff, not a claim
that every walk is shorter or that the whole region has earned traversal proof.

## Doors, people and road permission

Coordinates below are 16px walking tiles. Original room identities, interior
arrivals, return indices, room programs and all service content remain exact.

| Destination | Active door | Passive exterior return | Town warp indices |
| --- | --- | --- | --- |
| MnGrandRapidsPokecenter1F | (9,21) | (9,22) | 1 / 2 |
| MnGrandRapidsSurveyDepot | (21,23) | (21,24) | 5 / 6 |
| MnGrandRapidsMart | (29,29) | (29,30) | 3 / 4 |

Both real exit carpets in each room must return to that room's paired exterior
record. The passive interior third warp is not another playable exit carpet.
The clear native Fly/recovery exterior (6,24) remains unchanged.

`MN_GR_WORKER` moves (17,25)→(17,23). `MN_EXTRA_GR_BANK_FISHER` moves
(25,36)→(23,31); approach from (23,32), facing north. KNUTE's refusal,
regional team, battle PP, native prize and completion flag remain unchanged.
The river watcher remains (31,32). The four checkpoint wardens remain (2,24),
(2,25), (45,24), (45,25), with their original scripts and free inbound care.

The depot sign moves (22,12)→(26,22). West/east signs remain (14,22)/(38,22).
All three occupy native solid post quadrants with public reading approaches.
The west link remains `MnR09C01R00`, offset0, with town (0,24)/(0,25) paired
to route (63,24)/(63,25). The east link remains `MnR10C00R00`, offset7, with
town (47,24)/(47,25) paired to route (0,10)/(0,11).

The central building is the **Survey Depot**, not a Gym. Preserve MS09's
independent foreman victory, recovered instrument and separately acknowledged
handover. A win or pickup alone cannot replace handover. The western departure
still needs the seventh badge and completed headwaters report. Eastern departure
still needs the seventh badge and acknowledged depot clearance. Unqualified
inbound travelers can reach free care. Do not grant flags to make a short walk
look connected or turn the optional angler into a story requirement.

## Fast source loop and Rust publication

Parent: `content-packs/minnesota-reviewed-maps-v04.crystalpack`, SHA256
`47d45a3bb934e4ad863d3e92950e29dd55bdceb62d5824ff23952d55e11e7b48`.
Source SHA256: `ae52b80c42b7b667d4234c4b8f83ea5f4590097de89ac422972b24f3443643f1`.
Complete native town preimage, canonical JSON SHA256:
`d87fbae712e025fde98031703fc9a930190f188408da9bac28effd5f68b371f9`.
The frozen source's proposal status records its authoring epoch; current
acceptance and local selection are separate from that immutable source.

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_grand_rapids_depot_quay.py \
  --output planning/minnesota/renders/grand-rapids-depot-quay-FRESH
```

The executed warm source loop takes 0.146 seconds after loading. Add
`--installed content-packs/minnesota-reviewed-maps-v05.crystalpack` to compare
and render the installed typed map (3.11 seconds including child loading).
Each invocation requires a fresh ignored output and retains its source snapshot,
whole originals and before/after. Preserve rejected passes too.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin revise_grand_rapids_depot_quay
target/verdigrim-scene/debug/revise_grand_rapids_depot_quay \
  content-packs/minnesota-reviewed-maps-v04.crystalpack \
  47d45a3bb934e4ad863d3e92950e29dd55bdceb62d5824ff23952d55e11e7b48 \
  planning/minnesota/towns/grand-rapids-depot-quay-v01/source-v01.json \
  content-packs/minnesota-grand-rapids-depot-quay-FRESH.crystalpack
```

Serialize Cargo/publication with `.local/minnesota-cargo.slot.lock`. The Rust
leaf checks the full map preimage, complete facade copies, all 123 changed
metatiles, exact water/grass positions, original checkpoint/shore planes,
readable signs, doors, actor approaches and two clear street/pier lanes.
Its inverse native-map replacement must recover the complete parent. Pack
writes are fresh and ignored, with strict readback. Python only inspects and
renders source; it never writes game packs.

The independent whole-root preservation receipt is
`renders/grand-rapids-depot-quay-preservation-v1361/receipt.json`. All 742 other
maps and every program, room, encounter, roster entry, art and audio resource
remain exact. All 602 audio PCM/hash/frame/loop and desktop checks pass in
`.local/grand-rapids-depot-quay-publication-1793510381900378000/`.
Early Winona and Driftless dry-bank fishing is preserved exactly; fresh
zero-badge native casting/encounter/Continue checks also pass on the selected
parent in `renders/early-fishing-preservation-1793511282757575000/`.

## Native acceptance evidence

Eight town/story/service/gate cases pass in
`renders/grand-rapids-depot-quay-controller-1793510862564310000/`. They play all
three doors, six exit carpets, three signs, worker dialogue, native care/PC,
priced shop, both street and pier lanes, optional angler refusal, Save/Continue,
MS09's independent requirements and both real guarded seams. A further four
riverfront cases pass in
`renders/grand-rapids-depot-quay-river-runtime-1793511104901732000/`: original
native animation, angler battle PP/prize/Continue, missing-move/badge denial,
qualified Surf, dismount and registered-rod fishing. Initial local party,
position, resources and story conditions are explicit fixtures; these twelve
cases do not claim an uninterrupted earned campaign or regional balance.

The complete source view and five actual native saved-state cameras were
inspected. `renders/grand-rapids-depot-quay-native-1793511106012260000/` retains
the clinic/depot approach, depot door, quay shop, clear pier and angler nook.
Original roofs are complete; pines frame the street without body obstructions;
shop and bank share the useful waterfront approach. The actual saved night,
snow, clock, facing and party were not overridden. Native water/deck appearance
is the preserved original port assembly. Broader visual review remains open.

The combined source audit is
`renders/grand-rapids-reviewed-sources-v1363/receipt.json`: 28 final native
layouts, six route groups, 21 sources, eight Rochester and nine Grand Rapids
moved anchors, with inherited catalogs and green Safari configuration intact.
The fresh whole-state atlas is
`renders/grand-rapids-reviewed-state-v1364/full-state-native-current.png`.

The final current-source replay on the reproduced v05 pack passes all twelve
cases in `renders/grand-rapids-reviewed-controller-1793511703430559000/`.
Two production stdio sessions pass 269 calls with byte-empty stderr in
`renders/grand-rapids-depot-quay-stdio-v1365/`. They load actual controller saves,
walk the new service circuit, decline the angler, use the registered Old Rod,
encounter SCUTLET, use native RUN and return to the same bank. Input hashes
remain exact. The historical save inspector initially rejected version26;
that failure is retained, and the successful run uses a freshly rebuilt reader.

The wider mine/campaign replay helpers now resolve Grand Rapids' actual paired
exterior records instead of hardcoding the old clinic/depot positions. Those
helpers compile and pass all-target Clippy; a full earned regional course has
not been rerun on v05. No story condition, partner, reward or resource was changed.

Selection is recorded in
`renders/selected-grand-rapids-depot-quay-v1366/receipt.json`, which retains the
preceding world and hashes the exact acceptance proofs. The sixteen-stage
rebuild, old v04 compatibility, twelve publication refusals and nine corrupted
source-index refusals pass. Scoped formatting passes; the workspace check keeps
its unrelated earlier difference at `crates/offset-allocator/src/lib.rs:408`.
The full map/cave improvement goal remains active.

Postselection source previews, the default v05 source-index audit, exact
whole-root preservation and both early-fishing courses pass in
`renders/selected-grand-rapids-final-checks-v1371/receipt.json`. The active world
hash and all bound source/recipe/index/pack bytes remain exact. Generated packs,
renders, saves, verification logs and binaries remain ignored.
