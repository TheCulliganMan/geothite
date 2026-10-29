> Current grass review: [R19 verification](../revisions/route-grass-v01/current-r19-v01/README.md).
> Four grass metatiles turn the pine approach into seven habitat steps each way.
> Existing dry ledge drop, three-step grassy uphill return, old ramps and eight-step
> battle/grass choice remain. Six native cases, 15 stdio, audio and actual 2D pass.
> Current native Minnesota teams and encounters supersede historical source below.

# Prairie Forest Link — authored ecotone handoff

R19 keeps the **east Lake Itasca → south Detroit Lakes** bend. Its **32×30 metatile** source composition moves from dense pines through native round oak openings and a small spring wetland to prairie habitat. The decisions are a short fenced trainer road versus a measured grass detour, an optional two-ramp inspection spur, a grass-protected meadow reward, and three dry-land observations. It replaces the old 96×96 repeated reserve.

Authoritative source: [dossier](MN_R19.json), [terrain strokes](MN_R19.layout.json), [fast review/check tool](../tools/review_prairie_forest_link.py). No source packs or shared Rust registrations were edited by this author. The SQ01 script arrays and Headbutt binding below are explicit implementation contracts; they are not running native behavior yet.

## Visual review and fast loop

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_prairie_forest_link.py --self-test
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_prairie_forest_link.py --watch --serve --port 8785
```

The page is `http://127.0.0.1:8785/`. It reads the supplied ignored TUI pack. A warm full source render/check is roughly 155–185 ms; initial decoding is separate. JSON changes immediately rerender only this route. Generated images and verification output remain ignored under `renders/prairie-forest-link/`.

Reviewed complete render: [whole-route-93a61f7d.png](../renders/prairie-forest-link/whole-route-93a61f7d.png), 1024×960 original source pixels. Review [all player frames](../renders/prairie-forest-link/viewpoints.png), [the actual emptiest-ground frame](../renders/prairie-forest-link/viewport-emptiest_ground.png), [the routine oak bend](../renders/prairie-forest-link/viewport-widest_routine_bend.png), [wetland Surf probe](../renders/prairie-forest-link/surf-boundaries.png), and [anchor overlay](../renders/prairie-forest-link/anchors.png). Individual cameras are160×144, enlarged nearest-neighbor.

The old entire blockout was viewed before design. Native Route29/Route42 and actual Johto tree recipes were inspected; no generic illustration supplied geometry. The first pass had an empty green center and overly broad upper spur. Connected native oak fingers now bound the routine through passage at four actual walking tiles. The upper spur was reshaped with real cliff cut-ins. An automatically detected blank junction shoulder was closed with oak; the direction board was moved from the redundant arrival cluster to the actual road/grass fork. The lower approach was narrowed, and the prairie walker now occupies the useful field junction. The largest remaining blank ground square is3 metatiles; its real camera was inspected and contains the observation approach, meadow habitat and nearby resident. This measurement supports review; it is not a visual-quality certificate.

## World and native builder contract

| Native map | Metatiles | Role |
| --- | --- | --- |
| `MnR19C00R00` |32×24| Pine arrival, oak choice, inspection spur, wetland and observations |
| `MnR19C00R01` |32×6| Prairie habitat seam and Detroit approach |

Retain route `MN_R19`, prefix `MnR19`, **BADGE_6**, **SQ01**, and town identities `MnLakeItasca` / `MnDetroitLakes`. Root-reserved publication contract: group59, `MINNESOTA_PRAIRIE_FOREST_LINK`, original **johto** tileset, original Route29 music. Root owns group availability, registration, hub guards, world/story updates and external-pack publication. No derived art is needed. Preserve original maps/assets/audio.

| End | Gate metatile | Clear landing | Core gate / landing |
| --- | --- | --- | --- |
| LAKE ITASCA east |(30,6)|(28,6)|(60,12) / (56,12)|
| DETROIT LAKES south |(12,28)|(12,26)|(24,56) / (24,52)|

