# Aerodactyl canonical skin

The detailed fossil flyer now uses a canonical GLB through the normal background
builder, direct sculpture CLI, strict logical reader and Rust skin dispatch.
Registration reaches 233 skin variants and 42 static species files. The complete
model and rig improvement goal remains active.

All reviewed source geometry is preserved exactly in float32: 18,907 vertices,
31,516 triangles and 56 logical parts. Same-carrier packing reduces this to 27
runtime parts, nine joints, 149 accessors and 791,956 bytes. It combines head ink,
paired cheeks/horns/eye whites, upper/lower teeth, each wing's fingers/claws and
the rooted foot talons. Logical ranges restore every original part in order.
The 20k vertex, 40k triangle, 64 part/joint, 256 accessor and 1 MiB limits remain.

The skeleton has a weighted identity ground root, body, skull, lower jaw, two
wings, two hind limbs and a tail. All upper teeth and the supported upper palate
follow the skull. Lower teeth, lower lining and tongue follow the mandible.
Membranes, leading arms, branching fingers and hooked wrist claws share each
wing hinge. Feet and all six foot talons retain the ground root. Torso/neck and
hind-limb blends preserve root contact; the complete arrowhead follows its tail.

Idle lasts 3.2 seconds with 81 keys; attack/hit last one second with 65 keys.
Tracks use rotation-only source-cue timing with neutral endpoints. Each control
moves weighted geometry independently. The idle includes four wing beats per
cycle, preserving authored flight recognition. World Fly mode and canonical
indices/weights remain unchanged. Battle soles stay fixed without a floor clamp.

The first recoil closed the jaw below the original 0.156-unit clearance at a
tested sample. The final roar/recoil gasp opens the mandible, retaining its full
motion and the source clearance requirement. Teeth, palate and tongue stay on
their actual jaws. Independent ray checks prove real empty mouth space through
poses, and a closed-jaw negative control fails the clearance requirement.

Twenty stored Python checks pass in 22.567 seconds: twelve anatomy checks and
eight independently decoded skin checks. They cover exact source/packing,
hierarchy/inverse binds, budgets, closed outward solids, actual corner normals,
all controls/carriers, fixed soles, positive winding and strain, neutral
endpoints, occupied roots, real empty jaw space, fitted face/palate/tongue
support, and source/byte/species mutation rejection. The source tests now use
the existing indexed triangle ray oracle with the same barycentric calculation
and tolerances; test geometry and thresholds remain unchanged.

Three focused Rust checks pass in 1.11 seconds: GPU activation at original pack
size, canonical talons/jaw/independent controls and complete world flight bounds.
The original physical dimension is 1.8034 m, measured against standing height;
authoring height remains 1.30. Both wings retain authored flight recognition.
The background builder, direct CLI and reviewed candidate match canonical bytes
exactly. Support, uniform scale and complete animated bounds remain intact.

The ignored `target/pokemon-native-review/aerodactyl-skin-review.png` was inspected
inline after the final pose change. Previous static and decoded neutral are
compared at the same fixed neutral fit, followed by front/rear/side and attack
at 0.34. The narrow toothed skull, open lower jaw, rose tongue, swept horns,
scalloped violet membranes, branching spars, complete digits and arrow tail
remain distinct. This authoring view alone does not establish production
acceptance.

Reproduction:

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  crates/crystal-voxel-view/models/battle_species --only aerodactyl
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools \
  python3 -m unittest -v test_aerodactyl_sculpt test_aerodactyl_glb
```

Geometry and GLB SHA-256:

- Logical float32: `f49ef0e44a17c7426e61b0c429eb3215a39aa03fd15a48cf57d24a199b29aa0a`.
- Packed float32: `09cc8fb6a4257c7daabf5eb7a77b4bea649eaf9ec6f5b0235cc8cedce0a7503a`.
- GLB: `404c612ab9cc14d16104a328fc0c9d6a88fe84324aff4cf8b40e1d3b75b1a3f7`.

Native captures use `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image` and `CRYSTAL_PREVIEW_MUTE=1`.
The production path disables Winit and macOS primary-window retention and uses
the schedule runner. Binary preflight checks headless/mute tags and exact
compressed current GLB payloads. No windows, focus changes, shiny checks,
grass-height, AA/filtering, audio, pack or gameplay changes belong to this batch.

The shared schema/physical-size check passes in 95.83 seconds; the shared
elapsed-idle/exact-current-attack cue check passes in 95.31 seconds. With the
20 stored Python and three focused Rust checks, 25 scoped checks pass. Both
current native examples built successfully in 64 seconds with
`location-tester,bevy/zstd_rust`. These checks establish neither full-suite
success nor full gameplay acceptance.

Example offscreen world capture from this checkout:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_PARTY_SHINY=0 \
CRYSTAL_RENDER_TEST_PARTY_SPECIES=AERODACTYL \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=1000x700 \
CRYSTAL_RENDER_TEST_CAMERA='1 2 35 0' \
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic CRYSTAL_RENDER_TEST_WALK_STEPS=0 \
./target/pokemon-models/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map Route36 --x 20 --y 8 --view both --hour 12 --walk D \
  --screenshot target/pokemon-native-review/aerodactyl-world.png
```

## Native paired review

Three current production 3D/2D pairs and one moving-world frame were inspected
inline, offscreen and muted at 1000×700. All paired authoritative states are
preserved. Front/world captures retain 32 actual synthetic-input movement rows
each across five Route36 positions, (20,8) through (20,12). The named walking
battle admits the complete original 1.8034 m body with uniform scale and zero
support offset, no fallback, and unchanged complete-body camera, authentic
ground and omitted-geometry/shadow gates. The head, jaws, entire wings and arrow
tail fit the final battle view. World review uses normal camera zoom step 1;
model dimensions and FOV remain unchanged.

The pointed skull, twelve teeth, empty gape and rose tongue, swept horns,
scalloped violet wings with gray spars, wrist digits and arrowhead remain
readable. Frontal first-person views hide the narrow eyes behind the long snout;
the oblique battle view exposes the stern gaze. Strong interior mouth shadow
emphasizes the cavity. Thin wing edges/spars and talons alias at distance.
The moving world frame has tree occlusion behind the follower, hiding portions
of the far wing/tail. Broader scenery placement, lighting/AA, browser and camera
coverage remain open. Battle soles remain grounded; world Fly mode is retained.

Captures completed in 37.061, 39.594 and 62.306 seconds. Both executable identities
remain unchanged across all captures, with exact current compressed GLB payloads
verified in both tools before and after. Completed producer scripts, temporary
source/candidate/CLI copies and redundant intermediate frames were removed.
Final raw/paired images, receipts, two movement CSVs, one moving-world frame,
metrics and logs remain ignored under `target/pokemon-native-review`. No shiny
checks, grass, AA/filtering, pack, audio or gameplay changes belong to this batch.
The full model and rig improvement goal remains active.

| Species / view | Frame / checksum | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| Aerodactyl front | 1338 / 2462671042 | 1344 | 22.844857042 |
| Aerodactyl world | 1772 / 4077530040 | 1780 | 30.444799125 |
| Aerodactyl battle | 2121 / 650254649 | 2739 | 47.14085404 |

Stable executable SHA-256:
- `render_at_location`: `869730d7b49ea7527caf05f11e997cd2f78db631cc8050de3e7672dde0b5c96a`.
- `immersive_battle_3d`: `96392ca914d42e884187c762c01fab79b68626f996b9d84a2c35d5bc7ff255e9`.
