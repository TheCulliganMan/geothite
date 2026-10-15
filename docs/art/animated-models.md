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
former JSON models. Totodile uses a refined crocodile silhouette with cheek-wrapping red eyes, a smaller
open grin, pink tongue, four visible fangs, a broad fitted yellow chest chevron,
a fuller pear-shaped belly, tapered tail and stronger red dorsal plates. Its authored blue palette is used by default in location previews;
real saves still derive shiny appearance from their actual Pokémon DVs. Cyndaquil
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

## Chikorita increment

The [Chikorita sculpture and skin](chikorita-papercraft.md) replaces its static
JSON with a reproducible canonical GLB and species-specific idle, attack and
hit articulation. Four planted feet and three leaf joints give it distinct
quadruped mechanics. The original pack-derived physical height and battle
cue clock remain authoritative. This is one reviewed species increment;
the remaining static Pokémon still require their own art and animation work.

## Bayleef increment

[Bayleef](bayleef-papercraft.md) now uses its own connected long-neck sculpture
and a 16-joint anatomical skin, with a folded notched crown, curled collar and
planted quadruped motion. Its canonical GLB replaces the former static JSON;
this is a separate species pass after Chikorita.

## Meganium increment

[Meganium](meganium-papercraft.md) has a distinct mature long-neck sculpture,
a six-petal flower, paired curved antennae and a fitted opening jaw. Its
23-joint skin preserves planted feet while the petals and antennae follow
body motion. One canonical GLB replaces its previous static JSON.

## Totodile and Spearow attack response

Both attack clips now brace forward early in the existing source move cue.
Totodile's former reverse windup and 43%-progress peak become an 18%-progress
brace, with the jaw opening first at 12%. Spearow likewise pecks at 18%, with
beak opening and balancing wing spread at 12%. The longer recovery returns
both rigs to neutral; source programs still own displacement, effects, sound,
HP and timing. These are body response improvements, not move-specific attack
reconstructions.

Binary accessor comparison verifies that only the two GLBs' attack rotation
keys changed. Geometry, materials, weights, binds, idle/hit clips, neutral
dimensions and file sizes remain byte-identical. No secondary mesh is added.
Spearow's faster wing response uses 90 Hz one-time envelope sampling so its
conservative visibility padding stays within the existing 10% budget. Runtime
playback continues to follow the original source cue progress.

The 19 model-level Python checks pass. New stored-key regressions fail on the
previous clips and verify early forward motion, mouth opening and wing balance.
All 957 voxel tests pass, with three existing ignored checks. This includes
actual BodyPose move-cue sampling at 9/30/60 Hz, planted feet, dense off-grid
bounds, blends, physical scale and resource sharing. Before/after software
raster views inspect the decoded GLBs at 0%, 10%, 20% and 40% progress; they
are model inspection, not an in-game GPU capture. Native/browser GPU review
of this increment remains unverified.

On macOS, the broader actor/species/Johto source validators still stop on
unchanged Chikorita or Bayleef canonical regeneration. Chikorita's differences
include normal components near 1e-17 and 1e-15, while these validators require
byte-exact regeneration. Those assets and their recipes are unchanged by this
increment; the two changed species regenerate exactly and pass their checks.

## Grass starter attack response

Chikorita, Bayleef and Meganium now brace forward at 18% of the existing source
move cue instead of first moving backward and peaking at 34–54%. Their leaf,
crown, collar, flower and antenna chains retain distinct delayed follow-through
and return to neutral. Only attack rotation accessors in the canonical GLBs
change; binary comparison against HEAD confirms identical geometry, normals,
materials, weights, binds, idle/hit clips, neutral dimensions and file lengths.
The recipes reproduce every stored attack key exactly.

Decoded GLB before/after software views at 10%, 20% and 40% were inspected for
earlier forward motion and intact anatomy. These views are model inspection;
production GPU/native/browser presentation remains unverified. No gameplay
move programs, effect events, audio or damage timing change.

Run the stored-model regressions with:

```sh
python3 tools/test_chikorita_glb.py
python3 tools/test_bayleef_glb.py
python3 tools/test_meganium_glb.py
cargo test --locked -p crystal-voxel-view species_rig --lib
```

The macOS regeneration limitation also affects Meganium in this checkout:
its unchanged HEAD reader rejects its reviewed recipe, and expanded geometry
comparison differs in near-zero coordinates (about 1e-17). Chikorita and
Bayleef's unchanged HEAD recipes likewise fail byte-exact regeneration.
This increment preserves stored geometry rather than regenerating it on this
platform. Those failures prevent claiming a completely green authoring suite.
The six new onset and recipe-key checks pass independently of geometry export.
The three species deformation checks pass, including planted contacts, winding,
strain bounds and visible articulation; Bayleef and Meganium probes now sample
the earlier peaks without reducing their required movement. The later current-state full voxel run recorded all 15 active species-rig
regressions passing (one migration audit ignored), including the new onset
test, dense bounds and planted contact. The full voxel suite itself still
has 22 failures outside this rig test group; no complete suite pass is claimed.
The full roster model and rigging goal remains active.

## Wooper increment

Wooper now uses `models/battle_species/wooper.glb`, replacing its static JSON
with a nine-joint standard glTF skin. Its 22 existing anatomy parts, 3,991
vertices, 5,532 triangles, 0.70-unit neutral height and colors remain exactly
unchanged in f32 storage. The original axolotl recipe remains the editable
geometry source; `tools/wooper_glb.py` adds the skin and clips without storing
a second mesh. The neutral geometry digest is
`5a306ae7d1f9b79c7ce3897c0d538b2752c7cdf85c68f050fe8a1aa0b1bb801a`.

Idle surveys with a head turn, asymmetric external gill hinges and a delayed
paddle. Attack braces at 18% of the production move cue, with gill and tail
settling at 28%; hit recoils and returns to neutral. Both three-lobed feet stay
root weighted. Facial details follow the head rigidly; each gill's stalk and
branches share a hinge to prevent separation. The tail has a blended two-joint
chain. The first tail pass moved only 0.0135 units because torso and tail
rotations canceled; stronger independent tail articulation now passes the
0.02-unit visibility requirement while keeping deformation checks intact.

```sh
python3 tools/wooper_glb.py
python3 tools/test_wooper_glb.py
python3 tools/test_axolotl_sculpt.py
cargo test --locked -p crystal-voxel-view species_rig --lib
```

Five stored-GLB tests, six axolotl sculpture checks and the canonical
Wooper source-routing regression pass. They cover exact
neutral anatomy, canonical regeneration and mutation rejection, normalized
weights, inverse binds, grounded feet through clips and blends, floor clearance,
normal winding, bounded triangle strain, loop seams and early cue response.
Production review uses the fresh `content-packs/core-modular.browser.crystalpack`;
the pack is external ignored content and is not changed by this model work.
The first Rust attempt failed because its build script ran before the new
GLB existed and emitted no Wooper embedded payload. A subsequent full voxel
run generated that payload and passed all 15 active species-rig regressions,
including Wooper contacts and registry sharing. The later reviewer produced inspected production GPU frames, documented
below. Software model views alone cannot establish in-game acceptance.

Current-state roster audit: the exact species registry contains 225 entries,
not the 251 stated in historical review notes. Wooper replaces an existing
entry. The later Gyarados sculpture adds one entry, bringing current coverage
to 226 and leaving 25 missing exact bodies. Full roster appearance, remaining skins, normal/shiny views
and native/browser review are still required before the overall goal is done.

## Fresh-pack native review follow-up

The rebuilt production reviewer captured Wooper, Chikorita, Bayleef and
Meganium against `content-packs/core-modular.browser.crystalpack`. These actual
2,000 by 1,400 GPU images were individually inspected:

- `target/pokemon-native-review/wooper-wild.png`: grounded feet, intact paddle,
  attached gills and clear head silhouette. The teal body is dark under this
  arena lighting, and the smile loses contrast under the broad head; these
  remain appearance concerns rather than a finished material approval.
- `target/pokemon-native-review/chikorita-wild.png`: four contacts, intact
  petiole, readable broad leaf and fitted face. Olive shading and large body
  facets remain visible; this captures idle presentation, not the revised attack.
- `target/pokemon-native-review/bayleef-wild.png`: long neck, rooted collar,
  crown notch and toes remain readable. The lower muzzle is strongly shaded;
  the leaf and collar silhouette survives the production camera.
- `target/pokemon-native-review/meganium-wild.png`: long neck, six rooted petals,
  paired antennae and planted toes are visible. Red petal folds are dark and
  the muzzle seam is strongly shaded; tail and flower remain separate readable
  silhouettes. GPU attack motion and palette review remain incomplete.

The trainer Gust preview failed before drawing with a fresh-pack save-reference
mismatch: `EVENT_BEAT_BIRD_KEEPER_VANCE` versus an empty compiled trainer flag.
The wild preview uses the production controller and succeeds; it does not prove
that trainer fixture was repaired. Reproduce the successful capture, substituting
the desired species and output name:

```sh
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
  ./target/debug/examples/immersive_battle_3d \
  content-packs/core-modular.browser.crystalpack --wild-species WOOPER \
  --size 1000x700 --screenshot target/pokemon-native-review/wooper-wild.png
```

The images and logs remain ignored verification evidence. These are individual
native battle frames, not frozen matched 2D comparisons, browser approval or
GPU attack-animation acceptance. The overall model/rig goal remains active.

## Gyarados articulation

The [Gyarados sculpture and skin](gyarados-sculpture.md) now has nine authored
joints and idle/attack/hit clips, replacing its new static JSON with one
canonical GLB. Ground contacts, rigid facial/crest anatomy, an independent
jaw and delayed barbels/tail follow the same skinning pipeline as Wooper and
the starters. Neutral art, pack-derived physical scale and source move timing
remain unchanged. Model-level checks, decoded three-angle pose review and all 16 active Rust
rig tests and 12 battle skinning tests pass. The fresh GPU 3D/2D pair
preserves simulation state, and two
shiny captures show a complete HUD. Source-cue attack/hit GPU playback and
remaining art refinements remain open; see the linked review for evidence.

## Sudowoodo increment

[Sudowoodo](sudowoodo-sculpture.md) now has a dedicated exact body and nine-joint
skin, replacing its missing battle coverage with a leaning stone trunk,
forked crown, branch arms, six green fingers and fixed root pads. It uses one
canonical GLB and the shared source-cue playback pipeline. Stored-model, Rust rig and skinning checks pass. The initial native pair
preserves state, and the Gyarados matchup uses both exact bodies. Shiny review
found disappearing markings; a verified material correction is rebuilding
for fresh GPU inspection. The registry
now covers 227 species, with 24 dedicated bodies still missing.

## Lapras dedicated body (2026-10-13)

Following the requested move away from Sudowoodo refinement, Lapras gets a new
exact sculpture and nine-joint skin. See `lapras-sculpture.md` for its canonical
source, anatomy and reproducible authoring commands. The existing shiny flag and
pack palette apply to the same body. No additional shiny pipeline is added.

The final six independent GLB tests and focused canonical routing check pass.
Front/oblique/rear neutral and decoded idle/attack/hit inspection identified
hollow ears, now corrected with fleshy backing. The native held-state comparison
was inspected at `target/pokemon-native-review/lapras-3d.png` and `lapras-2d.png`;
its receipt reports `state_preserved=true`, checksum hash 2525914295 and LCD frame
43. Shell, neck, face, ears and four flippers read clearly; the pack owns the
relative scale against Sudowoodo. Its blue reads dark teal under the production
lighting, unlike the source sprite's royal blue; this remains a fidelity limit.
Native source-cue attack/hit and browser review remain open. Runtime
coverage is now 228 exact bodies out of 251 packed species, leaving 23 missing;
this does not complete the all-Pokémon model and rigging work.

Lapras validation passed: six independent Python anatomy/skin checks, focused
canonical routing, two Cargo Lapras integration tests (registry/shell attachment
and generic GPU activation at 2.4892 m), all 18 active rig regressions (one
existing migration audit ignored), and the whole 228-species model loading check.
The last two suites ran against the freshly compiled Cargo test executable
`target/debug/deps/crystal_voxel_view-096ec73f68f07b45`; redundant queued Cargo
rebuilds triggered by concurrent unrelated source edits were stopped afterward.
The wider Python routing suite still hits the previously recorded Mac Chikorita
exact-regeneration mismatch; the focused Lapras routing check passes.

## Snorlax dedicated body (2026-10-13)

Snorlax gets an original exact body and nine-joint skin; see
`snorlax-sculpture.md` for the source and anatomy. The seated broad torso, cream
mask/belly, sleeping eyes, pointed ears, drooping arms, padded soles and twelve
claws replace a missing-body fallback. The initial oblique review caught coarse
cream surfaces and blue showing through the belly; denser fitting with welded
smooth normals corrects both in the final neutral GLB review. Shininess uses
the existing flag and palette without separate geometry.

The final six Python mesh/skin checks, focused canonical routing and strengthened
seated-width/ear-height regression pass. A front review also corrected the
upright stance and tall narrow ears: final torso width and low seated shape
match the intended sleeping-bear silhouette. The native rebuild embeds the
final GLB hash. Its catalog confirms 229 exact bodies and 22 missing.

