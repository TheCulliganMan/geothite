# Route grass pass v01

> **Superseded pack lineage (2026-10-28):** The grass packs described below
> inherited rejected oversized/winter North Shore geometry. They are historical
> evidence, not accepted current game packs. All eleven packs were removed from
> `content-packs/` into `.local/route-grass-review/rejected-stale-coast-packs/`
> with a SHA-256 manifest. Preserve their authoring deltas and render originals;
> rebase terrain deltas onto the reviewed compact current descendant. North Shore
> must remain ice-free. All new visual review is 2D only. The old history below
> is superseded by the verified current-parent work linked next.

> Latest accepted terrain revision: [Mille Lacs](current-mille-lacs-v01/README.md). Nine grass cells strengthen the lookout climb3/dryreturn0, north-shore walk2 and picnic walk4. Five native cases, current Fly discovery, Surf-only reward, Continue,15clean stdio, audio, exact2D and preservation pass. The selected v02 pack descends from starter-return; all729 other maps remain exact. Eight other route/link/park groups, relevant town links and final current-story coverage remain open. Read world.json for the current selected pack.

> Previous accepted terrain revision: [Ely Shore Pass](current-ely-shore-pass-v01/README.md). Two green grass cells raise required crossings from one to five steps each way. Existing inland snow cells and the border remain exact. Four native cases, clinic/care/PC/Continue, 15 clean stdio calls, audio, exact 2D and preservation pass. Nine other route/link/park groups, relevant town links and final current-story coverage remain open. Read world.json for the selected pack.

> Previous outdoor review: [Staples Rail Junction](current-staples-junction-review-v01/README.md) is accepted unchanged. Its R15 approach already crosses three required grass steps; the small depot yard remains dry. Three native cases, 13 clean stdio calls and two exact native 2D views pass. The selected pack remains the Detroit Approach child. Ten other route/link/park groups, relevant town links and final current-story coverage remain open.

> Previous accepted terrain revision: [Detroit Lakes Approach](current-detroit-approach-v01/README.md). Seven grass cells, a native ledge and dry landing create a three-step grassy terrace climb and dry return. Native/Continue/stdio/audio/2D/preservation checks pass. It preserves all729 other maps. Eleven other route/link/park groups, relevant town links and final current-story coverage remain open. Read world.json for the current selected pack.

> Previous accepted outdoor revision: [Bemidji Northwoods](current-bemidji-northwoods-v01/README.md). Six grass cells and a native ledge give a five-step grassy uphill loop and dry return. It descends from the current warm-Cedar Castorock pack and preserves all729 other maps. Read world.json for the current selected pack. Twelve other route/link/park groups, relevant town links and final current-story coverage remain open.

Objective: review Minnesota's numbered routes and outdoor regional links; bring
encounter habitat onto useful walking paths with original Crystal pacing.
This is an in-progress statewide pass, not regional acceptance.

Accepted current-parent terrain work is recorded in
[R01](compact-rebase-v01/R01.md), [R02](compact-rebase-v01/R02.md),
[R03](compact-rebase-v01/R03.md), [R04](compact-rebase-v01/R04.md) and
[R05](compact-rebase-v01/R05.md). Their execution logs retain their recorded
identities. The704-map story descendant preserves those accepted terrain edits.
[R06](current-r06-audit-v01/README.md) is now freshly reviewed on that story
pack: the21-cell grass/ledge pass was already present, all1408native cells and
1,441,792source/native pixels match,7/0northern-band behavior holds before/after
Cut and without actor bodies,5native cases and12clean stdio calls pass. No new
R06terrain or selector change was needed. North Shore remains ice-free; all
new review is2D only. Read world.json for the latest accepted story/cave child.
[R07](current-r07-audit-v01/README.md) also passes: existing grass retained,
8/0meadow crossings, 3native cases on the current R03 cave child. Source/2D/stdio
receipts keep their original story binding with exact R07 equivalence proved.
[R08](current-r08-audit-v01/README.md) passes on that same current child: both
local grass climbs4/0, current mire retained,4native cases and15clean stdio calls.
No R08terrain or selector change was needed.
[R09](current-r09-v01/README.md) now adds8grass cells and4spoil boulders on the
current704-map story/R03 child, selected as route_grass_r09_revision: quarry4/0,
5native cases,13clean stdio calls, exact preservation of702other maps.
[R10](current-r10-v01/README.md) is selected on the705-map game, retaining the
Mille Lacs park payload:7grass/5rocks,4/0carrier circuit,6native cases and13clean
stdio calls. The accepted park parent and all earlier audits remain intact. R11-R20, outdoor
regional links and final current-story coverage remain open.
[R11](current-r11-v01/README.md) is now selected:8grass/1boulder, warm coast
retained, Birch3/0andLester4/0named climbs/returns,9native cases,15clean stdio
calls and759preservation checks. R12-R20, outdoor regional links and final
current-story coverage remain open. R12publication is held until the disjoint
reward successor is integrated onto R11.


