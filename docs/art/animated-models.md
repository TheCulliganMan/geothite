# Animated model assets

`crates/crystal-voxel-view/models/johto_characters/catalog.glb` is the canonical
human runtime asset: 75 distinct named scenes, each with the original sixteen
rigid joints, materials, and parent-relative bind translations. Its 977 shared
geometry accessor sets retain the exact runtime float32 positions and normals,
unsigned triangle indices, and RGBA colors. No welding, decimation, recentering,
normal repair, or arbitrary rescaling occurs. The characters remain rigidly
articulated; the file does not pretend to contain a deforming skin.

Each scene owns named `.walk` and `.run` animation clips with complete pelvis,
upper-body, arm, leg, and shoe movement. Clips use standard glTF translation and
quaternion channels, so a normal glTF viewer can play the full loops. Identical
animation sample accessors are shared across all characters. A normalized
one-second clip describes a complete distance-driven cycle; its duration is
not a prescription for game playback speed. The game keeps authoritative travel
and planted-foot IK in control. Its renderer selects and blends the clips using
traveled-distance phase; importing an asset never advances gameplay state.

`tools/human_locomotion.py` defines the production curves and includes checks
for exact loop endpoints, quaternion validity, foot contacts, and bounded
interpolation error. `tools/animated_glb.py` writes and reads the standard binary
format with Python's standard library. Runtime readers and tooling use the
checked-in GLB directly; a clean game build does not require Blender or
regenerated JSON. JSON rigs and a shared geometry table are authoring
intermediates, never a second tracked copy of the human catalog.

## Authoring and verification

```sh
# Original editable source generation; its temporary rig JSON is removed.
blender -b --python tools/build-johto-characters.py -- target/johto-character-kit --skip-preview
# The result contains catalog.glb and editable Blender source batches.

# Convert an existing transient authoring directory explicitly.
python3 tools/animated_glb.py --rig-directory /path/to/authored-rigs --output target/catalog.glb

# No Blender or game pack required for these checks.
python3 tools/test_animated_glb.py
python3 tools/human_locomotion.py
python3 tools/check-johto-characters.py
python3 tools/check-model-layout.py
python3 tools/check-room-furniture-bindings.py
python3 tools/check-facility-table-rigs.py
```

The ten current `character-families-01.blend` through `-10.blend` snapshots are
generated output, so they are not stored alongside the canonical catalog.
The recipe recreates separate named parts, local transforms, joint pivots,
materials and the studio layout. The unique older `characters.blend` is retained.
A complete Blender 4.3.2 replay reproduced all 917,024 triangle positions and
winding, materials and joint binds. Corner normals can vary by one six-decimal
export step (measured maximum 0.000001014); the checked-in GLB preserves the
original runtime normal bits. Regenerated authoring output therefore requires
review rather than blindly replacing the canonical file by hash.

The migration check compares every original JSON rig with its decoded GLB scene
while those inputs exist, including signed-zero float bits. Once retired, the
ongoing suite continues against the canonical GLB without JSON sidecars. It
checks all scenes, shared geometry, parent maps, materials, production clips,
and reader/exporter round trips. The Rust suite also retains compact per-rig
geometry digests captured from an exact comparison with the original Rust
loader, covering normalized normals, colors, indices, binds and UVs for all 75
looks. Optional STL previews read the GLB bind pose
and remain ignored output; see [local static previews](model-stl.md).

## Pidgeotto battle articulation

`models/battle_species/pidgeotto.glb` is the single canonical Pidgeotto model.
It retains 34 named anatomical parts and their materials, a shoulder hierarchy,
and the standard `pidgeotto.idle_wings` clip. The 0.8-second loop has 65 quaternion
keys per wing, exact closed endpoints, and smooth turnaround velocity. The first migration preserved the former Rust-loaded geometry bit for bit;
subsequent reviewed art changes keep explicit geometry fingerprints.

