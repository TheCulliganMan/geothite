# 3D rendering performance and Goldenrod browser crash handoff

Original goal: improve 3D lighting and windblown leaves, coastal mist and cave
snow/frost, after establishing playable full-detail rendering. Keep haze removed
and preserve gameplay timing. **The full goal is not complete.** The local
browser crash has a verified fix; final playable performance, environmental
effects and the specific ChatGPT MCP-app panel remain open.

## Production changes prepared for coordinated commit

The black screen was a WebGL pipeline failure: the voxel material requested 15
fragment uniform buffers against the adapter limit of 12, and Bevy quit after
invalidating the pipeline. The five VoxelSurface values now share one uniform
binding with an identically ordered shader struct, reducing the total to 11.
Geometry, textures, water/cutaway values and gameplay timing are preserved.

Terrain retirement releases the old hierarchy and cache-owned meshes, materials
and atlases before replacement allocation, with two pending updates for asset
removal/render extraction. The regression checks actual asset removal as well
as hierarchy/cache release. This is CPU/ECS lifetime proof, not driver-release
proof.

Browser construction retains exact grove recipes and source-bank context,
prepares one complete grove per update, and stages its buffers immediately.
Pending groups retain handles, collision triangles, bounds and fade ownership.
Final preparation is shared with the native compute path. Map/profile changes
cancel pending work. There is no 2D fallback, model reduction or game-tick change.
The grove geometry/bank equivalence regression, all 18 current occluder tests,
and the department-store retirement regression passed.

The full-detail staged browser passed six department-store exits and six returns
in Edge 155/Radeon RX 7900 XTX/WebGL2 through production input, including further
outdoor movement, Start/B and camera turns. No renderer errors, page crashes or
context loss occurred. The 2026-10-13 repeat used WASM SHA-256
`5BB923873CB6AC122529132ABDA0EE33A128A9DF140F9EA406F7E1A7B324DBB5`.
Outdoor images were inspected. Memory was recorded rather than used as the crash
gate, per the latest user instruction; the original 512 MiB target is unmet.

After removing the experiments, the final production configuration rebuilt
with --locked and passed a fresh six exits/six returns with the stronger
destination-terrain check, outdoor movement, menus and camera turns. Final WASM
SHA-256: `7FF8B4541DE47BEB07C9E7F7F81C5FF034F9142F9AD99973BAC823F8F0479363`.
Errors/crashes/context losses were zero; all samples were muted. Heap high water
was 1,129,840,640 bytes and largest tracked buffer 18,626,404 bytes. Original
memory budgets remain unmet and are not claimed as passing.

Passive native paired captures hold the same checksum, LCD frame and actor list
across 2D/3D readbacks. Ordinary play and movement captures retain the real clock.
Native timing warmups now finish shader compilation before accepting geometry;
the prior hats-only warmup was invalid. See the qualified shader-complete
New Bark/Route 29 timings in docs/johto-3d-performance.md.

QA probes preserve styled Bevy errors, use muted production input, require real
movement, observe each walking leg and bound blocked-browser operations. Current
terrain readiness must match the destination map when exposed. Their measured
rAF intervals are not GPU timestamps or displayed FPS.

## Deferred investigations removed from the commit candidate

Unaccepted bind-group retention, shared floating-point metadata, browser shadow
filter and WebGPU experiments were removed, including their feature flags,
configuration and extra lockfile dependencies. Diagnostic build-stage tracing
was removed. Owned previews were stopped; scratch remains ignored because
automatic approval review rejected recursive cache deletion (blocked by policy). Native receiver-filter experiments
that predate this work retain their existing opt-in behavior.

Those experiments provided direction, not final acceptance: the metadata
candidate measured roughly 53 ms median walking intervals in New Bark/Route 29;
the actual receiver filter removed about 45% of draw calls without reaching the
performance gate. Strict same-state visual acceptance was still pending.

An experimental metadata bundle also passed six exits/returns in Codex IAB via
registered production WebMCP controls, with outdoor images and walking/Start/B
verified, muted controls and empty error queries. It passed two small-house
exits/returns with destination terrain ready. These checks do not certify the
final production bundle in IAB, nor the distinct ChatGPT MCP-app panel. The
MCP-app inventory had no active game tab. The disposable previews were closed.

Establish fresh playable full-detail performance on the final production build
before registering the unfinished environmental effects. Keep the full goal
open; neither crash stability nor removed experimental timings complete it.

## Checkpoint state

Haze was removed when no authored fog profile exists. Lighting preserves authored
geometry and uses warm directional/cool fill shading. Environment particle code
exists in `environment_fx.rs`, but its plugin is **not registered**: leaves,
coastal mist and cave snow/frost remain unfinished and unapproved.

Default optimizations include exact small-mesh U16 indices, cached presentation
checks, an indexed occluder BVH and settled material-write guards. Experimental
shadow receiver filtering, cascade fitting, GPU culling, compact UV removal and
native pass timing remain opt-in. Do not switch them all on as a quality preset.

