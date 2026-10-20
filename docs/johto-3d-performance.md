# Connected Johto presentation and performance checks

## Retina release baseline and entity-cache CPU work

A separate release QA executable was built successfully in 4m08s, preserving
the installed production executable (which has no location-tester feature).
The immutable baseline SHA256 is
`c4bbf223560e1049d2bd96366ece66295f649a72eaa9187b9c1152bff8add19a`.
Quiet 15-second runs at 2560x1600 physical pixels, noon, full native quality:

| Scene | Update median / p95 | GPU queue completions/s |
| --- | --- | --- |
| New Bark (13,5) | 32.42 / 34.74 ms | 30.783 |
| Route29 default valid position | 36.31 / 38.31 ms | 27.279 |
| Roof (9,13), low-horizon camera | 85.46 / 99.86 ms | 11.473 |

All three passed existing 3D validation, with no readback during the interval
and no polling errors. Queue outstanding counts stayed at 36. These are
headless render/update observations, not display-presented FPS or GPU pass
timestamps. They establish that broader Retina performance is still inadequate.

The live CPU sample identified repeated randomized SipHash work on Entity IDs
in occluder_fade's per-frame visibility cache. Two type declarations now use
Bevy's EntityHashMap instead. Membership/visibility logic is unchanged. Shared
source checking passed in 9.52s; all 55 existing occluder/visibility regressions
passed, with no ignored tests. A separate release candidate built in 5m18s;
SHA256 `9007a3381b9e010d0b755346da0ba5d1b99b2de4e4fd8c380d3396f732485b48`.
Retina New Bark ABBA runs passed: baseline queue rates 31.242/30.803 per
second, entity-map candidate 31.544/31.182. Means 31.0225/31.363 differ by about
1.1%, too small to claim a meaningful frame-rate gain. It is a narrow CPU
cleanup, not the GPU solution; the installed app still has the prior type.

The first 10-second Metal recording was stopped by an overly short harness
finalization deadline. Export failed with Document Missing Template Error, so
it supplies no GPU attribution evidence. A shorter two-second recording used
no external finalization deadline. Instrumented runs are diagnostic,
not quiet FPS comparisons. The shorter capture completed and saved normally,
and its table exports succeeded. It contains 14,251 GPU interval rows, 2,285
encoder rows and 37,412 object-label rows. Process filtering is necessary:
Battle.net Helper also performs substantial GPU work. Earlier "quiet" runs
mean no competing own native/compiler job, not exclusive GPU access. Permission
to quit Battle.net was requested and is pending; it has not been closed.
Target-only labeled GPU work includes the shadow cascades, main opaque pass,
normal/depth prepass and SSAO. Vertex and fragment engine intervals overlap,
and many queued encoders lack labels; raw interval sums are not percentages of
frame time.

A private native culling probe now admits the roof and accepts the existing
NormalPrepass as genuine depth support, without requiring the separate tree-
depth diagnostic. Bevy's required DepthPrepass is still installed and owned
components are removed on unsupported maps. Existing diagnostics activate the
probe; no new runtime toggle was added, and no culling change is installed or
published. All ten component/lifecycle tests passed and its release build passed in
5m56s. Native camera-culling trials passed content validation but did not improve
throughput: town 30.949 queue completions/s (entity-map direct mean 31.363),
roof 10.160 (direct baseline 11.473), with roof median/p95 94.30/127.15 ms.
The private source was restored; no culling change was published or installed.
Shadow-only release/Retina diagnostics were also rejected. The stock-clamp
trial produced 315 updates, median/p95 46.62/58.08 ms, but its conservative
near-clamp guard activated zero cascades: that is fallback overhead, not an
active-culling result. The existing verified orthographic correction then
activated both cascades, certified all five skinned candidates and passed the
visible screenshot check. It produced only 96 updates over 15.04 seconds,
median/p95 126.81/374.30 ms. Its screenshot was visually inspected; throughput
is unacceptable regardless. These diagnostic runs remain subject to concurrent
Battle.net GPU activity. Neither camera nor shadow experiment is promoted to
production; no new product switch or profiler/timing implementation was added.

## Current verification load

The shadow-cache release build finished successfully in 61m11s. System inspection found multiple
unrelated Rust builds and approximately 19 GB of swap in use. The user declined
temporarily suspending other builds, so none are suspended or terminated.
The private application correction release build completed successfully. Preserve its saved executable; contended timings
must not be presented as an uncontended frame-rate improvement. This load is
not evidence of a game-source failure or a reason to restart a live compiler.

## Further CPU/GPU candidates under verification

A private shadow-receiver cache candidate compares the exact source Aabb and
GlobalTransform before recomputing a caster's eight transformed corners. The
receiver-plane/sun/height invalidation and stale-entity cleanup remain intact;
changed bounds/transforms recompute the original proof. The per-entity cache
also uses EntityHashMap. All ten receiver regressions, including transform-change coverage, passed
with zero ignored tests after a 13m51s contended build. The source change is
published to the shared working tree. Its separate release QA build passed
in 61m11s (log: `/tmp/geothite-shadow-input-cache-release-build.log`). The
immutable executable is `retina-release-shadow-input-cache-client`, SHA256
`98f90394dcf0605f01594e3b8750774c61cd0b08c9f58718e42290b2fa04c579`. It is
not installed or a verified frame-rate improvement.

