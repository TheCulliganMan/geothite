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

## Incremental GPU visibility trial

The fitted-layout fresh roof control produced 412 updates in 60.12s,
median/p95 130.43/299.88 ms, 6.848 queue completions/s, outstanding 1→1,
zero polling errors. Its reverse-order control then failed the existing
180-second validated-scene warmup. That failed run supplies no throughput
comparison. Control variation is too large to accept the fitted candidate's
apparent advantage; no fitted-layout source or app was promoted. The saved
executable remains immutable. Whole-tree geometry, the original application
source and its dependency placement are restored in the private snapshot.
The verified edge-clamp/occlusion hook is no longer registered.

Installed Bevy 0.20 source provides another concrete CPU path: entities marked
NoCpuCulling skip the per-refresh CPU frustum/visibility work, and render-side
GPU visibility tables process changed entities incrementally. This behavior is
also described in the [official Bevy rendering notes](https://github.com/bevyengine/bevy-website/blob/main/content/news/2026-06-19-bevy-0.19/index.md).
The private native gpu_static_visibility trial uses it for explicitly marked
panorama draws, requiring non-shadow-casting/non-shadow-receiving VoxelMaterial
meshes on the production world layer. Summit publication adds the marker; the
foreground room is never admitted merely because its material flags match.
Hidden upper-floor publication receives flags before roof entry, and cached
ownership persists across adjacent floors without entry-time archetype churn.
It requires actual GPU instance building, GPU frustum-culling support and
compatible perspective/indirect world views. Unknown bounds, skinned/morph
meshes, sky/no-frustum meshes and foreign layers retain their original path.
It owns only flags it inserts, restores CPU handling on incompatible views or
support loss, and retains hidden panorama ownership across adjacent floors.
The existing engine InheritedVisibility remains authoritative. No geometry,
material, shader, shadow configuration, controller state or content pack is
changed; no new runtime switch exists. Source is private and unpromoted.
The first four-guard run caught a missing ViewVisibility component in its
minimal test fixture (3 passed, 1 failed); the fixture is corrected. A preload
patch stopped on an exact-text mismatch, so only its own just-started partial
verification processes were cancelled before fixing the patch; other builds
were not interrupted. The corrected full-crate lifecycle build passed in 5m49s:
`/tmp/geothite-gpu-static-visibility-preload-tests.log`, all five tests passed
with zero ignored. They include hidden preload visibility, unchanged NoCpuCulling
change ticks at roof entry, explicit ownership removal, hardware/direct-view
fallback and preservation of foreign flags. All eight summit-vista regressions
passed, and terrain-worker cargo check passed in 13.02s. The native release
example build passed in 6m36s; log: `/tmp/geothite-gpu-static-visibility-release-build.log`.
Its saved executable SHA256 is
`35c2dea2961e00be28411c5a5460313cb1dff10158433b3c4925bb69b99f69cf`.
The first full-quality Retina roof 60-second comparison completed.
The first actual run confirmed GPU instance/frustum support and assigned
193,915 panorama entities across 261 publication updates. No occlusion-culling
or shader-correction hook is active. At 2560x1600, noon, Roof (9,13), camera
0/-1/5/0, the candidate completed 681 updates in 60.02s, median/p95
86.78/95.10 ms, 11.347 queue completions/s; the following unchanged-renderer
control completed 608 updates in 60.06s, median/p95 95.82/116.88 ms,
10.117 queue completions/s. Both had steady 36 outstanding submissions and
zero polling errors. The candidate screenshot passed validation and visual
inspection with the full roof, actor and forest present. The pair suggests
about 12% more queue completions, but needs repeat/movement checks before
acceptance. This is not displayed FPS or whole-game fluidity. An ordinary
synthetic-keyboard UUDUDUDUDUD roof walk completed through the unchanged
controller path; its mixed result is recorded below. Paired-2D fidelity and warm-stair latency remain unverified. Shared production source and the installed app are unchanged
by this experiment.

## Roof attribution and fitted tree-section follow-up

The roof shadow-receiver candidate passed all ten full-crate regressions in a
2m41s build, with no ignored tests. Before another release build, source
inspection found that summit_vista already tags its distant surfaces and tree
leaves NotShadowCaster. A new headless Metal recording of the unchanged
shadow-cache executable (target PID 84788) completed and exported successfully:
`/tmp/geothite-retina-roof-metal-whbbfE/roof.trace`. The two-second capture has
6,192 GPU rows, 599 encoder rows and 18,159 object-label rows. Encoder IDs are
joined using encoder-list column 12 to column 8; object-label fallbacks and all
process filters are per-file ID-resolved. The labeled target sums include
main opaque Vertex 361.279 ms and Fragment 85.760 ms, early prepass Vertex
251.289 ms and Fragment 63.205 ms, and both shadow cascade Vertex sums only
1.901 ms combined. Most target intervals are unlabeled (including 2218.670 ms
of Vertex intervals), so these are not full-frame percentages. Another
new_bark_3d process (85154) also performs GPU work; it was not interrupted.
The sample supports concentrating on opaque/prepass geometry, rather than
assuming the earlier New Bark shadow attribution applies to the panorama.
The short CPU sample caught mainly waits and does not identify a generation
bottleneck. No generator/disk cache is justified from it.

The private roof shadow-map eligibility change is restored, without a release
build or promotion. A new native-only section comparison retains exactly the
same section/draw count as the prior subdivision. It compares that grouping
with a three-axis, triangle-weighted bounding-surface-area partition, choosing
fitted groups only when their aggregate bound cost is lower. Original source
triangles, attributes, collision and sorted fade leaves are unchanged. The
full geometry-preservation regression build passed in 3m05s:
`/tmp/geothite-fitted-sections-tests.log`. All 56 occluder/visibility regressions
and three shared-grove cache tests passed with zero ignored tests. Triangle-
weighted bound cost changed from 1,139,330,962 to 907,555,991 for pine, from
311,745,232 to 232,965,785 for ancient forest, and from 245,578,877 to
208,586,101 for deciduous trees (about 20.3%, 25.3% and 15.1% lower). Draw
counts remain equal. This cost is a geometry-fit proxy, not GPU time or FPS.
Native helper feature guards were checked with the actual terrain-worker
configuration; cargo check passed in 12.60s. The private application reinstates
the identical verified shader/camera hook for the fitted-layout comparison;
no new culling logic is introduced. The release example build passed in 4m51s; log:
`/tmp/geothite-fitted-sections-release-build.log`. Its saved executable SHA256 is
`156d24e75311167b78b886a734802d996e5640990c1fa0d57f77382b49a3ecca`.
The 60-second full-quality Retina roof candidate completed: 703 updates in
60.11s, median/p95 84.90/114.85 ms, 11.364 queue completions/s, outstanding
11→31, zero polling errors. Both shader corrections verified and exactly one
world culling view activated. The screenshot passed existing validation and was
inspected: roof, actor and full forest remain present. A fresh unchanged-
renderer control is running. These observations are not yet an accepted speedup
or paired-2D/animated-bound visual acceptance. This is a geometry-layout experiment,
not a frame-rate gain or production integration. No new runtime switch exists.

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
The subdivision-only release build passed in 9m17s; log:
`/tmp/geothite-opaque-sections-direct-release-build.log`. Its saved executable
SHA256 is `6e70c6baa8bb1dd12788afa8c9568cbefd5f00be9a0c9dbf3b0602f9d7a41816`.
The application source and dependency placement exactly match the pre-probe
baseline. The shader-correction hook is not registered. A 60-second roof run
completed with the existing native render path and full geometry: 638 updates,
median/p95 93.44/110.26 ms, 10.547 queue completions/s, outstanding 31→36.
The immediately following control produced 498 updates, median/p95
106.92/193.89 ms, 8.284 queue completions/s, outstanding 36→36. Both had
zero polling errors. The control is also materially slower than the earlier
control (10.654/s), so the apparent subdivision gain remains unaccepted under
variable load. New Bark subdivision-only measured 1392 updates over 60.01s,
median/p95 40.78/78.55 ms; the fresh control measured 1544 updates over 60.03s, median/p95
38.54/44.76 ms. Queue completions were 22.608/s for subdivision versus
25.714/s for control, both with zero polling errors. Subdivision is not
promoted globally: New Bark regressed in this pair, and the apparent roof
benefit is still confounded by differing control performance. The immutable
subdivision executables remain available; the private voxel source is restored
to whole-tree geometry before the next isolated trial.

Source inspection identified another concrete shadow-work candidate:
shadow_receivers::supported_map admits New Bark, Route29 and Cherrygrove but
excludes TinTowerRoof. A separate private candidate now adds the roof
and covers it in the existing world/capture-camera production-system guard.
Its full crate test build is running in
`/tmp/geothite-roof-shadow-receiver-tests.log`.
Its geometric proof, bias padding, filter settings, receiver eligibility and
unknown-bound fallbacks are unchanged. The comparison has restored
whole-tree geometry, isolating the shadow filter from subdivision.
No production source or app change has been made for either trial.

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

## Dedicated panorama visibility layer verification

The initial NoCpuCulling walking comparison did not establish a benefit. With
ordinary synthetic keyboard input and the same 352 renderer updates, candidate
movement median/p95 was 91.44/126.08 ms versus control 94.44/104.44 ms. Both
covered three grid origins with zero additional terrain builds. The worse
candidate tail prevents promotion based on the stationary result alone.

A private refinement puts the explicitly non-shadow-casting panorama on layer
32, keeps foreground and sunlight on 31, and lets the world camera see both.
Bevy's actual GPU mesh collector otherwise includes these panorama entities
in shadow candidate tables before the later caster rejection. The new test
runs that collector directly: the world view retains all 129 fixture meshes,
the actor-capture view remains empty, and the sun retains one foreground mesh
instead of all 129. This preserves drawn geometry while avoiding irrelevant
shadow candidate membership. It is not evidence that this caused walking tails.
Receiver-bound proofs still include both layers. All six GPU visibility tests,
eight summit tests and ten receiver tests pass; terrain-worker checking passes.
The release build passed; the immutable QA executable SHA256 is
`cd524c267a523d0eefc62c7dc046317910197971f6436cc6057f577299bc189e`.
At full 2560x1600, noon, roof (9,13), the ordinary keyboard replay completed
352 movement frames across three grid origins with zero additional terrain
builds. Candidate movement median/p95 was 93.80/112.23 ms; the following
control was 82.47/219.96 ms. Overall candidate/control update counts were
632/648 over approximately 60 seconds each, median 93.25/81.91 ms.
The candidate's lower tail comes with a slower median and no overall
throughput gain; this pair does not justify production promotion. The actual
candidate screenshot was inspected with roof, trainer and forest present,
but paired-2D acceptance remains unperformed. No installed app or shared
renderer source was changed by this experiment. No user switch was added.

## Static panorama terrain batching trial

The dedicated GPU visibility layer did not establish an overall gain and is
removed from the next private candidate. The baseline native world layer and
CPU visibility path are restored. Static panorama textured/solid terrain now
uses the existing terrain_batches partitioner, rather than uploading one
whole-map draw per material. Every original triangle and all attributes are
retained; animated surfaces, textured scenery, tree geometry, materials and
transforms keep their original path. Browser behavior is unchanged. No new
runtime control is added. This path skips coverage scans that panorama draws
do not consume. All 23 batching tests passed, including exact ordered corner,
attribute and cutaway preservation, and all eight summit tests passed. Native
release compilation passed. The saved candidate SHA256 is
`e02b3399e5abd334bb004f096fd7cc44c63267e47b1be8761e380a8aab8bd039`.
The first full-quality Retina keyboard walk produced 682 updates/60.05s,
median/p95 87.48/98.73 ms, movement 84.49/94.19 ms. The subsequent baseline
produced 619 updates/60.12s, median/p95 93.29/169.67 ms, movement
97.18/176.94 ms. Both have 352 movement frames, three grid origins and zero
additional terrain builds. Other Rust compilers were active. Reverse order produced a second baseline
with 611 updates/60.08s, movement median/p95 93.86/127.48 ms, and a second
candidate with 670 updates/60.03s, movement 85.43/152.66 ms. Candidate
movement medians improved in both orders; mean update throughput rose about
10%. Tail results varied, so this is a bounded scene improvement under the
observed load, not isolated GPU timing or proof of overall fluid gameplay.
All runs retained three grid origins and zero additional terrain builds.
The first paired capture saved 3D but exceeded its existing 60-second deadline
before 2D; a retry with the existing 180-second capture allowance passed both
image verifications. Both images were visually inspected; the receipt reports
state_preserved=true, matching actor source and coordinates. The full roof and
forest remain present. The two-file batching change is now applied surgically
to shared source; current-checkout tests and a production build are running.
The installed application has not yet changed.

### Current-checkout and transition follow-up

All 23 batching tests passed against shared source, with zero ignored tests.
All eight current-checkout summit tests also passed with zero ignored tests.
Production app compilation is live, and destination-preload regressions are
being rebuilt against the completed external TUI pack. A native candidate replay from TinTower9F (7,8),
noon, 2560x1600 used ordinary synthetic keyboard input, 180 B inputs then D.
All 5,792 movement rows completed: 5,770 on 9F and 22 on TinTowerRoof. There
were zero additional foreground terrain builds. Every roof row reports
terrain_pending=false and terrain_active=true; the first roof row at (9,13)
is 30.649 ms, with the authoritative warp fade still in progress. This proves
arrival readiness for this replay, not total fade duration or a faster door
transition versus the earlier baseline. The full 180-second run completed
5,934 updates, median/p95 28.11/52.40 ms; this is mostly the interior and must
not be presented as roof frame rate.

### Production installation of static panorama batching

The current-checkout production build passed in 16m27s, with desktop-app and
voxel-view enabled and location-tester absent. The normal desktop installer
passed external-pack verification (388 maps, 251 species, 251 moves, 377 audio
programs), scenery/people bundle validation, strict code-signature validation
and its staged standalone installation check. It installed
`~/Applications/Geothite.app` in 41.3 seconds. A second direct check from /tmp
with CRYSTAL_OPEN_MODEL_ROOT unset passed against the installed resources and
existing external save path. The installed signature also passed a fresh
strict check. Both source and installed packs retain SHA256
`a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007`;
the installed executable SHA256 is
`1cb0ac7b3bf5b418b848e0afc8e884cb46e8682d7e491f0ad0e2d2a6d289c682`.
The existing new-bark destination-preload regression build is still active.
No source/catalog/audio/pack regeneration, visual quality reduction, user
performance switch, visible window launch, or commit was made. The broader
fluidity objective remains incomplete: roof rendering remains around eleven
updates per second in the full Retina QA scene, and tail stutter persists.

## Native normal-layout follow-up

The first destination-preload test invocation omitted voxel-view and therefore
selected zero tests; it does not count as verification. Enabling voxel-view
exposed that the fixtures also require location-tester fields (four compile
errors for render_test_hour). The corrected command enables both features and
remains compiling. No production code was changed to work around the fixtures.

The existing normal-layout diagnostic was exercised using the same immutable
static-batching QA executable and a full Retina synthetic keyboard roof walk.
Full float normals: 691 updates/60.04s, movement median/p95 82.45/90.25 ms.
Packed-normal controls: 671 updates/60.08s, movement 88.00/106.90 ms, then
687 updates/60.01s, movement 86.16/96.23 ms. The final reverse-order float
comparison is running. Every completed run has 352 movement rows, three grid
origins and zero additional terrain builds. These small differences are not
yet accepted as a speedup; no default or installed app has changed in this
follow-up. No new diagnostic switch or timing implementation was added.

The final float-normal run completed 732 updates/60.01s, movement median/p95
79.70/145.33 ms. The float runs averaged roughly 4.8% more updates than the
packed controls and both movement medians were lower, but tail behavior was
mixed and the second overall p95 was 156.63 ms versus 96.23 ms for its
preceding packed control. This does not establish a stutter fix or justify a
new native-wide default. The installed packed-normal default is unchanged.
A movement follow-up is now using the saved immutable fitted-tree-section
executable, whose earlier stationary comparison was too noisy to accept.
It is a distinct candidate (including the previously verified private GPU
occlusion shader correction), not the installed static-batching executable.

The corrected current-checkout destination-preload run passed all four tests,
zero ignored, in 4.54 seconds after compilation. It covers both house floors
preparing New Bark without gameplay advancement, connected source-coordinate
retention, nearest real building selection and route arrivals.

The saved fitted-tree candidate's movement follow-up completed 723
updates/60.06s, movement median/p95 77.11/108.05 ms. A fresh static-batching
control completed 697 updates/60.08s, movement 72.86/140.11 ms. Its slower
movement median and small overall difference do not establish a reason to
replace the installed batching renderer with this coupled experiment. It
remains unpromoted.

## Move panorama partitioning to its existing worker

A private follow-up addresses a concrete main-thread path: static batching
currently partitions all source triangles when a completed background map is
published. The worker now returns PreparedMap with finished static Mesh
vectors, leaving only GPU-asset/entity publication on the frame thread. The
existing border-sample trimming is moved verbatim before worker partitioning;
the tower attachment is calculated before source geometry retires. Animated
terrain, materials, tree meshes, geometry and transforms preserve their prior
paths. No new worker, cache, runtime option or timing tool is added. The
existing summit regressions and a separate release QA build are running.
This follow-up is private; the installed static batching version is unchanged.

The private worker-partitioning candidate passed all eight summit tests and
its release QA build. Its immutable executable SHA256 is
`9b3b4315bde6d5c0ed81c6f13f78e01ff5af9a4e5032fb7f4971b5b45bf6519f`.
Further source review found the same panorama publication path still builds
water masks/shoreline images synchronously. The next private version returns
those existing PreparedImages from the same terrain worker too. Existing
water-surface regressions include byte-for-byte worker preparation checks
across actual source roles/animation phases and authenticated dry results;
that suite and the combined release QA build are running. No installed app
or shared renderer source has changed in this follow-up.

A separate hidden-panorama visibility experiment reused the previously saved
NoCpuCulling executable and its matching pre-batching control, at TinTower9F
(7,8), noon, full Retina resolution, 120 seconds. Candidate: 4,608 updates,
median/p95 24.62/47.44 ms; control: 4,743 updates, 19.59/57.54 ms. The
candidate trace assigned 174,489 eligible panorama entities. Whole-run
throughput does not establish a gain. However, the final 30-second windows
differ: candidate 994 intervals, median/p95 27.43/54.73 ms; control 663,
39.93/84.09 ms. Both preload progress and concurrent compiler load can vary;
cache-readiness equivalence and a reverse-order result remain unproven.
This late-window evidence changes the next investigation toward hidden-cache
CPU costs but does not justify a production promotion yet.

### Combined worker verification and loading-time caveat

The combined worker/water release QA build passed in 9m12s; saved executable
SHA256 `aef541d20ef8c0f4939e9ed422e6b8f2fe599eb766fcc8d9cbdc2357912b2715`.
All eight summit tests passed. The private water suite passed 14 tests,
including byte-for-byte worker image preparation, and failed its older raised
pier fixture. The current checkout already corrects that fixture from a
repeated half-width Port14 face to the full-width Port08 drawing. Its existing
compiled test binary passes all 15 water tests. Preserve that current fixture
and the current Tohjo style change when merging; neither runtime water image
algorithm nor source geometry was changed by the worker follow-up.

The combined warm-stair replay ended at the 180-second measurement limit
before reaching D: 4,608 movement rows, all on 9F. It supplies no roof-arrival
proof. Interior median/p95/p99/max was 29.676/82.235/122.582/292.753 ms,
112 rows over 100 ms and two over 250 ms. Compiler load differed from the
earlier run; these statistics do not prove a reduction in loading stalls.

A near-immediate D plus 30 B inputs completed all 992 movement rows and
reached the roof with zero additional foreground builds. Critically, step QA
pauses movement logging while terrain_pending is true (render_walk.rs). Its
first ready row cannot measure total loading time. The unfiltered render
log first shows Roof at frame 60, 0.578438s. Matching the next five movement
intervals against the full render intervals identifies the first ready walk
row at render frame 1501, 14.552441s: a 13.974003-second readiness gap.
The unique five-interval match avoids treating skipped updates as zero time.
The earlier first-ready-frame timings likewise do not prove short loading
latency. No timing implementation was modified; this is analysis of existing
records. A matching main-thread/static-batching control is running.
The installed application and shared renderer source remain unchanged by
the private worker follow-up, and fast cold entry is still unproven.

The matching main-thread/static-batching near-immediate control completed all
992 movement rows and zero additional foreground builds. Its full render log
first shows Roof at 1.069899s; the unique next-five-interval match resumes
ready movement at 31.407565s, a 30.337666-second readiness gap. The preceding
worker candidate measured 13.974003 seconds by the same method. This single
contended pair supports the worker move as a useful loading improvement, but
does not establish isolated timing, general fluidity, or acceptable cold entry.

A further private change replaces the single map task with a FIFO queue of
at most two native tasks (one on browsers), using the existing async pool.
Only the front completed task may publish, and current publication finishes
before another result is consumed. Progressive sources cannot queue the
border early; resetting/leaving the panorama cancels queued tasks. The
foreground-ready gate and geometry/material paths stay intact. Two new
regressions exercise bounded prefetch/cancellation and out-of-order completion
with ordered publication. The private raised-pier fixture is aligned with the
already-corrected checkout fixture; runtime shoreline code is unchanged.
The expanded summit tests and separate release QA build are running. This
queue change and the worker move remain private and uninstalled.

All ten expanded summit tests passed with zero ignored tests, including
bounded native prefetch, cancellation on unrelated maps and deliberately
out-of-order completion with ordered publication. The release QA build is
still live. The short reverse-control (DBB, 60 seconds) ended before the
movement logger reached its first flush and provides no completed ready
latency. It ran while own and unrelated compilers were active, so no timing
comparison is accepted from it. Further performance comparisons will wait
for own compilation to finish; unrelated builds remain untouched.

The private queue candidate now passes all ten summit and all fifteen water
regressions (zero ignored), and its release QA build passed in 9m11s. Own
compilation is terminal before its near-immediate stair performance replay.
The replay uses DBB and a 120-second recording window; this retains enough
ready movement intervals for correlation with the complete render timeline.
A matching single-worker replay will follow. The installed app remains the
previously accepted static-batching version.

### Queue comparison and main-source integration

Own compilation was terminal during the two-map and matching single-worker
120-second DBB recordings. Both completed all 96 movement rows, three grid
origins and zero additional foreground builds. The two-map FIFO first
showed Roof at 1.381140s and resumed ready movement at 23.169698s, giving a
21.788558-second readiness gap. The following single-worker control first
showed Roof at 1.423397s and resumed at 86.970697s, giving 85.547300s. Both
use unique five-interval matches against the complete render timeline.
Unrelated compilers remained active and load varied substantially; this pair
supports overlapping preparation under the observed workload, not a stable
fourfold speedup or acceptable cold-entry latency.

The two-map executable SHA256 is
`4e16db7639ab8bbcd2d5543f0e3c293e041f4cf2476cf86092b5d513828c1a1d`.
Its full-resolution paired captures both passed validation and were visually
inspected at Roof (9,13), noon, camera 0/-1/5/0. Roof, trainer, forest,
lighting and scene extent remain present. The receipt states
state_preserved=true with matching Chris source, position and facing.

The worker/queue patch is now applied to the main summit module, preserving
the checkout's native-only HiddenDestination insertion and existing shoreline
fixture/Tohjo style changes. Current-checkout summit regression compilation
is running; production rebuilding/installation is still pending. No content
pack, controller, audio, geometry, material, source color, draw distance,
resolution, runtime switch or browser JavaScript was changed.

The main-source production build has now started with desktop-app/voxel-view
and without location-tester. Its log is
`/tmp/geothite-panorama-prefetch-production-build.log`. Current-checkout
summit tests are also live. The accepted earlier static-batching app remains
installed until these checks and packaging validation complete.

All ten summit regressions now pass against the current checkout with zero
ignored tests. The current-checkout water suite is running using that test
artifact. The production build remains live; no rebuild was restarted on an
observation timeout and the installed app has not yet been replaced.

### Prefetch failure ordering fix before installation

Review found that popping a failed front task left later tasks queued while
next still identified the failed source. On a later update, a successful
prefetched result could then be consumed using the wrong source index, even
though error state prevented accepting the panorama. The error branch now
clears all remaining tasks. A new regression feeds a failed first result and
a successful later result, then requires no map publication, no index advance,
no queued results and no blocking of the hidden preload's foreground room.

Only the two own superseded build trees were terminated before editing their
inputs (Cargo PIDs 58874 and 60623; process identities and descendants were
checked). Both sessions returned terminal status 143. Other builds/apps were
left untouched. The corrected current-checkout summit tests and production
build are running in new sessions, with logs
`/tmp/geothite-panorama-prefetch-error-shared-tests.log` and
`/tmp/geothite-panorama-prefetch-error-production-build.log`. No installation
of the superseded build was performed. Valid-pack rendering and loading paths
are unchanged by this failure-only fix.

Both corrected builds then terminated with E0308 in the concurrently updated
water-ripple call: images.get_mut returns AssetMut<Image>, while update takes
&mut Image. The closure now binds mut shore and passes &mut shore, preserving
the ripple algorithm. The error was independent of the prefetch fix; neither
failed build supplies a passing test/build result. New current-checkout test
and production build invocations are running with logs
`/tmp/geothite-panorama-prefetch-error-borrow-shared-tests.log` and
`/tmp/geothite-panorama-prefetch-error-borrow-production-build.log`.

The guarded borrow edit detected that the exact correction had already
landed in shared source before it wrote. Inspection confirmed mut shore /
&mut shore in the current file; no duplicate edit was applied. The subsequent
builds are using that corrected state and were not restarted.

### Worker/prefetch production installation completed

The corrected current-checkout summit suite passed all eleven tests, including
failed-source cancellation. The same compiled test artifact passed all fifteen
water-surface and all four water-ripple tests; zero ignored tests in all three
suites. The corrected production build passed in 11m56s. Desktop packaging
passed pack/art validation, strict signing and staged standalone checks, and
installed the app in 35.7 seconds. A direct installed-copy check from /tmp
with CRYSTAL_OPEN_MODEL_ROOT unset passed, as did a fresh strict signature
check. The installed executable SHA256 is
`709a6516e65819befd5021c73dd609bf5efc4d18e6528bbec5c26651c67c5016`.
Source and installed content packs both remain
`a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007`,
and the existing external save path is unchanged. No visible game was launched.
The installed app now includes static panorama batching, worker preparation
of static meshes/water images, bounded two-map native prefetch and failed-job
cancellation. Broader fluidity and fast cold entry remain incomplete.

Own compilation and packaging are terminal before the next GPU comparison.
The immutable prefetch QA executable is now running the full-quality Retina
roof keyboard replay with the existing WGPU_BACKEND=metal diagnostic. Installed
Bevy 0.20 settings source removes VALIDATION_INDIRECT_CALL in release when
DX12 is excluded from the backend mask, then reapplies environment overrides.
The native default mask is all backends even when Metal is selected. This
trial changes initialization flags, not geometry/materials/resolution; no
new toggle or production renderer-setting change is introduced yet.

## Metal backend-mask follow-up after installation

Using the same immutable prefetch QA executable, full-quality Retina Roof
(9,13), noon, camera 0/-1/5/0 and ordinary UUDUDUDUDUD keyboard replay:
Metal-only completed 680 updates/60.02s, overall median/p95 82.41/131.61 ms,
movement 79.83/117.43 ms. The following default-backend control completed
671 updates/60.06s, overall 86.70/105.54 ms, movement 86.14/93.12 ms. Both
completed 352 movement rows, three grid origins and zero additional terrain
builds. Own compilation was terminal, but unrelated compilers were active.
The approximately 1.4% throughput difference and worse candidate tail do not
establish a useful roof improvement. No renderer setting is promoted. The
same backend-mask diagnostic is now being exercised in New Bark, whose
shadow workload differs. No new diagnostic or timing implementation is added.

The matching New Bark (13,5), noon, Retina RRLLBB replay did not justify the
Metal-only mask. Candidate: 1,301 updates/60.03s, overall median/p95
42.41/73.93 ms, movement 36.22/46.52 ms. Default: 1,640 updates/60.03s,
overall 31.02/65.36 ms, movement 28.17/29.64 ms. Both completed 192 movement
rows, three grid origins and zero additional terrain builds. Own compilation
was terminal; unrelated work remained active. This pair cannot establish a
universal backend conclusion, but supplies no reason to change production
initialization. The installed settings remain unchanged.

## Private batching plus verified occlusion follow-up

A private QA-only combination now registers the previously verified in-memory
Bevy shader-edge correction/occlusion helper with the accepted static panorama
batching and native prefetch. Earlier whole-map draws were too coarse for
reliable benefit; the current terrain has bounded original-triangle batches.
The helper is restricted to Roof and a new guard requires leaving Town/Route29
on their original path. No geometry, normal/color precision, material or
resolution is reduced. The existing shader-source hash/device/depth guards
remain authoritative. The existing workspace sha2 dependency is made direct
in the private app manifest for this verification. Its tests/release QA build
are running; no new production GPU feature, player setting or timing tool is
introduced. A first guarded edit stopped on formatting before writing; the
corrected multi-file edit validated all replacements before writing them.

The private build completed successfully in 8m30s. All six selected helper
tests passed (two shader replacement/edge tests and four ownership/device/map
guard tests). Runtime verified both shader hashes and activated one Roof view.
The first full-quality Retina keyboard replay completed 642 updates/60.11s,
overall median/p95 87.51/136.69 ms, movement 82.49/130.95 ms. Its following
prefetch control completed 572 updates/60.08s, overall 103.18/183.04 ms,
movement 98.69/192.67 ms. Both completed 352 movement rows, three grid origins,
and zero additional terrain builds. No own compiler ran during these replays.
The reverse-order repeat is pending; this single pair does not establish a
repeatable improvement and the helper remains private, outside the installed app.

The reverse-order repeat rejected this combination: control completed 638
updates/60.10s, overall median/p95 90.65/112.19 ms, movement 88.81/102.29 ms;
the following candidate completed 604 updates/60.03s, overall 95.95/146.88 ms,
movement 94.39/138.67 ms. Both again completed all 352 rows, three grid origins,
and zero additional builds. No rustc process was present in the observed
candidate replay. The initial gain was not repeatable and the reverse trial
was worse. No GPU helper is promoted or installed. These runs do not resolve
the remaining full-quality roof frame-rate problem.

## Private occluder inverse-cache candidate

The next private candidate removes repeated full 4x4 inverses from visible
occluder ray checks. Each runtime group retains an affine inverse, refreshed
when its GlobalTransform changes, on first use, and after suspension. Camera
and player motion still recompute exact triangle obstruction tests using that
inverse. No geometry, fade policy, render distance, or player setting changes.
The hidden-hierarchy regression now moves a hidden ancestor, lets change ticks
elapse, and checks the resumed inverse against the current propagated pose.
This candidate is not installed or applied to shared production source.

Private release compilation and the explicitly selected voxel-view occluder
tests are pending. The initial crystal-bevy-only test invocation does not
select dependency unit tests and must not count as occluder coverage. The
corrected command selects both crystal-voxel-view and crystal-bevy with the
app's voxel-view/location-tester features, reusing the existing QA cache.

The release build passed in 7m33s. The corrected test invocation passed 54
checks and failed one unrelated pack-backed fixture because its environment
path was absent. Rerunning the compiled voxel-view test executable with
CRYSTAL_RENDER_TEST_PACK set to the completed external TUI pack passed all
55 selected tests, zero ignored. The initial app-only invocation selected
zero occluder tests and is not counted. The immutable candidate executable
SHA256 is 1d231fed5da1bfcb0d2302f6a4fd290f1ebb2d8f6eeccee1f57b71249da7020c.

Full-quality Retina Roof UUDUDUDUDUD replays, candidate/control/control/candidate:
815/667/593/701 updates in approximately 60 seconds each. Overall median/p95
frame times were 72.91/77.76, 85.96/132.73, 96.12/141.86, and 77.44/169.37 ms.
Movement median/p95 were 72.84/78.45, 83.70/119.99, 95.38/131.84, and
82.83/172.75 ms. All four completed 352 movement rows, three grid origins,
and zero additional terrain builds. Own compilation was terminal throughout.
Mean update throughput improved approximately 20%; the reverse candidate's
tail remained worse, so this is not evidence that stutters are eliminated.
New Bark and paired fidelity checks remain pending before promotion. The
private regression was additionally extended to move a visible ancestor
with a stationary camera/player; that extension has not yet been rerun.

New Bark RRLLBB candidate/control/control/candidate completed
1,383/1,504/2,004/1,505 updates in approximately 60 seconds each. Movement
median/p95 were 46.63/73.04, 36.88/43.11, 27.98/30.78, and 36.30/41.58 ms;
overall 42.66/68.07, 39.88/46.74, 29.06/38.33, and 40.07/45.54 ms.
All completed 192 rows, three grid origins and zero additional terrain builds.
Own compilation was terminal. This does not support global promotion: both
town candidate runs were slower than their adjacent controls, although control
variation was large. Full render-timing windows begin at different movement
positions after warmup; the movement traces themselves retain matching
first/last origins. Update counts should not be treated as an isolated
continuous-walking frame-rate measurement.

The older prefetch executable was used for those controls. To rule out source
snapshot differences, an exact matching control is now building from the same
private source tree and flags after restoring only the pre-cache occluder file.
The candidate plus extended visible-ancestor regression is saved separately
as candidate-files/occluder-inverse-cache-visible-regression.rs. Do not restore
it while that control compiler is active. No cache change is in shared
production source or the installed app, and paired fidelity remains pending
if a candidate survives the matched comparison.

While that control compiles, the unregistered private file
candidate-files/occluder-ray-bounds-candidate.rs prepares a follow-up using a
conservative world-space ray-bundle box. It retains the inverse cache, caches
transformed complete primary/overlay bounds per pose, and rejects only disjoint
boxes before exact triangle tests. The ray bundle includes the camera, all
player body samples, pit targets, nearby outdoor people and the actual indoor
furniture samples. Camera-wall outside-room policy and fade settling remain
outside the rejection. Nonfinite/unknown bounds retain the original exact path;
finite bounds are padded for transform rounding at large panorama coordinates.
Three new tests cover translated/mirrored/nonuniform/sheared exact triangle and
overlay hits, complete target inclusion, and fail-open invalid bounds. This
file is not registered, compiled, tested, promoted, or installed yet; it is
outside the current control compiler inputs. The earlier visible-ancestor
regression remains part of this candidate.

The exact matched control release build completed in 7m18s. Its immutable
executable SHA256 is a9f5528954c577e05da880e363f0755109d50ca12e9b80de5f7a1427c3b0d6ef,
different from the older prefetch control. A binary hash difference alone does
not identify a source behavior difference, but the newly built control gives
the required same-snapshot comparison. It is now running the New Bark replay;
the inverse-only candidate will follow. Own compilation is terminal; other
unrelated builds remain untouched. The production app remains unchanged.

Same-snapshot New Bark control/candidate: 1,458/1,460 updates over
60.06/60.05s, overall median/p95 39.77/76.45 versus 41.64/60.85 ms,
movement 33.80/74.91 versus 35.80/60.73 ms. Both completed 192 rows,
three grid origins and zero additional builds. The earlier town slowdown
was not established by this matched pair; throughput is effectively equal
and median/tail results are mixed. Other unrelated compilers were active
during this pair; own compilation was terminal.

Same-snapshot Roof control/candidate: 631/664 updates over 60.03/60.07s,
overall median/p95 88.69/161.44 versus 89.85/108.46 ms, movement
87.90/136.16 versus 88.95/116.82 ms. Both completed 352 rows, three grid
origins and zero additional builds. No rustc/cargo process was present in
the observed control replay. This is weaker than the earlier 20% estimate:
throughput improved approximately 5%, the median walking frame did not,
and longer pauses decreased. Neither pair proves consistently fluid play.
The inverse-only candidate is not promoted or installed.

The saved ray-bounds candidate is now the private source under test. The
correct pack-backed occluder test command and release QA build are running
against frozen private inputs. It includes the unverified visible-ancestor
regression and three new conservative-bounds tests. Production source and
the installed app remain unchanged. Future comparisons must use the new
matched control executable rather than the older prefetch binary.

## Lossless cluster data preparation (not a renderer)

The isolated candidate-files/exact_clusters.rs now partitions full source
triangles spatially into at most 64 triangles/192 vertices per cluster, with
complete float32 positions, normals, UVs and RGBA colors. Its payload stores
float bit patterns directly, including signed zero. Source face ordinals retain
duplicates, degenerates and winding; unused source vertices need not be drawn.
Five standalone Rust tests pass over generated adversarial fixtures: complete
corner reconstruction/payload round trips, bounds and limits, duplicate and
degenerate faces, winding/signed zero/HDR colors, invalid inputs, deterministic
partitioning and extreme finite coordinates. Output is ignored temporary QA
data. This is not integrated with Bevy, actual game models or a GPU renderer,
and supplies no frame-rate evidence.

Installed Bevy 0.20's native MeshletPlugin supports Metal and exposes its required
device-feature list, but its stock asset stores quantized positions, encoded
normals, UVs and no vertex colors. The new data preparation addresses preservation
only; rendering integration would still need exact positions/colors/normals,
no simplification/LOD selection, proper materials/prepass/fade behavior, verified
device/shader compatibility, capacity handling and real model/paired render tests.
Unsupported vertex layouts must retain the ordinary full-quality path. No
meshlet feature, dependency, engine modification, player control or production
rendering change has been added by this prototype.

The ray-bounds voxel-view test executable finished linking before the app-only
test target. After confirming its test list contained all three new bounds
tests, direct execution with the verified external TUI pack passed all 58
selected occluder tests, zero ignored, in 2.35s. This includes both hidden and
visible ancestor movement in the extended cache regression. The broader Cargo
invocation and release QA build remain active; no gameplay performance or
paired-fidelity result is available for ray-bounds yet. No production change
has been made by this candidate.

Both full Cargo targets completed successfully: all 58 selected occluder
tests passed, zero ignored (1.71s); the release QA build passed in 12m01s.
The immutable ray-bounds client SHA256 is
4120805bb5e738e9c5c393518d4ee7eea93ae0d9119730f267aa943cf9ebb3b1.

Full-quality Retina Roof candidate/control/control/candidate replays completed
692/617/533/614 updates over approximately 60 seconds each. Overall median/p95
were 84.62/154.99, 97.25/136.59, 110.29/177.29, 92.25/189.26 ms;
movement 84.08/134.69, 94.67/136.81, 109.76/176.01, 89.95/138.06 ms.
All completed 352 movement rows, three grid origins and zero additional builds.
Existing GPU completion receipts recorded 660/604/532/605 completed frames,
11.002/10.052/8.866/10.081 completions per second. Thus the throughput benefit
also appears in completed GPU work (about 9% and 14% in the two orders), not
only CPU submission counts. The tail remains mixed and all runs remain far
from fluid. Own compilation was terminal; unrelated work was left untouched.
The New Bark candidate guard replay is now running. Paired fidelity and
production promotion/installation remain pending.

The private tree-generator proof has also been prepared and registered only
under cfg(test), importing the standalone lossless geometry module. It checks
all complete corner bits and source faces for three actual native tree families
and two source seeds each. It has not been compiled or run; the five standalone
synthetic-fixture tests remain the only cluster results so far. This test-only
source edit occurred after both own compilers were terminal and does not alter
the immutable executables or assets used by the current comparisons.

## Ray-bounds production promotion

New Bark candidate/control/control/candidate completed 1,100/1,602/2,079/2,023
updates in approximately 60 seconds each. Overall median/p95 were
50.73/93.01, 37.53/40.45, 29.12/31.26, 29.19/36.07 ms; movement
34.32/56.47, 36.24/39.15, 28.45/29.88, 27.98/29.82 ms. GPU completion rates
were 17.869/26.685/34.649/33.715 per second. All completed 192 rows, three
grid origins and zero additional foreground builds. The large variation in
the first pair prevents a town benefit claim. No rustc/cargo process remained
in the observed later checks; the reversed pair has effectively equal median
frame time and about 3% lower candidate throughput. It does not establish a
material town slowdown, nor does it prove a town improvement.

Paired 2560x1600 Roof captures passed with state_preserved=true and were both
visually inspected. Full roof surfaces, forest, distant scenery, actor placement,
lighting and materials remain intact. An initial house route DBBBBBBBBBB ended
inside on carpet tile (6,7): that is an interior fidelity capture, not exit
evidence. The production natural-movement/controller path explicitly handles
directional edge carpets on an attempted outbound step. The corrected ordinary
DDBBBBBBBBBB route exits into NewBarkTown, ends at (13,6), completes its warp
with fade_alpha=0, and starts zero additional foreground terrain builds. Its
384-row trace includes three grid origins. Both egress images were inspected;
their paired receipt preserves state, all four source actors and placement.
No total cold-transition latency is inferred from the movement trace.

Camera clarification: CRYSTAL_RENDER_TEST_CAMERA is read by named screenshot
installation, not measurement-only validation. These measurement-only replays
therefore compare the same default gameplay camera, as their warmup images
show. The named Roof pair uses the requested low view (rotation 7, pitch 5).
Earlier descriptions of measurement-only runs as that low view are not
supported. No timing/capture implementation was changed to address this.

The accepted bounds algorithm is now applied surgically to shared production
occluder_fade.rs, with the conservative helpers/tests extracted into
occluder_fade/ray_bounds.rs. Unrelated source edits are preserved. There are no
new player settings or quality cuts. Private build inputs were refreshed from
the current shared Rust source, assets, modpacks and required build inputs;
content packs were excluded. The extracted-module regression run and native
release executable/bundlers are building against that frozen private snapshot.
The installed app is still the earlier prefetch build until these checks and
packaging finish. The private cluster test registration was removed by this
source refresh; its two candidate source files remain backed up, and its
native-generator test remains unrun. Only the five standalone cluster tests
are verified so far. The broader frame-rate goal remains unfinished.

The extracted-module regression compile found two missing names in its test
scope (SurfaceMeshData and CutawayUniform). Their imports are fixed in shared
source. A direct comparison proves the helper's runtime section is identical
to the frozen release input and the only file delta is that test import line.
The initial extracted-module Cargo test command ended with exit 101; it is not
counted as passing. The release compilation is unaffected and remains active.
Do not edit its private source while it runs. After it is terminal, copy this
test-only import correction into the private helper and rerun the focused
pack-backed occluder tests before installation. No runtime logic changed in
this correction; the production executable and its bundlers are built together
from the same frozen snapshot.

The release command completed successfully in 6m20s. After it was terminal,
the import-only test correction was copied into the private helper. The final
focused Cargo run passed 59 voxel-view occluder checks, zero ignored (the
current shared snapshot contains one additional existing check); the selected
app target ran zero matching tests and is not counted as app coverage.

Desktop content checking passed in 31.7s with the frozen artwork source and
existing external pack: 388 maps, 251 species, 251 moves and 377 audio programs.
Installation completed in 33.5s at ~/Applications/Geothite.app. A fresh direct
installed --check-installation from /tmp with CRYSTAL_OPEN_MODEL_ROOT removed
passed, as did strict deep signature verification. The installed executable
SHA256 is 16e094cf6da0aa6ae91a828961d88ca357e3df3c5a5aa4083fc6bba69bd85139.
The source and installed pack SHA256 both remain
a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007;
the external save path retains its existing 5474b1469c712c6703e195ae8d719d773d35b727b75822a7e324a0a1269befd5
profile suffix. No running installed game process was found before replacement;
no visible game was launched and no other app/build was closed or suspended.

The installed increment now includes cached occluder poses and conservative
complete-ray bounds automatically. No geometry, quality or user toggle changed.
The remaining roof throughput/stutter problem is not solved. Following
installation, the private cfg(test) native tree-generator cluster proof was
restored and its focused Cargo test build started; that build is now the frozen
private input owner. It has no production rendering integration yet and must
not be described as a GPU speedup.

The real native tree-generator proof passed all six selected checks. The
subsequent exact meshlet asset codec run initially failed on a test-only
AssetSourceBuilder API mismatch; after correction, all nine selected checks
passed, zero ignored, including the actual Bevy loader/saver round trip of a
Johto pine. These prove bit-preserving geometry/attribute and asset preparation,
not GPU rendering or a frame-rate gain. The private shader decoder prototype
is next; no meshlet rendering path is installed or promoted to shared source.
Other applications and builds remain untouched, as requested.

The private embedded-shader checks passed 11 selected tests. The following
composition run passed 13 tests, zero ignored, in 0.23s after a 2m39s build:
/tmp/geothite-native-exact-meshlet-shader-compose-tests.log. Both exact position
decoding in the real raster bindings and full attribute/analytic-gradient
resolution in the real material bindings compose via Bevy ShaderCache and pass
Naga validation. Engine sources are read from the installed pinned dependency;
no copied engine shader catalog is shipped. Narrow in-memory patches require
exact SHA256 identities for bindings, visibility resolve and cull-shared shaders;
unknown or already patched inputs are rejected. These are compiler/data checks,
not GPU execution, visual acceptance, or measured performance evidence.

Two integration constraints remain: the stock software raster always culls back
faces, so two-sided materials must retain the original renderer; and the pinned
meshlet material preparation cache keys by view/type, omitting per-material
specialization keys. A dedicated fixed-contract material type is needed before
using the game surface there. No engine dependency, production shader, material,
meshlet plugin, or player setting has been changed in shared source. A private
candidate adapting the actual painted surface to analytic derivatives is now
being checked, including its normal/motion-vector prepass. The installed app
remains the verified cached-pose/ray-bounds build above. The broader frame-rate
goal is still active; no other application or build was stopped or suspended.

The painted-surface check completed successfully: 15 selected tests passed,
zero ignored, in 0.33s after a 2m46s build. Log:
/tmp/geothite-native-exact-meshlet-painted-shader-tests.log. The actual game
painted shader composes and validates with analytic world/UV gradients, and
its normal/motion-vector prepass also validates. The ordinary shader path is
retained in the private adaptation; UV atlas flips also flip analytic UV
X derivatives. The decoder and native tree/codec checks were rerun in this
same passing command:

```sh
CARGO_TARGET_DIR="$PWD/target/performance-visibility" cargo test \
  --manifest-path .local/grass-material-check/source/Cargo.toml \
  --locked --profile wasm-dev --jobs 2 -p crystal-voxel-view \
  --lib exact_cluster_proof:: -- --test-threads=1
```

No GPU dispatch or gameplay throughput improvement is established for this
prototype. Next requirements are a capability-gated plugin, a dedicated
fixed-contract material type, conservative capacity accounting, matching
visibility/fade ownership, actual headless GPU execution, paired production
captures and repeated gameplay comparisons. Preserve the full-quality normal
renderer as the automatic fallback. No private compiler command remains
running from these checks, and no other app/build was interrupted.

The private exact-cluster prototype now has actual offscreen GPU evidence on
Apple M5, beyond the previous compiler-only checks. A native source Johto pine
(seed [4,8]) draws through the full-precision meshlet decoder and the actual
painted game material. Direct GPU readback verifies every serialized position,
normal, UV and color word; 1,179,648 output words across 512 clusters are checked
including unused zero padding. No source triangle, winding, color, resolution,
view distance or model detail was reduced. Source face/corner CPU proofs still
cover Johto pine, ancient forest and Kanto deciduous generators at two seeds.

Several failed hardware attempts were retained as failures, not speedups.
The initial magenta reference came from the fixture's default TonyMcMapface
without its LUT feature, corrected to production Tonemapping::None. The initial
blank candidate was a real pinned-Bevy mask bug: the per-view visibility Vec
was cleared, but zero words were not uploaded when all instances became
visible, so a previous hidden mask remained on the GPU. Fixed simultaneous
views isolated that issue. An apparent color difference then came from a sun
light assigned only to the reference view's layer; both views now receive the
same light. Format/LOD/raster diagnostic trials did not explain the blank
candidate and must not be cited as its cause.

The actual normal prepass exposed two further pinned-Bevy defects. Its stock
pbr_prepass shader reads flags in the meshlet branch without declaring them;
a source-hash-verified conditional declaration corrects that shader. More
fundamentally, the Rust meshlet prepass (and deferred prepass) binds material
at group 2, replacing geometry bindings, although its pipeline expects the
material at group 3. Disabling normals/SSAO or reducing quality is not an
acceptable workaround.

An isolated, licensed copy of published bevy_pbr 0.20.0 now lives only in the
private source snapshot, crates/bevy-pbr. Its upstream source revision is
b3bdd75becaffe9a16677cf5bae5bd6d198f052b. Four Rust files differ: reset/upload all
per-view mask words, bind prepass materials at group 3, honor inherited
visibility during instance extraction, and skip the stale raster table when
no visible instance remains. The dependency is excluded from workspace
membership, so offline resolution adds only the local bevy_pbr package rather
than pulling unused upstream development/exporter dependencies. The initial
member-based offline command failed for missing optional bevy_gltf; exclusion
resolved it. Shared Cargo manifests/lockfile and installed app are unchanged.
Shader extensions remain in the private harness, not in that source copy.

The final passing command is:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CARGO_TARGET_DIR="$PWD/target/performance-visibility" cargo test \
  --manifest-path .local/grass-material-check/source/Cargo.toml \
  --config 'profile.wasm-dev.package.crystal-voxel-view.opt-level=0' \
  --locked --profile wasm-dev --jobs 2 -p crystal-voxel-view \
  --lib exact_cluster_proof:: -- --include-ignored --test-threads=1 --nocapture
```

It passed 17 tests, zero ignored, in 6.14s after a 51.30s rebuild. Log:
/tmp/geothite-exact-native-gpu-proof-33-instances.log. Both real GPU cases run:
full source UV layout, and the production compact UV-less layout with normal
and depth prepasses. The compact reference uses into_compact_mesh, removing
the obsolete UV1 cutaway mask as production does. Bevy's meshlet PBR helper
retains its required UV0 field; a separate derived source-UV shader flag keeps
the original game's absent-UV semantics. There is no player-facing switch.

Both cases have 10,218 foreground pixels in each image with zero foreground
mask differences. Only 302 of 262,144 pixels differ; mean RGBA byte error is
0.0003147125, maximum channel difference 7, and foreground RGB mean error is
0.0107653 byte. Both source/candidate pairs were viewed. The matched images
are under .local/grass-material-check/exact-gpu-native-tree and
exact-gpu-native-tree-uvless-prepass. These are isolated native asset proofs,
not production map/2D acceptance or FPS measurements.

Both GPU cases put the visible object at instance 32, with 32 shared offscreen
instances rejected by GPU frustum culling. Camera-layer hide/reveal therefore
crosses a 32-bit mask-word boundary; restored images equal the initial GPU
image byte for byte. Hiding their common ancestor removes the last visible
instances. An ordinary red plane behind the old geometry fills every pixel
with (204,25,25,255), proving no invisible meshlet depth survives. Restoring the
ancestor reproduces the original GPU image byte for byte. Hidden camera images
contain only clear color (8,10,13,255). The plane and restored image were viewed.

No GPU renderer path is registered in the production game yet, and no meshlet
speedup is established. The installed cached-pose/ray-bounds app remains the
last verified release. Next work is automatic capability/readiness and capacity
handling, worker-side opaque-tree preparation, original fade/material ownership,
full production map integration, matched 3D/2D captures and repeated gameplay
throughput/transition checks before promotion. No other application or build
was stopped or suspended. All new implementation remains Rust/WESL and private;
no pack, audio source, model, compiled deliverable or player toggle changed.
The temporary official control asset in /tmp and failed diagnostic images were
removed after isolation; accepted images and private candidates remain ignored
for the continuing work. The PBR source and all prototype modules/manifests are
backed up under candidate-files before any future source snapshot refresh.

### Embedded exact-cluster shader loading, 2026-10-20

The private PBR dependency now embeds the five verified shader extensions.
The headless GPU harness uses the ordinary engine loader; its replacement
loader and pre-camera shader mutations were removed. The final suite passed
19 tests, zero failures and zero ignored tests, including both real GPU image
and 33-instance visibility lifecycle checks. Log:
`/tmp/geothite-exact-embedded-input-proof.log`. The source identity check pins
the three embedded library assets loaded by a renderless app; culling and
prepass execution are covered by the actual GPU cases.

A prepared-Mesh adapter now accepts full and compact RGB/UV-less vertex
contracts, retains indexed or non-indexed triangles, rejects animated UV tags
and unsupported/incomplete attributes, and does not mutate the ordinary mesh.
Its tests are included in the 19 checks. This adapter remains private and is
not yet connected to production grove preparation. The native game has not
yet registered the GPU path, and these checks establish no whole-game FPS
or transition improvement.

Bevy's meshlet documentation explicitly warns that low geometry/overdraw can
be slower and consume more memory. This is why runtime adoption still needs
full-scene measurements against the ordinary renderer at identical quality:
<https://docs.rs/bevy/latest/bevy/pbr/experimental/meshlet/struct.MeshletPlugin.html>.

### Private runtime cluster integration in progress

The private voxel renderer now prepares exact opaque grove geometry before
normal packing, retains it with the existing shared weak recipe cache, and
registers an in-memory source before AssetPlugin. Native device support gates
activation. The initial runtime scope is the existing non-shadow-casting and
non-shadow-receiving panorama groves; ordinary fade ownership and sorted
geometry stay authoritative. Raw geometry plus compressed input has a
conservative 192 MiB live budget, and scene cluster capacity is bounded.

The ordinary opaque draw remains available during asset/pipeline warmup. The
GPU child inherits its visibility. Material uniform updates are mirrored, and
ineligible materials or capacity overflow retain ordinary draws. This is
private implementation, not accepted production behavior. A voxel-library
`cargo check` passed (`/tmp/geothite-exact-runtime-check-2.log`); subsequent
plugin lifecycle correction moved material-plugin registration into build so
its nested render-asset plugin receives finish. The release location-tester
build is running, logged at `/tmp/geothite-exact-runtime-release-build.log`.
No full-map image or timing result is available yet.

### First full-map runtime attempt

The first release location-tester build completed in 8m32s, but its headless
2560x1600 TinTowerRoof capture failed before producing an accepted image.
wgpu reported multiple fragment entry points with automatic selection in
`prepass_pipeline`. Logs are `/tmp/geothite-exact-runtime-roof-first.log` and
`/tmp/geothite-exact-runtime-roof-pipeline-trace.log`. This is an integration
failure; no FPS result or visual acceptance is claimed.

Composition checks of the ordinary custom normal prepass and the stock normal
and depth-only prepasses each found a single fragment entry. The focused test
passed (`/tmp/geothite-exact-ordinary-prepass-depth-entries.log`), so it does
not reproduce the complete client's module-selection context. The private
prepass descriptors now select the canonical authored `fragment` entry
explicitly. The next full-map run must establish whether that fixes the
actual failure. The release rebuild is logged at
`/tmp/geothite-exact-runtime-release-build-2.log`.

Uniform-only material changes no longer restart GPU warmup, and in-flight
in-memory source bytes are removed when their geometry owner disappears.
Both corrections are in the rebuild. The installed app is unchanged.

### Full-map rendering and first runtime measurements

The second private release build completed successfully in 5m32s. Explicit
prepass fragment selection cleared the observed full-client validation error.
The full-quality 2560x1600 Roof capture completed; its trace recorded GPU draws
active for 1,048,448 clusters, with 136,554,026 conservatively charged geometry
bytes and one derived material asset. Source trace was moved from an accidentally
relative scratch filename into ignored local verification output.

The production-publisher GPU regression passed (one test, zero ignored):
`/tmp/geothite-exact-runtime-gpu-regression-2.log`. It compares ordinary and
GPU images, exercises depth/normal and alpha-mask shadow prepasses, checks
automatic replacement layers and the actual meshlet child, changes a source
uniform while requiring GPU-only camera layers to remain active, verifies
changed GPU pixels, and hides the parent while requiring a clear image. This
is a primitive runtime integration test, not full-map visual acceptance.

`exact-runtime-roof-paired-3d.png` and `exact-runtime-roof-paired-2d.png` were
viewed. Their receipt preserves state, actor source, position and facing at
noon with camera `0 7 5 0`. The forest differs visibly from the older renderer
captures; full-scene fidelity is not accepted until a matched dependency/feature
control explains that difference. No game geometry or quality change is
authorized as a performance tradeoff.

Immutable candidate: `exact-runtime-release-v2-client`, SHA256
`56221c2aefd5b335cf13abc2185f3a02f7c340ec47ecf6f8e9b8bbbd5875ece3`.
Control is the existing ray-bounds client, SHA256
`4120805bb5e738e9c5c393518d4ee7eea93ae0d9119730f267aa943cf9ebb3b1`.
At identical 2560x1600 default measurement camera, noon, Roof (9,13), 60 seconds
and synthetic `UUDUDUDUDUD`, the first candidate/control pair yielded:

| Metric | GPU candidate | Ordinary control |
| --- | ---: | ---: |
| Queue completions/second | 14.155 | 14.001 |
| Update median/p95 ms | 65.60 / 103.90 | 70.77 / 75.31 |
| Movement median/p95 ms | 73.21 / 112.18 | 71.90 / 76.52 |

Both completed all 352 movement rows, three grid origins and zero additional
terrain builds. No owned compiler ran during measurement. Queue callbacks
are not display-presented FPS. This pair establishes no useful gain: throughput
is nearly equal and candidate tails are worse. The reverse control completed
13.295 callbacks/sec, update 73.24/87.03 ms and movement 70.06/74.31 ms; the
following GPU repeat is running. Its outputs use `exact-runtime-roof-*-reverse`.

A subsequent source review found unconditional layer writes on already-published
parents/children. A conditional-write correction is prepared outside compiler
inputs as `candidate-files/exact_meshlets-stable-layers.rs`; it is not in the
measured v2 binary. The GPU path remains private and uninstalled.

### Streamed-instance readiness bug reproduced and repaired

The v2 reverse GPU measurement completed 7.167 queue callbacks/sec, update
median/p95 131.42/204.86 ms and movement 133.70/206.37 ms. The preceding control
was 13.295 callbacks/sec. Both retain 352 movement rows, three grid origins
and zero additional terrain builds. These data do not support installing v2.
Other builds remain authorized to run; no process was stopped for this work.

The private publisher now conditionally assigns render layers, avoiding
unchanged-component modifications. A real GPU regression verifies that a
settled parent's layer change tick remains unchanged across material updates.
The fixture now uses a translated, rotated, one-eighth-scale parent at large
world coordinates, matching panorama transform characteristics. That version
passed (`/tmp/geothite-exact-runtime-stable-layers-regression.log`).

A separate engine-source investigation found a concrete readiness defect:
`MeshletMeshManager::add_meshlet_meshes` consumes the CPU asset with
`Assets::remove_untracked` after uploading. Later instances sharing its handle
therefore see no CPU asset despite `AssetServer::is_loaded_with_dependencies`
remaining true. The publisher incorrectly interpreted absence as loading,
left those instances unpublished, and prevented completion of global warmup.
This could retain ordinary and GPU draws together indefinitely.

The new GPU regression reproduces the exact consumed-asset state and adds a
later shared instance. Before the repair it fails its 10-second publication
deadline (`/tmp/geothite-exact-runtime-late-instance-before-fix.log`). After
switching readiness to the authoritative asset-server load state, it passes
in 1.84s (`/tmp/geothite-exact-runtime-late-instance-after-fix.log`). Pending
unpublished instances remain ordinary layer-31 draws and no longer prevent
already-prepared instances leaving double-draw warmup. New publications still
reset warmup while their actual assets and pipelines prepare.

The current 20-test exact-cluster suite also passes with both real GPU tests
executed and zero ignored tests (`/tmp/geothite-exact-runtime-final-cluster-suite.log`).
These are correctness checks, not a speedup claim. The corrected release
location-tester build is running with log
`/tmp/geothite-exact-runtime-release-build-3.log`; full-scene performance and
forest fidelity must be checked again. No GPU prototype is installed.

### Hardware back-face mismatch reproduced and repaired

Release v3, containing streamed-asset readiness and stable-layer corrections,
completed in 6m56s and was preserved as the ignored
`exact-runtime-release-v3-client`. No performance or visual result is claimed
for that binary. While it linked, only its cfg(test)-excluded fixture was
changed: the release dependency files were checked and did not contain that
fixture, so active release inputs remained fixed.

The pinned meshlet hardware raster descriptor leaves culling at its default
(None), unlike its software rasterizer, which rejects non-positive triangle
area. Exact palette eligibility requires the ordinary material's back-face
culling. Closed meshes can conceal this difference. An open back-facing red
triangle added to the translated/rotated/scaled runtime fixture reproduced
extra GPU pixels: mean RGBA byte error 0.317234 exceeded the existing 0.1
image limit (`/tmp/geothite-exact-runtime-backface-before-fix.log`).

The private main-view meshlet hardware pipeline now explicitly culls back
faces, preserving the admitted ordinary palette contract. The shadow hardware
pipeline is unchanged; admitted panorama geometry does not cast shadows. The
same GPU regression now passes in 2.06s
(`/tmp/geothite-exact-runtime-backface-after-fix.log`), including the open
triangle, scaled placement, late shared instance, uniform mutation, stable
change tick and parent hiding checks. The current 20-test cluster suite also
passes with zero ignored tests (`/tmp/geothite-exact-backface-cluster-suite.log`).

This establishes a concrete culling defect and correction, not complete
forest visual acceptance or a new FPS result. The corrected release build is
running (`/tmp/geothite-exact-runtime-release-build-4.log`). Its full-map image
and walking comparisons remain required. The installed app is unchanged.

### Panorama-only worker preparation and v4 verification

Release v4 completed in 8m40s. Immutable QA client SHA256:
`d98fdd22ac4f69a492583889a43122e24437010044f241bc40407e2b6cf5e8b0`
(`exact-runtime-release-v4-client`). Its 2560x1600 Roof walk completed all 352
movement rows, three grid origins and no additional terrain builds. It recorded
7.192 queue completions/sec, update median/p95 130.66/233.99 ms and movement
127.41/234.05 ms. Other compilers were observed active and were left alone.
No useful performance improvement is established by this run.

The v4 paired 3D/2D images were viewed and preserve state, position, facing and
noon; receipt camera is `0 7 5 0`. The forest still differs from older control
captures, so full-scene appearance is not accepted. Existing trace events
confirm GPU-only activation at trace time 45.24s while the presentation remains
3D until 57.02s. Capture-log elapsed times have a different origin and must not
be compared directly to trace timestamps. The last 3D trace interval was
contended by compilation during capture, not a benchmark.

Worker review found unnecessary cluster encoding in every ordinary grove,
including normal destination-preload work that never uses GPU panorama draws.
The private source now defaults shared preparation and ordinary forest libraries
to no cluster encoding. A dedicated bounded library enables it only for the
native panorama worker, before normal packing. Browser preparation is unchanged.
The GPU regression verifies ordinary metadata count zero, panorama metadata
present and identical ordinary geometry signatures for both caches, while
retaining all previous rendering/lifecycle checks. It passes in 2.20s
(`/tmp/geothite-exact-panorama-scope-regression-2.log`); all ten forest-instance
regressions also pass (`/tmp/geothite-exact-scope-forest-tests.log`). The initial
scope test compile failed because its local closure shadowed the mesh prepare
function; renaming the test closure resolved it.

A fresh source/dependency-matched ordinary-renderer control is now building
(`/tmp/geothite-exact-source-matched-control-build.log`). This temporary private
comparison replaces only the app's exact-source registration call with a no-op,
so the GPU plugin is not registered. There is no new runtime or player option.
**After this compiler is terminal, preserve its client as the matched control
and restore the source registration before any GPU candidate or production
build.** The GPU-enabled app source is backed up at
`candidate-files/gpu-enabled-bevy_shell-before-control.rs`. The temporary block
is labelled `Temporary source-matched CPU reference` in private `bevy_shell.rs`.
The shared checkout's app source and installed app are unchanged by this
control. Full-map fidelity and performance are still unverified.

### Fresh matched control and automatic raster routing

The source/dependency-matched ordinary-renderer control completed in 6m19s
and was preserved as `exact-runtime-source-matched-control-client`, SHA256
`49f7a0b49f3f59727892c82bab9181b6a6cc83cda5e247f7dd53d99d96c5ce2c`.
The private app source registration was restored after the compiler completed;
no temporary control block remains. This control uses the current source and
Bevy features and omits GPU-plugin activation, with no runtime/player toggle.

Its same-quality 60-second Roof walk completed 963 queue callbacks, 16.022/sec,
update median/p95 54.39/103.24 ms and movement 53.90/94.62 ms. All 352 movement
rows, three grid origins and no additional terrain builds completed. The older
ordinary-control executable is not an adequate current-source image reference.
The fresh control warmup was viewed against v4 at the same pose. Mean RGB byte
differences were 0.0585/0.0662/0.0329 overall; the sampled right forest strip
was identical, while the left forest strip averaged 0.2272/0.3458/0.1449. These
are much smaller differences than the old-control comparison. A matching
low-camera 3D/2D control capture is running. Performance still favors the
ordinary renderer over forced-hardware v4; the data are not a GPU speedup.

The hardware-only routing override had remained from early debugging, even
after the actual visibility-mask defect was found. A new explicit GPU test
cleared only that reserved tag in memory and exercised Bevy's normal size-based
software/hardware policy. It passed in 3.13s with 1,179,648 decoded vertex words
across 512 clusters, 9090/9090 colored pixels, mean RGBA byte error 0.0015144,
normal prepass and all 33-instance mask/ancestor/depth lifecycle checks. Its
image was viewed. No existing threshold was relaxed. Log:
`/tmp/geothite-exact-auto-raster-gpu-proof.log`.

Game asset preparation now leaves the routing field zero to use this policy.
The two hardware-reference fixtures explicitly set their reserved tag in test
assets; the automatic fixture executes untouched production bytes. The focused
current cluster suite passes 21 tests and the production runtime module passes
11 tests, zero ignored in both (`/tmp/geothite-exact-auto-cluster-suite-focused.log`,
`/tmp/geothite-exact-auto-runtime-suite-focused.log`). An earlier overbroad
`exact_` filter ran 153 tests: 150 passed and three untouched model tests failed
(Ho-Oh and Lugia icon/model position distinction assertions; neutral rig source
comparison missing required source-path environment variables). That broad run
is not green and is retained at `/tmp/geothite-exact-auto-production-suite.log`.
No unrelated model source or asset was changed to bypass those failures.

The GPU-enabled automatic-routing release build is now running:
`/tmp/geothite-exact-runtime-release-build-5-auto.log`. It also includes the
validated panorama-only worker scope. Full-scene automatic-routing performance
and final visual acceptance remain pending. The installed game is unchanged.

### Automatic-routing full-scene result and visible admission

Release v5 automatic routing completed in 6m11s and was preserved as
`exact-runtime-release-v5-auto-client`, SHA256
`ea3ae8107d4f76e425d6fe9d9d4f29e0204829fd6e6e344c7f6894e0503ae34e`.
Its Roof replay recorded 11.026 queue completions/sec, update median/p95
87.43/116.53 ms and movement 82.42/107.37 ms. The following fresh matched
ordinary control recorded 14.146 callbacks/sec, update 69.83/83.22 ms and
movement 69.73/87.29 ms. Both completed 352 movement rows, three grid origins
and no additional terrain builds. No owned compiler ran during either replay;
other authorized builds remained running. Automatic routing improved over the
forced-hardware trial but does not beat the matched ordinary renderer. No GPU
prototype is accepted for installation from these results.

The next private change limits admission to bounds intersecting an active
world view. Previously offscreen maps could consume the bounded cluster table
while useful visible geometry remained ordinary. Excluded or over-budget
instances retain their full ordinary layer-31 draw; no geometry is dropped.
Published GPU children leaving the view are hidden before inherited visibility
propagation, so they do not remain logical GPU instances. Re-entry restores
them. Bounds/frusta are updated before admission, and newly created children
receive the source's current global transform immediately instead of spending
their first extracted frame at the identity transform.

World cameras are recognized by their existing layer-31 boundary, including
secondary captures. Their other layers are preserved; actor-only cameras are
untouched. Prepared palette identities remain cached while their source assets
live, so ordinary view movement does not discard other views' warmup state.

The expanded GPU regression passes: an offscreen shared instance remains
ordinary without consuming a published GPU slot; a visible later instance
reuses its uploaded asset; leaving/re-entering the view restores exact pixels;
a secondary layer-31/7 capture retains geometry and layer 7; inactive views
release logical GPU instances; activation restores exact pixels; the first
GPU child transform matches its source; existing material, back-face, scaled
placement and hidden-depth checks still hold. The focused production runtime
module passes all 11 tests, zero ignored, in 6.01s:
`/tmp/geothite-exact-visible-admission-runtime-suite.log`. The preceding
prototype test also passed (`/tmp/geothite-exact-visible-admission-regression.log`).

Release v6 is building with four jobs in the private target cache, without
stopping other builds: `/tmp/geothite-exact-runtime-release-build-6-visible.log`.
Full-scene benefit is unverified. The installed app remains unchanged.


### Visible-admission v6 result and CPU follow-up

The completed v6 full-resolution roof replay returned 763 updates in 60s,
median/p95 76.62/104.00ms; movement 83.24/114.69ms, three grid origins and
zero additional terrain builds. Queue completion callbacks were 12.714/s
(764/60.090688s), with one outstanding at both endpoints and no poll errors.
These are not display FPS or GPU timestamp durations. This remains slower
than the nearby same-source ordinary control (14.146 callbacks/s), so v6 is
not accepted or installed. Binary SHA256:
`99b2608a45147d82056cb1cafe6d515f51774f253f86d70d8c571ab01c47c4ae`.
Log: `/tmp/geothite-exact-runtime-v6-visible-roof-walk.log`.

A private CPU follow-up retains occluder ancestor visibility results between
frames and invalidates them on relevant visibility, parent-link and component
removal changes. Changing a render leaf does not rebuild the ancestor cache.
Current-frame hide/unhide and explicit Visible override semantics remain
required. The existing hidden-cache regression now also exercises reparenting
and removing parent/visibility components. Tests are running in
`/tmp/geothite-hierarchy-cache-tests.log`; benefit is not yet measured.
Other applications/builds remain untouched following the user's No.


The refined CPU cache checks Ref change ticks only for cached ancestor entities;
it does not scan all render leaves with Changed filters. All 59 focused
occluder tests pass with the external TUI pack (2.65s):
`/tmp/geothite-hierarchy-cache-focused-pack-tests.log`. The first broader run
omitted CRYSTAL_RENDER_TEST_PACK and failed the pack-requiring NPC fixture;
rerunning with the completed external pack passed all tests.

An ordinary-renderer release with this CPU change is building in the private
release target, session 91061, log
`/tmp/geothite-hierarchy-cache-release-build.log`. The private app's exact
geometry registration is temporarily omitted solely to match the existing
ordinary-renderer control. Restore it from
`.local/grass-material-check/candidate-files/gpu-enabled-bevy-shell-before-hierarchy-control.rs`
after the build reaches terminal state and saving an immutable benchmark
binary. No root source promotion or installation has happened for this cache.
Do not edit these private compiler inputs while this build is live. Next:
complete this exact build, save immutable client, restore registration, compare
ordinary control and CPU-cache candidate with the same 60s roof route; accept
only measured gains with unchanged visuals and passing visibility behavior.


The CPU-cache ordinary-renderer release completed successfully in 5m12s.
Immutable client: `.local/grass-material-check/hierarchy-cache-ordinary-client`,
SHA256 `9e8851225e9ceb3319292c56d79aaef7642e0654efc7b9251b2e026bf766ce09`.
The private app's GPU source registration has been restored after the build
terminated. The candidate's first 60s roof replay is session 33276, log
`/tmp/geothite-hierarchy-cache-roof-first.log`, output
`.local/grass-material-check/hierarchy-cache-roof-first`. Follow it with the
immutable same-source ordinary control (no CPU-cache change), same settings.
No installation or root source promotion yet.

The CPU approach is consistent with Bevy's documented query costs:
https://docs.rs/bevy/latest/bevy/ecs/query/struct.Changed.html and
https://docs.rs/bevy/latest/bevy/prelude/struct.Query.html.
Change detection is not an archetype-only filter; inspecting a bounded set of
known ancestors avoids scanning every matching scenery leaf. This is a design
rationale, not evidence of a full-scene gain.


CPU-cache first order (candidate then ordinary control), same 2560x1600 roof
route, 60s each: candidate 900 updates, median/p95 59.29/122.48ms,
movement 57.30/127.84ms, 14.608 queue completion callbacks/s (879/60.170528s).
Control 816 updates, median/p95 73.63/101.99ms, movement 72.15/105.25ms,
13.206 callbacks/s (794/60.124601s). Both three origins, zero extra terrain
builds, zero poll errors. Outstanding submissions grew 6→28 and 14→36.
The median/throughput improvement is accompanied by worse candidate tails;
this pair alone does not establish smoother gameplay. Other builds remained
running, untouched. Reverse-order repetition is required.

Reverse-order control is running as session 48937, log
`/tmp/geothite-hierarchy-cache-control-second.log`, output
`.local/grass-material-check/hierarchy-cache-control-second`. After terminal
completion, run the immutable `hierarchy-cache-ordinary-client` with output
`hierarchy-cache-roof-second` and the same flags. No owned compiler is running;
private GPU app registration is restored, and no root promotion/installation
has occurred.


Reverse order completed: ordinary control 841 updates, median/p95
70.44/97.08ms, movement 71.28/91.14ms, 13.738 callbacks/s
(825/60.054564s). CPU-cache candidate 939 updates, median/p95
61.50/73.73ms, movement 60.53/72.67ms, 15.632 callbacks/s
(940/60.134267s). Both three origins and zero extra terrain builds/poll errors;
outstanding 20→36 and 36→36. Median and callback throughput improve in both
orders, while first-order candidate tails were worse. The first warmup image
RGB mean-byte differences were 0.045814/0.036533/0.023660; candidate viewed.
This supports keeping the CPU change for production verification, not a claim
of fluid gameplay or a general 60 FPS result.

Only the ancestor-cache implementation and expanded hide/reparent/removal
regression have been promoted into root `occluder_fade.rs`; a pre-promotion
backup and exact patch are kept under ignored `.local/grass-material-check`.
No GPU prototype code/dependency was promoted. Root tests run as session
26536, `/tmp/geothite-hierarchy-cache-root-tests.log`, with the external TUI
pack and shared isolated target `target/performance-visibility`. Paired
candidate 3D/2D screenshots run as session 86582, log
`/tmp/geothite-hierarchy-cache-roof-paired.log`; inspect both and the preserved
state receipt before visual acceptance. No new app installation yet.


Root focused tests completed successfully: 59 passed, zero failed/ignored,
2.43s, `/tmp/geothite-hierarchy-cache-root-tests.log` (session 26536 terminal0).
Paired candidate capture completed successfully (session 86582 terminal0).
Both images were inspected: roof walkway/ridges and trainer placement remain
consistent with the 2D location; forest depth, scenery colors, cloud lighting
and character readability remain intact relative to the matched control.
The 3D view is perspective while 2D is top-down; no model fidelity claim beyond
preserving the cache control's scene is inferred. Receipt:
`state_preserved=true`, frame577/hash1478414892, virtualseconds10.527715626,
Chris center[-1248,960], facing[0,-1], presentation center[-1280,992],
zoom0/rotation7/pitch5/first-personfalse. Capture durations17.43/24.96s are
not door-transition measurements.

A fresh production source snapshot was copied from current root into ignored
`.local/hierarchy-cache-production/source` (Cargo files, crates, assets,
modpacks, tools, docs, README and optional .cargo), preserving current shared
model/source work without the private Bevy fork or GPU prototype. Its native
release build is session27746, log
`/tmp/geothite-hierarchy-cache-production-build.log`, target
`target/performance-release-native`, jobs2. Do not edit those frozen inputs or
restart a live build. It builds crystal-bevy desktop plus pack verification
and artwork bundlers. No new app installation yet. Next: poll this exact handle,
validate resulting pack/artifacts, run required controller/MCP/stdio regressions,
then package/install and verify content/signing/save preservation. The full
smoothness objective remains active; roof callback throughput15/s is not fluid
performance or displayed FPS.


Production build session27746 failed: the frozen snapshot lacked the compiled-in
`art/battles/arena-palettes.json`, and current buoy-course geometry referenced
Vec2 through a test-only import. Copied the small source palette directory and
LICENSE into the snapshot, and added a normal Vec2 import to root and snapshot
`mesh/buoy_course.rs`. An initial repair script wrongly asserted that `use
super::*` occurred only once (it also occurs in the test module), so its import
edit did not run; the subsequent build session42635 failed on the same three
Vec2 references. This was corrected by replacing only the top-level import.
Both failed sessions are terminal; no live build was restarted or modified.

Corrected production build is now session65654:
`/tmp/geothite-hierarchy-cache-production-build-fixed-2.log`. Its snapshot
inputs are frozen again. The snapshot has no meshlet feature, Bevy PBR path
fork or exact-geometry source registration. Required root gameplay regressions
run sequentially in session12184 (wasm-dev/jobs2, external TUI pack, isolated
performance-visibility target): `renderer_neutral_controller_`, then geothite
`mcp_drives_real_movement_and_production_start_menu`, then `--test stdio_clean`.
The test output is retained on the live command handle, not a separate log.
No installation yet. Poll those exact handles before taking the next action.


The renderer-neutral controller filter passed eight tests (14.11s), including
the required battle command/PP mutation and Mom/New Bark gate regressions.
One pre-existing department-store checkout test remained ignored; it is not
counted as passed. Session12184 is still live, now building the geothite MCP
and clean-stdio regressions. Corrected production session65654 remains live;
no compiler inputs changed after it started. Save/pack/executable pre-install
hashes were recorded under ignored
`.local/hierarchy-cache-production/before-install.json` (one existing save).


Required gameplay session12184 completed successfully (terminal0). Controller:
eight passed, one existing ignored; battle-turn/PP and Mom/gate included. MCP
movement/production Start regression: one passed in0.97s. Clean stdio
`normal_mcp_gameplay_keeps_stderr_clean`: one passed in1.47s, asserting empty
stderr through Mom dialogue in the spawned client. No additional testing is
required for this unchanged controller surface unless a later failure appears.
Production build session65654 remains live; packaging/install still pending.


### CPU ancestor cache installed and verified

Corrected production release session65654 completed in9m24s (terminal0).
Installer session3512 completed in32.3s (terminal0), log
`/tmp/geothite-hierarchy-cache-install.log`: external pack verified, scenery
and people bundled from frozen production artwork, staged check-installation
passed and strict deep signing verified before promotion. Installed app:
`~/Applications/Geothite.app`. Final session71134 (terminal0) reran installed
`--check-installation` from `/tmp` with CRYSTAL_OPEN_MODEL_ROOT unset, verified
strict deep codesign, and asserted both source/bundled pack hashes plus the
existing save hash unchanged. One save preserved byte-for-byte.

Installed executable SHA256:
`c111c64d1faac3f951071f600a92be87edc4fc0f5cf3299e4900e26229834d72`.
Unsigned release SHA256:
`3f2bdb0cdcd0fd2e431e1011857e597be7b68127daca39e07edc8bca75f39378`.
Unchanged pack SHA256:
`a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007`.
Verification metadata is ignored at
`.local/hierarchy-cache-production/installed-verification.json`.

No new gameplay toggle, timing-tool implementation, GPU prototype dependency,
pack generation, commit or visible window was introduced. This ships the CPU
cache and the necessary normal-build Vec2 import; current shared source/model
work was preserved in the frozen source. The earlier paired roof gains came
from the isolated same-source ordinary-renderer comparison and must not be
represented as measured display FPS for this newer production source snapshot.
The full loading/fluidity objective remains active. Next useful work is to
build the location-tester example from this same production snapshot (no
experimental fork) and inspect current hot paths under actual walking. No
owned compiler or test handle is live after these terminal completions.


### Current production renderer trace and dense-panorama follow-up

Production-snapshot location tester built successfully in6m21s (session40129
terminal0), log `/tmp/geothite-production-cache-location-build.log`. Immutable
ordinary client: `.local/hierarchy-cache-production/ordinary-frame-client`,
SHA256 `a56ac8aabe3b0717e03740c27846db4a792473894e025f9d385a8a5ffa65be15`.
Its 60s full-resolution roof walk with existing opt-in trace completed
(session16459 terminal0), log `/tmp/geothite-production-ordinary-roof-profile.log`.
805 updates, median/p95 76.60/99.07ms; movement70.88/98.90ms, two grid origins,
zero extra terrain builds; queue callbacks12.953/s (778/60.063071s), outstanding
8→36, no poll errors. Tracing and other builds make this diagnostic, not a new
untraced A/B benchmark. The bounded trace has131072 events and drops older
records; inspect retained late events rather than treating it as full startup.

Last20s diagnostic medians: render frame73.09ms, main71.149ms,
render.graph_submit55.687ms, prepare_gpu12.157ms, visibility envelope10.978ms,
occluder span9.892ms, check-visibility envelope8.426ms, transforms2.214ms.
Schedule envelopes nest/overlap, are not additive self times. A submission
wait is not a GPU timestamp duration. Occluder metadata reports167987 groups,
168841 leaf entities and1356360 deferred sorted leaves. Existing tracing also
walks these groups to produce metadata, so it can affect this diagnostic cost.

The native task allocation is4 foreground workers/5 background/1 IO on this
10-core host. Expanding foreground workers is deferred: the trace points more
strongly to render submission than scheduling. The next private experiment
uses stock Bevy two-phase GPU occlusion culling plus DepthPrepass for this
very dense panorama, retaining original geometry, detail, resolution and view.
Earlier ordinary-route occlusion checks failed to improve; no global enablement
is justified. This trial must prove a roof-specific benefit and exact visible
results before any production context-specific use. Bevy's installed source
confirms bounding-box conservative culling and depth-prepass requirement; GPU
capability support must also be confirmed in the runtime log.

Private trial only: added native world-camera DepthPrepass+OcclusionCulling in
`.local/hierarchy-cache-production/source/.../lib.rs`, preserving the ordinary
file at `.local/hierarchy-cache-production/ordinary-voxel-lib-before-occlusion.rs`.
No root runtime source, dependency or player toggle changed for this trial.
Build is live session72115, log `/tmp/geothite-panorama-occlusion-build.log`,
existing release target/jobs2. Freeze those compiler inputs. After terminal
completion, save an immutable candidate, verify capability/shader correctness,
run untraced full-resolution roof candidate/control pairs, and inspect paired
3D/2D plus motion/disocclusion behavior. Installed app stays the verified CPU
cache release. Other apps/builds are untouched.


Occlusion trial build session72115 completed successfully in5m41s (terminal0).
Immutable candidate `.local/hierarchy-cache-production/occlusion-frame-client`,
SHA256 `fd23b81750bce6fbf988383b189f70a0dffc008747f201fad1c799b3fa19cc36`.
Saved experimental lib.rs as `occlusion-voxel-lib-prototype.rs` in that ignored
directory and restored the production snapshot lib.rs to the ordinary renderer
after compilation. No root occlusion change or player switch exists.

Candidate first roof replay session3782 passed warmup/shader checks (terminal0).
Runtime explicitly logs GPU preprocessing fully supported on this device.
Untraced60s/2560x1600/UUDUDUDUDUD:1125 updates, median/p95 53.51/59.47ms,
movement53.49/58.33ms, callbacks18.751/s (1127/60.104419s), outstanding32→31.
Fresh ordinary control session58541 terminal0:909 updates,66.08/76.39ms,
movement58.52/71.91ms, callbacks15.130/s (910/60.145664s), outstanding36→36.
Both352movementrows/two origins/zero additional terrain builds/poll errors.
Logs `/tmp/geothite-panorama-occlusion-{roof,control}-first.log`.
No owned compiler was running; other compilers had completed without being
stopped. These are queue callbacks and host intervals, not displayed FPS.

The candidate warmup image was viewed and retained roof/trainer/forest detail.
Relative to the prior production-profile warmup (different settled animation
time), mean RGB byte differences were0.014296/0.012366/0.009049. This is an
initial consistency check, not complete moving-camera/paired-state validation.
Depth/prepass shader source applies the same grass deformation and clock as
forward rendering. Native whole-route occlusion is still not justified by
this roof result; earlier ordinary-route checks regressed.

Reverse-order ordinary control is live session75108:
`/tmp/geothite-panorama-occlusion-control-second.log`, output
`.local/hierarchy-cache-production/occlusion-control-second`. After terminal,
run immutable occlusion-frame-client with identical flags and output
`occlusion-roof-second`. If both orders improve, implement and test a scoped
automatic dense-panorama policy, including destination pipeline preparation,
return to ordinary views, secondary cameras and retained pre-existing depth
prepass ownership. Inspect paired3D/2D and moving/disocclusion captures before
promotion/installation. Installed app remains the verified CPU-cache release.


Reverse occlusion comparison completed. Ordinary session75108 terminal0:
1110 updates,53.17/58.55ms median/p95, movement52.52/58.07ms,
18.498 queue callbacks/s (1111/60.062003s), outstanding36→36. Candidate
session25422 terminal0:1137 updates,53.11/59.53ms, movement51.70/56.15ms,
18.951 callbacks/s (1139/60.103291s), outstanding32→31. Both zero extra
terrain builds/poll errors. The strong first-order gain did not repeat in
update intervals; culling remains rejected for promotion. Ordinary production
snapshot lib.rs is restored; no app/source/runtime toggle changed.

A direct current-source house-exit diagnostic completed session38264 terminal0,
log `/tmp/geothite-house-exit-current-profile.log`, output
`.local/hierarchy-cache-production/house-exit-current-profile`, trace
`house-exit-current.trace.json` in the same ignored root. Route PlayersHouse1F
(6,6), DDBBBBBBBBBB,20s,2560x1600,noon:896 updates,19.82/20.98ms,
movement384rows/19.54/20.93ms,two origins,zero extra terrain builds. Trace has
33325events,zero dropped. State changes: House3D/pendingfalse4.582751s;
NewBark3D/pendingtrue6.460295s; NewBarkpendingfalse8.445886s. **This is a
synchronous-pipeline QA trace, not normal-play door latency.** Median late
render.submit17.485ms/graph_submit17.128ms, main2.168ms, prepare_gpu1.506ms.
During entry, render.submit reaches1589.94ms while graph_submit remains small,
pointing to compilation/work before graph submission. Four empty-mesh warnings
are not a per-frame upload storm. Avoid dumping individual tree events: there
are thousands; aggregate/bound output.

Code inspection exposes a concrete preload mismatch: render_quality changes
Gaussian indoors to Hardware2x2 outdoors, while pipeline_preload specializes
hidden destination materials only using the current indoor WarmupView key.
It already warms directional-shadow variants but not the alternate color-view
shadow-filter key. Thus the first outdoor material needs another pipeline.
Destination initial grove residency is12 versus325requested at entry, also
worth investigating after the shader fix; do not conflate source-build hits
with full scene readiness or infer normal-play pause from forced-sync QA.

Expanded the private existing native GPU regression to start with a Gaussian
warmup view, require a compiled Hardware2x2 variant while the destination is
hidden, then switch the visible camera to Hardware2x2 and require exact cached
pipeline ID reuse. The before-fix test is building as session67498, log
`/tmp/geothite-destination-filter-regression-before.log`. Private snapshot
pipeline_preload.rs contains ONLY this test change so far; original backup:
`.local/hierarchy-cache-production/ordinary-pipeline-preload-before-filter-fix.rs`.
No implementation fix or root runtime change yet. After terminal, inspect the
specific failure, then prepare both legitimate color-filter variants while
preserving all other view/material bits and existing bounded queues; rerun the
same regression, and check the real house exit and matched 3D/2D. Do not add a
player setting. Installed app remains the verified CPU-cache release.


The expanded before-fix native GPU test session67498 failed as intended in0.97s
(terminal101), specifically: "outdoor hardware-filter shader must be warmed
while indoor view is Gaussian". This proves the missing specialization, not
normal-play pause duration. Implemented warming the live view plus Gaussian
and Hardware2x2 color-filter variants, clearing only the shadow-filter bits
and preserving every other material/view bit. HashSet and existing pending/
work/signature limits still bound speculative work; no camera, geometry,
quality setting, additional draw, control or shader implementation changed.
Prepass and directional-shadow key handling remains unchanged.

After-fix GPU session94710 passed in2.03s (terminal0), including hidden-to-visible
outdoor pipeline-ID reuse, directional shadow/camera-prepass reuse, exclusion
of hidden placeholder actors and future player-reveal material compilation.
The future-reveal selection then received an explicit Hardware2x2 filter to
avoid nondeterministic selection from the newly warmed alternatives. Final
session66359 passed in0.33s (terminal0), log
`/tmp/geothite-destination-filter-regression-after-fixed-selection.log`.

Root pipeline_preload.rs was byte-identical to the frozen original backup;
after reasserting that equality, promoted this implementation and expanded
regression to root without overwriting unrelated source work. The current app
is NOT yet updated with this fix. A release QA example is building from the
same corrected frozen production snapshot as session71490:
`/tmp/geothite-destination-filter-release-build.log`, release target/jobs2.
Freeze its private source inputs until terminal. On completion save immutable
`destination-filter-frame-client`, run the identical PlayersHouse1F(6,6)
DDBBBBBBBBBB diagnostic with separate output/trace and inspect presentation
readiness/cold pipeline work relative to ordinary-frame-client. Remember these
QA captures force synchronous compilation; never claim a production door latency
from them. Verify matched 3D/2D appearance, root GPU regression and required
gameplay checks before a native release/install. Also investigate why initial
preloaded grove residency is12 versus325requested if the readiness gap remains.
No other app/build was stopped; no owned build besides71490 remains live.


Root native GPU regression session1838 passed in3.06s (terminal0), log
`/tmp/geothite-destination-filter-root-regression.log`. QA release session71490
completed in4m49s (terminal0), log
`/tmp/geothite-destination-filter-release-build.log`. Immutable fixed client:
`.local/hierarchy-cache-production/destination-filter-frame-client`, SHA256
`d911ccad5734f2d3696c617d8361861e68131cff1e74f91d20eefca29cdac30f`.

Identical20s house-exit diagnostic completed session4183 terminal0, log
`/tmp/geothite-house-exit-filter-fixed-profile.log`, trace
`house-exit-filter-fixed.trace.json` in the production scratch root.878 updates,
median/p9525.35/31.18ms; movement384rows/24.61/28.95ms,two origins,zero extra
terrain builds.31740trace events,zero dropped. Presentation: House3D ready
6.452714s; NewBarkinactive/pending7.782711s; active/pending8.094859s;
fully settled8.778181s. The pending span is shorter than the earlier forced-sync
diagnostic, but startup/publication order differs and steady intervals are
worse. Do not claim a production latency or overall FPS gain from this pair.
Current QA forces synchronous pipeline compilation. The native regression
provides direct proof of hidden outdoor shader compilation/cache reuse.

Further confirmed source mismatch: destination_preload::prepare builds its
perspective using frame.viewport_size.x/y (classic1.111...), while live
sync_voxel_view uses the actual world-camera PerspectiveProjection.aspect_ratio
(1.6 for this target). In the old trace, preloading requests325trees, and later
actual wide view requests391. Initial residency12 is NOT a hard12tree preload
limit: it reflects partial publication before entry; both the first preload
and first activation already requested325. Do not invent a fixed small cap.
The preload settled_key also omits output aspect, so a window resize can leave
a hidden exterior settled for the old footprint. Next concrete fix: query the
existing world camera's Projection for lookahead frustum aspect, preserve the
fallback for minimal/headless fixtures, and invalidate settled readiness on
aspect change (retain original camera FOV/near/far and bounded stream budget).
An existing settled-destination regression can be expanded to cover resize;
verify actual requested grove footprint, not only a projection helper formula.

A related issue worth handling with care: host destination source_frame has
actors empty and center/view_center initially zero, then anchor_to_map. Live
camera.world_pose uses the player ground point, footing height and foreground
terrain adjustment; empty actors fall back to view_center with no such support.
Main publish_world also accounts for source-grid versus viewport-center offset
before anchoring. Do not duplicate approximate coordinates or mutate controller
state. Final fixed-run authoritative position is NewBarkTown, player visual
center[-1056,1504], grid origin[-32,-48]. A stable runtime-tile visual center
for a standard64px human has source coordinates
(2*tile+1-grid_size/2)*tile_size with Y inverted; actor_ground_point subtracts
half sprite height then adds one visual-cell height (zero offset for64/32).
Any predicted player must remain renderer-only and match the shared production
coordinate conversion; prefer reuse/refactoring over a second actor dispatcher.

No render people-review/game process was observed during the final read-only
process check. No other process was stopped. No owned compiler/test/replay is
currently live. Shader fix is in root and frozen production source; installed
app remains the previously verified CPU-cache release. Next: finish arrival-view
matching, validate real route/paired images, and package the verified changes.


### Arrival-view aspect and resize invalidation

Expanded the existing settled-hidden-destination regression with a world
camera and a1→2aspect resize while source/settings remain unchanged. Before
implementation, session21580 failed as intended (terminal101), log
`/tmp/geothite-destination-view-regression-before.log`: hidden destination was
not refreshed. Changed private prepare to read the existing VoxelWorldCamera
PerspectiveProjection.aspect_ratio (finite/positive), falling back to the
classic aspect only when no valid perspective camera exists. The predicted
FOV/near/far and world pose are retained. Added aspect.to_bits() to settled_key
so resizing can refresh the predicted footprint without another terrain build.
No player setting, geometry or rendering-quality change.

The expanded old test stub required realistic terrain data once it actually
re-entered prepare: its empty tile list caused cache retirement, then its empty
footing array caused an align_footings bounds panic. Completed the one-cell
fixture with image/tile and matching footing data. These failures were fixture
incompleteness, not evidence that production source grids are empty. Final
session38776 passed all6 destination tests in0.04s (terminal0), log
`/tmp/geothite-destination-view-regression-after-complete-fixture.log`.
Other terminal test sessions:61026 and51122 (fixture failures). A direct run of
the owned compiled test binary identified the missing footing array; it was
not a build restart or a stopped external process.

Root destination_preload.rs was verified byte-identical to the frozen original
backup `.local/hierarchy-cache-production/destination-preload-before-view-fix.rs`
before promotion. Only this tested aspect implementation/regression was then
copied into root. Source-frame player/pivot metadata has NOT been changed.
Avoid speculative camera/actor rewrites until the new replay shows whether
anything remains missing; the actual grove footprint must be inspected.

Release QA example is now building as session90254, log
`/tmp/geothite-destination-view-release-build.log`, existing isolated release
target/jobs2, frozen production snapshot. Freeze its private inputs. Root six
preload tests are live session86325, log
`/tmp/geothite-destination-view-root-tests.log`, performance-visibility target,
wasm-dev/crystal-voxel-view opt0/jobs2, external TUI pack. Next poll those exact
handles. After release terminal, save immutable destination-view-frame-client,
run PlayersHouse1F(6,6) DDBBBBBBBBBB20s profile with separate output/trace;
check preloaded requested tree footprint versus actual wide arrival, retained
3D/pending intervals, foreground build counts and the shader cache. Do not
claim normal-play door latency from forced-synchronous QA. Then inspect a
matched final NewBark3D/2D capture and required regressions before production
packaging/install. Installed app remains CPU-cache release; shader+aspect
fixes are in source only. No other apps/builds were stopped.


Root destination tests session86325 completed terminal0:6passed/0failed/0ignored,
0.04s, `/tmp/geothite-destination-view-root-tests.log`. Release90254 remains
live on the exact original handle; do not restart it or edit its frozen inputs.


Combined shader/aspect QA release90254 completed in4m51s (terminal0). Immutable
`.local/hierarchy-cache-production/destination-view-frame-client`, SHA256
`c2b438432163f5106f9dce3838c0bf2623fc348dc70b633dab1269258cf78568`.
Required root regressions session97522 completed terminal0: controller8passed
(including battle PP/Mom/gate),1existing ignored; MCP movement/Start1passed
in0.89s; clean stdio1passed in1.44s. No controller bypass was added.

Combined20s house-exit replay57174 completed terminal0:
`/tmp/geothite-house-exit-view-fixed-profile.log`, trace
`house-exit-view-fixed.trace.json` under the production scratch directory.
1013updates, median/p9519.93/21.01ms; movement384rows,19.75/21.77ms,two origins,
zero additional terrain builds.37014traceevents,zero dropped. House ready
4.765572s, first NewBark presentation6.488525s is already3D-active/pendingfalse.
There is no NewBark pending presentation in this replay. Preload uses actual
projection xscale1.1548494, requesting426trees rather than old325 at logical
xscale1.6629831. Later actual requests399–401, with399resident by6.9007s.
Maximum render.submit in the first2s after NewBark is108.415ms, versus the
old forced-sync entry maximum1589.94ms. These are synchronous-compilation QA
observations, not normal-play door latency or display FPS. The stronger
functional proof is correct wide-view preparation and ready3D on the first
Town presentation without another foreground terrain build. Do not add a
speculative actor/pivot rewrite when this verified footprint already covers
arrival; investigate only if a later route exposes missing geometry.

Paired final Town capture after the same house walk is live54396:
`/tmp/geothite-house-exit-view-fixed-paired.log`, outputs
`house-exit-view-fixed-{3d,2d}.png` in the production scratch root. Inspect both
and paired-state receipt before acceptance. It may overlap compilation, so
capture duration is not a latency benchmark. Native production release is
live89869, log `/tmp/geothite-destination-preload-production-build.log`, frozen
production snapshot, release target/jobs2. Freeze its inputs until terminal;
do not restart. Save/pack/current executable hashes for this installation are
at `before-preload-install.json` (separate from the earlier CPU-cache install).
After successful build and visual checks, package/install with frozen artwork,
verify content, outside-repo installed check, strict codesign and unchanged save.
Installed app remains CPU-cache release until that succeeds.


Paired final Town capture54396 completed terminal0. Both images were inspected:
house/player relationship, nearby teacher/fisher placement, neighboring lab,
forest, ground, water and materials remain present/readable, without a new
preload-related composition/lighting/scenery defect. Existing authored3D house
design differs from the classic facade; no model/color redesign was performed
or accepted here. Receipt state_preserved=true, checksumframe632/hash783970903,
lcdframe647, virtualseconds11.185214167, four actors with identical3D/2D
sources/positions/facing. Chris[-1056,1504], facing[0,-1]; presentation
NewBarkTown center[-1024,1536], zoom0/rotation7.5/pitch45/first-personfalse.
The screenshots are2560x1600 and show completed Town after the real DD house
exit; their capture durations overlap compilation and are not latency results.

Production build89869 remains live; this is the only owned process left.
Next poll that exact handle and inspect
`/tmp/geothite-destination-preload-production-build.log`. After terminal success,
install using:
CRYSTAL_OPEN_MODEL_ROOT="$PWD/.local/hierarchy-cache-production/source/assets/open-models"
python3 tools/desktop.py install --skip-build --profile release
 --target-dir "$PWD/target/performance-release-native"
 --pack "$PWD/content-packs/core-modular.crystalpack"
(join these lines into one environment-prefixed command). The installer must
verify pack/art, staged check-installation and signing before replacing app.
Then run installed --check-installation from /tmp with OPEN_MODEL_ROOT unset,
strict deep codesign, and compare save/source+bundled pack hashes against
`.local/hierarchy-cache-production/before-preload-install.json`. Record the
new executable hash. No visible window/app switch or process shutdown is needed.
Do not claim the full fast/fluid objective complete: roof/wider-game throughput
remains unresolved even though this house-exit readiness case now passes.


### Shader and arrival-width preload fixes installed

Native production89869 completed terminal0 in6m45s, log
`/tmp/geothite-destination-preload-production-build.log`. Installer99618
completed terminal0 in34.9s, log `/tmp/geothite-destination-preload-install.log`.
Pack/art verification, staged external-directory check-installation and strict
codesign passed before replacement. Installed `~/Applications/Geothite.app`
now includes the CPU ancestor cache plus both shader-filter and actual-camera
aspect/resize preload fixes. No meshlet fork, rejected occlusion trial, LOD,
quality reduction or new player toggle was installed.

Final installed verification24461 completed terminal0 from/tmp with
CRYSTAL_OPEN_MODEL_ROOT unset: pack/resources/external save path resolved,
strict deep codesign passed, bundled/source pack SHA and the existing save
SHA matched `before-preload-install.json`. One save unchanged byte-for-byte.
Installed executable SHA256:
`48fc51d38a67c6949821b774c04664df14677a8139ad838da4b4fa2589723c43`.
Unsigned release SHA256:
`29bc3af4f92b2bc2bf1821bcec7ebef0519904d884d24f609f65978c10828a8a`.
Pack SHA256 unchanged:
`a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007`.
Metadata: ignored `preload-installed-verification.json` in production scratch.

The verified headless near-immediate house exit recorded NewBark3D ready on
its first Town presentation, no extra foreground terrain build, with matched
final3D/2D game state and readable scenery/actors. This must not be converted
into a claimed normal-play latency or display FPS; QA forces synchronous
compilation. Required controller battle PP/Mom/gate, MCP movement/menu and
clean-stderr regressions passed as recorded above. Six destination tests and
the native hidden-to-outdoor pipeline cache-reuse regression passed in root.
No content pack was regenerated/staged/committed; no saved progress changed;
no visible window or external process shutdown was used.

All owned build/install/test handles are now terminal; no owned process needs
polling. Goal remains active: the dense roof and wider-game throughput are
not yet fluid. Do not call the full objective complete from the house-exit
case. Ordinary GPU occlusion failed to repeat its roof gain; it is rejected.
The older private meshlet prototype also failed matched throughput and is not
in root/app. Next work should address the remaining measured renderer cost
while preserving the installed appearance and current shared model edits.


### Private panorama-only two-section trial

Installed preload release is unchanged. Inspected panorama origin/transform:
it is already stable for ordinary walking, so no redundant per-frame movement
fix is justified. Stock GPU occlusion failed repeatability; the older four-
section global/combined-culling trials are documented near the beginning of
this journal and are not accepted gains. This new private experiment limits
opaque subdivision to TWO fitted sections and ONLY native summit-panorama
canopies, reducing the earlier entity-cost exposure. No new culling pass,
shader, LOD, geometry/detail reduction, player control or dependency is added.

Ported the existing tested opaque-section support into the current private
snapshot via a359-line patch whose git-apply check passed. It preserves newer
CPU/preload fixes. Original current files are backed up under ignored
`.local/hierarchy-cache-production/panorama-section-baseline/` (occluder,
worker wire, shared groves, forest library and summit vista). Port patch is
`panorama-sections-port.patch`. Fitted split count is2. Source collision and
sorted transparency remain whole; sharing, compression, exact vertex welding,
localization, upload, overlay handling and OpaqueObjectLeaf publication include
all sections. The unchanged worker wire explicitly rejects native sections
rather than dropping geometry. Its default/browser/terrain-worker path never
constructs them.

Scoped preparation: default shared_groves::prepare_surface remains whole;
prepare_panorama_surface invokes splitting before sharing. Native non-worker
Library::for_panorama_sections has a distinct bounded cache and only partitions
canopies; ground remains whole. SummitVista::Build lazily owns that library.
Browser/terrain-worker configurations use their original library. Normal town/
house/preload geometry is unaffected. No application culling probe from the
older candidate was ported.

Native occluder session70473 completed terminal0:61passed/0failed/0ignored,
2.95s, `/tmp/geothite-panorama-two-sections-tests.log`. Tests compare every
triangle's attribute bits before/after packing/localization for pine, ancient
and deciduous trees, preserve collision/fade leaves, reuse section handles,
and prove normal libraries remain unpartitioned while panorama libraries use
two draws with the same source transform/fade ownership. Source draw order
may differ, so actual pixel/visual comparisons are still mandatory.

Private release QA build is live56926, log
`/tmp/geothite-panorama-two-sections-release-build.log`, release target/jobs2.
Freeze these inputs and poll that exact handle. Worker serialization check
78732 is terminal101: build.rs finish_kernel_identity failed at line103 with
"kernel input metadata: No such file or directory". The snapshot omitted
root rust-toolchain.toml; other listed kernel inputs exist. This is a missing
snapshot input, not proof the worker changes compile. After the release build
is terminal, copy the root1.99.0 minimal toolchain manifest into the snapshot
and rerun the exact terrain-worker/occluder_fade::worker_wire:: suite, log
`/tmp/geothite-panorama-two-sections-worker-tests.log` (new log for retry).
Do not modify frozen inputs or restart the live release build to do this.

On successful release, save immutable panorama-two-section-client. Compare
against destination-view-frame-client (same frozen source/appearance except
this trial) on the60s full-resolution Roof9,13/noon UUDUDUDUDUD route, after
owned compilers/tests terminate. Use untraced AB/BA pairs; initial warmup and
queue callbacks are not display FPS. Retain only repeated gains with unchanged
pixels/whole-tree fades and passing2D pairing. A test pass alone does not prove a speedup; implementation remains private
until measured performance and visual checks justify root promotion/install. Do not touch other apps/builds, staging, packs or saved progress.


### Two-section trial rejected after measured comparison

Private release56926 completed terminal0 in6m58s. Immutable
`panorama-two-section-client` SHA256:
`e3f7c48628146bdf6dc2062cc593e34a3dc7a5405ec837f1e0c7580b00919b45`.
After terminal completion, restored root rust-toolchain.toml into the frozen
snapshot (same1.99.0 toolchain). Worker retry7780 completed terminal0:
6passed/0failed/0ignored in0.03s,
`/tmp/geothite-panorama-two-sections-worker-tests-fixed-input.log`.

Candidate paired capture12868 exited1: first3D image completed60.74s, then
classic second capture exceeded the shared60s QA deadline. The existing driver
keeps the original startup deadline across the pair; this is an application
capture timeout, not a missing live tool handle. No timing-tool code changed.
Matching ordinary control67480 completed terminal0,3D34.83s/2D43.66s total.
Candidate3D image was viewed: roof/player/forest retained; mean RGB byte
differences against same-source ordinary3D were0.074863/0.052678/0.026578.
These capture times are not controlled game load/door latency; candidate capture
overlapped worker compilation. It did NOT pass the full paired-state gate.

Untraced full-resolution roof walk candidate99921 completed terminal0:
1073updates, median/p9552.18/70.40ms, movement51.28/54.02ms,352rows/two origins,
zero extra terrain builds.1074queue callbacks/60.082251s=17.875/s,
outstanding36→36,no poll errors. Fresh matching control91176 terminal0:
1069updates,55.43/62.99ms, movement53.35/56.89ms,same rows/origins/zero builds;
1070callbacks/60.097988s=17.804/s,outstanding36→36,no poll errors.
GPU queue throughput differs only0.4%, and candidate tails are worse. This does
not justify extra entities/draws or additional acceptance work. Trial rejected;
no reverse-order claim, paired acceptance, root promotion or installation.

Archived five private modified source files under ignored
`panorama-two-section-source/`, then restored all five from
`panorama-section-baseline/`. Native parts/factory/library markers are gone
from active private source. The copied toolchain manifest remains, correcting
the snapshot input. Shader/aspect/CPU-cache fixes remain intact. All owned
build/test/capture/benchmark handles are terminal. Root/app/pack/save unchanged
by this trial; installed preload release remains SHA48fc51d3... as above.

Potential next CPU work (NOT implemented): inactive world views still scan all
occluder groups every host frame, even after every fade is opaque/clear. A
small local frame-history cache can skip this work only while input.2 is false
and an inactive full pass has proven all groups unblocked, opacity1,hold0,
blendingfalse. Do not scan again on active frames merely to maintain this cache.
Keep settling non-opaque fades with Time<Real> as before. On wake, invalidate
ancestor visibility and force refresh of cached inverse/bounds even when
transform/visibility changes happened many skipped ticks ago. Test moving a
clear object into the ray while inactive, then waking to a correct block; also
hide/unhide during skipped frames and verify hierarchy recovery. This could
remove retained-world CPU scans during battles/classic views without changing
3D quality or adding a toggle. It does not solve the remaining GPU throughput
ceiling and must not be presented as completion of the full objective.


### Private inactive-world occluder sleep path

Started from restored whole-tree/preload snapshot; no rejected subdivision or
GPU culling code remains active. Backup before this change is ignored
`inactive-fades-baseline.rs` in the production scratch root. Root occluder file
still matches that baseline; no root promotion or installation yet.

New lifecycle regression first failed on unchanged code (session25507 terminal101,
`/tmp/geothite-inactive-fades-before.log`): moving an object while a settled
owned camera was off changed its occluder cache tick32→122. The test then checks
correct blocking/inverse on wake, frozen hidden fades, unhide while still off,
real-time completion of visible fades, and recovery from old visibility ticks.

Implementation is only in private occluder_fade.rs. Local FadeFrameHistory
stores prior input, a dormant flag and IDs of hidden unsettled fades. After a
full inactive pass proves visible fades clear/opaque, skip repeated group scans.
Probe only hidden-fade ancestor chains while sleeping, so revealing one in
another view wakes settling even if the world stays off. New groups always
spawn with clear default fades; no gameplay/controller state is cached here.
On wake, clear ancestor visibility and force inverse/bounds refresh: changes
may have happened long before the last system change tick. Active walking
frames do not get an extra scan; settling-state scans run only when eligible
for sleep. Existing opt-in trace metadata is unchanged and may itself scan
all groups; do not mistake trace overhead for the normal runtime path.

First after-run27137 failed3 existing standalone fade tests (57passed), which
manually drive fades without a world camera. Preserved that supported context
without weakening those tests: cache only when cameras.single identifies one
owned world view. No/multiple world cameras retain the old processing path.
Owned-view version4539 passed all60tests in2.71s. Added the loading safeguard:
terrain_pending or profiles_pending keeps preparation running while inactive,
with regression checking new transforms still prepare during loading. Final
55872 passed60/0failed/0ignored in4.23s,
`/tmp/geothite-inactive-fades-loading-guard-tests.log`. No player toggle,
render-quality/shader change, serialization change or dependency added.

Private release QA build is live82619:
`/tmp/geothite-inactive-fades-release-build.log`, frozen production snapshot,
release target/jobs2. Freeze private inputs and poll the exact handle; do not
restart on observation timeout. On terminal success save immutable
`inactive-fades-frame-client`. Compare existing whole-tree baseline
`destination-view-frame-client` and candidate using offscreen paired roof
captures with existing CRYSTAL_LOADING_TRACE and sufficient existing capture
budget (e.g. CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=180). Use identical
Roof9,13/noon/2560x1600/camera'0 7 5 0', sequential runs with no owned compiler.
Capture separate `inactive-fades-{control,candidate}-{3d,2d}.png` and trace/logs.
Analyze the classic phase AFTER a completed3D presentation, not initial loading;
compare frame.occluder_fades/main CPU spans and inspect paired state/images.
Spans nest/overlap; trace metadata adds overhead. Do not claim displayed FPS,
startup/door latency or a3D GPU gain from this CPU-specific change.

If useful, verify root equality before selective promotion, root regressions,
required controller/MCP/stdio gates and current visual checks before native
packaging. Installed app remains verified preload release48fc51d3...; the full
fast/fluid objective remains active. This path targets retained-world CPU work
while classic/battle/presentation views own output; it does not solve the
remaining GPU throughput ceiling. No other app/build has been stopped.

Continuation check: exact QA build session82619 was polled repeatedly and remains live; no terminal result or compile errors yet. Root occluder source still matches inactive-fades-baseline.rs byte-for-byte. No private inputs changed, no candidate promoted, no app replaced, and no other processes stopped. Continue by polling82619; observation waits are not failures and must not trigger a restart.

Inactive-world QA build82619 completed successfully in10m23s. Immutable
inactive-fades-frame-client SHA256
6d01951dea6c49a1b710d6624ef49be852a626cddd81090b7e92abbe42d0c90e.
Sequential paired roof captures51854(control),93951(candidate) both terminal0,
180s existing budget, same whole-tree geometry/Roof9,13/noon/2560x1600/camera0,7,5,0.
No owned compiler ran during captures. Both 3D/2D pairs preserve state; inspected
candidate3D/2D and control3D: same roof/forest/tower/player/sky composition and
detail, no visible regression. Pair receipts preserve each run's game state;
wall-clock-driven animation/frame counters differ between runs.

Trace ring131072events each; measured only after the first active3D presentation
then inactive classic view, excluding first500ms. Control201samples: occluder
median8.213125ms/p9512.781833, main26.557750/48.115041. Candidate272samples:
occluder1.520166/2.438125, main18.753792/38.664333. Trace metadata still traverses
groups, so this is an opt-in traced CPU comparison, not displayFPS, normal-play
door latency, or an active3D GPU improvement. Paired artifacts and traces at
.local/hierarchy-cache-production/inactive-fades-{control,candidate}-*;
logs /tmp/geothite-inactive-fades-{control,candidate}-capture.log.

Confirmed root occluder source byte-identical to baseline backup, then promoted
only candidateoccluder_fade.rs. Root focused tests running58390,
/tmp/geothite-inactive-fades-root-tests.log. Installed app remains unchanged;
required gameplay gates, production build and packaging remain before install.

Post-promotion checks terminal0: root58390 occluder60passed/0failed5.16s;
controller14197 eightpassed/oneexistingignored35.03s (Mom/NewBarkgate and battle
PP turn included); MCP68705 onepassed1.57s; stdio43501 onepassed2.51s.
Logs /tmp/geothite-inactive-fades-{root,controller,mcp,stdio}-tests.log.
Production frozen-snapshot release build12212 is confirmed live by exact handle
poll after all tests. Log /tmp/geothite-inactive-fades-production-build.log.
Do not mutate private compiler inputs or restart on observation timeout. On
terminal success record app/pack/save hashes, install with desktop.py --skip-build
from performance-release-native and frozen open-model root, then outside-repo
check-installation/codesign/hash verification. Current installed app remains
preload48fc51d3...; this optimization is promoted in source but not yet installed.
Full fast/fluid objective remains active; active3D GPU ceiling remains unresolved.

Continuation: production12212 remains confirmed live after repeated exact handle
polls; latest process tree shows cargo79724 and active release rustc80194.
No errors/terminal result, private inputs remain frozen. No restart, app install,
or other-process interruption. Prepared before-inactive-fades-install.json:
current app48fc51d3..., unchanged sourcepack and one external save hashes.
Reviewed official Bevy custom instancing example (bevy.org/examples/shaders/custom-shader-instancing/):
same mesh/material handles enable automatic batching; current panorama
MapPublication already reuses handles, so no speculative instancing toggle added.
Shared grove preparation also already removes unused zeroUV0 and welds exact
opaque vertices; spawn-time COMPACT_SOLID_UVS opt-in is not a new discovery or
measured improvement. No code change to these paths. Next action remains poll
production12212 then package/verify using recorded hashes. Goal stays active.

Inactive-world optimization production12212 terminal0 in8m14s. Installer46095
terminal0 in32.4s, /tmp/geothite-inactive-fades-install.log. Installed check and
strict deep codesign97645 terminal0 from/tmp with CRYSTAL_OPEN_MODEL_ROOT unset;
/tmp/geothite-inactive-fades-installed-check.log confirms bundled pack/resources
and existing external save. Hash verification passed: signed installed app
0d74bff4d1885b8973be4d85a14e1fcac8dfe1b10abdb7be346a63bdfb52966a;
source/bundled pack unchanged a0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007;
one existing save unchanged e542a84db0edff578ad84294304fd108583da067472d624dabf85cd1ec6bd9b2.
Receipts before-inactive-fades-install.json and inactive-fades-installed-verification.json
in ignored hierarchy-cache-production workspace. No visible app launched, no
other processes stopped, no pack/generated asset committed.

No owned live build/test/capture remains. This turn finishes deployment of the
verified inactive-world CPU skip; full fast/fluid goal remains active and dense
outdoor GPU throughput remains unresolved. Next work should use installed-source
inactive-fades-frame-client as current whole-tree control. Shader inspection:
shared tree materials already backface-cull, zeroUV compaction and exact opaque
vertex welding already applied. Sky already has exact-zero cloud/star guards.
Potential next investigation is material specialization of the large unified
voxel_surface.wesl for palette foliage to eliminate unrelated source-atlas/water
branches while preserving every executed tree operation; no implementation or
performance claim yet. Reuse Bevy0.20 installed MaterialExtension key mechanism,
check variant prewarming and full visual/fade compatibility before promotion.
Do not repeat rejected sections/meshlets/occlusion trials without new evidence.

Private next GPU trial: uniform-zero guards in voxel_surface.wesl only. Wrap
battle contact loop in a uniform any-contact-W-positive check before its normal
predicate; each actor still uses unchanged W/shape checks and exact calculations.
Wrap cave-exit loop/addition in cave_sky.w!=0; disabled exposure contributes zero,
active exposure retains unchanged calculations. No new material/pipeline key,
shader variant, player toggle, geometry/resolution/detail change. No speed claim.
Baseline saved uniform-guards-baseline.wesl; immutable current control is
inactive-fades-frame-client6d01951d... (accepted installed source). Private QA
release86763 live, /tmp/geothite-uniform-guards-release-build.log; freeze private
inputs and poll exact handle. On success save uniform-guards-frame-client and
run sequential paired visuals and untraced fullresolution roof walking versus
current control; discard trial if improvement does not repeat. Also check
active battle contacts and cave mouth lighting; disabled-path roof alone cannot
prove enabled effects. Root WESL differs from frozen baseline at buoy_facing
(line1443); preserve that unrelated root change via selective guard edits only
if eventually accepted. Root/app currently retain unguarded shader. No owned
other process was stopped or quality changed.

Uniform-guards continuation: discovered existing native CRYSTAL_VOXEL_SHADER_REVIEW
loader in lib.rs (no tooling change needed). Current accepted immutable client
inactive-fades-frame-client loaded private candidate WESL and completed offscreen
paired roof50567 terminal0:3D46.25s/2D55.18s,180s existing budget. Compilation was
concurrent so capture duration is NOT performance evidence. Shader compilation
succeeded; inspected both images, preserved roof/forest/sky/player and classic
composition. uniform-guards-roof-3d.paired-state.txt state_preserved=true,
Roof9,13/noon/camera0,7,5,0/player[-1312,1056]facingup; no visual acceptance beyond
this disabled-effects scene. Logs /tmp/geothite-uniform-guards-roof-capture.log.
Private release86763 still confirmed live by exact poll. Inputs remain frozen.
Next: poll86763, save immutablecandidate when terminal0; sequential untraced
roof measures versus inactive-fades-frame-client only when owned compilers stop.
Use existing --measure outputdir --seconds60 --walk UUUUDDDD, headlessimage,
CRYSTAL_MEASURE_3D_VALIDATION=on, syntheticwalkinput/steps1, sameRoof9,13/noon/
2560x1600/camera0,7,5,0. Existing output includes summary.txt/render-timing.csv/
walk.movement.csv/gpu-completions.txt. Repeat in reverse order if useful.
Enabled cave sky and battle contact comparisons remain required before promoting.
No root shader/app change; installed0d74bff4... remains verified CPU optimization.

Uniform-guards enabled-scene check: candidate UnionCave1F explicit17,4 run12341
terminal1 because runtime rejects that tile as nonwalkable. Corrected by using
existing validated default walkable preview, not bypassing collision. Candidate
3680 terminal0 (2.36s3D/2.64spair), control39684 terminal0 (2.33spair),1280x800/noon,
same immutable inactive-fades-frame-client and candidate shader review only.
Candidate and control paired receipts state_preserved=true; inspected candidate
3D/2D and control3D: matching violet cave, tiered rock/water/ladder/player and NPC
composition with no visible change. Pair at default center[-1280,768], player
[-1248,800], camera0/7.5/45. This interior is away from a doorway and is not proof
of a nonzero exit-fill pixel; actual cave-mouth and battle-contact enabled
checks remain pending if performance justifies promotion. Captures under
uniform-guards-cave-{candidate,control}-*, logs /tmp/geothite-uniform-guards-cave-*.log.
Private release86763 remains confirmed live; no shader/root/app promotion and
no other process interrupted. Wait for exact terminal before clean performance
comparison. This turn adds shader/visual evidence, not a measured speed gain.

Uniform-zero shader guards REJECTED after reverse-order walking comparison.
QA86763 terminal0 in7m29s; immutable uniform-guards-frame-client SHA256
52b79fda275f01f34f4a6f4f9eb136678f968df9778bd41ddd2720576c669997.
Four sequential untraced fullresolution Roof9,13/noon/camera0,7,5,0/60s measures,
UUUUDDDD synthetic step1; no owned compiler concurrent. All256movementrows,
4origins,0extra terrain builds; no GPU poll errors. Sessions97127candidate-first,
9158control-first,68830control-second,41060candidate-second allterminal0.
Candidatefirst890updates median66.61/p9585.76ms; queue891/60.130916s=14.818/s,
outstanding36to36. Controlfirst850updates71.22/79.40ms;845/60.099766=14.060/s,
outstanding31to36. Reverse control912updates60.05/118.93ms;
907/60.108680=15.089/s,outstanding3to8. Reverse candidate880updates67.65/74.53ms;
881/60.095609=14.660/s,outstanding36to36. First throughput gain5.4% does not
repeat (reversecandidate2.8%lower), and median/movement results mixed. These
queue callbacks are not displayFPS/GPUtimestamps; background machine variation
remains, no processes were stopped. No evidence of repeatable useful improvement.

Artifacts at .local/hierarchy-cache-production/uniform-guards-{candidate,control}-{first,second},
logs /tmp/geothite-uniform-guards-{candidate,control}-{first,second}.log.
Archived trialshader uniform-guards-rejected.wesl, restored private source from
uniform-guards-baseline.wesl. Root/app never received guards. No further enabled
battle/cave-mouth acceptance needed for rejected code. No owned live processes
remain. Installed verified0d74bff4... CPU/preload build remains current. Full
loading/fluidity goal active; dense outdoor GPU ceiling unresolved. Do not
rebuild/retry these guards without new evidence; next work must target a more
substantial cost, preserving geometry/detail/quality and shared root edits.

New private active-world CPU trial: skip redundant visibility-cache population
scan when matched occluder query membership is unchanged. Existing everyframe
cached-ancestor change checks and all group ray/fade processing remain. Add
visible_group_count to existing FadeFrameHistory; QueryIter exact len uses
archetype lengths (no dynamic filter). Populate only when cacheempty or count
differs. Same-count substitutions also remove a required query component;
HierarchyRemovals additionally consumes Children and GlobalTransform removals,
invalidating when a cached entity is affected. Existing OccluderGroup/Visibility/
ChildOf removal and hierarchy change invalidations retained. No shader/material/
geometry/player toggle/quality/dependency change. Baseline membership-cache-baseline.rs;
accepted candidate membership-cache-candidate.rs in ignored private workspace.

Extended existing hidden hierarchy regression: remove old group's Children and
spawn a replacement under a new hidden root at the same matched group count;
replacement must suspend and stay unblocked. All60 focused occluder tests42385
terminal0 in2.23s, /tmp/geothite-membership-cache-tests.log. Mutation check75708
removed Children-event chain only and fails expected assertion at2155 (new
replacement incorrectly visible), /tmp/geothite-membership-cache-negative-test.log.
Restored byteexact tested candidate, then started frozen release QA60203 live,
/tmp/geothite-membership-cache-release-build.log. Inputs now frozen; poll exact
handle, do not restart on observation timeout. No root promotion/app change.

On terminal0 save immutable membership-cache-frame-client. Current matching
control inactive-fades-frame-client6d01951d... (installed0d74bff4 source). Use
same fullresolution untraced walking comparison in both orders and existing
paired roof visual gate. This CPU change saves one whole cache-population pass
but leaves main pergroup processing and GPU cost, so no predicted speedup or full
fluidity claim. If useful, promote selective code after rechecking root source,
required focused/controller/MCP/stdio tests and visual verification, then
production build/install/check/hashes. Full fast/fluid/loading objective active.

Membership-cache QA60203 terminal0 in5m20s. Immutable
membership-cache-frame-client SHA256
565464e51256d4b51baeef230cc52c3d648b456c3243268321db71acd026c662.
First untraced walking pair terminal0: candidate72340 1037updates median56.11/
p9572.08ms, movement55.79/63.67ms;1038queuecallbacks/60.052229s=17.285/s,
outstanding36to36/no pollerrors. Control21039 846updates70.27/80.18ms,
movement69.78/75.10ms. Both256movementrows/4origins/0extraterrainbuilds, same
Roof9,13/noon/2560x1600/camera0,7,5,0/UUUUDDDD/syntheticstep1. No owned compiler
concurrent. These rates are queue callbacks, not displayFPS or GPUtimestamps.
Promising first pair does not establish repeatability.

Reverse control47920 is live now, /tmp/geothite-membership-cache-control-second.log,
output .local/hierarchy-cache-production/membership-cache-control-second.
Poll exact handle; on terminal run membership-cache-frame-client with identical
env/args to membership-cache-candidate-second, then analyze both orders.
No code edits/private compilers currently live; root/app unchanged. Still need
traced active CPU/pairedvisual comparison and regression/promotion/install gates
if repeatable gain or direct useful CPU savings established. Full goal active.

Membership cache trial REJECTED. Reverse control47920 terminal0:841updates,
median68.61/p9583.98ms, movement66.49/82.93;842callbacks/60.102897s=14.009/s,
outstanding36to36. Reverse candidate41616 terminal0:754updates79.60/107.62ms,
movement74.69/104.27;739callbacks/60.118496s=12.292/s,outstanding20to36.
Bothsame256movementrows/4origins/0builds/no pollerrors. First apparent gain did
not repeat. Sequential paired trace captures candidate96228/control77235 both
terminal0; images inspected, pairedstate preserved, same complete roof/forest/
player/sky and classicview. Active3D trace window fromfirstactive+500ms through
lastactive: candidate188samples occludermedian5.02575/p9510.008833ms,
main30.903542/148.455584. Control205samples occluder4.376208/9.596625,
main19.160625/44.133125. No targeted CPU savings established either. Trace spans
are nested and opt-inmetadata adds overhead; no FPS/normaldoorlatencyclaim.
No further repeats justified for this trial. Archived membership-cache-rejected.rs,
restored private occluder from membership-cache-baseline.rs. Root/app never
received trial. No owned live process remains. Installed0d74bff4... accepted
inactiveCPU/preload build unchanged. Full goal remains active/incomplete.
Potential underlying invalidation frequency would require actual new evidence;
do not assume counting groups bypasses cache population in real scenes or
repeat this trial blindly. Preserve fidelity and other concurrent root edits.

New private coarse-region occluder trial (not the rejected membership cache).
Research rejected speculative outer-first triangle order before implementation:
Apple's Harness Apple GPUs with Metal WWDC2020/10602 documents opaque HSR that
already avoids overdraw independent of triangle order. Bevy0.20 bins expose
immutable entity lists; no renderer fork or ordering patch added. Existing
NoCpuCulling trials already recorded above are not a newly found solution.

Instead implemented exact conservative map-region gating for panorama groups.
New private occluder_fade/regions.rs; occluder_fade.rs adds spawn_region_groups
and selective original group processing; summit_vista.rs uses it only for each
owned MapPublication placement root. Every source collision surface, mesh,
material, draw/leaf, opacity update and pergroup ray test retained. Region stores
complete root-local union, transformed through current root GlobalTransform
with conservative padding. Nonfinite/unknown bounds and camera-wall policy
retain exact processing. Disjoint clear regions avoid per-member ray/inverse/
visibility-cache scans; ordinary groups remain on original path. Regions with
unsettled fades keep processing until clear; waking refreshes inverse/hierarchy.
Member local motion grows union, reparenting/parent removal invalidates proof
and falls back, removed region owners route live members to ordinary processing,
selection dedup prevents duplicate fade advance after structural edits. No
shader, quality, geometry, resolution, density, draw-distance, worker ABI,
dependency or player toggle change. Source collision/sorted/fade leaves remain.

Baselines region-gate-baseline.rs and region-gate-summit-baseline.rs saved under
.local/hierarchy-cache-production. First compile89174 failed QueryManyIter API
(Result items); corrected via fetch_next and let Ok elsecontinue. Fixed50845
passes61occluder tests7.21s. Structural fallback refinement60981 passes61/0failed/
0ignored2.23s, /tmp/geothite-region-gate-structural-tests.log. New integration
regression proves disjoint region leaves member inverse unbuilt, root movement
into rays blocks/fades, departing fades settle before sleep, memberlocalmotion
wakes while rootstatic, reparented transform follows real obstruction, and
removedRegion safely falls back. Summit focused suite passes11/0failed0.11s,
/tmp/geothite-region-gate-summit-tests.log. Formatted only new module afterward.

Private release QA49976 is live, /tmp/geothite-region-gate-release-build.log.
Freeze private inputs and poll exact handle; do not restart on observationwait.
On terminal0 save immutable region-gate-frame-client. Matching current control
inactive-fades-frame-client6d01951d... (installed0d74bff4... source). Run same
fullresolution untraced roof walking before/after and reverse if useful; paired
3D/2D captures with existing loadingtrace should establish actual active CPU
cost and unchanged visuals. No performance claim yet; GPU cost remains and
full fluidity objective active. Root/app untouched. Need additional regional
visibility/nonfinite/rotation/overriddenVisible checks before promotion if
measurements justify it, plus root regression/controller/MCP/stdio and packaging.

Additional installed-source house-exit evidence with NORMAL asynchronous shader
compilation: immutable inactive-fades-frame-client used --record (neither
--measure nor --screenshot, so bevy_shell synchronous_pipeline_compilation
QA override is not selected). Headlessimage/full2560x1600/noon/PlayersHouse1F6,6,
10seconds, syntheticstep1/DDBBBBBBBBBB; trace existing CRYSTAL_LOADING_TRACE.
Session68540 terminal0, /tmp/geothite-async-house-exit.log. No tool changes.
Functional capture overlapped private region compiler, so frame timings and
capture overhead are NOT normal-play latency/FPS evidence. 278updates/144PNG
captures. render-timing33House/245Town rows; firstTown atrecord0.847548s already
world3dactive=true, entryfxfalse. 247Town presentationtraceevents, ZERO terrain
pending, firstattime8.89912825s alreadyready/active3D. Fulltrace14250events.
Walk256rows:34House/222Town, actualcontrollerdoorwarp; firstTownpendingfalse,
terrainactive=true, normalwarpFadeOut63. Inspected frame17arrival,22midfade,
35settled: player/house/trees/water/NPCs present; existing house occluder fades
then restores, no incomplete/uncompiled body. This is stronger functional
preload evidence than previous forced-synchronous captures, not a controlled
before/after speedup. Artifacts async-house-exit/ and async-house-exit.trace.json.
Removed140taskPNGscratchframes after inspection; retained0/17/22/35 plus
originaltimestamps/render/walk receipts and retained-frames.txt. No sourcepack
or save changed. Private region QA49976 still live by exactpoll; no restart.

Region QA49976 completed terminal0 in5m01s. Saved immutable region-gate-frame-client
SHA256 d555e6364a71d2199da2af5070f427e562a16b479cb4c0438616b36029b25abf.
Untraced firstcandidate96888 terminal0:1014updates57.57median/68.50p95ms,
movement56.50/63.89ms,256rows/4origins/0builds. Matching baseline firstcontrol
nowlive (see current toolhandle), /tmp/geothite-region-gate-control-first.log,
output region-gate-control-first. Current tests61occluder+11summit green; source
stillprivate, no app/root promotion. Next poll control, analyze then reversepair
and traced pairedvisual toprove actual CPU scanavoidance. No owned compiler
concurrent with measurements; no other process stopped. No FPSgainclaimyet.

Region whole-frame comparison completed, NOT a stable FPS proof: controlfirst
30943 terminal0 996updates median58.42/p9571.31ms movement56.41/62.63;
997queuecallbacks/60.107239s16.587/s,outstanding36to36. Candidatefirst96888
1014updates57.57/68.50,movement56.50/63.89;1015/60.097160s16.889/s,36to36.
Reverse control5936 terminal0 685updates72.74/172.29,movement66.38/101.16;
721/60.102947s11.996/s,outstanding36to1. Candidate63848 terminal0 793updates
74.68/121.26,movement73.14/108.44;795/60.080930s13.232/s,outstanding2to2.
Allsame256movementrows/4origins/0builds/no pollerrors. Queue callbacks and
varying queue depths are not displayFPS; global results mixed/no consistent
frame-time gain. No owned compilers concurrent, other processes left alone.

Direct CPU/visual firstpair: candidate3622/control27832 terminal0. Pairedstate
preserved and candidate3D/2D/control3D inspected: same complete roof/forest/
player/sky, no visible defect. Active3D eventwindow firstactive+500ms to last:
candidate202occludersamples median1.9863955/p954.054791ms, main201samples
17.584458/32.788084. Control200samples occluder4.903354/9.063542,
main19.5854375/36.465167. Eachring131072events. This establishes promising
specific occluder CPU reduction, not GPU/FPS/door latency; trace metadata still
walks groups and CPU envelopes overlap. Need reverse CPU check before accept.

Reverse traced control25900 now live, log
/tmp/geothite-region-gate-control-second-capture.log, outputs
region-gate-control-second-{3d,2d}.png and region-gate-control-second.trace.json.
Poll exact handle, then run region-gate-frame-client identical command with
candidate-second prefixes and existing CRYSTAL_LOADING_TRACE, 180sbudget,
Roof9,13/noon/full2560x1600/camera0,7,5,0/headlessimage. Analyze activeCPU same
window, then finish regional visibility/nonfinite/rotation tests before selective
promotion/root gates/install if justified. All source remainsprivate, app
unchanged0d74bff4...; fullgoalactive/incomplete. No other owned live process.

Reversecontrol25900 terminal0; reversecandidate capture18534 nowlive, /tmp/geothite-region-gate-candidate-second-capture.log. Poll18534 then compare both reverse traces; no compiler/source edit during measurements.

Region reverse CPU check PASSED: control25900/candidate18534 terminal0, each
131072events and222active3Dpresentations. Same activewindow rule yields193control
occludersamples3.449334median/6.616417p95ms; candidate193samples1.297042/2.066375.
Maincontrol193samples11.040667/21.293125; candidate192samples9.082104/11.874541.
Bothpairedstatepreserved; inspected reversecandidate3D/2D, samefullscene.
Combined with firstCPU4.903354to1.9863955, this repeats targeted ~2–3ms CPU
savings. Accept as CPU work reduction, not stable overall FPS gain or GPU fix.

Added tests only (production logic unchanged): regional bounds enclose rotated,
negative/nonuniform/singular scales at largecoordinates; nonfinite/unknown
proofs retain exact path; hidden regional fades freeze and Visible overrides
wake correctly; parent removal invalidates oldproof; removed membership cannot
doubleadvancefade. Extracted regionalfixture helper. Private58290 passes64tests
in2.25s, /tmp/geothite-region-gate-edge-tests.log. Reconfirmed root two source
files byteidentical to baselines and new moduleabsent, then promoted only
occluder_fade.rs, summit_vista.rs, occluder_fade/regions.rs. No unrelated root
changes overwritten. Root28893 passes64/0failed3.13s; root summit11/0failed0.17s.
Controller26753 eightpassed/oneexistingignored13.59s, Mom/NewBarkgate and battle
PP mutation included. MCP46852 onepassed1.44s; stdio13561 onepassed1.83s. Logs
/tmp/geothite-region-gate-{root,root-summit,controller,mcp,stdio}-tests.log.
Scoped gitdiffcheckpassed. Root/source change now accepted, app not yet updated.

Frozen private production release32373 confirmed live after tests;
/tmp/geothite-region-gate-production-build.log. Freeze private inputs/poll exact
handle, no restart on observationwait. Pre-installhashreceipt
before-region-gate-install.json confirms currentinstalledapp still0d74bff4...;
sourcepack and one save recorded. On terminal0 install desktop.py --skip-build
from performance-release-native/frozenopenmodels, then outside-repo unsetmodel
root check-installation, strictdeepcodesign and before/after pack/savehashes.
Do not launch visiblewindows or stopotherprocesses. No other owned live handle.
Goal remains active/incomplete: CPU reduction verified, GPU/fluidity unresolved.

Region CPU optimization deployed: production32373 terminal0 in9m18s,
/tmp/geothite-region-gate-production-build.log. Installer55691 terminal0 in32.6s,
/tmp/geothite-region-gate-install.log. Outside-repo check8369 terminal0 with
CRYSTAL_OPEN_MODEL_ROOT unset and independent strictdeepcodesign terminal0.
Installed signed executable SHA256
3734307d19a3bc1054e41799a8e78ddb05aa19c06c9fb6bee1cc27cc2f53208d.
Source and bundled nativepack unchangeda0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007;
one external save unchangede542a84db0edff578ad84294304fd108583da067472d624dabf85cd1ec6bd9b2.
Receipt region-gate-installed-verification.json and check log
/tmp/geothite-region-gate-installed-check.log. No visiblelaunch, no other
process stopped, no generatedpack/buildasset staged/committed. No owned live
handle remains. Installed and root include conservative regionalCPU gate plus
accepted previous inactiveCPU/destinationpreload work. Immutable current QA
control is region-gate-frame-clientd555e636..., runtime code matches production;
later additions were testsonly. Full goal remainsactive/incomplete: repeated
2–3msoccluderCPUsavings verified, overallFPSresults mixed, dense GPUwork remains.
Next investigation should target substantive geometry/shadow work using current
source; no need to repeat completed region tests or rejected shader/cache/section/
meshlet/NoCpuCull trials without new evidence. Existing earlier layer32 fixture
proves reduced shadow candidate membership but combined old NoCpuCull result
was mixed; if revisiting, isolate layer change from unrelatedculling, preserve
both world visibility and shadow receiver bounds, and require useful current
source evidence. Keep all concurrent art/source changes intact.

New private opaque-only GPU visibility trial. Source audit ruled out layer32
alone: RenderGpuCulledEntities contains only NoCpuCulling entities, absent in
currentclient. Old mixed NoCpuCull trial admitted masked/transparent surfaces;
new trial explicitly limits admission to merged opaque leaves of regional
panorama trees. No rerun of the old unrestricted policy. Apple opaque HSR
research also ruled out speculative outer-first index ordering before changes.

Private opaque_gpu_visibility.rs automatically requires actual GPU instance
building plus frustum-culling support and compatible active perspective views.
No enable toggle/trace tool added. Initial regional opaque leaves alone get
PanoramaOpaque role/layer32; normal and future sorted/fading leaves remain31/
CPU culling. Worldcamera sees31+32, sun31 excludes GPU-only tree candidates;
shadow receiver/bounds/fitting proofs retain both layers because VoxelSurface
samples shadows explicitly. Geometry/material/shader/raster resolution/density/
drawdistance/pipelinekeys unchanged. GPU-only visibility follows inherited
hidden state incrementally. MaterialProof observes asset events and alpha-mode
class changes, avoiding an owner scan for ordinary uniform changes; changing
to nonopaque restores CPU mode. Registration after AssetEventSystems and before
VisibilityPropagate ensures same-frame fallback/reset. Unknown bounds, skin/
morph/NoFrustum, foreign layers, removed proof components, unsupported device/
projection/directdraw restore original CPU path. Inactive cached world retains
ownership, avoiding mass migration on classic/hide. Nativeonly admission;
browser tree leaves stay31. No Cargo/dependency/workerwire changes.

Backups of five modified existing files at opaque-gpu-visibility-baseline/:
lib.rs,occluder_fade.rs,shadow_bounds.rs,shadow_fit.rs,shadow_receivers.rs.
Newmoduleprivate only. First test77974 compile101 missing HashSet hasher type;
fixed explicit AssetId<VoxelMaterial> set. Script assertion initially prevented
test insertion (no source partial test write); corrected exact registration text.
Fixed24010 3pass/1fixturefail (missing ViewVisibility), added actual Visibility/
ViewVisibility fixture,98791 fourpass. Actual GPU collector test proves world
129meshes/capture0/sun129oldlayer versus1newlayer. Admission rejects Mask/Blend/
AlphaToCoverage, respects hidden/unhide, restores CPU on material/device/NoIndirect/
orthographic changes and preserves inactive ownership. Expanded existing lazy
fade regression ordinary+regional: opaque role/layer32 only, future sorted
leaves layer31/unmarked; exact pose/vertices/indices/material reuse/retention
stilltested. Existing shadow receiver proof now exercises layers31and32.

Final suite74178 terminal0: GPU4/occluder64/summit11/receiver10/shadowfit5/
shadowbounds7 =101passed,0failed. Logs
/tmp/geothite-opaque-gpu-final-{module}-tests.log. Source/root/app untouched by
trial; currentinstalled3734307d... regionCPU/preload build remainsverified.
Private release QA started /tmp/geothite-opaque-gpu-visibility-release-build.log;
see current exact handle and freeze inputs until terminal. Then save immutable
opaque-gpu-visibility-frame-client. Current matching control region-gate-frame-client
d555e636... . Compare current fullresolution roof walking bothorders, existing
traced CPUvisibility/pairedvisuals including firstfade, plus roofentry from upper
floor before promotion: Bevy collect_render_mesh_instance queues removal when
ViewVisibility false, so tagging hidden meshes alone does not prove GPU table
upload is prepaid at unhide. No FPS/latency benefit claimed. Full goalactive.

Opaque GPU visibility QA55346 terminal0 in5m17s. Immutable
opaque-gpu-visibility-frame-client SHA256
9e53dca0f50a0baf144eb5257b342023721086672b25a4f41deb50d4fb3d90cd.
First untraced pair candidate60812/control69446 terminal0, full2560x1600/
Roof9,13/noon/camera0,7,5,0/UUUUDDDD/syntheticstep1/60s. Candidate858updates
median68.69/p9581.08ms,movement68.11/79.47; control836updates70.62/84.52,
movement66.91/79.36. Both256rows/4origins/0extraterrainbuilds. Small/mixed first
pair is not a gain claim. Reversecontrol32754 now live, log
/tmp/geothite-opaque-gpu-control-second.log, output opaque-gpu-control-second.
Poll exacthandle; then identical candidate-second run. No owned compilers
concurrent, other processes untouched. Need traced CPUvisibility and paired
visuals if useful, plus actual roofentry/unhide gate before promotion.

Read-only localpack route probe91326 terminal0 using existing runtime/assets
rlib reexports; task scratch roof_route_probe.rs/executable deleted after success.
TinTower9F warp4 raw/runtime(7,9) ->Roof warp1(9,13); coordinates map directly
(core raw_event_tile_to_runtime_tile_checked no rescale). Actual natural walking
is camera-relative; one Down from(7,8) needs cardinal yaw0, not benchmarkyaw7.
For warm-cache roof entry proof use startingRoof(9,12), camera0,0,45,0 and DUD:
Down to9F, Up away from arrival stair, Down back toRoof. This must be validated
through real controller (route not yet executed). InitialRoofload prewarms full
panorama; related9F retains source geometry, testing hidden/unhide GPU instance
upload instead of conflating cold source generation. A prior default9Fstart
could reachroof beforepreloadready, so don't call it a warmed entry test without
source readiness evidence. Use existing --record for normal asynchronous shader
path, bound capture length and remove task PNG scratch after retaining transition
evidence. No new timing/input tooling or game dispatcher added. Current root/
installed3734307d... remain unchanged by this private trial; fullgoalactive.


Opaque-only GPU visibility trial disposition (2026-10-21): not installed.
The reverse walking pair completed: control 75.99/141.82 ms median/p95,
candidate 69.58/80.37 ms; completion callbacks 12.970 versus 14.200/s.
The first pair was 13.900 versus 14.211 callbacks/s. These are queue callbacks,
not displayed FPS. Baseline variability limits the inference from both pairs.

Paired stationary captures preserved state and the roof/forest composition.
Active-scene CPU medians were control/candidate: check_visibility 0.568/0.425 ms,
visibility_propagation 1.617/2.294 ms, main 8.755/8.769 ms. This does not establish
an aggregate CPU improvement. Installed Bevy check_visibility_cpu_culling scans
eligible entities on every active view; it does not skip the scan merely because
the camera is stationary. No moving-camera benefit should be inferred from an
assumed stationary-camera cache.

Normal asynchronous roof round trips completed through actual controller input:
start TinTowerRoof (9,12), camera 0 0 45 0, synthetic step 1, DUD followed by B.
Both reached TinTower9F (7,9), stepped away, and returned to Roof (9,13).
Both arrivals had terrain_pending=false and terrain_active=true. Candidate
session 32896 and control 93392 exited successfully. Logs are
/tmp/geothite-opaque-gpu[-control]-roof-roundtrip.log. Candidate recorded only
14 frames versus control 108, so these readback-heavy runs cannot establish a
comparative performance gain. Candidate returned-roof image was inspected;
geometry and scenery remained present. First/final images and CSV/trace receipts
remain ignored; intermediate task PNGs were removed.

With no clear aggregate CPU saving and unresolved capture behavior, leave this
trial unpromoted. Its six source files are archived under ignored
.local/hierarchy-cache-production/opaque-gpu-visibility-unpromoted-source/.
Restored all five original private source files from the recorded baseline and
removed the trial module. No root renderer code, installed app, pack, save,
other application, or other process was changed by this trial. The installed
3734307d... build still contains the verified destination preloading and region
fade CPU improvements. The broader fluid-gameplay goal remains active.


Native opaque tree vertex-fetch trial (2026-10-21), private and uninstalled.
The existing DFS triangle localization improves post-transform reuse but leaves
vertex attributes in the pre-localization buffer order. The new Rust pass
permutes opaque template attributes into first-index-reference order after
triangle localization, before shared template publication. This follows the
vertex-fetch stage described in https://github.com/zeux/meshoptimizer#vertex-fetch-optimization ;
no library, exporter, dependency, shader, material key, geometry approximation,
resolution change, per-frame scan, or player toggle was added.

Only unuploaded, unshared opaque leaves are eligible. The pass preserves all
ordered triangle attribute bits, winding, index width, vertex count (including
unused vertices), packed normal format, independent sorted fade buffers and
collision ownership. Unsupported formats and malformed indices return without
mutation. Browser and worker wire paths remain unchanged. Source backups are
ignored vertex-fetch-baseline/{occluder_fade,shared_groves}.rs; the two private
source hashes are in vertex-fetch-source-receipt.json.

Initial test build 38467 exited 101 because a test inspected Bevy's private
attribute_compression field; replaced that assertion with public attribute format
checks. Retry 27797 passed both new tests, including three real tree families,
packed normals, signed zero/NaN payload bits, unused vertices, U16/U32 indices,
malformed-index fallback and idempotence. Full occluder suite 35402 passed 66;
shared_groves suite 83756 passed 3. No failures. Logs:
/tmp/geothite-vertex-fetch-{tests,occluder-tests,shared-tests}.log.

Release QA build 37097 is running with the frozen private source and release
native target. Poll that exact handle; do not restart because an observation
expires. Log /tmp/geothite-vertex-fetch-release-build.log. Once terminal, save
an immutable candidate and compare full-resolution walking in both orders against
region-gate-frame-client d555e636... with no owned compiler running. Paired
visuals and normal asynchronous cached-entry behavior remain required before
promotion. No frame-time or gameplay improvement claimed yet. Installed app,
root renderer code, content packs, saves and other processes are unchanged.


Vertex-fetch QA release 37097 completed successfully in 6m49s. Frozen source
hashes still match the recorded receipt. Saved immutable vertex-fetch-frame-client,
SHA256 cba0df36ffb0ea55009ef47bc3aba791626bcd7532fbbd9ea7af1d3dbab1a256.
Started candidate-first full-resolution roof walking session 34145, log
/tmp/geothite-vertex-fetch-candidate-first.log, output directory
.local/hierarchy-cache-production/vertex-fetch-candidate-first. Same established
Roof (9,13), noon, camera 0 7 5 0, 2560x1600, synthetic step 1, UUUUDDDD, 60s.
No owned compiler runs concurrently. Poll 34145 until terminal, then run the
identical region-gate-frame-client control before the reverse-order pair.
No performance result or installation claimed while the measurement is live.


Vertex-fetch first untraced pair completed: candidate 34145, control 29074,
both exited 0. Candidate 857 updates, median/p95 69.66/86.08 ms; control 816,
71.57/88.85 ms. Walking 256 rows/four origins/no extra builds in each;
median/p95 candidate 68.46/81.11 ms versus control 72.73/90.51 ms.
Completion callbacks candidate 13.956/s (outstanding 17 to 36) versus control
13.592/s (36 to 36), no poll errors. These are queue callbacks, not displayed
FPS. Candidate warm-up image inspected: roof, player and dense forest present.
This modest first-pair difference needs repetition before a benefit claim.

Reverse-order control session 1631 is now running, output vertex-fetch-control-second,
log /tmp/geothite-vertex-fetch-control-second.log. Poll the same handle to terminal,
then run candidate-second with identical inputs. No source changes, owned compiler,
new installed build, quality reduction, or other-process termination.


Vertex-fetch reverse pair completed: control 1631 and candidate 56743 exited 0.
Control 814 updates, median/p95 71.56/92.54 ms; candidate 1019 updates,
53.68/104.06 ms. Walking median/p95 control 72.71/100.26 versus candidate
51.18/99.46 ms, both 256 rows/four origins/no extra builds. Completion callbacks
13.552/s control and 16.554/s candidate; both no poll errors. Candidate outstanding
11 to 36, control 36 to 36. These are callbacks, not display FPS.

Both orders favor candidate medians, but the candidate varies substantially
between runs and whole-run p95 worsens in the reverse pair. Do not claim a stable
fluidity/tail improvement or the larger reverse gain as a general result.
Further comparisons are justified by this unresolved variation, after visual
and normal asynchronous cached-entry checks. Current installed executable still
matches accepted 3734307d...; no installation or root renderer source promotion.

Candidate paired 3D/2D roof capture session 38233 is live, existing 180-second
capture deadline, no trace. Log /tmp/geothite-vertex-fetch-paired-capture.log;
outputs vertex-fetch-candidate-{3d,2d}.png under ignored hierarchy-cache-production.
Poll that exact handle, inspect both images and state-preservation receipt, then
run normal asynchronous roof round trip with the existing DUD route. No compiler
or other owned render runs concurrently, and no other process was stopped.


Vertex-fetch visual and cached-entry gates completed. Paired capture 38233
exited 0; candidate 3D/2D images inspected at the same TinTowerRoof position,
noon, facing and game state. Paired receipt state_preserved=true, frame 1044,
checksum 388821513. Roof surface, source color, player, tree silhouettes,
distant scenery and sky remain intact; no new visual defect observed.

Normal asynchronous cached roof round trip 95555 exited 0. Actual controller
DUD route reached 9F (7,9) at walking row 9 and returned Roof (9,13) at row 74.
Both arrivals terrain_pending=false, terrain_active=true, one source build;
presentation trace also records both arrivals ready with 3D active. Final
returned-roof frame 00113 inspected, geometry/scenery intact. First/final PNGs,
CSV and trace retained ignored; 112 intermediate task PNGs removed. Recording
readback timings are not comparative performance evidence.

Because the two short walking comparisons favor candidate medians but disagree
on slow-frame behavior, started a longer 120-second control 87637 using identical
full-resolution roof settings. Output vertex-fetch-control-long; log
/tmp/geothite-vertex-fetch-control-long.log. Poll that exact handle until terminal,
then run the immutable vertex-fetch-frame-client candidate-long for 120 seconds
with otherwise identical inputs. This extension addresses unresolved variance;
it is not a general FPS or fluidity claim. Source still frozen; no owned compiler,
root renderer promotion, installed app change, or other process termination.


Long control 87637 exited 0: 1512 updates over 120.08s, median/p95
77.93/114.54 ms. Walking 256 rows/four origins/no extra builds,
51.56/170.03 ms; completion callbacks 12.359/s, outstanding 8 to 36,
no poll errors. Its walking median also drops near the earlier candidate's
fast run, confirming substantial within/between-run variation in this environment.
Do not attribute the earlier larger candidate gain solely to vertex layout.

Matched candidate-long session 8576 is running for 120 seconds, identical
settings, output vertex-fetch-candidate-long, log
/tmp/geothite-vertex-fetch-candidate-long.log. Poll the exact handle until terminal.
No owned compiler or second renderer runs concurrently. No source promotion
or installation. A read-only tool lookup confirms xctrace is installed at
/Applications/Xcode.app/Contents/Developer/usr/bin/xctrace; no profiler was started
and no measurement tooling code was changed. The full fluid-gameplay objective
remains unresolved.


Vertex-fetch long pair completed: candidate 8576 exited 0, 1577 updates over
120.04s, median/p95 72.88/108.72 ms versus control 77.93/114.54 ms. Walking
candidate 70.53/99.64 ms versus control 51.56/170.03 ms: movement median is
mixed and must not be represented as a universal gain. Candidate callbacks
13.241/s, outstanding 18 to 6 (queue drain contributes), no poll errors;
control 12.359/s, outstanding 8 to 36. Callback rates are not displayed FPS.

Disposition: keep the exact vertex-fetch permutation. All three whole-run
comparisons favor candidate median (modest first-pair gain, larger variable
second, about 6.5% long-pair median reduction). Two favor p95, one worsens;
no stable global tail/FPS/fluidity claim. Unit bit proofs and inspected paired
visual/cached-entry gates passed. The full performance goal remains incomplete.

Both root target files matched the saved pre-trial baselines byte-for-byte,
so promoted only the verified occluder_fade.rs/shared_groves.rs files. Scoped
diff check clean, unrelated root changes preserved. Root occluder test session
70932 is live, /tmp/geothite-vertex-fetch-root-occluder-tests.log. Production
build also started against frozen private source, release native target,
/tmp/geothite-vertex-fetch-production-build.log; use its exact returned handle.
Benchmarks are finished; no rendering overlaps these compiler jobs. Before
installing, finish root relevant/shared tests and controller/MCP/stdio gates,
record current app/pack/save hashes, verify current installed build has not
changed, then use existing installer and outside-repo installation verification.
No install has occurred yet.


Root occluder 70932 exited 0, 66 passed. Root shared-template compile 44151
exited 101 after concurrent unrelated mountain source edits: outdoor_mountains.rs
references missing mountain_flanks::edge_fraction, RELIEF_HEIGHT_SCALE and
FILL_EDGE_REACH (seven E0425 errors). Full diagnostics remain in
/tmp/geothite-vertex-fetch-root-shared-tests.log. Do not overwrite or repair
another ongoing art change for this performance task. No regression assertion
ran in that failed compile; do not count it as a passing root suite.

Both promoted files still match the recorded, tested private source hashes.
The frozen private source passed shared_groves 3 and occluder 66 already and
remains the measured production source. Controller regressions now run against
that snapshot, log /tmp/geothite-vertex-fetch-controller-tests.log; use returned
exact handle. Production build 71614 remains active, unchanged frozen inputs.
Run MCP and stdio against the same snapshot after controller completion. Root
buildability is currently limited by the concurrent mountain edits; report that
limitation separately from the production snapshot's verification.


Vertex-fetch production-snapshot gameplay gates passed with the external TUI
pack: controller 76809 exited 0, 8 passed/1 existing ignored (standard mart),
including actual battle turn and Mom/New Bark gate. MCP 2705 exited 0, real
movement and production Start menu test passed. Stdio 56415 exited 0,
normal_mcp_gameplay_keeps_stderr_clean passed. Logs
/tmp/geothite-vertex-fetch-{controller,mcp,stdio}-tests.log. These validate the
frozen production source, not the concurrently changing root mountain edits.

Recorded before-vertex-fetch-install.json with app, source/bundled pack and
all current save hashes. Installed executable still accepted 3734307d...;
source/bundled packs match, one save recorded, no installed-game process found.
Production build 71614 remains live. Poll same handle to terminal; then recheck
app/pack/save hashes and running game before using the existing installer.
Do not overwrite a concurrent installation or change another running process.
No installation yet. Full goal remains active; this is a modest exact-data
render optimization, not evidence that dense scenes are already fluid.


Vertex-fetch deployment verified (2026-10-21). Production 71614 exited 0 in
9m59s. Pre-install app/pack/save and frozen-source hashes rechecked unchanged;
no installed-game process running. Installer 74945 exited 0 in 39.3s, using
existing desktop.py install --skip-build with frozen open-model root and native
pack. Installed executable SHA256:
e48c710abc24b26a6a62e3ab3d5c22d4a3ac9f033fc5f0b4c25b4229e2710729.

Outside-repository /tmp invocation with CRYSTAL_OPEN_MODEL_ROOT unset,
installed Geothite --check-installation session 17321 exited 0 and resolved
bundled Resources/core-modular.crystalpack plus the existing external save.
Independent codesign --verify --deep --strict exited 0. Source pack, bundled
pack and all recorded saves remain byte-identical. Receipts:
before-vertex-fetch-install.json and vertex-fetch-installed-verification.json
under ignored .local/hierarchy-cache-production. Logs:
/tmp/geothite-vertex-fetch-{production-build,install,installed-check}.log.
No visible launch, process closure, staging, commit, pack regeneration or audio
change. Scoped diff check remains clean. Concurrent root mountain compilation
errors were reported separately; production-snapshot tests passed.

This installs the exact-data vertex-fetch improvement while retaining all prior
verified preloading and CPU-region fixes. The 120-second comparison improved
median 77.93 to 72.88 ms, but the broader dense-scene fluidity objective remains
unachieved. Next investigate the dominant GPU work with the installed Xcode
Metal System Trace capability; template/help discovery completed successfully
(72879/68788 exited 0) without starting a profiling recording or changing timing
tools. Do not count profiler availability as a performance improvement.


Current-source GPU profiling follow-up (2026-10-21). Initial xctrace launch
98777 exited 54 because the existing headless guard required
CRYSTAL_MEASURE_3D_VALIDATION=on; no GPU inference from that attempt. Corrected
45-second launch 27406 reached the time limit but produced an oversized capture.
At 5.8 GB it left only 6.8 GiB free; SIGINT and SIGTERM requested cancellation.
xctrace then exhausted disk space while saving and aborted (134). A subsequent
SIGKILL command was not executed because its here-document could not allocate
space. Only task-owned profiling outputs were removed with validated Python
path checks. Two closed temporary kernel traces were also proven to match the
capture log creation times (1.96/2.04 seconds after their logs):
instrumentsT7yHK9.ktrace and instrumentsOogNfS.ktrace. Removing these released
5.96 GiB; free space recovered to 12 GiB and later 20 GiB. No other application,
build cache or unrelated older capture was intentionally closed or deleted.
The installed app, source/bundled pack and save hashes still matched the verified
vertex-fetch installation receipt after recovery. Do not retry a long unbounded
Metal recording on this disk.

A smaller successful recording used already-warmed task-owned client PID 22788
(session 28357), xctrace --attach 22788 --time-limit 5s --window 3s (session
53112). Both exited 0. Trace vertex-fetch-metal-short.trace was about 45 MB;
TOC export 93555, GPU interval export 58149 and encoder export 14101 exited 0.
Logs /tmp/geothite-metal-short-{target,record,export,gpu-export,encoders-export}.log.
Target-only Active depth-zero GPU intervals (230 rows): Vertex 77 intervals,
sum 119.325 ms/largest 23.397 ms; Fragment 75, 46.219/6.016 ms; Compute 78,
8.923/1.936 ms. Interval sums overlap, cover a limited sample and are not frame
percentages or displayed FPS. Other-process rows were excluded. Application
encoder labels are unavailable (encoder-list export has no rows), so do not
attribute a named pass from this capture. Existing earlier labeled capture
research already identifies vertex work as a stronger lead than indirect
preparation. No new profiler/timing feature was implemented.

Small derivable receipts under ignored hierarchy-cache-production:
metal-short-{toc,gpu-intervals,encoders}.xml and metal-short-gpu-summary.json.
This evidence supports a targeted current-roof normal-representation comparison,
not another generic fragment-shader guard trial. Existing prior packed-normal
Route29 comparisons were neutral; no current-roof conclusion follows from them.
The native normal-packing startup override already exists, and exact welding,
triangle/fetch localization and source geometry remain unchanged when it is off.

Started float-normal roof comparison 5992 with
CRYSTAL_PACKED_GROVE_NORMALS=off, same immutable vertex-fetch-frame-client and
same 2560x1600/Roof9,13/noon/camera0,7,5,0/UUUUDDDD/step1/60s settings.
Output float-normal-roof-first, log /tmp/geothite-float-normal-roof-first.log.
Poll exact handle to terminal, then compare default packed normals using the
same executable and reverse order if evidence warrants. No source or installed
normal policy changed; do not claim a performance benefit before measurement.


Current roof float-normal comparison completed: float 5992 exited 0,
967 updates, median/p95 59.39/86.09 ms; packed 42387 exited 0, 958 updates,
61.29/73.96 ms. Walking float 59.30/88.69 versus packed 58.83/66.15 ms;
both 256 rows/four origins/no extra builds. This is no convincing float-normal
win: whole-run median slightly improves while walking median is neutral and
slow-frame behavior worsens. Keep the installed packed-normal default; no source,
shader, normal policy or player toggle changed. No further repetition of this
weak candidate is justified by these results alone.

Bevy 0.20 mesh.rs RenderMeshInstanceGpuQueue::GpuCulling carries MeshCullingData
for both changed_cpu_culling and changed_gpu_culling queues (around lines
1090–1110). NoCpuCulling changes CPU visibility membership; its absence does
not itself prove meshes lack GPU culling data. Do not revive an old camera-
occlusion trial solely on that false premise.

Bounded launch profiling 77005 used --time-limit 30s --window 2s and >15 GiB
free-space preflight. Target validated the roof and finished its 15s diagnostic
run; profiler saved a 144 MB trace but returned 54. Encoder export 30949 exited
0 and again contains zero application encoder rows, so it provides no named-pass
attribution. Logs /tmp/geothite-metal-labeled-{target,record,encoders-export}.log.
Removed its raw trace and its single closed matching temporary instruments6Uul2I.ktrace
(created during that capture); free space is 25.74 GiB. Compact prior target-only
GPU XML/summary remains; completed profiler-run directories were removed.
No profiling process remains live.

Installed xctrace help list confirms instruments are enumerable. Its list
contains a distinct Metal Application instrument alongside GPU and Metal GPU
Counters; --instrument 'Metal Application' --show-recording-options succeeds
without starting a recording. Next determine whether explicitly selecting that
instrument supplies the missing application labels, retaining a bounded window
and disk headroom. If not, do not keep repeating identical label-free captures.
Apple's Metal developer workflow overview is at
https://developer.apple.com/documentation/Xcode/Metal-developer-workflows .
Full gameplay-performance goal remains active; current installed e48c710a...
vertex-fetch build is unchanged by these diagnostics.


Explicit Metal Application follow-up 24728 saved its bounded trace and returned
54. Target completed the validated 15-second diagnostic run. Export 27619
exited 0, but application encoder rows remain zero. Explicitly adding that
instrument did not solve attribution; stop repeating this approach. Retained
small metal-application-encoders.xml and logs; removed raw trace, target-only
profiling directories and closed matching temporary instrumentsWgAIyi.ktrace
(created 31.91s after its capture log). Free space 26.06 GiB. Prior target-only
stage summary/XML remains available. No installed/source-pack/save policy change.

Fine panorama occlusion preparation trial, private only. Saved nine current
source baselines under fine-occlusion-baseline before edits. The old two-section
trial alone was neutral; this trial targets up to sixteen spatial groups plus
real two-phase occlusion, preserving all triangles and draw distance. Ported
its five-file section ownership changes with a current-source three-way merge.
Initial patch check rejected the occluder hunk; no partial patch applied. Git
merge-file reported two additive test conflicts, resolved by retaining both
installed vertex-fetch tests and both section proofs. Region gating and the
installed vertex-fetch permutation remain intact; fetch permutation now also
visits every additional opaque section. No lib/shadow/GPU-policy edits yet.

Native panorama preparation uses up to sixteen fitted opaque sections. Ordinary
grove preparation and browser/worker paths retain existing behavior. Original
collision and sorted fade ownership remain whole; shared/weak-cache publication
retains each section. The section proof checks every ordered triangle attribute
hash before/after partitioning and again after packing, welding, triangle and
vertex-fetch localization, across pine, ancient and deciduous source trees.
Source fallback and unsupported-device behavior still need review before any
production adoption; this is not a frame-rate claim.

Section test session 3078 exited 0, all 68 occluder tests passed (including wire
and new section/fetch fidelity tests). Follow-up 93196 exited 0: shared_groves 3,
forest_instances 11, summit_vista 11. Total 93 passed, zero failed/ignored.
Logs /tmp/geothite-fine-occlusion-{section,shared_groves,forest_instances,summit_vista}-tests.log.
Current five private source hashes in fine-section-source-receipt.json. No
compiler, profiler or render process from this task remains live.

Next integrate conservative native camera occlusion using actual GPU-building/
culling capability, compatible perspective/indirect projection and a real
DepthPrepass. NormalPrepass alone does not imply a separately sampleable depth
texture (Bevy prepass/mod.rs); check component ownership and deferred incompatibility.
NoCpuCulling is not a prerequisite for GPU culling data. Incremental GPU-only
membership can be considered separately if fine groups create a measured CPU
visibility bottleneck. Then compare full-resolution current Roof walking both
orders against vertex-fetch-frame-client cba0df36... and verify cached entry/
visuals before promoting anything. Installed e48c710a... and root renderer code
remain unchanged by this private trial. Full fluid-gameplay goal remains active.


Combined fine-group/two-phase occlusion private trial implemented. Added native
panorama_occlusion.rs and native-only lib registration. It detects actual
RenderMeshInstances::GpuBuilding plus GpuPreprocessingSupport::Culling, then
adds only OcclusionCulling to a compatible world camera with an existing
DepthPrepass. Deferred, direct-draw, orthographic, missing-depth or unsupported
GPU states remove only this module's owned component. Pre-existing culling,
depth components, non-world cameras and sun remain untouched. Compatible
inactive cached cameras retain ownership. Scope is Roof and the three existing
summit-preload floors, allowing preparation during approach. No new environment
or player toggle, shader, material, renderer output or per-mesh frame scan.
The finer grouping remains a private trial and unsupported-device grouping
fallback still needs review before any production adoption.

Camera session 36721 exited 0: 3 lifecycle/capability tests passed. Expanded the
existing production fade fixture to exercise packed partitioned trees: every
opaque section is visible initially, all become hidden on fade while original
sorted geometry publishes in the same update, and all return without mesh or
material leaks. Session 82706 exited 0, both original and partitioned fade tests
passed. Prior 93 geometry/cache/worker/panorama checks passed; no production
performance claim follows from these tests.

QA release build started with frozen seven-file source receipt
fine-occlusion-source-receipt.json, log /tmp/geothite-fine-occlusion-release-build.log.
Use its exact returned handle until terminal; no input changes during compilation.
Then save an immutable fine-occlusion-frame-client and compare full-resolution
Roof walking in both orders against vertex-fetch-frame-client cba0df36..., with
no owned compiler concurrent. Confirm real culling activation/depth support,
paired state-preserved visuals and cached roof round trip before any promotion.
No root renderer or installed e48c710a... app change. Full goal remains active.


Fine-occlusion QA release exact live handle: 78946. Continue polling the same
build; no restart or source edit while it is active. This private trial currently
requires existing DepthPrepass and excludes deferred/direct/orthographic paths.
Source inspection confirms native High-quality SSAO requires both DepthPrepass
and NormalPrepass (Bevy ssao/mod.rs lines 111–114), so the planned default-quality
Roof measurements exercise the correct custom normal prepass.

Additional correctness guard to resolve before any production promotion:
require NormalPrepass as well. VoxelSurface's custom aperture/cutout fragment
is selected for the normal prepass, while a depth-only unlit path can otherwise
retain StandardMaterial's depth fragment. An explicitly retained depth texture
without normal output must not become an occluder for a visually discarded
surface. Preserve the frozen compiler input now; after it finishes, add the
normal requirement and a missing-normal lifecycle test before production use.
Also review unsupported-device coarse grouping fallback; finer groups remain
prepared even when camera culling is unavailable in this private trial. Neither
edge case is a claim of completion or approval for shipping. Installed app stays
e48c710a... and no other process is touched.


Fine-occlusion QA 78946 completed successfully in 5m24s; frozen seven-file
source hashes matched its receipt. Saved immutable fine-occlusion-frame-client
SHA256 6477dbfa5b23e8d936e7759c74564e91f3d6a8d032cee93b3e4a7586c8bb02b2.
This executable predates the additional NormalPrepass compatibility guard;
only use it for the default High/SSAO-on comparison, where both prepasses are
present and the compatible branch is identical. Do not ship it as production.

Added the NormalPrepass guard to private source and extended missing-normal
fallback coverage. 30744 exited 0, 3 lifecycle tests passed, native GPU test
initially ignored. Added an explicitly offscreen native test with no primary
window and an image target. It verifies actual GPU-building/culling support,
owned camera activation and real PreprocessBindGroups::IndirectOcclusionCulling
with both early_indexed and late_indexed bind groups populated. 31874 passed
on the native device in1.21s but exposed a fixture-only missing-LUT tonemapping
warning. Set fixture Tonemapping::None and DebandDither::Disabled; clean repeat
80276 passed in0.19s. No renderer/performance code changed for that fixture fix.
Guarded source hashes are in fine-occlusion-guarded-source-receipt.json. Any
production build must use this guarded source plus reviewed grouping fallback.

First untraced full-resolution comparison completed with explicit unchanged
High quality and SSAO on, no owned compiler concurrent: candidate 78898 and
control 79253 exited 0. Roof9,13/noon/2560x1600/camera0,7,5,0/UUUUDDDD/step1/60s.
Candidate1014 updates median/p95 56.75/79.22 ms; control868,69.53/93.76 ms.
Walking candidate50.92/58.70 versus control63.38/87.06 ms. Both256rows/four
origins/no extra builds. Queue callbacks candidate16.860/s (outstanding31to32)
versus control13.922/s (3to36), no poll errors; callback rates are not displayed
FPS and queue fill contributes to their difference. Median improvement is
about18.4% for this first pair, not yet a repeatable/global fluidity claim.

Reverse control 98868 is live, output fine-occlusion-control-second, log
/tmp/geothite-fine-occlusion-control-second.log. Poll exactly that handle to
terminal, then run candidate-second with identical explicit High/SSAO settings.
No source or installed app promotion. Subsequent gates include paired visuals,
normal asynchronous cached roof entry and unsupported-device coarse-grouping
fallback. All baseline/candidate model assets are the same frozen private root.
Full goal remains active.


Reverse control 98868 has now exited 0:928 updates,64.42/107.14ms median/p95,
walking62.19/94.71ms,256rows/fourorigins/no extra builds. Matched candidate-second
73959 is live, output fine-occlusion-candidate-second, log
/tmp/geothite-fine-occlusion-candidate-second.log. Poll this exact handle next;
all owned compiler/test jobs are terminal. No new source edits or install.


Fine occlusion reverse candidate 73959 exited 0:835 updates,61.82/114.39ms median/p95, walking56.05/91.25ms. Queue callbacks13.925/s versus reverse control14.887/s; no errors. The reverse pair does not establish stable throughput/tail improvement, so this candidate remains unpromoted.

Matched production 3D/2D captures completed with state_preserved=true for both candidate and control. Roof/forest/building/player detail retained on inspection. Ready-state CPU trace medians (ms), candidate versus control: check_visibility11.198/2.533; visibility propagation3.867/2.530; transform propagation1.115/0.510; fades1.934/1.995; main24.535/11.319. Fine sections add substantial CPU scanning, motivating incremental GPU visibility only for proven static opaque sections.

Private fine-incremental trial: admit only marked fine regional opaque noncaster/nonreceiver meshes with finite bounds, actual GPU culling support, compatible perspective indirect views, and opaque material. Move admitted draws from world31 to panorama32; world camera and receiver bounds include both, sun remains31. Restore original layer/culling on proof, material, device or view changes. Sorted fade draws remain on the original path. Unsupported views use the exact original coarse panorama library; unknown hardware defers job scheduling without consuming jobs. No player toggle.

Validation: panorama filter10 passed/1 explicitly GPU-ignored; ownership5 passed; actual offscreen VoxelMaterial early+late indexed occlusion regression passed with GPU visibility/layer32. Geometry/cache/shadow filters passed: occluder69, shared3, forest11, summit13, bounds7, fit5, receivers10. Source receipt fine-incremental-source-receipt.json. Release comparison still required; neither root implementation nor installed application changed for this trial.


Fine incremental release build48614 exited0. Immutable QA client SHA256 9341e7adead71374e4c407ce43cb0d3ea65622f52733c070238349bb4b5be74d. Full-detail High/SSAO roof2560x1600 60s comparisons, candidate/control then control/candidate; no owned compiler concurrent, other applications untouched. First candidate917updates62.59/88.47ms median/p95, walking58.716/68.891ms, callbacks15.279/s. First control682updates86.17/142.55ms, walking86.113/134.776ms, callbacks11.365/s. Reverse control814updates68.34/131.76ms, walking57.698/109.973ms, callbacks13.688/s. Reverse candidate807updates73.18/110.53ms, walking78.032/126.657ms, callbacks13.442/s. Every run256walkrows/fourorigins/oneinitialbuild/no pending terrain/no poll errors. Callback rates are not displayed FPS; queue backlogs varied.

The reverse pair reverses median/throughput/walking benefit; do not promote this composition. Archived all11 candidate source files in fine-incremental-unpromoted-source and restored the9 pre-fine-trial baseline files, removing both trial modules. Installed accepted vertex-fetch app and root implementation remain unchanged. Fine grouping+stock occlusion+incremental GPU visibility has not demonstrated repeatable fluidity. Avoid claiming this trial fixed gameplay or rerunning the same composition without new evidence.


Previous goal turn classified progress: tested and rejected fine/incremental composition using reverse-order full-resolution evidence; restored accepted private source. New investigation returns to transitions. Current accepted immutable vertex-fetch client47973 completed normal asynchronous --record10s, House1F6,6/noon/full2560x1600/High/SSAO/camera0,7.5,45,0/DDBBBBBBBBBB, no owned compiler concurrent. Trace vertex-fetch-house-exit.trace.json, log /tmp/geothite-vertex-fetch-house-exit.log. 220updates/112captures, median43.26/p95107.37ms includes PNG capture overhead, not normal gameplay FPS. 191Town presentationevents,2pending; first walk Town row alreadypendingfalse, terrainactive, normalFadeIn255. Settled frame32 inspected: player, houses, water, forest, NPCs present. Retained only frames0/17/32/111.

Ready Town trace identifies recurring water.ripples.update median2.317ms/max14.498ms (19updates); publication median0.658ms/max1.044ms. Candidate water_ripples.rs precomputes wet-cell neighbor stencils until the wet mask changes; dry cells stay zero, no-flux banks and original floating addition order retained. Tile-block wet-mask expansion removes per-sample dynamic width divisions. No quality/resolution/cadence change or player switch. Initial10tests passed, including80-step bitwise old-vs-new physics across1x1/1x17/19x1/48x48/127x93 grids and wet/dry/sparse/banked masks, plus shoreline mutation. Tile-mask oracle added, tests pending91279. Scratch optimized Rust comparison of actual old/new step functions,192steps each/bothorders:256square 10%wet original0.044-0.053ms vsstencil0.012-0.013;50%0.069-0.080 vs0.037-0.040;100%0.113 vs0.069.512square10%0.123-0.128 vs0.042;50%0.273-0.279 vs0.151-0.156;100%0.457-0.459 vs0.280-0.282. Isolated evidence only; full renderer validation pending. Baseline water-stencil-baseline.rs matches rootSHA4f3ea93c73d8e13ace2f38fd6d7309179ff54e02217a5b13527fc918935f844b.

Bevy0.20 research: https://docs.rs/bevy/0.20.0/bevy/render/render_asset/struct.RenderAssetBytesPerFrame.html and localrender_asset.rs/mesh/allocator.rs. Generic byte limiter is soft and accounts RenderMesh metadata preparation, while allocator independently walks ExtractedAssets meshes. Do not claim it bounds full mesh upload, or introduce delayed assets without verifying destination GPU readiness. No limiter change made.

Water tile-mask regression91279 terminal0:11passed0failed in0.24s. Current accepted house exit firstTownpending-to-ready275.828ms; two trace frames, thenreadywhile normalfade stillcoversarrival. Baseline capture overhead prevents general latency claims. Removed owned microbenchmark source/binary after recording results. Water release QA build log /tmp/geothite-water-stencil-release-build.log; source frozen to water-stencil-source-receipt.json until terminal. Root/app unchanged pending full-renderer performance and paired visuals.


Water release QA74917 terminal0, immutable water-stencil-frame-client SHA25607eb7184d374c971ef9b0c56e8c9328540469714b3a0195476ae47f4f1c70b45. Full-resolution NewBark13,6/noon/High/SSAO/camera0,7.5,45,0,30seconds/LLRR, existing trace, candidate-control-control-candidate21245 terminal0. No owned compiler concurrent. First candidate960updates29.19/43.12ms median/p95; control831updates35.36/47.77ms. Reverse control730updates38.95/81.89ms; candidate725updates39.61/75.67ms. Each128walkrows/twogridorigins/oneinitialbuild/no pendingterrain. Overall FPS improvement not established by reverse pair.

Ready-state water.ripples.update median/p95 ms: candidatefirst1.585791/2.377875(69samples), controlfirst1.794937/2.888875(62); controlsecond3.518605/7.091583(48), candidatesecond3.401750/6.269667(51). Unchanged nested publication medians0.562583/0.550146/1.036375/1.128917 respectively demonstrate changed CPU contention between pairs. Simulation median improves in bothorders; infer modest CPU savings only, supported by exact old/new physics and optimized isolated tests. No global fluidity claim. MainCPUmedians candidate/control4.020708/4.244042 and reversecontrol/candidate8.303333/8.129333ms.

Paired capture68135 terminal0, water-stencil-{3d,2d}.png and3d.paired-state.txt. Both visually inspected, state_preserved=true, frame119/hash3441219314, fouractors/samefacing/map/time. Forest/houses/shore/water/player retained. Initial CLI attempt --second-2d rejected; corrected actual --second option before successful capture. Promoted exactly water_ripples.rs after checking root baseline and frozen candidate hashes. No other root implementation touched. Private controller/MCP/stdio suite47495 live, /tmp/geothite-water-stencil-gameplay-tests.log; freeze private inputs until terminal. Root targeted11water tests and production app packaging still required. Installed app unchanged. Full loading/FPS/fluidity objective remains unmet.


Water gameplay suite47495 terminal0: controller8passed/1existingignored9.67s, real MCP movement/menu1passed0.80s, stdio1passed1.32s. Root water suite54492 terminal0:11passed0failed0.26s, /tmp/geothite-water-stencil-root-tests.log. Thus root source compiles against current concurrent work; unrelated edits preserved. Production build59230 currently live (exactwrite_stdin revalidated), log /tmp/geothite-water-stencil-production-build.log. Freeze private source/assets; poll this handle to terminal before packaging. Installer not started, app unchanged. Before installation revalidate current executable expectedSHAe48c710abc24b26a6a62e3ab3d5c22d4a3ac9f033fc5f0b4c25b4229e2710729, source/bundledpackSHAa0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007 and all saves. Lastprocesscheck no installed Geothite process; recheck immediately before replacing app and leave all other processes alone. After install use outsidecwd/tmp unsetCRYSTAL_OPEN_MODEL_ROOT --check-installation, strictdeepcodesign, exactpack/savecomparison. No visible launch authorized. Full goal remains active: this modest CPU improvement does not prove broad fluidity or dense-roof GPU performance.


Water optimization deployed. Production59230 terminal0, release6m21s, /tmp/geothite-water-stencil-production-build.log. Installer51017 terminal0 in38.8s, /tmp/geothite-water-stencil-install.log; used frozen open-model assets and completed shared native pack, --skip-build release target/performance-release-native. No installed game process before replace; other processes untouched. Fresh preinstall receipt before-water-stencil-install.json contains TWO current external saves (not the earlier one-save historical snapshot).

Outsidecwd/tmp unsetCRYSTAL_OPEN_MODEL_ROOT installed --check-installation69825 terminal0; actual bundled pack/resources and external save resolved. codesign --verify --deep --strict terminal0. First naive verification compared signed installed whole-file hash to linked binary and correctly found a mismatch caused by signing. Corrected verification43751 terminal0: installed exact byte match to signed .local/desktop-dist/Geothite.app executable; all15 named nonzero Mach-O section hashes identical to compiled release binary, signature independently valid. InstalledSHA256c1b1029edc700824f4f7ce42f08087cb773adf18f53990405b692033b9697f47; compiledunsignedSHA0805a968fa40f2db3f53dc2d74cee0c8259c5c25fa4ad5c68b90e688957ea5d7. Source and bundled pack remaina0153af8315e37960e4fb8949a0ec47ce7163eaff6e2cf9eadbb7c8843820007. Both external saves exactly unchanged. water-stencil-installed-verification.json retains receipt. No visible window launched.

Previous turn progress: validated and promoted CPU water change; this turn progress: production built/installed/standaloneverified. Full goal remains active/unmet: broad smoothness and dense-roof GPU cost remain. Potential next research only, no source changed: grass_vertex.wesl evaluates length(world_from_local[1].xyz) as a function argument for any UV-bearing mesh, although only sentinel grass/flower branches need it; check native generated behavior/real eligible geometry before assuming measurable savings. Static index compaction already implemented; do not duplicate it. Both prior fine grouping/occlusion compositions remain rejected.


Color-palette investigation59990 terminal0: actual native prepared/welded/packed opaque grove meshes across3placements per family retain exact colorbits. JohtoPine46331vertices/22185uniqueRGBA; RGBattributes555972bytes vsindex+RGBApalette540284 (2.8%less). Ancient13520/4745;162240vs130000 (19.9%less). Kanto79274/63894;951288vs1339400 (40.8%MORE). Palette lookup adds GPU indirection/storage binding, so no renderer implementation justified for denseJohto case from this evidence. Archived test-only probe color-palette-probe-unpromoted.rs and restored private occluder_fade.rs exactly; root/app unchanged. Shader grass-length hypothesis also ruled out for main densegrove draws by source: shared_groves always compacts allzeroUVs before upload, and grass vertexcall itself is VERTEX_UVS_A guarded. Do not spend a release build on that path for roofgroves.

Unresolved evidence gap in rejected fine-incremental trial: actual production CPUtrace never collected, only unitGPUfixture and wallframe comparisons. Next diagnostic uses immutable rejected candidate9341e7ad and accepted control07eb7184 on same roof; does not reactivate/promote rejected source. Determine actual remaining visibility/transform cost before choosing another renderer change.


Actual rejected fine-incremental production trace45497 terminal0, fine-incremental-ready.{trace.json,-3d.png,-2d.png}; matched accepted water-stencil control27327 terminal0, water-stencil-roof-ready.*. Roof9,13/noon/High/SSAO/2560x1600/camera0,7,5,0, no owned compiler concurrent. Ready+500ms schedule-envelope medians/p95ms candidate/control: check_visibility2.485/3.495(65samples) vs4.315/5.849(55); visibility_propagation7.061/10.168 vs3.830/4.574; transform1.135/1.863 vs0.748/0.967; fades2.244/4.011 vs2.944/3.476; main16.937/33.074 vs17.304/24.548; render.prepare_gpu4.825/11.786 vs3.873/5.600. These are envelopes, not exclusive system profiles. Real GPU admission does reduce CPUcheck cost relative to the earlier fine-only11ms evidence, but overall mainCPU is near-neutral and management overhead remains. Composition remains rejected; no new fine-group source promotion.

New private native opaque triangle-cache ordering trial. Primary references: https://tomforsyth1000.github.io/papers/fast_vert_cache_opt.html (LRU model32, recent-three0.75, valenceboost2/sqrt, cachedecay1.5) and https://github.com/zeux/meshoptimizer/blob/master/src/vcacheoptimizer.cpp (researchonly; no C++ dependency/source added). Own Rust implementation vertex_cache_order.rs uses fixed32cache/CSR adjacency/bounded256valence, deterministic tie breaks, malformed/highvalence fallback to existing linear DFS. Only native non-worker opaque grove GPU copies; collision and sorted fading leaves unchanged. Existing first-reference vertex-fetch permutation runs afterward. No player/environment toggle added.

Initial unit40449 terminal0:3passed, actual native prepared grove9recipes acrossJohto/Ancient/Kanto, triangle multiset/winding exact, no cache-model regressions acrossFIFO/LRU16/32/64. Johtoaggregate reductions12.19/11.56% at16,6.67/6.58% at32,2.34/2.44% at64. Ancient24.89/24.69%,20.83/20.66%,8.69/8.95%. Kanto5.48/5.42%,4.69/4.64%,3.70/3.66%. Simulations only, not GPU/FPS proof. Integrated productionpath22396 terminal0:3passed8.75s, exact actualattributes, triangle count/winding, direct candidate matches production output, cache-model assertions and malformed/degenerate/duplicate/unusedvertex/fan fallback tests. Added native/worker testguard and formatted only newmodule. Broader geometry/instance suite32307 live, /tmp/geothite-cache-order-regressions.log. Current private source receipt cache-order-source-receipt.json; installedacceptedwaterSHA c1b1029e... unchanged. Release GPU trial, paired visuals and transition readiness still required before promotion.

Cache ordering geometry suite32307 terminal0:occluder67passed10.99s, shared3passed6.04s, forest11passed7.67s. Private source hash receipt reverified before release QA. Build log /tmp/geothite-cache-order-release-build.log; freeze source/assets until terminal, save immutable cache-order-frame-client on success, compare to water-stencil-frame-client07eb7184... with identical full-resolution roof conditions in both orders. Include normal asynchronous cached roof entry and paired3D/2D plus source-build-cost check before promotion. All installed/root implementation remains acceptedwater baseline.

Previous goal turn classified progress: ruled out UV-shader/palette paths with actual geometry evidence; implemented and regression-tested private cache ordering. Current release QA1342 confirmed live by exactpoll; source remains frozen to cache-order-source-receipt.json. Comparison harness prepared with accepted water-stencil-frame-client baseline, roof9,13/noon/High/SSAO/full2560x1600/60s/UUUUDDDD/candidate-control-control-candidate. No renderer promotion until actual GPU, visual and transition gates pass.


Cache-order release QA1342 terminal0 in10m31s; compilation overlapped two observed unrelated root builds, left untouched. Frozen candidate hashes reverified, immutablecache-order-frame-clientSHA4645656490b2093b02c4fbc09e5cf226b5ee6fe2f37782417435c53f38b82019. Before measured interval those two observed builds had exited. Benchmark58969 terminal0, full2560x1600/High/SSAO/Roof9,13/noon/camera0,7,5,0/UUUUDDDD/60s, candidate-control-control-candidate. Candidatefirst709updates80.45/148.90ms median/p95, walking82.890/156.718ms, callbacks11.824/s. Controlfirst812updates73.34/118.17ms, walking75.880/118.487ms, callbacks13.576/s. Reversecontrol928updates58.95/116.66ms, walking56.7385/105.287ms, callbacks15.530/s. Reversecandidate797updates72.41/120.52ms, walking67.7305/104.303ms, callbacks13.276/s. Everyrun256walkingrows/fourorigins/oneinitialbuild/no terrainpending/no poll errors. Callback rates are not display FPS.

REJECTED: real median/throughput/walking behavior worsened in bothorders despite FIFO/LRU simulation gains. Do not promote from cache-model results or claim faster game. Archived three changed files in cache-order-unpromoted-source, restored exact pretriallib/occluder and removed vertex_cache_order.rs. Accepted water_ripples retained. Root and installed c1b1029e... app unchanged by this trial. Additional visual/transition acceptance gates unnecessary for a failed performance candidate; none claimed.

Primary research caveat https://zeux.io/2017/07/31/optimal-grid-rendering-is-not-optimal/ explains modeled cache replacement can differ from real hardware; this does not establish this trial's precise slowdown cause. Apple reference https://developer.apple.com/documentation/metal/tailor-your-apps-for-apple-gpus-and-tile-based-deferred-rendering requiresJS; Markdown endpoint unsupported by web tool, so no unseen content claimed. Full performance goal remains active; next investigation should target measured per-pass GPU work or transition blocking rather than retrying this same index heuristic.


Previous turn progress: rejected classic cache order from two-order measured regression and restored accepted source. This turn investigated actual CPU/RAM rather than another assumed index model. Baseline untraced roof11868 completed before sampler attachment (missed observation window, not a renderer failure). Automatic own-target sample98347 completed0, PID83360,5sCPU sample attached only after validated measurement began; /tmp/geothite-accepted-roof-cpu-profile.sample.txt (860KiB). Main-thread933samples,817inconditionwait; release strip=symbols prevents Rust function attribution. All-thread wait counts must not be interpreted as serial CPU percentages. Physicalfootprint9.4GiB/peak11.2GiB on24GiBmachine.

Owned vmmap profile70853 completed0, PID84071. At measurement start physical11.5GiB/peak12.0GiB, CPUmalloc4.3GiBallocated across5.42millionallocations, graphics mapped2.9GiB plusowned-unmappedgraphics3.1GiB; processswapped6.5GiB. After15sphysical10.9GiB/swapped6.6GiB, allocatedCPUheapstill4.3GiB. Existing browser_memory caps native meshslabs16MiB already; do not duplicate that fix. Successful primary AppleMarkdown fetch viaurllib confirms TBDR evaluatesgeometry before visiblefragments: https://developer.apple.com/documentation/metal/tailor-your-apps-for-apple-gpus-and-tile-based-deferred-rendering.md . No new diagnostic product setting or instrumentation code.

Concrete memory lead: BevyRuntimeShell retains bothruntime and RuntimeGameShell; CrystalRuntime derives deepClone of GameDataSet/runtime_files/audio. Native scenes/checkpoints can duplicate verified immutable content. Private candidate wraps those3fields inArc, shares browser runtimefileOnceLock too, retains mapcatalog/caches/mount lifetimes and publicborrowed APIs. No pack/schema/program/PCM change. Initial edit-script fixture assertion failed and a test invocation exposed9adaptationerrors; corrected iterator borrowing andtestfixture/COW edits. Sharing/isolation52707passed1test: all3pointerequal onclone, semantic equality, COWdata/filesleaveoriginaluntouched, audioexactequal.

Fullruntime203passed/52failed/1ignored initially; manytests use legacyvendorasset roots rather than CRYSTAL_RENDER_TEST_PACK. Firstpacklinks resolved only2, giving205/50failed. Added symlinks (no copies) for both root/sourcecontent-packs andapps/web/assets/data/content-packs. Remaining baseline comparison must run before identifying regression. Candidate3files archivedruntime-sharing-candidate-source withhashreceipt, exactbaseline3files restored solely for baselinefullsuite. Rootimplementation/app unchanged. Facing-onlypreloadheuristic not implemented: catalog layout andlastcornerapproaches need validation, cannot assume nearest-behinddoor is the sole blocker.

Runtime baseline1956 INVALID as comparison: source was restored with copy2 oldmtimes, Cargo reused candidate testbinary (newsharingtestappeared inbaselineoutput although sourcehasnone). 206passed/49failed/1ignored reflects candidatewithcompletepacklinks, not truebaseline. No regressionclassification fromthatresult. Touch restored3sources and rebuildbaseline; checknewtestABSENT in--list/output beforecomparing. Future source restores must copyfile or touch aftercopy2, never rely on oldmtime invalidation. Root/appunaffected.

Fresh baseline9707terminal101:255tests,205passed49failed1ignored39.15s, newsharingtestabsent. This is validunchangedbaseline. Candidate3archivedsourcehashes reverified/restoredwithcopyfile (freshmtimes) beforefreshcandidatefullrun. Packlinks remainexactsharedcompletednative/browser/TUIpacks; noexports/buildpipeline rerun. QA releasecandidate canbuildinparallelwithunittests; noGPUmeasurementuntilbothterminal.

Fresh candidate82907terminal101:256tests,206passed49failed1ignored52.13s. Exact failed-test set equals validfreshbaseline9707; all205previouspasses retained, onlyadditionalpassisnewsharing/isolationtest. Newfixturecount proves actualcandidatebinary. ConcurrentQAcompiler means suiteelapsedtimes are not performance evidence. Release17580 remainslive, /tmp/geothite-runtime-sharing-release-build.log. Source3hashesremainruntime-sharing-source-receipt.json; freezeprivateinputs. Saveimmutable runtime-sharing-frame-client onlyafterterminalsuccessfulbuild. Prepared one-off memory comparison runtime-sharing-memory-compare.py, control-candidate-candidate-control, nativefullroofHigh/SSAO, vmmapatmeasurementstart/+15s. It reportsCPUheapallocated bytes/counts andphysical/peak; its memory-probedframetimesareNOTcleanFPSgates. Untracedframecomparisons, normalasynchouseexit, paired3D/2D, mandatorycontroller/MCP/stdio androotchecksremainbeforepromotion. Root/appunchanged.

Runtime sharing release17580terminal0 in6m54s. Frozen3sourcehashes reverified. Saved immutable runtime-sharing-frame-client SHA25665bb5988419a2aa340501768208ac22f5d765697e54073f63ec2d9b194130104. Memorycompare3604 live, controlfirst; allownedcompilers/tests terminal before GPU runs. Previousgoalturnprogress: validfreshbaseline andcandidatefullsuitecomparisonproved no lostpasses/newfailures; buildcompletedthisturn. No promotion/install yet.


Runtime-sharing memory3604terminal0, fullroofHigh/SSAO2560x1600/30s/UUUUDDDD, control-candidate-candidate-control. Eachvmmapatmeasurementstart/+15s; profiler-probedtimingsdiagnosticonly. Latecontrolfirst:CPUallocated4.3G/5425130allocs,physical10.7G/peak11.6G. Candidatefirst:3.8G/4005987,physical10.2G/peak10.6G. Candidatesecond:3.8G/3998796,physical10.2G/peak10.6G. Controlsecond:4.3G/5429369,physical10.8G/peak11.2G. Consistent~0.5G lessCPUheap and~1.42-1.43millionfewerallocations; avoidexactMiBclaimfromroundedvmmapunits.

Unprobed40231terminal0candidate-control-control-candidate60sfullroof: candidatefirst769updates75.08/140.46msmedian/p95, walking75.8985/126.062ms, callbacks12.799/s; controlfirst786updates70.97/136.78ms, walking71.2515/135.175, callbacks13.143/s. Reversecontrol842updates69.45/120.22ms,walking71.084/120.469,callbacks14.109/s; candidate729updates79.74/129.90ms,walking82.485/158.693,callbacks12.125/s. All256walkrows/fourorigins/oneinitialbuild/pendingfalse/no pollerrors. Finalrenderpositionsmatchpreviouscontrols exactly(-1356.088,1144.692;origin-44,-36), so motion-endpoint hypothesis ruledout. No frame-rate gain; bothpairsworse. Do notpromoteonmemoryresultalone.

Audit log provescrystal-voxel-view DID compileinruntime-sharingrelease, notjustcachedwarnings; missingrestoredcachemodulewasproperlyinvalidated. No evidenceofstaleGPUindexordering inthiscandidate. Nextisfreshsame-sourcecontrol: verifycurrentcandidate3hashes, restorebaseline3withcopyfilefreshmtimes, buildidenticalreleaseQAflags. CurrentprivateRuntime sourceTEMPORARILYBASELINE, root/appremainacceptedwater. Preservefreshcontrolbinaryseparately, comparecode-sectionfingerprint witholderwaterclient ifpossible, thenrestorecandidatewithfreshmtimes BEFORE anycandidateproductionbuild. No needrebuildimmutablecandidate65bb5988...unlesssourcechanges. Memorybenefitreal,butFPSattribution/acceptanceunresolved.


Fresh same-source runtime-sharing control89363 terminal0. Immutable runtime-sharing-source-matched-control-client SHA25607eb7184d374c971ef9b0c56e8c9328540469714b3a0195476ae47f4f1c70b45 is BYTE-IDENTICAL to accepted water-stencil-frame-client. This rules out full-snapshot/code drift as the reason both unprobed candidate comparisons worsened. No redundant four-run repeat needed. REJECT runtime-sharing candidate: repeatable memory reduction does not justify both-order frame-time regressions. Candidate retained only in ignored trial archive; private three runtime files restored with copyfile/fresh mtimes to exact accepted baseline and sharing test confirmed absent. Root implementation and installed c1b1029e app unchanged. No new gameplay quality settings/toggles. User No interpreted as leave other game/processes/builds untouched. Only owned QA build observed; no process was closed or suspended. Goal remains unmet/active.


Previous goal turn progress: fresh control byte-identical to accepted client proved earlier runtime-sharing FPS regression was not source drift; candidate rejected/restored. New private CPU obstruction subdivision candidate subdivides explicit regional group ownership using conservative bounds, median-axis binary tree/maximum16members per leaf. Original render meshes/materials/collision/entities/fade state retained; no new quality toggle. Fades settle before sleeping, waking invalidates exact checks; later direct-member/reparent edits fall back to original all-member path. Added transform/parent Ref checks preserve initial authored proof. Finite/rotated/scaled bound comparison tests require all brute-force possible hits selected. Synthetic4096groups selects<=16nearby; tests retain settling/reentry/invalid-transform behavior. First occluder suite40980 terminal0:68passed. Added production ECS64group test requires distant group remain unvisited after initial propagation and immediately block after moving onto rays; whole suite67236 terminal0, see /tmp/geothite-subregion-tests.log. Private candidate archived subregion-candidate-source/hashreceipt; rootimplementation/app untouched. Root/private occluder pretrial difference only one whitespace line. Next release QA build and actual fullroof both-order comparisons plus async transition/pairedvisual/controller gates before any promotion. CPU selection reduction is NOT FPS evidence.

Subregion release32663 confirmed live on launch. Suite67236 exact69passed0failed in3.82s. Frozen candidate hashes44794a45...(occluder_fade.rs),6b9c9962...(regions.rs). Do not change private source/assets until release terminal. Save immutable subregion-frame-client on success; subregion-compare.py prepared full2560x1600High/SSAO/Roof9,13/noon/60s/UUUUDDDD candidate-control-control-candidate with acceptedwater07eb7184...control. No actual GPU comparison yet; goalactive.

Current continuation classified verified wait: release32663 re-polled live, rustc final new_bark_3d stage observed; no restart. Candidate2filehashreceipt revalidated, source remains frozen. Installed app/rootimplementation unchanged. Actual GPU comparisons cannot start until compiler terminal and immutableclient saved.


Subregion release32663 terminal0 in6m44s, frozenhashesverified; immutable subregion-frame-client SHAb3b7536e3ea935258a871349da69b29bfc01f01e4ace056ac997faf0c2c87ad4. Untraced GPU comparison75007terminal0 at fullroof2560x1600High/SSAO60s: firstcandidate884updates66.63/76.19msmedian/p95, walking64.3845/72.416ms,callbacks14.719/s; firstcontrol838updates70.50/80.68ms,walking70.525/78.301,callbacks13.951/s. Reversecontrol861updates61.56/128.36ms,walking55.3825/117.35,callbacks13.802/s; reversecandidate826updates71.74/121.24ms,walking52.7465/112.613,callbacks13.163/s. All256walkrows/fourorigins/oneinitialbuild/pendingfalse/no pollerrors. Callbacks notdisplayFPS. Walkingmedian/tails improvedbothpairs, but reverseoverallmedian/throughputworsened. HOLD promotion: actualscene performance mixed, no FPSclaim. Next short bothorder existingCPUtrace comparison to establish whether fade work itself is reduced (trace includes allgroupdiagnosticfold, so notpureproductionexclusivecost). No new timingtools/producttoggles introduced. Root/appunchanged;goalactive.


Subregion CPU trace56229terminal0, diagnostic bothorders10s, same167987groups allwindows. Last8active seconds: candidatefirstfade5.103/9.303msmedian/p95 vscontrolfirst5.069/10.789; reversecandidate4.757/8.176vscontrol4.954/8.667. Tracefold visitsallgroups and is inside span; envelopes are not exclusive/additive. Ringcapacity131072dropsoldestevents, finalwindowretained. Walkingcounts64/64/96/128?inspectexactCSVbeforeclaim; these short runs do NOT compare exactrouteendpoints. No consistentlargeCPUgain, mixeduntracedoverall performance stillHOLD. Code audit finds Branch::settle recursivelywalks EVERY subdivision everyframe evenwhile asleep, undermining spatial savings. Private refinement adds earlyreturn for !selected&&!awake, relying on tested selection/settling invariants; rerunwholeoccluder suite before rebuilding. Candidatev1b3b7536e...keptimmutable, root/appunchanged.

Exact shortCPUtrace walking CSVrowcounts {'candidate-first': 64, 'control-first': 64, 'control-second': 96, 'candidate-second': 128}. Confirmed differentrouteprogress; only diagnostic CPUenvelopes, not acceptanceproof. Current refinementtest34467 confirmedlive on launch; /tmp/geothite-subregion-pruned-tests.log. Freeze sourceuntilterminal.

Subregion-pruned suite34467terminal0:69passed0failed4.26s. Private v2 archived subregion-pruned-candidate-source and subregion-pruned-source-receipt.json: occluder44794a45...unchanged, regions994a5f0d3244895b27daa7f79b719250c37d08f2f0d0a589ff17cf4dca9b1620. Release58876 live, /tmp/geothite-subregion-pruned-release-build.log. Freezeinputs; onterminalsuccesssave distinct immutable subregion-pruned-frame-client; fullroof bothorder subregion-pruned-compare.py prepared against acceptedwater07eb7184...control. Originalv1fullscene result remainsmixed, no promotionclaimed. Currentturnprogress: completedCPUcomparison, identified all-subtree settling cost, refined/testedimplementation. Root/appremainacceptedwater; goalactive.

Subregion-pruned release58876terminal0; frozen2sourcehashesverified; immutableclient subregion-pruned-frame-client SHAeb4a65b4693341c71cd9ba5a5abc7b6a46fb1a7e89d90c5ca7303650461621f0. New fullroof/untraced60s/bothordercomparison99647live, /tmp/geothite-subregion-pruned-compare.log. No other ownedcompiler remains before comparison. Root/appunchanged. Source inspection Bevy0.20 bevy_transform/src/systems.rs confirms StaticTransformOptimizations defaultEnabled; no togglerequired. Earlier fine16sections/occlusion compositions already tested/rejected, do not confuse new CPUsubdivision with those GPUmeshchanges. Currentturnprogress completedrelease and startedactualcomparison, goalstillunmet.


Subregion-pruned untraced comparison99647terminal0: firstcandidate768updates73.38/141.97msmedian/p95, walking50.3195/118.018ms,callbacks13.041/s; firstcontrol691updates83.11/136.24ms,walking79.8055/130.137,callbacks11.511/s. Reversecontrol696updates83.68/130.73ms,walking88.0185/149.426,callbacks11.603/s; reversecandidate619updates89.48/162.19ms,walking94.021/157.060,callbacks10.293/s. All256walkrows/fourorigins/oneinitialbuild/pendingfalse/no pollerrors, exactfinalpositionequal acrossfourruns {'origin_x': '-44', 'origin_y': '-36', 'center_x': '-1408.000', 'center_y': '1152.000', 'tile_x': '8', 'tile_y': '12'}. Callbacks notdisplayFPS. REJECT CPUsubdivisionfamily: v1mixedoverall and tinyinconsistentfadeCPUchange; pruningv2stillfailsreversepair foroverall,walkingandtails. No repeatablefullgamegain. Archivedv1/v2source/hashreceipts remainignored; restoredbothprivatefilesexactacceptedbaselineusingcopyfile/freshmtimes. Rootimplementationandinstalledappunchanged. Additionalvisual/controller/installgates notclaimed/executedforfailedcandidate. Goalactive/unmet.

Next directCPUattribution action: accepted-baseline symbolized QA build46855 live via cargo rustc --release --jobs2 -pcrystal-bevy --featureslocation-tester --example new_bark_3d -- -Cstrip=none, sameprivate manifest/target. Additional rustc flags only exampletarget, preserving releaseoptimization/dependencyprofile; no gameinstrumentation changes. Log /tmp/geothite-accepted-symbolized-build.log. Five touched runtime/voxel sources verifiedacceptedbaseline and receipt accepted-symbolized-source-receipt.json. Freezeinputs until terminal; save distinct accepted-symbolized-cpu-client on success, verify nm contains Rustfunctions and compare executable code sections to immutable acceptedwatercontrol wherepossible before attribution. Earlier sample failedfunctionattribution onlybecause stripped; no repeatlabellesssampleclaim. Attach existing OSsample to owned activeactualscenePID afterready, bounded5seconds; no otherprocessclosed. Root/appunchanged,goalactive.

Current goalcontinuation classifiedverifiedwait: exactsymbolized build46855 re-polledlive; voxel/bevyoptimizedrustc processes observed, no restart/sourceedits. Frozenacceptedsource5filehashreceiptverified. Prepared ignored accepted-symbolized-sample.py for owned fullresolutionHighroof andNewBark30s runs; sample attaches ownPopenPID2safteractualmeasurementready,5s/1ms, no loadingtrace, 25sOSsampletimeout. Do not rununtilcompilerterminal and immutable accepted-symbolized-cpu-client saved plus Rustsymbolpresencechecked. Reports /tmp/geothite-accepted-symbolized-{roof,new-bark}.sample.txt; timingsprofilertaintednotFPSgate. Mac c++filt demangleslegacyRustfunctionnames successfully; systemtoolchainalsohasrustc_demangle rlib ifneeded, no helper/dependency introduced. Currentapp/root unchanged; goalstillactive.


Accepted symbolized build46855terminal0 in11m16s, immutable accepted-symbolized-cpu-client SHA0547e416a94a6ae7f221c408b46601c4899fe4acd0774340fa45b772108d3d70. nm verifiesRustv0functions; read-onlyMachOparser accepted-symbolized-section-comparison.json proves ALL15namedsections (includingTEXTtext,const,DATA,unwind) byte-identical to acceptedwatercontrol07eb7184. Thus onlysymbol/metadata retention, nochangedgamecode. CPU sample43920terminal0:roofownedPID6557andNewBarkownedPID6802, ready+2s,5s/1ms, full2560x1600High/SSAO, no loadingtrace. OSsample raw/demangled reports /tmp/geothite-accepted-symbolized-{roof,new-bark}[.demangled].sample.txt; c++filt handlesv0. No othergame/processclosed; ownrunsnormalterminal. Diskobservedlow7GiBwhilelinkingthen10GiBafter; no cachesdeleted.

Namedroof selfsamplecounts (NOTCPUpercent, allthreadwaitsnotadditivewalltime): CPUvisibility parallel317/serial148, city_lights::sync materialAssetIterator146, extract_meshes_for_gpu_building iterator129, wgputracker_drop98/UsageScopeDrop73, sync_interior_cutaway73, occluder_fade60, mark_dirty_trees45, visibility_propagation32, pipeline_preload_extract28. Main1087samples/936park; renderthread1087sample envelope containsactualencoding/submit+waiting. NewBark topCPUselfcounts lower: VisualTerraceDatumPartialEq25, streamed_imageupload18, siphash16, CPUvisibility13; allocator/Metal/waits dominate. Avoidconverting counts toexclusiveCPU% orGPUtime. ProfilertimingsnotFPSgates.

Concrete nextcodelead: city_lights::sync scansALLVoxelMaterial assets andcomparescity_night plusTWO[Vec4;32]arrays evenwhen day/night/windowoutputs unchanged; broad resourceis_changed flags remaintrue dueunrelated material/frame/cachechanges. sync_interior_cutaway separatelyscansallmaterials twice for exits/sky/capsule. rg confirms ONLYcity_lights::sync writes city_night/windows/planes (allotheroccurrencesdefinitions); clonescopythesevalues. Potential exactCPU-onlylightingrevision stamp (unannotatedAsBindGroup, reflectignore) canavoid1024bytecomparisons per unchangedmaterial whilepreservingnewmaterial/clone/panoramaclass behavior; NOTimplemented yet. Need immutableclone stamp independentstorage (notsharedArcAtomic acrossdifferentclassclones), distinctrevisiontokens forpanorama/normal, allchanged/newmaterialsupdatebeforefirstdraw, noAssetEventchurnforcachedvalues, andsameGPUuniformbindings. Assessagainstexistingclock/panoramatests. No newtimingfeatures/toggles. Root/appacceptedunchanged;goalunmetactive. CurrentturnprogressactualnamedCPUevidencechangesnextimplementationchoice.


Previous goalturn progress: bit-identical symbolizedacceptedbinary plusactualOSsamples identifiedcitymaterialscan146selfsamples vsfade60. New private citylightingrevision candidate: CPU-onlyreflectignored/unannotatedAsBindGroup Stamp(AtomicU64) field onVoxelSurface; independentclonestorage; monotonicallyuniqueper-systemHistory outputrevisions. Same cityuniforms comparedonceperframe asHistoryparameters, then8bytestamp perasset instead2x[Vec4;32]comparison. Unstamped/newmaterials retainexactoldcomparisonbeforewrite; alreadycorrectnew/clone stamprecordedviaAtomicwithoutAssetEventorGPUreupload. Sole productioncityuniformwriter verifiedbyrg inprivate ANDroot. No shader/binding/lightingmath/program/packchange. Rootcity_lights.rs has concurrentpanorama-emission enhancement absentfromfrozenprivatebaseline: PRESERVE atfuturepromotion, nevercopyprivateentirefileoverroot. Rootnormal/panorama statesneedseparaterevisions whenmergingthisoptimization.

Initialcitysuite41052terminal0:7existingpassed1.48s. Addedcloneindependence/cross-systemrevision/pipelinekeyinvariance and32materialexactclockoutputs across7hourspluslateclones/noidleAssetEvents; final10163terminal0, see /tmp/geothite-city-revision-tests.log. Candidate2files archived city-revision-candidate-source/hashreceipt. Private baseline city-revision-baseline; rootimplementation/appuntouched. NextreleaseQA, actualbothorderfullroof andNewBarkuntraced comparisons, normalasynchouseexit, pairedvisual/controller/MCP/stdio beforepromotion. Metadata changes are notFPSproof.

City-revision finalsuite exact9passed0failed1.13s. Release84354live, log /tmp/geothite-city-revision-release-build.log. Freezeprivateinputs tocity-revision-source-receipt.json (libbaba0472...,city626f78ea...). Saveimmutable city-revision-frame-client afterterminalsuccess. city-revision-compare.py prepared untracedfullroof60s/candidate-control-control-candidate againstacceptedwater07eb7184...; no current GPUjobs/CPUprofiles alive. Currentturnprogress implemented/testedactualprofileledCPUchange;root/appunchanged;goalactive.

Current continuationverifiedwait: exactcityrevisionrelease84354 repeatedlylive (finalnew_bark_3d stageobserved), norestart/sourceedit; twofrozenhashesreverified. Rootcurrentlib/citysaved city-revision-root-before withhashreceipt forlaternarrowmerge preservingconcurrentpanoramaemission. Prepared city-revision-new-bark-{day,night}-compare.py withfull2560x1600High/SSAO/camera0,7.5,45,0/NewBark13,6/hour12or22/30s/LLRR/bothorders, to runonlyAFTERroofgate+compilerterminal. Notrunorclaimed. Source/appunchangedthiswait;goalactive.


Cityrevisionrelease84354terminal0 in9m52s; immutable c139da139dd81b3af3d2328c43463ad43a87bf8291b46b0b2e9dba4c2caa9e37. Fullroof/untracedbothorder69023terminal0: firstcandidate850updates68.78/84.77msmedian/p95,walking67.9445/82.040,callbacks14.075/s; firstcontrol791updates74.92/105.40,walking68.794/90.012,callbacks12.882/s. Reversecontrol865updates70.20/110.73,walking56.3765/145.217,callbacks13.920/s; reversecandidate782updates73.65/99.59,walking75.536/108.190,callbacks13.017/s. All256rows/fourorigins/onebuild/pendingfalse/no pollerrors/exactfinalpositionequal. Mixedoverall andmarkedreversewalkingregression: doNOTpromote. Restoredprivate lib/city exactbaselinewithcopyfile;candidatearchive+9tests retainedignored. NoNewBarkday/night runsneededforfailedroofgate. Useraskedif13FPS; answered70ms~14updates/s, callbacksnotconfirmeddisplayFPS,andstilltooslow;continueauthorized.

New actualsampleledvisibilitylead: Bevy0.20CPUculling query includesnonrenderOccluderGroup parents dueVisibility+GlobalTransform, withoptionalVisibilityClass; setsViewVisibility evenwhenclassNone. Currentgroupscount167987. NoCpuCulling skipsCPUfrustum/reset sweeps anduseschangedInheritedVisibility-onlyupdate; it isnotinheritedbychildren. Private candidate insertsNoCpuCulling ONLYonprovennonrenderOccluderGroup owners atspawn; leaves keepordinaryCPU/GPU/frustum/layers/shadows. No shader/mesh/cameraquality changes ordevice-support dependency fornonrendercontainers. This differsfromfailedfine16GPUcullingtrials: zeroGPUdraworleafpolicychange. Savebase container-cull-baseline/occluder_fade.rs; nexttestproductionvisibilityinheritance/overrides pluscurrentoccludersuite. Root/appuntouched;goalunmetactive.

Container visibility71594terminal0:67passed0failed2.47s. New nonrender_fade_owners_skip_cpu_culling_without_changing_mesh_visibility usesactualAsset/Mesh/Visibility/Transformplugins andrealcameraFrustum, provesownerNoCpuCulling+noMesh, allmeshchildrenwithoutmarker, normalvisibleleaf, hiddenancestorInherited/Viewfalse, explicitVisiblechildoverride,trueInheritedbutViewfalseaftermovingfaroutoffrustum. Wholeoccluderfadesuiteincludingregionalmotion/sleep/fade regressionspasses. Candidateonlyoccluder_fade.rs archivedcontainer-cull-candidate-source/hashreceipt. Privatecity/librestoredacceptedexact;root/appunchanged. NextreleaseanduntracedbothordersGPUgate. No lossless-render/FPSclaimyet.

Container-cull release28170live /tmp/geothite-container-cull-release-build.log. Freezeprivateinputs; onechangedsourcehash0fbce88479a26e639286c7fae871e27f0fe0c82ead349d71105b723a6d30b7d6. Onterminalsuccesssaveimmutablecontainer-cull-frame-client and runpreparedcontainer-cull-compare.py againstacceptedwater07eb7184... fullroof60sHigh/SSAO2560x1600bothorders. Currentturnprogresscompletedcitycomparison/rejectedmixedcandidate, implementedactualsamplelednonrenderCPUvisibilityskip and67regressionspassed. Root/appunchanged;goalunmetactive.

Container-cull release28170terminal0; frozenoccluderhashverified andcity/libexactacceptedbaselineconfirmed. Immutablecontainer-cull-frame-client SHA2960440df67d7b139555e65e56ffc20d03fae68ad2c99b9ca9c33a5a5b70a30b. Fullroof/untraced/bothorder80712live /tmp/geothite-container-cull-compare.log. No ownedcompiler remainsduringcomparison. Root/appunchanged. CurrentturnprogresscompletedreleaseandstartedactualGPU/framecomparison;goalactiveunmet.


Container-cull fullroof comparison80712terminal0. Firstcandidate899updates65.52/75.64msmedian/p95,walking62.962/76.808ms,callbacks14.973/s; firstcontrol794updates73.25/92.58ms,walking72.242/86.910,callbacks13.233/s. Reversecontrol687updates78.43/173.80ms,walking71.985/144.278,callbacks11.418/s; reversecandidate777updates78.31/110.51ms,walking54.3345/129.227,callbacks12.652/s. All256walkrows/fourorigins/oneinitialbuild/pendingfalse/no pollerrors/exactfinalpositionequal ('-44', '-36', '-1408.000', '1152.000', '8', '12'). Updatecountimproves13.2%/13.1%, walkingmedianreduces12.8%/24.5%; tailimprovesbothorders, reverseoverallmedianflat. Provisional CPU/performance gate passed; NOTglobalfluidity/displayFPSclaim. Nextactualpairedvisuals, normalasynchouseexit, mandatorycontroller/MCP/stdio/rootchecks beforepromotion/install. Root/appstillunchanged.

Container acceptance jobsstarted afterframegate: pairedroof/NewBarkcaptures90635live (/tmp/geothite-container-cull-captures.log) viaimmutable2960440d...; privatecontroller/MCP/stdio sequentialsuite30078live (/tmp/geothite-container-cull-gameplay-tests.log) withabsoluteexternalTUIpack/headless/performance-visibilitytarget/wasm-dev/voxelopt0. Compiler contention permittedforvisuals, NOTforlatency/FPS; do normalasyncHouse1F6,6DDBBBBBBBBBB recordonlyAFTERsuitecompilerterminal. Inspecteachpairedstate receipt+3D/2Dimagesbeforepromotion. Roofupdatecounts13.2%/13.1%better, still~13-15updates/ssofullfluiditygoalunmet; notdisplayFPS. Bothcurrentjobsowned, noothergameclosed. Root/appunchanged;goalactive.

Pairedcaptures90635terminal0; inspectedcontainer-cull-{roof,new-bark}-{3d,2d}.png andbothstate receipts. Roofstate_preservedtrue/checksumframe983/hash3648450576/lcd983/oneChris,samepose/time; NewBarkstate_preservedtrue/frame148/hash3011848586/lcd151/fouractorssamepositions/facing/time. Alsoviewedacceptedwater-stencil-roof-ready-3d.png andwater-stencil-3d.png. Optimizationpreservesgoldroofcomposition, Chrisreadability/scale, distanttreepanorama/fog; NewBarkhouse/lab/shore/foliage/water/NPCs/materials/shadows unchangedvisually. Existingstylized3Drooftints differfrom2Dsourcebutarealreadyinaccepted3Dbaseline; noartretint/detailcutintroduced. No visibledefectfromparentcullingpolicyfound; sameappearancegatepassed. Mandatorygameplaysuite30078repolledlive; houseexitlatencyrecordmustwaituntilownedcompilerterminal. Root/appstillunchanged;goalactive.


Mandatorygameplay30078terminal0:controller8passed1preexistingignored11.54s (battlePP/commands,Mom/NewBarkgate,menus,scriptownership included), MCPproductionmovement/Start1passed0.82s, stdioClean1passed1.39s. /tmp/geothite-container-cull-gameplay-tests.log. NormalasyncHouseexit91870terminal0:323presentations,136House/187Town,0Townpending/allTownreadyfromfirstTownframe,no droppedtrace/scenery/battleerrors. HOWEVERmovementframe35takes1056.933ms afterfirstTownready, capturetimestamp17to18gap1.1116s. Do notcalltransitioninstant orclaim3secondproblementirelyfixedfromterrainreadyalone.

Diskcrisisduringcapture:414MiBfreeobserved. Nootherprocess/cachetouched. Removed5exactSHAverifiedrejectedownedexecutablecopies(runtime-sharing,subregionv1/v2,cityrevision,cacheorder) aftertheirtrialsfinished, keepingarchives/receipts/logs. Thenverifiednoactiveownedcompilerusingperformance-visibilitytarget; removed36obsoleteMachOtestexecutablesinOWNperformance-visibility/wasm-dev/deps, keepinglatestTWO percrystal_bevy/crystal_voxel_view/geothite, includinglatestpassing67tests/controller/MCP. Logical7.13GiBreclaimed;df13GiBfreeafterOSrecovery. Cleanupreceiptignoredcontainer-cull-owned-artifact-cleanup.json; no packs/saves/source/otherbuildcachesmodified. Rejectedimmutableclientpathsnowintentionallyabsent; doNOTrerunoldcomparisonsblindly.

Cleanasyncexitrerunneededbecausecriticallylowdiskmaytaintfirstcapturehitch. Preparedsamecommand/outputprefixcontainer-cull-house-exit-clean, preservefirstrecordforcomparison. Noownedcompiler/GPUjobactiveonlaunch. StillnotclaimhitchcauseorordinaryFPSfromPNGcapture. Root/appuntouchedpendingtransition/roottests/prodbuildinstall;goalactive.

Cleanhouseexit31658terminal0 after13GiBfree:328Townpresentations,0pending,319Townwalkingrows,median/p95/max21.704/63.356/257.854ms. FirstTownrows33:13.550msFadeOut63,34:257.854msFadeOut63,35:10.175msFadeIn255,36:79.554msFadeIn255. OriginalcriticallylowdiskcaptureTownmedian40.105/p95114.204/max1056.933; donotattributeallimprovementtocodedelta. StillonlycapturediagnosticsnotordinaryFPS/absoluteUIlatency. Needacceptedwaterbaselinefreshsame13GiBconditions fortransitionhitchcomparison. Preparedcontrol-clean script onlyclientchangeswater-stencil07eb7184.../uniqueoutputprefix, hashverified; pack/camera/house6,6/noon/High2560x1600/DD10B/10s/normalasyncunchanged. Root/appstillunchanged;goalactive.

Control-clean6130terminal0:374Townpresentations,0pending/fromfirstTownready, median/p95/maxTownwalking21.548/57.526/155.586ms. Candidateclean21.704/63.356/257.854ms; normalframesroughlyequal, maximumcaptureframevariable. No houseexitspeedupfromNoCpuclaim; firstdrawhitch155-258ms remainsfuturetarget, PNG/GPUreadback/clockphaseconfoundabsoluteUIlatency. Visuallyinspectedcandidatecleanframes0/18/33/164: correcthouse,firstTownunderexistingfade/fullbuild,clearsettledTown/final,Chrisreadablebuildings/forest/water/NPCsretained. Neithercleanrecordhasterrainpending. Existingprefetchcontractpreserved; fullfluiditygoalstillunmet.

PromotedONLYNoCpuCullingnonrenderowner insertion+productionvisibilityregression intoROOToccluder_fade.rs afterassertingrootdifffromprivatebasewasONLYoneblankline; preservedthatblank. Rootbeforearchived container-cull-root-before/occluder_fade.rs. Privatecandidatehashunchanged. No citymetadata/Arc/BVH/index/GPUdraw/sourceartchange promoted. Runrootoccluderregressionsnow; appnotyetinstalled.

Rootoccludertests88674live /tmp/geothite-container-cull-root-tests.log, performance-visibilitytarget. Production64681live /tmp/geothite-container-cull-production-build.log, privatefrozenmanifest/native-releasetarget/jobs2, binscrystal-bevy+verify_crystalpack+bundle_open_models+bundle_open_people/featuresvoxel-view,desktop-app. RootNoCpu/testsnowpromoted, appSTILLoldc1b1029e...untilbothchecks+productionterminal. Beforeinstallfreshallsavehashes/installedapp+packexpectedhashes/noinstalledappactive, preserveothersprocesses; strictcodesign,outsidecwdcheckinstallation andsigned-vscompiledMachOsection verification afterinstall. No fullgoalcompletionclaim; firstdrawcapturehitchandglobal14FPSrooftopremain.

Current continuation verified both builds 88674/64681 still live. Root/private Cargo.lock byte-identical, both Bevy 0.20; fresh dependency compilation observed. Disk externally changed from 13 GiB free to 455 GiB free without additional cleanup by this agent; no attribution of external change. Own cleanup remains five rejected binary copies and 36 obsolete owned test executables. Do not restart builds based on space changes: poll their exact handles until terminal.

Offline first-Town trace evidence: candidate clean render.frame 196.018 ms/render.submit 179.735 ms, internal render.graph 20.691 ms/graph_submit 19.598 ms, largest main envelope 59.162 ms. Accepted control render.frame 122.515/render.submit 116.270 ms, main 29.221 ms. These are CPU envelopes, not exclusive costs or GPU durations. Terrain was ready throughout. The hitch associates with rendering/submission rather than pending terrain; precise cause remains unproven. Source search finds explicit indefinite device polls only in GPU tests, not the capture controller. After compilers and installation finish, the existing code-identical accepted-symbolized client can support a bounded OS sample covering the house exit (attach when recording starts; first Town near 0.8 s), to distinguish submission, cleanup, upload or compilation stacks. No new product timing feature or toggle is needed. Full fluidity and first-draw latency remain unresolved. Root/app unchanged this continuation; goal active.


Non-render fade-owner CPU culling optimization is now installed. Root session 88674 passed all 67 occluder regressions (0 failed, 14.56 s), including real camera visibility, hidden ancestor/explicit child override, and ordinary mesh frustum culling. Production session 64681 completed successfully (47m53s); install session 26085 and verification 85248 exited 0. Installed executable SHA256 `1f36ef1877996012cd93d63ae6c81b6550ad4f62c2ff03c42c12abb692b0e0f2`; compiled unsigned `8353a4cf55efbfcef9a4d076bd71912ac182d98d4438484ee405bac34e1e22db`. Strict deep codesign, signed distribution equality, all 15 named Mach-O section hashes versus compiled executable, and installed --check-installation from /tmp passed. Fresh before-container-cull-install.json and container-cull-installed-verification.json prove unchanged source/bundled pack and all external saves, rechecked after installation verification. No visible launch or other process termination.

Bounded first-entry stack sample 72882 exited 0 on accepted code-identical symbolized baseline, owned PID 45378. It intentionally overlapped the still-running production compiler to gather named stacks; its timings are diagnostic only and include screenshot readback/PNG work. Shader processing and render pipeline creation occur in the transition sample. Trace submits 31 destination pipeline variants from 27.275s through 30.865s; first Town publication is 28.774s, so the existing warmup queue continues after entry. Terrain was ready at first Town. This does not prove which compilation or submission work causes a normal-play stall. Ignored accepted-entry-stack-analysis.json records evidence/limitations. Next work should identify late destination pipeline signatures or other first-draw costs before implementing another optimization. Dense scene remains roughly 13–15 updates/s in prior QA; full fluidity goal is still unmet.


Next private candidate raises pipeline warmup submissions from 2 to 8 per frame, retaining all materials, keys, asynchronous compilation and draw policy. Archives pipeline-burst-{baseline,candidate-source}; pipeline-burst-source-receipt.json verifies frozen pipeline_preload.rs and accepted NoCpu occluder_fade.rs. Release QA build88511 live, /tmp/geothite-pipeline-burst-build.log. On success save distinct pipeline-burst-frame-client and verify source receipt. Prepared pipeline-burst-house-compare.py uses accepted NoCpu immutable control2960440d... in candidate/control/control/candidate order, normal async House1F6,6/noon/High2560x1600/DD10B10s, checking Town readiness, early-entry tails, final positions and late-preload counts. Run only after owned compilers/checks are terminal. Budget change not promoted or accepted.

Current AGENTS.md Rust style requirements and official Google Rust guide read. Formatted only the new visibility regression; exact comparison against rustfmt output proves this and the retained preexisting blank line are the only root/private occluder differences. Private frozen inputs remain unchanged. cargo fmt --all -- --check exits1 on broad existing formatting differences (first offset-allocator); Clippy29343 exits101, with four deny-level expressions confirmed in pre-trial snapshot: bad bitmask in core multiplayer and approximate constants in scenery_finish/pine/shore_ecology. Logs /tmp/geothite-container-cull-{fmt-check,clippy}.log and ignored style-check-results receipt. Unrelated art/source remains untouched. Installed NoCpu optimization still verified; goal active.


Pipeline-burst build88511 terminal0 (9m02s), immutable candidate SHA cac38643f39da1cb1b86026716dcaca8f75a44c8e085cb3860f12e3a0b9c3a21. Four-run normal-async comparison64876 terminal0: first candidate/control initial-eight Town max45.562/106.578ms, whole-route max119.922/207.627ms, whole-route p9590.452/90.888ms; reverse candidate/control initial-eight max199.170/39.665ms, whole-route max246.297/212.688ms, whole-route p95167.650/164.685ms. All runs endTown13,6 with active terrain/no pending and fade0/phaseNone. REJECT eight-pipeline submission burst: reverse entry and whole-route tails regress. Private pipeline_preload.rs restored exact root baseline with fresh mtime; root/app never received budget change. Trial verdict/receipts/source archives retained. SHA-verified rejected immutable executable and excess capture PNGs removed; first/arrival/final captures and CSV/trace results retained in ignored evidence. pipeline-burst-frame-client intentionally absent.

Attribution correction: frame.publish_world is source publication, not proven first displayed Town image; post-swap speculative shaders may include return-House variants. Neither the 31 submissions nor post-source-Town timestamps alone identify the exact blocking Town shader. accepted-entry-stack-analysis.json and verdict record this caveat; avoid causal claims from those counters. Next CPU target from earlier named samples/source: sync_interior_cutaway performs two full material scans every frame (lighting then capsule). Combining them can preserve exact material state while reducing traversal; not yet implemented or performance-accepted. Installed NoCpu CPU optimization stays SHA1f36ef18...; full fluidity remains unmet.


Cutaway material traversal fusion implemented privately, not promoted: src/interior_cutaway/material_sync.rs collects lighting/capsule changes in one immutable Assets traversal and mutates only affected handles, replacing the prior two full scans in sync_interior_cutaway. Base-terrain final full-cutaway updates remain unchanged. New tests assert exact cave exits/sky/darkness, actor capsule retention, fade and unrelated reveal components, late same-ID replacement, and no Modified events for idle/actor-only motion. Added leaf module and formatted it with pinned rustfmt; baseline files lib.rs/interior_cutaway.rs archived in cutaway-fusion-baseline; three candidate files archived and frozen by cutaway-fusion-source-receipt.json. Unit session3694 live /tmp/geothite-cutaway-fusion-tests.log, private manifest/performance-visibility/wasm-dev opt0 voxel/jobs2/filter interior_cutaway::. Freeze private inputs until terminal. After pass run existing cave_reveal_tracks_supported_player_and_preserves_actor_and_fade_materials regression, then release QA build and immutable cutaway-fusion-frame-client. Prepared cutaway-fusion-compare.py uses accepted NoCpu control2960440d... and full2560x1600High roof9,13/noon/60s UUUUDDDD in candidate/control/control/candidate order. No performance acceptance, paired visuals, gameplay gates or installation claimed for fusion yet. Root/app still accepted NoCpu; goal active.


Cutaway fusion unit3694 terminal0:4passed/0failed in0.01s (two new material-state/replacement/event regressions plus existing capsule tests). Existing renderer_tests::cave_reveal_tracks_supported_player_and_preserves_actor_and_fade_materials also passed1/0 in0.00s; full production system verifies supported player, retained fade, actor capsule, darkness/daylight and disabled-state behavior. Frozen3-file receipt verified. Release QA build78873 now live /tmp/geothite-cutaway-fusion-build.log, private manifest/native release target/jobs2/location-tester/example new_bark_3d. Freeze private source/assets until terminal. On success save immutable cutaway-fusion-frame-client with hash receipt and run prepared fullroof both-order cutaway-fusion-compare.py after owned compiler jobs are terminal. No gameplay/performance/visual/install acceptance yet for fusion; root/app still accepted NoCpu.


Cutaway fusion release78873 terminal0 (16m03s); three frozen candidate source hashes verified. Immutable cutaway-fusion-frame-client SHA0917606130717c31cd1fe05fc8f661916b005279b78d52f3ca6e28600da2a252, binary receipt saved. Fullroof both-order untraced comparison11488 now live, /tmp/geothite-cutaway-fusion-compare.log; candidate-first confirmed underway. No owned compiler remains during measurements. Retain frozen QA source/assets until comparison terminal. Existing material/capsule4-test suite and whole-system cave regression1-test passed; performance/paired visual/gameplay/installation acceptance still pending. Root/app remain installed accepted NoCpu.


Cutaway fusion fullroof comparison11488 terminal0. First candidate639updates80.13/196.11ms median/p95, walking79.322/197.335; control728updates78.24/144.21, walking76.3135/138.423. Reverse control821updates72.66/116.08, walking75.568/114.062; candidate801updates69.72/125.64, walking67.5925/119.65. All256walkingrows/fourorigins/onebuild/no pendingterrain/no GPU pollerrors; exact finalRoof8,12/origin-44,-36 matches. REJECT fusion: lower update counts and worse tails in both orders, mixed medians. New unit4 and existing cave-system1 regressions passed but do not prove gameplay speedup. Restored private lib.rs/interior_cutaway.rs exact archived baseline with fresh mtimes; removed only newly created private leaf/directory. Root/app never received fusion. SHA-verified rejected immutable executable removed; cutaway-fusion-frame-client intentionally absent. Source archive, receipts, CSVs/validated warmup evidence and verdict retained ignored.

Next attribution: resolution-attribution-compare.py runs the SAME accepted NoCpu executable2960440d... at1280x800/full2560x1600/full/small, High/SSAO/same16:10camera0,7,5,0/Roof9,13/noon/60s UUUUDDDD. Existing headless measurement validates readback before timing, drains30frames, no per-frame screenshot capture. Session6944 live, /tmp/geothite-resolution-attribution-compare.log; no owned compiler remains. Checks identical route/build/readiness and collects walking/overall frame times. This is diagnostic output-size attribution, not a proposed shipping resolution/quality change. Finish both orders and inspect composition before inferring whether pixel work dominates. Source/assets frozen until terminal.

GPU source recheck: Bevy0.20 per-pass GPU timestamps unavailable on current Metal inside-pass/encoder feature set; existing render_metrics guard already handles this, no instrumentation changes made. Shared grove materials already backface-cull; opaque surface fade discard guards are already specialized, stock alpha_discard is guarded by MAY_DISCARD. Uniform guards and selective depth prepass were previously tested/rejected; no repeat shader candidate justified from that source inspection. Full fluidity goal remains unmet; installed NoCpu unchanged1f36ef18...


Resolution attribution6944 terminal0. SAME accepted NoCpu binary: small-first992updates58.25/82.80ms median/p95, walking58.6985/93.712; full-first719updates76.08/123.96, walking72.7075/82.252; reverse full797updates73.62/100.78, walking74.5855/115.093; small1052updates56.26/65.71, walking54.1395/75.122. All256walkrows/fourorigins/onebuild/no pending, exactfinalRoof8,12/origin-44,-36. Inspected first small/full validated warmups: same composition, visible roof/player/forest extent/camera scale, differing output sampling. Quarter pixel count reduces overall median about24% both orders; most frame cost persists. This separates output-size-dependent work from remaining cost, not CPU vsgeometry vsfixed-resolution shadow work. No quality/resolution change proposed or shipped. Ignored resolution-attribution-verdict.json records evidence. Next inspect stock GPU frustum culling availability/policy on ORIGINAL opaque leaves, distinct from previously failed fine16+occlusion compositions; implementation unproven. Goal remains unmet, installed NoCpu unchanged.


New private native GPU-frustum candidate on ORIGINAL combined opaque leaves (no fine subdivision, HZB, shadow bridge, layer change, geometry/material/pipeline change). Parent occluder_fade.rs wires native-only occluder_fade/gpu_frustum.rs. A per-App shared capability flag requires stock GpuPreprocessingSupport::Culling AND RenderMeshInstances::GpuBuilding; sync assigns NoCpuCulling only to bounded, frustum-cullable OpaqueObjectLeaf entities with actual AlphaMode::Opaque VoxelMaterial. Fading/sorted and base-alpha surfaces retain CPU policy. Active direct-draw Camera3d views intersecting layer31 restore only markers owned by this module; unrelated capture layers and foreign NoCpu markers are preserved. Unsupported/no-render apps remain CPU. No new player/env toggle. Baseline parent archived native-frustum-baseline; final2-file source archived native-frustum-candidate-source-v2 and frozen receipt.

Initial policy63377 passed1; preliminary full suite20801 passed68/0 with1GPUignored. Tightened actual-material v2 suite11428 passed68/0,1ignored in2.55s. Explicit native GPU proof25068 passed1/0 in1.20s: production VoxelMaterial/shaders, windowless64px image, ready stock indexed frustum compute pipelines and main+shadow bind groups, real GPU collector membership, hidden parent removal/explicit-visible child, and direct-view CPU fallback. This proves selected/prepared paths, not elapsed GPU duration or indirect parameter count readback. Bevy teardown command-queue warnings occurred; no source logging added. Test verification receipt saved.

Release QA build56158 live /tmp/geothite-native-frustum-build.log, native release target/private manifest/jobs2/location-tester/new_bark_3d. Clippy also started in performance-visibility, log /tmp/geothite-native-frustum-clippy.log (record exact returned handle next); prior preexisting errors outside this change documented. Freeze private source/assets until jobs terminal. On release success save immutable native-frustum-frame-client/hash and run prepared native-frustum-compare.py against accepted NoCpu2960440d... in both orders, original fullroof conditions. Normal async doorway/paired same-state3D2D/gameplay gates/install still required if perf succeeds. Root/app unchanged; goal active.

Clippy exact session84279 confirmed live alongside native-frustum release56158; no restart or source edits.


Native-frustum lint84279 terminal101: same four preexisting deny errors plus3 introduced warnings (two complex Query types,8-arg sync). To fix candidate before measurement, explicitly cancelled ONLY owned release56158 using exact private cargo command/descendants PIDs62684,63161; handle terminal143. This was a required source correction, not a restart after observation timeout. No other build/game/process touched. Replaced new-module glob imports with explicit imports and grouped query state in named SceneryMeshes SystemParam plus query aliases; culling/material/fallback logic unchanged. First v3 test51318 failed missing PluginGroup test import and one unused test import; corrected before retry. Final source archived native-frustum-candidate-source-v3 and active receipt refreshed; v2 receipt retained separately.

Final v3 suite27129 terminal0:68passed/0failed/1GPUignored2.47s. Explicit offscreen GPU proof99627 terminal0:1passed/0failed1.15s. Clippy20572 terminal101 ONLYfour preexisting errors; zero diagnostics in new gpu_frustum.rs. All-target lint coverage incomplete because baseline errors stop it; not claiming globally clean. New leaf standalone rustfmt --check passes; private cargo fmt --all check7849 has broad preexisting differences. native-frustum-v3-test-verification.json records exact source/tests/limitations. Release54528 live /tmp/geothite-native-frustum-v3-build.log, same private/native-release/location-tester/jobs2/example. Freeze source/assets until terminal; save immutable native-frustum-frame-client only after success/hash verification, then run prepared native-frustum-compare.py after all owned compilers terminal. Root/app unchanged; no perf acceptance/install claimed; goal active.


Native-frustum final release54528 terminal0 (8m02s), frozen2-file receipt verified. Immutable native-frustum-frame-client SHA0acf66213c2e78475ea2148f0b4799b77318870241e536ac26fdc9ac1c376463, receipt saved. Fullroof initial comparison80744 terminal0: candidate1046updates54.70/68.73ms median/p95, walking53.538/58.202; control660updates80.09/197.02, walking70.8005/160.533; reverse control629updates86.60/180.06, walking95.059/207.82; candidate555updates99.73/192.26, walking105.317/209.397. All256rows/fourorigins/onebuild/no pending/no pollerrors/exactfinalsameRoof8,12. Viewed first candidate/control validated warmups: roof/player/forest geometry, scale, materials and visible shadows match compositionally; not a paired-state visual acceptance yet.

HOLD rather than accept/reject on this mixed quartet: strong first pair gain but reverse regression, with chronological slowdown across all four runs. Initial result receipt native-frustum-initial-comparison.json. One additional reversed counterbalanced replication28976 now live, /tmp/geothite-native-frustum-recheck.log: control/candidate/candidate/control, identical original fullroof60s settings, native-frustum-recheck-* outputs. Records deidentified host-start loadavg and aggregate CPU percentage via existing OS commands before each run, outside measured interval; no game instrumentation or defaults added. This tests unresolved order/load sensitivity; do not infer host load caused changes without evidence or accept first pair alone. No owned compiler active. Keep source/assets frozen; root/app still accepted NoCpu1f36ef18..., goal active.
