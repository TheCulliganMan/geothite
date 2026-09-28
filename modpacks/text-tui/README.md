# Text TUI

The Text TUI is a native Rust renderer for Geothite. It presents the same typed
runtime state as a compact plain-text snapshot or a responsive fullscreen
terminal UI. The renderer reads the user-supplied compiled content pack at run
time; this modpack contains no game data, exported command catalog, ROM, audio,
or generated pixels.

## Run it

```sh
cargo run --release --locked -p crystal-assets --bin pack_text_tui -- . \
  content-packs/realtime-clock.browser.crystalpack \
  content-packs/text-tui.crystalpack

cargo build --release --locked -p geothite
./target/release/geothite play content-packs/text-tui.crystalpack --name CHRIS
```

See the repository `AGENTS.md` crystalpack runbook for the exact Rust build
command and required external export-workspace inputs.

Resume a save and enable F5 quick-saving:

```sh
cargo run --locked -p geothite -- play /path/to/game.crystalpack \
  --load /path/to/game.crystalsave --save /path/to/game.crystalsave
```

Generate a stable agent-, log-, and screen-reader-friendly snapshot without
opening a terminal:

```sh
cargo run --locked -p geothite -- dump /path/to/game.crystalpack
```

Run the local runtime as a stdio MCP server:

```sh
./target/release/geothite mcp content-packs/text-tui.crystalpack --name CHRIS
```

Controls match the established TypeScript CLI: arrows, WASD, or HKL move;
`Z`/`J`/Space is A; `X`/`K`/`B`/Escape is B; Enter is Start; Tab is Select;
`.` waits eight frames; `R` refreshes; `?` opens help; F5 saves; and `:q!`
then Enter quits without saving.

## Renderer features

- A renderer-neutral, serializable `TextSnapshot` shared by plain text and TUI.
- Centered collision-aware maps with distinct player-facing, NPC, warp, sign,
  wall, grass, and water glyphs.
- Battle cards with HP bars, status, active-party context, and danger colors.
- Dialogue, vertical menus, yes/no prompts, selected-line semantics, bounded
  action history, and context-sensitive input hints.
- Responsive wide/stacked layouts, an alternate-screen lifecycle guard,
  Unicode-safe wrapping, compact output, and a JSON theme.
- Direct Rust input through `crystal_bevy::VisibleShellController`, the same
  production controller that owns the graphical client's story events, warps,
  trainer sight, battles, text, prompts, and rich menu workflows.

The text renderer projects controller-owned screens into terminal-friendly
dialogue and menu rows; it does not implement parallel game rules. Both keyboard
and MCP inputs call the same production controller. The terminal currently
accepts the trainer name through `--name` and does not yet own an audio output
device, so those presentation adapters are not claimed as graphical-client
parity.

## Required gameplay regression

`dump` is only a rendering smoke test. With the external TUI pack built, run:

```sh
cargo test -p crystal-bevy \
  renderer_neutral_controller_plays_mom_and_new_bark_gate
cargo test -p geothite mcp_drives_real_movement_and_production_start_menu
cargo test -p geothite --test stdio_clean
```

These tests verify actual movement, the house warp, Mom's blocking introduction,
dialogue input ownership and page progression, the New Bark west-exit block and
return movement, and the production Start menu through the MCP dispatcher.
The process-level stderr regression also walks into Mom's dialogue and rejects
any debug output that would overwrite Ratatui's alternate screen. Optional
dialogue tracing is available only when `CRYSTAL_DIALOGUE_TRACE` is set.
