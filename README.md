# PokeCrystal Rust Port

This workspace is the Rust game port. It intentionally excludes the existing
MCP, web UI, CLI agent tooling, and TypeScript app surfaces.

## Crates

- `crystal-core`: deterministic game state, timing, input, battle/world rules,
  save state, and multiplayer-ready simulation boundaries.
- `crystal-assets`: loaders for ASM-derived data exported from
  `vendor/pokecrystal` and the existing TypeScript asset pipeline.
- `crystal-audio`: music, SFX, cries, and audio command playback data.
- `crystal-net`: transport-neutral multiplayer protocol types.
- `crystal-render-api`: read-only presentation snapshots shared with optional
  Bevy render mods.
- `crystal-bevy`: desktop game shell for rendering, input, and audio.
- `crystal-voxel-view`: optional clean-room 2.5D overworld renderer.

The port should move file by file from the game runtime surfaces under
`packages/core/src`, `packages/assets/src`, `packages/exporters/src`, and
`vendor/pokecrystal`. Do not port MCP, web routes, agent workflows, or desktop
packaging.

## Play

The playable target is the Bevy shell. It reads one definitive compiled pack
and enters the title screen by default:

```sh
cargo run -p crystal-bevy -- \
  --pack /path/to/core-modular.crystalpack \
  --save-path /tmp/pokecrystal.crystalsave
```

Existing saves can be loaded directly:

```sh
cargo run -p crystal-bevy -- \
  --pack /path/to/core-modular.crystalpack \
  --load-save /tmp/pokecrystal.crystalsave \
  --save-path /tmp/pokecrystal.crystalsave
```

### Native multiplayer

Native builds can host or join an exact pack-bound multiplayer session. Each
player must use the same compiled pack and session id, a distinct nonzero
player id, and a valid display name. Start the host first:

```sh
cargo run -p crystal-bevy -- \
  --pack /path/to/core-modular.crystalpack \
  --load-save /path/to/host.crystalsave \
  --save-path /path/to/host.crystalsave \
  --multiplayer-host 0.0.0.0:3737 \
  --multiplayer-session friends \
  --multiplayer-player-id 1 \
  --multiplayer-player-name CHRIS
```

Then join from the other game process, using the host's reachable address:

```sh
cargo run -p crystal-bevy -- \
  --pack /path/to/core-modular.crystalpack \
  --load-save /path/to/peer.crystalsave \
  --save-path /path/to/peer.crystalsave \
  --multiplayer-join 192.0.2.10:3737 \
  --multiplayer-session friends \
  --multiplayer-player-id 2 \
  --multiplayer-player-name KRIS
```

The connection fails closed when protocol, session, modpack, or compiled-pack
identity differs. Host acceptance and all socket reads are nonblocking, so
waiting for or communicating with a peer does not stall the game loop.

When `--pack` is omitted, `core-modular.crystalpack` must be beside the
executable. When `--save-path` is omitted, saves are written under `saves/`
beside the pack. The release executable exposes no spawn, smoke, script,
inventory, battle, or other state-mutation command line.

Keyboard controls are arrows for the D-pad, `Z` for A, `X` for B, `Enter` for
Start, and Right Shift for Select.

### Browser build

Build the WASM game with its switchable 2D / 2.5D renderer, generate its JavaScript bindings, and serve
`rust/web-dist` with the Rust server:

```sh
npm run build:browser-audio-runtime
cd rust
rustup target add wasm32-unknown-unknown
cargo build -p crystal-bevy --features fullscreen-scaling,voxel-view --target wasm32-unknown-unknown --profile web-release
wasm-bindgen --target web --out-dir web-dist --out-name crystal-bevy \
  target/wasm32-unknown-unknown/web-release/crystal-bevy.wasm
cp web-client/index.html web-client/mobile-player.css web-client/touch-controls.js web-client/gamepad-controls.js web-client/view-toggle.js web-client/browser-session.js web-client/server-clock.js web-client/social-chat.js web-client/social-chat.css web-client/webmcp.js web-dist/
cp -R web-client/audio-runtime web-dist/
cp ../content-packs/core-modular.browser.crystalpack web-dist/
gzip -9 -k web-dist/crystal-bevy_bg.wasm
gzip -9 -k web-dist/core-modular.browser.crystalpack
cargo run -p crystal-web-server --bin crystal-web-server -- --dir web-dist --port 8080
```

To check that the browser event loop survives startup, copy
`web-client/browser-startup.test.html` into `web-dist` and open
`http://localhost:8080/browser-startup.test.html`. The test must report `PASS`
after two live game-loop observations. Also verify the visible title screen
and that Start opens the title menu.