## Authored so far

- R01 Driftless Trail: four patches at the Rochester woodland crossing
  (4–6,12), creek trail approach (11–13,13–15), eastern lowland bend
  (23–25,15), and limestone foot (12–13,9–10). Coordinates are metatiles.
  Existing trainer at core (25,29) retains its dry lawn metatile (12,14).
  Tall grass increases from 17 to 32 metatiles (68 to 128 walking tiles).
- R02 Prairie Farm Road: golden grass at the western field neck (7–9,15),
  west bank approach (10–11,12–13), east bank field edge (22–24,13), and
  Rochester farm bend (26,9–10). Grass increases from 43 to 55 metatiles
  (172 to 220 walking tiles). Existing gold source pixels/palette are reused.
  The bridge deck and farm entrances retain their existing terrain.

The explicit remaps change only ordinary floor/lawn into native encounter grass.
They preserve trees, water, rocks, ledges, doors, flowers, actors and signs.
The before layouts in `before/` are retained inputs for the Rust publisher.

## Publication and evidence

Base: ignored `content-packs/minnesota-shore-gate-v02.crystalpack`.
Combined review pack: ignored
`content-packs/minnesota-route-grass-r01-r02-v01.crystalpack`.
Published sequentially with existing Rust `revise_authored_route`.

The current R01 dossier has stale cave rows: the installed cave has later wall
and stair changes. Publication used a local dossier copy with the installed room
rows, validated by the existing native geometry checks. This did not change any
room in the pack. Local inputs are
`.local/route-grass-review/MN_R01.installed-dossier.json` and the equivalent R02
file. The independently executed comparison checks the entire before/after pack,
allowing only the exact exterior terrain delta from the archived source layouts.

R01's exact preservation comparison and adversarial checks passed. The production
controller test `driftless_controller_walks_ramp_ledge_dock_dialogue_pickups_and_both_rooms`
passed on the R01 pack (1 passed). The cached production test executable required
`RUST_MIN_STACK=33554432`; its first default-stack attempt overflowed.
This fixture has no party and proves traversal/events, not wild encounters.

Native 2D R01 capture at core (11,24), noon, was inspected against source art and
original Route29: `planning/minnesota/renders/route-grass-v01/r01-west-native-v01.png`.
The full source/native before/after renders and immutable comparison reports live
under `planning/minnesota/renders/route-grass-v01/` and are retained.

R02's populated-party production test
`prairie_farm_journey_crosses_grass_bank_bridge_and_wins_all_three_with_saved_pp`
passed on the combined pack: four wild encounters escaped, all three trainers
and five partners defeated through native input, and PP, party and flags survived
Continue. The native 2D view at core (21,25), noon, was inspected against source:
`planning/minnesota/renders/route-grass-v01/r02-bank-native-v01.png`.

The initial combined comparison proved the exact pack delta and identities, but
its source-render/recipe checks failed because the base Kanto sheet already
contained appended blocks. The reviewer now rebuilds explicit recipes from the
original source prefix instead of appending them twice. This retains independent
source/native pixel and collision comparisons. The combined full comparison and existing adversarial checks passed, including
both pack identities, all source/native recipe pixels and collision, and all
fields outside the exact declared exterior terrain delta. The first failure
report is retained. Successful immutable report:
`planning/minnesota/renders/route-grass-v01/da8243273e17-to-f5451f93f667/comparison-326ddcbd1afb762af0c71158cbf43b2f608108edc3045ec16485d512a7b6c953.json`.
The tool retained 34 render outputs. Python compilation and scoped whitespace
checks also passed.

## Still required

- Complete R11–R20 one at a time, with Crystal-reference grass and ledges and actual
  source/native visual review. Retain each route's identity and original gates.