The native pair `target/pokemon-native-review/snorlax-3d.png` / `snorlax-2d.png`
was inspected; `state_preserved=true`, checksum hash 690171214, LCD frame 40.
The feet are grounded and sleeping face, padded soles and claws read at source
pack size against Sudowoodo. The source view is Snorlax's back sprite, so the
front artwork and neutral reviews supplement that comparison. The cream mask
edge still facets under native lighting and the blue renders dark teal; these
remain fidelity limits. Native source-cue attack/hit, browser and overworld
review are open. Both Cargo Snorlax integration tests pass: canonical registry
sharing with planted soles, and generic GPU skin activation at 2.1082 m.
All 19 active rig regressions pass (one existing migration audit ignored), as
do all 15 skinning regressions and the full 229-species model loading check.
The three shared suites ran against the freshly compiled test executable in
`target/pokemon-models/debug/deps/`; they did not trigger additional source
rebuilds during concurrent unrelated work.

Source coverage is now 229 exact bodies of 251 species, leaving 22
missing; dedicated body coverage does not prove completion of all model/rigging
improvements or visual acceptance.

## Pikachu dedicated body (2026-10-13)

Pikachu gets an original exact mouse body and eleven-joint skin; see
`pikachu-sculpture.md` for anatomy and authoring commands. Initial neutral review
caught a narrow torso against an oversized bolt tail. Final geometry widens the
body and narrows the tail before runtime integration. Separate black/gold ear
parts share their exact ring and skin weights, preserving the seam throughout
all clips. Shininess remains the existing flag and palette.

Seven independent Python GLB tests and focused source routing pass. Both Cargo
Pikachu integration checks pass: neutral registry sharing with fixed mouse pads,
and eleven-joint GPU skin activation at 0.4064 m. The freshly built test
executable passes all 20 active rig regressions (one existing migration audit
ignored), all 16 skinning regressions and the 230-species reusable model check.

The first native build failed because a concurrent arena edit referenced
`location_details` before its definition existed. The helper is present in the
current source and the fresh native rebuild passes. Its catalog independently
confirms 251 packed species, 230 exact bodies and 21 missing.

The native held pair `target/pokemon-native-review/pikachu-3d.png` /
`pikachu-2d.png` was inspected. The receipt reports `state_preserved=true`, hash
3247630517 and LCD frame 60. Pikachu is grounded at source-pack size beside
Sudowoodo. Face, red pouches, eye glints, tipped ears and paws remain readable.
The small physical body follows its 0.4064 m reference; no scene size clamp was
added. The flat bolt overlaps its head/ears from this camera angle and needs
further readability review. The source view is the back sprite; front/rear
neutral and decoded pose reviews supplement the comparison. Browser and native
source-cue attack/hit review remain open.

Source coverage is now 230 exact bodies of 251
species, leaving 21 missing; body coverage does not complete the full appearance,
rigging and visual acceptance goal.

## Bulbasaur dedicated body (2026-10-13)

Bulbasaur gets an original exact quadruped body and nine-joint skin. See
`bulbasaur-sculpture.md` for anatomy and authoring commands. A front-oblique
review caught an over-tall bulb and long-legged stance; the corrected silhouette
is lower and broader. Stored shading checks then caught inward normals on the
scalloped bulb caps. Centered radial fans with planar end rings correct those
caps, and all seven Python GLB checks now pass. Four pads and twelve claws stay
fixed while the torso/head, four limbs and folded bulb respond to source cues.
Shininess keeps the existing flag and palette.

The focused source-routing and six-lobe geometry checks pass. The corrected
native build passes and its embedded asset hash matches the final GLB. The
fresh client catalog confirms 251 packed species, 231 exact bodies and 20
missing.

The held native pair `target/pokemon-native-review/bulbasaur-3d.png` /
`bulbasaur-2d.png` was inspected. The receipt reports `state_preserved=true`,
hash 355223347 and LCD frame 47. Its bulb, broad head, red eyes, dark markings
and planted claws are readable at source-pack size beside Sudowoodo. The source
view uses the back sprite, supplemented by inspected official front artwork
and neutral model views. Production lighting darkens the authored mint-teal
body; the source sprite is bright green. Color fidelity, native source-cue
attack/hit, browser and overworld review remain open. Both Cargo Bulbasaur
integration tests pass: canonical neutral sharing with fixed pads/claws, and
nine-joint GPU skin activation at 0.7112 m. The freshly built Cargo test
executable passes all 21 active rig regressions (one existing migration audit
ignored), all 17 skinning regressions and the 231-species reusable model check.

Source coverage is now 231 exact bodies of 251 species, leaving 20 missing;
this does not complete the full model, rigging and visual acceptance goal.

## Charmander dedicated body (2026-10-13)

Charmander gets an original exact orange lizard body and eleven-joint skin;
see `charmander-sculpture.md` for anatomy and authoring commands. The rounded
head, cream belly, three-digit paws, planted clawed feet, curved tail and five
folded-fire layers replace a missing body. Small eye patches were initially
over-tessellated; reducing their subdivision density retains their fit and the
silhouette while meeting the runtime vertex budget. Seven independent Python
GLB checks pass, including fixed pads/claws and tail/fire chain follow-through.
Shininess retains the existing flag and palette.

A stale camera test supplied `Vec3::ONE` to the now-scalar `BattleBody::modeled`
constructor; its unit scale is corrected to `1.0` without changing the intended
camera fixture. Canonical source routing passes. The native build passes and
its embedded asset matches the final GLB. The fresh client catalog confirms
251 packed species, 232 exact bodies and 19 missing.

The actual native pair `target/pokemon-native-review/charmander-3d.png` /
`charmander-2d.png` was inspected; `state_preserved=true`, checksum hash
3584115452 and LCD frame 47. Orange body, blue eyes, cream belly, curved tail
and planted claws read at source-pack size beside Sudowoodo. The source view
is the back sprite; front/rear decoded idle/attack/hit reviews supplement it.
The flame presents a thin edge from this camera angle and needs a broader
volume/readability pass. Native source-cue motion, browser and overworld review
remain open. Both Cargo Charmander integration tests pass: canonical neutral
sharing with planted pads/claws and eleven-joint GPU skin activation at
0.6096 m. The freshly compiled test executable passes all 22 active rig
regressions (one existing migration audit ignored), all 18 skinning regressions,
the 232-species reusable model check and both camera regressions. The shared
suites ran directly from that executable without triggering more rebuilds.
Source coverage is now 232 exact bodies of 251 species, leaving 19 missing;
the full model, rigging and visual acceptance goal remains unfinished.

## Squirtle dedicated body (2026-10-13)

Squirtle gets an original exact blue turtle body and nine-joint skin. See
`squirtle-sculpture.md` for anatomy and authoring commands. The rounded head,
segmented cream chest, brown carapace, closed rolled rim, three-digit paws,
planted toes and solid curled tail replace its missing battle body. Initial
review caught a hollow-looking curl; filling its center corrects the silhouette.
Shell, rim and plates share a rigid torso owner, while limbs/head and tail
articulate independently. Shininess retains the existing flag and palette.

Seven independent Python GLB checks and focused source routing pass. The
native rebuild passes and its embedded asset matches the final GLB. The fresh
client catalog confirms 251 packed species, 233 exact bodies and 18 missing.

The native pair `target/pokemon-native-review/squirtle-3d.png` / `squirtle-2d.png`
was inspected: `state_preserved=true`, hash 2100597877, LCD frame 36. Shell,
cream plate joins, maroon eyes, blue limbs and filled tail curl read at source
size against Sudowoodo. The source view is its back sprite. Front/rear decoded
idle/attack/hit reviews supplement it. Production blue reads more teal than
the source sprite's blue, a remaining color-fidelity limit. Native source-cue
motion, browser and overworld review remain open.

The initial test build raced a transient moved-value error in
`interior_models.rs`, where a material-role length check followed
`surface.into()`. Current source moves that check before the conversion.
The retry passes both Squirtle integration tests: canonical neutral sharing
with fixed pads/toes and nine-joint GPU skin activation at 0.508 m. The newly
compiled test executable passes all 23 active rig regressions (one existing
migration audit ignored), all 19 skinning regressions and the 233-species
reusable model check, without triggering additional rebuilds.

Source coverage is now 233 exact bodies of
251 species, leaving 18 missing; the full model/rigging and visual acceptance
goal remains unfinished.


## Geodude: faceted boulder and articulated fists

Geodude adds the seventeenth standard species skin and its previously missing
exact body. It has 29 closed parts, 12,179 vertices, 15,352 triangles and eight
joints. The canonical GLB is 544,164 bytes; neutral digest is
`90658750b47d9380750c88431617ae2860b5f5cdb8e6d1d0af8fb320124ab33d`.
The source recipe and independent checks are documented in
`geodude-sculpture.md`. Existing shininess flag and pack-palette routing are
unchanged. No additional shiny asset or geometry was introduced.

Seven Python model checks and focused canonical routing pass. The native
client build passed and its embedded catalog reports 234 exact bodies of
251 packed species, leaving 17 missing.

The held native pair `target/pokemon-native-review/geodude-3d.png` /
`geodude-2d.png` was inspected. Receipt: `state_preserved=true`, hash
3139018129, LCD animation frame 45, virtual time 1.117300167 seconds. Rock
facets, angular brows, bent arms and clenched fingers read at source size;
the camera partially occludes the near arm. The 2D reference is the back
sprite. Production materials are warmer gray than the source's green-gray
shading. Native source-cue motion, browser and overworld review remain open;
the broader model/rigging goal remains unfinished.

Both Geodude Rust integration tests pass, including shared canonical ownership,
fixed boulder contact across clips and eight-joint GPU skin activation at
0.4064 m. The freshly compiled test executable passes all 24 active species
rig regressions (one existing audit ignored), all 20 skinning regressions and
the 234-species reusable model check.

The first Rust test attempts encountered concurrent scenery edits: the planter
filter omitted newly added mansion material roles, and a new special-room test
called a missing helper. The filter now preserves those roles with the same
surviving-vertex remap as other attributes; its existing preservation regression
passes. The special-room owner corrected its fixture before the successful
retry. No game content pack or audio was changed by this model work.


## Pidgey: original tiny bird and articulated feather chains

Pidgey adds the eighteenth standard species skin and a previously missing exact
body: 43 closed parts, 8,404 vertices, 13,064 triangles and ten joints. The GLB
is 431,212 bytes with neutral digest
`814a75e55669f0361edc7c35026a7a0b112085cda12d4dcc485693bfd1990fb7`.
The source recipe and checks are documented in `pidgey-sculpture.md`.
Shininess uses the existing flag and pack palette with the same skin.

Seven independent Python model checks and focused canonical routing pass.
The native client rebuild passes; its embedded catalog reports 235 exact
bodies of 251 packed species, leaving 16 missing.

The held native pair `target/pokemon-native-review/pidgey-3d.png` /
`pidgey-2d.png` was inspected. Receipt: `state_preserved=true`, hash
2417923469, LCD animation frame 50, virtual time 1.190217375 seconds. Dark
cheek wedge, cream throat/breast, short crest, pink bill and planted feet,
folded cream wing tips and brown tail read at the tiny source size. Native
brown is less saturated than the source sprite's ochre. The 2D reference is
the back sprite with a raised wing; folded neutral/idle model posture does
not reproduce that sprite pose. Native source-cue motion, browser and world
review remain open. This is progress toward the complete appearance/rigging
goal, not full visual acceptance.

Both Pidgey Rust integration checks pass: canonical neutral sharing and fixed
pads/toes/claws across clips, plus ten-joint GPU skin activation at source
size (0.3048 m). The fresh compiled test executable passes all 25 active rig
regressions (one existing audit ignored), all 21 skinning regressions and the
235-species reusable model check. Targeted whitespace checks pass. The broader
model/rigging goal remains active; most exact bodies still require dedicated
rig and full production visual review.


## Oddish: blue bulb and five articulated leaf chains

Oddish adds the nineteenth standard species skin and a previously missing
exact body. The final GLB has 19 closed parts, 8,865 vertices, 14,620 triangles,
12 joints and 441,232 bytes. Neutral digest:
`b138445f423d89daff929c82b90cc4950b9a9534b53d767887664b65e1ada7ab`.
The recipe and checks are documented in `oddish-sculpture.md`. Existing
shininess flag and pack-palette routing share identical geometry and skin.

Seven independent Python model checks pass on the final asset, including
fixed feet and deformation throughout all clips. A depth-span assertion was
added and verified to guard the wide fan from a side view. Focused canonical
routing passes. The final native rebuild passes; its embedded catalog reports
236 exact bodies out of 251 packed species, leaving 15 missing.

The first production review exposed a narrow upright tuft when the mostly
planar leaves were viewed from the battle camera. Spreading the five blades
through depth and lowering the outer leaves corrected that silhouette. The
final held pair `target/pokemon-native-review/oddish-3d.png` / `oddish-2d.png`
was inspected. Receipt: `state_preserved=true`, hash 3978529836, LCD animation
frame 37, virtual time 0.947527042 seconds. The fan is wider from the production
view, with blue bulb, small red eye and rooted feet visible at source size.
Some blades are edge-on or occluded from this camera. The 2D reference is the
back sprite. Native bulb is bluer, and leaves darker, than the source sprite's
gray-green bulb and bright leaves. Native source-cue motion, browser and world
review remain open. The broader appearance and rigging goal remains active.