Local Bevy 0.20 WgpuSettings::default source keeps VALIDATION_INDIRECT_CALL
when the requested backends include DX12. The native default is Backends::all,
so a Metal-selected renderer retains the extra validation passes. Bevy's own
release path removes this flag when DX12 is excluded. The valid Metal trace
contains indirect-draw validation encoders. No registry source was modified.
An existing release QA executable with WGPU_BACKEND=metal passed native scene
validation: 283 updates/15.01s, median/p95 47.57/110.29 ms. The immediately
following default-path run passed too: 236 updates/15.02s, 59.16/146.50 ms.
Multiple unrelated rustc jobs were active throughout, in addition to previously
observed Battle.net GPU activity. These contended sequential observations do
not establish a production gain; quiet repeated comparisons are still required.
The original candidate executable's existing corrected shadow bridge was also
checked with Metal-only selection: both cascades activated, all five skin
candidates certified, scene validation passed, but only 229 updates/15.13s
(median/p95 54.88/141.52 ms). This does not establish a culling benefit over
the direct Metal run, and remains contended. No culling source is reinstated.
No startup/backend change has been published or installed. Official Bevy
[0.19 rendering notes](https://bevy.org/news/bevy-0-19/) also describe Metal's
partial bindless support, but adapting this game's uniform-heavy material
extension would require a separate implementation and validation.

## GPU follow-up: instance preparation versus raster work

The successful Metal trace was rejoined using the encoder label (encoder-table
column 8), not the enclosing command-buffer label (column 6). Target-process,
Active, depth-zero intervals attributable to indirect-draw validation total
2.363 ms on Compute across 200 split intervals, plus 0.246 ms on Fragment.
Early mesh preprocessing totals 3.748 ms Compute; early shadow indirect-argument
building 3.244 ms; main indirect arguments 1.910 ms. These are split/overlapping
interval sums within a two-second instrumented capture, with some queued
encoders still unlabelled, not full-frame percentages. They do not support
attributing the main slowdown to these passes. A one-setting private CPU
instance-builder comparison was prepared but not applied; no production
preparation/backend setting has changed. Vertex work in the opaque and shadow
passes remains the stronger lead. Runtime foliage currently merges each full
opaque tree into one bounding box, while retaining up to 16 finer fade leaves;
full-detail opaque partitioning may improve culling granularity without LOD.
A private four-section prototype is now prepared outside the frozen build
source, not applied or compiled. It sorts existing fade-leaf references along
their largest center extent and merges four opaque sections, keeping source
collision/fade leaves intact. Sharing, upload, normal compression, exact vertex
welding and draw publication include every section. The worker wire stays
unchanged (sections are native-only and explicitly rejected by that serializer).
Its new test compares multisets of per-triangle hashes of every indexed vertex
attribute before and after partitioning, then repeats against the packed,
welded and reordered GPU copies for pine, ancient and deciduous trees. It also
checks unchanged collision/fade ownership and shared section handles. Syntax
checking passed; type checking, all tests and actual render comparisons remain.
Prepared files are `candidate-files/{occluder,shared-groves,gpu-culling,occluder-wire}-opaque-sections-probe.rs`
under the ignored grass-material-check workspace. The optional camera comparison
accepts the existing NormalPrepass and the roof through the existing QA switch;
no new product switch or culling change is published. The release-cache baseline
must finish and be copied before these files replace any frozen build inputs.
This remains an unverified hypothesis, not a measured improvement.

## Upstream culling edge fix identified

[Bevy PR #26170](https://github.com/bevyengine/bevy/pull/26170/files)
merged October 20. It clamps depth-pyramid reads before unsigned conversion,
preventing out-of-range samples from forcing false visibility near the right
and bottom edges. The local pinned Bevy 0.20 sources still contain the old
unclamped code in both affected shaders. Local SHA256 values:

- `bevy_pbr/src/render/occlusion_culling.wesl`:
  `296459c7e90f4ed8c72fbce1d8316eba3bc52fa0b9d8aad5fdb253d64cc68489`
- `bevy_core_pipeline/src/mip_generation/experimental/downsample_depth.wesl`:
  `b6af6beaee45da0968df79aac22b72588c77cdcdd52c558af81ed89e8c009f34`

The downstream culling comparison should account for that fix before treating
the previous negative trial as final. The current normal-prepass camera uses
MSAA off, so its occlusion-depth sampling correction is relevant even without
the MSAA downsample branch. No registry source, pinned dependency, product
shader or current build input was modified. A local adoption still needs
source verification, conservative edge tests and actual Metal image/performance
checks; no speedup is established by the upstream report.

An app-owned Rust patch is prepared outside frozen build inputs as
`candidate-files/edge-clamp-probe.rs`. It verifies the two exact stock digests,
changes one occlusion quad and two source-load clauses in memory, and preserves
all imports/paths/definitions. Asset events trigger revalidation; steady frames
skip hashing. The prepared culling integration waits for both sources to verify
and removes its owned culling state if verification is lost. Two exact pure
Rust tests passed in a standalone harness (the replacement function and f32
edge/one-pixel-mip coordinate coverage). Full-source matching and reversible
replacement checks passed against both installed files without modifying them.
Corrected digests are `318b8ed74b72d2446296ce989692b0f0f09e0c7c55b7849e6ae8fc2275530b7f`
and `b855ee7997b4be24d620191d735de106ab6009923786f8d7308c79c38ab4f128`.
The prepared helper also passed metadata type checking against the exact Bevy
and SHA2 libraries used by the live release build. A small isolated harness then
ran seven tests successfully against the cached dependencies from the passing
renderer-test fingerprint: the two source/coordinate checks and five actual
Bevy ECS culling/ownership guards, including verification loss and an existing
normal prepass. The harness substitutes the unrelated shadow bridge registration
hook, camera/sun marker types and trace sink; it does not prove that the full
renderer or GPU shader pipeline works. Its first direct release-library linking
attempts failed because of abort/unwind strategy and Apple/Rust LLVM-bitcode
mismatch; using the existing non-LTO test-profile libraries resolved the harness
linking. The real release Cargo build was untouched by these harness attempts and later
completed successfully, as recorded in the full-tree trial section.
An eighth isolated test then passed using both actual installed shader files
in a MinimalPlugins/AssetPlugin Bevy App, without a window or GPU. It starts
with no sources, adds them individually, verifies the exact corrected content,
observes 64 steady frames with zero Modified events, rejects an unknown reload,
recovers from stock reload, rejects duplicate target paths, recovers after
removing the duplicate, and becomes unverified when a source is removed.
All eight isolated tests passed (zero ignored); the fixture's cache paths are
local to the temporary verification harness, not project code or shipped data.
Full-crate partition compilation, GPU shader compilation and Metal comparisons
remain pending. The correction is now applied only to the private application trial described
below; it is not installed or integrated into the shared voxel renderer.

## Native opaque tree sections under verification

The installed Bevy 0.20 source was also checked for its MeshletPlugin path.
It supports Metal and the adapter exposes its required atomic/subgroup features,
but MeshletMesh::from_mesh requires exactly POSITION/NORMAL/UV_0, omits the
trees' vertex colors, quantizes positions and constructs simplified LODs. Using
that stock conversion would violate this task's geometry/color preservation
requirements. No meshlet feature, converter, dependency or asset was added.


Following the corrected whole-tree culling run, the private snapshot now trials
up to four spatial opaque sections per source tree. Every source triangle is
retained; the original collision surface and sorted fade leaves remain shared.
Preparation, normal packing, index localization, upload and weak cache reuse
include all sections. Browser/terrain-worker preparation remains unchanged;
the worker wire rejects native sections rather than silently dropping them.
The private voxel regression build passed in 18m45s; log:
`/tmp/geothite-opaque-sections-tests.log`. Its new test passed per-triangle
hash comparisons of every vertex attribute, including actual packed/localized
copies, across pine, ancient forest and deciduous trees. All 56 occluder/visibility
tests and all three shared-grove cache tests passed, with no ignored tests.
The private release example build passed in 11m22s; log:
`/tmp/geothite-opaque-sections-release-build.log`. Its saved executable SHA256 is
`e23a2949820e7572e681abc00058dfce8630b2557bfb2689bfa44a7672f080c0`.
The native New Bark Retina comparison is running. It retains the same corrected
camera-culling application probe for comparison against the saved whole-tree
correction and unchanged-renderer executables. No section changes are
published or installed. More draw sections may cost more CPU; an actual native
comparison is required before acceptance. The saved corrected whole-tree
executable remains immutable for comparisons.

At New Bark (13,5), noon, 2560x1600, the section candidate passed screenshot
validation and visual inspection (buildings, characters, forest, water and
ground present). The 15-second sequence produced candidate/control/whole-tree
correction update counts of 365/361/371, median update times of
41.77/41.56/40.92 ms and p95 times of 59.27/48.15/46.03 ms. Candidate/control
queue completions were 23.448/24.038 per second with zero polling errors.
This is no useful New Bark improvement. The first roof comparison recorded candidate/control medians of 79.99/100.91 ms,
p95 119.31/110.61 ms, and queue completion rates of 11.755/9.904 per second.
Both passed visible screenshot validation and inspection with zero polling
errors. Candidate outstanding work changed from 1 to 12 versus a steady 36
in the control, so this is not yet an isolated GPU-throughput estimate.
The reverse-order control/candidate repeat recorded 192/179 updates,
median 71.84/84.09 ms, p95 133.71/92.98 ms, and queue completion rates
10.458/11.842 per second. The repeated candidate queue rate is close to its
first run (11.755), but the second control accumulated 34 outstanding submissions,
so its CPU update median is not steady GPU throughput. A 60-second control and
candidate comparison using the existing measurement path is next, to reduce
queue-startup effects. No new measurement tooling was added. The panorama keeps
active_frames at zero until every source map and border placement is complete;
the existing 30-active-frame warmup therefore does not accept partial scenery.
No result is accepted for production yet.

The 60-second comparison reversed the apparent short-run benefit. Control:
638 updates, median/p95 89.88/127.54 ms, 10.654 queue completions/s, outstanding
36→34. Section+culling candidate: 512 updates, median/p95 114.82/169.20 ms,
8.532 queue completions/s, outstanding 1→1. Both had zero polling errors.
The combined candidate is rejected for promotion; the short-run roof gain is
not accepted evidence of a sustained improvement. To isolate subdivision from
culling overhead, the private application hook and SHA2 dependency placement
are restored to the pre-probe baseline. The voxel section implementation stays
unchanged for a release comparison with ordinary native culling. The saved
combined executable remains immutable; shared production sources are unchanged.
 The static images do not establish
animated culling bounds or paired-2D acceptance.

## Full-tree corrected culling trial

The saved shadow-cache release executable passed the full-quality native New
Bark warmup at (13,5), noon, 2560x1600. Its screenshot was inspected: the houses,
characters, forest, water and ground remain present. This is a baseline scene
check, not a new paired-2D visual acceptance. The contended 15.24-second sample
had 169 updates, median/p95 59.37/217.85 ms. It cannot establish a gain over the
earlier quieter runs. The native pack SHA256 remains
`a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007`.

The next private trial isolates corrected camera culling on the existing whole
trees. Only frozen crystal-bevy application inputs changed: `bevy_shell.rs`,
`bevy_shell/culling_edge_probe.rs` and its `edge_clamp.rs` child. The voxel crate,
mesh layout, resolution, materials, geometry and production bundle are unchanged.
The trial is compiled only with location-tester. It observes the current native
world layer (31), an active perspective camera and its existing depth/normal
prepasses, genuine GPU preprocessing support and both verified shader sources.
It owns only culling components it inserts. Orthographic/foreign views and
unsupported maps retain their existing state. This temporary app-level boundary
is for comparison; any accepted production integration belongs in the voxel
renderer. No new player or diagnostic switch was introduced.

All 13 isolated tests passed, including three application-boundary checks in
addition to the prior guards/real-source lifecycle tests. These do not prove
Metal shader execution or performance. The first private build failed because crystal-bevy exposes SHA2 only as a
development dependency; the application library needs it as a normal dependency.
The private manifest now moves that existing workspace dependency into the normal
section. No new package or lockfile change is needed. The retry completed successfully in 28m49s; its log is
`/tmp/geothite-edge-app-release-build-retry.log`; the original failure remains in
`/tmp/geothite-edge-app-release-build.log`. The private pre-trial manifest is saved
for restoration. The shared repository manifest is unchanged. The tree-section prototype is still
unapplied. Before promotion, actual screenshots, movement/warps and animated
bounds (including grass/flower vertex displacement and skinned actors) need
verification, as do cold-entry pipeline readiness and matched frame-rate runs.
The current user decision is to leave other builds running.

The immutable corrected trial executable SHA256 is
`581c1936f951798d9d24f32dfa586cfab13b1361225abf8eaeabff90da33ce55`.
The actual Metal run verified both shader corrections and activated exactly one
world-camera culling view. The full-quality New Bark screenshot passed the
existing validation and was inspected: buildings, forest, ground, water and
characters remain visible. Static inspection does not establish animated-bound
correctness. The 15.03-second sample recorded only 83 updates, median/p95
149.87/390.58 ms under concurrent compilation. The immediately following control recorded 105 updates in 15.01 seconds,
median/p95 107.34/312.00 ms and 6.898 queue completions/s, versus 5.593/s
for the correction. Both runs had zero polling errors. Sequential runs under
variable contention are not an isolated performance estimate, but provide no
evidence for promotion. The trial is not installed; enabling this path by
itself is rejected as the current performance solution.

The pinned Bevy source resolves one warmup concern: gpu_preprocess.rs
prepare_preprocess_pipelines queues both early/late occlusion kernels whenever
the device supports culling, independently of any camera's opt-in. The core
create_downsample_depth_pipelines function likewise queues both phases and
multisample variants once at initialization. No extra warmup timer or player
setting is needed for that queueing. Actual pipeline readiness, depth-texture
allocation and doorway behavior still need runtime verification. MeshCullingData
uses the entity Aabb directly; animated grass/flower displacement bounds must be
validated rather than assuming NoFrustumCulling bypasses occlusion (it does not
in the pinned mesh_preprocess shader).
The frozen scene confirms the concrete animated-bound path: terrain entities
use terrain_transform (translation only); spawn_terrain_entity supplies no swept
Aabb. Bevy calculate_bounds derives/refreshes bounds from Mesh geometry through
AssetChanged<Mesh3d>, while grass_vertex.wesl subsequently bends rooted grass
and sways flower tags 86000/86200. MeshCullingData consumes the entity Aabb.
No visible popping is established yet, but static position bounds alone do not
prove conservative coverage of shader displacement. Preserve that distinction
when reviewing any performance gain. No animated-bound change is applied in
the current isolated correction trial.

## Upper-floor panorama preloading installed

The shared implementation prepares the complete roof panorama on native
TinTower7F–9F after the foreground has settled for 30 active frames. It retains
that cache across those floors and the roof, pauses in 2D/menu ownership, clears
outside the bounded group and revalidates source/profile revisions. Browser
speculation remains off because its whole-map mesher does not use a CPU worker.

Source publication appends one map per update, loading its art as needed and
pinning the authenticated animation phase. All 32 connected maps and the border
remain present. GPU handles and grove recipes are shared across repeated border
placements: eight placements per update during background preparation, 128
while roof entry waits. The growing panorama stays hidden until complete and
uses the existing staged-destination pipeline warmup. No new user switch, LOD,
resolution reduction, color reduction or timing-tool implementation was added.
Hidden work never owns the room's terrain_pending/active_frames. Water material
registration changes only when its coordinates actually change.

Validation:

- Ten fresh renderer tests passed; one unrelated external cave-export test was
  ignored. They cover foreground priority, pending paint, exact image reuse,
  partial-source waiting, hidden/complete visibility, cache reuse/invalidation,
  disposal and real material AssetEvents. Border coverage/asset reuse is tested
  with actual groves under both placement budgets.
- The external-pack source regression passed: progressive extraction matches
  complete extraction for all 32 maps, tiles, ledges, terrace datum and atlas
  bytes; handles/animation phase remain stable across chunks, epoch changes
  restart correctly and the authoritative game checksum is unchanged.
- The final ordinary-input native stair run passed: 5,792 movement rows, three
  grid origins, zero additional foreground terrain builds, zero pending-terrain
  rows. All panorama sources were published before the Down step from walkable
  9F (7,8) onto the real stair (7,9). First roof row was (9,13), active and ready.
  An earlier requested start at (7,10) was rejected as unwalkable and is not
  counted. The final run included a system-profiler interval, so its timing is
  not an uninstrumented FPS benchmark.
- A preliminary quiet bounded-placement run reduced the longest update gap
  from 1,448 to 237 ms and p95 from 27.36 to 24.92 ms. This is a single controlled
  comparison of the earlier bounded candidate, not a repeated final FPS proof.
- Final cold full-scene capture took 33.64 seconds (paired 2D at 44.45), versus
  baseline 31.50/42.03. These include preparation, warmup and readback, not just
  gameplay fade. This is a small cold-load cost, not a cold-start speedup; the
  demonstrated benefit is preparing before the normal stair transition.
- Final paired captures preserved state and were visually inspected against 2D
  and baseline at identical position/camera/hour. The 700x200 roof region is
  exactly RGB-identical (140,000 pixels); cloud animation phases differ.
- Shared-source type checking passed in 26.80 seconds. The production build
  passed in 6m14s with desktop-app/voxel-view and no location-tester feature.
  Pre-sign executable SHA256:
  `cfc16a6ebdf912dc83e96a7494da1198fe1f0d6e9819e2ddd2a3e85813183199`.
  Installation completed in 31.6 seconds at ~/Applications/Geothite.app.
  Strict deep codesign verification passed. The independent installed
  --check-installation run from /tmp, with the model-root override removed,
  passed and resolved the bundled pack/resources and the unchanged save path.

Rejected private trials were not shipped: worker-only mesh conversion regressed
quiet update timing; a fixed eight-placement budget stretched cold capture past
2m32s and was stopped; hiding the unfinished scene alone still took 101.14s.
The dual budget removes that severe regression. Two profiler attempts missed
already-exited captures and provide no profile evidence. The final live sample
succeeded and shows visibility/query work and repeated hierarchy hashing; it
identifies further CPU investigation, not a proven dominant bottleneck.

The native pack remains unchanged (SHA256
`a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007`).
The implementation was published only after comparing the installed source
snapshots; unrelated lib.rs changes were preserved. Nothing was staged or
committed.

Hardware is Apple M5 with a 2880x1864 Retina display. The comparisons above use
1280x800 physical targets and the wasm-dev QA profile (six workspace crates at
opt-level 1). They do not establish shipped release FPS at Retina resolution.
Release/Retina frame-rate validation and broader GPU optimization remain
outstanding; the full performance goal is not complete. A separate release QA
example build is now running. The installed executable was built without
location-tester; its exact pre-sign production binary was retained before the
separate example build. No tester build is being installed. Apple Instruments
is available, including Metal System Trace and Game Performance, for subsequent
GPU diagnosis using existing tools.

## Summit panorama shared painted textures

The panorama previously synchronously called painted_stone::image and
bedrock_image, including PNG decoding and mip generation, then allocated two
new Image assets. It now waits for the active terrain's published pair and
clones those exact handles into its material. The original samplers, mip chains,
formats and source pixels are therefore shared; no fallback decoder runs on
the frame thread and no duplicate pair is uploaded for this scene.

Five summit tests passed, including the production sync regression proving
pending preparation allocates no material/images, then readiness uses exactly
the two supplied Image assets and the same handles. Eleven sky, five painted
image and 42 renderer checks also passed on that fresh harness (counts overlap
other runs). This implementation is in shared source. Its isolated native QA
build completed in 4m01s, with the sky-prepass experiment excluded from that
binary (SHA256 `8465a0fdbf89baa3f5b876b4172e05188ffdb72b581df35ea9a44fc2adea2058`).
Baseline and candidate roof 3D/2D captures both passed state_preserved=true.
The inspected scene preserves appearance; the same 1000x200 roof region is
pixel-identical. The baseline used bounded loading tracing, whose ring dropped
startup events. Candidate capture disabled tracing. Neither elapsed capture
times nor the truncated trace establish a loading-time or decode-count result;
the production-system regression establishes image handle reuse/allocation.
No timing/capture implementation was changed. The combined production build with the independently verified sky-prepass
removal finished in 8m33s. Installation at `~/Applications/Geothite.app` completed
in 32.1s. Strict deep codesign verification passed. Pre-sign executable SHA256:
`e6579afd043edcc3cf211e3cd72788672ee6f580f529ea2ab91d24562ee76b2c`.
The native pack SHA256 remains
`a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007`.
An independent installed `--check-installation` run from `/tmp`, with the model
root override removed, passed and resolved resources from the app bundle. The
existing save identity/path is unchanged.

Further source evidence: script_callbacks::tick_visible_screen_fade holds the
entry fade while terrain_pending is true. Summit preparation keeps that flag
until all 32 connected Johto source maps plus the border finish, and starts only
after arrival on TinTowerRoof. This is a larger remaining transition issue.
Leaving TinTowerRoof currently discards the built
panorama, so return visits rebuild it. Also the GPU occlusion-culling experiment
only admits NewBarkTown/Route29, not the panorama-heavy roof. Neither behavior
has been changed. A roof-specific reuse/culling investigation may be more useful
than more vertex codecs, subject to retained-state and full-quality validation.


## Sky prepass removal installed

SkyMaterial now opts out of Bevy's depth/normal prepass and shadow work. Its
existing specializer disables depth writes in forward and prepass pipelines;
the dome is already NotShadowCaster/NotShadowReceiver. Bevy 0.20's prepass queue
skips materials with prepass_enabled=false. The color shader, geometry, camera
and depth-write policy are unchanged; no replacement pass was added.

An isolated native build completed in 3m16s (SHA256
`4fe9bbb185d8753ed11ef69a6a66e184abfa332cb0eb31ae5703675fca6ebfb1`).
The first roof baseline timed out before its paired 2D result and is not counted.
With the existing 180-second capture allowance, all four low-horizon day/night
baseline/candidate captures passed state_preserved=true. Visual inspection found
no new defect; the 1000x200 roof region (150,550)-(1150,750) is pixel-identical
between baseline and candidate in both lighting conditions. Sky cloud phases
differ across runs. This verifies the pass removal's inspected appearance, not
a throughput gain. The change is installed with panorama texture reuse above.
No capture or timing implementation changed.

## Packing comparison retry

A fresh ABBA retry completed at 30.475, 24.846, 31.423 and 23.405 queue completions/s.
The first two runs overlapped rustc PID 83176, the third also overlapped a voxel
test process, and only the final baseline had no observed competing process.
This retry is not a clean comparison and does not establish a packing speedup.
The implementation was not accepted. Its compiled client, candidate source copy
and measurement artifacts were removed after recording these results; no
packing default or toggle remains in the shared source.

## Static terrain normal packing trial (not accepted)

A temporary exact-attribute grass reuse probe found just 24 duplicate vertices
among 48,384 across nine family/scale cases (Meadow, ParkShort, ParkLong; units
8, 32, 64). Meadow had no duplicates. The probe was removed; adding global
welding here would spend preparation work for negligible reuse.

The trial applied Bevy's established Snorm16x2 normal
encoding to native static spatial batches after exact coverage publication.
It preserves source positions, colors, UVs, triangles, bounds and ownership;
worker/browser float attributes remain unchanged. Existing native grove normal
packing uses the same decoder and precision. The new grass regression passed at units 8/32/64: eight bytes saved per
vertex, every index/position/color/UV and coverage/bound unchanged, decoded
normal-vector error below 0.00015. The first terrain suite passed 23/24; the
remaining cave regression assumed Float32 storage, and now checks exact float
normals or decoded packed normals within the same bound while keeping exact
position/UV and triangle assertions. The corrected terrain suite passed all 24 checks. Renderer 42, shadow
receiver 10 and grass-rustle seven checks also passed on the fresh harness.
The pipeline-preload test filter ran zero tests (one GPU test is ignored); it
is not counted as validation. The native comparison build completed in 3m59s. Route29 and ordinary-input
house-exit 3D/2D captures both completed with state_preserved=true. The inspected
geometry, pigment, lighting and shadows match the prior appearance, including
existing canopy stipple; NPC/water animation phases differ across runs.
Baseline client SHA256 `99f10c9fda7ea3c7e72f917662c1aaca46c303e51c00f1f0a039d45e4289a2ed`;
candidate `38e0bd0df3a76967fff8482fe6c741a5ee22c75d73863d3f240080b911fa9510`.
The Route29 ABBA run completed with rates 18.203, 16.374, 16.902, 14.67
queue completions/s. All four runs overlapped varying compiler jobs; the
candidate did not establish a speedup. The shared terrain implementation and
cave regression were restored byte-for-byte to their verified baseline. The
private candidate and its added tests were subsequently removed. No toggle or unverified default change
was retained in the shared source. The private packing prototype and its
artifacts were subsequently removed after inconclusive retries. The house-exit CSV also verifies 384 frames,
zero additional terrain builds, and active NewBarkTown terrain on its first
authoritative arrival frame (13,5).
Native visual checks passed for the inspected route and house scenes. The trial
was not accepted or installed. Existing installed grove normal packing is
unaffected by this rejected extension to static terrain.


## Current native bottleneck evidence

A five-second macOS sample of the corrected native Route29 client at (50, 9),
noon, 1280x800, during an existing 20-second measurement run showed repeated
Metal command-buffer-capacity waits under Bevy PendingCommandBuffers::finish
and wgpu command_encoder_finish. This supports investigating GPU work and
submission pressure next; it does not identify a single shader or prove a
displayed FPS limit. The client and sample both exited successfully. No timing
tool, queue-depth override, engine fork or new runtime toggle was added.
The pinned Bevy 0.20 render_context.rs uses parallel finishing on native and
exposes no runtime sequential-finishing switch. Bevy’s release guidance on
retained rendering/change-driven specialization agrees with the already
installed material isolation fixes, rather than suggesting per-frame mutation
of static terrain. Sources: https://bevy.org/news/bevy-0-19/ and the installed
bevy_render-0.20.0/src/renderer/render_context.rs.

## Resolution diagnostic after the receiver correction

Same-client Route29 (50,9), noon, full native effects, sequential 15-second
1280x800 / 2560x1600 / 2560x1600 / 1280x800 runs completed successfully. Queue
completions/s were 23.203, 18.919, 18.477, 25.589 respectively. Four times the
pixels reduced the average rate by about 23%, but unrelated compilation and
the production install overlapped parts of these runs. This is diagnostic
evidence only, not a clean speedup comparison or displayed FPS. Geometry,
shadow/submission work and pixel effects require further investigation.
The first 640x400 probe failed the existing minimum-height screenshot verifier;
it was stopped, is not counted, and its outputs were removed. No timing code,
production quality setting or resolution policy was changed.

## Shadow receiver-height correction installed

The receiver-height scan now excludes standard nonreceivers, including the
3,000-unit sky sphere. VoxelMaterial geometry remains conservative because its
shader samples directional shadows explicitly. Nonreceiving objects retain
their ability to cast shadows, and morph/unknown-bound/grass safety guards remain.
Ten receiver tests, 42 renderer tests, 11 sky tests, 47 fade tests and seven grass
tests passed in the frozen native QA source. A New Bark capture at (13, 6), noon,
1280x800 emitted an early-loading trace of 150 to 106 retained casters
(213 candidates in both runs). This is an initial streaming count, not a
settled-scene count; subsequent house-exit traces contain thousands of candidates.
Fresh five-second settled runs then verified 2,622 candidates in both cases,
with 1,911 retained before and 1,567 after (344 fewer, 18.0%). These tracing
runs overlapped the production compile and establish counts, not a separate
FPS gain. Both clients exited successfully.
The inspected scene preserved geometry and visible shadows; NPC animation phases
differ between runs. Sequential ABBA runs at 2560x1600 measured GPU queue completions/s
29.832, 30.326, 30.094, 29.543. Baseline mean 29.688; candidate mean 30.21.
These are queue completions, not display-presented FPS, and do not prove a
material fluidity improvement.
Native synthetic-keyboard house exit and Route29 DDLLDD grass walk subsequently
passed with state_preserved=true in both 3D/2D pairs. Inspected house and grass
captures retain existing appearance, including the previously observed canopy
stipple. The production build finished in 7m37s and installation completed in 69.6s at
`~/Applications/Geothite.app`. Strict deep codesign verification passed. The
pre-sign release executable SHA256 is
`e9681df57d9312e20336377cad4a23a10ad499075a8a64ba3cb5e8f57f5b1f08`.
The external native pack SHA256 remains
`a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007`. The correction is now installed in production. An independent run from `/tmp`
with `CRYSTAL_OPEN_MODEL_ROOT` removed passed `--check-installation`, resolving
resources inside the installed bundle and preserving the existing save path.


## Rejected opaque position packing

A native Bevy position-packing trial reduced opaque position storage from 12 to
8 bytes per vertex (28 to 24 total bytes for the existing RGB/packed-normal
layout). Sorted fades and source collision stayed exact; nine family/scale
cases retained every nondegenerate face without reversals. The unchanged
bank-geometry regression and the grove suites passed. A speculative fade-layout
adapter also preserved pipeline prewarming for the exact float-position fades.

Native capture caught a Bevy 0.20 extraction issue: `Mesh::take_gpu_data`
overwrites the precomputed packed AABB with a failed Float32 recomputation.
Trees disappeared with render-only usage. Retaining main-world data selected
Bevy's clone extraction and restored the full rendered scene. This workaround
also retains CPU vertex data, making actual frame benefit essential.

Matched frozen-source optimized-development clients then ran ABBA comparisons
at noon, full default quality, 15 seconds per run, with validated warmup and no
readback during measurement. No competing Geothite renderer or compiler was
observed in any of the eight runs. GPU queue depth was 36 at both endpoints,
with no polling errors. These are completion rates, not displayed FPS:

| Scene | Baseline completions/s, mean | Packed completions/s, mean |
| --- | ---: | ---: |
| Route 29 (50,9), 1280x800 | 28.8095 | 28.6335 |
| New Bark (13,6), 2560x1600 | 30.1645 | 30.0835 |

There was no useful rendering gain. The entire packing trial, CPU-retention
workaround, fade-layout adapter and trial-only tests were removed. The three
runtime files were restored byte-for-byte to their saved baseline; the isolated
forest test adjustment was reverted without touching concurrent density work.
No toggle was added or left behind. The installed grass-material CPU fix is
unaffected. Shadow receiver-height handling is the next active investigation.

## Rooted-grass material isolation (installed)

Walking springs previously wrote `grass_rustle.contact` to every VoxelMaterial.
Bevy 0.20's `mark_meshes_as_changed_if_their_materials_changed` marks meshes
using a changed material for extraction again. The spring therefore dirtied
static buildings, grove finishes, actors and destination preparation even
though those vertices do not carry the rooted-grass shader tag.

A retained grass material now clones the exact configured solid finish when
terrain publishes. Only static batches containing a shader-tagged grass vertex
bind it; buildings and trees keep the static finish, and flowers retain their
own clock material. Worker preparation records that identity before CPU vertex
buffers retire, and the worker response carries the bit. Legacy responses
without the field derive it from their restored UVs. The shader, geometry,
spring curve, contact coordinates and rebound are unchanged. Cutaway and map
finish updates include the grass material. At destination-cache handoff the
previous grass material settles once, preventing a hidden spring value from
keeping the conservative shadow-culling guard active.

Seven grass regressions pass, including 64 real Bevy asset-event updates proving
positive/rebound/settling motion without Modified events for static, actor or
flower materials. Renderer publication (42), terrain batching (23), and
hidden-destination (5) tests pass. These prove removed invalidation and retained
bindings/finish; they do not establish an overall FPS improvement. The external
pack worker roundtrip passes on eight source maps with exact prepared output.
The handoff regression also passes: previous hidden grass settles once and
removing the active handle settles its last spring without modifying static
finishes.

Corrected frozen-source native checks used ordinary production input at noon,
1280x800, and offscreen image capture. The house route `DDBBBBBBBBBB` entered
New Bark (13,5), then settled at (13,6), with the first town frame active/not
pending and the foreground build count fixed at one. Route 29 `DDLRLRUU`
returned to (50,9), and `DDLLDD` entered the grass interior at (48,13), all
without additional terrain builds. Each paired 3D/2D capture preserved game
state. Existing receiver traces showed two spring intervals (seven frames
without pruning) during the interior walk, then resumed pruning. The first
edge-only walk did not activate the spring and is not used as spring evidence.
The Route 29 canopy fade also appeared in the earlier client; it is not a new
material-ownership defect.

Visual checks used the optimized development profile, not release FPS data.
The production-only desktop release built successfully in 15m16s; installation
took 59.6s. Installed `~/Applications/Geothite.app` passed its own installation
check from `/tmp` with the artwork override removed, plus deep/strict codesign
verification. Native pack hash and the existing external save path are
unchanged. This is the verified grass CPU-invalidation fix, not proof of the
broader fluid-FPS goal. The frozen build predates an unrelated later public
understory-diagnostic API addition; that addition is preserved in the checkout.

## Release GPU investigation (unresolved)

Offscreen release checks at New Bark (13,6), noon, retained full quality,
showed about 39 ms median graph submission versus 0.6 ms graph encoding.
Existing diagnostic shadow/SSAO controls isolated substantial GPU cost; they
are not shipped quality changes. Completion throughput from this unpaced,
deeply queued harness is not displayed FPS. Baseline CPU compilation overlap
also prevents treating diagnostic differences as precise speedup percentages.

A conservative paired-plane shadow-ray prototype passed ten
culling regression tests and preserved 20,000 generated receiving rays, but retained
1,876 versus 1,911 shadow submissions out of 2,622. It showed no convincing
frame-rate gain and was removed, including its test and runtime code. No new
performance toggle was added. The earlier installed preload/image-sharing and
flower-material fixes remain in place.

A remaining walking-specific issue is the existing shadow-culling fallback
while grass springs: shader deformation exceeds the stored mesh bounds, so
the system deliberately keeps stock shadow lists. A safe improvement needs
bounds covering both positive and rebound motion, with matching receiver and
caster treatment; removing that guard alone would be incorrect. No such
change has been implemented or accepted yet.

## Shared native painted-image preparation (installed)

Early destination work could start a separate painted-image worker before the
foreground had image handles to lend. Native preparation now shares one task
and one resulting pair of image handles between the foreground and destination
queues. The first consumer transfers the decoded images into Assets; every
consumer then receives those exact handles. No extra decoded global copy is
retained. Geometry jobs and source atlases remain independently owned.

The decoded bytes, mip levels, texture descriptors, samplers and usage flags
are unchanged. Image upload may overlap unfinished geometry, while terrain
publication still waits for its prerequisites. Canceling one scene detaches
its owner; work needed by another scene survives, and dropping the last owner
cancels pending work. No user-facing control was added. Cargo check passes.
All five painted-image tests pass, covering exact bytes/descriptors/samplers,
one shared image pair, last-owner cancellation and retained work for a second
scene. All five destination-preload checks pass. The hidden-scene test initially
omitted the destination's independent atlas from its expected count; the
corrected assertion requires four total images (source, retained atlas, and the
single painted pair), and verifies the foreground receives the same pair.
All 42 renderer checks pass, including atomic publication and map-handle reuse;
the flower filter still passes 37 checks with one existing skip. Counts overlap.

Matched shared/separate-owner review clients are building from an ignored
frozen source snapshot: 1,045 source files and 848 linked static inputs are
hash-guarded. Only three runtime source files differ. An initial private build
failed because its removed allocator directory was still listed as a workspace
member; that private manifest is corrected, keeping the original patched
allocator. The shared checkout was not changed by the snapshot repair. The shared-image implementation is now installed; no latency or FPS gain
is accepted from these contended comparisons.

The model compiler also rejected symlinked model inputs in the private snapshot;
those 623 files now use hard links. Shared and separate-owner clients built in
4m22s and 3m37s. A first baseline attempt incorrectly reused the shared artifact
because copy2 preserved old source timestamps. Their identical executable hashes
caught this before measurement. The invalid copy was removed, source files were
written with current timestamps, and the baseline rebuilt. Final executable
hashes differ, and all source/static guards passed. The snapshot is restored to
the shared-owner source after the builds.

Real-pack paired house exits then confirmed two preparation jobs/four PNG
decodes with separate owners versus one job/two PNG decodes with shared owners.
Both paths already reused the finished painted images in this scenario: two
painted uploads totaling 16,768,144 bytes each. The proven saving is redundant
CPU decoding and mip preparation, not reduced GPU upload traffic. Each paired
capture preserved its game state; both reached New Bark with active terrain,
no pending terrain and zero additional foreground builds. The reviewed 3D views
retain the same geometry, finish, lighting and readability. Native roof styling
still differs from the 2D source as before. Compilation overlapped all runs, so
no latency or displayed-FPS gain is attributed to these timings. The first
shared run also confirms one preparation job. The production release build
passed in 11m03s; complete desktop content verification took 692.5s. A late
two-line source-region cache-key addition changed lib.rs during compilation;
review confirmed that all image-ownership code still matches the validated
build. That later cache-key edit remains in the checkout for its subsequent
build. The verified app was installed at `~/Applications/Geothite.app` in 60.5s.
The installed copy passed `--check-installation` from outside the checkout with
the model-root override removed, and strict deep signature verification passed.
It resolves the same existing save path. No visible window was opened.
Own temporary frozen clients, model links, source snapshot, capture images and
receipts were removed after validation; Cargo build caches remain.

## Flower clock material isolation (installed)

A three-second macOS CPU sample of settled headless New Bark succeeded. It shows
material specialization (including repeated MeshPipeline/MeshLayouts clones),
mesh metadata and GPU-buffer preparation during steady play. This is sampled
CPU evidence under compilation load, not GPU timing or a frame-rate claim.
The sample is `/tmp/geothite-render-cpu-sample.txt`; the paired headless run
completed and preserved its state, with no additional terrain builds.

Production `flower::animate` previously changed `cache.solid_material` on every
frame. The installed Bevy 0.20 `mark_meshes_as_changed_if_their_materials_changed`
marks every Mesh3d using a changed material, so the blossom clock invalidated
shared static terrain/grove users. Animated solid geometry now owns a separate
retained material cloned from the complete configured solid finish. Publication
reuses its handle and binds the animated entity to it on every rebuild. The
clock, phase, geometry, shader, shadow behavior and texture handles are unchanged.
Cutaway updates still reach both solid domains. No new control was added.

Two new production tests check handle reuse/finish refresh and 64 actual Bevy
asset-event updates: only the animated material may be modified by the clock.
The existing production publication regression now checks the animated binding
and environment fields across its map/revision sequence. The flower-filtered
suite passed 37 checks (one existing external-rig check ignored), including
both new asset ownership tests. Compilation completed in 5m14s; execution took
0.08 seconds. Its output is `/tmp/geothite-flower-material-tests.log`. The
publication regression also passes (2m55s rebuild; 0.05s execution). Its
output is `/tmp/geothite-flower-material-publication-tests.log`. The same
completed test harness passes all 42 renderer checks, 55 occluder-fade checks
and nine shadow-receiver checks. These filter counts overlap other runs and
are not a unique-test total. No displayed-FPS gain is claimed yet. The change is now installed in the
release app. Frozen clients isolated the original and flower variants during
verification; their temporary capture/build snapshot is removed afterward.

A fresh three-second macOS CPU sample comparison used the frozen shared-clock
client and frozen flower-only client at New Bark (13,6), noon, 1280x800, after
GPU-validated warmup. Both render runs and samples exited successfully. The
shared-clock call tree includes base_specialize (26 samples at its observed
node) and MeshLayouts clone descendants; the flower-only sample contains no
base_specialize node. This supports removal of the sampled repeated CPU work,
not a numerical CPU percentage, GPU-time saving, or displayed-FPS gain. Samples
are `/tmp/geothite-cpu-cache-original.sample.txt` and
`/tmp/geothite-cpu-cache-flower.sample.txt`. Sampling and concurrent scheduling
make these runs unsuitable as frame-rate benchmarks.

A further flower-only rightward route reached New Bark x=13 through x=17
at y=6, retaining zero additional terrain builds across 384 frames. Its
three-second sample also contains no base_specialize node. The route completed
successfully; sampling and concurrent compilers exclude it from FPS comparisons.
The fresh renderer-neutral controller suite passed eight checks (one existing
ignored), including Mom/New Bark and authoritative battle execution. All five
renderer preload and 55 fade checks also pass using the completed renderer
harness. The fade command initially omitted CRYSTAL_RENDER_TEST_PACK; its
external-pack assertion failed, then passed with that required input set.
The first production release build succeeded in 15m16s; full desktop build and
content verification took 953.7s. The pack verifier reported 388 maps, 251
species, 251 moves and 377 audio programs. Eleven source inputs changed during
that build, so it was not installed. The release refresh passed in 8m27s (534.8s including content verification).
Its flower material, renderer integration, destination preload and tilt-shift
shader inputs match the checked source. Three later Pokemon gait edits changed
during compilation; this app contains the compiled gait revision, not those
subsequent edits. Those source edits are preserved. The app was installed at
`~/Applications/Geothite.app` in 66.7s. The installed executable passed `--check-installation`, resolving its pack and
resources from the installed bundle. `codesign --verify --deep --strict` also
passed. Own frozen clients, linked source snapshot, captures and receipts were
removed from ignored `.local/early-destination-check`; Cargo caches remain.
No visible app window was opened. The wasm-dev comparison clients inherit dev debug assertions;
their driver overhead and intervals must not be presented as release-app FPS.

## Native initial-room preload trial (verification in progress)

Native destination-source publication and hidden-scene preparation no longer
wait for the initial room to be active/non-pending. They still require an active
published source frame, enabled rendering and a different, authenticated warp
or nearby-door destination. The existing native worker, bounded spare scene,
source art, hidden roots and promotion path are reused. Browser initial-room
readiness gates remain unchanged. No game input, scripts, pack, audio or quality
preset was changed, and no user-facing performance control was added.

The real-pack house-floor regression now exercises early publication from both
PlayersHouse2F and PlayersHouse1F with renderer status inactive/pending, requiring
a valid New Bark source frame, Mom lookahead and an unchanged controller checksum.
A renderer regression requires a destination to finish while the initial room
is pending, remain hidden, start exactly one build and leave the current frame
and renderer status unchanged. All five renderer preload regressions pass,
including that new hidden-scene check. The real-pack publisher check also
passes for both house floors (one test, 1.75 seconds). The first invocation
encountered stale dependency libraries after concurrent movement API changes;
it was stopped and retried successfully against current sources. The headless
review client built successfully in 6m48s. Eight production controller checks pass (one
existing ignored), including Mom/New Bark and authoritative battle execution.
The broader publisher suite passed the real connection-arrival check but found
a stale distant-door assertion: it expected no destination at a corner with an
authenticated nearby route connection. The test now requires that route, then
clears only its disposable catalog connections to independently verify distant
doors produce no request. Production selection is unchanged. The corrected
three-test publisher suite passes (100.43 seconds under compilation load).
No latency gain is claimed. The release app now includes the early native
publication path and flower material isolation described above. The existing completed development executable was
saved in ignored `.local/early-destination-check/before`. It predates concurrent
natural-movement changes and is not an isolated preload baseline. A matching
baseline is required before attributing a latency difference to this trial,
now included in the installed release. Its targeted test output is in
`/tmp/geothite-early-destination-tests.log`. First-room readiness and cold-door
latency must both be checked before keeping the change.

Current-source headless captures at 1280x800 were inspected against their
state-preserved 2D pairs for PlayersHouse1F and the New Bark house exit. Room
props, actor positions, town geometry, materials, shadows and readability remain
aligned; this is not full-game visual acceptance. With exact-step input, a
single D stops at house tile (6,7), so the crossing route is `DDBBBBBBBBBB`.
The actual crossing reached New Bark with no new foreground terrain build,
`terrain_pending=false` on its first visible map frame, and completed fade/input
settling in 549.093 ms. Its moving-frame median was 85.07 ms / p95 263.56 ms,
and screenshot production took 67.51 seconds. Compilers overlapped this whole
run, including pack decode and GPU preparation; neither exit nor frame numbers
establish a speedup. The earlier saved executable also predates movement and
camera changes, so it is not a matched baseline. The speculative camera differs
from the live actor-follow camera, but the measured arrival did not wait for
missing terrain; no camera/preload patch was made from that hypothesis.


The existing render-walk harness waits for 90 ready-renderer frames before its
first input. A route without extra B buttons still has that fixed warmup; it is
not a zero-warmup first-image door press. Keep this identical in comparisons
and examine startup as well as the doorway interval so extra preparation time
is not mistaken for reduced total latency. No harness timing has been changed.

The current-source baseline built in 10m25s, and its wrapper restored both
shared source gates. Five source files changed during compilation (script data,
wild-challenge implementation/test, natural movement and frame publication), so
that executable is excluded from isolated preload attribution. To avoid repeated
moving-source comparisons, an ignored source snapshot is now building baseline
and candidate sequentially. Only the two native readiness gates differ within
that snapshot. Root assets/models remain linked to their existing originals;
no game programs, packs or generated audio were copied. The snapshot keeps the
original patched allocator implementation and guards it as a linked input;
its first attempt failed with a duplicate allocator lockfile identity, repaired
by removing only the snapshot's unused allocator workspace member. The retry is
no longer running: it reached a missing `art/battles/arena-palettes.json` include
and failed. The private art-directory link is repaired. Part of the Bevy graph
was rebuilt as well. The repaired retry completed all three clients: baseline (6m07s),
early preload (3m35s), and early preload plus flower material isolation (4m06s).
All source and 744 linked static-input hash guards passed. The private snapshot
ends at the flower variant; no wrapper remains live. All artifacts are ignored.

Paired warm house exits preserved their 2D state and showed the destination
ready on its first visible frame, with no extra foreground terrain build.
However, baseline settling took 591.514ms and the flower variant 3015.082ms,
including a 2741ms render-submit stall. These are exploratory measurements under
concurrent compilation, not an improvement claim. The harness waits 90 ready
frames before walking, so neither is a cold-door test.

Four stationary unpaced GPU-validated runs and four paced screenshot routes
also produced mixed results. Every set overlapped compilation. Additionally,
the original renderer monitor matched only a short executable-name list and
missed other tasks' `immersive_battle_3d` clients. Its `renderer_overlap=false`
receipts do not establish GPU isolation. Future comparisons must monitor all
Geothite example clients. Unpaced offscreen completion counts are not displayed
FPS, and the screenshot path uses a different render schedule. No frame-rate
or transition gain is accepted from these results; neither trial was installed during those comparisons.

## Rejected extra terrain pipeline cache callback

The installed Bevy 0.20 color specializer clones MeshPipeline before checking
its specialization cache. Its prepass specializer already checks the same kind
of cache first. A trial gave terrain materials that cache-first color callback,
delegating misses to Bevy unchanged. Installation ran after asset preparation
and before specialization, only for uniquely owned VoxelMaterial properties
still using Bevy's stock callback. Shared properties, other material types and
custom callbacks remain untouched. Reprepared materials are eligible again.
No shader, key, layout, geometry, draw, or quality setting changes. No toggle
was added. Cargo check and both ownership/repreparation tests passed; the
flower filter still passes 37 checks (one existing ignored). Test compilation
took 5m59s, and all 42 renderer checks also pass. The fourth frozen client adds
only this module and registration to the flower variant. Its source and all
744 linked inputs passed guards before and after the 2m11s build. An initial
command omitted the required location-tester feature and did not build.

The real house exit completes with state-preserved 2D pairing, zero additional
terrain builds and New Bark ready on its first visible frame. Its full 3D image
was inspected against the paired 2D view: buildings, foliage, actors, water,
shadowing and readability remain present. Native roof styling differs from 2D
as before. This is not full-game visual acceptance. The cache variant settled
in 219.544ms across ten route frames; the flower variant took 213.977ms. Neither
establishes an exit improvement. The cache run overlapped compilation, whereas
the subsequent flower run's monitor observed no compiler or renderer overlap.

Four further alternating runs all overlapped compilation. Cache route medians
were 42.98/74.18ms, flower medians 52.38/70.53ms; results are mixed and excluded
from speedup attribution. All completed with zero new terrain builds and the
same pack hash. These screenshot schedules are not displayed FPS. Three-second CPU samples succeeded for both the flower-only and callback
variants. Neither sample showed base_specialize, while the callback sample
included its per-frame material scan. Absence in a short sample is not proof
of zero calls, but this provided no evidence that the extra callback benefits
the flower-only scene. The callback module and registration were removed from
production source. No toggle or fallback experiment remains. Its passing tests
and frozen executable are trial evidence, not an accepted optimization or an
installed update. The fresh shared-clock sample does show repeated specialization and layout
cloning; see the flower-material evidence above.

## Map-space walking and static scenery

The optional-world publisher now anchors terrain and actors to stable source-map
coordinates and publishes `VisualWorldFrame.view_center` for the camera rig.
Traversal endpoints receive the same translation. Classic sprite transforms and
authoritative controller state are untouched. The existing terrain-vector reuse
path remains; there is no second per-frame grid clone. Hidden destinations use
the same coordinate convention. Orbit/first-person rigs, grove streaming,
near-camera wind selection, sky source sectors, and forward/shadow blossom phase
account for the camera origin. Tin Tower's panorama root also avoids publishing
unchanged transforms/visibility.

This targets actual Bevy work: the installed 0.20 `extract_meshes_for_gpu_building`
query includes `Changed<GlobalTransform>`, while transform propagation supports
skipping unchanged hierarchies. See [Bevy transform documentation](https://docs.rs/bevy/latest/bevy/prelude/struct.Transform.html).
Previously, subtile camera interpolation changed a retained root and all its
mesh descendants. The source-map convention keeps those scenery transforms
fixed and moves the camera instead. Geometry, textures, source art and presets
are unchanged.

The render API suite passed 28 checks (one existing ignored). Initial camera
translation equivalence and a 128-sample retained-root walking proof passed.
A stronger test now passes Bevy's actual transform propagation and observes zero
changed static descendants across 128 camera-motion samples. All 118 focused
surrounding renderer checks pass (camera, sky, groves, fade ownership, shadow
receivers, destination adoption, panorama, wind, grass and terrain tracking),
including the no-actor first-person preload and sky-sector checks. Production
walk publication passes at 60/30/9 Hz. Eight renderer-neutral controller checks
pass (one existing ignored), including Mom/New Bark and battle execution.
Both worker request checks pass, including exact camera-origin float bits.
All 50 battle/location anchor checks pass after correcting a stale disposable
Surf-preview gate: a pack declaring the all-251 overlay expects its actual
Remoraid level-20 rare Route44 water slot, while an original pack still expects
Poliwhirl level25. Rates, slots, RNG, controller behavior and packs are untouched.
The source-extent publication check also passes, as do 51 selected coordinate
checks covering scrolling, translation, negative origins, surface registration
and shifted geometry. Release compilation completed in 14m44s; desktop build
and content verification completed in 939.4 seconds, checking 388 maps,
251 species, 251 moves and 377 audio programs. Installation completed in
31.6 seconds at `~/Applications/Geothite.app`. The installed
`--check-installation` and strict deep code-signature verification passed.

The current development executable built in 3m41s. New Bark's 2560x1600/noon
`RRLLBB` captures and state-preserved 2D pairs were inspected: composition,
scenery, actor placement, materials, lighting and readability remain aligned.
All four preserve state within their pair. Before/after runs advance different
animation phases, so this is not a claim of whole-frame pixel identity.
Moving-frame medians (route frames 32–127) were before 74.408/62.231 ms,
after 42.732/69.353 ms; three of
four overlapped unrelated compilation and cannot establish a gain. At 1280x800
with the existing CPU trace, before 40.161/50.659 ms and after 45.676/48.351 ms;
only the first baseline was compiler-free. These results likewise do not prove
a speedup. Render-stage trace intervals include scheduling/device contention
and are not GPU timestamps. Later quiet 1280x800 moving-frame medians were
37.081 ms baseline versus 36.943/36.668 ms updated; respective p95 values were
38.550 versus 38.337/37.669 ms. A reversed final baseline overlapped compilation
and is excluded. This is effectively flat FPS, despite the verified reduction
in transform work; no noticeable frame-rate improvement is claimed.

Normal headless house exit, Cherrygrove night, Union Cave, house interior,
bedroom-to-ground-floor stairs and first-person views completed and were
inspected against their state-preserved 2D pairs. Scale, composition, scenery,
materials, lighting, character placement, cutaways and readability remain
aligned. The immediate house exit settled in 1,294.991 ms; publication included
a 138.160 ms frame, and terrain became ready before the authored fade finished.
This does not establish faster cold transitions. Earlier destination work and
publication hitches remain targets. An opt-in GPU diagnostic attempt reported
unavailable pass timestamps on Apple M5/Metal and lost its diagnostic device
while allocating a counter buffer. It yielded zero samples; no GPU timing claim
is made. Subsequent ordinary headless captures all passed. No timing-tool fix
was attempted. Broader fluidity and cold-transition improvements remain open.
No timing-tool implementation or performance toggle was added. Temporary
comparison captures, traces, inspection source and executables were removed;
Cargo caches remain.

### Remaining performance leads

The native initial-room/exterior overlap trial above removes the readiness
wait from `bevy_shell/destination_preload.rs::publish` and
`destination_preload.rs::prepare`; browser scheduling is unchanged. Both first
playable room and cold exit still need comparison. The five renderer checks
pass, and current-source real-pack publisher verification passes for both
house floors. The headless client and paired exit captures pass; an isolated
latency comparison remains pending.

The installed Bevy 0.20 source also offers camera-level `NoCpuCulling` and
incremental GPU-visible entity storage. With static terrain transforms now
available, a device-supported camera-only trial may avoid repeated CPU frustum
work while preserving the existing CPU shadow/receiver filtering on meshes.
This needs actual GPU-preprocessing support, conservative animated bounds,
matched edge/cutaway/shadow checks and real frame evidence. It is distinct from
the rejected hardware-occlusion and direct-drawing prototypes; no new culling
policy has been enabled by this work. Further installed-source review shows
`reset_view_visibility` and `SetViewVisibility::set_visible` already bypass
change detection when an entity remains visible. Static visible meshes therefore
do not necessarily trigger repeated extraction through `Changed<ViewVisibility>`.
That proposed extraction rationale does not justify a culling-policy change;
entity-level opt-out would also bypass the existing CPU receiver filter.

The CPU sample also captures `bevy_pbr::material::base_specialize` repeatedly
cloning MeshPipeline/MeshLayouts before asking SpecializedMeshPipelines for an
already-cached result. Bevy's prepass specialization checks `get_pipeline` first.
A future VoxelMaterial-only callback could do that lookup and delegate misses to
the stock function, using the public MaterialProperties callback. Ownership,
scheduling, exact cache semantics, GPU validation and actual benefit still need
proof; no callback override or Bevy dependency patch has been implemented.

A standalone world-bounds cache prototype was rejected without integration.
It reused the exact Aabb/GlobalTransform results across receiver/cascade scans.
All 21 isolated checks passed, including equal caster lists across 192 frames
of camera motion, actor transforms, resized bounds and missing bounds. Its
fixture used small material/trace stubs, so those checks were not production
render acceptance. Optimized same-process comparisons on 3,000 boxes and two
cascades kept the same results but were slower with the cache. Four stationary
64-update samples measured 32.728/36.564/33.555/37.178 ms original versus
46.036/119.407/41.426/37.039 ms cached. Four moving samples measured
47.518/50.174/49.459/60.005 ms original versus
70.590/70.905/68.709/67.176 ms cached. Concurrent compilation affected absolute
costs, but there was no useful improvement to justify integration. The prototype
source, harnesses and binaries were removed; no production cache or toggle was
added. Evidence remains in `/tmp/geothite-shadow-bounds-cost.log`.

## Prepared scenery and animated bounds ownership

Prepared scenery leaves now retain their already-computed bounds with Bevy's
`NoAutoAabb`. The same ownership marker preserves the full movement envelopes
supplied for skinned people, costumes and world Pokémon. A regression runs
Bevy's actual mesh-insertion bounds system: without the marker it replaces the
movement envelope with the smaller neutral-mesh box. Static-leaf tests also
check that their supplied bounds enclose the actual vertices, including the
separate opaque and deferred fade copies. Ordinary view/shadow culling remains
active. This follows [Bevy's precomputed-bounds API](https://docs.rs/bevy/latest/bevy/camera/visibility/struct.NoAutoAabb.html),
also checked against the installed 0.20 source.

World Pokémon only publish a joint transform when its value changes. Animation
sampling and its cue clock stay the same. Cached envelope preparation now
selects additional samples from the measured angular-speed bound and smallest
neutral span, targeting at most 9% added diameter padding with a 120 Hz
preparation cap. This fixed Cyndaquil and Horsea exceeding the existing 10%
padding budget. The original analytic between-sample safety margin remains;
physical dimensions, vertices and playback keys are unchanged. Dense off-grid
samples and blended poses pass for every supported species.

All 109 focused renderer checks pass, covering owned/static/animated bounds,
source rigs and paintings, independent skin joints, geometry/fade ownership,
shadow receivers, material keys, hidden pipeline reuse and battle layout/size/
contact/hit anchors. Eight renderer-neutral controller checks pass, including
Mom, the New Bark gate and an authoritative battle turn; one other controller
check remains ignored. This does not certify the complete renderer library.

The no-dwell house-exit replay starts in `PlayersHouse1F` at (6,6) and presses
`DBBBBBBBBBB`, without the earlier twelve-button preload wait. At 2560×1600/noon,
compiler-free baseline transitions settled in 1.319 and 1.241 seconds; one
compiler-free updated transition settled in 0.963 seconds. Further updated
captures overlapped unrelated compilation (1.379, 0.893 and 4.280 seconds) and
are excluded from the comparison. One clean updated capture does not establish
a repeatable cold-exit gain. Under-load latency and sustained frame pacing
remain open. The final production image and its state-preserved 2D pair were
inspected for scenery, characters, materials, shadows and readability.

Two old opt-in experiments were tested and removed entirely. Inactive cave/
battle fragment specialization had essentially flat frame intervals. Direct
native drawing sometimes helped stationary views, but reversed walking was
mixed: original medians 45.263 / 37.351 ms versus candidate 38.642 / 41.332 ms.
The engine's normal drawing path remains the production default. The two
prototype modules, environment controls and material-key branches are gone;
no timing-tool implementation was changed.

The release build and desktop content verification completed in 411.9 seconds,
checking 388 maps, 251 species, 251 moves and 377 audio programs. Packs and audio
were not changed.
Installation completed in 30.3 seconds at `~/Applications/Geothite.app`.
The installed `--check-installation` and strict deep code-signature verification
passed. Comparison captures and the temporary baseline executable were removed;
Cargo caches remain. The pre-existing interactive process was subsequently
closed after the user confirmed their progress was saved; later comparisons
can run without that competing game process.

## Rejected tree shadow-part refinement

A native-only candidate retained the original spatial leaf bounds alongside
merged opaque tree geometry. All 11 shadow proof checks and 48 scenery ownership
checks passed, including every triangle fitting an original part. At the New
Bark noon view it reduced shadow-caster entries from 6,154 to 6,108 (46 entries,
0.75%). All four reversed-order captures overlapped unrelated compilation;
frame timings therefore establish no gain. The candidate, metadata, component,
and its three additional tests were removed instead of retaining another
control. Existing proven receiver culling and full geometry are preserved.
The saved older game was closed with the user's explicit approval. Temporary
comparison images, logs and executable were removed; Cargo caches remain.

## Rejected receiver-height arithmetic shortcut

A two-corner affine height shortcut matched the original eight-corner result
across 10,000 affine transforms, including reflection, nonuniform scale and
shear; all ten receiver tests passed. However, an optimized isolated arithmetic
comparison with compilation idle took 7.96–8.77 ms per million bounds versus
6.97–7.98 ms for the original SIMD-friendly calculation (six alternating pairs).
The validity/overflow guard outweighed the reduced corner count. This is an
arithmetic comparison, not gameplay frame-rate evidence. The shortcut and its
additional test were removed; no runtime toggle or benchmark tool was added.
Temporary executable/source were removed. This prompted the coordinated
camera/actor/light/cutaway map-space work documented above; changing only the
terrain transform would not have preserved the view.

## Native two-cascade shadow budget and linear face preparation

The native high preset now uses two directional-shadow cascades instead of
three. The closest cascade retains its range and 2048-pixel map resolution;
the last cascade still covers the same world distance. One outer map now
covers the farther range, trading some distant shadow detail for one fewer
complete geometry pass. Low and browser presets keep one cascade. This is
automatic and preserves native image resolution, source textures and geometry.

Tree preparation now expands a vertex's adjacency list only once, releasing
the temporary write cursor before output allocation. This gives linear work
in vertices plus adjacency entries, even at high-valence junctions, and retains
the previously verified triangle ordering and vertex-cache workload counts.

At 2560×1600/noon, matched ordinary-input before/after/after/before captures:

| Scene | Three-cascade medians | Two-cascade medians |
| --- | --- | --- |
| New Bark | 50.84 / 51.68 ms | 42.20 / 41.90 ms |
| Cherrygrove | 53.59 / 51.53 ms | 48.73 / 55.48 ms |

New Bark improved about 18%; Cherrygrove was mixed and does not establish a
speedup. A reversed actual walking pair measured median 50.85 → 41.46 ms and
p95 53.04 → 43.93 ms. An earlier new walking run had p95 spikes of 114.28 ms,
so elimination of all stutter is not established. A Goldenrod pair measured
46.42 → 19.57 ms, but its baseline varied substantially between other checks;
this single pair does not establish a city-wide gain. Existing day-cycle logic
disables geometry shadows at night, so night timing variation is not a
cascade-count benefit. The older interactive process remained running during
these shared-device offscreen comparisons.

Paired day/night images were visually inspected at the corresponding 2D
position, time and state. Characters, buildings, nearby/distant shadows and
night windows remain readable. The prepared house exit settled in 280.515 ms
with no additional terrain build. Full-game smoothness and 60 FPS remain
unverified.

First-use vertex-buffer reordering and Bevy hardware occlusion culling were
implemented, tested and removed after failing to establish a repeatable
worthwhile gain. Neither leaves a disabled switch or unused runtime path.
No timing-tool implementation was changed.

A subsequent receiver-box refinement was tested and removed. At the same
New Bark position and resolution, settled baseline medians were 41.829 / 42.124 ms
versus 42.122 / 41.999 ms for the candidate. Moving receiver bounds invalidated
its decision cache, increasing the observed culling-system CPU cost from about
0.5 ms to 1.4 ms; conservative wind bounds also retained 6,204 rather than 6,154
casters. It did not earn its added cost. No candidate runtime path or toggle is
retained, and the installed build remains the verified two-cascade build above.
The restored implementation passed all nine receiver regressions. Installed content
and strict code-signature checks passed again; comparison scratch files were removed.

All 94 focused checks passed, covering render quality, atmosphere/cascade updates,
receiver culling, opaque/fade geometry, shared caches, streaming, transitions and
large-map retirement. The full renderer suite is not represented as clean.

The optimized build and content checks completed in 424.5 seconds; installation
with `python3 tools/desktop.py install --skip-build --profile release --target-dir
target/performance-release-native` completed in 29.5 seconds at
`~/Applications/Geothite.app`. Installed `--check-installation` and
`codesign --verify --deep --strict` both passed. Temporary comparison executables,
images and traces were removed; packs/audio were not changed. The pre-existing
interactive process remains open and needs relaunching to load this build.

## Native opaque tree face locality

Spatial leaf partitioning scattered faces that share vertices. Native cached
opaque tree meshes now traverse connected faces together before GPU upload.
This retains every triangle's winding, every vertex attribute, full source
collision, and the independent distance-sorted transparent leaves. Index width
is preserved. The work runs once during ordinary cached mesh preparation;
there is no new environment switch and no browser/worker wire change.

The source-tree regression verifies exact triangle multiplicity, attributes,
collision ownership and unchanged transparent indices. A simulated 32-entry
FIFO post-transform cache needed 65,949 → 36,314 vertex transforms for Johto
pine, 12,251 → 7,557 for ancient forest, and 35,002 → 16,349 for Kanto deciduous.
These are workload counts, not GPU timestamp measurements.

Matched headless native captures at 2560×1600/noon used ordinary B inputs,
then compared before/after/after/before at each location. Both executables used
eight-sample AO and full native resolution; triangle order is the rendering
change in this comparison.

| Scene | Baseline medians | Locality medians | Change in paired mean of medians |
| --- | --- | --- | --- |
| New Bark | 61.89 / 64.64 ms | 51.14 / 51.01 ms | −19% |
| Cherrygrove | 69.21 / 75.29 ms | 52.48 / 52.24 ms | −28% |

The older interactive game remained running, so these are shared-device
headless frame intervals, not exclusive-device GPU timestamps or guaranteed
on-screen FPS. Both reverse baselines were slower than both optimized runs.
Paired 2D/3D captures preserved state; full geometry, pigment, shadows,
characters, signs, buildings and coast were visually inspected.

A real `RRLLBB` walking replay improved median intervals from 62.15 to
51.35 ms (p95 65.69 → 56.22 ms). Both builds visited the same nine
core positions (x=9…17, y=6), ended at (9,6), changed grid origin nine times
and submitted no additional terrain build. The prepared house exit settled
in 424.083 ms with no extra terrain build; stairs settled in 292.292 ms.
Mom, her seated companion, the player and house geometry remained correct in
paired 2D/3D inspection. These results do not establish 60 FPS or full-game
performance acceptance.

All 98 focused checks passed: opaque/source fade geometry (46), shared grove
preparation (3), forest instances (9), streaming/culling (10), render quality
(10), terrain animation (6), transition regressions (13), and the large-map
retirement safeguard (1). No timing-tool implementation was changed.

The optimized desktop build and content verification completed in 550.5 seconds;
installation to `~/Applications/Geothite.app` completed in 29.9 seconds using
`python3 tools/desktop.py install --skip-build --profile release --target-dir
target/performance-release-native`. The installed executable's
`--check-installation` and `codesign --verify --deep --strict` both passed.
Temporary comparison binaries/captures were removed. The existing interactive
process was not closed without confirmation; it needs relaunching to use this
build. Packs and audio were not changed or published.

Bevy research used its [profiling guide](https://github.com/bevyengine/bevy/blob/main/docs/profiling.md)
and the installed 0.20 renderer source. Eight-sample filtered ambient occlusion
retains contact radius and thickness while reducing the previous 18-sample
pass. Frame-time differences for that change were modest and variable.
Native 16-bit vertex-color packing and a 3D-only bicubic scaling prototype were
removed after they did not establish a repeatable worthwhile frame-time gain;
full native resolution and the original pigment representation remain in use.

## Automatic CPU and sky shader work reduction

Stationary characters no longer sample an unused locomotion pose or interpolate
it with zero weight. Idle breathing, seated motion, riding and moving animation
remain live. Head/hand proportions are composed before publishing joint
transforms, and unchanged joints/root transforms retain their change ticks.
This avoids redundant hierarchy propagation from resetting and rescaling static
joints. The duplicate actor-root assignment was removed.

The sky caches woodland sectors by terrain revision, map/grid geometry and
horizontal camera position. Actor-only frame publication and vertical camera
motion reuse the result. Recalculation retains scratch storage and computes each
tile's angle/distance once rather than twice. GPU sky shading skips cloud noise
where its elevation mask is exactly zero, daytime star hashes, and angular haze
lookup where its elevation mask is zero. All changes run automatically.

Verification: 9 sky tests and 89 focused character/scenery/loading tests passed.
The character suite used the supplied rigs, including all 78 spawn/proportion
cases and mounted hand/foot contacts. A broader unconfigured renderer run had
2773 passes, 131 failures and 105 ignored tests; failures included required
external-pack checks and assertions outside the changed paths. This does not
constitute a clean full renderer suite.

Offscreen native New Bark checks at 2560×1600, noon, eight ordinary B input
segments, used the same scene before/after/after/before. For the last 200
character-update spans while 3D was enabled, median CPU durations were
0.1117/0.1014 ms before and 0.0935/0.0930 ms after (about 12% lower across
those paired medians). This stage is only a small part of the frame budget.
Overall median frame intervals were 61.86/64.91 ms before versus
60.79/62.67 ms after, with variable tails; no reliable overall FPS claim is made.
Day and 22:00 shader captures compiled, retained state and matched paired 2D
positions/time. Characters, full scenery, water and night windows were visually
inspected. The high camera view does not establish a sky-dominated GPU speedup.

The final optimized desktop build completed in 641.1 seconds. Installation via
`python3 tools/desktop.py install --skip-build --profile release --target-dir
target/performance-release-native` completed in 42.9 seconds at
`~/Applications/Geothite.app`. The installed `--check-installation` and
`codesign --verify --deep --strict` checks both passed. The existing live game
session was not restarted. Temporary capture/trace/baseline files were removed;
content packs and audio were not changed.

## Automatic house and intermediate-room preparation

Destination preloading now includes the current indoor room and up to four
adjacent authored indoor warp destinations while preparing the outdoor scene.
Completed character meshes and textures are reused by appearance, so Mom can
already be uploaded before entering the ground floor. Changing indoor rooms
refreshes this character lookahead without rebuilding the prepared outdoor atlas.
Small native interior-to-interior transitions submit replacement terrain in the
first update. Large, outdoor and uncertain transitions retain their retirement
fence, including the department-store memory safeguard.

Matched offscreen 2560×1600 runs using ordinary inputs from PlayersHouse2F
(`BBBBBBBBUBBB`, noon) measured visible map change through settled presentation:

| Run | Before | Automatic preparation |
| --- | ---: | ---: |
| First | 670.716 ms | 289.296 ms |
| Second | 665.236 ms | 298.842 ms |

The prepared house exit into New Bark remained 293.146 ms with no additional
terrain build. These are loading results; overall frame intervals did not show
a corresponding FPS improvement. Paired 2D/3D captures retained game state and
were visually inspected at the same position and time.

Character reuse, indoor destination lookahead, immediate small-room submission,
large-map retirement and the Mom/New Bark gate controller regressions passed.
The unproven pine distance-detail experiment and its environment switches were
removed. These loading changes run automatically.
Final-source replay after removing the tree experiment and adding CPU/sky changes
settled the stairs in 330.958 ms and the prepared outdoor exit in 473.450 ms.
The outdoor exit still submitted no additional terrain build; its 114/134 ms
arrival frames show why individual wall-clock captures vary. Neither result is
represented as a guaranteed maximum transition time.

## Animated terrain source snapshots shared within each refresh

Sparse animation refresh now snapshots each distinct authenticated 8×8 source
image once, in a contiguous array, and refers to that snapshot for every atlas
slot using the image. Previously each matching water, lava, flower or pillar
occurrence allocated a separate 256-byte pixel buffer. Atlas writes and tile
handles remain unchanged, and all source validation finishes before atlas
mutation, preserving failure atomicity and source/destination alias behavior.

All six terrain-animation regressions pass: exact retained-slot repainting,
water/lava/static geometry preservation, flower persistence, selected pillar
poses, rejection of stale/forged/cross-map/invalid sources, and standard versus
streamed material bindings. This source change is not yet included in the
installed app. No measured frame-rate claim is made for this allocation change.

## Settled grove updates skip repeated publication and culling

Once the requested forest placements are complete and their draw visibility is
applied, an identical camera matrix, terrain offset, sun vector and shadow ground
plane now returns immediately. Previously even a completed selection cloned its
resident set, walked publication queues and tested every active tree against the
view/shadow footprint again. Changes to any key input run the ordinary path;
unfinished placements cannot create a stable selection, and battle warming
invalidates it. Tree detail, material, collision and shadow policy are unchanged.

The final source passes ten grove-stream, four destination-preload, nine
forest-instance and three cached-publication regressions, including every camera
key invalidation and off-camera shadow retention. This is avoided CPU work in
unchanged views, not a measured whole-game FPS claim. The optimized release build and native content verification succeed in 441.4
seconds; installation at `~/Applications/Geothite.app` succeeds in 29.6 seconds.
The actual installed executable passes `--check-installation` and strict deep
signature verification. The existing user game process is left running. No
new displayed-FPS or doorway-latency claim is made from these CPU regressions.

## Nonblocking cached grove publication

The native frame-thread cached-placement path now uses `try_lock` for both
prototype and ground-cover metadata. Worker contention returns a cache miss,
leaving the existing asynchronous job path to finish it. Canopy-only placements
skip the ground metadata lock entirely. Geometry generation still belongs to
the workers; no visual policy, collision data or content pack changes.

A regression holds each real metadata mutex while calling the cached path,
checks immediate deferral, then releases it and compares the recovered geometry
exactly against normal preparation. It also proves canopy-only publication
succeeds while ground metadata is busy. Nine forest-instance, ten grove-stream
and three cached-publication regressions pass. This removes a possible
frame-thread wait; no measured overall FPS improvement is asserted. This change is included in the verified installed release described above.

## Packed native grove normals installed

Ordinary native grove preparation now uses Bevy 0.20's built-in octahedral
`Snorm16x2` normals rather than `Float32x3`. Position, triangle order/winding,
colors, UVs, bounds and original collision data remain exact. Palette tree
vertex stride falls from 40 to 32 bytes. The opaque and sorted render copies
use the same layout, so the destination preloader specializes the actual future
fade layout. Invalid/unsupported normals, small groups and already shared or
uploaded meshes retain their original representation. Browser and worker-wire
builds retain float attributes; their protocol has not changed.
`CRYSTAL_PACKED_GROVE_NORMALS=off` is the native startup comparison override.
The optimized release app is installed after the release checks below.

The new regression checks every render copy of pine, ancient and deciduous
trees, exact positions/indices/colors/UVs/bounds, unchanged collision ownership,
eight bytes saved per vertex, matching Bevy opaque/fade layout keys and decoded
normal-vector error below 0.00015. A separate regression performs the real
opaque-to-sorted fade and restores the packed draw. All 45 occlusion/fade,
eight forest-instance, three shared-cache and three cached-publication tests
pass with the new native default. The test-only geometry signature also reads
Bevy's packed normal representation; cache/source comparisons remain exact.

Initial same-executable New Bark checks at noon, 2560×1600, native pack and
ordinary `BBBB` input used off/on/on/off order. Excluding the first 32 route
updates, medians were 70.636, 51.585, 67.792 and 73.178 ms (p95 104.807,
70.657, 71.811 and 115.002 ms). These initially encoded only the opaque copy.
That default view has no fading groves; the revised implementation additionally
packs sorted render copies to preserve the preloader's future-layout contract.
These variable capture intervals support further testing, not a final FPS claim.

Route 29's first close orbit-5/zoom-2 movement pair measured 44.52 ms original
and 79.21 ms packed. A reverse repeat was essentially neutral at 38.09 ms packed
and 37.93 ms original. With both render copies packed, another pair measured
38.64 versus 38.55 ms. Each route visits six authoritative positions without
another terrain build. Frozen Route 29 (52,9) crown comparison with both copies
packed differs at 86 of 152,100 pixels by at most one channel level. Moving
captures had larger fade-history differences and are not a pixel-exact comparison.
The sampled New Bark roof/forest regions also differ by at most one channel
level. Pine, Ilex ancient trees and Route 2 deciduous trees were visually
inspected against their paired 2D state; no missing geometry or shader errors
were observed. Existing foliage dissolve stippling remains outside this change.

A prepared house exit using `BBBBBBBBBBBBDBB` still promotes New Bark without
another foreground terrain build, finishes the ordinary fade and releases input.
The packed capture's arrival-to-ready update intervals total 566.820 ms versus
287.018 ms disabled; first outdoor intervals are 74.905/70.251 ms. The release checks below recheck this difference. Its trace prepares 22 pipeline requests
indoors; initial destination preparation completes several seconds before exit.
A later first-reveal frame includes a 90 ms CPU graph-encoding interval in the
existing image-capture path. These headless intervals include GPU readbacks,
share the GPU with the older running user session and are not displayed FPS.
No timing-tool implementation or content pack was changed.

### Final packed-normal release verification

The application release build succeeds in 368.8 seconds and the matching
location-tester example in 4m36s. The actual native pack and existing scenery/
people bundles validate; no pack is regenerated. Installation succeeds in
30.4 seconds at `~/Applications/Geothite.app`, retaining the same per-user save.
Only the additional bank regression's test code changed after these builds;
the production packing implementation remained unchanged.

The bank-geometry regression now compares reference normals in Bevy's encoded
render format and canonicalizes folded-seam float residue within one epsilon.
Its normal-vector codec error still must be below 0.00015. All position, color,
UV, winding, footprint, authored-cell and footing comparisons remain exact.
That regression passes both enabled and disabled; disabled retains its original
raw-normal-bit equality. Together with the 59 focused checks above, 60 focused
regressions now pass. The actual installed executable also passes `--check-installation` and strict
deep signature verification, separately from the staging bundle.

All following release comparisons use the same executable, native pack,
2560×1600 physical pixels, noon and the existing headless image path. Four
prepared house exits use off/on/on/off order and ordinary `BBBBBBBBBBBBDBB`
input from PlayersHouse1F (6,6):

| Mode | First outdoor interval | Map change through input release |
| --- | ---: | ---: |
| Original A | 64.525 ms | 297.293 ms |
| Packed A | 53.208 ms | 284.364 ms |
| Packed B | 61.051 ms | 290.391 ms |
| Original B | 62.851 ms | 289.414 ms |

Every exit promotes the town without an additional foreground terrain build,
finishes the fade, and returns authoritative input. The development capture's
longer packed handoff did not recur in these release runs. Paired 2D/3D final
state was inspected; sampled roof/forest regions differ at only 5/214,200 and
114/300,000 pixels, respectively, by at most one channel level.

Four ordinary stationary `BBBB` New Bark release checks use off/on/on/off order
(excluding their first 32 route updates):

| Mode | Median | p95 |
| --- | ---: | ---: |
| Original A | 68.367 ms | 81.004 ms |
| Packed A | 63.445 ms | 65.216 ms |
| Packed B | 67.768 ms | 72.776 ms |
| Original B | 46.915 ms | 93.999 ms |

Route 29 orbit-5/zoom-2 `RLRL` movement initially observes 43.700 ms packed
versus 61.522 ms original (p95 107.100/103.697 ms). Reverse order observes
45.581 ms original and 45.346 ms packed (p95 48.266/47.882 ms). Each run visits
six authoritative positions and starts no extra terrain build; source 2D state
and the production 3D scenery/shadows were inspected.

The consistent reduction is eight bytes per eligible vertex, including future
fade copies, with the prepared doorway latency preserved. The frame intervals
remain variable and do not establish a general FPS improvement. They include
readbacks and share the GPU with the older user game session, which was not
closed, paused or restarted. This increment does not complete the broader
fast/fluid gameplay objective. No timing-tool implementation changed.

## Native Cherrygrove receiver culling installed

The existing native receiver proof now also accepts CherrygroveCity in source.
It preserves complete meshes, cascades, filter size and resolution, retaining
stock lists on uncertain bounds, morphs, unsupported filters or active grass
springs. Nine receiver regressions pass, including the new map gate and retained
capture-camera lists. The fully optimized release app has now been installed,
and the actual installed copy passed content and strict signing checks.

Four same-executable headless image runs at noon, 1280×800, ordinary `BBBB`
input and the external browser-compatible pack compare the original list with
the filtered list in off/on/on/off order. Excluding the first 32 route updates:

| Run | Median | p95 |
| --- | ---: | ---: |
| Original A | 59.482 ms | 118.500 ms |
| Filtered A | 56.787 ms | 58.618 ms |
| Filtered B | 56.486 ms | 57.795 ms |
| Original B | 68.575 ms | 69.978 ms |

The default view retained 9,289 of 13,301 shadow submissions. The selected Mart,
forest and ground regions were pixel-exact. Paired close orbit-2/zoom-2 ordinary
`RLRL` movement runs each visit six authoritative positions within Cherrygrove,
with no additional foreground terrain build. Their paired 2D state and 3D
composition/shadows were inspected. Rotated Mart/Center roofs, Center facade and
ground regions also match exactly; one forest region differs at three pixels
by one channel level. The rotated view retains 4,369 of 5,020 shadow submissions.

An initial movement pair was slower with filtering: full-route medians were
48.07 ms original and 72.07 ms filtered (p95 96.14/165.14 ms). A reverse-order
repeat observed 40.165 ms filtered and 42.129 ms original, with similar p95
62.693/62.286 ms. Both repeats again visit six authoritative positions without
extra terrain builds. The older user game session remained open and shared the
GPU; release compilation was active during the repeat. These samples establish
reduced shadow work and the checked images, not consistently smoother movement
or displayed FPS. The final native-pack release checks below resolve the
wide-view comparison; close-camera movement still has mixed individual results.

### Final release checks using the actual native pack

The production release build and matching offscreen example both succeed. The
app build verifies the external native pack (388 maps, 251 species, 251 moves,
377 audio programs) and bundles existing scenery/people. Installation uses the
existing release command, keeps the per-user save location, and regenerates no
content pack. The installed app's own `--check-installation` and strict deep
signature verification succeed at `~/Applications/Geothite.app`.

Native-pack, 1280×800, default wide-camera checks repeat off/on/on/off using
ordinary `BBBB` input. Medians/p95 below exclude the first 32 route updates:

| Release run | Median | p95 |
| --- | ---: | ---: |
| Original A | 65.375 ms | 69.249 ms |
| Filtered A | 56.624 ms | 58.149 ms |
| Filtered B | 56.251 ms | 58.218 ms |
| Original B | 65.750 ms | 66.917 ms |

This is approximately 14% lower median in that view, with 9,289 rather than
13,301 shadow submissions. The checked Mart and ground regions match exactly;
151 pixels in the sampled forest region differ by at most four channel levels.
At 2560×1600 physical pixels, one additional pair observes 70.084/162.153 ms
original and 54.613/141.862 ms filtered (median/p95). The 696,000-pixel Mart and
240,800-pixel ground regions match exactly. Both high-resolution scenes were
visually inspected for scenery, shadow, color, silhouette and character fidelity.

Four close orbit-2/zoom-2 `RLRL` release movement runs use on/off/off/on order.
Filtered medians are 34.963 and 30.558 ms; originals are 33.611 and 32.585 ms.
Individual runs remain mixed, so no close-camera speedup is established. Each
visits six authoritative positions in Cherrygrove with zero extra foreground
terrain builds. Release scenes and their paired 2D state were inspected; the
source geometry, lighting, shadow filtering and resolution are unchanged.

All intervals include the existing headless image-capture path. The older game
session remains open and shares the GPU; these are not displayed FPS or a claim
of consistently fluid whole-game performance. No timing-tool implementation was
changed. This increment ships the verified wide-view work reduction; the broader
frame-rate objective remains open.

## Full house-exit handoff verification

An additional offscreen production-input run checks the entire handoff rather
than only the first outdoor frame. The fully optimized native example used the
existing browser-compatible local pack, PlayersHouse1F (6,6), noon,
1180×812 pixels and `--walk BBBBBBBBBBBBDBB`. Ordinary B input leaves time for
background preparation before the ordinary Down input exits the house.
New Bark becomes authoritative at route frame 393; at frame 403 the terrain
is active, preparation is clear, the fade is finished and the warp input owner
is released. The eleven sampled update intervals total 280.644 ms. The route
continues in New Bark with zero additional foreground terrain builds. This
is a prepared-destination headless observation, not a cold-exit guarantee or
a displayed-FPS measurement. The initial house and content loading are excluded.

The destination GPU-program preparation starts before the doorway. The existing
preload, fade, and grove-stream regressions pass. The resulting town screenshot
was inspected for complete scenery, shadows, character placement and readability.
A separate four-run experiment partitioned opaque pines without changing their
triangles. Although geometry and 44 fade/occlusion regressions passed and paired
images retained the scene, concurrent GPU/compilation load left the performance
benefit uncertain. That prototype was removed; it is not part of the installed
app. No timing-tool implementation was changed for these checks.

## Fully optimized native release installed

The macOS app is now built with the existing native release profile: optimization
level 3, ThinLTO, one codegen unit and stripped symbols. The performance sources
were unchanged. Concurrent department-store bench edits occurred during checking,
outside the field and house maps below; this is not a whole-tree byte-identical
build A/B. The installed executable is about 147 MiB,
compared with 256 MiB for that development executable. The release build verified
the external pack (388 maps, 251 species, 251 moves, 377 audio programs), bundled
models/people, passed signing and installation checks, and retained the existing
per-user save location. No content pack was regenerated.

Verified commands:

```sh
python3 tools/desktop.py build --profile release --target-dir target/performance-release-native --jobs 2
python3 tools/desktop.py install --skip-build --profile release --target-dir target/performance-release-native
```

Eight headless image-target probes use the same graphics settings and stationary
production B input for 512 updates, at 12:00. Another Geothite session remained
open and shared the GPU; other compilation was also observed. These intervals
include the existing capture path and are not uncontended displayed FPS.

| Map | Physical pixels | Development median | Release median |
| --- | --- | ---: | ---: |
| New Bark | 1280×800 | 61.91 ms | 64.14 ms |
| New Bark | 2560×1600 | 74.67 ms | 81.88 ms |
| Route 29 | 1280×800 | 83.96 ms | 73.91 ms |
| Route 29 | 2560×1600 | 86.66 ms | 85.82 ms |

The New Bark 1280×800 trace reduced the sampled main CPU envelope from 4.85 to
4.10 ms and view preparation from 4.87 to 3.98 ms. Render submission still
accounted for about 43–48 ms. Release optimization therefore does not establish
that the GPU/frame-rate bottleneck is solved. Quadrupling pixel count raises
cost, but substantial submission cost remains even at the lower resolution.

Release production-input checks at 2560×1600 exercise Select/B and movement in
New Bark (five authoritative positions), Route 29 movement (six), and the house
exit. All retain only their initial foreground terrain build. The first house
exit outdoor frame is 63.740 ms with New Bark installed; the ordinary fade
finishes and returns input. Final 3D scenes were inspected against paired 2D
state at preserved positions, time and facing. Geometry, color, lighting and
readability remain intact. The running older game session was not closed or
restarted by these checks.

## Native destination GPU pipeline preparation

Hidden destination terrain now prepares its color, directional-shadow and active
camera-prepass programs through Bevy 0.20's existing specialization caches. Its
future player-reveal programs use the same fade-material recipe as live
rendering, including masked and blended surfaces. Preparation only accepts
meshes beneath the bounded staged destination roots and uses the active world
camera's actual keys. Promotion removes that staging marker; hidden placeholder
actors and inactive/classic cameras cannot trigger speculative compilation.

The queue processes at most 64 ready mesh/material pairs and two specialization
requests per render update. Duplicate signatures, pending pairs and asset wait
lifetimes are bounded. An empty queue skips view/cache scanning. Ordinary native
play retains Bevy's asynchronous compilation; one-shot image QA keeps its
existing synchronous capture policy. No extra camera, draw, material binding,
approximate mesh or audio/content change is introduced.

The offscreen GPU regression compiles a hidden production voxel material before
visibility, then proves that the ordinary visible color, shadow and normals
passes use those exact cached pipeline IDs. It also checks a real hidden
authored group's future reveal material and rejects a hidden placeholder outside
the staged scene. The 43 occluder/fade, four destination, ten grove-stream and
49 encounter-terrain regressions pass. Destination promotion tests require the
old room to acquire the staging marker and the promoted town to lose it.

Same-binary native headless production input runs use PlayersHouse1F (6,6),
`--walk BBBBBBD --hour 12 --size 1180x812`, in on/off/on order. All reach New Bark
with zero additional foreground terrain builds and finish the ordinary fade.
Both enabled runs request 13 material specializations indoors. Their first
outdoor frames are 71.074 and 69.985 ms, compared with 135.650 ms disabled; their
largest outdoor movement intervals are 71.074 and 104.027 ms. No enabled outdoor
render submission exceeds 100 ms. Final scenes were inspected against paired
2D state. These are short capture observations with warmed driver state, not a
guarantee about every cold driver launch or a whole-game FPS result. The rebuilt
native app and its external pack passed installation checks.

`CRYSTAL_DESTINATION_PIPELINES=off` restores the original preparation behavior
for comparisons. The GPU regression is opt-in:
`cargo test --locked --profile wasm-dev -p crystal-voxel-view
hidden_material_compiles_before_visibility_and_visible_draw_reuses_its_pipeline
-- --ignored`.

## Doorway fades wait for visible scenery, not the outer preload band

The production grove streamer still prepares and retains its wider camera band,
but its presentation fence now waits only for the actual draw/shadow footprint
(the existing 1.15 margin). Missing outer-band trees continue preparing without
holding the doorway fade or its input owner. Visible trees and off-camera trees
casting visible shadows still hold the fence. This changes game loading behavior;
no timing or capture tooling was changed.

Ten grove-stream regressions pass, including a new case proving that an outer
preload tree releases the fence, rebased foreground geometry holds it, and an
off-camera visible shadow caster holds it. Four destination-preload, three
cached-grove and 49 encounter-terrain regressions also pass.

A fresh native headless production input route from PlayersHouse1F (6,6), using
`--walk BBBBBBD --hour 12 --size 1180x812`, reaches NewBarkTown with zero
additional foreground terrain builds. The prepared town is installed on the
first outdoor frame; its fence clears on the next update. The ordinary fade
finishes and background grove residency continues to 635. The final scene and
an arrival-reveal image were inspected against the paired 2D state.

The observed arrival frame was 135 ms. An earlier cold comparison still spent
1.68 seconds in render submission on first outdoor presentation, despite scene
preloading. The later capture benefits from warmed GPU state, so these figures
do not establish a causal speedup or elimination of the cold GPU hitch. The
conservative distant-needle prototype did not reduce work in the tested default
camera views and was removed entirely rather than shipping an unused path.

## Native shadow-receiver filtering enabled

Native New Bark Town and Route 29 now use conservative receiver-aware shadow
caster filtering by default. Browser builds retain the opt-in setting. The
filter rejects only casters proven unable to reach a camera-visible receiver in
the relevant cascade and overlap, with filter/bias padding. Geometry, materials,
cascade transforms/resolution and shadow filtering stay unchanged. Unknown
bounds, active grass deformation and unsupported filters retain Bevy's stock
lists. Disabled shadow lights skip the receiver scan entirely.

All eight geometry/cache/production-system regressions pass. Fresh native
stock/filtered pairs cover both maps at the ordinary camera and rotated, zoomed
views (`NewBarkTown --orbit 2 --zoom 2`, `Route29 --orbit 5 --zoom 2`), all at
12:00 and 1180x812. Each filtered capture was inspected against its stock and
paired 2D views. Differences were confined to animated actors, actor shadows,
water and wind; scenery shadows stayed intact. Default-view observations
reduced New Bark submissions from 14,010 to 9,124 and Route 29 from 13,413 to
9,203. These are caster-list reductions, not an FPS percentage.

`CRYSTAL_SHADOW_RECEIVERS=off` restores stock lists for comparisons.
`CRYSTAL_SHADOW_RECEIVER_TRACE=1` opts into the existing bounded-rate console
report; normal play and the on/off optimization override emit no report. Earlier
opt-in checkpoints below document the investigation before native enablement.

The new native default also passed production `RLRL` input in both maps without
a shadow opt-in environment variable: 128 updates each, five/six authoritative
tiles, only the initial foreground terrain build, active filtering and zero
shadow console reports. Final caster candidates were 14,010→9,124 in New Bark
and 13,630→9,251 in Route 29. Each final image was inspected against paired 2D
at preserved game state. The New Bark stock/filtered camera comparisons retained
784,830 and 882,185 exactly matching pixels outside explicit actor/water regions.
The rebuilt native app and its external pack passed installation checks.

Those movement captures still had slow frame intervals (about 57/61 ms medians)
with the synchronous one-shot renderer and other local workloads. They prove
default behavior and work reduction, not attainment of the requested fluidity.

## Bounded night window-light evaluation

Night materials now carry the populated emitter span in the previously reserved
`city_night.y` component. The fragment shader evaluates only that prefix rather
than scanning all 32 slots per pixel. Interior holes, emitter order, finite-area
light integration, pane emission, gain and uniform layout remain unchanged.
Empty emitter sets retain night pane emission while taking zero receiving-light
loop iterations. This avoids empty work on routes and sparse exterior scenes.

All seven city-light regressions pass. They cover holes through the last slot,
late-created materials, night/day transitions, empty emitter sets, map clearing,
separate window rectangles and authored kit/pane fidelity. Offscreen native
Route 29 captures at 22:00 compared the original 32-slot shader with the bounded
shader: 958,076 of 958,160 pixels matched exactly (99.991%); the remaining 84
pixels were around animated scene elements. New Bark at 22:00 retained its
window emission and illumination. Both current 3D scenes were inspected against
paired 2D at preserved game state.

These checks prove removed empty loop iterations and presentation fidelity,
not an end-to-end FPS gain. The native headless `--measure` path currently
rejects operation without a one-shot image screenshot, so it was not used to
claim timing results. No timing tool changes were made for this optimization.

## Incremental tilt-shift Gaussian

The two world-focus passes now advance paired Gaussian coefficients by
multiplication, using two initial exponentials per blurred pixel rather than
re-evaluating two exponentials per tap pair. The original three-sigma tail,
normalization, linear-filtered paired offsets, focus band, sample support and
alpha preservation remain intact. The recurrence follows the
[GPU Gems incremental Gaussian method](https://developer.nvidia.com/gpugems/gpugems3/part-vi-gpu-computing/chapter-40-incremental-computation-gaussian).
There are no additional buffers, uploads or passes.

An offscreen native GPU regression renders both production shader variants
through both filter passes, reads the resulting pixels back, and compares fine
color/alpha edges at 127x65, 320x257 and 1184x812. Cases cover shifted and
out-of-frame focus, 3px/36px diameters and a larger 64px custom blur. Maximum
8-bit channel error must be at most one and mean error below 0.01. The GPU test
and all ten existing quality/focus tests pass.

Alternating warm batches on this Mac showed about a 2–4% reduction for the
normal town diameter at 812px framebuffer height (8.04px after the readable
exterior scale). The smallest 3px case was essentially unchanged. At the 36px
maximum, initial batches showed 7–9% reduction, with later checks varying from
roughly 1–7% under changing local GPU load. Absolute savings are a few hundredths
of a millisecond for this effect. These are isolated batch wall times including
GPU completion, not whole-game FPS. A tested row-storage compute alternative
was slower here and was removed. A production New Bark capture was inspected
against paired 2D at identical game state; player/scenery focus and composition
remained intact. The native app and external pack passed installation checks.

Run the native check without opening a window:
`cargo test --locked --profile wasm-dev -p crystal-voxel-view
incremental_gaussian_preserves_pixels -- --ignored --nocapture`.
`CRYSTAL_TILT_SHIFT_KERNEL=direct` selects the original shader for comparison.

## Return-door character reuse

A completed destination character cache now exchanges assets with the active
room instead of replacing the spare cache after arrival. The previous room's
original meshes, materials, paintings and inverse-bind assets remain available
for its return doorway. This still uses one active asset set and one spare;
choosing a different destination retires the old spare normally. Unfinished
preloads retain the existing partial-adoption path and cannot displace the
current room's people.

The round-trip regression prepares the real player, Mom and Fisher, exchanges
house/town caches in both directions, and requires zero new return preparation
jobs and no mesh, material, image or skinning asset growth. All four actor-worker
regressions pass, including partial preload protection and job cancellation.

The production offscreen route `BBBBBBDBBBBBBU`, starting at PlayersHouse1F
(6, 6), visited NewBarkTown and returned to PlayersHouse1F (6, 7). Each of the
six required character appearances was prepared and registered once. Neither
operation repeated on return, and the foreground terrain-build counter remained
at its initial value. Paired captures preserved the checksum and showed the
same player/NPC positions and source room layout. The desktop bundle was rebuilt,
its external pack validated, and the app installed without launching a window.

This run still had long render-submission tails. Its median main schedule was
about 9 ms; the synchronous offscreen render graph's submission envelope had a
roughly 241 ms p95. These are overlapping CPU schedule envelopes under other
local workloads, not GPU timestamps or production FPS. Return-asset reuse is
verified; broad frame fluidity remains unproven.

## Progressive destination character uploads

Destination people now register their completed production meshes, paintings,
materials and inverse-bind assets before the door is crossed, at most one
appearance per host frame. This uses the same upload function as ordinary
character spawning; it creates no entities or simulation state and never falls
back to body refinement on the frame thread. Arrival adopts those original
handles, including when only part of the destination has been published.
The destination remains a single bounded cache and retires roles no longer
requested. Changed source identities resume preparation.

The regression uses actual Elm/scientist appearances, checks the one-appearance
publication limit, then spawns both characters and requires all body/material/
skinning handles and asset counts to remain unchanged. Empty staging must
publish nothing. This moves asset registration/upload ahead of arrival; it does
not itself precompile every view-dependent graphics pipeline.

## Hidden destination scenery

Cached destination and previous-room scenery now suspend camera-obstruction
queries, deferred collision-tree construction and fade updates while their
visibility hierarchy is hidden. Visibility is checked from current components
so a doorway promotion rechecks obstruction on its first visible frame, even
without camera movement. Explicit visible children retain Bevy's override
semantics. The 43 obstruction/fade regressions pass, including nested hidden
parents, unchanged hidden component ticks and mesh counts, and correct resume.
The production house-exit route still reused terrain with zero new foreground
builds, and paired 3D/2D captures preserved game state. A cold screenshot capture still
stalled at arrival (1.82 s versus 141 ms on repeat); the repeat trace showed
preparation finishing indoors and small arrival CPU work. The capture path
forces synchronous pipeline compilation, whereas ordinary play uses Bevy's
asynchronous default. Those capture times therefore do not establish the
production cold stall or prove that the entire doorway delay is solved.

## Retained water animation textures

Animated water phase masks now use retained linear RGBA8 GPU storage. Source
phase updates patch the existing texture rather than allocating a replacement.
The offscreen GPU regression checks unchanged texture and view identities,
reads the updated pixels back byte-for-byte, and requires an actual format
change to allocate the matching replacement texture. Mask sampling remains
nearest and linear; source phase, role and shoreline bytes are unchanged.

Streamed terrain atlases also skip the material-user scan on texture phases.
The ordinary image fallback explicitly emits Bevy 0.20's lazy `AssetMut`
modification notification when a replacement really needs material rebinding.
Six terrain-animation tests cover that distinction and preserve flower,
pillar, water and lava behavior; seventeen water tests and the shared render
API tests pass. A production New Bark capture was checked against paired 2D.
These checks prove GPU storage reuse and presentation correctness, not a
whole-game FPS figure.

The GPU check is offscreen and opt-in:
`cargo test --locked --profile wasm-dev -p crystal-render-api
linear_phase_mask_keeps_gpu_storage_and_exact_updated_bytes -- --ignored`.


## Doorway preloading

Native 2.5D play prepares an indoor map's exterior while the current room is
playable. Outside, it prepares the nearest authored doorway within eight
walking tiles, covering entry into buildings as well as exits. The destination
comes from the pack's actual warp graph, including both floors of the player's
house leading to New Bark Town. Source selection
is shared with the production compositor. The normal terrain builder prepares
and uploads one hidden destination, including nearby streamed groves; the door
promotes those assets instead of rebuilding them. An unfinished preload is
adopted on arrival. Its original source frame travels with the asynchronous job,
independent of the host's next doorway request. A resident base scene is also
promoted while grove pages are still settling; the active streamer finishes
those pages. Both paths keep the existing work instead of rebuilding New Bark.
Adoption preserves the foreground build counter rather than counting the
speculative job as a new doorway submission. The previous room occupies the same single spare slot for
return travel. Changed blocks, palettes or geometry profiles invalidate reuse. Completed
preloads stop resynchronizing: hidden roots retain their visibility change ticks
and no grove scan or terrain build repeats on idle frames. Camera changes and
outdoor caster-footprint changes resume the necessary preparation; daylight
motion does not wake an indoor destination without streamed groves.

Verified native house-exit input at (6, 6): the former path started one new
terrain build after crossing the door; the preloaded path started zero and had
New Bark terrain installed on the first outdoor frame. The outdoor fade
completed during the input route. A paired 3D/2D capture retained the same
position and game state. Regressions cover both house floors, exact production
tiles/pixels/ledges/heights, unchanged game checksum, recolor rejection and
promotion of existing scene handles.

A second real-input route starts in New Bark Town at (6, 5), approaches Elm's
Lab and enters its ordinary introductory scene. It crosses into the lab with
zero additional foreground terrain builds. Paired captures and a direct lab
load confirm that preloading retains the room's geometry and presentation.
Source regressions select the lab, player house and neighbor house from their
actual nearby warp positions.

The destination also carries resolved actor source identities. Native workers
prepare the required human rigs, paintings and complete appearance meshes
before entry. Arrival consumes the same prepared meshes rather than starting
another character job; already uploaded common roles are shared only when the
destination needs them. One destination owns the speculative staging cache.
The real lab-entry route now reports no pending preparation on any published
lab frame, including the first one, and still reaches Elm's ordinary dialogue.
Actor regressions cover adoption without foreground jobs, cancellation, worker
limits and retirement of unused uploaded characters. Paired 3D/2D captures
retain the room and dialogue state.


Browser speculation requires the existing terrain worker; it never runs a
blocking speculative mesher on the browser frame thread. Browser builds without
that worker retain their normal loading path.


## Source palette conversion, October 19

Cave sky fill and animated water palette sampling now share a lazy 256-entry
linear-color table. Each byte channel retains the exact bits of the previous
sRGB formula; source images, lighting and water colors remain unchanged.
An optimized native CPU microbenchmark on the local Mac measured 25.6 million
channel conversions at 87.07 ms for the formula and 9.04 ms for the lookup.
These are conversion timings, not whole-frame or transition measurements.

Run the exhaustive equivalence test with `cargo test --locked -p
crystal-voxel-view source_color`. The opt-in microbenchmark is
`cargo test --locked --release -p crystal-voxel-view
source_color::tests::palette_conversion_timing -- --ignored --nocapture`.
The benchmark runs without a renderer or visible window.

Measurements below use the same native game, Cherrygrove (29,4), camera zoom 1,
orbit −1, 1180×812 window, optimized development build, and LLVM 19 llvmpipe
software OpenGL adapter. Compilation and other heavy jobs were paused during
comparison. These are local application-frame timings, not hardware-GPU claims.

| Configuration | Median | p95 | Updates / elapsed |
| --- | ---: | ---: | ---: |
| Full-detail foliage + dynamic shadows, no screenshot readback | 103.98ms | 129.34ms | 300 / 32.09 s |
| Adaptive foliage + software-renderer contact-shadow profile, no readback | 45.21ms | 54.02ms | 687 / 32.03 s |
| Adaptive profile with bounded pipelined capture | 50.50ms | 59.36ms | 622 / 32.15 s |

The final row retained 618 actual source frames, about 19.2 captured frames/s.
The previous recorder retained 123 frames over 40 s, about 3.1 frames/s. Capture
has no motion interpolation or artificial speedup; variable timestamps preserve
real wall time. In those six original outdoor maps, known software adapters use baked contact
grounding instead of rerendering the broad terrain halo into a dynamic shadow
map; hardware adapters
retain dynamic shadows. A native-only `CRYSTAL_MODELED_SHADOWS=on|off` flag
supports controlled comparisons.

## Real movement defect fixed

Full rendering included NPCs in the expanded visual-world halo, but retained
movement updates checked the smaller classic halo. A roster mismatch returned
before updating scrolling transforms, and the fallback silently acknowledged
stale presentation state. This produced visible450–530 ms holds in the earlier
native trace, then large position jumps. It also reproduced at 60 Hz.

The retained path now uses the same expanded halo as full rendering, and failed
updates force reconciliation. The real-pack regression observes the published
`VisualWorldFrame` at 60/30/9 Hz, verifies every moving sample against the current
interpolation, and checks both camera and map-relative player displacement.
The original test failed at 60 Hz host frame 6; the fixed test passes at all three
cadences, without duplicate moving samples.

Measured published player speeds were 7.4659, 3.7159 and 1.1354 gameplay tiles/s at
60, 30 and 9 Hz. Existing host walk-timer catch-up policy is unchanged, including
its frame-cadence-dependent speed at slow update rates. Scripts, collisions,
warps, authoritative tile coordinates and gameplay input ownership are unchanged.

## Character gait

Gait now tolerates sparse publication without falsely restarting idle on each
held sample. Foot contact/return velocity, pelvis movement and turning are smooth.
Slow walking uses its original grounded two-bone IK; high-speed travel blends
into a longer 2.8-tile run cycle with shorter planted contact and a real aerial
phase. Limbs never scale and the root remains exactly at the published foot.

At actual 30 Hz production pace, measured footfalls dropped 6.636→2.654/s; at 60 Hz,
13.332→5.333/s. Maximum sampled joint changes dropped about 50°→24.6° per frame.
Twenty actor/prop/locomotion tests cover reachability, stance anchoring, transitions,
phase continuity, sparse frames, source changes and camera independence.

## Scenery costs

Near foliage retains full geometry. Outer-halo trees and grass use authored,
silhouette-matched LODs with the same source bounds, palette and ground origin.
A 68×66 stress viewport (384 trees, 1,024 grass cells) dropped from 1,131,508 vertices /
406,842 triangles /59.2 MB to 558,364 /205,082 /29.3 MB. Median mesh construction across
three runs dropped 115.74→84.22 ms. No scene objects were removed.

Contact-shadow lookup now uses spatial buckets. An independent 179,520-sample
benchmark dropped 879.59→29.93 ms, with bit-identical resulting colors. This reduces
terrain rebuild cost, separately from steady-state frame rate.

Use `CRYSTAL_SCENERY_METRICS=1` for native geometry/build diagnostics and
`CRYSTAL_SCENERY_FULL_DETAIL=1` to compare the original full foliage geometry.
Use `--measure <empty-directory> --seconds 32 --walk LLRR` for a no-readback trace,
or `--record` for actual framebuffer captures. No captures are CI inputs.


## Gym scenery checkpoint, October 1

Stationary native checks used the same default camera, 1180×812 window, optimized
development profile and llvmpipe adapter. Each run measured twelve seconds after
initial settling, without framebuffer readback. Cargo, Blender and other heavy
jobs were paused. This is the cost of the real scene, including its actors.

| Scene / build | Median | p95 |
| --- | ---: | ---: |
| Viridian, published baseline (paired rerun) | 55.45 ms | 65.59 ms |
| Viridian, first authored kit | 77.07 ms | 90.43 ms |
| Viridian, optimized authored kit | 68.09 ms | 79.81 ms |
| Goldenrod, published baseline (paired rerun) | 85.73 ms | 100.35 ms |
| Goldenrod, first authored kit | 118.61 ms | 130.57 ms |
| Goldenrod, optimized authored kit | 94.06 ms | 105.69 ms |

The first comparison caught excessive small bevels repeated across planters and
maze faces. The corrected exports retain all 364 closed components, complete
bounds, foliage and major silhouettes. Bevels narrower than 0.03 model units are
squared, and exact per-material attribute indexing preserves the expanded
triangle stream. The twenty prototypes dropped from 12,620 to 6,220 triangles
and 37,860 to 14,326 vertices. Actual production geometry is now:

| Scene | First-kit solid triangles / vertices | Optimized solid triangles / vertices |
| --- | ---: | ---: |
| Goldenrod | 187,944 / 559,580 | 107,304 / 261,896 |
| Viridian | 79,584 / 235,904 | 37,344 / 75,152 |

The final paired rerun still costs 22.8% more median frame time in Viridian and
9.7% more in Goldenrod than the published source-art baseline. This regression
remains an optimization task; the new scenes are not certified fluid on this
software renderer. Additional vertex/fragment work and cutaway eligibility are
candidates for controlled measurement, not established causes. The two initial
baseline medians were 53.27 and 84.93 ms, consistent with the rerun but not identical.
The full per-frame traces and captures remain ignored local QA artifacts.

## Reveal-aware scene submission follow-up

These are historical measurements of the earlier capsule reveal. The current
[whole-object translucency](occluder-translucency.md) keeps covered joins in their
source-owned wall groups and no longer registers this secondary-draw selector.

A second paired twelve-second, no-readback run used the same llvmpipe adapter,
window and camera. Its baseline is the optimized authored kit above, not the
older source-art build. These are single paired samples, with raw per-frame
traces retained locally.

| Scene | Prior optimized kit, median / p95 | Reveal-aware candidate, median / p95 |
| --- | ---: | ---: |
| Viridian | 69.23 / 78.39 ms | 60.62 / 68.22 ms |
| Goldenrod | 89.82 / 105.96 ms | 95.03 / 110.85 ms |

The Viridian improvement is about 12.4% in this pair. Goldenrod did not improve;
its remaining cost and the earlier source-art comparison remain open. These
scenes are still not certified fluid on the software renderer.

The public terrain mesh retains every maze face. At terrain construction, the
runtime partitions source-proven covered joins into cached secondary draws.
Selection conservatively restores joins whose projected bounds overlap the
player's reveal capsule, including faces behind the player. It uses the shared
material uniform and current propagated camera/terrain transforms, before
visibility propagation. Uncertain projection restores all candidate faces.
Camera/player movement changes visibility, without rebuilding or uploading
geometry or mutating the material each frame.

Keeping the secondary faces matters: deleting them permanently exposed missing
stone courses inside the reveal. Six matching native camera/position views now
retain that geometry, and the previously failing stone patch matches the
original pixels exactly. Animated actors and isolated raster edges still vary
between captures; this is not a claim of whole-frame pixel identity.

Separately, the two planter imports omit only faces proven strictly inside an
opaque, closed convex component. Coplanar, partially covered, nonconvex and
transparent parts are retained. Full bounds and every retained indexed corner
remain unchanged. Goldenrod solid geometry falls from 107,304 triangles /
261,896 vertices to 98,867 / 244,077; that reduction has not established a
frame-time benefit in these measurements.

Those earlier `CRYSTAL_CUTAWAY_DIAGNOSTIC=zero-radius` / `omit-uv1` runs did not
establish capsule arithmetic as the main cost. In the current renderer,
`zero-radius` disables blocker detection and `omit-uv1` removes only the legacy
eligibility attribute; the forward fade shader no longer reads UV1. Unset/normal
and browser builds use ordinary whole-object fading. Disabling reveal is not an
optimization used by the game.

## Facility furniture and ship floor finishes

A paired twelve-second run compares the published institutional checkpoint
(`63579d8`, remote `f54f1d8`) with four source-specific facility furniture models
and the restrained ship floor materials. Both builds use the optimized native
profile, the same 1180×812 window, llvmpipe, default positions and orbit −1.
No screenshot/recording readback or other build/render workload ran during the
measurement windows. These are single paired samples, not a hardware benchmark.

| Scene | Published checkpoint, median / p95 | Furniture/floor increment, median / p95 |
| --- | ---: | ---: |
| Fast Ship B1F | 85.74 / 97.71 ms | 81.97 / 91.06 ms |
| Power Plant | 64.56 / 75.77 ms | 65.04 / 75.51 ms |

The ship sample is modestly faster; Power Plant is approximately unchanged.
This visual increment does not resolve the low software-renderer frame rate.
The floors partition existing coplanar surfaces, with at most ten triangles
per finished source cell and no per-frame geometry/material reconstruction.
The four new furniture meshes total 2,784 authored triangles before placement.

## Ship room-front wall correction

The connected B1F fronts add closed source-shaped wall shells where flat
black/blue strips remained. Their dark steel crowns and ivory shoulders are
fitted with all adjacent B1F wall pieces to a shared 28px display height; the
original floor, source collision and actor support remain unchanged.

Two 12-second native no-readback runs use the same 1180×812 window, default B1F
player position, orbit −1 and llvmpipe adapter. The new closed fronts with 16px
walls measured 95.66ms median/113.26ms p95; the 28px fit measured 94.75/110.77ms.
No extra mesh topology is introduced by the height fit. These single samples
show no additional measured height cost, but they are not a general speedup.
The prior published floor/furniture checkpoint measured 81.97/91.06ms under
the same setup. The current scene remains slower and does not meet a fluid
frame-rate target. Static terrain submission/culling remains active work.

Ordinary held keyboard input also passed both room entrances with zero
additional terrain builds. The east route then reached a trainer sight event
and displayed its source dialogue. Movement capture includes GPU readback and
is not the static timing evidence above.

## Static terrain batches and native camera controls

Large static textured/solid surfaces now use bounded spatial draws. Each entire
triangle keeps its exact positions, normals, UVs, colors and wall-reveal bits.
The partition does not remove faces or source objects. Bevy applies its normal
camera and shadow-frustum tests to each draw; off-camera shadow casters remain
eligible for their light. Small or uncertain inputs stay in one mesh.

Regions are nominally 16×16 source cells, capped at eight per axis and 64 per
material domain. Mesh partition and full bounds are prepared once on the terrain
worker. Each revision replaces the old hierarchy and bounds; retained frames
reuse meshes/materials and only update the root transform. Deferred new roots
receive the live scrolling offset before their first draw.

Twelve-second no-readback samples below retain the same optimized native build,
1180×812 window, default map position, orbit −1 and llvmpipe renderer. `off`
uses the new hierarchy with one static draw per domain; `on` partitions it.
Heavy local work was paused. The second ship run reverses the comparison order.

| Scene / trial | Original median / p95 | Candidate off | Candidate on |
| --- | ---: | ---: | ---: |
| Fast Ship B1F / 1 | 97.26 / 113.26 ms | 101.00 / 146.67 ms | 93.23 / 111.51 ms |
| Fast Ship B1F / 2 | 96.91 / 112.47 ms | 91.67 / 107.74 ms | 91.42 / 108.99 ms |
| Viridian Gym | 60.21 / 70.31 ms | 62.38 / 71.15 ms | 60.52 / 72.31 ms |
| New Bark / 1 | 89.75 / 113.32 ms | 88.96 / 102.51 ms | 78.79 / 88.31 ms |
| New Bark / 2 | 97.05 / 116.06 ms | not repeated | 79.50 / 96.82 ms |

New Bark's two medians fall by about 12–18% against their unchanged baseline
samples. The ship gain is small and the Gym is approximately unchanged. These
results do not establish a universal improvement or a fluid frame rate, and
hardware-GPU performance has not been measured.

The full 811-test voxel suite passes, with two existing benchmarks ignored.
Tests retain exact triangle-corner/cutaway data, bounded fallback behavior,
visible samples across 112 camera arrangements, fresh revision bounds,
visibility inheritance, retained handles and first-draw scroll placement.
Sixteen paired native views cover five maps, both ship-wall sides, New Bark
and Viridian reverse views, and the unchanged busy Power Plant/Goldenrod
surfaces. Scrolling crosses five grid origins without rebuilding terrain;
a real B1F ladder transition changes maps with one expected rebuild. Native
F3 roundtrip and viewport resize retain the scene. Visual review found no
new seams, missing boundaries or coplanar-order defect in these cases.

Native Q/E and PageUp/PageDown input now works across active 3D maps. The old
single-map art-preview gate blocked those advertised controls outside New Bark.
A real ship session verifies orbit/zoom and their reverse inputs; a regression
covers four map families, one-step edge handling and hidden-world rejection.
Browser controls and gameplay state remain unchanged.

### Shadow diagnostic, with defaults preserved

The historical software contact-shadow profile applies to the six original
outdoor maps. Other maps still use dynamic shadows. Native diagnostic overrides
now work on all maps: `CRYSTAL_MODELED_SHADOWS=on|off`, and
`CRYSTAL_MODELED_SHADOW_RESOLUTION=512|1024|2048` (default 2048).

| Scene | 2048 median / p95 | 1024 | 512 | Shadows off |
| --- | ---: | ---: | ---: | ---: |
| Fast Ship B1F | 90.17 / 102.27 ms | 87.39 / 100.03 ms | 94.09 / 116.08 ms | 51.93 / 61.12 ms |
| Goldenrod Gym | 85.88 / 104.97 ms | 83.94 / 95.41 ms | 89.85 / 111.15 ms | 51.48 / 58.78 ms |

Removing the pass has a large cost benefit but visibly changes the lighting and
cast-shadow treatment. It is not a like-for-like graphics optimization. Smaller
maps provide no clear gain in these samples. Neither diagnostic changes the
shipping default, and the shadows-off figures are not the normal performance
claim for this increment. All original geometry and real timestamp traces remain
available for subsequent targeted work; no interpolation or time warping is used.

## Whole-section camera translucency

The current renderer replaces capsule-shaped fragment removal with source-owned
wall/roof sections that ease toward 20% opacity when their actual triangles block
the player. Empty U-room interiors do not count as walls. Multiple actors and
props behind a faded section remain visible; unblocked sections restore opaque
depth writes. Linked textured inscriptions follow their solid housings.

The terrain worker builds bounded triangle BVHs and centered draw leaves once
per revision. Each source group retains fixed opaque/blended material handles;
movement changes only fade state and changed uniforms. Physical shadow alpha
remains intact. The former capsule-only maze-join selector is no longer registered:
all original joins stay in their source-owned wall sections. Source triangles,
UVs, colors, winding and footing are preserved. The [behavior and review notes](occluder-translucency.md)
cover lifecycle and geometry regressions plus the four same-camera ship views.

This implementation previously passed 818 voxel tests, native and WebAssembly
builds, and still-image reviews across ship, lighthouse, maze, store and exterior
views. The execution workspace was replaced before that increment was published.
The recovered source replays the original authoring steps and again passes all
818 voxel tests, with two existing benchmarks ignored. Fresh native/WebAssembly
builds, motion, timing and exact-checkout visual verification remain pending;
the earlier raw traces are no longer available. Shipping shadow policy
is unchanged.

## Canonical-grove New Bark checkpoint, October 12

Native GPU comparisons can render the production cameras into an image target
with `CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`. This does not require an unlocked
desktop. `CRYSTAL_MEASURE_3D_VALIDATION=on` verifies a nonblank GPU readback before
measurement, drains thirty further frames, and performs no readbacks during the
timed interval. The receipt records the selected target. Window-only captures
that produced a black image were rejected and supply no performance evidence.

New Bark, orbit 2, 640×480 logical size (1280×960 physical readback), retained
26,592,578 vertices and 18,520,816 triangles in all three runs. These fifteen-second
application-update timings used the same frozen executable. Concurrent compilation
contaminated the measurements; they do not establish a speed improvement or a
playability claim.

These initial runs used the existing adaptive outer-halo grass setting. Canonical
grove trees retained their complete geometry. A separate full-detail comparison
must explicitly set `CRYSTAL_SCENERY_FULL_DETAIL=1`; the initial runs do not prove
full grass detail.

| Native GPU culling experiment | Median | p95 | Updates / elapsed |
| --- | ---: | ---: | ---: |
| Disabled | 53.28 ms | 197.91 ms | 199 / 15.11 s |
| Shadow views only | 58.80 ms | 105.96 ms | 233 / 15.04 s |
| Camera and shadow views | 63.04 ms | 176.34 ms | 189 / 15.03 s |

GPU culling remains opt-in through `CRYSTAL_GPU_CULLING=camera|shadows|on` and is
limited to New Bark and Route 29. Static roof, facade and grove comparison regions
matched exactly in the corresponding captures; animated regions differed. This
is evidence for those views, not a guarantee for every camera position.

Idle customization polling now reuses the revision-tracked runtime snapshot;
opening and saving retain fresh integrity validation. The external-pack regression
checks cache reuse, action validation and invalidation after saving. Atmosphere
synchronization also avoids writing unchanged light, cascade, ambient, fog and
FXAA values when source animation updates the frame. A Bevy change-tracking
regression verifies those components stay clean and that actual map lighting
changes still propagate. These fixes preserve model geometry and rendering values;
their combined native timing effect is not isolated by the current runs.

An explicitly full-detail follow-up measured the newer build at 58.26 ms median /
110.42 ms p95 (239 updates / 15.01 s), and the older build at 48.33 ms /
89.58 ms (283 / 15.06 s). Both validated actual GPU image readbacks. However,
the newer build contained 27,060,550 vertices / 18,688,198 triangles, while the
older build contained 27,022,658 / 18,664,176. Geometry changed between those
frozen builds and concurrent compiler load also changed. This is not an isolated
comparison of the lighting-update fix and establishes no performance gain.

The same newer executable also validated Route 29 at full detail: 27,993,114
vertices / 18,270,074 triangles, 1280×960 GPU image readback. Its fifteen-second
trace measured 62.90 ms median / 191.59 ms p95 (178 updates / 15.01 s) while
compilers were active. Both reference scenes therefore remain performance work,
not evidence that the application is fast enough for the requested effects.

A diagnostic New Bark run with `CRYSTAL_MODELED_SHADOWS=off` used that exact newer
executable and identical full-detail geometry counts. Its validated image-target
trace measured 15.11 ms median / 44.28 ms p95 (760 updates / 15.01 s), compared
with 58.26 / 110.42 ms for shadows enabled. Compiler contention continued in
both runs. The large difference supports prioritizing shadow submission and
rendering investigation; it does not precisely quantify GPU pass time. Shadows
remain enabled in production. Removing shadows is not the proposed optimization.

Additional haze is removed. Environmental particles remain unregistered while
rendering performance is being improved. Compact solid UV removal, shadow fitting
and GPU culling experiments remain disabled by default.

The native `CRYSTAL_SHADOW_RECEIVERS=on` experiment keeps all authored cascade
projections and meshes. After Bevy's light visibility pass, it rejects a caster
only when conservative ray/plane intervals cannot reach the camera frustum within
the complete scene's receiver-height slab. Unknown bounds retain the stock list.
Receiver normal/depth bias and the fixed shadow-filter footprint inflate the
proof bounds. The experiment is limited to New Bark and Route 29 and remains
disabled pending broader camera/movement comparisons. Two full-crate regressions
passed; all three current geometry-function tests, including rotated camera planes,
also passed in a temporary harness compiled directly from the current function.
The native build passed.

The first same-executable New Bark pair retained 27,064,458 vertices / 18,690,094
triangles in both runs and validated 1280×960 image readbacks. Baseline measured
48.13 ms median / 76.94 ms p95 (291 updates / 15.00 s); receiver culling measured
25.61 / 45.28 ms (560 / 15.01 s). The filter retained 4,700 of 14,267 cascade
submissions. Selected north/east grove, roof and facade regions matched exactly.
Animated actors and water differed between capture times. A second filtered run
measured 25.51 / 41.34 ms (581 / 15.02 s); its reversed-order baseline measured
35.79 / 49.00 ms (419 / 15.01 s). Outside four declared actor/water animation
rectangles, 1,092,850 pixels in the first pair matched exactly. Concurrent
compilers remain a timing limitation: the relative median improvement varied
from approximately 29% to 47%. This is a promising measured
candidate, not a claim that every camera or movement path has passed.

Route 29's corresponding full-detail, same-executable pair retained 27,993,114
vertices / 18,270,074 triangles. The first baseline measured 43.15 ms median /
51.17 ms p95 (348 updates / 15.02 s), versus 44.61 / 80.83 ms filtered
(313 / 15.06 s): this pair did not improve. In reverse order the filtered run
measured 32.85 / 69.34 ms (393 / 15.02 s) and baseline 61.51 / 133.02 ms
(217 / 15.01 s), with compilation active. The filter retained 4,656 of 13,185
cascade submissions. Selected grove, grass, sign and ground regions matched
exactly. Variable load prevents a firm Route 29 speed claim.

Native movement QA can explicitly use `CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic`
with `--walk`. It injects the generated key snapshot after physical-input focus
handling, before the existing production input-owner dispatcher. It does not
change actual window focus, coordinates, game ownership or controller behavior.
The default keyboard QA path still requires focus. CSV traces label the input
source, retain the real focus value, and record authoritative tile coordinates;
successful movement now requires those coordinates to change as well as visible
presentation movement. The native build passed. Both full-detail GPU movement
runs completed with `focused=false` and input explicitly labelled `synthetic`.
New Bark's 64-frame down/up route observed authoritative tiles (13,6) through
(13,10), five grid origins, and no additional terrain build; moving and final
captures were retained. Its trace measured 32.33 ms median / 66.63 ms p95.
Route 29's right/left route observed (50,9) through (54,9), five origins and no
additional build, at 71.26 / 116.13 ms under compilation load. These prove the
production input/presentation path works without desktop focus, not physical
keyboard handling while unfocused or achievement of the requested speed.

The next receiver-filter refinement restricts each cascade's candidate receivers
to Bevy's actual depth selection interval, including the preceding overlap start.
Bias/filter padding uses that cascade's texel size. Meshes, cascade matrices and
resolution stay unchanged. Four current geometry-function regressions, including
overlap and biased samples, passed; the native build passed. Broader camera
comparisons of this refinement remain pending, and it stays opt-in.

Its first New Bark pair retained 27,065,738 vertices / 18,690,734 triangles in
both runs. Validated GPU image-target timings measured 60.34 ms median / 102.70 ms
p95 baseline (236 updates / 15.01 s), and 32.87 / 63.68 ms filtered
(445 / 15.03 s). Cascade submissions fell from 14,267 to 4,542. Outside the same
actor/water animation rectangles, 1,092,850 pixels matched exactly. Compilation
remained active. This supports the candidate but does not isolate its incremental
gain over the earlier global receiver filter or prove the final speed target.

These native image-target traces ran with the real window unfocused and the
production low-power unfocused update policy. They are application-update timings,
not focused-window FPS or direct GPU pass timings. The current WebAssembly
`cargo check --locked --target wasm32-unknown-unknown -p crystal-bevy --features location-tester`
passed after excluding the native synthetic-input hook from browser compilation.

The refined filter's full-detail New Bark down/up GPU movement run passed: 64
frames, five grid origins, authoritative tile changes, and no additional terrain
build. Actual window focus remained false. Moving and final image readbacks
rendered successfully. The movement trace measured 25.27 ms median / 58.19 ms p95;
this remains an unfocused, contended application trace rather than a final
focused-window performance certification.

Route 29's refined-filter down/up GPU movement run also passed: 64 frames,
authoritative tiles (50,7) through (50,11), five grid origins, and one initial
terrain build with none added during movement. The trace retained actual
`focused=false` and `input=synthetic`. The moving capture was visually inspected;
the final image readback was validated. Median/p95 application-update times were
35.38 / 73.71 ms under contention, with 13,121 cascade submissions reduced to
4,326. This remains a movement and rendering check, not a firm speed result.

The next CPU refinement caches a caster decision only while its exact world
bounding-box corners and all receiver planes, sun direction, height limits and
bias padding remain equal. Changes recompute the decision; unseen caster and
cascade entries are removed. Five current function/cache regressions passed,
including movement and receiver-input invalidation, and the voxel-view native
check passed. The frozen native-v7 executable's alternate New Bark camera angle
retained the same 27,065,738 vertices / 18,690,734 triangles in both runs.
Baseline measured 50.80 ms median / 166.24 ms p95 (206 updates / 15.09 s);
cached receiver filtering measured 26.30 / 125.67 ms (363 / 15.01 s).
Submissions fell from 14,121 to 4,750. Sampled frames reused over 14,100
decisions while recomputing 6–10 moving-caster decisions. Filter-system elapsed
time ranged from 2.060 to 5.698 ms in those samples, including preemption.
These results compare the complete cached filter with stock culling; they do
not isolate the cache's incremental speed benefit. Roof, facade, north/east
grove and ground regions matched exactly. After excluding explicit actor,
actor-shadow and water-animation rectangles, 1,125,650 pixels matched exactly.
The captures were visually inspected. Native unfocused policy and concurrent
compilation still limit any claim about final playable frame rate.

The same executable's alternate Route 29 angle retained 27,993,114 vertices /
18,270,074 triangles in both runs. Filtered-first measured 49.99 ms median /
85.67 ms p95 (289 updates / 15.11 s); the following baseline measured
58.10 / 97.72 ms (235 / 15.04 s). Submissions fell from 13,119 to 4,753.
Outside explicit actor and actor-shadow rectangles, 1,184,450 pixels matched
exactly; the filtered capture was visually inspected. This pair shows a modest
gain under contention, while earlier Route 29 measurements remained variable.

Validated native timing now accepts opt-in
`CRYSTAL_RENDER_TEST_UPDATE_POLICY=focused`, which copies the existing focused
Winit update mode to its unfocused mode for the benchmark. Actual window focus,
input ownership and the fixed game tick remain unchanged. The default is
`production`, preserving the low-power unfocused policy. Results record the
requested policy, fixed tick, actual focus and both actual update modes.
The native check passed; its regression and runtime build are pending. These
application-update measurements still require GPU completion evidence before
they can establish GPU throughput, especially for an image target without
window presentation pacing.

Validated timing now observes native queue-completion callbacks after the render
graph's submissions. It polls without waiting and records submitted/completed
counts at both ends of the ready interval, outstanding work, poll errors and
completion rate in `gpu-completions.txt`. This is completed-work throughput,
not GPU timestamp duration or display-presented FPS. The actual callback and
report functions compiled against the cached Bevy library; the receipt's ECS
regression passed in an extracted harness, verifying that warmup counts are
excluded and outstanding work is retained. The extracted update-policy test
also passed. Full-crate checks and a GPU runtime observation remain pending.

`cargo test --locked -p geothite` passed all thirteen tests, including authoritative
MCP movement and the production Start menu, and the spawned stdio regression that
enters Mom's dialogue with byte-for-byte empty stderr. This verifies those gameplay
paths; it is not a claim of graphical movement performance.

## Fresh shader-complete native checks

One frozen Windows DX12 executable, full-detail scenery, Radeon RX 7900 XTX,
640×480 logical / 960×720 physical image target, focused update policy, and
15-second intervals produced these results. Host compiler contention remained.
A warmup GPU readback was inspected before timing; no readback ran during timing.
Timing warmups now compile shader pipelines synchronously: earlier warmups could
accept terrain before character-body shaders finished, invalidating that comparison.

| Scene | Receiver filter | Median update | p95 update | Completed submissions/s |
| --- | --- | ---: | ---: | ---: |
| New Bark | off | 15.28 ms | 49.04 ms | 49.656 |
| New Bark | on | 13.59 ms | 36.24 ms | 60.750 |
| Route 29 | off | 5.59 ms | 9.07 ms | 163.682 |
| Route 29 | on | 5.46 ms | 9.43 ms | 165.318 |

All four queue receipts recorded zero poll errors; outstanding work was 1→1,
1→2, 4→5 and 1→1 respectively. These are completed-work observations and CPU
updates, not GPU timestamp durations or displayed FPS. The fixed game tick
remained 0.016742706 seconds. Full-detail warmup images contain terrain and
complete characters. Receiver filtering remains opt-in; these results do not
establish a final browser-performance claim or authorize the unfinished effects.

Earlier full-detail browser probes failed to produce playable observations while
their renderer remained CPU-active, with roughly 6.6/6.4 GB private memory.
Cooperative grove construction and staged uploads subsequently completed both
outdoor scenes with production walking. Steady rendering remains too slow:
the opt-in retained-binding Route 29 diagnostic measured 71.6/107.5 ms median/p95
rAF intervals, with both binding caches hitting on every steady frame. These are
not GPU timings or displayed FPS. The browser walking probe now
preserves renderer errors, supports muted Edge runs and checks full-detail readiness.
Its host deadline and owned-browser cleanup passed an explicit short-deadline check.

The same executable's production synthetic-input `RLRL` checks passed in both
maps: 128 frames each, five distinct authoritative tiles and five grid origins,
with only the initial terrain build. Inspected movement/final images retain
complete characters and scenery. Movement update traces under contention were
31.91/76.80 ms median/p95 in New Bark and 15.51/35.79 ms on Route 29. These are
movement observations, not FPS; stationary throughput does not erase that cost.

## Route clarity and connected-area preparation (macOS)

Route woodland fog now has at most 12% opacity and starts 16 source tiles
beyond the camera target, ending at 40 tiles. Dedicated forest atmosphere is
preserved. Paired production 2D/3D captures at noon were inspected on Route 29
(50,9) and Route 26 (10,54), at 1280×800. Nearby paths and landmarks remain clear.
Woodland density and sky sectors retain source-keyed results; source animation
updates skip unchanged phases while still resynchronizing replacement atlases
and re-entry into 3D.

Outdoor lookahead prepares the nearest connected boundary within 24 core tiles,
as well as nearby doors. Its source snapshot is immutable; arrival prediction
must not advance gameplay. Small tangential movements retain an in-flight
arrival anchor. This improves preparation lead time without adding another
resident hidden scene.

An offscreen production synthetic walk east from Route 29 (58,9), using
`render_at_location --map Route29 --x 58 --y 9 --hour 12 --view 2.5d --walk RRRR`,
crossed into New Bark Town. All 128 recorded frames retained active terrain,
with no pending terrain, no fade, and only the initial terrain build. The
`Wait, RENDER!` authored scene still executed. This verifies that crossing's
loading behavior; neighboring building/tree style changes at map ownership
boundaries remain a separate visual continuity limitation.

Timing uses the native `new_bark_3d` measurement harness, image target, focused
update policy, 1280×800, noon, 15-second ready intervals, Apple M5 / Metal.
Set `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`,
`CRYSTAL_RENDER_TEST_UPDATE_POLICY=focused`, and
`CRYSTAL_MEASURE_3D_VALIDATION=on`, then run:

```sh
cargo build --locked -p crystal-bevy --features location-tester --example new_bark_3d
./target/debug/examples/new_bark_3d content-packs/core-modular.browser.crystalpack \
  --map Route29 --x 50 --y 9 --hour 12 --size 1280x800 \
  --measure target/route29-measure --seconds 15
```

The output directory must be fresh. Warmup validates a rendered image; timed
intervals omit image readback. Headless measurement keeps pipelined rendering
and uses an unthrottled schedule, avoiding the screenshot timer's artificial
62.5 Hz limit. Queue callbacks report completed submissions per second, **not
display-presented FPS or GPU timestamp duration**.

Early heavily contended checks measured 8.512 completed submissions/s on Route
29 (106.24 ms median / 214.06 ms p95 application intervals) and 10.175/s on Route
26 (86.17 / 208.31 ms). These establish poor throughput, not a controlled
before/after comparison. Camera occlusion culling fell to 5.940/s on Route 29
and remains disabled. Adding Route 26 to the conservative shadow receiver
filter produced inconsistent off/on/on/off results: 19.794, 12.455, 18.524,
16.702 submissions/s. That experiment was removed. No stable performance gain
or target framerate is claimed from these contended comparisons.

The external-pack destination regressions passed all three checks: both house
floors, doorway approach, and actual controller crossings. Route 26 crossed
west from (0,94) into Route 27 (79,4); Route 29 crossed west from (0,7) into
Cherrygrove City (39,7). Predicted destinations matched authoritative controller
results, and prediction left the gameplay checksum unchanged. Eleven sky
checks, seven terrain-animation checks, and the retained-water-palette check
also passed.

The rendered Route 26 (1,94) `LLLL` crossing into Route 27 likewise passed:
128 frames, five grid origins, one initial terrain build, zero additional
builds, terrain active throughout, no pending terrain, no fade, and no warp
phase. Its inspected image retained complete player/follower geometry. Under
concurrent compilation, movement intervals were 62.24 ms median / 185.84 ms
p95; these are movement observations, not stationary FPS.

### Adaptive decorative woodland floor

The ordinary scenery mode now retains at most one original understory plant
per source-anchored 2×2-cell patch. This bounds decorative mesh/entity work
without moving or simplifying any retained plant. Every tree, gameplay grass
cell, object and collision remains unchanged. Dedicated ancient forests keep
their dense floor. Source-cell selection is stable across chunk splits.
`CRYSTAL_UNDERSTORY_FULL_DETAIL=1` restores only the old decorative floor for
native A/B timing; `CRYSTAL_SCENERY_FULL_DETAIL=1` also retains it alongside
that mode's existing scenery policies.

The adaptive-cover check verifies a strict subset of original plants, at most
one draw per aligned patch, deterministic selection, and more than two-thirds
fewer decorative draws across 1,024 source patches. The chunk-splitting
invariant passes in both density modes. All nine forest-library checks passed
with adaptive cover and again with `CRYSTAL_UNDERSTORY_FULL_DETAIL=1`, including
cached/worker agreement and unchanged tree triangles and placement.

One frozen optimized native executable then compared only decorative floor
density, with the same settings and 15-second intervals described above. Run
order was Route 29 dense/adaptive, then Route 26 adaptive/dense. Every timed
row retained its requested route; battle timing files contained no samples.

| Scene | Decorative floor | Median update | p95 update | Completed submissions/s |
| --- | --- | ---: | ---: | ---: |
| Route 29 (50,9) | dense override | 59.77 ms | 71.93 ms | 15.992 |
| Route 29 (50,9) | adaptive default | 36.70 ms | 74.20 ms | 23.877 |
| Route 26 (10,54) | dense override | 45.75 ms | 138.55 ms | 13.498 |
| Route 26 (10,54) | adaptive default | 39.85 ms | 75.72 ms | 22.573 |

Observed queue throughput improved approximately 49% and 67%, respectively.
Outstanding submissions were 36→35, 13→8, 2→36 and 5→2 in table order; all
receipts recorded zero poll errors. This is headless completed-work throughput,
not presented FPS. Host contention remains, Route 29 p95 did not improve, and
these results do not establish smooth 60 FPS. Warmup images were validated
before timing, with no readback during timed intervals. The adaptive images
retain full trees, source grass, paths and landmarks with a less crowded floor.

Final paired 2D/3D captures were inspected again at both measured positions
with adaptive cover enabled. Paths, encounter-grass footprints, landmarks and
full character geometry remain readable. The final adaptive Route 26→27
synthetic crossing repeated all 128 frames with active terrain, zero pending
terrain, no fade/warp phase and no additional terrain build.

## Battle entry and continuous walking

The battle bridge now publishes its modeled background during the source
sliding introduction and entry narration. Source portraits remain overlays
with controller-owned clipping, offsets and lifetime. Pokémon remain hidden
until their existing reveal boundary; entry extraction does not advance the
battle. Portraits follow the chosen camera's player/enemy screen order and
restore their original transforms in classic mode. Route battle fog now shares
the field view's 12% opacity cap; dedicated forest fog remains unchanged.

A real Route 29 walking encounter exposed another source of persistent 2D
fallback: its retained metadata still named the starting cell while the
continuous avatar was already halfway toward the committed destination. The
old anchor classifier accepted a centered pre-step avatar or a committed
landing, and rejected that legitimate intermediate foot. The classifier now
accepts the actual foot on the same authenticated source corridor before cell
publication. Source map/grid/texture/revision, facing and exact corridor checks
remain; a destination endpoint cannot use stale starting-cell metadata.

The initial native recording had 2,008 battle rows in source 2D with
`encounter has no checked terrain anchors`. With the anchor fix, all 647
published battle rows used modeled 3D and the actual Route 29 scenery, without
a terrain fallback. Native traces validated the observed midpoint rather than
inventing an anchor or changing the authoritative encounter. All twenty
walking-anchor checks passed, including continuous positions and rejection of
cold, stale, mismatched and off-corridor evidence. The production battle-command
regression also passed, including authoritative move execution and PP mutation.
The existing five intro checks and send-out timing check passed. Camera-order /
classic restoration compositor tests and route/forest battle-fog checks passed.

A final frozen paired capture at production FIGHT/PKMN/PACK/RUN was inspected
in both 3D and 2D. It retained the same Cyndaquil/Pidgey battle, levels, HP and
menu state. The clearer 3D view preserves the grass and terrain, complete
bodies and readable HUD. The selected opposite camera correctly places the
player and its HUD on the right.

These observations do not yet prove seamless battle entry. An earlier traced
handoff included a 2.739-second application interval during preparation. A
subsequent non-traced Route 29 recording observed an 847 ms interval after
handoff; these different host/instrumentation conditions are not a controlled
before/after comparison. Preparation still needs profiling and removal from
the visible handoff. The selected 3D field now remains under the transition as
described below. Diagonal continuous movement and additional encounter sources
also need native coverage before making a general no-fallback claim.

Build the native review client with:

```sh
CARGO_TARGET_DIR=target/route-performance-build cargo build --locked \
  -p crystal-bevy --features location-tester --example immersive_battle_3d
```

Headless image recording can now use `--record` directly without a dummy
`--screenshot`. Keep `CRYSTAL_RENDER_TEST_HEADLESS=1` and
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`; the recording owns its image target
and does not require a nonexistent one-shot file.

A final native Violet Gym trainer recording completed successfully: 1,707
published battle rows all used modeled 3D with the witnessed gym floor and no
terrain fallback. Its intro, trainer narration, send-out and ready-battle
images were inspected. This verifies one real trainer path in addition to the
Route 29 wild encounter; it does not close the remaining preparation and
transition work above. The direct headless `--record` path also exited
successfully without requiring a dummy one-shot screenshot.


### Battle-entry effects stay on the 3D field

The selected 3D field now remains active through the controller's transition.
Its final movement and actor poses continue to extract from the authenticated
field. Palette flashes, spin/scatter/box masks, trainer graphics and cave SCX
rows apply to that field through the existing post-process pass. A 3D entry
skips construction/upload of flat-map transition textures. Classic mode retains
its original presenter and clock. F3 invalidates the transition presenter even
when a passive paired capture retains the same source atlas.

Eleven transition checks passed, including every native wipe frame and every
cave scanline, source trainer-pixel equality, retained field extraction and
unchanged gameplay checksum. A direct native GPU pixel check passed at
127×65, 320×257 and 1184×812: neutral blur output is preserved, full-view
masks/flash work, SCX direction/wrapping matches native pixels, and trainer
art retains square pixels. The effect remains available with blur disabled.
Recording and opt-in source-hold checks also passed.

An offscreen Violet Gym pair froze actual controller frame 110 after ordinary
input. Its receipt reports `state_preserved=true`, the same checksum/LCD frame
and actor poses in 3D and classic views. Both images were visually inspected:
the wipe covers the selected viewport while the 3D floor, characters and
scenery remain underneath. No flat field replaces the 3D field. Reproduce the
same source-phase review with the built review executable and a fresh output
prefix:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic CRYSTAL_RENDER_TEST_WALK_STEPS=1 \
CRYSTAL_RENDER_TEST_HOLD_ENTRY_AFTER=110 \
./target/route-performance-build/debug/examples/immersive_battle_3d \
  content-packs/core-modular.browser.crystalpack --gym-encounter \
  --walk UAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA \
  --size 1280x800 --screenshot target/entry-review-3d.png \
  --second target/entry-review-2d.png
```

The hold pins the first actually observed source frame at/after the requested
value; it never rewinds or manufactures a transition frame. It pauses only
this explicit native QA review and requires an ordinary input route.

A full native trainer recording completed with all 130 observed transition
updates retaining active 3D field/effects, followed by 1,772 modeled battle
updates. A Route 29 Cyndaquil/Sentret recording completed with all 58 observed
transition updates retaining the 3D field, followed by 1,193 modeled battle
updates. Eight frame-zero updates correctly left effects neutral while the
committed final walk finished. Flash, wipe, intro and narration images were
inspected; the reversed camera's trainer portrait stays beside its own side.
The Route 29 recording still exposes the 847 ms preparation interval above,
so a general seamless-entry claim remains premature. Recording CSVs now name
field/battle renderer activity, effect activity and controller transition/intro
frames, making this remaining gap distinguishable from a 2D fallback.

A subsequent Route 29 run used `--measure` with
`CRYSTAL_MEASURE_3D_VALIDATION=on` and `CRYSTAL_LOADING_TRACE` instead of
continuous screenshot readback. Its trace isolates a 595.69 ms
`battle.establish_camera` span inside a 681.50 ms main frame, followed by a
311.19 ms render submission interval. Terrain freezing occurred earlier and
took 78.33 ms. Camera fitting and the following GPU preparation/submission
must therefore be addressed separately; eliminating recording readback alone
does not eliminate the entry pause. This is diagnostic evidence, not a
controlled improvement measurement or a seamless-entry acceptance result.

CPU sampling located repeated full object scans in the camera sight proof.
Frozen encounters now index those immutable object bounds once per generation.
The index prunes impossible ray hits and retains the original detailed surface
predicate at every candidate leaf; it does not remove camera candidates or
relax admission. All 50 encounter-terrain and four walking-ground checks
passed, including 7,938 indexed-versus-flat ray/height comparisons and the
indexed faded-wall obstruction check. A subsequent native Route 29 replay
retained modeled 3D scenery for all 285 battle rows, with no fallback. Its
camera fit still took 523.94 ms. Overall update timing differed substantially
from the earlier run, so these timings do not establish a controlled speedup.
The synchronous exhaustive fit and following render submission remain entry
latency work; the spatial index alone is not seamless-entry acceptance.

### Native battle preparation overlaps the field transition

`VisualBattleFrame.preparing` separates the upcoming offscreen scene from
presented battle ownership. Once the committed final walk has completed, the
retained battle-start snapshot supplies the upcoming bodies during the source
field transition. On native builds, the exhaustive camera search runs on an
async worker copy of the frozen encounter. Its request key includes bodies,
viewport, UI reservation and retained generation. A changed key discards the
pending request; a completed fit reapplies only its checked anchors/layout and
rechecks live geometry, preserving live handles, opacity and actor state.
Browser camera fitting currently retains its synchronous implementation.

The hidden image target, retained terrain, skinned actors and aura pipelines
can render during preparation. The battle composite and source UI remain
unpresented until the controller's existing handoff. Classic transitions do
not request this preparation. The transition cleanup guard now preserves an
explicit preparation packet while still retiring stale presented battles.
This guard is covered by the external-pack extraction/checksum regression.
The hidden-target check, all 50 terrain checks (including live-fit
revalidation), 11 source transition checks and 13 recording checks passed.

A native Route 29 replay with ordinary synthetic input, no continuous image
readback and opt-in loading tracing published 42 transition updates, all with
active 3D field, then 564 presented battle updates, all modeled 3D with no map
fallback. Thirty-four transition updates carried a preparation packet; eight
frame-zero updates correctly waited for the committed walk. Camera fitting ran
once on worker thread 6 from trace time 23.027 to 24.254 seconds. Sixteen main
frames continued on other threads during that search. The last transition
extraction occurred at 26.793 seconds; there was no handoff camera refit.
The worker's 1,226.77 ms duration is not a computation-speed improvement claim.

The first two seconds of presented battle still contained a 135.99 ms update
interval. Render preparation around the handoff also remained substantial.
The CPU search no longer blocks the handoff in this replay, but GPU work,
overlap cost and broader entry coverage still need improvement before claiming
seamless entry. Recording CSVs explicitly distinguish `battle_preparing` from
presented `battle_frame_active` and `battle_3d_active`.

A new Violet Gym 3D/classic pair held the actual source transition at frame
110. Both images were inspected: the 3D gym remains beneath the authored wipe
and trainer graphic, with no hidden battle scene leaking into the field. The
receipt reports `state_preserved=true`, checksum frame 310/hash 236677086,
LCD frame 595 and matching source actor positions/facings. This is a matched
source-phase presentation check, not latency acceptance.

The subsequent native Violet Gym trainer replay retained the 3D field on all
147 transition updates and modeled battle scenery on all 2,770 presented
battle updates, with no map fallback. Its only camera fit ran on worker thread
4 for 23.03 ms, well before the last transition extraction at trace time
10.446 seconds. The largest update interval in the first two seconds of
presented battle was 31.99 ms. This verifies the trainer handoff separately
from the slower Route 29 case; it does not establish equivalent timing for all
encounter sources or browser builds.

### Connected scenery retains source-map ownership

The host now publishes compact `VisualSourceRegion` rectangles from the same
resolved map placements used by the source-tile renderer. Base-map priority,
connected-map reverse lookup and off-map border fallback follow that resolver.
Live extraction and destination preparation both retain the regions. Terrain
cache identity includes them, and worker requests preserve them through asset
rebinding. Legacy frames without regions retain their previous map behavior.

Building asset selection and tree/grove variants now use the placement's source
map and that map's local tile coordinates. A New Bark lab visible from Route
29 therefore selects the lab before crossing; entering New Bark no longer
changes its asset from the generic gym. Tree variant coordinates likewise
survive the active-map coordinate rebase. Source-map shape lookup in the main
terrain build also uses cell ownership. This metadata does not change gameplay
coordinates, collision, scripts or source art.

The 30 render-API checks passed, including ownership/local-coordinate stability
across a map handoff and invalid-region rejection. The complete native lab
drawing selects the same lab asset on both sides. Worker roundtrip/rebinding
and retained-terrain checks passed. Four related external-pack checks also
passed with the completed TUI pack configured.

A fresh offscreen production walk from Route 29 `(58,9)` east into New Bark
was visually reviewed, including moving frames and a matched 3D/classic pair.
The lab keeps its green-roofed 3D model before and after the boundary. All 128
movement rows retained active terrain, none had pending terrain or fade, and
only the initial terrain build occurred. The authored `Wait, RENDER!` scene
still ran. The paired receipt reports `state_preserved=true`, checksum frame
844/hash 2864923885, LCD frame 1178 and matching source actor positions/facings.
Reproduce with the built location tester:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=1280x800 \
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic CRYSTAL_RENDER_TEST_WALK_STEPS=1 \
./target/route-performance-build/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map Route29 --x 58 --y 9 --hour 12 --view both --walk RRRR \
  --screenshot target/route-source-review.png
```

That review also exposed ground-finish discontinuity: connected town ground
used route dirt before the active map changed, then town paving afterward.
The source-owned ground pass below addresses that case. Other crossings still
need review; the building/variant correction alone was not complete transition
acceptance.

### Connected ground retains its own palette and grain

The production ground pass now selects its map style and local grain/shader
coordinates from the physical surface's source region. Town gravel therefore
appears while viewed from the route and retains the same finish at handoff.
Tree-contact shadow offsets use the same source-local coordinates as their
grove variants. Courtyard expansion and paving eligibility also honor cell
ownership. The blend field and shadow index remain shared across the scene;
this does not run an additional full finishing pass for each connected map.
Frames without region metadata retain the legacy path.

A two-map handoff regression compares the same connected floor before and
after the active-map rebase. Colors, grain/shader coordinates, vertices and
indices are identical; a separate assertion proves town cells already differ
from the old route-only pigment. Eleven existing ground, paving and shadow
checks passed, covering contact-shadow lookup, prop exclusion, courtyard
limits, bank colors and retained support geometry.

A fresh 1280×800 offscreen Route 29 `(58,9)` eastward crossing was inspected at
moving frames and as a matched 3D/classic pair. New Bark paving is present
before the crossing and remains afterward, alongside the unchanged lab asset.
All 128 movement updates retained active terrain, with no pending terrain,
fade or additional terrain build. `Wait, RENDER!` still executed. The pair
reports `state_preserved=true`, checksum frame 610/hash 1953177725, LCD frame
893 and matching source actor positions/facings. Broader crossing coverage,
remaining battle-entry render intervals and browser camera preparation remain
separate outstanding work.

The current native build was also remeasured after the ground change using the
same executable, completed browser pack, 1280×800 image target, noon, focused
update policy and 15-second ready intervals without continuous readback. Order
was Route 29 dense/adaptive, then Route 26 adaptive/dense. Dense mode sets
`CRYSTAL_UNDERSTORY_FULL_DETAIL=1`; adaptive leaves that override unset.

| Scene | Decorative plants | Median update | P95 update | Completed GPU submissions/s |
| --- | --- | ---: | ---: | ---: |
| Route 29 (50,9) | dense override | 63.85 ms | 95.30 ms | 14.527 |
| Route 29 (50,9) | adaptive default | 49.65 ms | 85.89 ms | 18.343 |
| Route 26 (10,54) | adaptive default | 45.37 ms | 76.54 ms | 20.100 |
| Route 26 (10,54) | dense override | 67.22 ms | 136.79 ms | 12.967 |

All timed updates retained active 3D world rendering; none entered battle, and
all queue-poll error counts were zero. Adaptive throughput was about 26% higher
on Route 29 and 55% higher on Route 26 in this pass. Both adaptive warmup images
were inspected. These are queue callbacks, not displayed FPS or GPU timestamp
durations. The host was not isolated from other work, and this compares plant
density within the current executable rather than measuring a before/after
speedup from ground finishing. Absolute route performance remains insufficient
evidence for a general smooth-gameplay claim and needs further optimization.


### Native Route 29 shadow cost investigation

A further same-executable diagnostic pass used the completed external browser
pack, Route 29 (50,9), noon, 1280×800 offscreen image target, focused updates,
and 15-second ready intervals without continuous readback. These are temporary
QA overrides; production shadows, filtering, resolution and blur remain intact.

| Probe | Completed GPU submissions/s |
| --- | ---: |
| Default | 18.916 |
| Modeled shadows disabled | 42.758 |
| Tilt-shift disabled | 22.572 |
| Both disabled | 41.031 |
| Shadow resolution 1024 | 21.335 |
| Shadow resolution 512 | 18.382 |

Shadow rendering is the largest identified cost in this pass. Lower resolution
alone was inconsistent. Shadow-only GPU culling without the orthographic clamp
patch did not activate its guarded optimization. With the existing opt-in patch,
it activated two safe cascades but measured only 10.112 completions/s; neither
option was enabled by default. Receiver-culling CPU traces were typically below
1 ms and do not explain the entire shadow cost. The host also ran unrelated
work, so these results are diagnostic evidence rather than isolated benchmarks.

A temporary hardware-2x2 shadow-filter probe measured 21.102 completions/s
(43.34 ms median update); a subsequent Gaussian baseline measured 26.066
(32.71 ms). Both warmup images were inspected. That did not establish a repeatable
filter improvement, so the temporary selector was removed and Gaussian remains
the default. An interrupted initial Gaussian startup is excluded from results.

The GPU timing investigation exposed a separate diagnostic failure on M5 Metal:
basic timestamp queries are enabled but timestamps inside passes and encoders
are unavailable. Bevy's recorder still allocated counter buffers and eventually
lost the device. Diagnostic recorder installation now checks all required device
features before initialization. Boundary-only devices report unavailable timing
without allocating those unusable queries; CPU-only devices without basic
queries can still use safe CPU diagnostics. Five diagnostic policy/schema tests
passed. The guarded native run exited successfully with all 234 timed updates
in active 3D and zero captured GPU timestamp frames, correctly reporting
`bevy_gpu_timestamps_unavailable_for_device`. The earlier failed trace is not a
valid completed performance run. These controls are opt-in and do not affect
normal gameplay. Queue callbacks remain neither displayed FPS nor GPU duration.


### Resumable browser battle-camera preparation

Browser hidden battle preparation now retains a camera-search snapshot between
render updates rather than running the entire stage search in one update. It
preserves the ordinary low-angle tier, higher fallback tier, end-on fallback,
source-ground support checks, score/distance pruning and final admission proof.
A preparation update batches inexpensive rejections for up to four milliseconds
between stages. A complete stage remains indivisible, so this is a scheduling
budget rather than a hard four-millisecond deadline. Native preparation retains
its existing background task.

Pending browser searches use the same body/support/viewport/UI/terrain identity
as native preparation. A changed identity replaces the search; leaving the battle
cancels it. The completed result contributes only layout and anchors and must
pass the current live terrain proof before presentation. Hidden preparation
continues to avoid claiming that the battle has been presented. This change
requires a live browser entry recording before it can establish a stall-free
handoff; source inspection or a successful compile alone is insufficient.

The native regression compares resumed and exhaustive results for a tall body
requiring a higher shot, a rejected retained footprint, and a derived stage with
nearby source-ground candidates. It passed with exact camera/body-pose/anchor
agreement, including the relocated-stage result. The separate hidden-target
preparation regression also passed. These are scheduling/geometry proofs, not
browser frame-time measurements.

`cargo check --locked -p crystal-voxel-view --target wasm32-unknown-unknown
--features terrain-worker` completed successfully using the isolated route
performance target directory. A disposable production browser bundle is being
built for live measurements; no stall-free browser result is claimed yet.


Pending browser verification now has two disposable location-tester starts:
`preview=route29-border` at Route 29 (58,9), and `preview=route26-border` at
Route 26 (1,94), both requiring `multiplayer=off`. They select only a starting
position; crossing still uses ordinary controller input and source connections.
The opt-in loading trace also records a bounded `presentation.frame` event at
Last with world/battle 3D activity, pending terrain, entry-effect state and hidden
battle preparation. This is submitted app state, not proof of a GPU-presented
frame; screenshots and interval measurements are still required. It avoids
missing single-frame fallback states between bridge polls. Normal untraced play
does not emit these events.

The disposable bundle build uses the existing matching wasm-bindgen 0.2.128
executable through `WASM_BINDGEN`; the globally installed 0.2.129 was correctly
rejected by the build script. The first full wasm-dev build remains in progress.
Inspection found that the worker-enabled browser build's
`scenery-full-detail` selection also forced dense decorative plants. Decorative
plant density is now separate: browser routes use adaptive placement while
retaining full authored models; `understory=dense` is a same-build location-tester
comparison. Dedicated ancient-forest ground remains dense. Native full-detail
environment overrides preserve their existing behavior. The immutable density
choice travels in terrain-worker requests, rejects an already configured
mismatch before geometry construction, and is reported separately by WebMCP.
Workers configure it only after request validation. The build was intentionally
restarted after this protocol change so the main and worker artifacts agree.
Native adaptive-density speedups still must not be presented as browser
speedups; actual browser A/B measurements remain required.

The final native worker-policy regression passed, including rejection of a
mismatched density request before any geometry build. Both other worker policy
and trace-scope checks passed. The explicitly enabled external-pack round trip
also passed for New Bark, Viridian, Route 2, the player's ground floor, both
Magnet Train stations, Dark Cave and Route 29, comparing exact prepared output.
Eight understory geometry regressions passed, including unchanged tree triangles,
source footing, adaptive-budget subset selection and connected-plot placement.
These checks validate the request boundary and native behavior; browser speed
and handoff acceptance remain pending the completed production client.


### Hardware browser validation and battle composite repair

The first completed production browser bundle exposed a real WebGPU failure on
battle entry. The camera fit finished during the field effect, but the battle
composite shader failed validation and the app stopped updating behind a black
canvas. Per-frame activity flags alone had appeared correct. Capturing shader
compilation information identified the exact error: Naga's decimal emission of
f32::MAX rounded above WebGPU's representable range in `capture_finite2`.
The helper now tests the all-ones IEEE exponent instead. It still rejects both
infinities and every NaN payload, and accepts all finite values including signed
zero, subnormals and maximum finite values; it no longer emits that float limit.

On the M5's offscreen Metal WebGPU adapter, the captured original shader
reproduced the error and the repaired shader compiled without errors. A direct
GPU check of the production helper matched the independent finite-number
reference for 16,673 boundary and randomized pairs. The rebuilt full browser
client then passed `tools/browser-transition-check.mjs`: all 80 field-entry
rows and 726 presented battle rows retained 3D, with zero shader errors,
validation errors, device losses, worker failures or fallback jobs. The 3D
Pidgey/retained-forest image and classic counterpart were visually inspected;
the production view toggle preserved the observed game state. This verifies
one browser wild-entry path, not all transitions or hitch-free timing.

The same browser investigation found a separate route blocker. An immediate
Route 29 (58,9) crossing entered New Bark correctly but recorded 110 app frames
without active 3D while destination terrain was pending. The in-flight
preload currently replaces the live scene before its geometry is resident.
The trace also found speculative New Bark asset publication occupying hundreds
of milliseconds during a Route 29 battle effect. Continuous connected-scene
retention and budgeted/preemptible speculative publication remain required;
this crossing is explicitly not accepted as seamless.

The new route-border preview names also needed entries in the browser's 3D
worker policy; they now agree with the Rust starts. The existing worker-policy
checks pass. Browser reference captures use the production view-toggle button,
since the native F3 shortcut is not a reliable browser control.

Default headless Chromium on this machine selected SwiftShader and exposed no
WebGPU adapter. With `--enable-gpu`, both its headless shell and unified headless
binary reported Apple Metal and the M5 renderer, with timestamp-query support.
This matches [Chromium's headless GPU guidance](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/gpu/using-gpu-hardware-in-headless-chrome.md).
The walking performance tool now enables hardware selection for headless runs,
records adapter/renderer identity, and can require it with
`REQUIRE_HARDWARE_GPU=1`. It also reports active-3D app update throughput:
requestAnimationFrame can keep ticking when the game app has stopped, so its
intervals alone cannot establish the game's frame rate. These counters and
GPU timestamps still do not prove display-presented FPS.

### Lower concurrent-build-load native diagnostic

The existing QA executable was remeasured at the earlier Route 29/26 positions,
noon and 1280×800 for 15-second ready intervals after other compiler jobs had
finished. Our browser build remained active, so this is not an isolated host
or final-source acceptance run. The executable retained the temporary shadow
filter selector; that selector remains removed from production source.

| Scene/filter | Median update | P95 update | Completed GPU submissions/s |
| --- | ---: | ---: | ---: |
| Route 29 Gaussian | 32.95 ms | 54.70 ms | 25.121 |
| Route 26 Gaussian | 39.71 ms | 80.18 ms | 22.621 |
| Route 26 hardware 2x2 | 35.52 ms | 70.01 ms | 25.373 |
| Route 29 hardware 2x2 | 34.63 ms | 74.38 ms | 25.541 |

All 1,501 timed updates retained 3D and all poll-error counts were zero. The
filter again did not establish a consistent improvement on both routes, so
Gaussian was retained. The change in absolute readings reinforces the host-load
limitation on earlier numbers; it is not another code speedup. In the earlier
guarded trace's final 15 seconds, render preparation/submission dominated the
CPU-side render envelopes while light visibility and receiver pruning remained
below one millisecond at the median. Those envelopes can include driver waits
and are nested; do not sum them or present them as GPU timestamps. Browser GPU
pass timing is the next direct measurement of that cost.


A first clean hardware-browser stationary Route 29 sample completed on the fixed
bundle using the standard preview (50,9), fixed preview hour 16, 1280×900,
wasm-dev, default workers, adaptive decorative plants and full authored models.
`REQUIRE_HARDWARE_GPU=1`, `REQUIRE_DEFAULT_WORKERS=1`, `REQUIRE_EMPTY_PARTY=1`
and `GPU_WATERFALL=1` all passed. It recorded 344 active-3D app updates over
10.071 seconds (34.157 updates/s), requestAnimationFrame median 33.3 ms and P95
33.4 ms, with a 416.6 ms maximum. This is one adaptive sample, not a browser
before/after gain and not directly comparable to the native noon/1280×800 runs.

Thirty timestamped GPU samples were available. Median directional-shadow pass
time was 13.959 ms and the main opaque pass was 11.796 ms; the transparent 3D
pass was 0.721 ms and each blur pass about 0.098 ms. This directly identifies
shadow and opaque geometry rendering as the dominant GPU work. The run reported
zero device/context losses, GPU validation or shader errors, texture allocations
or composed texture upload bytes during measurement. The long maximum and
observed speculative publication stalls still prevent a smooth-gameplay claim.


The complete same-build hardware browser density comparison also passed. Order
was Route 29 adaptive/dense, then Route 26 dense/adaptive; all cases used the
same fixed bundle, Apple Metal adapter, 1280×900, fixed preview hour 16,
wasm-dev and 10-second stationary windows. Full authored model detail stayed
enabled. Dense selected only `understory=dense`; the default used adaptive
plants. Each case retained the original empty party, completed both workers
without failures/fallbacks, and reported zero GPU, shader or context/device errors.

| Scene | Decorative plants | Active 3D app updates/s | rAF median / P95 | Maximum rAF interval |
| --- | --- | ---: | ---: | ---: |
| Route 29 (50,9) | adaptive | 34.157 | 33.3 / 33.4 ms | 416.6 ms |
| Route 29 (50,9) | dense | 23.831 | 33.4 / 50.1 ms | 483.3 ms |
| Route 26 (10,54) | dense | 34.144 | 33.3 / 33.4 ms | 50.1 ms |
| Route 26 (10,54) | adaptive | 48.244 | 16.7 / 33.4 ms | 33.5 ms |

Adaptive update throughput was about 43% higher on Route 29 and 41% higher on
Route 26 in this pass. Thirty timestamped GPU samples per case showed shadow
medians changing from 21.561 to 13.959 ms on Route 29 and 16.810 to 10.879 ms on
Route 26; opaque-pass medians changed from 16.253 to 11.796 ms and 10.486 to
8.094 ms respectively. No composed texture allocations/uploads occurred during
the measured windows. App update throughput, rAF cadence and GPU pass duration
are distinct measurements; none is a direct display-presented FPS counter.
These are stationary results; movement/crossing timing still needs acceptance.

Both adaptive routes were additionally captured through the production Start
menu to keep gameplay state fixed while selecting the 3D and classic views.
`tools/browser-transition-check.mjs ... reference` passed with exact equality
of the observed player/map/status/menu/flow state, healthy workers, and no GPU
errors. Both pairs were visually inspected: path/grass/flowers, route buildings,
cliffs, character placement, restrained fog and retained tree scenery remain
readable. The extra decorative plants are reduced while source encounter grass,
trees and gameplay geometry remain intact. These reference cases do not
supersede the failed fast border-crossing result above.

The subsequent implementation retains the completed scene while adopting an
in-flight connected destination's work queue. The original atlas/build receipt
stays intact; root transforms and actor footings use reciprocal source-region
coordinates. Retention checks the authored acreage in the existing inner halo
for matching source cells, art, ownership, shapes and elevation. The outer
replacement margin and synthetic active-map border fill are not claimed as
identical authored terrain. Actual Route29 → NewBarkTown and Route26 → Route27
controller crossings passed this check. A shoreline discrepancy at the Route26
edge was repaired: synthetic perimeter water now uses the adjacent native map's
border instead of whichever map became active, preserving the neighboring
cliff's corners.

Speculative scene publication waits during battle preparation/entry. Grove
bounds are resolved with terrain preparation and transported from the worker;
publishing a completed scene no longer recomputes every grove's terrace
support on the frame thread. Six preload tests, three cached-grove tests and
six retained-terrain/animation tests passed. The browser bundle is rebuilding;
these implementation tests do not yet supersede the failed browser crossing
or establish a new frame-time result.

The first browser retention build crossed both tested borders with zero 3D
gaps: Route29 produced 164 presentation rows (81 retained) and Route26 produced
233 (29 retained), with healthy workers and no GPU errors. This is not yet
seamless acceptance: rAF maxima were 1.53/1.45 seconds during concurrent CPU
compilation, and the Route26 recording showed synthetic border forest changing
to water when the completed destination arrived. Publication setup itself fell
to 0.5 ms; the remaining long frame includes uninstrumented terrain/camera work
and GPU asset preparation. Follow-up code caches connected coverage proofs,
avoids validating ordinary same-map frames through that path, samples complete
authored height data without eagerly solving an unused legacy terrace field,
and assigns 3D border fill/style to the nearest native source. Classic LCD border
selection remains unchanged. Forty terrace tests passed, including a check that
authored datum sampling never initializes the legacy solver. Browser retesting
of these follow-ups is still required.


### Final walking GPU comparison

A frozen source snapshot avoids concurrent sky/scenery edits changing the CPU
kernel between worker and client builds. The four walking runs used headless
Chromium on Apple M5 Metal, a 1280×900 viewport, 16:00, the same wasm-dev build,
full authored models, normal shadows, healthy default workers and an empty
party. Route29 walked horizontally; Route26 walked vertically. No Cargo/rustc
processes were running during the matrix. Other OS graphics activity was not
inventoried.

| Route | Old dense decoration | Adaptive decoration | Gain | rAF p95, dense → adaptive |
| --- | ---: | ---: | ---: | ---: |
| 29 | 26.60 canvas frames/s | 38.27 canvas frames/s | 43.9% | 50.0 → 33.4 ms |
| 26 | 32.35 canvas frames/s | 45.92 canvas frames/s | 42.0% | 33.4 → 33.4 ms |

These rates count successful WebGPU canvas texture acquisitions in the active
keyboard windows over full interaction wall time, including observation
overhead. They establish actual canvas rendering activity, not OS/display
presentation confirmation. All cases reported Apple/metal-3, fallback=false,
full models, and the requested density policy; their worker, GPU, original-party
and authoritative-movement checks passed.

The latest route handoffs retained active 3D for all 294 Route26 and 254 Route29
presentation rows, with zero gaps/dropped events and matching classic game
state. Their rAF medians were 16.7 ms and p95 33.4 ms. Fast startup crossings
still recorded cold publication spikes of 250/283 ms, so these are not claimed
as a guarantee of zero first-use stalls. Browser people now reuse the existing
one-prepared-appearance-per-frame uploader; its asset-reuse regression passed.

A valid wild encounter retained 3D through all 82 field-entry rows and 682
battle rows. Falkner testing exposed an assumption that the player always
stands at a tile center. Continuous movement can stop inside that tile beside
the trainer. The host now captures its published displacement, and the ground
proof accepts that bounded position while retaining the separate exact NPC
witness, source terrain, occupied-tile exclusion, and corridor checks. All 52
encounter-ground tests and the continuous-approach host regression pass.
The final browser trainer run passed: all 142 field-entry and 1,077 battle
presentation rows stayed in 3D, with zero gaps, dropped events, GPU errors or
worker fallbacks. The production classic toggle preserved the game state; the
paired gym captures were visually reviewed. Its median and p95 rAF interval
were 16.7 ms, with a cold maximum of 466.7 ms (not a zero-hitch claim).
The final production-controller suite passed eight tests; MCP movement/Start
and clean-stderr regressions also passed.

Browser camera searches run incrementally during entry. If a search remains
when the controller presents battle, its remaining stages finish before the
first battle shot is published. This prevents the classic canvas appearing
between the retained field and the checked 3D camera; residual cold search work
can still cause a frame hitch. A follow-up final-build run retained 3D for all
66 wild-entry and 410 wild-battle rows, and all 137 trainer-entry and 1,079
trainer-battle rows. Both passed worker/GPU/state-pair checks with zero gaps or
dropped events. These runs overlapped other workspace compilation and are
continuity checks, not replacements for the isolated walking measurements.
Their cold maximum rAF intervals were 816.6 ms (wild) and 749.8 ms (trainer).
A second final-build wild run also passed all 66 entry and 421 battle rows
with zero 3D gaps, worker/GPU failures or game-state changes between views.
Both final wild runs encountered Sentret; the earlier Pidgey run is the case
that identified the deadline defect. No final Pidgey acceptance is claimed.


### Follow-up: source faints, battle entry and destination lead time

The original entry-only check did not cover a full attack/faint cycle. A new
normal-input Falkner regression reproduced fourteen presentation frames in
classic view during the first faint, despite healthy WebGPU workers. It also
revealed that the original field-entry palette flash still strobed the whole
3D field even when its presentation flag stayed active. The immersive field
now stays clear during that controller-owned interval. Classic mode retains
the original entry effect and both modes retain the same input/audio timing.

Picture resizing, removal and extracted OAM rows now remain local to the
battler over the retained 3D scenery instead of requiring the entire classic
scene. Available species models stay modeled during a faint: original picture
shifts are mapped to their neutral silhouette height and the existing floor
occludes them. The camera stays fixed during this short source animation.
Unsupported source pictures still use exact source art; this is not a claim
that every move, capture or legacy palette effect has a modeled replacement.
Trainer result narration retains the proven encounter ground until the
controller returns to the field.

Six targeted faint/fog tests and six battle-camera tests passed. The model
regression checks entity identity, original frame preservation, floor-relative
drop and stable layout. The external-pack source regression covers faint,
return, removal and both sides of Water Gun's generic row extraction. The
production controller suite passed eight tests, with one existing ignored
test. A broader legacy immersive fixture suite is not a passing acceptance:
some pack scripts, including PsychicM at table index 94, do not compile into a
fixture timeline. Its failures must not be represented as tested move support.

Connected destinations begin warming forty core tiles before the edge. Base
terrain and profiles must be available, but decorative grove streaming no
longer postpones that work. An already-proven connected terrain union also
avoids an unnecessary script fade-in wait while the destination replaces it.
Outdoor cameras share the same hardware shadow-filter pipeline across routes
and towns, avoiding a filter change at their boundary. Native decoration uses
the adaptive policy unless full understory is explicitly requested.

The first Route29 filter probe used headless Chromium, Apple M5 Metal,
1280×900, 16:00, full models, an empty party and healthy default workers. Across
thirty GPU timestamp frames, median opaque-pass cost was 17.83 ms with Gaussian
filtering and 16.61 ms with Hardware2x2; the shadow-map pass was 17.07 and
16.78 ms respectively. Canvas acquisition throughput was 20.78 and 21.49/s.
Concurrent workspace compilation produced very high system load, so those
rates do not establish isolated FPS or hitch-free play. Hardware filtering
makes shadow edges sharper; geometry and shadow-map resolution are unchanged.
A three-second-leg exploratory walk crossed into New Bark and is excluded
from route-only FPS comparisons. The walking harness now rejects that mistake.

The follow-up also found that the original Route26 vertical path can stop
between a one-way ledge and an NPC. That blocked run is excluded. Ordinary
left inputs first move the test to x=8; six subsequent 1.5-second vertical
legs travel between y=44 and y=55. Every measured leg must now change the
authoritative position and remain on the named map. This stricter corridor
pair recorded 25.16 canvas acquisitions/s with Hardware2x2 and 26.08/s with
Gaussian under concurrent compilation; it does not establish an FPS gain.
Route29's corresponding 1.5-second pair recorded 19.46/s and 20.17/s. These
loaded-machine results are a limitation, not acceptance of route smoothness.

A cold Route26→27 crossing retained 3D in all 193 presentation rows, including
92 connected-retention rows, with no worker/GPU errors or dropped events. Its
maximum rAF interval was still 450 ms. The crossing input took 613 ms over
eight controller frames; the trace included 291.5 ms in modeled actor sync and
237.6 ms in GPU asset preparation. This identifies remaining foreground
character costs rather than proving a seamless handoff. The clean-stderr
production MCP regression passed again.

Frame-aligned video review confirms a modeled enemy sinking below the actual
gym floor throughout its faint, with a steady camera, and a clear field during
entry instead of the old palette strobe. A full Falkner run then passed through
both faints, result narration, Zephyr Badge, TM31 and return to the field: all
6,147 presentation rows retained 3D (117 entry and 2,691 battle), with no dropped
events, worker/GPU failures or changes between the final classic/3D state pair.
It used Ember; it is not evidence for every enemy move. The longer harness
collects bounded trace epochs and rebases their timestamps; that collection
can stall rAF and must not be used as an FPS benchmark. A trusted gesture
unlocks the muted source-audio clock so authored badge WaitSFX can finish.
Earlier attempts that stopped before dialogue settled or overflowed the trace
are not passing full-cycle evidence.

Before browser character lookahead, the outdoor-filter build's cold
Route29→New Bark check kept 3D for all 301 rows, with 289 connected-retention
rows, but still recorded a 649.9 ms maximum rAF interval. The crossing command
took 752 ms over eight controller frames. Actor upload/instance preparation
remained visible in the trace. Neither cold-crossing check establishes zero
loading delay.

The final browser build also warms destination character roles through the
existing single worker queue, prioritizing current-map roles. It retains live
actor updates while future roles prepare, keeps uploaded handles shared, and
uploads at most one new prepared appearance per frame. It does not retire the
live GPU cache while battle owns presentation. Matching worker/client kernels
were built from one frozen source snapshot, excluding concurrent art changes.

All four final border continuity cases passed with identical classic game
state, healthy workers, no GPU errors, no dropped events and no 2D gaps:

| Border | Cold rows / peak rAF | Warm rows / peak rAF |
| --- | --- | --- |
| Route29→New Bark | 204 / 699.9 ms | 190 / 700.0 ms |
| Route26→27 | 218 / 499.9 ms | 196 / 500.0 ms |

These peaks are unresolved stalls. Route29's warm trace still spends 548.3 ms
in GPU asset preparation, while Route27 actor setup still peaks at 308.7 ms.
The four eight-frame commands in the warm Route29 case took 371/472/452/374 ms;
Route26 took 433/765/392/364 ms. Character lookahead has not established a
hitch-free transition or an overall FPS gain. Final paired images were reviewed
at the same authoritative position/state, including the real New Bark gate
text. The landscape stays visible and characters retain their footing; this
is continuity acceptance, not acceptance of all scenery art or performance.

The expanded battle driver opens with Leer before Ember and verifies an enemy
turn and actual HP loss. The observed Gust counterattack changed player HP
from 51 to 46 while retaining 3D, but no Gust move cue was published by this
pack: this is not evidence for a complete Gust animation. An overly strict
intermediate assertion expecting that missing cue failed and is not recorded
as passing animation support. Browser bridge/bundle tests passed twelve cases;
worker policy/transport checks passed two.

The final expanded battle regression passed: 8,075 presentation rows (106
entry, 2,964 battle and eighteen source-faint rows), world→battle→world,
verified PP spending, a five-HP enemy counterattack, badge/TM completion,
matching final classic state, no 2D gaps/dropped events and healthy workers/GPU.
The worst rAF interval was 1,966.6 ms during this instrumented long run; it is
not smoothness acceptance. The final run used the browser character-lookahead
build and the stricter chunked-trace harness.

Final walking checks used that same optimized WebGPU build on Apple M5 Metal,
1280×900, 16:00, full models, Hardware2x2 shadows, default workers and an empty
party. Six 1.5-second legs each changed the authoritative position and stayed
on their route. Route29 alternated horizontally from x=44 to x=54–56; Route26
used ordinary left inputs to reach x=8, then alternated vertically between
y=44 and y=55. Both passed hardware, worker, GPU, movement and party checks,
with no texture allocations/uploads during measurement.

| Route | Canvas acquisitions/s | rAF median / p95 / peak |
| --- | ---: | --- |
| 29 | 19.52 | 33.4 / 66.8 / 133.3 ms |
| 26 | 24.21 | 33.3 / 66.8 / 200.0 ms |

These are canvas acquisitions over interaction wall time, not confirmed display
presentation. This task's compilation had finished, but other system activity
remained and the recorded load averages were about 33 and 31. They do not
establish isolated performance or an improvement over the earlier idle matrix.
Route lag and border upload/setup spikes remain unresolved. The local test
servers, frozen source snapshot and generated QA recordings/bundles were
removed after verification; ignored source content packs were preserved.


### Trainer portrait overlay correction

A user screenshot exposed a separate defect that continuity-only checks missed:
classic player/trainer portrait sprites were deliberately exempted from the
immersive compositor's hidden layer. A fully active 3D battlefield could still
show the oversized pixel trainer. The compositor now hides all portrait-tagged
entities while either the retained 3D field or battle is active, including
standalone portrait entities. Classic mode and genuine classic fallback restore
the original entities. Controller timing, dialogue and send-out state are
unchanged. The compositor regression now rejects the old exemption and checks
field preparation, battle, classic fallback and the view toggle separately.

The corrected compositor regression and classic-transform restoration regression
both passed on the native test build. An additional legacy backpic PNG check
could not load its filesystem fixture (`apps/web/assets/gfx/player/chris_back.png`);
that check is not passing evidence. This correction was verified through render
layers, not a newly rebuilt browser screenshot or deployed client.
