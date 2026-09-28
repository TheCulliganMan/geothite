# Repository rules

- Keep implementations in this Geothite repository. Audio synthesis and export
  must use Rust. Only minimal browser integration JavaScript belongs in the
  website; do not introduce TypeScript or an external exporter dependency.
- Do not add pret disassembly, ROMs, raw or exported command dumps, duplicate
  audio source catalogs, generated PCM files, or new compiled artifacts to this
  repository or its shipped bundle. Game content packs must remain external and ignored, including regenerated
  packs. Never commit them or restore them from old Git history.
- Read audio programs from the user-supplied local content pack. Do not extract
  and commit another copy of those programs or add generated audit manifests.
- Build products and temporary verification output are not deliverables to
  commit. Keep them ignored and remove task-generated scratch artifacts when
  finished. Never stage them as part of an “all code” commit.
- Verify audio changes against the pack's PCM hashes, frame counts, and loop
  ranges. Preserve unaffected audio and unrelated game content.

## Crystalpack build and local runbook

### Non-negotiable TUI/MCP architecture

- The standalone text client must construct and drive
  `crystal_bevy::VisibleShellController`. This is the supported production
  controller boundary shared with the graphical client. It owns authored
  scripts, page-by-page text, input ownership, coordinate events, trainer
  sight, warps/connections, battles, Start/Pack/Party/PC/shop surfaces, and
  visible presentation settling.
- Never implement a second dispatcher in `crystal-tui`, and never drive the
  game by calling `crystal_runtime::RuntimeGameShell::tick()` alone.
  `RuntimeGameShell` is intentionally lower-level: a tile can move while Mom,
  the New Bark exit gate, warps, menus, or post-input script continuation are
  silently skipped. If a frontend needs behavior that the controller does not
  expose, extend `VisibleShellController` and add a regression there.
- `geothite dump` proves only that the pack loads and a snapshot renders. It is
  not evidence that input plays the game. Do not report the TUI or MCP as
  working from a dump, a screenshot, or a coordinate change alone.
- Keep MCP and keyboard input on the same `VisibleShellController::press`
  path. Renderer cursor state may mirror a production cursor for display, but
  it must never become authoritative game state.
- Fullscreen Ratatui owns stdout and stderr while the alternate screen is
  active. Never add unconditional `println!`/`eprintln!` calls to gameplay,
  dialogue, movement, audio, or controller paths used by `geothite play`.
  Diagnostic output must be opt-in behind a named trace environment variable;
  dialogue tracing uses `CRYSTAL_DIALOGUE_TRACE`. Keep event data in bounded
  in-memory logs when tests need it.

- Never hand a user a `/path/to/...` placeholder when a runnable local pack is
  requested. First check the ignored `content-packs/` directory and validate the
  chosen pack with the Rust client.
- The known-good base pack path in this checkout is
  `content-packs/realtime-clock.browser.crystalpack`. Build the dedicated TUI
  pack from it with Rust:

  ```sh
  cargo run --release --locked -p crystal-assets --bin pack_text_tui -- . \
    content-packs/realtime-clock.browser.crystalpack \
    content-packs/text-tui.crystalpack
  ```

  Packs remain ignored by Git and must never be staged or committed.
- To build fresh packs, use a compatible external export workspace. It must
  contain `vendor/pokecrystal/`, `apps/web/assets/`,
  `apps/web/assets/data/content-packs/core-modular.generated.json`,
  `asm-source.lock.json`, and the Rust-generated battle animation provenance
  consumed by `pack_core`. From this Geothite checkout run:

  ```sh
  cargo run --release --locked -p crystal-assets --bin pack_core -- \
    /absolute/path/to/compatible-export-workspace
  ```

  The builder writes `content-packs/core-modular.crystalpack` and
  `content-packs/core-modular.browser.crystalpack` inside the external
  workspace. Do not copy either into tracked source paths.
- `/Users/ryanculligan/repos/pokecrystal-python` currently contains a legacy
  export schema, not the compatible generated manifest. Do not claim a pack was
  built from it unless `pack_core` finishes successfully; its present failure
  starts with the missing explicit `MASTER_BALL` item identity.
- Validate a pack and the text renderer without entering fullscreen mode:

  ```sh
  cargo run --locked -p geothite -- dump \
    content-packs/text-tui.crystalpack
  ```

  This is a render smoke check only. Before calling the client playable, run
  both controller and stdio-MCP regressions with the freshly built external
  pack present:

  ```sh
  CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
    cargo test -p crystal-bevy \
    renderer_neutral_controller_exposes_battle_commands_and_executes_a_turn
  cargo test -p crystal-bevy \
    renderer_neutral_controller_plays_mom_and_new_bark_gate
  cargo test -p geothite mcp_drives_real_movement_and_production_start_menu
  cargo test -p geothite --test stdio_clean
  ```

  The controller regression must prove all of the following: the game starts
  as the requested trainer in `PlayersHouse2F`; Start exposes the production
  menu; the bedroom/ground-floor warp fires; Mom intercepts the player and A
  changes the visible dialogue page; movement is rejected while dialogue owns
  input; leaving the house enters New Bark; walking to the west exit shows
  `Wait, CHRIS!`; and completing that scene returns the player to `(5, 8)` in
  `NewBarkTown`. The MCP regression must prove its `move` tool changes the same
  authoritative position and its `press start`/`observe` path exposes the
  production Start menu. The battle regression must use the external pack,
  retain the core battle phase, expose the production FIGHT/PKMN/PACK/RUN and
  move menus, execute the selected move through the authoritative battle
  engine, observe its PP mutation, and expose the resulting battle dialogue.
  A battle HUD or input log without those state changes is not a passing test.
  Any failure is a gameplay blocker, not a renderer polish issue. The stdio
  regression walks into Mom's dialogue in a spawned client and requires stderr
  to remain byte-for-byte empty, preventing trace lines from corrupting the
  fullscreen terminal.

- Build the `geothite` executable and launch the interactive terminal UI from
  this checkout with:

  ```sh
  cargo build --release --locked -p geothite
  ./target/release/geothite play content-packs/text-tui.crystalpack
  ```

- Run the same local game session as a stdio MCP server with:

  ```sh
  ./target/release/geothite mcp content-packs/text-tui.crystalpack
  ```
