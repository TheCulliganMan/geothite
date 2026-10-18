# Growlithe and Arcanine per-limb support packing

Current camera/follower follow-up: The shared physical following-distance change now separates Arcanine’s body
from the trainer in the reviewed Route36 views. Sign and foliage overlap
still occur during traversal and remain open.
See [physical companion spacing](pokemon-following-distance.md) for the
current checks, captures and remaining limitations. Earlier review records
below retain their original evidence and conclusions.


This revision corrects walking attachment without changing the sculptures,
materials, battle weights, skeletons or animation keys. The prior exporter
combined paw creases, leg stripes and adult ankle fur/tufts across all four
legs. Production walking IK assigns whole runtime primitives to one leg, so
such a combined support-detail primitive can follow the wrong paw. A new
stored-GLB regression fails against that prior cross-leg packing.

The exporter now keeps each support category separate per limb. The walking
regression checks that every leg, complete paw, paw crease, leg stripe and
adult ankle detail belongs to its corresponding leg's IK tree. Complete paws
and their fitted creases share the same rigid foot carrier. Non-support face,
body, mane and tail ownership remains unchanged. All four legs retain ground
contact and bounded geometry throughout 64 sampled locomotion phases.

| Species | Runtime primitives | Accessors | GLB bytes | Vertices | Triangles | Canonical joints |
| --- | --- | --- | --- | --- | --- | --- |
| Growlithe | 38 | 209 | 713,180 | 16,302 | 26,140 | 18 |
| Arcanine | 46 | 236 | 845,220 | 19,539 | 31,404 | 18 |

The existing geometry, primitive, accessor, joint and byte limits are unchanged.
Independent old/new decoded models retain identical logical stored f32 geometry
pins, source order, normals, materials, indices, skin weights, hierarchy,
inverse binds and exact clip keys. Their sampled battle positions and normals
also match exactly across idle, attack and hit. Only runtime packing, its
source ranges and corresponding buffer organization change.

Current packed geometry pins:

- Growlithe: `93977ea316f368e22fd7a1061f565e63ca5fd01264a654a6b628f2d65961ab99`.
- Arcanine: `b8b53169922d6cb2ea77f87da6f84f68bb50279d875021a4bec7b884a9382b7e`.

Current full GLB SHA-256:

- Growlithe: `7992322e92b22fed3e7126566fb07353a2fe06149008cfd4123311ee699b9e7c`.
- Arcanine: `67bc957266807278d0401e5f6efbac5f2e79ccc406bf9ad5d1429595433c4fa1`.

All 18 Python anatomy/skin/packing checks pass in 47.518 seconds. All four
Rust GPU/original-size, battle-support and per-leg walking checks pass in
2.33 seconds. Background Blender reproduces both canonical GLBs byte for byte.
The final shared registry check passes in 104.93 seconds and source-cue/GPU
playback timing passes in 141.42 seconds. Native examples were rebuilt in
1 minute 16 seconds with `crystal-assets` at optimization level 2. Every launch
uses explicit headless image capture, preview mute and normal coloration.
Preflight requires the headless parse guard, mute support and exact current
compressed GLB payload in each executable; identities remain unchanged after
both batches. No windows, tabs or application activation are used.

The production world actor path constructs `SkinnedMesh` from the world rig,
advances distance-based locomotion and publishes each sampled joint transform
in `new_bark_actors.rs`. The Route36 review starts at `(20, 8)`, hour 12,
and holds the ordinary `DUDUDUDU` inputs. Its 256-frame trace stays on Route36
and visits column 20, rows 5 through 12. Moving screenshots 00, 01, 04 and 05
for each species were inspected inline. Clear walking views show supported
stride poses with per-leg markings, paw details and adult ankle fur attached
to their own limbs. Screenshot readback is asynchronous; intermediate frames
are additional native movement evidence rather than exact-frame 2D pairs.
The independent per-leg binding and 64-phase Rust checks supply the numerical
ownership, support and bounds assertions.

Both traversals end beneath a fruit tree, obscuring parts of the models.
Those paired endpoints are retained as context, not accepted as clear model
views. Additional settled 3D/2D pairs at the original position use camera
`3 2 50 0`; they show the unchanged scale, matte materials and complete body
silhouettes. The walk cameras are `5 2 30 0` for Growlithe and `3 2 40 0` for
Arcanine. All six pairs were inspected and preserve their authoritative state.
The standing Arcanine view and some traversal frames still overlap the trainer
or a sign. Follower spacing and broader moving-world visual acceptance remain
open; this correction does not change the model size or environment geometry.

