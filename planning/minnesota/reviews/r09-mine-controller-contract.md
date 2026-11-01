# R09 Survey Mine preservation and acceptance contract

Status: exact b3b0 native review pack published; all five assets, four production-controller, native199/nine capture and audio377 checks PASS. Guarded source promotion applied after explicit Root review/authorization, with independent outside-owned byte proof.
Root accepted the final source candidate for native implementation review after
viewing it. This is not user visual approval or earned Q06 progression.

## Frozen inputs and actual installed preimage

Duluth's final candidate is `caves/MnR09Cave1F.candidate.json`, immutable copy in
`renders/caves/r09-candidate/243f04da49a01f59/`, SHA256
`243f04da49a01f59e6fe8fb004049fd942c3acac1e1e6a5ad6c865f89b650d43`.
Previous source is `caves/MnR09Cave1F.previous.json`, SHA256
`8553987f7da3cfaa3e8bdb3a680686c9a32f9c869b8615bf01e87782b0f0e088`.
The prior827 candidate remains retained. Only the final mouth-face metatile6,8
changed from827; all story/actor/warp contracts remain identical.

Read-only inspection of actual written c16 refinements and a85 refuge packs
confirms their entire cached `MnR09Cave1F` modules are equal. The previous source
matches all132 installed blocks and the current route's whole room. Installed
identity is group54/map5;12×11 native cave, border1, environment CAVE,
`nite`/`PALETTE_NITE`, MUSIC_UNION_CAVE, LANDMARK_MN_IRON_RANGE, phone18,
FISHGROUP_LAKE, no connections/scenes/coordinate/background events. These fields
and every actor/program/index/party/constant remain exact in the result.
The full a85 pack SHA is
`a85fac64ab31c86722c486b50213053eb16ba3f6453f99e41bd81c962badb980`.

| Native object | Body / required approach | Behavior retained |
| --- | --- | --- |
| MN_R09_MINE_SCOUT |19,7 /19,8 Up|Scientist, standing south, ordinary script; exact original two pages |
| MN_R09_DEPOT_FOREMAN |9,5 /9,6 Up|Rocket, standing south, radius0, talk script; no sight-trigger trainer conversion |
| MN_R09_SURVEY_CACHE |7,5 /7,4 Down|Poké Ball sprite, OBJECTTYPE_SCRIPT, saved quest event; no bag item or native itemball |

All three have event flag `-1`, exact object identifiers and source scripts.
The cache sprite does not disappear after recovery; repeat interaction prints
its original empty-crate text. `script_field_pickups`, item grants/takes/checks,
shops and ordinary trainer_scripts remain empty.

Room warp1 at13,19 LADDER returns to MnR09C00R00 warp2 at40,42 FLOOR.
Clear indoor arrival13,18 remains FLOOR. Exterior active warp1 at40,41 CAVE
enters room warp1. This is a genuine original passive return, not a synthetic
step away from an active CAVE. All exterior/module/metadata/spawn records remain
unchanged.

## Story and every executable index

Current namespace values must be preserved, never allocated again:
`ENGINE_GLACIERBADGE=33` (seventh badge),
`EVENT_MN_HEADWATERS_REPORT=2218`,
`EVENT_MN_DEPOT_FOREMAN_DEFEATED=2219`,
`EVENT_MN_MINE_INSTRUMENT_RECOVERED=2220`,
`EVENT_MN_DEPOT_CLEARED=2277`, `MN_LODE_FOREMAN=32` within GRUNTM.
Victory sets only foreman defeat. Separately accepting and acknowledging the
meter text sets only recovered. Neither completes the later Grand Rapids depot
handover. Report/badge prerequisites are retained. The native scripted battle
request's own event_flag is empty; completion is the explicit post-battle script.

The implementation authority is existing `iron_range_story.rs`, especially
`validate_source` and `scripts`; do not copy a new story implementation. Index
inventory below is a review contract, not another exported script catalog.

