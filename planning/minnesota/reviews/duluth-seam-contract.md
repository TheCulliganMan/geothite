# Duluth regional seams — implementation contract

**Planned integration, not installed.** This audit read the completed
`content-packs/minnesota-cities-review.crystalpack` on 2026-10-26: 498 maps,
file SHA256 `6e3295367d6f2b4f6b07ef8f95f403968f2dcd3fe4790991b6e5f7380ef879af`,
content identity `10b50e16410e3a73b3ffe29228cc15abc4b6a021482d5e1a3419658e9afe1713`.
Duluth has **no connections and no coordinate events**. Its fourteen warps
belong to the seven paired building doors/returns; none is a regional gate.
The neighboring routes have their internal chunk connections, but no Duluth
connection. This document changes no map source, native implementation or pack.

The world currently describes these two links as `gatehouse_warp`. The
recommended implementation below is **reciprocal walking connections**, following
the installed Boundary Waters mechanism. Root must explicitly update those
two world-edge records when implementing it; do not claim the old warp plan
already describes a walking seam. Keep the original port and landing anchors.
There is no gatehouse room or regional warp index to reuse.

## Inputs and native coordinate rule

Use the frozen Duluth candidate, not a changing live town file:

- Town: `.local/minnesota-city-source-freeze/MnDuluth-932ac34a722a1600a92f90933734efe357bdf86732ad87f21759bf1360eacf6a.json`.
- Layout: `planning/minnesota/renders/duluth/source-snapshots/MnDuluth.layout-213a8b6dc5f38c07fb89ca990cb70dab130e6e3abeb09418ec4c5bf86d9ef06b.json`.
- Canonical gyms: `.local/minnesota-city-source-freeze/gyms-887afc43d19bc0573ca5ec361f4c141771347421040f70f100f02fea4997a189.json`;
  Gym8 object SHA256 `c1554f9d93d56117086ad978f6150c923bb877c336785e6e3ff30b07de9360ee`.

Other inspected input file hashes:

| File under `planning/minnesota/` | SHA256 |
|---|---|
| `world.json` | `4a24afa89a66c18c56bfbc00378ef00917b2d185aca1d20e88c5eb7b0ba8b476` |
| `story.json` | `27afee2095e41403efd8a20279d1baca956f9755902e248a6788727df7aabeaf` |
| `routes/MN_R10.json` | `862a929551200665b9eaa4b43ad474636acc69b5961f4e438cf6729f21b92d77` |
| `routes/MN_R10.layout.json` | `2db59b63a1184b5f0975c30f3ff8fec1bebbce2bf6cf18c81895410f3ff94e5a` |
| `routes/MN_R11.json` | `da3061d8281037a6501872113360125b472c716bfb2181e2af57355775b2e54f` |
| `routes/MN_R11.layout.json` | `a52fababd7f5c7873fc01eb7b90eca02bfec9582812e13d74f7a5df42b3760e5` |

The two inspected route chunks match these live layouts exactly. A changed
hash requires a scoped comparison of the actual chunk and checkpoint geometry;
do not silently migrate the frozen city or regenerate Gym8.

One metatile contains 2×2 core gameplay tiles. All coordinates below are
zero-based. `offset` is in **metatiles**. Native
`connection_destination_tile_components` subtracts `2 * offset` from the
perpendicular core coordinate. For north/west it sets the destination's other
coordinate to its **actual** bottom/right edge; for south/east it sets it to 0.
The trigger position is the attempted tile just outside the source map.

Read the implementation, not a visual atlas:

- [`connection_for_tile_checked`](../../../crates/crystal-core/src/world/session.rs): native boundary trigger.
- [`connection_destination_tile_components`](../../../crates/crystal-assets/src/game_data.rs): checked offset calculation.
- [`resolve_connection_transition`](../../../crates/crystal-assets/src/game_data_encounters_and_loading.rs): destination lookup.
- [`boundary_basecamp_connections.rs`](../../../crates/crystal-assets/src/boundary_basecamp_connections.rs): additive reciprocal connections, physical guards, coordinate planes, collision/bypass checks and atomic split publication.
- [Boundary controller cases](../../../crates/crystal-bevy/src/bevy_shell/tests/boundary_basecamp_connections.rs): real crossings, every prerequisite refusal, inbound care, transverse arrivals, Surf and Continue.

