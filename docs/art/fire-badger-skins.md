# Quilava and Typhlosion canonical skins

The fire-starter evolutions replace the old Blender primitive assemblies with
original fire-ferret/badger sculptures and thirteen/seventeen-joint canonical
GLBs. The normal background builder, direct sculpture CLI, strict logical
readers and Rust dispatch use the same deterministic bytes. Registration now
contains 241 skin variants and 34 static species files. Full-roster model and
rig improvement remains active.

| Species | Previous vertices / triangles | Current vertices / triangles | Logical / runtime parts | Joints | Accessors | Bytes |
| --- | --- | --- | --- | --- | --- | --- |
| Quilava | 1,620 / 2,740 | 10,047 / 13,028 | 68 / 39 | 13 | 147 | 370,604 |
| Typhlosion | 1,912 / 2,698 | 12,377 / 17,110 | 77 / 45 | 17 | 194 | 506,608 |

Quilava has a low elongated torso, shaped muzzle, angular red gaze, short
triangular ears, four short curved supports, four grounded pads and twelve
blunt toes. Six folded fire blades form separate neck/rump clusters. Typhlosion
has an upright broad cream trunk, midnight dorsal coat, longer shaped skull,
separate open mandible, tongue, upper fangs, curved forelimbs, palms, six complete
fingers, strong haunches, two grounded pads and six blunt toes. Seven fire blades
form a broad shoulder mantle. All fire blades use the existing Cyndaquil folded
ember/orange/gold construction, with five coherent layers per blade.

Each body/head coat comprises two closed half-solids meeting on exactly shared
surface positions. Exterior normals come from the corresponding unsplit whole;
the cream/midnight border has no raised shell. The fitted eye sockets, whites,
red irises, pupils and glints conform to their actual stored parents. Quilava's
small closed lip line follows its cream muzzle. The first native front view
revealed that the lateral eyes hid around the skull and left Typhlosion's face
looking blank. Both gazes now fit around the forward brow. A source regression
ray-tests the white gaze against both actual skull halves from the front;
posed fitted-layer checks remain intact. A near-degenerate Quilava glint triangle
was corrected by adjusting the patch's placement and size, without lowering
solid or triangle-area tolerances. Typhlosion's gape contains real
empty space between upper skull and lower jaw. Both local pack 2D back views
were inspected; these are not front-sprite or gameplay evidence.

The first Quilava preview stood too high on its supports. Its torso/head were
lowered, supports shortened, muzzle shortened/rounded, and neck fire made taller
than the smaller rump cluster, matching the local 2D back reference more closely.
The first source checks also exposed a pad/leg gap and a detached ear root.
Lower leg endpoints and deeper ear roots fix those actual occupied attachments;
checks retain their original overlap thresholds.

Quilava's controls comprise a weighted identity ground root, torso, skull, four
support controls and six fire-blade controls. Typhlosion adds mandible, shoulders,
wrists, hind limbs and seven fire-blade controls. Coat halves share the same
body/head carrier. Full palms and fingers share each wrist, and every fire
blade's five layers share its own control. Foot pads/toes stay on the fixed
battle root; limb/thigh blends transition up to their articulated roots.

Idle lasts 3.2 seconds with 81 keys; attack/hit last one second with 65 keys.
Tracks are rotation-only, with source-cue timing and neutral endpoints. Each
non-root control moves strongly weighted geometry independently. Packing combines
only same-material geometry on the same carrier; per-foot toe groups stay
separate. Source ranges restore all logical parts in order. Runtime names are
unique and bounded. Standard 20k vertex, 40k triangle, 64 runtime-part/joint,
256 accessor and 1 MiB limits remain unchanged.

Twenty-one Python checks pass against final stored GLBs in 11.702 seconds. They
cover source identity, closed outward connected solids, corner normals, shared
coat seams, complete digits, actual occupied skull/limb/ear/nose/jaw/fire roots,
fitted layer vertices/edges/centres, front-visible gaze, real empty jaw samples, detached/filled
negative controls, hierarchy/inverse binds, independent controls, coherent
carriers, fixed soles, positive posed winding/strain, neutral endpoints,
packing/names/limits and mutated source/byte/species rejection.

Four focused Rust checks pass in 0.79 seconds: original-size GPU activation for
both species, canonical supports/coats/gaze/jaw/controls, and complete four/two
support world walking with preserved upper bindings. All sampled vertices stay
above the floor and inside full locomotion bounds, and gait support checks
retain ground contact. Pack dimensions remain Quilava 0.889 m and Typhlosion
1.7018 m against standing height; authoring heights are 1.20/1.45. Uniform scale,
support offsets, animated bounds and stage admission gates remain intact.
Background exports, direct CLIs and reviewed candidate GLBs match byte for byte.

The ignored `target/pokemon-native-review/fire-badger-skin-review.png` was
inspected inline: previous model at the same height reference, decoded neutral,
front, rear, side and decoded attack at 0.34. The elongated/standing silhouettes,
shared coats, pointed muzzles, small red gaze, complete paws and neck/rump versus
shoulder fire groups remain distinct. Thin fire profiles and small gaze/digit
features remain distance-dependent. This authoring review alone does not
establish production acceptance.