Both final Oddish Rust integration checks pass: canonical neutral sharing and
fixed root feet across clips, plus twelve-joint GPU skin activation at source
size (0.508 m). The freshly compiled executable passes all 26 active species
rig regressions (one existing audit ignored), all 22 skinning regressions and
the 236-species reusable model check. Targeted whitespace checks pass. No
pack, audio program or unrelated game content was changed. The full model and
rigging goal remains unfinished; most exact bodies still need dedicated rigs
and complete production visual acceptance.


## Poliwag: fitted belly coil and articulated tadpole tail

Poliwag adds the twentieth standard species skin and a previously missing
exact body. The final GLB has 19 closed parts, 15,265 vertices, 25,532 triangles,
five joints and 681,504 bytes. Neutral digest:
`2914f4cf3bdb2b4ea6f4795e1586e3d93f4dee15f5b0050ca2981fdcc37c2f2d`.
The recipe and checks are documented in `poliwag-sculpture.md`. Existing shiny
flag and pack-palette routing share identical geometry and skin.

Seven independent Python model checks pass. The coherent-belly check was
extended and rerun to verify that the forward pink lips remain visible ahead
of the actual stored belly surface. Focused canonical routing passes. The
native rebuild passes; its embedded catalog reports 237 exact bodies out of
251 packed species, leaving 14 missing.

Initial neutral inspection found hidden lips and stubby feet; a rounded lip
volume projecting forward of the belly and longer paddle feet fixed those
features. The initial export exceeded budgets; fewer subdivisions in small
eye patches preserved their outlines while retaining denser belly fitting.
The final front and side neutral reviews were inspected.

The held native pair `target/pokemon-native-review/poliwag-3d.png` /
`poliwag-2d.png` was inspected. Receipt: `state_preserved=true`, hash
2589769990, LCD animation frame 42, virtual time 1.028859708 seconds. The belly
spiral, forward pink lips, ivory eyes, long planted blue feet and pale thin
oval tail with blue centerlines read at source size. The source view is the
back sprite. Native belly edges still show some faceting, and the blue is
lighter than the source back sprite's gray-blue shading. The tail remains
opaque under the existing material contract. Native source-cue motion,
browser and world review remain open; the full appearance/rigging goal is
unfinished.

The first native/test builds encountered a concurrent Ecruteak gate edit:
lintel bounds still referenced a removed `z`. The gate now uses the actual
`entry`/`front` doorway limits; the native retry passed. No pack or audio
content was changed by this model work.

Both Poliwag Rust integration checks pass: canonical neutral sharing and
fixed paddle feet throughout clips, plus five-joint GPU skin activation at
0.6096 m. The fresh test executable passes all 27 active species rig
regressions (one existing audit ignored), all 23 skinning regressions and the
237-species reusable model check. The existing shrine/ruin source-bounds and
outward-winding regression also passes after the gate compile repair.
Targeted whitespace checks pass. The full model/rigging goal remains active;
most exact bodies still need dedicated rigs and full production visual review.


## Clefairy: preserve shared sculpture, add eleven-joint skin

Clefairy's existing shared icon sculpture was migrated to one canonical
`models/actor_props/icon_clefairy.glb`, with no neutral geometry change. Both
the legacy icon and explicit species routes reuse it. The former JSON was
removed, and the pure authoring CLI now emits the exact same GLB without a
parallel JSON. The original and new float32 geometry digests match:
`0d173aea43c62408317aabf296a57c3a202a674df5464cd94b29fc04e872accc`.
The asset has 35 closed parts, 15,257 vertices, 24,216 triangles, eleven joints
and 715,428 bytes. Details are in `clefairy-sculpture.md`.

Seven GLB checks and all eight existing sculpture checks pass. Focused canonical
routing and both duplicate-source rejection tests pass. The authoring CLI was
executed and its output compared byte-for-byte with the shipped GLB. Blender
is unavailable on this host: its batch route was updated and syntax-checked,
but the Blender process was not executed. The pure Python exporter and the
native client build were executed successfully.

The native catalog now reports 238 dedicated catalog bodies out of 251 packed
species, with 13 outside that catalog. Its wording was corrected: those 13
may have shared or fallback meshes rather than no model. Clefairy already had
a detailed shared sculpture; this step adds its rig and explicit catalog entry,
not a new geometry copy. Earlier catalog counts should be read in that narrower
sense, not as proof of complete visual fidelity or physical model absence.

The held native pair `target/pokemon-native-review/clefairy-3d.png` /
`clefairy-2d.png` was inspected. Receipt: `state_preserved=true`, hash
1910648216, LCD animation frame 37, virtual time 0.943902333 seconds. Brown ear
tips, rolled forelock, narrow face, raised hands/claws, grounded feet and
scalloped rear wing lobes read at source size. Tail curl and opposite wing are
partly occluded from this camera. Native pink is muted relative to the source
back sprite. Source-cue motion, browser and world review remain open; full
model/rigging acceptance is unfinished.

The new sharing test first used an incorrect crate-root actor module path,
which was corrected before the local test build. The GPU integration passed,
but the sharing fixture then caught an assumed battle-directory path for the
actor-space canonical GLB. That fixture now points to the actual actor asset.

Both Clefairy Rust integration checks pass: canonical icon/species neutral
sharing, planted feet/toenails throughout clips, and eleven-joint GPU skin
activation at 0.6096 m. The compiled executable passes all 28 active species
rig regressions (one existing audit ignored), all 24 skinning regressions and
the 238-species reusable model check. The broader actor suite initially found
an outdated unsupported-Pidgey assertion; it now recognizes Pidgey and Clefairy
as canonical while retaining unsupported-species guards. All 12 actor tests
pass on the fresh retry. Targeted whitespace checks pass.

There are 21 registered standard species skins. This does not prove all 251
species have improved rigs or accepted production visuals. The full goal
remains active, with remaining rigs and visual fidelity checks unfinished.


## Jigglypuff: preserve shared balloon, add seven-joint skin

Jigglypuff's shared sculpture is now one canonical
`models/actor_props/icon_jigglypuff.glb` reused by its legacy icon and explicit
species paths. The former JSON was removed. The old and new float32 neutral
digests match exactly:
`be7a901fb200b5528a7ec3fbba34c9944776fdb953d1eb15b69f58cf9419664e`.
Two regenerated ear Y components required signed-zero canonicalization;
positions, topology, colors and every original float32 normal bit are preserved.
The asset has 22 closed parts, 13,706 vertices, 22,956 triangles, seven joints
and 627,952 bytes. Details are in `jigglypuff-sculpture.md`.

The first deformation audit caught a tiny arm-cap inversion under blended
weights. Giving each short arm one rigid owner preserved topology while
retaining shoulder motion. The final seven GLB checks and all eight existing
sculpture checks pass. Focused canonical routing passes. The pure sculpture
CLI was executed and its output matches the canonical GLB byte-for-byte,
without JSON. Modified authoring/routing scripts parse. Blender is unavailable;
its batch route was updated and syntax-checked, not executed. The native
client rebuild succeeded.

The native catalog reports 239 dedicated catalog bodies among 251 packed
species, with 12 using shared or fallback bodies outside that catalog. This
is not a claim that only twelve bodies need quality or rigging work.
Jigglypuff already had a detailed shared sculpture; this step adds its rig and
explicit catalog entry while preserving its geometry. Shininess remains the
existing flag and pack palette on the same skin.

The held native pair `target/pokemon-native-review/jigglypuff-3d.png` /
`jigglypuff-2d.png` was inspected. Receipt: `state_preserved=true`, hash
1141753145, LCD animation frame 37, virtual time 0.953778083 seconds. Rounded
ears with dark linings, rolled forelock, layered eyes, short arms and grounded
feet remain readable at source size. Native pink is muted compared with the
source back sprite; the eyes read teal. The source view is its back sprite.
Source-cue motion, facial/singing sprite fidelity, browser and world review
remain open; the full model/rigging goal is unfinished.

Both Jigglypuff Rust integration tests pass: shared canonical icon/species
neutral geometry, fixed feet through clips and seven-joint GPU skin activation
at 0.508 m. The fresh executable passes all 29 active species rig regressions
(one existing audit ignored), all 25 skinning regressions and all 12 actor tests,
including the 239-species reusable model check. Targeted whitespace checks pass.
There are now 22 registered standard species skins. This remains progress
within the full goal, not proof that all 251 species have improved rigs or
accepted production visuals.


### Named NPC identity correction (2026-10-13)

The publisher now resolves Copycat's Dodrio, Mr. Fuji's Pidgey/Psyduck/Nidorino,
and the Route 28 celebrity's Fearow to their authored species. Crystal reuses
`SPRITE_MOLTRES`, `SPRITE_RHYDON`, and `SPRITE_GROWLITHE` icon art for these
objects; those tokens do not establish the NPC species. Exact map, object,
script, sprite and bitmap guards preserve replacements and unrelated objects.
The dialogue and cries already name the correct Pokemon and remain unchanged.

The regression checks the five identities against dialogue and cry commands
in the external pack and resolves its materialized art like the production
client. Native paired 3D/2D previews at noon, orbit 1, zoom 1 show the corrected
Dodrio, Pidgey, Psyduck, Nidorino and Fearow. All three pairs preserve state;
checksums are 96276739 (Copycat), 2848103444 (Fuji), and 3373714584 (Route 28).
This accepts the identity correction, not every species' geometry, scale,
materials or animation. Generic dolls and other shared-icon NPCs remain
separate review work.


### Shared NPC icon audit continuation (2026-10-13)

The guarded identity mapping now covers all 15 named NPCs using the shared
Moltres, Rhydon and Growlithe sprite tokens whose scripts identify a different
species. The additional ten are Charcoal Kiln's Farfetch'd, the Rocket base's
Murkrow, Viridian's Speary (Spearow) and Rattey (Rattata), Pewter's male Nidoran,
National Park's Persian, both Meowth NPCs, Celadon's female Nidoran, and Cerulean's Kangaskhan.
The two actual Rhydon NPCs keep their existing identity. The pack-backed test
also checks each object's `writetext` link; Murkrow is established by its named
text because its interaction deliberately has no cry. Nicknames and both
Nidoran text prefixes are preserved.

The identity and exact-mesh regressions passed. A newly built production
`new_bark_3d` captured and was inspected against paired 2D at all eight new
locations, at noon, orbit 1 and zoom 1. All eight receipts report preserved
state. Farfetch'd and Murkrow are visible in these fresh sessions; no story
flags were changed. The previews establish correct species selection, not
final acceptance of existing geometry, scale, materials or animation.
Rattey correctly keeps a source sprite card: Rattata has no dedicated mesh or
standard skin yet and is the next model gap to address. Registered skins stay
at 22; the full all-Pokemon model and rigging goal remains unfinished.

Verification commands:

```sh
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/core-modular.browser.crystalpack" \
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-bevy compiled_named_pokemon --lib
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view published_named_pokemon --lib
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked -p crystal-bevy --features location-tester --example new_bark_3d
```


### Rattata model and nine-joint skin (2026-10-13)

Rattata now has an original canonical battle-space GLB and pure authoring
recipe, replacing its missing dedicated body and Rattey's sprite-card fallback.
The model has a low purple quadruped barrel, fitted face and ear bowls, two
incisors, six whiskers, four planted paws and a raised curled tail. Nine joints
articulate torso, neck, head, ears and three tail sections. Existing source
cues own the idle/attack/hit response; the floor root and paws stay fixed.
The body material was brightened after a production comparison against 2D.

The 42-part model has 11,494 vertices, 20,432 triangles, 569,804 bytes and a
0.82-unit neutral silhouette. The external pack confirms one-foot Rattata
height (0.3048 m). Neutral geometry/material SHA-256:
`ce7c14e7cf1694a4131d3ed3e3c709fe51f9b2ff7618089d3c40bf731aeb8a3d`.
There is one canonical GLB, with no duplicate JSON or generic fox body.

All seven Python topology/anatomy/skin checks and the focused canonical-routing
check pass. Both Rattata Rust integration checks pass. The fresh test executable
passes all 30 active rig regressions (one existing audit ignored), 26 skinning
regressions and 13 actor/model regressions, including the 240-body catalogue.
Final native paired battle and Viridian 3D/2D previews were inspected and
preserved state. Battle hash 567313881 / LCD 36 / time 0.908207625; Viridian
hash 2556627940 / LCD 40 / time 0.935438249.

The battle proof uses the supported wild fixture because the attempted Vance
trainer fixture failed pack event-flag validation; that mismatch remains open.
Browser rendering and detailed source-cue visual review remain open. There are
now 23 registered standard skins and 240 dedicated catalogue bodies. Eleven
species remain outside this catalogue; all-species model and rigging completion
is not established. See `rattata-sculpture.md` for verified commands and limits.


### Raticate preserved sculpture and eight-joint skin (2026-10-13)

