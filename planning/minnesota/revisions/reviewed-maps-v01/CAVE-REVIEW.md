# Current cave inspection


The current v09 garden successor changes only R02 and its additive gift.
All nineteen complete cave/mine modules and programs remain exact to v08,
verified by `renders/r02-working-farmyard-preservation-v1426/receipt.json`.
The nine gameplay courses and native views below keep their original v08
execution bindings; they are not relabeled as fresh v09 gameplay. The source
review tool resolves the active selector automatically.

## V08 nineteen-room review, preserved by v09

The selection used for this review was `content-packs/minnesota-reviewed-maps-v08.crystalpack`,
SHA256 `38366f43092e040d40e1084ac4437cef967d812029d4e0c9e04d160b06139df7`.
The earlier fourteen-room sheet counted only `CAVE` environments. Haulage,
Sump, Ventilation, Pump Gallery and Drill Core are native rock rooms classified
as `DUNGEON`; they belong in the cave review too. There are nineteen rooms in
this combined scope: seventeen stone rooms and the two Voyageurs ice floors.

- [All nineteen complete current rooms](../../renders/current-all-caves-mines-v1422/all-current-caves-and-mines-original.png)
- [Source owners, dimensions, palettes, actor anchors and real shaft arrivals](../../renders/current-all-caves-mines-v1422/receipt.json)
- [Current fossil-gallery dark and Flash-lit native views](../../renders/current-fossil-gallery-native-v1416/actual-dark-and-lit-cameras.png)
- [Both connected fossil-gallery levels](../../renders/current-fossil-gallery-connected-v1419/connected-levels-2x.png)

`review_current_caves.py` resolves the selected pack and source index directly
from `world.json`, checks every complete installed row array against its current
source, and resolves every authored warp to an existing actual arrival. It does
not silently omit a missing or reclassified known mine floor. The story mine's
one closed shaft block is an explicit overlay on its original terrain; both
its exact foreman-dependent callback and the original closed block are checked.
No old open floor is restored to make a source comparison pass.

The full sheet and individual originals were inspected alongside the existing
cave reference work. These are native **source** views. Actor marks are authored
anchors; callbacks can hide or move them in play. The dark gallery remains dark
in the inventory. Its separate illuminated source overview represents actual
Flash and acknowledged-clue state, verified by played native captures below.
The source/link inventory does not prove traversal or earned access in all
nineteen rooms. No terrain change was justified by the current source inspection
of the mine floors or the illuminated fossil gallery.

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_current_caves.py \
  --output planning/minnesota/renders/current-caves-FRESH
