# Connecting-cave geographic audit (playtest report 21)

Status: preliminary source inspection only. No game content, selection, saves,
warp records or cave geometry were changed. This is not a complete statewide
inventory or verification against the current Northstar pack.

Coordinates below are 16-pixel gameplay tiles. Exterior Route 5 coordinates
are global across route chunks; increasing x is east and increasing y is south.

| Source | Exterior mouths | Corresponding interior mouths | Preliminary observation |
| --- | --- | --- | --- |
| Route 1 limestone hollow | (18, 17) | (9, 11) | One exterior mouth. This is an optional hollow, not a Winona–Rochester passage. Its fossil gallery is reached by a stair and returns to the hollow; it does not provide a second surface exit. |
| Route 5 Rock Cut | (16, 77), (30, 27) | (5, 17), (11, 5) | The second mouth lies northeast of the first both outside and inside. Source records agree in direction; this alone does not verify the current pack or the intervening route. |
| Route 20 granite cleft | (60, 33), (76, 31) | (5, 13), (13, 13) | East/west order agrees, but the recorded interior mouths have the same y while the eastern exterior mouth is two tiles north. The route dossier may not represent the current installed room; inspect actual pack records and geometry before deciding whether any change is needed. |

Sources: `routes/MN_R01.json`, `routes/MN_R05.json`, `routes/MN_R20.json`,
`caves/johto-quality-v01/MnR01Cave1F.json`, `playable/mine-and-hollows.md`,
`caves/current-r05-v01/README.md`, and
`crates/crystal-assets/src/driftless_fossil_gallery.rs`. The Route 5 runbook
records earlier production traversal, but that historical evidence is not a
fresh test of the current Northstar composition.

To finish report 21, identify the actual current Northstar pack and enumerate
all surface-to-underground warp pairs, including multi-floor chains and their
relative surface coordinates. Trace each underground path and preserve separate
active entrances and passive returns. Design a real east–west Route 1 passage
with two geographically corresponding surface mouths, lit early traversal,
and existing hollow, fossil gallery, actors, items, scripts and saves accounted
for. Build an ignored candidate from the integrated parent, visually inspect the
whole room and production views, then exercise both directions, same-mouth
returns, reentry, persistence and unrelated-content preservation. Reconcile any
shared source and publication records only after coordination with the owners
of concurrent changes. Do not check off report 21 until the complete inventory
and actual gameplay checks pass.

## Unpublished Route 1 draft

`r01-passage-draft.json` and `northstar_driftless_passage.rs` stage a review
candidate against `minnesota-story-review-fixes-v09.crystalpack`. They retain
all existing hollow coordinates and add an eastward extension ending at
(31,13); the existing west exit remains (9,11). The proposed new surface mouth
is (40,19), east and slightly south of the existing (18,17); its passive return
is (40,20). The existing outdoor road remains available, and the passage uses
the hollow's native NITE palette. The separate lower gallery remains dark.
The new geometry, publisher, exterior approach and gameplay are not yet
verified or selected. The draft's whole-room art still needs visual refinement.

A static collision flood on the draft, with the original guide and item
occupied, reached both exits and the lower gallery ladder. This is not a
controller, save, progression, publication or visual-acceptance test.
On the v09 pack the actual R20 interior exits are (5,21) and (23,21),
not the earlier 9×8 dossier values. The exterior east/west ordering agrees;
the small north/south discrepancy and other connecting chains still need
geographic review.

## Candidate-pack entrance inventory

A read-only scan of the v09 candidate grouped Minnesota `CAVE` and `DUNGEON`
maps through their warp edges, then inspected warps from `ROUTE` and `TOWN`
maps using the installed collision quadrants. It found two underground
components with two active surface cave mouths: R05 Rock Cut and R20 Granite
Cleft. Route 1, R03, R09 and its six-room Iron chain, R10, R12, R13, R17,
Voyageurs Ice and Inner Basin each have one active surface cave entrance in
this scan. Indoor buildings classified as `DUNGEON` were excluded from the
cave-mouth count. Passive floor return records were distinguished from active
CAVE tiles. This inventory does not follow map-edge connections, prove
reachability, or certify a later integration pack.