| Program | All operational indices and meaning |
| --- | --- |
| MN_R09_DEPOT_FOREMAN,17 commands |0 face;1 defeat check;2 repeat branch;3 native seventh-badge check;4 badge denial;5 report check;6 report denial;7–10 original seen-text open/write/wait/close;11 win/loss labels;12 exact GRUNTM load;13 real battle;14 native map reload;15 defeat event;16 after-text jump |
| MN_R09_SURVEY_CACHE,14 commands |0 face;1 recovered check;2 repeat branch;3 defeat check;4 locked branch;5 open;6 ready text;7 yes/no;8 decline branch;9 recovery text;10 wait for acknowledgement;11 recovered event;12 close;13 end |
| MN_R09_MINE_SCOUT,1 command |0 original jumptextfaceplayer |
| MnIronRangeForemanAfter / ForemanBadgeDenied / ForemanReportDenied |Each0 jumptextfaceplayer to its same-named Text label |
| MnIronRangeCacheAlready / CacheLocked |Each0 jumptextfaceplayer to its same-named Text label |
| MnIronRangeCacheDeclined,4 commands |0 original declined text;1 wait;2 close;3 end |

Text program indices remain exact too: scout0–4 (two pages), ForemanSeen0–6
(three pages), ForemanAfter0–4 (two), CacheRecovered0–4 (two), CacheAlready0–3
(two, with second paragraph one line). ForemanWon/Lost/BadgeDenied/ReportDenied
and CacheReady/Declined/Locked each0–2 (one page). Every original line, para,
done command and typed body is retained; no rewriting or recombining pages.

Typed indexes: one scripted trainer battle, load12/start13; seven flag commands
(foreman1,3,5,15;cache1,3,11); two face commands(foreman0/cache0); one map reload14;
19 text/menu commands; nine control commands; two runtime commands(foreman11/12).
MapScripts has two section commands at0/1. Cached MapEvents has nine at0–8:
warp record2 and scout/foreman/cache object records6/7/8. All other typed
command vectors remain empty. The exact full native module, not these counts,
is the preservation comparison authority.

| Original enemy | Level / exact move PP; all PP Ups0 |
| --- | --- |
| GRAVELER |35: MAGNITUDE30, SELFDESTRUCT5, HARDEN30, ROLLOUT20 |
| MACHOKE |35: KARATE_CHOP25, SEISMIC_TOSS20, FORESIGHT40, VITAL_THROW10 |
| MAGNETON |36: SONICBOOM20, THUNDER_WAVE20, LOCK_ON5, TRI_ATTACK10 |

All have no held item and DVs attack13/defense8/speed10/special8/HP8. Preserve
whole trainer: name LODE FOREMAN, GRUNTM identity32, empty win/lose quotes,
items[None,None], base reward10, AI move flags935/item-switch68, original seven
AI layers and MUSIC_ROCKET_ENCOUNTER. Do not rematerialize different learned
moves or select a different grunt template in the published result.

## Concrete API blockers and bounded implementation

`authored_cave_revision.rs:68` deliberately rejects nonempty story arrays.
Removing that guard alone is wrong: typed `Interior` ignores those raw arrays,
and `room_events` certifies only the scout, not the installed three-object room.
Keep the generic guard; use a dedicated R09 leaf API accepting base, explicit
old/new room JSON and the canonical story JSON. Source comparison still allows
only rows/purpose/candidate_status/review_contract to differ.

There is a separate existing split-payload defect. On actual c16/a85 the cached
room contains all Q06 scripts/actors, but global MapEvents has only seven
records/scout and global NPC payload has only the scout. All19 Q06 executable
and text roots are absent globally. `iron_range_story.rs:111` inserts only the
cached room. Ordinary uncached assembly therefore cannot certify this preimage.
Do not hide this by skipping comparison or tolerating arbitrary missing indexes.

The dedicated operation should:

1. Verify the completed native pack and exact old rows/attributes/owned block
   payload. Certify retained raw source against canonical Q06 story authority,
   canonical existing `iron_range_story` program helper, original actor fields
   and exact source-legal native trainer template/party. Generate expectations
   only in temporary validation clones; no lowered story enters the result.
2. Certify the ONLY supported stale split preimage: old section has exactly
   original scout/warp, old NPC payload exactly scout, existing scout/text roots
   exact, and precisely the19 canonical Q06 roots absent. Reject a partial fix,
   different NPC/event payload, conflicting existing root or any typed drift.
3. Clone installed module and replace blocks only. Validate collision with all
   three actual bodies, designated adjacent approaches, all FLOOR reachable,
   exact exit/perimeter, each native hop and ordinary return.
4. Synchronize only those cache-owned19 roots, exact two missing object-event
   records and NPC mirrors through the established split insertion helper.
   Existing script roots/section metadata must remain byte-structure equal.
   Reassemble with cached R09 removed; require the whole module/indexes equal
   the preserved native module, plus only its two raw transport section labels.