The battle renderer uploads three immutable groups once: body, left wing, and
right wing. Each battler owns its wing transforms and presentation clock; an
individual switch resets only that actor. Camera fitting and source-row
registration use one conservative animation envelope. Neutral geometry still
sets canonical physical size, ground contact and hit anchors. The parent actor
retains source-program displacement and visibility, and palette flashes cover
all three groups in both full and reduced modes. Animated actors always use
live row rendering rather than the optional static capture cache.

The authoring recipe remains `tools/build-battle-species.py`; its Pidgeotto
export writes GLB directly through `tools/pidgeotto_glb.py`. No JSON or STL
sidecar is required. A clean game build uses the committed GLB without Blender.

```sh
python3 tools/test_pidgeotto_glb.py
python3 tools/check-battle-species.py
cargo test --locked -p crystal-voxel-view pidgeotto
# Disposable native fixture with a legal level-25 Pidgeotto and production inputs.
cargo run --locked -p crystal-bevy --example immersive_battle_3d --features location-tester -- /path/to/external.crystalpack --pidgeotto --live
```

Native battle verification used the production controller, an 800×550 window,
and the automatic llvmpipe software profile. The eight-second clip contains
224 actual captured frames (about 28 fps), with median/p95 update times of
29.38/45.57 ms. Video timestamps differ from the recorded originals by at most
0.5 ms due to encoding precision; no frames are interpolated. At 1180×812,
recording averaged about 12 fps, while a separate measurement without game
screenshots had median/p95 times of 42.17/65.40 ms. These viewport-specific
measurements do not establish a general performance gain.

The complete voxel suite passes 843 tests (two existing ignored), and all 36
source-check commands pass. A real-pack controller regression verifies legal
move PP consumption and original animation dispatch. The full browser client
compiles for Wasm; browser rendering remains unverified in this environment.
Native F3 round trips restore every part, and a real Gust turn retains its
source objects and sounds with constant mesh/material counts throughout.

The next papercraft art pass replaces the round bird face and comb with a swept
coral crest, angular eye markings, a hooked pink beak, layered cream/brown
feathers and an alternating tail fan. All 34 part identities and the idle clip
remain intact. Neutral height stays 1.000 authored unit with Y=0 footing;
the verified pack dimension still supplies the physical size. The broader wing
and tail silhouette is framed by the same shared camera fitting.

Triangles change from 1,796 to 1,752. Flat crease normals require 4,538 vertices
instead of 1,274, and the canonical GLB grows from 81,500 to 159,404 bytes.
The renderer still uses the same three immutable draw groups. A matching
800×550 native recording captured 217 real frames over eight seconds, with
median/p95 updates of 29.63/47.23 ms. This is a visual improvement with similar
observed pacing in these single runs, not a claimed optimization.

This is an idle wing clip, not a replacement for any original attack sequence.
Pokémon beyond the weighted skins below retain their existing static-part
or whole-body presentation until their individual rigs and clips are verified.

## Cyndaquil and Totodile GPU skins

`models/actor_props/battle_cyndaquil.glb` and `battle_totodile.glb` replace their
former JSON models. Totodile retains its reviewed neutral geometry. Cyndaquil
now uses the explicit folded-panel sculpture described below. Both have standard
glTF skins and named idle, attack and hit clips.
Cyndaquil has ten joints, including five separate layered flame quills and two
forelegs. Totodile has eight joints, including a hinged jaw, separate forepaws
and a two-part tail. Rear paws and toe contacts stay planted. Body/coat/inlay
surfaces share the same continuous weight field, while facial details stay
rigidly attached to the appropriate head or jaw joint.

The bounded Rust reader parses each canonical GLB once. Each battler owns its
joint entities and small reused pose buffers. Bevy GPU skinning deforms the
immutable mesh; there is no per-frame mesh rebuild, asset parsing or material
allocation. Inverse binds are shared per species. The neutral mesh still sets
canonical physical size, footing and hit anchors. A conservative animation
envelope controls camera fit and culling without rescaling the creature.