Bind these to the existing Itasca west and Detroit north hub ports, using clear passive FLOOR arrivals. They are warp-port contracts rather than invented outer open walking seams. The two authored route chunks need reciprocal north/south connections at offset0. The seam is core y47/48; native grass continues through the southern habitat there. No water touches that internal seam. Root must retire the ten unused old chunks; trainer/event reservations07–10 remain unused, not reassigned. No interior is reserved or required.

All geometry is explicit schema2 orthogonal strokes with `reject_cliff_cuts:true`. Main road:
`(30,6) → (24,6) → (24,11) → (17,11) → (17,18) → (12,18) → (12,28)`.

The main paved lane is normally two core tiles. Native canopy and cliff edges shape its usable shoulders. `shortcut:null`, `cut_gates:[]`, `one_way_ledges:[]` are intentional: the original spur brief requests steps rather than cliff jumps, and no useful sealed Cut target is needed here. Actual source CUT_TREE inventory is zero, including variants that can hide Cut quadrants.

## Exact traversal contracts

**Fenced road versus grass.** Merrit stands at core **(43,21)**, south-facing, sight3. The actor and actual native sight cells **(43,22)/(43,23)/(43,24)** cover all four rows between the upper/lower rails. Core approach(49,23) and exit(39,23) have a zero-grass ordinary road when trainer sight is not forbidden. Avoiding those sight cells costs **eight TALL_GRASS steps in either direction**, in habitat rectangle **[40,26,48,30]**. Native forest above the lane and the wetland/oak closure below prevent a free dry bypass. The complete travel still has a nonbattle grass option before trainers are defeated. Source checks use the actual facing/range rather than merely labeling a trainer as mandatory.

**Oak opening.** Real Johto round-canopy recipes61/5d and individual3c/3d/62 contrast with the eastern pine border05. These are native HEADBUTT_TREE collision, not fake enlarged trees or incidental Cut. At the routine bend, core x32..35 on y33 is the four-tile walking passage; x31 and36 are real HEADBUTT_TREE edges. The oak reader and **Max Repel** sit on a short western branch; the separate Ultra Ball remains at the pine arrival.

**Western inspection spur.** Native74 ramps at **meta(5,18)/(9,18)** support ordinary ascent and descent. Each is `[WALL,FLOOR,FLOOR,FLOOR]`. Lower walking approaches are core(11,38)/(19,38), upper approaches(11,35)/(19,35). The heads connect by a **fourteen-step upper path**, with the native cliff bounds kept intact. No HOP_DOWN cell substitutes for a step. Hyper Potion, Hiker Dane, the meadow approach and different lower return justify visiting the loop; a long empty detour is not the reward. The second lower approach is grass, which remains legal ordinary walking.

**Wetland.** The pond has two lobes below the oak edge. The dry western bank reaches the observation sign and Fisher Lorne. A real launch is core **(43,33) FLOOR → (43,34) WATER**; native46 protects the nearby west bank at meta(21,18)/(21,19). It is a local wetland, not another huge lake or marina. All160 reachable water cells remain inside the route. The complete244-cell outer perimeter contains zero WATER, so no buoy artwork is required at an unconnected boundary. Actual Surf launch/dismount and railing rejection remain controller checks.

**Prairie.** Flowers, meadow habitat and round-tree fragments gradually replace the pine passage. The Revive at(33,41) requires **two actual grass steps** to reach an interaction approach in meadow rectangle[30,38,36,44]. A full source-floor footing is preserved under the item. A separate shared southern habitat **[22,48,28,52]** forces **four grass steps** on every ordinary Itasca–Detroit trip. No side branch goes around that tree barrier, and the optional western spur never bypasses it.

## Exact actor and resource inventory

All coordinates are native16px core tiles. Dossier dialogue is original, eighteen columns maximum. Six active trainer IDs/defeat flags01–06 keep their original three-member reserved parties; eighteen opponents total. Reservations07–10 are unused.

