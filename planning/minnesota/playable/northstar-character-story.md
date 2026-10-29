# Northstar character profiles

The current story sources are summarized in [STORY.md](../STORY.md). These
profiles describe the roles present in those sources. They do not add biography,
new quests or validation claims. Follow the final `dialogue_overhaul` text in
[story.json](../story.json) where it replaces an earlier scene label.

| Person | Concern | What the player sees or does |
| --- | --- | --- |
| Mom | The player and their new partner | Lunch before departure, a request to meet the partner and a place to rest on return |
| Dr Tamarack | River Pokémon, the field crew and the cold egg Reed brings in | Entrusts Ivy's original readings to the player, tends the egg and offers it after delivery |
| Ivy | Missing Ellis, diverted meters and her crew | Receives the envelope, directs the player to Jo, then helps at the intake |
| Ellis | Stolen equipment and getting home | Needs the holding power disabled and the cell opened; returns to Jo and gives the St. Paul lead |
| Jo | Ellis's safety | Explains his disappearance and responds to his return before the story moves on |
| Mina | The shipment records and Lode's use of the equipment | Challenges the operation, holds the next shipment and asks the player to follow the crew north |
| Nell | Finn is still beneath the running drill | Asks for shutdown and reacts when he gets out |
| Finn | The crew under the drill and Ellis's damaged meter | Leaves after shutdown and offers the meter for repair in Grand Rapids |
| Sera | Refuses a forced transfer | Leaves when the player intervenes between her and Reed |
| Reed | Paid backing, a stronger team and authority | Seeks ICE work, sees the captive and still accepts; later protects Lode's shipment, drill and intake |

## Character continuity

Reed has six battles: Winona, Rochester, the ICE relay, St. Paul, the mine and
the Boundary intake. His interview, recruitment and transfer scenes connect
those encounters. He knowingly stays with the operation as its harm becomes
clear. Defeat does not redeem him. Halden remains the fourth League trainer;
other trainers with their own identities must not be cast as Reed.

Ellis's rescue is separate from defeating a guard. Mina's request is separate
from a depot battle. Finn's handover is separate from stopping the drive.
Found papers and the empty meter case cannot grant those spoken quest results.
The old event names remain exact implementation identifiers; they do not make
an obsolete paper interaction authoritative again.

Tamarack's Gillpup egg is optional. Declining or having a full party preserves
the offer. Only successful delivery records receipt. It does not require a
hatch, capture, PC fallback or new story gate.

## Minnesota convention

Use the named characters and relationships established by the sources. Do not
add a surname, hometown, family tie, job or personal history to make a profile
seem fuller. Keep dialogue direct and tied to the current scene. Avoid forced
regional accents, interchangeable quest speeches and claims that the whole
region is healed when the pumps stop.

The starters are Pouchid, Kindlit and Birchit. The egg is Gillpup; Reed's first
partner is Crookit. All later parties and visible creatures must resolve through
[story-pokemon.json](../story-pokemon.json), the active
[roster](../pokemon/roster.json) and [curation](../pokemon/roster-curation.json).
Do not copy Johto species from historical fixtures into a current profile.
Appearance and sprite acceptance belong in the [cast records](../npcs/cast/README.md).

## Implementation and evidence

The [current composition record](../revisions/current-story-v01/README.md)
describes the compact regional publication and its recorded chapter checks.
The [rival arc record](rival-arc.md) retains the older publication history;
[ICE headquarters](ice-headquarters.md) and [Iron Range workings](iron-workings.md)
retain their layout and local-test details. Their pack identities, fixtures and
page counts apply to their own versions.

The previous character-story-v5 publication and its chapter/balance results are
retained in the [historical record below](#historical-character-story-v5-record).
They do not certify the later recurring-rival composition or final dialogue edits. No new gameplay, visual or
balance test is claimed by this documentation cleanup. A full uninterrupted
ordinary-party campaign remains unclaimed.

## Historical character-story-v5 record

The following record retains the earlier publication, source state and test
fixtures. Its character text and commands are superseded for current authoring.

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