Idle follows elapsed presentation time. Attack channels sample the existing
real move cue's normalized progress. A real visible HP-loss cue starts one
short local hit reaction, even when one HP is lost. Transitions blend over
0.10 seconds, while an explicit source rewind samples its requested pose
directly. The original source program remains responsible for move effects,
colors, sound events and command timing; body sampling does not mutate it.
Hidden actors pause idle and observe cue retirement so F3 does not replay an
old hit. Articulated actors cannot enter the optional static image cache.

Normal builds consume the checked-in GLBs without Blender. The actor authoring
recipe exports both through `cyndaquil_glb.py` and `totodile_glb.py`. Cyndaquil can
also regenerate directly from `cyndaquil_sculpt.py` without Blender; JSON
intermediates and optional geometry previews remain ignored. No duplicate
runtime geometry is checked in.

```sh
python3 tools/test_cyndaquil_glb.py
python3 tools/test_totodile_glb.py
python3 tools/check-actor-props.py
cargo test --locked -p crystal-voxel-view species_rig
cargo test --locked -p crystal-voxel-view battle_view::skinning
# These fresh preview sessions have no save destination. Use the normal FIGHT
# menu to select Ember or Bite against Vance's unchanged trainer party.
cargo run --locked -p crystal-bevy --example immersive_battle_3d --features location-tester -- /path/to/external.crystalpack --enemy-gust --starter cyndaquil --live
cargo run --locked -p crystal-bevy --example immersive_battle_3d --features location-tester -- /path/to/external.crystalpack --enemy-gust --starter totodile --live
```

The importer and integration gates verify exact neutral geometry, normalized
weights, loop seams, finite joints, planted feet, independent instances,
source-clock ownership, stable mesh/material counts and entity retirement.
Native GPU playback and capture results are recorded separately; a passing
importer test alone is not proof of visible animation.

Native verification at commit `67374f5` used an 800×600 window, a 600×450
modeled target, llvmpipe/OpenGL, balanced vertex lighting, shadows off and a
30 Hz capture cap. Cyndaquil/Gust/Ember recorded 662 real frames over 25.15
seconds (median/p95 update 29.74/57.33 ms). Totodile/Gust/Bite recorded 666
frames over 25.11 seconds (31.79/54.42 ms). Both remained modeled throughout,
kept 13 meshes and 33 materials, and showed changing joint silhouettes during
idle as well as valid attack/hit presentation. These are individual software
renderer observations, not a controlled performance comparison. The first
restored motion is deliberately subtle at the canonical small starter sizes.

A separate Totodile/Water Gun capture exposed the existing two-row extraction
fallback: that move still switches to the original 2D scene before returning
to the animated model. It is not evidence of a completed 3D Water Gun attack.
The source's `BattleAnim_UserObj_2Row` behavior remains a tracked presentation
gap. Bite and Ember require no such extraction. Native move effects retain
original source sprites; these clips demonstrate body articulation, not an
all-effects completion claim.

The focused Rust gate passed 80 voxel/import checks and 13 distinct real-pack
controller/capture checks. All 23 species authoring tests passed. The normal
native build and full-feature Wasm compilation passed. Browser GPU playback
is still unverified in this environment.

A subsequent complete voxel run passed 875 tests with zero failures and three
ignored checks; the ignored one-time source migration comparison was also run
successfully on its preserved inputs. All 38 source-asset check commands passed
after the repository reference checker was taught the two canonical starter
GLBs. Formatted test-only filesystem paths are not counted as embedded assets;
production model references remain literal and duplicate JSON is rejected.

## Reconstructed Gengar skin

`models/battle_species/gengar.glb` is the canonical Gengar asset. Its 27 named
parts, four materials, 6,952 vertices and 5,560 triangles preserve the reviewed
papercraft sculpture's neutral position, normal, color and index bits. This is
a newly reconstructed rig, not a claim to recover the missing older rig bytes.
The 338,208-byte GLB replaces its former JSON geometry; the deterministic
authoring recipe is `tools/gengar_glb.py`.

