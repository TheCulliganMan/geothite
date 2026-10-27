# Brainerd four-seam implementation contract — PLANNED ONLY

No source map, world record, native pack or story state was changed for this
contract. Brainerd’s published city currently has no regional connections. The
four `gatehouse_warp` records in `world.json` are reservations with unallocated
warp indices, not playable walking links. Implement these seams in a separate
bounded native connection change, after the city/controller candidate is stable.

Coordinates below are **16px core tiles**, zero based, unless explicitly marked
metatiles. Connection offsets count **32px metatiles**. The runtime transverse
mapping is `target_core = source_core - 2 * source_offset`; reciprocal offsets
must have opposite signs. A seam crossing lands on the neighbor’s actual outer
core row/column, not its authored interior landing metadata. No new warp events
are needed or permitted for these four walking connections.

## Exact maps, lanes and offsets

The route identifiers and requirements come from `world.json`, `story.json` and
the four current route dossiers. Collision comes from the actual written498-map
candidate SHA256
`6e3295367d6f2b4f6b07ef8f95f403968f2dcd3fe4790991b6e5f7380ef879af`.
The raw-mirror repair pending for that candidate does not alter these borders.

| Town direction | Reciprocal route chunk / extent | City edge lanes → route edge lanes | City offset / route reverse offset | Existing requirement |
|---|---|---|---|---|
|south|MnR06C00R00,32×24 metatiles; route north edge|(32,47),(33,47) → (42,0),(43,0)|−5 / +5|BADGE_5 = ENGINE_STORMBADGE|
|north|MnR07C00R01,32×24; route south edge|(32,0),(33,0) → (20,47),(21,47)|+6 / −6|BADGE_6 = ENGINE_MINERALBADGE|
|west|MnR15C00R00,32×18; route east edge|(0,24),(0,25) → (63,24),(63,25)|0 / 0|BADGE_6 = ENGINE_MINERALBADGE|
|east|MnR18C00R00,32×24; route west edge|(63,24),(63,25) → (0,10),(0,11)|+7 / −7|BADGE_6 = ENGINE_MINERALBADGE|

Brainerd is32×24 with johto art. R06/R07/R18 use mn_headwaters; R15 uses kanto.
R06’s complete route is32×44 (second chunk32×20); R07 is32×48. R15’s endpoint
chunk really is only18 metatiles tall. R18’s complete route is40×24, with its
existing8×24 eastern chunk retained. Do not substitute a full32×24 template for
R15 or change any existing route-internal zero-offset connection.

## Current border mismatch and minimal geometry work

Brainerd already exposes exactly the two FLOOR tiles in each city edge pair;
all other city outer core tiles are WALL. No city exterior block edit is needed
for alignment. None of the listed route openings is yet a reciprocal connection.

| Neighbor edge as actually published | Required bounded source adapter |
|---|---|
|R06 north FLOOR x40–45,y0|Keep42/43. Change outer metatiles(20,0) and(22,0), currently native02 FLOOR, to complete native05 pine WALL matching adjacent(19,0)/(23,0). Keep middle(21,0)=01 FLOOR.|
|R07 south FLOOR x20–23,y47|Keep20/21. Change outer metatile(11,23), currently02 FLOOR, to native05 pine WALL matching(9,23)/(12,23). Keep(10,23)=01 FLOOR.|
|R15 east FLOOR x63,y22–25|Keep24/25. Change outer metatile(31,11), currently31 FLOOR, to native0f pine WALL matching(31,10)/(31,13). Keep(31,12)=31 FLOOR.|
|R18 west has **no FLOOR opening**; metatile(0,5)=05 WALL|Author a real path at metatile(0,5), using native01 FLOOR matching existing(1,5). This opens both core lanes y10/11 and joins x2/3. Preserve native05 WALL at(0,4)/(0,6) and all BUOY boundary cells y40–47.|

These are **proposed source metatile edits**, not applied changes. Re-run actual
source shore/cliff fitting and inspect each entire route and both seam cameras;
if final fitting changes an adapter beyond these five explicit outer cells,
inspect and explain that exact delta before freezing it. Do not patch Python
pack bytes or create new art. The native Rust builder must require the reviewed
adapter recipe and exact collision rather than quietly repair an arbitrary pack.

