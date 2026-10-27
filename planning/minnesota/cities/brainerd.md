# Brainerd: lake-country hub

**SOURCE ONLY city.** One complete 32×24-metatile town, four city-owned rooms,
and a reference to the existing canonical Gym6. Original native pixels and the
reserved regional ports are retained. The source reviewer does not install city
doors, services, story rewards, recovery or walking seams. The canonical gym has
a separate Rust publication path; Root owns binding that room to this city and
verifying actual gameplay.

The northern pine trail enters a quiet report court. The western road passes
short POKECENTER and MART porches. A fishing pier extends into the northeastern
lake, with a separate visible launch. The southwestern Gym stands on a low
terrace: climb the southern ramp to challenge SIVREN, then hop east toward the
southbound street. An eastern lane leads to a small camp-supply cabin. There is
no Brainerd Glacier Line station; the rail service is a separate network.

## Source and quick review

- [Town dossier](../towns/MnBrainerd.json)
- [Explicit hand-authored terrain](../towns/MnBrainerd.layout.json)
- [Whole town](../renders/brainerd/whole-town.png)
- [Current revision and source-check report](../renders/brainerd/revision.json)
- [Reviewer](../tools/review_brainerd.py)

From the repository root:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_brainerd.py \
  --pack content-packs/minnesota-quality-review.crystalpack