| Trainer | Tile / facing / sight | Class and party |
| --- | --- | --- |
| `MN_R19_T01` MERRIT |(43,21) south3| PICNICKER: Pineco28, Skiploom28, Wooper27 |
| `MN_R19_T02` AVEN |(57,13) west1| BIRD_KEEPER: Skiploom28, Stantler28, Pidgeotto27 |
| `MN_R19_T03` ELIN |(33,29) north1| POKEFANF: Stantler29, Wooper29, Girafarig28 |
| `MN_R19_T04` DANE |(19,31) east1| HIKER: Wooper29, Pidgeotto29, Heracross28 |
| `MN_R19_T05` TESS |(11,41) east1| PICNICKER: Pidgeotto30, Girafarig30, Pineco29 |
| `MN_R19_T06` LORNE |(39,37) east1| FISHER: Pineco30, Heracross30, Skiploom29 |

Resident `MN_R19_OAK_READER` is(27,29), east-facing; `MN_R19_PRAIRIE_WALKER` is(27,39), west-facing. They give local context and do not complete SQ01 or Q05. Initial sight does not seal all through-travel; Merrit's required cost has the independent grass alternative.

| Pickup / original persistent flag suffix | Tile | Reward |
| --- | --- | --- |
| `MN_R19_ITEM_01` / `EVENT_MN_R19_ITEM_01` |(11,29)| Hyper Potion×1, upper spur |
|02|(53,13)| Ultra Ball×1, pine turnout |
|03|(29,29)| Max Repel×1, oak branch |
|04|(33,41)| Revive×1, meadow pocket |

All four are walking-access and independent of the eventual quest Max Repel. No item requires Surf. Source encounter rates and seven grass slots retain the original dossier: Pineco/Skiploom/Stantler/Wooper/Pidgeotto/Girafarig/Heracross, levels28–34; night changes retain Noctowl/Ariados. Existing three water slots and rate4 remain.

Signs: fork/directions **(36,20)**; pine observation **(54,14)**; wetland observation **(40,34)**; prairie observation **(22,44)**; ramp directions **(14,36)**; Detroit directions **(26,52)**. Each source sign quadrant is native WALL with reachable same-chunk walking adjacency. Every observation is independent of Surf, Cut and battles.

## SQ01 implementation handoff — pending

Preserve **Three lake observations**, BADGE_6, completion **EVENT_MN_LAKE_OBSERVATIONS_REWARDED**, exactly one MAX_REPEL, and no Q05 prerequisite. Three new observation events are proposed: **EVENT_MN_LAKE_OBSERVATION_01/02/03**. Root must allocate them through the existing story-constant/pack-mutation path; prose or an unparsed JSON property does not create a runtime flag.

The three real R19 signs mark pine, wetland and prairie observations on the Itasca/Detroit loop. Exact native script-label arrays in `quest_contract.script_labels` replace the generic text-only sign labels: `opentext`, `writetext SignText`, `waitbutton`, `closetext`, `setevent OwnObservation`, `end`. Reading sets only that sign's event after acknowledgment. Repeated reads are idempotent. Root must rerun the production runtime-command parser after attaching them.

The **planned Detroit Lakes ranger** owns reward delivery, not a new R19 NPC. Current `MnDetroitLakes.json` has no authored terrain layout/resident anchor, so a native placed ranger does not yet exist. Proposed identity `MN_DETROIT_LAKE_OBSERVATION_RANGER` and source commands are in the dossier, but root must supply a real town FLOOR anchor and interaction approach when authoring that town. This external physical dependency is unresolved, not disguised by an invented coordinate.

The ranger script checks native `ENGINE_MINERALBADGE` (existing source constant31 / Johto index5), then completion, then each observation event. Missing badge/notes never grant or complete. Otherwise text introduces the gift, `giveitem MAX_REPEL,1` must succeed, and only then `setevent EVENT_MN_LAKE_OBSERVATIONS_REWARDED`. Full-bag failure retains retry; already complete returns acknowledgment without another item. All separate native dialogue labels are included and fit eighteen columns. Existing consumable Max Repel inventory does not suppress the legitimate first quest gift.

Root should attach town/reward labels in a focused native leaf module or a typed quest-authoring hook, then reparse runtime/object/field-pickup commands and validate. Test all eight note subsets, independent acknowledgments, missing badge, first success, full bag/retry, repeated talks, save/reload, and dialogue input ownership through VisibleShellController. These are production requirements, not passed source checks. Town/route travel and Q05 remain available independently.