The conservative receiver filter removes only shadow submissions that cannot
reach the camera's receivers. Its cache invalidates on exact caster world-corner
or receiver-plane/light/height/bias changes. Geometry, materials, shadow
projection/resolution and gameplay are unchanged. Five extracted geometry/cache
regressions passed. New Bark and Route 29 GPU captures from multiple angles retain
full model geometry and matching static pixels outside documented animation
rectangles. See `docs/johto-3d-performance.md` for qualified measurements.

Native image-target GPU readbacks and synthetic production-input walking work
with the real window unfocused. **No desktop unlock/focus is needed.** Ordinary
keyboard input still requires focus. Synthetic QA uses the production input-owner
dispatcher and checks authoritative tile changes; it does not change coordinates
or game ownership directly.

Timing defaults to the production low-power unfocused Winit policy. Opt-in
`CRYSTAL_RENDER_TEST_UPDATE_POLICY=focused` copies normal focused update behavior
without forging focus or modifying the fixed game tick. Validated native timing
also records queue-completion callbacks, submitted/completed interval counts,
outstanding work and poll errors. These are completed-work observations, not GPU
timestamps or display-presented FPS. Current callback/report functions compiled
against cached Bevy; the extracted policy and ECS receipt tests passed. Fresh full executable/GPU observations passed for New Bark and Route 29;
see the qualified shader-complete native results in `docs/johto-3d-performance.md`.

Latest implementation: map changes now release the old terrain hierarchy and
cache-held mesh/material/atlas references before the replacement build. Two
pending updates allow asset removal/render extraction before the synchronous
browser build starts. The unchanged models are rebuilt afterward. The regression
`department_store_exit_retires_old_terrain_before_allocating_goldenrod` verifies
hierarchy removal, cache release, and that no new build starts during those two
updates. Its full-crate run passed (1 test, 1,478 filtered out) with the
`wasm-dev` profile in an isolated target directory.
This corrects a lifetime overlap; it does **not** yet prove the reported crash is
fixed. Dense grove geometry across the 116x114 source window remains a likely
memory pressure contributor. Measure its build/allocation peak next.

## Verification and limits

Verified earlier in this task:

- `cargo test --locked -p geothite`: all 13 tests passed, including production
  MCP movement/Start and Mom's spawned-client byte-for-byte empty stderr.
- External-pack Bevy controller regressions passed for Mom/New Bark exit gating,
  and battle command/move selection with authoritative PP mutation/dialogue.
- Native location-tester build passed for the refined receiver filter/cache.
- `cargo check --locked --target wasm32-unknown-unknown -p crystal-bevy --features location-tester`
  passed after excluding the native synthetic-input hook from browser compilation,
  **before** the latest terrain-retirement edit.
- Full-crate department-store terrain-retirement regression passed.
- Five actual extracted receiver geometry/cache tests passed.
- Actual extracted queue-callback/report functions compiled; focused-policy and
  GPU receipt ECS tests passed. These do not replace full application verification.
- Full-detail GPU synthetic movement: New Bark down/up and Route 29 right/left
  and down/up changed authoritative tiles, moved presentation/grid origin and
  added no terrain build after the initial build.
- Frozen native-v7 alternate-angle New Bark baseline/filter: median 50.80/26.30
  ms, p95 166.24/125.67 ms, identical geometry. 1,125,650 static pixels matched.
- Route 29 alternate-angle filter/baseline: median 49.99/58.10 ms, p95
  85.67/97.72 ms; 1,184,450 static pixels matched. Earlier Route 29 pairs varied.

These timing runs were unfocused application-update traces under compiler/other
renderer contention, not focused-window FPS or GPU timestamp measurements.
GPU-culling/cascade-fit/compact-UV experiments did not establish reliable gains.
Metal did not provide the requested per-pass GPU timestamps. Do not infer GPU
shadow duration from CPU pass timings or overwritten diagnostic keys.

The old browser-review bundle was probed at a local department-store preview.
It predates the retirement change. A plain static server lacks `/v1/clock`, so
its startup reports a clock error; the probe's observations must not be called
successful production browser validation. Headless Chrome used ANGLE SwiftShader
software rendering. Production bridge presses moved from store (8,4) through
(8,7), warped into Goldenrod City (24,28), then moved to (24,29). WASM linear
memory rose from 680,329,216 bytes (649 MiB) to 1,010,630,656 bytes (964 MiB).
The probe completed without a page crash. This supports a memory-pressure
investigation but does not reproduce the reported embedded-ChatGPT crash. That old bundle did not establish an after-fix result; the fresh local
six-cycle browser result is documented above.

## Ownership and transfer

Owned source: renderer experiments (`shadow_receivers`, `shadow_bounds`,
`shadow_fit`, `gpu_culling`, `compact_vertices`, `render_metrics`, `environment_fx`),
`occluder_fade.rs`, native `render_walk`/`render_record`/`render_measure_validation`,
presentation polling changes in `player_customization`/`save_management`, the
small `handheld_input` visibility change, terrain-retirement hunks and regression
in `lib.rs`, and performance documentation.

