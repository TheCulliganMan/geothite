# Build and run game content

Geothite's Rust builders generate the game content from the source and toolchain
pinned in `pret-source.lock.json`. The initialization script assembles that source,
exports runtime catalogs, compiles and verifies the packs, and publishes local
agent inputs. Source checkouts, ROMs, intermediate exports, PCM, and packs stay
outside tracked source.

## Desktop app packaging

The packaging commands consume the completed local native pack; they do not
re-export content or rebuild packs. The default is
`content-packs/core-modular.crystalpack`. Packs are copied only into ignored local
distribution output, never embedded into Rust source or tracked in Git.

```sh
just desktop-content     # Once: prepare external content, if missing
just desktop-build       # Compile and validate; no window or installation
just desktop             # Compile, validate and install for the current OS
just desktop-run         # Launch the installed copy without rebuilding
just desktop-package     # Compile and produce the native distributable
just desktop-repack      # Repackage the last successful build, no Cargo
just desktop-install     # Install the last successful build, no Cargo
just desktop-check       # Revalidate content and artwork, no window
```

The Unix implementation is `python3 tools/desktop.py`, requiring Python 3.9+.
Windows uses `tools/build-desktop.ps1`; its existing `-Pack`, `-InstallDirectory`
and `-Launch` options remain available, with `-Action` and `-SkipBuild` added.
All commands build for the current host OS and architecture. Windows/Linux
runners need native verification before claiming release support there.

Both use persistent `target/desktop-build` and the `release` profile by default,
so normal builds, installs and packages use full game and engine optimization.
`CARGO_TARGET_DIR` selects another cache. For faster development builds, select
`GEOTHITE_DESKTOP_PROFILE=wasm-dev` (or pass the existing profile option). That
profile uses optimization level 1 for game crates and level 2 for engine
dependencies; it trades runtime performance for shorter compilation. Non-release
desktop builds keep an incremental cache in this dedicated target directory;
set `CARGO_INCREMENTAL=0` to disable it when disk space matters more than update
time. Keep the same profile when repackaging or installing with `--skip-build`.
The first toolchain/content/Bevy build is not a two-minute operation. Warm-cache
updates and repackaging avoid that work; the script reports actual elapsed time.
Each packaging run verifies the actual pack's desktop title/intro metadata.
The Rust scenery and people validators also parse the artwork;
all platforms load the staged production client with `--check-installation`
before publishing. This checks resources and performs a temporary write probe at
the save location, including atomic backup suffixes, without opening a window.
A package check alone does not certify gameplay; use the AGENTS.md controller,
battle, MCP and clean-stderr regressions when gameplay changes.

Verified on Apple Silicon on 2026-10-18: `just desktop-package` completed in
132.9 seconds with a warm cache, including incremental game-crate compilation,
Rust content/art validation, signing and DMG creation. `just desktop-install`
completed in 51.8 seconds. Initial compilation took over ten minutes. The DMG
was mounted read-only, its SHA-256 and app signature checked, and its actual
client passed `--check-installation` from `/tmp` with the artwork override unset.
The bundled pack matched the original byte-for-byte. The installed app opened
through Launch Services and reached trainer selection and the opening dialogue.

| Host | Installed app | Distribution under `.local/desktop-dist/` |
| --- | --- | --- |
| macOS | `~/Applications/Geothite.app` | `Geothite-VERSION-macos-ARCH.dmg` and `.sha256` |
| Windows | `%USERPROFILE%\Games\Geothite` and Desktop shortcut | `Geothite-VERSION-windows-ARCH.zip` and `.sha256` |
| Linux | `~/.local/opt/geothite` and application-menu entry | `Geothite-VERSION-linux-ARCH.tar.gz` and `.sha256` |

The macOS app uses `Contents/MacOS/Geothite`, `Contents/Resources` and an
`Info.plist`, with no shell launcher. Resources and artwork resolve relative to
the executable, including when opened from Finder or a mounted, read-only DMG.
Only system dynamic libraries are accepted. Packaging uses Apple's `sips`,
`iconutil`, `codesign`, `otool` and `hdiutil`, without an external bundler dependency.
The DMG has an Applications shortcut for drag-and-drop installation; `just desktop`
installs to the current user's Applications folder without administrator access.
The bundled game starts through the production title flow, not a location preview.

