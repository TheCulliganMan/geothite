# Sandshrew and Sandslash canonical skins

The normal builder now publishes these original armored-mammal sculptures as
canonical GLB skins; the generic recipes and old static JSON files are retired.
Production builder/reader and Rust registration are implemented and verified.
Native visual review and shared verification results are recorded below.
The registry now contains 221 skins and 54 static species files remain.
The full-roster model and rig goal remains active.

The new `tools/sand_mammal_sculpt.py` recipe gives Sandshrew fitted forehead/bridge
armor and eighteen separate beveled dorsal plates over a rooted shell mantle.
Sandslash has angular fitted eyes, eighteen swept rooted quills and two longer
ivory digging claws per hand. Sandshrew retains three shorter hand claws. Both
have a tapered muzzle, fitted eyes/pupils, supported pinnae/insets, continuous
bent arms, broad palms, distinct haunches/ankles, complete foot pads, three
planted foot claws per side and a continuous armored tail.

`tools/sand_mammal_glb.py` exports standard skins with fifteen controls for
Sandshrew and seventeen for Sandslash: floor root, body, head, two ears, three
controls per arm, two supported legs, two tail carriers and (Sandslash only)
two quill carriers. Arm weights project onto the actual authored centerline.
Complete feet/toes remain fixed during battle clips, while ankles blend into
their own leg controls. Markings share the underlying anatomical carrier.
Both claws dig down during attack; every control moves actual stored geometry.
Idle is 3.2 seconds/81 keys, attack and hit are 1 second/65 keys, with rotation
channels and neutral endpoints. Source cues retain timing ownership.

Packing combines only coherent armor/quill assemblies and the digits of each
individual hand or foot. Source ranges preserve all logical solids and order;
left/right foot ownership stays separate for later world rig integration.

| Species | Logical / runtime parts | Vertices | Triangles | Joints | Accessors | Bytes |
| --- | --- | --- | --- | --- | --- | --- |
| Sandshrew | 58 / 33 | 14,089 | 22,600 | 15 | 171 | 542,764 |
| Sandslash | 53 / 30 | 15,013 | 26,080 | 17 | 157 | 594,240 |

All existing budgets remain unchanged: 20k vertices, 40k triangles, 64 runtime
parts, 64 joints, 256 accessors and 1 MiB. Authoring reference heights are
1.02/1.18 units; these are not evidence of physical game dimensions. Production
integration must retain actual pack dimensions through reference scaling.

Eleven tests pass in 25.446 seconds against independently decoded stored canonical
GLB bytes, checked equal to the normal exporter. They cover closed connected outward solids, actual triangle shading,
grounding, anatomy, occupied-volume roots, original geometry digests, inverse
binds/hierarchy, complete joint usage, fixed feet, winding/strain over nine
samples of each clip, neutral endpoints, independent controls, both-hand digging,
strict source/byte/species mutation rejection, posed armor roots and every
twenty-sided shoulder-root cap vertex remaining inside the torso throughout
sampled clips. The six source anatomy checks are inherited in the eleven GLB
checks rather than counted twice.

```sh
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools \
  python3 -m unittest -v test_sand_mammal_glb
```

The ignored `target/pokemon-native-review/sand-sculpt-review.png` was inspected
inline. Each row compares the actual old prototype with the candidate's front
quarter, rear quarter, front and independently decoded digging attack at 0.44.
Initial bead-like armor was replaced by beveled rectangular plates; gaps were
adjusted so the rows no longer read as continuous bands. Dark forehead stickers
were replaced by same-color fitted armor. Shoulder roots were moved fully inside
the torso and independently tested. Sandslash now has longer paired claws and
its severe eyes/quill silhouette differ from Sandshrew's rounded face/plates.

The software authoring review does not prove production visual acceptance.
World walking first failed on `Claw / complete Claw -1`: the low hand claws
were incorrectly rebound to a leg tree. Packing now uses the explicit runtime
`Hand claw /` category, retaining all original logical source names/ranges and
exact geometry. The regression fails before the correction and passes after it;
no shared role heuristic, geometry, limit or motion threshold was changed.

Four focused Rust checks pass in 1.14 seconds. The canonical check verifies
geometry metrics, hierarchy, fixed complete battle feet, floor/bounds and
independent anatomical motion. Two GPU checks use original 0.6096/0.9906 m
fixtures. World walking checks both complete support trees over 64 phases,
retains every upper-body carrier (including hand claws), and verifies each
complete toe assembly belongs to its actual foot. The generic builder branch
raises rather than providing an alternative low-detail recipe. Background
Blender and normal lazy strict readers reproduce/read the final stored assets.

No windows, browser tabs, focus changes, shiny checks, grass, AA/filtering,
pack, audio or gameplay changes belong to this batch.

| Species | Logical float32 geometry SHA-256 | Packed float32 geometry SHA-256 |
| --- | --- | --- |
| Sandshrew | `0b32692642bd87c17470fc0f74ed582ce3947706815903e432f8dee73866f314` | `2d57a940df4c3b341cf18d234d61cf58c391b2175e1bde7022a7afccc1386b8f` |
| Sandslash | `a659e2f579e749032c71e861f3c7400665afa6c779f215b4c117cc28a96f4ccd` | `fddac7741a7fbab34202801708542b62ccd5cd3a901a46bc37209062e627301d` |

