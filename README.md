# Geothite

A Rust implementation of Pokémon Crystal with browser and desktop play,
online multiplayer, and an optional 2.5D overworld renderer.

Geothite separates deterministic game logic from rendering and networking.
The browser build brings the game, audio, touch controls, and multiplayer
into one self-hosted application.

## Features

- Classic 2D presentation with a switchable 2.5D overworld view.
- Keyboard, touch, and gamepad controls with responsive browser layouts.
- Shared-world multiplayer, player chat, battle invitations, and trading.
- Browser saves tied to a persistent player identity.
- A shared server clock for the hosted game's day and night cycle.
- A Docker build that packages the server and browser client together.

This is an actively developed project. See the [fidelity audit](FIDELITY_AUDIT.md)
for implementation status and remaining differences from the original game.

## Quick start

Install Docker with the Compose plugin, then clone the repository:

```sh
git clone https://github.com/TheCulliganMan/geothite.git
cd geothite
```

Create a signing secret once for a new installation:

```sh
umask 077
printf 'CRYSTAL_AUTH_SECRET=%s\n' "$(openssl rand -hex 32)" > .env
```

Start the game:

```sh
docker compose -f docker-compose.production.yml up -d --build
```

Open [localhost:3003](http://localhost:3003). The first build compiles Rust and
WebAssembly dependencies and can take several minutes. Docker includes the
required toolchains and game pack; a separate Rust installation is unnecessary.

Keep `.env` across updates. For remote access, put the server behind HTTPS.
See the [deployment guide](docs/deployment.md) for configuration, reverse
proxying, upgrades, health checks, persistence, and cleanup.

## Playing

In the browser, use the arrow keys to move, `Z` for A, `X` for B, `M` for the
game menu, and Right Shift for Select. Press `Enter` to open chat and `Esc`
to close it. Phones expose touch controls; standard gamepads are supported.
Click or press a key in the game to enable browser audio.

Use the **2.5D** control to switch the overworld renderer. Battles and menus
keep their 2D presentation. Camera zoom and rotation are available in 2.5D.

Browser identities are created automatically. Save through the game's normal
menu; saves remain in that browser. Clearing browser storage removes the local
identity and saves. Server-side multiplayer ratings persist separately.

### Multiplayer

Face a nearby player and press A, or click their name in chat, to choose
Battle, Trade, or Whisper. Invitations can be accepted or declined in the game.
Players must use compatible game packs to share a world.

Chat supports nearby Say, map-wide General, Trade, Looking for Group,
private whispers, and custom channels. Type `/help` for available commands.

### Desktop play

For native play, install the Rust toolchain specified in `rust-toolchain.toml`
and your platform's Bevy build dependencies. Supply a compatible game pack:

```sh
cargo run -p crystal-bevy -- \
  --pack /path/to/core-modular.crystalpack \
  --save-path /path/to/player.crystalsave
```

Add `--load-save /path/to/player.crystalsave` to load an existing save.
Enable the optional renderer with `--features voxel-view` before `--`.
Native controls use `Enter` for Start, alongside the arrow keys, `Z`, `X`,
and Right Shift.

## Development

The workspace is organized around focused Rust crates:

| Path | Responsibility |
| --- | --- |
| `crates/crystal-core` | Game state, timing, battles, and world rules. |
| `crates/crystal-assets` | Compiled game data and content packs. |
| `crates/crystal-audio` | Music, sound effects, and playback data. |
| `crates/crystal-net` | Multiplayer protocol and transport. |
| `crates/crystal-bevy` | Desktop and WASM game client. |
| `crates/crystal-render-api` | Shared presentation snapshots. |
| `crates/crystal-voxel-view` | Optional 2.5D renderer. |
| `crates/crystal-web-server` | HTTP hosting and multiplayer relay. |
| `web-client` | Browser page, controls, audio runtime, and chat UI. |

Check the server and run the browser session tests (the latter require Node.js):

```sh
cargo check --locked -p crystal-web-server
node --test web-client/browser-session.test.mjs web-client/webmcp.test.mjs
```

The production Dockerfile builds the server and WASM client from the same
revision. Keep protocol changes coordinated across both. Generated game packs
should be regenerated using the canonical exporter, not edited manually.

## Contributing

Bug reports should include the revision, platform or browser, reproduction
steps, and any relevant logs. For gameplay differences, describe the expected
Pokémon Crystal behavior and what happened instead.

Keep changes focused and add regression coverage for behavior fixes. Include
screenshots for visual changes and the checks you ran in pull requests.

## Further reading

- [Deployment and maintenance](docs/deployment.md)
- [Game fidelity audit](FIDELITY_AUDIT.md)
- [Renderer inspection](RENDER_AT_LOCATION.md)
- [Operation profiling](tools/OPERATION_PROFILING.md)