- Re-audit R01/R02 against the new grass-one-way / dry-ledge-return guidance.
- Review outdoor regional links, river walking banks, and Safari/portage habitat;
  towns need encounter grass only where meaningful gameplay calls for it.
- Reconcile winter source/installed art versions for R12/R13 before publication.
  The first atlas attempt reached R12 and stopped on unavailable compiled art;
  incomplete atlas renders are not regionwide evidence.
- Integrate accepted terrain into the selected current Minnesota game pack,
  preserving concurrent regional roster/story work, before statewide acceptance.
- Verify the resulting whole pack with production traversal/encounters and
  persistence, preserve unrelated content, and retain all renders.

No shared/default game pack has been replaced. This review pack contains only
this pass's published terrain changes relative to the selected base.

## Current route

R03 is accepted in `minnesota-route-grass-r03-ledges-v01.crystalpack` (identity
prefix `afa3251e09d5`), following the user clarification
to work one at a time and use Crystal routes as references. Its directional
contract is grass northbound, a dry ledge return southbound. See [R03.md](R03.md).

The directional proof, exact pack comparison, native captures, geometry controller
and populated journey passed. Review fixture setup and remaining limitations are
recorded in the route runbook. No shared/default pack has been replaced.


R04 is accepted in `minnesota-route-grass-r04-v01.crystalpack`, identity prefix
`62c8591aa01b`. Its National Park / Route35 reference pass adds eleven native
grass metatiles to walking loops while retaining the civic park choice and dry
trainer lane. Both geometry and the populated five-encounter/two-trainer journey
passed, including medicine, Ether and Save/Continue. See [R04.md](R04.md).


R05 is accepted in `minnesota-route-grass-r05-v01.crystalpack`, identity prefix
`bb3be3720b57`. Thirty-three grass metatiles connect habitat to the two outdoor
climbs; existing southbound ledges keep dry returns. The cave shortcut, river
ports and all other content remain exact. Native geometry, five-trainer journey,
Surf fixtures, audio and Continue passed. See [R05.md](R05.md). R06 is next.


R06 is accepted in `minnesota-route-grass-r06-v01.crystalpack`, identity prefix
`4e1d845335f6`. Nineteen new grass metatiles and one new northern ledge create a
seven-grass-step northbound loop and dry southbound return. Native tests passed
both hop lanes, the five-trainer journey, Cut and Surf, audio and Continue.
The original optional lookout remains Cut-gated. See [R06.md](R06.md). R07 is next.


R07 is accepted in `minnesota-route-grass-r07-v01.crystalpack`, identity prefix
`73eee2da63ba`. Nineteen new grass metatiles connect meadow/woodland habitat to
walking paths. The expanded outward meadow requires eight grass steps; the
existing sandbank keeps a dry return. Eight native trainer wins, Cut/Surf,
audio, pickups and Continue passed. Canonical T01 naming remains untouched;
exact native proof uses the installed lineage. See [R07.md](R07.md). R08 is next.


R08 is accepted in `minnesota-route-grass-r08-v01.crystalpack`, identity prefix
`75f7b910a3f7`. Twelve additional grass metatiles connect its trail necks and
local climbs. Birch and Cedar each have a four-grass-step climb and dry hop;
the whole-lake alternative and dry Cedar shore path remain available. Native
ledge lanes, Cut, populated two-trainer journey, pickups, Continue, stdio and
audio passed. See [R08.md](R08.md). R09 is next. Shared game integration is open.


R09 is accepted in `minnesota-route-grass-r09-v01.crystalpack`, identity prefix
`4c5fbe211a9f`. Eight new grass metatiles reclaim useful quarry paths; four
attached boulders close cliff-foot bypasses. The shelf circuit requires four
grass steps outward and keeps a dry original ledge/ore-track return. Native
geometry, five wild encounters, Cut, trainer/PP, healing, Continue, stdio and
audio passed. See [R09.md](R09.md). R10 is next; shared game integration is open.


R10 is accepted in `minnesota-route-grass-r10-v01.crystalpack`, identity prefix
`3d7362f74a63`. Seven new grass metatiles join useful paths; five attached rock
shoulders seal native cliff-foot bypasses. The upper carrier circuit requires
four grass steps and its original ledge/lower road keep a dry return. Native
wild encounters, stair, mine, Surf, Calder/PP, Continue, stdio and audio passed.
See [R10.md](R10.md). R11 is next; shared current-game integration remains open.
