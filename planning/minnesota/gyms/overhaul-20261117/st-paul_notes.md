# St. Paul: two civic routes

Authoring candidate on exact refinement v02 (`a9f23e3b...f37a9527`). This is a
static design, not an installed room or gameplay verification. Use `st-paul.json`
with `st-paul-contract.json`; do not run the historical four-gym rebuild.

The lobby offers a signposted choice: the north pad reaches BELVA's narrow
wooden archive, while the south pad reaches ORVEN's steel workshop. Win either
match to qualify for BRANNIC. The other opponent remains an optional training
route. Archive shelves, workshop machinery, the shared council grille and the
wooden public lobby use native source pixels and distinct floor/wall families.
The four reciprocal pad pairs all remain; no random destinations or resetting
trainer victories.

## Geometry and indexed pads

Retain `MnStPaulGym`, `MN_ST_PAUL_GYM`, `mn_gym_4`, dimensions 9x12 metatiles,
arrival (2,21), exits (2,23)/(3,23), exterior warp records1-3 and their existing
town destinations. Coordinates below are walking tiles, not metatiles.

| Existing pair indices | From | To | Role |
| --- | --- | --- | --- |
| 4 / 5 | (3,13) | (3,7) | Lobby north / archive |
| 6 / 7 | (5,9) | (11,9) | Archive / council anteroom west |
| 8 / 9 | (5,17) | (11,13) | Lobby south / workshop |
| 10 / 11 | (15,21) | (15,9) | Workshop / council anteroom east |

Leader (13,3) and guide (6,20) stay. BELVA moves (3,3) -> (5,3);
ORVEN moves (13,14) -> (13,16). Preserve their object identifiers and slots.
Both explicit sightlines point down, radius2. Every pad is a native WARP_PANEL
quadrant; adjacent floor permits stepping off and returning through the pair.
The source renderer draws standing actors facing south, matching this contract.

## Control and qualification contract

There is no mechanical switch. Route choice is made by taking a labeled pad.
The guide and each opponent's after-text in JSON are ready to copy into the
canonical dialogue; all lines are ASCII and at most18 columns.

`court.all = []`; `court.any = [EVENT_MN_GYM_4_TRAINER_1_DEFEATED,
EVENT_MN_GYM_4_TRAINER_2_DEFEATED]`. Compute the existing COURT_OPEN qualification
before applying the gate. Do not accidentally require both old trainer wins.

The one block transition, in metatile coordinates, is:

| x | y | Off | On | Requires | New visual receipt |
| --- | --- | --- | --- | --- | --- |
| 7 | 2 | G | c | EVENT_MN_GYM_4_COURT_OPEN | EVENT_MN_GYM_4_V2_GRILLE_OPEN |

All historical flag numbers, native leader class, parties, rewards, rematch and
circuit scripts remain owned by the selected pack. The JSON copies current
installed teams, not the historical nonregional party placeholders. Existing
completed leader victories must continue to authorize return/rematch access.
The old schema-wide validator requires an individually mandatory trainer and a
single legacy court gate; this fork intentionally uses the companion predicate
and the new overlay's validator instead. JSON fields still match Gym schema.

## Static evidence and limits

The native glyph allocation is77 of192. The authored-cell model includes solid
actors and reciprocal pad transfers: both trainers and both exits are reachable
with the grille shut, the leader is not; either-route qualification makes the
leader approachable. This is topology evidence, not a controller test.

Whole room, arrival and leader views plus the open-grille state are retained in
`target/gym-overhaul-20261117/balance-postgame/v04/st-paul/`. Prior visual drafts
and source reference sheets remain retained under the same ignored review root.
Save migration must handle the relocated actors/pads and changed block geometry;
it must not merely whitelist the new pack hash.
