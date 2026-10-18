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
env GPU_WATERFALL=1 \
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
The 120-prototype canopy audit preserves 7,144,800 source indices, checks that
every density class retains a full tree, and confirms all tree UVs are zero. Character finger shaping now uses conservative projected
triangle bins with source order preserved. A frozen-source comparison across
16 supplied-rig cases measured 5,974 ms versus 603 ms for that isolated step,
with bit-exact geometry, normals, UVs, colors, indices and skin data. This is
not a whole-game loading-time measurement. The targeted geometry, placement,
fade, character and shadow regressions pass (48 tests).

As of the user's October 14 directive, graphical QA runs concurrently across
chats without waiting for `tools/with-graphics-lock.py`. Capture timelines even
under contention and record concurrent workloads; those measurements do not
establish isolated performance. Keep test commands alive until their browser
closes, and leave other chats' windows and processes alone. The optional lock
wrapper remains available for an explicitly requested isolated comparison.

Native play now selects Bevy's supported CPU clustering backend, retaining
point/spot/rect lights, reflection/irradiance probes, decals and device limits.
The same frozen native binary measured GPU scene envelopes of
38.875 / 35.578 / 38.041 ms for GPU / CPU / GPU clustering, using 30 warmed
3D cycles per run on Apple M5 under shared graphics contention. All five GPU
clustering passes disappeared; significant image differences were confined to
animated characters. These are GPU scene envelopes, not displayed FPS or
isolated CPU assignment measurements. `CRYSTAL_CLUSTERING=gpu` restores the
GPU backend for comparison. Shadow HZB remains an opt-in experiment because
its three controls did not establish a net gain.

The prepared local native client was updated and launched with `just play-3d`.
The October 15 encounter-guard update is installed for subsequent launches;
the existing game window was left running. The binary and both `just play-3d`
and `just trace-3d` default to CPU clustering. A production-input Route 29 Cut interaction now displays
the authored notice in both 2D and 3D without crashing; controller story,
battle/PP, MCP movement/menu and clean-stdio regressions pass with the external
TUI pack. The first capture route walked past the Cut tile; only the corrected
capture ending at (46, 10), facing left, proves the graphical interaction.
The guarded native executable also completes twelve alternating Ilex Forest
walking legs from (1, 37), with an empty party and no Repel, without the former
invalid-battle crash. Paired captures preserve authoritative state. Walking
encounters require a usable party before committing a battle; real controller
tests cover empty, fainted and healthy parties, including a valid wild battle.
The native bedroom route `--walk S` exercises Select and its registered-item
notice, not Start; Start-menu coverage comes from the controller regression and
browser input tests.

The prepared browser copy on port 3003 now includes the matching WebGPU/WebGL
mains and terrain/appearance workers, together with the published tree-height
variation. All 50 production files were verified over HTTP after an atomic
directory exchange; the existing clock-derived pack and model inputs retained
their hashes. A fresh default WebGPU session then passed a house exit/return,
walking, Start and return to 2D with successful worker jobs and no fallback or
graphics errors. Prior height-v14 renderer directories remain available for
already-open pages; the new renderer configuration selects the worker build.

On macOS, the runner now retains the existing primary window handle holder
outside `app.run()` and releases it after rendering has shut down. This avoids
the observed main/render-thread wait when the renderer would otherwise destroy
the last native window owner during exit. The former moving 20-second
reproducer exited normally three times without forced cleanup. Ilex, the Cut
notice and the bedroom Select notice also passed paired 2D/3D checks and normal
exit. The fixed binary is installed for subsequent `just play-3d` launches;
an already-running game keeps its current executable until relaunched.

Direct native drawing remains an opt-in comparison using
`CRYSTAL_NATIVE_DRAWING=direct`. It removes indirect-parameter-building encoder
labels, but matched stationary A/B/A mean update intervals were
42.43 / 43.75 / 72.54 ms under changing contention, with no reliable whole-frame
gain. Standard indirect drawing remains the default. A running-thread CPU
sample identified buffer uploads and Metal staging allocations as significant;
grass material synchronization was minor in that sample. CPU stage envelopes
alone do not establish the time spent in either operation or exclude waits.

