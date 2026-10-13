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

### Pack-build requests are implementation work

- When the user asks to build a crystalpack or make the game playable, do the
  build work. A missing local `.crystalpack` is not, by itself, a reason to
  stop and ask the user to supply one.
- Read `docs/game-content.md`, `docs/cold-start-3d.md`, and the Rust
  `crystal-assets` builders. If the user references a URL or another Git
  revision, inspect that exact document/revision before drawing conclusions
  from this checkout's documentation.
- Trace and execute the documented Rust pack-build path. Investigate missing
  source inputs, generated manifests, schema mismatches, and exporter steps;
  repair or implement the required Rust build steps within the authorized
  task. Keep pret checkouts and generated game content outside tracked source.
- Do not infer that pack-generation code is absent merely because the pack,
  an intermediate JSON file, or a prepared export workspace is absent. Any
  blocker report must identify the actual attempted command, its concrete
  failure, and the code path responsible. Do not add a README warning claiming
  there is no build pipeline in place of doing this investigation.
- A client executable, `--help`, a ROM, a render dump, or a running compiler
  is not a generated pack or a playable game. Validate the actual resulting
  pack, run the gameplay regressions below, and launch the production client
  before reporting the user's playable-game request complete.
- Document the verified build/run commands and required inputs in the README
  and content guide. Never describe an unexecuted or failed build as working.

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
- Initialize fresh game content with `tools/build-crystalpack.ps1` on Windows
  (WSL Ubuntu-24.04) or `bash tools/build-crystalpack.sh` on Linux x86_64.
  Install the build tools listed in `docs/game-content.md` first. The script
  initializes Git LFS, assembles the source/toolchain pinned in
  `pret-source.lock.json`, runs the Rust exporters and compiler, verifies all
  audio PCM hashes/frame counts/loop ranges, and runs the gameplay regressions
  below before publishing packs. The external workspace defaults to
  `$HOME/.local/share/geothite/export-workspace` in the Linux environment.
- Use the completed shared packs in this checkout:
  `content-packs/core-modular.crystalpack` for native play,
  `content-packs/core-modular.browser.crystalpack` for browsers, and
  `content-packs/text-tui.crystalpack` for the text client/MCP. Packs remain
  ignored and must never be staged or committed.
- One agent owns source export and pack publication. Other agents consume the
  completed shared pack; do not run competing exporters against the shared
  external workspace. Give every concurrent agent a distinct
  `CARGO_TARGET_DIR`, for example `target/agent-title`, and set
  `CRYSTAL_RENDER_TEST_PACK` to the absolute completed TUI pack path when testing.
  Do not delete another agent's build cache to bypass a lock.
- The initializer uses its own persistent Cargo cache. Set
  `CRYSTAL_PACK_BUILD_DIR` to choose its Linux cache directory and
  `CRYSTAL_PACK_BUILD_PROFILE` to `release` (default) or `wasm-dev`.
  Windows forwards both settings into WSL.
- Visually inspect production work against 2D at the same position, time,
  facing, and game state. Critique composition, scale, materials, lighting,
  readability, and character/scenery fidelity; fix defects and inspect again
  before claiming visual acceptance.
- Validate a pack and the text renderer without entering fullscreen mode:

  ```sh
  cargo run --locked -p geothite -- dump \
    content-packs/text-tui.crystalpack
  ```

  This is a render smoke check only. Before calling the client playable, run
  both controller and stdio-MCP regressions with the freshly built external
  pack present:

  ```sh
  export CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack"
    cargo test --locked -p crystal-bevy \
    renderer_neutral_controller_exposes_battle_commands_and_executes_a_turn
  cargo test --locked -p crystal-bevy \
    renderer_neutral_controller_plays_mom_and_new_bark_gate
  cargo test --locked -p geothite mcp_drives_real_movement_and_production_start_menu
  cargo test --locked -p geothite --test stdio_clean
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
