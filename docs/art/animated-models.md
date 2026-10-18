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

`models/battle_species/pidgeotto.glb` is the canonical standard GPU skin: 34
named anatomical parts, 14 joints and source-relative idle, attack and hit clips.
It retains every original geometry bit, neutral bound, physical size and exact
0.8-second wing loop. Closed flexible shoulder covers keep the original wings
attached through their full motion; head, wrists, shins, tail layers and crest
have independent controls. Battle claws stay planted; world motion retains Fly
clearance. The shared renderer owns immutable geometry, per-actor clocks,
source cues, palette flashes, visibility and full animated camera bounds.
The former dedicated wing-node parser and three-group renderer are removed.

The authoring recipe `tools/build-battle-species.py` writes through the public
`tools/pidgeotto_glb.py` wrapper. The strict reader, STL exporter and legacy demo
consume canonical geometry without a JSON sidecar. Background Blender and both
CLI paths reproduce the same production bytes. A clean game build needs no
Blender installation.

Twenty-one Python and 32 distinct scoped Rust checks pass. Three production
2D/3D pairs and a moving world frame were inspected inline, entirely headless,
offscreen and muted. Battle labels identify PIDGEOTTO and SENTRET. Original
1.0922-metre size and full-body admission are preserved. Wide side views overlap
feathers and reduce facial readability. No dedicated shiny validation was run.
See the [production skin guide](pidgeotto-skin.md) for geometry pins, controls,
rebuild commands and the limits of this review.

## Cyndaquil and Totodile GPU skins

`models/actor_props/battle_cyndaquil.glb` and `battle_totodile.glb` replace their
former JSON models. Totodile uses a refined crocodile silhouette with cheek-wrapping red eyes, a smaller
open grin, pink tongue, four visible fangs, a broad fitted yellow chest chevron,
a fuller pear-shaped belly, tapered tail and stronger red dorsal plates. Its authored blue palette is used by default in location previews;
real saves still derive shiny appearance from their actual Pokémon DVs. Cyndaquil
now uses the rounded four-legged sculpture described below. Both have standard
glTF skins and named idle, attack and hit clips.
Cyndaquil has twelve joints, including five flame anchors and four separate
legs. Totodile has eight joints, including a hinged jaw, separate forepaws
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


## Cyndaquil quadruped and Bevy body fire

The canonical Cyndaquil is authored in `tools/cyndaquil_sculpt.py`: a rounded
forehead, downward-curving muzzle, fitted dark teal hood, closed cheek eyes,
low body and four separate curved legs with rounded, planted soles. Its 34
anatomy parts retain the original face and quill identities and append four
limbs. The twelve-joint rig articulates forelegs and hindlegs independently.
The canonical GLB has 16,314 vertices and 8,138 triangles. No duplicate mesh
catalog, Blender source or game content pack is needed for this change.

```sh
python3 tools/cyndaquil_glb.py --output crates/crystal-voxel-view/models/actor_props/battle_cyndaquil.glb
python3 tools/test_cyndaquil_glb.py
```

The body reference stays at `0.716729`; its pack-derived physical body height
remains 0.508 m. The full authored fire silhouette grows to 1.25, so the body
reference ratio is now `0.716729 / 1.25`. Battle footing and hit anchors still
measure the body independently of its flames. Idle is 3.2 seconds; attack and
hit remain one-second normalized clips driven by existing battle cues. All
four sole contacts remain on the identity root. The fourteen Python checks
cover reproducibility, geometry, winding, skin weights, face attachment,
planted soles, loop closure, attack response and bounded pose quantization.

`battle_aura.rs` replaces the instance's solid flame indices with attached
Bevy effects. The canonical mesh and animation remain available to other
renderers. An animated density shader integrates curling, noisy orange flame
tongues with a bright core and red edges; retained particles add rising wisps
and embers. Cyndaquil's quill bounds form one broad fire bank. Ponyta and
Rapidash retain separate mane, tail and ankle emitters. The Charmander line,
Quilava, Typhlosion, Magmar and Moltres use their authored flame anatomy;
Gastly replaces its outer vapor lobes with drifting translucent purple wisps.
Unrelated orange body surfaces do not become emitters.

Effects follow the current skin matrices, honor appearance palettes and
source flashes, pause when hidden, and retire with the actor. Shared meshes
and materials and a fixed particle pool bound allocation. They do not emit
move commands, change source cue clocks or mutate gameplay. The renderer
regressions cover anatomy selection, anchor skinning, stable allocation,
visibility, pause and party-switch retirement.

Native review uses `immersive_battle_3d --walking-encounter` with the local
`content-packs/core-modular.browser.crystalpack`. Set
`CRYSTAL_RENDER_TEST_PARTY_SPECIES=CYNDAQUIL` (or another affected species),
walk into grass with `--walk`, and capture `--screenshot` plus `--second` for
a matched production 3D/2D pair. Set `CRYSTAL_RENDER_TEST_SECOND_VIEW=same` for
two advancing 3D samples when inspecting the fire animation. Temporary capture
output belongs outside tracked source.

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
The fresh native rebuild passes, as do all 16 terrain-batch regressions. The
new production pair preserves core 5741, checksum 273226458, LCD 6497 and time
119.858704791. Both images were personally inspected. Ground now reads around
both Sunflora and Pidgey, with Sunflora's supporting feet visible rather than
isolated silhouette cutouts in a solid foreground grass wall. The diagnostic
confirms 15,162 actual grass bounds reaching world height 7.049357. The dense
background grass and distant gray slab still limit full-scene acceptance; no
grass height or source terrain was changed in this work. The final shot uses
an open foreground beside the grass, not a claim that every new steeper
candidate was exercised visually. Small Sunkern ground-contact review remains
open, along with broad all-species, motion and browser verification. Owned
obsolete capture jobs are terminal; QA captures, receipts and logs remain
ignored.


### Small Sunkern field contact review (2026-10-14)

The updated foreground-grass camera admission also succeeds for Sunkern versus
Sentret. Both images were personally inspected. The pair preserves core 3760,
checksum 2641092532, LCD 4516 and time 81.258626295. The tiny seed base is exposed
against the supporting ground, both leaves remain readable, and its source
height stays unchanged. Contact shading remains subtle; this verifies visibility
and placement in this field shot, not every species or full-scene acceptance.
The tall grass backdrop and distant gray slab remain visible limitations.


### Bellossom standard dancer skin and source palette (2026-10-14)

Bellossom now uses a canonical standard GLB instead of duplicate static JSON.
The original 45-part source matched its original recipe before edits. Petal
normals now share connected patches; the fifteen curved skirt leaves retain
analytic controls with 16 transverse samples/21 rows instead of 20/25. All 30
non-skirt parts preserve indexed f32 triangle positions. Intentional palette
corrections follow the supplied front/back/normal palettes and inspected front
sprite: body/crown centers RGB8 (49,140,66), red petals/outer skirt (222,66,49),
white inner skirt and black/white face details. This corrects the old green/gold
skirt. Dark rear leaves remain an authored red shade. Pack height digits 104
establish 0.4064 m; authored span stays 0.95.

The asset has 18,524 vertices, 26,980 triangles, 45 parts, 25 joints and 227
accessors in 739,892 bytes, within unchanged loader limits. Neutral digest:
`21bc079e44b6abd8d1012172d7fb1c909e679d18d143ceafc613241861785f57`.
Torso/neck/head, two-stage arms, fifteen leaves and two complete crowns respond
through rotation-only idle/attack/hit clips. Crown/arm tips lag the body brace.
Fitted face parts follow the head; each crown's cup, rays and well share an
owner. Lowest skirt tips stay root-owned. Source-frame motion and gaits remain
unverified, and shinies retain the same model and existing palette route.

Six stored-skin checks, eight anatomy checks, both fresh Rust floor/envelope and
GPU integration checks, all 81 variant schemas and byte-identical Blender
regeneration pass. Full art validation passes: 61 editable sources, 609 runtime
models in 609 files, 758 embedded references. Native rebuild and paired normal/
shiny visual reviews are pending; no visual acceptance is claimed. The standard
registry now spans 56 species/81 variants, plus separate legacy Pidgeotto.
Temporary pack reader, extracted references, old static audit copy and focused
Blender output were removed; ignored QA logs remain. The all-species objective
and broad source-motion/browser fidelity remain incomplete.


Bellossom neutral review subsequently rejected black crown wells that read like
additional eyes. Both now use the pack green. An executed audit of the prior
asset in the owned build cache proves the entire binary buffer, nodes, skins,
clips and mesh schema unchanged; only two material assignments/digest differ.
The corrected front software view was inspected. Six stored checks pass again,
and Blender reproduces the corrected canonical file exactly. The eight anatomy,
two Rust, 81-schema and full-art results above predate this material-only fix.
Both final-revision Rust registry/GPU checks now pass. The final native rebuild
passes; the paired native review is recorded below. Temporary software views and focused builder output were removed.


### Bellossom final normal/shiny native integration review (2026-10-14)

The final-revision native rebuild succeeds. Personally inspected retained-map
3D/2D pairs preserve state: normal versus Pidgey, core 3073, checksum 430773371,
LCD 3872, time 76.65057125; shiny versus Sentret, core 3428, checksum 471454070,
LCD 4273, time 86.457739664. Normal green body/centers and red/white skirt, and
shiny blue body/centers and orange-red flowers agree with the source palette
change. Fitted face, arms, skirt layers and crowns remain attached; height and
the shared model/rig are unchanged. Native rest-pose/rig/palette integration is
accepted. Contact shadows remain weak, so full grounding polish is open, alongside
source-frame motion, gaits, browser review and the all-species objective. The two
encounters have different camera compositions; comparisons are only between the
matched views within each state receipt. QA ran without a shared graphics lease
under concurrent load, so durations are not performance evidence. QA captures,
receipts and logs remain ignored; both capture processes are terminal.

### Retained-floor contact shading (2026-10-14)

Contact shading now reaches upward-facing retained floor surfaces as well as
grass roots. Its ellipse follows the neutral model footprint and facing, while
its height stays on the encounter's checked ground during animation lifts.
Current posed bounds still control grass visibility independently. Hidden actors
clear both contact uniforms. Grass height is unchanged by this work.

The two focused Rust regressions pass: long, thin footprints retain their axes
through yaw, and lifted/hidden actors retain or clear their checked-floor contact.
The earlier circular shader prototype was inspected in a matched normal
Bellossom/Pidgey 3D/2D pair (core 5827, checksum 702890493, LCD 6670,
time 135.588032707, state preserved) and made contact beneath the skirt and feet
clearer. The final oriented-ellipse native build and matched normal Bellossom/Sentret
review pass: core 2048, checksum 2194992681, LCD 2662, time 48.008425291,
state preserved. The skirt and tail support have visible contact shading on
the retained floor; grass remains behind the readable bodies. This accepts the
contact-material integration, while broader scenery and exact motion remain open.
Shiny-specific verification is skipped at the user's request; this change does not alter the
shared palette path.

### Qwilfish standard skin and local source colors (2026-10-14)

Qwilfish now has a canonical seven-joint standard skin in place of duplicate
static JSON. The migration audit proved all 36 parts retain exact stored f32
positions, normals and triangle order: 9,014 vertices and 13,644 triangles.
Its pectoral fins and fitted folds blend from the body; three tail stages drive
the peduncle and coherent spotted paddle. Face and all rooted spines share the
body owner. Yaw-only idle/attack/hit clips preserve every vertex's floor height,
have neutral endpoints and retain authoritative source-cue timing.

The actual local pack front sprite was inspected, and front/back palettes agree:
navy RGB555 (4,6,16), yellow (27,29,11), white and black. This corrects the former
teal upper body and pink lips; lips now use source yellow. The first exporter
preview was too dark because colors were converted twice. The stored-material
regression caught it; the corrected recipe leaves conversion to the geometry
helpers. Final front and rear software previews were inspected. The pack's
height 108 means 1′8″ (0.508 m), with the existing 0.91 normalized reference span.

Focused Blender species-builder output is byte-identical and emits no duplicate
static JSON. All 14 stored-GLB/anatomy checks pass. The final asset is 378,948 bytes, uses
142 accessors and has rest digest
`84580db6a0528140ae8fb939c5808b542d23f52df996fb2eb072b8352d1dd6d1`.
Both focused production Rust checks pass: GPU skinning at the pack height and
off-grid clip sampling for coherent attachments, floor height and culling bounds.
The initial native build and matched normal Qwilfish/Sentret pair pass their
integration checks (core 2251, checksum 1983176184, LCD 2959,
time 54.653736872, state preserved). The actual views exposed a yellow paddle
center where the sprite reads white. Both fitted paddle centers now use source
white. A material-only audit proves the complete binary buffer, nodes, skin,
clips and schema remain identical, with exactly two material assignments and
the rest digest changed. Final canonical/palette checks pass; the fresh
white-center native rebuild passes. The final matched normal pair was inspected
(core 1456, checksum 3488417035, LCD 2095, time 36.573381584, state
preserved). Body/fin/spine attachments, scale and contact shading remain intact;
the visible paddle portion is white. The paddle is partly behind the body in
this composition, so this accepts native integration rather than a complete
source-frame or all-angle fidelity claim. The earlier 14 Python,
two Rust and focused Blender results predate this material-only correction.
Exact source-frame
motion, field gaits and browser review remain open. Shiny-specific checks and
captures are skipped at the user's request. The standard registry now contains
82 variants / 57 species; the broader all-species goal remains active.

### Wobbuffet standard elastic skin and local source colors (2026-10-15)

Wobbuffet now uses a canonical 11-joint GLB in place of duplicate static JSON.
Its four basal lobes remain fixed to the identity root. The continuous lower
body blends into a torso owner shared by the fitted face. Both curved arms
have independent shoulder/elbow/hand chains, and the tail has three stages
with its paddle, eyes and pupils sharing one rigid owner. Rotation-only
idle/attack/hit clips have neutral endpoints and delayed hand/tail motion;
authoritative source battle cues still own playback timing.

The migration audit proves exact stored f32 positions, normals and triangle
order for all 19 parts: 11,791 vertices and 19,260 triangles. Actual local
front/back sprites and normal palettes were inspected: RGB555 body (12,21,31),
mouth shade (7,14,16), black and white. Body color is now sRGB8 (99,173,255),
the open mouth and tail are black, and the lower mouth uses (57,115,132),
replacing the former burgundy/pink expression. The pack's height 403 is 4′3″
(1.2954 m), with the existing 1.14 normalized reference span unchanged.

Six stored-skin and nine migrated anatomy checks pass. The focused Blender
species builder is byte-identical and emits no duplicate static JSON. The
neutral software preview was inspected. The asset is 503,824 bytes, uses 111
accessors and has rest digest
`4afb5da2de8fcfaaa070c632289d645dc8b67e7790a2f57b19dc3dc3e5b20a09`.
Both focused Rust checks pass: GPU skinning at the pack height, and off-grid
clip sampling for planted support, coherent owners and culling bounds. The
native build passes. The matched normal Wobbuffet/Pidgey pair was inspected:
core 1957, checksum 1205809066, LCD 2544, time 45.984559665, state
preserved. The complete raised-arm silhouette, both tail eyes, fitted face and
planted basal lobes are readable, with contact shading on retained terrain.
Native rest-pose/rig/color integration is accepted. The existing
salute pose is preserved; exact sprite-frame face/arm motion, field gaits and
browser review remain open. Shiny-specific checks and captures remain skipped
at the user's request. The standard registry now contains 83 variants / 58
species; the all-species goal remains active.

### Dunsparce grounded wing-and-drill skin (2026-10-15)

Dunsparce now uses a canonical seven-joint standard skin in place of duplicate
static JSON. The migration audit proves exact stored f32 positions, normals
and triangle order for all 19 parts: 11,833 vertices and 12,248 triangles.
Yaw-only body motion preserves belly height and keeps fitted eyes, jaw and back
ribbons coherent. Each small feather and its fitted fold share one rigid wing
owner. The first blended feather chain inverted a thin edge during attack;
rigid feather ownership fixes that deformation without removing wing motion.
Three tail stages articulate the stem and drill, with the entire spiral rigid
to the cone. The drill turns about its authored tilted base-to-tip axis.
Rotation-only idle/attack/hit clips have neutral endpoints and an identity root;
authoritative source battle cues own timing.

Actual local front/back sprites and normal palettes were inspected: RGB555
gold (31,24,6), blue (8,15,23), white and black. The body now uses sRGB8
(255,198,49), blue regions use (66,123,189), eyelids/feathers use white, and
feather folds/drill grooves use source blue. The pack's size 411 is 4′11″
(1.4986 m); the existing shared layout continues to measure Dunsparce along
nose-to-tail Z, rather than treating that length as vertical height. The 0.85
normalized vertical span is unchanged.

All seven stored-skin and eight anatomy checks pass, and the focused Blender
builder emits byte-identical GLB with no duplicate JSON. Neutral front and side
software views were inspected. The final asset is 460,848 bytes, with 99
accessors and rest digest
`97c7da81fb99f3363b601244efa047a94585dd0fa8c4b6faad88f5537b4fde4f`.
Both focused production Rust checks pass: GPU skinning at the pack length and
off-grid clip sampling for grounded support, fitted owners and culling bounds.
The native build and matched normal Dunsparce/Pidgey review pass: core 2245,
checksum 2951808166, LCD 2916, time 52.583664208, state preserved. The full
longitudinal silhouette, grounded belly, fitted back bands, both feathers and
tilted spiral drill remain readable without clipping. Native rest-pose/rig/color
integration is accepted; exact source-frame motion remains open. Exact source-frame
eye/body motion, field gaits and browser review remain open. Shiny-specific
checks and captures remain skipped at the user's request. The standard registry
now has 84 variants / 59 species; the all-species goal remains active.

### Shuckle fixed carapace and soft-body skin (2026-10-15)

Shuckle now uses a canonical eight-joint GLB instead of duplicate static JSON.
Its shell, six annular collars, spots, recessed core and four grounded feet
remain rigid to the identity root. Each limb's middle blends to one joint while
its ends fade back to fixed weights; yaw-only limb motion preserves height.
A base/bend/head chain articulates the neck with the complete face rigid to its
head owner. Rotation-only idle/attack/hit clips have neutral endpoints and
delayed upper-neck/head response. Authoritative battle cues own playback timing.

The migration audit proves exact stored f32 positions, normals and triangle
order for all 27 parts: 14,875 vertices and 19,836 triangles. Actual local
front/back sprites and normal palettes were inspected. Source RGB555 shell
(21,4,3) becomes sRGB8 (173,33,24), soft-body (31,22,5) becomes (255,181,41),
and collars/spots are white with black eyes, mouth and recessed interior.
The pack's height 200 means 2′0″ (0.6096 m); the normalized 0.91 span is unchanged.

All six stored-skin and eight anatomy checks pass. Fixed shell/collar geometry
retains the six independently tested apertures throughout every clip. The
focused Blender builder emits byte-identical GLB and no duplicate JSON. Neutral
front and rear software previews were inspected. The final asset is 594,848
bytes with 132 accessors and rest digest
`2a7a21948c474962c403086b8e508fc560f2c001c1448311316a0dbb46bc2206`.
Both focused production Rust checks pass: GPU skinning at the pack height,
and off-grid clip sampling for fixed hard geometry, coherent face and culling
bounds. The native build and matched normal Shuckle/Pidgey review pass: core 1959,
checksum 3950609093, LCD 2608, time 46.914735668, state preserved. The fixed
shell, seated neck/collars, complete head and visible grounded feet remain
readable, with floor contact shading. Native rest-pose/rig/color integration is
accepted. Exact source-frame
neck retraction, field gaits and browser review remain open. Shiny-specific
checks and captures remain skipped at the user's request. The standard registry
now contains 85 variants / 60 species; the all-species goal remains active.

### Gligar coupled sail/pincer and venom-tail skin (2026-10-15)

Gligar now uses a canonical 15-joint standard skin in place of duplicate static
JSON. Feet and talons remain fixed; the torso and legs blend above that support.
The fitted face follows the head, ears are articulated, and tongue/crease share
one owner. Arm, sail and spar weights agree by position, with complete crescent
pincers retaining their rigid open centers/jaws. A staged curled tail reaches
the venom bulb and stinger. Rotation-only idle/attack/hit clips have neutral
endpoints and delayed pincer/tail response; source battle cues own timing.

The initial sail caps inverted thin triangles during motion. Interior support
and constrained Delaunay flips preserve every original sail boundary position,
while removing the long cap diagonals responsible for folding. A wider pincer
blend limits strain. Neutral inspection also prompted a narrower, flatter tongue
with preserved head/mouth overlap. The other 28 parts retain exact stored f32
positions, normals and triangle order. The final model has 32 closed parts,
17,422 vertices and 26,768 triangles, within existing runtime limits.

Actual local front/back sprites and normal palettes were inspected: RGB555
pink (25,6,18), violet (8,7,20), white and black. Source sRGB8 colors are
(206,49,148) and (66,57,165); sails and tongue use violet. The pack's height 307
is 3′7″ (1.0922 m), with the original normalized 1.06 span unchanged.

All eight stored-skin and ten anatomy checks pass. The final focused Blender
builder is byte-identical and emits no duplicate JSON. Final front and earlier
refined rear software previews were inspected. The GLB is 756,232 bytes with
179 accessors and rest digest
`9d3742959ab097a791488894bd78c071e6ace9c2dcb6c41b7dbd7e81ced1c10d`.
Both final production Rust checks pass: GPU skinning at the pack height and
off-grid sampling for planted support, coherent owners and culling bounds. The
final native build and matched normal Gligar/Pidgey review pass: core 2025,
checksum 1364756158, LCD 2637, time 46.529815542, state preserved. Open pincers,
visible sails, fitted face, smaller violet tongue and grounded pads/talons remain
readable with floor contact shading. The curled tail is partly hidden in this
composition; its full rest shape was inspected in the rear software preview.
Native rest-pose/rig/color integration is accepted. Exact source-frame
face/wing motion, field gaits and browser review remain open. Shiny-specific
checks and captures remain skipped at the user's request. The standard registry
now contains 86 variants / 61 species; the all-species goal remains active.

### Yanma compact four-wing and segmented-abdomen skin (2026-10-15)

Yanma now uses a canonical 16-joint skin instead of duplicate static JSON. Four
rigid wing owners each carry their complete sail, both fitted bands and three
veins. The fitted head/face is coherent; six leg middles articulate between the
thorax and fixed forked toes. Three abdomen stages carry the fitted collars and
coherent terminal cap, fins and prongs. Rotation-only idle/attack/hit clips have
neutral endpoints, an identity root and delayed rear-wing/abdomen response;
authoritative source battle cues own timing.

The original 20,268 vertices exceeded the skin loader's 20,000 limit. Smaller
thorax radial sampling and vein cross-sections yield 19,976 vertices and 26,452
triangles. The 41 non-wing, non-thorax parts, including every toe, retain exact stored f32
positions, normals and triangle order. Each wing's three veins share a runtime
primitive: 66 named source parts become 58 runtime primitives, with lossless
source ranges in top-level GLB metadata. Initial per-primitive range annotations
were rejected by the strict loader schema; moving them to top-level metadata
preserves that validation. Identical normal/skin/animation accessor reuse keeps
243 accessors below the existing 256 limit, without relaxing any budget.

Actual local front/back sprites and normal palettes were inspected. Source
RGB555 green (9,26,10) and red (20,7,7) become sRGB8 (74,214,82) and
(165,57,57). Wing bands are red, limbs green, wings/glints white, and lenses,
veins, mouth and abdomen collars black. The pack's size 311 is 3′11″ (1.1938 m).
The shared long-body layout now measures Yanma along nose-to-tail Z, preserving
uniform scale rather than applying the listed body dimension to its raised-tail
vertical span. The normalized 0.95 vertical span and floor origin are unchanged.

All seven stored-skin and nine anatomy checks pass. Neutral front and rear
software views were inspected. The final GLB is 755,228 bytes with rest digest
`53cceeb9aa5759c6c8bace4f6e9dcb73b4f0d839dd6657e14b814d767c2c9711`.
The separately reviewed packed-runtime digest is
`85a1c4865714a1dd738973cce3244a9b4708e155890fe20061208182eed8114b`.
The loader hashes the 58 packed primitives; source reconstruction retains the
66-part source digest. Both forms remain independently pinned and checked.
Final focused Blender output is byte-identical and emits no duplicate JSON.
Both final production Rust checks pass: GPU skinning at the pack length and
off-grid sampling for coated wings, planted toes, segmented tail and culling
bounds. The native build passes. Matched normal 3D/2D review against Pidgey
shows the upward wing pairs and red panels clearly; the green head/legs, red
segmented tail and fixed toe contacts remain coherent. Near-side wings still
foreshorten at this angle. The paired receipt preserves core frame 2398, hash
469961496, LCD frame 3092 and virtual time 59.517074416
(`state_preserved=true`). Captures are ignored local QA artifacts.
Exact sprite-frame wingbeats, flight/field gaits and browser review remain open.
Shiny-specific checks and captures remain skipped at the user's request. The
standard registry now contains 87 variants / 62 species; the all-species goal
remains active.

The two wider model regressions also pass on the corrected packed asset:
`every_exact_species_is_grounded_and_volumetric` and
`body_pose_uses_elapsed_idle_time_and_exact_current_attack_progress`. These
cover neutral geometry and elapsed pose sampling; they do not establish visual
acceptance for the whole catalog.

The front wings rest at 12 degrees and rear wings at 18 degrees of upward
dihedral. Every sail, both fitted panels and all three veins rotate together;
surface-fitting checks project along each actual wing normal. This exposes the
panels at low production camera elevations while retaining the floor origin.

### Murkrow planted witch-crow and articulated broom skin (2026-10-15)

Murkrow now uses a canonical 11-joint GLB and the shared production skinning
path. Its full 62-part sculpt is preserved: 13,359 vertices, 16,616 triangles,
exact original f32 positions/normals and triangle order. The local pack's normal
palette supplies violet (57,57,140), gold (214,214,24), white and black. Height
108 gives the production 0.508 m scale; the normalized 0.78 span is unchanged.

Fixed toes and talons support a yawing torso. The neck and throat blend into
the coherent fitted head; the lower bill articulates separately. Each folded
wing keeps its three layered vanes together. Three hat feathers and the bound
broom tail settle with delayed motion. A lengthwise tail weight transition
removes attack pinching. The clips retain an identity root, neutral endpoints
and authoritative battle-cue timing.

The canonical asset is 533,448 bytes with 239 accessors and digest
`1eb3ca435d7fe293a9371188874d4c5e7007e7f2ad87cddd52b1e3202fbbf132`.
All 15 Python checks pass. Neutral front/rear software views were inspected.
Both focused production Rust checks pass, and final focused Blender output is
byte-identical with no static JSON. The native build passes. Matched normal 3D/2D review against Pidgey shows
planted gold toes, a readable hat and bound broom tail against taller grass.
Core frame 2544, hash 1815780038, LCD frame 3204 and time 57.765340872
are preserved (`state_preserved=true`). Ignored local captures are QA artifacts. Shiny checks/captures remain skipped. Exact motion, field gaits
and browser review remain open. The standard registry now has 88 variants /
63 species; the all-species goal remains active.

The wider `every_exact_species_is_grounded_and_volumetric` and
`body_pose_uses_elapsed_idle_time_and_exact_current_attack_progress` regressions
also pass on this asset. They cover catalog geometry and elapsed pose sampling,
without establishing visual acceptance for unrelated models.

### Misdreavus six swept locks and coherent necklace skin (2026-10-15)

Misdreavus now uses a canonical 15-joint skin and the shared production path.
All 41 original parts retain exact stored f32 geometry, normals and triangle
order: 14,175 vertices / 23,794 triangles. Its actual local normal palette uses
blue (74,74,140), pink (247,49,156), white and black. The pack height 204 gives
0.7112 m uniform production scale; the 1.02 normalized height is unchanged.

Five hem lobes remain on the identity root while the upper body blends into
the coherent head. Every fitted facial layer, necklace gem, highlight and cord
shares the head owner. Six hair bases and delayed curl joints use common
position-based weights across all five 24-vertex blue/pink seams. Widened base
transitions prevent attack strain. Packed-byte accessor reuse preserves exact
signed-zero normals in similar gems. Rotation-only clips have neutral endpoints
and use authoritative battle-cue timing.

The GLB is 567,252 bytes with 168 accessors and pinned digest
`65520ddfee63c5e0fa3d39643396f88dfe2b9e2fbefd7bc24355cc2491b59935`.
All 16 Python checks pass; front/rear software views were inspected; focused
Blender output is byte-identical without static JSON. Both focused production Rust checks pass: GPU skinning at pack size and
off-grid seam closure, necklace coherence, hem support and culling checks.
Matched normal native 3D/2D review against Sentret shows a clear face,
necklace and swept curls against taller grass, with no visible seam gaps.
Core frame 3135, hash 3238905079, LCD frame 3849 and time 71.5263535 are
preserved (`state_preserved=true`); captures remain ignored local QA artifacts. Shiny checks remain skipped.
Exact motion, field gaits and browser review remain open. The standard registry
now has 89 variants / 64 species; the all-species goal remains active.

The wider `every_exact_species_is_grounded_and_volumetric` and
`body_pose_uses_elapsed_idle_time_and_exact_current_attack_progress` regressions
also pass on this asset. They establish catalog geometry and elapsed-pose
coverage, without claiming visual acceptance for unrelated models. The native
client build passes.

### Teddiursa and Ursaring planted bear-family skins (2026-10-15)

Both bears now use canonical 12-joint skins on the common production path.
Feet and foot claws stay fixed beneath articulated leg middles and yawing
bodies. Teddiursa's held honey paw inherits the head transform to preserve
muzzle contact. Ursaring's chest, neck and fitted ruff share a deformation
field; its annular ring stays on the torso. Lunar openings, fitted faces and
palms/claws remain coherent. Arms, wrists, ears and tails use rotation-only
source-cue clips, identity roots and neutral endpoints.

Every original indexed triangle corner and triangle order is preserved.
Teddiursa's exact attribute deduplication yields 19,943 vertices / 23,880
triangles across 37 parts. Ursaring's crease-aware ring/ruff shading and
attribute deduplication yield 19,016 vertices / 27,116 triangles across 48 parts.
Both fit the unchanged loader limits. Source normal palettes replace invented
cream colors. Pack heights give 0.6096 m and 1.8034 m respectively; normalized
height remains 1.14 for both.

Teddiursa: 734,596 bytes, 167 accessors, digest
`38a7cdab920c315dac0608ae661e9f9408e9a3b2709000393b88439a3dafbe3b`.
Ursaring: 710,068 bytes, 207 accessors, digest
`3edc0d94b47fe39e7234b958db6c16a2a8254ae548ad21262b4d5994473260a0`.
All 16 Python checks pass. Both focused Rust checks pass: GPU skinning at pack sizes and off-grid planted
paws, honey contact, coherent feature ownership and culling checks. Focused
Blender exports are byte-identical without static JSON. Front/rear software
views were inspected; Matched normal native 3D/2D reviews against Pidgey pass for both bears.
The cub's crescent, held paw and planted feet read clearly; the adult's scale,
annular ring, claws and grounded paws remain clear against the retained grass.
The layered ruff retains its sculpted sharp borders. Rear details were also
inspected in neutral software views. Teddiursa preserves core frame 2596,
hash 1244057618, LCD frame 3287 and time 59.617404872. Ursaring preserves
core frame 2446, hash 1622029825, LCD frame 3230 and time 58.524571168.
Both paired receipts report `state_preserved=true`. Captures remain ignored
local QA artifacts. Shiny checks remain skipped. Exact
motion, field gaits and browser review remain open. The standard registry now
has 91 variants / 66 species; the all-species goal remains active.

The wider `every_exact_species_is_grounded_and_volumetric` and
`body_pose_uses_elapsed_idle_time_and_exact_current_attack_progress` regressions
also pass. These cover catalog geometry and elapsed-pose sampling, without
claiming visual acceptance for unrelated models. The native build passes.

### Snubbull and Granbull folded-ear and jaw skins (2026-10-15)

Both bulldogs now use canonical 13-joint skins on the common production path.
Fixed paws and toe grooves support blended leg middles and yawing torsos.
Snubbull's dress spots stay on the body, while folded ears carry their fitted
tips. Granbull's lower jaw and tusks move together; its fitted mouth and tongue
stay coherent on the head. Fists carry cuffs and knuckle clefts. Rotation-only
idle/attack/hit clips use neutral endpoints and authoritative battle cues.

All original f32 geometry, normals and triangle order remain exact. Snubbull
has 39 parts / 16,555 vertices / 19,516 triangles, 635,380 bytes and 185 accessors.
Digest: `70065daa04c182094aef38bc6772adf416da67efaf666cc9bffe28b7b2854412`.
Granbull has 42 parts / 14,140 vertices / 19,048 triangles, 576,372 bytes and
193 accessors. Digest:
`386b9e7d3742f158136ec0acd0401377fefafc9dc1cf76afbdebbebb5467276e`.
The local normal palettes replace invented colors. Pack sizes give 0.6096 m
and 1.397 m uniformly; normalized height remains 1.06 for both.

All 14 Python checks and both focused production Rust checks pass. Front/rear
software views were inspected. Explicit left-to-right patch centroids preserve
original signed-zero bytes across native and Blender Python versions; final
focused Blender exports are byte-identical without duplicate JSON. Matched normal native 3D/2D reviews against Pidgey pass for both bulldogs.
Snubbull's dress spots, collar, folded ear tips and planted white paws read
clearly. Granbull's scale, curved lower tusks, jaw, cuffs and grounded paws
remain clear against retained grass. Rear details were also inspected in
neutral software views. Snubbull preserves core frame 2806, hash 2193979768,
LCD frame 3541 and time 65.505895167. Granbull preserves core frame 3311,
hash 2711752641, LCD frame 4126 and time 88.313459166. Both paired receipts
report `state_preserved=true`. Captures remain ignored local QA artifacts. Shiny checks remain skipped. Exact motion,
field gaits and browser review remain open. The standard registry now has
93 variants / 68 species; the all-species goal remains active.

The wider `every_exact_species_is_grounded_and_volumetric` and
`body_pose_uses_elapsed_idle_time_and_exact_current_attack_progress` regressions
also pass on these skins. They cover catalog geometry and elapsed-pose sampling,
without claiming visual acceptance for unrelated models. The native build
passes after an explicit f32 annotation fixes a shared dungeon UV closure.

### Houndour and Houndoom bone-armored canine skins (2026-10-15)

Houndour now uses a 13-joint skin; Houndoom uses 12 joints. Fixed paws, toe
clefts, cuffs and adult claws support yawing leg middles and torso. Bone plates
stay on the torso, while the collar, skull pendant and fitted skull eyes stay
coherent with the neck. The head carries facial layers and rigid adult horns.
The adult jaw/tongue, puppy ears and three-stage tails articulate separately;
the adult spade stays coherent with the tail tip. Rotation-only clips retain
identity roots, neutral endpoints and authoritative battle-cue timing.

All original indexed triangle positions/order remain exact. Smooth dorsal
plates and chest markings replace rough flat shading: Houndour has 11,024
vertices / 14,248 triangles across 44 parts; Houndoom has 15,865 vertices /
22,736 triangles across 67 named source parts. Four same-owner claw groups
reduce the adult to 59 runtime primitives, preserving all twelve named claws
through top-level source ranges. Both independent source/runtime hashes remain
pinned, and existing loader limits remain unchanged.

Houndour: 464,124 bytes / 181 accessors; source and packed digest
`f35d0751435b77708a37e8d0c8f5ece071f3de0b65c8fb2586c3775b5d9c1ce6`.
Houndoom: 634,844 bytes / 218 accessors; source digest
`4fe7c0e7bec59d663b355075b40d4ecea6f9ee20620fa2a9da28d1145fb15a80`;
packed digest `2d2d89e4bd3ce410dfbbeee0d49659d1c6b78e4f9d0e143e14fa02b54bfde878`.
Actual normal palettes supply coat/orange/white/black colors; puppy paws use
orange. Pack heights give 0.6096 m and 1.397 m uniformly, with normalized 1.20
height unchanged. Original left-to-right patch centroids preserve exact
native/Blender geometry bytes.

All 15 Python checks pass. Front/rear software views and the corrected smooth
front view were inspected. Focused Blender exports are byte-identical without
static JSON. Both focused production Rust checks pass: GPU skinning at pack sizes and
off-grid fixed paw/cuff support, coherent features, tail movement and culling
bounds. Matched normal native 3D/2D reviews against Pidgey pass for both dogs.
Houndour's corrected smooth back plates, forehead shield, orange paws and
ankle bands read clearly. Houndoom's horns, skull pendant, dorsal plates,
claws and complete curling tail/spade are visible with grounded paw placement.
Houndour preserves core frame 4301, hash 4159166349, LCD frame 5193 and time
103.78049458. Houndoom preserves core frame 5845, hash 1331536335, LCD frame
6684 and time 145.51494033. Both paired receipts report `state_preserved=true`.
Captures remain ignored local QA artifacts. Shiny checks remain skipped; exact motion, field gaits and browser
review remain open. The standard registry now has 95 variants / 70 species;
the all-species goal remains active.

The wider `every_exact_species_is_grounded_and_volumetric` and
`body_pose_uses_elapsed_idle_time_and_exact_current_attack_progress` regressions
also pass. They cover catalog geometry and elapsed-pose sampling, without
claiming visual acceptance for unrelated models. The native client build passes.

### Miltank and Tauros coherent bovine skins (2026-10-15)

Miltank now has a 14-joint skin; Tauros has 17 joints. Fixed cloven hooves
support articulated leg middles and yawing torsos. Miltank's udder, four teats
and rear spots stay coherent on the torso; its forehooves, inner-ear panels and
three-stage tail articulate. Tauros's mane/strands share an owner, while horns,
facial layers and six forehead stud layers stay on the head. Its three tails
have separate base, bend and tuft chains with distinct motion. Rotation-only
clips use neutral endpoints and authoritative battle-cue timing.