Fourteen joints articulate the torso, coherent facial core, shoulders/hands,
ears and dorsal groups. The grin, teeth, eyes and eyelids follow one rigid face
region with the supporting body surface. All 345 low-foot/toe vertices remain
fixed, while 1,265 vertices blend multiple influences across continuous body
regions. Idle loops in 3.6 seconds. Attack and hit channels use the shared
source-cue/real-HP-loss playback contract above, with distinct arm, hand, ear
and back motion rather than a copied starter skeleton.

Decoded tests check every authored key and midpoint plus blends: 1,371,136
fused-body triangle checks across 412 poses found no winding reversals or new
degeneracy. This caught and corrected two early weight-boundary folds before
integration. A separate dense audit sampled every vertex through 1,603 clip
poses and 2,100 blended poses. The conservative camera/culling envelope
contains that motion with approximately 5.2%, 6.7% and 9.1% extra extent in
X/Y/Z. Physical size still uses the unchanged neutral geometry and the pack's
Pokédex dimension, never the animation envelope.

The normal native fixture uses a legal level-25 Gengar, learned Shadow Ball
and the original level-20 Sudowoodo encounter. A real turn showed Shadow Ball,
its source palette inversion, Sudowoodo's Rock Throw, an 18-HP loss and the
articulated hit settling back to idle. Every recorded update remained modeled;
mesh/material counts stayed at 11/33. At 800×600 with the same 600×450 balanced
vertex-lit llvmpipe/OpenGL target and shadows off, 726 actual frames were
captured over 25.04 seconds. Median/p95 update times were 30.58/40.10 ms.
This is one software-renderer observation, not a controlled speedup claim.

An earlier level-40 preview critically knocked out the opponent and entered
the existing classic faint fallback. That mostly-classic recording is retained
only as evidence of the fixture limitation; its timings are not a 3D result.
Faint, send-out, capture and unsupported source-row sequences still have their
documented presentation limits.

The complete voxel suite passes 876 tests with zero failures and three ignored
checks; the one-time neutral migration audit also passed separately. All 39
source-asset check commands pass, including 15 Gengar tests, strict single-file
canonical selection and Pidgeotto compatibility. Native compilation and the
full-feature Wasm check pass; browser GPU playback remains unverified.

```sh
python3 tools/test_gengar_glb.py
python3 tools/check-battle-species.py
cargo test --locked -p crystal-voxel-view species_rig
cargo run --locked -p crystal-bevy --example immersive_battle_3d --features fullscreen-scaling,voxel-view,location-tester -- /path/to/external.crystalpack --shadow-ball --size 800x600 --live
```

## Optional articulation proof

```sh
python3 tools/animated_glb.py --repo . --out target/animated-glb --human-catalog
```

This compatibility command exports separate trainer-wave and Pidgeotto-wing
proofs plus an optional human catalog with demo waves. Proof clips are clearly
marked as demonstrations and are not production locomotion or original attack
choreography. The separate proof does not replace Pidgeotto's production idle clip.
Proof outputs belong in ignored directories and are not additional tracked
geometry. The exporter does not migrate any other Pokémon.

## Native integration check

The native game runs imported locomotion with ordinary controller movement.
The verification route completed 64 movement frames across five grid origins
without an additional terrain rebuild. At rest, matching before/after captures
were pixel-identical outside the trainer's live breathing/blink region.
All 832 voxel tests pass (two existing ignored tests), including exact geometry
digests for all 75 rigs, loop seams, planted feet at 60/30/9 Hz, source-distance
phase, and entity/mesh reuse. All 35 asset/source Python checks pass.
The full browser client compiles for `wasm32-unknown-unknown`; browser rendering
was not exercised in this cloud environment.