Restart the Rust server after updating its source, even if `web-dist` already
contains the new client. A stale server can serve the page while returning 404
for `/v1/clock`, causing “Server clock is unavailable” at startup. If a background
service manages the server, update its executable path before restarting it.
In another terminal, verify readiness with
`curl --fail http://127.0.0.1:8080/v1/clock`;
the response must contain `unixMillis` and `"timeZone":"UTC"`.

On phones, use the sliding D-pad and A/B, Start and Select touch buttons.
The canvas stays clear of the touch controls in portrait and landscape. Standard
browser gamepads use the D-pad or left stick, bottom/right face buttons, and
Start/Back. On Steam Deck, select a Gamepad Steam Input layout. Controller or
keyboard play hides the touch controls; touching the screen restores them.
Settings and chat stay collapsed until opened.

The hosted build includes the `realtime-clock` modpack. All players use the
server's UTC clock from `GET /v1/clock`, including the real weekday and Crystal's
morning/day/night boundaries. The browser synchronizes before startup, every
30 seconds, and when returning to the tab; it advances samples using
`performance.now()`, never the device wall clock or time zone. After two minutes
without synchronization gameplay pauses until the server is reachable again.
Manual clock settings cannot offset server time. The server builds this modpack
on top of its encounter modpack from the verified browser pack automatically. Its distinct modpack identity
uses a separate save slot; existing base-pack saves are retained.
To produce the clock modpack independently, run
`cargo run -p crystal-assets --bin pack_realtime_clock -- .. <base-pack> <output-pack>`.
It requires the hosted browser clock when played.

Open `http://127.0.0.1:8080` to launch the game in the classic 2D renderer.
The original 640×576 LCD surfaces scale as one complete unit so text and tiles
stay readable at any aspect ratio. Click or press a key in the game before
expecting sound because browsers require a user gesture to unlock audio
playback. The browser pack contains compact MIDI files using the
`pokecrystal-midi-v1` sequencer profile. The bundled TypeScript synthesizer
compiles MIDI to canonical PCM in memory only when a sound is first requested,
verifies it against the pack manifest, and sends it to WebAudio. Browser saves
persist in local storage under a slot scoped to the
stable multiplayer player ID. Reloading or restarting the browser restores
that same slot and exposes Continue once the save passes normal modpack
validation.

The production image contains no PCM audio catalog or audio volume. The server
serves precompressed `.wasm.gz` and `.crystalpack.gz` siblings when the browser
accepts gzip.

### Fullscreen scaling mod (Docker multiplayer default)

The Docker-hosted WASM multiplayer client builds with `fullscreen-scaling`.
It renders at the browser's native framebuffer resolution and reveals more of
the 2D world as the viewport grows. Classic bitmap text, dialogs, menus, and
battle screens remain compact panels (normally 480 CSS pixels wide),
with whole physical pixels on standard and high-DPI displays. Title and opening
artwork scale independently to the viewport. The start menu crops out unused LCD
paper and places native controls beside the artwork on wide screens, or below
it on portrait screens. Small maps fit the available area at whole physical-pixel scales independently
of the text scale, and field dialogue sits at the bottom of the viewport.
Full-screen game menus such as the Town Map share one centered transform for
their artwork, labels, and markers, and hide the overworld until closed. The preset
name screen separates its portrait, choices, and dialogue into responsive regions.
Use the browser's
fullscreen button to enter fullscreen; browsers require a user gesture.

This is a client presentation mod, so everyone uses the same compiled game
pack, multiplayer protocol, and saves. Terrain, priority tiles, NPCs, and remote
players share the expanded view. The camera preserves a scrolling margin and
increases integer scaling on very large displays to stay within rendered terrain.
Small windows fit the complete retro UI. Screen fades cover the full viewport.

Build and run the hosted client from the repository root with your configured
`CRYSTAL_AUTH_SECRET`:

```sh
docker compose -f docker-compose.production.yml up -d --build pokecrystal-multiplayer
```

Open `http://localhost:3003`. For native preview, pass
`--features fullscreen-scaling` to `cargo run -p crystal-bevy`.
The ordinary native build retains the original LCD presentation.

### Optional 2.5D overworld mod

The browser multiplayer build includes this renderer. Click **2.5D** next to
the mute button to enable it; click again to return to 2D. The browser remembers
your selection. In 2.5D, use **− / +** to zoom from 75% to 200%, **↶ / ↷**
to rotate in 45° steps, and the percentage button to reset the camera. Zoom
and rotation are remembered across reloads. Arrow keys follow map directions.
Keyboard controls and multiplayer stay the same, and menus
and battles retain their original 2D presentation. Rebuild the Docker image
to include the renderer and its browser toggle.