The first unrestricted 30-frame stationary depth-pass experiment passed GPU,
shader and device-loss checks in all three controls. Depth-off GPU spans were
50.97 ms and 28.29 ms; depth-on was 89.21 ms, including a 7.30 ms early depth
pass. External contention changed substantially between controls, so this does
not establish an isolated regression or gain. The experiment remains off by
default. Its paired screenshots retain 99.737% exact pixels, with 0.067% of
pixels changing by more than eight channel levels. The separate receiver
pruning experiment reduced caster candidates from 180 to 165 but showed no
GPU timing benefit in this sample; it also remains opt-in.

Runtime trees now defer hidden sorted fading children and transparent material
allocation until their first fade. The first-fade regression covers scroll,
rotation, scale, retention-first operation, populated default material assets,
reuse and map teardown. All 33 fade tests pass. Terrain preparation fuses three
coverage scans while retaining independent validation and sorted AABB results;
all 19 terrain tests pass. Isolated 73,728-triangle coverage measurements were
4.163 to 3.588 ms for opaque input and 3.763 to 3.437 ms for mixed grass input.
These are preparation microbenchmarks, not whole-game loading claims. Opt-in
nested schedule envelopes now distinguish transform propagation, visibility
propagation, camera visibility and light visibility work; all three trace tests
pass. Envelopes may include interleaved independent work and are not additive.

The same-bundle eager/lazy/eager walking controls measured CPU main spans of
18.72 / 16.14 / 20.77 ms under contention. The lazy sample retained 903 leaf
entities versus 5,479 in the eager control, deferring 4,576 sorted leaves.
Transform propagation was 3.15 / 2.21 / 4.44 ms and camera visibility checks
0.85 / 0.51 / 0.96 ms. All three captures passed shader/device checks and real
keyboard movement. Paths and concurrent workloads varied, so these are
observations rather than an isolated percentage or universal FPS claim.

Pack loaders now retain private immutable verification provenance. Successful
file and browser-byte loaders certify only after every runtime integrity check;
runtime construction skips the duplicate check for that certified wrapper.
Uncertified diagnostic inputs still receive full verification. Four loader
regressions pass, including valid CBOR with corrupt semantic metadata. An
eight-case alternating native control on the canonical 388-map browser pack
measured runtime construction medians of 1,443.7 ms with duplicate verification
and 241.7 ms with certification reuse. These unoptimized, contended native
measurements do not establish browser savings or compare directly to browser
profiles and capture workloads.

Native AArch64 builds enable the SHA-256 hardware backend. An alternating
legacy/accelerated/legacy control hashed the actual 581,973,763-byte native
pack three times per variant: median SHA times were 1,221 / 216 / 1,238 ms.
All nine digests matched exactly, and each pass also checked the existing frame
checksum. This measures the hashing stage rather than total game startup;
pack decoding, runtime construction, and integrity checks remain required.
Browser targets retain their existing backend.

Native task pools allocate one I/O worker, approximately half the available CPU
threads to asynchronous preparation, and the remainder to frame computation.
On a 10-core host that policy assigns 1 / 5 / 4 workers, versus the previous
3 / 3 / 4. Tree work reads the actual initialized asynchronous pool size and
keeps publication bounded. A fresh native Bevy process confirmed the actual
1 / 5 / 4 pool assignment. Actor warmup now initializes the shared rig catalog
once, then prepares independent character bases and their paintings on the
asynchronous pool, with at most one active job per loading worker.
Final CPU appearance work (skinning attributes, accessories, and mounted
bounds) now runs there too, using app-owned staging consumed by the existing
GPU publisher. Actual worker/direct comparisons preserve exact mesh bytes,
index widths, bounds, and painting data; first-pose and lifecycle regressions
also pass. A contended same-binary Route 29 control measured the worst main
character-update call at 924 ms with appearance warmup disabled, versus
1.4–4.5 ms enabled. `CRYSTAL_ACTOR_APPEARANCE_WARMUP=off` retains that control.
This measures a loading stall, not steady-state FPS or browser performance.

Native painted terrain images decode on the existing asynchronous pool while
terrain geometry builds. Publication waits for both exact images, moves them
into the existing asset cache once, and keeps no extra decoded copy. A
contended Route 29 off/on/off control measured terrain publication at
59.5 / 15.9 / 52.0 ms, with matching scenery in paired 2D/3D captures. Exact
bytes, image format, sampler, cancellation, atomic publication, and map-handle
reuse tests pass. `CRYSTAL_PAINTED_IMAGE_WARMUP=off` retains the control;
browser image decoding is unchanged.