R06/R07/R15’s surplus open edge cells currently project onto city WALL. Leaving
those cells open would create an invisible seam barrier or unsafe target landing;
that is not an acceptable substitute for the visible pine closure above.
R18 cannot connect by metadata alone: its route endpoint begins one metatile
inside a closed tree boundary. Do not confuse its gate(1,5) with an actual edge.

## Interior landings and checkpoints

Retain the exact authored port metadata. These safe interior metatile anchors
are not warp targets and do not replace the literal edge arrivals above.

| Passage | Existing city gate / landing metatile | Existing route gate / landing metatile | Proposed explicit public anchor pairs, core tiles |
|---|---|---|---|
|R06|gate16,22; landing16,20|gate21,1; landing21,3|city(32,40),(33,40); route(42,6),(43,6)|
|R07|gate16,1; landing16,3|global gate10,46; landing10,44; chunk-local gate10,22, landing10,20|city(32,7),(33,7); chunk(20,40),(21,40)|
|R15|gate1,12; landing3,12|gate30,12; landing28,12|city(7,24),(7,25); route(56,24),(56,25)|
|R18|gate30,12; landing28,12|gate1,5; landing3,5|city(56,24),(56,25); route(7,10),(7,11)|

Freeze those public core anchors only after actual FLOOR/object/reachability
validation; they are designated rows within existing landing metatiles, not a
claim that metadata alone verifies collision. Each must connect by ordinary
walking to its real service destinations and route main path. Preserve the
ranger25,19, all six existing exterior residents, five facade pairs, the southern
ramp and east ledge, campsites, lake and railed pier.

Use two visible stationary native Officers at each city-side throat, never a
Cut tree that can be walked around or a hidden coordinate-only barrier. Proposed
anchors below are currently FLOOR, unoccupied and sealed laterally by WALL at
the body plane. They do not require changing city terrain.

| Passage | City inner trigger pair | Guard body pair | City outer trigger pair | Outward / inward direction |
|---|---|---|---|---|
|R06|(32,43),(33,43)|(32,44),(33,44)|(32,45),(33,45)|DOWN / UP|
|R07|(32,4),(33,4)|(32,3),(33,3)|(32,2),(33,2)|UP / DOWN|
|R15|(4,24),(4,25)|(3,24),(3,25)|(2,24),(2,25)|LEFT / RIGHT|
|R18|(59,24),(59,25)|(60,24),(60,25)|(61,24),(61,25)|RIGHT / LEFT|

Use new object/script identifiers such as `MN_BRAINERD_R06_GUARD_WEST` and
`MnBrainerdR06Check`; these are identifiers, **not new story permission flags**.
Allocate both lane objects explicitly, with `event_flag=-1` temporary visibility.
Six original actors plus eight guards means14 authored events. The native loaded
object roster supports12 simultaneous slots, so verify actual viewport allocation
and both guards’ collision/visibility after every entry and reload; do not assume
all authored events are loaded or move existing NPCs to hide a roster failure.

## Exact story and input policy

Preserve existing route requirements only: R06 needs Badge5; R07/R15/R18 need
Badge6. MS05 still requires Badge6 and grants the existing HM_SURF/report once at
ranger25,19. Do not require that HM, a new ticket, paid admission, the report flag,
a new chapter event, all eight badges or the lighthouse signal for these dry-road
seams. A Brainerd rail stop is outside the Glacier Line contract.

The existing world entries do not specify a free-inbound exception. This contract
therefore proposes **the same existing badge check in both directions**, until
Root explicitly chooses and records a different inward-care policy. The free
Boundary camp’s unconditional inward care must not silently become a new
Brainerd story rule. Normal earned badges persist, so ordinary qualified return
travel remains open. A denied fixture must always be able to walk back to its
origin; no refusal may trap the player on the other side of an invisible gate.

Use the existing
[Boundary connection builder](../../../crates/crystal-assets/src/boundary_basecamp_connections.rs)
and its controller cases as semantic references: visible bodies, both lane
coordinate planes, actual VAR_FACING, native applymovement, input ownership,
temporary appear/disappear, safe refusal and real crossing. Do not copy its
all-eight-badge/signal/restoration requirements or its inbound policy.

A concrete two-plane flow is:

1. At the inner plane, arrivals facing inward away from the gate need no escort.
   Outward and transverse arrivals execute the existing badge check. On refusal,
   show native text then push one tile inward. On success, hide both guard
   objects temporarily and move the player two native steps outward across the
   body plane to the outer plane.
2. At the outer plane, arrivals facing outward away from the gate need no escort.
   Inward and transverse arrivals execute the same check. Refusal pushes one
   tile toward the edge (still inside the city). Success moves two native steps
   inward across the cleared bodies. Both A interactions with either guard call
   the same eligibility/plane-aware flow; turning while blocked cannot bypass it.
3. Validate each push/escort destination as FLOOR and unoccupied in the real
   input fixture. Restore bodies whenever requirements remain unmet; fresh map
   entries begin closed. No event grants eligibility, no visibility flag claims
   a story milestone, and text must release input before ordinary walking resumes.

The outer refusal destinations are respectively y46, y1, x1 and x62; all remain
in-map. The inner refusal destinations are respectively y42, y5, x5 and x58.
Check both members of every pair and all four facings, including a transverse
arrival across the two-lane plane. Existing walking must never cross WATER.
Conservatively include every reachable Surf tile in the bypass flood proof:
this lake-country hub will eventually have Surf, but its pier/lake cannot provide
an unguarded path around the east checkpoint. Keep all buoy/rock boundaries.

## Registration and required proof before calling it playable

Replace the four obsolete planned gatehouse-warp records with explicit
per-passage guarded walking records only when the native implementation exists;
do not append a duplicate gate or assign fictional warp indices. Register both
real directions/offsets, clipped chunk extents, two exact edge pairs, existing
required badge, trigger/body anchors and the chosen two-direction policy.
Current `check_boundary_walking_links.py` rejects any non-Basecamp
`guarded_connection`; narrow its identity scope and add an equally strict Brainerd
validator in that implementation change. `check_plan.py` also assumes zero
translation on plain `connection` records. Neither current check alone can
validate these proposed nonzero guarded seams.

The native builder must deny missing/wrong source, occupied direction/anchors,
extra edge openings, unsafe collision/Surf bypasses and stale split payloads.
It must preserve every unrelated map, original script/NPC/party/audio, existing
route-internal connection, HOME/recovery entry, city facade pair and canonical
Gym6 except already agreed return destinations. Refresh changed attributes,
map blocks and global script/NPC split mirrors through the existing native
insertion helpers; the Gym7 mirror defect demonstrates why cached map modules
alone are insufficient. Force real split reassembly in an asset test.

Required production-controller cases, using only `VisibleShellController::press`
after labeled initial fixtures:

- Each of eight lane crossings in both directions lands on the exact opposite
  edge tile and walks to its public anchor; prove real map-name and tile changes.
  Test R15’s clipped extent, all three nonzero offset pairs and the west zero-offset seam with the native
  resolver, not a screenshot or a coordinate log alone.
- Missing Badge5 denies R06; earned first five allow it. Missing Badge6 denies
  each of R07/R15/R18 while Badge5 alone still permits R06. First six allow all
  four. All other flags, items and the absence of HM_SURF leave this dry-road
  eligibility unchanged. No test may set a permission flag during gameplay.
- Both lanes, inner and outer approaches, all four facings, guard A interaction,
  transverse plane arrivals, reload and save/Continue must obey the same policy.
  Text rejects movement while it owns input; refusals and escorts spend no money,
  alter no party/Bag and grant no milestone. Denied players can retreat normally.
- With both bodies blocking, a flood over walking plus conservative Surf reach
  cannot connect inner public landings to either outer seam lane. Demonstrate
  actual denied side/Surf attempts as well; verify no off-map lake exit.
- A labeled local fixture must defeat actual Gym6 trainers/leader, spend selected
  move PP, earn Badge6 and then open the Badge6 seams through input; keep the
  ranger’s success-only Surf gift and repeat/Continue proof. This is local earned
  eligibility, not an earned Winona-to-Brainerd regional journey claim.
- Render and inspect whole city, all four adapted whole routes, matching native
  160×144 seam/checkpoint cameras in closed/open states, and both neighboring
  views after each actual crossing. Retain all originals, crops, conversions and
  rejected renders. Existing sparse-junction visual concerns remain separate.

Until these changes and proofs pass, status stays **planned four-seam contract**.
