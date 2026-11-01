# City inventory and frozen native candidate audit

Read-only audit, 2026-10-26. Shared plan, sources, canonical gyms and controller
code were not changed. Root owns the pending combined pack publication/build.
The exact inspected live bytes, hashes and three in-memory checker probes are
retained in `.local/city-inventory-audit-deb9b9580471/`. Shared files are being
edited externally; this report describes that snapshot, not later live writes.

`python3 planning/minnesota/tools/check_plan.py` passes:20 routes,19 hubs,52
chunks,78 planned adjacency/gate contracts,152 planned maps. This proves the
checker’s stated inventory/symbolic scope, not native collision or gameplay.

## Findings

1. **P1 — live/frozen source generation is mixed.** Current Brainerd Gym6
   descriptor has dimensions12×11 but still the frozen4f41 object hash,
   arrival10,17 and carpets10,19/11,19. The frozen Gym6 used by the five passing
   native asset cases is10×10. The live canonical object is12×11 with
   arrival12,19 and carpets12,21/13,21. Re-signing only a hash or copying only
   dimensions cannot produce a coherent candidate. Root is deliberately publishing
   from the complete historical frozen town/layout/canonical input bundle below.
   Do not restore or overwrite the externally edited live source.
2. **P1 — St. Paul/St. Cloud gym references contain stale arrival contracts.**
   St. Paul declares11×8 but arrival12,21 and carpets12,23/13,23; all their y
   coordinates exceed room height16. Both frozen and inspected live canonical
   Gym4 require arrival10,13 and carpets10,15/11,15. St. Cloud declares7×12 but
   arrival12,21 and carpets12,23/13,23; those lie on the canonical eastern WALL
   border. Both canonical versions require6,21 and6,23/7,23. Their reference
   object hashes also match neither inspected live nor the common frozen file.
   Those cities need a complete canonical-reference reconciliation before native
   integration; their inventory matching itself cannot establish safe arrivals.
3. **P1 — global Gym6/Gym7 contracts remain reservations.** They lack the
   registered exterior DOOR/passive/index fields and second carpet in
   `interiors.json`; Gym6 still lists arrival12,19 and first return12,21.
   `world.json` has no `authoring_file` for these two rooms, so the checker
   intentionally skips their canonical validation. It does not validate every
   room extent or warp pair. The current Brainerd native builder independently
   rejects a noncanonical room and preserves the installed canonical module.
4. **P1 visual — source paper stations are not native-installed objects.**
   Frozen Gym6 lists PINE5,11, REED12,11 and CHART8,13 with SPRITE_PAPER.
   `gym_scripts::install` installs three BGEVENT_READ entries, with no station
   `npc()` or source sprite consumption. `gym_art::install` copies the exact
   declared native metatiles; CHART lies on ordinary tower09 timber FLOOR.
   The reviewed source preview manually overlays paper sprites and excludes
   their bodies from its flood fill. Native debug event markers are optional
   debug overlays (`menu_rendering.rs`), not ordinary source art. Therefore the
   preview’s visible papers and blocking bodies are not evidence for native
   station appearance/collision. Static canonical-art flood comparison confirms
   the difference: preview seven bodies gives102 closed/159 open tiles; native
   installer four-body roster gives105/162. This is source-collision evidence,
   not controller evidence. The comparison is retained in the audit snapshot.
   Production native pixels have not been seen in this audit. A native capture and readable marker/collision design are still
   required; an interaction test alone does not resolve discoverability.
5. **P2 — inventory checks deliberately stop short of semantic checks.** All
   three invalid in-memory fixtures were accepted without touching disk: global
   skipped Gym6 arrival999,999; clinic exit target changed to MnDuluth/warp99;
   outdoor ranger prerequisite changed to BADGE_1. New nonGym checks compare
   source coordinates, inventories and extents, but not each source exit’s target/
   target-warp or local actor story gates. Add narrowly scoped checks where those
   semantics become part of the promised plan verification; keep canonical native
   verification in its actual builder/reviewer. Do not label symbolic PASS as
   all paired native doors/services/gates passing.
6. **P2 — service descriptions need implementation qualifiers.** The shared
   `source_behavior` string says healing, PC and blackout recovery for Brainerd,
   while its builder deliberately preserves the old recovery table. Healing/PC
   implementation exists, but controller cases and new clinic blackout recovery
   remain pending. The global inventory is a plan and must retain that distinction.

## Six-city matrix

All six exterior extents and ports match `world.json`. All32 facade targets
are represented in the world room roster; no missing visible-building room was
found. Station references were resolved through their separate transit source,
not mistaken for missing town-local room rows. Registered nonGym door/return
coordinates and indices match the source inventories. Their real collision and
native behavior remain the responsibility of source/native/controller checks.

| City | Exterior | Facades / world rooms | Canonical and arrival result |
|---|---:|---:|---|
| St. Paul |32×24|5/5|Gym4 stale/outside bounds; depot separate16×12 source uses public arrival15,21 and four carpets14–17,23, passive5.|
| St. Cloud |24×18|4/4|Gym5 stale eastern-border arrival/exits; separate10×10 depot arrival11,17 and carpets10,19/11,19 pair to city passive8.|
| Brainerd |32×24|5/5|Live mixed Gym6 descriptor; correct frozen10×10 bundle retained. Global Gym6 reservation not synchronized.|
| Bemidji |24×24|6/6|Town Gym7 geometry/hash matches frozen11×10; live canonical object hash changed. Global contract omits full door/carpet binding.|
| Duluth |32×24|7/7|Town Gym8 geometry/hash matches frozen12×12; live canonical object hash changed. Complete global binding is present.|
| Detroit Lakes |24×18|5/5|No gym. Separate12×10 depot arrival11,17 and carpets10,19/11,19 pair to city passive2; rail service remains planned.|