The normal build keeps the original 2D Game Boy presentation. To opt into the
experimental renderer mod, build the Bevy shell with the non-default
`voxel-view` feature:

```sh
cargo run -p crystal-bevy --features voxel-view -- \
  --pack /path/to/core-modular.crystalpack \
  --save-path /tmp/pokecrystal.crystalsave
```

The mod consumes a read-only render snapshot. It does not change movement,
collision, scripts, random state, battles, saves, or replay checksums. Menus,
dialog, fades, and battles continue to use the faithful 2D compositor. Its
clean-room shape profile is keyed by stable tileset/metatile artwork identity,
never gameplay collision. Presentation settings are the only way to change
between 2D and 2.5D. Unsupported maps, incomplete frames, terrain
builds, and renderer errors report 2.5D as inactive without exposing the 2D
overworld.

For repeatable map screenshots and side-by-side 2D/2.5D inspection, see
[RENDER_AT_LOCATION.md](RENDER_AT_LOCATION.md).

## Verification

For per-system CPU waterfall captures of the actual Bevy renderer, see
[operation profiling](tools/OPERATION_PROFILING.md). Tracing is opt-in and
includes command application and render work across threads.

Verify the pinned ASM checkout and reference ROM before exporting:

```sh
npm run verify:asm
npm run verify:pack
npm run asm:boot
```

From the repository root, rebuild the definitive core pack:

```sh
./export
```

Run the Rust compile and test gates from this directory:

```sh
cargo test --workspace --no-run
cargo test -p crystal-bevy --bin crystal-bevy tests:: --features bevy-shell
```

Runtime diagnostics and scenario construction belong in library integration
tests or dedicated test targets, not in the production game executable.

### Deploy online multiplayer

Set `CRYSTAL_AUTH_SECRET` to a stable random signing secret of at least 32 bytes
(for example, generate one with `openssl rand -hex 32`). Keep it across restarts.

```sh
docker compose -f docker-compose.production.yml up -d --build pokecrystal-multiplayer
```

Open the game's HTTPS URL and play. No invite or account is required. The browser
automatically requests a server-issued player identity from `POST /v1/session`
and remembers its signed credential locally for future visits. Credentials last
ten years. The server chooses identities and verifies them on WebSocket connections.
Browser saves remain local; multiplayer ratings live in the Docker data volume.

The page requires HTTPS (localhost is also supported) for its per-player Web Lock.
A second tab for the same player shows an explanation instead of opening a
duplicate connection or writing the same save concurrently.

Terminate TLS at your proxy and forward `/v1/ws` with WebSocket upgrades to port
8080. Allow an idle timeout longer than 45 seconds. The server sends heartbeat
pings every 15 seconds, expires unresponsive clients after 45 seconds, and bounds
socket writes to two seconds. Failed critical delivery disconnects the client
and cancels its match; presence departures are never silently dropped.

Verification commands from `rust/`:

```sh
cargo test --locked -p crystal-web-server -p crystal-net
node --test web-client/browser-session.test.mjs
cargo check --locked -p crystal-bevy --features fullscreen-scaling,voxel-view --target wasm32-unknown-unknown
```

The Docker build caches Cargo dependencies and compiled targets, and uses two
parallel Cargo jobs by default (`--build-arg CARGO_BUILD_JOBS=N` overrides this).
The `web-release` profile keeps size optimization within crates and disables
whole-program LTO, which exhausted an 8 GiB builder on this game. Static page
and audio updates are copied after compilation, so they do not rebuild Rust.

### WebMCP agents in the hosted game

