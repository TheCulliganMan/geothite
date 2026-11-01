# Psyduck and Golduck standard skins

Both ducks use their original neutral recipes in `tools/duck_sculpt.py` and
one canonical GLB each. `tools/duck_glb.py` writes standard skins; the retired
JSON copies are gone. Original f32 geometry, normals, part names and triangle order remain
unchanged. Primary body colours now match the verified normal pack palettes;
bill, webbing, eye, hair and ruby accent materials are preserved. The authoring CLI and battle-source pipeline
publish GLB and remove obsolete JSON outputs.

| Species | Parts | Vertices | Triangles | Bytes | Joints | Accessors | Packed height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Psyduck | 28 | 3,754 | 3,828 | 184,140 | 10 | 131 | 0.7874 m (2′7″) |
| Golduck | 45 | 4,974 | 4,460 | 235,196 | 11 | 186 | 1.7018 m (5′7″) |

Both neutral silhouettes are 1.03 authored units tall. The existing bounded
reader limits are unchanged. Identical skin/index accessors share storage
without merging anatomical parts.

Neutral digests:

- Psyduck: `4d28e9c51e3d5ed94fcccfe356d5cc60dc9ee2d7645332334c7bb7d6ef5b9b4c`
- Golduck: `e7ab59e4de9d3d78f761557cc5b62d6e41442f54c3e25b7fffbecba18892178d`

Psyduck keeps its egg body, confused broad bill, three black hair strands and
hands at its temples. Golduck keeps its slimmer swimmer, four swept crown
fins, forehead ruby, webbed hands and long tail. A spatial torso/head field
articulates the fused body while keeping fitted eyes, nostrils, upper bills and
ruby coherent on the head. Lower bill and seam share a jaw joint. Palms,
fingers, web membranes and hand claws share hand owners. All legs, webbed feet,
foot membranes and foot claws remain planted on the floor root.

Psyduck's shoulder branches follow its head to retain the temple-hand pose;
Golduck's branches follow its torso. Both articulate arms, hands, lower bill
and hair/crown. Psyduck has a single short-tail hinge; Golduck has a blended
two-joint tail. Idle is 3.6 seconds with 73 keys; attack/hit are one second with
65 keys. The torso/head brace early, while wrist, crest and tail response
settles later. All clips are rotation-only and source battle cues own timing.

Eleven stored-anatomy and deformation checks pass for both ducks, including
connected closure, shading, fitted facial surfaces, true webbing, inverse
binds, coherent ownership, planted feet, floor clearance, winding and strain.
The focused canonical-routing check passes. Pure authoring exports were
executed into ignored temporary output, compared byte for byte to canonical
assets, and verified to remove stale JSON. The full Blender batch integration
was syntax checked, not run.

Both Rust integration checks pass with the corrected pack heights. Broader
regressions pass: 32 active rig tests (one existing audit ignored), 28 skinning
tests and 13 actor/model tests. Fresh native paired 3D/2D previews were inspected
for both battles and Psyduck in Mr. Fuji's house; all three preserved state.

| Pair | Hash | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: |
| Psyduck battle | 3782508079 | 34 | 0.78077375 |
| Golduck battle | 1039952895 | 35 | 0.739003 |
| Psyduck world | 964474920 | 35 | 0.764338999 |

Psyduck's broad bill, hair and temple palms stay readable at its smaller packed
scale. Golduck's crown, forehead ruby, long tail and clawed webbing stay distinct
at its taller scale. Neutral world placement keeps Mr. Fuji's named Psyduck in
its original authoritative slot. The battle comparisons use the source back
sprites and the production 3D camera. Primary body colours were corrected from the pack’s normal RGB555 anchors:
Psyduck `(31, 31, 0)` becomes RGB8 `(255, 255, 0)`; Golduck `(8, 11, 24)`
becomes `(66, 90, 198)`, using the runtime’s bit replication. Fresh paired
battle previews now read yellow and indigo. Mr. Fuji’s Psyduck also reads yellow
in its unchanged world slot. Lighting still shades those base colours.

An independent old/new GLB comparison confirmed identical positions, normals,
indices, joint assignments, weights, hierarchy, inverse binds and animation
clips. The added stored-material regression checks the RGB555 conversion and
preserved bill, ruby and hair accents. All 12 Python tests pass. Browser
rendering and detailed source-cue visual motion remain open; these previews
verify the colour correction, not final all-species visual acceptance.

```sh
python3 tools/duck_glb.py
python3 -m unittest discover -s tools -p 'test_duck*.py' -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view duck --lib
```


Verified production build and paired captures:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked -p crystal-bevy \
--features location-tester --example immersive_battle_3d --example new_bark_3d
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
./target/pokemon-models/debug/examples/immersive_battle_3d \
content-packs/core-modular.browser.crystalpack --wild-species PSYDUCK --size 1000x700 \
--screenshot target/pokemon-native-review/psyduck-color-battle-3d.png \
--second target/pokemon-native-review/psyduck-color-battle-2d.png
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
./target/pokemon-models/debug/examples/immersive_battle_3d \
content-packs/core-modular.browser.crystalpack --wild-species GOLDUCK --size 1000x700 \
--screenshot target/pokemon-native-review/golduck-color-battle-3d.png \
--second target/pokemon-native-review/golduck-color-battle-2d.png
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
./target/pokemon-models/debug/examples/new_bark_3d \
content-packs/core-modular.browser.crystalpack --map MrFujisHouse \
--hour 12 --orbit 1 --zoom 1 --size 1000x700 \
--screenshot target/pokemon-native-review/psyduck-color-world-3d.png \
--second target/pokemon-native-review/psyduck-color-world-2d.png
```
