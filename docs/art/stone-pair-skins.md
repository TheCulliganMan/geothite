# Graveler and Golem canonical skins

The stone pair now uses canonical GLBs through the normal background builder,
direct sculpture CLIs, strict logical readers and Rust dispatch. Registration
reaches 237 skin variants and 38 static species files. Full-roster model and rig
improvement remains active.

| Species | Previous vertices / triangles | Current vertices / triangles | Logical / runtime parts | Joints | Accessors | Bytes |
| --- | --- | --- | --- | --- | --- | --- |
| Graveler | 18,118 / 23,836 | 18,118 / 23,836 | 40 / 22 | 10 | 114 | 709,896 |
| Golem | 20,876 / 35,124 | 19,904 / 33,180 | 77 / 29 | 12 | 154 | 811,200 |

Graveler retains its irregular faceted boulder, jagged fitted forehead, grin,
four distinct arms/palms, twelve blunt fingers and six rear stones. Golem keeps
all 32 polygon plates, shell seams, reptile skull, open jaw, tongue, fangs,
three lower teeth, six hand claws, six forward toes and two heel claws. Its
palms use 24×16 rather than 28×20 sphere sampling, foot pads use 28×18 rather
than 32×22, and recessed oral chamber uses 28×18 rather than 32×22. This removes
972 vertices and 1,944 triangles while retaining the complete shell and all
digits. Existing 20k vertex, 40k triangle, 64 part/joint, 256 accessor and 1 MiB
limits remain unchanged. Signed zeros are normalized for stable hashes.

Packing combines only same-material assemblies on the same controls: Graveler
body markings, rear stones and each palm's fingers; Golem polygon plates,
coherent head markings/fangs, each hand's claws, each foot's toes and lower
teeth. Exact source ranges restore all original logical parts in order. Left
and right fixed-root toes remain separate runtime parts for walking support.
An initial export gave the two differently colored plate groups the same
runtime name. Production parsing rejected it. Packed groups now receive unique
bounded names, checked independently before publication; geometry is unchanged
by that correction.

Graveler's ten controls comprise a weighted identity ground root, boulder body,
four shoulder/arm controls and four wrists. Complete palms and their three
digits follow each wrist. The folded lower forearms stay with their own lower
arm; upper/lower pairs retain independent motion. Fitted face and rear rubble
remain on the boulder. Both short feet stay fixed during battle.

Golem's twelve controls comprise the ground root, shell body, skull, mandible,
two shoulders, two elbows, two wrists and two hind limbs. All shell plates
follow the seam body. Gaze, upper muzzle, oral chamber and fangs follow the
skull. Tongue and lower teeth follow the mandible, preserving real empty jaw
space through roars and recoil gasps. Complete palms and hand claws share their
wrists. Hind-limb blends transition to fixed feet, forward toes and heel claws;
no floor clamp is used.

Idle lasts 3.2 seconds with 81 keys; attack/hit last one second with 65 keys.
Tracks use rotation-only source cues and neutral endpoints. Every non-root
control moves strongly weighted geometry independently. Existing world walking
provides two complete support chains per species. It preserves canonical
bindings for all upper anatomy and supplies four/two independent arm chains
for Graveler/Golem. Toe and heel labels remain anatomical and do not become
extra legs. Full locomotion bounds, floor clearance and support checks pass.

Thirty-two checks pass against the final stored GLBs in 34.448 seconds: 24 source
anatomy checks and eight independently decoded skin checks. They cover source/
packing/hierarchy identity, inverse binds, limits, bounded unique part names,
closed outward solids and corner normals, all independent controls, complete
digit carriers, fixed soles, every posed triangle's positive winding and strain,
neutral endpoints, occupied limb/wrist/digit/rock/jaw roots, fitted face and
polygon-plate support, actual empty jaw samples through all posed solids,
negative controls and source/byte/species mutation rejection.

Four focused Rust checks pass in 1.47 seconds: original-size GPU activation for
both species, canonical faces/shells/ground supports/controls, and complete
two-support world walking with preserved upper bindings. Pack dimensions remain
Graveler 0.9906 m and Golem 1.397 m against standing height. Authoring heights
remain 1.10 and 1.25. The normal background builder, direct CLIs and reviewed
candidates match canonical GLB bytes exactly. Uniform scale, support offsets,
complete animated bounds and stage gates remain intact.

The ignored `target/pokemon-native-review/stone-skin-review.png` was inspected
inline after the naming correction. Named rows compare previous static,
decoded neutral, front/rear/side and decoded attack at 0.34 with the same fixed
neutral fit. Graveler's lower palms remain distinct from the larger upper pair;
Golem's plate pattern, stern red gaze, empty mouth and complete claws remain
readable. The modest sphere reduction retains the original silhouette. These
authoring images alone do not establish production acceptance.

Reproduction:

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  crates/crystal-voxel-view/models/battle_species --only graveler,golem
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools \
  python3 -m unittest -v test_graveler_sculpt test_golem_sculpt test_stone_pair_glb
