# Expanded native State Fairgrounds

The user rejected the small fair as too sparse and asked for a bigger, denser
fair with more vendors and larger structures. The current authoring is one
48×36-metatile exterior (96×72 walking tiles),2.25× the earlier map area.
It contains255 outdoor characters,32 actual shops and39 staffed counter positions.
NPCs remain independent objects behind original Crystal COUNTER tiles.

## Distributed crowd and exhibitions

The local `content-packs/minnesota-state-fair-maze-trainers-v7.crystalpack`
candidate fills the existing255-record exterior capacity:245 people (including
35 battle trainers) and10 Pokémon exhibits. Ten adjacent judges explain
grooming, care, showmanship, fleece and junior handling. These are interactive
exhibition conversations; there is no scored contest or player ribbon award.
All137 new visitors and13 new trainers have dialogue, with local fair subjects and distinct group
numbers. The crowd is distributed across food courts, midway, picnic areas,
garden, southern projects and entrance, rather than uninterrupted queue walls.

There are12 slower walkers,16 brisk runners (including the original four),
54 rotating characters (six are exhibition Pokémon), and173 standing characters.
Walking and running use collision-checked clockwise floor circuits with16- and
8-frame steps respectively. Rotation stays on the actor's tile. Original
merchants, trainers and judging pairs retain fixed positions. Trainer sight
corridors, doors, customer approaches, review anchors and original loops remain
clear. Source review reaches4870 tiles and every actor/entrance/customer tile.

Rebuild the candidate with the Rust publisher:

```sh
CARGO_TARGET_DIR=target/fair-maze CARGO_BUILD_JOBS=2 \
  cargo build --locked -p crystal-assets --bin pack_state_fair \
  --profile wasm-dev --config profile.wasm-dev.opt-level=0
target/fair-maze/wasm-dev/pack_state_fair \
  content-packs/minnesota-fair-grass-base-v1.crystalpack \
  planning/minnesota/zones/state-fairgrounds.layout.json \
  planning/minnesota/zones/state-fairgrounds.json \
  content-packs/minnesota-state-fair-maze-trainers-v8.crystalpack
```

The publisher requires a fresh output name. The v8 name above is reserved for
a reproduction; it is not a claim that an eighth candidate has been built.
Source renders are retained under `renders/state-fair/maze-trainers-exhibition-source-v7`.
The current35 trainers use the existing singles battle engine. True two-on-two
battles are not implemented by this crowd revision.

Final verification: six targeted Rust/controller/preservation cases pass.
All35 trainers use real moves/PP, award native money and retain defeat flags;
the original patrols complete two laps, pause for interaction and retain their
positions through Continue. The new ambient case observes walking, running,
rotation and actual dialogue. Rust rebuild identity and preservation pass.
All377 audio PCM hashes, frame counts and loop ranges match. All19 untouched
headless views have zero static-pixel mismatches outside documented animated
regions. Captures and source comparisons are retained in
`renders/state-fair/maze-trainers-native-v8`; the rejected first capture helper
attempt is retained in `maze-trainers-native-v7`.

Standard all-target Clippy for core/assets/Bevy stops at the existing
`clippy::bad_bit_mask` in core multiplayer1332. Workspace formatting still
reports unrelated offset-allocator differences; scoped changed Rust files pass.
Logs are retained in `renders/state-fair/maze-verification-v7`.
The older verification records below describe their named predecessor packs.