Native water masks and shoreline images also prepare inside the existing
terrain job from its captured frame. Palette sampling and live phase uniforms
stay on main; stale terrain results drop their owned images. With PNG warmup
enabled, a same-binary water off/on/off control measured publication at
14.5 / 0.46 / 15.3 ms. The paired 2D control was pixel-identical; 3D differences
were confined to animated characters. Exact image/phase, stale-publication,
and all 22 water/shoreline regressions pass. Set
`CRYSTAL_WATER_IMAGE_WARMUP=off` for the original publication path. Browser
water-image publication remains unchanged.

Native terrain preparation also converts independent spatial batches on that
existing pool. Domain classification and batch ordering remain deterministic;
parallel conversion preserves mesh attribute bytes, index widths, bounds, and
all three collision coverage variants. Small jobs stay serial. Exact-output
tests include a parent terrain task running on a single-worker pool to check
that nested preparation cannot starve its children. Set
`CRYSTAL_TERRAIN_BATCH_PARALLEL=off` for a same-binary serial control. A
contended native Route 29 serial/parallel/serial control measured total mesh
preparation at 467.5 / 368.8 / 475.8 ms, with the solid-surface stage at
391.3 / 291.0 / 395.4 ms. Parallel conversion used five workers. Paired
production 2D/3D captures retained scenery, with image differences confined to
animated characters. These are preparation timings, not whole-startup or FPS
measurements.

Native spatial partitioning caches each triangle's bucket in one byte rather
than repeating its f64 centroid calculation. The temporary cache is released
before compact vertex buffers are allocated (about 2.37 MiB for Route 29).
An additional contended same-binary off/on/off control measured partitioning at
246.5 / 195.1 / 239.1 ms. Exact-output tests cover boundary, negative, reflected,
and extreme coordinates. `CRYSTAL_TERRAIN_BUCKET_CACHE=off` retains the original
calculation control; browsers retain their existing memory policy.

The browser release profile optimizes `crystal-assets`, `ciborium`,
`ciborium-ll`, and `sha2` at level 3. A production-loader Wasm probe using the
same default size optimization and these four overrides measured a median
863.05 → 472.30 ms in three paired ABBA rounds after warmup. CBOR decoding
measured 524.60 → 259.35 ms and identity hashing 263.95 → 136.40 ms. All cases
returned the same certified pack identity and rejected corruption. The small
probe grew by 712,854 bytes; this is not a measured game-bundle size increase
or whole-startup improvement. The shared host was under memory pressure.
An earlier probe optimized every crate and is not the shipping-profile control.

The browser packager now enables separate Rust terrain and appearance workers
and packages both WebGPU and its matching WebGL fallback by default. The game
controller and GPU publication remain on the main thread. Main and worker CPU
kernel identities must match; failed or incompatible workers retain the direct
loading fallback. `CRYSTAL_CPU_WORKERS=0` builds without either worker;
`CRYSTAL_TERRAIN_WORKER=0` and `CRYSTAL_ACTOR_WORKER=0` select them individually.
For a full-detail direct control, also set `CRYSTAL_SCENERY_FULL_DETAIL=1`.
The runtime URL controls `terrainWorker=0` and `actorWorker=0` remain available.

Workers start on first 3D use. A genuine never-3D Continue session downloaded
and initialized neither worker, submitted no jobs, and retained the same main
Wasm capacity as its explicit opt-out control. It keeps the original validated
model strings until activation: about 29.14 million characters, or approximately
31 MB of additional main JavaScript heap in Chromium. That is a bounded memory
tradeoff, not a claim of lower total 2D memory. Movement and input checks passed.

Earlier full-detail WebGPU worker tests covered New Bark, Route 29, Ilex Forest,
and Dark Cave with full 116 by 114 regional terrain, real walking, production
Start menus, 30-frame GPU captures, and same-state 2D/3D comparisons. Eight house cycles
(16 warps) and the final return to 2D passed without worker fallback or rendering
errors. Late-cycle combined Wasm capacity stayed near 1.224 GB. Route 29 used
about 919 MB in the main Wasm and 2.00 GB across main and both workers, essentially
unchanged by moving mesh allocations into the wire format. These capacities
exclude JavaScript, browser and driver memory. WebKit also passed an actual
WebGL house exit/return and final 2D switch with both workers.