Native timing used `--features location-tester` on both builds, a 32-tile terrain
margin, the 1180×812 viewport, New Bark `(13,6)`, orbit `-1`, zoom `3`, and Mesa
25.0.7 llvmpipe/OpenGL. Four warmed 15-second no-readback runs in baseline,
GLB, GLB, baseline order measured median/p95 milliseconds of 172.62/292.88,
150.56/241.65, 135.00/227.57, and 116.89/207.85. This noisy sample does not
establish a frame-rate improvement or a repeatable added cost. A separate
800×550 actual-timestamp recording averaged about 8 fps and remains a technical
check, not a fluid-performance demonstration. The earlier unmatched comparison
used a 48-tile fullscreen-scaling margin for only one build and is excluded.

The preview accepts `--size WIDTHxHEIGHT` for explicit capture dimensions.
Recording a scripted walk waits for its first ordinary input after warmup;
measurement-only runs keep their existing start condition. Normal play and
the authoritative movement clock are unchanged.


## Cyndaquil folded-panel refinement

The canonical Cyndaquil is now rebuilt from original, explicit paper panels in
`tools/cyndaquil_sculpt.py`. This replaces the previous remeshed clay body with
an authored wedge muzzle, broad cheek and chest folds, asymmetrically tucked forepaws, slimmer
planted toes and a coat following the exact head/back fold vertices. The closed
sleepy-eye ribbons sit on the cheeks, below the cap. Five flame assemblies have
unequal licks, thinner ridges, tapered embedded roots and different twists so
large colored facets remain legible from the runtime flank. They are permanent
sculpture anatomy; the game still owns all emitted Ember sprites and timing.

The stable 30 anatomy identities and ten-joint hierarchy are preserved. Source
geometry has 4,506 split-normal vertices and 2,086 triangles, versus 5,128 and
7,180 previously. `battle_cyndaquil.glb` is the only canonical runtime geometry.
The earlier refinement Blender file is retained as historical editable work;
no new duplicate `.blend`, STL, mesh JSON or encoded geometry is checked in.
The actor recipe imports the same authored panels for optional Blender editing
and review. Make canonical sculpture changes in the Python recipe, then review
and update the pinned geometry digest deliberately.

```sh
python3 tools/cyndaquil_glb.py --output crates/crystal-voxel-view/models/actor_props/battle_cyndaquil.glb
python3 tools/test_cyndaquil_glb.py
python3 tools/check-actor-props.py
python3 tools/check-johto-models.py
```

The body-height reference remains bit-identical at `0.7167289853096008` and the
full silhouette height remains `0.9100000262260437`. The existing runtime metre
calibration, `0.716729 / 0.91`, is unchanged. Both meshes have exactly zero as
their minimum Y. Neutral bounds before and after are:

| Coordinate | Previous minimum / maximum | Folded-panel minimum / maximum |
| --- | --- | --- |
| X | -0.32725501 / 0.32725501 | -0.34094277 / 0.34557807 |
| Y | 0 / 0.91000003 | 0 / 0.91000003 |
| Z | -0.53695601 / 0.53695601 | -0.54588354 / 0.47300000 |

Idle remains 3.2 seconds and attack/hit remain one-second normalized clips.
Idle adds a nose-led sniff and asymmetric paw curls. The attack braces
immediately, without a preparatory windup that would conflict with Ember's
first source emission. The flame sheets follow the brace, and a hit produces a
short recoil with delayed quill recovery. These curves create no move events,
change no source ticks and do not move the root. All 480 vertices at or below
Y=0.105 stay exactly planted through every sampled clip; loop endpoints return
to the exact neutral pose. Byte-weight pose error remains below 0.00015 model units (under 0.11 mm
at the canonical 0.508 m body height); this bound accommodates the larger
articulation while retaining the compact normalized-byte skin format.

The fourteen Python model checks cover full recipe-to-GLB reproducibility,
reviewed geometry/order, closed panel volumes and normal winding, normalized
influences, rigid face/quill attachment, planted feet, smooth loop closure,
immediate attack bracing, the unchanged height datum and rejection of altered
storage. Actor and full source-registry checks also pass. Neutral views are
reviewed from the actual GLB in Blender and a simple software rasterizer;
posed raster views use decoded GLB animation keys. Production battle timing
and full scene readability still require the native integration capture, not
just these model-level renders.
