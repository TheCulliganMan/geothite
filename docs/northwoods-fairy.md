# Northwoods Fairy rules

The Northwoods mod adds Fairy as an eighteenth type while retaining the Game Boy Color presentation. The regional roster remains 220 species across 100 families; thirteen regional species across eight families gain Fairy, alongside thirteen returning species. Base stats, evolutions and all sprite palettes are unchanged.

The later [Northwoods animation overlay](northwoods-move-animations.md) preserves
these Fairy v2 rules and adds distinct native presentations for all nineteen
extension moves. Its web presentation is published as release
`20261027-northwoods-12`.

| Rule | Result |
| --- | --- |
| Fairy attacks vs Dragon, Dark, Fighting | 2× |
| Fairy attacks vs Fire, Poison, Steel | ½× |
| Dragon attacks vs Fairy | Immune |
| Bug, Dark, Fighting attacks vs Fairy | ½× |
| Poison, Steel attacks vs Fairy | 2× |
| Dark, Ghost attacks vs Steel | Neutral |

All other native matchups remain intact. Foresight does not remove Dragon immunity. Fairy follows the modern per-move split: Play Rough is physical; Fairy Wind, Disarming Voice, Dazzling Gleam and Moonblast are special. Hidden Power, Metronome and Conversion2 retain their native random-selection pools. Charm, Sweet Kiss and Moonlight keep their effects and become Fairy.

| Regional species | Fairy typing |
| --- | --- |
| Slipperina → Cyprielle | Grass/Fairy |
| Chislet → Buttermuse | Ice/Fairy |
| Dozebear → Denumbra | Normal/Fairy |
| Mitwist → Mispair → Shawlter | Ghost/Fairy → Ghost/Fairy → Fairy/Flying |
| Tiltawisp | Electric/Fairy |
| Choirbog | Poison/Fairy |
| Babehemoth | Ground/Fairy |
| Flarkindle | Ghost/Fairy |

The [decision record](../planning/minnesota/pokemon/fairy-balance.json) lists each retype, old/new typing, complete 18-type defensive matchups and counter progression. The [import catalog](../planning/minnesota/pokemon/fairy-moves.json) defines the five Fairy attacks plus Poison Jab and every earned learner. Slipper Waltz is also Fairy. Moonblast is restricted to selected late-game learners; it is not granted to every Fairy.

Physical Poison attackers Barricant, Castanake and Mosquitan gain Poison Jab. Early Acid/Sludge/Metal Claw access is improved for specified lines, and Latchquill gains the existing Iron Head at level 36. These changes make Poison and Steel practical answers to Fairy without changing stat budgets. Ice/Fairy butter royalty takes 4× damage from Steel; Grass/Fairy orchids take 4× damage from Poison. Crowscare Dark/Ghost now takes 2× Fairy damage, Taconarch Steel/Dragon stays neutral, and Knotitan Dragon/Fighting takes 4× Fairy damage.

Returning retypes: Cleffa/Clefairy/Clefable, Igglybuff/Jigglypuff/Wigglytuff, Togepi/Togetic, Marill/Azumarill, Snubbull/Granbull and Mr. Mime. New native learners resolve by national number, including Mr. Mime's source-specific key.

The final v2 pack has **471 species and 270 moves**, including **26 Fairies**. Its SHA-256 is `7a5de8c0d568f0c57a04b879ad121e6e726ae471360a9860896a2c2329a6d6b2`. The full progression audit passes with zero findings across 22,000 default move sets, 623 real battles spending PP and dealing damage, and 120 evolution boundaries. Preservation tests cover all original runtime/audio bytes, all base stats, original learnset entries and original move definitions apart from the three intentional status-move retypes.

The opt-in Rust pipeline consumes the existing external native pack; it does not overwrite it:

```sh
CARGO_TARGET_DIR=target/agent-northwoods-publish cargo build --locked -p crystal-assets \
  --bin pack_fairy_type --bin pack_minnesota_pokemon --bin pack_northwoods_moves \
  --config 'profile.dev.package.crystal-assets.opt-level=0' \
  --config 'profile.dev.package.crystal-core.opt-level=0'

target/agent-northwoods-publish/debug/pack_fairy_type types \
  content-packs/minnesota-geography-review.crystalpack \
  content-packs/northwoods-fairy-base-v1.crystalpack

target/agent-northwoods-publish/debug/pack_minnesota_pokemon \
  content-packs/northwoods-fairy-base-v1.crystalpack planning/minnesota/pokemon \
  target/minnesota-pokemon/native planning/minnesota/pokemon/species-profiles.json \
  content-packs/northwoods-fairy-species-v2.crystalpack

target/agent-northwoods-publish/debug/pack_northwoods_moves \
  content-packs/northwoods-fairy-species-v2.crystalpack \
  content-packs/northwoods-fairy-northwoods-v2.crystalpack

target/agent-northwoods-publish/debug/pack_fairy_type moves \
  content-packs/northwoods-fairy-northwoods-v2.crystalpack \
  content-packs/northwoods-fairy-v2.crystalpack \
  planning/minnesota/pokemon/fairy-moves.json
```

The optional `MOVES_JSON` argument reads authored learners at runtime, so move-data iteration does not require recompiling the exporter. All resulting packs remain ignored. New species encounter placement remains the separate existing map-integration task. This ruleset changes the authored species and battle pack; it does not claim a newly playable regional campaign.

Primary references: [official type chart](https://sg.portal-pokemon.com/game/type-chart/) and the [pinned move definitions](https://github.com/smogon/pokemon-showdown/blob/8f9ab4949d0bf64d5a5ac985a298d1313f2b9a29/data/moves.ts). The single-battle engine implements the listed damage and secondary effects, without abilities, contact/sound flags, terrain or doubles mechanics.

The Fairy baseline of the [published Pokédex](https://geothite.ryanculligan.com/pokedex/?type=FAIRY) was release `20261027-northwoods-10`. Current sprite release `20261027-sprites-11` retains those rules and the verified interface while refreshing Dozebear, Denumbra and Mudmender. All 1,926 public file/page byte checks pass, plus six health/private-route checks. The final native proof includes nine production tests, all eleven controller moves and twenty signature variants. A native local-label parser fix restores Gust/Growl/Moonlight loop declarations with colons; Gust matches its exact 91-frame clock and the legacy declaration form.

Scoped Rust formatting, exporter/audit unit tests and full web integration pass. Workspace formatting still reports existing differences beginning in `offset-allocator/src/lib.rs:408`; all-target Clippy remains blocked by the existing `crystal-core/src/multiplayer/mod.rs:1332` bad-bit-mask error. The optional cold 3D material test was stopped before execution; its attempt is recorded without a passing claim.

All renders, rejected sources, final battle captures and exported palette variants are retained. The content-addressed archive now records **66,148 files** in `planning/minnesota/renders/creature-archive/inventory-170e2401835c607849bbb02de2ef2a8f974f0b464944095148045ae0515c16f4.json`.