5. Derive identity, verify, write/reload through Rust. Preservation permits only
   cached room.blocks, owned map_blocks, exact above split repairs and identity.
   All509 other modules, trainer/catalog/flag values, report, runtime files,
   programs already present, art/audio, HOME and recovery remain exact.

Root has made `blocks`/`validate_terrain` pub(crate), visibility-only. Reuse them;
keep this operation separate from Root's generic lateral-drop work. Add no new
story flags/actors/items and do not call the Q06 overlay on the published result.

## Geometry and actual controller acceptance

Final source has three native30 stair blocks at meta4,3;8,5;5,7. The actual native
collision is `[FLOOR,FLOOR,FLOOR,WALL]`, so use the left lane. Careful: at the
scout stair16,9 is WALL; its upper approach turns through17,10. Do not demand an
invented straight north lane. Other straight approaches are8,5→8,6→8,7→8,8 and
10,13→10,14→10,15→10,16. Each wall side must reject input, and each useful stair
must be traversed both ways through production input.

Native39 has TWO playable south-hop lanes: heads10,7/11,7, hop10,8/11,8,
landings10,10/11,10, reverse10,9/11,9 WALL. Ordinary return lengths are9/11 FLOOR
steps with all three bodies occupied. Both shoulders must be blocked. Never BFS
onto HOP_DOWN; approach from above, press Down to invoke the actual jump, reject
reverse, then walk the real return. No cave size expansion/new mouth is needed.

The controller acceptance should reuse the existing three
`iron_range_story_controller_` cases on the revised native pack, with source
geometry assertions, plus a focused geometry/scout-page case if needed. Preserve
real trainer turns and PP/HP/experience mutation, actual enemy roster, loss
recovery, input ownership and separate quest acknowledgements. Fixture setup
must explicitly label local position, legal party/moves, initial seventh badge
and report flags/RNG; synchronize badge engine flags and badge state up front.
No party/flag/HP/position writes after initial setup.

Required stages: missing badge denial, missing report denial, locked cache
before win, actual win with all three enemies, only defeat set, Continue,
repeat foreman without another battle/reward, cache No with no recovery,
Continue, cache Yes with recovered still false until the fully printed original
pages are acknowledged, only recovery set, Continue, repeat empty-crate notice
without bag/money/party changes, passive mine return and deliberate reentry.
Loss must follow actual combat and native blackout without victory/recovery/depot
flags, then Continue. Scout original pages must fully print and own movement:
`The foreman took` + `the instrument.` and second page `The upper passage` +
`reaches his cache.`. No dialogue rewrite is authorized.

Assets negatives must include matching false old/new story source, typed battle
index/flag/prompt drift under valid identity, original trainer move/level/AI
mutation, wrong story prerequisite/milestone, any stale split variation or root
collision, blocked designated approach, severed stair, unsafe hop/return and
unpaired exit. A valid unchanged native/audio fixture control must precede
malformed-pack negatives. Written reload and true uncached reconstruction are
required, not an accessor that returns the cached room. Final production NITE
captures and a source-only guarded promotion follow passing proof. Implementation now exists in the dedicated leaf described below. No Cargo/export,
pack publication or shared-source promotion has been executed by this worker.

## Implemented source handoff and completed native acceptance

The dedicated API is now implemented in
`crates/crystal-assets/src/r09_mine_revision.rs` and exported as
`revise_r09_mine_terrain(base, previous_room_json, revised_room_json, story_json)`.
The standalone Rust publisher is `crates/crystal-assets/src/bin/revise_r09_mine.rs`.
Existing Q06 source/program/trainer helpers received visibility-only changes;
their behavior and the shared generic story rejection are unchanged.

The operation certifies all native actor/program roots against expectations
produced only in temporary validation clones, then changes the original cached
module's blocks. It accepts only the exact historical scout-only NPC/MapEvents
and absent19-story-root preimage, synchronizes those owned mirrors using the
existing insertion function, and removes the cache to prove full production
assembly with every typed program/index. It never replaces the cached story
with a freshly lowered one. Exactly two raw section roots are present in the
uncached expectation; no executable field is filtered or normalized.

Five focused assets cases in
`crates/crystal-assets/tests/r09_mine_revision.rs` cover full other-data
preservation, actual written reload/uncached equality, matching false-source
rejection, native stair/hop/actor/mouth/boundary negatives and ten valid-identity
native corruption variants. The unchanged native/audio control is required
before native index/party/AI/flag/split negatives. All generated native packs and
verification remain Root-owned; no Cargo/export was run by this worker.

