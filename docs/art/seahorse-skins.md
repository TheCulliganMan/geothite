# Horsea, Seadra and Kingdra skins

Canonical GLBs now replace the three static exports. Normal builder, strict
readers, species dispatch and GPU/world registration use these exact assets.
The registry contains 226 skins and 49 static species files. The full-roster
model and rig goal remains active. Native paired review is recorded below; stored checks alone do not establish
visual acceptance.

The family retains its distinct rounded/serrated/branched-crown anatomy, actual
hollow snouts and recessed dark chambers, fitted eyes, coiled tails, breast
plates and folded fins. Skin reference height remains 1.07 authoring units.
Original physical pack dimensions are checked by the focused GPU fixtures;
all three also pass original-size production stage admission below.

| Species | Original vertices / triangles | Current vertices / triangles |
| --- | --- | --- |
| Horsea | 37,625 / 57,516 | 15,790 / 24,912 |
| Seadra | 19,211 / 29,044 | 13,214 / 20,324 |
| Kingdra | 40,366 / 61,464 | 19,154 / 31,156 |

Eye layers and Kingdra's breast fit actual stored parent triangles instead of
subdivided chords. Fins, body/head sections, thin folds, rays and horn surfaces
use bounded tessellation. Thin eight-sided fold tubes share smooth normals,
removing redundant facet splits while preserving their actual cross-sections.
No existing 20k-vertex, 40k-triangle, 64-part, 64-joint, 256-accessor or 1 MiB
limit is increased. Numerical zero signs are normalized for portable export.

The stronger checks exposed an existing self-intersection near Horsea/Seadra's
tail root. Its initial bend now clears the inward turn, and its full forty-sided
root cap seats inside the torso. Kingdra's root seats farther inside the chest.
All complete tail caps are tested through poses with a 0.0002 burial margin.
A ray at a shared mesh edge can hit both adjacent triangles at the same surface;
the interval oracle deduplicates crossings within 1e-9 before pairing solid
intervals. This corrected a false failure on Kingdra's already-interior vertex.
Outside points are rejected by negative controls; the burial margin is unchanged.

Seadra's ivory fin tips now clip actual parent triangles on both sides rather
than using chords which could sink at edge midpoints. Kingdra's rear armor
seams have a 0.0032 authoring offset, retaining the strict 0.0001 front-surface
clearance at vertices, edge midpoints and triangle centres.

`tools/seahorse_glb.py` supplies standard exporters and bounded strict readers.
Horsea/Seadra have ten joints and Kingdra twelve: identity structural root,
body, head, paired main fins, paired head/neck fins, three tail carriers and
(Kingdra only) two crown carriers. The root is identity and has no directly
weighted foot vertices for these swimmers. Every non-root control influences
stored geometry independently. Hollow snout walls, lining and chamber share
the head; fitted eyes share it too. Breast folds/rear seams follow the body;
fin rays/tips and crown branches follow their actual parent controls.

Tail pivots and weights use the original authored centerline and radii, shared
with the sculpture recipes. Weights match the actual cross-section circles:
nearest-centerline projection can jump to an adjacent turn in a tight coil,
which initially produced inverted triangles. Each ring now retains coherent
weights along arc length, with broad transitions across three controls. Body
and tail use yaw, preserving the authored waterline without a floor clamp.
Head, fins and crowns add distinct pitch/yaw/roll. Idle lasts 3.2 seconds with
81 keys, attack/hit one second with 65 keys, with neutral endpoints and
rotation-only source-cue timing.

Packing combines only coherent same-material assemblies: breast/rear folds,
per-side fin rays, per-side crown branches and Horsea's per-side head-fin rays.
Logical ranges restore every original solid and its order; no geometry is
discarded to reduce part count.

| Skin | Logical / runtime parts | Joints | Accessors | Bytes |
| --- | --- | --- | --- | --- |
| Horsea | 36 / 24 | 10 | 137 | 687,580 |
| Seadra | 34 / 30 | 10 | 161 | 584,640 |
| Kingdra | 47 / 35 | 12 | 185 | 846,336 |

