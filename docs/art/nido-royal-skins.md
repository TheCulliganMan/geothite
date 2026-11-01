# Nidoqueen and Nidoking sculptures and skins

Current camera/follower follow-up: Nidoking’s reviewed first-person horn clipping is corrected by the shared
trainer-eye and physical following-distance change. The original stationary
Route36 location and a walking endpoint both retain the complete horn.
See [physical companion spacing](pokemon-following-distance.md) for the
current checks, captures and remaining limitations. Earlier review records
below retain their original evidence and conclusions.


The original blue queen and purple king now use canonical standard GLB skins.
`tools/nido_royal_glb.py` exports the existing species recipes through the normal
builder, strict reader and Rust registry. The retired static JSON assets are
removed.

| Species | Logical solids | Runtime primitives | Vertices | Triangles | Joints | Accessors | GLB bytes | Reference height |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Nidoqueen | 61 | 39 | 18,397 | 30,024 | 17 | 215 | 754,588 | 1.40 |
| Nidoking | 70 | 44 | 19,990 | 32,920 | 19 | 242 | 822,952 | 1.40 |

The retired versions used 55,955/48,831 vertices and 85,324/74,756 triangles,
respectively. Planar ear subdivision is removed, fitted layers follow the
actual parent triangles, and redundant radial samples are reduced on rounded
solids. Authored profiles, species colors, floor zero and full height remain.
Nidoqueen retains rounded brown-lined ears, a short horn, paired cream chest
shields, four abdominal plates and two claws per foot. Nidoking retains
ragged teal-lined ears, its large forward horn, cheek barbs, an actually open
jaw with six teeth and tongue, broad single toe claws, and three tail collars.
The queen's fitted chest shield is 0.013 authoring units thick to keep the posed
surface within the same stand-off check as the other fitted layers.

Both skins have an identity floor root, independent ears and horn, three
controls per arm, separate hindquarters and three tail controls. Nidoking also
has a lower-jaw control with its lower canines, and a child tongue control.
Closed queen snout, cream chin and mouth seam move together with the head.
Feet and complete toe claws stay fixed during canonical battle clips; ankles
blend into their supported leg controls. Body armor shares the body carrier.
The low heavy tail, collars and spines share a monotone longitudinal field;
torso and tail controls yaw to retain actual floor clearance. Idle is 3.2 s
with 81 keys, and attack/hit are 1 s with 65 keys, with neutral endpoints and
source battle cues retaining timing ownership.

Packing retains every source-solid range and original order. Hand and foot
claws remain separate per limb so the production world gait can assign each
whole runtime primitive to its own support tree. Limits remain 20,000 vertices,
40,000 triangles, 64 primitives, 64 joints, 256 accessors and 1 MiB.

| Species | Logical geometry SHA-256 | Packed geometry SHA-256 |
| --- | --- | --- |
| nidoqueen | `f9b60413bc6f14f2980c440f61f8d7ce304219d68853bb4c160456b855793307` | `5b9950cf9bb193673825a3db9214c65d3f2eb037367486e563ec310be35d6132` |
| nidoking | `99b3efee252edb4d2e162769e3f8d55f4f958cf55f4ee7befd2b240da698f8cd` | `0730055ac57991e822b343db39eb9020d0416bbd179956f8e49ee721f937c346` |

Canonical GLB SHA-256:

- Nidoqueen: `6f4292e9fc7c5dbaad296c68ee486e175aeec66ebd049f1c1c83e273fc49d0ad`.
- Nidoking: `71983dd428f29c4ed48b5633d1f899762484c15b27fc2dbdc3d7a32e8428c771`.

Verified authoring and runtime checks:

```sh
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools python3 -m unittest -v \
  test_nidoqueen_sculpt test_nidoking_sculpt test_nido_royal_glb
CARGO_TARGET_DIR=target/pokemon-models \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
  cargo test --locked -p crystal-voxel-view nido_royal --lib -j1 \
  --features bevy/webgl2,bevy/bevy_winit,bevy/x11,bevy/bevy_gilrs,bevy/zstd_rust,crystal-assets/test-fixtures \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0 \
  --config profile.dev.package.crystal-assets.opt-level=2 \
  --config profile.dev.package.crystal-core.opt-level=0
```

All 28 Python checks pass: exact export/reader parity and pins, independent
hierarchy and inverse binds, closed outward connected solids, shading and
triangle winding, floor/size/budget, species anatomy, occupied-volume roots,
empty jaw, fitted cap vertices/edges/centers on posed parents, clip strain,
neutral endpoints, independent control motion and mutation rejection.
All four focused Rust checks pass, including GPU skin activation at
1.2954/1.3970 m, canonical support/floor/bounds, and two production walking
support trees with per-limb claws over 64 gait phases. Background Blender
builder outputs match the canonical GLBs byte for byte; strict catalog readers
return all 61/70 logical solids. Software before/after, front and attack views
were inspected inline without opening a window.

