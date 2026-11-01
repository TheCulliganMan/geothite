# Outdoor boulder setbacks

Ten existing boulders move one32px metatile north, leaving a continuous native72
front wall in their old cells. This is an additive candidate on the unselected
outdoor stair-repair v02 pack, not a replacement for current selected geography.

`plan.json` records every old coordinate, destination preimage, private tileset,
whole-map guard and parent SHA256. `views.json` records the matching native
review cameras. Events, actors, encounters, connections, art, audio and field
move tables must remain exact. The winter route keeps its icy grass and pines.

| Map | Original boulder coordinates | New coordinates |
| --- | --- | --- |
| MnBoundarySafariC00R01 | (16,5), (18,5) | (16,4), (18,4) |
| MnBoundarySafariC01R00 | (22,16) | (22,15) |
| MnBoundarySafariC01R01 | (19,16) | (19,15) |
| MnR05C00R01 | (11,10) | (11,9) |
| MnR09C00R00 | (20,17) | (20,16) |
| MnR10C00R00 | (8,6), (26,8) | (8,5), (26,7) |
| MnR11C00R01 | (17,21) | (17,20) |
| MnR13C00R00 | (20,12) | (20,11) |

Full-resolution before/after maps and crops are retained under
`renders/outdoor-stair-landings-v01/boulder-setbacks-v48/`. Inspect the combined
map after all listed edits, not only each isolated move. Static body-aware
FLOOR/TALL_GRASS components retain all preexisting connections on all eight maps;
`topology.json` records that limited claim. It does not prove native traversal,
Cut progression or earned regional access.

The first publication attempt used the previous compiled builder and failed
strict verification because it repeated `minnesota-outdoor-cliff-joins-v1`.
Nothing was selected or overwritten. The Rust builder now gives this boulder-only
stage its own `minnesota-outdoor-boulder-setbacks-v1` provenance and rejects
reinstallation. Both attempts' logs are retained. The fresh build/publication,
full byte-preservation audit, all377 audio checks and native input are pending.

Native verification uses
`outdoor_setback_boulders_leave_walkable_rims_and_solid_cliff_faces` with
`CRYSTAL_OUTDOOR_BOULDER_PLAN` pointing at this plan and
`CRYSTAL_DULUTH_CITY_CONTROLLER_PACK` pointing at its published child. Each
initial local fixture crosses both exposed rim lanes, rejects walking north
into the relocated rock and south through the high cliff, and preserves the
party and bag. Fixture setup is not an earned campaign.

Corner boulders, attached ore piles and stacked rock formations remain outside
this ten-move phase. Their contours need individual corner/top/side repairs;
replacing all of them with a straight72 would erase their actual shape.

A separate native ten-boulder v02 candidate now exists. Its verified publication
and completed Cargo-built library/publisher hashes are recorded in
`renders/outdoor-stair-landings-v01/boulder-publication-v59.json`. This reused
the public Rust API from a completed library without modifying another owner's
cache. It does not include the later seven-corner source addition. The queued
v50 CLI publication uses its own fresh v01 path and remains separate. Both are
unselected; preservation/audio and production-input evidence must be resolved
for the exact candidate used for integration.

The v59 strict whole-pack comparison passed: exactly20 terrain cells change on
eight maps; every other game-data field, runtime/art/program/audio file and
audio manifest remains exact. Receipt:
`renders/outdoor-stair-landings-v01/boulder-preservation-v59/receipt.json`.
This is preservation evidence, not the dedicated boulder walking proof.

All377 audio hashes/frame counts/loop ranges pass for v02 in
`boulder-audio-v59.log`. Production MCP v60 passed nine of ten rim courses.
The remaining Iron Range case exposed Mina facing north through the cliff at
(41,35) when the cleared rim reaches(41,34). Its failure is retained and
acceptance is withheld. `../outdoor-trainer-sightlines-v01/plan.json` turns her
along the reachable lower bench while retaining the battle. The composed
v66 pipeline applies that repair before rerunning the boulder cases and
verifying seven further corners.

The composed terrain v02 production MCP run now passes all17 boulder/corner
cases with210 actual calls and empty stderr, including Mina's formerly failing
upper-rim crossing. Evidence: `current-terrain-stdio-v72/receipt.json` and
`current-terrain-mcp-proof-v72.json` under the outdoor-stair render folder.
Initial local position/partners/Repel are explicit setup. No trainer is marked
defeated and no state is repaired after input. Mina's retained lower-bench
battle, scaled roster and fresh renderer checks remain separately queued.