## Optional native Headbutt activity — pending

The placed round oak collision supports a real existing field action. `headbutt_contract` proposes binding **MnR19C00R00** to a cloned per-map **Route29 headbutt table**, overriding only levels to28 while preserving its common/rare weights and time-of-day sleep rules. The actual supplied Route29 table was inspected; it includes Pineco in the rare pool. Do not export another source catalog or mutate Route29 itself.

Use the existing typed `field_encounters` per-map table and native HEADBUTT resolver. Generic authored-route publication currently does not assign this table. Without that binding, the source oak art/collision alone does not promise encounters. Controller tests should prove move availability/denial, actual tree targeting and field-encounter origin/RNG selection, with source weight/time rules preserved. No custom interaction dispatcher is needed.

## Verification status

Source checks pass with33 native metatile varieties,881 connected ordinary/grass core cells,112 accessible grass cells,160 reachable Surf-water cells and no outer WATER. The battle alternative costs8 in both directions; shared prairie costs4; the optional Revive costs2. Both native ramps and their fourteen-step upper connection, three walking observation signs, all ordinary actors/pickups and all declared path joins pass. All source trees have been inventoried for incidental Cut; none exist.

Twelve negative fixtures are rejected: wrong trainer facing/range, dry grass detour, removed southern habitat, either missing ramp, a hop replacing the step, incidental Cut canopy, observation in Surf water, unclosed outer pond edge, widened empty oak shoulder and reward moved out of its habitat. The widest/automatically emptiest cameras were separately inspected. Python compiles. Source rendering is fast and uses the actual supplied pixels/collision; no native content was published by this author.

**Native playability remains pending.** Root must register the two maps, reconcile original world port/chunk reservations, attach SQ01 and optional Headbutt data, publish with Rust, run the review tool's `--verify-published`, then test hub badge gates, bidirectional native seam travel, Merrit's sight versus eight-step habitat, both ramps/uphill/downhill, all signs/resources/battles, actual Surf/rail boundaries and quest save/reward behavior through VisibleShellController. Recheck source/native pixels in the same state/camera and preserve all unaffected maps/audio. A picture, native dump or coordinate-only move is insufficient evidence.

## Native observation and Headbutt implementation handoff

The focused leaf `crates/crystal-assets/src/prairie_forest_link_modpack.rs` now
prepares native publication through `build_prairie_forest_link_modpack` and the
shared authored builder. Source geometry, actor IDs and both clipped dimensions
remain frozen. Root owns shared registration and export: group59,
`MINNESOTA_PRAIRIE_FOREST_LINK`, manifest `minnesota-prairie-forest-link`, landmark
`LANDMARK_MN_PRAIRIE_FOREST_LINK`, label `MN_PRAIRIE_FOREST_LINK`, original Johto
art and Route29 music. The actual raster's highest native metatile is `7a`; no
new art sheet is needed. The two maps are32×24 and32×6 metatiles, with six
trainers/eighteen opponents, two residents, four pickups, six signs and no rooms.

Root calls `bind_prairie_forest_link(pack, modules, dossier_json)` after ordinary
route events/constants and before native module publication. The hook validates
SQ01's three exact observation/event pairs and replaces only their existing
background `jumptext` roots. Each original Text body stays intact:

| Sign / global tile | Safe approach | Saved event |
| --- | --- | --- |
| `MN_R19_OBSERVE_PINE` `(54,14)` | `(55,14)` | `EVENT_MN_LAKE_OBSERVATION_01` |
| `MN_R19_OBSERVE_WETLAND` `(40,34)` | `(41,34)` | `EVENT_MN_LAKE_OBSERVATION_02` |
| `MN_R19_OBSERVE_PRAIRIE` `(22,44)` | `(23,44)` | `EVENT_MN_LAKE_OBSERVATION_03` |