Packaged native clients (`desktop-app`, without `location-tester` or `dev-tools`)
play music and sound effects by default. Native developer clients remain muted,
including release builds of preview tools; `CRYSTAL_PREVIEW_SOUND` no longer
enables them. `CRYSTAL_PREVIEW_MUTE=1` can explicitly silence a packaged client.
Opt-in `CRYSTAL_AUDIO_TRACE=1` records the native sink volume and playback timing.
The Mac window's title-bar Settings button and native Settings menu expose
volume (0–100% in 10% steps), mute, the 2D/3D toggle, a direct checkable First Person
toggle, default/close/wide orbit, camera reset, fullscreen and Reset Settings.
Turning First Person off returns to orbit; enabling it from 2D also enables 3D.
Orbit uses the interpolated trainer footing as its pivot, so clamped indoor
viewports no longer rotate around the room center. These controls edit
presentation resources, while the production controller retains gameplay input.
The master gain scales music and sound effects separately from authored music
fades and fanfare suppression. Developer mute remains authoritative.
Preferences persist atomically in
`~/Library/Application Support/Geothite/settings.json`; camera changes are
debounced, and normal exit flushes pending settings. F3, mouse, controller and
fullscreen changes are mirrored back to both menus. `CRYSTAL_SETTINGS_TRACE=1`
records effective presentation preferences for live checks.
After building, `python3 tools/test_desktop_settings.py` checks preference
restoration, real AppKit target/action dispatch and Metal-layer preservation
using an offscreen window that is never shown.
Normal web play enables sound on its first trusted click, touch or keypress.
Browser previews, `?development=1` and embedded training views keep their master
gain muted while preserving source audio timing. A new `crystal.audio.preference` storage key
records only deliberate user choices; the old forced-mute value is ignored.

Audio-default verification on 2026-10-18 passed all 107 browser regressions and
all 377 PCM hash/frame/loop comparisons in each native and browser pack. The
installed Mac client started both title music and sound effects at sink volume
`1.0`. Its executable matched the rebuilt DMG, whose signature and checksum passed.
The commands used for the pack and browser checks were:

```sh
target/desktop-build/wasm-dev/verify_crystalpack content-packs/core-modular.crystalpack --desktop --audio
target/desktop-build/wasm-dev/verify_crystalpack content-packs/core-modular.browser.crystalpack --desktop --audio
npm run test:browser
```

New desktop saves use `~/Library/Application Support/Geothite/saves` on Mac,
`%LOCALAPPDATA%\Geothite\saves` on Windows, and `$XDG_DATA_HOME/geothite/saves`
(or `~/.local/share/geothite/saves`) on Linux. Usable existing portable saves beside the
pack continue to use that location. Overlong legacy filenames are copied to the
bounded user-data filename without deleting originals or replacing newer saves. Explicit `--save-path` now selects both the
save target and the title menu's Continue lookup. Startup errors from the Mac
GUI show a native alert and are recorded in `~/Library/Logs/Geothite/startup-error.log`.
`--check-installation` suppresses alerts for automated checks.
`just desktop-run` also captures normal launch output in
`~/Library/Logs/Geothite/game.log` on macOS.

Local Mac packages are ad-hoc signed and are not notarized. For public
Gatekeeper distribution, install a Developer ID Application signing identity and
configure an existing `notarytool` keychain profile. Then run:

```sh
GEOTHITE_SIGN_IDENTITY='Developer ID Application: Your configured identity' \
GEOTHITE_NOTARY_PROFILE='your-existing-keychain-profile' just desktop-release
```

