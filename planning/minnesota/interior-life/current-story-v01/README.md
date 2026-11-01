# Interior and current story composition

The local composition retains the 716-map interior/fair branch, adds the current
thirteen story rooms, and carries the accepted R01, R05 and R03 terrain changes.
All presentation checks use native 2D. Red Wing has no station or stale station
timetable listing. The shared regional selector and its parent allowlist are
not changed by this local publication.

## Reproduce

Use the repository's pinned Rust toolchain and ignored, completed inputs. Build
`pack_northstar_current_story`, `pack_minnesota_interiors` and
`rebase_accepted_terrain` from `crystal-assets`. Completed output paths are
immutable: choose fresh output names for a replay.

The first two terrain contracts retain the 716-map branch while applying the
accepted R01 cave and R05 grass changes. Their completed outputs are
`content-packs/minnesota-interiors-r01-current-v01.crystalpack` and
`content-packs/minnesota-interiors-r05-current-v01.crystalpack`.

The regional flavor overlay corrects eight text labels across seven rooms.
It uses the active AUGRUB, PIPBOOM, GILLPUP, GOOSNIDE and PINIT names, with a
coherent bird reference for the grouse observer. The frozen original room-story
source is retained. No residents, teams, services or terrain change in this step.

```sh
target/interior-life/wasm-dev/pack_minnesota_interiors \
  content-packs/minnesota-interiors-r05-current-v01.crystalpack \
  planning/minnesota/interior-life/current-story-v01/regional-flavor.json \
  content-packs/minnesota-interiors-regional-flavor-v01.crystalpack
target/interior-life/wasm-dev/pack_northstar_current_story \
  content-packs/minnesota-interiors-regional-flavor-v01.crystalpack \
  planning/minnesota \
  content-packs/minnesota-interiors-current-story-v03.crystalpack \
  planning/minnesota/interior-life/current-story-v01/composition.json
target/interior-life/wasm-dev/rebase_accepted_terrain \
  content-packs/minnesota-interiors-current-story-v03.crystalpack \
  content-packs/minnesota-northstar-current-story-v03.crystalpack \
  content-packs/minnesota-r03-bank-seep-v02.crystalpack \
  planning/minnesota/interior-life/current-story-v01/r03-terrain.json \
  content-packs/minnesota-interiors-r03-current-story-v01.crystalpack
```

The optional story composition argument uses the same exact-parent checks as
the canonical builder. Chapter preimages, original art/audio, established
ordinary teams, and the regional content audit remain enforced. Native wild
forms explicitly authored in the active roster retain their chosen stage;
legacy imports and trainer/gift binding keep their level checks. Unknown names
and zero or overflowing encounter levels remain errors.

## Verification

The strict Rust publishers completed with 729 maps and read back their output.
The thirteen added rooms are the Boundary intake, six ICE floors and six Iron
Range workings. Their map IDs are allocated from the actual parent. The
separate R03 synchronization changes only `MnR03Cave1F`.

The story preservation regression replays the complete builder and checks all
existing room dialogue, resident identities/positions, nonstory modules,
outdoor terrain, original runtime files, audio, encounter catalogs, and numeric
event identities. It also rejects stale or unreviewed parents and duplicate
installation. The focused encounter regression passes without relaxing trainer
or gift checks.

Ten fresh complete facade frames and four river/bridge frames match installed
native source pixels exactly, without exclusions. All fourteen production
frames and the whole city were visually inspected. The complete St Paul
module equals the previously accepted repaired city. Render and preservation
receipts remain ignored under
`planning/minnesota/renders/interior-life/current-story-{city-native,shore-native,composition}-v01/`.

All five room-life controller cases pass on the final child (202.44 seconds):
117 new residents remain reachable, care restores resources, full pockets keep
unclaimed rewards through Continue, weather/labor errands persist, and all
eight corrected regional partner conversations print through actual input.
All three St Paul cases pass (71.66 seconds), covering 39 ordinary door pairs,
both clinics and PCs, both bridges, Surf underpasses, river seams and Continue.
All 377 audio programs pass PCM/hash/frame/loop verification again.

All four furnishing contracts pass independently: main rooms (62.05 seconds),
clubs (35.63), harbor care (13.23), and fishing houses (13.08). Residents,
readable furniture details and native returns remain usable. All four fair
controller cases pass (279.32 seconds): seven room interactions, six carpets,
stamps/quiz/prize, battles, 32 shops, sixteen food effects, full-pocket retries
and Continue.