All original f32 geometry, normals and triangle order remain exact. Miltank:
45 parts / 17,281 vertices / 17,976 triangles, 651,592 bytes, 210 accessors;
digest `cab41187fb7632ada815e8d5ef241e6ed3e56af5ba964d6195e44cf5a09e851c`.
Tauros: 42 parts / 14,025 vertices / 16,504 triangles, 567,740 bytes, 194
accessors; digest
`9c7a80af7788de4b71e74f6217e1abc0eced54d06e9a6280dbd6fabd57da2edc`.
Local normal palettes replace invented colors. Pack heights give 1.1938 m and
1.397 m uniformly. Tauros's authored body reference excludes raised tail tufts:
1.1175279 from ground to horn tip, while the full 1.28 silhouette remains in
fit/culling bounds. This prevents tail pose from shrinking the bull's body.

The 15 anatomy/rig checks pass; the additional body-reference regression and
final canonical roundtrip pass after the metadata-only reference change.
Front/rear software views were inspected. Original patch centroids keep
native/Blender bytes identical; focused exports match without static JSON.
Both focused production Rust checks pass, including pack sizes, Tauros
body-reference scaling, fixed hooves, coherent features and culling bounds.
Matched normal native reviews pass. Shiny
checks remain skipped; exact motion, field gaits and browser review remain
open. The standard registry now has 97 variants / 72 species; the all-species
goal remains active.

The wider `every_exact_species_is_grounded_and_volumetric` and
`body_pose_uses_elapsed_idle_time_and_exact_current_attack_progress` regressions
also pass. These cover catalog geometry and elapsed-pose sampling, without
claiming visual acceptance for unrelated models. The native client build passes.

Tauros's matched normal 3D/2D review against Pidgey passes: grounded cloven
hooves, the mane and all three tail shafts/tufts are visible at the corrected
body scale. The far horn and some forehead details are obscured at this angle;
front/rear software views cover those details. The paired receipt preserves
core frame 2102, hash 1577348228, LCD frame 2719 and time 48.875408417
(`state_preserved=true`). Miltank's first view exposed incorrect white inner
ears. A magnified actual source frame shows dark interiors; both inner panels
now use black. The binary geometry/weights, nodes, skins, clips, accessors and
materials are identical; only two primitive material assignments and the source
hash changed. Final canonical/palette checks pass.

Miltank's final matched normal 3D/2D review against Pidgey passes. The dark
inner ears now match the source; planted hooves, raised forehooves, udder and
tail remain coherent. The paired receipt preserves core frame 2044, hash
873632486, LCD frame 2639 and time 46.967535708 (`state_preserved=true`).
Both focused Rust checks and the native rebuild pass after the material-only
correction; the final Blender export is byte-identical. Shiny checks remain
skipped at the user's request.


### Delibird planted webs and coherent held sack (2026-10-15)

Delibird now uses an eight-joint canonical standard skin. The original 28 parts,
9,916 vertices and 17,808 triangles preserve exact positions, normals and
triangle order. Normal materials use the local pack's orange/lavender palette,
white and black. Flat webs stay on the identity root; the fitted collar shares
the torso/head field; the lowered flipper and entire held sack share one rigid
owner. The raised flipper, bill and paired crest fans articulate separately.

The 402,984-byte asset replaces static JSON. Neutral digest:
`a1665f55e8b724b62c20970d1f7b663dd06a46707a6c9764531d71e7c6bd1d4d`.
All 12 anatomy/skin Python regressions pass, and focused Blender/native bytes
match. Both focused Rust checks and the wider catalog geometry/elapsed-pose
regressions pass. Front/rear software views were inspected. Normal native review
passes. Shiny checks
are skipped. The registry contains 98 variants / 73 species; exact motion,
field gaits, browser review and the broader all-species goal remain open.


The matched normal native 3D/2D review against Sentret passes. Delibird's
webbed feet meet the retained terrain, the held wing and sack remain joined,
and its collar, bill and crest fans read clearly at the pack-based size. The
paired receipt preserves core frame 2584, hash 2663410303, LCD frame 3230 and
time 58.524416959 (`state_preserved=true`). The native build passes. Rear sack
folds were inspected in the software view; exact motion, field gaits and
browser review remain open. Shiny-specific checks remain skipped.


### Stantler coherent antlers and planted cloven hooves (2026-10-15)

Stantler now uses a twelve-joint standard skin. All 51 original parts preserve
exact positions, normals and triangle order: 15,993 vertices, 20,548 triangles
and height 1.28. Normal materials use the local pack's brown/navy palette, white
and black. The 614,848-byte canonical GLB replaces static JSON; neutral digest
`7508b413c731950b19f25b4f91fd096dedd8d449e8158fa2439e06ae8c98d843`.

Hooves and fitted clefts stay planted, neck/chest share a spatial field, and
antler stems, hollow loops, branches and orbs stay rigid to the head. Ears,
leg middles and the two-stage curled tail articulate separately. Six decoded
skin checks pass. Front/rear software views were inspected. Original ordered
patch centroids are preserved across Python versions; Girafarig's shared recipe
produces unchanged static geometry and materials. All nine anatomy checks pass, and final
Blender/native exports are byte-identical. Both focused production Rust checks pass. The wider catalog grounding and actual elapsed-pose checks also pass.
Normal native review passes. Shiny checks remain
skipped; exact motion, field gaits and browser review remain open. The registry
now contains 99 variants / 74 species; the all-species goal remains active.


The native build and matched normal 3D/2D review against Pidgey pass. Stantler's
hooves meet the retained terrain, its neck and flank markings remain fitted,
and the curled tail stays attached at the verified pack scale. This capture
shows a side profile, so the antler planes are nearly edge-on; the front/rear
software views establish the hollow loops, branches and separate orbs. The
paired receipt preserves core frame 2817, hash 4132824355, LCD frame 3718 and
time 80.771744292 (`state_preserved=true`). Shiny checks remain skipped. Exact
motion, field gaits, browser review and broader species work remain open.


### Girafarig fitted neck and independent rear-facing head (2026-10-15)

Girafarig now uses a thirteen-joint standard skin, preserving all original
indexed triangle positions/order. Curved eye/spot shading reduces vertices
from 24,775 to 14,025; the other 40 parts retain their original normals. The
64 original parts plus eight fitted dark spot centers pack into 57 runtime
parts by grouping spots into five shared material/deformation fields. Root metadata restores source parts
and ordering. The asset has 22,432 triangles, 245 accessors and 594,400 bytes.
Source digest `9b38583cbb8ceee957133a2774eebd309860117c5f3014922201877716781810`;
packed/Rust digest `6c3c3a187cdce79b3adbe6a17a77a5214cfc55b7c5b0dfed356b2dc46d3bcb4c`.

Hooves remain fixed; both body colors share the torso, neck/markings/mane
share a spatial field, and both heads own their fitted facial features. Ears,
leg middles and a three-stage tail articulate separately. Normal colors use
the local pack's gold/pink, white and black; pack height 411 is 1.4986 m.
All nine shared anatomy and eight decoded-skin checks pass. Blender/native
bytes match, and front/rear software views were inspected. Stantler remains
unchanged. Focused production Rust checks passed before the spot centers; final Rust
and normal native review pass. Shiny
checks are skipped. The registry has 100 variants / 75 species; exact motion,
field gaits, browser review and the all-species objective remain open.


The first normal native comparison against Hoppip exposed solid pink front
spots. The source sprite has dark centers inside pink borders; eight fitted
centers now correct that detail. All nine anatomy and eight skin checks pass
again, including fitting each center against its actual outer panel. Final
Blender/native bytes are identical. The initial capture is not final visual
acceptance for the corrected material layering. Shiny checks remain skipped.


The corrected matched normal native 3D/2D review against Pidgey passes.
Dark spot centers sit inside pink borders, the long neck and two-color trunk
remain coherent, and all hooves meet the retained terrain at the pack-based
scale. The second head faces away from this camera; its fitted gaze/grin were
inspected in the rear software view. The paired receipt preserves core frame
2747, hash 2531382474, LCD frame 3381 and time 60.614596083
(`state_preserved=true`). Both final focused Rust checks, the final native
build and wider catalog grounding/elapsed-pose regressions pass after the
added centers. Shiny checks remain skipped; exact motion, field gaits, browser
review and the all-species objective remain open.


### Phanpy and Donphan articulated trunks and planted pads (2026-10-15)

Both elephants now use 13-joint canonical standard skins. Three Phanpy pads
remain planted while its lifted foreleg/pad/nail share one torso-parented
joint. Donphan's pads, twelve nails and ankle cuffs stay fixed; the tire bends
across body/head/trunk stages while nine armor plates remain rigid on their
corresponding joints. Actual posed-volume checks retain every tire/plate join.
Ears own fitted markings, heads own fitted faces, and trunk/tail tips follow
delayed rotation-only source-cue clips. The baby's trunk uses a monotone field
to avoid nearest-segment jumps across its tight bend.

Phanpy preserves 28 original parts; four ear markings are retessellated at two
subdivision levels to fit existing budgets: 32 parts, 18,472 vertices, 31,284
triangles, 163 accessors, 729,772 bytes. Normal colors use the local blue/warm
palette; pack height 108 is 0.508 m. Neutral digest:
`4f4d743119d23709e102ad164adb17c52d236d9dca587c4ef10b3176e6fab0f4`.
Donphan preserves all original positions/normals/triangles: 49 parts, 17,395
vertices, 29,596 triangles, 186 accessors, 608,840 bytes. Its local normal
palette is gray/dark green; pack height 307 is 1.0922 m. Neutral digest:
`7a95df265d655ae2c8f7f273c5e42040da683449abe331b9c667955a295f70ca`.
Both retain normalized height 1.02. Parallel static JSON is retired.

All 17 anatomy and seven shared decoded-skin checks pass. Final Blender/native
bytes match, and front/rear software views were inspected. The old static
builder branches were replaced so production export actually reaches the
canonical skins. Both focused production Rust checks pass. Both matched normal native reviews pass.
Shiny checks are skipped. The registry has 102 variants / 77 species; exact
motion, field gaits, browser review and the all-species objective remain open.


Phanpy's matched normal native 3D/2D review against Pidgey passes. Its small
pack-based scale, grounded pads, lifted forefoot, upturned trunk and fitted ear
markings read clearly against the retained grass. The paired receipt preserves
core frame 2387, hash 2681977306, LCD frame 2965 and time 52.322835457
(`state_preserved=true`). The native build and wider catalog grounding/actual
elapsed-pose regressions pass. Shiny checks remain skipped.


Donphan's matched normal native 3D/2D review against Sentret passes. Grounded
pads and fixed cuffs, the hanging curled trunk, raised tire plates, broad body
and outward tusks read clearly at the verified pack scale. This side angle
obscures part of the far tusk/ear; front/rear software views cover those details.
The paired receipt preserves core frame 2052, hash 4214981733, LCD frame 2677
and time 48.122411497 (`state_preserved=true`). The native build and wider
catalog grounding/actual elapsed-pose regressions pass. Shiny checks remain
skipped; exact motion, field gaits and browser review remain open.


### Marill and Azumarill welded bellies and articulated buoys (2026-10-15)

Both water rabbits now use twelve-joint canonical standard skins. Every
original position, normal and triangle is preserved. Body color regions share
forty welded seam vertices and one spatial field with the adult belly spots.
Faces share the head, ear linings share their bases/tips, feet stay fixed, and
arm paddles and three-stage tail wires articulate with attached buoys.
Source-cue clips use rotations only and neutral endpoints.

Marill has 19 parts, 3,437 vertices, 5,648 triangles, 115 accessors and 191,256
bytes; digest `39c4a5f6e20db9f7c4aeddef3353131a3dda11ddd1befd021dfcb5aa1e88fbe7`.
Azumarill has 25 parts, 4,873 vertices, 7,448 triangles, 140 accessors and 250,292
bytes; digest `19aba9a5fabbfd5f187d24b28f9c10fc291c0312046b6142dfa73d3e11219333`.
Normal palettes use the local pack's species-specific blue/warm colors, white
and black. Pack heights 104 and 207 are 0.4064 and 0.7874 m respectively. Both
retain normalized height 1.02; parallel static JSON is retired.

All six anatomy and eight skin checks pass, including posed ear/spot fitting,
actual wire/ball attachment and the welded seam throughout motion. Final
Blender/native bytes match. Front/rear software views were inspected. Both focused production Rust checks pass. Both normal native reviews pass. Shiny checks are skipped. The
registry has 104 variants / 79 species; exact motion, field gaits, browser
review and the all-species objective remain open.


Both matched normal native 3D/2D reviews pass. Marill against Rattata shows
planted feet, its round ears, clean belly seam and attached zigzag buoy tail;
the paired receipt preserves core frame 1933, hash 2779120648, LCD frame 2547
and time 45.426498708 (`state_preserved=true`). Azumarill against Sentret shows
the bent/upright ears, fitted spots, wavy belly boundary, planted paddles and
attached buoy at the verified pack size; its receipt preserves core frame
2080, hash 2842356162, LCD frame 2671 and time 47.184705956
(`state_preserved=true`). Rear details were also inspected in software views.
The native build and wider catalog grounding/actual elapsed-pose regressions
pass. Shiny checks remain skipped; exact motion, field gaits, browser review
and the all-species objective remain open.


### Natu and Xatu planted talons and coherent symbolic plumage (2026-10-15)

Natu now uses ten joints; Xatu uses eleven. Toes/talons remain fixed while
ankles blend into the body. Fitted faces share the head; wing markings on
both faces share each rigid wing. Natu articulates its crest and two tail
feathers; Xatu articulates independent two-stage head streamers, with collar,
chest symbols and rear bars sharing the torso/head field. Source-cue clips
use rotations only, neutral endpoints and delayed streamer tips.

Natu preserves 42 parts exactly; four large leading-wing coatings use two
subdivision levels to fit existing limits. It has 46 parts, 18,853 vertices,
28,820 triangles, 189 accessors and 766,412 bytes; neutral digest
`42b91ce42d9cbfb2367f3640cdd1641c5da72aedb65da5cc41b412d9d8b09fd1`.
Xatu preserves all original geometry: 49 parts, 15,289 vertices, 24,060 triangles,
199 accessors and 593,840 bytes; digest
`5196df1ef836e954c4904ec06cf3448e3a701eba946050d5a38a477d116e785b`.
Normal colors use the local green/red, white and black palette. Pack heights
8 and 411 are 0.2032 and 1.4986 m. Both retain normalized height 1.04. Static
JSON is retired and builder hooks directly export canonical skins.

All seven anatomy and seven skin checks pass, including fitted front/back
wing details and posed Xatu symbolism. Both focused Rust checks pass at the
verified pack sizes. Blender/native bytes match; front/rear software views
were inspected. The native build and both normal native reviews pass. Shiny checks
are skipped. The registry has 106 variants / 81 species; exact motion, field
gaits, browser review and the all-species objective remain open.


Natu's matched normal native 3D/2D review against Pidgey passes. Its small
pack-based scale, planted toes/talons, round body, swept crest and layered wing
markings read clearly against the retained grass. The paired receipt preserves
core frame 2260, hash 1900440669, LCD frame 2941 and time 53.010732125
(`state_preserved=true`). The native build and wider catalog grounding/actual
elapsed-pose regressions pass. Shiny checks remain skipped.


Xatu's matched normal native 3D/2D review against Pidgey passes. Its tall
pack-based silhouette, planted toes/talons, folded sleeves, fitted chest and
wing markings, hooked bill and swept streamers read clearly. The far streamer
and rear bars are partly obscured at this angle; rear software views cover
them. The paired receipt preserves core frame 2043, hash 2373058012, LCD frame
2672 and time 46.734551292 (`state_preserved=true`). Both normal native reviews,
the native build and wider catalog grounding/actual elapsed-pose regressions
pass. Shiny checks remain skipped; exact motion, field gaits, browser review
and the all-species objective remain open.


### Electric sheep coherent fleece, band seams and delayed beacons (2026-10-15)

Mareep/Flaaffy now use thirteen joints each; Ampharos uses nine. Ground supports
stay fixed, fleece/crown curls retain fitted grooves, head features/horns stay
coherent, and the banded ears/tails share spatial weights across their seam
rings. Flaaffy's cape and Ampharos's belly/collars follow their body fields.
Beacons and highlights share terminal tail joints. Source-cue clips use
rotations only, neutral endpoints and delayed tips.

The first two stages adopt the current shared band's refined connected-ring
geometry; 16 Mareep and 23 Flaaffy parts retain exact original geometry.
Ampharos preserves 25 parts while its lower body cage is widened/lowered to
retain posed foot contact. Paddle-length weighting fixes flipper stretch.
All six sheep, four Ampharos and eight shared skin checks pass, including
actual posed foot/body/ring contacts and closed band seams through every clip.
Final Blender/native bytes match; front/rear software views were inspected.
The native build passes. Both focused production Rust checks pass. All three normal native reviews pass.

Mareep: 26 parts, 6,382 vertices, 11,840 triangles, 140 accessors, 314,456 bytes;
digest `058c61bce99e43bc24f2bf17a0cabe8bb62a0a6475fcefcd081d34cf9f580c8d`.
Flaaffy: 33 parts, 10,302 vertices, 16,636 triangles, 170 accessors, 427,480 bytes;
digest `eaabe519c0cb6edfa85b9b9ddc5a9b1ca5f5a4fa3bbdedfe948dc57bb68bb8d6`.
Ampharos: 26 parts, 2,402 vertices, 4,700 triangles, 121 accessors, 136,560 bytes;
digest `19d3138402ccf1b0ffed193b47dea3a25ec684a5f40a1593079e3c5fac04527e`.
Normal palettes use the local pack; heights 200/207/407 are 0.6096/0.7874/1.397 m.
Static JSON is retired. Shiny checks are skipped. The registry has 109 variants
/ 84 species; exact motion, field gaits, browser review and the full objective
remain open.


All three matched normal native 3D/2D reviews against Pidgey pass. Mareep's
planted supports, scalloped fleece, fitted curl and connected bands read clearly;
its receipt preserves core frame 1229, hash 2673380250, LCD frame 1794 and time
31.150453125. Flaaffy's planted pads, fitted toe caps, cape/crown curls and tail
remain coherent; its receipt preserves core frame 1411, hash 51933262, LCD frame
1970 and time 35.068303209. Ampharos's strengthened body/foot joins, long neck,
collar rings, flippers and banded lamp tail read clearly at the verified scale;
its receipt preserves core frame 2080, hash 1828807289, LCD frame 2798 and time
50.957366501. Every paired receipt has `state_preserved=true`. Ampharos's profile
obscures some facial/belly detail; front/rear software views cover those areas.
The native build, focused Rust checks and wider catalog grounding/actual
elapsed-pose regressions pass. Shiny checks remain skipped; exact motion,
field gaits, browser review and the full all-species objective remain open.

### Pichu, Cleffa and Igglybuff planted baby skins (2026-10-15)

Pichu now has an original species-specific sculpture: oversized mouse head,
broad black ears with fitted yellow insets, warm cheeks, jagged black collar,
short reaching paws, supporting shins, planted toes and a rooted lightning tail.
Cleffa and Igglybuff preserve all 17/19 parts' exact neutral f32 geometry. All
three use their local pack normal palettes and `100` (0.3048 m) Pokédex height.
The generic Pichu prefab and static fairy JSON copies are retired.

The three standard skins have 10/9/6 joints. Feet stay fixed; torso/head fields
keep joins continuous. Faces stay with the head, ears and arms articulate,
Cleffa's rolled tuft shares its fitted crease owner, Igglybuff's three crown
lobes share an owner, and tails move from rooted pivots. Rotation-only idle,
attack and hit clips have neutral endpoints and production cue timing.

Eight skin checks and nine fairy sculpture checks pass, including actual posed
appendage contacts, foot grounding, face fitting, winding and strain. Blender
exports match native bytes. Both focused Rust integration checks, the native
build, and the wider catalog grounding/elapsed battle pose regressions pass.
Front/rear software previews were inspected. All three normal paired reviews pass;
shiny checks remain skipped. Registry: 112 variants / 87 species. Exact motion,
field gaits, browser review and the full all-species objective remain open.

Pichu: 24 parts, 9,462 vertices, 9,104 triangles, 381,376 bytes;
digest `45b03dd0a788ebf6714334c7c25ab472bab589aa9d5b70e2512ad105b7e992be`.
Cleffa: 17 parts, 14,716 vertices, 24,376 triangles, 587,892 bytes;
digest `25d7809325a90fc9b89a9451ee3785256001f63b95cfb0a1568e7a3d7bd45725`.
Igglybuff: 19 parts, 16,964 vertices, 28,684 triangles, 630,808 bytes;
digest `de4ed97b7ae946d32303e70a839c3880f24b1a01e0036b72be5aa78bc97010a4`.
See [baby skin authoring and checks](baby-fairy-sculptures.md).

All three matched normal native 3D/2D reviews pass. Pichu's broad tipped ears,
fitted gaze/cheeks, jagged collar, planted toes and lightning tail read clearly
against Pidgey. Cleffa's star ears, rolled tuft, tiny face and planted feet read
clearly against Sentret. Igglybuff's crown, forehead spiral, red eyes and grounded
petal feet read clearly against Pidgey. Their small production scales follow the
pack. Each paired receipt preserves state, and encounter terrain is accepted.
Pichu preserves core frame 1654, hash 892235761, LCD frame 2259 and time
39.108565458. Cleffa preserves core frame 1639, hash 2539579671, LCD frame 2253
and time 39.007266249. Igglybuff preserves core frame 1921, hash 3830763378,
LCD frame 2624 and time 46.141430541. Profile views conceal some rear detail;
the inspected front/rear software views cover it. Shiny checks remain skipped.

### Magby coherent crown, clawed arms and open bill (2026-10-15)

Magby now has a standard eleven-joint skin. The root fixes feet and toe claws;
leg middles blend into the torso. Body/collar weights join the torso, neck and
head; the fitted belly shares the body field. The fitted gaze, annular open bill
and recessed oral backing stay with the head. Five embedded crown lobes move
as one bank, raised hands retain their claws, and a two-stage tail bends from
a rooted base. Rotation-only clips use neutral endpoints and production cue timing.

The previous static model exceeded the skin vertex limit. Twenty-nine parts
retain exact neutral f32 geometry. Four crown lobes, the belly and pupil/glint
surfaces use reduced sampling; the central crown retains the authoring span.
The canonical skin has 38 parts, 19,764 vertices, 35,032 triangles, 766,584 bytes;
digest `aab2245cffeaa6b35e780fc79dee5fa83f7d6b482103151f3e1cfd6acec8c60b`.
The local normal palette is purple-pink/orange/white/black. Pack height `204`
sets the 0.7112 m production scale. Static JSON is retired.

All ten sculpture and seven skin checks pass, including actual posed roots,
fixed claws, face clearance, winding, strain and filled/detached negative controls.
Blender bytes match; front/rear software views were inspected. Both focused Rust
checks, the native build and wider catalog grounding/elapsed battle pose
regressions pass. The normal paired review passes. Shiny verification stays skipped.
Registry: 113 variants / 88 species. Exact motion, field gaits, browser review
and the full all-species objective remain open.

The matched normal native 3D/2D review against Sentret passes. Magby's planted
feet/claws, compact body, fitted bib, open bill, raised clawed hands, lobed crown
and pointed tail remain readable at the verified scale. The terrain is accepted;
the paired receipt preserves core frame 1123, hash 854216493, LCD frame 1693,
time 29.498578709 and `state_preserved=true`. Front/rear software views cover
surfaces concealed by the production profile. Shiny checks remain skipped.

### Elekid open contact blades and coherent striped arms (2026-10-15)

Elekid now has a standard nine-joint skin. Feet stay fixed; leg middles blend
into the torso. Body, waist band and fitted bolt share a spatial field. Gaze and
mouth stay with the upper body. Each arm connector, striped forearm, fist and
three claws share one owner, keeping wraps fitted through movement. Two plug
blades rotate from rooted pivots while retaining their actual through holes.
Rotation-only idle/attack/hit clips have neutral endpoints and production cue timing.

Twenty-eight parts retain exact neutral f32 geometry. Only four arm bands reduce
longitudinal sampling. Electabuzz's 70 parts and colors regenerate unchanged.
Elekid: 32 parts, 19,466 vertices, 33,128 triangles, 139 accessors, 715,700 bytes;
digest `5087eaed56db849c823c7e069ef7879375ebb39c6d3f68c84de469b858ad076a`.
Normal palette uses local pack yellow (255,255,41), white and black. Pack height
`200` sets 0.6096 m production scale. Static Elekid JSON is retired.

Eight skin checks pass, including actual posed roots, fixed feet, winding,
strain and five rays through each posed plug hole against all posed anatomy.
All thirteen pair sculpture checks pass, retaining neutral hole/filled/detached controls and
fitted layer support checks. Blender bytes match; front/rear software views
were inspected. Both focused Rust checks, the native build and wider catalog
grounding / elapsed battle pose regressions pass. The normal paired review passes. Shiny verification stays skipped. Registry: 114 variants /
89 species. Exact motion, field gaits, browser review and the full objective remain open.

The matched normal native 3D/2D review against Hoppip passes. Elekid's yellow
egg body, fitted gaze and bolt, open contact holes, striped forearms, clawed
fists and planted feet read clearly at the verified scale. The terrain is
accepted; the paired receipt preserves core frame 2194, hash 498538258, LCD
frame 2811, time 49.87255725 and `state_preserved=true`. Inspected front/rear
software views cover surfaces concealed by the battle profile. Shiny checks
remain skipped.

### Smoochum fitted gaze, recessed kiss rim and coherent bob (2026-10-15)

Smoochum now has a standard eleven-joint skin. The root fixes the toes; leg
middles blend into the body. Body, chest, fitted bib and neck share a spatial
field. Head, fitted gaze, recessed mouth, kiss rim, bob and fringe stay coherent.
Two side locks move from rooted pivots; the four crown tufts share one owner.
Each small palm and its three fingers move with the arm. Rotation-only clips
use neutral endpoints and production attack/hit timing.

Sixteen parts retain exact neutral f32 geometry, including body, head, lips,
fringe, arms, legs, fingers and tallest tuft. Dense facial layers and selected
round/hair parts reduce sampling to fit the skin limit. Jynx's 45 parts and
colors regenerate unchanged. Smoochum: 40 parts, 19,907 vertices, 34,392 triangles,
202 accessors, 815,752 bytes; digest
`90026857a3c50573313f78f6fbd0fd12ed25f7cdae53bbd869a6a0f5e2c792d2`.
The local normal palette uses purple-pink (165,8,156), gold (239,198,33), white
and black. Pack height `104` sets 0.4064 m production scale. Static JSON is retired.

All eleven final sculpture checks and seven skin checks pass, including posed
roots, fixed feet, winding, strain, fitting and recessed-mouth negative controls.
Blender bytes match; front/rear software views were inspected. Both focused Rust
checks and the native build pass. The normal paired review passes.
Shiny verification stays skipped. Registry: 115 variants / 90 species. Exact
motion, field gaits, browser review and the full objective remain open.

The matched normal native 3D/2D review against Sentret passes. Smoochum's
bob, crown tufts, fitted gaze, recessed gold kiss rim, white collar, small hands
and planted toes remain readable at the verified small scale. Terrain is
accepted; the paired receipt preserves core frame 1937, hash 350074450, LCD
frame 2551, time 45.114936375 and `state_preserved=true`. Front/rear software
views cover surfaces concealed by the battle profile. The wider catalog
grounding and actual elapsed battle pose regressions pass. Shiny checks remain
skipped.

### Tyrogue continuous crown, bent arms and wrapped wrists (2026-10-15)

Tyrogue now has a standard twelve-joint skin. Shoes and fitted toe contours stay
fixed. Body/neck weights retain contact, crown/gaze share the head, ears retain
their fitted contours, and each bent arm has separate upper-arm/forearm owners.
Wrist wraps, palms and all three fingers share forearm owners. Belt/shorts stay
with the torso; leg middles blend into the body. Rotation-only idle/attack/hit
clips use neutral endpoints and production cue timing.

Nineteen parts retain exact neutral f32 geometry: body, belt, shorts, shorts
legs, arms, wraps, fingers, legs and shoes. The crown and selected facial/round
parts reduce sampling; face layers regenerate on the crown. Raised toe contours
fix clearance at stored face centers/edge midpoints. Tyrogue: 40 parts, 19,924
vertices, 33,424 triangles, 195 accessors, 764,016 bytes; digest
`6f08d0c81d0c633e32fd55a565109a56759eeb7250212e0e4dfef93812dc71a2`.
The local normal palette supplies pink (189,123,148), brown (140,57,24), white
and black; pack height `204` sets 0.7112 m production scale. Static JSON is retired.

All ten final sculpture checks and seven skin checks pass, including actual
posed roots, fixed shoes, winding, strain, face clearance and detached controls.
Blender bytes match; final front/rear software views were inspected. Both focused
Rust checks, the native build and normal paired review pass. Shiny checks stay skipped.
Registry: 116 variants / 91 species. Exact motion, field gaits, browser review
and the full objective remain open.

The matched normal native 3D/2D review against Sentret passes. Tyrogue's
continuous crown, fitted gaze, wrist wraps and three-finger hands, joined shorts
and planted shoes remain readable at the verified scale. Terrain is accepted;
the paired receipt preserves core frame 2543, hash 3028213223, LCD frame 3253,
time 59.500983002 and `state_preserved=true`. Front/rear software views cover
surfaces concealed by the production profile. Shiny checks remain skipped.

The wider catalog grounding and actual elapsed battle pose regressions pass.

### Hitmontop fixed spin point and raised balance limbs (2026-10-15)

Hitmontop now has a standard eleven-joint headstand skin. The root fixes the
spin point to the floor. Helmet/gaze/folds yaw together; body, belly and neck
share a spatial field. Raised legs retain their ankle wraps, blue feet, fitted
pads and three white spikes. Each arm owns its cuff and fist. A three-stage tail
ends with one owner for the blue ball and white point. Rotation-only clips use
neutral endpoints and production battle cue timing. Exact full spinning cycles
remain open.

Twenty-five primary solids retain exact neutral f32 geometry. Only ten fitted
markings reduce sampling. Hitmontop: 35 parts, 17,394 vertices, 29,828 triangles,
163 accessors, 659,272 bytes; digest
`554e1edaafe9860ab6aa4b5c306a67549b3f8e7c2773e3adefad4901867a3de3`.
The local normal palette uses gold (198,156,33), blue (82,82,173), white and black;
pack height `407` sets 1.397 m production scale. Static JSON is retired.

All eleven final sculpture and seven skin checks pass, including posed
roots, an immutable spin point, raised feet, winding, strain and detached controls.
Blender bytes match; front/rear software views were inspected. Both focused Rust
checks, the native build and wider catalog grounding / elapsed battle pose
regressions pass. The final normal paired review passes.
Shiny checks stay skipped. Registry: 117 variants / 92 species. Exact motion,
field gaits, browser review and the full objective remain open.

The final matched normal native 3D/2D review against Pidgey passes with visible
terrain. The grounded spin point, broad helmet and fitted gaze, raised spiked
feet, wrapped hands and tail ball read clearly at the verified scale. The terrain
is accepted; the receipt preserves core frame 2286, hash 79035160, LCD frame
2974, time 53.206215084 and `state_preserved=true`. Front/rear software views
cover surfaces concealed by the battle profile. Two earlier captures with blank
terrain were rejected after an embedded grass-shader import error; rebuilding
with the current corrected shader restored terrain. Those captures do not
establish visual acceptance. Shiny checks remain skipped.


### Continuous Hitmontop headstand idle (2026-10-15)

The helmet now completes one forward turn per 3.2-second idle loop, carrying
all articulated descendants around the fixed spin point. A smooth speed change
slows the turn near the neutral boundary without reversing it. Literal identity
endpoints retain the standard reader contract; shortest-arc quaternion
interpolation crosses the signed quaternion seam continuously. Attack/hit timing
stays source-cue-relative. Exact move-specific choreography remains open.

Neutral anatomy, materials, joints, weights and all original geometry digests
are unchanged. The updated standard GLB has 164 accessors and 660,816 bytes.
Its neutral digest remains
`554e1edaafe9860ab6aa4b5c306a67549b3f8e7c2773e3adefad4901867a3de3`.
Eight Python skin checks pass, including 161 trajectory samples proving a full
forward revolution, smooth wrap, stationary spin point, raised feet, winding,
strain and actual posed contacts. Four fixed-camera phase renders were inspected.
Blender regeneration is byte-identical. Three focused Rust checks, the native
build and all-species 9/30/60 Hz production playback regression pass. The updated
spin asset's normal paired terrain review passes. Shiny checks remain skipped;
the all-species goal is active.

The updated spin asset's matched normal 3D/2D review against Pidgey preserves
core frame 2565, hash 2286343595, LCD frame 3347, time 61.322099125 and
`state_preserved=true`. Terrain is accepted and visible. The side-phase snapshot
retains floor contact and coherent raised limbs; fixed-camera phase renders
cover front/rear details throughout the revolution.

### Hitmonchan planted boots and articulated boxing gloves (2026-10-15)

Hitmonchan now has a standard twelve-joint skin. Boots, ankle cuffs and piping
stay fixed. Body/neck weights retain contact; crown and fitted gaze share the
head. Shoulder guards, upper arms and forearms articulate separately. Gloves,
cuffs, thumbs and fitted seams share wrist owners. Both mitts advance during the
attack; idle uses smaller guard motion. Leg middles move above fixed ankles.
Rotation-only clips retain neutral endpoints and production source-cue timing.
Exact move-specific choreography remains open.

Twenty primary parts preserve exact neutral f32 geometry. The crown and selected
markings reduce sampling; fitted gaze regenerates against the actual crown.
Hitmonchan: 36 parts, 18,362 vertices, 30,096 triangles, 177 accessors, 716,544 bytes;
digest `59c5d015861346b850e7aca45fa3056246a8d8bc950ce5d9793a0035d7856daf`.
The local normal palette supplies tan (173,123,99), pink (222,24,148), white and
black; pack height `407` sets 1.397 m production scale. Static JSON is retired.

All eleven sculpture and eight skin checks pass, including crown-cap continuity,
actual posed roots, fixed boots/cuffs, fitting/clearance, winding, strain, forward
mitt movement and detached controls. Blender bytes match; front/rear and attack/hit
software views were inspected. Both focused Rust checks pass. The native build and both wider catalog grounding / elapsed battle pose
regressions pass. The normal paired review passes. Shiny checks stay skipped.
Registry: 118 variants / 93 species. Exact motion, field gaits, browser review
and the full objective remain open.

The matched normal native 3D/2D review against Sentret passes. Hitmonchan's
crown, asymmetrical guard, pink mitts and kit, pleated hem and planted boots
remain readable at the verified scale. The held glove obscures part of the gaze;
the inspected attack/hit phase views cover the changing arm positions. Terrain
is accepted; the receipt preserves core frame 2807, hash 2119763869, LCD frame
3550, time 64.156540085 and `state_preserved=true`. Shiny checks remain skipped.

### Hitmonlee continuous coils and a planted-support kick (2026-10-15)

Hitmonlee now has a standard ten-joint skin. The right foot, claws and sole stay
fixed. Left hip/knee joints lift and advance the attacking foot, then return it
to neutral; idle/hit keep both feet planted. The continuous nine-rib leg coil
blends between hip and knee. The lifted foot, claws and sole share the knee.
The right support field joins fixed toes to the torso. Upper-body gaze stays
fitted; arm/wrist fields bend the five-rib wrist coils while palms and curled
fingers share wrist owners. Clips keep neutral endpoints and production cue timing.
Exact move-specific choreography remains open.

Twenty-seven parts retain exact neutral f32 geometry, including all four coiled
solids and primary anatomy. Only pupils/cores and sole markings reduce sampling.
Hitmonlee: 33 parts, 19,747 vertices, 35,276 triangles, 151 accessors, 775,640 bytes;
digest `ac4facbaf5bc5ea34f73342afaa5bc98e8ffa962e8704c7c7d34da37f8ffd0ac`.
The local normal palette supplies ochre (181,115,41), gray (123,132,123), white and
black. Pack height `411` sets 1.4986 m production scale. Static JSON is retired.

All nine sculpture and eight skin checks pass, including actual posed roots,
fixed support, a lifted/advancing kick, neutral return, winding, strain, coil ring
density and detached controls. Blender bytes match; front/rear and kick/hit
software views were inspected. Both focused Rust checks, the native build and
wider catalog grounding / elapsed battle pose regressions pass. The normal paired
review passes. Shiny checks stay skipped. Registry: 119 variants /
94 species. Exact motion, field gaits, browser review and the full goal remain open.

The matched normal native 3D/2D review against Pidgey passes. Hitmonlee's
headless silhouette, fitted gaze, coiled limbs, curled hands and planted feet
remain readable at the verified scale. The idle snapshot complements the
inspected kick/hit phase views. Terrain is accepted; the receipt preserves core
frame 2585, hash 1293280905, LCD frame 3305, time 58.295432041 and
`state_preserved=true`. Shiny checks remain skipped.


### Togepi and Togetic: complete anatomy retained in standard skins

`tools/egg_fairy_glb.py` replaces both static JSON assets with canonical standard
GLBs. Togepi has seven joints, 27 parts, 17,825 vertices and 28,896 triangles
(720,576 bytes, 119 accessors). Togetic has ten joints, 37 parts, 16,755 vertices
and 26,728 triangles (710,840 bytes, 178 accessors). Every neutral f32 position,
normal, color and triangle is preserved. Cached raw recipe geometry avoids
rebuilding it for normalized positions and skin pivots.

Togepi's hollow shell and fitted rings share a rigid shell owner. Its rounded
star head, face and raised arms articulate independently. Togetic's continuous
pear torso and long neck use a shared spatial weight field; the head carries
its fitted face, and each wing carries all its feathers and seams. Both use an
identity floor root for their feet and fitted toe details, with supported leg
middles. Idle, attack and hit use the existing source-cue timing, rotation-only
tracks and literal neutral endpoints. Pack heights are 0.3048 m and 0.6096 m.