Canonical GLB SHA-256: Sandshrew
`cbb86e347627bcf5dd2d3bc0bfc117067bd09317b7d29589181dd79a9cbc5e04`;
Sandslash `059f3a6e7d3c0a04f8333ed13e75c3eaa8abfbb5f838e81d6ac5e01ba80f7bb6`.
No duplicate JSON catalogs or intermediate compiled assets are added. The
software renderer consumes decoded bytes directly for its authoring review.


Production build and export commands:

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  crates/crystal-voxel-view/models/battle_species --only sandshrew,sandslash
CARGO_TARGET_DIR=target/pokemon-models \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
  cargo test --locked -p crystal-voxel-view sand_mammal --lib -j1 \
  --features bevy/webgl2,bevy/bevy_winit,bevy/x11,bevy/bevy_gilrs,bevy/zstd_rust,crystal-assets/test-fixtures \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0 \
  --config profile.dev.package.crystal-assets.opt-level=2 \
  --config profile.dev.package.crystal-core.opt-level=0
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked -p crystal-bevy \
  --features location-tester,bevy/zstd_rust \
  --example immersive_battle_3d --example render_at_location -j1 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0 \
  --config profile.dev.package.crystal-assets.opt-level=2 \
  --config profile.dev.package.crystal-core.opt-level=0
```


## Final production verification

The all-species schema check passes in 82.28 seconds and the shared exact
battle-cue GPU timing check passes in 121.19 seconds. Together with eleven
stored-asset Python checks and four focused Rust checks, seventeen unique
scoped checks pass. This does not claim the complete gameplay suite passes.
The native examples rebuilt in 56.65 seconds after source/assets were frozen.

Six production 1000×700 captures were inspected inline alongside the same-state
2D reference: ordinary first-person, world orbit and battle for each species.
Two additional actual moving-world frames were inspected. Rendering used
`CRYSTAL_RENDER_TEST_HEADLESS=1`, `CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`
and `CRYSTAL_PREVIEW_MUTE=1`; no native window, browser tab, activation or focus
change was needed. No shiny checks were run.

World captures use the external completed browser pack, Route36 (20,8), hour
12, synthetic walking D, `--view both`, front camera `0 0 20 1` or world camera
`3 2 35 0`. Each of four movement traces contains 32 samples and five actual
authoritative coordinates, (20,8) through (20,12). Final paired images preserve
authoritative state. Extra moving images are asynchronous readbacks and are
not claimed as additional same-time 2D pairs.

Battle captures use Route29's actual walking encounter, held production battle,
`--walk RALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALA`,
`--size 1000x700`, a `--screenshot` 3D target and `--second` 2D target.
The final named pairings are SANDSHREW/SENTRET and SANDSLASH/PIDGEY. Both HUDs
identify their actual species. Last terrain admissions accept complete bodies
with `reason=None`, no fallback and zero support offset. Stage scale/reference
calculations preserve the original pack dimensions: Sandshrew 0.6096 m,
Sandslash 0.9906 m. No shrinking, support adjustment or relaxed frustum,
terrain or omission gate is used.

| Capture | Authoritative frame / hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| sandshrew front | 2246 / 1613522650 | 2256 | 38.515005542 |
| sandshrew world | 2648 / 1621819197 | 2655 | 47.376478959 |
| sandshrew battle | 2590 / 2317871493 | 3212 | 54.977697208 |
| sandslash front | 2211 / 1566325286 | 2217 | 37.910242333 |
| sandslash world | 1901 / 2577512871 | 1909 | 32.549492292 |
| sandslash battle | 2168 / 2918785893 | 2771 | 47.336365 |

Before launch, both executables were checked for headless/mute support and
exact current compressed canonical GLB payloads, decoded equal to source.
Streamed hashes, byte sizes and modification timestamps remained stable through
all final captures:

| Executable | SHA-256 | Bytes | mtime ns |
| --- | --- | --- | --- |
| render_at_location | `66563c6727b9f0cd366f36717bea32527d70f3a9815c2750a2ae965f7d546bc5` | 408798448 | 1792301500839698814 |
| immersive_battle_3d | `c1161f9d8d4715ae6ce9af3c85f4acaab6b0dde24f343cb1a48af1c03b48b161` | 408547384 | 1792301499644449281 |

Sandshrew reads as the smaller plated mammal, with visible muzzle, quiet fitted
eyes, three short hand claws and shell/tail profile. Sandslash has clear severe
eyes, longer paired digging claws and a complete swept quill silhouette. Both
retain complete visible feet in the reviewed world, moving and battle frames.
The front camera puts the smaller Sandshrew below centre with substantial scene
above it; broader camera composition remains open rather than enlarging the
model to fill the frame. Profile views naturally reduce forehead/eye detail.
The original-size species silhouettes and anatomy are accepted in these scoped
native scenes. Browser, broader moving-world/camera/scenery work, remaining
species, existing Surf fixture failure and other project-wide issues remain
open. No complete playable-game acceptance is claimed.

Ignored final evidence is under `target/pokemon-native-review/sand-*`: full
comparison pairs/raw images, paired-state receipts, world movement CSVs,
selected moving-world frames, final test/build logs, executable identity and
asset hashes. The software authoring/decoded-attack sheet is
`sand-sculpt-review.png`. Temporary capture producers, obsolete candidate
metrics and battle-introduction movement scratch output were removed after
review. Native build products and external packs remain ignored and unstaged.