The final baseline is Root's glacial-cleft composition:
`content-packs/minnesota-northstar-glacial-cleft-review.crystalpack`, SHA256
`1630ad063aa4a1cf252087bdf743ca48a2a8cb6c9c46ad95cb7aceb1c74c3b05`.
Root's preservation proof retains every R09 cached/split and trainer byte from
a85, including the precise historical defect repaired here. Tests default to
this1630 baseline and immutable243 before/after rooms. `CRYSTAL_R09_MINE_STORY`
can freeze the canonical story file; current observed story SHA256 is
`b9ecfc591324e1b22b5dcad4b204c668580b46a1ffbe039f617bdb2eab227ad5`.

Root executed native publication and assets checks. Equivalent commands (shared warm target/profile flags omitted; none run by this worker):

```sh
CRYSTAL_R09_MINE_REVISION_PACK="$PWD/content-packs/minnesota-northstar-glacial-cleft-review.crystalpack" \
  cargo test --locked -p crystal-assets --features test-fixtures \
  --test r09_mine_revision -- --ignored --test-threads=1
cargo run --locked -p crystal-assets --bin revise_r09_mine -- \
  content-packs/minnesota-northstar-glacial-cleft-review.crystalpack \
  planning/minnesota/renders/caves/r09-candidate/243f04da49a01f59/MnR09Cave1F.previous.json \
  planning/minnesota/renders/caves/r09-candidate/243f04da49a01f59/MnR09Cave1F.candidate.json \
  planning/minnesota/story.json \
  content-packs/minnesota-northstar-mine-cleft-review.crystalpack
```

Optional assets source overrides are `CRYSTAL_R09_MINE_PREVIOUS_ROOM` and
`CRYSTAL_R09_MINE_REVISED_ROOM`. Native controller acceptance now passes; guarded source
promotion has been applied after explicit Root review/authorization. One compatibility boundary to watch in diagnostics:
the temporary canonical trainer reconstruction expects the current standard
GRUNTM1–31/foreman32 namespace. The published namespace is never reallocated.
A future independently extended grunt namespace may need an isolated trainer
certification adjustment; it must not change this exact native32 identity or
relax whole-party/template comparisons.

## Controller leaf and first assets execution

Root's `.local/r09-mine-native-assets-first.log` passed four of five cases,
including all ten native typed/party/flag/split drift negatives and actual
roundtrip. The geometry case's proposed scout-stair removal was accepted because
that bay retains a legitimate ordinary alternative. This is safe alternative
terrain, not evidence of a failed safety guard; no stair quota was added to the
API. The one negative now removes the scout's actual standing-floor block9,3,
which violates actor collision/access. Prior full test source and failed log are
retained. Root corrected rerun passed all five cases in34.99s, `.local/r09-mine-native-assets-corrected-negative.log`.

New focused leaf `crates/crystal-bevy/src/bevy_shell/tests/r09_mine.rs` supplies
`r09_mine_controller_original_mouth_three_stairs_two_short_hops_scout_and_continue`.
It reuses shared production input/dialogue/Continue helpers, not another
controller. Empty party/local passive exterior placement, money1000 and replay
RNG are explicit initial geometry setup; no progress or party mutations follow.
It explicitly steps every adjacent stair tile both ways, attempts each actual
wall side, jumps both native39 lanes via Down input, rejects reverse and walks
9/11 ordinary return steps with all three actors occupied. It reads both fully
printed original scout pages, proves text owns movement, retains resources and
semantic flags across Continue, then proves genuine passive return, walking
away/back and deliberate reentry. Existing three iron_range_story cases remain
unchanged for actual foreman/cache combat and quest acceptance.

The leaf's optional source env is `CRYSTAL_R09_MINE_SOURCE`, default immutable243.
Its native input pack env is `CRYSTAL_R09_MINE_CONTROLLER_PACK`; Root's planned
output is `content-packs/minnesota-northstar-mine-cleft-review.crystalpack`.
Root registered and executed this case in2.73s, `.local/r09-mine-controller-geometry-first.log`. Existing three real Q06 cases passed in12.29s, `.local/r09-mine-controller-story-first.log`; shared binary/pack receipt is `.local/r09-mine-controller-first-receipt.json`. Static probe validation against
frozen243 rows and actual1630 art confirms all three exact stair lanes/wall sides,
both hop shoulders and the two exterior Floor cells. Those static probes alone did not establish input behavior. The subsequent actual pack, pixel and input checks below now pass.

