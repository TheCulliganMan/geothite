# Rochester: original readings and the missing shipment

This is a versioned chapter and verification record. Its commands, parties,
page counts and results apply to the packs named below. For current narrative,
character roles and completion actions, read [STORY.md](../STORY.md),
[PROGRESSION.md](../PROGRESSION.md) and the
[current composition record](../revisions/current-story-v01/README.md).
Use the current composition path for new publications. Later source changes
need their own pack verification. Historical fixtures do not establish an
uninterrupted ordinary-party campaign.

## Historical chapter record

The connected candidate is
`content-packs/minnesota-northstar-rochester.crystalpack`: 526 maps, 251 species,
251 moves and 377 audio programs. SHA-256:
`51d455252b0b876a2a9d48a1b6666d72354f67bdb3cfc20f702e04382c421670`.
Its exact input is the [521-map Winona opening](winona.md), SHA-256
`2f8c74a72a51affc94fc416763505fca5e8842bbb52b17f5662ee212e40eb2da`.
This lineage retains the integrated Minneapolis ICE chapter, northern plot,
Avrin gym and Ryan's waterfall cabin. Independent later map branches are not
implicitly merged here.

## Earned story and travel

New Game still begins upstairs in Winona. After the guardian scene, a real
starter and the original readings, the player can walk west into Driftless
Trail and arrive at Rochester's eastern road. A two-lane Winona checkpoint
requires the starter event; neither badge nor admission payment is required.
The return trip is free. Only two forest metatiles open: Winona `(0,9)` and
Rochester `(19,9)`. All original R01 terrain, actors, encounters, cave and farm
remain exact.

The actual reciprocal seams use core coordinates:

| Crossing | Source edge | Destination | Metatile offset |
| --- | --- | --- | --- |
| Winona west → R01 east | `(0,18)/(0,19)` | `(63,24)/(63,25)` | -3 |
| R01 east → Winona west | `(63,24)/(63,25)` | `(0,18)/(0,19)` | +3 |
| Rochester east → R01 west | `(39,18)/(39,19)` | `(0,24)/(0,25)` | -3 |
| R01 west → Rochester east | `(0,24)/(0,25)` | `(39,18)/(39,19)` | +3 |

The clinic's separate survey recipient stands at `(11,8)`, approached from
`(11,9)`. The native scene checks the starter and originals events, then reads
the two MS01 pages: the meters never arrived, and the crate has a Lode stamp.
Movement remains blocked during dialogue. Only acknowledging both pages sets
`EVENT_MN_DELIVERED_ROCHESTER_PARCEL`. Repeats acknowledge the retained copy;
they do not grant resources or repeat delivery. The survey bundle remains the
canonical event rather than a new bag item.

Gym entry is independent of delivery. Before delivery the leader directs the
player to the clinic with the parcel. Delivery enables the existing first-gym
qualifications: Tavel, Elvik, then Varel. The two trainers, actual leader battle,
Care Badge storage and retryable Headbutt TM use the native gym implementation.

## Physical publication

Rochester is a city authored at 20×16 metatiles: a brick clinic courtyard,
central grove, two residential lanes, mart and gym. All five visible doors are
enterable. Original Johto Modern art, source collision and native room art are
used. No incidental Cut tree or unbound walking/Surf edge is added. The western
R02 port remains a closed reservation.

| Room | Exterior door → safe return | Passive interior arrival | Carpet exits |
| --- | --- | --- | --- |
| Clinic | `(15,7)` → `(15,8)` | `(10,13)` | `(10,15)/(11,15)` |
| Mart | `(29,9)` → `(29,10)` | `(6,5)` | `(6,7)/(7,7)` |
| Gym | `(30,23)` → `(30,24)` | `(6,17)` | `(6,19)/(7,19)` |
| Clerk's home | `(7,7)` → `(7,8)` | `(5,6)` | `(5,7)/(6,7)` |
| Patient's home | `(7,23)` → `(7,24)` | `(5,6)` | `(5,7)/(6,7)` |

Five new maps use group 74. The existing Gym1 keeps group 63/map 1 while its old
8×10 review room is explicitly replaced with the captured canonical 7×10
planter route. Its three owned trainers and five private art files are refreshed;
unrelated trainer/event numbers remain stable. The captured source is
`revisions/rochester-gym1.json`; its selected-object digest is
`160c6bb9b03aa6baabfb4717d270a6866b2b6b49ebf1627c205921274f076f87`.
City return destinations and the two authored delivery-refusal lines are explicit
city adaptations. Other gym source drafts are outside this publication.

All original HOME/recovery destinations remain exact; city and clinic recovery
records are additive. No state, badge or quest completion is awarded by installing
the pack. Existing saves from a different pack or inside the obsolete gym are
not promised a migration.