After integrating the published ordinary-tree height variation, both workers
and both renderer variants were rebuilt from one frozen source generation.
Route 29 at `(50, 9)`, facing down at preview hour 16, retained the same
authoritative paired state and tree silhouettes as the published height build.
The candidate retained full detail; the published reference reported reduced
detail, so this is not a global pixel-identity comparison. The new generation
also passed the original empty-party Ilex walk on WebGL and WebGPU, two WebKit
house exit/return cycles, and genuine never-3D 2D with no worker downloads,
initializations or jobs. Route 29 combined Wasm capacity remained about 2.002 GB.
Its heavily contended stationary trial recorded an 83 ms median frame interval
and 18.99 seconds to full terrain readiness; host load was about 22, versus
about 6 in the earlier walking trial. These different trials do not establish
a frame-rate or full-readiness improvement.

Paged restoration uses a soft 4 ms cooperative budget. Actual callbacks can
exceed it: headers of 7–11 ms and one 16.2 ms advance were recorded. Cold pack
verification and image publication still cause main-thread long tasks. One
contended New Bark trial responded to Start at 3.83 seconds and completed full
terrain at 9.28 seconds; these results do not establish faster whole startup
or full terrain readiness. Foreground trees can still obscure parts of Ilex's
view; the worker changes retain existing geometry and composition.

An earlier, historical terrain-only experiment moved 1.9 seconds of work off
the main thread and responded to controls at 3.54 rather than 6.57 seconds,
but full terrain readiness was slower: 9.11 rather than 8.17 seconds. That
experiment supported further development, not a claim that loading was fixed.
The current guarded builds also pass the original empty-party Ilex walking
regression without adding a starter or Repel; healthy-party wild battles remain
covered by the production controller tests.

Exact vertex welding of merged opaque tree meshes retains triangle order and
every vertex attribute. The current 120-prototype audit removes 14.83% of
prepared vertices and about 13.25 MB without reducing tree detail. A same-module
30-frame WebGPU legacy/weld/legacy control measured GPU spans of
38.66 / 36.65 / 39.06 ms under contention. Scenery captures matched; these
samples do not establish an isolated FPS improvement. Three subsequent house
exit/return cycles passed real movement and menu checks with no GPU errors or
device loss. Wasm capacity remained at 1,061,158,912 bytes; live GPU buffers
returned to 134,020,965 bytes in the final indoor sample. Loading and frame
times still need improvement.

The runtime forest implementation now selects shared full-detail prototypes
rather than generating a unique world-space mesh for every grove. Both tree
families retain three branch architectures, four prototype height/phase recipes,
and source density classes. Every authored tree remains present, including
dense interiors. Forest’s complete 4×4 canopy drawing retains an ancient tree;
its separate 2×2 Headbutt drawing retains a small pine at the authored location,
without an added ground colony. Stable source coordinates select the prototype, instance height, quarter-turn and a
trunk offset of up to 32% of the plot width on each axis. Fitting keeps the
rooted flare inside the blocked plot. Each placement replaces the shared
prototype's trunk offset, so repeated models do not repeat the planting grid.
Ordinary Johto and Kanto trees have a stable per-placement height factor
from 0.6 to 1.4 of their source recipe. Shared pine instances replace their
prototype height with the target height, so variation is applied once.
Ancient trees retain their existing height variation, with their complete
canopy and bole reduced vertically by 30% around the source footing.
National Park retains its centered trees and formal hedges. Ordinary Kanto
canopies use the same coordinate-seeded offset. Instance
translation and scale fit the authored plot. This deliberately replaces unique
per-placement branch geometry with a reusable modeled library. Bank-conforming
groves retain the original terrain-shaped path.