Raticate's existing 63-part pear-shaped brown sculpture now uses one canonical
standard-skin GLB. Its f32 positions, normals, colours, triangle order and named
parts remain bit-for-bit unchanged. The head/body and fitted face move together;
ears, forepaws and bare tail articulate while the splayed feet and toe claws
stay planted. Idle/attack/hit channels are rotation-only and source-cue timed.
The migration follows the existing self-contained Gengar authoring pattern.

Neutral digest `f6043d52ed1588580d3e4c8f44480d8d3cb2b8b2f268e6ccd7220287d76edd0d`;
5,642 vertices, 7,764 triangles, 283,264 bytes and a 0.96-unit neutral silhouette.
The external pack confirms height digits 204, giving a 0.7112 m battle model.
Identical index and skin accessors share storage: 247 accessors fit the unchanged
256-accessor limit without merging parts or reducing anatomy. Canonical pure
CLI re-export is byte-for-byte identical. The retired JSON is gone, and source
catalogue/build routing prevents its duplicate recreation. The full Blender
batch was syntax checked, not run.

Seven Python model/skin checks, the focused canonical-routing check and both
Raticate Rust integration checks pass. The fresh Rust executable passes 31
active rig tests (one existing audit ignored), 27 skinning tests and 13 actor
regressions, including all 240 dedicated catalogue bodies. A freshly built
production native battle preview was compared with paired 2D: preserved state,
hash 2486622478, LCD frame 36 and virtual time 0.88753225. The brown pear body,
incisors, forepaws and cream bare tail remain readable and distinct from Rattata.

There are now 24 registered standard skins. The 240-body catalogue count is
unchanged by this migration. Browser rendering, detailed source-cue visual
motion and the full all-Pokemon improvement/rigging goal remain unfinished.
See `raticate-sculpture.md` for the verified commands and production limits.


### Psyduck/Golduck standard skin pair (2026-10-13)

Both original duck anatomies now have canonical standard-skin GLBs, with neutral
f32 geometry, normals, colours and triangles preserved exactly. Psyduck's temple
hands, broad bill and hair differ from Golduck's webbed paddles, cranial fins,
ruby and long tail. Their fused torso/head fields, lower bills, arms/hands,
crests and tails articulate while all webbed feet and claws stay on the root.
Psyduck has ten joints and Golduck eleven. No reader limits were widened;
shared accessors keep the assets to 131 and 186 accessors respectively.

Psyduck: 28 parts, 3,754 vertices, 3,828 triangles, 184,188 bytes; neutral digest
`33ee456438cd902af1eccd5e00ce9b4f59a1a88bc398b812ab619e39f2dbafaa`.
Golduck: 45 parts, 4,974 vertices, 4,460 triangles, 235,196 bytes; neutral digest
`204f972548ff6318987cf7688ba9943a700fd2191ad20d564ff9e2aae284c976`.
Both authored silhouettes are 1.03 units tall. The external pack confirms
Psyduck height digits 207 (0.7874 m) and Golduck 507 (1.7018 m).

All 11 stored-anatomy/skin checks, the focused canonical-routing check and both
Rust integration checks pass. Both pure authoring outputs were executed,
compared byte for byte with the canonical assets, and verified to remove stale
JSON. The full Blender batch integration was syntax checked, not run. The fresh
Rust executable passes 32 active rig regressions (one existing audit ignored),
28 skinning regressions and 13 actor regressions, including the 240-body
catalogue. Native paired battle views for both ducks and Mr. Fuji's Psyduck
were inspected and all three preserved state. Receipts and commands are in
`duck-skins.md`.

There are now 26 registered standard skins; dedicated catalogue bodies remain
at 240. The preserved materials render Psyduck warmer gold and Golduck more
teal than their source sprite colours. Colour fidelity, browser rendering and
detailed source-cue visual motion remain open. The full all-Pokemon model and
rigging goal is unfinished.


### Duck body palette correction (2026-10-13)

Psyduck’s authored gold and Golduck’s teal body materials now use the normal
palette anchors read from the verified local pack: RGB555 `(31, 31, 0)` and
`(8, 11, 24)`, respectively. Runtime bit replication produces RGB8
`(255, 255, 0)` and `(66, 90, 198)`. Accent materials, geometry, skinning,
hierarchy and clips were independently compared with the preceding GLBs and
remain unchanged. The shared lighting and shiny appearance path are unchanged.

Current material-inclusive neutral digests:

- Psyduck: `4d28e9c51e3d5ed94fcccfe356d5cc60dc9ee2d7645332334c7bb7d6ef5b9b4c`; 184,140 bytes.
- Golduck: `e7ab59e4de9d3d78f761557cc5b62d6e41442f54c3e25b7fffbecba18892178d`; 235,196 bytes.

All 12 Python duck regressions and both focused Rust duck checks pass. The
fresh Rust test executable passes 32 active rig tests (one existing ignored),
28 skinning tests and 13 actor tests. A fresh production build and three paired
3D/2D captures verify yellow Psyduck and indigo Golduck in battle, plus yellow
Psyduck at Mr. Fuji’s house. Each pair preserves state; capture receipts and
commands are recorded in `duck-skins.md`. The crown/ruby, bills, webbing and
hair remain readable. Browser rendering and detailed motion acceptance remain
open. Registered skins remain 26 and dedicated catalogue bodies 240; the
all-Pokemon model and rigging goal remains unfinished.


### Zubat, Golbat and Crobat standard skins (2026-10-13)

Three static bat anatomies now use canonical standard glTF skins. Original f32
geometry, normals, materials and triangles were preserved. The complete hollow
face, mouth, eyes and teeth keep one body owner; ears/insets share ear bones.
Each upper wing has shoulder, wrist and tip joints, with the same continuous
weight field across membrane and spars. Crobat has four additional lower-wing
joints. Zubat/Golbat have ten joints and Crobat fourteen; their assets remain
within the existing byte, vertex and accessor bounds. Retired JSON files were
removed, and both pure authoring and batch export paths publish GLB.

The first in-plane wrist bend inverted a narrow contour triangle during idle.
Out-of-plane wrist/tip bending replaced it, and all 12 stored bat anatomy/skin
checks passed, including deformation winding and strain. The routing, registry
and GPU checks pass at verified external-pack heights. The broader Rust suites
pass 33 active rig tests (one existing audit ignored), 29 skinning tests and
13 actor tests. The pure authoring CLI matched canonical bytes and removed
stale JSON. Full Blender batch execution remains unverified.

Fresh native paired battle captures were inspected for all three, and every
pair preserved state. Hollow mouths, Zubat’s eyeless face and Crobat’s lower
wings remain present. The side camera compresses wing silhouettes, and the
preserved Zubat/Golbat materials look teal against their blue source sprites.
Palette fidelity, wing presentation, browser rendering and detailed source-cue
motion acceptance remain open. The broad Python discovery separately exposed
a Chikorita writer/asset mismatch; it was not counted as a passing suite.
Details, hashes and commands are in `bat-skins.md`.

There are now 29 registered standard skins and 240 dedicated catalogue bodies.
The all-Pokemon model and rigging goal remains unfinished.


### Chikorita canonical rebuild repair (2026-10-13)

The broad model routing audit exposed a Chikorita writer/GLB mismatch. Direct
payload comparison localized it to tiny axis-aligned coordinates and normals,
with a maximum writer/stored discrepancy of about `1.24e-15`. The neutral
recipe now serializes position/normal components below `1e-12` as exact zero.
A guarded old/new audit allowed only those tiny components to change; all other
coordinates/normals, indices, materials, skin data, hierarchy and animation
payloads were preserved. No new anatomy or motion is claimed for this repair.

The canonical asset remains 320,824 bytes, 19 parts, 7,463 vertices, 3,220
triangles and 11 joints. Its current material-inclusive neutral digest is
`f2a79d3d20fc56f5584b35c3e7655b0d0654ba12273775dc0e62cca7a8bd9774`.
The canonical reader and pure authoring CLI were executed and reproduced
identical bytes. All 34 Python checks pass: 11 Chikorita geometry/skin/clip
checks and 23 canonical model routing checks, including the previously failing
starter check. Both focused Chikorita Rust checks pass; all 33 active rig
regressions pass (one existing audit remains ignored).

A fresh production build was inspected against paired 2D using the wild-species
fixture. The raised head, fitted dark eyes, seated collar buds, four rooted feet
and curved crown leaf remain readable. Physical scale and appearance retain
the preceding model. The paired receipt reports preserved state, checksum
3504188543 at LCD frame 36, virtual time 0.839257999. This validates the rebuild
repair; browser and detailed source-cue visual motion acceptance remain open.
Registered skins remain 29 and dedicated catalogue bodies 240. The full goal
remains unfinished.

```sh
PYTHONPATH=tools python3 -m unittest test_chikorita_glb test_battle_model_assets -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked \
  -p crystal-voxel-view chikorita --lib
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked -p crystal-bevy \
  --features location-tester --example immersive_battle_3d
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
./target/pokemon-models/debug/examples/immersive_battle_3d \
content-packs/core-modular.browser.crystalpack --wild-species CHIKORITA --size 1000x700 \
--screenshot target/pokemon-native-review/chikorita-zero-battle-3d.png \
--second target/pokemon-native-review/chikorita-zero-battle-2d.png
```


### Butterfree and Beedrill standard skins (2026-10-13)

Two original static insect anatomies now use canonical standard glTF skins.
Their f32 geometry, normals, materials, named parts and triangles were
preserved. Butterfree has 18 joints and Beedrill 20. Four wing fields keep
rims, membranes and veins together; antennae have two-joint chains, fitted
compound eyes follow the head, and Beedrill has separate drill joints.
Butterfree's foot pads stay rooted. Beedrill's gaster/stripes have one owner,
with a blended petiole and a rooted low stinger tip. Reader limits were not
widened: Butterfree has 254 accessors and Beedrill 217. Asset details and
neutral digests are in `wing-insect-skins.md`.

Initial in-plane wing motion inverted a narrow perimeter triangle. The final
out-of-plane hinge motion passes all 12 Python anatomy/skin/routing checks,
including winding, edge strain and floor clearance. Both focused Rust registry
and GPU integration checks pass at the verified local-pack heights. The
skinning suite passes 30 checks and the actor suite 13. The pure authoring CLI
matched both canonical files and removed seeded obsolete JSON. The full
Blender batch remains unverified.

A broad rig run hit five anatomy-count errors after a concurrent Totodile
update changed its on-disk model from 27 to 30 parts after the executable was
built. A fresh Cargo rig-suite run against current code/assets passes all 34
active tests (one existing audit ignored). The same fresh executable passes
30 skinning checks and 13 actor checks. Fixture errors now identify the species
so future mixed build/file snapshots are easier to diagnose.

A newly built production client was inspected against paired 2D for both
insects. Both pairs preserve state. Wing veins, compound eyes, blue foot pads,
forearm lances and gaster stripes remain readable. The preserved materials
differ from the source back-sprite palettes, and this camera angle compresses
the far wings. Palette fidelity, wing presentation, browser rendering and
detailed source-cue motion acceptance remain open. These previews verify
rig integration, not final all-species visual acceptance.

There are now 31 registered standard skins. A new verified-pack coverage audit
confirms 251 species versus 240 dedicated battle bodies. The eleven battle
catalogue gaps are Diglett, Entei, Hoothoot, Onix, Raikou, Sentret, Slowpoke,
Staryu, Suicune, Unown and Voltorb. Several already have separate world/doll
assets; Hoothoot and Sentret lack such named assets in this checkout. Closing
these gaps and improving the remaining static rigs are still part of the full
all-Pokemon goal.


### Hoothoot body and standard skin (2026-10-13)

Hoothoot fills a battle-catalogue body gap with a new original round owl, fitted
clock-hand brows and eyes, folded wings, beak, breast chevrons and tail fan.
One anatomical foot supports the rest stance and the second is tucked behind
the lower body. Ochre/navy materials match the verified local pack’s RGB555
palette anchors; physical height is 0.7112 m (height digits 204). The canonical
GLB has 38 parts, 16,597 vertices, 25,884 triangles, 12 joints, 190 accessors and
733,832 bytes. Neutral digest:
`93fbea6a47abff0fff1f6ee6315f7ac09ea26bb1bf60a26b0d293a8ae7784555`.

The complete face follows the body, with lower root support shared by the
breast marks. Clock brows and folded wings have root/tip fields; the tail and
jaw articulate. All legs/feet/toes/claws retain the floor root. Eight stored
anatomy/skin checks and two routing/catalogue checks pass. The pure authoring
CLI reproduced canonical bytes; the Blender batch was syntax checked only.
Final fresh Rust suites pass 35 active rig tests (one existing ignored),
31 skinning tests and 13 actor tests. Reader bounds were not widened.

The first native preview revealed eye-cap penetration into the body. Refining
only the eye caps and adding a stored-surface clearance check fixed it. A
freshly rebuilt repeated native/2D pair shows complete eye rings, preserved
state and a readable owl silhouette at pack scale. Its checksum is 3627163496,
LCD frame 53, virtual time 1.27802025. Browser rendering, additional angles and
detailed source-cue motion remain open; not every six-frame source pose is
claimed to match the authored one-foot rest stance. Details and verified
commands are in `hoothoot-sculpture.md`.