The [official2026 fair map](https://assets.mnstatefair.org/pdf/26-MSF-color-Map-web.pdf)
informs the west midway/grandstand, southern livestock and eastern food/dairy
areas. This is a condensed game map, not a street-for-street reproduction.

## Purchasable foods and beer — current verified revision

The foods v2 candidate now sells16 fair foods. Seven additions are hotdish,
pronto pups, mini donuts, funnel cake, fried candy, walleye and beer. The existing
corn item is labeled SWEET CORN; cookie pails and cheese curds remain available.
The complete price/HP/counter table appears below.

Current pack SHA256: bf688621307c6e4805296fa0412b67777327899c507512432200569d0b5f7ee8.
It retains664 maps,474 species,270 moves,85 outdoor NPCs and377 audio programs.
Use current client source for the additive healing/confusion item payload.
This remains a separate fair candidate with the existing bounded St Paul bind.

Two final production-controller component cases pass on foods v2: every32
merchant and all16 purchases, payment, HP changes, consumption, beer Continue
and first aid (118.22seconds); and purchased beer in the field, Save/Continue,
confusion carried into an actual trainer battle, a second beer spending a turn
with an enemy response, the visible confusion notice and native victory clearing
(10.00seconds). Fixtures supply the initial local position, party and money;
all subsequent gameplay uses VisibleShellController input.

The first foods pack also passes the complete eight-case fair suite in421.91
seconds, including all22 trainers, four walking loops, overlapping challenges,
all29 wild species, rooms, stamps, prizes, full-bag recovery and city travel.
Its three overlong descriptions were then shortened to the native18-character
Pack width. The publisher and preservation test now enforce the two-line,
18-character description limit and12-character item names for every fair food.

Final preservation/rebuild passes in62.25seconds. Every terrain block, NPC,
entrance, encounter entry, interior and native tile asset remains intact;
original items, HOME and compiled audio remain unchanged. The only existing
fair item edit is the sweet-corn name/description. The new whole-map source
render was inspected and is exactly identical to the reviewed crowd image:
1,769,472 RGBA pixels, zero differences. No new GPU captures are claimed for
this item-only revision; the prior native visual record remains below.

The first final-menu attempt exposed a stale test route after a moving NPC
blocked a step. The helper now reads live NPC positions and replans from the
actual player tile; walkers remain active and no game state is moved directly.
The corrected menu case and the beer case both pass with that compiled helper.
The first missing-file preservation attempt preceded final pack publication;
the successful final readback/rebuild supersedes it. All logs and rejected
passes are retained in renders/state-fair/foods-verification-v1.

All76 core item regressions and both party-healing regressions pass. Scoped
formatting passes. Standard all-target Clippy still hits the unrelated core
bad_bit_mask error at multiplayer/mod.rs:1332. Allowing the previously documented
bad_bit_mask, never_loop and voxel approx_constant lints exposes missing radio
oracle JSON fixtures in the core radio_text integration test. Core library
Clippy and asset/runtime/Bevy all-target Clippy pass with those lint allowances.
Workspace formatting still reports unrelated differences starting in
offset-allocator/src/lib.rs:408. These limitations and exact logs are in the
[verification receipt](../renders/state-fair/foods-verification-v1/receipt.json).

## Busy fair crowd and battles

The crowd revision keeps85 outdoor NPCs. Twenty existing fairgoers now battle,
joining ELI and JAMIE for22 trainers. Twenty trainers challenge through sight;
the two show-ring trainers retain their talk challenge. Sight lanes cross the
queues, food rows, picnic courts, gardens and walking loops. MAX and NELL both
cover core(13,27) in the midway, deliberately creating successive challenges.
Every trainer has a regional-themed legal party, native reward, defeat flag
and post-battle dialogue. The39 staffed counter positions retain their vendors.

Ten additional non-battling visitors include seven kids and three adults.
Four use clockwise rectangular walks: a cookie runner, midway runner, picnic
walker and garden runner. Their native sprites animate while they move.
They share ordinary collision with the player and other NPCs, wait when
blocked, pause for conversation and resume their route afterward. The route
is derived from the actor's live position on its original rectangle; no extra
save cursor or second dispatcher is used. Existing source movement types keep
their behavior. Current client source is required for the new patrol movement.

The current candidate is `content-packs/minnesota-state-fair-foods-v2.crystalpack`.
Build it with the same Rust command below, using that new immutable output name.
The prior crowd and grass candidates and all rejected passes remain retained.
The crowd verification below records its seven-case run. Native source comparison excludes only the
four bounded walking regions from static pixel assertions, retains untouched
captures and full differences, and reports the excluded areas explicitly.
Moving sprites are reviewed directly and exercised through the controller.

## Crowd verification record

The664-map crowd v2 pack has SHA256
`f48effca14a8fcd0d7a0a1df84f1a17950355be0b27f9c580b6276f21a8cbbaf`.
It retains474 species,270 moves and the parent's377 identical audio programs.

Seven controller cases pass: all22 actual trainer battles with native move PP,
money rewards and defeat flags; a new trainer win through Continue; overlapping
midway challenges; four patrols completing two laps apiece, blocked by the
player, pausing for conversation and continuing from saved positions; every32
merchant/nine foods; all prior activities, rooms/city travel and rewards; and
actual grass battles with all29 species. Individual trainer cases declare
prior wins against other trainers solely to isolate each challenge; the midway
case leaves both rivals undefeated. The loop cases use an explicit replay DIV
clock. These fixtures do not prove earned regional campaign access.

Preservation/rebuild passes. Every terrain block, native fair tileset, wild
encounter entry and all three interiors equal the prior verified grass fair.
Original maps, HOME, assets and compiled audio equal the species-enriched parent.
Two core patrol tests and twelve original autonomous-movement regressions pass.
Seventeen untouched native captures pass5929986 static pixel comparisons;
336894 pixels in the bounded walker regions are excluded from that assertion
and reported explicitly. The complete frames and full differences remain
retained. Whole-map and native views were inspected for sprite/tile fidelity.

Native/source history is under `renders/state-fair/crowd-source-v1`,
`crowd-source-v2`, `crowd-source-final`, `crowd-native-v1` and `crowd-native-v2`.
The first crowd source used a stationary three-frame Game Boy kid for a patrol;
the published revision uses the native six-frame TWIN walking sprite. The
first controller pass exposed renderer-neutral idle frames that skipped NPC
movement and an outdated75-object test expectation. The fixed scheduler uses
the shared production tick for loaded autonomous actors. A later run was
stopped before its census to apply this loaded-object guard; the final complete
seven-case run supersedes it. The first native comparison tool did not yet
recognize the new patrol token; that attempt and its untouched capture remain.

Changed leaf files pass scoped formatting. Standard all-target Clippy encounters
the existing core `bad_bit_mask` error. Affected asset/Bevy all-target checks
and the core library pass with the existing `bad_bit_mask`, `never_loop` and
voxel `approx_constant` lints allowed as applicable. Workspace formatting still
reports unrelated existing differences. Logs and final hashes are retained in
`renders/state-fair/crowd-verification-v1`.

## Food and fair encounters

The grass revision adds14 deliberate native tall-grass patches:108 metatiles,
or432 walking tiles. Each patch has a bounded encounter zone containing only
tall grass. Customer tiles, paths, actors and entrances remain clear. All29
active food/carnival species, including every evolution, appear at every time
of day in six habitat groups:

| Habitat | Species |
| --- | --- |
| Bakery and breakfast | Doughlin, Crumcone, Pailbaron, Flapip, Stackajack |
| Grain and rice exhibits | Oatlet, Millimp, Flourgeist, Riceling, Reedraft, Zizalith |
| Dairy | Curdlet, Squeakurd, Chislet, Buttermuse, Lutebrisk |
| Hotdish and smokehouse | Totlit, Casserogre, Cinderfin, Brinebraze |
| Orchard and sugarbeets | Crispip, Cidermaw, Zapsap, Tapacitor, Beetynamo |
| Midway and popcorn | Tiltawisp, Pipcorn, Pufflare, Popummel |

This selection follows the active roster and its curation file. Retired
Leflet/Lefold, Berryboo/Muffiend, Morlit/Morelord and Torchcap stay retired.
The species overlay installs the complete current223-species regional roster;
the fair encounter selection uses13 existing families. No extra species are
invented. Authored minimum levels are10,23 and36 for first, second and third stages.
The inherited Wild Challenge rules can scale battles to the visiting party;
the species overlay preserves those parent-pack rules.
Each habitat uses native seven-slot tables and rate25, with the same selection
morning, day and night. Signs explain the local habitats.

The Rust publisher rejects missing species, missing evolutions at any time,
non-grass or overlapping encounter zones, inaccessible grass, unbound grass
and grass on a booth's customer tile. The fair adds only its own encounter
catalog entry; all pre-existing map encounters remain identical.

## What occupies the larger fair

A sixteen-metatile food pavilion joins eight cooks behind one continuous
native counter. Cookie pails, roasted corn, pretzels, cheese curds, root beer,
ice cream, pickle sticks and kettle corn use ordinary priced shop menus.
The nearby lemonade, milk, produce and bakery stands form another food row.

An eight-metatile red-roofed exhibitors' hall sells useful seed, stone,
bird-keeping and craft items. The midway retains its free quiz and ring toss,
with ball/ribbon stands and customer queues. Southern4-H and rural-craft
pavilions add a care question, growing projects and additional merchants.
The livestock, show ring, garden beds, dairy forecourt and picnic clusters
have their own visitors, handlers and browsing lanes. Main streets remain two
walking tiles wide and connect useful destinations in several loops.

The grandstand, livestock barn and dairy exhibition have wider native facades
and retain their three real interiors and safe returns. Twenty-two trainers battle, twenty through
sight-triggered challenges. First aid restores the party through native HealParty.
The care question awards one BERRY, while the original three-stamp trail
awards one RARE CANDY. Full-bag recovery and once-only flags remain explicit.

Sixteen fair foods are sold through the original native mart menus and consumed
through Pack USE on a selected party member, in the field or in battle. Sweet
corn retains the original `FAIR_CORN_STICK` ID used by the ring-toss prize.

| Food | Price | HP | Counter / customer tile |
| --- | ---: | ---: | --- |
| Cookie pail | 500 | 50 | Cookies `(55,12)` |
| Sweet corn | 300 | 20 | Corn `(59,12)` |
| Pretzel | 200 | 20 | Pretzels `(63,12)` |
| Cheese curds | 400 | 30 | Curds / hotdish `(67,12)` |
| Root beer | 500 | 60 | Drinks `(71,12)` |
| Ice cream | 450 | 50 | Ice cream `(75,12)` |
| Pickle stick | 100 | 10 | Pickles `(79,12)` |
| Kettle corn | 300 | 30 | Kettle corn `(83,12)` |
| Fair honey | 250 | 20 | Honey `(15,60)` |
| Hotdish | 650 | 80 | Curds / hotdish `(67,12)` |
| Pronto pup | 250 | 30 | Corn `(59,12)` |
| Mini donuts | 350 | 50 | Bakery `(81,24)` |
| Funnel cake | 500 | 60 | Bakery `(81,24)` |
| Fried candy | 600 | 80 | Kettle corn `(83,12)` |
| Walleye | 800 | 100 | Market snacks `(7,60)` |
| Fair beer | 500 | 100 | Drinks `(71,12)` |

Beer also causes four turns of actual confusion. Field use persists through
Save/Continue and enters the next battle with that counter. First aid and
Pokémon Center healing clear it. It uses the normal
confusion action checks, self-damage, switching and battle-end clearing rules.
Using it in battle consumes the player's turn and permits the enemy response.
Its description, vendor dialogue and recovery notice explain the effect. Root
beer remains an ordinary healing drink. Full-HP, fainted and egg targets do not
consume a beer or gain confusion. HP recovery is capped at maximum HP.

The additive optional `Item::confusion_inflict_turns` payload is absent from
serialized native items; existing packs and item definitions remain compatible.
The engine selects the combined recovery behavior from that payload. No badges
or regional story progress are granted for visiting the fair.

## Raised NPC and art capacity

The fair stays on one map. The user explicitly requested raising the engine
limit rather than splitting the grounds to work around it.

`crystal-core::world::object_capacity::MAX_MAP_OBJECTS` is255, using all nonzero
byte-sized object IDs. Source-sized maps retain their original twelve-slot
allocation behavior. Larger authored maps can allocate up to255 visible NPC
slots. Loading/unloading, interaction, collision, appear/disappear and save
restore use the same capacity policy. Slot0 remains the player.
Save parsing accepts expanded slots and pack-aware restore still checks each
record against the actual compiled map. The previous15-record pack rejection
is replaced with the real byte-sized limit. Existing source maps are unchanged.

The fair-only tileset opts into a256-glyph bank. Original glyph IDs, every
original block's bitplanes and palette colors are preserved; referenced native
glyphs are copied to their corresponding wider-bank addresses. Only the new
alias changes. Eleven exact native block recipes import Underground counters,
planters, benches, bollards and the red park roof. No illustrated booth, vendor
pixels or exported duplicate source catalog is included.

## Sources and bindings

- `zones/state-fairgrounds.layout.json`: all explicit terrain/structure strokes.
- `zones/state-fairgrounds.json`: NPCs, stock, doors, parties, flags and views.
- `crates/crystal-assets/src/state_fair_modpack.rs`: Rust publication.
- `crates/crystal-assets/src/state_fair_scripts.rs`: native fair activities.
- `crates/crystal-assets/src/state_fair_patrols.rs`: bounded native-floor routes.
- `crates/crystal-core/src/world/object_patrol.rs`: clockwise perimeter steps.
- `crates/crystal-bevy/src/bevy_shell/tests/state_fair_crowd.rs`: actual trainer, crowd and patrol checks.
- `tools/review_state_fair.py`: fast whole-map/collision/source-snapshot review.
- `tools/review_state_fair_native.py`: untouched GPU captures and exact source
  comparison, including current native viewport scale and connected terrain.

The fair south opening is core(16,71)/(17,71), linked to St Paul north
core(16,0)/(17,0), offset0. The city patch remains exactly eight metatiles at
x8,y0..7, changed to Kanto55 brick. All other city blocks, NPCs, scripts,
rooms and connections remain unchanged. The builder rejects stale source.
Map group80 is reserved by this candidate and must be checked when integrating
into a newer regional lineage.

## Reproduce the local candidate

Packs, captures and build output stay ignored. Run from the repository root.
Start with the completed660-map parent
`minnesota-route-quality-r04-v01.crystalpack`. Install the current regional
species and moves on that parent before adding the fair. The completed
species-enriched parent is `minnesota-fair-grass-base-v1.crystalpack`.

```sh
target/agent-northwoods-publish/debug/pack_fairy_type types \
  content-packs/minnesota-route-quality-r04-v01.crystalpack \
  content-packs/minnesota-fair-grass-types-v1.crystalpack
target/agent-northwoods-publish/debug/pack_minnesota_pokemon \
  content-packs/minnesota-fair-grass-types-v1.crystalpack \
  planning/minnesota/pokemon target/minnesota-pokemon/native \
  planning/minnesota/pokemon/species-profiles.json \
  content-packs/minnesota-fair-grass-species-v1.crystalpack
target/agent-northwoods-publish/debug/pack_northwoods_moves \
  content-packs/minnesota-fair-grass-species-v1.crystalpack \
  content-packs/minnesota-fair-grass-moves-v1.crystalpack
target/agent-northwoods-publish/debug/pack_fairy_type moves \
  content-packs/minnesota-fair-grass-moves-v1.crystalpack \
  content-packs/minnesota-fair-grass-fairy-v1.crystalpack \
  planning/minnesota/pokemon/fairy-moves.json
target/agent-northwoods-publish/debug/pack_northwoods_animations \
  content-packs/minnesota-fair-grass-fairy-v1.crystalpack \
  content-packs/minnesota-fair-grass-base-v1.crystalpack
```

The [species runbook](../../../docs/northwoods-fairy.md) documents building
these Rust exporters; [animation verification](../../../docs/northwoods-move-animations.md)
describes the final animation overlay.

```sh
CARGO_TARGET_DIR=target/agent-minnesota-tiles CARGO_BUILD_JOBS=2 \
  cargo build --locked -p crystal-assets --bin pack_state_fair \
  --profile wasm-dev --config profile.wasm-dev.opt-level=0
target/agent-minnesota-tiles/wasm-dev/pack_state_fair \
  content-packs/minnesota-fair-grass-base-v1.crystalpack \
  planning/minnesota/zones/state-fairgrounds.layout.json \
  planning/minnesota/zones/state-fairgrounds.json \
  content-packs/minnesota-state-fair-foods-v2.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_state_fair.py \
  --pack content-packs/minnesota-fair-grass-base-v1.crystalpack \
  --output planning/minnesota/renders/state-fair/foods-source-v1
RUST_MIN_STACK=67108864 \
  CRYSTAL_FAIR_TEST_PACK="$PWD/content-packs/minnesota-state-fair-foods-v2.crystalpack" \
  CARGO_TARGET_DIR=target/agent-minnesota-tiles CARGO_BUILD_JOBS=2 \
  cargo test --locked -p crystal-bevy --lib state_fair \
  --profile wasm-dev --config profile.wasm-dev.opt-level=0 \
  -- --ignored --test-threads=1
RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/agent-minnesota-tiles CARGO_BUILD_JOBS=2 \
  cargo test --locked -p crystal-assets --test state_fair_modpack \
  --profile wasm-dev --config profile.wasm-dev.opt-level=0 -- --ignored
target/agent-minnesota-tiles/wasm-dev/verify_crystalpack \
  content-packs/minnesota-state-fair-foods-v2.crystalpack --audio --desktop
CARGO_TARGET_DIR=target/agent-minnesota-tiles CARGO_BUILD_JOBS=2 \
  cargo build --locked -p crystal-bevy --example render_at_location \
  --features location-tester --profile wasm-dev --config profile.wasm-dev.opt-level=0
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_state_fair_native.py \
  --pack content-packs/minnesota-state-fair-foods-v2.crystalpack \
  --binary target/agent-minnesota-tiles/wasm-dev/examples/render_at_location \
  --output planning/minnesota/renders/state-fair/crowd-native-v1
```

The publisher/capture runner require new immutable output names after revisions.
Initial local placement, party, low HP and money in tests are explicit fixtures;
subsequent travel, dialogue, shops, item use, activities, battle and persistence
use `VisibleShellController`. They do not establish earned regional access.

## Grass candidate verification

The664-map `content-packs/minnesota-state-fair-grass-v2.crystalpack` contains
474 species and270 moves. Its SHA256 is
`94c73d8d9e7eabf88f17ac1adc424be8954600327d871625d1633d5436fc2ebf`.

Preservation/rebuild, every existing encounter table, native art, the missing
evolution rejection and counter/road checks pass. The zone-origin regression
passes. The three existing production fair activity cases pass. Thirteen
production views match every one of368640 source pixels each. All377 audio
PCM hashes, frame counts and loop ranges match.

The final species census uses explicit local placement, party, held Smoke Ball
and deterministic DIV fixtures, followed only by controller input. Protect
provides a native move/PP turn before RUN, and Smoke Ball avoids failed-escape
damage under the inherited Wild Challenge scaling. This is local encounter
verification, not earned campaign access. The completed census encountered all29 species in actual wild battles
and spent native move PP in each of the six habitat groups; the case passed
in59.28seconds. Both native RUN and preserved party state were exercised.

Standard all-target Clippy remains blocked by the existing core bad-bit-mask
lint. Allowing that and the existing never-loop lint exposes three unrelated
voxel approximate-constant errors. With those existing lints allowed, affected
asset/Bevy all-target checks pass. Workspace formatting still reports unrelated
existing differences; changed leaf files pass scoped rustfmt checks.

The first grass source pass put the feed booth's immediate customer tile into
tall grass. That rejected version and its source snapshot are retained. The
published revision moves the grass back; both the Rust publisher and fast
source review now reject that counter obstruction. Whole-map history is under
`renders/state-fair/grass-source-v1` and `grass-source-v2`; native comparisons
and untouched GPU captures are under `renders/state-fair/grass-native-v1`.

## Previous dense candidate verification record

The664-map `content-packs/minnesota-state-fair-dense-v1.crystalpack` has SHA256
`a51d33df781aef89a5d465251b6fec5bb7e511e2397d927dbae1cf65c01dc61d`.

Three production controller cases pass: all32 merchant menus and cancellation,
all nine food purchases/HP changes/consumption, the care question's wrong and
correct branches/once-only BERRY, first aid after real battles, both trainer
wins with native PP use, stamps, free ring toss, full-bag reward recovery,
all three room returns, St Paul travel both ways and Save/Continue.
The preservation/rebuild/invalid-NPC case passes, including every original
Johto block's exact bitplanes/colors after bank expansion. The byte-overflow
verifier rejects256 records. New live-slot tests exercise20 simultaneous NPCs,
slot64 through save validation, and the last slot255 through allocation/restore.
All377 audio PCM hashes, frame counts and loop ranges pass.

Eight production native views match368640 source pixels each with zero
mismatches. They cover both ends of the main food pavilion, midway, show ring,
garden, connected entrance, red exhibitors' hall and4-H projects. Original
GPU descriptors, untouched captures, source references and differences are in
`renders/state-fair/expanded-native-v2`. The first capture attempt's garden
placement overlapped a visitor; it is retained as a rejected fixture. The final
view uses adjacent free floor. Source reviews now reject occupied view anchors.
Whole-map/source/player previews and immutable JSON versions are retained under
`expanded-source-v1` through `expanded-source-v4`, with the same-scale old/new
comparison in `expanded-source-v3/before-after.native.png`.

The full core library run reports2238 passed and8 unrelated failures: five
battle-effect tests lack external move-effect source inputs, two special-routine
tests require an external fixture (one is then poisoned), and the existing
invalid-state save-encoding assertion fails. Focused original object tests pass.
Standard all-target Clippy stops on the existing `bad_bit_mask` in core
multiplayer1332. With `bad_bit_mask`/`never_loop` allowed, core all-target checks
also encounter missing external radio `include_str!` fixtures. The core library
and affected asset/Bevy all-target checks pass separately with those two
existing lints allowed. Scoped changed leaf files are formatted; workspace-wide
formatting still reports unrelated existing differences.

The native style remains original Crystal tiles with separate NPC sprites.
Animated ride art remains future work. The grass candidate installs the
existing regional species and the29 food/fair encounters listed above. No
rejected oversized illustration is used in this fair.