| Pair | State frame | State hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- | --- |
| growlithe world | 3321 | 3797884665 | 3390 | 58.654868333 |
| growlithe standing | 1022 | 3272990513 | 1022 | 17.375843208 |
| growlithe battle | 3592 | 3193325102 | 4327 | 76.988377001 |
| arcanine world | 3028 | 3624226619 | 3077 | 53.203353167 |
| arcanine standing | 1143 | 3489880886 | 1143 | 19.415537626 |
| arcanine battle | 2713 | 2408993628 | 3381 | 58.56206925 |

The normal Route29 walking-encounter captures retain named modeled actors
with `fallback=None`, zero support offset and `accepted=true reason=None` at
the original pack heights. Whole-body support, admission and omission gates
remain unchanged. Stage counters are candidate observations, not percentages.

| Player / opponent | Pack height (m) | Uniform scale | Reference height | Support Y | Stage checked / unsupported |
| --- | --- | --- | --- | --- | --- |
| growlithe / SENTRET | 0.7112 | 2.094089 | 0.9 | 0.0 | 54 / 1 |
| arcanine / SENTRET | 1.9050 | 3.8832695 | 1.3 | 0.0 | 210 / 133 |

Executable SHA-256:

- render_at_location: `1d0bad0440779161ec1b289cc0fce8d6ff77a24eece48cb88ba34ba249bf214a`.
- immersive_battle_3d: `c74f3300ea4af8adf0dcaa3922eae1bb0f33361b667a4b9853e3eac287b6deec`.

Final ignored evidence uses the `target/pokemon-native-review/canine-walk-`
prefix: selected native PNGs, paired-state receipts, movement CSVs, build/test
logs, exact old/new battle comparison and executable identities. Temporary
asset copies, export duplicates and helper scripts are removed. Browser review
remains open. Shiny checks are skipped; grass, AA and filtering are untouched.


---

## Prior sculpture and first-skin record

# Growlithe and Arcanine sculptures and skins

The original `tools/fire_canine_sculpt.py` recipes replace the shared generic
Blender dog geometry. Both retired meshes had 1,986 vertices, 2,796 triangles
and 48 primitives, with only a small head-height change separating them.
The new assets deliberately spend more of the existing geometry budget on
species-specific anatomy while reducing runtime primitive counts.

| Species | Logical solids | Runtime primitives | Vertices | Triangles | Joints | Accessors | GLB bytes | Reference height |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Growlithe | 57 | 32 | 16,302 | 26,140 | 18 | 190 | 718,636 | 0.90 |
| Arcanine | 81 | 34 | 19,539 | 31,404 | 18 | 206 | 874,164 | 1.30 |

The previous normalized heights remain unchanged, as do the 20,000-vertex,
40,000-triangle, 64-runtime-part, 64-joint, 256-accessor and 1 MiB limits.
The canonical files are `growlithe.glb` and `arcanine.glb`, produced by
`tools/fire_canine_glb.py`. No external mesh, rig, clip or reference bitmap is
included. The obsolete generic dog builder explicitly rejects both names;
the pure exporter handles them before Blender batching. The geometry is
original parameterized work within this repository.

Growlithe has a compact round orange trunk, shorter supports, rounded fitted
eyes with brown pupils and glints, a low swept cream forelock, cream chest
ruff and curled fluffy tail. Arcanine has a longer guardian trunk, taller
supports with bent hocks, a full cream ruff framing the orange face, focused
almond eyes, a swept forelock, feathered cream ankle fur and a broader curled
plume. Both keep rooted pointed orange ears and dark insets, cream snouts,
black noses, closed smiles and supported lower mandibles.

Tiger markings are closed thin layers clipped from the actual body and leg
triangles, with correctly rotated side and dorsal projection frames. They
follow the stored skin rather than protruding as fins or spanning curved
surfaces with raised chords. The same construction supports the fitted
forehead blaze, lips and paw creases. Parent-derived cap normals follow the
stored surface; creased buried walls keep their actual outward orientation.
Eye and ear layers use adaptive fitted contours. All four complete pads
reach the floor, with two fitted toe separations per pad. Every cream fur
tuft has an occupied root inside its real parent volume.

The first software review exposed an overly severe puppy gaze and an adult
ruff that left the orange chest uncovered. Rounded puppy eyes, a lower
forelock, a forward adult ruff and outline-oriented fur tufts improve the
silhouettes without changing the physical-size path. Raising the lower jaws
seats them against the snouts. Small-fur sampling reductions keep the adult
within the existing budget while retaining the denser face and plumes.

Both rigs have identity floor roots, body/neck/head articulation, two ears,
a closed lower-jaw carrier, four supported limb fields, three monotone-height
tail carriers, a main ruff carrier, a forelock and two side-ruff carriers.
Complete paws and adult ankle fur stay fixed in battle. Leg stripes lie in
the rigid middle portions of their corresponding limbs. All fitted face,
ear, body, back and paw markings share the appropriate parent carriers.
Idle lasts 3.2 seconds; attack and hit are one-second rotation-only clips
whose timing belongs to the production source cues.