## Verification and remaining work

- The earned production-controller case starts at New Game, chooses Cyndaquil,
  walks R01 with native RUN for incidental wildlife, reaches the real gym before
  delivery, reads and acknowledges the two clinic pages, continues from saves,
  checks repeat resources and walks home through both reciprocal seams. No
  position, party, level, flags, bag, money or RNG are supplied by that journey.
- A second fresh-start case proves both exit lanes refuse departure without a
  partner and do not grant progress.
- A separate local combat fixture supplies an owned legal level 16 Quilava and
  completed delivery. It earns both trainer victories, Varel's actual defeat,
  badge and one TM, checks EXP/selected Ember PP, repeat behavior, both city
  carpet exits and Continue. The additional earned audit below uses no supplied
  partner or delivery state; neither case establishes regional battle balance.
- Two preservation/rejection cases pass against the exact written identity:
  five added maps, three declared existing-map changes, actual uncached
  reconstruction, only Gym1's private art/roster changes, bounded metadata and
  recovery changes, and exact unrelated data/audio. Duplicate publication,
  unsafe arrivals, missing delivery prerequisites and changed gym source fail.
- Actual stdio passes the opening on this pack, then the delivery/city flow from
  an earned arrival save: gym prerequisite, both pages and input ownership,
  repeat, all five facades, paired clinic/gym exits and Start. Both processes
  exit with byte-empty stderr.
- All four Winona cases pass on the successor. Six core coordinate-event tests
  and eleven shared controller cases pass, including original Mom/New Bark
  and battle-command/PP behavior on the original TUI pack.
- All 377 PCM hashes, frame counts and loop ranges pass on the written pack.
  Whole city, all rooms and native noon/down-facing clinic-court, survey-desk
  and gym views were inspected. Every original, stage and rejected render is
  retained in `renders/rochester/`.

The published-source check matches all ordinary terrain/actors after the one
declared city-mouth patch, and matches Gym1's native private glyphs/palette
pixel-for-pixel to its original source recipes. The shared ordinary-town helper
extraction also reproduced the prior Winona pack byte-for-byte.

Uncached verification exposed the native `SCENE_ALWAYS` sentinel: serialized
`-1` must remain active even when the map has no scene value. The shared core
matcher now accepts both native forms and the existing empty authored form.
This follows the [native constant definition](https://github.com/pret/pokecrystal/blob/master/constants/script_constants.asm)
and the matching pinned local source. The event-specific scene comparison and
pack-order behavior remain covered by regression tests. The checkpoint also
omits a dead badge-refusal branch when its only requirement is a starter.

A fresh earned-combat audit exposed a separate native stat-effect bug: Leer,
Smokescreen and Growl in this unchanged pack omit optional `stat`/`amount`
annotations. The turn engine now obtains their ordinary stage parameters from
the native effect program when both annotations are absent. Explicit authored
parameters, including zero and partial pairs, keep their previous precedence.
This follows the pinned external source's `data/moves/effects.asm` programs;
no source dump or additional content catalog is added. Two regressions execute
unannotated native effects through complete turns, verify actual stage changes,
PP spending and zero damage, and preserve explicit overrides. The available
626 turn cases pass; five existing source-inventory cases fail because their
hard-coded `../../../vendor/pokecrystal` location is absent. The strict earned audit now has one complete New Game success, recorded
below. A later run legitimately blacked out at Varel and failed that audit;
repeatability, retry coverage and early-party balance remain open.

The earned audit starts with the level-5 Cyndaquil from the actual opening,
delivers the originals, walks to Brook/Lena/Pat, collects the existing bluff
Potion and earns Tavel/Elvik's victories. It buys eight Potions (and conditionally
two more) at the real mart, with exact money/item assertions. Before Varel it
searches the existing R01 grass for an ordinary level-8-or-higher Rattata,
Sentret or Hoothoot, weakens and catches it with an opening Poké Ball, then
returns to the clinic. Native Smokescreen/Leer, FIGHT, forced replacement,
Potion targeting and the pending move-learn menu all participate in the win.
Only the actual victory and acknowledged reward produce badge1 and one
Headbutt TM. Continue preserves the party, resources and earned trainer flags.
No party, position, level, EXP, inventory, money, event, badge or RNG writes
supply this journey. The bounded live-RNG audit remains ignored/manual: its
later blackout is not a passing earned-badge result. No all-starter balance or
reliable fixed-input win rate is claimed.