## Actual written proof and held source preflight

Final native review pack is
`content-packs/minnesota-northstar-mine-cleft-review.crystalpack`, SHA256
`b3b0c67847b6fa2ac1f65ba0e5075a55f5b7372657a9d69c7193b2a41ba15e69`,
internal identity
`75c7a0f115f11bfe87d23e387af71f9a2cf2d77cd3174c1764d614f168dc6ccb`.
Written preservation/capture report is immutable
`renders/caves/r09-native/b3b0c67847b6fa2a/review-01/native-verification-and-captures-cf52daad1dfed90c26928d292302cc8bad7dea5442b75c53eeb228e9c54f071d.json`.
All199 checks pass: all509 other modules/full cached story typed indexes remain
exact, with only the explicitly certified19-root/NPC/event mirror repair and
owned block changes. All nine actual GPU views match207,360 complete LCD pixels,
including original three actors, CHRIS, forced NITE and native foreground. Every
original/raw GPU buffer/descriptor/conversion/source stage is retained. Whole,
contact and widest-upper frame were personally inspected; Root separately
viewed all nine and accepted this bounded terrain pass. User final approval and
earned regional progression are not inferred.

Root's native audio log `.local/r09-mine-native-audio.log` verifies all377 PCM
hashes/frame counts/loop ranges on the actual b3b0 file. All five assets cases
and four controller cases pass, including played victory/loss, prerequisites,
separate Yes/No/recovery acknowledgement, repeat/Continue, scout full pages,
three stairs, both hops with9/11 ordinary returns and genuine passive return.

Source promotion script `caves/promote_r09_mine.py` passed with `applied:false`
against immutable `renders/caves/r09-promotion/preflight-9e922e540b3ab0ea/`.
That folder contains whole before route/interiors, original room, frozen243
candidate, proposed room/contract, exact native199/viewed receipts and all
execution logs/receipt. Their hashes are validated before proposal. It splices
only one decoded R09 room and one global interior record, preserving every byte
outside those objects. The safe `arrival_core13,18` remains distinct from the
explicit `native_warp_arrival_core13,19`; passive warp2 at40,42 is original.
Preflight preimages remain unchanged:

- `routes/MN_R09.json`: `5478955ee0d7aad3c0ee77416e41b6d87c46da2d5b6d60804d3cc80b75ef07b0`
- `interiors.json`: `88547e94877f192f488a53a724ab957791c9a49e35a9a97afc0c968201979b0c`

The frozen Root viewed receipt is
`renders/caves/r09-native/b3b0c67847b6fa2a/review-01/root-viewed-receipt-d6f1e14323ef1feabb166d0a1ab41f2e2d0367fb371ace3bb7b75f05ce52caa2.json`.
Application was held until Root reviewed the concrete records and supplied explicit authorization. It is now applied as recorded below. Shared STATUS/atlas updates remain Root-owned.

## Applied source promotion

Root reviewed the exact freeze/proposals/evidence and explicitly authorized
application. The script first required complete route/interiors file hashes to
match the reviewed5478955e/88547e94 epochs; both matched. It then applied only the
owned room and owned global interior contract, with all four passing controller
case logs still frozen. The untouched243 candidate and b3b0 native pack hashes
were checked again afterward.

Full application evidence is retained at
`renders/caves/r09-promotion/preflight-9e922e540b3ab0ea/application-1793071697845541000/`:
full before and after route/interiors, proposed records, original/candidate room,
script snapshot, native199/viewed reports, all assets/controller/audio logs and
binary receipt, `promotion.json`, `application-full-receipt.json`, and independent
`outside-owned-byte-proof.json`. The independent proof decodes each unique owned
record and compares its entire preceding prefix and following suffix separately;
both match exactly. Every original noneditorial/nonterrain actor/story/warp field
is unchanged; rows equal frozen243.

R09 application epoch hashes (before subsequent unrelated promotions):

- `routes/MN_R09.json`: `69516449f5425495e81d03697ea098dcc3ff8b69318460364d1aae22e0ff9221`
- `interiors.json`: `baa3158979a1703cb86fb3506eca0648fd98b3bdc8d4e6c20bc77a1b5e19c8f0`

Root was notified the shared interiors write is complete before starting its
separate R01 promotion. No STATUS/atlas or unrelated authoring source was edited
by this worker. Current local native behavior and preservation are proven;
earned regional Q06 journey, old-save block-override migration and final user
visual approval remain separate acceptance work.