The loaded WASM page automatically registers seven tools with the
[WebMCP draft API](https://webmachinelearning.github.io/webmcp/):

- `pokemon_observe`: live status, dialogue, rendered text, menu selections,
  naming keyboard, battle presentation, local terrain, multiplayer state and outcome.
- `pokemon_status`, `pokemon_map_info`, `pokemon_flow_state`,
  `pokemon_recent_events`: focused read-only views of that observation.
- `pokemon_press`: one Game Boy button (`up`, `down`, `left`, `right`, `a`,
  `b`, `start`, `select`), held for 1–60 presentation frames, followed by a
  fresh observation. Default: one frame. Menus, naming, battles and saving
  use the same input path as keyboard play.
- `pokemon_multiplayer`: request `battle`, `trade`, or `time_capsule` with
  the player directly ahead, using the game's existing multiplayer controls.
  Respond to incoming requests with A or B.

An agent opens the game, discovers its tools, calls `pokemon_observe`, then
chooses actions from the live results. No separate MCP endpoint, token passed
in tool arguments, game-state editor, routing script or agent account is needed.
The browser's existing multiplayer identity and saves are used. Calls are
serialized, inputs are validated in JavaScript and WASM, and cancellation,
human keyboard input, loss of focus or page exit release agent-held input.
Already processed actions are not rolled back. The normal game continues
animating between calls; action results can still show an ongoing animation.

This targets `document.modelContext.registerTool(tool, {signal})` in the
September 4, 2026 draft, including asynchronous registration, tool annotations,
execution cancellation and registration lifetime. Hosting must use HTTPS
(localhost also qualifies). Docker sends `Origin-Agent-Cluster: ?1` and a
same-origin `tools` permissions policy. Browser and agent-host WebMCP support
is required: the game cannot make an unsupported agent discover tools. The
Controls dialog reports support or registration errors. It does not install
a polyfill or emulate an older API; normal keyboard play remains available.

Verification:

```sh
node --test web-client/webmcp.test.mjs web-client/browser-session.test.mjs
cargo test -p crystal-bevy --features fullscreen-scaling --lib webmcp
cargo check -p crystal-bevy --features fullscreen-scaling,voxel-view --target wasm32-unknown-unknown
```

### Browser chat

Press `Enter` to open chat, then `Enter` again to send and return to the game.
`Esc` closes chat while keeping your draft; `/` opens a command. The keyboard
menu key is `M`; touch and controller Start are unchanged. Typing in chat does
not press game buttons.
The channel selector controls where your next message goes; incoming messages
from joined channels appear together with distinct colors.

- `/s message`: Say to nearby players.
- `/1 message`: General for the current map.
- `/2 message`: Trade across the current world.
- `/3 message`: Looking for Group across the current world.
- `/w player-ID message`: private whisper; click a sender's name to address one.
- `/r message`: reply to the last incoming whisper.
- `/join name`, `/leave name`: join or leave a custom channel; built-in channels
  also accept their numbers or names. Select a joined custom channel to speak.
- `/battle player-ID`, `/tradewith player-ID`: invite a nearby trainer.
- `/cancel`: cancel a pending invitation.
- `/help`: show commands.

The chat-enabled client and relay use hosted protocol version 2 and must be
updated together. Chat stays on the authenticated game connection during trades and battles.
Channels and whispers are isolated by world and exact modpack. Messages are
limited to 280 characters and five sends per five seconds; custom channel names
use 1–24 letters, digits or hyphens. Channel membership is restored on reconnect
within the same page; chat history is held in memory for that page only.

Face a nearby player and press A (`Z`) to open their actions, or click their
name in chat. Choose Battle, Trade, or Whisper. Nearby trainers also appear as
clickable names while chat is open. Invitations offer Accept and Decline buttons.
The existing `C`/`V`/`T` shortcuts still invite the player ahead to battle, trade,
or the Time Capsule; `Z` accepts and `X` declines or cancels an invitation.
Each player can have one pending invitation. Invitations expire after 30 seconds
and cancel if either player disconnects or leaves range, releasing both players
to try again. Matchmaking and pending direct invitations are mutually exclusive.

### Modern physical/special move split

Build the optional pack from the repository root with `./export-modern-move-split`.
It writes `content-packs/modern-move-split.crystalpack`, assigning Physical,
Special, or Status to each of Crystal's 251 moves. Physical attacks use Attack
and Defense; special attacks use the separately stored Special Attack and
Special Defense stats. Battle AI, screens, and Counter/Mirror Coat follow the
move's category. The base game retains Crystal's original type split.
See [the modpack instructions](../modpacks/modern-move-split/README.md) for layering
on the Gen 3 pack, browser output, and source regeneration.


### Self-contained Docker build

The production Docker build includes the exported browser pack and embedded
modpack JSON. No sibling checkout or prepared source snapshot is required.
The server and WASM browser client are compiled from the same checkout.

Set a stable random `CRYSTAL_AUTH_SECRET` of at least 32 bytes in an ignored
`.env` file, then run:

```sh
docker compose -f docker-compose.production.yml up -d --build
docker compose -f docker-compose.production.yml ps
curl --fail http://localhost:3003/healthz
```

Keep the signing secret stable across updates. Ratings persist in the named
data volume; browser saves remain local. Do not delete the data volume during
routine updates. The browser pack is generated: regenerate it with the canonical
exporter when content changes rather than editing it by hand.
