# Trainer rematches

An opt-in Rust overlay for the supplied local Crystal content pack. After you
defeat a trainer, talk to them again. Their standard conversation plays first,
followed by “I've trained hard! Let's go again!” Advance that text to start
another battle. No phone number, call, cooldown, or extra rematch confirmation
is required; any original conversation prompts still work. Defeated trainers do
not challenge you again just for walking past them.

The overlay covers every ordinary trainer table and every scripted trainer
team in the supplied pack, including gym leaders and story opponents when
their NPC is present and available to talk. It preserves story-controlled NPC
visibility. Leaders finish their original badge and TM rewards before offering
rematches. Phone readiness flags are preserved, and a ready phone trainer uses
the same scaled rematch instead of starting a second, unscaled battle.
Rematches end without repeating story rewards or callbacks.
New League challenges retain their original battles that open the exit doors.

Each team's highest level matches the rounded average of your three strongest
non-egg party members (or all members if you have fewer than three). Fainted
Pokémon count; changing the lead or adding an egg does not lower the target.
Original levels are minimums, relative team levels stay intact, and levels cap
at 100. Species, held items, DVs, and trainer AI stay as authored.

Moves start with the original battle set. As levels rise, the Pokémon learns
its species' additional level-up moves in order, replacing the oldest move
when four slots are full. Moves are distinct and start with full PP. Species
with no later level-up moves keep their authored moves. A rematch never unlocks
a rival's or phone trainer's future team before you have beaten that team.

## Build and play

The local builds are `content-packs/trainer-rematches.crystalpack` (native),
`content-packs/trainer-rematches.browser.crystalpack` (browser), and
`content-packs/trainer-rematches-tui.crystalpack` (TUI/MCP).

The input must be an existing verified local pack, initialized using the
[content setup](../../docs/game-content.md) if needed. Keep every generated pack
under ignored `content-packs/`; no source export or new audio catalog is needed.

```sh
cargo run --locked -p crystal-assets --bin pack_trainer_rematches -- \
  content-packs/text-tui.crystalpack content-packs/trainer-rematches-tui.crystalpack
cargo run --locked -p geothite -- play content-packs/trainer-rematches-tui.crystalpack
```

Use the same builder with `core-modular.crystalpack` or
`core-modular.browser.crystalpack` to create a native or browser overlay. It
preserves the input's audio storage, audio hashes, frame counts, loops, and
unrelated content. The overlay has its own pack/save identity; keep base-game
saves separate. Victories and scaled in-progress battles persist in its saves.
The updated client writes save format 25 and validates/migrates existing
format-24 saves on load.

## Regression checks

```sh
export CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack"
export CRYSTAL_REMATCH_TEST_PACK="$PWD/content-packs/trainer-rematches-tui.crystalpack"
cargo test --locked -p crystal-assets --test trainer_rematches
cargo test --locked -p crystal-bevy renderer_neutral_controller_trainer_rematches
```

The asset suite checks every trainer table and every generated scripted team,
move progression, level bounds, win/loss eligibility, save integrity, and
preservation of original content and audio. The controller regression uses
actual A input for an ordinary trainer, a gym leader, and a ready phone trainer.
It checks the standard dialogue before the added line, movement ownership,
scaled battles, saves during dialogue and battle, and a turn with real PP
consumption and battle dialogue.