Shared integration files: `crates/crystal-voxel-view/src/lib.rs` (registration,
atmosphere guards; the retirement/regression hunks are committed separately), `mesh.rs` (exact
index/occupancy changes), `voxel_surface.wesl` (directional color; other peers own
water work), voxel Cargo features, and `crates/crystal-bevy/src/bevy_shell.rs`
(native QA module integration). Preserve peer changes; the coordinating transfer
chat collects shared integrations after checkpoints. Do not stage entire shared
files as exclusively owned work. No authored model assets are owned by this task.

Find this checkpoint with:

```sh
git log --oneline -- docs/handoff/render-performance-and-goldenrod.md
```

Use `scripts/cold-start-3d.sh` and `docs/cold-start-3d.md` from the coordinating
chat. Transfer actual Git LFS objects, not only pointer files. Transfer ignored
content packs separately. Known local packs are
`content-packs/realtime-clock.browser.crystalpack` and
`content-packs/text-tui.crystalpack`; never commit or restore packs from Git.
Native external models use `CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models"`.
Browser bundles require their converted open-model/people assets and real Rust
server `/v1/clock`; a plain Python static server is insufficient for full startup.
No audio source, synthesis, content pack or PCM was changed by this task.

Ignored old-host evidence exists under
`~/.codex/artifacts/geothite-atmosphere-reference/` and
`~/.codex/artifacts/geothite-goldenrod-crash/`. Frozen executables and cloned Cargo
caches were temporary under the old host's `/var/folders/.../geothite-gpu-pair-*`.
These paths are evidence/scratch, **not portable deliverables**. Rebuild on the
new host. Do not commit any of them.

## Resume on the new host

1. Follow the cold-start guide, verify the external pack and LFS/open-model
   assets, then run the outstanding terrain regression and fresh target checks:

   ```sh
   cargo test --locked -p crystal-voxel-view department_store_exit_retires_old_terrain_before_allocating_goldenrod --lib
   cargo check --locked -p crystal-bevy --features location-tester
   cargo check --locked --target wasm32-unknown-unknown -p crystal-bevy --features location-tester
   cargo test --locked -p crystal-bevy gpu_receipt_counts_only_the_ready_interval_and_retains_outstanding_work --lib
   cargo test --locked -p crystal-bevy focused_policy_changes_only_the_opt_in_unfocused_update_mode --lib
   ```

2. Build/serve the browser through the cold-start script's documented commands.
   Start a disposable `?multiplayer=off&preview=department-store` location-test
   session at (8,4), enable 3D using the normal controls, and press down through
   the store exit using production keyboard/bridge input. Verify entry into
   Goldenrod City, continued movement and Start. Record WASM high-water memory,
   largest GPU allocation, context loss, page crashes and browser process memory.
   Do not use screenshots or coordinates alone to claim playable behavior.
   Compare the same assets/build before/after the lifetime fix. Repeated entry/
   exit and camera rotations must stay stable. Existing
   `tools/browser-voxel-memory.mjs` instruments WebGL uploads and heap size, but
   its New Bark fixture must be extended for this exact store transition.

3. If Goldenrod still exceeds memory limits, fix build/partition peak storage or
   stream exact full-detail geometry using proven camera/shadow coverage. Do not
   reduce model quality, foliage geometry, texture resolution or gameplay tick
   rate to pass. Current broad synchronous builds are not a final architecture.

4. Once the browser crash is resolved, complete same-executable full-detail
   New Bark/Route 29 performance checks. Native build/measurement example:

   ```sh
   cargo build --locked --profile wasm-dev -p crystal-bevy --features location-tester --example new_bark_3d
   CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" CRYSTAL_PREVIEW_MUTE=1 \
     CRYSTAL_SCENERY_FULL_DETAIL=1 CRYSTAL_SHADOW_RECEIVERS=on \
     CRYSTAL_MEASURE_3D_VALIDATION=on CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
     CRYSTAL_RENDER_TEST_UPDATE_POLICY=focused \
     ./target/wasm-dev/examples/new_bark_3d content-packs/realtime-clock.browser.crystalpack \
     --map NewBarkTown --orbit 0 --size 640x480 --measure /tmp/geothite-new-bark-focused --seconds 15
   ```

   Use a fresh empty output directory for each run. Repeat with receiver filtering
   off and Route29. Inspect `validated-warmup.png`, update-policy/readback receipts,
   `gpu-completions.txt` and real timing CSV. Native app-bundle launch was needed
   on the old macOS host; the disposable Rust `.app` launcher pointed at a frozen
   executable and set the capture flags, without requiring foreground focus.
   For movement QA use `CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic`, `--screenshot`
   and `--walk DU`/`RL`; inspect authoritative tiles in the movement CSV.

5. Establish a real playable performance result before enabling costly effects.
   Then finish source-anchored leaves/coastal mist/cave snow/frost and lighting
   against full-detail references. Keep haze removed. Validate particle placement,
   camera angles, cave surfaces and cost; the unregistered effects prototype is
   not an approved final visual result.