Browser and broader moving-world, camera, follower-spacing and aliasing review
remain open. No shiny checks or grass/AA/filtering changes are included.

## Offscreen production review

The shared registry schema and exact current battle-cue GPU regression both
pass (72.49/86.55 s). Together with the 28 Python and four focused Rust checks,
34 unique targeted checks pass. The native examples build in 46.69 s.

Before capture, both executables were checked for the strict headless guard,
preview-mute path and exact compressed current canonical GLB payloads. Native
source disables Winit, uses ScheduleRunner, and skips macOS primary-window
retention in headless mode. Every run sets:

```sh
export CRYSTAL_RENDER_TEST_HEADLESS=1
export CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image
export CRYSTAL_PREVIEW_MUTE=1
export CRYSTAL_RENDER_TEST_PARTY_SHINY=0
export CRYSTAL_RENDER_TEST_WINDOW_SIZE=1000x700
export CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=300
export CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic
export CRYSTAL_RENDER_TEST_WALK_STEPS=0
```

Native build:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked -p crystal-bevy \
  --features location-tester,bevy/zstd_rust \
  --example immersive_battle_3d --example render_at_location -j1 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0 \
  --config profile.dev.package.crystal-assets.opt-level=2 \
  --config profile.dev.package.crystal-core.opt-level=0
```

Each species runs Route36 `(20,8)` at hour 12, `--view both`, first at camera
`0 0 20 1`, FOV 100, then camera `5 2 30 0` with `--walk D`. Each world CSV has
32 actual movement ticks across five authoritative coordinates in Route36.
The actual moving-00 image for each species was inspected as well as all six
final 3D/2D pairs, rendered at 1000×700 per view. Moving readback is asynchronous;
those extra images supplement the final frozen same-state pairs and numerical
64-phase support checks. They are not separate time-matched 2D comparisons.

Battle captures use the existing browser pack with the normal native production
client, `--walking-encounter --walk` with `RALA` repeated 20 times, Route29,
`CRYSTAL_RENDER_TEST_HOLD_BATTLE=1` and `CRYSTAL_ENCOUNTER_TRACE=1`.
The six final receipts all preserve authoritative state:

| Species / capture | Checksum frame / hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| nidoqueen / front | 1531 / 1850923357 | 1531 | 25.907560458 |
| nidoqueen / world | 1558 / 1831562114 | 1561 | 26.881860916 |
| nidoqueen / battle | 2233 / 494209253 | 2853 | 49.077931667 |
| nidoking / front | 2179 / 3606267956 | 2179 | 37.042521209 |
| nidoking / world | 1880 / 820569544 | 1888 | 32.151991416 |
| nidoking / battle | 2744 / 2774479070 | 3481 | 59.648671834 |

Both named encounters show the original physical height, zero support offset,
full terrain admission with `accepted=true reason=None`, and no fallback.
Whole-body geometry, animated bounds, frustum and omission checks remain intact.

| Species / opponent | Metres | Scale Y | Reference height | Support Y | Checked / unsupported stage candidates |
| --- | --- | --- | --- | --- | --- |
| nidoqueen / SENTRET | 1.2954 | 2.4520073 | 1.4 | 0.0 | 203 / 8 |
| nidoking / SENTRET | 1.3970 | 2.6443214 | 1.4 | 0.0 | 254 / 16 |

The wider world and battle views show complete grounded models, readable
species silhouettes and attached limbs, armor and tails. Nidoqueen's close
first-person view retains her face, horn and armor. **Nidoking's close
first-person view clips the forward horn at the top edge.** That view is not
visually accepted: camera/follower distance remains a follow-up, with authentic
model dimensions retained. It does not occur in the reviewed wider world and
battle views. Broader world/browser/camera acceptance remains open.

The executable identities remained unchanged through all six captures:

- render_at_location: `c6eac0facbca86fecc1a5789464575718349d1aa331b714563f7d0ff3c2b73c4`.
- immersive_battle_3d: `3bbb46fa5ad0a16675d85fa764a3e365c6c72fc171ac9297ad84445bc5f15346`.

The ignored `target/pokemon-native-review/nido-royal-` evidence prefix retains
software/native PNGs, final paired-state receipts, world movement CSVs,
executable identities and final test/build logs. Temporary source backups,
old JSON copies, helper scripts, duplicate exports and battle-intro frames
are removed. Rendering stayed offscreen and muted throughout; no windows,
application activation, shiny checks, pack publication or audio changes were
performed for this batch.
