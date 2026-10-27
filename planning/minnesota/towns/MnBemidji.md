# Bemidji — native Cut-garden revision

**Current gym remake:** the native 6×10 Cut garden supersedes the historical
11×10 quiz previews below. The current source and local-city commands are in
[the gym runbook](../gyms/README.md#bemidji-review-loop);
[review the complete room](../renders/gyms/bemidji-og/Bemidji-review.png).
The 479-map city candidate binds both carpets to the real facade. Q05 and
regional seams remain separate work. Older hashes and captured input tuples
below describe their named historical versions.

The current local city candidate publishes the hand-authored terrain, six real
facades and six rooms. Native gym rewards, healing/PC/shop, losing-battle
recovery, Continue and safe room returns have focused controller evidence.
Q05 and regional walking seams remain integration work. The source is 24×24
metatiles / 48×48 walking tiles, preserving all original exterior dimensions
and four route reservations. Later captured hashes below refer to their
historical source and native milestones.

Review the [whole town](../renders/bemidji/whole-town-64d6697855.png),
[all player-scale cameras](../renders/bemidji/viewpoints.png),
[all six rooms](../renders/bemidji/rooms.png) and the
[canonical gym gate states](../renders/bemidji/gym-gate-state-review.png). These are original-art source previews,
not native gameplay captures. The canonical latest aliases are additional
copies: every original, rejected pass, crop, conversion, enlarged preview and
source snapshot is retained under the same render directory and its history.

## Composition and play

The northwest pine shelf contains Orlis's Grass Gym and an inhabited cottage.
Two native downhill ledge blocks drop into the west/Itasca street; the ordinary
return climbs the short east ramp. At the native camera scale, trees close the
ledge ends and make the drop legible. The complete ordinary return is 29
walking steps versus the three-tile native hop displacement. No Cut tree is
placed, including incidental Cut quadrants hidden in ordinary canopies.

The north clinic and east residential/shop lane connect to the original
north/south civic spine. The lower west precinct contains a timber heritage
hall, shaded exhibit apron and caretaker. The shoreline turns east into a real
pier: a player can walk to its end, pass the fisher and return. Steel rail
collision closes both sides and native rock blocks close the tip. Only a
marked low launch farther south meets Surf water. The lake's east and south
edges have native blocking BUOY rocks; there is no unconnected Surf exit.

The lumber heritage glade identifies the Paul and Babe subject through its
caretaker, plaque and readable indoor bookshelves. A literal giant Paul statue
and blue ox are **not present**. No generic native monster, rock or trophy is
misrepresented as either statue. `landmark_art_contract` reserves two future
bespoke figure positions and requires native-camera review and real blocking
collision. That asset work remains open. The complete timber hall already
uses original Johto22/23/28/29 pixels and collision, copied into four appended
metatiles80–83. No source pixels or palettes are edited.

This is a town with safe public streets; it deliberately has no random-encounter
grass. Regional habitat remains on the four surrounding routes. Bemidji is not
a Glacier Line stop.

## Original regional ports

All values below are exact existing reservations, in metatiles. They are source
contracts pending Root's reciprocal regional registration; the authoring checker
does not claim a walk into a neighboring native map.

| Route | Side | Gate | Landing | Open outer core lane |
| --- | --- | --- | --- | --- |
| R08 Northern Lakes | South | 12,22 | 12,20 | 24/25,47 |
| R09 Iron Range West | East | 22,12 | 20,12 | 47,24/25 |
| R14 Headwaters | West | 1,12 | 3,12 | 0,24/25 |
| R17 Border Country | North | 12,1 | 12,3 | 24/25,0 |

No other walking tile reaches the exterior perimeter. No reachable lake water
touches an open exterior perimeter tile. Both R09/Q05 prerequisites and R17's
existing regional prerequisites must retain their authored semantics during
native integration; the city itself does not invent a bypass.

## Facades and reciprocal rooms

Coordinates below are 16px core tiles. Every visible native DOOR is assigned;
there are no decorative false doors. Each exterior active warp has a distinct
safe passive return directly below its door. Every room enters through passive
FLOOR warp3. The canonical gym currently returns to its separate review hall;
Root must rebind its two active carpets and passive entry destination to the
city door contract during the city publication, preserving all puzzle scripts. Two bottom carpet quadrants leave through room warps1 and2,
returning to the exterior passive warp, never to its active door.

| Exterior | Door | Safe return | Active / passive | Interior | Room arrival / carpets |
| --- | --- | --- | --- | --- | --- |
| Gym | 10,9 | 10,10 | 1 / 2 | MnBemidjiGym | 10,17 / 10–11,19 |
| Shelf cottage | 7,15 | 7,16 | 3 / 4 | MnBemidjiShelfHouse | 6,5 / 6–7,7 |
| Clinic | 31,9 | 31,10 | 5 / 6 | MnBemidjiPokecenter1F | 10,13 / 10–11,15 |
| North cottage | 33,17 | 33,18 | 7 / 8 | MnBemidjiNorthHouse | 6,5 / 6–7,7 |
| Mart | 42,19 | 42,20 | 9 / 10 | MnBemidjiMart | 6,5 / 6–7,7 |
| Heritage hall | 9,37 | 9,38 | 11 / 12 | MnBemidjiHeritageHouse | 8,9 / 8–9,11 |

The clinic and shop preserve their shared reserved interior dimensions and
arrivals. The current canonical gym is 11×10, arrival10,17; Root must reconcile
its older 12×12 shared planning reservation before city publication. The three new cottages/hall require Root to add shared interior
records. Each has explicit native rows, actor coordinates, source sprite,
facing, dialogue, carpets and passive return. All reachable room boundaries
are closed except their actual two carpet exits. Nurse and clerk have actual
COUNTER quadrants and reachable approaches. The clinic PC has an actual PC
quadrant and reachable approach; these source checks are not proof its native
service command has been lowered.

## Story, battles and services

Q05 officer `MN_BEMIDJI_OFFICER` remains at **21,19**, preserving the shared
actor anchor. `CompareHeadwaterReport` requires BADGE7 **and**
EVENT_MN_HEADWATERS_READING, then sets EVENT_MN_HEADWATERS_REPORT. Repeat dialogue
points to the mine. The raw reading alone must not open R09. The source checker
rejects a report contract lacking the independent reading. Root must verify
native refusal, successful comparison, once-only behavior and save/reload.

The shared `../gyms/interiors.json` gym7 is the sole native authority. The
city dossier's room maps its current mixed original Park/Gate/Champion glyphs
into the same sorted symbol IDs used by the native gym compiler. Its canonical
object SHA is checked, so a shared gym change fails this preview instead of
silently keeping an alternative. Initial and requirements-complete source views
are retained. The latest canonical layout deliberately makes Orlis physically
approachable; shared native script prerequisites enforce the plant-stage
challenge. The source view does not pretend that logical requirements are a
physical wall or that the puzzle has been played.

Orlis retains the exact story party and Grass/Pine Badge identity:
Tangela38, Jumpluff39, Bellossom40; requires BADGE6; win sets BADGE7 and
EVENT_MN_GYM_7_DEFEATED (native Johto badge index6). Loss must use normal blackout
without rewards; repeat conversation must not grant another reward. Native
ERIKA trainer class and the shared original CooltrainerM field sprite remain
exactly as declared in the canonical gym. The canonical reward is TM_GIGA_DRAIN.

The canonical room requires the SHOOT → BLOOM → SEED station sequence. moss
(Tangela35) and Elvren (Hoppip35/Gloom36) are required trainers; ash
(Weepinbell36) is optional. The three stations, guide at8,16, Orlis at10,3,
canonical requirements refresh at metatile5,1 and all three unique trainer/sequence flags are
mapped from that shared source. Source dialogue is wrapped for preview; the
canonical station prompt is separately retained verbatim. Native trainer sight,
wrong answers, cancellation, real battles/PP, ordered flags, once-only badge/TM,
blackout and saved puzzle progress remain to be verified by the gym owner.

Canonical healing, recovery registration, shop money/inventory, PC, story text
and trainer dialogue must flow through production `VisibleShellController`
commands. No frontend dispatcher, boating subsystem or alternate currency is
introduced. Root may reuse the existing Ecruteak music and shore fishing group;
all unrelated audio, source maps and data must remain preserved.

## Fast source loop and evidence

From the repository root:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_bemidji.py
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_bemidji.py --serve --port 8797
```

The watcher decodes the pack once, watches only the town dossier and layout,
and watches the shared gym dependency as well. It serves the full town, all
rooms, all 160×144 cameras and visible error state.
`--render-only` intentionally skips checks for a rejected artistic experiment;
its output must never be cited as passing geometry. A completed strict source rerender took 4.761 seconds before the canonical gym
preview replacement; current measured watcher timing is recorded in the
render report. This excludes the one-time pack decode. The default source pack is
`content-packs/minnesota-quality-review.crystalpack`, content hash
`5fe3071c6242e932096912bb3195982ff5ce7e5bd4f821a94985e9d652e50424`.

The strict checker validates both endpoints of all four original ports, every
actor and sign approach, all six DOOR quadrants, distinct exterior returns,
room carpets and passive arrivals, indoor exterior leakage, legal parties and
items, service counters/PC, ledge direction and return, no incidental Cut,
exact native timber art recipes, source camera positions, exactly two low
launch edges and native water-boundary closure. Deliberate negative cases
include false doors, displaced ports, unsafe exterior/room arrivals, wrong
returns, open lake boundary, bank/pier rail holes, reversed ledge, missing Q05
reading, changed timber palette, reused active return warp, canonical actor drift and
canonical puzzle-gate drift, a legal but noncanonical leader level and a wrong
station answer.

I inspected the authentic Johto/modern tile atlases and Goldenrod source town,
then the whole candidate, every room and 160px views including the widest
through segment. Rejected passes remain saved. Revisions removed a duplicate
mart road that created excessive paving, filled broad empty shoulders with
connected pine pockets, changed the heritage hall from another cyan facade to
native timber, narrowed the shelf ledge approach, opened its real landing,
closed surplus east boundary tiles and moved an NPC that trapped walkers on
a one-tile shore path. Native camera confirmation and literal statue art are
still outstanding; this source pass is not visual acceptance of the entire
Minnesota region.


## Frozen handoff and measured loop

Final exterior layout SHA256:
`bb51f6f46fcfd75d2dd3e2db482d6490d94889688f246f1072cc1626b236ae41`.
Historical initial city dossier SHA256 (including its canonical gym preview):
`864c040ada4894ce4fe1baa71ca1cef89307bcb141258031c295591b79717f4e`.
Historical canonical shared gym7 object SHA256:
`0b17e9a10a3068c78305fdaaee642e6df993907781fddc1b12ed0cc391fb70c4`.
Whole PNG file SHA256:
`f2e9dd8292267874cdab15068998963479fbbc872af75eed8572f220e0b43c1e`.

The final strict run passes 16 negative cases. A live cached-pack watcher was
exercised by touching the layout timestamp while preserving its exact bytes:
5.014 seconds from change to fresh report, 4.692 seconds rendering/checking.
HTTP page and JSON status were read back successfully. The historical review server used
`http://127.0.0.1:8797/`; this does not assert an active server. The immutable `watch-proof-*.json` artifact records this
measurement. The complete source/negative report is `source-review.json`, with
its own immutable content-named report beside it. Source snapshots are exact
byte copies named with their full SHA256, not reconstructed JSON.

At this historical source handoff, native lowering had not begun. The native
implementation below supersedes that status. Root retains shared registration
and publication ownership; all shared world, gym, interior registry, story and
actor files remain outside this worker's scope.

## Native implementation handoff (not yet published)

The focused implementation is now `crates/crystal-assets/src/bemidji_city_modpack.rs`
with exact source art in `bemidji_city_art.rs`; the public entry is:

```rust
build_bemidji_city_modpack(base, town_json, layout_json, canonical_gyms_json)
```

Root owns registration and publication. The base must already contain the
**current canonical** native Gym7. The builder reconstructs expected Gym7 art,
module and trainer parties in an isolated verification clone using the shared
Rust gym builders, and rejects any difference from the installed runtime
module before changing the city. A matching source hash alone is insufficient.
This checks the actual module/puzzle/scripts and parties, rather than guessing
from names or block counts. Only its three warp target records (including the
passive arrival's destination) and matching raw event wire data may change.
They point to the city's passive gym return2; all coordinates, source and
compiled puzzle/battle/menu tables, parties and rewards remain unchanged.

The exterior and five non-gym rooms reserve group70, denying collisions in
map names, native constants, group metadata, art, services and landmark. Four
appended metatiles80–83 retain exact original timber pixels and collision.
Original source tilesets/files remain intact. The compiled modern derivative
protects every originally used glyph/address/palette; only unused addresses
receive imported native glyphs.

The authored nurse lowers real `HealParty` followed by native `blackoutmod`
to the specified indoor clinic floor. The shop uses native `pokemart` with its
five authored items. The actual PC quadrant invokes native `PokemonCenterPC`.
All runtime/object/shop command tables are parsed. These are implementation
facts, **not verified service gameplay**. Production controller checks must
prove HP/PP/status recovery, blackout/Continue, money/item mutation, PC input
ownership and safe exits.

Two additive recovery records in the native addon range (identifiers >=256)
are deliberate. Ordinary town→clinic entry
already remembers the source town in the native engine, so the town has a safe
exterior recovery at31,10. After authored healing, `blackoutmod` remembers the
clinic's source arrival10,13. Every existing recovery record, including HOME,
remains unchanged. No player badge/item/story state is granted during build.

The officer remains at21,19 with ordinary supplied conversation. No new
CompareHeadwaterReport scene or report-flag grant is fabricated; Root must bind
the actual existing Q05 story separately. Regional connections also remain
unbound, and no Paul/Babe sprites are invented.

`crates/crystal-assets/tests/bemidji_city.rs` provides asset-level preservation,
real command-table lowering, unsafe source rejection, canonical installed-module
mismatch and occupied-group rejection cases. Without an override it constructs
the canonical gym fixture in memory from the completed external TUI pack; this
never publishes a pack file. Root may set `CRYSTAL_BEMIDJI_CITY_BASE_PACK` to an
already completed current gym pack. Rust formatting passes; Root's warm compile
and execution results must be recorded before calling these tests verified.

The canonical reference was refreshed after the shared source added explicit
room dimensions/arrivals/guide metadata. Every exterior and non-gym field was
compared and preserved. Current city dossier SHA is
`e9b52ce20c0aa539d0f8d56081a92d9c0601ce23a023fdaf2c9ab4261b728409`;
current selected canonical Gym7 object SHA is
`fbf4b769601297e303f9abbeaf23dd963588e59edfcb9a990a387f36d409f7af`.
The exterior layout and whole PNG hashes remain unchanged. Earlier frozen
hashes above describe the saved initial source handoff and remain reusable.


## Historical captured native input tuple (ae5824)

The newest canonical adaptation supersedes the earlier elite-only gym preview
and its metadata hashes. Shared gym source is read once and retained verbatim:
`../renders/bemidji/canonical-inputs/gyms-ae5824fbd466f1f14ab5454a085738a67fe18e5e77f9fe268013cba8fe25d788.json`.
Its selected Gym7 object SHA is
`f07eefcb74139791db1c5d5f626401d2656bb556c4ea41cd6d9c5bdd761f9378`.
The matching city dossier SHA is
`7780aa2383bbb575633f16ef8e6c88c82e4636e256ac0bd6bbdfc621d7a3ce79`.
Every exterior and non-gym field was explicitly compared and preserved.
The whole town PNG remains exactly the earlier file hash.

Use `review_bemidji.py --gyms` with that saved canonical JSON to reproduce the
resolved preview while other workers edit the live shared file. The default
continues to check the live shared authority. `CRYSTAL_BEMIDJI_CITY_GYMS_SOURCE`
selects the same immutable input for the native test fixture. Native publication
must consume the same captured strings that produced its base gym module.
The matching captured source review passes all 16 negative cases; latest full
render/check took0.905seconds after one-time pack decoding. All old source,
room/whole images and conversions remain retained. This is source evidence;
Root's native tests/captures remain required.


## Frozen three-city review candidate (current)

Root selected one common all-eight-gym input for the three-city composition:
`../renders/bemidji/canonical-inputs/gyms-b3f25b09e4ce114fd561ff28309464fb9e45500bb159dcb6d145ce2f06f7bbbb.json`.
This is a verbatim saved input, not a claim that the live shared authority is
frozen. Set `CRYSTAL_BEMIDJI_CITY_GYMS_SOURCE` to that exact file for tests.
Its selected Gym7 object SHA256 is
`73bb00cfbc3ed74c5b9a82d26d52f7d9a9cae895be830f2294e938350316376c`.
Matching city dossier SHA256 is
`4c11c7d89c73620009e57e8e9281e645638108c314d09c94075763c84e74411e`.
All exterior and non-gym fields were compared and preserved. The exterior
layout and whole PNG hashes above remain unchanged.

The resolved source preview passes all 16 negative cases; the last render/check
took 2.277 seconds after one-time pack decoding. All earlier input tuples,
source bytes, images and conversions remain saved. The canonical room uses
its actual mixed native source art, exact actor/station positions and script
contract; this worker does not substitute an alternative gym under its ID.

Root's native compilation passed. The first runtime test uncovered and repaired
a fixture path normalization error; the next uncovered the actual native
palette-map extent (224 entries, with every source glyph index checked).
The subsequent run passed the unsafe-source rejection test but blocked final
pack verification on additive recovery IDs28/29 outside Crystal's original
28-row spawn table. This is a concrete integration blocker. Existing HOME or
source spawn rows must not be repurposed to hide it. A supported native recovery
extension is under review before this candidate can be published.

The asset service regression now exercises actual native interaction lookup
through nurse/clerk counters and the PC quadrant. It applies parsed HealParty
and checks HP/status/PP recovery without changing money/bag/flags, and opens
the parsed mart, buys at the native item price, and rejects a purchase without
funds without mutation. This test still needs execution after the recovery
blocker is repaired. Production VisibleShellController input, page ownership,
blackout/Continue and PC surfaces remain separate required evidence.


## Native addon recovery support (execution pending)

The supported catalog now distinguishes the original Crystal source range
0..27 from additive native recovery identifiers256..65535. The gap28..255
remains invalid. The shared helper is
`crystal_core::systems::special_routines::is_runtime_recovery_spawn_identifier`;
both runtime catalog verification and `GameDataSet::runtime_spawn_point` use
it. Exact catalog membership, known map identity, bounds, source coordinate
agreement and walkable arrival requirements remain mandatory. Original HOME0,
all source spawn records and source table export count remain unchanged. Fly
validation still accepts only original source table identifiers.

Bemidji allocates two catalog records above the original byte range without
replacing any source record. The fifth asset regression exercises real native
Teleport preparation/commit and blackout mutation to both destinations. It
checks safe actual arrival positions, saved catalog identity, full recovery,
native blackout money loss, sparse Safari cleanup, unchanged original HOME
fallback and production Fly parser rejection of addon IDs. The core catalog
regression retains the previous rejected28 case and tests the two permitted
ranges plus reserved gap. These new checks await Root's warm execution and
Clippy before publication.

The current captured preview is available at `http://127.0.0.1:8797/` while its
review process runs. Its HTTP status was read back against the exact b3f input
and4c11 dossier:16 negative cases passed,0.875-second warm render/check. This
preview remains source evidence, independent of native controller acceptance.

Root's executed native core catalog regression now passes all7 tests
(`.local/addon-recovery-core-tests.log`), including the additive namespace,
reserved gap and existing coordinate/token validation. Native asset service,
Teleport and blackout execution remains pending.


## Production-controller regression handoff (not yet executed)

`crates/crystal-bevy/src/bevy_shell/tests/bemidji_city.rs` contains7 external-pack
controller cases. Root owns module registration, Cargo and publication. The
fixture supplies only initial location, legal partners/injury, six prior badges,
money and deterministic RNG. Every later action uses production button input,
frame settling, save and Continue. No post-setup state/position/menu/battle/flag
write appears in these cases. The fixtures do not claim an earned campaign.

Coverage: all6 actual facades and both exit carpets, stable passive arrivals,
HOP_DOWN/reverse wall/ordinary ramp return; nurse HP/status/PP restoration; real
PC deposit and withdrawal; item price, quantity, bag/money mutation and shop
closure; fixed officer conversation without fabricated report progression.

The recovery case starts outside the clinic and enters its actual facade,
verifying the production warp remembers the town. One branch leaves without
healing; the other uses the authored nurse and verifies `blackoutmod` changes
the saved destination to the clinic floor. Both branches walk through town to
the real canonical moss trainer, play a losing battle through FIGHT/SPLASH,
observe real PP consumption and native loss, then require the correct recovery
map/coordinates, full HP/status/PP recovery, money loss, no victory flag/badge,
safe passive arrival and preserved Continue state. A separate case uses
Start→Pokemon→Abra→TELEPORT with actual A-menu input and checks the clinic
arrival/retained inventory and money. No direct battle-start/loss/warp/Teleport
engine call substitutes for these production inputs.

After Root publishes the matching captured candidate, the intended command is:

```sh
CRYSTAL_BEMIDJI_CITY_CONTROLLER_PACK="$PWD/content-packs/minnesota-cities-review.crystalpack" \
  cargo test --locked -p crystal-bevy --lib bemidji_city_controller_ \
  -- --ignored --test-threads=1
```

This command and its7 cases remain unexecuted at handoff. Shared Gym station
schema drift currently rejects the frozen common input before native city
fixtures can build; Root is reconciling that builder/input contract.


## Current isolated authority epoch887afc43 (live dossier untouched)

Root selected a new exact shared-authority capture:
`../renders/bemidji/canonical-inputs/gyms-887afc43d19bc0573ca5ec361f4c141771347421040f70f100f02fea4997a189.json`.
Its Gym7 object SHA256 is
`4ea1bea34bc4a614f2d5f91b248cc0cf06439395d2f2a9ec2020defab2e3b463`.
The matched city candidate is a separate immutable file,
`../renders/bemidji/canonical-inputs/MnBemidji-c9cc0571747fa1b43b399ae3d23573ac8f66c791e58a7debb0c181cd5dfcb571.json`.
Its SHA is the full hash in that filename. The live town dossier remains4c11;
this adaptation never overwrote it. Every exterior and non-gym field was deeply
compared and preserved. All prior b3f input bytes/images remain reusable.

Current authority Gym7 contains zero stations, not sprite-less station events.
Its guide/gate refer to MOSLEN and ELVREN; its true actor set is leader, guide and
three trainers. The separate candidate removes the three obsolete paper
residents and reflects that exact canonical contract. Winding beds are now
unmarked, and the earlier SHOOT/BLOOM/SEED interaction is absent from this
epoch. No paper actor, readable marker, station or replacement puzzle was
invented to hide that gameplay/fidelity limitation. The canonical geometry,
leader/guide/trainer anchors and all five non-gym rooms remain unchanged.

Isolated previews are under `../renders/bemidji/epochs/887afc43/`. The full gym
was personally viewed without paper bodies. All16 source negatives pass,
including rejection of an invented station body; render/check took0.717s.
The whole town and all five non-gym room PNGs are byte-identical to the prior
epoch. The source-review report SHA256 is
`f8ecb934215a074fe78fbffde5f965bb5faa12d58a069a365e1e7217afea8723`.
A separate captured-input preview server uses `http://127.0.0.1:8798/`;8797
continues to refer to the older b3f input. Neither source preview proves play.

`review_bemidji.py --gyms ... --design ... --output ...` now supports review
without modifying any live city file. The native asset fixture accepts
`CRYSTAL_BEMIDJI_CITY_TOWN_SOURCE` for the immutable city candidate alongside
`CRYSTAL_BEMIDJI_CITY_GYMS_SOURCE` for the exact authority snapshot. Production
builder behavior is unchanged. The seven production-controller cases use
unchanged geometry/trainer anchors and do not depend on the removed puzzle.


## Executed native assets and controller audit

Root's frozen887/c9cc native asset run (`.local/cities-two-887-tests.log`)
passed4 cases: source/art/audio/HOME preservation, real healing/shop/PC
interaction semantics, malformed-source rejection, and strict installed-gym
mismatch/collision rejection. The fifth executed native town recovery/Teleport
assertions before failing on a test-only assumption that the supplied source
pack had a Fly row. No gameplay or recovery failure was established there.

That test now exercises the unchanged production Fly payload parser with an
isolated schema fixture, accepts original sourceIDs0/27 and rejects28/255,
each actual addon recovery ID and65535. It never installs a live Fly row or
changes the source pack. The five-case rerun is pending. Root separately
reported strict native city publication/composition passing; production
controller acceptance remains pending.

The controller audit strengthened readiness to wait for blackout and field
travel phases/animations, since lower-level pending script work does not own
those presentation clocks. The loss fixture now starts with a legal burn.
The no-nurse branch must retain it before battle; only the nurse branch cures
it before battle. Actual battle turns must damage HP and consume Splash PP.
Both recovery branches must then restore full HP/status/PP, and Continue must
preserve the entire typed party/storage, money, flags, badges and recovery
memory. Every state write remains confined to initial fixture setup.

Native art SHA1e5c3e4d… and builder SHAc2651c20… remain unchanged from the9a87
frozen bundle. Root's renderer aliases for flower/water animation use native
logical glyphs03/14, whose source pixels and palettes are explicitly preserved
by the derivative. No original native source art was edited.


## Split-source Gym7 mirror correction (awaiting republish)

A subsequent read-only audit of Root's written498 revision found a concrete
source-preservation bug: typed Gym7 events and module event commands returned
to Bemidji, but `map_scripts[MnBemidjiGym_MapEvents]` still returned to the
review hall. The old pack and its audit remain retained. A strict runtime pack
verification pass alone did not establish split-source mirror consistency.

The focused Bemidji builder now calls the existing
`insert_compiled_map_split_payloads` after the validated Gym7 rebind, matching
the other city builders. The preservation test requires the original global
event section to change only the3 warp target-map/target-index pairs, requires
exact equality with the module event wire, and preserves every other original
map-script label and NPC payload. It then removes only cached Gym7 in an
isolated data clone and reassembles from actual split payloads, comparing all
three returns and event commands. This deliberately avoids the assembly API's
cached-module fast path. No source art, map geometry, party, reward or script
behavior changed. Root owns a new immutable publication after checks pass.

Current builder SHA256:
`f809e932aad6d5c4656d7619a55206ea010b7a0cca226b32ef9a1a0569a20a93`.
Current native asset test SHA256:
`a376b097b04ceed02e11d78b45c1e716a467df2f870e692e6ca6e5091784ad6d`.
Current audited controller test SHA256:
`a4ef23f0df7b544b84897ab6f1e1eb05997a2c737a0d862ed8acbe6da860a033`.
These hashes identify source ready for Root's next execution, not a completed
controller/playable claim.

## Four-seam review contract (planned only)

[The actionable connection review](../reviews/bemidji-seam-contract.md)
records the actual cropped R08/R09/R14/R17 neighboring chunks, reciprocal
metatile offsets, both walking lanes, native edge arrivals, existing internal
links, authored safe anchors, and the required world-source reconciliation
from `gatehouse_warp` to the proposed walking connections. Exact story checks
remain Badge6 for R08/R14, Badge7 for R17, and Badge7 plus completed headwaters
report for R09. MS06/MS07 remain unbound; the fixed officer does not grant a
replacement report.

The inspected pack's actual file SHA256 is
`6e3295367d6f2b4f6b07ef8f95f403968f2dcd3fe4790991b6e5f7380ef879af`;
its internal content identity beginning `10b50e` is not a file hash or new
publication. Static native collision checks pass each proposed two-lane inner
checkpoint against actor-free Surf bypass and retain access from both inside
lanes to all six doors, officer approach and pier when eight proposed guard
bodies plus original NPCs block. The retained static proof SHA256 is
`72cf669b68d5ee7bc0a0c428014ee520543308d49b9c43b71af667bc43cb5c77`.
It does not execute native connection or checkpoint behavior. Sixteen real
production crossings, denial/escort/reentry/Continue and source-preservation
acceptance are specified for the future connection layer. No map, pack,
shared source, or Rust implementation changed for this review.

## Executed controller diagnostics and current repairs

Root's actual combined city run
`.local/minnesota-cities-controller-887-review-profile.log` executed all seven
Bemidji cases: ledge/ordinary return and fixed-officer dialogue passed. Four
cases reached their final Continue assertions before failing because the
fixture set trainer-card badges without their engine aliases, and Continue
canonicalizes explicit false versus absent flag entries. The recovery-loss
case failed its combined PP/damage oracle and remains unproven.

The corrected initial fixture reconciles both representations of its six prior
badges before any production input. Continue compares the complete sets of
true event and engine flags while retaining exact money, badges, typed party,
storage, bag, position and recovery-memory checks. This still detects every
true-to-false or false-to-true change; it allows only semantically equivalent
sparse false storage.

The loss oracle now also observes the authoritative active scripted-battle
Pokemon, whose HP and moves can change before the overworld party projection
commits; whiteout subsequently heals the party. It still requires actual native
attack selection, observed Splash PP consumption and HP damage, a real losing
result, money loss, full recovery and Continue. The original legal level45
burned Magikarp fixture is retained. A failed rerun reports which observation
was absent and the lowest actual combat HP. No state is written after setup.
Current controller source SHA256:
`895e098705b3ce9001854149f8f54a00d459ba9530000208d931a0eedea2428c`.
Root owns compilation and execution of these repairs; passing is not assumed.

## Prairie rustle read-only audit and focused regression handoff

The current prairie source palettes expose a real color mismatch: static
encounter grass uses background palette7 while the prior grass-rustle loader
used NPC palette6 independently. Source-pixel reconstructions show green
day/morning rustle, blue night rustle and black dark rustle on yellow prairie.
The actual retained background palette7 remains gold/brown in all four groups.
These [bounded comparisons](../renders/bemidji/audit-prairie-rustle/comparison-source-reconstruction.png)
are labeled source reconstructions, not production captures. The exact encoded
source PNG, original decoded image, frames, composites, all resizes and hashes
remain retained. No pack or renderer source was edited by this worker.

Root owns the loader/caller/cache repair. The new focused test leaf
`crates/crystal-bevy/src/bevy_shell/tests/prairie_grass_rustle.rs` tests exact
regional key selection; actual sourcePNG-derived RGB counts against native
NPC6 or regional BG7 through all four times; unchanged alpha/OAM geometry;
native→both regional variants→native cache stability; and genuine missing-
asset error isolation. The native fixture verifies the pack, constructs and
retains its runtime, and materializes its owned presentation mount. Two cases
require the external pack and are ignored by default; the key contract needs
no external assets.

`CRYSTAL_PRAIRIE_GRASS_RUSTLE_TEST_PACK` selects the pack (default current
cities review). Optional `CRYSTAL_PRAIRIE_GRASS_RUSTLE_CAPTURE_DIR` writes24
actual production-function frame images: unscaled18×9 RGBA originals,
native plus both prairie aliases × four times × two frames. Names contain
the encoded PNG SHA256; exclusive creation rejects conflicting bytes. No
conversion or overwrite occurs. Captures and test success remain pending
Root's execution. Current rustle test SHA256:
`7fac271692b8b9f602e2ea0abf2507f844bf8c411a4253d2fea7be9b556e031d`.
