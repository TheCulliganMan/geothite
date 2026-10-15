# Move 3D work to a new host

The source branch is `feat/connected-johto-3d`. Transfer its commits and any
saves you want to keep. Generate fresh ignored content with `just crystalpack`
after installing the tools in [game content setup](game-content.md), or transfer
an already verified local pack separately.
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
transfer tool. The verified shared browser pack is
`content-packs/core-modular.browser.crystalpack`; `content-packs/` also holds
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
Generate the canonical packs with `just crystalpack`, or restore your separately
transferred `content-packs/` directory here. The native and TUI packs are
`content-packs/core-modular.crystalpack` and `content-packs/text-tui.crystalpack`.
Install
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

`build` compiles the native client and validation tools together with
`voxel-view`, then validates the actual pack and artwork with Rust. `play` starts the normal production title and
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
by Cargo.lock under the ignored `target/cold-start-3d-tools/` directory,
preserving any globally installed CLI version. `web` selects that local CLI,
uses the existing Rust browser build pipeline, bundles
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

## Transfer verification on 2026-10-12

- The finalized script's `build` action passed with `CRYSTAL_BUILD_PROFILE=wasm-dev`
  and `CARGO_TARGET_DIR=$PWD/target-bevy20-verify`, compiling the native 3D client
  and validating the supplied realtime-clock pack and actual LFS artwork.
- `play --help` passed when invoked from outside the checkout, verifying the
  compiled client path and argument forwarding. This did not open a GPU window
  or prove gameplay.
- `cargo check --locked --target wasm32-unknown-unknown -p crystal-bevy --features location-tester`
  passed against the final source. Browser packaging, GPU rendering and the
  ChatGPT crash scenario were not rerun as part of this transfer check.
- All 22 shell command blocks in the transfer/goal notes passed syntax checks;
  all local handoff links resolve, and Git LFS integrity passed.
- An offline clone from the bundle, local LFS-object restore, content-pack
  restore and Rust pack-load smoke check passed in a fresh temporary checkout.
  The restored checkout was clean, and its temporary files were removed.

The six goal notes in [the handoff index](handoff/README.md) preserve unfinished
visual work and failing/pending checks, including inverted outdoor rock-cap
triangles, character defects and the unverified ChatGPT Goldenrod crash fix.


## Loading waterfall diagnostics

For a 30-frame WebGPU waterfall, run the walking harness with
`GPU_WATERFALL=1` and `loading-trace=on`, then render its report:

```sh
python3 tools/with-graphics-lock.py -- env GPU_WATERFALL=1 \
  PROBE_STARTUP_INPUT=1 SCENARIO=walking node tools/browser-walking-performance.mjs \
  'http://127.0.0.1:3004/?preview=route29&renderer=webgpu&multiplayer=off&loading-trace=on' \
  .local/loading-empty-storage.json 2.5d .local/webgpu-30-frame.json
.local/loading-charts-env/bin/python tools/render-frame-waterfall.py \
  .local/webgpu-30-frame.json
```

The chart renderer needs Matplotlib in its Python environment. Timestamp queries
are enabled only in this opt-in QA harness. It records exactly 30 render
submissions, reads timestamps after walking, and pairs them with 30 consecutive
Rust render frames using their submit-time clock bounds. CPU schedule spans and
GPU passes have separate clocks. Empty unwritten query pairs are reported and
excluded; sub-resolution passes can report zero duration. These are averaged
work timings, not displayed FPS, and include query/resolve instrumentation.
The completed full-detail captures measured the CPU main stage at 15.80 ms
before the latest changes and 10.50 ms afterward. Engine-frame interval changed
from 37.28 to 33.53 ms; the current opaque 3D GPU pass is still 16.65 ms and its
shadow pass 5.44 ms. Those GPU passes remain performance work. Both captures
passed real keyboard movement with no graphics errors. The rebuild explicitly
includes `crystal-voxel-view/scenery-full-detail`; the lower-detail build rejected
by the harness is not the served WebGPU bundle.

The palette-surface pipeline now specializes its two whole-object discard
modes from the material cutoff mode. Opaque trees omit these instructions;
fading clones retain the original source-alpha and screen-coverage tests.
Location-test browser builds accept `surface-discard=legacy` for a same-binary
comparison with the original uniform-guarded instructions. This switch does not
change geometry, lighting, shadows or fade values. The full-detail same-binary walking comparison measured the opaque pass at
17.73/17.86 ms with legacy instructions and 15.66 ms with specialization.
A fixed-camera comparison at Route29 `(50, 9)`, facing down, measured
20.96 versus 17.42 ms. Both fixed captures retained position/facing/time;
99.75% of screenshot pixels matched, with visible differences limited to NPC
turning and small idle animation. All five 30-frame captures and the 2D walking
regression passed without graphics errors. GPU clock variation and movement
make these observed workload timings rather than a universal percentage claim.
The QA artifact runs on port 3005; port 3004 keeps the prior playable bundle.
Whole-game loading acceptance remains open.

