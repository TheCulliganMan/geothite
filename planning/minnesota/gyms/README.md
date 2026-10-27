# Minnesota gym interior implementation

The user rejected the planter revisions, including the later solvable zigzag.
The current first-gym candidate is [Rochester's two-switch lane course](rochester-gates.md):
open the left lane, beat Elvik and open the upper gate, then return to change
to the right lane. The gold gates visibly change; ordinary walls stay fixed.
Its literal control run and native closed/open views pass.

[current.json](current.json) selects [rochester-gates.json](rochester-gates.json)
for source review. The matching current town binding is
`../towns/MnRochester.gates.json`; world/interior reservations use the updated
extent and indexed arrival. Historical gym/town sources remain unchanged for
older pinned publishers. The other seven need the same design review.
Earlier implementation evidence below does not constitute user acceptance.

Names are original: leaders, trainers and other newly named characters must
not reuse real people or existing Pokémon characters. Minnesota city names
remain the geographic setting. Anonymous roles such as a guide or warden are
fine; native trainer classes describe roles, not character identities.

All eight original-style remakes now have source/native visual review and
production-controller evidence. The current combined 480-map room pack passes
all eight courses. These are revised candidates, not inferred user approval;
earlier rejected renders remain archived. The current 541-map story successor
retains all eight current rooms with real city facades and passes one
production suite covering every local gym course. See [the combined candidate](../playable/eight-gyms.md).
The continuous earned regional journey is still incomplete.
The [River Valley successor](../playable/river-cities.md) now proves one
continuous second-to-third gym course through the actual R03 walking connection,
with explicit initial first-badge/strong-party fixtures.
The authoritative design and stage contract are in [GYMS.md](../GYMS.md).
The [fourth-gym approach](../playable/gym-four-approach.md) now connects
Minneapolis through R04 to St Paul's real facade. Its checkpoint requires the
Current Badge alone. Dispatch is checked by the courier in the later story chapter. One continuous local course
earns the rescue and third/fourth badges through both puzzles and trainer
sequences, with native clinic care and return travel. Prior badges/party are
initial fixtures. The separate courier, lake-country and Leech shore chapters
now extend regional progression; the full earned campaign remains unfinished.

Mankato now has a separate native city candidate with real clinic, mart and
Gym2 doors. Its local production course earns Caldren, Jorel and Torven's wins,
badge2 and one TM through the facade and both returns. Native city ledge, care,
PC access, priced shopping and Continue also pass. See the exact contracts,
fixture limits and executed commands in [the Mankato runbook](../playable/mankato.md).
Rochester's first-gym city binding is independently verified in its opening
story pack. St. Paul's 488-map successor now proves the native gym facade,
all pad pairs, services and reciprocal R04 crossing; its courier and rail
remain unfinished. See [St. Paul](../playable/st-paul.md). The 492-map
[St. Cloud successor](../playable/st-cloud.md) now verifies the city facade,
complete wind course, services and reciprocal R05/R06 crossings. These cities
are now merged with the opening/northern story and current gyms in the
538-map successor. Missing links and continuous regional progression remain.

`interiors.json` contains explicit room rows and native metatile pixels from
the external pack. Wind slats explicitly replace source collision with native
directional walking; other source collision stays unchanged. Rust composes the selected art and lowers
battles, saved screen changes, badges and retryable TM rewards. Source graphics,
game catalogs, generated packs and renders remain external or ignored.
Control interactions in the later machinery drafts use background events on
real fixtures; there are no Pokédex or paper objects scattered around the rooms.

Each focused native review pack includes one gym and a small `MnGymReviewHall`
vestibule with a reciprocal door. It preserves source maps, HOME, recovery, items, species and audio.
It does not publish the eight city exteriors or claim earned regional travel.
City integration must bind each gym to its actual facade and prove both ways.

## Rochester review loop

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_gyms.py \
  --gym MnRochesterGym \
  --output planning/minnesota/renders/gyms/rochester-og

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_minnesota_gyms -- \
  content-packs/minnesota-boundary-basecamp-connected.crystalpack \
  planning/minnesota/gyms/interiors.json \
  content-packs/minnesota-gyms-rochester.crystalpack 1

.local/minnesota-review-env/bin/python planning/minnesota/tools/review_gyms.py \
  --pack content-packs/minnesota-gyms-rochester.crystalpack \
  --gym MnRochesterGym --verify-published \
  --output planning/minnesota/renders/gyms/rochester-og
```

The final `1` exports only gym 1. The verified pack contains 473 maps: the
471-map base, Rochester and its 5 × 5 review vestibule. The other seven drafts
stay out of this review build. Source previews use the native indoor palette.
The source/native comparison checks every terrain pixel and collision quadrant.
Every render is archived, including rejected passes.

Native captures are headless. For example, using the existing production
renderer executable:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576 \
  target/gym-controller/wasm-dev/examples/render_at_location \
  --pack content-packs/minnesota-gyms-rochester.crystalpack \
  --map MnRochesterGym --x 6 --y 17 --view 2d --hour 12 \
  --screenshot planning/minnesota/renders/gyms/rochester-og/native-entrance.png
```

The controller fixture's level-100 Mew and prior badges are explicit test setup.
They do not prove ordinary-party balance or earned city progression. Gameplay
verification must use `VisibleShellController::press`, including movement,
text, selected attacks, TM use, room returns and Continue. A screenshot or a
successfully loaded pack does not establish playability.

## Rochester verification

The current room is a static native planter maze. Its focused controller check
passes. It walks to Elvik and Varel before qualifying and requires refusal, beats Tavel,
saves and reenters, requires another leader refusal, beats Elvik and Varel, and
checks both carpets, Continue and the single Headbutt reward. Real selected
attacks must consume PP. Battle snapshots must show Tavel, Elvik and Varel with
generic trainer classes. The earlier pressure-mat course is a rejected version.

Four focused checks pass: the full route, practice-trainer loss, parcel refusal
and leader loss, and the full-TM-stack retry. Strong
party and completed-practice fixtures isolate those cases; they do not prove
ordinary-party balance. Source/native terrain and collision comparisons plus
headless player-camera captures pass for this layout. The 473-map published
pack preserves all 471 base maps, source files, trainers, catalogs and audio.
The plan checker passes 160 map contracts; this is symbolic progression, not
an earned trip through the region.

[Current complete-room and native-camera review](../renders/gyms/rochester-og/Rochester-review.png).
The name check also rejects authored gym names present in the source trainer
catalog. Scoped lint passes with existing `never_loop` findings allowed and
no new warnings in changed modules. Full lint remains blocked by the existing
core bit-mask comparison and voxel approximate-constant errors; full formatting
reports unrelated workspace changes.

To build the focused controller test with the pinned toolchain:

```sh
RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib minnesota_rochester --no-run
```

Run it against the published external pack:

```sh
RUST_MIN_STACK=67108864 \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-gyms-rochester.crystalpack" \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/minnesota-gyms-rochester.crystalpack" \
  target/gym-controller/wasm-dev/deps/crystal_bevy-eef222887c5cf4d6 \
  minnesota_rochester_controller_planter_maze_requires_both_trainers_and_persists \
  --ignored --nocapture --test-threads=1
```

The test executable hash is the currently verified build, not a stable interface;
use the executable printed by Cargo if dependency changes produce another hash.
Changed leaf Rust files pass formatting checks. Full asset, controller and runtime Clippy checks hit an
existing `bad_bit_mask` error in `crystal-core/src/multiplayer/mod.rs:1332`;
repository-wide formatting also reports existing changes outside this work.
Scoped asset Clippy passed with dependencies excluded and the existing
`never_loop` findings allowed; this is not a clean full-workspace lint result.

## Mankato review loop

Mankato is the third original-style remake. It uses the original Pewter gym's
wooden floor and low rock boundaries, with three native one-way banks, a west
stair, Jorel's area and Torven's upper court. The old quarry remains a rejected
visual version.
Native entrance, west-bank and court captures have been inspected. The
source/native comparison passes for every terrain pixel and collision
quadrant. Publication rejects false hop landings, ordinary dirt substituted
for a ledge and a return contract placed away from the actual ascent.

Publish only Mankato and its small review vestibule:

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_minnesota_gyms -- \
  content-packs/minnesota-boundary-basecamp-connected.crystalpack \
  planning/minnesota/gyms/interiors.json \
  content-packs/minnesota-gyms-mankato.crystalpack 2

.local/minnesota-review-env/bin/python planning/minnesota/tools/review_gyms.py \
  --pack content-packs/minnesota-gyms-mankato.crystalpack \
  --gym MnMankatoGym --verify-published \
  --output planning/minnesota/renders/gyms/mankato-og
```

Run its focused production-controller checks:

```sh
RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-gyms-mankato.crystalpack" \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/minnesota-gyms-mankato.crystalpack" \
  cargo --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib minnesota_mankato -- \
  --ignored --nocapture --test-threads=1
```

Gym controller tests require an explicit current pack; they do not silently
load an older eight-room draft. The level-100 Mew, prior badge and initial room
placement are fixture setup. Only Rochester's fixture seeds its parcel flag;
an isolated Mankato pack does not register that unrelated prerequisite.
The missing-first-badge and one-HP Magikarp fixtures test refusal and loss.
The west-bank capture seeds Caldren's flag for visual review; the controller test
must independently earn that win. The court capture uses explicit placement
in the initial closed-stair state.

The preservation/publication test passes with the updated native walking graph
and the isolated Mankato builder. Rochester's full staged controller regression
also passes with the native-hop path planner. All three Mankato controller tests
pass against the current wood-and-rock pack: the course, real one-way drops, shorter
court return, both required trainer wins, Torven's badge and single TM, repeat
protection, both carpets and Continue; loss without opening the lower stair;
and refusing Torven without the first badge after the two trainer wins. Battles
select actual moves and verify PP consumption.

All 471 original maps and runtime files, source item/species/field catalogs and
HOME/recovery records match the delivered 473-map pack exactly. The complete
377-program PCM/hash/frame-count/loop check passed before the original-style art
revision. The final pack's compiled audio, manifest and compression fields
are identical to that verified pack; the art revision did not change audio.

The changed leaf Rust files pass formatting. Full Clippy still stops on the
existing `bad_bit_mask` error at `crystal-core/src/multiplayer/mod.rs:1332`;
workspace formatting reports unrelated existing differences. Scoped asset and
controller Clippy passes with dependencies excluded and the existing `never_loop`
findings allowed. The two warnings in the owned test helper were fixed; other
pre-existing warnings remain. None of these lint results establishes a clean
full-workspace check.

[Current complete-room and native-camera review](../renders/gyms/mankato-og/Mankato-review.png).

The original-style 8 × 10 revision passes all three focused controller checks
and the all-room publication/preservation check. The west-bank test also rejects
a deliberate right-end bypass. Native court/west-bank images seed trainer flags
only for visual review; the controller separately earns those victories.
Original names and generic Camper, Picnicker and Hiker classes are asserted in
actual battle snapshots. Changed Rust files pass formatting. The full lint
failures remain the existing core bit-mask and voxel approximate-constant errors;
scoped tests lint passes with the existing `never_loop` findings allowed.

## Minneapolis review loop

The current room is a 5 × 9 original-style electric gym. Both required trainer
wins enable a fixed adjacent switch pair. A wrong second choice clears only
the two private switch flags and restores both barriers. Completed circuits
stay open. Six solid native cans use background events; there are no menu
choices, book objects or invisible floor controls.

[Current complete-room and native-camera review](../renders/gyms/minneapolis-og/Minneapolis-review.png).

Build the individual gym, then bind it to the actual city facade:

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_minnesota_gyms -- \
  content-packs/minnesota-boundary-basecamp-connected.crystalpack \
  planning/minnesota/gyms/interiors.json \
  content-packs/minnesota-gyms-minneapolis.crystalpack 3

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_minneapolis -- \
  content-packs/minnesota-gyms-minneapolis.crystalpack \
  planning/minnesota/towns/MnMinneapolis.json \
  planning/minnesota/towns/MnMinneapolis.layout.json \
  planning/minnesota/story.json \
  content-packs/minnesota-minneapolis-gym-remake.crystalpack
```

The individual pack has 473 maps; the city pack has 478. All 471 original maps,
source files/trainers/catalogs, original HOME/recovery records and compiled
audio are preserved. The city adds five maps and rebinds only the gym's three
warps and landmark. `native_content_sha256` pins every other gym-module field, including
mirrored native commands; only `warp_event` records and `events.warps` are
excluded. The city preservation test accepts the same room in a full-eight-gym
fixture and rejects stale pins and premature story milestones.

Four standalone controller checks pass against the individual pack. The city
facade check passes against the city pack and earns both trainers, switches,
badge and one TM, saving inside the first open barrier and using both carpets.
All four existing city gameplay checks pass, including its separate battle,
rescue, note, loss and native services. Initial test party/prior badges/placement
are fixtures. These checks do not claim a trip earned from an earlier city.

Use the shared Cargo configuration above when rebuilding the controller tests.
Run `minnesota_minneapolis_controller` with `CRYSTAL_GYM_TEST_PACK` set to the
absolute individual pack path. Run `minnesota_minneapolis_city_controller` with
that variable set to the absolute city pack path. Existing city tests use
`CRYSTAL_MINNEAPOLIS_TEST_PACK` and the exact namespace filter
`bevy_shell::tests::minneapolis::minneapolis_controller_`. Set
`CRYSTAL_RENDER_TEST_PACK` to the same selected pack and `RUST_MIN_STACK=67108864`.
All are ignored external-pack tests and require `--ignored --test-threads=1`.

Source/native terrain and collision comparisons pass. Native locked, first-
switch and solved views use the same `(3,8)` camera and noon indoor state.
Their trainer/switch flags are explicit visual fixtures; controller tests earn
the wins and switch states separately. The native facade view uses `(42,24)`.
Every render, including rejected versions, remains archived.

Changed leaf Rust files pass formatting and scoped lint with the existing
`never_loop` findings allowed. Full lint remains blocked by existing core
bit-mask and voxel approximate-constant errors; workspace formatting reports
unrelated existing differences. Packs and render output remain ignored.


## St. Paul review loop

The current 7 × 10 room borrows the original Saffron gym's native floor, walls
and pads. Four pairs connect the foyer, Belva's room, Orven's room and the court.
Either trainer can be first. The early court route reaches a closed grille and
allows return by the same pad. Both wins release the grille; every pad remains
usable. The earlier large galleries are a rejected visual version.

[Current complete-room and native-camera review](../renders/gyms/st-paul-og/St-Paul-review.png).

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_minnesota_gyms -- \
  content-packs/minnesota-boundary-basecamp-connected.crystalpack \
  planning/minnesota/gyms/interiors.json \
  content-packs/minnesota-gyms-st-paul.crystalpack 4

.local/minnesota-review-env/bin/python planning/minnesota/tools/review_gyms.py \
  --pack content-packs/minnesota-gyms-st-paul.crystalpack \
  --gym MnStPaulGym --verify-published \
  --output planning/minnesota/renders/gyms/st-paul-og
```

Four production-controller checks pass: the full course, reverse trainer order,
loss and missing badge 3. The course enters the court early, proves blocked
movement and safe return, then earns both battles. It rides every pair both
ways, saves on pads without bouncing, wins Brannic's battle with real PP
consumption, verifies one Steel Wing reward and uses both exit carpets.
Prior badges, test party and initial placement are fixtures; the play does not
claim an earned regional journey.

Rebuild/run the checks with the shared Cargo configuration and prefix
`minnesota_st_paul`, using the absolute individual pack path in both
`CRYSTAL_GYM_TEST_PACK` and `CRYSTAL_RENDER_TEST_PACK`, `RUST_MIN_STACK=67108864`,
and `--ignored --test-threads=1`.

The source/native terrain and collision comparison passes. Native entrance,
Belva-room and closed/open court views have been inspected. The court pair
uses `(11,6)`, noon and the same facing. The open view seeds the two trainer
flags as a visual fixture; the controller separately earns both victories.
The actual 473-map pack preserves all 471 source maps, files, original
trainers/catalogs, HOME/recovery and compiled audio.

The St. Paul source town retains its exterior and three ordinary service rooms.
Its canonical gym reference now carries the correct rows/hash/allocation and
carpet anchors. `review_st_paul.py` passes its source checks and 22 semantic
negative cases. Future native town lowering must bind only records 1–3 to city
return 10 and preserve all eight internal pad records 4–11. Town/services/depot
publication and actual facade round trips remain pending; the source review is
not a substitute for those checks.

Changed leaf Rust formatting and scoped lint pass with existing `never_loop`
findings allowed. Full lint remains blocked by existing core bit-mask and voxel
approximate-constant errors; workspace formatting reports unrelated differences.
Packs and every native/source render remain ignored and retained.


## St. Cloud review loop

The 9 × 11 remake uses the original Flying gym's blue stone and raised edges.
Four dark slatted lanes retain the original explicit forced-walking exception:
source `tower:0b` pixels with native `WALK_RIGHT`/`WALK_LEFT` in every quadrant.
Ordinary stone never hides a gust. Both pairs use upper-east/lower-west flow.
Wyrel and Hallen are required before Saven; both return lanes stay available.

[Current complete-room and native-camera review](../renders/gyms/st-cloud-og/St-Cloud-review.png).

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_minnesota_gyms -- \
  content-packs/minnesota-boundary-basecamp-connected.crystalpack \
  planning/minnesota/gyms/interiors.json \
  content-packs/minnesota-gyms-st-cloud.crystalpack 5

.local/minnesota-review-env/bin/python planning/minnesota/tools/review_gyms.py \
  --pack content-packs/minnesota-gyms-st-cloud.crystalpack \
  --gym MnStCloudGym --verify-published \
  --output planning/minnesota/renders/gyms/st-cloud-og
```

The three `minnesota_st_cloud` controller checks pass against this pack. The
course proves both required battles, live forced movement, opposing input,
empty-frame settling, wrong-direction attempts, both return lanes, saving,
Saven's badge and single TM, repeat protection and both carpets. Loss and
missing badge 4 grant nothing. The battle snapshots assert the original names
Wyrel, Hallen and Saven with generic classes. Prior badges, party and initial
position are fixtures; earned regional travel is not claimed.

Rebuild/run with the shared controller Cargo configuration, absolute pack path
in `CRYSTAL_GYM_TEST_PACK` and `CRYSTAL_RENDER_TEST_PACK`,
`RUST_MIN_STACK=67108864`, and `--ignored --test-threads=1`.
The source/native pixel and collision comparison passes. Native captures use
`(4,19)` at arrival, `(7,14)` beside the first gate and `(12,10)` beside the
second. Review flags seed the appropriate trainer wins only for imagery;
controller tests separately earn them. The screenshot tool validates the
initial map before applying preview flags, so gate tiles themselves cannot be
used as initial camera positions. Every render remains archived.

The actual 473-map pack preserves all 471 source maps, files, original trainers,
HOME/recovery, catalogs and audio bytes/manifest/compression. The all-room
publication/preservation check passes, including rejection of wind into walls
and forced movement hidden on ordinary planks. Changed leaf formatting and
scoped lint pass with the existing `never_loop` findings allowed. Full lint
still hits the pre-existing core bit-mask and voxel approximate-constant errors;
workspace formatting reports unrelated existing differences.

Canonical city metadata, source hash, allocation, viewpoints and carpets are
synchronized. The canonical source phases reach 44 / 105 / 156 resting tiles
at arrival, after Wyrel and after Hallen. The current non-gym town fields,
exterior layout and station are unchanged by this revision. The older
whole-town reconciliation baseline already differs from current source in
non-gym fields and the exterior layout; this task leaves that historical
snapshot intact and checks the canonical gym separately. Native town/services,
facade binding, rail and regional travel remain pending. Future lowering must
retarget only the three review-hall records to the city's safe return record 6.

## Brainerd review loop

The previously verified 5 × 8 pool gym uses the original Cerulean gym's native
Port water, brick piers and perimeter deck. Lorla's first victory enables her
repeat-interaction pier control. Choose CENTER to reach Bravel; after both wins,
choose LEFT to reach Sivren. The other pier closes, so both choices remain
meaningful. The circuit requires both trainer wins and badge 5 before Sivren.

[Native pier comparison](../renders/gyms/brainerd-og/native-pier-comparison.png).

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_minnesota_gyms -- \
  content-packs/minnesota-boundary-basecamp-connected.crystalpack \
  planning/minnesota/gyms/interiors.json \
  content-packs/minnesota-gyms-brainerd.crystalpack 6
```

The retained four `minnesota_brainerd` controller checks cover the complete
course, loss, missing badge 5 and a later poolside placement that cannot bypass
the trainer requirements. They earn the battles with real PP use, change/retract
native collision, save on both pier states, use both carpets and verify a single
Rain Dance TM. Initial party, prior badges and placement are explicit fixtures.
The native pool-edge bounds are checked with actual directional inputs.

Arrival is `(4,14)`, both carpets `(4,15)` / `(5,15)` exit, and the guide is
`(7,14)`. Lorla `(2,10)` operates the native pier menu; Bravel `(2,7)` leaves the
walkway clear after defeat; Sivren is `(5,3)`. All Port pixels and collision
recipes stay native, including the brick's walkable LADDER collision with no
extra ladder warp. An enclosed-pool exception permits proper poolside water
access while preserving the bounded dry perimeter and empty connection list.

The standalone room is the canonical source for the separate Brainerd city
binder. Its preservation/rebinding tests were verified in the earlier pool
remake; a city pack must consume the current full-gym source and preserve the
canonical actors, party, reward and pier callbacks. This section records that
verified milestone, not a new earned regional playthrough.

## Original cast and completion status

Every authored gym leader and practice trainer uses an original name and a
generic native class/sprite. Publication tests reject names already present in
the source trainer catalog; battle snapshots assert the current authored names.
Original source-game maps, trainers and historical reference images stay intact.
All eight rooms have original-style remakes. Remaining city/aggregate integration
needs its own proof.
Earlier green gameplay tests and saved screenshots do not establish visual
approval of an unrevised room or completion of the whole eight-gym objective.


## Bemidji review loop

The 6 × 10 Cut garden is reviewed against the original Celadon and Azalea gyms.
It uses native lawn/flower/wood tiles and native Kanto tree32→6d, remapped to its
private art IDs without altering pixels or collision. Two required cuts open
separate courts; every other possible Cut quadrant is rejected. The guide
supplies HM01 once, so a previous optional quest is not a hidden prerequisite.

[Complete review](../renders/gyms/bemidji-og/Bemidji-review.png) ·
[Matched native Cut views](../renders/gyms/bemidji-og/native-hedge-comparison.png).

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_minnesota_gyms -- \
  content-packs/minnesota-boundary-basecamp-connected.crystalpack \
  planning/minnesota/gyms/interiors.json \
  content-packs/minnesota-gyms-bemidji.crystalpack 7

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_minnesota_cities -- \
  --city bemidji content-packs/minnesota-gyms-bemidji.crystalpack \
  planning/minnesota/gyms/interiors.json \
  planning/minnesota/towns/MnBemidji.json \
  planning/minnesota/towns/MnBemidji.layout.json \
  content-packs/minnesota-bemidji-gym-remake.crystalpack

.local/minnesota-review-env/bin/python planning/minnesota/tools/review_gyms.py \
  --pack content-packs/minnesota-gyms-bemidji.crystalpack \
  --gym MnBemidjiGym --verify-published \
  --output planning/minnesota/renders/gyms/bemidji-og
```

The first output has 473 maps; the city output has 479. Both retain all 471
source maps, every original file and trainer, species/items/moves catalogs,
audio programs/manifest/compression and HOME/recovery. Gym Cut rules and the
city's inherited field-move rules are additive private tileset entries.
The city changes only the gym's three return destinations and their native
wire representation. It accepts either the single-gym or full-eight review
source, and reconstructs the complete canonical module/art/parties/Cut rules
to reject a stale installed room before binding the facade.

The Cut garden uses native Cut followed by a native coordinate event on the
opened tree tile. That event records permanent pruning and the native TILES
callback reopens the gap, including on Continue under the player's feet.
It does not change temporary Cut behavior on original or unrelated maps.
The HM gift and trainer victories have independent saved flags. Two required
wins and badge 6 still govern Orlis. Brindel remains optional.

Native comparison cameras are `(3,13)` and `(7,7)`, noon, default facing. The
before/after images differ only in the named pruning flags; these are clearly
labeled visual fixtures. The controller checks separately earn the openings.
The whole room, entrance, both closed hedges and both pruned states were viewed.
All versions, including rejected passes and conversions, remain archived.

The current canonical gym metadata is synchronized in the city, world and room
contracts. Current non-gym dossier fields and exterior source remain unchanged.
The city source checker covers the replacement contracts, legal trainer parties,
ordinary approaches, facade pairing, bounded shore, services and 16 negative
cases. Its source report is separate from native gameplay evidence. Q05's report
and regional walking seams remain unbound; local tests do not complete them.


Verified results for this remake:

- Four `minnesota_bemidji_controller_` checks pass against the 473-map room:
  physical Cut and both required victories; loss/prior-badge refusal; move and
  Hive Badge denial; B cancellation; early cutting cannot skip either trainer;
  optional Brindel; actual PP use and win; a single badge/TM; both carpets;
  native map bounds; and Continue on the former tree tile.
- `minnesota_bemidji_city_controller_` passes against the 479-map city: actual
  facade, guide's HM grant, normal Start/Pack/party teaching, reusable HM,
  both cuts, all trainer battles, badge/TM, Continue and both city returns.
  It begins with three legal battle moves and no Cut or HM; only initial
  placement, previous badges and the partner are fixtures.
- Seven existing `bemidji_city_controller_` checks pass after updating their
  canonical anchors. They cover all six facades/carpets, downhill return,
  real healing and PC transfer, priced shopping, actual losing-battle clinic
  recovery, Continue, Party-menu Teleport and unchanged officer dialogue.
- Five city asset regressions and the all-room publication/preservation
  regression pass. The latter rejects wrong native replacement, incidental
  Cut, missing Cut and either free route around the hedges. The actual saved
  packs were also compared, including every original map/file/catalog/audio
  field and only the permitted three gym return-wire changes.

Use the shared controller Cargo profile/configuration above and
`RUST_MIN_STACK=67108864`. Set `CRYSTAL_GYM_TEST_PACK` and
`CRYSTAL_RENDER_TEST_PACK` to the absolute standalone pack for the four room
checks. For the city-gym check use the city pack instead. The seven existing
city checks read `CRYSTAL_BEMIDJI_CITY_CONTROLLER_PACK`. Run ignored controller
checks serially with `--ignored --test-threads=1`.

Changed Rust leaves pass formatting and scoped Clippy, with the existing
`never_loop` findings allowed. No warnings point into the changed leaves.
Full all-target Clippy still fails the pre-existing core multiplayer bit-mask
comparison; workspace formatting still reports unrelated existing differences.
These broader failures were retained, not represented as passing checks.
The remaining city/aggregate audit is still required for the goal.

Exact controller commands (the existing city service tests can use the same
profile/configuration and `CRYSTAL_BEMIDJI_CITY_CONTROLLER_PACK`):

```sh
RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-gyms-bemidji.crystalpack" \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/minnesota-gyms-bemidji.crystalpack" \
cargo --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  minnesota_bemidji_controller_ -- --ignored --test-threads=1

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-bemidji-gym-remake.crystalpack" \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/minnesota-bemidji-gym-remake.crystalpack" \
cargo --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  minnesota_bemidji_city_controller_ -- --ignored --test-threads=1
```


## Duluth review loop

The 7 × 10 ice-course remake uses the original Mahogany gym's native ice,
rough corner patches, blue floor/wall and statues, plus native metal stops.
The former 12 × 12 wood/stone bays are rejected. A static phase graph reaches
75 / 93 / 97 / 114 resting tiles before battle and after Livra/Tordel/Anver.
Three wins govern the upper court. There are no book controls, quizzes, field
moves or random puzzle answers.

[Complete review](../renders/gyms/duluth-og/Duluth-review.png) ·
[Matched native court states](../renders/gyms/duluth-og/native-court-comparison.png).

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_minnesota_gyms -- \
  content-packs/minnesota-boundary-basecamp-connected.crystalpack \
  planning/minnesota/gyms/interiors.json \
  content-packs/minnesota-gyms-duluth.crystalpack 8

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_minnesota_cities -- \
  --city duluth content-packs/minnesota-gyms-duluth.crystalpack \
  planning/minnesota/gyms/interiors.json \
  planning/minnesota/towns/MnDuluth.json \
  planning/minnesota/towns/MnDuluth.layout.json \
  content-packs/minnesota-duluth-gym-remake.crystalpack

.local/minnesota-review-env/bin/python planning/minnesota/tools/review_gyms.py \
  --pack content-packs/minnesota-gyms-duluth.crystalpack \
  --gym MnDuluthGym --verify-published \
  --output planning/minnesota/renders/gyms/duluth-og
```

The individual pack contains 473 maps; the Duluth city candidate contains 480.
Actual serialized comparisons preserve every original map/file/catalog/trainer,
field-move rule, HOME/recovery and audio byte/manifest/compression field. City
binding changes only three gym warp destinations and matching native wire args.
It accepts a selected-room or full-eight source; the complete native module,
art, party and program are reconstructed for comparison before rebinding.

Four `minnesota_duluth_controller_` checks pass on the standalone pack. They
prove live long slides and opposing input, empty-frame motion, floor stops,
blocked stops, turning after Continue on both ice and floor, retreat before
Tordel, the west return, all three wins, loss/prior-badge refusal, PP spending,
repeat protection, both carpets and map bounds. A two-tile slide can settle
within one controller press; the tests require its correct endpoint while
requiring a visible intermediate ICE tile on longer slides.

Seven `duluth_city_controller_` checks pass on the 480-map city candidate. Six
cover all seven facades/fourteen carpets, the ledge and ordinary return,
healing/PC, priced shopping and exact Q07 denial/success/repeat. The seventh
starts at the actual gym facade, earns all three wins and badge 8, checks both
carpets/Continue, then walks to the real workshop and acknowledges the receiver.
The signal remains unset immediately after the badge. Only prior badges, the
upstream depot event, starting street position and legal party are fixtures.
No progression is supplied after input begins.

Use `RUST_MIN_STACK=67108864`, `CARGO_TARGET_DIR=target/gym-controller` and the
same Cargo profile/configuration as the Bemidji commands above. For the four
room checks, set `CRYSTAL_GYM_TEST_PACK` / `CRYSTAL_RENDER_TEST_PACK` to the
absolute standalone pack path. For the seven city checks, point both plus
`CRYSTAL_DULUTH_CITY_CONTROLLER_PACK` at the absolute city candidate, then use
`--lib duluth_city_controller_ -- --ignored --test-threads=1`. The city-variable
name matters: omitting it reads an older retained candidate by default.

Seven Duluth asset tests and the all-room publication/preservation test pass.
Current whole-town geometry, seven room contracts and 18 source cases also
pass (17 rejections and one accepted unrelated-source/hash mutation). The old
whole-town render baseline already differs from current non-gym source, so its
historical byte-equality check is not claimed as green or rewritten. Current
non-Gym8 fields and exterior layout bytes were preserved during this remake;
canonical room renders/cameras and current source checks run separately.

Native cameras `(2,8)` and `(11,10)` expose the two rough patches and stops;
`(5,7)` shows the same court before/after Anver. Named preview flags are visual
fixtures, separately earned by controller tests. The whole room, entrance,
patch/stop views, matched court states and actual city facade were inspected.
Every render, earlier rejected pass and conversion remains archived.

Changed Rust leaves pass formatting and scoped Clippy with the existing
`never_loop` findings allowed; no warnings point into the changed files. Full
all-target Clippy still fails the existing core multiplayer bit-mask comparison.
Workspace formatting still finds unrelated differences. These remain reported
failures rather than broad passing claims.

The canonical-only source review leaves the old whole-town baseline intact:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_duluth.py \
  --pack content-packs/text-tui.crystalpack --gym-only
```

## Combined current rooms and northern facade candidate

[Eight rooms at one native tile scale](../renders/gyms/remade-circuit/all-gym-interiors.png).

The current all-room export contains 480 maps. The northern candidate contains
498, binding Duluth, Brainerd and Bemidji while preserving the other five room
modules and review hall. Actual serialized comparisons retain all 471 original
maps/files/catalogs/trainers/HOME/recovery/audio, plus the complete canonical
art, battle/puzzle/reward scripts and parties. Only the declared three return
wires for gyms 6/7/8 change during the northern city overlay.

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_minnesota_gyms -- \
  content-packs/minnesota-boundary-basecamp-connected.crystalpack \
  planning/minnesota/gyms/interiors.json \
  content-packs/minnesota-gyms-remake.crystalpack

.local/minnesota-review-env/bin/python planning/minnesota/tools/review_gyms.py \
  --pack content-packs/minnesota-gyms-remake.crystalpack --verify-published \
  --output planning/minnesota/renders/gyms/remade-circuit

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_minnesota_cities -- \
  content-packs/minnesota-gyms-remake.crystalpack \
  planning/minnesota/gyms/interiors.json \
  planning/minnesota/towns/MnDuluth.json \
  planning/minnesota/towns/MnDuluth.layout.json \
  planning/minnesota/towns/MnBrainerd.json \
  planning/minnesota/towns/MnBrainerd.layout.json \
  planning/minnesota/towns/MnBemidji.json \
  planning/minnesota/towns/MnBemidji.layout.json \
  content-packs/minnesota-northern-gyms-remake.crystalpack
```

The combined production-controller regression
`minnesota_gyms_controller_earns_all_eight_native_puzzles_badges_tms_and_continue`
passes on the all-room pack. Use the preceding controller Cargo configuration
and the absolute all-room pack in `CRYSTAL_GYM_TEST_PACK` and
`CRYSTAL_RENDER_TEST_PACK`, with `--ignored --test-threads=1`.
It solves all eight native courses and earns their battles, badges, single TMs
and Continue states from explicit separate fixtures. The eight-room terrain
pixel/collision comparison also passes, and the full contact sheet was viewed.
No continuous regional travel or ordinary-party balance is inferred.

Brainerd's `brainerd_city_controller_all_facades_both_carpets_and_safe_arrivals`
also passes on the northern pack. Set `CRYSTAL_BRAINERD_CITY_CONTROLLER_PACK`
to its absolute path and `CRYSTAL_CITY_GYMS_SOURCE` to this checkout's current
`planning/minnesota/gyms/interiors.json`; its default frozen source belongs to an
older historical candidate. The separate current Bemidji and Duluth city
playthroughs remain documented above. This overlay preservation/facade proof
does not claim a newly executed full three-city journey.

The remaining integration audit must bind Rochester, Mankato, St. Paul and
St. Cloud to their actual native facades, reconcile the final aggregate with
Minneapolis's current city candidate, and prove actual city progression. The
review hall is a development facility, not a replacement for that journey.

See the [border integration audit](../playable/gym-border-audit.md) for the R05
forest collision fix and earned-Wind timing in the eight-gym controller suite.

The [implementation audit](implementation-audit.md) records all eight room
requirements, current source equivalence, sixteen negative cases and the
continuous final-gym approach.
