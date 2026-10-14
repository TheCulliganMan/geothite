# Build and run game content

Geothite's Rust builders generate the game content from the source and toolchain
pinned in `pret-source.lock.json`. The initialization script assembles that source,
exports runtime catalogs, compiles and verifies the packs, and publishes local
agent inputs. Source checkouts, ROMs, intermediate exports, PCM, and packs stay
outside tracked source.

## Initialize once

The single platform-selecting command is `just crystalpack`.

Install Git, Git LFS, and the Rust toolchain selected by `rust-toolchain.toml`.
The build environment also needs Bash, curl, tar, make, a C compiler, and
`sha1sum`/`sha256sum`. Windows uses the `Ubuntu-24.04` WSL distribution by default;
install these tools and Rust inside that distribution, plus Git LFS on Windows.
Native graphical play also needs the platform libraries listed in
[cold-start-3d.md](cold-start-3d.md).

On macOS, install Bison, pkgconf, and libpng with Homebrew. The initializer
builds the pinned RGBDS source outside the checkout, uses Homebrew's Bison
without changing shell configuration, and verifies the assembled ROM's SHA-1.

From the checkout, run on Windows:

```powershell
.\tools\build-crystalpack.ps1
```

On Linux x86_64:

```sh
bash tools/build-crystalpack.sh
```

The same Bash command runs on macOS. Paths and checksum verification do not
require GNU `realpath` or GNU checksum command options.

The external workspace defaults to `$HOME/.local/share/geothite/export-workspace`
inside the Linux build environment. The script selects the exact pret commit and
tree, downloads and checks RGBDS 1.0.4, and checks the assembled ROM identity.
It initializes Git LFS and verifies artwork, then runs the repository's Rust
exporters for game data, title and intro presentation, graphics, animation, and
audio. It generates the manifest and source provenance in the external workspace.

Both browser and TUI audio verification check every program's PCM hash, frame
count, and loop range. Controller tests exercise dialogue ownership, Mom, the
New Bark exit gate, production menus, and a battle turn with PP mutation. MCP
tests exercise the same input controller and require clean stderr. The local
packs are replaced only after these checks and the text client build pass.

The published files are:

| Client | Local pack |
| --- | --- |
| Native game | `content-packs/core-modular.crystalpack` |
| Browser | `content-packs/core-modular.browser.crystalpack` |
| Terminal and stdio MCP | `content-packs/text-tui.crystalpack` |

All three remain ignored by Git.

The independent radio oracle tests use external fixtures at runtime rather than
embedding files from another checkout into the test executable. To run those
opt-in comparisons, set `CRYSTAL_ORACLE_FIXTURE_ROOT` to their external fixture
directory and `CRYSTAL_RENDER_TEST_PACK` to a completed pack, then use
`cargo test --locked -p crystal-assets --lib radio_ -- --ignored`. Missing
configured inputs fail those tests; normal test compilation needs no sibling
repository or game-content fixtures.

### Fresh-source verification on 2026-10-13

Two independent fresh remote checkouts at `a55d3ea`, each with empty source,
export, Cargo registry and build caches, completed the default `just crystalpack`
command with exit code 0 and no prompts on macOS. Both initialized the pinned
pret source and RGBDS 1.0.4 and assembled the reference ROM successfully.

Each run certified the continuous production intro against all 28 pinned ROM
scenes, verified all 377 audio programs' PCM hashes, frame counts and loop
ranges, and passed the battle, Mom/New Bark, MCP movement/Start-menu and
byte-empty stderr regressions. All three published pack formats matched
byte-for-byte between both runs and the shared checkout. Packs remain ignored;
verification logs are external to the repository.

### Source rebuild verification on 2026-10-14

The follow-up source rebuild completed with exit code 0 using the existing
pinned external workspace and an isolated cache:

```sh
CRYSTAL_PACK_BUILD_PROFILE=wasm-dev \
CRYSTAL_PACK_BUILD_DIR="$HOME/.cache/geothite/agent-source-timing" \
just crystalpack
```

All 28 intro scenes and 377 audio programs passed certification. The actual
candidate passed battle execution, Mom/New Bark progression, Research Center
startup, MCP movement/Start-menu, and byte-empty stderr regressions before all
three canonical packs were published. The updated production TUI was launched
successfully. This reused source and build caches; the independent cold-start
proof remains the two runs described above.

The packs now contain 159 additional authored scene constants and the authored
Surf executable closure. Strict comparisons preserved every existing script
record, all unrelated game data, and every runtime/audio byte. No generated
packs or verification output are tracked.

## Play

The initialization script prints the built text executable's absolute path and
its play command. For a native Windows terminal build:

```powershell
cargo build --release --locked -p geothite
.\target\release\geothite.exe play content-packs/text-tui.crystalpack
```

The same executable supports the stdio MCP session:

```powershell
.\target\release\geothite.exe mcp content-packs/text-tui.crystalpack
```

Build and launch the graphical client with the production title and save flow:

On Windows, `just desktop` builds and installs the graphical executable with
an embedded icon, the validated native pack, model assets, and a Desktop
shortcut. Install `just`, the pinned Rust toolchain, and the Visual Studio C++
tools with Windows SDK first. Keep Git LFS artwork checked out. Use
`just desktop-content` to run the initializer above when native content has not
yet been generated. To select another completed native pack, use
`just desktop 'content-packs/outdoor-base-visual.crystalpack'` or pass its path.
The install lives in `%USERPROFILE%\Games\Geothite`; launch `Geothite.lnk` from the
Desktop. Saves stay in the install's `saves` directory across recipe updates.
The recipe rejects packs that fail the production presentation consumers,
including uncertified intro timing, before changing the install.
The installed app resolves its pack and models beside the executable, opens
without a console, and shows startup errors in a Windows dialog.

For a direct Cargo launch:

```sh
cargo run --release --locked -p crystal-bevy --features voxel-view -- \
  --pack content-packs/core-modular.crystalpack \
  --save-path content-packs/player.crystalsave
```

Press F3 to switch between 2D and 3D. To deploy the generated browser pack:

```sh
docker compose -f docker-compose.production.yml up -d --build
```

## Share packs and isolate builds

Run pack initialization once for the checkout. Agents use the published packs;
they do not regenerate exports independently or overwrite the shared workspace.
Use a distinct Cargo target directory for each agent's client builds:

```powershell
$env:CARGO_TARGET_DIR = Join-Path $PWD 'target/agent-title'
$env:CRYSTAL_RENDER_TEST_PACK = Join-Path $PWD 'content-packs/text-tui.crystalpack'
```

```sh
export CARGO_TARGET_DIR="$PWD/target/agent-title"
export CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack"
```

Choose a different target directory for every concurrent agent. The initializer's
persistent cache defaults to `$HOME/.cache/geothite/pack-builder`; override it
with `CRYSTAL_PACK_BUILD_DIR`. `CRYSTAL_PACK_BUILD_PROFILE` accepts `release`
(default) or `wasm-dev`, and applies consistently to builders and tests. Windows
forwards these two settings into WSL. Keep all generated packs and captures ignored.