All three roots are in `MnR19C00R00`. Exact command order is `opentext`0,
`writetext`1, `waitbutton`2, `closetext`3, `setevent`4, `end`5. The allocator adds
only those three events, taking fresh IDs after the current EVENT namespace and
checking collisions/overflow. Ordinary sign pages and acknowledgment precede
the flag. Repeat reads only set the same event; they do not give an item.

**The Detroit Lakes hand-in remains pending.** This route installs none of the
dossier's draft ranger/reward scripts, places no fake town actor, allocates no
new `EVENT_MN_LAKE_OBSERVATIONS_REWARDED` and grants no SQ01 reward. Existing
completion metadata in a future base would be retained unchanged. Reading all
three signs records observations and cannot falsely complete the quest.

The Headbutt binding adds exactly one typed `FieldEncounterData` entry for
`MnR19C00R00`, cloning only Route29's Headbutt table and changing all common/rare
entry levels to28. Original species, weights, buckets and time-of-day sleep
rules remain exact; Route29 and every other field encounter table are unchanged.
The original native HEADBUTT move, collision rules and global field scripts are
used. The actual resolver reads the current-map table at exact source
`HeadbuttScript:4` / `TreeMonEncounter`; native startbattle8/resume9 retains
`BATTLETYPE_TREE` and validates the route's modified level in its saved origin.
There is no custom interaction or encounter dispatcher.

An explicit native-component oak probe uses target `(31,20)` from FLOOR
`(32,20)` facing left. Initial player ID4 yields native tree score0; divider
samples `[255,0,53,0]` yield chance0 and rare entry54, selecting Pineco28. This
component timing is not a promise about divider consumption during animated
controller input. The hook also validates the frozen two-ramp metadata and both
actual native `74` placements. Generic connectivity alone could miss removal
of one ramp because the other still reaches the spur.

`crates/crystal-assets/tests/prairie_forest_link.rs` has five focused suites:
source-preserving publication and exact events/parties/art/audio; native flag
mutation/idempotence and production save round-trip; exact Headbutt table data,
1,000 source-selection comparisons and actual typed tree encounter/battle-origin
components; named/schema-guarded quest, Headbutt and terrain mutations; and exact split/flattened source-certificate positives/negatives.
Component mutation calls are deliberately labeled and do not simulate dialogue
input. Root must execute these tests; pinned Rust1.99 formatting alone passes.
Production-controller ownership is separate, and native publication, gameplay,
earned SQ01 progression and final visual approval remain pending.

Frozen authoring hashes remain unchanged:

- Dossier `94312842d9155b6597cb2339e6843fe858e44dcd75024063f3df1d5cd9533d8d`.
- Layout `95d33df286cc57a615163c49b06c16bdd7934a9c93c6719a483b79688b09977e`.

The complete frozen `93a61f7d` source render was inspected again for this native
handoff. No geometry or source-art changes were made.

## Production-controller observation/Headbutt handoff — 2026-10-26

The separate controller leaf is
`crates/crystal-bevy/src/bevy_shell/tests/prairie_forest_link.rs`. It consumes the
actual external `content-packs/minnesota-prairie-forest-link.crystalpack` through
`CRYSTAL_PRAIRIE_FOREST_LINK_TEST_PACK` after root publishes it. This author
coordinated the exact native sign/event and Headbutt contracts with the native
leaf author. No source layout, native binder, shared registration or pack was
changed while preparing this fixture. Scoped rustfmt passes; compilation and
execution remain with root's warm target.

Three ignored tests are prepared:

- `prairie_forest_link_controller_walks_two_ramp_loop_reads_observations_and_continues`
  uses an empty party and ordinary production walking. It visits the east and
  south endpoint approaches, crosses the clipped y47/48 seam both ways, and
  walks the four actual shared habitat steps. Before defeating Merrit, actual
  input follows the safe route `(49,23)↔(39,23)` in both directions and counts
  exactly eight entered TallGrass cells each way; his defeat flag stays false.
  Both native `74` ramps are traversed up/down through their actual Floor
  quadrants. The upper itinerary `(11,35)→(12,35)→(12,32)→(19,32)→(19,35)`
  contains fourteen ordinary steps, is walked in both directions, and links
  back through the lower meadow. No ledge or direct position change replaces
  the ramp loop.