```

Add `--watch` for repeated source checks and renders. The reviewer loads the
completed local pack once; it never exports content or runs Cargo. The initial
successful render/check pass took 1,008 ms; a later warm pass took 807 ms, including
whole town, all rooms, individual 160×144 player cameras, 4× nearest-neighbor
previews, source preservation and collision checks. Concurrent system load can
increase that time. Authoritative JSON is separate from ignored renders.

Every pass retains the exact two authoring files before validation, including
rejected passes. Every rendered full-resolution original, RGBA and RGB form,
crop, scale, player view, contact sheet and rejected image is immutable in render
history. Latest aliases are extra copies. The `source-passes/` archive records
invalid edits as well as successful revisions.

## Exact geometry and returns

Coordinates below are 16px gameplay tiles; each metatile contains 2×2 of them.
The four reserved world ports remain unchanged:

| Neighbor | Side | Gate metatile | Landing metatile |
|---|---|---|---|
| MN_R06, St Cloud | south | 16,22 | 16,20 |
| MN_R07, Leech Lake | north | 16,1 | 16,3 |
| MN_R15, Detroit Lakes | west | 1,12 | 3,12 |
| MN_R18, Mille Lacs | east | 30,12 | 28,12 |

The only outer walking cells are the two-core lanes at north x32–33, south
x32–33, west y24–25 and east y24–25. Root must derive and validate reciprocal
native seam offsets against the actual adjacent route chunk dimensions. A
planned port is not a published connection.

| Room | Actual exterior DOOR | Safe exterior FLOOR | Exterior active / passive index | Room arrival3 | Room exit1/2 |
|---|---|---|---|---|---|
| Pokecenter1F, 10×8 | 9,21 | 9,22 | 1 / 2 | 10,13 | 10,15 / 11,15 → exterior2 |
| Mart, 6×4 | 19,21 | 19,22 | 3 / 4 | 6,5 | 6,7 / 7,7 → exterior4 |
| Gym, canonical dimensions | 12,35 | 12,36 | 5 / 6 | canonical arrival3 | canonical carpet1/2 → exterior6 |
| RangerStation, 8×6 | 23,15 | 23,16 | 7 / 8 | 8,9 | 8,11 / 9,11 → exterior8 |
| Outfitter, 6×4 | 47,35 | 47,36 | 9 / 10 | 6,5 | 6,7 / 7,7 → exterior10 |

All five visible facade DOOR quadrants are bound. Exterior entry targets the
room's passive arrival3; both native exit carpets target the matching exterior
passive return. Source flooding stops at active exterior doors, so a facade
cannot be used as a walking tunnel. Indoor border blocks have native WALL
collision; unexpected perimeter openings and interior floor triggers fail review.

The Gym uses the exact complete Olivine-style three-wide facade rows
`18 1f 19 / 1c 1d 1e`. Clinic and MART use `18 19 / 1a 1b` and
`18 19 / 1a 17`; the ranger lodge uses `18 19 / 16 1e`.
The smaller outfitter uses native `14 15` as a complete low cabin. None of these
facades invents or paints a door into the source pixels.

## Lake, pier and one-way return

The lake is inset: no WATER touches the town perimeter. It therefore exposes no
unconnected off-map Surf seam. Ordinary walking reaches every service, story
actor, Gym and regional port before Surf. The pier's deck is x47–48, y14–19;
western rail44 blocks x46, eastern rail46 blocks x49, and the northern head40
blocks y12. The pier entrance and two-core passing space are real FLOOR.
The fisher occupies one deck lane, leaving the other open. The public launch
is FLOOR `(44,20)`, facing WATER `(44,19)`. Source geometry permits Surf there;
native move/badge denial and successful launch still need controller testing.

The raised Gym has one ordinary ramp, native block74 at metatile `(10,20)`.
Replacing it with native cliff72 disconnects the Gym from the town's walking
graph, while other services remain public. The eastward ledge is native4d at
metatile `(12,19)`: FLOOR approach `(23,38)`, HOP_RIGHT `(24,38)`, reverse WALL
`(25,38)`, landing `(26,38)`. Reverse-column side seals `(25,37)` and `(25,40)`
are WALL. The ordinary climb back is **27 walking tiles**, through the ramp
rectangle `[20,40,22,42)`. Removing that passage disconnects the return. The
three-tile hop therefore changes the return trip rather than decorating a road.
No Cut tree is placed, and the checker inventories every quadrant for incidental
CUT_TREE collision.

## Services, story and battle handoff

Gym6 has one authority: [gyms/interiors.json](../gyms/interiors.json), object
`MnBrainerdGym`, SHA256
recorded in `MnBrainerd.json` and each immutable source-review report. The
shared source is changing outside this worker; the native preflight must match
the exact captured canonical JSON rather than trusting an older hash.
The city dossier stores its source path, ID and signature, not alternative room
rows, art, residents, parties, puzzle stations or rewards. The reviewer resolves
the canonical object in memory and draws every glyph using its exact source
tileset/block and collision quadrants. It never writes or publishes a gym pack.
A stale signature or city-owned functional override fails review.

SIVREN's party and moves, sprite/anchor, native badge6, retryable TM_RAIN_DANCE,
trainers, all stations and the gate come directly from that canonical object.
LORLA and BRAVEL retain their canonical trainer identities; PINE and REED require
the respective trainer defeats. CHART requires both markers, offers the exact
west-to-east order menu, and opens the court only after the correct response.
The source walkthrough checks trainer1 → PINE → trainer2 → REED → CHART, every
approach and all prerequisites. Its declared event sets are source fixtures,
not native script execution or earned wins.

The viewer now takes dimensions, guide position, room arrival, both exits,
station cameras and leader court directly from the same captured canonical
source. The room is currently 10×10 with arrival `(10,17)` and carpets `(10,19)` /
`(11,19)`; the exterior facade and passive6 remain `(12,35)` / `(12,36)`.
Earlier 12×12 counts and views are historical. Current source-review JSON records
the exact canonical hash, phase reachability and retained images for each pass.
No Surf is needed for the dry puzzle or leader approach.

Root must preserve the existing canonical `mn_gym_6` module/art/scripts/flags/
trainers, then replace the three review-hall destinations with MN_BRAINERD
passive exterior warp6. City entry targets native room arrival3. The focused
native builder does that rebinding without replacing the functional gym module.

The story ranger stays at the already reserved outdoor anchor `(25,19)`, with
an accessible south-facing approach `(25,20)`. MS05 requires BADGE6. The ranger
reports the habitat discrepancy and grants the existing HM_SURF once; only a
successful grant sets EVENT_MN_BRAINERD_REPORT and
EVENT_MN_SURF_GIFT_RECEIVED. A full Bag must permit a retry. Repeated interaction
points toward the dry Leech Lake/Bemidji route. Preserve inherited Surf engine
move/badge permission rather than implementing a new boating permission.
The ranger archive has only the report clue and a volunteer; it must not award
Boundary Waters' receiver/chief/restoration flags.

The clinic declares a native nurse, actual counter approach, PC and safe recovery
anchor. The MART declares normal prices, money and inventory semantics, pending
native installation. The small outfitter offers one SUPER_REPEL with
EVENT_MN_BRAINERD_CAMP_REPEL set only after successful giveitem, full-Bag retry and
no duplicate reward. All ordinary actors have native sprites and free approaches;
all visible physical signs have actual readable approaches.

Root needs to register the two new optional room IDs, the local
EVENT_MN_BRAINERD_CAMP_REPEL flag and exact native city constants. Reuse canonical
MN_GYM_6, MN_GYM_6_TRAINER_1/_2 and their existing puzzle/defeat/TM flags; the city
must not allocate alternative T01/T02 trainers or flags. Do not replace the existing
story ranger identity or anchor. The source author did not edit shared world,
interiors, actors, story, train files or Rust publication code.

## Sight review and remaining evidence

I inspected the authentic native Olivine City map, Johto/modern source atlases,
the complete first town render, all five rooms, each important 160px exterior
camera and the computed widest-floor camera. The first render was rejected:
broad camp and report lawns, a double-roof Gym facade, an excessively deep tan
terrace and duplicated ranger/outfitter workrooms made its identity weak.

The revised source replaces those empty shoulders with attached stepped pines,
a short camp-to-report trail, a physical fence along the service porches, a
railed lake promenade, a smaller terrace silhouette and the exact native Gym
facade. The supply cabin now has its own small house interior, table, map and
gear lockers. The worst camera moved from an empty eastward shoulder to a
readable report-lane turn with the ranger, flowers and a close pine boundary.
The central crossing is deliberately wider than an ordinary lane because it
joins four regional roads; its destinations remain visible in adjacent cameras.

The source reviewer rejects twenty meaningful mutations: false facade door,
active carpet arrival, active exterior return, moved regional port, exposed
outer water, removed pier rail/head, removed ramp, false ledge, incidental Cut,
missing badge prerequisite, unrelated Boundary receiver hook, walkable room
border, alternative city-owned gym rows/actors/summary party, stale canonical gym
signature, invented HM item, a visible cabin without its room and moved ranger
anchor. These establish source contracts, not visual
acceptance or runtime gameplay.

Pending native evidence: reciprocal four-way movement, each door/return in both
directions, native denied/reverse/forward ledge inputs and full walking return,
healing HP/PP/status, PC and MART money/inventory, all Gym battles with selected
move PP changes, loss/win/repeat badge ownership, ranger's before/success/full-Bag/
repeat and save/Continue behavior, optional gift persistence, Surf denial/success,
headless actual native captures and exact preservation/audio verification.


## Canonical reconciliation review

The earlier frozen city candidate had an independent practice-pool gym under
MnBrainerdGym. That candidate is retired. Its full source, reviewer and runbook
were saved byte-for-byte under `renders/brainerd/canonical-reconciliation-before/`;
all its whole/cropped/converted renders remain in their original revision folders.
The exterior terrain, doors, returns, residents, all four ports and remaining
four city-owned rooms are unchanged by this reconciliation.

The initial reconciliation inspected the earlier canonical closed/open whole
rooms and five player views. Its timber-strip WATER concern is historical: the
current frozen Gym6 has no water tiles. The final frozen closed/open comparison
and four closed player views were inspected again. The log court barrier is
visible before CHART; one passage appears in the declared complete phase, and
SIVREN stays inside her court. These previews preserve exact authoritative art.

## Native implementation handoff

The focused Rust implementation is `build_brainerd_city_modpack(base, town_json,
layout_json, canonical_gyms_json)`. Root has registered its four leaf modules and
public export; the first warm library check passed. The meaningful `Room.kind`
field is now validated against each exact room identity instead of suppressing
its warning. Asset integration tests are in `crates/crystal-assets/tests/brainerd_city.rs`;
the actual compiled asset binary passed all five tests with aligned frozen
source inputs in 6.38 seconds. The retained log is
`renders/brainerd/native-handoff/native-asset-tests.log`. The later test fixture
source override needs Root’s warm recompile; final pack publication and native
controller gameplay are still pending.

The builder reserves native group69 only when runtime metadata, group constants,
map identities, landmark and manifest are free. It compiles the supplied full
32×24 town strokes through the existing typed `Layout` deserializer/compiler,
with city-specific source and geometry checks. It adds only five maps: exterior,
clinic, MART, ranger archive and outfitter. It creates ten actual exterior
active/passive warps, two carpet exits plus passive3 in each new room, native
HealParty and PokemonCenterPC specials, and MART_MN_BRAINERD with the exact authored
stock and native shop behavior. It never changes HOME or the existing recovery
table; a new Brainerd blackout recovery registration remains future work.

Q04/MS05 now has a native implementation at the original ranger anchor25,19:
check the native sixth badge, offer the existing HM_SURF, branch on a full TM
pocket before setting either story event, then persist the once-only reward.
No badge or story event is granted during installation. The optional camp
SUPER_REPEL has the same success-only/full-Bag/repeat structure and a visible
receipt message. Production controller tests still need to prove actual item,
party HP/PP/status, money, menu ownership and persistence changes.

Canonical Gym6 is a precondition. The builder regenerates its expected native
art, full module, scripts, objects and trainer definitions in a private scratch
clone using the existing gym installer. It compares actual installed art files,
palette/collision, Cut/Whirlpool aliases, every module field and exact trainer
parties/moves against that expected result. A matching source SHA alone is not
accepted. The scratch pack is discarded; it never replaces the installed gym.
Only the existing three gym warp destinations and their raw native warp arguments
are rebound to MN_BRAINERD passive6. Glyphs, events, menu puzzles, actors, rewards,
trainers, metadata and all other module fields remain the original canonical ones.

The shared canonical gym file is being revised outside this worker. Consequently,
Gym6 dimensions, actors, arrival and exits must come from the same frozen canonical
JSON that built the supplied native pack. The source viewer now resolves those
fields dynamically; current 10×10 source has arrival10,17 and carpets10,19/11,19,
while the town facade12,35 and passive exterior12,36 remain fixed. The earlier
12×12 review images are historical artifacts. Any source/runtime mismatch causes
an explicit error. Root will freeze matching inputs and compose the actual pack.

The integration test fixture captures the source strings once and builds the
canonical gym pack in memory from the completed external TUI pack. This avoids
assuming an older compiled gym matches a changing source file. An explicitly
matching fixture can instead be supplied with CRYSTAL_BRAINERD_CITY_TEST_PACK.
Set `CRYSTAL_CITY_GYMS_SOURCE` to an explicit frozen canonical JSON path to
reuse Root’s common candidate; the fixture captures it once and aligns only
the city reference in memory. Production builder checks remain strict. No source
exporter, competing pack publisher or cold Cargo build ran in this worker. Root
owns publication and the warm test/build cache.

The four regional ports retain their authored positions and have no invented
train stop, teleport or implicit gate. They still need actual reciprocal walking
seams and earned regional journey/controller evidence. The code and asset tests
alone do not make the full regional game playable.


## Historical b3 native asset candidate

The accepted candidate uses full eight-gym file SHA256
`b3f25b09e4ce114fd561ff28309464fb9e45500bb159dcb6d145ce2f06f7bbbb` and
Gym6 object SHA256
`4f41dfd5f12153ce79067ecb6adb7d0b9b0644d99bfa762e38de45505d787187`.
All exact source strings used by the five passing native asset tests are under
`renders/brainerd/native-asset-input-captures/`; the handoff manifest lists their
full paths, SHA256 values, Rust implementation hashes, verification limits and
unchanged render hashes. Do not silently substitute a later live canonical file.

The final source review is `renders/brainerd/807c952bd026/review.json`: twenty
negative mutations rejected, closed court102 and opened court159 reachable
walking tiles. Whole town, whole closed/open Gym6 and all closed player views
were personally inspected. Exterior terrain/actors and all four nonGym room
images match the prior saved render bytes. The canonical guide’s19-character
line `PINE. The east says` is retained and reported as a dialogue-width diagnostic;
production controller wrapping is still unverified.

Reproduce that frozen source preview with:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_brainerd.py \
  --canonical-source .local/minnesota-city-source-freeze/gyms-b3f25b09e4ce114fd561ff28309464fb9e45500bb159dcb6d145ce2f06f7bbbb.json \
  --town-source planning/minnesota/renders/brainerd/native-asset-input-captures/MnBrainerd-575ffd6aef85fb633ad76de6d77cec12aedf468b5d6ce67904f0fb649eb4ee09.json \
  --layout-source planning/minnesota/renders/brainerd/native-asset-input-captures/MnBrainerd.layout-2af837ce974c8c9c34e1de98153a090ec234f425e62c1900526c992591f577a9.json
```