That command uploads the signed DMG to Apple's notary service, waits for an
accepted result, then staples and validates its ticket. Normal build/install/
package commands do not upload anything. See
[Apple's notarization documentation](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution).
An Intel Mac must build its own native artifact; this checkout's local Apple
Silicon artifact does not establish Intel, Windows or Linux runtime acceptance.

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

All three remain ignored by Git. The initializer applies the shared
[All Pokémon, solo](../modpacks/all-pokemon/README.md) mod to both source pack
formats before creating the TUI pack. All 251 species are catchable by default;
Celadon Department Store 4F sells all six evolution stones and a ¥2,100 Link
Cable. Use the cable from PACK on a trade-evolving Pokémon, holding its usual
trade item when required. Successful use consumes the cable and required held
item. This gives native, browser, TUI and MCP clients the same content policy.
The source-only exports in the external workspace retain vanilla tables.

To enhance an existing verified pack without rebuilding source exports:

```sh
cargo run --locked -p crystal-assets --bin pack_all_pokemon -- \
  content-packs/core-modular.crystalpack content-packs/all-pokemon.crystalpack
cargo run --locked -p crystal-assets --bin pack_all_pokemon -- \
  content-packs/core-modular.browser.crystalpack content-packs/all-pokemon.browser.crystalpack
cargo run --locked -p crystal-assets --bin pack_text_tui -- . \
  content-packs/all-pokemon.browser.crystalpack content-packs/all-pokemon.text-tui.crystalpack
```

These commands preserve the supplied pack's audio storage and leave its source
file intact. The additional manifest identities separate modded saves from
older saves; no save migration happens automatically. Rebuild clients from
this checkout for Link Cable held-item behavior.

### Targeted source-text repair on 2026-10-17

Battle Tower selects 120 greeting, win and loss messages from
`data/battle_tower/trainer_text.asm`, outside the usual source text directories.
The Rust exporter includes this file and rejects missing selected messages.
With the pinned source and an already prepared external workspace, the repair
used these `pack_core` stages (built with the `wasm-dev` profile):

```sh
workspace="$HOME/.local/share/geothite/export-workspace"
cargo run --locked --profile wasm-dev -p crystal-assets --bin pack_core -- \
  "$workspace" --export-asm-text
cargo run --locked --profile wasm-dev -p crystal-assets --bin pack_core -- \
  "$workspace" --pack-prepared-workspace
```

These stages produce vanilla source packs. Apply `pack_all_pokemon` and then
`pack_default_battles` to each format, derive the TUI pack with `pack_text_tui`,
and run the initializer's publication checks before replacing shared packs.
The verified repair added exactly 120 messages to each published pack; all old
texts, other game-data fields, runtime files and embedded audio were identical.
All 377 PCM hash/frame/loop checks passed for each pack. Twelve Tower controls,
early-game controller checks, earned Hall/Continue, MCP movement/menu and
clean-stderr checks passed. The new pack identities require matching saves;
existing saves were not rewritten.

### Solo-completion pack verification on 2026-10-16

The shared Rust overlay builder generated native, browser and text-client packs
from the existing verified local content. The native pack passed production
presentation validation. The TUI pack passed all 377 audio PCM hashes, frame
counts and loop ranges. The real-pack item tests passed purchases, all ten solo
trade evolutions, failed-use inventory preservation, 251-species encounter
coverage and round-trip preservation of unrelated content and audio.

This run reused content and build caches. Pack tools used
`target/solo-completion` with the `wasm-dev` profile and an assets optimization
override of 0. Client regressions use the standard optimized `wasm-dev` profile.
The standard controller suite passed all seven renderer-neutral regressions,
including cable use with a complete evolution confirmation, Mom/New Bark,
battle command menus and PP mutation, department-store clerks, running shoes,
Research Center and trainer rematches/save. The native earned Hall-of-Fame
ceremony, save and Continue regression also passed. MCP movement/Start-menu
and byte-empty stderr checks passed. The production TUI was launched with
`target/solo-completion/wasm-dev/geothite play content-packs/all-pokemon.text-tui.crystalpack`;
Enter opened its production Start menu and Ctrl-C exited cleanly.

The three canonical files in `content-packs/` were then atomically replaced
with the verified native, browser and TUI candidates. Byte comparisons matched
all three published files to those tested candidates. The independent
`all-pokemon.*.crystalpack` files remain available for explicit selection; all
packs stay ignored by Git. Existing saves retain their original identities.

The executed pack commands were:

```sh
target/solo-completion/wasm-dev/pack_all_pokemon \
  content-packs/core-modular.browser.crystalpack content-packs/all-pokemon.browser.crystalpack
target/solo-completion/wasm-dev/pack_text_tui . \
  content-packs/all-pokemon.browser.crystalpack content-packs/all-pokemon.text-tui.crystalpack
target/solo-completion/wasm-dev/pack_all_pokemon \
  content-packs/core-modular.crystalpack content-packs/all-pokemon.crystalpack
target/solo-completion/wasm-dev/verify_crystalpack \
  content-packs/all-pokemon.crystalpack --desktop
target/solo-completion/wasm-dev/verify_crystalpack \
  content-packs/all-pokemon.text-tui.crystalpack --audio --desktop
```

The independent radio oracle tests use external fixtures at runtime rather than
embedding files from another checkout into the test executable. To run those
opt-in comparisons, set `CRYSTAL_ORACLE_FIXTURE_ROOT` to their external fixture
directory and `CRYSTAL_RENDER_TEST_PACK` to a completed pack, then use
`cargo test --locked -p crystal-assets --lib radio_ -- --ignored`. Missing
configured inputs fail those tests; normal test compilation needs no sibling
repository or game-content fixtures.

Independent PC and Mailbox LCD references can be captured from the completed
external ROM and symbols with the optional [menu QA tool](menu-oracle.md).
Those tests use `GEOTHITE_ORACLE_FIXTURES`, separately from the radio fixture
setting. The emulator environment and all generated reference content remain
external; no pack, audio exporter, or shipped dependency changes are required.

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

### Wild Challenge and standalone variants

[Wild Challenge](../modpacks/wild-challenge/README.md) scales wild encounters
with the strongest three non-egg party members and builds legal competitive
movesets from the supplied pack. Standard packs enable it together with trainer
profiles. It preserves encounter species, authored level floors, unrelated
content and audio. Build a separate ignored terminal variant with:

```sh
cargo run --locked --profile wasm-dev -p crystal-assets --bin pack_wild_challenge -- \
  content-packs/text-tui.crystalpack content-packs/wild-challenge-tui.crystalpack
```

Use `core-modular.crystalpack` or `core-modular.browser.crystalpack` as the input
for the corresponding graphical format. The builder preserves storage format,
verifies the input and output, and rejects overwriting the input. Rebuild the
Rust client to use the rules. Modded saves have a distinct pack identity; choose
a separate save path. The mod's README describes the level curve, move selection
and external-pack regression command.

The native `wild-challenge.crystalpack`, browser
`wild-challenge.browser.crystalpack`, and terminal
`wild-challenge-tui.crystalpack` outputs have been built and reloaded locally.
The verified terminal workflow uses the optimized `wasm-dev` profile:

```sh
export CARGO_TARGET_DIR=target/agent-wild-challenge
cargo build --locked --profile wasm-dev -p geothite
"$CARGO_TARGET_DIR/wasm-dev/geothite" play content-packs/wild-challenge-tui.crystalpack
```

The Wild Challenge integration suite, required battle and Mom/New Bark
controller regressions, MCP movement/Start test, and clean-stderr regression
passed against the generated terminal pack. Interactive launch, Start-menu
input, and a scratch save/reload were also verified.

### Launch a client

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

## Serve a prepared browser copy

`just serve` starts the Rust server on `127.0.0.1:3003` using the existing
ignored `.local/desktop-3d-runtime/crystal-web-server` executable and `web/`
bundle, including its local browser pack, Wasm modules, and model assets.
Server data stays under that runtime's `data/` directory. Open
[localhost:3003](http://localhost:3003/?multiplayer=off); Ctrl-C stops serving.
Use `just serve 3004` for another port, or `just serve 3004 /absolute/runtime`
for another prepared directory with the same layout.

This command reuses a prepared build. For source rebuilding, run
`sh scripts/cold-start-3d.sh setup-web` once, then
`CRYSTAL_PACK=content-packs/core-modular.browser.crystalpack sh scripts/cold-start-3d.sh web`.
The source path is documented in [the host runbook](cold-start-3d.md).
On 2026-10-13, the source-built loading test client completed three 3D
house-exit/re-entry cycles with production movement and Start-menu input in
Chromium/WebGPU, without rendering errors or context loss. A separate
WebKit/WebGL bundle also completed an exit/re-entry, movement and menu check
without render errors or context loss. Its 2D bedroom/house
renderer remains available; the captured 2D house frame matched the WebGL frame
pixel for pixel. The controller, battle/PP, stdio-MCP movement/menu and clean
stderr regressions passed against the external TUI pack. Browser memory is
reported as an observation; no fixed memory budget is an acceptance requirement. See the
[loading diagnostics](cold-start-3d.md#loading-waterfall-diagnostics) for commands
and the measured limits.

## Optional trainer rematches

The [trainer-rematches overlay](../modpacks/trainer-rematches/README.md) adds
repeat conversations and scaled battles to ordinary and scripted trainers.
Their standard dialogue plays before the short rematch line and battle.
Build it from the completed local pack without rerunning source export:

```sh
cargo run --locked -p crystal-assets --bin pack_trainer_rematches -- \
  content-packs/text-tui.crystalpack content-packs/trainer-rematches-tui.crystalpack
cargo run --locked -p geothite -- play content-packs/trainer-rematches-tui.crystalpack
```

The native/browser inputs use the same builder. Keep outputs ignored and use
the overlay's own saves. See the mod guide for scaling rules and regressions.

## Share packs and isolate builds

The initializer composes [competitive battles](../modpacks/competitive-battles/README.md)
and [Wild Challenge](../modpacks/wild-challenge/README.md) into its native,
browser, and text candidates before publication gates. Hosted pack preparation
uses the same Rust composition. Build a separate local variant from an existing
completed text pack with:

```sh
cargo run --locked -p crystal-assets --bin pack_default_battles -- \
  content-packs/text-tui.crystalpack content-packs/default-battles.crystalpack
```

The builder enables trainer-class profiles, per-move categories, scaled wild
encounters, legal wild movesets, and the shared wild move selector. It preserves
other content and audio, verifies and reloads its candidate before publishing,
and is idempotent. Live wild selection and mutation replay use the same method;
tutorial and contest battles retain their original rules. Native/browser inputs
use the same command. Source and output paths must differ.

The local standard native, browser, and text packs were refreshed from their
completed predecessors with this Rust builder. Their previous versions remain
ignored as `core-modular.before-default-battles.crystalpack`,
`core-modular.browser.before-default-battles.crystalpack`, and
`text-tui.before-default-battles.crystalpack` for existing saves. Use a new save
with the upgraded packs. Content/audio preservation and pack round trips passed
for both native and browser storage, along with the trainer/wild integration,
controller battle and Mom/New Bark, MCP movement/Start, and clean-stderr gates.
The refreshed release text client was launched and accepts real keyboard input:

```sh
./target/competitive-battles-release/release/geothite play content-packs/text-tui.crystalpack
```

The prepared native `just play-3d` client was rebuilt against these rules and
checked with movement and matching 2D/3D captures. This refresh composed the
existing external packs; it did not rerun the full source exporter.

The matching prepared browser runtime is `.local/default-battles-runtime`.
Its Rust server, WebGL/WebGPU clients, and terrain/actor workers were rebuilt.
Relaunch it with:

```sh
just serve 3006 .local/default-battles-runtime
```

Open `http://localhost:3006/?multiplayer=off`. Browser 2D checks exercised Start,
keyboard movement, and complete trainer/wild turns through the production
controller, including PP consumption and an actual trainer switch. A separate
headless outdoor 3D performance probe rendered New Bark and accepted Start,
but exceeded its 240-second host deadline; it is not a passing performance
result. The older server on port 3003 was left running separately.

Use the generated file as `CRYSTAL_RENDER_TEST_PACK` for the controller and stdio gates.
The full source initializer must complete its gates for a new canonical pack
publication.

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

### Hall-of-Fame publication gate

The initializer also requires
`kanto_lance_controller_completes_earned_ceremony_and_continues_home` against
the verified native candidate before publication. This earns Lance's victory
through the production controller, runs the original Hall dialogue/machine
approach and ceremony, waits for THE END, returns to title and applies the
production Continue transaction. It checks that credits own movement/Start
input while their presentation clock advances. The source exporter binds
credits metadata to the pinned external source; packs without that operation
fail this gate even when their early-game controller checks pass.

On 2026-10-16 the initializer completed with exit zero on macOS using the
existing pinned source workspace and this task's isolated cache:

```sh
CRYSTAL_PACK_BUILD_PROFILE=wasm-dev \
CRYSTAL_PACK_BUILD_DIR="$PWD/target/kanto-resume" \
CARGO_BUILD_JOBS=1 bash tools/build-crystalpack.sh --jobs 1
```

The normal optimized earned ceremony gate passed without a stack override,
alongside all 28 intro scenes, 377 audio hash/frame/loop checks and the existing
controller/MCP/clean-stderr gates. All three canonical packs were published.
Non-title game data, all 4,915 runtime assets and audio fingerprints matched the
prior pack. The resulting client was launched with
`target/kanto-resume/wasm-dev/geothite play content-packs/text-tui.crystalpack`;
Start opened its production menu and the client exited cleanly. This reused
source and build caches and is not another cold-start proof.

## Natural controller movement

The 3D client defaults to continuous movement for existing and new packs;
arrows, WASD, D-pad, and left-stick movement follow the camera in orbit and
first person, with equal-speed keyboard diagonals. Native Q/E and controller
shoulders rotate continuously while held. 2D and the text client retain tile
controls. Existing Natural Movement packs need no regeneration. The optional Rust
`pack_natural_movement` builder records that choice explicitly in an external
pack, preserving all content and audio. See
[the movement modpack](../modpacks/natural-movement/README.md) for controls,
build arguments, and the `natural_movement: false` opt-out.

The verified outputs are `content-packs/natural-movement.crystalpack` and
`content-packs/natural-movement.browser.crystalpack`, built from the matching
core modular packs. Both remain ignored. Native validation used the production
`render_at_location` example in headless image mode; controller, battle, MCP,
and clean-stdio regressions use the completed external text pack.
