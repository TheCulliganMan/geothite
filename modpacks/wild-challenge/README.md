# Wild Challenge

A Rust-built rules pack that keeps wild encounters relevant as your team grows
and gives them legal, complementary movesets. Standard native, browser, and
terminal/MCP builds compose it with the competitive trainer profiles and move
split through `build_default_battle_modpack`. The separate builder below also
supports standalone variants.

Wild levels use the rounded mean of your strongest three non-egg party members
(or the whole party if you have fewer than three). Most encounters range from
three levels below that mean to one level above it. The original encounter level
is always a floor, and scaling stops at level 100. A team at levels 50, 40 and 30
therefore encounters level 37–41 Pokémon on an early route. Adding a weak party
member, moving it to the lead, or fainting your strongest Pokémon does not lower
that team's anchor. Depositing Pokémon changes the team used for scaling.

Grass, caves, surfing, fishing, Headbutt, Rock Smash, Sweet Scent, fixed wild
battles and roamers use these rules. Fixed encounters use the central level
(the team mean minus one), retaining their authored minimum. Roamers only gain
levels; existing DVs and proportional damage persist. Repel compares against the
scaled encounter. The catching tutorial and Bug-Catching Contest retain their
original rules.

Movesets are assembled from each species' level-up moves available at its actual
encounter level, legal TM/HM and egg moves, and moves retained from its earlier
evolutions, all read from the supplied pack. The selector
prefers reliable STAB, coverage appropriate to the species' offensive stats,
recovery/status/setup, and complementary coverage. Bulky species can use paired
Rest/Sleep Talk. TM and egg attack power is limited to 50 before level 10, 65 before 20,
85 before 30, and 150 thereafter. Species with restricted move pools retain
those restrictions. Captured Pokémon keep their level and moves.

The selector respects the pack's damage categories. Standard packs use the
full competitive planner; standalone overlays on older packs retain Basic,
Setup, Types and Smart scoring. Live controls and mutation replay call the same
`select_wild_enemy_move_for_battle` method, including the tutorial/contest
exceptions. Wild Pokémon gain no trainer items or switching.
Trainer rosters, species stats, encounter species/rates, authored learnsets,
story scripts, graphics and audio remain unchanged.

Build from an existing verified local pack:

```sh
cargo run --locked --profile wasm-dev -p crystal-assets --bin pack_wild_challenge -- \
  content-packs/text-tui.crystalpack content-packs/wild-challenge-tui.crystalpack
```

The same builder accepts the native or browser base pack and preserves its
storage format. Reapplying existing rules preserves identity and configured
scaling. Use a distinct output path. All generated packs stay ignored.
Build fresh base content with the initializer in
[the content guide](../../docs/game-content.md) if needed.

The following local outputs have been built and reloaded through the Rust pack
verifier:

| Client | Base in `content-packs/` | Output in `content-packs/` |
| --- | --- | --- |
| Native graphical | `core-modular.crystalpack` | `wild-challenge.crystalpack` |
| Browser | `core-modular.browser.crystalpack` | `wild-challenge.browser.crystalpack` |
| Terminal / MCP | `text-tui.crystalpack` | `wild-challenge-tui.crystalpack` |

Rebuild the Rust client when using the new rules. This terminal build and launch
were verified locally with the optimized `wasm-dev` profile:

```sh
export CARGO_TARGET_DIR=target/agent-wild-challenge
cargo build --locked --profile wasm-dev -p geothite
"$CARGO_TARGET_DIR/wasm-dev/geothite" play content-packs/wild-challenge-tui.crystalpack
```

Use `mcp` in place of `play` for a stdio MCP session. Enter opens Start, Z or
Space presses A, and X presses B. Type `:q!` then Enter to quit. To save, launch
with `--save saves/wild-challenge.crystalsave`, then use F5 or `:wq`. Resume with
`--load saves/wild-challenge.crystalsave`. Choose a separate save path: pack
identities include the rules, and an existing base save does not silently
migrate into a modded game.

Run the external-pack regression suite:

```sh
cargo test --locked --profile wasm-dev -p crystal-assets --test wild_challenge
```

`CRYSTAL_WILD_BASE_PACK` optionally chooses another verified base pack. The
default is this checkout's `content-packs/text-tui.crystalpack`. No source
catalogs or generated game data are checked into this mod directory.

All 13 Wild Challenge integration tests passed, covering all 251 species at
five level bands, actual battle stats and move PP, captures, Repel, fishing,
fixed encounters, roamer damage and binary saves, and unchanged unrelated
content and audio. A full competitive wild turn also replays to the same state,
spends real PP, and rejects a forged move atomically. Tutorial/contest selector
exemptions are checked. The core roaming and binary-save regressions also passed.
With `CRYSTAL_RENDER_TEST_PACK` set to the generated terminal pack, the production
battle-turn, Mom/New Bark, MCP movement/Start, and clean-stderr regressions
passed. The interactive terminal client was launched as CHRIS, its Start menu
opened, and a scratch save written and reloaded successfully.