Reproduction:

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  crates/crystal-voxel-view/models/battle_species --only quilava,typhlosion
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools python3 -m unittest -v \
  test_fire_badger_sculpt test_fire_badger_glb
```

| Species | Logical float32 geometry SHA-256 | Packed float32 geometry SHA-256 | GLB SHA-256 |
| --- | --- | --- | --- |
| Quilava | `3257af9b39de38a6140fe858ea41e7f55a2372a26e2ea46b8dd3e2eadb53096c` | `a93592c6a5759ecb4c8dec112b2a655d5283c6514da75bfdb4ba66f4b8ce7650` | `e2aa4cd27676e8d8bd2135233b0bf4e3d598e02bf6192289074c23e28f571184` |
| Typhlosion | `4a5f8f7b25c03d42556da0b55a6af3eaad6be1dc77cd684959bd2d11819072fe` | `ed5bd1e628a7763fd30dce90c6d0e7fa6c09a8c8d5d93bb35eebafe28ae81402` | `03f496fc6724bf122464e0c48e6840ea0e313656adf2a7124b5c395390430b55` |

Native review uses `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image` and `CRYSTAL_PREVIEW_MUTE=1`, with
normal palette only. No visible windows, GUI application launches or focus
changes are permitted. No shiny verification, grass-height, AA/filtering,
audio, pack or gameplay changes belong to this batch.

The shared schema/physical-size check passes in 80.40 seconds and the shared
elapsed-idle/exact-current-attack cue check in 97.82 seconds. With 21 stored
Python and four focused Rust checks, 27 scoped checks pass. Both native examples
built in 43.82 seconds with `location-tester,bevy/zstd_rust`. These checks do
not establish full-suite or gameplay acceptance.

Example offscreen world reproduction from this checkout:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_PARTY_SHINY=0 \
CRYSTAL_RENDER_TEST_PARTY_SPECIES=QUILAVA \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=1000x700 \
CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=300 \
CRYSTAL_RENDER_TEST_CAMERA='3 2 35 0' \
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic CRYSTAL_RENDER_TEST_WALK_STEPS=0 \
./target/pokemon-models/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map Route36 --x 20 --y 8 --view both --hour 12 --walk D \
  --screenshot target/pokemon-native-review/fire-badger-quilava-world.png
```


## Native paired review

Six final 1000×700 production pairs and two moving-world frames were inspected
inline, headless and muted after the gaze correction. All paired authoritative
states are preserved. Four front/world traces each contain 32 real synthetic
input rows across five Route36 positions, (20,8) through (20,12). Both named
walking battles admit complete original-size models with uniform scale, zero
support offset and no fallback. Camera, authentic-ground, omitted-geometry,
shadow and complete animated-bounds gates remain unchanged.

Quilava's elongated coat, short curved supports, cream muzzle, visible red gaze
and distinct neck/rump fire clusters read in world and battle views. The neck
cluster is taller than the rump cluster. Its first-person view looks down over
the low head, but both revised eyes remain visible. All four supports retain
path contact through movement. Typhlosion's upright cream trunk, midnight back,
shaped muzzle, empty jaw, complete fingers/toes and seven-blade fire mantle
remain distinct. The revised eyes show from the front with the same camera,
physical size and pose system as the rejected blank-faced first pass.

Profile views obscure far limbs and narrow the folded fire silhouettes. Tiny
gaze highlights and blunt toes become small at world distance. The mandible's
underside casts a strong angular shadow in the front view. The fire mantle
partly overlaps nearby foliage in projection while walking. These remain
lighting, scenery placement and broader camera/material coverage concerns;
browser and full gameplay are unverified. No grass-height, AA/filtering, audio,
pack or gameplay changes belong to this batch.

Final captures took 37–54 seconds each. Both binaries retain identical SHA-256,
size and modification time before/after the complete run. Preflight verifies
headless/mute tags and exact current compressed GLBs in both examples. Every
final raw image and state receipt is newer than both rebuilt executables.
Completed producers, source/candidate/CLI copies, rejected previews and redundant
intermediate frames were removed. Final raw/paired images, receipts, four
movement CSVs, two moving-world frames, metrics and logs remain ignored under
`target/pokemon-native-review`. Full-roster model and rig work remains active.

| Species / view | Frame / checksum | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| Quilava front | 1728 / 1648692486 | 1735 | 29.660940751 |
| Quilava world | 1668 / 3984888264 | 1676 | 28.56997275 |
| Quilava battle | 1811 / 2134337053 | 2388 | 40.917623167 |
| Typhlosion front | 1837 / 3057692363 | 1845 | 31.572284875 |
| Typhlosion world | 2082 / 1153407213 | 2096 | 36.170662584 |
| Typhlosion battle | 1683 / 4151815281 | 2332 | 40.030472164 |

Stable executable SHA-256:

- `render_at_location`: `ef6eb4468b70b63650241faa57a90174f0b124d3d615e16df929bf331bd112ec`.
- `immersive_battle_3d`: `af5e39e850f2db9133edf739b373b22071efecfaaed06fbefada716f5996238e`.