The reviewer snapshots all three input byte strings before validating or
rendering; `--canonical-source` marks an explicit frozen candidate without
changing the declared canonical authority or writing a pack. Every original,
conversion, player crop and prior/rejected render remains preserved.

## Historical b3 controller regression candidate

`crates/crystal-bevy/src/bevy_shell/tests/brainerd_city.rs` adds six explicit
local controller fixtures. Initial placement, prior badges, legal party, injury,
money, full-pocket inventory and deterministic divider bytes are labeled setup.
Every later action uses `VisibleShellController::press`; no script, battle,
reward, position or menu state is directly advanced by a test. Ordinary paths
reuse `authored_route_fixture::indoor_walk`; the focused Gym6 path planner also
reads actual persistent gate block overrides.

The cases target five facades with both carpets and passive arrivals, real east
hop/reverse denial/ordinary ramp return, actual HP/status/PP healing and PC menu,
native priced MART inventory/money mutation, optional full-Bag refusal/retry/once,
and two canonical trainer battles plus PINE/REED/CHART cancellation/wrong/correct
answers before SIVREN. Battles require actual selected-move PP loss and subsequent
victory events. Badge6 is earned in that isolated local fixture; the ranger must
refuse Surf before it, then grant the HM/report once after the earned victory.
Save/Continue compares party, Bag, badges, flags, money, position and gate state.
These tests do not prove earned regional travel, balance, blackout recovery or
unimplemented reciprocal city seams.