All seven rival-arc controller cases pass (416.55 seconds), including all 24
grunt battles, three-starter early rival branches, loss guards, acted release,
optional station aftermath and Continue. The headquarters geometry case also
passes its card, eight pads, six doors, optional rooms and rescue course.
Its intended-level case exposed stale city expectations during the shop
return: both the canonical story pack and this child put the office return at
`(25,34)` and the Mart return at `(73,26)`, while the test still expected
`(27,34)` and `(55,16)`. The test now resolves both reciprocal indexed ports
from the actual pack and asserts passive FLOOR arrivals. The retry passed the
office exit, then exposed the generic indoor planner's exclusion of native
bridge decks. A read-only native collision search cannot reach the Mart with
FLOOR alone and can reach it with `BRIDGE_NS_DECK`/`BRIDGE_EW_DECK` included.
The two city walks now use the existing bridge-aware room/city walker. A replay
loads the actual reached relay checkpoint and exercises the remaining nurse,
shop, boss, rescue and Continue inputs. The checkpoint replay passes (86.23
seconds): five Super Potions are available after the paid shop visit, the
enforcer is defeated, Ellis is rescued, the separate dispatch note remains
unclaimed, and the rescue survives Continue. The full three-starter course
also passes (289.66 seconds), using the actual unlocked nurse before both
the rival and enforcer. A prior retry exhausted its party before the rival;
the revised route walks back for care. Fixture levels, ordinary DVs and carried
supplies are unchanged. Battle results, supplies, power shutdown, rescue and
Continue come from production input. Failed invocations remain retained.

The passing replay uses the reached input copy
`target/interior-life/current-story-relay-replay-v01/relay.crystalsave`, SHA-256
`a1cc0c29f180e9015c91e798e10071c26a6ac28def25baef535a87d44e5cb5fa`.
After the v04 Bevy test build completes, set `CRYSTAL_ICE_RELAY_CHECKPOINT` to
that absolute path and `CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR` to a fresh isolated
checkpoint directory. Run
`ice_headquarters_reached_relay_checkpoint_walks_city_bridges_and_finishes_rescue`
with `--ignored --test-threads=1`; require one executed case, not an empty
filter result. This replay covers one actually reached local-fixture checkpoint;
it is not a three-starter or earned-campaign claim.

The warming hut is now included in the canonical room registry. Its current
single carpet, passive arrival and reciprocal exterior indices match the
compiled module and frozen Rainy River source. Both the original keeper and
the added mitten mender are recorded, with the border report kept separate.

All thirteen added story floors are also registered with their actual paired
native stair/panel indices and explicit purposes. Thirteen complete 2D frames
match all source pixels; every full layout was reviewed against native
facility, warehouse and cave art. The later
[Voyageurs surveyor overlay](../VOYAGEURS-SURVEYOR.md) then inhabits the last
character-free room, retaining this entire story/city composition.

Initial positions, parties and story prerequisites in local controller tests
are explicit fixtures; they do not prove earned campaign progression. This
composition does not complete the broader all-interiors objective, the Falls
regional circuit, or passenger-service work. The independently selected
regional branch has subsequently added the Mille Lacs park and further route
work; those additions are not yet in this local 729-map child.

The controller fixtures use the actual final child and cumulative resident
relocations. Story Save/Continue cases additionally require an isolated
checkpoint directory; omitting it fails fixture setup before a checkpoint can
be made. The rejected first invocation is retained as
`target/interior-life/current-story-rival-controller-v01.log`.

```sh
export CRYSTAL_INTERIOR_PACK="$PWD/content-packs/minnesota-interiors-r03-current-story-v01.crystalpack"
export CRYSTAL_WINONA_TEST_PACK="$CRYSTAL_INTERIOR_PACK"
export CRYSTAL_INTERIOR_ACTOR_RELOCATIONS="$PWD/planning/minnesota/interior-life/terrain-sync-v01/resident-relocations.json"
export CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR="$PWD/target/interior-life/current-story-isolated-saves"
export RUST_MIN_STACK=67108864
```

Run the focused `interior_life_`, `st_paul_refinement_` and `rival_arc_`
filters in the `crystal-bevy` library test executable with
`--ignored --test-threads=1`. The checkpoints are temporary verification output,
not content-pack inputs or shipped artifacts.

Fair checks use `CRYSTAL_FAIR_TEST_PACK` set to the same final child and
`CRYSTAL_FAIR_SOURCE` set to
`planning/minnesota/interior-life/fair-woodland-v01/dossier.json`. The first
invocation omitted these two fixture variables and was rejected before play;
both that log and the four-case passing retry remain retained. Furniture
checks set `CRYSTAL_INTERIOR_LAYOUT_SOURCE` to each of `layouts.json`,
`club-layouts.json`, `harbor-care-layouts.json` and `fishing-layouts.json`.

Changed Rust leaves pass `rustfmt --check`. Workspace formatting still fails
first at the unrelated `offset-allocator/src/lib.rs:408`. Normal all-target
Clippy stops at the existing core `bad_bit_mask` denial in
`multiplayer/mod.rs:1332`. The supplemental all-target check of both affected
crates passes with `--no-deps` and the existing `bad_bit_mask`/`never_loop`
denials allowed. Baseline warnings remain; the new story-composition,
encounter-regression and regional-conversation leaves have no diagnostics.