Neutral pins: Togepi
`975cb4f931bddf2545df3621a30a743f70bb3662f218c61eb5ae90879d34b53b`;
Togetic
`5971f35977b196a3a2a62472e147b9b0743a53573c66770e9b4bdbc892dea912`.

All 18 sculpture tests and six shared skin tests passed, as did the two focused Rust rig/GPU regressions and the two catalog/timing regressions. Blender regeneration is byte-identical for both GLBs. The production native client built successfully; normal 3D/2D battles against Pidgey were visually inspected with matching game state and accepted, visibly rendered terrain. Shiny checks were skipped as requested.

Final native pairs: Togepi core 2781/hash 1494234955, LCD 3504, time
64.27343179; Togetic core 3708/hash 1318871937, LCD 4500, time 84.546857166.
Both have `state_preserved=true` and visible accepted terrain. Final ignored
QA prefixes are `togepi-normal` and `togetic-normal` under
`target/pokemon-native-review`. The registry now contains 121 variants.
The larger model goal remains open, including move-specific choreography,
field gaits and browser review.


### Remoraid and Octillery: fins, curling tentacles and a hollow aiming siphon

`tools/jet_fish_glb.py` replaces both static JSON assets with standard GLBs,
retaining every neutral f32 position, normal, color and triangle. Remoraid has
18 parts, 6,822 vertices, 7,724 triangles and seven joints (263,080 bytes,
89 accessors). Octillery has 52 parts, 19,798 vertices, 21,740 triangles and
11 joints (741,696 bytes, 229 accessors). The existing loader limits are intact.

Remoraid uses a floor-height preserving body yaw, caudal body field, rigid tail
lobes and individual fin strokes. Octillery uses a shared mantle/arm field,
fixed lower supports, eight phased upper curls and a connected aiming siphon.
Sucker rims and wells share their actual arm deformation field. Both siphon
walls share the same longitudinal field, preserving a real open mouth.
The broad tip transition and bounded curl angle preserve local edge strain.
Fitted faces, stripes and crown spots follow their actual supports. All clips
retain the standard rotation-only idle/source-cue timing and neutral endpoints.
Pack heights are 0.6096 m and 0.889 m.

Neutral pins: Remoraid
`02f2a0f1061633dd1b6772fc886c1ccce95ddc5237d63a8e3ca29a280a6d0514`;
Octillery
`11c7a88eb486c7a59ae19980866dcfc9a9d0f25510be9472abfaa419a8a2c36f`.

All 16 Python anatomy/skin checks, two focused Rust rig/GPU regressions and two wider catalog/timing regressions passed. Blender regeneration produced byte-identical GLBs. The production native client built successfully; final paired 3D/2D battles were visually inspected with preserved game state, accepted terrain and visible ground. Shiny checks were skipped as requested.

Remoraid against Sentret: core 2552/hash 4071485440, LCD 3222, virtual time 58.125843916; `state_preserved=true`.

Octillery against Pidgey: core 2956/hash 819259712, LCD 3720, virtual time 68.988857374; `state_preserved=true`.

Final QA prefixes are `remoraid-normal` and `octillery-normal` under ignored
`target/pokemon-native-review`. Front/rear and fixed-camera neutral, idle and
attack art views were also inspected. The registry now contains 123 variants.
The full model goal remains open, including move-specific choreography,
field gaits and browser review.


### Swinub and Piloswine: planted hooves and articulated sniffing coats

`tools/fur_pig_glb.py` replaces both static JSON assets with canonical standard
skins, preserving every neutral f32 position, normal, color and triangle.
Swinub has 22 parts, 7,044 vertices, 8,496 triangles and eight joints (278,932
bytes, 97 accessors). Piloswine has 22 parts, 5,062 vertices, 9,432 triangles
and nine joints (211,732 bytes, 90 accessors). Existing loader limits are intact.

Both rigs keep all four hooves on the identity floor root. A broad shank blend
uses each leg's centerline to avoid circumferential U8 weight steps around tiny
pole rings. The coat transitions into a sniffing head; its fitted eyes, nose,
nostrils, front fur strands and ivory tusks remain coherent. Swinub's dorsal
stripes stay on the body and its rear wisps articulate together. Piloswine's
drooping ears have separate owners. The clips retain the standard rotation-only
idle/source-cue timing and literal neutral endpoints. Pack heights are 0.4064 m
and 1.0922 m.

Ordered patch centroids preserve the original Python 3.9 sculpture under newer
Blender Python. The dorsal-patch helper accepts the existing stable patch
function for this recipe while retaining its default behavior for other models.
Both complete canonical GLBs remain byte-identical in Python and Blender.

Neutral pins: Swinub
`99b43c8174c27d71d246f211d8ca6a0ce2764cb9d6e60cc2153b1fe25ffc0591`;
Piloswine
`b6b3ef74640643f89b461d55f276157aad0cfb2f7a39d17214b9a9eb8d3ea0b9`.

All 13 Python anatomy/skin checks, two focused Rust rig/GPU regressions and two wider catalog/timing regressions passed. Blender regeneration is byte-identical to both canonical GLBs. The production native client built successfully. Final paired 3D/2D battles were visually inspected with preserved game state and accepted, visibly rendered terrain. Shiny checks were skipped as requested.

Swinub against Sentret: core 1762/hash 3963713493, LCD 2338, virtual time 41.149872167; `state_preserved=true`.

Piloswine against Rattata: core 1919/hash 3115544402, LCD 2543, virtual time 44.664665584; `state_preserved=true`.

Final QA prefixes are `swinub-normal` and `piloswine-normal` under ignored
`target/pokemon-native-review`. The registry now contains 125 variants. The full
model goal remains open, including the remaining catalog, move-specific
choreography, field gaits and browser review.


### Sneasel: coherent hooked-claw strike and asymmetric feathers

`tools/sneasel_glb.py` replaces the static JSON with a canonical standard skin
while retaining every neutral f32 position, normal, color and triangle. The
asset has 38 parts, 8,779 vertices, 9,936 triangles and 13 joints (377,268 bytes,
176 accessors), within the existing loader limits. Pack height is 0.889 m.

Feet and curved toe talons stay on the identity floor root. The torso, neck and
fitted face articulate above supported hocks. Each palm and both hooked hand
claws share one wrist owner; the raised left hand and both talons physically
advance into the attack while idle uses a smaller guard stroke. Independent
ear and plume joints retain the silhouette, and the three tail feathers and
rooted stem move together with floor-height preserving yaw. Fitted face and
forehead gem follow the head; the chest gem follows the torso. Standard
rotation-only idle/source-cue timing and neutral endpoints are retained.

Ordered patch and eye-inset centroids preserve the original Python 3.9 sculpture
under Blender's newer Python. The neutral f32 digest is
`0509cb264ed9206131887afeee6bb84b852aecfbf0d0743e4c46f08652d0f19a`.

All 15 Python anatomy/skin checks, two focused Rust rig/GPU regressions and two wider catalog/timing regressions passed. Blender regeneration is byte-identical to the canonical GLB. The production native client built successfully. Front/rear and fixed-camera neutral, idle, strike and recovery views were inspected, followed by a paired normal 3D/2D battle against Pidgey with preserved game state and accepted, visibly rendered terrain. Shiny checks were skipped as requested.

Final normal pair: core 1202/hash 3785036497, LCD 1780, virtual time 30.835087251; `state_preserved=true`.

Final QA prefix: `sneasel-normal` under ignored `target/pokemon-native-review`.
The registry now contains 126 variants. The full model goal remains open,
including the remaining catalog, move-specific choreography, field gaits and
browser review.


### Skarmory: rigid metal wing strokes, planted talons and opening bill

`tools/skarmory_glb.py` replaces the static JSON with a canonical standard skin,
preserving every neutral f32 position, normal, color and triangle. The asset
has 42 parts, 3,635 vertices, 4,288 triangles and ten joints (181,568 bytes,
167 accessors), within existing loader limits. Pack height is 1.7018 m.

All eight hooked talons stay on the identity floor root beneath supported
haunches and hocks. Each wing's steel leading edge and four red blades share
one rigid owner, preserving their shapes and overlap during the stroke. The
neck and head articulate through a spatial field; rigid armor seams follow
their actual support sections. The fitted eyes, crest and four nape plumes
follow the head. The tail spear has its own joint, and the connected lower bill
and dark mouth seam open from a shared hinge. The jaw never closes past neutral.
Standard rotation-only idle/source-cue timing and neutral endpoints are retained.

Ordered patch and eye-inset centroids preserve the original Python 3.9 geometry
under Blender's newer Python. The neutral f32 digest is
`4a314b8b28c7023ad5044af9003e534b194def610a87be4865b89bde70c1e997`.

All 15 Python anatomy/skin checks, two focused Rust rig/GPU regressions and two wider catalog/timing regressions passed. Blender regeneration is byte-identical to the canonical GLB. The production native client built successfully. Front/rear and fixed-camera neutral, idle, wing-stroke and recovery views were inspected, followed by a paired normal 3D/2D battle against Hoppip with preserved game state and accepted, visibly rendered terrain. Shiny checks were skipped as requested.

Final normal pair: core 1332/hash 453190722, LCD 1965, virtual time 35.581795208; `state_preserved=true`.

Final QA prefix: `skarmory-normal` under ignored `target/pokemon-native-review`.
The registry now contains 127 variants. The full model goal remains open,
including the remaining catalog, move-specific choreography, field gaits and
browser review.


### Smeargle: a coherent held brush-tail painting stroke

`tools/smeargle_glb.py` replaces the static JSON with a standard skin. All 39
non-arm parts preserve their original neutral f32 positions, normals, colors
and triangles. The 26 arm bands adopt the current shared continuous two-span
arm builder, adding 382 vertices and 728 triangles while preserving adjacent
ring frames and normals. Four arm material groups use the existing packed-model
pattern; all 65 source parts and their order round-trip through source ranges.
The runtime has 43 primitives, 16,875 vertices, 21,004 triangles and 13 joints
(669,624 bytes, 204 accessors). Existing loader budgets remain unchanged.

Paws, toe grooves and brown ankle markings stay on the identity floor root.
The face, floppy ears and beret articulate above supported shins. The right
palm, its fingers, held tail section and both brush layers share a painting
owner; the tail blends back to an attached torso root. The paint tip advances
through a forward stroke while the hand remains in contact. All 14 points at
every adjacent arm-band ring and all 36 points of the scalloped paint seam
stay exactly shared through decoded idle, painting and recoil. The left hand
gestures independently. Standard rotation-only source-cue timing and neutral
endpoints are retained. Pack height is 1.1938 m; authoring height remains 1.14.

Source neutral f32 digest:
`41339016b1387248e3407fbba1d1412e7fee742f47744f29bf3f6dff2211a753`.
Packed runtime digest:
`e87a6aa122e9560a34741eb17c164e9ae34f2da00990129878ef18676460b595`.

All 16 Python anatomy/skin checks, two focused Rust rig/GPU regressions and two wider catalog/timing regressions passed. Blender regeneration is byte-identical to the canonical GLB. The production native client built successfully. Front/rear and fixed-camera neutral, idle, painting and recovery views were inspected, followed by a paired normal 3D/2D battle against Hoppip with preserved game state and accepted, visibly rendered terrain. Shiny checks were skipped as requested.

Final normal pair: core 1260/hash 1815607576, LCD 1868, virtual time 32.666663291; `state_preserved=true`.

Final QA prefix: `smeargle-normal` under ignored `target/pokemon-native-review`.
The registry now contains 128 variants. The full model goal remains open,
including the remaining catalog, move-specific choreography, field gaits and
browser review.


### Slugma and Magcargo: grounded mantles, articulated stalks and a rigid shell

`tools/lava_glb.py` replaces both static JSON models with standard skins while
preserving every original neutral f32 position, normal, color and triangle.
Slugma has 12 parts, 2,290 vertices, 4,108 triangles and five joints
(114,632 bytes, 63 accessors). Magcargo has 22 parts, 5,554 vertices,
10,276 triangles and six joints (247,368 bytes, 103 accessors).
Existing loader budgets remain unchanged. Authoring height remains 0.98;
verified pack heights are 0.7112 m and 0.7874 m.

The low mantle keeps its contact height while the raised neck and independent
eye stalks articulate. Eyes and pupils follow each stalk; fitted face details
follow the neck. Slugma's folds follow its mantle. Magcargo's shell, full
spirals, fissures and complete chimney share one rigid owner and rock above
an attached seat. Five posed rays through all model parts retain the recessed
molten floor as the highest center hit, more than 0.085 below the chimney lip.
Standard rotation-only source-cue timing and neutral endpoints are retained.

Neutral f32 digests:

- Slugma: `035d33384e2d2375f1a6ad51fee8bc97efe63d2449179ac3fcc79f83522d8ade`
- Magcargo: `47cde235b8a7adf55ab9c3c7eeaeb86d824ec44959fe3bf46e7764a8b74cbe5b`

All 14 Python anatomy/skin checks, two focused Rust rig/GPU regressions and two
wider catalog/timing regressions passed. Blender regeneration is byte-identical
for both canonical GLBs. The production native client built successfully.
Front/rear and fixed-camera neutral, idle and attack views were inspected,
followed by normal paired 3D/2D battles with preserved game state and visibly
rendered terrain. Both bodies meet the floor. Shiny checks were skipped as
requested.

Final normal pairs (`state_preserved=true`):

- Slugma versus Sentret: core 2536/hash 1639352788, LCD 3164,
  virtual time 56.093004792; QA prefix `slugma-normal`.
- Magcargo versus Pidgey: core 1966/hash 2484868359, LCD 2659,
  virtual time 47.260232457; QA prefix `magcargo-normal`.

Evidence remains under ignored `target/pokemon-native-review`.

The registry now contains 130 variants. The full model goal remains open,
including the remaining catalog, move-specific choreography, field gaits and
browser review.


### Porygon2: a curious gaze, forward nod and flexible leaf tail

`tools/porygon2_glb.py` replaces the static JSON with a six-joint standard skin.
The old static mesh exceeded the 20,000-vertex rig limit. Eye borders and whites
retain their outlines with two interior subdivisions instead of three, and
pupils/glints are refitted. The nine non-eye parts preserve every original
neutral f32 position, normal, color and triangle. The model now has 17 parts,
14,189 vertices, 23,912 triangles, six joints, 609,300 bytes and 95 accessors.
Existing loader budgets remain unchanged; authoring height stays 1.05 and
verified pack height is 0.6096 m.

Both splayed paddles and their fitted reflections stay fixed on the identity
floor root. One spatial field supports the torso and its seated cyan coat and
bends the slender neck into a rigid head. The head carries the eyes, pupils,
glints, crown reflection and spatulate bill. Idle gives a curious gaze, attack
advances the bill with a forward nod, and recoil reverses that gesture.
Two tail joints give the rooted leaf an independent delayed sway. Standard
rotation-only source-cue timing and neutral endpoints are retained.

Neutral f32 digest:
`449d2c9aa5b3d054b2a96818696b9e677a72369d926bfa80e39093976bcb08ac`.
The nine unchanged non-eye parts retain digest
`4d894b9d33428b82d3b02db38aae4014af151a38302fc0e740d558250bdfcd5c`.

All 16 Python anatomy/skin checks passed, along with the final canonical
non-eye geometry pin. Two focused Rust rig/GPU regressions and two wider
catalog/timing regressions passed. Blender regeneration is byte-identical to
the canonical GLB. The production native client built successfully.
Front/rear and fixed-camera neutral, idle, attack and recoil views were
inspected, followed by a normal paired 3D/2D battle against Pidgey. The paddles
meet visibly rendered terrain, the bill and gaze remain readable, and game
state is preserved. Shiny checks were skipped as requested.

Final normal pair: core 3596/hash 3811849301, LCD 4422, virtual time
80.779143917; `state_preserved=true`. Final QA prefix: `porygon2-normal` under
ignored `target/pokemon-native-review`.

The authored registry now contains 131 variants. The full model goal remains
open, including remaining catalog refinement, move-specific choreography,
field gaits and browser review.

## Complete Pokémon locomotion and battle wiring

All 251 species and all 26 Unown forms resolve to an articulated runtime rig.
Authored GLB attack and hit clips retain their cue-relative timing. Older named
source meshes receive a cached Rust skeleton from their anatomical parts. The
adapter preserves the neutral mesh and creates no exported assets or pack copies.
Pidgeotto retains its authored wing hinges and idle beat.

World Pokémon select walking, flight, gliding or floating profiles. Walking
phase measures map travel in model units, excludes camera scrolling, settles at
stops and resets across discontinuities. Its cadence stays readable at fast map
speeds instead of making short legs and prolegs strobe. Complete limbs, shoes, toes, cuffs and
ankle flames follow their supports. Two-bone leg chains keep foot orientation
level and bone lengths fixed. Wing overlays affect limb roots once; descendants
retain their authored articulation. Wingbeat rates distinguish large birds,
small birds, bats and insects. A cached pose review supplies downstroke clearance.
Perched Butterfree NPCs retain their resting-wing adaptation.

Cloyster floats with its shell, spikes, eyes and teeth on one body. Its battle
attack and recoil keep the same floating clearance. Caterpie and Weedle use six
body segments and eight independent prolegs: paired supports follow a travelling
body wave. Their heads, eyes and antennae stay together. Omanyte and Omastar sweep
their tentacles along the ground rather than bending them as vertical legs.

Playback writes local joint transforms in place. Meshes and inverse binds remain
shared; each actor owns its pose and clock. Coincident material and normal splits
share skin weights. Gameplay movement, damage, PP and input ownership remain
with the production controller.

The motion regressions cover the full roster and Unown forms, checking bind
geometry, cue sampling, hierarchy order, fixed leg lengths, level feet, floor
clearance, connected surfaces and locomotion bounds. The separate geometry and
windowless GPU review helpers are explicit, ignored tests; they write only to
the requested scratch directory. Geometry metrics supplement visual inspection.

```sh
CARGO_TARGET_DIR=target/pokemon-motion cargo test --locked \
  --config profile.test.package.crystal-voxel-view.opt-level=1 \
  -p crystal-voxel-view --lib pokemon_motion

CRYSTAL_POKEMON_REVIEW_DIR="$PWD/target/pokemon-motion/review" \
CRYSTAL_POKEMON_REVIEW_SPECIES=CLOYSTER,CATERPIE,WEEDLE,CROBAT,TYROGUE,PONYTA \
CARGO_TARGET_DIR=target/pokemon-motion cargo test --locked \
  --config profile.test.package.crystal-voxel-view.opt-level=1 \
  -p crystal-voxel-view --lib pokemon_motion_ -- --include-ignored --test-threads=1
```

Native location captures can use `CRYSTAL_RENDER_TEST_HEADLESS=1` together with
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`. This runs the production shell with
a virtual window component and a schedule runner, without creating an OS window.
It requires a one-shot screenshot and preserves the existing paired-state checks.
`CRYSTAL_RENDER_TEST_PARTY_SPECIES` selects the disposable location party lead.
The following matched 3D/2D movement capture was executed with the external pack:

```sh
CARGO_TARGET_DIR=target/pokemon-motion cargo build --locked \
  --config profile.dev.package.crystal-voxel-view.opt-level=1 \
  -p crystal-bevy --features location-tester --example render_at_location

CRYSTAL_RENDER_TEST_HEADLESS=1 \
CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_PARTY_SPECIES=CATERPIE \
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=960x720 \
target/pokemon-motion/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map NewBarkTown --x 9 --y 8 --view both --hour 16 --walk RRR \
  --screenshot target/pokemon-motion/review/caterpie-walk.png