Each graphical app owns a bounded library of at most 120 canopy prototypes and
12 ground-cover prototypes. Ancient canopies retain their source roots and broad
crowns at 70% of their previous height; small Headbutt drawings keep their separate scale.
Only in Ilex Forest, at corners joining at least three ancient canopy plots,
large divided ferns, flowers, loose broadleaf plants, and fallen leaves span
all qualifying authored plots. Divided fern fronds meet at shared edge midpoints so growth connects
between trees while leaving open ground and irregular borders. Small trees do not qualify a junction. Other maps, including National Park,
do not receive the new underbrush.
Foliage pigment varies subtly within crowns, fern fronds, and grass blades.
Overlapping local color fields preserve lighting and avoid uniform tile tints.
Ground cover has its own
placement and scale: crown fitting, tree height and trunk offsets never shrink
or move it. Its leaves stay inside blocked plots; large ferns rise 1.7–2.1 tiles, below a 2.25-tile ceiling.
Instances
share GPU mesh handles and collision trees, permitting Bevy's normal instancing;
cutaway/fade state remains independent. The library survives regional rebuilds
and is released with its app, so GPU handles cannot cross asset stores. Cold
prototype construction runs outside the library lock, with one initialization
per key. Full-detail prototypes, rather than simplified crowns, are generated.

The streamer prepares beyond the camera/shadow footprint, retains a wider band
to avoid rebuilding trees during reversals, and prioritizes the camera rather
than the source-coordinate origin. Preloaded outer-band trees stay hidden until
needed, avoiding extra draw/shadow work. Native publication accepts at most two
completed cold grove tasks per frame. Already-cached shared placements now
bypass that cap in camera order, within a 4 ms CPU preparation/publication budget
and a 64-placement ceiling. Deferred GPU work is outside that CPU budget and
needs native frame measurement. The production-source geometry control checks
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

The local compact pack is compatible with this native renderer. The current
browser verification uses the `web-release` profile, the
`wasm32-unknown-unknown` target, full-detail scenery, and the production renderer
with `location-tester`, `fullscreen-scaling`, and both worker feature hooks.
Packaging requires wasm-bindgen CLI 0.2.128; `WASM_BINDGEN` selects its executable.
`tools/new-bark-3d-web.sh` now packages separate WebGPU and WebGL builds by
default. Set `CRYSTAL_WEBGPU=0` to build only WebGL. Bevy selects WebGL uniform
padding and texture views at compile time, so a dual-feature binary cannot
serve as its own WebGL fallback. The loader selects an available adapter;
`renderer=webgl` forces the legacy bundle. The modern bundle also probes its
adapter in Rust before renderer construction. GPU shader normalization remains
in Rust; `gpu-shaders.js` only forwards browser shader descriptors.

The helper defaults to the `wasm-dev` iteration profile and one Cargo build
job. Set `CRYSTAL_WEB_PROFILE=web-release` for the profile used in the current
worker verification. Enabled workers require full-detail scenery; the helper
includes it in both main backends and worker builds. It strips the debugging
name section from generated Wasm. A historical Azalea release experiment took
32.20 seconds to finish terrain versus 32.88 seconds for its iteration baseline;
that result was not accepted as a loading-time improvement.

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
The outer `render.submit` span includes Bevy's whole render system, including
graph execution, final readback submission and presentation. Nested
`render.graph_begin`, `render.graph_encode`, `render.graph_submit` and
`render.graph_finish` spans distinguish the public render-graph schedule stages.
They are CPU envelopes inside `render.graph` and `render.submit`; do not add
their durations to either parent or interpret them as GPU execution time.
Graph boundaries require tracing enabled before app creation (`loading-trace=on`
or `CRYSTAL_LOADING_TRACE`). Their diagnostic ordering barriers are absent from
ordinary play; trace timings therefore include that diagnostic overhead.
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
`CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`
without creating an OS window or changing focus, including when the desktop is
locked. Native one-shot screenshots default to this offscreen mode. Automated
checks must use it; visible/live review requires an explicit user request.

The Chromium 156 buffer-retirement regression completed three full-detail
exit/reentry, movement and menu cycles without renderer errors or context loss,
and its 2D capture is pixel-identical. Tracked GPU-buffer peak fell from about
1,562 to 860 MiB compared with the preceding run. An additional ten-second idle
check settled indoor mesh slabs at 47.6 MiB. To include that ownership check, add `BROWSER_SETTLE_MS=10000` to the
command above. Reports include per-buffer labels and a final return to 2D.
These storage counts exclude driver overhead and are not whole-process memory.