The two actual local story actors remain consistent across town source,
`actors.json` and story milestones: ranger25,19 requires BADGE_6 and grants
HM_SURF/report success-only; Bemidji officer21,19 requires BADGE_7 plus
EVENT_MN_HEADWATERS_READING and sets EVENT_MN_HEADWATERS_REPORT. Their presence
and matching metadata do not prove earned travel or a controller transaction.

## Canonical versions and candidate paths

Common frozen eight-gym source:
`.local/minnesota-city-source-freeze/gyms-b3f25b09e4ce114fd561ff28309464fb9e45500bb159dcb6d145ce2f06f7bbbb.json`.
The inspected live canonical file hash is
`887afc43d19bc0573ca5ec361f4c141771347421040f70f100f02fea4997a189`;
exact bytes are in the audit snapshot. All five inspected live Gym4–8 object
hashes differ from that frozen file. Geometry changes are concentrated in Gym6;
matching dimensions on other gyms do not establish matching scripts/actors/party.

| Gym | Frozen object SHA256 | Inspected live object SHA256 |
|---|---|---|
|4|37f423607fb2799e42469c5ef73bd909c9db1135b474e6bcbfd1ee942c7ba093|d37fa3ab34ac0b4e46882d220af02a7d125efc1c90fd68d254898f02a0cc92d2|
|5|8bbc7e3f30ce59d4270b6681bf4e150a8792d7491e9f5607a9cc3c6dbdd54130|613e0d5bf0b7a176b55ae5623e5420b4aab2e25bb169b48124625d48b31b8315|
|6|4f41dfd5f12153ce79067ecb6adb7d0b9b0644d99bfa762e38de45505d787187|e2194a0521626c3f8fc786d49bcdcdfbd641b83f4c9c49ec010816a4c6f6dd53|
|7|73bb00cfbc3ed74c5b9a82d26d52f7d9a9cae895be830f2294e938350316376c|4ea1bea34bc4a614f2d5f91b248cc0cf06439395d2f2a9ec2020defab2e3b463|
|8|c809cf8ffa6763fa78b65311e360ebf9c144c323770ddee02d4fb4fd533c1aa1|c1554f9d93d56117086ad978f6150c923bb877c336785e6e3ff30b07de9360ee|

Correct Brainerd frozen town/layout and all five passing asset-test evidence:
[handoff manifest](../renders/brainerd/native-handoff/manifest.json).
Exact town source ends `MnBrainerd-575ffd6aef85fb633ad76de6d77cec12aedf468b5d6ce67904f0fb649eb4ee09.json`;
layout ends `MnBrainerd.layout-2af837ce974c8c9c34e1de98153a090ec234f425e62c1900526c992591f577a9.json`,
both under `renders/brainerd/native-asset-input-captures/`. Frozen source preview
is `renders/brainerd/807c952bd026/`. Controller source is held stable pending
Root’s real build/run diagnostics; no new test PASS is claimed here.

## Brainerd visual review and travel limits

Personally inspected the entire frozen town, all ten source player-camera
panels (including the automatically chosen WIDEST_FLOOR at29,19), and exact
closed/open Gym6 plus its player views. Original/cropped/converted/rejected
renders remain saved. The lake has a closed boundary, actual pier deck with end/
side rails, connected road network and a readable southern ramp/ledge. The gym
terrace and pine-camp pocket have recognizable destinations and framing.

The strongest remaining composition weakness is broad blank paving around
clinic-porches, lake-launch and four-way-decision. WIDEST_FLOOR shows a large
empty junction beside a pine clump, with neither destination nor meaningful
choice visible in its central camera. Signs alone do not fill that activity gap.
Before visual acceptance, narrow unused paving or place a coherent activity with
an actual interaction/route choice; keep all four reserved ports and functional
DOOR/ledge/ramp contracts. No redesign was made during this read-only audit.

All four Brainerd regional edges remain planned `gatehouse_warp` contracts with
unallocated warp-index status. Native city code intentionally leaves them
unbound; the source’s open boundary roads do not prove reciprocal travel, badge
gates or safe off-map behavior. Bemidji/Duluth regional edges have the same
planned/native distinction. The checker’s symbolic champion/hub reachability
explicitly does not execute gameplay. No Brainerd train stop was invented;
Glacier Line stations and fairgrounds reservations remain separate planned scope.


## Follow-up: checker repair and new integration epoch

Root repaired the nonGym unsafe carpet-target/index omission after this snapshot.
The same in-memory clinic mutation now fails explicitly with
`MnBrainerdPokecenter1F: unsafe carpet destination/index`; no shared files were
mutated to prove this. The original finding and saved probes remain historical
evidence. This repair does not make actor gating or canonical-room checks part of
that inventory checker.

Root’s next integration uses the exact887 canonical source from the matrix above,
not b3. A separate immutable Brainerd candidate aligns only canonical hash, extent,
passive arrival and carpet coordinates; it preserves all exterior/nonGym content
and every source field. Its town SHA256 is
`a6e99c25936262a775e76ddb29721e308ca140c37ef6dc36bf4eeb9fbea6c7fa`.
See [887 handoff](../renders/brainerd/native-handoff-887/manifest.json).
New Gym6 has no READ stations or paper actors: NESSA and BRAM wins directly open
the upper gangway. Thus the b3 paper-marker mismatch is historical for this new
Gym6 candidate. Preview5afb9fd303ae was inspected as source art, not native pixels;
new native/controller execution remains pending. Live city/world drafts were not
overwritten.