Eighteen sculpture and eight independently decoded skin checks pass against
the canonical stored GLBs (26 checks in 52.382 seconds). They cover closed outward topology/shading,
exact logical geometry and lossless packing, current budgets, hierarchy/inverse
binds, independent controls, coherent fitted layers, winding/strain/endpoints,
fixed waterline, posed occupied-volume roots, complete buried caps, uncrossed
coils, fitted vertices/edges/centres, genuine snout space through poses with a
filled-mouth negative control, and byte/source/species mutation rejection.
Winding/strain, fitting and overlap thresholds remain intact. The original
sculpture tests now enforce the actual 20k/40k skin budgets.

The ignored `target/pokemon-native-review/seahorse-skin-review.png` was inspected
inline. Each row compares the old static model with the candidate's quarter,
rear, front and independently decoded attack at 0.34. The original creature
silhouettes, openings and fin patterns remain readable. The exposed flat tail
root seen in the intermediate review is now buried, and the corrected curls
remain clear. This software review is authoring evidence; it does not prove
production visual acceptance. Native original-size stage admission, gameplay,
browser and broader world/camera/material acceptance are not claimed.
All rendering stayed offscreen. No windows, browser tabs, focus changes, shiny
checks, grass, AA/filtering, pack, audio or gameplay changes are included.

Reproduction:

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  crates/crystal-voxel-view/models/battle_species --only horsea,seadra,kingdra
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools \
  python3 -m unittest -v test_seahorse_sculpt test_kingdra_sculpt
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools \
  python3 -m unittest -v test_seahorse_glb
```

The normal background Blender builder publishes canonical GLBs and removes the
superseded static JSONs. Both direct sculpture CLI exporters produce identical
canonical bytes; their temporary duplicate exports were removed. Source checks
now read strict stored GLBs and compare float32 geometry pins. No duplicate
binary or geometry catalog is added. Test/builder logs, metrics/hashes and the
authoring sheet remain ignored under `target/pokemon-native-review`.

| Species | Logical float32 geometry SHA-256 | Packed float32 geometry SHA-256 |
| --- | --- | --- |
| Horsea | `1d71e4150b202fc44087225fd5faee2f7145de1e9908bb0f5300c1069614814c` | `9ced391e14c1462d6aec4e5fbb3cb7ea1882fe08cde7b7f9ab6f62a02c08c133` |
| Seadra | `85b7509d919860ae3f0cf71a914251e1aa2b962b9d355286d7f95b24fc60d1a5` | `77563b2fa80988f87c72a2e83f27d69fcd5eb992cbc3f8585d30819ed651aa89` |
| Kingdra | `4ab6120c7b38e50dbe6f6fb9576d64349656f9c94f9e9cfe495ebadf086642a6` | `630e46aff5bc1a55ef3e52dbacfb4617e6be6b2a5381389b8ee019795bfda69e` |

Canonical GLB SHA-256:

- Horsea: `2a6ccc8e6fc80598139141db2897ba86fc9a13e7d76cf792ad4f15376a44ae39`.
- Seadra: `fd2aed23c40327b625ca945737530fd21f847888b417ac190e2548024a1b11b2`.
- Kingdra: `e7fa23cad380dfcb115f32b42603fc56694c8cc55dd7218503da2a37ed8887c6`.

## Production integration checks

Five focused Rust checks pass in 1.99 seconds: canonical hierarchy/independent
controls and waterline, original-size GPU activation for all three species, and
complete world locomotion bounds with original fin/snout weights. The world
regression expects the existing tail-base yaw controller (joint 7), preserving
the authored coil weights; an initial assertion incorrectly expected no tail
controller and was corrected. No production locomotion heuristic changed.
The original physical sizes are Horsea 0.4064 m, Seadra 1.1938 m and Kingdra
1.8034 m. No model shrinking or gate relaxation is included.

The shared schema/physical-size check passes in 69.95 seconds and the shared
elapsed-idle/exact-current-attack cue check in 77.99 seconds. Together with the
26 stored Python checks and five focused Rust checks, 33 scoped checks pass.
The two production capture examples built successfully in 41.00 seconds.

The captured binaries use `location-tester,bevy/zstd_rust`. All native launches
set `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`, `CRYSTAL_PREVIEW_MUTE=1` and normal
party coloration. The headless path disables Winit and macOS primary-window
retention and uses the schedule runner. Binary preflight verifies the headless
and mute tags plus the exact compressed current GLB payloads. No visible
window or browser is used.

Example offscreen world reproduction from this checkout:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_PARTY_SHINY=0 \
CRYSTAL_RENDER_TEST_PARTY_SPECIES=SEADRA \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=1000x700 \
CRYSTAL_RENDER_TEST_CAMERA='3 2 35 0' \
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic CRYSTAL_RENDER_TEST_WALK_STEPS=0 \
./target/pokemon-models/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map Route36 --x 20 --y 8 --view both --hour 12 --walk D \
  --screenshot target/pokemon-native-review/seahorse-seadra-world.png
```

