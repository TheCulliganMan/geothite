# Build and run game content

Geothite's Rust builders generate the game content from the source and toolchain
pinned in `pret-source.lock.json`. The initialization script assembles that source,
exports runtime catalogs, compiles and verifies the packs, and publishes local
agent inputs. Source checkouts, ROMs, intermediate exports, PCM, and packs stay
outside tracked source.

## Initialize once

Install Git, Git LFS, and the Rust toolchain selected by `rust-toolchain.toml`.
The build environment also needs Bash, curl, tar, make, a C compiler, and
`sha1sum`/`sha256sum`. Windows uses the `Ubuntu-24.04` WSL distribution by default;
install these tools and Rust inside that distribution, plus Git LFS on Windows.
Native graphical play also needs the platform libraries listed in
[cold-start-3d.md](cold-start-3d.md).

From the checkout, run on Windows:

```powershell
.\tools\build-crystalpack.ps1
```

On Linux x86_64:

```sh
bash tools/build-crystalpack.sh
```

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