# Geothite

A Rust implementation of Pokémon Crystal with browser and desktop clients,
multiplayer, and an optional 2.5D overworld renderer. Game logic, rendering,
audio, and networking are implemented in this repository.

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

Or run in a Linux x86_64 build environment:

```sh
bash tools/build-crystalpack.sh
```

The script assembles the pinned pret source outside the checkout, exports game
content with Rust, verifies audio and gameplay, and publishes ignored native,
browser, and TUI packs under `content-packs/`. Concurrent agents use these packs
and separate Cargo target directories; see the [build runbook](docs/game-content.md#share-packs-and-isolate-builds).

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
See the [deployment guide](docs/deployment.md) for configuration, reverse
proxying, upgrades, health checks, persistence, and cleanup.

## Playing

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

For native play, install the Rust toolchain specified in `rust-toolchain.toml`
and your platform's Bevy build dependencies. Launch with the generated pack:

```sh
cargo run --release --locked -p crystal-bevy --features voxel-view -- \
  --pack content-packs/core-modular.crystalpack \
  --save-path content-packs/player.crystalsave
```

Add `--load-save content-packs/player.crystalsave` to load that save.
Native controls use `Enter` for Start, alongside the arrow keys, `Z`, `X`,
and Right Shift. Hold B (`X`) with a direction to run at twice walking speed;
[running shoes](modpacks/running-shoes/README.md) are enabled by default.

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
| Rotate optional 2.5D camera | LB / RB |
| Zoom optional 2.5D camera out / in | LT / RT |

Sticks use a deadzone and stable cardinal directions. Disconnecting a controller
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
- [Deployment and maintenance](docs/deployment.md)
- [Game fidelity audit](FIDELITY_AUDIT.md)
- [Renderer inspection](RENDER_AT_LOCATION.md)
- [Operation profiling](tools/OPERATION_PROFILING.md)

## Alternative frontends

The renderer-independent `crystal-runtime` crate owns game sessions, commands,
snapshots, saves and audio cue resolution. `crystal-audio::pcm` decodes the same
packed sounds for custom audio devices. See [the frontend assessment and integration
guide](docs/FRONTEND_ARCHITECTURE.md) for the headless example and remaining UI
controller work.