Root must register the test module and execute the candidate against the actual
combined native pack. The worker has not run these new controller cases yet:

```sh
CRYSTAL_BRAINERD_CITY_CONTROLLER_PACK="$PWD/content-packs/minnesota-cities-review.crystalpack" \
CRYSTAL_CITY_GYMS_SOURCE="$PWD/.local/minnesota-city-source-freeze/gyms-b3f25b09e4ce114fd561ff28309464fb9e45500bb159dcb6d145ce2f06f7bbbb.json" \
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
  cargo test --locked -p crystal-bevy brainerd_city_controller_ -- --ignored --nocapture
```

The current shared installer represents puzzle stations as native READ background
events on FLOOR; the source preview's paper sprites are metadata-derived review
markers. Controller interactions establish facing by a final ordinary step to
an adjacent approach, then press A, without inventing station collision. This
preview/native station presentation difference remains explicitly reported.


## Current887 integration candidate

The new isolated candidate uses exact full canonical file887afc43, Gym6 object
`e2194a0521626c3f8fc786d49bcdcdfbd641b83f4c9c49ec010816a4c6f6dd53`,
12×11, arrival12,19 and carpets12,21/13,21. NESSA5,13 and BRAM18,13 are the two
actual required trainers. Both real victory events open the upper gangway; this
version has no PINE/REED/CHART stations. MARA12,5 and her exact native party,
badge and TM_RAIN_DANCE reward remain canonical. No changing live draft was
rewritten. The575+b3 candidate and all previous source/renders/test evidence are
historical and preserved.

