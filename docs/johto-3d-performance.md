# Connected Johto presentation and performance checks

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
including that new hidden-scene check. The real-pack publisher check and client
build are still running; no latency gain is claimed. The installed app remains the verified
map-space version below. The existing completed development executable was
saved in ignored `.local/early-destination-check/before` for comparison; this
trial has not been installed. Its targeted test output is in
`/tmp/geothite-early-destination-tests.log`. First-room readiness and cold-door
latency must both be checked before keeping the change.

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

The host's `bevy_shell/destination_preload.rs::publish` and renderer's
`destination_preload.rs::prepare` both wait for the current room to be active
and no longer pending. This deliberately prioritizes the initial room, but also
holds a known exterior's preparation until the room is fully playable. A native
initial-room/exterior overlap trial should measure both time to first playable
room and cold exit; publishing a hidden scene must never make it visible or
advance controller state. Existing warp-graph selection already finds New Bark
from both house floors, so no new dispatcher or source pack is needed. This
trial has not been implemented.

The installed Bevy 0.20 source also offers camera-level `NoCpuCulling` and
incremental GPU-visible entity storage. With static terrain transforms now
available, a device-supported camera-only trial may avoid repeated CPU frustum
work while preserving the existing CPU shadow/receiver filtering on meshes.
This needs actual GPU-preprocessing support, conservative animated bounds,
matched edge/cutaway/shadow checks and real frame evidence. It is distinct from
the rejected hardware-occlusion and direct-drawing prototypes; no new culling
policy has been enabled by this work.

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
