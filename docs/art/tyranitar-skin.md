# Tyranitar canonical skin

Tyranitar replaces the 1,552-vertex primitive assembly with an original armored
sculpture and fourteen-joint canonical GLB. The background builder, direct
sculpture CLI, strict logical reader and Rust dispatch use identical bytes.
Registration reaches 242 skin variants and 33 static species files. Full-roster
model and rig improvement remains active.

| Previous vertices / triangles | Current vertices / triangles | Logical / runtime parts | Joints | Accessors | Bytes |
| --- | --- | --- | --- | --- | --- |
| 1,552 / 2,768 | 14,734 / 23,128 | 70 / 34 | 14 | 192 | 640,520 |

The local pack's first front animation frame and back sprite were inspected
with its normal GBC palette, directly from pack-contained PNGs. References stay
ignored under `target/pokemon-native-review/tyranitar-pack-reference.png`.
The new silhouette has a broad angular skull, five crown points, four swept
cheek points, four dorsal spines, a heavy curved tail and three tail crests.
It retains two short complete arm chains, broad palms, six ivory hand claws,
thick haunches, short ankles, planted pads and six large ivory toes. The lower
mandible, recessed dark throat, tongue, two upper canines and three lower teeth
leave a real empty gape. Front-visible angular whites, red irises, pupils,
glints and nostrils conform to actual skull surfaces.

The continuous torso contains shallow geometric depressions beneath its two
black triangular vent marks. Both thighs have matching recessed armor marks.
The slate diamond belly shield and two green transverse bands follow their
actual stored parents. Source checks compare the actual dented torso with its
undented counterpart, and reject floating armor/face layers at vertices, edges
and triangle centres. The first eye outline crossed the skull's projected
edge; lowering the gaze keeps every fitted layer supported while preserving
its angled glare. No support or geometry tolerance was weakened.

Controls comprise the weighted identity ground root, torso, skull, mandible,
two shoulders, two elbows, two wrists, two hind limbs and two tail controls.
Crown/cheek plates, gaze, nostrils, throat and upper canines share the skull.
Tongue/lower teeth share the jaw. Complete palms/claws follow their wrists.
Torso armor follows the body, and thigh markings follow the corresponding
upper-leg region. Foot pads/toes remain on the fixed battle root; hind-limb
blends transition up to their articulated roots. Tail weights follow the
source's sampled cross-section rings and arc distances; tail crests use their
embedded root's carrier weights and remain attached through the poses.

Idle lasts 3.2 seconds with 81 keys; attack/hit last one second with 65 keys.
Tracks are rotation-only, with source-cue timing and neutral endpoints. Every
non-root control moves strongly weighted geometry independently. Packing
combines only coherent same-material geometry: skull ink/ivory, crown/cheek
armor, irises, belly bands, torso vents, dorsal/tail crests, each hand's claws,
each foot's toes and lower teeth. Source ranges restore all 70 logical parts
in order. Names are unique/bounded. Standard 20k vertex, 40k triangle, 64
runtime-part/joint, 256 accessor and 1 MiB limits remain unchanged.

Twenty-one Python checks pass against final stored assets in 11.022 seconds:
13 sculpture checks and eight independently decoded skin checks. They cover
source identity, closed connected outward solids, corner normals, complete
anatomy, actual skull/jaw/limb/claw/spine/tail roots, fitted armor/gaze support,
front-visible white gaze, real empty mouth samples through posed solids,
negative detached/filled controls, actual vent depression, uncrossed curved
parts, hierarchy/inverse binds, independent controls, coherent carriers, fixed
soles, positive posed winding/strain, neutral endpoints, packing/names/limits
and mutated source/byte/species rejection.

Three focused Rust checks pass in 0.61 seconds: original-size GPU activation,
canonical armor/jaw/ground supports/controls, and two-support world walking
with preserved upper armor and tail bindings. Two complete world arm chains
remain distinct. All sampled vertices retain floor clearance and stay inside
full locomotion bounds; gait support checks keep contact. Pack height remains
2.0066 m against standing height; authoring height is 1.40. Uniform scale,
support offsets, animated bounds and stage admission gates remain intact.
Background, direct CLI and reviewed candidate GLBs match byte for byte.

The ignored `target/pokemon-native-review/tyranitar-skin-review.png` was inspected
inline: old sculpture at the same height reference, decoded neutral, front,
rear, side and decoded attack at 0.34. The shaped skull, angled glare, empty
jaw, segmented belly, complete claws and heavy curled tail improve the old
round primitives. Rear-on dorsal spines are foreshortened; small markings and
claws remain distance-dependent. The olive coat and slate belly use the existing
3D material style rather than the four-color sprite palette. This authoring
review alone does not establish production acceptance.