There are now 32 registered standard skins and 241 dedicated battle bodies.
Remaining catalogue gaps are Diglett, Entei, Onix, Raikou, Sentret, Slowpoke,
Staryu, Suicune, Unown and Voltorb. The full all-Pokemon goal remains active.


### Sentret body and support-tail skin (2026-10-13)

Sentret fills another dedicated battle-body gap with a canonical 38-part,
13-joint GLB. Its white belly annulus leaves the brown center visible; one tail
chain supports the body and another carries the free curled tip. The pack's
normal colors and 0.7874 m height are verified. The first geometry review led
to a rounded tail cap; the first native review led to rounded ear shells.

Eight final stored anatomy/skin tests, both focused Rust registry/GPU-skin
checks and the 33-species schema check pass. Broad suites before the final ear
correction passed 36 rig checks with one existing ignored check, 32 skin checks
and 13 actor checks. The rebuilt native paired battle view shows the corrected
ears, belly ring, raised paws, suspended feet and striped tail support. State
is preserved, checksum 3743123137, LCD frame 95, virtual time 2.785530208.
Details and remaining visual/motion work are in `sentret-sculpture.md`.

There are now 33 registered standard skins and 242 dedicated battle bodies.
Remaining catalogue gaps are Diglett, Entei, Onix, Raikou, Slowpoke, Staryu,
Suicune, Unown and Voltorb. The all-Pokemon model and rigging goal remains active.

### Visible species labels (2026-10-13)

Native world Pokemon now carry compact nameplates above their actual rendered
mesh bounds. The names consume the publisher's resolved `species:` identity,
including the shared Moltres/Rhydon/Growlithe NPC art. Camera projection follows
the current view; inactive world views retire the labels. Field prompts and
optional object debug labels use the same identity and species display spelling.
The Bevy shell enables its bundled default font so these labels render.

The external-pack test checks names against dialogue, cries and text links for
all 15 named shared-art NPCs. Four identity/replacement regressions pass. A
rebuilt native paired Fuji view visibly labels Pidgey, Psyduck and Nidorino;
the plates are readable, separate and clear of their faces. The final paired
receipt preserves state, checksum 856260372, LCD frame 62, virtual time
1.663009583, at noon, orbit 1 and zoom 1. Browser and dense crowd label layouts
still require separate visual review. The ordinary 2D
reference retains its existing sprite presentation.


### Staryu body and radial skin (2026-10-13)

Staryu fills another battle-body gap with an original continuous five-arm lens,
fitted arm ridges, a raised gold setting and a faceted red core. Its canonical
10-part, 12-joint GLB is shared by the world icon; the old icon JSON is removed.
Five arm chains articulate while the two lower tips remain anchored. Body and
core colors and the 0.7874 m height come from the verified local pack.

Nine stored anatomy/skin tests, two canonical-routing tests, 37 active Rust rig
checks with one existing ignored check, 33 GPU-skinning checks and five
actor-prop checks pass. The pure CLI reproduces canonical bytes. Builder
branches were syntax checked, with full Blender batches still unexecuted.

A rebuilt native paired Staryu/default-wild battle preserves state, checksum
3691665458, LCD frame 54, virtual time 1.296023334. Its body, collar and ruby are
visible. The default side camera compresses the flat star silhouette; native
composition acceptance across camera angles, browser rendering and detailed
source-frame motion remain open. Details are in `staryu-sculpture.md`.

The catalogue now declares 34 standard skins and 243 dedicated battle bodies.
Remaining gaps are Diglett, Entei, Onix, Raikou, Slowpoke, Suicune, Unown and
Voltorb. The full all-Pokemon model and rigging goal remains active.


### Voltorb shared body and grounded orb skin (2026-10-13)

Voltorb fills another battle-body gap using its original closed hemispheres and
fitted stern eyes. One canonical eight-part GLB replaces the world icon JSON
and serves both world and battle routes. The support root remains fixed; one
coherent body bone rocks the shell and face while the red/white equator stays
welded. Red, white and black materials match the verified pack; neutral f32
positions, normals and triangle order were preserved. Electrode is unchanged.

Six new stored-skin checks, six orb anatomy checks, two canonical-routing checks,
38 active Rust rig checks with one existing ignored check, 34 GPU-skinning checks
and five actor-prop checks pass. The pure CLI reproduces canonical bytes and
emits no duplicate Voltorb JSON. Both builder branches were syntax checked.

The rebuilt native paired view preserves state, checksum 396955799, LCD frame
37, virtual time 0.921693208. Voltorb's deep red cap, white underside and near
stern eye remain readable, and the equator stays continuous. Detailed motion,
additional camera angles and browser acceptance remain open. Details and
verified commands are in `voltorb-skin.md`.

There are now 35 standard skins and 244 dedicated battle bodies. Remaining
catalogue gaps are Diglett, Entei, Onix, Raikou, Slowpoke, Suicune and Unown.
The all-Pokemon model and rigging goal remains active.


### Lapras pack blue/tan correction (2026-10-13)

A fresh native review confirmed that Lapras still used teal rather than the
source blue. Body materials now match the pack's RGB555 `(8,16,28)` blue and
`(28,21,13)` tan; spots/brows follow the same blue hue. An executed GLB comparison
proves the complete binary geometry, weights, inverse binds and clip data
unchanged, as well as nodes, mesh schema and animation channels. Gray shell,
rim and eye materials are preserved. The new neutral digest is
`8695d9198539d60d3aaaaea4daf8dfb27f129c3a57c6ef5259af9fa807ec217e`.

Six stored-model checks, a new independent pack-color check and both focused
Rust integration checks pass. The rebuilt native paired view preserves state,
checksum 3736814282, LCD frame 37, virtual time 0.942495959, and shows the blue
body and tan underside clearly. Shape/composition, detailed source-frame
motion, legacy world props and browser fidelity remain open. This accepts the
held-view color correction, not complete Lapras acceptance. Details are in
`lapras-sculpture.md`; catalogue counts remain 35 skins and 244 battle bodies.


### Onix shared rock-chain body and skin (2026-10-13)

Onix fills another battle-body gap using its original thirteen-rock sculpture.
The canonical 36-part GLB replaces the `big_onix` actor JSON and serves both
world and battle routes. Ten rising-chain bones and one coherent head bone
follow an identity floor root; the first three tail rocks remain fixed. Buried
connectors blend adjacent owners. Neutral f32 geometry and triangle order were
preserved, and base stone/dark materials match the verified pack.

The first native view exposed unwanted alternating dark boulders; all rocks
now use the base stone color while joints/jaw retain their darker shade. That
correction preserved the complete binary geometry, weights and clip data.
Eight final skin checks, eleven source checks and two routing checks pass.
Broad Rust suites before the final material correction passed 39 active rig
checks with one existing ignored check, 35 skin checks and five actor-prop
checks; both focused Onix integration checks pass afterward. The pure CLI
reproduces canonical bytes. Full Blender batches remain unexecuted.

The rebuilt repeated native pair preserves state, checksum 3502953395, LCD
frame 36, virtual time 0.894852042. The rock chain, face and horn remain readable
at the pack's 8.7884 m body length, scaled through its authored centerline.
Detailed source-frame motion, more camera angles, world-specific motion and
browser fidelity remain open. Details are in `onix-skin.md`.

There are now 36 standard skins and 245 dedicated battle bodies. Remaining
catalogue gaps are Diglett, Entei, Raikou, Slowpoke, Suicune and Unown. The full
all-Pokemon model and rigging goal remains active.


### Diglett capsule and fixed-soil skin (2026-10-13)

Diglett fills another battle-body gap with a new continuous capsule and fitted
face emerging through separate soil. One 15-part GLB replaces the old icon
JSON and serves both battle and world aliases. The root keeps soil and the
buried base fixed; the visible body and face share one bone. Pack colors and
the 0.2032 m height are verified. Both scale and source projection now use the
measured capsule span, excluding soil.

The first front review showed a pointed cap, corrected to an elliptical dome.
The first native view showed a regular smooth soil ring, corrected to varied
faceted clods. Nine final stored checks and two routing checks pass. Broad Rust
suites before these final geometry corrections passed 40 active rig checks with
one existing ignored check, 36 skin checks, five actor-prop checks and 21 layout
checks; all three focused Diglett integration checks pass afterward. The pure
CLI reproduces canonical bytes. Full Blender batches remain unexecuted.

The rebuilt repeated native pair preserves state, checksum 3361697257, LCD
frame 55, virtual time 1.391161. The small capsule, nose, near eye and irregular
clods remain readable beside Sudowoodo. Detailed source-frame motion, more
angles, world-specific motion and browser fidelity remain open. Details are
in `diglett-sculpture.md`.

There are now 37 standard skins and 246 dedicated battle bodies. Remaining
catalogue gaps are Entei, Raikou, Slowpoke, Suicune and Unown. The full
all-Pokemon model and rigging goal remains active.


### Slowpoke body and standard skin (2026-10-13)

Slowpoke now has a dedicated sleepy quadruped battle body and an 11-joint
standard skin. Both world aliases share the canonical GLB, replacing their
old sphere proxies. The low torso, broad head, blank stare, rounded ears,
open pale muzzle, four clawed feet and continuous raised pale-tipped tail
use the verified pack palette and 1.1938 m height. Rotation-only idle/attack/hit
clips articulate the head, ears, legs and tail while feet remain planted.

Eight stored-model checks, two routing/catalogue checks, pure CLI byte equality,
41 active rig checks (one existing ignored), 37 GPU-skinning checks and five
actor-prop checks pass. Builder branches are syntax checked; full Blender
batches were not executed. Front and oblique neutral reviews and the rebuilt
native 3D/2D battle pair were inspected. The held pair preserves state, checksum
1352458233, LCD frame 48, virtual time 1.25993175. The native camera keeps the
whole body in frame, grounded, with the face and tail readable beside Sudowoodo.

See `slowpoke-sculpture.md` for anatomy, ownership, digest and verification.
There are now 38 registered standard skins and 247 dedicated battle bodies.
Entei, Raikou, Suicune and Unown remain catalogue gaps; remaining static rigs,
source-frame motion, further native angles and browser fidelity keep the full
all-Pokemon objective unfinished. Temporary source sprite copies and the pack
reader helper were removed. Packs and audio are unchanged.


### Unown A–Z battle bodies and glyph skins (2026-10-14)

Unown now has all 26 authored letter silhouettes and standard three-joint skins.
A presentation-only form field comes from the same selected DVs as the source
picture, including Transform's picture-change boundary. Geometry, bounds, rigs
and actor cache identity use the exact letter; species labels and physical
measurements remain Unown. Unresolved forms retain source art. The generic world
icon shares the I body, replacing its old JSON proxy.

The palette and 0.5080 m height come from the verified external pack. The eye
lens and fitted pupil stay coherent; the reverse side is dark like the backpics.
Closed secondary loops use torus topology, and B/G/O were rounded after source
comparison. Eight final stored-model checks, three routing checks and exact CLI
reproduction pass for all forms. Production extraction checks cover every DV
letter and both Transform sides. The API key check passes.

Thin glyphs needed the existing finer 90 Hz envelope sampling to satisfy the
unchanged camera-padding budget. The full suite then passed 42 active rig checks
(one existing ignored). After the final O correction, four focused rig/GPU/form
replacement/envelope checks and the complete schema check pass. Five actor-prop
checks pass. Builder branches were syntax checked; full Blender batches were
not executed.

Initial native views were almost edge-on. A letter-plane hint now opens only
Unown toward the establishing camera while preserving opponent-facing stance,
physical size, support, hit registration and the other participant's pose.
All 22 layout and 40 GPU-skinning checks pass after this correction. The pose
stays fixed during the shot; terrain validation remains authoritative.

Repeated paired native 3D/2D views were inspected for A–Z plus shiny I/V. All
28 pairs preserve state. Letters, loops and branches remain readable and within
frame at the pack size. Shiny I/V use the same geometry and skins with the
existing palette flag. Representative final checksums: A 4060871309, O 99271016,
shiny I 2943203603 and shiny V 1045820805. Held views do not establish full
source-frame motion, hovering, world-specific letter motion or browser fidelity.

See `unown-sculpture.md` for details and runnable commands. The catalogue now
has 248 dedicated bodies; Entei, Raikou and Suicune remain gaps. The standard rig
registry has 64 variants covering 39 Pokémon species. Remaining static rigs,
source-frame motion and visual requirements keep the complete goal active.
Temporary source sprites and the Rust pack reader were removed. Packs and
audio were not changed.


### Entei armored body and 16-joint skin (2026-10-14)

Entei now has a dedicated battle body shared with its world alias. The old
feline sphere proxy is replaced by a heavy torso, raised neck, hanging mane,
pointed bib/beard, red cheek plates, silver muzzle shield, swept amber crown,
fitted stern eyes, broad pale paws, thick ankle bands, six dorsal blades and
white smoke trails. Pack palette anchors and height 2.0066 m remain authoritative.

The first native view showed card-like face/mane panels. They now wrap around
the head and shoulders; the muzzle is seated closer and crown tips sweep back.
Eight final stored-model checks and CLI byte equality pass. Before that
correction, 44 active rig checks (one existing ignored), 41 GPU-skinning checks,
five actor-prop checks and two routing checks passed. Afterward, focused Entei
registry/GPU checks, the complete schema check and routing/CLI checks pass.
Builder branches are syntax checked; full Blender batches were not executed.

