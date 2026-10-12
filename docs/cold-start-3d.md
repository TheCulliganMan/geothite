# Move 3D work to a new host

The source branch is `feat/connected-johto-3d`. Transfer its commits, the
separately supplied ignored content pack, and any saves you want to keep.
Tracked models, textures, editable Blender sources and Rust tools travel with
Git. Do not copy `target/`, scratch captures, ROMs or disassembly into Git.

On the old host, after all working chats have committed their owned changes:

```sh
git status --short
git log -8 --oneline
git bundle create /tmp/geothite-3d.bundle feat/connected-johto-3d
tar -C .git/lfs -czf /tmp/geothite-3d-lfs.tar.gz objects
```

The bundle transfers local commits without requiring a push. Git bundles do
not contain Git LFS artwork: the second command archives those local objects.
Copy both files and `content-packs/` separately to the new host using your preferred file
transfer tool. The known-good local pack is
`content-packs/realtime-clock.browser.crystalpack`; `content-packs/` also holds
local saves and optional external models. Packs remain ignored on both hosts.
Create the bundle only after the final commits, or regenerate it afterward.
Uncommitted changes are not included in a Git bundle.

On the new host, with the bundle in the current directory:

```sh
GIT_LFS_SKIP_SMUDGE=1 git clone --branch feat/connected-johto-3d geothite-3d.bundle geothite
cd geothite
git remote set-url origin https://github.com/TheCulliganMan/geothite.git
git lfs install --local
mkdir -p .git/lfs
tar -C .git/lfs -xzf ../geothite-3d-lfs.tar.gz
git lfs checkout
mkdir -p content-packs
```

Install Git LFS before cloning and extracting the artwork. If you instead
clone from GitHub after the source and LFS objects have been pushed, use
`git lfs pull` to retrieve the artwork. LFS pointer files cannot render models.
Restore the separately transferred `content-packs/` directory here. Install
Rust using rustup; `rust-toolchain.toml` selects Rust 1.99.0 automatically.
Native Linux builds require a C compiler, pkg-config, ALSA, udev and X11
development libraries. On Debian/Ubuntu, install these before building:

```sh
sudo apt-get update
sudo apt-get install build-essential pkg-config libasound2-dev libudev-dev libx11-dev libxkbcommon-dev
```

macOS requires Xcode command line tools (`xcode-select --install`). Native
play requires a working graphical session and GPU driver. A headless host can
serve the browser build for a browser with WebGL2 support.

```sh
./scripts/cold-start-3d.sh build
./scripts/cold-start-3d.sh play
```

`build` validates the actual pack and artwork with Rust, then compiles the
native client with `voxel-view`. `play` starts the normal production title and
save flow; F3 toggles 3D. The separate `preview` command opens a fresh New Bark
developer location preview in 3D, preserving story scripts and gates:

```sh
./scripts/cold-start-3d.sh preview --map CherrygroveCity
```

For a browser host:

```sh
./scripts/cold-start-3d.sh setup-web
./scripts/cold-start-3d.sh web
```

Open `http://localhost:3003/?multiplayer=off` for normal play or
`http://localhost:3003/?multiplayer=off&preview=new-bark` for a developer preview.
`setup-web` installs the Wasm target and the wasm-bindgen CLI version selected
by Cargo.lock. `web` uses the existing Rust browser build pipeline, bundles
validated artwork, and serves locally. Set `CRYSTAL_WEB_HOST=0.0.0.0` to listen
on the host's network interfaces and `CRYSTAL_WEB_PORT` to change the port.
Server state stays ignored under `target/new-bark-3d-data`; preserve that
directory separately if you later need its server-side saves.

The script runs from any working directory. `CRYSTAL_PACK` and
`CRYSTAL_OPEN_MODEL_ROOT` override input locations; relative overrides resolve
from the calling directory. Builds use `target/` and default to two Cargo jobs;
set `CARGO_BUILD_JOBS` for the destination's memory budget. Native builds accept
`CRYSTAL_BUILD_PROFILE=dev` (default), `wasm-dev` (optimized without debug
symbols), or `release`, and `CARGO_TARGET_DIR` for a different build cache.
The browser helper uses the repository's `target/` directory. `check` performs
pack/art validation without opening a window. A successful check or build does
not establish gameplay or rendering quality; run the controller and MCP
regressions in AGENTS.md and inspect the 3D client on the destination GPU.
