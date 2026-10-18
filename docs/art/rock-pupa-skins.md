# Larvitar and Pupitar canonical skins

The two pre-evolutions replace the old Blender primitive assemblies with
original rock-larva/pupa sculptures and eleven/four-joint canonical GLBs.
The background builder, direct sculpture CLI, strict logical readers and Rust
dispatch use identical bytes. Registration reaches 244 skin variants and 31
static species files. Full-roster model and rig improvement remains active.

| Species | Previous vertices / triangles | Current vertices / triangles | Logical / runtime parts | Joints | Accessors | Bytes |
| --- | --- | --- | --- | --- | --- | --- |
| Larvitar | 1,432 / 2,552 | 8,328 / 13,104 | 45 / 20 | 11 | 121 | 367,740 |
| Pupitar | 816 / 1,112 | 4,503 / 5,224 | 28 / 9 | 4 | 58 | 204,580 |

Both local pack front/back sprites were inspected with their normal GBC
palettes, directly from pack-contained PNGs. References remain ignored under
`target/pokemon-native-review/rock-pupa-pack-reference.png`. Larvitar retains
one long swept crown, a shaped broad skull, fitted angular red gaze, long black
cheek tears, a red belly diamond, small side vents, two short forelimbs, rounded
palms, six blunt green fingers, two rounded haunches, planted pads and six blunt
green toes. Its broad curved tail ends in three distinct fan points. The old
adult-style dorsal spikes are removed. Pupitar has a faceted tapered blue core,
angular visor, three crown points, six paired side points, front/back chevron
seams and a pointed lower fold. It has no invented arms or feet.

All surfaces remain closed connected solids. Gaze, markings and shell seams
conform to their actual stored parents. Both white gazes are checked for
visibility ahead of the skull. The first software preview made Pupitar's gaze
too small; wider, taller fitted lenses and proportionate irises improve
readability before publication. The initial chevron outline crossed itself;
parallel lower/upper edges correct the actual polygon. No geometry or support
tolerance was weakened.

Larvitar's controls comprise a weighted identity ground root, torso, skull,
two shoulders, two wrists, two hind limbs and two tail controls. Horn, gaze,
nostrils, closed lip and cheek tears share the skull. Complete palms and fingers
share wrists. Body marks follow the torso. Planted pads/toes remain fixed in
battle while thigh blends transition up to the hind limbs. The heavy tail stem
has its own control, and all three fan points share the attached tip control.

Pupitar's four controls comprise the weighted ground root, cocoon core, visor
and lower fold. Crown/upper side points and gaze share the visor. The four lower
side points and front/back seams share the core's fully weighted upper region.
Core and lower-fold blends transition from their fixed ground vertices to their
own controls. Sealed armor receives restrained motion. Existing grounded
locomotion is retained without adding hover or procedural feet.

Idle lasts 3.2 seconds with 81 keys; attack/hit last one second with 65 keys.
Tracks are rotation-only, with source-cue timing and neutral endpoints. Every
non-root control moves strongly weighted geometry independently. Packing
combines only coherent same-material geometry: head ink/ivory, irises,
each palm/finger group, each foot's toes, tail fan points, body vents,
Pupitar's visor/body spike groups and its front/back seams. Source ranges
restore all 45/28 logical parts in order. Runtime names are unique/bounded.
Standard 20k vertex, 40k triangle, 64 runtime-part/joint, 256 accessor and 1 MiB
limits remain unchanged.

Twenty-one Python checks pass against final stored GLBs in 9.850 seconds:
13 source checks and eight independently decoded skin checks. They cover
source identity, closed connected outward solids, corner normals, complete
anatomy, actual skull/crown/limb/digit/fan/side-point/lower-fold roots, fitted
gaze/marking/front/back seam vertices/edges/centres, front-visible white gaze,
negative detached controls, uncrossed curved anatomy, hierarchy/inverse binds,
independent controls, coherent carriers, fixed ground vertices, positive posed
winding/strain, neutral endpoints, packing/names/limits and mutated source/byte/
species rejection.

Four focused Rust checks pass in 0.59 seconds: original-size GPU activation for
both species, canonical gaze/ground/controls, and grounded world locomotion.
Larvitar retains two complete walking supports and two arm chains. Pupitar
retains its canonical joint count/weights and no leg chains or fake arms.
Both keep ground contact and full locomotion bounds across all sampled vertices.
Pack dimensions remain 0.6096/1.1938 m against standing height; authoring heights
are 1.04/1.20. Uniform scale, support offsets, full bounds and admission gates
remain intact. Background exports, direct CLIs and reviewed candidates match
byte for byte.

The ignored `target/pokemon-native-review/rock-pupa-skin-review.png` was inspected
inline: previous sculptures at the same height reference, decoded neutral,
front, rear, side and decoded attack at 0.34. Larvitar's single horn, red diamond,
cheek tears, complete paws and three-pronged tail improve the generic baby
Tyranitar shape. Pupitar's visor, larger angular gaze, crown/side points and
front/back segmented shell improve the old simple cocoon. Side-on points are
foreshortened, and small details remain distance-dependent. This authoring
review alone does not establish production acceptance.

