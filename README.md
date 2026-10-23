# Geothite

A Rust implementation of Pokémon Crystal with browser and desktop clients,
multiplayer, and an optional 2.5D overworld renderer. Game logic, rendering,
audio, and networking are implemented in this repository.

The default content builds enable [All Pokémon, solo](modpacks/all-pokemon/README.md):
all 251 Pokémon have renewable wild encounters, and Celadon Department Store 4F
sells a Link Cable for solo trade evolutions alongside all six evolution stones.

The source exporter also includes Battle Tower's dynamically selected trainer
dialogue. See the verified [source-text repair runbook](docs/game-content.md#targeted-source-text-repair-on-2026-10-17)
for refreshing a prepared external workspace and preserving the default mods.

## Features

- Classic 2D presentation with a switchable 2.5D overworld view.
- Keyboard, touch, and gamepad controls with responsive browser layouts.
- Shared-world multiplayer, player chat, battle invitations, and trading.
- Browser saves tied to a persistent player identity.
- A shared server clock for the hosted game's day and night cycle.
- A Docker build that packages the server and browser client together.

This is an actively developed project. See the [fidelity audit](FIDELITY_AUDIT.md)
for implementation status and remaining differences from the original game.

## Setup

### Native desktop app

With the local content pack prepared, the same commands select your current OS:

```sh
just desktop             # Build, validate and install
just desktop-run         # Open the installed app immediately
just desktop-package     # Build a distributable in .local/desktop-dist/
just desktop-repack      # Package the last build without compiling
```

On macOS, `just desktop` installs **Geothite.app** in `~/Applications` with a
crystal icon, the production title/new-game/Continue flow, the supplied native
pack and 2.5D artwork. `just desktop-package` creates an architecture-specific
DMG with an Applications shortcut and SHA-256 checksum. The app runs independently
of this checkout; F3 switches between 2D and 3D. Saves live in
`~/Library/Application Support/Geothite/saves`, so replacing the app preserves them.
The desktop app plays music and sound effects by default. Normal web play enables
sound on the first click, touch or keypress, subject to browser autoplay rules;
the sound button remembers deliberate mute choices. Native developer clients,
browser location previews (`?preview=...`), embedded training views and
`?development=1` sessions always stay muted.

On Mac, **Settings** appears in the window title bar and the Mac menu bar.
It controls master volume, mute, 2D/3D view, a checkable First Person toggle,
camera presets and fullscreen. Orbit rotation follows the character, including
inside rooms where the classic viewport stops scrolling.
Settings follow keyboard/controller changes and persist in
`~/Library/Application Support/Geothite/settings.json`. Camera reset and
Reset Settings are available in the same menu.

On Windows, installation goes to `%USERPROFILE%\Games\Geothite` and creates a
Desktop shortcut; packaging creates a ZIP. Linux installation goes to
`~/.local/opt/geothite` with an application-menu entry; packaging creates a native
`tar.gz`. Build on each target OS. Windows and Linux packaging are prepared;
the macOS path is the focus of local verification. These are native host builds,
not cross-compilation or a universal Mac binary.

Install Rust 1.99 (selected by `rust-toolchain.toml`), `just`, and Git LFS.
macOS also needs Xcode command line tools and Python 3.9+; Windows needs the Visual
Studio C++ tools/Windows SDK; Linux needs the libraries in
[the native runbook](docs/cold-start-3d.md). On a fresh checkout, run
`git lfs pull` and `just desktop-content` once. Preparing source content and a cold
Bevy build can take substantially longer than two minutes. Desktop commands use
fully optimized release builds by default and share a persistent build cache.
`desktop-repack`, `desktop-install` and `desktop-run` skip compilation.

Earlier Apple Silicon checks using the `wasm-dev` development profile produced
a DMG in **2m 13s** and installed it in **52s** with a warm cache. The initial
build took substantially longer. That installed app was opened through Launch
Services and reached the interactive New Game flow.
`just desktop-run` captures its launch log in `~/Library/Logs/Geothite/game.log`.

`just desktop-install` installs the last build without compiling, and
`just desktop-check` validates the last build's content/art without opening a
window. Pass a different local pack as a recipe argument, for example
`just desktop-package content-packs/all-pokemon.crystalpack`.
Use `GEOTHITE_DESKTOP_PROFILE=wasm-dev` for shorter development builds with
lighter game-code optimization. Keep the same profile when building and installing.
Close the installed app before updating it. All build products, copied packs,
and distributables remain ignored; they are never source deliverables to commit.

Mac packages use ad-hoc signing for local use by default. For public distribution,
configure `GEOTHITE_SIGN_IDENTITY` with a Developer ID Application certificate
and `GEOTHITE_NOTARY_PROFILE` with an existing `notarytool` keychain profile, then
run `just desktop-release`. This signs with hardened runtime, submits the DMG to
Apple, and staples/validates the notarization ticket. It requires your Apple
Developer credentials; the local package is not notarized. See
[Apple's distribution guide](https://developer.apple.com/documentation/xcode/packaging-mac-software-for-distribution).

Generate local game content with the repository's Rust builders, then use Docker
with Compose for your own server. [Game content setup](docs/game-content.md)
contains the required tools, pinned source inputs, validation, and play commands.

### 1. Clone and initialize game content

```sh
git clone https://github.com/TheCulliganMan/geothite.git
cd geothite
```

With Git LFS and Rust installed, initialize from Windows using WSL Ubuntu-24.04:

```powershell
.\tools\build-crystalpack.ps1
```

With `just` installed, the same entry point selects the platform initializer:

```sh
just crystalpack
```

On macOS, source assembly also requires Homebrew Bison, pkgconf, and libpng.
RGBDS is built from its pinned source commit in the external workspace. The
direct Bash entry point works on macOS and Linux x86_64:

```sh
bash tools/build-crystalpack.sh
```

The script assembles the pinned pret source outside the checkout, exports game
content with Rust, verifies audio and gameplay, and publishes ignored native,
browser, and TUI packs under `content-packs/`. Concurrent agents use these packs
and separate Cargo target directories; see the [build runbook](docs/game-content.md#share-packs-and-isolate-builds).
Publication also checks an earned Lance victory through the Hall of Fame,
credits, title return and the production Continue transaction to New Bark.

### 2. Create your server secret

Create a signing secret once for a new installation:

```sh
umask 077
printf 'CRYSTAL_AUTH_SECRET=%s\n' "$(openssl rand -hex 32)" > .env
```

### 3. Start the server

```sh
docker compose -f docker-compose.production.yml up -d --build
```

Open [localhost:3003](http://localhost:3003). The first build compiles Rust and
WebAssembly dependencies and can take several minutes. Docker includes the
required toolchains and copies your locally supplied game pack into the image;
a separate Rust installation is unnecessary for the Docker build.

Keep `.env` across updates. For remote access, put the server behind HTTPS.
The web **2D / 3D / TUI** chooser swaps renderers within the same game session.
Use `/?view=tui` or `/tui/` to open the original painted terminal surface;
`/?view=2d` and `/?view=3d` select the graphical views. Switching keeps the
current trainer, position, dialogue, battle, menu, and save state.

See the [deployment guide](docs/deployment.md) for configuration, reverse
proxying, upgrades, health checks, persistence, and cleanup.

## Playing

For the prepared browser copy in this checkout, run `just serve` and open
[localhost:3003](http://localhost:3003/?multiplayer=off). Use `just serve 3004`
to select another port; Ctrl-C stops the server. This quick command reuses the
ignored `.local/desktop-3d-runtime` server and browser bundle without rebuilding.
See [local browser serving](docs/game-content.md#serve-a-prepared-browser-copy)
for input requirements and rebuilding from source.

`just play-3d` launches the prepared Bevy desktop client directly in the
bedroom with 3D enabled. For the same entry in the browser, open
[the 3D bedroom](http://localhost:3003/?preview=bedroom&multiplayer=off).
The prepared browser build selects an available WebGPU adapter and falls back
to WebGL when one is unavailable. `just trace-3d` records the native loading
waterfall in `.local/loading-trace.json`; browser tracing uses
`&loading-trace=on`. See [loading diagnostics](docs/cold-start-3d.md#loading-waterfall-diagnostics).


| Action | Keyboard |
| --- | --- |
| Move | Arrow keys or WASD |
| Confirm / interact (A) | `Z` |
| Back / cancel (B) | `X` |
| Game menu (Start) | `Space` |
| Select | `Backspace` |
| Open chat / send message | `Enter` |
| Close chat | `Esc` |

Rebind Chat, Start, and Select in **Personalization → Keyboard bindings**;
**Save key bindings** keeps the choice in this browser across reloads. Chat, Start, and
Select require different keys. Enter still sends messages while typing in chat.
Nearby Say messages appear briefly above the speaking trainer in 2D and 2.5D.
Whispers and global channels stay in the chat panel.
Phones expose touch controls; standard gamepads are supported.
Click or press a key in the game to enable browser audio.

Use the **2.5D** control to switch the overworld renderer. Battles and menus
keep their 2D presentation. Camera zoom and rotation are available in 2.5D.

Browser identities are created automatically. Save through the game's normal
menu; saves remain in that browser. Clearing browser storage removes the local
identity and saves.

Multiplayer ratings, the user directory, and leaderboard totals persist in a named Docker volume across container rebuilds
and restarts. Keep the signing secret in `.env` unchanged so returning players
can reuse their identities. Active connections, chat history, and in-progress
matches are temporary; players reconnect after a server restart. See
[persistence and cleanup](docs/deployment.md#persistence-and-cleanup).

### Multiplayer

Face a nearby player and press A, or click their name in chat, to choose
Battle, Trade, or Whisper. Invitations can be accepted or declined in the game.
Players must use compatible game packs to share a world.

Chat supports nearby Say, map-wide General, Trade, Looking for Group,
private whispers, and custom channels. Type `/help` for available commands.

The compact community window has **Chat**, **Social**, and **Leaderboard** tabs.
Social lists every trainer known to this server, including offline users, with
search and live connection status. Click an online name to whisper; battle and
trade invitations require the trainer to be nearby.

Leaderboards rank PvP wins and completed battles, PvE wins and completed battles,
completed player trades, and highest combined level of the six non-egg party
members. Ties share a rank. PvP results and trades require confirmation from both
participants; cancelled sessions do not count. PvE follows the game's encounter
results (including captures as wins), excludes tutorials and link battles, and
persists in normal saves. Save restores cannot reduce or double-count a trainer's
recorded totals. Tracking and the user directory begin as players use this update;
historical battles and trades were not stored.

### Desktop play

Build the native client with the 3D overworld renderer enabled:

```sh
git lfs install --local
git lfs pull
cargo build --release --locked -p crystal-bevy --features voxel-view
```

The executable is `target/release/crystal-bevy` (`crystal-bevy.exe` on Windows).
Press `F3` during play to switch between 2D and 3D. Game content is compiled
with the Rust `pack_core` command in [Game content setup](docs/game-content.md).

The 3D renderer includes material-specific highlights, sky reflections, filtered
detail textures and camera-fitted shadows. Native hardware defaults to the high
quality preset with SMAA Ultra, supported SSAO and HDR; browsers use balanced
quality. Character paintings use filtered mipmaps, and authored 3D textures
use anisotropic filtering for clearer oblique views and less texture shimmer.
See [render quality settings](docs/render-quality.md) for presets and fallbacks.

In the 3D overworld, the first Pokémon in the party follows the trainer's
trail. Reordering the party changes the companion, and it stays with the
trainer across map transitions. Surf shows the trainer seated on the Pokémon
that used the move. That Pokémon remains the mount when the party is reordered;
when it is the lead, it accompanies the trainer by carrying them.
The rider follows the Pokémon's animated back through turns and swimming bob,
with a seated pose fitted to its body width.
This is the sole 3D Surf presentation: the Pokémon recorded by the field move
carries the trainer. Pokémon retain their pack's Pokédex dimensions in every
overworld role; mounting never enlarges or shrinks them to fit the rider.

In 3D, stop on a chair, bench, or stool, or face an empty seat directly ahead.
After a short pause, the trainer sits automatically. Moving away makes them stand.
Human NPCs placed on recognized seats use the same seated pose. The shared animation
blends the hips, legs, hands, and body position smoothly, including interrupted transitions.
This animation preserves the game’s movement and collision rules.

For native play, install the Rust toolchain specified in `rust-toolchain.toml`
and your platform's Bevy build dependencies. Launch with the generated pack:

```sh
cargo run --release --locked -p crystal-bevy --features voxel-view -- \
  --pack content-packs/core-modular.crystalpack \
  --save-path content-packs/player.crystalsave
```

Add `--load-save content-packs/player.crystalsave` to load that save.
3D uses natural movement by default for every input device: walk diagonally,
stop between tiles, and use partial left-stick tilt for slower movement.
Arrow keys, WASD, the D-pad, and the left stick follow the camera, including
while it rotates and in first person. Combine two directions for diagonals
at the same walking speed. Collision, encounters, doors, and
scripts still use the shared production controller. 2D and the text client
retain their normal tile controls. See [natural movement](modpacks/natural-movement/README.md)
for the optional pack builder and opt-out setting.

Native controls use `Enter` for Start, alongside the arrow keys, `Z`, `X`,
and Right Shift. Hold B (`X`) with a direction to run at twice walking speed;
[running shoes](modpacks/running-shoes/README.md) are enabled by default.

The [competitive battle mod](modpacks/competitive-battles/README.md) adds a
physical/special category for each move, a move inspection board, and a
trainer AI that plans attacks, recovery, and switches through the Rust battle
engine. Six trainer-class profiles share the same search strength, with distinct
styles for pressure, setup, disruption, endurance, switching, and balanced play.
The pack initializer and hosted pack preparation enable it by default.
Its builder can also create a separate ignored pack from existing local content.

Standard packs also enable [Wild Challenge](modpacks/wild-challenge/README.md):
wild Pokémon scale with your team, receive legal complementary movesets, and
use the competitive planner. Native, browser, and terminal/MCP builds share
these defaults. Existing external content can be refreshed with the Rust
`pack_default_battles` builder documented in the content guide. These rules
change pack identity; use a new save for a newly upgraded pack.

The refreshed local browser game is available at
<http://localhost:3006/?multiplayer=off>. Its prepared runtime can be restarted
with `just serve 3006 .local/default-battles-runtime`.

Native Bevy enables gamepads and responsive window sizing by default, starting
with an initial window up to 1280×800, fitted to the display’s DPI and size. Steam Deck and other handhelds can use their standard gamepad or
Steam Input's gamepad output; keyboard mappings remain available alongside it.

| Native controller action | Standard / Xbox button |
| --- | --- |
| Move | D-pad or left stick |
| Confirm / interact | A (south) |
| Cancel / run while moving | B (east) |
| Game menu / Select | Start / Back |
| Toggle borderless fullscreen | X (west), or keyboard `F11` |
| Toggle optional 2.5D view | Y (north) |
| Steer 3D camera | Right stick (yaw and pitch), or hold either mouse button and drag |
| Cycle close orbit / wide orbit / first person | Select / Back, or keyboard right Shift |
| Rotate 3D camera continuously | Hold LB / RB, or keyboard `Q` / `E` |
| Zoom optional 2.5D camera out / in | LT / RT |

Drag the game to rotate and tilt: use either mouse button on desktop, or the
left button in the browser. Two-finger trackpad scrolling rotates horizontally
and zooms vertically; pinch zooms. Gamepad Select / Back cycles **close orbit
(150%) → wide orbit (75%) → first person**, aligning first person with the
trainer's facing direction. The right stick looks around in first person.
Touch dragging rotates horizontally and zooms vertically in orbit view, and
looks vertically in first person; two fingers pinch to zoom.

In the 3D overworld, forward follows where the camera looks and left/right
strafe across the view with keyboard, D-pad, or stick. The first-person preset switches to the trainer’s eye level and hides
the local player model. Menus and 2D retain the original Select action.

Sticks use a deadzone; menus and tile movement keep stable cardinal selection. Disconnecting a controller
releases its inputs; a docked controller can take over from an idle built-in pad.
After focus loss or a native overlay, release the controls before resuming.
Linux source builds also need the development package for `libudev` (for example,
`libudev-dev` on Debian/Ubuntu), used by the native gamepad backend.

Wide screens dock field dialogue and panels in a right-hand column. Portrait
screens place dialogue at the bottom center. Both 2D and 2.5D views use this
responsive layout, with smaller insets to keep panels on small screens.

### Browser agent play

In browsers that implement the current WebMCP `document.modelContext` API,
the game registers tools to observe the current presentation, read player and
party status, inspect visible surroundings and input flow, press original Game
Boy buttons, and initiate the existing multiplayer interactions. Open **How to
play → Agent tools** to see whether registration succeeded.

The WebMCP surface follows the same information boundary as a human player. It
returns on-screen text and menus, inspectable trainer/party state, and relative
positions of people visible in the viewport. It does not expose walkthroughs,
objectives, event or engine flags, hidden objects, absolute world coordinates,
map dimensions, collision data, or pathfinding. Every game action still goes
through the normal input system, and a human keypress cancels agent input.

## Development

The game uses Bevy 0.20 and Rust 1.99.0, pinned in `rust-toolchain.toml`.
Shared dependency versions and local crate paths live in the workspace
`Cargo.toml`; each renderer selects its own Bevy features.

For optional independent PC and Mailbox LCD comparisons, see the
[menu reference capture guide](docs/menu-oracle.md). Its emulator environment
and generated references remain external to the repository and game bundle.

Run the graphical client with the optional Bevy development tools and a local pack:

```sh
cargo bevy-dev -- \
  --pack content-packs/realtime-clock.browser.crystalpack
```

`F8` shows or hides FPS diagnostics, `F9` toggles the frame-time graph within
that overlay on native builds, and `F10` toggles UI layout outlines.
The browser supports the FPS counter; its WebGL2 renderer cannot display the graph.
Bevy's 3D render-buffer overlay
uses `F1` to cycle modes and `F2` to cycle opacity. Overlays start hidden.
The feature also enables Bevy's development assertions. Compiled game packs
require a restart after rebuilding. These tools are
opt-in for the graphical client and work with both renderer modes. Normal
desktop, browser, and terminal builds do not enable them. Use `cargo bevy-check`
to check the graphical client and examples, or `cargo bevy-test` to run its tests.
Existing `F3` view,
`F6` capture, and `F11` fullscreen controls remain available.

Native state and schedule diagrams are included in `dev-tools`. Enable file
export explicitly when launching the actual graphical client:

```sh
CRYSTAL_BEVY_DIAGRAMS=target/bevy-diagrams cargo bevy-dev -- \
  --pack content-packs/realtime-clock.browser.crystalpack
```

Open `target/bevy-diagrams/main-index.md` or `render-index.md` for schedule
diagrams with Mermaid previews. Each schedule also has Graphviz ordering and
component/resource-access DOT files; the JSON contains query filters,
set run conditions, conflicts, and dependencies inserted by schedule build
passes. Solid arrows represent
ordering, dashed arrows represent set membership; access graphs label reads,
writes, and conflicts. The currently executing collector schedules (`Main`
and `ExtractSchedule`) are excluded. Render diagrams refresh as Bevy builds
additional schedules, including per-view schedules; `F7` refreshes the main
schedule export. No Graphviz installation is required for Mermaid previews.

`observed-state-flow.md` and `.mmd` record the controller's visible phases and
menus as you play. They show only transitions observed during that session,
not every possible game transition. The controller remains authoritative;
these tools read its presentation cache and never dispatch input. File export
is native-only and disabled unless `CRYSTAL_BEVY_DIAGRAMS` is set. All output
under `target/` is ignored and should be removed after use.

The workspace is organized around focused Rust crates:

| Path | Responsibility |
| --- | --- |
| `crates/crystal-core` | Game state, timing, battles, and world rules. |
| `crates/crystal-assets` | Compiled game data and content packs. |
| `crates/crystal-audio` | Native Rust synthesis, source linking, and audio auditing. |
| `crates/crystal-net` | Multiplayer protocol and transport. |
| `crates/crystal-bevy` | Desktop and WASM game client. |
| `crates/crystal-render-api` | Shared presentation snapshots. |
| `crates/crystal-tui` | Native text snapshots and terminal UI. |
| `crates/crystal-voxel-view` | Optional 2.5D renderer. |
| `crates/crystal-web-server` | HTTP hosting and multiplayer relay. |
| `web-client` | Browser page, controls, thin WASM audio worker, and chat UI. |

Check the server and run the browser session tests (the latter require Node.js):

```sh
cargo check --locked -p crystal-web-server
npm ci
npm run test:browser
```

The production Dockerfile builds the server and WASM client from the same
revision. Keep protocol changes coordinated across both. Generated game packs
should be regenerated using the canonical exporter, not edited manually.
Audio programs come from your separately supplied pack; the Rust exporter
is in this repository. See [Native Crystal audio](audio/README.md) for export and regression checks.
No TypeScript or JavaScript build tool is required to synthesize or export audio.

## Contributing

Bug reports should include the revision, platform or browser, reproduction
steps, and any relevant logs. For gameplay differences, describe the expected
Pokémon Crystal behavior and what happened instead.

Keep changes focused and add regression coverage for behavior fixes. Include
screenshots for visual changes and the checks you ran in pull requests.

## Further reading

- [Meshtastic battle/trade profile](modpacks/meshtastic/README.md)
- [Flygon modpack](modpacks/flygon/README.md)
- [Native Rust text renderer and TUI](modpacks/text-tui/README.md)
- [Wild Challenge: player-relative levels and competitive wild movesets](modpacks/wild-challenge/README.md)
- [Trainer rematches with level and moveset scaling](modpacks/trainer-rematches/README.md)
- [Deployment and maintenance](docs/deployment.md)
- [Game fidelity audit](FIDELITY_AUDIT.md)
- [Renderer inspection](RENDER_AT_LOCATION.md)
- [Repeatable CPU/GPU gameplay audit](tools/PERFORMANCE_AUDIT.md)
- [Operation profiling](tools/OPERATION_PROFILING.md)

## Alternative frontends

The renderer-independent `crystal-runtime` crate owns game sessions, commands,
snapshots, saves and audio cue resolution. `crystal-audio::pcm` decodes the same
packed sounds for custom audio devices. See [the frontend assessment and integration
guide](docs/FRONTEND_ARCHITECTURE.md) for the headless example and remaining UI
controller work.