Logical stored-buffer geometry pins:

- Growlithe: `7b296f21adc1228824b647f6dabb74d2d87403ac0b70947aef11dd6031b3023e`.
- Arcanine: `c045e509afaec7694872924d7f56edf85fc0ff5fd260a100a6777861ee265471`.

Packed geometry pins:

- Growlithe: `ff9ffd4264d0d7919944c2a6c483434f015d9111b377415a864a3896cc7a51ac`.
- Arcanine: `c622250351727cfb9a37e93dac844556013708c1c253e3fc6be9147957ca4d8d`.

All 17 Python pair checks pass in 41.635 seconds. They independently read
stored geometry, hierarchy, inverse binds, weights, clip keys and source-part
ranges; check closed outward volumes and actual normals; prove anatomical
contacts and four grounded pads; articulate every non-root carrier; retain
real roots and uncrossed limbs, fur and tails through every clip; test fitted
vertices, edges and face centers against the actual posed parent; limit
marking standoff; and check winding, edge strain, floor contact, neutral
endpoints, budgets and strict-reader mutation rejection. All four Rust GPU,
battle-support, bounds, independent-carrier and four-foot walking checks pass
in 2.31 seconds. Background Blender dispatch reproduces both exact canonical
GLBs. Final neutral, oblique, rear, idle, attack and hit software views were
inspected without opening an application window.


## Production review

Both shared registry and source-cue timing regressions pass in 114.81 and
115.57 seconds. The final native build completes in 1m 22s. Every native
capture uses explicit headless/image-target controls and mute, after checking
both executable guards and exact embedded compressed GLBs. Canonical assets
and executable identities remain stable through the captures. All seven
runs are entirely offscreen; six selected 1000×700 3D/2D pairs were inspected.

Growlithe's Route36 first-person view uses ordinary `0 0 20 1` controls and
keeps the full puppy in frame with grounded front paws and readable muzzle,
eyes, ears and forelock. The elevated view shows its compact stature beside
the trainer and its connected striped trunk and curled tail. Its Route29
GROWLITHE/SENTRET battle shows both complete models with correct HUD names,
no fallback and zero support offset. Terrain admission succeeds at the
original 0.7112 m pack height; the trace reports `checked=63 unsupported=0`.
The three captures complete in 52.89, 32.27 and 75.70 seconds.

Arcanine's close orbit initially used `5 0 20 0`; the trainer obscured its
chest and paws. The final repeat uses the existing `5 1 20 0` orbit controls
to separate the silhouettes and make the mane, eye, snout, tiger markings,
ankle fur and plume readable. Some trainer occlusion remains, and
large-companion spacing still needs world-view polish. The elevated side
view and Route29 ARCANINE/PIDGEY battle retain its full large scale and
visible grounded paws. Terrain admission succeeds at the original 1.9050 m
pack height, with zero support offset and no fallback. The trace reports
`checked=392 unsupported=410`; no scale, support, frustum, omission or
admission rule is loosened. Selected close-orbit, elevated and battle
captures complete in 38.44, 28.53 and 79.91 seconds respectively.

Every selected pair records `state_preserved=true`:

| Capture | State frame/hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| growlithe frontal | 2604 / 3478440453 | 2604 | 44.780355375 |
| growlithe elevated | 1627 / 1067035331 | 1627 | 27.651746334 |
| growlithe battle | 2887 / 32346622 | 3534 | 61.288220374 |
| arcanine frontal | 2010 / 1237056984 | 2010 | 34.09165575 |
| arcanine elevated | 1434 / 3919235847 | 1434 | 24.360891459 |
| arcanine battle | 2800 / 1096653092 | 3503 | 61.097962042 |

Exact canonical asset SHA-256 values:

- growlithe: `e07b7ca77d5d6bdf3d574a8261023d6834994b303d199d6b834d0cafa74e4919`.
- arcanine: `53bb1bc22d4702f55d973bc4bdf271ce896e8360175ef7610b332f1338d195a7`.

Browser and moving-world visual review remain open. Walking-intro captures
are not proof of GPU animation playback; the decoded clip, independent
matrix and Rust GPU checks supply their narrower evidence. Shiny checks are
skipped; grass height, AA and filtering are unchanged by this batch.

## Prior walking attachment audit

The later Nidoran packing audit exposed a gap in the walking checks: gait IK
assigns whole runtime primitives to individual legs. The prior exporter joined `Paw crease`, `Leg stripe`, `Ankle fur` and `Ankle tuft` categories across
multiple legs. Battle skin checks and floor/bounds assertions do not establish
that these combined details follow the correct walking limb. The current correction above splits those support categories per leg and
adds explicit per-leg ownership assertions. Standing/battle receipts above
remain evidence for those recorded views; moving-world acceptance stays open.
