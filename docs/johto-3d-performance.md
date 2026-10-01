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
real wall time. Known software adapters use baked contact grounding instead of
rerendering the broad terrain halo into a dynamic shadow map; hardware adapters
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

Native `CRYSTAL_CUTAWAY_DIAGNOSTIC=zero-radius` and `omit-uv1` are explicit
comparison controls; unset/`normal` preserves the reveal, and the browser always
uses normal rendering. Their controlled runs did not establish capsule arithmetic
as the main cost. Disabling reveal is not an optimization used by the game.

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