The audit uncovered two further production blockers. Battle Pack vertical
navigation was reaching the field Pack owner; a controller regression now moves
between item rows and party targets and proves exact Potion consumption and
bench healing. Reward move learning was invoking a field-only evolution accessor
before the defeated battle had released cleanup, and treating an ordinary
non-evolving level check as an error. Completed, flagged rewards now finish the
check without ending the battle; a remaining live opponent still prevents it.
Five deferred-evolution cases pass, including replacement/decline with no
eligible evolution, a later trainer opponent, actual eligible evolution and
preserving the field guard. New leaves introduce no Clippy warnings under the
documented allowances. The production client is rebuilt with these fixes.

Actual stdio also loads the retained earned badge save, visits Varel again,
reads the Mankato direction, walks to free clinic care and opens Start. Badge
count and money remain unchanged and stderr stays byte-empty. Opening and
pre-delivery stdio are rerun successfully. PC transfer and broader service/balance
coverage, R02/Mankato, remaining regional links, courier, League and Kettlurk
remain unfinished. The completed local chapter is not a full regional playthrough.
Final user visual approval remains open.

Scoped formatting passes. Workspace formatting still fails first at
`offset-allocator/src/lib.rs:408`. Default all-target Clippy stops at the existing
`crystal-core/src/multiplayer/mod.rs:1332` bad-bit-mask error. With the four
documented `never_loop`, `bad_bit_mask`, `approx_constant` and
`useless_conversion` allowances, core all-target checking reaches pre-existing
missing relative radio-fixture includes in `crystal-core/tests/radio_text.rs`
(`../../../../tools/asm-oracle/fixtures/...`). Core library Clippy and
assets/Bevy all-target Clippy complete under those allowances with existing
warnings. Shared spawned-client clean-stderr also passes. No broad green
workspace/all-target core claim is made.

## Executed build and replay

Use the pinned Rust toolchain. Packs, builds, review saves and renders stay ignored.

```sh
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
export RUST_MIN_STACK=33554432
cargo build --locked --profile wasm-dev -p crystal-assets --bin pack_rochester
target/northstar-story/wasm-dev/pack_rochester \
  content-packs/minnesota-northstar-winona.crystalpack \
  planning/minnesota/towns/MnRochester.json \
  planning/minnesota/towns/MnRochester.layout.json \
  planning/minnesota/story.json \
  planning/minnesota/revisions/rochester-gym1.json \
  planning/minnesota/playable/rochester-links.json \
  content-packs/minnesota-northstar-rochester.crystalpack
cargo test --locked --profile wasm-dev -p crystal-assets --test rochester -- --ignored
export CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-rochester.crystalpack"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/rochester-review-saves"
cargo test --locked --profile wasm-dev -p crystal-bevy --lib \
  'bevy_shell::tests::rochester::' -- --ignored
cargo test --locked --profile wasm-dev -p crystal-bevy --lib \
  'bevy_shell::tests::winona::' -- --ignored
cargo test --locked --profile wasm-dev -p crystal-core --lib coord_event_
cargo build --locked --profile wasm-dev -p geothite
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_winona_mcp.py \
  --pack content-packs/minnesota-northstar-rochester.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_rochester_mcp.py \
  --save .local/rochester-review-saves/winona-rochester-earned-arrival-74798-1793078738725228000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_rochester.py \
  --pack content-packs/minnesota-northstar-rochester.crystalpack --verify-published
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-rochester.crystalpack --audio --desktop
```

The manual earned acceptance run (which can fail on an ordinary live-RNG loss)
is separate from the deterministic mechanic regressions:

```sh
cargo test --locked --profile wasm-dev -p crystal-bevy --lib \
  rochester_earned_ -- --ignored --nocapture
cargo test --locked --profile wasm-dev -p crystal-bevy --lib \
  renderer_neutral_battle_pack_
cargo test --locked --profile wasm-dev -p crystal-assets --lib deferred_
cargo test --locked --profile wasm-dev -p crystal-core --lib native_stat_
```

The successful earned badge save is
`.local/rochester-review-saves/winona-rochester-earned-care-badge-85160-1793082601073571000.crystalsave`.
It is from this exact unchanged pack and the rebuilt client, rather than the
separate level-16 gym fixture. Its actual stdio replay was executed with:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_rochester_mcp.py \
  --earned-badge \
  --save .local/rochester-review-saves/winona-rochester-earned-care-badge-85160-1793082601073571000.crystalsave
```

The fresh New Game and retained earned arrival can be played locally:

```sh
target/northstar-story/wasm-dev/geothite play \
  content-packs/minnesota-northstar-rochester.crystalpack
target/northstar-story/wasm-dev/geothite play \
  content-packs/minnesota-northstar-rochester.crystalpack \
  --load .local/rochester-review-saves/winona-rochester-earned-arrival-74798-1793078738725228000.crystalsave
```

The interactive commands are provided for local play. Verification used the
actual stdio process and offscreen captures without opening a window.