## Exact ports and reciprocal walking records

| Link | City port gate / landing, metatiles | Actual neighbor and extent | Neighbor original gate / landing, local metatiles |
|---|---|---|---|
| R10 west | `(1,12)` / `(3,12)` | `MnR10C01R00`, **16×20**, core 32×40; global route origin meta `(32,0)` | `(14,12)` / `(12,12)`; global `(46,12)` / `(44,12)` |
| R11 north | `(16,1)` / `(16,3)` | `MnR11C00R02`, **32×24**, core 64×48; global route origin meta `(0,48)` | `(17,22)` / `(17,20)`; global `(17,70)` / `(17,68)` |

Do not pad R10's last chunk to 32×24 or use x63 as its right edge.

| Source | Direction / offset | Target | Source edge lanes → attempted crossing → destination lanes |
|---|---|---|---|
| `MnDuluth` | west / **0** | `MnR10C01R00` | `(0,24),(0,25)` → `(-1,24),(-1,25)` → `(31,24),(31,25)` |
| `MnR10C01R00` | east / **0** | `MnDuluth` | `(31,24),(31,25)` → `(32,24),(32,25)` → `(0,24),(0,25)` |
| `MnDuluth` | north / **−1** | `MnR11C00R02` | `(32,0),(33,0)` → `(32,-1),(33,-1)` → `(34,47),(35,47)` |
| `MnR11C00R02` | south / **+1** | `MnDuluth` | `(34,47),(35,47)` → `(34,48),(35,48)` → `(32,0),(33,0)` |

R11's original endpoint core `(34,44)/(35,44)` is **inside** its chunk, not a
walking seam. Preserve it and extend the final two rows to the edge. Both
route arrivals and both city arrivals must be ordinary `FLOOR`, not a door,
warp carpet, sign, guard body or water tile.

Useful safe interior positions, before considering temporary guard bodies:

| Side | Clear interior lanes | Original landing lanes |
|---|---|---|
| Duluth west | `(4,24),(4,25)` **after the sign fix below** | `(6,24),(6,25)` |
| R10 east | `(30,24),(30,25)` | local `(24,24),(24,25)`, global `(88,24),(88,25)` |
| Duluth north | `(32,4),(33,4)` | `(32,6),(33,6)` |
| R11 south | `(34,46),(35,46)` **after extension** | local `(34,40),(35,40)`, global `(34,136),(35,136)` |

## Required geometry revision — separate review candidate

These are proposed edits, **not applied** to the frozen city or current routes.
The complete proposed six-metatile change was checked in memory against actual
published collision. Retain the current whole-town 02bb render and make a new
whole-town/source-camera review before accepting the sign relocation.

| Map / source position, metatiles | Current → proposed native block | Reason |
|---|---|---|
| R10 global `(47,10)`; chunk`(15,10)` | `02` → `05` | Close extra east-border Floor core y20/21. |
| R10 global `(47,11)`; chunk`(15,11)` | `02` → `05` | Close extra east-border Floor core y22/23. |
| R10 global `(47,13)`; chunk`(15,13)` | `02` → `05` | Close extra east-border Floor core y26/27, joining the existing rail end at global `(46,13)` without changing that rail. |
| R11 global `(17,71)`; chunk`(17,23)` | `05` → `01` | Open core x34/35, local y46/47; existing y44/45 already Floor. |
| Duluth`(2,12)` | `45` → `01` | Remove sign's blocking core `(4,24)` from the required west refusal landing. |
| Duluth`(3,11)` | `05` → `45` | Reinstall that same sign above the approach, with its event moved from core `(4,24)` to **`(6,22)`**. Preserve `DULUTH_SIGN` ID/text. |

In all three installed tilesets, `05` is WALL × 4 with no Cut quadrant; `01` is
FLOOR × 4. `45` is `[WALL,FLOOR,FLOOR,FLOOR]`. The proposed new sign's core `(6,22)`
is Wall and its east/south approaches are Floor. Its three added walking
quadrants remain attached to the public road. Original west landing`(6,24)`
stays Floor. No door, room, actor, ledge or water-launch anchor moves.

