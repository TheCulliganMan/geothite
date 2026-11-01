# Raticate standard skin

Raticate keeps its existing pear-shaped brown sculpture, cream belly and face,
four incisors, six whiskers, cupped ears, grasping forepaws and long bare tail.
The new canonical asset is
`crates/crystal-voxel-view/models/battle_species/raticate.glb`.
The retired JSON is not retained as a second geometry source. The GLB supplies
its own neutral authoring input through `tools/raticate_glb.py`, following the
existing Gengar migration pattern.

All 63 part names, their order, f32 positions, normals, material colours and
triangle order are preserved. Neutral digest:
`f6043d52ed1588580d3e4c8f44480d8d3cb2b8b2f268e6ccd7220287d76edd0d`.
The asset has 5,642 vertices, 7,764 triangles and 283,264 bytes, with a
0.96-unit neutral silhouette. The external pack lists height digits 204
(two feet, four inches), scaling the battle model to 0.7112 m.

Eight standard joints are the floor root, torso, two ears, two forepaws, tail
base and tail tip. The fused head/body, eyes, muzzle and incisors remain
coherent on the torso. Ear interiors share their shell bones; forepaw digits
and claws share their hand bones. The long splayed feet, toes and toe claws
stay planted on the root. Tail-shaft and segmentation vertices use one common
spatial blend field.

Idle is 3.6 seconds with 73 keys. Attack and hit are one second with 65 keys.
The torso and forepaws brace early; ears and the tail settle later. The root is
identity and clips are rotation-only; production battle cues retain timing and
effects ownership.

Identical index and skin accessors share immutable storage without merging
anatomical parts. There are 247 accessors, fitting the existing 256-accessor
reader limit. No reader limits were enlarged. Canonical re-export preserves
byte-for-byte storage as well as neutral geometry.

All seven Python anatomy, closure, shading, skin, floor, winding and strain
checks pass. The canonical-routing check and both Rust integration checks
also pass. Broader regressions pass: 31 active rig tests (one existing audit
ignored), 27 skinning tests and 13 actor/model tests. The full Blender batch
export was syntax checked, not executed; its Raticate path re-exports the
canonical GLB instead of recreating the retired JSON. The pure CLI was executed
and its output compared byte for byte.

A fresh native paired battle 3D/2D capture was inspected. State was preserved:
hash 2486622478, LCD frame 36, virtual time 0.88753225. The model fits its
footing ring at packed scale, with readable incisors, forepaws and cream tail.
Its warm brown pear body stays distinct from Rattata's low purple quadruped.
The source view is Raticate's back sprite; shaded material tones and the 3D
camera reveal more of the side face. Browser rendering and detailed source-cue
visual motion remain open. This is a verified rig migration, not proof of
all-species visual completion.

```sh
python3 tools/raticate_glb.py \
--input crates/crystal-voxel-view/models/battle_species/raticate.glb \
--output target/pokemon-native-review/raticate-reexport.glb
python3 -m unittest discover -s tools -p test_raticate_glb.py -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view raticate --lib
```


Verified native capture:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked -p crystal-bevy \
--features location-tester --example immersive_battle_3d
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
./target/pokemon-models/debug/examples/immersive_battle_3d \
content-packs/core-modular.browser.crystalpack --wild-species RATICATE --size 1000x700 \
--screenshot target/pokemon-native-review/raticate-battle-3d.png \
--second target/pokemon-native-review/raticate-battle-2d.png
```