Fresh repeated normal/shiny native 3D/2D pairs were inspected. Both preserve
state: normal checksum 979079861, LCD frame 54, virtual time 1.330801542;
shiny checksum 1399097452, LCD frame 52, virtual time 1.303138791. The whole
body and smoke fit the view, the mask/crown/coat read as integrated volumes,
and all paws remain planted. Shiny Entei reuses the same geometry and rig
with the existing palette flag.

See `entei-sculpture.md` for anatomy, digest, evidence and commands. There are
now 249 dedicated battle bodies; Raikou and Suicune remain catalogue gaps.
The standard rig registry has 65 variants covering 40 Pokémon species.
Detailed source-frame motion, remaining static rigs, further native/world
views and browser fidelity keep the full goal unfinished. Temporary source
sprites, the official-art reference and the Rust reader were removed. Packs
and audio were unchanged.

### Raikou and Suicune: battle-body catalogue completed

Raikou and Suicune now use canonical standard skins in battle and world prop
routing. Their old world JSON proxies and dead proxy-builder geometry are
removed. Both focused Blender build paths reproduced each reviewed GLB byte
for byte. Raikou has 16 joints and Suicune has 18, including separate mane and
ribbon chains. Pack-derived scales are 1.9050 and 2.0066 metres respectively.
Their stored geometry, support ownership, deformation, and source palette
checks pass. The standard registry now has 67 variants across 42 species.

The canonical routing test reports 251 dedicated battle bodies. This closes
catalogue coverage, not complete rigging: many bodies remain static, and
source-frame motion, world views, browser fidelity, and further visual
refinement remain part of the full goal. The registry schema, all 43 GPU
skinning tests, and all five actor-prop tests pass with Suicune registered.
Both legends' normal and shiny native captures were inspected against their
matching 2D states, with `state_preserved=true` for every pair. See
`raikou-sculpture.md` and `suicune-sculpture.md` for anatomy and receipts.

The Totodile editable-source failure was a compression mismatch. A read-only
Blender export reproduced the runtime neutral digest exactly; `store_source`
normalized the existing Zstandard save to the required gzip container and
updated its identity. Other source records and the runtime Totodile GLB were
preserved. Source-identity verification passes. The broader model checker
also had a stale Totodile count and two icon-name expectations; those were
corrected to match the strict readers and verified production assets.

The full authoring checker is still not green: Bayleef's stored GLB differs
from its Python export in one position accessor. The largest numeric change
is approximately 3.7e-18 across 288 values; JSON differs only in the neutral
geometry digest. This needs a deterministic authoring repair before claiming
complete source validation. Packs and audio were unchanged; build products
remain ignored. References and scratch exports were removed after review.

### Bayleef portable axis coordinates

The Bayleef source/export mismatch was traced to tiny axis residues from cage
construction. Its authoring recipe now snaps position magnitudes below 1e-15
to exact zero after topology construction. The shipped GLB was regenerated
and its registry digest updated to
`3a47bbf1035133575ef25812194da3caec7ea9596b6cc9a5261317acec7864c0`.
A comparison against the prior artifact found 303 changed coordinate values,
with maximum displacement 4.5283909989986894e-17. Normals, indices, materials,
weights, joints, inverse bind matrices, and every animation key were preserved
byte for byte. Shape, scale, contacts, and motion are therefore preserved.

Python 3.9 and Python 3.14 now generate identical 427,600-byte GLBs. All eleven
stored-model regressions and both focused production registry/GPU tests pass.
The rebuilt native 3D/2D pair was inspected: scale, fitted eyes, collar leaves,
notched crown, floor contacts, and overall silhouette are preserved. It reports
`state_preserved=true`, checksum `3123604715`, LCD frame 64, virtual time
`1.610970792`. The full authoring check now clears Bayleef and stops at Meganium.
Its remaining position drift is approximately 3.5e-18; its collar-normal
difference is already within the reviewed reader's 1e-14 tolerance. That next
deterministic-source repair and broader rigging work remain outstanding.

### Meganium portable axis coordinates

Meganium's remaining recipe drift was likewise traced to tiny body-axis
coordinates. The recipe and reviewed stored position streams now snap
magnitudes below 1e-15 to exact zero after topology construction. A comparison
against the previous GLB found 534 changed coordinate values, with maximum
movement 8.927674986245612e-17. Every other accessor, including collar normals,
skin weights, inverse binds, and animation keys, is preserved byte for byte.
The reviewed-container guard and its existing 1e-14 normal tolerance are
unchanged. The new neutral digest is
`9573b5a8f0df3ae27e03195807cf2ca92cfff5067d3f83af6ea294a9d7712643`;
the reviewed file hash is
`c1b1396416dfb26c141402d9f9f77c1de841fa3431a945ab480810ef5ec7d347`.
Python 3.14 validates the reviewed stored geometry. All fourteen stored-model
tests and both focused production registry/GPU tests pass, including planted
contacts, flower/jaw clearance, smooth endpoints, and deformation limits.
The native capture example was rebuilt and its updated 3D/2D pair inspected.
The complete silhouette, fitted face, six flower petals, antennae, scale, and
floor contacts are preserved. The pair reports `state_preserved=true`, checksum
`2868564361`, LCD frame 55, at virtual time `1.498212875`. The full authoring
check now clears Meganium and reaches a reference-validation assertion for
`assets/open-models/interiors/bookcase.mesh.json`, outside the model directory.
That checker/source-layout discrepancy remains to be investigated. This repair
does not establish complete catalogue rigging, source-motion fidelity, or
browser acceptance.

### Imported model reference validation and next rigging target

The broader source checker reached legitimate imported interior meshes outside
`crystal-voxel-view/models`. Its literal-reference pass now recognizes exactly
five existing assets in `assets/open-models/interiors`: bed, bookcase, chair,
dining table, and television. Each still receives bounded JSON and geometry
validation: nonempty positions, matching normals, finite values, triangle
indices within range, and valid RGBA colors. These imported references are
not counted as authored Pokémon models, and arbitrary external paths remain
rejected. All five assets passed these checks; all five runtime open-model
regressions passed. The complete authoring check remains running.

A production Furret baseline was inspected against its matching 2D state.
The pair preserved checksum `195517439`, LCD frame 70, virtual time
`2.056688083`. Its current model has protruding eye assemblies, stacked body
and tail bands with visible seams, and a cream body that differs from the
pack's warmer palette. Furret needs a continuous anatomical body with fitted
facial features and integrated banding before standard rigging. This is a
baseline critique, not acceptance of the existing model.


### Furret and Noctowl production milestones

Furret now uses a fourteen-joint continuous banded body and tail with fitted
facial details and two fixed hind paws. Noctowl uses a fourteen-joint crowned
owl skin with fitted eyes, folded wings, a fan tail and sixteen fixed support
parts. Both replace their former static battle JSON and route through the
canonical exporters, species registry and GPU skinning path. Their focused
Blender builders reproduce the canonical GLBs exactly. See
[the Furret evidence](furret-sculpture.md) and
[the Noctowl evidence](noctowl-sculpture.md) for geometry, playback and paired
native normal/shiny receipts.

Furret passed ten combined Python checks, two focused production Rust tests,
the schema check for 68 variants and all 44 GPU checks. Noctowl passed eight
combined Python checks, two focused production Rust tests and the schema check
for 69 variants. The full authoring check after Noctowl passed 59 editable
sources, 609 runtime model files and 731 embedded references. The checker now
resolves explicit Unown form labels against their registered GLBs; the logical
species index still counts the complete family once.

The registry has 69 standard variants covering 44 species; the legacy Pidgeotto
rig is separate. The complete 251-species body catalogue is not full rig
coverage. Ledyba and Ledian have independently checked geometry drafts with
continuous spotted shells, distinct feet and proportions, but remain
unpublished and unrigged. See [their draft evidence](ladybird-sculptures.md).
World motion, browser fidelity and the remaining species still require work.


### Ledyba and Ledian production milestones

Ledyba and Ledian now use distinct six-limb skins with fitted eyes, facial masks,
glove seams and shell spots. Their two wing cases close into a continuous dome
and open outward during playback. Ledyba has fourteen joints and two fixed white
feet; Ledian has sixteen joints, fixed red boots, two flight membranes and a
fitted rear head spot. Both canonical GLBs replace the old static battle JSON.
Their real focused Blender builds reproduce the canonical bytes exactly.

Nine stored-model tests and ten combined canonical/routing Python checks passed,
along with two production Rust playback/GPU tests. The schema passed for all
71 standard variants and all 46 GPU tests passed. Full authoring validation
passed 60 editable sources, 609 runtime model files and 733 embedded references.
Four normal/shiny native 3D/2D pairs were inspected with preserved state. Detailed
counts and receipts are in [the ladybird evidence](ladybird-sculptures.md).

The standard registry covers 46 species through 71 variants. Legacy Pidgeotto
is separate; the 251-species body catalogue still contains many static models.
The full goal remains open, including remaining rigs, world motion and browser
fidelity. The prior ladybird draft status above describes the earlier checkpoint.


### Canonical Pokémon skins in the overworld

The production overworld now uses GPU skinning for resolved exact species with
standard rigs, plus the explicit legacy aliases whose geometry already comes
from those canonical rigs. Generic icon families remain unresolved static art;
the renderer does not infer a species from their silhouette or palette.

`new_bark_actors::WorldPokemonSkin` builds the same joint hierarchy and inverse
bind matrices as the battle path. Meshes, weights and bind assets are shared;
each actor owns separate joint entities, pose storage and idle time. Idle
playback uses the authored standard clip while placement, facing and scale still
come from the published visual frame. Palette/source replacement rebuilds the
owned skin, and map changes, hiding and despawn retire its child joints.
Animated bounds prevent neutral-pose culling of moving parts. No simulation or
input dispatcher was added.

Three focused production-world regressions pass: shared immutable mesh/bind
assets with independent joints and fixed feet; source replacement and retirement
without orphan joints; and exact canonical vertex order for every explicit
rigged prop alias. The existing variable-prop replacement/cache regression also
passes. A fresh native Mr. Fuji's house 3D/2D pair was inspected: checksum
1331218822, core frame 46, LCD animation frame 50, virtual time 1.338462583
seconds, six actors and preserved state. Pidgey and Psyduck render through the
new skin path at their published positions; the currently static Nidorino body
retains its existing presentation. The review images stay ignored under target.

This adds overworld idle articulation to the existing rig coverage. It does not
add walking gaits, source-specific motion for every Pokémon or missing species
rigs. Browser playback is not yet visually accepted. The all-Pokémon goal remains
open.


## Spinarak and Ariados canonical six-limb skins

Spinarak and Ariados now use the original spider recipe with fitted eye layers,
curved mandibles, rounded banded limbs and whole-limb exterior normals. Each
canonical GLB has eighteen joints, including six two-joint limb chains. Walking
tips stay on the fixed floor root; head, horn, mandibles and Ariados's raised
legs articulate through idle, attack and hit clips. Shared material rings use
identical spatial influence fields. The canonical GLBs replace the static JSONs
and use the existing generic battle and overworld skin paths.

Eleven stored anatomy/skin checks, the 251-species canonical route check, two
production Rust tests, the complete 73-variant schema check, and all 47 GPU
skinning tests passed. The focused Blender builder reproduces both GLBs
byte-for-byte. Full art validation passed for 60 editable sources, 609 runtime
models and 737 references. Four native normal/shiny 3D/2D pairs were inspected
at preserved game state. Full evidence, source height/palette anchors and
capture receipts are in [spider-skins.md](spider-skins.md).

Standard registry coverage is now 48 species across 73 variants, including the
26 Unown forms. Remaining species rigs, walking gaits, source-specific motion
and browser acceptance keep the all-Pokémon objective unfinished. Source packs,
audio and unrelated gameplay content were unchanged. Temporary spider helpers,
extracted sprites, draft geometry and focused build products were removed;
verification images/logs remain ignored under target.


## Quagsire canonical padded-foot skin

Quagsire now has a thirteen-joint canonical GLB over its continuous amphibian
recipe. The pads have thicker rounded cages and fitted toe folds; palm folds
follow their actual surfaces. Body blue and dorsal purple match the supplied
pack PNG palette. Pads and toe folds stay on the identity floor root, while
pelvis, torso, head, arms, palms, legs and the three-joint paddle articulate.
The dorsal ridge uses the same spatial field as the body-to-tail transition.
Wooper's canonical GLB remains byte-identical.

Native inspection rejected the first pads' hard silhouette facets. A second
subdivision rounds them, with fewer lower-belly rows and retained head detail
to preserve the existing geometry budgets. The final Quagsire has 22 parts,
4,380 vertices, 6,836 triangles, thirteen joints, 127 accessors and 236,652 bytes.
Its digest is d97c02d58f80145da2df67f1902083c13ccc862253c75e9332e5d8b04e5e36ba.

Twelve stored anatomy/skin/route checks, two Quagsire production tests, all 74
standard variant schema checks, all 48 GPU tests, and the shared bounds
preservation regression passed after the revision. The focused Blender builder
reproduces both Quagsire and unchanged Wooper byte-for-byte. Full art validation
passed for 60 editable sources, 609 runtime models and 747 references. Final
normal and shiny native pairs were inspected against 2D at preserved state.
Full details and receipts are in [quagsire-skin.md](quagsire-skin.md).