**Why all these edits matter:** R10 currently has eight Floor cells at its
east border (y20–27), but Duluth has only y24/25. A direction-wide connection
cannot mean “only use these two lanes”; leave the extra cells open and callers
can request wrong-collision arrivals. R11 currently has **no south-border
Floor**. A connection record alone cannot make its pine boundary traversable.
Finally, simply cloning Boundary's west checkpoint onto Duluth would force an
upper-lane refusal into the existing sign at`(4,24)`.

The in-memory six-metatile candidate leaves exactly R10 east Floory24/25 and
R11 south Floorx34/35. Removing all NPC bodies for a conservative Floor/grass/
door/Water flood, either proposed inner plane blocks its city edge; with both
planes closed, all seven exterior passive returns remain reachable. This is
static collision evidence, **not** native forced-movement or controller proof.

If Root declines the sign revision, keep the city unchanged and redesign the
west script explicitly. It needs lane-specific refusal and a longer upper-lane
inbound escort around the sign. Do not use a blocked straight refusal, pretend
one upper actor seals both lanes, or quietly move the frozen sign in a pack.

## Physical checkpoints and exact story requirements

The following placement is for the reviewed six-metatile candidate above.
Both lanes use the same behavior, mirrored for north and west.

| City checkpoint | Edge | Outer inbound plane | Temporary body | Inner outbound check | Refusal destination |
|---|---|---|---|---|---|
| R10 lane0 | `(0,24)` | `(1,24)` | `(2,24)` | `(3,24)` | `(4,24)` |
| R10 lane1 | `(0,25)` | `(1,25)` | `(2,25)` | `(3,25)` | `(4,25)` |
| R11 lane0 | `(32,0)` | `(32,1)` | `(32,2)` | `(32,3)` | `(32,4)` |
| R11 lane1 | `(33,0)` | `(33,1)` | `(33,2)` | `(33,3)` | `(33,4)` |

All these candidate positions are ordinary Floor, currently unoccupied by
objects/events. Keep the original passive `MN_DULUTH_NORTH_GUARD` at`(33,5)`;
it cannot substitute for the two new physical bodies. Suggested additive IDs
are `MN_DULUTH_R10_GUARD_NORTH/SOUTH` and `MN_DULUTH_R11_GUARD_WEST/EAST`, with
unique script roots `MnDuluthR10Check` and `MnDuluthR11Check`. Check global
script/object collisions before adding anything. Temporary visibility uses
native `event_flag=-1`, never a progression flag as an object visibility switch.

- **Duluth→R10:** require `BADGE_7` and `EVENT_MN_DEPOT_CLEARED`, exactly the
  two R10 world-edge requirements and route `required_flags`. Native badge7 is
  `ENGINE_GLACIERBADGE`. Q06 also requires the headwaters report upstream;
  **do not add that prerequisite to this seam** or equate a foreman victory
  with `DEPOT_CLEARED`.
- **Duluth→R11:** require all eight native badge flags plus
  `EVENT_MN_LIGHTHOUSE_SIGNAL`. Native badge order is
  `ENGINE_ZEPHYRBADGE`, `ENGINE_HIVEBADGE`, `ENGINE_PLAINBADGE`,
  `ENGINE_FOGBADGE`, `ENGINE_STORMBADGE`, `ENGINE_MINERALBADGE`,
  `ENGINE_GLACIERBADGE`, `ENGINE_RISINGBADGE`.
- **R11→Duluth:** source town's `north_checkpoint_contract.inbound_access`
  explicitly says **free**. Do not require flags to return to the city/clinic.
- **R10→Duluth:** free inward care is the recommended directional policy,
  matching Boundary's checkpoint design. It is **not yet specified** by the
  current symmetric `gatehouse_warp.requires_flags` record; Root must record
  this policy when changing that record to a walking seam. Until then this is
  a proposed policy, not an already installed or authoritative world promise.

The native Q07 receiver is already in the workshop at core `(11,5)`. Its all-eight
badges plus `EVENT_MN_DEPOT_CLEARED` checks grant **only**
`EVENT_MN_LIGHTHOUSE_SIGNAL`. Preserve that receiver, its anchor, repeat flow
and canonical Gym8 party/reward. The north seam checks the resulting signal;
it does not grant it. `EVENT_MN_SURVEY_RESTORED`, Safari fees, Root Beer Lady,
Split Rock lamp repair and any train ticket are not Duluth seam prerequisites.
The story file still labels MS10's native binding pending; that prose is stale
relative to the actual city receiver, not proof the regional seams exist.