The separate
`shadow-receivers=on` control enables conservative receiver culling for QA;
it remains disabled by default pending moving-camera visual acceptance.

Merged opaque tree meshes restore 16-bit indices after merging when every
final offset fits; larger meshes and browser alignment retain 32-bit indices.
The 144-prototype audit preserves 8,166,816 source indices and confirms that
all tree UVs are zero. Character finger shaping now uses conservative projected
triangle bins with source order preserved. A frozen-source comparison across
16 supplied-rig cases measured 5,974 ms versus 603 ms for that isolated step,
with bit-exact geometry, normals, UVs, colors, indices and skin data. This is
not a whole-game loading-time measurement. The targeted geometry, placement,
fade, character and shadow regressions pass (48 tests).

When multiple chats share the Mac or Linux checkout, serialize graphical QA:

```sh
python3 tools/with-graphics-lock.py -- COMMAND ARGUMENTS
```

The advisory kernel lock lives in ignored `.local/graphics-qa.lock`. Use the
wrapper for native previews, captures and browser QA; CPU-only builds and tests
can continue concurrently. Keep browser test commands alive until their browser
closes. Existing unwrapped previews must be coordinated with their owner first.
The lock releases automatically when its command exits, including failures, and
is inherited by the child so killing only the wrapper cannot admit a second
render test while its game still runs. Never remove the lock file while tests
may be waiting on it.

The runtime forest implementation now selects shared full-detail prototypes
rather than generating a unique world-space mesh for every grove. Both tree
families retain three branch architectures, four prototype height/phase recipes,
source density classes, and the existing sparse-interior placement rule. Stable
source coordinates select the prototype, instance height, quarter-turn and a
small trunk offset within the blocked plot. Each placement replaces the shared
prototype's trunk offset, so repeated models do not repeat the planting grid.
National Park retains its centered trees and formal hedges. Ordinary Kanto
canopies use the same coordinate-seeded offset. Instance
translation and scale fit the authored plot. This deliberately replaces unique
per-placement branch geometry with a reusable modeled library. Bank-conforming
groves retain the original terrain-shaped path.

Each graphical app owns a bounded library of at most 144 prototypes. Instances
share GPU mesh handles and collision trees, permitting Bevy's normal instancing;
cutaway/fade state remains independent. The library survives regional rebuilds
and is released with its app, so GPU handles cannot cross asset stores. Cold
prototype construction runs outside the library lock, with one initialization
per key. Full-detail prototypes, rather than simplified crowns, are generated.

The streamer prepares beyond the camera/shadow footprint, retains a wider band
to avoid rebuilding trees during reversals, and prioritizes the camera rather
than the source-coordinate origin. Preloaded outer-band trees stay hidden until
needed, avoiding extra draw/shadow work. Native publication accepts at most two
completed grove tasks per frame. The production-source geometry control checks
6,912 placements against the original conservative source bounds; those placements
reuse 92 generated prototypes. Desktop/browser moving-scene performance and
visual acceptance for this redesign remain pending; these are geometry-control
results, not a claim that the playable build has been replaced.
The isolated instancing bundles pass actual Route 29 keyboard walking in both
Chromium backends. WebKit and Firefox each pass three full-detail house
exit/reentry cycles, including walking and the production Start menu, without
context loss, device loss or shader errors. Tracked WebGL buffers return to
about 17 MiB inside the house after each cycle. These checks validate that
regression path; they do not establish acceptable frame times or cold loading.
An isolated native walking CPU sample also found repeated hashing of complete
palette keys during subtile scanning. The scanner now resolves art once for each
source map with the exact palette/time key, then borrows those art references for
texture, animation and priority lookups. Eight renderer regressions pass,
including distinct palette/time animation families, vertical scrolling, retained
LCD pixels and map transitions. Four production-controller regressions, MCP
movement/menu input and clean stderr pass. The native example compiles with
voxel rendering enabled; a measured speedup for this lookup change is pending.

The WebGPU renderer also explicitly destroys mesh buffers absent from every
allocator slab for three submitted frames. Live and allocator-reserved slabs
remain usable; this avoids waiting for browser garbage collection to retire
obsolete allocations. WebGL retains its dedicated index-buffer path.
The full-source mesh builders retain every tree for reference/regression captures.