```

Native field battles against Pidgey were also inspected as matched 3D/2D
captures for Cloyster and Caterpie. Both retained `state_preserved=true` through
the paired readbacks. Cloyster maintains visible clearance beneath its complete
shell; Caterpie keeps a low horizontal body with its prolegs on the ground.
These native comparisons supplement the full-roster pose and GPU checks.


### Porygon: rigid angular panels, scanning head and grounded paddle swivels

`tools/porygon_glb.py` replaces the static JSON with a seven-joint standard skin.
Every original neutral f32 position, normal, color and triangle is preserved:
16 parts, 4,789 vertices and 2,008 triangles. The GLB uses 206,136 bytes and
94 accessors within unchanged loader budgets. Authoring height stays 1.05;
verified pack height is 0.7874 m.

The rose hull and seated cyan chest share one rigid owner; the angular neck,
scanning head and upright tail articulate independently. All side-panel eye
layers and the bill follow the head. Entire cages remain rigid, preserving
flat facets and exact edge lengths. Each paddle swivels around its own vertical
axis, keeping every vertex at its original contact height. Actual occupied
joins remain attached through idle, forward attack nod and recoil. Standard
rotation-only source-cue timing and literal neutral endpoints are retained.

Neutral f32 digest:
`d2c2a841701ee9ed8d38fdc4c27b87b5e1f5544ec665ebba0372b4f1fba4abb7`.

All 16 Python anatomy/skin checks, two focused Rust rig/GPU regressions and two
wider catalog/timing regressions passed. Blender regeneration is byte-identical
to the canonical GLB. The production native client built successfully.
Front/rear and fixed-camera neutral, idle, attack and recoil views were
inspected, followed by a normal paired 3D/2D battle against Sentret. Porygon's
paddles meet visibly rendered terrain, its crisp silhouette and side eye remain
readable, and game state is preserved. Shiny checks were skipped as requested.

Final normal pair: core 3052/hash 813288004, LCD 3789, virtual time
69.23359; `state_preserved=true`. Final QA prefix: `porygon-normal` under
ignored `target/pokemon-native-review`.

The authored registry now contains 132 variants. The full model goal remains
open, including remaining catalog refinement, move-specific choreography,
field gaits and browser review.


### Chinchou and Lanturn: connected beacons, fin strokes and grounded pelvic paddles

`tools/angler_glb.py` replaces both static JSON models with standard skins.
Chinchou retains every original neutral f32 position, normal, color and
triangle: 21 parts, 12,754 vertices, 11,116 triangles and ten joints
(475,424 bytes, 121 accessors). Lanturn retains all original triangle positions,
order and colors. Its twelve-ring mask gains shared vertices and smooth
crease-aware normals, reducing the model from 24,483 to 17,971 vertices without
changing its shape. All other 28 parts retain their exact original neutral
geometry and normals. Lanturn has 29 parts, 15,572 triangles and twelve joints
(658,740 bytes, 150 accessors). Existing loader budgets remain unchanged.

Coherent pectoral fins stroke with their fitted folds. Two-stage flexible
stalks carry rigid bulb/center pairs; Lanturn's branch stays attached to the
main beacon, and its two tail lobes share a linked distal owner. Body yaw and
independent pelvic swivels retain contact height. Standard rotation-only
source-cue timing and neutral endpoints remain. Authoring height is 0.95;
verified pack heights are 0.508 m and 1.1938 m. The builder now dispatches both
canonical GLBs before Blender batching.

Neutral f32 digests:

- Chinchou: `0924cdb724dca3b1b63d2893ac161eda23672ff9052be7728c7d84b216c52828`
- Lanturn: `50ecc25097080930e9c2f4d69a37ea5388ae765b0c4289cfd4b91e6395944019`

All original triangle positions/colors are separately pinned. Lanturn's
28 unchanged non-mask parts retain digest
`1cc08ef37293e186351c9799d5cb65f2f61c13ecc3361ab201c2dcf89d750263`.
The clearance test selects all 1,472 front mask triangles by actual geometric
winding, keeping smoothed vertical rim normals out of the front-face test.

All 16 Python anatomy/skin checks passed, along with the final unchanged-part
canonical pin. Two focused Rust rig/GPU regressions and two wider catalog/timing
regressions passed. Blender regeneration is byte-identical for both canonical
GLBs. The production native client built successfully. Front/rear and
fixed-camera neutral, idle, attack and recoil views were inspected, followed
by paired normal 3D/2D battles against Pidgey. Both pelvic contact layers meet
visibly rendered terrain; eyes, fin folds, attached beacons and lures remain
readable. Game state is preserved. Shiny checks were skipped as requested.

Final normal pairs (`state_preserved=true`):

- Chinchou: core 1351/hash 3306949276, LCD 1932, virtual time 33.478359749;
  QA prefix `chinchou-normal`.
- Lanturn: core 2219/hash 3830333439, LCD 2843, virtual time 49.772844623;
  QA prefix `lanturn-normal`.

Evidence remains under ignored `target/pokemon-native-review`.

The authored registry now contains 134 variants. The full model goal remains
open, including remaining catalog refinement, move-specific choreography,
field gaits and browser review.


### Mantine: coherent two-tone wing strokes and an attached articulated Remora

`tools/mantine_glb.py` replaces the static JSON with a thirteen-joint standard
skin. Every original triangle position, triangle order and color is retained.
The mantle, two wing borders and five Remora markings gain shared vertices
and crease-aware smooth normals, reducing 40,933 vertices to 19,485. The
other 24 parts preserve every original neutral f32 position, normal, color
and triangle. Mantine has 32 parts, 27,416 triangles, 824,704 bytes and 168
accessors within unchanged loader budgets. Authoring height stays 1.02;
verified pack dimension is 2.1082 m, measured along the authored nose-to-tail
extent using the existing fish rule. This preserves uniform proportions
without inflating a flat ray by its upright fin height. Corsola's canonical JSON and exact recipe
remain unchanged.

Coherent marked wing spans lift in a swimming stroke. Extra flex is confined
to the unmarked distal tips, preserving the narrow zigzags and every shared
blue/pale seam. The body turns without changing belly contact height. Cephalic
paddles articulate independently; both terminal tail layers share a linked
owner. Remora follows the proximal right wing, carries its fitted face and
attachment fin together, and flicks its linked tail. The resting pose is the
lowest wing pose, keeping the attached fish above the floor. Standard
rotation-only source-cue timing and literal neutral endpoints remain.
The builder now dispatches the canonical GLB before Blender batching.

Neutral f32 digest:
`712a8adc15f3cc0fde8aab46912528e87185c6ea8c953ec4283a9f00db1dd7b7`.
Original triangle-position/color digest:
`5e376dff70947ca933869c3ad0a542b0ca33a0d379d048719cf81f29ace4910e`.
The 24 unchanged parts retain digest
`541ff047c2b76639172432964a8ddaa140c2321232185f563fc6842834240053`.

All ten sea anatomy checks and nine Mantine skin checks passed, including the
final unchanged-part/longitudinal-span pin and independent distal-tip check.
Three focused Rust size/rig/GPU regressions and two wider catalog/timing
regressions passed. Blender regeneration is byte-identical to the canonical
GLB. The production native client built successfully. Front, rear and
underside neutral/idle/attack/recoil views were inspected, followed by a normal
paired 3D/2D battle against Sentret. The corrected fish scale produces an
accepted shot on visibly rendered terrain; belly contact, the broad silhouette,
seated zigzags, face and attached Remora are readable. Game state is preserved.
Shiny checks were skipped as requested.

Final normal pair: core 1145/hash 1722243278, LCD 1816, virtual time
32.474173165; `state_preserved=true`. Final QA prefix: `mantine-normal` under
ignored `target/pokemon-native-review`.

The authored registry now contains 135 variants. The full model goal remains
open, including remaining catalog refinement, move-specific choreography,
field gaits and browser review.


### Corsola: coherent complete coral stems, expressive arms and four planted pads

`tools/corsola_glb.py` replaces the static JSON with a nine-joint standard skin.
Every original neutral f32 position, normal, color and triangle is retained:
37 parts, 7,359 vertices and 14,180 triangles (286,056 bytes, 149 accessors).
Existing loader budgets remain unchanged. Authoring height stays 0.93;
verified pack height is 0.6096 m.

Five complete coral stems articulate independently. Each primary stem carries
its rounded tip, thick lateral fork and fork tip as one coherent unit. Both
arms carry their rounded tips through a visible forward/upward gesture. The
crown, white bowl, fitted smile/flecks and eyes/glints share one body owner.
Body yaw retains the four pads fixed on the identity floor root. Idle gives
a gentle branch sway, attack gestures with both arms, and recoil settles back.
Standard rotation-only source-cue timing and literal neutral endpoints remain.
The builder now dispatches the canonical GLB before Blender batching.

Neutral f32 digest:
`70a819e196fadc5fade144ed07e3483aa3a404f9c1a373f572975adb9d9f65d2`.

All 18 Python sea-anatomy/Corsola-skin checks, two focused Rust rig/GPU
regressions and two wider catalog/timing regressions passed. Blender
regeneration is byte-identical to the canonical GLB. The production native
client built successfully. Front/rear and fixed-camera neutral, idle, attack
and recoil views were inspected, followed by a normal paired 3D/2D battle
against Sentret. The pads meet visibly rendered terrain, the crown and complete
forks remain readable, and game state is preserved. Shiny checks were skipped
as requested.

Final normal pair: core 1669/hash 2191196249, LCD 2317, virtual time
41.757402709; `state_preserved=true`. Final QA prefix: `corsola-normal` under
ignored `target/pokemon-native-review`.

Native verification also repaired the concurrent KantoHouse variant's missing
building-only tree-placement match arm; the final production build passed.
The authored registry now contains 136 variants. The full model goal remains
open, including remaining catalog refinement, move-specific choreography,
field gaits and browser review.

## Moltres: planted talons and articulated flame banks

Moltres now uses the canonical GLB skin route. Its 62 recoverable source parts
pack into 46 runtime primitives, 19,671 vertices, 26,024 triangles, 12 joints
and 800,464 bytes. The former static source contained 58,645 vertices and
75,968 triangles. All 26 core anatomy parts retain their original float32
geometry; the remaining 36 retain their projected outlines within 0.00000015
authoring units. Interior retessellation and crease sharing fit the existing
renderer limits without increasing them.

The root fixes every foot, digit and talon. Blended shins bridge those contacts
to a yawing body. Neck and head motion keep gaze and bill coherent. Each wing
has its own leading-arm joint and delayed flame-bank joint; its warm and hot
layers move together. Crown and tail articulate independently. Idle, attack
and recoil clips return exactly to neutral. The verified pack height is
2.0066 metres; authoring height stays 1.28.

All 21 Python anatomy/skin tests, two focused Rust rig/GPU checks and two wider
catalog/timing regressions passed. Blender regeneration is byte-identical.
Front/rear neutral, idle, attack and recoil software views were inspected.
Shiny checks were skipped as requested. Geometry pins and source-preservation
details are in `moltres-sculpture.md`.

The fresh production client built successfully with the shared image feature's
required `bevy/zstd_rust` backend. Its normal paired 3D/2D encounter against
Sentret was inspected: the Moltres name is explicit, gold leading wings and
layered flames read clearly, and both grey feet meet visible terrain. The
camera accepted the shot and preserved state: core frame 2629/hash 275580255,
LCD animation frame 3377, virtual time 60.621271291. Final ignored QA prefix is
`moltres-normal`. The authored registry contains 137 variants. The full model
goal remains open, including remaining catalog refinement, move-specific
choreography, field gaits and browser review.

## Zapdos: coherent thunder fans and fixed floor contact

Zapdos now uses the canonical GLB skin path: 54 parts, 18,495 vertices,
28,584 triangles, nine joints and 729,848 bytes. Reducing only the excess
interior pupil tessellation brings the former 20,739-vertex source within
the existing renderer limits. All 52 non-pupil parts preserve their exact
original float32 geometry, normals, colors and triangle order. The canonical
geometry pin is
`75512d822cf34c6097ea7c8421a159f7bad0291a985a4ae20ab5ae2642ba041f`.

Every foot, digit and talon stays on the identity floor root. Supported shins
and thigh feather spats bridge to the yawing body. Head and neck motion keep
the fitted gaze, throat, closed lance bill, crown and cheek rays coherent.
Each complete gold fan, its three black underfeathers and fitted shoulder
covert share one wing carrier. Both lightning tail fans share a separate yaw
joint. Idle, attack and recoil clips return exactly to neutral. The verified
pack height is 1.6002 metres; authoring height remains 1.28.

All eleven anatomy tests, eight decoded-skin tests, two focused Rust rig/GPU
checks and two wider catalog/timing regressions passed. Blender regeneration
is byte-identical. Fixed-camera front and rear neutral, idle, attack and
recoil views were inspected. The fresh native client built successfully.
Shiny checks were skipped as requested.

Normal paired 3D/2D inspection against Pidgey shows the explicit Zapdos name,
readable gold-and-black lightning silhouette and planted talons on visible
terrain. The accepted camera preserves game state: core frame 3263/hash
2569537076, LCD animation frame 3994, virtual time 73.532027458. Final ignored
QA prefix is `zapdos-normal`. The authored registry now contains 138 variants.
The full goal remains open, including the remaining catalog, move-specific
choreography, field gaits and browser review.

## Articuno: coherent feather banks and an articulated streamer

Articuno now uses the canonical GLB skin path. Its 109 recoverable source parts
pack into 49 runtime primitives, 19,074 vertices, 26,704 triangles, 11 joints
and 777,424 bytes. The former static source had 43,145 vertices and 63,816
triangles. Reduced cap tessellation, crease sharing and channel path sampling
fit the unchanged renderer limits. All 32 core anatomy parts, including the
entire main tail ribbon, retain their original float32 geometry. The other
74 feather and fitted-marking outlines retain their projected boundaries
within 0.000000112 authoring units. The three channels remain fitted to the
actual ribbon surface.

Feet, digits and talons stay on the identity floor root. Supported shins bridge
to the yawing body. Bib, throat, gaze, bill and crown use coherent body, neck
and head carriers. Each complete layered wing has one carrier. Three tail
joints use smooth arc-length weights shared by the ribbon and its channels;
yaw preserves their floor clearance while the curl articulates. Idle, attack
and recoil clips return exactly to neutral. The local pack's verified height
is 1.7018 metres; authoring height remains 1.28.

Eleven anatomy tests, nine decoded-skin tests and two focused Rust rig/GPU
checks passed. These include posed occupied joins, supported fitted layers,
windings and strain, channel-to-ribbon distance, ribbon self-intersection,
fixed contacts, tail clearance and culling bounds. The relocated shared
point-to-triangle helper passed two existing dragon-ribbon checks. Blender
regeneration is byte-identical. Front/rear neutral, idle, attack and recoil
software views were inspected. The production native client built
successfully. Shiny checks were skipped as requested.

The two wider catalog/timing regressions passed. Normal paired 3D/2D inspection
against Sentret shows the explicit Articuno label, readable layered wings and
pale bib, the full articulated curl with clear channels, and planted grey feet
on visible terrain. The accepted camera preserves state: core frame 3950/hash
123451209, LCD animation frame 4754, virtual time 88.511306625. Final ignored
QA prefix is `articuno-normal`. The authored registry now contains 139 variants.

The same capture still shows apparent idle tail clearance on Sentret; review
that support contact separately. The full goal remains open, including the
remaining catalog, move-specific choreography, field gaits and browser review.

## Ground contact shading: Sentret support follow-up

The apparent Sentret idle hover flagged in the Articuno review came from a
contact shadow centered on the complete silhouette, about 0.25 authoring units
from its supporting curl. The decoded rig and production skin pose keep the
actual tail support on the floor. A new renderer regression reproduced the
shadow displacement before the fix; geometry and animation remain unchanged.

Grounded actors now cache floor-connected geometry within 6% of model height.
The seed tolerance is 0.00001 of height. Connectivity stays below that ceiling,
and crossing edges are clipped exactly, excluding low suspended plumes that
connect through an elevated torso. The registered footprint supplies both
the soft ground patch and retained-grass contact shading. Floating bodies
retain their whole-body footprint. Full animated bounds, physical scale,
source offsets and grass height remain unchanged.

Five focused checks passed for Sentret's rendered floor and tail patch,
Articuno's feet, floating-body behavior, visibility, ground height, disconnected
low geometry and tessellation invariance. Three follow-up regressions passed
for pinned tail support, elapsed animation timing and adjacent earth grain.
The shared earth-grain fixture's UV arguments were corrected to its existing
tuple signature. Its test passed. The opt-in CPU benchmark stayed ignored.

The production native client built successfully. An initial point-sized patch
was visually rejected as too small; the final connected footprint produces
readable contact under both Sentret tail bases in complementary orientations.
The normal 3D/2D pair is accepted on visible terrain and preserves state:
core frame 2488/hash 2189612142, LCD animation frame 3144, virtual time
56.2137815. Final ignored QA prefix is `sentret-support-final`, with the
original baseline under `sentret-support-before`. Shiny checks were skipped.
The full model goal remains open, including the remaining catalog,
move-specific choreography, field gaits and browser review.

## Lugia: five-digit palms and an articulated open roar

Lugia now uses the canonical GLB skin path. Its 56 recoverable source parts
pack into 34 runtime primitives, 19,823 vertices, 34,796 triangles, 13 joints
and 852,748 bytes. The former static source had 34,165 vertices and 61,688
triangles. Selective paddle and ellipsoid sampling and conforming local belly
refinement fit the unchanged renderer limits. The 25 retained core meshes
preserve topology, normals and colors; their normalized position drift is
below 0.000000375 units from the resampled rounded pads' floor datum. The
highest fingers retain their original sampling.

Both pads and all six toes stay on the identity floor root. Supported haunches
and ankles bridge to the yawing torso. The fitted belly and dorsal plates move
with the body. Neck and complete gaze have separate carriers. Wing arms blend
toward independent palm joints, each carrying a complete five-digit fan.
The lower mandible and tongue share an opening joint; the actual mouth void
stays clear. Two yawing tail joints carry the paired navy fins and preserve
height. Idle, attack and recoil clips return exactly to neutral. Authoring
height remains 1.45; the verified pack height is 5.207 metres.

Thirteen anatomy tests, nine decoded-skin tests and three focused Rust
routing/rig/GPU checks passed. The mouth-volume oracle now merges duplicate
shared-edge crossings before pairing occupied intervals, so the float32
ellipsoid filler negative control cannot disappear on its center seam. The
strengthened posed-jaw check was rerun successfully. Blender regeneration
is byte-identical. Fixed-camera front/rear neutral, idle, attack and recoil
software views were inspected. Shiny checks were skipped as requested.

Native acceptance remains open. Route 35 and Route 38 also rejected the
complete animated footprint on the source ground (zero supported staging
candidates). A first Route 38 movement-only run stopped at battle intro text;
the subsequent ordinary A-input run reached Lugia's battle and confirmed the
terrain rejection. No size clamp, source-floor change, footprint relaxation
or grass-height change was introduced. The latest native build and the
Route 35 fixture regression passed. No paired native acceptance is claimed.


## Lapras seated battle contact correction (2026-10-15)

A fresh production Route 29 walking encounter reproduced Lapras hovering
above the source grass. The accepted native 3D/2D baseline preserved state
at core frame 3182, checksum 673262487, LCD animation frame 3994 and virtual
time 80.012000502. The renderer's generic floating-battler adjustment added
0.24 authoring units (16% of its height), despite the canonical skin's fixed
belly and shell. This is separate from grass height and source geometry.

`pokemon_motion::battle_hovers` now distinguishes Lapras's seated battle
presentation from its swimming locomotion. Battle sampling no longer adds
the artificial root lift to Lapras. Ground contact uses the existing cached
floor-connected belly region, excluding the extended flipper tips. Swimming
locomotion, authored geometry, weights, clips, pack dimensions and palette
remain unchanged. Actual hovering species retain their existing clearance.

An independent stored-vertex regression first failed at idle time zero with
floor 0.24. After correction it passes for 33 samples of each idle, attack
and recoil clip. The same regression checks unchanged Cloyster and Porygon
clearance. Four focused Rust checks pass for seated vertices, the production
contact-shadow cache, dedicated Lapras registry and GPU activation at pack
size. The existing complete-shell Cloyster locomotion regression also passes.
Shiny verification remains skipped at the user's request.


The rebuilt native Route 29 pair `lapras-grounded-3d.png` /
`lapras-grounded-2d.png` passes terrain admission (`accepted=true`) on the
original ground. The receipt preserves state at core frame 1453, checksum
1571462043, LCD animation frame 2091, virtual time 36.800318958. The pair was
visually inspected: the tan belly meets the field, the contact shadow sits
under the seated body, and the blue neck, face, shell spikes, curled ears and
four flippers remain readable at unchanged pack size. No shader errors were
reported. This run encountered Pidgey; the earlier baseline encountered Rattata,
so each native view is paired with its own identical-state 2D view rather than
claimed as a frame-identical cross-run comparison.

The catalogue-wide elapsed-idle/source-attack timing regression also passed.
This closes the added battle-hover/contact defect only. Lapras's broader
shape and detailed face/flipper refinement, legacy world props and browser
visual acceptance remain open. No grass-height or shiny changes were made.


## Lapras fitted face refinement (2026-10-15)

The round staring eyes were replaced with rising almond outlines, smaller
upright pupils and fitted highlights. Brows now follow that gaze. Two small
nostril solids and a shallow curved mouth crease complete the muzzle. The
crease follows the actual stored head triangles; it does not span the face
as a floating straight chord. Its back is shallowly embedded while the front
stays visible. The almond caps needed conforming subdivision to avoid cutting
through the strongly curved outer eye corners; the first coarse candidate
failed an independent stored-triangle clearance check and was replaced.

All gaze, brow, nostril and mouth geometry follows the existing head joint.
The other 34 source parts preserve every position, normal, triangle, color and
skin influence exactly. Their geometry SHA-256 is
`04b4c4324b15652e0a1c9d26b0299374318a80bbf1e02330e50d0cfd77070cba`.
A before/after binary audit also proves identical hierarchy and all animation
key times and rotation values. The seated battle contact correction remains.
Height is still 1.50 authoring units and pack size remains 2.4892 metres.

The canonical GLB now has 45 parts, 19,471 vertices, 23,332 triangles, nine
joints, 253 accessors and 843,872 bytes. Its neutral/source pin is
`e783b1adb22c94e3a0914dde725c5eef44c4197fc93e4736edc49e24e455548e`.
All allocation limits stay unchanged. The common Blender builder previously
silently omitted Lapras; its dedicated registration and canonical export path
now regenerate the byte-identical GLB through the real builder.

Nine stored-model Python checks and four focused Rust checks passed, including
closed outward solids, smooth normals, eye/pupil/highlight attachment at
triangle centers and edge midpoints, visible fitted mouth clearance, retained
source geometry, rigid head ownership, animation winding/strain, seated belly,
contact shading and GPU activation at pack size. The complete exact-species
grounding/volume regression passed. Fixed-camera before/after front and oblique
face views, plus idle, attack and recoil views, were inspected. Shiny checks
remain skipped as requested.


The rebuilt production Route 29 pair `lapras-face-final-3d.png` /
`lapras-face-final-2d.png` was inspected at core frame 2312, checksum
1402440659, LCD animation frame 2990 and virtual time 54.018008459.
The receipt confirms `state_preserved=true`; terrain admission is
`accepted=true`. Against Sentret, the narrow white eye, fitted pupil, small
nostril and curved mouth crease remain visible, while the shell, flippers,
blue/tan palette and seated belly stay readable. The original terrain is
retained and no shader errors were reported. Native acceptance here covers
this face refinement and retained contact, not complete model or browser
fidelity. At that stage, broader flipper/shell refinement and legacy world
props remained open; the later flipper/shell work is recorded below.


## Canonical phoenix skin (2026-10-15)

The canonical source is now `models/battle_species/ho_oh.glb`, authored by
`tools/ho_oh_sculpt.py` and `tools/ho_oh_glb.py`. The obsolete static JSON was
removed. All 207 source parts remain recoverable in order from 38 runtime
primitives. Same-carrier red flight vanes, ivory bands, green tips and warm
coverts pack into separate coherent banks. Gold tail vanes and their pale
terminals, crest petals, fixed digits and talons also pack by carrier/color.
No third-party geometry is used.

The model has 19,747 vertices and 25,200 triangles, down from 81,715 and
118,516. Outlines retain their authored curve sampling while interior feather
subdivision is reduced. Head, crest, mouth, haunch, shin, digit and talon
sampling is reduced selectively. The ivory belly uses conforming local cap
refinement at its original thickness: a coarse candidate clipped the breast
and was rejected. The shared helper was extracted from Lugia without changing
Lugia's generated GLB bytes. Body, neck, both beaks and both feet preserve all
six original source parts exactly after float32 conversion: positions,
normals, triangles and colors. Their retained geometry SHA-256 is
`ff91b55266642639ceff0b02e907dd38191e49dcc3724b9060c18a9e0dc4d529`.

The ten-joint skin fixes feet, eight gripping digits and all ivory talons on
the identity floor root. Supported shins and haunches bridge to the yawing
breast. Neck and gaze articulate independently. Each complete wing bank moves
with its own shoulder, preserving both painted faces. The lower beak has an
opening joint, and the entire golden fan has a separate yawing tail carrier.
Idle, attack and recoil contain rotation keys only and return literally to
neutral. Authoring height remains 1.45. The verified local pack supplies
height digits 1206 (12 feet 6 inches), or 3.81 metres, classification RAINBOW.

The source geometry pin is
`67e4806121613127491e740385e7974e9afdb7b32f52dbe8722674ffa277214b`;
the packed runtime pin is
`4e246f222c484ed35240a4bd22ce63b024b482e85b707d9968dbea9da3fac218`.
The skin uses 179 accessors within unchanged renderer limits. The common
Blender builder exports the same canonical GLB. Neutral, idle, attack and
recoil front/rear software views were inspected against the original static
model. No detached paint or visible loss of the tricolor silhouette was seen.

Thirteen stored anatomy checks, seven decoded-skin checks and three focused
Rust routing/rig/GPU checks passed. They cover closed solids, normals,
self-intersections, actual fitted-layer clearance, occupied roots, a real
open jaw with a filler negative control, coherent carriers, fixed floor
contacts, tail height, posed winding/strain, neutral endpoints and culling
bounds. The jaw-volume oracle merges duplicate shared-edge crossings before
pairing occupied intervals. The wider grounding/volume and source-cue timing
regressions passed. Shiny verification remains skipped as requested.


The final canonical GLB is 800,780 bytes. Blender regeneration is byte-identical
and the production native executable built successfully. Current full-scale
walking-encounter staging remains unresolved: Route 29 found 78 supported
ground candidates but rejected every camera composition; Route 38 found no
supported complete-body candidates. A wider 1400 by 700 Route 29 viewport
still rejected the camera. These are actual native rejections, not accepted
visual captures. No full-size native 3D/2D acceptance is claimed for this skin.
Source dimensions, animated footprint, terrain admission and grass height were
kept intact. The previously recorded static-era arena capture is historical evidence
for the earlier model only. Large-bird native staging and browser acceptance
remain open.


## Large-body far-plane correction (2026-10-15)

The authentic-scene camera check compared its perspective far plane with
radial distance to the furthest animated corner. Perspective clipping and the
retained-neighborhood ray limits instead measure view-axis depth. This could
reject a wide wing that was completely inside both the frustum and original
terrain. The far plane, backdrop depth margins and omission/shadow queries now
use the maximum camera-space depth of every animated corner.

An independent regression first reproduced the false rejection with a wide
body. It now projects all corners through an explicit perspective matrix,
checks clip-space bounds and rejects both a genuinely clipped wing and
insufficient original terrain. Body poses, scale and full ground-footprint
checks remain intact. Bevy's installed 0.20 fog shader uses radial distance,
so fog keeps that separate metric, starts beyond the complete battler,
and finishes inside the proven terrain range. A separate guard rejects
views that cannot meet both requirements.

All 47 encounter-terrain regressions pass, including actual ground cells,
source offsets, surf support, source omission/shadow bounds and finite camera
fallbacks. Native large-bird acceptance is recorded separately below.


When the complete low-angle stage search fails, the camera now tries 60-,
68- and 75-degree views through the identical admission checks. All supported
stages at the ordinary 6–48-degree elevations take precedence over this second
pass. Each pass uses a layout copy and publishes only an accepted result.
An independent higher-view witness proves the fallback fits its complete
frustum, keeps original scale and still rejects missing wing ground. Enlarging
that fixture's genuine terrain restores the exact preferred low view.
All 48 terrain tests and three battle-view integration checks pass.


The final production build passed. A native Route 29 Articuno/Pidgey pair was
visually inspected at core frame 2184, checksum 37399760, LCD animation frame
2869 and virtual time 50.084423956. `state_preserved=true` and terrain admission
is `accepted=true`. The ordinary low-view pass succeeds: both full wing banks,
face, white breast, planted feet and complete blue streamer remain visible at
pack size. Original grass, ledge and forest remain readable behind the pair;
no exposed missing terrain or shader errors were seen. The matching 2D view
uses that same held game state. Verification files use prefix
`target/pokemon-native-review/articuno-camera-final`.

Ho-Oh still fails Route 29 admission after both camera tiers: each tier checked
78 supported staging candidates, but the complete view could not satisfy the
retained-neighborhood and scenery checks. No native Ho-Oh acceptance is claimed.
The wider and higher camera support is proved by the independent fixtures;
large-bird production staging remains open. Model geometry, physical size,
original source ground and grass height were not changed. Shiny verification
was skipped as requested.


## Celebi original onion fairy and skin (2026-10-15)

Celebi's generic 22-part sphere/cone/diamond-wing placeholder was replaced by
an original continuous swept onion bulb, fitted dark/blue gaze, three subtle
folds, blue-tipped feelers, a little smile, pale chest collar, three-finger
hands, rounded wing leaves and green drop-shaped feet. The first sculpt's
point and old boot shape were corrected during review. A crowded antenna
interval produced a proper face intersection; removing its redundant control
point resolved the loop while retaining occupied blue-tip roots.

The canonical GLB has 38 closed parts, 14,678 vertices, 23,056 triangles,
11 joints, 181 accessors and 616,320 bytes. Its source/neutral SHA-256 is
`94a8a577208273651ba7294a48345f701b30142a260b5fc2e920b21e252837fb`.
Authoring height remains 0.98 with floor zero. The verified local pack gives
height digits 200 (0.6096 metres), classification TIMETRAVEL. Foot roots stay
fixed, shanks blend to the torso, the complete fitted gaze/folds share the head,
each hand's three fingers share its arm, and each feeler's blue tip shares its
antenna joint. Wings flutter independently through six cycles per 3.2-second
idle. Rotation-only idle, attack and recoil return literally to neutral and
source cues own timing. No shiny variant was generated or verified.

Ten stored-model Python checks, two focused Rust checks and the two wider
regressions pass. They cover closed solids, normals, proper intersections,
actual fitted-parent triangle/edge clearance, occupied roots and a detached-tip
negative control, correct owners, ground contact, posed winding/strain,
independent motion, culling bounds, registry routing and GPU activation at
verified pack size. Blender regeneration is byte-identical. Front/rear neutral,
peak flutter, attack and recoil software views were inspected against the old
placeholder. Details and reference are in `celebi-sculpture.md`.


The wider exact-species grounding/volume and elapsed-idle/source-attack timing
regressions passed. The rebuilt production native Route 29 encounter against
Hoppip passes terrain admission (`accepted=true`). Its inspected 3D/2D pair
preserves state at core frame 1525, checksum 2305946238, LCD animation frame
2116 and virtual time 36.53826475. The swept head, dark/blue fitted gaze,
blue feeler tips, small hands and pale wings remain readable beside the source
grass at verified pack size. Rounded feet meet the field; the original forest
and grass remain in place. No shader errors were reported. This verifies the
held native view; browser visual acceptance remains open. Verification files
use prefix `target/pokemon-native-review/celebi-normal` and remain ignored.


## Mew original psychic cat and looping-tail skin (2026-10-16)

Mew's old generic 27-part upright torso/button snout/block ears were replaced
by an original compact pink pear body, short throat, soft feline cheeks/chin,
fitted tapered gaze, soft ears with fitted interiors, a tiny smile, little
three-finger hands, rounded haunches and long tapered paws with fitted toe
creases. A continuous looping whip ends in an oval pink terminal. Rounded
Bezier eye outlines and denser tail centerline sampling replaced the first
angular candidate; the initial slipper paws were replaced by downward tapers.
Mewtwo's subsequent dedicated replacement is documented below.

The canonical GLB has 40 closed parts, 18,486 vertices, 30,224 triangles,
13 joints, 190 accessors and 758,636 bytes. Its neutral/source SHA-256 is
`36d3fc53223d21ac492d3d93879589a130773ba3d884c34a65f6173b3980d37e`.
Authoring height is 0.78 with minimum zero; ears establish standing height
rather than the looping tail. The verified local pack gives height digits
104 (0.4064 metres), classification `NEW SPECIE`. Torso, neck, head, ears,
hands and hind limbs respond separately. Three yawing tail joints blend by
nearest-centerline arc length, and the oval tip shares the fully weighted
distal carrier. Rotation-only source-cue clips preserve tail height and return
literally to neutral. Existing render-only hovering clearance is unchanged.

Ten stored-model checks, three focused Rust checks and the two wider
regressions pass: canonical/schema limits, closed solids, normals, actual
fitted-layer clearance, occupied roots and a detached-tip negative control,
correct owners, three tail carriers, posed floor/winding/strain, independent
motion, neutral endpoints, routing, culling bounds and GPU activation at pack
size. An actual posed-vertex check confirms Mew's conservative fitting envelope
already contains its runtime hover lift; no renderer or physical-size change
was necessary. Blender regeneration is byte-identical. Front/rear neutral,
idle, attack and recoil software views were inspected against the old
placeholder. Shiny checks remain skipped. Details: `mew-sculpture.md`.


The wider exact-species grounding/volume and elapsed-idle/source-attack timing
regressions pass. The rebuilt production Route 29 encounter against Sentret
passes terrain admission (`accepted=true`). Its inspected 3D/2D pair preserves
state at core frame 1148, checksum 1008858066, LCD animation frame 1686 and
virtual time 29.109839834. The feline face, soft ears, small hands, long paws,
looping whip and oval tail tip fit fully in view. The standard hover clearance
remains visible at verified pack size. Original grass and forest remain in
place; no shader errors were reported. This verifies the held native view;
browser visual acceptance remains open. Verification files use prefix
`target/pokemon-native-review/mew-normal` and remain ignored.

## Mewtwo genetic psychic cat

Mewtwo now uses a dedicated pale genetic body, purple pelvis and heavy curved
tail, narrow fitted violet gaze, blunt cranial projections, back-neck arch,
three-finger hands with rounded contact tips, powerful haunches and planted
three-pad feet. The old all-purple generic static cat is removed. Its standard
glTF skin has 52 parts, 15,722 vertices, 27,880 triangles, 17 joints and 227
accessors in 633,436 bytes. Neutral/source digest:
`ef903cf6d3fbd2c2b7ed2d8777e1d91726c56940e30f77aaa767e5e50eceb521`.
Authoring height is 1.45 with floor zero; verified pack height digits 607
provide 2.0066 metres, classification `GENETIC`. Identity-root foot pads stay
planted while blended hocks, hips, torso, neck, shoulders, elbows, wrists and
three arc-weighted tail carriers articulate at source-cue time.

Eight stored-model checks, two focused Rust checks and the two wider
exact-species grounding/volume and elapsed-idle/source-attack regressions pass.
Blender regeneration is byte-identical. Front and rear neutral, idle, attack
and recoil views were inspected. Shiny verification remains skipped. Details:
`mewtwo-sculpture.md`.

The rebuilt production Route 29 encounter against Pidgey passes authentic
terrain admission (`accepted=true`). The inspected 3D/2D pair preserves state
at core frame 1332, checksum 1546947256, LCD animation frame 1943 and virtual
time 34.104813792. The whole body fits at original pack scale, foot pads rest
on source ground, and both species are named in the HUD. Original grass and
forest remain in place; no shader errors were reported. Held native acceptance
is complete; browser visual acceptance remains open. Final ignored evidence
uses prefix `target/pokemon-native-review/mewtwo-normal`.

## Heracross single-horn beetle

Heracross now has a dedicated blue armored abdomen, fitted belly plate and
seams, broad thorax collar, golden fitted gaze, scalloped mask seam,
continuous flattened forked horn, bead antennae, serrated gauntlets and pale
hooked claws. The generic round-eyed, mitten-handed static beetle is removed.
Its standard glTF skin has 47 parts, 19,534 vertices, 32,148 triangles,
12 joints and 223 accessors in 814,644 bytes. Neutral/source digest:
`a33ddd1481ba4cb3435132daf36f3c5770e6b8155e24afc627142a32a82d923d`.
Authoring height is 1.16 with floor zero; verified pack height digits 411
provide 1.4986 metres, classification `SINGLEHORN`. Tarsi and foot claws stay
on the identity root. Coxae and shins blend into body/leg carriers. Body,
thorax, head, antennae, upper arms and forearms articulate independently;
each gauntlet, its serrations, knuckle and two claws remain coherent.
Attack lowers the head and horn at source-cue time.

Nine stored-model checks and two focused Rust checks pass for canonical/schema
limits, closed solids, normals, fitted triangle/edge clearance, occupied roots
with a detached-bead negative control, coherent owners, planted claws, posed
floor/winding/strain, independent motion, neutral endpoints, source cues,
registry routing, culling bounds and GPU activation at verified pack size.
Blender regeneration is byte-identical. Front and rear neutral, idle, attack
and recoil software views were inspected against the old static model. Shiny
verification remains skipped. Details: `heracross-sculpture.md`.

The two wider grounding/volume and elapsed-idle/source-attack timing regressions
pass. The rebuilt production Route 29 encounter against Pidgey passes terrain
admission (`accepted=true`). The inspected 3D/2D pair preserves state at core
frame 3014, checksum 2825358273, LCD animation frame 3737 and virtual time
72.298003834. The complete horn, beads and claws fit at original pack scale;
foot pads rest on source ground and the HUD names both species. Original grass
and forest remain in place; no shader errors were reported. Held native
acceptance is complete; browser visual acceptance remains open. Final ignored
evidence uses prefix `target/pokemon-native-review/heracross-normal`.

## Pinsir stag beetle

Pinsir now has one continuous oval brown carapace, stern fitted ivory eyes,
a dark upright mouth with four paired tooth bars, three closed transverse
armor seams, huge curved pale pincers with five thorns each, thin raised arms
and three-claw hands and feet. The generic round-eyed static beetle is removed.
Its standard glTF skin has 53 parts, 14,082 vertices, 23,756 triangles,
13 joints and 199 accessors in 565,000 bytes. Neutral/source digest:
`347418889561cb58a6f9b3f05d441ca8f6bbe2a7b8fb6c4df81ec1a24ea01d84`.
Authoring height is 1.16 with floor zero; verified pack height digits 411
provide 1.4986 metres, classification `STAGBEETLE`. Tarsi and foot claws stay
on the identity root. Shanks and the oval shell blend through leg, abdomen
and upper carapace carriers. Each pincer owns its thorns, each hand owns its
claws, and shoulders/elbows/wrists articulate separately. Attack closes the
pincers at source-cue time.

Nine stored-model checks and two focused Rust checks pass for canonical/schema
limits, closed outward solids, normals, fitted triangle/edge clearance,
occupied roots with a detached-spike negative control, coherent owners,
planted feet, posed floor/winding/strain, self-intersections, independent
motion, neutral endpoints, source cues, registry routing, culling bounds and
GPU activation at verified pack size. Blender regeneration is byte-identical.
Front and rear neutral, idle, attack and recoil views were inspected against
the old static model. A folded thumb curve was corrected before acceptance.
Shiny verification remains skipped. Details: `pinsir-sculpture.md`.

The two wider grounding/volume and elapsed-idle/source-attack timing regressions
pass. The rebuilt production Route 29 encounter against Sentret passes terrain
admission (`accepted=true`). The inspected 3D/2D pair preserves state at core
frame 2454, checksum 1836009259, LCD animation frame 3109 and virtual time
54.840088499. Complete pincers, thorns and raised hands fit at original pack
scale, foot pads rest on source ground, and the HUD names both species.
Original grass and forest remain in place; no shader errors were reported.
Held native acceptance is complete; browser visual acceptance remains open.
Final ignored evidence uses prefix `target/pokemon-native-review/pinsir-normal`.

## Scizor red steel mantis

Scizor now has an angular three-pointed head and golden gaze, a narrow red
thorax and waist, hanging abdomen with fitted dark armor, thin articulated
arms, genuinely open pincers with independent lower jaws and coherent false
eyes, four pale veined wing leaves, long red greaves and grounded pointed toe
caps. The old generic round-headed static insect with painted claw splits is
removed. Its 55 source parts remain exactly recoverable from 43 runtime parts,
with 18,558 vertices, 27,348 triangles, 17 joints and 239 accessors in
813,656 bytes. Source digest:
`9b03e41cd8685a3ec5521fbc993abcdc872bd931a17cfdfb4fb1fdb1982bf7f3`.
Packed neutral digest:
`b460947f4351d6f0f28a3d92168aab50ab81ca53215d603ad2f41aa1893d64a8`.
Authoring height is 1.28 with floor zero; verified pack height digits 511
provide 1.8034 metres, classification `SCISSORS`. Fixed greaves/pads/toe caps
support blended femurs and hip armor. Thorax, abdomen, neck, head, shoulders,
elbows, wrist pincers, lower jaws and paired wing carriers articulate at
source-cue time. Actual stored jaw-tip separation decreases during attack
while retaining clearance.

Eleven stored-model checks and two focused Rust checks pass for source recovery
and schema limits, closed solids, normals, fitted triangle/edge clearance,
actual empty jaw apertures and a filled-gap negative control, occupied roots
and a detached-jaw control, coherent owners, planted armor, posed floor,
winding/strain, self-intersections, independent motion, jaw closure, neutral
endpoints, source cues, registry routing, culling bounds and GPU activation
at verified pack size. Blender regeneration is byte-identical. Front and rear
neutral, idle, attack and recoil views were inspected against the old generic
static model. Shiny verification remains skipped. Details: `scizor-sculpture.md`.

The two wider exact-species grounding/volume and elapsed-idle/source-attack
regressions pass. The rebuilt production Route 29 encounter against Pidgey
passes terrain admission (`accepted=true`, 867 candidate stages checked).
The inspected 3D/2D pair preserves state at core frame 6686, checksum 135714849,
LCD animation frame 7633 and virtual time 169.180527293. Complete pincers and
cranial points fit at original pack scale; open jaws and false eyes read
clearly, foot pads rest on source ground, and the HUD names both species.
Original grass and forest remain in place; no shader errors were reported.
Held native acceptance is complete; browser visual acceptance remains open.
Final ignored evidence uses prefix `target/pokemon-native-review/scizor-normal`.

## Scyther green mantis

Scyther now has a rounded green thorax and pale breast shield, cream waist,
segmented tapered abdomen, angular ivory gaze and muzzle, swept cranial points,
curved ivory scythes with fitted green spines, four pale membranes visible on
both wing faces, pointed hip armor, pale knee stalks and grounded three-claw
feet. The generic round-headed static insect is removed. Its 57 source parts
remain exactly recoverable from 36 runtime parts, with 19,533 vertices,
30,360 triangles, 13 joints and 191 accessors in 811,488 bytes. Source digest:
`ea4538d82e02fd1c896ad210cfe655355ddd53b56953e5ecdff59f86131452ea`.
Packed neutral digest:
`d27719b98ff97185509c5375d650cfb8bfa4a25db49e9fe13115972b00903d3d`.
Authoring height is 1.18 with floor zero; verified pack height digits 411 provide
1.4986 metres, classification `MANTIS`. Fixed greaves/pads/nails support blended
pale shanks and independent armored coxae. Thorax, abdomen, neck, head, elbows,
blades and paired wing carriers articulate at source-cue time. The actual
blade tips sweep independently of the upper arms during attack.

Thirteen stored-model checks pass for source recovery/schema limits, closed
solids, normals, fitted front and rear face clearance, blade cutouts, occupied
roots and detached-blade/vein negative controls, coherent owners, planted feet,
posed floor/winding/strain, self-intersections, independent motion, blade-tip
sweep, neutral endpoints and source cues. The thin ribs have actual embedded
vertices within their membranes, preserving the delicate ridge geometry.
Blender regeneration is byte-identical. Front/rear neutral, idle, attack and
recoil views were inspected against the old generic model. Elbow stalks were
extended into blade roots, hip motion made independently visible, and pale
membranes carried through to the rear wing faces before acceptance. Shiny
verification remains skipped. Details: `scyther-sculpture.md`.

Two focused Rust checks pass for exact registry routing, production geometry,
coherent gaze/blade-spine/wing/armor ownership, independently visible head,
blade, wing and coxa motion, planted armor, culling bounds and GPU activation
at verified pack size.

The two wider exact-species grounding/volume and elapsed-idle/source-attack
regressions pass. The production Route 29 encounter against Rattata passes
terrain admission (`accepted=true`, 854 candidate stages checked). The inspected
3D/2D pair preserves state at core frame 4531, checksum 1806325052, LCD animation
frame 5374 and virtual time 113.673177877. Complete curved blades, crest and
wing silhouette fit at original pack scale; elbow stalks meet their blades,
clawed feet rest on source ground, and the HUD names both species. Original
grass and forest remain in place; no shader errors were reported. Held native
acceptance is complete; browser visual acceptance remains open. Final ignored
evidence uses prefix `target/pokemon-native-review/scyther-normal`.

### Starmie: dual purple stars and articulated radial arms

The generic four-part Starmie is replaced by two continuous offset purple
five-arm stars, ten seated ridges, an eight-spoke gold mount and a faceted
rose gemstone. The 24-joint rig supplies separate layer carriers and ten
two-joint radial chains while the entire core mount remains coherent.
The protected lowest rear tip preserves the authoring floor; existing Float
presentation supplies the intended hover. The model uses 23 parts, 2,966
vertices, 3,344 triangles and 215,488 bytes at authoring height 1.03.
The verified pack height is 307 (1.0922 metres), classification MYSTERIOUS.

Eight Python checks, two focused Rust checks, both shared model/timing
regressions and byte-identical Blender generation pass. Front and rear
idle/attack/hit views were inspected. The production Route 29 encounter
against Pidgey passes terrain admission; the inspected native 3D/2D pair
preserves state at core frame 2391, checksum 2002455108, LCD animation frame
3021 and virtual time 54.047793501. Complete star layers and the raised core
remain readable at original size, with a ground shadow under the intended
hover and correct species names in the HUD. No shader errors were reported.
Shiny verification was skipped at the user's request; browser visual
acceptance remains open. See `docs/art/starmie-sculpture.md`; final ignored
evidence uses prefix `target/pokemon-native-review/starmie-normal`.

### Gastly: dark ghost face and a drifting vapor cloak

The generic ball, red gaze and tooth-bar grin are replaced by a continuous
dark spherical orb, broad fitted white eyes with tiny black vertical pupils,
a pink crescent mouth and two narrow ivory fangs. An irregular purple gas
halo and nine tapered comma curls form a continuous surrounding cloak.
The 11-joint skin keeps the face coherent while eight blended gas sectors
drift independently, preserving the authoring floor. Existing Float
presentation supplies the intended hover. The closed opaque papercraft
model has 19 parts, 18,911 vertices, 26,448 triangles and 809,624 bytes at
authoring height 1.05. Verified pack height is 403 (1.2954 metres), GAS.

Front/rear idle, attack and hit views were inspected. The first native side
view exposed a thin planar mist silhouette; the corrected cloak has a
0.34-unit pre-normalization depth and curls distributed forward and back.
Eight Python checks, two focused Rust checks, both shared model/timing
regressions and byte-identical Blender generation pass for the final model.
The production Route 29 encounter against Sentret passes terrain admission.
The inspected final 3D/2D pair preserves state at core frame 2110, checksum
2652738907, LCD animation frame 2734 and virtual time 47.96444296. Vapor depth,
attached visible eye/fang, original pack size, ground shadow and species
labels remain readable. No shader errors were reported. Shiny verification
was skipped at the user's request; browser visual acceptance remains open.
See `docs/art/gastly-sculpture.md`; final ignored evidence uses prefix
`target/pokemon-native-review/gastly-normal`.

### Haunter: a continuous spectral head and six independently curling claws

The generic sphere, spike leaves and simple red-eyed grin are replaced by
an original continuous ghost head with tall ear points, a ragged shroud,
slanted white eyes, fine vertical pupils, a pink open grin and eight purple
pointed teeth. Two genuinely detached palms each carry three curved purple
claws. The 16-joint rig keeps the face coherent while six blended body hinges,
two independent hands and six digit carriers supply idle, attack and hit
articulation. Existing Float presentation supplies the intended hover.

The model has 23 closed parts, 9,833 vertices, 13,444 triangles and 432,788
bytes at authoring height 1.10. Verified pack height is 503 (1.6002 metres),
GAS. Adaptive face fitting reuses the existing indexed triangle-surface reader
through an optional sampler; the optimization preserves complete GLB bytes,
and existing helper callers retain the same default. Front, rear and side
views were inspected. Native inspection exposed small dark marks on the eyes;
extra clearance did not resolve them. Smooth convex lenses with five-point
outlines and seated backs replace the ill-conditioned cap faces, removing
4,332 draft vertices. A regression checks each front eye triangle against its
convex normal as well as the actual stored parent surface. Nine Python
checks, two focused Rust checks, both shared model/timing regressions and
byte-identical Blender generation pass for the final asset. Shiny verification
is skipped at the user's request. See `docs/art/haunter-sculpture.md`.

The final production Route 29 encounter against Pidgey passes terrain admission.
The inspected 3D/2D pair preserves state at core frame 1172, checksum 2351786109,
LCD animation frame 1840 and virtual time 34.827198458. The white eyes are clean;
the complete head, shroud and detached three-clawed hands remain readable at
original pack size, with a ground shadow beneath the intended hover and correct
species labels. No shader errors were reported. Browser visual acceptance
remains open. Final ignored evidence uses prefix
`target/pokemon-native-review/haunter-normal`.

### Eevee: broad amber gaze, a fluffy ruff and a continuous cream-tipped brush

The generic fox is replaced by an original compact evolution fox with a rounded
head, large pointed ears and fitted warm brown pinnae, broad dark amber eyes,
a small muzzle and gentle smile. A continuous cream ruff and three chest lobes
surround the short neck. Rounded hind thighs, short tapered limbs, four planted
paws and subtle toe creases support the body. The bushy raised tail has a cream
end and a gently serrated material cut. The 47-part model has 10,813 vertices,
19,076 triangles and 481,024 bytes at authoring height 0.80. The verified pack
height is 100 (0.3048 metres), EVOLUTION.

The 13-joint skin supplies separate body/neck/head carriers, two ear carriers,
four softly blended limb carriers and three tail carriers. Paws and toe creases
stay fixed to the authoring root. Both tail materials share 24 identical seam
positions and quantized weights, preserving a closed color boundary through
idle, attack and hit. Front, rear and side poses were inspected. Nine Python
checks, two focused Rust checks, both shared model/timing regressions and
byte-identical Blender generation pass. Shiny verification is skipped at the
user's request. See `docs/art/eevee-sculpture.md`.

The production Route 29 encounter against Pidgey passes terrain admission.
The inspected native 3D/2D pair preserves state at core frame 3039, checksum
2392749496, LCD animation frame 3708 and virtual time 66.17505725. The full ears,
gaze, cream ruff and brush tip remain readable at original pack size; paws meet
source ground and the HUD names both species. No shader errors were reported.
Browser visual acceptance remains open. Final ignored evidence uses prefix
`target/pokemon-native-review/eevee-normal`.

### Vaporeon: pleated white frill and articulated mermaid tail

The generic blue fox is replaced by an original cyan aquatic cat with a smooth
face, violet gaze, fitted navy crown and tall central crest. Blue-rimmed head
fans carry cream membranes and two seated rays on both sides. A continuous
scalloped white neck fan frames the head. Rounded shoulder and hind-leg roots,
four planted paws, a long curling tail, paired broad flukes and seated navy
dorsal ridges complete the silhouette. The 57-part model has 13,565 vertices,
23,164 triangles, 232 accessors and 569,232 bytes at authoring height 1.00.
Verified pack height is 303 (0.9906 metres), BUBBLE JET.

The 16-joint skin provides independent head fans, neck frill and paired flukes,
three tail carriers and softly blended limbs while retaining four fixed paws.
The face, crown and crest stay coherent. Front, rear and side poses were
inspected; both fan faces retain their cream membranes and seated rays.
Nine Python checks, two focused Rust checks, both shared model/timing
regressions and byte-identical Blender generation pass. A shared room-test
compilation error was repaired by mutating its Arc slice through a temporary
vector; the external room-donor regression also passes. Shiny verification
is skipped at the user's request. See `docs/art/vaporeon-sculpture.md`.

The Route 29 native encounter against Pidgey passes terrain admission and the
inspected 3D/2D pair preserves state at frame 3871, checksum 1754180436, LCD
frame 4606 and virtual time 85.294497. The face, head fins, crest, white frill,
dorsal ridge and planted paws are readable at original size with correct HUD
labels and no reported shader errors. A second accepted down/up approach
preserves frame 2906, checksum 761625219, LCD frame 3580 and time 65.686622126.
Those two cameras self-occlude the tail. An additional opposite-side capture
using `CRYSTAL_BATTLE_REVIEW_YAW=-70` exposes the full curl and both flukes,
closing the native tail-angle review. Its inspected 3D/2D pair preserves frame
3219, checksum 409803199, LCD frame 3890 and time 69.192291249, with terrain
admission passing at original size and no reported shader errors. The native
review selector only chooses existing supported angles and retains normal
support, omission, scale and full-body/frustum checks; its focused test and
seven camera/facing/rig regressions pass. Browser visual acceptance remains
open. Optional Route 35 debug capture
attempts hit a worker stack overflow and then an active-world-frame timeout
with larger stacks; these do not count as visual acceptance. Final ignored
evidence uses `vaporeon-normal`, `vaporeon-tail-vertical` and
`vaporeon-opposite` prefixes under
`target/pokemon-native-review/`.

### Jolteon: white needle collar and grounded lightning fox

The generic yellow fox is replaced by a lean lightning fox with long angular
legs, four planted paws, sharp upright ears, fitted violet ear insets, narrow
almond gaze and swept cheek bristles. A continuous twelve-point white collar
and three long chest needles frame the throat. Eight occupied back/rump
bristles replace the generic brush tail. The 50-part canonical GLB contains
10,892 vertices, 17,840 triangles, 213 accessors and 468,260 bytes at authoring
height 1.05. The verified pack supplies height 207 (0.7874 metres), LIGHTNING.

The 11-joint rig provides coherent body/quills and head/cheeks/muzzle/gaze,
blended neck and four limbs, independent ears and a separate collar carrier.
Paws and toe creases remain fixed on the root. Source-cue timing and neutral
clip endpoints are retained. Eight Python checks and both focused Rust
rig/GPU checks pass; Blender generation is byte-identical. Front, side and
rear poses of all three clips were inspected. Initial eye caps intersected
the head; subdivided fitted surfaces correct this. Shiny verification is
skipped at the user's request. See `docs/art/jolteon-sculpture.md`.

The rebuilt native example passes a Route 29 walking encounter against Pidgey.
The inspected 3D/2D pair preserves state at core frame 1357, checksum
3790288244, LCD animation frame 1904 and virtual time 32.880172499. Terrain
admission passes at original pack size. Yellow body and violet gaze, sharp
ears, white collar, rump bristles and grounded paws remain readable; HUD labels
match the 2D state. No shader errors were reported. The two shared regressions
for grounded volumetric models and exact source-cue pose timing pass.
Evidence uses `target/pokemon-native-review/jolteon-normal`; browser visual
acceptance remains open.

### Flareon: full cream collar and flexible curled plume

The generic orange fox is replaced by a compact flame fox with a voluminous
scalloped cream collar, broad curled cream plume, swept cream forehead flame
and two seated wisps. Deep blue eyes, charcoal ear insets, short smiling muzzle,
rounded haunches and four planted paws complete the silhouette. The 46-part
canonical GLB has 11,495 vertices, 20,424 triangles, 209 accessors and 512,692
bytes at authoring height 0.97. Verified pack height is 211 (0.889 metres), FLAME.

The 14-joint skin has independent ears and collar, four blended limbs and a
continuously weighted three-carrier tail. The plume is one closed cream solid
with gentle longitudinal fur ridges; the forehead tuft stays coherent with the
head and gaze. Paws and toe creases remain on the neutral root. Eight Python
checks and both focused Rust rig/GPU checks pass. Blender generation is
byte-identical. Front, side and rear poses of all three clips were inspected.
Shiny verification is skipped at the user's request.
See `docs/art/flareon-sculpture.md`.

The initial native view exposed a red cast in the orange body material. The
body/face palette was corrected against that production result; geometry,
weights and motion are unchanged. The final material passes all eight Python
checks and both focused Rust checks, with byte-identical Blender generation.
Both shared volumetric-model and exact source-cue timing regressions also pass.

The final native Route 29 walking encounter against Pidgey passes terrain
admission at original pack size. The inspected 3D/2D pair preserves core frame
3535, checksum 3189029028, LCD animation frame 4259 and virtual time
80.209025832. `CRYSTAL_BATTLE_REVIEW_YAW=-110` selects an existing establishing
angle while retaining terrain support, omission, scale and full-body/frustum
checks. The orange body, blue gaze, dark ear insets, cream forehead tuft,
collar and full plume remain readable, with planted paws and matching FLAREON
HUD labels. No shader errors were reported. Final ignored evidence uses
`target/pokemon-native-review/flareon-normal`; browser visual acceptance
remains open.

### Espeon: ruby forehead lens and independently forked tail

The generic lilac fox is replaced by a slender sun cat with broad triangular
ears and fitted dark blue pinnae, silver-violet almond gaze, a fitted ruby
forehead lens, swept cheek fans, joined shoulders, rounded haunches and four
planted paws. Its long curved tail ends in two separately curled tips joined
at a smooth occupied hub. The 44-part canonical GLB has 10,809 vertices,
19,164 triangles, 208 accessors and 484,268 bytes at authoring height 1.00.
Verified pack height is 211 (0.889 metres), SUN.

The 15-joint rig provides coherent head/face/gem/cheeks, independent ears,
a blended neck and four limbs, three tail carriers and independently blended
fork-tip carriers. The hub and both fork roots share the final stem carrier
before the branches fade into their own tip joints. Paws and toe creases remain
on the neutral root. Nine Python checks and both focused Rust rig/GPU checks
pass. Blender generation is byte-identical. Front, side and rear poses of
all three clips were inspected. Shiny verification is skipped at the user's
request. See `docs/art/espeon-sculpture.md`.

The rebuilt native example passes a Route 29 walking encounter against Pidgey.
The inspected 3D/2D pair preserves core frame 4069, checksum 2848246175, LCD
animation frame 4787 and virtual time 93.811969876. Terrain admission passes
at original pack size. `CRYSTAL_BATTLE_REVIEW_YAW=-110` selects an existing
establishing angle with normal support, omission, scale and full-body/frustum
checks. The lilac body, ruby lens, violet gaze, broad ears, cheek fans and both
fork tips remain readable, with planted paws and matching ESPEON HUD labels.
No shader errors were reported. Both shared grounded-volumetric/model and exact
source-cue timing regressions pass. Final ignored evidence uses
`target/pokemon-native-review/espeon-normal`; browser visual acceptance remains
open.

### Umbreon: fitted hollow gold rings and seamless banded ears/tail

The generic dark fox is replaced by a compact charcoal moonlight companion
with tapered ears, a thick rising tail, red almond eyes and vertical pupils,
a small muzzle, joined shoulders, rounded haunches and four planted paws.
Hollow gold rings fit the forehead and all four outer limbs; continuous gold
bands wrap both ears and the tail. The 46-part canonical GLB has 12,563 vertices,
21,084 triangles, 195 accessors and 497,056 bytes at authoring height 1.05.
Verified pack height is 303 (0.9906 metres), MOONLIGHT.

The 13-joint skin has independent ears, a blended neck and four limbs, and
a continuously weighted three-carrier tail. Face/forehead and side limb rings
remain with their actual head and shoulder/haunch surfaces. Paws and toe creases
stay fixed on the neutral root. All six gold-band material seams share exact
positions and quantized weights through idle, attack and hit. Ten Python
checks and both focused Rust rig/GPU checks pass. Front, side and rear poses
of all three clips were inspected; Blender generation is byte-identical.
Shiny verification is skipped at the user's request.
See `docs/art/umbreon-sculpture.md`.

The rebuilt native example passes a Route 29 walking encounter against Sentret.
The inspected 3D/2D pair preserves core frame 3215, checksum 875707673, LCD
animation frame 3941 and virtual time 72.125268584. Terrain admission passes
at original pack size. `CRYSTAL_BATTLE_REVIEW_YAW=-110` selects an existing
establishing angle with normal support, omission, scale and full-body/frustum
checks. The charcoal body, red gaze, hollow gold rings, both ear bands and
full banded tail remain readable, with planted paws and matching UMBREON HUD
labels. No shader errors were reported. Both shared grounded-volumetric/model
and exact source-cue timing regressions pass. Final ignored evidence uses
`target/pokemon-native-review/umbreon-normal`; browser visual acceptance
remains open.

## Lapras swept flippers and scalloped shell (2026-10-16)

The four stretched ellipsoid flippers are replaced by broad shoulder roots,
flattened swept paddles and softly tapered tips. Blue upper and cream lower
solids share the complete cut boundary and exact quantized skin weights, so
the new undersides remain closed throughout the original flipper motion.
Fore paddles sweep forward; smaller rear paddles sweep back. The shell now
rises from a low plate into a dome with a gently scalloped edge and matching
rolled rim. Eight swept blunt spikes seat into the actual stored shell.
The visual reference remains the
[official Lapras artwork](https://sg.portal-pokemon.com/pokedex/0131/).

The 31 untouched parts retain every stored position, normal, triangle, color
and skin influence, including the fitted face, curved neck, seated belly,
body, ears, horn, spots and tail. Their geometry digest is
`d23c482ab66936a2dc341ddcef7fc2fcf51bc17b2ccc88ee8ef084d3122b4643`;
their joint/weight digest is
`b526800229af653bfd7036db354d2675682a68ffcf628947e1739966ebea157d`.
All joint rests, inverse binds, animation key times and rotations are unchanged.
The source-cue timing and nine-joint rig remain intact. Authoring height stays
1.50 and verified pack height stays 2.4892 metres. Original blue/tan RGB555
colors are retained, including the new cream undersides. Battle contact stays
seated; world swimming behavior is unchanged.

The canonical asset now has 49 parts, 19,313 vertices, 21,968 triangles,
234 accessors and 780,808 bytes. The source/neutral digest is
`7a083c865c869d1fa5f832195c3ed2ec042b1267773294cc39fe47cd3c6b789b`.
Shared normal, skin and index accessors reduce the GLB by 63,064 bytes despite
adding four material solids. Vertices fall by 158 and triangles by 1,364.
All existing schema, geometry, allocation and byte limits remain intact.

Eleven Python checks pass, including all material seams through animation,
occupied flipper roots and shell/rim/spike intersections with detached negative
controls, unchanged geometry/influences/rests/keys, fitted face clearance,
closed solids, actual normals, floor, winding and strain. Five focused Rust
checks pass: flipper seams/ownership/culling/floor, seated shell, seated battle
belly, contact shading and GPU activation at pack size. Blender generation is
byte-identical. Fixed-camera before/after views and front/oblique/rear poses of
idle, attack and hit were inspected. Shiny verification is skipped as requested.

The rebuilt native Route 29 encounter against Pidgey passes terrain admission
with the normal camera search and original 2.4892-metre pack size. Its inspected
3D/2D pair preserves core frame 2580, checksum 2751870234, LCD animation frame
3304 and virtual time 58.331329543. The receipt confirms `state_preserved=true`
and the terrain log confirms `accepted=true`. Swept paddles and cream edges,
the scalloped rim and seated spikes, retained face and blue/tan colors, and
seated belly contact remain readable with matching LAPRAS HUD labels. No
shader errors were reported. Both shared grounded-volumetric/model and exact
source-cue timing regressions pass. Final ignored evidence uses
`target/pokemon-native-review/lapras-shell-normal`, with fixed-camera comparison
and animation sheets under `lapras-shell-comparison.png` and
`lapras-shell-poses.png`. This closes the flipper/shell refinement; legacy
world props and browser visual acceptance remain open.

### Vulpix: six independent orange curl plumes

The generic orange fox is replaced by a compact russet fox with three curled
orange forelock waves, broad brown gaze, fitted dark ear insets, a small muzzle,
a fitted cream chest bib, rounded haunches and four planted dark paws. Six full
orange plumes form a layered curl fan. The 48-part canonical GLB has 13,624
vertices, 22,736 triangles, 227 accessors and 597,188 bytes at authoring height
0.85. Verified pack height is 200 (0.6096 metres), FOX.

The 17-joint rig provides independent ears, a blended neck and four limbs,
a common tail-fan carrier and six independently blended curl carriers. Face
and forelock stay coherent with the head; paws and toe creases remain fixed.
The initial fan was too thin and open; fuller plumes and tighter coils correct
its silhouette. Conforming local subdivision fixes the initial cream bib
intersection with the actual curved neck. Nine Python checks and both focused
Rust rig/GPU checks pass, including every curl articulating independently,
occupied roots, floor/culling and fitted surfaces. Front, side and rear poses
of all three clips were inspected; Blender generation is byte-identical.
Shiny verification is skipped at the user's request.
See `docs/art/vulpix-sculpture.md`.

The rebuilt native Route 29 walking encounter against Pidgey passes terrain
admission at original pack size. The inspected 3D/2D pair preserves core frame
3540, checksum 2552778670, LCD animation frame 4244 and virtual time
77.847123207. `CRYSTAL_BATTLE_REVIEW_YAW=-110` selects an existing establishing
angle with normal support, omission, scale and full-body/frustum checks.
The russet body, brown gaze, orange forelock and curl fan, cream bib and grounded
dark paws remain readable with matching VULPIX HUD labels. No shader errors
were reported. Both shared grounded-volumetric/model and exact source-cue
timing regressions pass. Final ignored evidence uses
`target/pokemon-native-review/vulpix-normal`; browser visual acceptance
remains open.

### Ninetales: cream mane and nine independently flowing golden-tipped plumes

The generic pale fox is replaced by an elegant cream fox with a longer muzzle,
sharp red gaze, fitted warm ear insets, full mane, long chest point, swept cheek
fur and three crown wisps. Joined shoulders, rounded haunches and four planted
paws support nine flowing plumes: seven rise into a high fan and two lower tails
sweep forward. Each has a continuous golden end. The 56 authored parts pack
losslessly into 46 runtime parts, 14,231 vertices, 23,784 triangles, 250 accessors
and 662,324 bytes. Authoring height is 1.30; verified pack height is 307
(1.0922 metres), FOX.

The 20-joint rig has independent ears, a blended neck and four limbs, a common
tail-fan carrier and nine independently blended plume carriers. Matching tip
materials share a primitive while retaining every source vertex's actual skin
assignment. Source ranges restore all anatomy in order. All nine cream/gold
interfaces retain exact positions and quantized weights through animation;
paws stay fixed on the neutral root. Ten Python checks and both focused Rust
rig/GPU checks pass. Front, side and rear poses of all three clips were
inspected; Blender generation is byte-identical. Renderer limits remain intact.
Shiny verification is skipped at the user's request.
See `docs/art/ninetales-sculpture.md`.

The rebuilt native Route 29 walking encounter against Sentret passes terrain
admission at original pack size. The inspected 3D/2D pair preserves core frame
3435, checksum 2149984195, LCD animation frame 4096 and virtual time
74.372243042. `CRYSTAL_BATTLE_REVIEW_YAW=-110` selects an existing establishing
angle with normal support, omission, scale and full-body/frustum checks.
The long muzzle and red gaze, cream mane and crest, high plume fan and lower
forward tails, golden ends and grounded stance remain readable with matching
NINETALES HUD labels. No shader errors were reported. Both shared grounded-
volumetric/model and exact source-cue timing regressions pass. Final ignored
evidence uses `target/pokemon-native-review/ninetales-normal`; browser visual
acceptance remains open.


# Venonat sculpture and rig

Venonat's generic stacked insect is replaced by an original continuous fuzzy
purple body, large faceted pink compound lenses, fitted lens glints, a pink
snout and paired ivory fangs. Curved white feelers end in broad pale leaves.
Small purple arms carry ivory palms and three curled claws each. Short ankles
join two broad ivory feet. Three pointed crown tufts complete the silhouette.

`tools/venonat_sculpt.py` authors 33 closed parts, 15,723 vertices and
22,268 triangles. `tools/venonat_glb.py` exports a canonical 646,940-byte GLB
with 10 joints and 154 accessors at authoring height 1.10. The verified local
pack supplies height digits 303 (0.9906 metres), INSECT. Pack size is retained.
The neutral geometry digest is
`b1425a1645ef672eb73a014ed63285503c3f184ddecb3f782c466bea7163d4f3`.
All existing renderer and schema limits remain intact.

A coherent body carrier keeps the eyes, glints, nose, fangs and crown together.
Feelers, arms and claw hands articulate separately. Blended lower body and
ankles allow motion while both feet and their toe creases remain fixed to the
floor root. Idle, attack and hit retain neutral endpoints and source-cue timing.

Eight Python checks pass: canonical bytes and limits, closed outward solids,
actual face normals, fitted glints on the faceted lenses, anatomical owners,
occupied feature roots through motion with a detached negative control,
floor/endpoints/winding/strain/motion, and source-cue metadata. Front, side and
rear poses of all three clips were inspected. Blender generation is
byte-identical. Shiny verification is skipped at the user's request.

Rebuild with `python3 tools/venonat_glb.py`. Run asset checks with
`python3 -m unittest discover -s tools -p test_venonat_glb.py`.

Both focused Rust checks pass, covering runtime routing, exact anatomy owners,
floor/culling bounds, fixed feet, independent feeler/arm/hand motion and GPU
activation at verified pack height. Shared grounded/volumetric and source-cue
pose-timing regressions also pass.

The rebuilt native Route 29 walking encounter against Pidgey passes normal
terrain admission at original pack size. The inspected 3D/2D pair preserves
core frame 2199, checksum 584843726, LCD animation frame 2794 and virtual time
48.732609791. `CRYSTAL_BATTLE_REVIEW_YAW=-110` selects an existing supported
angle with the normal size, support, omission and whole-body/frustum checks.
Compound eyes, feelers, fangs and claw hands read clearly; both broad feet meet
the ground. The matching VENONAT HUD names the modeled Pokémon. No shader
errors were reported. Final ignored evidence uses
`target/pokemon-native-review/venonat-normal`. Browser visual acceptance
remains open.


# Venomoth sculpture and rig

The generic stacked insect and angular fans are replaced by an original lilac
moth with long swept forewings and smaller scalloped hindwings. Fine dark veins
follow both faces of the actual wing surfaces. A round head carries pale blue
lenses, fitted dark round pupils and ivory reflections, paired curved ivory
mandibles and three pointed lilac antennae. Six short tucked legs flank a long
gray lavender abdomen with three seated zigzag seams. The visual reference is
the [official Venomoth artwork](https://www.pokemon.com/us/pokedex/venomoth).
Reference pixels are not shipped.

`tools/venomoth_sculpt.py` authors 53 closed parts, 19,412 vertices and
30,580 triangles. `tools/venomoth_glb.py` exports a canonical 784,824-byte GLB
with 17 joints and 223 accessors at authoring height 1.16. The verified local
pack supplies height digits 411 (1.4986 metres), POISONMOTH. Original pack size
is retained. The neutral geometry digest is
`7670a960d16e7c38c3e79a8ffe12ba570ca202b6e77bb25f1d7f21d17dcb5323`.
Existing renderer and schema limits remain intact.

Four independent wing joints carry each complete membrane and its matching
veins. Three antenna carriers articulate separately from the head, keeping the
pupils, glints and jaws coherent with the face. The striped abdomen turns on
its own carrier. Six leg carriers keep each tucked toe coherent with its leg;
leg roots blend into the thorax. The floor root remains neutral, and yaw-only
thorax/abdomen motion preserves the lower oval's floor contact. Idle, attack
and hit retain neutral endpoints and source-cue timing. Existing Fly locomotion
is unchanged; no new battle hover is introduced.

Nine Python checks pass: canonical bytes and schema limits, closed outward
solids, actual face normals, fitted pupils and glints, anatomical owners,
occupied roots through motion with a detached negative control, seated veins
and zigzag seams on stored surfaces, floor/endpoints/winding/strain/motion,
and source-cue metadata. Front, side and rear poses of all three clips were
inspected. Blender generation is byte-identical. Shiny verification is skipped
at the user's request.

Rebuild with `python3 tools/venomoth_glb.py`. Run asset checks with
`python3 -m unittest discover -s tools -p test_venomoth_glb.py`.

Both focused Rust checks pass, covering runtime routing, exact anatomy owners,
floor/culling bounds, independent wings/antennae/legs/abdomen and GPU activation
at verified pack height. Shared grounded/volumetric and source-cue pose timing
regressions also pass.

The rebuilt native Route 29 walking encounter against Sentret passes normal
terrain admission at original pack size. The inspected 3D/2D pair preserves
core frame 1801, checksum 3957057059, LCD animation frame 2417 and virtual time
42.825647333. `CRYSTAL_BATTLE_REVIEW_YAW=-110` selects an existing supported
angle with normal size, support, omission and whole-body/frustum checks.
The four swept wings and veins, pointed antennae, pale round gaze, ivory jaws,
tucked legs and striped abdomen remain readable. The lower oval meets the
ground, and the matching VENOMOTH HUD identifies the modeled Pokémon.
No shader errors were reported. Final ignored evidence uses
`target/pokemon-native-review/venomoth-normal`. Browser visual acceptance
remains open.


# Paras and Parasect sculptures and rigs

The generic eight-limbed insects and flattened mushroom ellipsoids are replaced
by original orange mushroom crabs. Four braced walking legs and two broad,
tapered foreclaws form six insect appendages. Paras has a russet rear shell,
round white eyes with large dark pupils and glints, six forehead beads, an
orange snout, paired ivory mouth claws and two small pink fungal umbrellas.
Parasect has blank pale eyes, a broader muzzle and a large pink fungal dome
that overhangs its head. Golden markings follow the actual stored cap surfaces.
Inset underside disks, radial gills and rooted cream stalks complete the fungi.
The visual references are the official
[Paras](https://www.pokemon.com/us/pokedex/paras) and
[Parasect](https://www.pokemon.com/us/pokedex/parasect) artwork.
Reference pixels are not shipped.

`tools/mushroom_crab_sculpt.py` authors both species.
`tools/mushroom_crab_glb.py` exports canonical skins within all existing limits:

| Species | Parts | Vertices | Triangles | Joints | Accessors | Bytes | Authoring height | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Paras | 56 | 19,755 | 31,860 | 11 | 213 | 711,892 | 0.78 | 0.3048 m |
| Parasect | 36 | 13,877 | 22,168 | 10 | 152 | 543,484 | 1.14 | 0.9906 m |

The verified local pack supplies height digits 100 and 303, respectively;
both are classified MUSHROOM. Original pack size is retained. Neutral digests:

- Paras: `9cf3e0d314c5f90ef570f927288a43e2dd32e7421e7d6218202fc1807ec9d82f`.
- Parasect: `ee0b479c17022294ba5cbc8084b4d2312aeb028868773ecd51113da09416838b`.

A yaw-only body and coherent head carry each face. Foreclaws articulate
independently from the front carapace. Four blended leg carriers flex between
the body and fixed walking toes. Each fungus has its own carrier shared by its
cap, golden markings, underside, gills and rooted stalk; the two juvenile fungi
can move separately. Idle, attack and hit retain neutral endpoints and source
cue timing. No terrain, grass-height, size or camera-admission rules change.

Eight Python checks pass across both assets: canonical bytes and schema limits,
closed outward solids, actual normals, fitted gaze/mouth/golden spots on stored
surfaces, exact skin owners, occupied feature roots through motion with a
detached negative control, floor/endpoints/winding/strain/motion, and source-cue
metadata. Front, side and rear views of all three clips were inspected for
both species. Blender generation is byte-identical. Shiny verification is
skipped at the user's request.

Rebuild with:

```sh
python3 tools/mushroom_crab_sculpt.py --out crates/crystal-voxel-view/models/battle_species
```
 Run asset checks with
`python3 -m unittest discover -s tools -p test_mushroom_crab_glb.py`.

Three focused Rust checks pass, covering both runtime species routes, exact
anatomical owners, floor/culling bounds, fixed walking toes, independent claws,
legs and fungal caps, and GPU activation at both verified pack heights. Shared
grounded/volumetric and source-cue pose timing regressions also pass.

The rebuilt native Route 29 Paras encounter against Pidgey passes normal
terrain admission at original pack size. The inspected 3D/2D pair preserves
core frame 2009, checksum 134018625, LCD animation frame 2623 and virtual time
45.983132001. Twin fungi, golden spots, juvenile gaze, mouth claws and digging
foreclaws remain readable with the correct PARAS HUD. Four walking toes meet
the ground. No shader errors were reported. Final ignored evidence uses
`target/pokemon-native-review/paras-normal`.

The native Route 29 Parasect encounter against Sentret also passes normal
terrain admission at original pack size. The inspected pair preserves core
frame 2148, checksum 1893713661, LCD animation frame 2778 and virtual time
48.692868291. The overhanging pink/gold cap, blank pale eyes, broad snout and
tapered foreclaws remain readable with the correct PARASECT HUD and grounded
walking feet. No shader errors were reported. Final ignored evidence uses
`target/pokemon-native-review/parasect-normal`. Both captures use
`CRYSTAL_BATTLE_REVIEW_YAW=-110`, selecting an existing supported angle with
normal scale, support, omission and whole-body/frustum checks. Browser visual
acceptance remains open for both species.


# Metapod and Kakuna sculptures and rigs

The static cocoon recipes now produce original, articulated canonical GLBs.
Metapod has a clearer projecting face, hooked crown, rear point, curved lower
tail and three fitted tail segments. Rounded ivory crescents and dark pupils
replace the rectangular eyes. Sloping green lids, cheek shields and seated
seams complete the pale green chrysalis. Kakuna has a rounded golden crown,
triangular dark eyes with small ivory glints, broad folded arm plates, a raised
central shield and three tapered abdominal seams. Its shield uses structural
cross-sections at the actual shell's profile breaks, avoiding thin overlapping
coatings and split highlight fans. The visual references are the official
[Metapod](https://www.pokemon.com/us/pokedex/metapod) and
[Kakuna](https://www.pokemon.com/us/pokedex/kakuna) artwork.
Reference pixels are not shipped.

`tools/cocoon_sculpt.py` authors both species. `tools/cocoon_glb.py` supplies
the canonical skins and strict authoring readers. All existing limits remain:

| Species | Parts | Vertices | Triangles | Joints | Accessors | Bytes | Authoring height | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Metapod | 15 | 10,664 | 15,788 | 4 | 80 | 455,168 | 0.80 | 0.7112 m |
| Kakuna | 14 | 11,069 | 16,336 | 5 | 80 | 478,004 | 0.80 | 0.6096 m |

The verified local pack supplies height digits 204 and 200, respectively;
both are classified COCOON. Original pack size is retained. Neutral digests:

- Metapod: `6b3fdcfefd6a88f8417852ca46a6d144a9d4d35c5e5e712b3c42c8b61576b36f`.
- Kakuna: `2174b2bfbb87197fe326bd5fd82a05a06dc5539f943618d45a086dcb10d1f805`.

A neutral floor root fixes the lowest shell points. Blended shell and head
carriers give small cocoon motions while the complete gaze stays coherent with
the head. Metapod has an independent blended tail fold. Kakuna has separate
folded-arm carriers shared by each plate and its seam. Idle, attack and hit use
restrained rotation-only motion, neutral endpoints and existing source-cue
timing. No terrain, grass-height, size or camera-admission rules change.

Eleven Python checks pass across both assets. These cover canonical bytes and
schema limits, closed outward solids, actual normals, physical bounds, fitted
faces and plates on stored surfaces, Metapod's continuous bent profile, exact
skin owners, fixed lower shell points, seated fold roots through motion with a
detached negative control, floor/endpoints/winding/strain/articulation, and
source-cue metadata. Shiny verification is skipped at the user's request.

Rebuild with:

```sh
python3 tools/cocoon_sculpt.py --output crates/crystal-voxel-view/models/battle_species
python3 -m unittest discover -s tools -p 'test_cocoon*.py'
```

Three focused Rust checks pass, covering both runtime species routes, exact
head/fold owners, fixed lower-shell contacts, floor/culling bounds, independent
head/tail/arm-fold motion and GPU activation at verified pack heights. Both
shared grounded/volumetric and source-cue pose timing regressions pass.
Front, side and rear poses of all three clips were inspected for both species;
Blender generation is byte-identical.

The rebuilt native Route 29 Metapod encounter against Pidgey passes normal
terrain admission at original pack size. The inspected 3D/2D pair preserves
core frame 2421, checksum 1062851264, LCD animation frame 3031 and virtual time
53.275264209. The hooked crown, projecting face, sleepy gaze, rear point and
segmented lower tail remain readable with the correct METAPOD HUD. The lower
shell tip meets the ground. No shader errors were reported. Final ignored
evidence uses `target/pokemon-native-review/metapod-normal`.

The native Route 29 Kakuna encounter against Sentret also passes normal terrain
admission at original pack size. Its inspected pair preserves core frame 2578,
checksum 1752407553, LCD animation frame 3260 and virtual time 57.848109083.
The rounded crown, triangular gaze, folded arms, clean central ridge and
abdominal bands remain readable with the correct KAKUNA HUD and planted lower
shell tip. No shader errors were reported. Final ignored evidence uses
`target/pokemon-native-review/kakuna-normal`. Both captures select an existing
supported angle with `CRYSTAL_BATTLE_REVIEW_YAW=-110`, retaining normal scale,
support, omission and whole-body/frustum checks. Browser visual acceptance
remains open for both species.


# Caterpie and Weedle sculptures and rigs

The generic detached-looking larva segments are replaced by continuous,
annulated bodies with raised heads and curled tails. Caterpie has a continuous
cream underside, five hollow yellow circles on each green side, six raised
cream leg knobs, broad yellow eye rims, black lenses and fitted ivory glints,
a two-part green snout, a red Y antenna with rounded ends and a soft pale tail
point. Weedle has a tan body, fourteen pink prolegs including six planted pads,
a broad rosy nose with a fitted reflection, small glossy black eyes and curved
ivory head and tail stings. Neck caps are buried inside the heads. The visual
references are the official [Caterpie](https://www.pokemon.com/us/pokedex/caterpie)
and [Weedle](https://www.pokemon.com/us/pokedex/weedle) artwork.
Reference pixels are not shipped.

`tools/larva_sculpt.py` authors both species. `tools/larva_glb.py` exports
canonical skins within all existing renderer and schema limits:

| Species | Parts | Vertices | Triangles | Joints | Accessors | Bytes | Authoring height | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Caterpie | 33 | 13,539 | 25,360 | 12 | 152 | 507,296 | 0.95 | 0.3048 m |
| Weedle | 24 | 10,537 | 19,772 | 12 | 112 | 415,840 | 0.95 | 0.3048 m |

The verified local pack supplies height digits 100 for both species, classified
WORM and HAIRY BUG respectively. Original pack size is retained. Neutral digests:

- Caterpie: `527fcc82fab88435eb538b1b93614b03f896566ecefafc9d0f6e05e5cb419931`.
- Weedle: `ef69247a874d0f1cec8f314d3885cd93ddaa5356fd45ce850dda1f75a0a162ea`.

Eight blended body carriers create small crawling waves. A coherent head
carries the complete gaze and snout. Caterpie's fork branches articulate
separately; its rounded antenna ends share those carriers. Weedle has separate
head- and tail-sting carriers. Caterpie's lowest belly points and Weedle's
planted pink pads stay on the neutral floor root. Raised prolegs and side rings
use the body skin field. The two closed green/cream body halves share 114 exact
stored boundary points and matching quantized skin influences through every
clip. Rings sit fully on the green surface and retain open centers.
Idle, attack and hit keep neutral endpoints and source-cue timing. No grass,
terrain, scale or camera-admission rules change.

Ten Python checks pass across both assets: canonical bytes and anatomy/limits,
closed outward connected solids, actual normals, fitted reflections, exact
anatomy owners and body carriers, exact animated green/cream seams, rings
seated on the stored green surface, occupied feature roots through motion with
a detached negative control, floor/endpoints/winding/strain/independent motion,
and source-cue metadata. Shiny verification is skipped at the user's request.

Rebuild and check with:

```sh
python3 tools/larva_sculpt.py --out crates/crystal-voxel-view/models/battle_species
python3 -m unittest discover -s tools -p test_larva_glb.py
```

The world crawl adapter reuses the eight authored body joints. It preserves
the continuous skin and both sides of the cream/green seam. Existing world IK
creates six independent Caterpie proleg chains and fourteen Weedle chains;
the head follows its authored front-body parent once. A smaller arch fits the
tiny supports' reach, preventing planted pads from being pulled above the
floor. The world regression samples 64 travel phases, checks support lengths
and level feet, enforces ground contact and a closed material seam, and returns
the complete skin to its exact neutral rest pose. Canonical battle bindings
and source-cue clips remain separate from these world-only leg chains.

Four focused Rust checks pass: runtime routing/anatomical ownership/floor/
culling/material seams and independent tips, both verified-size GPU paths,
and the world crawling regression. The three supporting pairs use staggered
phases so they cannot all enter swing together; this closes Weedle's brief
airborne interval. Front, side and rear poses of all three clips were inspected
for both species, and Blender generation is byte-identical.

All three shared grounded/volumetric, exact source-cue pose timing and readable
map-speed cadence regressions pass. The rebuilt native Route 29 Caterpie
encounter against Pidgey passes normal terrain admission at original pack size.
Its inspected 3D/2D pair preserves core frame 5514, checksum 2831601804,
LCD animation frame 6261 and virtual time 116.21247896. The small larva remains
readable with its red fork, yellow-rimmed gaze, cream body, side markings and
curled tail; the belly meets the ground and the HUD says CATERPIE. No shader
errors were reported. Final ignored evidence uses
`target/pokemon-native-review/caterpie-normal`.

The native Route 29 Weedle encounter against Sentret also passes normal terrain
admission at original pack size. Its inspected pair preserves core frame 5081,
checksum 1179181144, LCD animation frame 5848 and virtual time 113.864353625.
The tan segmented body, rosy nose and prolegs, black gaze and both ivory stings
remain readable with the correct WEEDLE HUD and grounded pink pads. No shader
errors were reported. Final ignored evidence uses
`target/pokemon-native-review/weedle-normal`. Both captures select an existing
supported angle with `CRYSTAL_BATTLE_REVIEW_YAW=-110`, retaining normal scale,
support, omission and whole-body/frustum checks. Browser visual acceptance
remains open for both species.


### Krabby and Kingler: broad pincers and independent closing jaws

Both crabs now use canonical skinned GLBs from `tools/crab_sculpt.py` and
`tools/crab_glb.py`. Broad flattened palms and tapered movable jaws replace the
round pincer tubes. Kingler has a dominant claw more than twice its smaller
pincer's height, five crown points and additional ivory jaw teeth. Krabby
keeps equal pincers and two crown points. Four explicit ivory walking toes
meet the ground. Twelve joints provide a coherent shell/face, separate arms,
wrists and closing jaws, four braced leg carriers and a fixed floor root.
World locomotion groups each leg, knee and toe into one complete walking chain.
Legacy static JSON files are removed and all runtime/builder/checker routes use
the canonical GLBs. Full details are in [crab-sculptures.md](crab-sculptures.md).

Krabby has 31 parts, 17,883 vertices, 25,576 triangles, 140 accessors and
678,056 bytes. Kingler has 36 parts, 18,026 vertices, 25,908 triangles,
156 accessors and 674,292 bytes. Both authoring heights are 0.97; verified pack
sizes remain 0.4064 m and 1.2954 m. Eleven Python checks pass, including fitted
faces, closed outward geometry, stored normals, occupied roots through motion,
a detached negative control, planted toes, winding/strain, anatomical owners,
independent jaws and source-cue contracts. Blender generation is byte-identical;
front, side and rear poses of all three clips were inspected. Four focused Rust
checks pass for GPU size, routing/owners/floor/culling/independent jaws and
64-phase world contact. The existing Paras/Parasect contact regression also
passes. Both shared grounded/volumetric and source-cue timing regressions pass.
Shiny checks remain skipped at the user's request.

The rebuilt native Route 29 Krabby/Pidgey pair passes terrain admission and
preserves core frame 3412, checksum 2771847911, LCD animation frame 4151 and
virtual time 76.7544295. The Kingler/Pidgey pair preserves frame 3538,
checksum 1460042242, LCD frame 4257 and time 79.629184374. Both inspected pairs
show full pincer silhouettes, correct species HUDs and grounded toes, with no
shader errors. Final ignored evidence uses `krabby-normal` and `kingler-normal`
under `target/pokemon-native-review/`. Both captures select the existing
supported -110-degree angle while retaining normal scale/support/omission and
whole-body/frustum checks. Grass height is untouched. Browser visual acceptance
remains open for both species.


### Shellder and Cloyster: overhanging hood and articulated hollow armor

Both bivalves now use canonical skins from `tools/bivalve_sculpt.py` and
`tools/bivalve_glb.py`. Shellder's upper hood tilts over its black face,
replacing the vertical ring silhouette. Its lower valve tilts upward, its
rear hinge sits deeper in the hood, and its tongue has independent base/fold
articulation with a matching crease. Cloyster has two closed hollow armor
halves with independent rear hinges and matching spikes, a separate blue-gray
mantle and horn, charcoal lining, and a darker central orb carrying its
complete gaze and grin. Original geometry and normal physical size remain
within the existing limits. No external geometry was imported. Details are
in [bivalve-sculptures.md](bivalve-sculptures.md).

Shellder has 19 parts, 15,585 vertices, 22,292 triangles, six joints,
88 accessors and 544,000 bytes. Cloyster has 24 parts, 17,021 vertices,
23,892 triangles, five joints, 100 accessors and 657,744 bytes. Both authoring
heights remain 1.05; verified pack heights are 0.3048 m and 1.4986 m. The
legacy static JSON files are removed and all runtime/builder/checker routes
use the canonical GLBs.

Fifteen final Python checks pass for geometry parity, bounded schemas,
closed outward shells, shading, fitted faces and details, actual hollow
cavities, occupied roots through motion with a detached negative control,
ownership, shell floor baseline, endpoints, winding/strain and independent
valves/tongue. Four final Rust checks pass for both verified-size GPU paths,
production routing/owners/floor/culling/independent articulation and 64 phases
of world swimming. Existing battle and world mollusk hover is preserved.
Front, side and rear poses of all three clips were inspected for both models;
the side review prompted a deeper rear hinge on Shellder. Final Blender exports
are byte-identical. Both shared all-species grounded/volumetric and exact
source-cue pose timing regressions pass. Shiny checks remain skipped.

Native Route 29 Shellder/Pidgey admission passes at original pack size; its
inspected 3D/2D pair preserves core frame 2592, checksum 1129406471,
LCD animation frame 3197 and virtual time 55.903143251. The Cloyster/Pidgey
pair preserves frame 3367, checksum 3177080196, LCD frame 4078 and time
72.56970316499999. Full silhouettes, the correct species HUDs, coherent faces
and existing hover placement remain visible, with no shader errors. Final
ignored evidence uses `shellder-normal` and `cloyster-normal` under
`target/pokemon-native-review/`. Both captures select the existing supported
-110-degree angle with all normal size/support/omission and whole-body/frustum
checks active. Grass height is untouched. Browser visual acceptance remains
open for both species.


### Grimer, Muk and Ditto: planted flowing bodies and soft appendages

All three slimes now use canonical skinned GLBs from `tools/slime_sculpt.py`
and `tools/slime_glb.py`. Grimer has a wider tall open mouth and shorter,
rounder digits. Muk has a wider mouth and retains its long sagging arm and
short opposite hand. Ditto has fuller upper lobes and a stronger asymmetric
side bulge. Each retains one continuous flowing body and a broad flat puddle
sole, with at least 48 contact vertices fixed to the floor root. Torso and
coherent head carriers blend through the body; Grimer and Muk add independent
palms and six digit carriers, while Ditto adds two lateral lobes with matching
highlight influences. Rotation-only idle, attack and hit clips retain neutral
endpoints and source-cue clocks. World sliding preserves the complete sole.
Legacy static JSON files are removed and all runtime/builder/checker routes
use the canonical GLBs. Details are in [slime-sculptures.md](slime-sculptures.md).

Grimer has 26 parts, 18,739 vertices, 25,172 triangles, 13 joints,
146 accessors and 730,324 bytes. Muk has 26 parts, 16,163 vertices,
21,396 triangles, 13 joints, 139 accessors and 633,072 bytes. Ditto has
eight parts, 9,310 vertices, 13,980 triangles, five joints, 51 accessors
and 403,272 bytes. All authoring heights remain 0.96; verified pack sizes
remain 0.889 m, 1.1938 m and 0.3048 m. Existing limits remain unchanged.

Seventeen Python checks pass for bounded canonical skins, normalized weights,
hierarchy/binds, exact geometry, closed outward solids and normals, real
occupied arm/palm/finger roots through motion with a detached negative control,
fixed floor contact, local articulation, endpoints/winding/strain, actual
fitted faces and Ditto highlights through poses, true annular lips, supported
folds, indexed-ray equivalence and source cues. Posed marking clearance follows
authored front surfaces; embedded edge and back walls remain intentionally
buried. Five Rust checks pass for all three pack-sized GPU paths, production
routing/owners/floor/culling and independent digits/lobes, plus 64 phases of
world sliding. Front, side and rear poses of all clips were inspected for each
model; Blender exports are byte-identical. Both shared all-species grounded/
volumetric and source-cue timing regressions pass. Shiny checks remain skipped.

All three rebuilt native Route 29 captures pass normal terrain admission at
original pack size. The inspected Grimer/Pidgey 3D/2D pair preserves frame
2795, checksum 1195468295, LCD frame 3471 and time 62.241414707. Muk/Sentret
preserves frame 2871, checksum 2640698820, LCD frame 3509 and time
62.198563916. Ditto/Pidgey preserves frame 2671, checksum 2973483575,
LCD frame 3340 and time 58.788624415. Full silhouettes, coherent faces,
correct species labels and grounded bases remain readable; no shader errors
were reported. Final ignored evidence uses `grimer-normal`, `muk-normal` and
`ditto-normal` under `target/pokemon-native-review/`. The existing supported
-110-degree angle is selected with normal size/support/omission and whole-body/
frustum checks active. Grass height is untouched. Browser visual acceptance
remains open for all three.


### Seel and Dewgong: grounded seal supports and articulated curved tails

Both seals now use canonical skins from `tools/seal_glb.py` over their
individual `seel_sculpt.py` and `dewgong_sculpt.py` recipes. Seel adds four
supported front-paddle creases and keeps its reclining body, raised lobed tail,
cream muzzle, ivory tusks, glossy teal-reflected gaze and playful tongue.
Dewgong's upper body, head and crown sweep farther forward, retaining its
continuous hooked body, white muzzle, gentle gaze, lower canines and scalloped
fans. Both have blended bodies, coherent head carriers, two continuous tail
carriers, independent paired tail fans and front flippers. Seel's paddles sweep
around Y while remaining level on the floor, and its tongue has a separate
carrier shared with its crease. Dewgong's low belly is anchored to the floor.
World sliding retains those supports and the authored articulation. Legacy
static JSON files are removed; runtime/builder/checker routes use the canonical
GLBs. See [Seel](seel-sculpture.md) and [Dewgong](dewgong-sculpture.md).

Seel has 27 parts, 13,304 vertices, 21,680 triangles, 10 joints,
137 accessors and 559,936 bytes. Dewgong has 22 parts, 15,206 vertices,
23,740 triangles, nine joints, 117 accessors and 649,548 bytes. Both authoring
heights remain 1.03; verified pack sizes remain 1.0922 m and 1.7018 m.
Existing geometry, primitive, accessor and byte limits remain unchanged.

All 26 dedicated combined Python checks pass, including occupied appendage
roots through motion with a detached negative control, posed gaze clearance,
low support, local articulation, winding/strain, and Dewgong's static signed-ray
and posed triangle-intersection body checks. The same run passed 27 existing
sea, seahorse and Sentret checks. Four focused Rust checks pass for both
verified-size GPU paths, routing/owners/floor/culling and independent tails/
flippers/fans/tongue, plus 64 phases of grounded world sliding. Three existing
terrain seam checks also pass. Front, side and rear poses of all clips were
inspected for both models, and Blender exports are byte-identical. Both shared
all-species grounded/volumetric and exact source-cue pose timing regressions
pass. Shiny checks remain skipped at the user's request.

Native Route 29 admission passes at original pack size for both models. The
inspected Seel/Sentret 3D/2D pair preserves frame 4184, checksum 3435595935,
LCD frame 4919 and time 90.181717666. Dewgong/Hoppip preserves frame 3470,
checksum 2172026011, LCD frame 4201 and time 75.831987126. Complete silhouettes,
raised paired tail fans, coherent faces and correct species labels remain
readable. Seel's low paddles and Dewgong's belly meet the terrain. No shader
errors were reported. Final ignored evidence uses `seel-normal` and
`dewgong-normal` under `target/pokemon-native-review/`. The existing supported
-110-degree angle is selected with normal size/support/omission and whole-body/
frustum checks active. Grass height is untouched. Browser visual acceptance
remains open for both species.


### Rhyhorn and Rhydon: armored support chains and articulated drill

Both rhinoceroses now use canonical skins from `tools/rhino_glb.py` over their
individual sculptures. Rhyhorn gains fitted red irises and a shorter nasal
horn while retaining its seated dorsal armor, broad wedge head and four
stocky legs. Rhydon retains all 64 authored solids, including the real drill
spiral, open fanged jaw and fitted annular tail collars. Redundant sampling is
reduced to fit the unchanged skin limits. Both have coherent head and body
carriers, independent jaws and ears, articulated tails and grounded pads.
Rhydon also has independent arms, hands and drill. World walking uses four
complete support chains for Rhyhorn and two for Rhydon; left and right toe
groups remain separate. Legacy JSON files are removed and builder, reader and
runtime routes use the canonical GLBs. See [Rhyhorn](rhyhorn-sculpture.md) and
[Rhydon](rhydon-sculpture.md).

Rhyhorn has 46 parts, 14,488 vertices, 25,368 triangles, 11 joints,
178 accessors and 520,208 bytes. Rhydon's 64 source solids occupy 48 runtime
material groups, 18,156 vertices, 28,304 triangles, 15 joints, 242 accessors
and 741,340 bytes. Authoring heights remain 1.11 and 1.40; verified pack
heights remain 0.9906 m and 1.905 m.

All 31 combined Python checks pass, covering closed outward solids, strict
logical/packed geometry and weights, occupied attachment roots, local joint
articulation, grounded supports, winding/strain, fitted markings and source
cues. Rhydon's actual open jaw and both collar surfaces remain valid through
all three clips. Four focused Rust checks pass for pack-sized GPU activation,
runtime articulation and world support chains. Both shared geometry and exact
source-cue pose timing regressions pass. Front, side and rear poses of all
clips were inspected; Blender exports are byte-identical. Shiny checks remain
skipped at the user's request.

The rebuilt native Route 29 captures pass normal terrain admission at original
pack sizes. The inspected Rhyhorn/Pidgey 3D/2D pair preserves frame 4472,
checksum 3264865042, LCD frame 5203 and time 97.399773957. Rhydon/Pidgey
preserves frame 2374, checksum 3422758645, LCD frame 3085 and time
55.014122498. Complete silhouettes, coherent faces, seated armor and correct
species HUDs are readable; their feet meet the terrain. No shader errors were
reported. Ignored evidence uses `rhyhorn-normal` and `rhydon-normal` under
`target/pokemon-native-review/`. The existing supported -110-degree angle is
selected with normal size/support/omission and whole-body/frustum checks
active. Grass height is untouched. Browser visual acceptance remains open.


### Machop family: seated muscular limbs and independent arm chains

Machop, Machoke and Machamp now use canonical skins from
`tools/fighting_glb.py` and their dedicated sculpture. Curved limbs and
interpolated torso profiles replace coarse transitions. Complete shoulder and
hip caps are buried in the body; red gaze, Machoke arm markings and Machamp
lips follow their actual parent surfaces. Mature trunks and waist bands share
the torso's rings, removing the gray patches exposed by the pose review.
Each arm has independent shoulder, elbow and wrist carriers; Machamp has four
complete chains. Hands, fingers and thumbs follow their wrist. Continuous
longitudinal arm weights avoid nearest-segment discontinuities at bent elbows.
Bodies blend through pelvis/chest/neck/head, facial details and crests remain
coherent, and Machop has an independent tail. Battle pads/toes stay fixed;
world walking uses two complete support chains. Legacy JSONs are removed and
builder, reader and runtime routes use the GLBs. See
[fighting sculptures](fighting-sculptures.md).

Machop has 32 parts, 4,377 vertices, 7,288 triangles, 13 joints,
133 accessors and 226,332 bytes. Machoke has 41 parts, 5,621 vertices,
9,264 triangles, 12 joints, 162 accessors and 271,936 bytes. Machamp has
51 parts, 6,598 vertices, 10,992 triangles, 18 joints, 199 accessors and
335,256 bytes. Authoring heights remain 0.85/1.15/1.45 and verified pack
sizes remain 0.7874/1.4986/1.6002 m. Existing limits are unchanged.

All 12 combined Python checks and five focused Rust checks pass, including
occupied limb/finger/crest/tail roots, complete buried shoulder and hip caps,
fitted eyes and markings through poses, front/rear garment coverage, fixed
floor supports, isolated joint articulation, winding/strain and source cues,
GPU activation at verified pack sizes, animated bounds and 64 phases of world
walking. Both shared all-species geometry and exact source-cue timing
regressions pass. Front, side and rear poses of all clips were inspected;
Blender exports are byte-identical. Shiny checks remain skipped. Grass height
is untouched. Browser visual acceptance remains open for all three.

Native Route 29 admission passes at original pack size for all three. The
paired 3D/2D views were inspected at preserved state:

| Pair | Frame | Checksum | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: | ---: |
| Machop / Rattata | 2249 | 343043607 | 2872 | 50.97786054 |
| Machoke / Sentret | 3168 | 2308503285 | 3853 | 70.069877625 |
| Machamp / Sentret | 2244 | 714168474 | 2851 | 49.662108499 |

Complete silhouettes, coherent red gaze, seated shoulders, clean garments and
correct species HUDs remain readable; all three models' feet meet the terrain.
Machamp's four arms and fists are distinct in the production view. No shader
errors were reported. Final ignored evidence uses `machop-normal`,
`machoke-normal` and `machamp-normal` under `target/pokemon-native-review/`.
The existing supported -110-degree angle is selected with normal size/support/
omission and whole-body/frustum checks active. Grass height is untouched.
Browser visual acceptance remains open.


### Ponyta and Rapidash: coherent layered fire and articulated equine skins

Both horses now use canonical skins from `tools/fire_horse_glb.py` over their
dedicated sculptures. The slender juvenile and athletic adult retain cream
bodies, arched necks, fitted gaze and ears, shaped legs, dark blue hooves and
seven closed flame banks. Fourteen yellow core layers remain fitted to both
sides. Rapidash retains its horn, real open jaw and raised left forehoof.
Redundant interior sampling is reduced to meet the existing skin limits;
adaptive fitting preserves actual clearance. Matching the cores to the fire
banks' interpolated normals removes broad diagonal highlights exposed by the
pose review. The shared adaptive helper's default remains unchanged.

Bodies and shoulders remain coherent; necks blend torso/neck/head influences.
Ears, tail stems, tail plumes, manes and crowns/forelocks have independent
carriers. The adult jaw carries its tongue. Each ankle bank and its two core
faces share the matching leg carrier. Supporting hooves stay fixed in battle;
Rapidash's elevated foreleg articulates independently. World walking uses four
complete chains per horse and carries ankle fire with the matching limb,
retaining the adult's authored elevated foreleg. Lossless material packing
combines each pair of core faces while preserving every logical solid.
Builder, asset readers and runtime routes use the GLBs; old static JSONs are
removed. See [Ponyta](ponyta-sculpture.md) and
[Rapidash](rapidash-sculpture.md).

Ponyta has 47 logical solids, 40 runtime material groups, 15,002 vertices,
22,280 triangles, 14 joints, 200 accessors and 637,600 bytes. Rapidash has
54 logical solids, 47 runtime groups, 17,555 vertices, 26,744 triangles,
15 joints, 228 accessors and 724,584 bytes. Authoring heights remain 1.10 and
1.30; verified pack heights remain 0.9906 m and 1.7018 m. Limits and physical
sizes are unchanged.

All 33 combined Python checks pass, including actual attachment roots through
motion, strict logical/packed schemas and weights, both posed core faces,
empty adult jaw samples, fixed supports and raised forehoof, isolated joint
motion, winding/strain and source-cue contracts. Four focused Rust checks pass
for verified-size GPU activation, runtime owners and independent carriers,
animated culling, plus four complete world chains through 64 phases. Front,
side and rear poses of all clips were inspected; Blender exports are
byte-identical. Shiny checks remain skipped. Grass height is untouched.
Browser visual acceptance remains open for both species.


Both shared all-species geometry and exact source-cue pose timing regressions
pass. Native Route 29 admission passes at original pack size for both horses.
The inspected Ponyta/Hoppip 3D/2D pair preserves frame 2063, checksum
971999260, LCD frame 2668 and time 46.357391624. Rapidash/Hoppip preserves
frame 3192, checksum 1320301839, LCD frame 3901 and time 70.527144961.
Complete silhouettes, coherent gaze, flowing fire and correct species HUDs
remain readable. Ponyta's four hooves and Rapidash's three support hooves meet
the terrain; the adult's raised foreleg and open jaw remain distinct. No shader
errors were reported. Ignored evidence uses `ponyta-normal` and
`rapidash-normal` under `target/pokemon-native-review/`. The existing supported
-110-degree angle is selected with normal size/support/omission and whole-body/
frustum checks active. Grass height is untouched. Browser visual acceptance
remains open.


### Kangaskhan: independently articulated mother and seated baby

Kangaskhan now uses a canonical skin from `tools/kangaskhan_glb.py` over its
dedicated sculpture. All 70 logical solids remain: armored mother, open mouth,
raised clawed fists, fitted thigh armor, broad grounded feet, balancing tail,
open cream pouch and complete pale violet baby. Reduced radial/interior
sampling and adaptive fitted paint fit the unchanged skin limits. Lossless
material packing produces 46 runtime groups with 18,743 vertices, 31,108
triangles, 20 joints, 252 accessors and 823,132 bytes. Authoring height remains
1.40; verified pack height remains 2.2098 m. See
[Kangaskhan](kangaskhan-sculpture.md).

Mother body, belly, pouch and dorsal armor share one carrier; head, jaw, ears,
arms and wrists articulate separately. The tail has two continuous carriers.
Haunch armor follows its supporting haunch, and ankles blend into fixed feet
and toe claws. The baby has a separate body, blended neck, head, ears and two
arm/hand grip carriers. It remains seated with both hands holding the rim.
Grip joints use explicit grip names, preventing walking arm swing from being
applied to the baby's hands. World walking retains two complete support chains
and preserves every baby influence. Builder, asset readers and runtime routes
use the GLB; the old static JSON is removed.

All 19 combined Python checks pass, including exact logical/packed geometry,
weights, actual roots, baby containment and both grips through motion, fitted
mother/baby/armor layers, real empty jaw and pouch interior, fixed feet,
isolated joint motion, winding/strain and source-cue contracts. The narrow baby
seat contact is proved with a shared occupied-volume point in the cup floor;
the generic overlap grid skips that thin interval. Detached controls remain
active. Three focused Rust checks pass for verified-size GPU activation,
runtime owners and independent mother/baby carriers, animated floor/culling,
and two world leg chains through 64 phases with baby weights preserved and
grips excluded from walking arm swing. Both shared all-species geometry and
exact source-cue timing checks pass. Front, side and rear poses of all clips
were inspected; Blender exports are byte-identical. Shiny checks remain
skipped. Grass height is untouched. Browser visual acceptance remains open.

Both shared all-species geometry and exact source-cue pose timing checks pass.
The rebuilt native Route 29 Kangaskhan/Rattata 3D/2D pair was inspected at
preserved state: frame 4570, checksum 339953756, LCD frame 5413 and virtual
time 99.975341419. The complete parent, seated baby and rim grips, open jaw,
raised fists and correct KANGASKHAN HUD remain readable; both feet meet the
terrain. Normal terrain admission passes at the original pack size and the
existing -110-degree review angle, with normal size/support/omission and
whole-body/frustum checks active. No shader errors were reported. Ignored
evidence is `target/pokemon-native-review/kangaskhan-normal`. Grass height is
untouched. Browser visual acceptance remains open.


### Tangela: rooted independent curls and coherent gaze

Tangela now uses a canonical skin from `tools/tangela_glb.py` over its dedicated
sculpture. All 41 closed vines remain, including four coherent eye-frame ropes,
front/layered curls, nine rear wraps, four side curls and four rear curls.
The existing side curls turn farther around the body for profile readability.
Hidden root centers sit inside the actual stored core, correcting lateral caps
that only grazed its surface. Reduced radial/path redundancy and smooth rope
normals retain the tangle within unchanged limits. All 49 logical solids pack
losslessly into 10 runtime material groups: 17,318 vertices, 31,236 triangles,
39 joints, 162 accessors and 896,856 bytes. Authoring height stays 1.00;
verified pack height stays 0.9906 m. See [Tangela](tangela-sculpture.md).

Body, face, eyes, pupils and four framing ropes share a coherent carrier.
The other 37 strands have independent carriers blended from embedded roots to
free curls, with distinct idle phases. Attack and hit use source-cue clocks
and return to neutral endpoints. Red shoes stay fixed in battle; world walking
uses two shoe chains and preserves every body/vine influence. Material packing
preserves each logical solid and its actual weights. Builder, asset readers
and runtime routes use the GLB; the old static JSON is removed.

All 16 combined Python checks pass, including closed uncrossed vines, actual
occupied root witnesses, fixed root caps, shoe/eye attachments, both pupils
remaining unobstructed by all 41 vines through motion, fixed floor, winding/
strain, isolated articulation, distinct phases and cue contracts. Three focused
Rust checks pass for verified-size GPU activation, all 37 vine carriers,
animated floor/culling and two world shoe chains through 64 phases with all
body/vine influences preserved. Front, side and rear poses of all clips were
inspected; Blender exports are byte-identical. Shiny checks remain skipped.
Grass height is untouched. Browser visual acceptance remains open.

Both shared all-species geometry and exact source-cue pose timing checks pass.
The rebuilt native Route 29 Tangela/Pidgey 3D/2D pair was inspected at preserved
state: frame 2551, checksum 3912378689, LCD frame 3214 and virtual time
56.338423042. The full tangle, clear pupils, red shoes and correct TANGELA HUD
remain readable; both shoes meet the terrain. Normal terrain admission passes
at original pack size and the existing -110-degree review angle, with normal
size/support/omission and whole-body/frustum checks active. No shader errors
were reported. Ignored evidence is `target/pokemon-native-review/tangela-normal`.
Grass height is untouched. Browser visual acceptance remains open.


### Drowzee and Hypno: coherent tapirs and hand-held pendulum

Both dedicated tapir sculptures now use canonical `tapir_glb.py` skins.
The 43/48 named solids remain intact, including Drowzee's welded wavy waist
and Hypno's scalloped ivory ruff and open metal ring. Reduced redundant
surface samples bring both within unchanged runtime limits: Drowzee has
19,170 vertices, 32,056 triangles, 13 joints, 188 accessors and 711,100 bytes;
Hypno has 19,105 vertices, 32,320 triangles, 14 joints, 204 accessors and
695,476 bytes. Authoring height remains 1.12. Verified pack heights are
0.9906 m and 1.6002 m. See [tapir anatomy](tapir-sculptures.md).

Both rigs use fixed battle feet, continuous blended torso/neck and leg fields,
coherent fitted faces, independent ears and three carriers per arm. Hypno's
cord and ring share a carrier parented to its right wrist and pivoting at the
held cord root. World walking builds two full support chains and preserves
pendulum influences. Idle/attack/hit follow established source-clock contracts.
Builder, strict readers, runtime GPU routing and world rigs use the GLBs;
static JSON duplicates are removed. Species::ALL now contains 187 entries.

All 16 Python checks and three focused Rust checks pass. These cover fixed
feet, geometry limits and shading, welded waist motion, independent carriers,
occupied hand/cord/ring/lip connections, bounded strain, winding, identity
endpoints, verified pack-size GPU activation and two world support chains
through 64 phases. Blender reproduces both files byte for byte. Front, side
and rear previews of all three clips were inspected. Shiny checks are skipped
and grass height is untouched. Browser visual acceptance remains open.


The focused Rust rig/GPU/world checks passed (3 tests, 1.41 seconds), as did
the shared all-species geometry and exact source-cue pose timing checks
(2 tests, 86.53 seconds). The freshly built native Drowzee/Hoppip Route 29 pair
was inspected at preserved state: frame 2928, checksum 2531053417, LCD frame
3543 and virtual time 64.29578896. The complete rounded body, fitted sleepy
face, wavy waist and fingers remain readable, with both feet on terrain and
the correct DROWZEE HUD. Normal admission passed at original pack size and
the existing -110-degree review angle with support, omission, whole-body and
frustum checks active. No shader errors were reported. Ignored evidence:
`target/pokemon-native-review/drowzee-normal`.


The native Hypno/Sentret Route 29 pair was also inspected at preserved state:
frame 1600, checksum 3201743356, LCD frame 2207 and virtual time 38.088585666.
The pointed ears, fitted gaze, ivory ruff, raised hands and connected open
pendulum remain readable; both feet meet terrain and the HUD says HYPNO.
Normal admission passed at original pack size and existing -110-degree review
yaw with all support/omission/whole-body/frustum constraints active, no fallback
and no shader errors. Ignored evidence: `target/pokemon-native-review/hypno-normal`.
Browser visual acceptance remains open. No shiny checks or grass-height edits
were made.


### Abra and Alakazam: curved anatomy, seated roots and psychic props

Abra and Alakazam now use canonical `psychic_fox_glb.py` skins over their
original dedicated sculptures. Curved torso/cuirass profiles, palms, insteps,
forearms, fingers and a curved banded Abra tail improve the static anatomy.
Both retain the tapered fox skulls and their closed or focused gaze. Adaptive
fitted ear/gaze patches clear the stored surfaces. Hidden upper-arm roots sit
inside the torso, and Alakazam's moustache blades have substantial embedded
muzzle roots. See [psychic fox anatomy](psychic-fox-sculptures.md).

All 43/53 logical parts remain. Abra has 7,254 vertices, 12,040 triangles,
15 joints, 185 accessors and 335,248 bytes; Alakazam has 7,513 vertices,
12,264 triangles, 17 joints, 222 accessors and 343,788 bytes. Authoring height
remains 1.12; verified pack heights remain 0.889 m and 1.4986 m. No runtime,
size, terrain or camera limits were relaxed. Species::ALL now has 189 entries.

Both rigs have fixed battle feet/claws, blended torso/neck and limbs, coherent
heads/gazes, independent ears and three carriers per arm. Abra adds two tail
carriers shared by the matching brown band; Alakazam adds separate moustache
and held-spoon carriers. Each spoon pivots at its curled-finger grip beneath
the wrist. World walking uses two complete support chains and retains all
prop/tail influences. Idle, attack and hit follow source-cue clocks and neutral
endpoint contracts. Runtime, readers and builder use GLBs; old JSON duplicates
are removed.

All 12 Python and three focused Rust checks pass, including actual anatomical
root occupancy through clips, fitted surface clearance, supported spoons,
independent articulation, planted feet, winding/strain, GPU activation at pack
size and two support chains across 64 world phases. Final front/side/rear pose
sheets were inspected. Blender exports are byte-identical. Shiny checks remain
skipped; grass height is untouched. Browser visual acceptance remains open.


Both shared all-species geometry and exact source-cue timing checks passed
(2 tests, 133.82 seconds). The native client was rebuilt before capture.
The Abra/Pidgey Route 29 pair was inspected at preserved state: frame 3659,
checksum 293534093, LCD frame 4382 and virtual time 79.20587775. The closed gaze,
pointed muzzle/ears, chest armor, curled fingers, banded tail and clawed feet
remain readable. Both feet meet terrain and the HUD says ABRA. Normal terrain
admission passed at original pack size and the existing -110-degree review
yaw with normal support/omission/whole-body/frustum checks active, no fallback
and no shader errors. Ignored evidence: `target/pokemon-native-review/abra-normal`.


The native Alakazam/Sentret Route 29 pair was also inspected at preserved state:
frame 2118, checksum 4140350985, LCD frame 2820 and virtual time 49.969093667.
The focused eyes, pointed ears/muzzle, seated moustache blades, chest/shoulder
armor, curled fingers and both spoons remain readable. Both clawed feet meet
terrain, and the HUD says ALAKAZAM. Normal admission passed at original pack
size and existing -110-degree review yaw with normal support, omission,
whole-body and frustum checks active, no fallback and no shader errors.
Ignored evidence: `target/pokemon-native-review/alakazam-normal`. Browser
visual acceptance remains open. Shiny checks and grass-height edits remain
skipped.


### Kadabra: connected neck, seated moustache and fitted tail band

Kadabra now uses a canonical `kadabra_glb.py` skin over its reviewed detailed
sculpture. The neck extends into the skull, both moustache halves have embedded
muzzle roots, and the tail band conforms to clipped triangles of the actual
gold tail. The other 56 parts retain exact reviewed geometry. The star, focused
gaze, brows, three pink waves, asymmetric hands, ivory finger/toe claws, heavy
curled tail and hollow held spoon remain. See [Kadabra](kadabra-sculpture.md).

All 60 logical parts pack losslessly into 38 runtime groups: 4,776 vertices,
5,848 triangles, 18 joints, 223 accessors and 293,136 bytes. Authoring height
stays 1.12; verified pack height stays 1.2954 m. Limits and native terrain/camera
constraints are unchanged. The runtime, readers and builder use the GLB; the
old JSON duplicate is removed. Species::ALL now contains 190 entries.

The floor root and complete feet/claws stay fixed in battle. Torso/neck and
limbs blend continuously, head/gaze remain coherent, and ears, wrists,
moustache, tail and held spoon have independent carriers. Anatomical part
identity keeps each moustache root on its own carrier across the center plane.
The fitted band shares the tail field. World walking builds two complete
support chains and preserves prop/tail influences. All clips use source-cue
clocks with neutral endpoints.

All nine Python checks pass, including exact unchanged geometry, lossless
ranges, closed solids and shading, occupied neck/moustache/limb roots under
motion, supported spoon and bowl/stem contact, planted feet, winding/strain,
independent articulation, strict readers and reproducible fitted band geometry.
A restored short-neck control fails the actual skull contact check. The final
front/side/rear pose sheets were inspected; the default Blender builder exports
byte-identical canonical skins. The explicit detailed-source review mode was
also executed. Shiny checks remain skipped; grass height is untouched. Browser
visual acceptance remains open.


The three focused Rust checks passed against the final skin (0.40 seconds):
GPU activation at the verified 1.2954 m pack height, authoritative rig sampling
and bounds with fixed complete feet and anatomical moustache owners, and two
world support chains through 64 phases with tail/moustache/spoon weights
preserved. The native example was rebuilt before capture.


Both shared all-species geometry and exact source-cue timing checks passed
(2 tests, 124.45 seconds). The rebuilt native Kadabra/Hoppip Route 29 pair was
inspected at preserved state: frame 2921, checksum 3542893732, LCD frame 3623
and virtual time 63.956251295. The connected neck, rooted swept moustache,
forehead star, focused eyes, three pink waves, curled fingers, raised held
spoon and continuous tail band remain readable. Both clawed feet meet terrain,
and the HUD says KADABRA. Normal terrain admission passed at the original
1.2954 m pack height and existing -110-degree review yaw, with normal support,
omission, whole-body and frustum checks active, no fallback and no shader errors.
Ignored evidence: `target/pokemon-native-review/kadabra-normal`. Browser visual
acceptance remains open. Shiny checks and grass-height edits remain skipped.


### Koffing and Weezing: cratered orbs and independently phased vapor

The original gas sculptures now use canonical `gas_pair_glb.py` skins.
Local orb/cloud sampling reductions and adaptive fitted patches retain all
thirteen radial craters per orb, recessed expressions, skull/crossbones,
rooted canines, adult joining stalks and both five-lobed gas billows. All
23/35 original logical parts remain. See [gas pair](gas-pair-sculpture.md).

Koffing has 10,563 vertices, 18,264 triangles, 12 joints, 127 accessors and
442,308 bytes; Weezing has 16,974 vertices, 28,756 triangles, 14 joints,
188 accessors and 687,324 bytes. Authoring heights stay 1.00/1.20, and verified
pack heights stay 0.6096/1.1938 m. Both now fit the unchanged geometry/skin
limits. Runtime, readers and builder use the GLBs; old JSON duplicates are
removed. Species::ALL now contains 192 entries.

Faces, emblems and fangs remain coherent with each orb. Weezing's upper orb
and rear node have independent carriers joined by blended stalks. Each
root-parented cloud has a separate carrier and four lobe carriers, with eight
distinct lobe phases. Occupied roots retain core/lobe connections, and coherent
tiny polar cap fields prevent quantized-weight pinching. Intentional float
locomotion and battle hover remain. Clips follow source-cue clocks and neutral
endpoint contracts.

All 20 combined Python checks pass, including actual crater emptiness at rest
and under motion, fitted features, occupied roots, floor/winding/strain,
independent carriers and gas phases. Final front/side/rear poses were inspected;
Blender outputs are byte-identical. Shiny checks remain skipped and grass height
is untouched. Browser visual acceptance remains open.


Four focused Rust checks pass (1.78 seconds): GPU activation for each species
at its verified pack height, all independent carriers with coherent features,
animated floor/culling bounds and preserved intentional battle hover, and 64
world floating phases with every authored influence preserved. Both world rigs
have no walk legs or additional limb motion.


The shared all-species geometry and exact source-cue timing checks passed.
An additional unrelated `outdoor_entry_mat` regression failed at its expected
marked-cell count (0 rather than 8). Its temporary image-sample lifetime was
corrected to unblock test compilation; this changes test setup only. The
failure is separate from the passing model/rig checks and remains recorded in
`target/pokemon-native-review/gas-shared-tests.log`.


The rebuilt native Koffing/Sentret Route 29 3D/2D pair was inspected at preserved
state: frame 2506, checksum 2573841080, LCD frame 3143 and virtual time
55.148248124. The cheerful grin, tongue/canines, cratered silhouette, lower
skull/crossbones and separate billows remain readable, with the correct KOFFING
HUD and intentional hovering clearance. Normal terrain admission passed at the
original 0.6096 m pack height and existing -110-degree review yaw, with normal
support/omission/whole-body/frustum constraints active, no fallback and no
shader errors. Ignored evidence: `target/pokemon-native-review/koffing-normal`.


The native Weezing/Pidgey Route 29 pair was also inspected at preserved state:
frame 2347, checksum 3242319440, LCD frame 3031 and virtual time 54.178510084.
The linked main/upper orbs, rear node, strained expressions, solid canines,
craters, cream emblems and independent billows remain readable, with the
correct WEEZING HUD and intentional hovering clearance. Normal admission
passed at the original 1.1938 m pack height and existing -110-degree review
yaw with normal support/omission/whole-body/frustum checks active, no fallback
and no shader errors. Ignored evidence: `target/pokemon-native-review/weezing-normal`.
Browser visual acceptance remains open. Shiny checks and grass-height edits
remain skipped.


### Magnemite and Magneton: open magnet hinges and axial screw motion

Both dedicated steel-orb sculptures now use `magnet_glb.py` canonical skins.
Adaptive fitted eyes, simpler flat cross-slot caps and smooth helical thread
normals reduce redundant geometry while keeping the sphere/eye proportions,
open beveled horseshoes, red/blue pole blocks, three slotted screws per orb,
3.5-turn crown threads and three joined Magneton bodies. Eye shading matches
stored parents. See [magnetic sculptures](magnet-sculptures.md).

Magnemite retains 18 logical parts in 9 runtime groups: 5,966 vertices,
9,140 triangles, 7 joints, 64 accessors and 286,008 bytes. Magneton retains
54 logical parts in 27 groups: 17,898 vertices, 27,420 triangles, 19 joints,
152 accessors and 687,728 bytes. Heights remain 0.96 authoring and verified
pack heights 0.3048/0.9906 m. Geometry, skin, size, camera and terrain limits
are unchanged. Runtime, readers and builder use GLBs; old JSON duplicates
are removed. Species::ALL now contains 194 entries.

Each unit has a coherent orb/eye carrier, two magnet hinges with attached pole
blocks, one crown carrier and two face-bolt carriers. Crowns twist around each
original unit's actual axis; lower-unit turns are retained. All metal parts
stay rigid, and the three adult body joins remain occupied through motion.
Source-cue idle/attack/hit clips return to neutral endpoints. Intentional
battle/world hover remains, with all mechanical weights preserved.

All 16 Python and four focused Rust checks pass: lossless source ranges,
strict geometry/clip readers, closed solids/shading, true roots and cluster
joins, open notches and solid arms, rigid lengths, floor/winding, independent
carriers, correct crown axes, pack-size GPU activation and 64 world floating
phases without extra limb motion. Final front/side/rear pose sheets were
inspected; Blender exports match the canonical bytes. Shiny checks remain
skipped and grass height is untouched. Browser acceptance remains open.


Both shared all-species geometry and exact source-cue timing checks passed
(2 tests, 90.99 seconds). The rebuilt native Magnemite/Pidgey Route 29 pair
was inspected at preserved state: frame 2450, checksum 720946100, LCD frame
3103 and virtual time 54.029684417. The coherent eye, face bolts, threaded
crown, open horseshoes and red/blue poles remain readable, with the correct
MAGNEMITE HUD and intentional floating clearance. Normal terrain admission
passed at the original 0.3048 m pack height and existing -110-degree review
yaw with normal support/omission/whole-body/frustum constraints active, no
fallback and no shader errors. Ignored evidence:
`target/pokemon-native-review/magnemite-normal`.


The native Magneton/Pidgey Route 29 pair was also inspected at preserved
state: frame 2803, checksum 4239097170, LCD frame 3441 and virtual time
61.380406917. All three joined bodies, eyes, face screws and colored open
magnets remain readable, with the correct MAGNETON HUD and intentional
floating clearance. Normal terrain admission passed at the original
0.9906 m pack height and existing -110-degree review yaw, with normal
support/omission/whole-body/frustum constraints active, no fallback and no
shader errors. Ignored evidence: `target/pokemon-native-review/magneton-normal`.
Shiny verification remains skipped; browser visual acceptance remains open.


### Cubone and Marowak articulation (196 registered authored rigs)

Cubone and Marowak now use canonical GLBs, retaining all 45/52 original named
parts. Both have 14 joints: supported bodies, neck/skull, two three-joint arms,
thighs and two tail carriers. Skull paint and horns remain coherent; gripping
fingers/thumb and rigid bifurcated club travel together on the actual left wrist.
World walking uses two complete support chains. Source heights remain 1.00/1.18;
no model scale or camera admission rule changed. See the species sculpture
notes for geometry budgets and neutral pins. Blender byte parity and final
front/side/rear pose sheets were inspected. Runtime checks are in progress.
Shiny verification stays skipped and browser visual acceptance remains open.

All 32 Python checks pass: 13 Cubone anatomy checks (18.519 seconds),
13 Marowak anatomy checks (17.956 seconds), and six stored skin checks
(67.347 seconds). They cover exact canonical geometry and budgets, closed
solids and actual normals, fitted face clearance, empty recessed spaces,
rooted anatomy and gripping fingers, full hierarchy/weights/binds, independent
carriers, actual occupied joins through clips, fixed feet/claws, bounded edge
strain and winding, source-clock endpoints and mutation rejection. The first
independent-carrier check caught imperceptible Cubone thigh articulation;
the supported thigh yaw was increased while keeping all feet fixed.

The Rust pack reader confirms CUBONE = 104 / LONELY = 0.4064 m and
MAROWAK = 303 / BONEKEEPER = 0.9906 m. Four focused Rust checks passed
in 2.20 seconds: both pack-size GPU activations, fixed battle supports with
club/wrist ownership and bounds, and 64 phases of two-foot world walking
with the held club's authored weights preserved. Native paired battle review is recorded below.



Native Cubone/Sentret Route 29 3D/2D pair inspected: frame 3283, checksum
1716862610, LCD frame 3963, virtual time 73.682151458. The original 0.4064 m
size, oversized smooth skull, recessed eye/nostrils, rooted feet, curved fingers
and held club remain readable. Correct CUBONE HUD; no fallback or shader error.
Normal terrain admission passed at the existing -110-degree review yaw with
support, omission, whole-body and frustum gates intact. State was preserved.
Ignored evidence: `target/pokemon-native-review/cubone-normal`.

Native Marowak/Pidgey Route 29 3D/2D pair inspected: frame 4728, checksum
3246843888, LCD frame 5500, virtual time 105.379601959. The original 0.9906 m
size, long rounded skull/muzzle, narrow gaze, raised held club, muscular tail
and grounded broad feet remain readable. The diffuse fog light preserves the
bone/skin distinction. Correct MAROWAK HUD; no fallback or shader error. Normal
terrain admission passed at the existing -110-degree review yaw with support,
omission, whole-body and frustum gates intact. State was preserved. Ignored
evidence: `target/pokemon-native-review/marowak-normal`.

Both current canonical compressed payloads are present in the native executable.
World first-person/elevated and browser reviews remain open.


### Croconaw and Feraligatr (198 authored registry entries)

Canonical 14-joint skins replace both static JSONs. Selective jaw subdivision,
curved body/limb profiles, rounded digits, rooted teeth, seated curved bevels
and centerline-following tail fins retain all 49/80 logical solids in 28/41
runtime groups. Published assets have 12,078/14,951 vertices and 19,044/21,756
triangles, within the unchanged runtime limits. Original pack sizes remain
1.0922/2.3114 m. Thirteen Python checks, the published strict reader, four
focused Rust checks and exact Blender byte parity pass. Signed-zero position
canonicalization makes exports reproducible without changing numeric geometry,
weights or clips. Final pose sheets are inspected; native multi-view and browser
reviews remain pending. See `crocodile-sculptures.md` for details and pins.


The shared all-species grounded/volumetric geometry and exact production
source-cue timing checks both pass against the current 198-entry registry
(2 tests, 124.73 seconds). Fresh battle and world review executables build
successfully; all four current canonical compressed payloads are present in
the battle executable. Multi-view native review continues below.


Native Croconaw/Sentret Route 29 pair inspected at preserved state: frame
5213, checksum 1248705665, LCD frame 6187, virtual time 137.530594338. The
original 1.0922 m body, rounded broad snout, coherent eyes/paint, rooted teeth,
rounded fingers/toes and attached tail diamond remain readable against the
retained terrain. Correct CROCONAW HUD; no fallback or shader error. Normal
terrain admission passed at the existing -110-degree review yaw with support,
omission, whole-body and frustum constraints intact. Ignored evidence:
`target/pokemon-native-review/croconaw-normal`.

Native Feraligatr/Sentret Route 29 pair also inspected at preserved state:
frame 4638, checksum 2873737362, LCD frame 5613, virtual time 128.680985458.
The original 2.3114 m size is retained. The whole red-crested silhouette, broad
rounded jaws, seated teeth, clean V belly, curved limbs/armor, counterbalancing
tail and grounded clawed feet fit in the actual scene. Diffuse fog light leaves
the blue/cream/red anatomy readable. Correct FERALIGATR HUD; no fallback or
shader error. Normal terrain admission passed at the existing -110-degree
review yaw and normal support/omission/whole-body/frustum constraints. Ignored
evidence: `target/pokemon-native-review/feraligatr-normal`.

World first-person review is not accepted yet. `render_at_location` with
CUBONE in NewBarkTown (5,8), image-target paired capture and first-person
camera aborts with a Compute Task Pool stack overflow. It reproduced with
both RALA and LR ordinary inputs. The debugger diagnosis is recorded below;
no model resizing or admission relaxation was used. Browser review remains open.


World QA stack diagnosis: LLDB stopped in
`GameDataSet::apply_runtime_mutation_command_with_checksum` at its stack probe.
The unoptimized native QA override reserves 0x1b1000 + 0xf00 = 1,777,408 bytes
(about 1.70 MiB) in that function, before its caller frames. This is the
Compute Task Pool overflow, not an observed mesh/bind failure. Native world
review was rebuilt with the repository's normal optimized `crystal-assets`
profile (opt-level 2), retaining the same geometry, sizes, controller inputs and
cameras. No runtime source or thread-stack limit was changed. World review
remains unaccepted until the actual paired captures pass. Diagnostic evidence:
`target/pokemon-native-review/cubone-world-debug.log` and
`target/pokemon-native-review/world-stack-disassembly.log`.


Stationary native world review now runs with normal optimized assets. The
New Bark gate capture was rejected because it showed dialogue rather than a
Pokémon. Route36 (20,8), noon pairs preserve state and show Cubone/Marowak
fronts in first person and sides at elevated gameplay scale. Their skulls,
held clubs and support remain coherent at original size. Croconaw and
Feraligatr elevated silhouettes also remain readable, but close limb
transitions need more organic contour work. Feraligatr's first-person body
view crops its head at the normal follower distance; face acceptance remains
open. Per-species documents record receipts and limitations. Browser and
moving-world acceptance remain open. Shiny checks remain skipped per user
direction; grass height is owned by another agent.


Crocodile contour correction after actual close native review: exposed elbow
and knee closing disks are buried inside adjoining curved surfaces; feet
receive local silhouette sampling. The first Croconaw hock curve folded
internally, caught by a new independent triangle-crossing check and fixed
with a straight buried shin root. Feraligatr knee armor follows the full
joined thigh/hock envelope. Logical/runtime part counts, original size,
hierarchy, binds, clips and all unrelated geometry/weights are preserved.
All 15 Python checks and four focused Rust activation/support/world-walking
checks pass; Blender exact export parity holds; the complete exact-species
schema check passes after correcting an unchanged Heracross vertex fixture.
Current Feraligatr paired native battle/elevated views show coherent joins,
support and full silhouette. First-person frontal face readability remains
limited by its large jaw at the normal follower gap. Final Croconaw native
review is recorded in the per-species art guide. Browser/moving-world
acceptance remain open; shiny checks remain skipped.


Wartortle and Blastoise canonical skins increase the explicit authored rig
registry from 198 to 200 entries. All 68/73 named source solids remain in
39/44 losslessly grouped runtime parts. Their original pure geometry recipes
now use adaptive fitted surfaces and focused round-body sampling within the
existing budgets. Both fifteen-joint rigs preserve rigid shells and scutes,
supported feet, wrist-owned claws, distinct feather ears or complete hollow
cannon assemblies, and two tail carriers. Wartortle's cloud tail gains real
billowing volume after side-view critique, with an in-plane blend that avoids
the initial thin-rim fold. All 34 Python anatomy/skin checks pass across final
bytes; final Blender export parity holds. Four focused Rust production/GPU/
walking checks pass. Shared and native multi-view receipts are recorded in
`docs/art/turtle-sculptures.md`. Seventy-five static species files still remain,
with browser/moving-world acceptance and previously documented large-model
framing work also open. Shiny checks remain skipped; grass is untouched.


Turtle native review: both current canonical GLBs are verified inside the
fresh production example binaries. Original-size paired battles show correct
WARTORTLE/BLASTOISE names and both exact models with no fallback; terrain
admission passes without rule changes. Stationary first-person and elevated
views show the revised tail, coherent fitted shell/gaze and real cannon rims.
Both still expose a visible wrist junction needing further contour work;
these models are not fully visually accepted yet. The 200-entry registry
schema and production cue checks pass (2 tests, 193.15 s). Receipts, critiques
and remaining acceptance work are in the turtle art guide.


The turtle wrist correction replaces each separate arm/palm pair with one
closed forelimb, reducing source/runtime parts to 66/37 and 71/42. Smooth
radius interpolation and eased terminal bends remove the visible wrist ring
in final native first-person and elevated views. Both fifteen-joint rigs,
original claws, shell and feature ownership remain intact. All 35 Python
geometry/skin checks and four focused Rust activation/support/walking checks
pass; both Blender exports match canonical bytes. Final native pairs preserve
state and the current compressed assets are verified in the executable.
The turtle art guide records the final pins, counts, critiques and receipts.
Browser, moving-world and final-revision battle visual acceptance remain open;
75 static species files still remain. Shiny checks remain skipped.


Magmar increases the explicit authored rig registry from 200 to 201 entries.
Its original 55 source solids remain in 39 losslessly grouped runtime parts.
Adaptive fitted layers, focused round-body sampling and a locally rounded
open bill fit 19,923 vertices/30,320 triangles in the existing limits.
The seventeen-joint skin preserves coherent bill/tongue/gaze, continuous
forelimb/cuff fields, wrist-owned claws, fixed complete battle supports,
independent crown flames and a three-carrier burning tail. Thin flame banks
and their fitted glow layers move together after a differential rim-fold
regression. All 20 final Python checks and three production activation,
support and walking checks pass. The 201-entry schema passes on final bytes;
shared source-cue timing also passes, and Blender export parity holds.
Native paired first-person/elevated/battle views preserve state, correct HUD
names and authentic terrain admission at 1.2954 m with no fallback.
The rounded bill is accepted; faint close-range belly shadow bands remain
open, as do browser/moving-world visual acceptance. See
`docs/art/magmar-sculpture.md` for final pins, counts and receipts.
Seventy-four static species files remain. Shiny checks stay skipped; grass
and renderer AA/filtering are untouched.


Electabuzz geometry preparation preserves all 70 named parts while reducing
64,252 vertices/113,628 triangles to 19,502 vertices/33,748 triangles through
adult-only adaptive fitting and focused round/stripe sampling. All thirteen
pair anatomy checks pass on current canonical assets; Elekid's canonical
bytes remain exact. Electabuzz still uses its static JSON while its rig
migration is in progress, so the authored registry remains 201 and 74 static
species files remain.

Automated native screenshots now default to headless image capture. The
window-system plugin and macOS native-window retention are disabled in this
mode, and the scheduler drives the production renderer directly. Eight
capture regressions pass, including default selection and image target,
viewport, ownership and paired-state behavior. An explicit headless paired
GPU capture completes at 1000×700 with state preserved (Route36, frame 2178,
checksum 3615010567). AGENTS.md requires offscreen automation and explicit
user authorization for visible/live renders. Shiny checks remain skipped.


Electabuzz completes its canonical standard-skin migration, increasing the
explicit authored registry from 201 to 202 entries. The 70 source solids
remain in 41 losslessly grouped runtime parts (19,502 vertices, 33,748
triangles, 19 joints). The rig follows the distinct raised-left/forward-right
arms, preserves the open fang-bearing jaws and knuckled fists, shares smooth
arm/leg/tail fields with stripe wraps, and articulates antennae and scalp
tufts independently. Complete feet/claws stay fixed in battle; world walking
uses two complete support chains. All 21 Python geometry/skin/anatomy checks,
three production activation/support/walking checks and two shared registry/
cue checks pass. Blender export parity holds, and Elekid's bytes are exact.
All final native close/elevated/battle review is headless and muted. Paired
state, original 1.0922 m pack size, correct HUD names and authentic terrain
admission are verified with no fallback. See the electric-pair art guide for
pins and receipts. Browser/moving-world acceptance remain open. Seventy-three
static species files remain; shiny checks stay skipped and grass/AA/filtering
are untouched.


## Shell bipeds: Slowbro and Slowking

Slowbro and Slowking increase the authored rig registry from 202 to 204.
Their 74/62 original named solids pack losslessly into 40/35 runtime parts.
Focused round sampling and fitted surfaces reduce their former
45,019/38,798 vertices to 19,528/19,169, with 32,764/31,984 triangles.
Both remain below the existing geometry, accessor, part and byte limits.
Each has 18 joints, supported feet/claws, independent shoulder/elbow/wrist
chains, body/neck/head motion and a two-joint tail. Slowbro's whole shell
follows its tail attachment with a separate biting jaw; Slowking's crown
and ruby stay coherent over three independently moving collar sectors.
Both mouths on Slowbro remain empty through the sampled source clips.

The inside of the broad elbows exposed closest-path weight discontinuities
in the first pose checks. A continuous longitudinal arm field removes the
jump without reducing the authored motion or changing the limits. Final
sampled edge ratios are approximately 0.89–1.09. All 33 Python checks and four
production GPU/support/walking checks pass; background Blender reproduces
both GLBs byte for byte. Their art guides record geometry pins and source
provenance. Seventy-one static species files remain. Shiny checks stay
skipped, and grass height, AA and filtering are untouched by this batch.


The final 204-entry registry/schema and cue-timing regressions also pass.
Six current native captures are headless and muted, with guarded executable
identity and exact embedded GLBs. Each 3D/2D pair preserves state. Both
Route29 battles name the intended species and SENTRET and use the models
without fallback; the full battle views show grounded anatomy. Ordinary
first-person close framing crops the upper face/head, so close-view
acceptance remains open alongside wrist seam polish, browser and moving-world
review. A separate terrain-admission trace was not collected; no admission
claim is made from these receipts. Art guides retain exact paired-state
checksums and clocks. No scale, scenery, grass, AA or filtering changes were
made for this batch.


## Continuous forelimbs for Slowbro and Slowking

The two overlapping palm solids on each biped are replaced by continuous
closed arm/wrist/palm surfaces, using smoothly interpolated centerlines and
radius profiles as on the turtles. Eased elbow bends remove self
intersections; every original hand claw stays rooted through the source
clips. A stored-surface regression rejects large closing fans outside the
buried shoulder. The 70/58 non-arm/palm solids retain exact f32 geometry,
normals, indices and colors.

Current Slowbro/Slowking assets contain 19,322/18,831 vertices,
32,532/31,512 triangles, 72/60 logical parts and 38/33 runtime parts.
The 18-joint rigs and original physical scale remain. All 34 Python tests,
four production GPU/support/walking tests and two shared schema/cue tests
pass. Background Blender reproduces both current canonical GLBs exactly.
The registry remains at 204 entries; 71 static species files remain.

Six fresh native reviews are entirely headless and muted, with verified
executable identity and exact embedded GLBs. The first-person portraits use
ordinary wide controls `0 0 -10 1`, making the faces/crowns and wrists
readable. Every 3D/2D pair preserves state. Both Route29 battles name the
intended species and PIDGEY without model fallback and show grounded anatomy.
This revision collects the named encounter trace: both are accepted at their
actual unchanged 1.6002/2.0066 m pack heights, with no scale, support,
omission or admission workaround. Art guides record the exact paired-state
checksums and clocks. The earlier tight `1 0 20 1` view still crops upper
heads; camera behavior is unchanged. Browser and moving-world visual review
remain open. Shiny checks are skipped; grass, AA and filtering are untouched.


## Lickitung: tongue and tail articulation

Lickitung increases the explicit authored rig registry from 204 to 205, replacing its static JSON with one canonical GLB. Focused round
sampling and conforming surfaces reduce 61,784 vertices and 108,716 triangles
to 17,028 vertices and 29,256 triangles, with 33 logical solids packed
losslessly into 24 runtime primitives. The file contains 18 joints, 157
accessors and 758,680 bytes; all existing limits remain intact.

Body/neck/head, independent shoulder/elbow/wrist chains and supported legs
carry three tongue and three tail joints. Continuous palms remove the old
separate hand solids while retaining rooted white nail tips. The fitted
tongue groove interpolates the actual stored, quantized parent-triangle
weights. Fixed complete feet and toes preserve battle floor contact and
production two-foot walking. The closed head retains a physical oral recess;
independent stored-geometry rays verify its depth. All original pink, cream,
white and dark facial colors remain.

Native close review caught a jagged mouth fringe on the initial cap. Finer
head sampling and a 1 mm source-space lining clipped directly from the head
triangles avoid lifted chords over the recess and reduce the fringe. An
independent test limits the exposed lining standoff to 0.1% of body height,
in addition to the existing clearance threshold. Faint close mouth-rim shadow
aliasing remains open for shading polish; no renderer workaround is added.

All 20 Python geometry/skin regressions, three production GPU/support/walking
checks and two shared registry/cue checks pass. Background Blender dispatch
reproduces the canonical bytes exactly. Three final native captures are
entirely offscreen and muted, with verified executable identity and exact
embedded GLB. Every 3D/2D pair preserves state. The Route29 LICKITUNG/SENTRET
battle uses both models without fallback and is admitted at the original
1.1938 m pack height with zero support offset. The art guide records source
provenance, pins and exact state receipts. Seventy static species files
remain. Browser/moving-world review stays open. Shiny checks are skipped;
grass, AA and filtering are untouched by this batch.


## Mr. Mime: ten glove digits and supported slippers

Mr. Mime increases the explicit authored rig registry from 205 to 206,
replacing its static JSON with a canonical GLB. Focused round sampling and
removal of coplanar fan subdivisions reduce 63,305 vertices and 105,964
triangles to 19,663 vertices and 33,212 triangles. All 67 logical source
solids remain, packed losslessly into 32 runtime primitives. The file has
26 joints, 218 accessors and 917,600 bytes under the unchanged limits.

Body/neck/head, separate shoulder/elbow/wrist chains, hips and knees, two
complete fan carriers and ten whole-digit carriers preserve the asymmetric
mime pose. Every glove keeps five rooted fingers with rounded tips and
fitted rosy pads. Complete blue slippers and curled toes remain fixed in
battle. The physical smile and fitted face share the head transform; its
concave closed lining follows the actual front head triangles at 1 mm
source-space standoff. Stored-ray checks reject raised chords and keep the
oral probes empty through every clip.

All 22 Python geometry/skin regressions, three Rust GPU/support/walking
checks and two shared registry/cue checks pass. Background Blender dispatch
reproduces the canonical bytes exactly. Six software neutral/idle/attack/hit
views and three native 3D/2D pairs were inspected without opening any window.
The native build contains the exact final GLB and headless/mute controls;
every capture explicitly uses them and preserves its paired game state.
Route36 first-person/elevated views retain the complete physical size and
grounded slippers, with partial foliage occlusion of the raised glove. The
Route29 MR__MIME/PIDGEY battle shows both models without fallback and is
admitted at the original 1.2954 m pack height with zero support offset.
The art guide records exact pins and paired-state receipts. Close mouth-rim
shadow aliasing remains open for polish. Sixty-nine static species files
remain; browser and moving-world visual review remain open. Shiny checks,
grass, AA and filtering are outside this batch.


## Chansey and Blissey: contained eggs and continuous palms

The egg nurses increase the authored rig registry from 206 to 208,
replacing both static JSON files with canonical GLBs. Chansey reduces
28,420 vertices and 51,296 triangles to 10,390 vertices and 18,704 triangles:
24 logical solids, 16 runtime primitives, 17 joints, 124 accessors and
499,164 bytes. Blissey reduces 39,460 vertices and 70,872 triangles to
19,379 vertices and 34,472 triangles: 38 logical solids, 19 runtime
primitives, 25 joints, 163 accessors and 918,928 bytes. All limits and
physical-size paths remain unchanged.

Both eggs stay contained in actual open pink cups on the body carrier.
Chansey's distal palms retain their egg grip while the shoulder/elbow
chains flex; six ribbons and two tail carriers move independently. Blissey
adds independent crown/whole-curl pairs, two shoulder-frill assemblies and
ten individual skirt-petal carriers. Fixed complete feet retain battle
floor contact and production two-foot walking. Fitted faces remain coherent.

Software review caught Blissey's faceted recess above the happy mouth.
Denser facial sampling and a tighter physical depression remove that patch
while preserving the empty-mouth probes. Its closed lining conforms to the
actual front triangles at 1 mm source-space standoff. Welding at export
precision prevents collapsed slivers, with unchanged Mr. Mime bytes.
Native close review then exposed dark seams at separate pink hand/arm
joins. Continuous forelimbs replace two Hand solids per species; eased
centerlines prevent inside-wrist crossings and retain rounded palms. A new
regression rejects large closing fans outside the buried shoulders. Other
source solids are preserved through that correction.

All 33 final Python pair checks, four Rust GPU/support/walking checks and
two shared registry/cue regressions pass. Background Blender dispatch
reproduces both canonical GLBs exactly. Final software pose sheets and six
selected production native 3D/2D pairs were inspected entirely offscreen and
muted, after checking executable identity and exact embedded payloads. All
pairs preserve game state. Close views show the corrected palms; Blissey's
review camera uses existing pitch-20 controls to keep its silhouette in
frame after rejecting a cropped pitch-minus-10 view. Elevated and battle
views show grounded feet. Both battles have named HUDs, no model fallback
and successful admission at unchanged 1.0922/1.4986 m pack heights with zero
support offset. The species art guides record exact geometry pins, asset
hashes, capture clocks and state receipts.

Sixty-seven static species files remain. Browser and moving-world visual
review remain open. Shiny checks are skipped; grass, AA and filtering are
untouched by this batch.


## Growlithe and Arcanine: distinct striped canine skins

The authored rig registry grows from 208 to 210. Both retired static dogs
used the same coarse 1,986-vertex, 2,796-triangle, 48-part topology with only
small head-height changes. New original parametric sculptures distinguish
the compact round-eyed puppy and long-legged, full-maned guardian. Growlithe
has 16,302 vertices, 26,140 triangles, 57 logical solids, 32 runtime parts,
18 joints, 190 accessors and 718,636 bytes. Arcanine has 19,539 vertices,
31,404 triangles, 81 logical solids, 34 runtime parts, 18 joints, 206
accessors and 874,164 bytes. The added detail remains under every existing
limit; normalized reference heights stay 0.90/1.30 and runtime primitive
counts decrease from the old 48 per species.

Round puppy eyes, focused adult eyes, fitted warm ear interiors, cream
snouts, rooted black noses and closed jaws accompany distinct forelocks,
cream ruffs and curled plumes. The guardian adds cream ankle fur and more
body/tail fur. Side, back, forehead, leg and toe markings conform to actual
stored parent triangles; no floating stripe fins are used. Four complete
paws stay fixed in battle. The rigs add body/neck/head motion, independent
ears and jaws, four supported limb fields, three monotone tail carriers and
independent main/side ruffs and forelocks. All fitted layers retain the
appropriate parent carriers.

All 17 Python anatomy/skin regressions, four Rust GPU/support/four-foot
walking checks and two shared registry/cue checks pass. Background Blender
reproduces the exact GLBs. Final software poses and six selected native
3D/2D pairs were inspected entirely offscreen and muted, with exact current
embedded payloads and stable executable identities. All selected game states
are preserved. Named GROWLITHE/SENTRET and ARCANINE/PIDGEY battles use both
models without fallback and pass authentic terrain admission at the original
0.7112/1.9050 m pack heights with zero support offset. Existing controls rotate
Arcanine's orbit view to improve the trainer/model separation; large-companion
spacing still needs world-view polish. The fire-canines guide records pins,
asset hashes, physical-size traces, capture clocks and state receipts.

Sixty-five static species files remain. Browser and moving-world visual
review remain open. Shiny checks are skipped; grass, AA and filtering are
untouched by this batch.


## Distinct small Nidoran skins: 210 to 212 registered rigs

The original female and male sculptures now use canonical `nidoran_f.glb`
and `nidoran_m.glb` standard skins. Redundant planar-ear subdivision is removed
without changing their notched outlines. Markings clip the actual stored
parent triangles; the minute male mouth contour adjustment removes quantized
slivers. Their distinct pear/barrel bodies, pinnae, horns, cheek barbs, spots,
four paws and eight claws remain. Both retain the original normalized heights
and pack physical sizes. The female has 19,256 vertices/31,768 triangles,
40 runtime primitives and 184 accessors; the male has 16,782/28,160,
36 primitives and 173 accessors. Both use 11 joints with articulated head,
ears, four supported ankle fields and two tail carriers. Vertex counts fall
about 41% from the retired static files, within unchanged runtime limits.

Claw sets and haunch markings remain separate per limb. The gait regression
proves that each support belongs to its corresponding leg's IK tree and every
complete claw set follows its actual foot. This closes a gap in floor/bounds-only
checks, which cannot detect marks or claws following another leg. The same
packing audit identified the fire-canine support-detail issue corrected in
[the fire-canines guide](fire-canines.md#growlithe-and-arcanine-per-limb-support-packing).

All 17 Python checks, four Rust GPU/support/per-leg walking checks and two
shared registry/cue checks pass. Background Blender reproduces both exact
GLBs. Final software poses and all six selected native 3D/2D pairs were
inspected offscreen and muted with exact embedded payloads and stable binary
identities. The paired states remain unchanged. NIDORAN_F/PIDGEY and
NIDORAN_M/RATTATA battles retain modeled actors without fallback and pass
authentic terrain admission at the original 0.4064/0.5080 m heights with zero
support offset. The [small-Nidoran guide](small-nidoran-sculptures.md) records
pins, asset/executable hashes, capture clocks, physical-size traces and paired
state receipts. No shiny checks, grass changes, AA or filtering work is included.

Sixty-three static species files remain. Browser and moving-world visual
acceptance remain open, as do previously recorded camera, follower-spacing
and large-legendary admission issues. The full model/rig goal remains active.

## Fire-canine walking support details stay with their own legs

Growlithe and Arcanine retain their original sculptures, materials, battle
weights, skeletons and exact decoded animation keys. Their old runtime packing
combined paw creases, leg stripes and adult ankle fur/tufts across multiple
legs, while production walking IK assigns each runtime primitive to one leg.
A new stored-GLB regression rejects that cross-leg assembly. The exporter now
packs those categories per limb, and the Rust walking regression checks every
support's actual IK tree, with complete paws and toe creases on their own
rigid foot carrier. Non-support face/body/fur/tail weights remain intact.

Growlithe now has 38 runtime primitives, 209 accessors and 713,180 GLB bytes;
Arcanine has 46, 236 and 845,220. Logical geometry, normals, source order,
materials, indices, skin weights, hierarchy, inverse binds, clip keys and
sampled battle positions/normals compare exactly with the prior assets. Vertex
and triangle counts, physical-size paths and all runtime limits are unchanged.
The [fire-canines guide](fire-canines.md) records current packed/asset hashes
and preserves the prior sculpture and capture record separately.

All 18 Python checks, four Rust GPU/support/per-leg walking checks and two
shared registry/cue checks pass. Background Blender reproduces both exact
assets. Six native 3D/2D pairs and eight selected production walking frames
were inspected entirely offscreen and muted, with exact embedded payloads and
stable executable identities. Settled pairs preserve their authoritative state;
intermediate GPU readbacks supplement the per-leg/64-phase numerical checks.
Named GROWLITHE/SENTRET and ARCANINE/SENTRET battles pass authentic terrain
admission at original 0.7112/1.9050 m pack heights with zero support offset and
no fallback. Traversal endpoint tree occlusion and trainer/sign overlap remain
visible, so broader moving-world/follower-spacing acceptance remains open.
Browser review remains open. No shiny verification, grass, AA or filtering
changes are included.

The read-only source-range audit also identified bilateral support assemblies
on Lickitung, Mr. Mime, Scizor and Scyther. Those are corrected in the following
[Bilateral walking support packing](world-support-packing.md) revision, which
also separates mantis shin armor/pads and classifies hip/greave roles. Many
other multi-source assemblies already stay within one limb and do not need
changes. The registry remains at 212 rigs, with 63 static species files and
the previously recorded broader visual/admission work still outstanding.
The full model and rig objective remains active.


## Bilateral shells, markings and mantis lower-leg supports

Lickitung's thigh markings and Mr. Mime's hip/knee shells now retain separate
runtime parts per leg. Scizor and Scyther retain separate greaves and rigid
foot pads, with toe caps/claws local to each foot. The prior packed mantis
armor joined shin and foot roles across both legs. Walking classification now
recognizes hip and greave categories. The new four-species regression proves
each support's actual IK tree, rigid foot/toe ownership, floor contact and
bounds through 64 locomotion phases. Stored-GLB regressions fail against the
previous bilateral assemblies and pass after correction.

All four logical sculptures, materials, battle weights, skeletons, inverse
binds and decoded clips remain unchanged. Independent old/new decoding also
matches sampled battle positions/normals exactly. Runtime primitive counts
are now 26/34/47/40 for Lickitung/Mr. Mime/Scizor/Scyther, within unchanged
limits. Current packed and full-asset pins, metadata and evidence are in
[the support-packing guide](world-support-packing.md); the original art guides
link to that current runtime revision.

All 44 Python checks and 31 Rust locomotion checks pass. The latter include
251 species and 26 Unown selectors for bind identity, attack/hit movement and
finite unit-scale world poses; the four-species support check supplies the
stronger 64-phase per-vertex floor/bounds and ownership coverage. Four
canonical-skin checks, four original-size GPU checks and two shared registry/
cue checks also pass, for 85 total checks. Background Blender reproduces all
four exact GLBs. Eight native 3D/2D pairs and four production walking frames
were inspected offscreen and muted with exact embedded payloads and stable
executable identity. Every pair preserves its authoritative state. Named
battles pass authentic terrain admission at original 1.1938/1.2954/1.8034/
1.4986 m heights with zero support offset and no fallback model.

The registry remains at 212 rigs and 63 static species files. Browser review,
broader moving-world/camera/follower-spacing acceptance, mouth-rim aliasing
notes and large-legendary admission remain open. No shiny checks, grass, AA,
filtering, game-content or audio changes are included. The full model and rig
objective remains active.

## Nidorina and Nidorino skins: 214 registered rigs

The original blue hornless Nidorina and purple horned Nidorino sculptures now
use canonical `nidorina.glb` and `nidorino.glb` standard skins. Redundant planar
ear subdivision is removed, markings clip actual stored parent triangles,
and Nidorino's rounded quarter sampling is reduced to 28×20. The original
silhouettes and normalized height 1.12 remain. Vertex counts fall about
66%/34% from the retired static models. Nidorina has 19,478 vertices,
32,064 triangles, 35 runtime primitives, 15 joints, 201 accessors and 808,672
bytes; Nidorino has 19,860/32,172, 39 primitives, 13 joints, 189 accessors and
748,348 bytes. All existing runtime limits remain unchanged.

Nidorina articulates head, pinnae, three-carrier forelimbs, paired haunches and
two tail carriers, with complete hind paws/nails fixed in battle and hand
claws following their wrists. Its world gait now matches the authored biped's
two planted supports. Nidorino articulates head, pinnae, jaw, horn, four
supported limbs and two tail carriers, with complete paws/nails fixed in
battle. Per-limb spot/claw packing retains their correct walking IK trees.
Both preserve identity floor roots, occupied attachments, coherent fitted
features and rotation-only, source-cue-timed idle/attack/hit clips.

All 25 Python anatomy/skin checks, four Rust original-size GPU/battle-support/
per-leg locomotion checks and two shared registry/cue checks pass, for 31
focused checks. Two recipe parity checks were repeated after moving imports
inside the test setup/parity methods to preserve borrowed-helper import cost.
Hitmonlee's imported legacy side-patch helper keeps its original behavior;
Hitmonlee and both small Nidoran exporters remain byte-identical. Background
Blender reproduces both exact GLBs and normal strict readers load 59/74
logical solids. Builder, reader and production Rust routes use the canonical
GLBs, and obsolete JSON assets are removed.

Final software comparisons/poses, six native 3D/2D pairs and two production
walking frames were inspected offscreen and muted. Exact embedded payloads
and stable executable identity bind the captures to the current assets. Every
pair preserves its authoritative state. Named NIDORINA/SENTRET and
NIDORINO/RATTATA encounters pass authentic whole-body terrain admission at
the original 0.7874/0.8890 m pack heights with zero support offset and no
fallback. [The middle-Nido guide](nido-middle-skins.md) records current pins,
asset/executable hashes, capture clocks, physical-size traces and receipts;
the species guides preserve the prior static-art record separately.

Sixty-one static species files remain. Browser and broader moving-world,
camera, follower-spacing, aliasing and large-legendary admission work remain
open. No shiny checks, grass, AA, filtering, game-content or audio changes
are included. The full model and rig objective remains active.


## Nidoqueen and Nidoking canonical skins

The active full-roster goal advances from 214 to 216 registered skins, with
59 static species files remaining. The existing royal sculptures now have
17/19-joint canonical GLBs and normal builder, strict-reader and Rust routes.
Queen geometry falls from 55,955 to 18,397 vertices; king geometry from 48,831
to 19,990, preserving their distinct rounded/ragged ears, short/large horns,
paired/single chest shields, closed/open jaws and substantial spined tails.
Both have supported arm/wrist chains, independent ears/horn, three yawing tail
carriers and complete fixed battle feet. The king has separate lower jaw and
tongue controls. All limits and real physical dimensions remain intact.

All 28 Python checks, four focused Rust checks and two shared registry/cue GPU
regressions pass (34 unique checks); background Blender reproduces both assets
exactly. Six production 3D/2D pairs and two actual world walking frames were
inspected offscreen and muted. Every final pair preserves authoritative state.
Named NIDOQUEEN/SENTRET and NIDOKING/SENTRET encounters admit complete original
1.2954/1.3970 m bodies at zero support offset without fallback, shrinking or
relaxed terrain/frustum/omission gates. Exact embedded current GLB payloads
and unchanged executable identities bind the captures to these assets.

Nidoking's close first-person camera clips its forward horn. Wider world and
battle views retain the full silhouette; first-person camera/follower distance
is an explicit unresolved issue, with full dimensions retained. Browser,
broader moving-world, camera, follower-spacing, aliasing and large-legendary
admission work remain open. [The royal-Nido guide](nido-royal-skins.md) records
pins, hashes, physical-size traces, clocks, receipts and this visual limitation.
No shiny checks, grass, AA, filtering, pack or audio changes belong to this
batch. The full model and rig goal remains active.


## Physical companion distance and first-person framing

The full-roster goal advances through a shared presentation correction; rig
count remains 216 and 59 static species remain. First-person eye height now
matches the rendered trainer's 1.25 presentation scale. Following distance
uses each unchanged model's animated forward/height/width bounds and the
pack's physical dimension, while preserving sprite/card size and grounding.
It records actual path history, checks every initial source tile, and resets
teleports against gameplay step size independently of body distance.
A fractional-gap rounding defect found during native review is fixed without
extrapolating the path; its regression fails under the old strict comparison.

Thirty-one unique checks pass: three renderer tests (including complete
original-size projection for six species over 32 gait phases), eighteen camera
tests, eight follower tests and two real-pack Surf ownership checks. A further
Surf save/reorder check is blocked before follower/app initialization by the
local Route44 source-water-table fixture guard; no guard, source data or pack
is modified to bypass it. Full gameplay/Surf verification is not claimed.

Nine current production 3D/2D pairs and three extra actual moving frames were
inspected offscreen and muted, with exact current embedded model payloads and
stable executable identity. All nine frozen pairs preserve authoritative
state. Nidoking's horn and Feraligatr's eyes now remain visible in the reviewed
ordinary first-person views, and Arcanine has clear trainer/body separation
at full size. Maximum-zoom orbit views still crop large bodies; wider views
retain silhouettes, with some foreground-foot and sign overlap still open.
[The companion guide](pokemon-following-distance.md) records commands, receipts,
identities, the Surf fixture failure and scope limits. Broader world, camera,
browser, scenery, aliasing and large-legendary admission acceptance remain open.
No shiny checks, model resizing, geometry/rig asset changes, grass, AA,
filtering, game-content, audio or pack changes belong to this batch.


## Poliwhirl, Poliwrath and Politoed

The registry now contains 219 canonical skins; 56 static species files remain.
The three generic frog prototypes are replaced by distinct authored sculptures
and twelve/twelve/fifteen-control rigs. Poliwhirl has friendly raised eyes and
padded gloves, Poliwrath has angular eyes, a heavier body and raised boxing
guard, and Politoed has its own head, cheeks, smile, web digits and curled crest.
The blue frogs punch with both hands; Politoed has independent head/crest
motion. Arc-length elbow weighting preserves the raised guard under its
stronger attack. Feet and full per-leg toe assemblies retain support ownership.

All thirteen Python checks, five focused Rust checks and two shared schema/cue
checks pass (20 unique checks). Background Blender reproduces the final assets.
Nine production 3D/2D pairs and three actual moving-world frames were inspected
inline, offscreen and muted with unchanged executable identities and exact
current embedded model payloads. All paired authoritative states are preserved.
Named battles admit complete original 0.9906/1.2954/1.0922 m bodies with zero
support offset and no fallback or relaxed gates. The raised guard keeps
Poliwrath's lower legs visible. Foliage/glove overlap and moving-world sign
proximity remain broader scene issues; browser and full-camera acceptance
remain open. [The frog guide](poli-frog-skins.md) records geometry, hashes,
commands, clocks, receipts and limitations. No shiny checks, grass, AA/filtering,
audio, pack or gameplay changes belong to this batch. The full model and rig
goal remains active.


## Sandshrew and Sandslash authoring and skin candidates

Original plated/quilled mammal sculptures and fifteen/seventeen-control skin
exporters are authored, with eleven checks passing against independently decoded
generated bytes. Both hands dig, complete feet remain fixed, every control moves
actual geometry, and posed armor roots plus all shoulder-cap vertices stay
supported. Offscreen software comparisons exposed rounded bead-like shell tiles,
continuous bands and incorrect hand-claw counts; these were corrected and the
latest neutral/rear/front/decoded-attack comparisons inspected inline.

Production builder/Rust registration, canonical asset publication, scoped native
checks and same-state production 3D/2D review remain pending. These candidates do
not yet change the 219-skin/56-static totals. The full-roster goal remains active.
[The candidate guide](sand-mammal-skins.md) records geometry, controls, test
coverage, exact hashes, remaining integration work and the offscreen-only rule.
No shiny, grass, AA/filtering, content-pack, audio or gameplay work is included.


## Sandshrew and Sandslash production integration

The two authored candidates are now canonical production skins. The registry
contains 221 skins and 54 static species files remain. The normal background
Blender builder and lazy strict readers publish/read the stored assets, while
the retired generic branch rejects accidental use. Rust registers exact
geometry pins, fifteen/seventeen controls, fixed complete battle feet and
separate complete per-foot toe assemblies for world walking.

A new world regression exposed low hand claws being rebound to a leg. The
packed category is now explicitly `Hand claw /`; exact geometry and logical
source ranges are preserved, with no shared gait heuristic change. The same
regression fails before the packing fix and passes afterward. All eleven
stored-asset Python checks, four focused parser/GPU/world checks and two shared
schema/cue checks pass (seventeen unique scoped checks).

Six same-state production 3D/2D pairs and two actual moving-world frames were
inspected inline, offscreen and muted. All paired authoritative states are
preserved, with exact current embedded asset payloads and stable executable
identities. Named battles admit complete original 0.6096/0.9906 m bodies at
zero support offset without fallback, resizing or relaxed gates. Sandshrew's
shell and short claws differ clearly from Sandslash's swept quills, severe
eyes and longer paired claws. The smaller Sandshrew remains low in the ordinary
first-person composition; broader camera, browser and world/scenery acceptance
remain open. [The canonical guide](sand-mammal-skins.md) records hashes,
commands, receipts, clocks, verification and limitations. No shiny checks,
grass, AA/filtering, audio, pack or gameplay changes belong to this batch.
The full model and rig goal remains active.


## Clefable and Wigglytuff geometry and skin preparation

The existing sculptures now fit current skin budgets. Wigglytuff falls from
28,880/48,060 vertices/triangles to 18,648/31,548 by reducing redundant eye-frame
and mouth subdivision, retaining the original broad fitted belly. Clefable's
brown ear caps now conform to actual parent triangles, fixing an exposed edge
intersection while reducing geometry to 15,603 vertices and 27,448 triangles.
Normal background Blender static exports match the current source exactly.

Eleven/seven-control standard skin candidates retain coherent fitted faces,
planted feet, independent ears/arms/forelocks and Clefable's wings/two tail
carriers. Actual stored wing sides determine their physical control ownership.
Eighteen source checks and eight independently decoded skin checks pass, and
Python runtimes produce identical candidate bytes. Offscreen software old/new,
front/rear and decoded attack views were inspected inline. Production GLB
publication, builder/Rust integration, scoped native checks and same-state
production visual review remain pending, so totals remain 221 skins and 54
static species files. [The preparation guide](fairy-pair-skins.md) records exact
metrics, hashes, test oracles and remaining gates. No shiny, grass, AA/filtering,
audio, pack or gameplay changes are included. The full-roster goal stays active.


## Clefable and Wigglytuff production skins

The optimized fairies are now canonical production skins. Normal background
Blender, direct sculpture CLI exports and strict lazy readers use GLB, with old
static JSONs retired. Rust registers exact geometry/hierarchy/pins and eleven/
seven controls. Complete feet/toenails retain battle support and separate world
support trees; fitted facial/ear layers, forelocks, wings, tails and hand details
retain their actual carriers. Totals are 223 skins and 52 static species files.

All twenty-six stored-asset Python checks, four focused parser/GPU/world checks
and two shared schema/cue checks pass (32 scoped checks). Six same-state
production 3D/2D pairs and two moving-world frames were inspected inline,
offscreen and muted, with exact embedded current GLBs and stable executable
identities. All paired authoritative states are preserved. Named battles admit
complete original 1.2954/0.9906 m bodies with zero support offset, no fallback,
no shrinking and no relaxed terrain/frustum/omission rules. Clefable's stage
search rejected unsuitable border/scenery shots before finding its valid view.

Ear/wing/tail silhouettes, eyes, mouths, belly and feet remain readable in the
scoped views. Front foliage overlaps one Clefable hand/wing, profile hides some
face/wing detail, and faint Wigglytuff belly lighting bands remain broader
presentation polish. Browser, full camera/world/material acceptance and other
remaining species are open. [The canonical skin guide](fairy-pair-skins.md)
records commands, geometry, hashes, physical sizes, receipts, clocks and limits.
No shiny checks, grass, AA/filtering, audio, content pack or gameplay changes
belong to this batch. The full model and rig goal remains active.


## Horsea, Seadra and Kingdra geometry and skin preparation

The three existing sea-creature sculptures now fit the 20k/40k skin budgets:
15,790/24,912 vertices/triangles for Horsea, 13,214/20,324 for Seadra and
19,154/31,156 for Kingdra. Actual-parent eye/breast/fin-tip surfaces, bounded
curved tessellation and smooth thin-ray normals remove redundant geometry.
The old Horsea/Seadra coil self-intersection and sinking Seadra fin-tip edges
are fixed. All tail-root caps seat within their torsos. Kingdra's rear seam
clearance is corrected without relaxing fitting thresholds.

Ten/ten/twelve-joint standard skin candidates provide independent heads, main
and head/neck fins, three tail carriers and Kingdra's paired branched crowns.
Tail weights match actual authored cross-section circles along arc length,
avoiding weight jumps to adjacent coil turns. Identity-root/body/tail yaw
preserves the authored waterline without a clamp; all snout parts retain their
head carrier and the real empty bore survives poses and a negative control.
Eighteen sculpture checks and eight independent decoded-skin checks pass.

Normal background Blender static exports match the recipes/pins, and Python
runtimes produce identical candidate skin bytes. Offscreen old/new/front/rear
and decoded-attack software views were inspected inline. Production GLB
publication, builder/Rust registration, scoped GPU/world checks and original-size
same-state native review remain pending; totals stay 223 skins and 52 static
species files. [The preparation guide](seahorse-skins.md) records exact metrics,
hashes, root/fitting oracles, evidence and remaining gates. No shiny checks,
grass, AA/filtering, audio, pack or gameplay changes are included. The full
model and rig goal remains active.

## Horsea, Seadra and Kingdra canonical skins

Production integration now replaces the three static exports with strict
canonical GLBs, bringing registration to 226 skins and 49 static species files.
Normal background builder, direct sculpture CLIs, logical readers, species
cache/dispatch and GPU/world paths use the stored ten/ten/twelve-joint skins.
All 33 scoped checks pass (26 stored Python, five focused Rust and two shared
schema/pose-cue checks). Nine current production 3D/2D pairs and three moving
frames were inspected inline, entirely offscreen and muted. Every paired state
is preserved. Named battles admit complete original 0.4064/1.1938/1.8034 m
bodies with zero support offset and no fallback, shrinking or relaxed gates.

Mouths, fitted eyes/breasts, curls, fins and crowns remain readable in scoped
views. Profile hides some detail and Kingdra's moving tail overlaps a nearby
sign. Browser, broader camera/world/material coverage and remaining species
stay open. [The skin guide](seahorse-skins.md) records hashes, commands, tests,
original sizes, paired clocks and executable identities. No shiny checks,
grass, AA/filtering, audio, pack or gameplay changes belong to this batch.
The full model and rig goal remains active.

## Goldeen, Seaking and Magikarp canonical skins

The three original fish sculptures now publish strict canonical GLBs through
the normal builder, direct CLI, logical readers and Rust/GPU/world dispatch.
Registration contains 229 skins and 46 static species files. Geometry drops
from 43,891/35,909/36,302 to 16,748/19,923/15,922 vertices while retaining every
named part, actual oral cavity and fitted marking. Six/eight/nine-joint rigs
provide coherent caudal swimming, independent pectoral/dorsal/pelvic controls
and Magikarp barbels. Shared source tail roots retain one anatomical carrier;
all world bindings match canonical weights exactly. Existing limits stay intact.

Thirty-four scoped checks pass: 19 stored Python, five focused Rust, two shared
schema/cue checks and eight datum/cache regressions. Nine same-state production
3D/2D pairs and three moving-world frames were reviewed inline, offscreen and
muted. All paired states are preserved. Named battles admit complete original
0.6096/1.2954/0.889 m longitudinal dimensions with no fallback, support offset,
shrinking or relaxed camera/ground/omission proof.

Profiling a slow capture found repeated complete source terrace-datum builds.
A bounded exact-input cache now reuses those immutable results while retaining
the original calculation for changed map/ledge/tile/permission evidence. Coastal,
waterfall, overlap-priority, camera-crop and terrain/actor-support regressions
pass. No terrain height or gameplay rule changed. [The fish skin guide](goldfish-skins.md)
records geometry, bindings, hashes, checks, physical references, captured clocks,
executable IDs and limits. Dark far-fin shadows, thin-edge aliasing, browser and
broader camera/world/material work remain open. No shiny checks, grass-height,
AA/filtering, audio, pack or gameplay changes belong to this batch. The full
model and rig goal remains active.

## Dratini, Dragonair and Dragonite canonical skins

The dragon family now uses seven/seven/fifteen-joint canonical skins through
the normal builder, strict logical readers, Rust dispatch and direct sculpture
CLI. Registration reaches 232 skin variants with 43 static species files.
Continuous coils and ivory ribbons share actual spine-ring bindings. Dragonite's
arm bindings follow the source tube rings, retaining stronger forearm attacks
within the existing strain limit. Fixed battle contacts, coherent wings,
complete claws, original dimensions and all standard limits remain intact.

Twenty-eight scoped checks pass: 21 stored Python, five focused Rust and two
shared schema/cue checks. Nine same-state native pairs and three moving-world
frames were reviewed inline, headless and muted. All three battles admit full
original dimensions without fallback or relaxed scene gates. Dragonite's world
review uses a wider normal zoom to include its tail. Broader follower/scenery
overlap, lighting/AA, browser and camera coverage remain open. The
[dragon skin guide](dragon-family-skins.md) records geometry, bindings, checks,
physical references, captured clocks and executable identities. No shiny,
grass-height, AA/filtering, pack, audio or gameplay work is included. The full
model and rig goal remains active.

## Aerodactyl canonical skin

The detailed fossil flyer now uses nine canonical controls for its ground root,
body, skull, mandible, wings, hind limbs and arrow tail. Every source vertex and
triangle remains unchanged. Same-carrier packing retains all 56 logical parts
in 27 runtime parts within all existing limits. Production builder, direct CLI,
strict reader and Rust dispatch use the same GLB. Registration reaches 233 skin
variants and 42 static species files; full-roster improvement remains active.

Twenty-five scoped checks pass: 20 stored Python, three focused Rust and two
shared schema/cue checks. The recoil gasp preserves the source jaw clearance;
upper/lower teeth, palate and tongue follow their actual jaws. Soles stay rooted
in battle, while existing world Fly mode and canonical bindings remain intact.
Three current same-state native pairs and one moving-world frame were inspected
inline, headless and muted. The named battle admits the complete original
1.8034 m body without fallback or relaxed scene gates. The
[Aerodactyl skin guide](aerodactyl-skin.md) records geometry, controls, checks,
physical references, captured clocks, executable IDs and limits. Tree occlusion,
thin-edge aliasing and broader lighting, browser and camera coverage remain
open. No shiny checks, grass-height, AA/filtering, pack, audio or gameplay
changes belong to this batch.

## Omanyte and Omastar canonical skins

The nautilus pair now uses fifteen/nineteen-joint canonical skins through the
normal background builder, direct CLIs, strict readers and Rust dispatch.
Registration reaches 235 skin variants and 40 static species files. Ten actual
root-cap hinges animate each tentacle independently; shell ribs/spirals/spines,
coherent gaze and the adult's four open beak quadrants retain their carriers.
A pre-existing reversed polar-cap triangle on each ribbed shell is repaired
with ear clipping, preserving shape and counts. All standard limits remain.

World crawling reuses canonical hinges and exact weights, composing its strokes
with the authored clock while retaining floor contact. Thirty-seven scoped
checks pass: 31 stored Python, four focused Rust and two shared schema/cue
checks. Six same-state production pairs and two moving-world frames were
inspected inline, headless and muted. Both battles admit complete original
0.4064/0.9906 m bodies without fallback or relaxed scene gates. The
[nautilus skin guide](nautilus-skins.md) records source corrections, anatomy,
controls, checks, physical references, captured clocks, executable identities
and limits. Profile occlusion, thin-line aliasing, broader lighting, browser and
camera coverage remain open. No shiny checks, grass-height, AA/filtering,
audio, pack or gameplay work is included. Full-roster improvement remains active.

## Graveler and Golem canonical skins

The stone pair now uses ten/twelve-joint canonical skins through the normal
background builder, direct CLIs, strict readers and Rust dispatch. Registration
reaches 237 skin variants and 38 static species files. Graveler retains four
independent arm/wrist chains and its coherent fitted rock face. Golem retains
all 32 polygon plates, an actual open jaw and complete claws while modest palm,
foot-pad and recessed-chamber reductions bring it within the standard vertex
budget. Same-carrier packing preserves all logical parts; runtime group names
are unique and bounded. All existing limits remain intact.

Thirty-eight scoped checks pass: 32 stored Python, four focused Rust and two
shared schema/cue checks. Fixed battle supports, occupied limb/digit/jaw roots,
real empty mouth space, plate/face support and complete two-support walking
pass. Six current same-state native pairs and two moving-world frames were
inspected inline, headless and muted. Both named battles admit complete original
0.9906/1.397 m bodies without fallback or relaxed scene gates. The
[stone-pair skin guide](stone-pair-skins.md) records geometry, controls, checks,
physical references, captured clocks, executable identities and limits.
Profile/foliage occlusion, close scenery placement, deep shadows, thin-edge
aliasing and broader browser/camera/material coverage remain open. No shiny
checks, grass-height, AA/filtering, audio, pack or gameplay work is included.
Full-roster model and rig improvement remains active.


## Charmeleon and Charizard canonical skins

The fire-dragon evolutions now use original shaped skulls, real open jaws,
fitted bibs and angular gazes, complete digits, curved burning tails and
thirteen/fifteen-joint canonical skins. Charmeleon has one swept red crest;
Charizard has twin orange horns and broad scalloped teal bat wings. Registration
reaches 239 skin variants and 36 static species files. Standard geometry,
packing, physical-size and admission limits remain intact.

Twenty-six scoped checks pass: 20 stored Python, four focused Rust and two
shared schema/cue checks. Six current production 3D/2D pairs and two moving-world
frames were inspected inline, headless and muted. Both battles admit complete
original 1.0922/1.7018 m bodies without fallback. Charmeleon keeps two grounded
world supports; Charizard preserves authored flight. The
[fire-dragon skin guide](fire-dragon-skins.md) records anatomy, controls, checks,
physical references, capture clocks, binary identities and limits. Charizard's
moving tail partly crosses foliage; scenery/sign placement, profile occlusion,
strong shadows, thin-edge aliasing and broader browser/camera/material coverage
remain open. No shiny checks, grass-height, AA/filtering, audio, pack or gameplay
work belongs to this batch. Full-roster model and rig improvement remains active.


## Quilava and Typhlosion canonical skins

The fire-starter evolutions now use original low/upright badger sculptures with
shared-surface cream/midnight coats, shaped muzzles, actual fitted red gaze,
complete toes and thirteen/seventeen-joint canonical skins. Quilava keeps four
short supports and separate six-blade neck/rump fire clusters; Typhlosion has
complete forepaws, an actual open jaw and seven independently moving shoulder
fire blades. Registration reaches 241 skin variants and 34 static species files.
Standard geometry, packing, physical-size and admission limits remain intact.

Twenty-seven scoped checks pass: 21 stored Python, four focused Rust and two
shared schema/cue checks. Six final same-state production pairs and two
moving-world frames were inspected inline, headless and muted. Both battles
admit complete original 0.889/1.7018 m bodies without fallback. Four/two complete
walking support chains preserve upper bindings and stay inside full bounds.
An initial native review exposed lateral eyes hidden from the front; refitting
both gazes around the forward brow fixes the blank face and adds a real
skull-occlusion regression. The [fire-badger skin guide](fire-badger-skins.md)
records anatomy, controls, checks, physical references, captured clocks,
executable identities and limits. Small details, profile occlusion, strong
mandible shadows, foliage overlap and broader browser/camera/material coverage
remain open. No shiny checks, grass-height, AA/filtering, audio, pack or gameplay
work belongs to this batch. Full-roster model and rig improvement remains active.


## Tyranitar canonical skin

Tyranitar now has an original armored sculpture and fourteen-joint canonical
skin through the normal background builder, direct CLI, strict reader and Rust
dispatch. A shaped skull, real open toothed jaw, fitted red gaze, segmented
belly shield, actual recessed armor marks, complete claws, rooted crown/cheek/
dorsal spines and a heavy curved tail replace the old primitive assembly.
All 70 logical parts survive coherent packing into 34 runtime parts. Standard
geometry, storage, physical-size and admission limits remain intact.
Registration reaches 242 skin variants and 33 static species files.

Twenty-six scoped checks pass: 21 stored Python, three focused Rust and two
shared schema/cue checks. Three final same-state production pairs and one
moving-world frame were inspected inline, headless and muted. The battle admits
complete original 2.0066 m Tyranitar without fallback. Two complete world
supports preserve upper armor and tail bindings, stay inside full bounds and
retain ground contact. The [Tyranitar skin guide](tyranitar-skin.md) records
anatomy, controls, checks, physical reference, captured clocks, binary identities
and limits. Strong jaw shadows, far-limb/spine occlusion, sign/foliage placement
and broader browser/camera/material coverage remain open. No shiny checks,
grass-height, AA/filtering, audio, pack or gameplay work belongs to this batch.
Full-roster model and rig improvement remains active.


## Larvitar and Pupitar canonical skins

The pre-evolutions now use original local-sprite-guided sculptures and
11/four-joint canonical skins through the normal background builder, direct
CLIs, strict readers and Rust dispatch. Larvitar retains one tall horn, black
cheek tears, a red diamond, complete blunt digits and a three-pronged tail;
the old adult-style dorsal spikes are removed. Pupitar has a faceted visor,
larger angular gaze, three crown points, six side points, front/back shell
seams and a grounded lower fold. Coherent packing restores all 45/28 logical
parts from 20/nine runtime parts. All standard limits remain intact.
Registration reaches 244 skin variants and 31 static species files.

Twenty-seven scoped checks pass: 21 stored Python, four focused Rust and two
shared schema/cue checks. Six final same-state production pairs and two moving
frames were inspected inline, headless and muted. Both battles admit complete
original 0.6096/1.1938 m models without fallback or relaxed gates. Larvitar keeps
two complete ground supports; Pupitar remains legless, retains its canonical
bindings and keeps lower-fold contact without adding hover. The
[rock-larva/pupa skin guide](rock-pupa-skins.md) records anatomy, controls,
checks, physical references, capture clocks, binary identities and limits.
Small-scale detail, profile occlusion, green-body/grass contrast and broader
browser/camera/material coverage remain open. No shiny checks, grass-height,
AA/filtering, audio, pack or gameplay work belongs to this batch. Full-roster
model and rig improvement remains active.

## Electrode coherent shell and expression

Electrode now shares the existing orb exporter with Voltorb and uses a
canonical two-joint skin. Its normal red/white/ink palette matches the local
pack. Angled thicker brows and longer centred pupils refine the fitted grin.
The shared equator and complete face move together above a fixed support ring;
legless world locomotion preserves canonical weights. Voltorb's GLB is unchanged.
Electrode retains 17,210 vertices, 23,480 triangles and ten parts, with 48
accessors / 675,972 bytes under all original limits. The registry reaches
245 skin variants and 30 static species files.

Twenty-three scoped checks pass: 18 Python, three focused Rust and two shared
schema/cue checks. Three headless/muted same-state production pairs and a
moving world frame were inspected inline. The battle admits the complete
original 1.1938 m model with uniform scale and no fallback; HUD labels identify
both Pokémon. High-view grin foreshortening, side occlusion, subtle tooth seams
and broader browser/camera/material coverage remain open. The
[Electrode skin guide](electrode-skin.md) records commands, controls, source and
binary hashes, checks, capture clocks and limits. No shiny, grass, AA, audio,
pack or gameplay changes belong to this batch. Full-roster work remains active.

## Kabuto and Kabutops fossil anatomy and skins

The two old primitive assemblies are replaced by original domed/recessed
Kabuto and swept-head, ribbed, paired-scythe Kabutops sculptures. Canonical
six/thirteen-joint GLBs retain complete fitted faces, rooted claws/limbs,
scythes, crown blades, spurs, talons and tail. Four/two complete world support
chains preserve ground contact; upper-body bindings remain canonical. Coherent
packing restores 23/42 source parts from 20/27 runtime parts. Geometry is
5,970 / 9,130 vertices and 9,356 / 14,756 triangles, with all original limits
intact. Registration reaches 247 skin variants and 28 static species files.

Twenty-three scoped checks pass: 17 stored Python, four focused Rust and two
shared schema/cue checks. Six headless/muted same-state production pairs and
two moving world frames were inspected inline. Both battles admit complete
original 0.5080 / 1.2954 m models with uniform scale and no fallback; HUDs name
both Pokémon. The initial scythe self-intersection was corrected by changing
the curve and root width. The [fossil skin guide](fossil-scythe-skins.md) records
commands, anatomy, controls, hashes, checks, capture clocks and visual limits.
High-view gaze occlusion, side foreshortening, foliage overlap and broader
browser/camera/material coverage remain open. No shiny, grass-height, AA,
audio, pack or gameplay changes belong to this batch. Full-roster work remains
active.

## Pineco and Forretress coherent mechanical armor

Both detailed neutral sculptures retain exact f32 geometry, normals, colors
and triangle order while gaining canonical seven-joint skins. Pineco has
independent plate whorls, crown and stem above a fixed core tip; Forretress has
a fixed lower shell, hinged upper shell and four articulated hollow ports.
Coherent packing reduces 54/48 source parts to 13/18 runtime parts under all
original limits. Registration reaches 249 skin variants and 26 static species
files.

Twenty-four scoped checks pass: 18 stored Python, four focused Rust and two
shared schema/cue checks. Six headless/muted same-state production pairs and
two moving world frames were inspected inline. Both battles admit complete
original 0.6096/1.1938 m models with uniform scale and no fallback. The
[armored bug skin guide](armored-bug-skins.md) records commands, hierarchy,
geometry preservation, checks, hashes, capture clocks and visual limits.
Physical contacts stay fixed, but weak contact shadows can still make the
legless silhouettes look suspended, especially Pineco's narrow core tip.
Grounding readability remains open before full visual acceptance, alongside
broader browser/camera/material coverage. No shiny, grass-height, AA, audio,
pack or gameplay changes belong to this batch. Full-roster work remains active.

## Lower-body battle grounding

A faint additional ground-occlusion layer now complements the existing tight
support shadow for grounded modeled battlers. Its footprint clips the neutral
lower 30% of the sculpture, uses exact triangle intersections, excludes high
plumes, and stays within that horizontal extent. Tight contact, floating
footprints, original dimensions, physical placement and admission/terrain gates
retain their contracts. The change improves the floor cue beneath Pineco's
armor while retaining Sentret's tail support.

Eight shadow regressions pass; one unrelated dense-forest benchmark remains
ignored. Three final same-state production battle pairs for Pineco, Forretress
and Sentret were inspected inline, headless and muted, with unchanged binaries
and no fallback. The [battle grounding guide](battle-grounding.md) records the
algorithm, validation, scope and capture identities. World/browser/hardware-
shadow coverage and broader roster fidelity remain open. No shiny, grass, AA,
audio, pack or gameplay work belongs to this change. Static roster count stays
26 and the full model/rig goal remains active.

## Bellsprout family botanical skins

Bellsprout, Weepinbell and Victreebel now have canonical 10/9/10-joint skins.
Fitted facial details, occupied mouth floors, rooted hood/tendrils and
ear-clipped concave leaf caps improve the original sculptures. Common-parent
leaf attachment weights survive production welding, while separate leaf tips,
mouths, stem and tendrils remain articulated. Bellsprout retains two real root
support chains; the legless evolutions retain fixed body tips and no invented
limbs. Original dimensions, full motion bounds and admission limits hold.

Twenty-five scoped checks pass: 18 stored Python, five focused Rust and two
shared schema/cue checks. The [botanical skin guide](pitcher-skins.md) records
anatomy, metrics, rebuild commands, hashes and review limits. Registration
reaches 252 skin variants and 23 static species files. Oddish's separate use
of the legacy shared leaf cap is corrected in the follow-up below. Full-roster
model and rig improvement remains active.

## Oddish concave leaf-cap correction

The shared botanical audit confirmed cap/side-wall crossings in all five
Oddish blades. Ear-clipped true-section caps replace the old concave vertex
fans, retaining every boundary position, dimension, joint node, material,
weight and animation key. Every non-blade part remains exact. The corrected
canonical GLB has 8,845 vertices, 14,620 triangles, 19 parts and 12 joints;
the strict source digest is updated without relaxing parsing limits.

Thirteen distinct scoped checks pass: ten stored Python, two focused Rust
and the shared anatomy/order/physical-size schema check. Three production
2D/3D pairs and one moving world frame were inspected inline, entirely
offscreen and muted. Original 0.508 m battle admission has no fallback and
keeps physical support at zero. The [Oddish guide](oddish-sculpture.md#concave-cap-correction)
records preservation, negative controls, metrics, clocks, binary identities
and visual limits. This closes the specific legacy-cap audit item. Registry
counts stay 252 skin variants and 23 static species files; full-roster visual
and rig improvement remains active.

## Gloom and Vileplume botanical sculptures and skins

The old generic assemblies are replaced by a sleepy four-lobed spotted Gloom
bud with rooted drool and a broad five-petal Vileplume crown with fitted
mottling, pale annular throat and actual recessed nectar well. Canonical
11/12-joint skins preserve coherent faces, crown details and independently
hinged sepals/petals. World gait retains two complete supports and both arms,
with upper bindings preserved. Original authored/pack dimensions, full motion
bounds, floor contacts and model-admission limits hold. Coherent packing
reduces 30/34 logical parts to 18/23 runtime parts.

Sixteen final Python checks and four final focused Rust checks pass. Two
shared schema/cue checks pass before the tip-only refinement; exact joint
nodes, clip keys and counts remain unchanged by that correction. The
[botanical crown guide](rafflesia-skins.md) records anatomy, checks, source and
packed hashes, rebuild commands and visual limits. Registry count reaches
254 skin variants and 21 static species files. Full-roster model and rig
improvement remains active.

Six final headless/muted same-state pairs and two moving world frames were
inspected inline. Gloom retains its byte-identical capture group; corrected
Vileplume has a new group with stable binaries and no fallback. Rounded petal
ends replace visible flat cuts. The final default Vileplume world view remains
substantially obstructed by foliage, with the moving frame exposing it through
existing dither fading. World visibility and broader visual coverage remain
open; no full visual acceptance is claimed. No shiny, grass-height, AA, audio,
pack or gameplay changes belong to this batch.

## Raichu curled ears and electric-mouse walking supports

Raichu's primitive static assembly is replaced with a cream-bellied orange
sculpture, fitted facial details, curled brown/gold ears, planted brown paws
and a thin curved whip ending in a golden bolt. Its thirteen-control skin
preserves coherent face, hand, ear-lining and tail carriers. Structured ear
grids resolve folds and sinking found in the initial concave plate prototype.
World support classification now uses bilateral anatomical evidence when an
asymmetric tail places the full bound center outside both legs. This fixes
Raichu and the unchanged Pikachu asset while preserving valid existing axes,
source bounds, placement and physical dimensions.

Twenty-six scoped checks pass. Six headless/muted same-state production pairs
were inspected inline with stable binaries and no battle fallback. Both world
camera directions remain partly obstructed by foliage, so world visibility
acceptance stays open. The [Raichu guide](raichu-skin.md) records anatomy,
hashes, checks, capture clocks and limitations. Registry count reaches 255 skin
variants and 20 static species files. Full-roster improvement remains active.

## Ekans continuous grounded coil and skin

The generic spherical-headed serpent and four neck rings are replaced by an
original purple coil with a tapered muzzle, fitted golden slit gaze, one broad
surface-following collar, continuous ventral strip and raised segmented tail.
Five controls articulate the neck, head and tail while retaining the low coil
on the floor root. An initial crossed neck bend was corrected in geometry;
neutral and posed intersection checks pass. Original authoring height,
longitudinal size calibration, uniform scale and full animated bounds hold.

Sixteen Python checks, three focused Rust checks and the shared schema check
pass. The [Ekans guide](ekans-skin.md) records anatomy, metrics, exact hashes,
rebuild commands and review status. Registry count reaches 256 skin variants
and 19 static species files. Full-roster model and rig improvement remains active.

Three headless/muted same-state production pairs and two moving frames were
inspected inline. Front and battle views are clear, with both battle HUD names
and complete models without fallback. The world view remains partly obscured
by foreground branches. A standing-height-only helper postcheck was corrected
to honor the existing Ekans length calibration; fresh saved battle outputs and
unchanged binary identities were audited without rerendering. The guide records
that recovery and its native clock. No shiny, grass-height, AA, audio, pack or
gameplay changes belong to this batch.

## Arbok rounded threat hood and articulated fanged jaw

The generic spherical-headed cobra and flat hood are replaced by an original
rounded mantle with fitted dark/ivory/red false eyes and red threat chevron,
angular narrow-eyed upper head, real mouth opening, rooted curved fangs,
dark throat, pink tongue and independently moving lower jaw. Eight controls
retain the low coil on its floor root and articulate the neck, head, raised
tail, unmarked outer hood folds and mandible. The complete fitted mask shares
the central chest carrier. Hood geometry now lies across the front of the
neck, preserving the visible central marking. Local outer-fold pivots and
restrained motion resolve measured stretching without relaxing limits.

Twenty-one final Python checks and three final focused Rust checks pass. The
[Arbok guide](arbok-skin.md) records anatomy, exact source/packed hashes,
rebuild commands and review status. Original dimensions, longitudinal size
calibration, uniform scale, full bounds and admission gates remain intact;
Ekans stays byte-identical. Registry reaches 257 skin variants and 18 static
species files. Full-roster model and rig improvement remains active.

Production front review prompted sharper peaked/notched dark brows, angular
pale false eyes, a dark-bordered red chevron and lower real-eye placement.
Geometry checks require real brow notches and coherent complete mask carriers.
Joint nodes, inverse binds and every clip key remain byte-identical. The final
shared schema check also passes, making 25 scoped checks overall. Three refined
headless/muted same-state production pairs and two moving frames were inspected
inline with stable binaries, complete models without fallback and readable
battle HUD names. Front and battle views are clear; stationary world foliage
still obscures much of the animal. Broader visibility and palette parity remain
open. No shiny, grass-height, AA, audio, pack or gameplay changes belong to this
batch.

## Steelix exact-neutral steel-chain and jaw skin

The original angular steel sculpture now has ten controls for a grounded root,
seven rising-chain segments, head and independently hinged jaw. Three low
boulders and the tail stay pinned; rigid armor and spikes retain their carriers,
buried joints blend along actual center axes, and complete face/upper/lower
teeth follow the appropriate skull or jaw. Every original neutral f32 position,
normal, color and triangle remains unchanged. Coherent packing retains 48
logical ranges in 32 runtime parts. Original physical length calibration,
uniform scale, full bounds and admission gates remain intact.

Twenty-two Python checks, three focused Rust checks, two shared schema/cue
checks and all 24 fresh layout checks pass, including the earlier unresolved
Steelix length fixture; a production follower fixture regression brings the
total to 52 scoped checks. The [Steelix guide](steelix-sculpture.md)
records hierarchy, preservation, measurements, hashes, commands and review
status. Headless, muted front/world/battle pairs were inspected inline with
unchanged binaries and preserved authoritative state. Front and battle show
Steelix clearly; the wider world framing still clips the giant at the left
edge. The default short Route36 trail and obstructed Route29 battle fixture
are explicitly rejected as visual evidence. Registry reaches 258 skin variants
and 17 static species files.
Full-roster model and rig improvement remains active.


## Dugtrio independent capsule skin

Three continuous rounded emerging capsules replace the static ellipsoid
assembly, with distinct heights, fitted narrow black eyes and highlights,
rooted oval noses and a shared bed of varied faceted soil. Four controls keep
the soil and buried bases fixed while each mole articulates independently.
Staggered source-cue attack/hit clips and phased idle preserve coherent faces,
original physical height, uniform scale and full animated bounds.

Eleven Python checks, three focused Rust checks and two shared schema/cue
regressions pass: 16 scoped checks. The [Dugtrio guide](dugtrio-skin.md) records
anatomy, ownership, measurements, hashes, commands and production review
status. Three same-state production pairs and two moving frames were inspected
inline, entirely headless and muted, with exact current model data and unchanged
binaries. Front and battle are clear with grounded soil and readable species
HUD names; the stationary world view still has foreground foliage obstruction.
Registry reaches 259 skin variants and 16 static species files.
Full-roster model and rig improvement remains active.

## Tentacool and Tentacruel grounded tentacle skins

Continuous rounded bells, embedded red lobes, fitted mantle eyes and blue
curved tendrils replace both static assemblies. The source uses the normal
local-pack palette and original physical heights. Six/24 controls articulate
Tentacool's two tendrils and Tentacruel's ten visible tendrils plus side feelers;
distal contacts remain on the fixed floor root. Source-cue clips retain
coherent bell faces and highlights.

These tendril-supported bodies no longer receive the generic world/battle
hover lift or world minimum hover clearance. Their swimming classification
stays intact. Fifteen Python, four focused Rust, ten related unaffected and
two shared schema/cue checks pass: 31 scoped checks. The
[tentacle guide](tentacle-skins.md) records anatomy, palette, ownership,
measurements, hashes, commands and production review status. Registry reaches
261 skin variants and 14 static species files. Six same-state production pairs
and two moving frames were inspected inline, headless and muted. Both models
show readable real gaze and grounded tendrils in city and battle views, with
readable battle species names, no fallback and unchanged binaries. Early close
views exposed hidden upper gaze; advancing the mantle, raising the eyes and
widening pupils fixed readability while preserving every joint, inverse bind,
clip key, bell, lobe, arm and tendril. Full-roster work remains active.


## Jynx grounded gown and independent five-digit hands

The detailed adult dancer recipe is retopologized from 48,266 vertices and
79,520 triangles to 15,311/24,740, retaining shaped hair and hands while
improving the folded gown, enlarged gaze and broad recessed kiss rim. Its
normal local-pack palette and original 1.397 m height are retained. Twenty-two
controls cover the pinned hem, torso/neck/head, two hair curtains, both
shoulder/elbow/wrist chains and all ten fingers. Coherent packing preserves
45 logical anatomy ranges in 35 runtime parts. Smoochum remains byte-identical.

Twenty Python, three focused Rust and two shared schema/cue checks pass:
25 scoped checks. The [Jynx guide](jynx-skin.md) records anatomy, packing,
measurements, hashes, commands and production review status. Registry reaches
262 skin variants and 13 static species files. Three same-state production
pairs and a moving world frame were inspected inline, entirely headless and
muted, with exact current model data and unchanged binaries. Close/world/
battle views show the complete model and grounded hem; gaze, kiss aperture,
hair, hands and battle species names are readable. No fallback is used.
Full-roster work remains active.


## Mankey and Primeape rooted fur and articulated fists

Both static assemblies now use continuous pelts, rooted fur locks, fitted
angry gaze and pig nostrils, bent limbs, formed clenched fists and three-toed
paws. Mankey has a long articulated curl tail; Primeape has a wider pelt,
denser fur, larger fists and dark wrist/ankle bands. Twelve/ten controls
preserve coherent faces and fists, fixed battle paws, and two complete
production walking support chains. Original physical heights, normal local-
pack palettes, full animated bounds and admission rules remain intact.

Nineteen Python, three focused Rust and two shared schema/cue checks pass:
24 scoped checks. The [pig-monkey guide](pig-monkey-skins.md) records anatomy,
rig ownership, measurements, hashes, commands and production review status.
Registry reaches 264 skin variants and 11 static species files. Six same-state
production pairs and both moving world frames were inspected inline, entirely
headless, offscreen and muted. Original-size models and paws are grounded;
battle views show distinct anatomy and actual species names without fallback.
World facial detail remains small at the wide camera distance. Binary identities
and canonical GLB hashes remained unchanged throughout the review. Full-roster
model and rig improvement remains active.


## Meowth and Persian feline bodies, coherent faces and curled tails

Both static assemblies are replaced by original continuous feline bodies,
fitted gaze, rooted whiskers, formed paws and curled tails. Meowth retains a
large engraved coin, pointed dark ears, broad white eyes and two beckoning
palms; Persian has a lean long quadruped trunk, flowing throat, narrower gaze,
forehead jewel and four complete supports. Fifteen/thirteen controls keep
faces and charms coherent, articulate ears and three-control tails, and pin
battle paws. Original physical dimensions, normal local-pack palettes,
animated bounds and admission rules are preserved.

Nineteen Python, three focused Rust and two shared schema/cue checks pass:
24 scoped checks. The [feline guide](feline-skins.md)
records anatomy, ownership, measurements, hashes, verified exports and review
status. Six same-state production pairs and both walking world frames were
inspected inline, entirely headless, offscreen and muted. Complete original-size
models stay grounded; battle views show readable anatomy and actual species
names without fallback. Facial detail is small in wide world views. Binary
identities and canonical GLB hashes remained unchanged throughout the review.
Registry reaches 266 skin variants and nine static species files. Full-roster
model and rig improvement remains active.


## Doduo and Dodrio independent necks and hinged bills

The detailed runner sculpts now fit production limits while retaining all
37/61 logical anatomy ranges. Closed eyes follow the actual stored head
surfaces; normal local-pack palettes and wider Dodrio pupils improve fidelity
and readability. Twelve/nineteen controls articulate separate neck-base/tip/
head/jaw chains, two supporting shins and Dodrio's three fan plumes. Head gaze
and crests remain coherent; lower bills hinge along authored head yaw axes.
All battle toes and claws stay planted, while world motion uses two complete
walking supports. Original physical heights and full animated bounds remain.

Twenty Python, three focused Rust and two shared schema/cue checks pass:
25 scoped checks. The [runner-bird guide](dodo-sculptures.md)
records anatomy, retopology, ownership, measurements, hashes, exports and review
status. Six same-state production pairs and both walking frames were inspected
inline, entirely headless, offscreen and muted. Models and claws stay grounded;
battle views separate the heads and show actual species names without fallback.
Heads overlap in wide side views; Doduo's raised bill underside is very dark in
the fixed front lighting and clearer in battle. Current binary identities and
canonical GLB hashes remained unchanged across the captures.
Registry reaches 268 skin variants and seven static species files. Pidgeotto
also retains its earlier wing-node GLB and still needs a standard skin/cue
upgrade. Full-roster model and rig improvement remains active.


## Pidgeotto full-body production migration

Pidgeotto now uses the shared standard skin and cue path described above.
At that migration the registry contained 269 standard variants and seven
static species files. The later Pidgeot/Fearow migration below updates that count.
All 251 species retain model paths. Registry coverage alone does not establish
full-roster visual quality; the broader model and rig improvement remains active.


## Pidgeot and Fearow standard-skin production migration

The [raptor-bird skin guide](raptor-bird-skins.md) records canonical 20/19-control
skins with coherent heads and bills, two-stage necks, separate wing/wrist banks,
five tail feathers, crest/crown/mane and planted clawed feet. Adaptive fitted
markings and reduced interior feather subdivisions bring both sculptures under
existing loader limits while preserving all anatomy names, sampled outlines
and exact original neutral bounds. The shared production renderer, source
readers, STL exporter and background builder use the single GLB for each bird.
Authored flight is retained without changing the world renderer’s threshold.

Fourteen canonical anatomy, twelve decoded-skin/API and five scoped Rust checks
pass: 31 checks. The background builder reproduces canonical bytes. Six native
same-state 2D/3D pairs and both moving world frames were inspected inline,
entirely headless, offscreen and muted. Battle HUDs identify PIDGEOT/PIDGEY and
FEAROW/PIDGEY; complete wings and planted claws fit at original pack dimensions.
Wide profile banks overlap and reduce facial detail; the sampled world position
has existing occlusion dithering and NPC overlap. Fearow’s bill foreshortens
head-on and remains clear in profile and battle. Existing terrain admission
rejected early Fearow stages before finding a valid Route 29 composition;
size, FOV, bounds and terrain guards remain unchanged. Independent asset and
binary identity audits pass across all six captures.

The registry now contains 271 standard variants and five static species:
Exeggcute, Exeggutor, Farfetch’d, Ivysaur and Venusaur. All 251 species retain
model paths. Full-roster model and rig improvement remains active.