Implementation behavior, following Boundary's actual native flow:

1. On an inner-plane step, check every required badge/event. Success hides both
   bodies and permits departure. Denial presents text then forces **one inward
   native step**: RIGHT for R10, DOWN for R11, to the exact refusal cell above.
   No badge, item, money, party, story event or permission state changes.
2. On the outer plane, only an explicitly **outward-facing** step skips the
   inbound escort: LEFT for R10, UP for R11. All other facings—including a
   sideways lane change—hide the pair and force **two inward steps** through
   body to inner plane. Reconcile visibility afterward, showing bodies again
   when requirements are missing. A traveler may then walk inward to public
   services. Do not make free inbound care contingent on progression.
3. Do not copy Boundary's direction match literally: its configured inward
   directions omit DOWN; Duluth's north passage needs it. Fail on unsupported
   directions and occupied/invalid cells rather than returning a connected,
   unguarded pack.
4. The old town-only north contract lists trigger y2 and a single refusal
   `(32,4)`. It is **source-only**. For this physical design y2 is the body,
   **y3 is the check**, and refusal preserves x32 or x33. Root must reconcile
   that contract rather than install both overlapping versions.

## Publication and proof required before claiming regional play

Root owns implementation, shared-source updates, registration and publication.
Add the four records without replacing existing route-internal connections,
and update `connection_flags` from all directions. Keep city warps 1–14, rooms,
canonical gym binding, existing NPCs, encounter/party tables, HOME/recovery
records, unrelated pack data and audio exact. Route source strokes and global
contract revisions must describe the same accepted geometry as the pack.

Use the Boundary builder's atomic clone/validate/install pattern. Update both
the full map modules and split script/NPC/attribute payloads, lower native
object/movement/variable command tables once, derive a new pack identity and
perform strict written readback. Reject a conflicting direction, missing target,
bad offset, changed extent, occupied guard plane, damaged Floor or bypass.

Required asset regressions:

1. Resolve **all eight** lane crossings through actual
   `connection_for_tile_checked` and `resolve_connection_transition`; assert
   the table above and destination Floor. An R11 offset 0 mutation must fail.
2. Enumerate **every** source border cell admitted by the new direction,
   including Surf-reachable cells. No additional ordinary walking/water lane
   may resolve into a wall or bypass the inner check. Preserve R10's existing
   west internal water seam and R11's north internal seams.
3. Conservatively remove every NPC body and block only the logical inner
   planes. Prove ordinary walking/Surf cannot reach either city edge, while
   every city building return/service remains accessible. Negative-test an
   opened shoulder and a removed coordinate event; a visible sprite alone is
   not the logical gate.
4. Preserve each unaffected module/field and written readback. No incidental
   Cut, outer Water escape, sign collision/approach defect or reverse ledge
   shortcut may appear. Compare the new whole city and west/north 160px cameras
   by sight before final acceptance of the proposed sign move.

Required production-controller cases must drive `VisibleShellController::press`
with explicit initial fixtures, not rewrite position/flags after starting:

- Both lanes in both directions, proper facings, reentry and Continue: eight
  crossings, exact safe arrivals, outward passage without an inbound bounce.
- R10: independently omit badge7 and depot. R11: independently omit each of
  eight badges and signal. Assert actual refusal dialogue, denied departure,
  inward forced landing and unchanged resources/flags; then prove success
  with the full guard set. These are fixtures, not earned regional travel.
- Unqualified inward travel on both lanes: no softlock, both bodies reconcile,
  public clinic/PC/shop and workshop remain reachable. Then attempt to reverse
  outward and prove refusal. Repeat after save/Continue and a building visit.
- Sideways outer-plane arrivals must escort inward rather than strand the
  player outside the bodies or permit reversing through the exit.
- Actual native Surf from every public harbor bank cannot reach either closed
  seam around the guard plane. Preserve buoy/rock boundaries and route water
  seams; no water connection between Duluth and either route is planned.

The existing six Duluth controller tests cover local doors, ledge, services and
receiver only. They do **not** cover these absent seams. Likewise, passing local
R10/R11 traversal does not prove a connected Grand Rapids→Duluth→Grand Marais
journey or earned Q06/Q07 progression. Keep those acceptance claims separate.