Reproduction:

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  crates/crystal-voxel-view/models/battle_species --only larvitar,pupitar
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools python3 -m unittest -v \
  test_rock_pupa_sculpt test_rock_pupa_glb
```

| Species | Logical float32 geometry SHA-256 | Packed float32 geometry SHA-256 | GLB SHA-256 |
| --- | --- | --- | --- |
| Larvitar | `31356ca86c649dca327d270c28c6f8bb377c43f3d00e1855151e473f99510f7c` | `61cd43cca9f55febba20701c2ce9244510d85dd4a826f4476c903f0815bb9706` | `43e1dcfd3f6b5f78bfe3ad0e5510e3277a6f77325a84ac4227dde6e550eea799` |
| Pupitar | `1905fcbbb5bdf9066860f8fb4d6b5c2a3646da2fd4098cd3fc75a43c59d1e6b7` | `b972d03ec14adb898e754bb68511dad4e7f7f131aee538229c455b77ecc3a492` | `bec27e6a6b7bb2d3ee42a7b7581284c0abff812d34d20a85e0b80900e032da34` |

Native review uses `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image` and `CRYSTAL_PREVIEW_MUTE=1`, normal
palette only. No visible windows, GUI application launches or focus changes
are permitted. No shiny checks, grass-height, AA/filtering, audio, pack or
gameplay changes belong to this batch.


The shared schema/physical-size check passes in 77.48 seconds and the shared
elapsed-idle/exact-current-attack cue check in 75.64 seconds. With 21 stored
Python and four focused Rust checks, 27 scoped checks pass. Both native examples
built in 44.17 seconds with `location-tester,bevy/zstd_rust`. These checks do
not establish full-suite or gameplay acceptance.

## Native paired review

Six final 1000×700 production pairs and two moving-world frames were inspected
inline, headless and muted. All paired authoritative states are preserved.
Four front/world traces each retain 32 real synthetic-input rows across five
Route36 positions, (20,8) through (20,12). Both named walking battles admit
complete original 0.6096/1.1938 m models with uniform scale, zero support offset
and no fallback. Camera, authentic-ground, omitted-geometry, shadow and full
animated-bounds gates remain unchanged. No physical dimension or FOV changed
for review.

Larvitar's single tall horn, shaped head, red gaze/diamond, black cheek tears,
complete green paws and fan tail remain identifiable. Feet stay on the path
through movement. Its small original size limits detail at world distance;
profile views obscure far limbs and foreshorten/overlap the three tail points.
The green body blends with the taller grass in the battle composition. This
batch does not change grass height, lighting, camera placement or model scale
to compensate.

Pupitar's larger gaze, angular visor, three crown points, six paired side points
and chevron shell seams read in front/battle views. Its lower fold retains
path contact without hover, and walking adds no fake limbs. Side-on points
foreshorten into rounded root sections, and seam/iris details become small at
world distance. Broader camera/material/scenery combinations and browser/full
gameplay remain unverified. All render work stays offscreen and muted.

Final captures took 30–52 seconds. Both executables retained identical SHA-256,
size and modification time before/after the run. Preflight verifies headless/
mute tags and exact current compressed GLBs in both binaries. Every final raw
image and state receipt is newer than both rebuilt executables; the capture
helper also rejects stale per-process outputs. Completed batch-specific
producers, source/candidate/CLI copies and redundant frames were removed.
Final raw/paired images, reference, receipts, four movement CSVs, two moving
frames, metrics and logs remain ignored under `target/pokemon-native-review`.
The shared ignored `run-headless-review.py` and `review-source-models.py`
helpers remain for ongoing roster work and must be removed when that full task
finishes. Full-roster model and rig improvement remains active.

Example offscreen world reproduction:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_PARTY_SHINY=0 \
CRYSTAL_RENDER_TEST_PARTY_SPECIES=PUPITAR \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=1000x700 \
CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=300 \
CRYSTAL_RENDER_TEST_CAMERA='3 2 35 0' \
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic CRYSTAL_RENDER_TEST_WALK_STEPS=0 \
./target/pokemon-models/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map Route36 --x 20 --y 8 --view both --hour 12 --walk D \
  --screenshot target/pokemon-native-review/rock-pupa-pupitar-world.png
```

| Species / view | Frame / checksum | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| Larvitar front | 1357 / 1194302930 | 1363 | 23.212714625 |
| Larvitar world | 1200 / 3698599911 | 1202 | 20.504918082 |
| Larvitar battle | 1323 / 2432559064 | 1891 | 32.408151918 |
| Pupitar front | 1338 / 2522483987 | 1343 | 22.825656708 |
| Pupitar world | 1198 / 2327428333 | 1205 | 20.532218 |
| Pupitar battle | 1431 / 1348831062 | 2088 | 35.715058127 |

Stable executable SHA-256:

- `render_at_location`: `9b7345a02861312be148cad6766a0775ba2aa2fb1e161ca71d61372c89dc9b33`.
- `immersive_battle_3d`: `a9881550c3532da7afacfb30cd3398a57f69e28e094fc6f5b26bbe3efda0b12f`.