Standard coverage is now 49 species across 74 variants. Remaining species,
walking gaits, source-specific motion and browser acceptance still prevent
completion of the all-Pokémon goal. No source packs, audio or gameplay content
were changed. Temporary Quagsire helpers, extracted sprites, draft geometry
and focused outputs were removed; QA images and logs remain ignored under target.


### Resting Butterfree NPC pose and source-sized scale (2026-10-14)

Butterfree NPCs now fold coherent wing panels around the canonical root hinges.
A derived Idle envelope includes those rotations; a separate wing material
uses diffuse transmission while the body remains opaque. The supplied moth
icon's maximum visible frame height is 14/16 of its card. Matching NPCs now use
that stable height without changing authoritative position, facing, dialogue
or follower/battle scale and articulation.

The focused regression passes for panel weights, complete Idle bounds, fixed
feet, source-sized scale, unchanged follower scale, shared joints, separate
materials and recursive retirement. The production build passes. Two fresh
Route34IlexForestGate pairs at (5,4), default and orbit 1, were inspected against
2D at identical game state within each pair. Both receipts preserve state:
2940242446 (LCD 223, 4.403709791s) and 1597392490 (LCD 398, 7.417859708s).
The smaller perch clears the teacher's face and keeps its wing pattern readable
from the broadside view. The edge-on view retains attached, visible wings.
This accepts the NPC perch pose and scale; crowded caption placement, broader
palette/motion fidelity and browser acceptance remain open. See
[wing-insect-skins.md](wing-insect-skins.md) for details.

Standard coverage remains 49 species across 74 variants. The full all-Pokemon
model and rigging goal remains active. Pack/icon extraction helpers and temporary
source pixels were removed; native QA captures and logs remain ignored.


### Hoppip standard wind-plant skin (2026-10-14)

Hoppip replaces its static JSON with a canonical eleven-joint GLB over unchanged
sculpture geometry: 21 parts, 3,923 vertices, 5,740 triangles, 122 accessors and
202,816 bytes. Its neutral digest remains
01a26a1397af9b2999e457e68761ebc3373359dc53a6e9ee5ff6cf0d07f4c5e9.
The feet stay fixed; leaf blades and their veins share coherent root/tip fields.
Each fitted ear inset stays with its shell, while flippers and tail articulate.
Idle, attack and hit use the existing rotation-only source-cue timing boundary.

Four stored-skin checks, six family anatomy checks, both focused Rust checks,
all 75 standard variant schema checks, all 49 GPU checks and full art validation
pass. The focused Blender builder reproduces the canonical asset byte-for-byte.
Full validation covers 60 editable sources, 609 runtime models and 749 references.
Native normal and shiny pairs preserve state and confirm attached skin and
pack-owned 0.4064 m height. Review rejects final visual acceptance for the pale
normal pink and edge-on crown readability; source-palette correction is next.
The shiny flag uses this same geometry, rig and clips. Details and receipts are
in [hoppip-skin.md](hoppip-skin.md).

Standard coverage is now 50 species across 75 variants. Skiploom, Jumpluff and
other remaining skins, walking gaits, source-frame motion and browser review
still prevent full completion. Temporary Hoppip pack readers, comparison scripts
and focused outputs were removed; QA images and logs remain ignored. Source
packs, audio and gameplay content were unchanged.


### Hoppip pack body and leaf palette (2026-10-14)

The verified pack's front/back/normal palettes agree on body RGB555 (21,6,7)
and leaf (11,23,6). The canonical Hoppip materials now use RGB8 (173,49,57) and
(90,189,49). An executed audit proves the complete binary geometry/skin/clip
buffer, nodes, skins, mesh schema and animation data unchanged. Five stored
skin/material checks, six family anatomy checks, both focused Rust checks,
byte-identical Blender regeneration and the production build pass.

Normal and shiny paired native views were personally inspected against 2D at
preserved state; the deep red replaces the rejected pale pink and the shiny
flag still recolors the same model. An additional Pidgeotto fixture also
preserves state. Receipts and remaining leaf-crown defects are documented in
[hoppip-skin.md](hoppip-skin.md). Current asset digest is
008e60652d5067fa358c74a2d3ee74c563c22b5b193c988c1b21fda06b224da8;
size is 202,820 bytes. This accepts color correction, not final model/motion
or browser fidelity. Coverage remains 50 species across 75 variants and the
all-Pokemon goal remains active.


### Hoppip curved blades and fitted midrib correction (2026-10-14)

Two levels of topology-preserving midpoint refinement support closed curved
crown blades while retaining their serrated XY silhouette. Dense veins now
follow each blade's actual cross-section and bow. A new ray-based regression
caught and corrected the old midrib partly leaving the leaf silhouette.
Six stored-skin/surface checks, six family anatomy checks, both fresh Rust
checks, deterministic Blender regeneration and the production build pass.
The new asset has 21 parts, 6,077 vertices, 8,380 triangles, eleven joints,
122 accessors and 286,536 bytes; digest is
10632e7b0c178569f270d0e36b649a9403b9ddab13be50e7f22bca01bcd7ab4a.

A fresh native pair preserves state (2038482523, core frame 0, LCD frame 114,
3.236874334s) and was inspected against 2D. The geometry remains attached, but
its crown still reads narrowly from the oblique battle camera. Final visual
acceptance is rejected; blade splay still needs work. The broad off-grid
animation-envelope regression remains in progress. See
[hoppip-skin.md](hoppip-skin.md). Coverage remains 50 species / 75 variants;
the full objective stays active. Focused Blender outputs were removed, while
QA images/logs remain ignored. No pack, audio or gameplay data was changed.


### Hoppip crown splay and fitted leaf-tip hinges (2026-10-14)

A fixed authored -35-degree crown angle and gentler root-starting bow make
both closed serrated blades readable in the production battle view. The
veins share the same transform and rebuilt surface normals; their tip hinges
now sit on the fitted midrib. The positive-angle prototype was rejected by
native inspection and corrected before acceptance.

The final asset has 21 parts, 5,963 vertices, 8,380 triangles, eleven joints,
123 accessors and 284,556 bytes. Digest is
aaa5bb6c980f1ba8d3327f7f611a059193c29ae7092e35f06acab3b9181507b4.
Seven stored-model checks, six family anatomy checks, both fresh Rust checks,
byte-identical Blender regeneration and the production build pass. The focused
Rust check now samples all Hoppip vertices off-grid through all three clips
against the floor and computed culling envelope. The earlier all-75-variant
envelope run passed for the preceding bowed asset in 511.63s.

Fresh normal/shiny native pairs were inspected against 2D at preserved state:
2686567075 (core frame 0, LCD 56, 1.452777292s) and 3671073830 (core frame 0,
LCD 55, 1.485138209s). Two serrated blades now remain distinct; materials and
source-sized support are preserved, and shininess uses this same model.
This accepts native crown readability, not every motion/angle or browser.
Details are in [hoppip-skin.md](hoppip-skin.md). Coverage remains 50 species
across 75 variants; the full all-Pokemon objective remains active. Temporary
prototype code/models and focused outputs were removed, while QA images and
logs remain ignored. No source pack, audio or gameplay content was changed.


### Skiploom leaf-ear and radial flower skin (2026-10-14)

Skiploom replaces its static JSON with a standard eighteen-joint GLB. Its
original positions, normals and triangle order are preserved; body/leaf-ear
materials and petals now use the verified pack green and gold. Root-owned
feet stay fixed. Ear blades and folds share body/base/tip fields, while the
stalk, flower center and eight radial petals have separate articulated owners.
Idle, attack and hit retain the rotation-only source-cue boundary and neutral
endpoints. Hoppip remains byte-identical and Jumpluff remains unchanged.

The canonical asset has 24 parts, 5,182 vertices, 10,028 triangles, eighteen
joints, 145 accessors and 265,012 bytes; digest is
d29a5c2236032d30afba0c5c73b696928dd169c36eb8225b2cd27c43b468fcb1.
Four stored-skin checks, six family anatomy checks, both focused Rust checks,
all 76 standard variant schemas, byte-identical Blender regeneration and the
production build pass. Full art validation passes for 60 editable sources,
609 runtime models and 751 embedded references. The focused Rust check covers
every vertex off-grid through all clips against floor and culling bounds;
GPU integration retains the verified 0.6096 m pack height.

Normal and shiny native pairs were inspected against 2D at preserved state:
3084625203 (core 0, LCD 62, 1.636727750s) and 2295057978 (core 0, LCD 55,
1.344080292s). They establish attached rig and green/gold versus violet/yellow
appearance on the same model. Final visual acceptance remains open because
leaf-ear visibility and the narrow oblique body silhouette need refinement.
Details are in [skiploom-skin.md](skiploom-skin.md).

Standard coverage is now 51 species across 76 variants. Remaining species,
source-specific motion, walking gaits and browser acceptance still prevent
completion of the full objective. Temporary pack readers and focused outputs
were removed; QA remains ignored. No source pack, audio or gameplay data was
changed.


### Skiploom leaf-ear spread and matching hinges (2026-10-14)

Skiploom's leaf blades and fitted fold lines now share an authored -35-degree
rotation around each ear root. Tip hinges use the same transform, and the skin
field is evaluated in inverse ear coordinates to preserve its body/base/tip
attachment. Body, flower, support geometry, palette and pack height are unchanged.
Hoppip still regenerates byte-for-byte.

Five stored-model checks, six family anatomy checks, both fresh Rust checks,
byte-identical Blender regeneration and the production build pass. The new
check verifies tip hinges inside the closed blades and front/side ear spread
relative to body width. The asset remains 24 parts, 5,182 vertices, 10,028
triangles, eighteen joints, 145 accessors and 265,012 bytes; digest is
9541ae752be3d0669e5836358537f971a6bf1d42ca2d2d72db639008e0be4987.

Fresh normal and shiny native pairs were inspected against 2D at preserved
state: 1122343596 (core frame 0, LCD 63, 1.563403209s) and 3351867875
(core frame 0, LCD 55, 1.337014709s). Both leaf ears now extend beyond the body
with readable folds, resolving the preceding oblique overlap. This accepts
ears in the held native view; broader body silhouette, detailed motion, gaits
and browser acceptance remain open. Details are in
[skiploom-skin.md](skiploom-skin.md).

Coverage remains 51 species across 76 variants; the full all-Pokemon goal
remains active. Focused scratch output was removed, QA stays ignored, and no
pack, audio or gameplay content changed.


### Jumpluff three-head skin with all bristles retained (2026-10-14)

Jumpluff replaces its static JSON with a thirteen-joint standard GLB. All 103
source components preserve positions, normals and triangle order, including
all 84 bristles. Each head's 28 bristles share a rigid render group; global
component ranges let authoring readers expand the original anatomy. Runtime
primitive extras retain the existing strict `part` schema and no loader limit
was increased. The runtime has 22 groups, 5,836 vertices, 10,380 triangles,
thirteen joints and 125 accessors.

Root-owned feet stay fixed. Lower stems and seed heads articulate separately;
the crown has a two-joint stem, seed-head joint and independent bracts. Each
sphere and its bristles share a rigid owner. Idle, attack and hit follow the
existing rotation-only source-cue boundary. The verified pack owns 0.7874 m
height and blue/tan palette colors. Hoppip and Skiploom remain byte-identical.

Four stored-skin checks, six family anatomy checks, both focused Rust checks,
all 77 variant schemas, byte-identical Blender regeneration and the first
production build pass. Full art validation covers 61 editable sources,
609 runtime models and 752 references. Initial normal/shiny comparisons
rejected the all-tan/all-lilac spheres. A white-sphere correction preserves
the entire geometry/skin/clip buffer and structural tables; its four stored
checks, six family checks, both fresh Rust checks, Blender regeneration and
production build also pass.

The current asset is 283,968 bytes; grouped neutral digest is
70c2f3002f9ec982475c30f2e55a58d41bf5979a66fcb91187473cf80e295e24.
Corrected normal and shiny pairs preserve state: 3260994057 (core 0, LCD 61,
1.504502625s) and 1744052987 (core 0, LCD 48, 1.223257084s). They verify skin
integration and white puff surfaces on the same model, but final visual
acceptance remains open for smooth sphere appearance, source underside shading
and far-head visibility. Details are in [jumpluff-skin.md](jumpluff-skin.md).

Coverage is now 52 species across 77 variants. Remaining models, detailed
motion, walking gaits and browser acceptance prevent completion of the full
objective. Temporary readers/comparison scripts and focused outputs were
removed; QA remains ignored. No source pack, audio or gameplay content changed.


### Jumpluff lower seed-head separation (2026-10-14)

The lower stems, spheres and bristles now share a −35° horizontal rest-space
turn about their attachments. Seed-head joint pivots follow the transformed
sphere centers; stem weights use the inverse local transform. Neutral heights,
support feet and crown remain unchanged. Hoppip and Skiploom remain byte-identical.
The canonical GLB is 283,964 bytes with digest
`62a6860a398942b33c5be4dfee51d645e90b2efdc32e9d5c010c9e4b6f0fe00f`.

