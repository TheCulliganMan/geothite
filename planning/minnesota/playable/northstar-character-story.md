# Northstar character story

The authored revision gives Reed an ICE recruitment scene and later paid
obstruction at the drill. He knows whom ICE is holding. His aftermath keeps
him resentful; restoring the water does not redeem him.

Tamarack protects displaced eggs and offers an optional Gillpup egg. Ivy asks
for the missing crew. Ellis returns home to Jo and gives the next lead after
the rescue. Mina challenges the depot courier and asks the player to check
her brother's lift crew. Nell stays above the mine because Finn is below;
Finn brings out Ellis's pressure meter and asks for its repair in Grand Rapids.
Each request comes from a person in the scene. Old dispatch, shipping, and
meter-case text cannot award their quest results.

## Executable content

- Minneapolis cargo floor: native guard, Uma, a winding route to the breaker,
  sealed relay stair, and an opened return route after power isolation.
- Minneapolis relay floor: native guard, Reed battle, separate holding-power
  control, then the original enforcer battle and captive-release control.
- Iron Range: public foreman battle, crosscut, pump-gallery maze and valve,
  witnessed piston strokes, Reed battle, separate drive shutdown, and Finn
  walking out before offering the meter quest.
- Save/Continue is checked between completed scenes. Mid-dialogue frame saves
  are render references, not evidence that an interrupted script can Continue.

## Regional teams

`../story-pokemon.json` is the final binding authority. It filters the roster
through active curation, validates actual pack learnsets and evolution levels,
and rejects unresolved story species. Ordinary trainer party sizes and levels
are preserved; explicitly authored faction teams are the documented exception.

ICE mixes Cindore, Cartick, Leechimp, Midgebit, Sludgeski and the Wrigglet line.
The enforcer's Wrigglet carries a Berry. Reed's mine team ends in Pupump;
the Lode chief also uses Pupump with a Gold Berry. Mosquitan is reserved for
its legal level range, not installed as an underlevel boss.

## Local publication

Packs are ignored local build products. Preserve the frozen input packs and
publish to a new version when the source changes. Run the regional binder
last; it also rebuilds Minnesota script cry indices from the final scripts.

```sh
RUST_MIN_STACK=67108864 target/northstar-story/wasm-dev/pack_northstar_ice_lair \
  content-packs/minnesota-northstar-events-opening-v3.crystalpack \
  planning/minnesota/lairs/minneapolis-events.json \
  planning/minnesota/lairs/minneapolis-cargo.json \
  planning/minnesota/lairs/minneapolis-relay.json \
  content-packs/minnesota-northstar-ice-lair-v5.crystalpack
RUST_MIN_STACK=67108864 target/northstar-story/wasm-dev/pack_northstar_drill_lair \
  content-packs/minnesota-northstar-ice-lair-v5.crystalpack \
  planning/minnesota/lairs/iron-events.json \
  planning/minnesota/lairs/iron-approach.json \
  planning/minnesota/lairs/iron-gallery.json \
  planning/minnesota/lairs/iron-core.json \
  content-packs/minnesota-northstar-drill-lair-v4.crystalpack
RUST_MIN_STACK=67108864 target/northstar-story/wasm-dev/pack_northstar_regional_cast \
  content-packs/minnesota-northstar-drill-lair-v4.crystalpack \
  planning/minnesota/story-pokemon.json \
  planning/minnesota/pokemon/roster.json \
  planning/minnesota/pokemon/roster-curation.json \
  planning/minnesota/pokemon/species-profiles.json \
  content-packs/minnesota-northstar-character-story-v5.crystalpack
```

These commands use the Rust publishers built with the pinned toolchain,
`CARGO_TARGET_DIR=target/northstar-story`, profile `wasm-dev`, and
`CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0`. Publication verifies the written pack's
runtime identity. Native gameplay, balance, stdio and visual results are
recorded separately; publication alone is not playability acceptance.

## Acceptance record

The v5 publication completed and runtime-verified with 559 maps. Its SHA-256 is
`83f05198cfbe3a27b041eef0737e4882ca5f75bca503dc36f4ec5e1b9e4eb0f5`.
All nine Northstar asset tests passed, covering dialogue publication and
idempotence, both lairs, regional binding, rejected invalid input, original
source-region preservation and unchanged audio. An independent v3 read audit
found zero legacy party species across all 165 Minnesota trainers and exact
agreement with the six authored faction teams. The v5 revision raises the
enforcer's Leechimp to its legal level-22 Bubblebeam, strengthening Water
coverage. The final regional asset suite was repeated against the v4 drilling
parent and passed both tests, including the current faction policy.

All nine final-candidate native tests passed: three opening/egg/loss cases,
Mina, both ICE entry/full-rescue cases, two intended-level balance cases, and
the full drill course. The balance fixture
uses a level-23 starter with Gillpup, Cindore and Notchbill at level 22,
midrange DVs, no trained stat experience, and two Super Potions. Its solo
comparison has no items. The drill course uses a declared prior-access fixture
with Borealynx 48, Mudmender 45, Portabeau 44, Goosnide 44 and six Super Potions;
it verifies scenes and geometry, not intended-level battle balance.

Paired native 2D/2.5D captures at noon, the same position and facing, were
reviewed in `../renders/story-events-v2/native-drill-v4/`. The worker/foreman
pages hold opposite piston states with the actors readable above the text.
Those captures use v4; v5 changes a trainer party only. The opening egg case
and ICE recruitment comparisons were reviewed separately in their versioned
render directories. No visible window or focus change was used.

Mina and Finn also passed refusal/retry, quest acknowledgement, repeat dialogue
and Start over actual stdio MCP with the rebuilt client and v5 earned scene
saves. Both clients exited cleanly with byte-empty stderr.

Prepared parties from all three starter families won. The solo Grass and Fire
starters lost; solo Portabeau won at 32/70 HP. This is a measured fixture
comparison, not a guarantee for every RNG sequence or party. The complete ICE
route passed recruitment, both floors, actual rival/enforcer battles, separate
controls, release, return home, and the acknowledged Ellis quest.

Scoped Rust formatting passed. Full workspace `cargo fmt --all -- --check`
still fails on pre-existing `offset-allocator` formatting (starting at
`src/lib.rs:408`). Clippy completed for both affected crates and all targets
with the location-tester feature and the existing allowances for never_loop,
bad_bit_mask, approx_constant and useless_conversion. New Northstar warnings
were fixed; unrelated existing warnings remain. A one-line sibling-module
import repair in `tests/river_route_fun.rs` was required to compile the shared
Bevy test target. No unrelated formatting/refactoring was applied.

The chapter tests use explicit prior-access fixtures for the later quests.
They do not establish a fresh, uninterrupted regional Hall of Fame run.
The historical boosted Hall of Fame journey is not proof of this revision.
