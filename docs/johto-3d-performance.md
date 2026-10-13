# Connected Johto presentation and performance checks

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