Five stored checks, six family checks and both fresh focused Rust checks pass;
Blender reproduces the GLB byte-for-byte and the production native build passes.
Personally inspected normal/shiny paired views preserve state (checksums
334151549 / 806345474, core frame 0; LCD 55 / 56; times 1.31781525 /
1.430568916). More of the far lower head is visible beyond the body. This
accepts that visibility improvement, with partial occlusion still present.
Smooth puff surfaces, underside shade colors, proportions, source-frame motion,
gaits and browser review remain open; the all-Pokemon goal remains incomplete.
Focused builder output was removed; only ignored QA captures/logs remain.


### Jumpluff scalloped puff undersides (2026-10-14)

Each seed head now has a white upper region and palette-colored lower region,
meeting at a shared scalloped latitude. Both are closed solids with internal
caps, and both retain the head's rigid joint alongside all 28 bristles. Bristle
roots are slightly wider and tips longer. The stem-contact regression uses
both regions, including the crown underside where its stem enters. No loader
limit or material schema was changed.

The canonical GLB has 106 source components, 25 runtime groups, 6,247 vertices,
10,560 triangles, thirteen joints, 136 accessors and 299,068 bytes; grouped
digest `2de06589332b313b7e2264c47f04d319ddb334f09c1caca6632cf7a69b7659fd`.
Six stored checks, six family checks, both focused Rust checks, all 77 variant
schemas and the production build pass. Blender reproduces the file byte-for-byte;
Hoppip and Skiploom remain byte-identical. Full art validation passes for 61
editable sources, 609 models in 609 files and 753 embedded references.

Personally inspected native normal/shiny pairs preserve state (checksums
3326295098 / 3224073217, core frame 0; LCD 55 / 54; virtual seconds
1.360544501 / 1.363543083). The white tops now visibly retain tan undersides
normally and lilac undersides when shiny. This accepts the material separation;
fine texture remains sparse at battle scale. Body proportions, source-frame
motion, gaits, browser review and the all-species goal remain open. Focused
builder output was removed; only ignored QA captures and logs remain.


### Aipom prehensile tail-hand rig (2026-10-14)

Aipom now uses a standard twenty-joint GLB for torso, neck/head, ears, arms/hands,
a four-stage tail through the palm, three tail fingers and three crown tufts.
Feet and toe folds stay root-owned; fitted face parts share the head owner.
Tail and arm weights follow their authored centerlines. The tail hand responds
after the body brace. Idle, Attack and Hit have neutral endpoints and no root
motion; source-frame motion and walking gaits remain open.

Smoothing connected patches reduces 35,580 vertices to 17,804 while an audit
preserves every triangle position, triangle order and material in all fifty
parts. The subsequent primary palette change deliberately matches the supplied
pack: RGB8 purple (99,33,115), tan (239,189,99), with height 207 / 0.7874 m.
The duplicate static JSON was retired. The GLB has 30,016 triangles, 248
accessors and 760,964 bytes, within unchanged loader limits; neutral digest
`630e5b04d9e707d595af256a2d8f5fec236362a767aa7aae0541b262aacb8c83`.
Exact patch-center zeros are normalized across authoring runtimes.

Five final stored checks, seven anatomy checks, both fresh Rust registry/GPU
checks, all 78 variant schemas and the native production build pass. Blender's
production builder reproduces the GLB byte-for-byte. Full art validation passes
for 61 editable sources, 609 runtime models in 609 files and 754 embedded
references. Temporary pack readers, audit helpers/GLB and focused builder
output were removed; QA captures/logs remain ignored. A minimal shared-worktree
Bevy toolbar constructor fix was needed to finish the native build.

Personally inspected normal/shiny paired native views preserve state:
checksums 3232759518 / 172433161, core frame 0; LCD 51 / 54; times
1.413380542 / 1.211226667. The normal purple/tan and shiny magenta/pink surfaces,
attached tail fingers and grounded supports verify integration and palette
correction. Full visual acceptance remains open: the palm projects narrowly
and the face needs a clearer view. Tail-hand scale/orientation, source motion,
gaits, browser review and the all-Pokemon goal remain incomplete. The standard
registry now contains 78 variants spanning 53 species, plus the separate legacy
Pidgeotto path.


### Aipom tail-palm rest orientation and field review (2026-10-14)

The complete palm and all three fingers turn −35° about the unchanged tail
endpoint. Bind pivots follow the same turn, finger weight fields use its inverse,
and palm/finger animation quaternions preserve local curl axes through
conjugation. All 43 unaffected groups retain exact geometry, normals, indices
and weights. The seven hand groups retain dimensions and triangle order; edge
length error is below 0.000000213 units. Heights, support feet, topology, part
counts and joint counts are unchanged. The current GLB is 761,020 bytes, digest
`5dfead35f664f46a4a4ae8d8b59a8f22345dae52cf785d5d9a200fed882a0140`.

Six stored checks, seven anatomy checks, both fresh focused Rust checks and the
production build pass. Blender reproduces the GLB byte-for-byte. The initial
direct fixture lacked retained map provenance and timed out; the Route 36
scripted encounter reached battle but failed the checked-ground shot test.
Neither was treated as visual acceptance. A disposable party species/shiny
option now works with the existing walking and Route 36 field fixtures. Real
controller input, scripts, encounter rolls and retained-map guards remain
authoritative; no camera guard was relaxed.

Personally inspected normal/shiny Route 29 pairs succeed after ordinary grass
encounters. Both preserve state: normal versus Pidgey, core frame 3059, checksum
1597936004, LCD 3765, time 76.585598167; shiny versus Sentret, core frame 2599,
checksum 630686738, LCD 3348, time 71.004649628. The palm and all three fingers
read clearly and remain attached. This accepts the rest pose and rig alignment
under the retained-map camera; earlier direct-arena views use a different shot.
Detailed face fidelity, source-frame motion, gaits, browser review and the
all-species objective remain open. Owned draft GLB, profiler output and focused
builder output were removed; QA captures/receipts/logs remain ignored.


### Sunkern and Sunflora standard skins (2026-10-14)

Canonical weighted GLBs replace the two static JSON sources. A pre-migration
recipe audit matched both originals; the migration retains every triangle
position, normal and index with intentional material corrections from the
supplied pack. Sunkern has seven joints, 18 parts, 11,694 vertices, 16,524
triangles and 96 accessors in 475,748 bytes. Sunflora has 25 joints, 30 parts,
9,183 vertices, 13,696 triangles and 195 accessors in 478,692 bytes. Existing
loader limits and shared GPU mesh/bind ownership remain unchanged.

Sunkern's body and coat share weights, and its two-stage leaves share bend
fields with their fitted midribs. Sunflora's feet remain fixed, its face stays
with the flower disk, its leaf details deform together, and all sixteen rays
have disk-rooted rigid owners. Rotation-only idle/attack/hit clips have neutral
endpoints, stationary roots and delayed botanical responses. Detailed source
motion and gaits remain unverified. Pack heights are 0.3048 m and 0.7874 m.

Pack reference inspection corrected Sunkern's coat panels to green and
Sunflora's disk/smile to white. The initial black-coat Sunkern capture was
rejected. Current neutral digests are
`2c11fc4e36f23be374beb989f959372f218b2cb292dcc91286cb8804df72ea8b`
and `59a70f169f6dc674d2348163e6f97ba96ebbddb58d225f932dc95e5e1c4da797`.
After the green correction, five stored checks, ten anatomy checks, both fresh
Rust family deformation/GPU checks and the native build pass. Blender reproduces
both assets byte-for-byte. Before that material-only correction, all 80 standard
variant schemas and the full art validator passed: 61 editable sources, 609
runtime models in 609 files and 757 embedded references.

Personally inspected normal/shiny Sunkern retained-map pairs preserve state:
normal core 1726, checksum 2531288909, LCD 2327, time 40.765335458; shiny core
2132, checksum 2538937951, LCD 2844, time 51.328301835. Both leaves remain
visible, panels match the source green and the shared appearance path applies
the source's brighter shiny palette without another model or rig. Source-height
scale stays small; leaf silhouette refinement remains open. The normal Sunflora pair also preserves state: core 1759, checksum 566196954,
LCD 2374, time 41.498861499. Its white face, golden rays, leaf arms and feet stay
attached and visible. Petal silhouettes remain sharper than the source, and a
distant gray terrain slab remains visible in this retained-map scene. The shiny
Sunflora pair preserves core 1302, checksum 1498643714, LCD 2008 and time
36.292988541. Both images were inspected: brighter yellow rays and olive
body/leaf surfaces agree with the source palette change. This accepts native
rig and palette integration; shape, detailed motion and browser fidelity remain
open. The queued shiny capture was deferred once for another GPU measurement,
then completed successfully without interrupting an active capture. Registry coverage is 80 variants spanning 55 species,
plus the separate legacy Pidgeotto path. Browser acceptance and the all-species
objective remain incomplete. Temporary pack readers, reference PNGs and focused
builder output were removed; QA captures, receipts and logs remain ignored.


### Sunflora rounded petal caps (2026-10-14)

The retained-map review identified excessively pointed front petals. Their
outlines now end in broad rounded caps while preserving the maximum reach and
sample count. An executed decoded-binary audit proves all 22 non-petal parts
retain exact positions, normals, indices, joints and weights; node and animation
schema also remains exact. Sunkern is unchanged. Ray ownership and palette
routing remain unchanged. The eight caps now retain over 70% of full petal width
in the outer fifth, versus 16.7% on the old first ray; a stored regression checks
each cap in its radial coordinates. Smoother normal grouping removes 48 vertices.
Sunflora now has 9,135 vertices, 13,696 triangles, 30 parts, 25 joints and 195
accessors in 477,324 bytes. Current digest:
`3342555c766e836e7ea17ed6fc8a931fb727e29fbb06f97eee739104ea670f70`.

Six stored geometry/skin checks, ten anatomy checks, both fresh Rust family
checks and the native build pass. Blender reproduces both canonical family GLBs
byte-for-byte. Personally inspected normal/shiny paired reviews preserve state: normal core
1369, checksum 2891771878, LCD 1958, time 34.028171958; shiny core 1681, checksum
3389832729, LCD 2361, time 41.560019001. Both show broad rounded rays with
attached face and leaf arms, using the unchanged palette path. Grass partly
hides feet in the normal view; the shiny view exposes both supporting pads.
These shots use different encounter/camera compositions. They accept the new
rest silhouette and retained rig integration, not source-frame motion. Earlier
receipts describe the pointed-petal revision. Temporary baseline GLB and focused
Blender output were removed; QA captures and logs remain ignored. The full
all-species objective, source-frame motion, gaits and browser fidelity remain
incomplete.


### Foot-contact shading in retained grass (2026-10-14)

User review identified a floating appearance over grass. Battle layout already
places each neutral support minimum on its checked ground anchor, while its
flat contact-shadow mesh lies below the blades. Retained grass now receives a
bounded contact-darkening field centered on the actual neutral body footprint
and ground elevation. The field ignores animation-envelope expansion, follows
the presented body pose, clears for absent bodies, and shades only native tagged
grass near the supporting floor. It does not change model positions, physical
scale, blade geometry or grass height. The user assigned taller grass to another
agent; that work is separate.

All 21 focused grass regressions and the native build pass. The new regression
checks ground elevations and verifies that a large animation envelope leaves
contact position/radius unchanged. A production paired capture preserves core
2737, checksum 3121419762, LCD 3442 and time 64.397087581. Both images were
inspected and no shader errors were reported. The current taller-grass geometry
fills most of the low camera with near blades; only body apertures expose the
feet. Contact patches are visible there, but this is not visual acceptance of
the floating fix or the full scene. Taller-grass/camera integration and a clear
final ground-contact review remain open. QA images/receipts/logs stay ignored.


### Battle lens clearance from taller grass (2026-10-14)

The previous rejected review exposed a separate problem: static sight coverage
correctly excludes grass opened by the body aperture, but that also allowed a
camera inside tall blades. Original tagged grass triangles now publish their
own bounded coverage on the terrain worker, before render-only CPU mesh
retirement. Frozen encounters transform and index that evidence alongside their
existing immutable scenery metadata. Camera admission rejects grass intersecting
the lens and the first 0.30 world units in front, with 0.15-unit bounds padding.
It retains all existing source-ground, physical size, full-body, authentic-map
and omitted-shadow checks. No model, grass height, blade geometry or source
encounter state changes. The existing shot search selects a proven angle.

The focused real-rig camera regression passes, including rejection inside grass
and acceptance above its tops with unchanged body poses. Both coverage tests and
the GPU CPU-retirement regression pass; the native example rebuild succeeds.
The lens-only Sunflora pair preserves core 2964, checksum 2727119297, LCD
3599 and time 65.636199623, but its personally inspected 3D view still has a wall
of foreground grass. It is rejected as visual acceptance. The obsolete queued
Sunkern capture was cancelled before starting. Admission now also checks the
first 60% of the ray toward the composition center, and the established shot
search adds 40° and 48° candidates. The expanded regression passes for grass at
the lens, a curtain farther ahead, and low grass that must remain allowed.
A fresh native rebuild and paired review of this broader check are pending.
The ground-contact review and all-species objective remain open.