Native character preparation starts on the background pool while terrain builds.
Only requested portraits are decoded and resampled; the complete roster is
checked for missing files, and corrupt requested artwork fails explicitly.
Rig-independent hand/head preparation is shared by characters using the same
rig. Clothing remains independently fitted. Collision-tree construction caches
triangle bounds and centroids, then compresses its final indices. Groves outside
the conservative ledge region skip the unchanged ledge-shaping pass.

Paired full-detail native captures on October 14, 2026 reached the bedroom in
3.97 seconds and Azalea in 13.64 seconds. Matching-state 2D captures retained
identical RGBA bytes. These are verified capture timings, not a loading target
or a guarantee for another host. The browser investigation remains unfinished:
the latest measured outdoor full-readiness time was 31.38 seconds, with the
production Start menu responding during scenery loading. Browser release-profile
and native window-input verification are still pending.

For the prepared local client, `just play-3d` starts `PlayersHouse2F` in 3D and
skips the intro; `just trace-3d` records `.local/loading-trace.json`. A trace can
also be collected with the source-built native example:

```sh
CARGO_TARGET_DIR=target/desktop-3d-loading cargo build --locked -p crystal-bevy --features location-tester --example new_bark_3d
CRYSTAL_LOADING_TRACE="$PWD/.local/loading-trace.json" \
  ./target/desktop-3d-loading/debug/examples/new_bark_3d \
  content-packs/core-modular.browser.crystalpack --map PlayersHouse2F --x 3 --y 3
```

The local compact pack is compatible with this native renderer. The browser build
used for verification includes `location-tester,allocation-tester,fullscreen-scaling,browser-webgpu,crystal-voxel-view/scenery-full-detail` with the `wasm-dev` profile and the
`wasm32-unknown-unknown` target. Use a matching wasm-bindgen CLI when packaging it.
`CRYSTAL_WEBGPU=1 tools/new-bark-3d-web.sh` packages separate WebGPU and
WebGL builds. Bevy selects WebGL uniform padding and texture views at compile
time, so a dual-feature binary cannot serve as its own WebGL fallback. The
loader selects an available adapter; `renderer=webgl` forces the legacy bundle.
The modern bundle also probes its adapter in Rust before renderer construction.
GPU shader normalization remains in Rust; `gpu-shaders.js` only forwards browser
shader descriptors to that function.

The packaging helper also accepts `CRYSTAL_WEB_PROFILE=web-release` and defaults
to one Cargo build job. Combine it with `CRYSTAL_WEBGPU=1` and
`CRYSTAL_SCENERY_FULL_DETAIL=1` to package both full-detail backends using that
profile. The helper strips the debugging name section from generated Wasm.
The release candidate completed the full-detail Azalea browser test with no
graphics errors, but full terrain readiness still took 32.20 seconds versus
32.88 seconds for the iteration baseline. It remains unpublished and is not
accepted as a playable loading-time fix.

Branch and twig generation now compute each ring's repeated normal and pigment
once per limb. A control against the original production functions verifies
identical position, normal, color and UV bits and indices for 512 branches and
512 twigs. Whole-scene loading improvement from this change remains unverified.
The terrain-finishing waterfall also separates bank approaches, rock geometry,
cave crowns and palette, shore ecology, terraces and authored landmarks.

Johto pine visibility bounds now follow the generator's fitted crown margin
instead of extending two whole plot widths past each edge. They retain the
understory extent and extra bank-lip room; source geometry and the shadow-volume
test are unchanged. An independent production-source control checks containment of
287,400,744 vertices across 9,216 groves and both families, including narrow
plots and a bank-lip displacement control. The browser prepares 441 groves
instead of 513 and reaches the same final resident count of 405. Full readiness
still takes 30.43 seconds. One house exit/return cycle passes with no graphics
errors and byte-identical matching 2D captures; playable loading remains
unaccepted.

Rock-cap restoration now indexes triangle centroids by source cell instead of
filtering the complete scene's indices for every restored corner. It retains
the original strict height and cell-boundary tests, processes appended cap and
wall triangles in the same order, and releases the temporary index afterward.
All eight rock-contact regressions pass, including whole-mesh comparisons with
the original full scan. The browser full-scene measurement improves from
30.43 to 27.30 seconds, with no graphics errors or lost trace events. Rock
sculpting drops from 1.20 to 0.90 seconds. Loading remains unaccepted: the
worst first-frame work still blocks for 4.59 seconds.