[Exact887 manifest](../renders/brainerd/native-handoff-887/manifest.json) lists
immutable inputs and complete environment paths. Town source SHA256 is
`a6e99c25936262a775e76ddb29721e308ca140c37ef6dc36bf4eeb9fbea6c7fa`;
layout remains2af837ce. Set `CRYSTAL_BRAINERD_CITY_SOURCE` and
`CRYSTAL_BRAINERD_CITY_LAYOUT` for asset tests, alongside
`CRYSTAL_CITY_GYMS_SOURCE` and `CRYSTAL_BRAINERD_CITY_TEST_PACK` pointing to the
published480-map `minnesota-city-gym-base.crystalpack`. The fixture now consumes
exact source strings without in-memory reference repair or field removal.
The strict builder and all five native asset cases remain intact.

The same canonical environment drives controller Gym6 door/arrival/carpet and
progression geometry. The sixth case now requires two real trainer wins, actual
selected-move PP loss, a still-locked court after the first win, automatic court
opening after both wins, and the real leader battle/badge/TM before the ranger’s
Surf gift. It does not synthesize an obsolete quiz. Prior unexecuted controller
and asset test sources are retained in `renders/brainerd/controller-before-887/`.

Exact887 source preview `renders/brainerd/5afb9fd303ae/` passes20 negative source
mutations; its closed/open walking counts are129/202 with52 WATER tiles. The
whole town, both whole Gym phases and arrival/LORLA/BRAVEL player views were
personally inspected. The teaching pier has readable raised wood edges and a
clear upper gangway barrier. The exterior, actor scene and all four nonGym room
images match the earlier saved bytes. Canonical guide lines over18characters
remain reported verbatim; native wrapping and controller gameplay are unverified.

Native asset tests and controller cases for887 are pending Root’s actual warm
build/run; the old five PASS result must not be attributed to this new candidate.
Root repaired the global checker’s unsafe nonGym carpet target/index omission;
the audit’s saved old failure probes are explicitly historical.

## Clinic recovery implementation plan — pending

[Exact recovery plan](../reviews/brainerd-clinic-recovery-plan.md) traces the
remaining service gap. Retain two safe addon-range spawn rows: town clinic return
9,22 and clinic passive floor10,13; allocate after all current source/city rows
without hardcoding256 or overwriting Bemidji. The nurse must execute native
`blackoutmod MN_BRAINERD_POKECENTER_1F` after HealParty, with parsed/global split
payloads refreshed. HOME and source Fly destinations remain unchanged.

The plan specifies actual no-nurse/healed Gym6-loss branches, Party-menu Teleport
for both remembered destinations, safe arrival and Save/Continue acceptance.
No recovery code, source map, pack or test was changed for this documentation.
Current37a09 written-pack preservation passed81/81; Brainerd recovery remains
pending and must not inherit Bemidji’s separate evidence.