- The same walking test reads all three ordinary direction/ramp signs, speaks
  to both residents and collects all four real resources. Pine `(54,14)`,
  wetland `(40,34)` and prairie `(22,44)` observations then use their actual
  native background scripts. Directional input cannot move during dialogue;
  each event remains false until acknowledgment and only the corresponding
  event changes. Repeated reads leave events and bag unchanged. Production
  Save/Continue preserves the three observation events, four pickup flags,
  exact party/bag and position. A continued reread grants nothing.
- `prairie_forest_link_controller_oak_without_headbutt_does_not_dispatch_or_mutate_tree`
  places an empty party at `(32,20)` beside the genuine `HEADBUTT_TREE`
  quadrant `(31,20)`. Native Left/A must neither walk into the tree, open a
  usable Headbutt choice, begin battle, change flags nor replace any map block.
- `prairie_forest_link_controller_native_headbutt_starts_tree_battle_spends_pp_and_continues`
  is a separate explicit initial Mew60/Headbutt/six-badge/player-ID4 fixture at
  the same oak approach. Player ID4 gives the native tree score0 there. Its
  replay stream is installed once before input; real A/YES attempts may miss
  and continue consuming that same native stream. No subsequent divider or
  state rewrite forces a hit. The test requires actual `BATTLETYPE_TREE`,
  `HeadbuttScript` startbattle index8/resume9, original rare-bucket species at
  level28 and native battle provenance at `(32,20)`. It does not assume that
  component replay timing guarantees Pineco in the production controller.
  Native Fight/move inputs must select Headbutt, spend real battle PP, win
  actual experience and leave a living partner at the original approach.
  Headbutt leaves the oak's blocks and quest flags unchanged. Save/Continue
  preserves the resulting party/HP/PP/experience, bag, flags and position.

The latter three bullet points describe the three tests above: the walking
fixture includes the observation, resources and Continue checks. All movement
planning uses shared `RouteGrid`; authoritative gameplay uses only
`VisibleShellController::press` and native presentation waits. Initial placement,
party/move/badges/player ID and replay setup are explicit test fixtures. No
reports, completion events, resources, menu cursor or later position are
injected. These tests do not earn BADGE6, establish regional hub travel, teach
Headbutt or prove route balance. Trainer teams remain unchanged; the populated
battle in this handoff is the genuine optional tree encounter.

### Detroit reward remains unresolved

The native R19 binder installs only the three observation roots and their saved
events. It does not allocate/execute
`EVENT_MN_LAKE_OBSERVATIONS_REWARDED`, place
`MN_DETROIT_LAKE_OBSERVATION_RANGER`, or attach Detroit's reward commands. The
controller test verifies that no R19 map silently contains that hand-in or an
item-grant script. The route's actual Max Repel pickup is collected once and
stays exactly one before/after observations; it cannot stand in for a delivered
quest reward. A real Detroit town actor/anchor, six-badge/missing-note guards,
actual Max Repel transaction, full ITEM-pocket retry and saved completion still
need separate native authoring and production-input tests.

After native publication and shared module registration, root can use its
existing warm target:

```sh
CRYSTAL_PRAIRIE_FOREST_LINK_TEST_PACK="$PWD/content-packs/minnesota-prairie-forest-link.crystalpack" \
CRYSTAL_PRAIRIE_FOREST_LINK_GEOMETRY_SAVE="$PWD/.local/prairie-forest-observations.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy prairie_forest_link_controller_ -- --ignored --test-threads=1
```

Optional `CRYSTAL_PRAIRIE_FOREST_LINK_HEADBUTT_SAVE` exports the actual fixture
Headbutt victory for later input/MCP review. Each regression's temporary
Save/Continue file has a unique process/time filename and is removed. Source
renders, published-pack comparison, native cameras, actual field/battle inputs,
original HOME/audio regressions and the unresolved town hand-in retain separate
acceptance statuses. This handoff reports authored tests, not executed gameplay.