Grove collision surfaces now retain exact positions and indices and construct
their triangle BVH only when a ray reaches the actual source bounds. The bounds
test only rejects distant rays; a nearby ray always uses the original triangle
intersection code, including concave openings. Construction releases the
temporary source buffers and reuses the resulting immutable tree. Ordinary
authored wall surfaces keep their eager construction. All 27 occluder regressions
and three shared-grove cache/retirement regressions pass, with zero ignored.
Before the instancing redesign, the WebGPU full-scene test reached readiness in 21.90 seconds versus 27.30
seconds for eager collision preparation. It builds 10 BVHs instead of 449.
Three house exit/return cycles pass with no graphics errors or device loss;
matching 2D captures remain byte-identical. Wasm capacity peaks at 1.63 GiB,
versus 1.83 GiB in the older three-cycle reference (different compiled profile).
The forest capture has a visible color difference from the prior build, so 3D
visual acceptance remains open. A same-binary `loading-collision=eager` control
and fade-count traces were added to distinguish algorithm effects from
changes between compiled assets. The later tree redesign requires fresh moving-scene
verification. Loading and publication remain unaccepted.

Append `loading-trace=on` to the browser URL. The generated bindings expose
`crystal_loading_trace_read()` for Chrome trace JSON and, in location-test builds,
`crystal_loading_memory_read()` for live/peak Rust allocations. That export and
the counting allocator require the explicit `allocation-tester` feature (or
`CRYSTAL_ALLOCATIONS=1` with the packaging script); ordinary previews omit it so
allocation counters do not distort performance. Trace spans cover
pack fetch/verification, artwork installation, regional preparation/publication,
tree prototype preparation, instance reuse/upload and every render frame's preparation, queue,
sort and submission stages. Captures are bounded at 131,072 events and expose
`droppedEvents`; a complete verification capture requires zero drops.
CPU render/submission spans are not GPU timestamps or displayed FPS.
Snapshots also include unfinished spans with `inProgress: true` and their
elapsed duration. Completion replaces that live span with its final event,
so a stalled regional operation remains visible before it finishes.

Allocation-test builds also accept `allocation-stacks=on`. They capture one
browser stack per large allocation layout, bounded to 64 layouts. For named
Wasm functions, build the final diagnostic binary with `cargo rustc` and
`-- -C strip=none`, then package with wasm-bindgen `--keep-debug`. This larger
artifact is for diagnosis; ordinary builds omit the counting allocator and
stack bridge. The browser harness checks actual WebGPU device loss,
uncaptured validation errors and shader compilation errors as well as WebGL
context loss.

The local `offset-allocator` 0.2.0 patch initializes node records on demand.
It retains the upstream allocation limit and freelist ordering rather than
reserving 128K records for every mesh slab. Upstream tests and a differential
run of 635,326 operations cover fragmentation, resets and metadata exhaustion.

Merged opaque objects keep their exact transparency-sorting meshes deferred
until the object first fades. Hidden sorting meshes no longer upload alongside
every opaque object. Fade activation publishes the original meshes before
render extraction, and later fades reuse them. The WebKit WebGL regression
passed three full-detail exit/reentry, movement and menu cycles with zero
context losses and pixel-identical returned 2D captures. Chromium WebGL and
the published port-3003 WebKit build also passed a cycle. To exercise that
backend, add `renderer=webgl` to the URL below and use `webkit` as the engine.

The source-built verification command is:

```sh
BROWSER_HEADLESS=0 node tools/browser-house-memory.mjs \
  'http://127.0.0.1:3003/?preview=house-door&multiplayer=off&loading-trace=on' \
  .local/house-memory.json chromium 3
```

Headed Chromium is necessary for hardware WebGPU measurements on this Mac;
the latest headless trial selected software WebGL. The command verifies gameplay,
full detail, actual rendering errors and context loss, and reports memory without
an arbitrary pass/fail budget. Three WebGPU cycles completed without shader
errors/context loss. Live allocations settle after later returns while the
non-shrinking WASM heap remains larger. Native paired captures can use
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image` when the desktop is locked.

The Chromium 156 buffer-retirement regression completed three full-detail
exit/reentry, movement and menu cycles without renderer errors or context loss,
and its 2D capture is pixel-identical. Tracked GPU-buffer peak fell from about
1,562 to 860 MiB compared with the preceding run. An additional ten-second idle
check settled indoor mesh slabs at 47.6 MiB. To include that ownership check, add `BROWSER_SETTLE_MS=10000` to the
command above. Reports include per-buffer labels and a final return to 2D.
These storage counts exclude driver overhead and are not whole-process memory.