```

Fresh ignored outputs preserve all full-resolution originals and the exact tool
snapshot. Six in-memory corruption cases reject a missing mine, reclassified
mine, changed grotto rows, open story shaft, wrong gate flag and missing warp
arrival in `.local/current-cave-review-guards-v01/receipt.json`. The original
selected pack and source files remain unchanged.

## Current fossil gallery, Gunflint grotto and public mine play

Three fossil-gallery controller courses pass again on the exact selected v08
pack in `renders/current-fossil-gallery-controller-v1415/`. They play the
original HM gift and Pack teaching, both fitted stairs, no-badge/dark refusal,
actual Flash, all fossil pages and acknowledgement, hidden-cache reveal,
original Rare Candy, full-pocket native Toss/retry, held-page Continue and both
level returns through the original outdoor mouth. They use declared original
Minnesota local partners and explicit badge/capacity variants. No progress,
position, party, bag, HP/PP or RNG is rewritten after fixture setup.

Nine actual reached native saves are captured in
`renders/current-fossil-gallery-native-v1416/`. Both dark views were inspected;
all seven illuminated640×576frames match the current native source exactly.
The actual dark/lit sheet and complete connected-level source were inspected.
This is fresh selected-pack play and render evidence, using the retained current
controller and renderer binaries; no new compiler run is claimed for the replay.

Four current Gunflint grotto cases and the public R09 mine course pass in
`renders/current-gunflint-cave-controller-v1418/`. Gunflint covers both dry fin
paths, both spring-bank paths,30/36 stairs and side denial, the real Max Ether
and native PP use, actual exterior return, both Surf denials, real launch,
Itemfinder/pillar Revive/use, both full-pocket refusals, native Toss/retry and
Continue. R09 covers its original cameras, three stairs, short drops, ordinary
returns and actual original mouth with the story shaft still guarded.

Otta's separate original rest refusal/acknowledgement/full-party HP/status/PP
and Continue course passes in `renders/current-gunflint-rest-v1421/receipt.json`.
The first attempt in v1418 omitted that case's required output environment
variable and stopped before setup; it is retained as a failed harness invocation
and is not counted as passing. The corrected run supplies the documented
`CRYSTAL_GUNFLINT_SPRING_REVIEW_DIR`; game code and assertions are unchanged.

These nine current controller cases prove their explicit local mechanics.
They do not establish intended-level difficulty, earned campaign arrival, all
nineteen rooms' current traversal or final user visual acceptance. The v08 pack
and its selection are unchanged by this review. Earlier detailed Voyageurs,
R10, Inner Basin and Haulage proof remains retained below and in their runbooks.

## Retained fourteen-room inspection


The then-selected v04 pack, SHA256
`47d45a3bb934e4ad863d3e92950e29dd55bdceb62d5824ff23952d55e11e7b48`,
contains fourteen Minnesota CAVE environments. Their complete installed native
source views were rendered at one metatile scale and inspected together:

- [All fourteen current caves](../../renders/selected-cave-inventory-v1359/all-current-caves-original.png)
- [Exact inventory, dimensions, palettes, actors and mouths](../../renders/selected-cave-inventory-v1359/receipt.json)

Each room's full-resolution terrain, actor composition and whole original is
retained in its own subdirectory. These are source views, not new production
camera comparisons or evidence that access was earned. Dark/night declarations
are respected; the lamp gallery has not been artificially brightened for review.
The later Rochester, Grand Rapids, Mankato, Brainerd v07 and R16 v08
publications preserve every cave byte. Their exterior reviews do not count as
new cave traversal evidence.

## Current Voyageurs ice course

The detailed selected-v04 course now passes both native floors, forced turns,
all rewards, services, both ladder mouths, the ordinary return and Continue.
Three current cases pass in
`renders/current-voyageurs-ice-play-1793509092044545000/receipt.json`, including
full-pocket refusal, native toss/retry and the surveyor's dialogue/input ownership.
Nine matched native/source cameras have zero differing pixels in
`renders/current-voyageurs-ice-pixels-1793509062981563000/capture-progress.json`;
both complete floor camera sheets were inspected. The actual room arrays still
match `ice-caves/turning-chambers-v01/current.json` exactly. No replacement terrain
was justified by this inspection; offset ice tongues, distinct cache branches
and stopping rocks already provide the intended decisions.

The first replay failed an overly strict travel helper that prohibited any
ordinary wild encounter. The corrected local companion fixture has a Smoke
Ball, and travel can use native RUN if a Repel expires. The successful full
journey log contains no wild encounter, so it does not establish that fallback
branch was exercised. There are no party/position/flag/RNG writes after setup.
This is a declared local navigation fixture, not earned regional progression.
Current all-target Bevy Clippy and scoped formatting pass alongside the retained
build logs. Grand Rapids' later candidate preserves both rooms exactly.

## Detailed review criteria

Inspect `MnVoyageursIce1F` (14×12 metatiles) and `MnVoyageursIceB1F` (16×12)
through native input before changing their broad ice chambers. A large ice field
needs meaningful stop/turn decisions and discoveries; open area alone does not
establish either a defect or enjoyable play. The next acceptance must follow
both actual floor mouths, every intended item/service approach, useful stopping
rocks, the return trip and Continue. Check the native ice/Strength rules and
current original Minnesota partners. Keep the main regional walking route and
ordinary snow distinct from an optional sliding puzzle.

Compare their composition with the supplied original Ice Path reference, and
compare production cameras with each room's own native 2D at matched map,
position, time, facing and state. Record an observable gameplay or composition
problem before authoring a replacement. Keep exact room/warp,
actor, item, encounter and story preimages; never publish from the older
eleven-room pack merely because its historical ice regression passed.

## Preserve the existing cave work

R10's 24×18 survey cave is the largest stone room in the sheet. Its ceiling
spur, bent Escape Rope hollow, two short drops, lake bank, SILAS/NORA challenges
and reader already have an explicit [fifty-block refinement](../../caves/r10-survey-spurs-v01/README.md).
The whole-room thumbnail does not justify undoing that topology or moving its
five activities. Judge its widest ordinary walk in actual player cameras.

That current R10 review now passes on selected v05. All twelve complete
640×576 native frames match the current source exactly in
`renders/current-r10-survey-native-1793512209712062000/capture-progress.json`;
the complete camera sheet was inspected against the retained Union Cave and
Mt. Mortar references. The actual lower chamber has a spring-side activity,
separate supply hollow and connected ordinary return; a whole-room thumbnail
does not justify replacing the existing bedrock spur. Terrain remains unchanged.

Three current controller cases pass in
`renders/current-r10-native-run-controller-1793512723808119000/`: both stair
flights, both short drops and uphill returns, original reader, both rewards,
real exterior travel, both optional challenges/PP-spending wins, full-pocket
Toss/retry and Continue. The earlier apparent staircase failure was a navigation
fixture stopping while a native wild battle owned input. Its retained state
contains a level49 PELLETANK against the supplied level48 party: current wild
scaling can allow such an encounter through Repel. The updated local fixture
supplies Smoke Balls and uses native RUN only for wild encounters; trainer
challenges are still played. No state is rewritten after fixture setup.

The independent two-process replay in
`renders/current-r10-wild-return-stdio-1793512729439941000/receipt.json` starts
from that exact retained failure save, uses native RUN without editing it,
resumes walking, then checks fresh-process Continue, the return step and Start.
All31 RPC calls pass with byte-empty stderr and unchanged inputs. The fixture
helper's build, scoped formatting and all-target Bevy Clippy pass in
`.local/current-r10-native-run-build-1793512657011192000/`. This is current local
mechanics and visual evidence, not earned regional arrival or campaign balance.

The current Inner Basin fitted bedrock and Haulage clearances belong to the
reviewed source closure. Their real stairs, attached faces and ordinary
returns take precedence over old cliff-hop expectations. The dark lamp gallery
requires its real light state for a detailed visibility review.

The other compact caves have distinct forks, shoulders, sheltered care banks
and supply notches. Keep their native assemblies and current bindings while
reviewing each trip. This inspection does not complete statewide cave quality,
an earned regional journey or user visual acceptance.

The selected v10 River Bluffs exterior revision preserves all 19 complete cave
modules, including Rock Cut, exactly. Its three fresh Rock Cut controller cases
and actual two-mouth stdio journey pass in `renders/r05-wooded-terraces-cave-fishing-v1456/`
and `renders/r05-current-mouths-stdio-v1461/`. The source is the current shelf-drop
`after.json`; the older canonical Johto-quality preimage is retained history.
Earlier v08 cave views and earned saves keep their original identities.

The selected v11 Pine Country exterior revision preserves every complete cave
module and both R06 cabin/guru rooms exactly. Its native Cut/stair/Ether/reset
course and all three guru gift/door/repeat journeys pass on current bytes; no cave
terrain or older earned-save identity is relabeled as newly played progression.