## Native paired review

Nine same-state production pairs (front, world and battle for each species)
and three moving-world frames were inspected inline. All captures were
1000×700, offscreen and muted. Six world/front receipts preserve authoritative
state and retain 32 movement rows spanning five actual Route36 positions,
(20,8) through (20,12). Three named battles preserve state, report no fallback
and pass complete terrain/frustum/omission admission at the original physical
sizes above, with zero support offset. Unsuitable scenery/border shots were
rejected before acceptance. No scale reduction, fake support or relaxed gate
was used. Both executable identities and embedded exact current GLBs remained
stable throughout capture.

The hollow mouths, fitted eyes/breasts, fin patterns and distinct head/crown
silhouettes remain readable in these views. Horsea is correctly much smaller
than its evolutions. Profile views conceal the far eye, fin layers and inward
coil opening; frontal views reveal the actual curl. Kingdra's moving frame has
a nearby sign overlapping the lower tail, leaving broader scene placement open.
Shared foliage detail and lighting remain broader presentation work. Existing
Float locomotion for these swimmers is retained. Browser, all camera/world/
material combinations and full gameplay acceptance are not claimed. No shiny
checks, grass, AA/filtering, audio or pack changes belong to this batch.

Evidence prefixes under ignored `target/pokemon-native-review` are
`seahorse-{horsea,seadra,kingdra}-{front,world,battle}`. Final pairs, raw images,
paired receipts, movement CSVs, three selected moving frames and logs remain.
Temporary producers and redundant intermediate captures were removed.

| Species / view | Frame / checksum | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| Horsea front | 1540 / 1443452487 | 1545 | 26.264662958 |
| Horsea world | 1455 / 4022501714 | 1457 | 24.792077084 |
| Horsea battle | 1733 / 4152974603 | 2290 | 39.147990458 |
| Seadra front | 1499 / 1090040784 | 1504 | 25.524218583 |
| Seadra world | 1374 / 3448261069 | 1379 | 23.438065833 |
| Seadra battle | 1519 / 2750188969 | 2128 | 36.380027334 |
| Kingdra front | 1739 / 2733538943 | 1742 | 29.64306025 |
| Kingdra world | 1520 / 2716464495 | 1526 | 26.021129624 |
| Kingdra battle | 1575 / 2574795240 | 2243 | 38.725930792 |

Stable executable SHA-256:
- `render_at_location`: `d779e93804ba88c96a99a3fb05f53fe3338b47f45ef6d2d17b061010ad496ceb`.
- `immersive_battle_3d`: `e41d9edd0acd63e8a6a4d9a217bb10a4ffc9f712065984ac0a6323b97a324da9`.