The v09 R05 active mouths are global (16,77) and (30,27), paired to room
(5,17) and (11,5), respectively: northeast in both coordinate systems. R20
active mouths are global (60,33) and (76,31), paired to room (5,21) and
(23,21): the east/west ordering agrees, but the two-tile northward exterior
offset is not reflected in the room. The Iron haulage/approach records are
underground `DUNGEON`/`CAVE` links rather than additional surface mouths.
Both two-mouth candidates still require actual traversal and full geometric
review on the final integrated pack.

The draft eastern Route 1 return was also flooded through FLOOR, TALL_GRASS
and LADDER quadrants with installed actor positions treated as blocked; both
route edge pairs and the original hollow return were reachable. This ignores
directional movement rules and does not replace ordinary-input verification.

The same unpublished Rust candidate also moves the R20 east interior ladder
from (23,21) to (23,19), reflecting the exterior's two-tile northward offset.
The exterior warp target remains the original east mouth; the former interior
ladder tile remains floor for existing position saves. A static collision flood
reaches the new ladder and former tile from the west arrival with existing
actors occupied. The native-art draft was inspected, but neither this change
nor save compatibility has been exercised through the game controller yet.

The previously queued pack builder, preservation test, controller tests and
audio verifier were cancelled at the coordinator's direction. The controller
tests require the resulting ignored review pack; all must pass before this draft
is considered playable. No active pointer or user save has been changed.
The v09 scan also found no map-edge connections on Minnesota `CAVE` or
`DUNGEON` maps, so there are no additional underground edge links in that
candidate to add to the warp graph. Scripted or later-pack transitions remain
outside this check.

Integration conflict: concurrent task 13 work changes `MN_R01_SIGN_03Text` and
`routes/MN_R01.json` to describe the current hollow as a dead end. That is
accurate for its existing candidate but contradicts this proposed connection.
If the passage is accepted, apply its sign after that overlay and coordinate a
single source reconciliation; do not apply the dead-end sign afterward. The
source dossier remains untouched while that work is active.
A separate name check for Minnesota cave, cavern, tunnel, mine, grotto, cleft,
and underpass maps found none outside the two scanned environment classes;
this is a heuristic and does not rule out differently named scripted spaces.
A scan of local script bodies on those maps also found no `warp`,
`warpfacing`, `warpmod`, or `setwarp` commands. Global specials or unrecognized
transition commands are not excluded by that check.

In the current draft, the surface displacement from west to east is
(+22,+2) gameplay tiles: (18,17) to (40,19). The interior displacement is
also (+22,+2): (9,11) to (31,13). The 20×8 source retains every original
hollow block except the four blocks opened at its east edge, and a static flood
reaches every unoccupied FLOOR tile. The retained gallery ladder remains at
(11,9). `review-v5` under ignored `target/r01-passage-draft/` contains the
native-art source views and source-only check; it does not certify a pack.

Commands for the coordinator (not run to completion):

```sh
CARGO_INCREMENTAL=0 cargo run --locked -p crystal-assets --bin pack_northstar_driftless_passage -- \
  content-packs/minnesota-story-review-fixes-v09.crystalpack \
  planning/minnesota/caves/geographic-audit-v01/r01-passage-draft.json \
  target/northstar-driftless-passage-review-v01.crystalpack
CRYSTAL_DRIFTLESS_PARENT="$PWD/content-packs/minnesota-story-review-fixes-v09.crystalpack" \
  CARGO_INCREMENTAL=0 cargo test --locked -p crystal-assets --lib \
  driftless_passage_preserves_other_maps_and_pairs_both_mouths -- --ignored --nocapture
CRYSTAL_DRIFTLESS_REVIEW_PACK="$PWD/target/northstar-driftless-passage-review-v01.crystalpack" \
  CARGO_INCREMENTAL=0 cargo test --locked -p crystal-bevy --lib \
  northstar_driftless_passage:: -- --ignored --nocapture
```

Audio verification against the generated pack, affected-crate Clippy, and actual
native image inspection remain required. Do not promote or publish on source-only
checks.