Native execution exposed two real Headbutt integration gaps before publication.
The scoped leaf now additively retains the existing source Headbutt menu/context
roots in `StandardScripts.GlobalScriptRoots`, then uses the production global
materializer; every previous root and source body stays intact. The supplied
`HeadbuttScript` has the exact eleven-command success path followed by the exact
four-command local no-battle tail. Root extended the shared exact certificate
to recognize that flattened export as well as the split eleven-command form,
while still certifying the qualified local branch independently. Published
source remains fifteen commands. A fifth asset regression rematerializes both
valid component shapes and rejects changed success, flattened-tail and qualified
local-tail commands. Actual final asset/controller execution remains root-owned.

The first controller run exposed a fixture path-selection error: shortest
walking distance chose ten entered grass cells, while the authored least-grass
alternative costs eight. The fixture now chooses lexicographic
`(entered grass cells, walking distance)` paths over the existing RouteGrid's
clear, unoccupied, trainer-safe cells, and still executes/verifies every chosen
adjacent step through the production controller. The exact eight-cell quota
remains unchanged in both directions. The introduced nested-if clippy warning
in dialogue collection has also been fixed. Root owns rerunning this fixture.
Native Headbutt execution separately exposed missing source-root materialization
and a synchronous visual-host-effect boundary; the native author/root are
repairing those shared/runtime paths. No source program is trimmed and the
actual Headbutt battle/PP/provenance proof remains required.

### Actual controller rerun and inherited battle-level policy

Root's current production-controller run in `.local/prairie-forest-controller-current.log` passes both the two-ramp/grass/three-observation/Continue journey and the no-Headbutt denial. Native Headbutt now executes its actual visual effect and starts the source `BATTLETYPE_TREE` encounter with the correct origin; the remaining test stopped at its incorrect assumption that the materialized opponent must stay level28. Actual opponent level was59. No Headbutt victory/PP/Continue success is claimed from that interrupted test.

The existing default-battle overlay explains the difference. `resolve_tree_mon_encounter` writes the actual source selection's level28 to `wCurPartyLevel`; `start_scripted_wild_battle` reads it and applies the inherited `challenge_level(state, request.level, 2)`. The current default `WildChallengeRules` anchors on the explicit level60 Mew, applies offset−1 and scripted variation2 with spread2, and therefore materializes level59. Authored level28 is the encounter floor, not a promise to disable the pack's pre-existing scaling.

The controller assertion now independently requires actual `wCurPartyLevel == "28"`, actual selected species memory matching the battle, every common/rare source table entry still level28, and rare-tree species membership. It compares the battle level with the existing public `WildChallengeRules::encounter_level` using the actual initial battle party and scripted variation2 (or28 if that policy is absent). This invokes the production policy instead of duplicating it or weakening to an arbitrary range. No encounter data, battle policy, source program, RNG stream or post-input state is changed. The real selected-Headbutt PP, victory/experience and Continue checks remain required after this point. Scoped `rustfmt --check` passes; Root owns warm recompilation and the rerun.

### Headbutt victory reached; route-specific flag guard repaired

The next actual run reaches and wins the native Headbutt battle, spends selected Headbutt PP, keeps the review partner alive and awards experience; all those assertions execute before the reported failure. The test then incorrectly compared the complete `EventFlagMemory` to its pre-battle value. The log shows normalization of unrelated native engine-flag storage, including badge aliases, rather than a route observation or reward command. Global memory representation is not the intended local invariant.

The leaf now captures and compares the three SQ01 observation flags, the pending Detroit reward identity, all six authored R19 trainer-defeat flags and all four pickup flags. It explicitly requires their initial state to be false and preserves that exact fourteen-flag map after the tree battle and Continue. Reading the well-formed unallocated Detroit completion identity does not create/register/set it. Actual badge state, money and bag must also stay unchanged; oak metatiles, standing position, Normal movement mode, selected PP loss, victory experience, living party and exact party/bag/location Continue checks remain required. No battle policy, source program, game state after setup or RNG stream is rewritten. Scoped `rustfmt --check` passes. Save/Continue success remains pending the rerun past this repaired assertion.