Reproduction:

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  crates/crystal-voxel-view/models/battle_species --only tyranitar
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools python3 -m unittest -v \
  test_tyranitar_sculpt test_tyranitar_glb
```

- Logical float32 geometry SHA-256: `498ef4bcb69caee77da98e21dd671ac5d3250750ceda5f1c397a036e7023834d`.
- Packed float32 geometry SHA-256: `ab12485cb05e61b1722d768f92af6cc5151592507139262e80a58ce2c360db33`.
- GLB SHA-256: `455357d35a6e7f16ed9f693b7296e6a10c358abf0afdc98056a0c92d9bd11cb4`.

Native review uses `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image` and `CRYSTAL_PREVIEW_MUTE=1`, normal
palette only. No visible windows, GUI application launches or focus changes are
permitted. No shiny verification, grass-height, AA/filtering, audio, pack or
gameplay changes belong to this batch.


The shared schema/physical-size check passes in 74.91 seconds and the shared
elapsed-idle/exact-current-attack cue check in 75.99 seconds. With 21 stored
Python and three focused Rust checks, 26 scoped checks pass. Both native
examples built in 44.28 seconds with `location-tester,bevy/zstd_rust`. These
checks do not establish full-suite or gameplay acceptance.

## Native paired review

Three final 1000×700 production pairs and one moving-world frame were inspected
inline, headless and muted. All paired authoritative states are preserved.
Front/world traces each retain 32 real synthetic-input rows across five Route36
positions, (20,8) through (20,12). The walking battle admits complete original
2.0066 m Tyranitar with uniform scale, zero support offset and no fallback.
Camera, authentic-ground, omitted-geometry, shadow and complete animated-bounds
gates remain unchanged. No physical size or FOV changed for review.

The angular skull, visible red glare, open tooth-lined mandible, crown/cheek
plates, segmented slate belly, recessed triangular marks, broad clawed feet,
dorsal spines and heavy curled tail remain readable. Two complete walking
supports stay at path height, and the tail crests stay with the moving tail.
The source's olive/slate material style is less saturated than the four-color
pack sprite. Front jaw undersides receive strong angular shadows. Profile
views obscure far limbs; rear-on spines are foreshortened. The moving view
passes close to the sign and nearby foliage in projection. These remain
broader follower/scenery placement, lighting and camera/material concerns.
Browser, all environments/cameras and full gameplay remain unverified.

Captures took 31–63 seconds. Both binaries retained identical SHA-256, size and
modification time before/after the complete run. Preflight verifies headless/
mute tags and exact current compressed GLBs in both binaries. Every raw image
and paired-state receipt is newer than both rebuilt executables, and the
working capture helper additionally rejects stale per-process output.
Completed Tyranitar-specific producers, source/candidate/CLI copies and
redundant frames were removed. Final raw/paired images, source reference,
receipts, two movement CSVs, moving-world frame, metrics and logs remain ignored
under `target/pokemon-native-review`. The shared ignored
`run-headless-review.py` helper remains for the ongoing full-roster task and
must be removed when that task finishes. No shiny checks, grass-height,
AA/filtering, audio, pack or gameplay changes belong to this batch.

Example offscreen world reproduction:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_PARTY_SHINY=0 \
CRYSTAL_RENDER_TEST_PARTY_SPECIES=TYRANITAR \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=1000x700 \
CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=300 \
CRYSTAL_RENDER_TEST_CAMERA='3 2 35 0' \
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic CRYSTAL_RENDER_TEST_WALK_STEPS=0 \
./target/pokemon-models/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map Route36 --x 20 --y 8 --view both --hour 12 --walk D \
  --screenshot target/pokemon-native-review/tyranitar-world.png
```

| View | Frame / checksum | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| front | 1369 / 3026858321 | 1374 | 23.380846667 |
| world | 1190 / 1326905575 | 1194 | 20.344727333 |
| battle | 1715 / 124133308 | 2389 | 40.804550708 |

Stable executable SHA-256:

- `render_at_location`: `8171962efc0d0559154e467708fa34403c4ebc1f8b6becd465f11f5c6467a7e8`.
- `immersive_battle_3d`: `9e6dbd980376abe353f087651e2509e420813fab773b217d2a00d4c3ad7ab7e7`.
