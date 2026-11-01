# Elite Four and Champion finale

The installed League already had five battles, registration, room gates and a
Hall of Fame ceremony. Its dialogue was a short scaffold: four repeated victory
and exit speeches, a brief Casey introduction, and a stale Reed name in room 4.
This revision writes the cast and ending in `dialogue.json` and installs them
with the Rust `pack_northstar_league_dialogue` publisher.

- **Fern** surveys nests in roadside prairie grass. She rewards watching for an
  opening instead of overlooking small opponents.
- **Flint** maintained underground supports. He tests pressure and adaptation;
  being a mine worker never makes him a Team Lode supporter.
- **Veil** volunteers on night searches. Her quiet attention informs how she
  battles, without a supernatural monologue.
- **Halden** guides canoe trips and reads momentum. He is the last Elite Four
  member. This restores the existing story-source identity and distinguishes
  him from the antagonist rival Reed.
- **Casey** recognizes the reopened Ely trail, then insists the title must be
  earned in battle. Defeat brings a concrete assessment of the player's win,
  respect for every party member and a promise to challenge the new champion.

All five have individual introductions, defeat quotes, player-loss quotes,
post-win conversations, registration denials and championship-aware rematch
introductions. Only a completed Hall of Fame recording selects rematch dialogue;
beating Casey without recording does not prematurely award the title. Repeating
a finished room still uses its after-conversation and cannot pay out again.
The registrar names the full cast and explains the five-match run. The Hall
of Fame recognizes the winning team while acknowledging that recovery outside
continues; it does not erase the damage or absolve the villains.

The overlay preserves native teams, levels, moves, held items, encounter and
battle rules, badge/restoration access, run counter, exit gates, native loss,
Hall of Fame recording, credits and recovery. It changes only seven League
maps' scripts/text caches, the fourth trainer's name and publication metadata.
Every installed League party is validated against the active Minnesota roster,
excluding retired families. Terrain, art, audio and unrelated maps stay exact.

The first immutable publication is
`content-packs/minnesota-northstar-league-dialogue-v01.crystalpack`, based on
`content-packs/minnesota-region-polish-final-v03.crystalpack`. Do not load saves
bound to that parent into this publication. This is an ignored local game pack.

```sh
RUST_MIN_STACK=67108864 target/northstar-story/wasm-dev/pack_northstar_league_dialogue \
  content-packs/minnesota-region-polish-final-v03.crystalpack \
  planning/minnesota content-packs/minnesota-northstar-league-dialogue-v01.crystalpack
```

The latest immutable child is
`content-packs/minnesota-northstar-league-dialogue-v02.crystalpack`, replayed on
the currently selected `minnesota-current-grass-r04-v02.crystalpack` so the grass,
compact geography and cave work remain present. The active grass author has
the final overlay command for their next selection; this chapter does not
overwrite their world/terrain selector.
File SHA-256:
`bd36007878c271ef4842d021d8742ef8df5d370408c88bb9d8254cb472d2eda0`.

The first publication passed both asset cases (13.21 seconds), both native
cases (143.15 seconds), all five actual stdio introductions and a separate
postgame Continue/registration/refusal/retry replay. All stdio processes exited
cleanly with byte-empty stderr. All 36 native text programs fit 18 columns.
The current child also passed both asset cases in 11.29 seconds and both native
cases in 118.57 seconds. All five fresh production stdio introductions pass on
that exact pack, using `target/route-grass/wasm-dev/geothite`. A separate postgame
stdio Continue/registration/refusal/retry replay also passes with byte-empty
stderr. The reached saves
are `.local/northstar-league-written-v02-saves/`; the native Hall of Fame save
is `hall-of-fame.crystalsave`. Logs live under
`.local/northstar-league-dialogue-current-*`.

Assets/Bevy all-target Clippy passes with the existing baseline lint allowances
and pre-existing warnings. Changed leaf files pass scoped rustfmt and whitespace
checks; workspace formatting still reports unrelated existing differences.
The native proof declares prior badges/restoration, six level-100 Minnesota
partners and carried medicine. It proves scripts and persistent progression,
not an earned campaign or a difficulty acceptance.

Launch the written finale's current complete game pack with:

```sh
target/route-grass/wasm-dev/geothite play \
  content-packs/minnesota-northstar-league-dialogue-v02.crystalpack
```

Reapply the overlay to later terrain successors with the same Rust publisher
and a fresh ignored output filename. It rejects inactive League species,
preserves all teams and all other map modules, and does not copy old geometry
back into a newer parent.