```

| Species | Logical float32 geometry SHA-256 | Packed float32 geometry SHA-256 | GLB SHA-256 |
| --- | --- | --- | --- |
| Graveler | `4734b3c5b5cc465ed932c49ea3e186c7ed55e64ac8d80ccc36ab5a486ea93ea6` | `6c318d8d37db542a1b5cf1901ab815c6ff9674aba735a626ae390a3c8b676049` | `683e116b62306df1d33ad7469ce35739ba6f6e9d19e5ef79eb3a6e5fb99042a3` |
| Golem | `553f96c63633520a526ec732e47bce30fd56e62774bae51893be60f6134800de` | `9b16cb2ddc7577b4f64ae437f97e894b2154f75f5626f1dab4fcb1564a0b9671` | `6214eba0a8e469571ec5d91259c633c5eb44678a632615d393f7110f5f4fecd0` |

Native captures use `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image` and `CRYSTAL_PREVIEW_MUTE=1`.
The path disables Winit and macOS primary-window retention and uses the schedule
runner. Binary preflight checks headless/mute tags and exact compressed current
GLBs in both examples. No windows, focus changes, shiny checks, grass-height,
AA/filtering, audio, pack or gameplay changes belong to this batch.

The shared schema/physical-size check passes in 105.78 seconds and the shared
elapsed-idle/exact-current-attack cue check in 105.89 seconds. With 32 stored
Python and four focused Rust checks, 38 scoped checks pass. Both native capture
examples built successfully in 60 seconds with `location-tester,bevy/zstd_rust`.
These checks establish neither full-suite success nor full gameplay acceptance.

Example offscreen world reproduction from this checkout:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_PARTY_SHINY=0 \
CRYSTAL_RENDER_TEST_PARTY_SPECIES=GOLEM \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=1000x700 \
CRYSTAL_RENDER_TEST_CAMERA='3 2 35 0' \
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic CRYSTAL_RENDER_TEST_WALK_STEPS=0 \
./target/pokemon-models/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map Route36 --x 20 --y 8 --view both --hour 12 --walk D \
  --screenshot target/pokemon-native-review/stone-golem-world.png
```

## Native paired review

Six current production 3D/2D pairs and two moving-world frames were inspected
inline, offscreen and muted at 1000×700. Every paired authoritative state is
preserved. Four front/world captures retain 32 actual synthetic-input movement
rows each across five Route36 positions, (20,8) through (20,12). Both named
walking battles admit complete original 0.9906/1.397 m bodies with uniform scale,
zero support offset and no fallback. Complete-body camera, authentic-ground,
omitted-geometry/shadow and full animated-bounds gates remain unchanged.

Graveler's irregular boulder, jagged forehead, narrow glare, pink grin and four
arm/palm groups remain recognizable. Front views expose the smaller lower pair;
profile views hide the far arms and some feet. Golem retains its plate pattern,
stern red gaze, tooth-lined empty jaw, tongue, broad pads and complete claws.
The reduced palms, foot pads and recessed mouth chamber retain their silhouette
and roots. Both moving-world views keep supports at path height.

Foliage partly occludes Graveler's far upper fingers in the front scene. Golem's
moving shell passes close to the sign in projection. These remain broader
follower/scenery placement and camera-coverage concerns. Shell seams and thin
claws alias at distance; some ivory surfaces and the mouth chamber receive deep
shadows. Broader lighting/AA, browser, all world/material combinations and full
gameplay remain unverified. No physical dimension, support gate or FOV changed
to satisfy the review.

Final captures took 39–94 seconds each using the same stable rebuilt executables
throughout. Exact current compressed GLBs and binary identities were checked
before and after. Completed producer scripts, temporary source/candidate/CLI
copies and redundant intermediate frames were removed. Final raw/paired images,
receipts, four movement CSVs, two moving-world frames, metrics and logs remain
ignored under `target/pokemon-native-review`. No shiny checks, grass-height,
AA/filtering, audio, pack or gameplay changes belong to this batch. The full
model and rig improvement goal remains active.

| Species / view | Frame / checksum | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| Graveler front | 2101 / 4084250299 | 2112 | 36.262972791 |
| Graveler world | 1829 / 3090789491 | 1837 | 31.337844501 |
| Graveler battle | 2268 / 736087815 | 2912 | 49.966768293 |
| Golem front | 1976 / 1465921694 | 1985 | 33.852396334 |
| Golem world | 1751 / 1045163357 | 1759 | 29.969815541 |
| Golem battle | 1545 / 3158597296 | 2171 | 37.094500458 |

Stable executable SHA-256:
- `render_at_location`: `be48b784d50e19ac45c9e67746da7bf5453e5879447111cdaef629c74b5eeae3`.
- `immersive_battle_3d`: `a9cb16d0a46176ecc95b87ff1e98dcbe700cb337d07a38f2f98ca98067241750`.
