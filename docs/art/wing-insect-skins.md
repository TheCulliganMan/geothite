# Butterfree and Beedrill standard skins

The original `tools/wing_insect_sculpt.py` recipes now publish canonical GLB
skins through `tools/wing_insect_glb.py`. The retired JSON copies are removed.
Original f32 positions, normals, materials, part names and triangles are
preserved. Butterfree retains its violet body, compound ruby eyes, blue mittens
and feet and four broad white wings. Beedrill retains its striped gaster,
petiole waist, rear stinger, forearm lances and four narrow translucent-coloured
wing solids. The assets are authored geometry, not extracted game content.

| Species | Parts | Vertices | Triangles | Bytes | Joints | Accessors | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Butterfree | 59 | 5,428 | 9,420 | 296,184 | 18 | 254 | 1.0922 m (3′7″) |
| Beedrill | 42 | 4,304 | 7,304 | 265,608 | 20 | 217 | 0.9906 m (3′3″) |

Both authoring reference heights remain 1.04. The verified local pack supplied
height digits 307 and 303. Reader limits and physical sizing are unchanged;
shared index/skin accessors keep both assets inside the existing bounds.

Neutral digests:

- Butterfree: `4b94b9ecfabca7dee88ec0d5b725d50116cb1f2a139d5a2817342e53cd4ee550`.
- Beedrill: `998ef3cf0f5f76c6ada93622528f1806dab5361c60d1b8ecb43c991f1979b900`.

The thorax, head and abdomen have separate owners. Compound eye sockets,
lenses and reflections stay rigid on the head, as do the mouth and proboscis.
Each of four wings has root and tip joints. Rim, inset membrane and veins share
one continuous spatial field for the corresponding wing pair. Two antenna
chains follow the head. Butterfree's arms and mittens share arm bones, while
its legs and foot pads remain on the floor root. Beedrill adds separate drill
bones and a blended upper-arm/wrist field. Gaster stripes follow the abdomen;
the petiole blends abdomen and thorax, and the low stinger tip retains the root.

Idle lasts 2.8 seconds with 85 keys. Attack/hit last one second with 65 keys.
The thorax/head brace early, with trailing wing tips and antennae. All clips
have identity endpoints and use rotations on the existing source battle cue
clock. Initial in-plane wing motion inverted a narrow perimeter triangle;
out-of-plane root/tip motion replaced it before visual acceptance.

The pure authoring CLI was executed and reproduced both canonical files byte
for byte, removing seeded obsolete JSON. The Blender batch branch publishes
GLB for these two while preserving the Metapod/Kakuna JSON path. It was syntax
checked; full Blender batch execution remains unverified.

```sh
python3 tools/wing_insect_glb.py
PYTHONPATH=tools python3 -m unittest test_wing_insect_glb test_wing_insect_sculpt \
  test_battle_model_assets.CanonicalSources.test_wing_insects_resolve_one_canonical_skin_each -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked \
  -p crystal-voxel-view wing_insect --lib
```

All 12 focused Python anatomy/skin/routing checks and both fresh Rust
registry/GPU integration checks passed. The skinning suite passed 30 checks and the actor suite passed 13. The initial broader
rig run hit five anatomy-count errors after Totodile changed from 27 to 30 parts
on disk after the executable was built. A fresh Cargo build against current
code/assets passes all 34 active rig checks (one existing audit ignored).
The same fresh executable also passes all 30 skinning and 13 actor checks.
The fresh production build finished and both native battle pairs were inspected
against 2D at the same authoritative state and clock. Each receipt preserves
state.

| Battle pair | Hash | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: |
| Butterfree | 2972568340 | 42 | 1.018563792 |
| Beedrill | 1964747682 | 39 | 0.928452209 |

Butterfree's wing veins, compound ruby eyes, blue mittens and foot pads remain
readable. Beedrill's forearm lances, four wing solids and striped gaster remain
distinct. Both retain their verified pack heights. The source back sprites
show a bluer Butterfree and a more orange Beedrill than these preserved
materials. Source-palette fidelity and far-wing readability remain open; the
pairs verify rig integration and anatomy, not final all-species visual acceptance.

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked -p crystal-bevy \
  --features location-tester --example immersive_battle_3d
# Run once for BUTTERFREE and once for BEEDRILL, with matching lowercase filenames.
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
./target/pokemon-models/debug/examples/immersive_battle_3d \
content-packs/core-modular.browser.crystalpack --wild-species BUTTERFREE --size 1000x700 \
--screenshot target/pokemon-native-review/butterfree-battle-3d.png \
--second target/pokemon-native-review/butterfree-battle-2d.png
```
Production visual acceptance, browser verification and detailed source-cue
motion review remain separate requirements. Registering these two rigs does
not prove that all Pokemon models and rigs are complete.

### Resting world Butterfree review (2026-10-14)

Butterfree NPCs use a resting wing pose on the existing canonical skin. Each
wing panel rotates around its root hinge as a coherent panel; followers and
battle models retain the canonical articulated weights. The derived NPC rig
has an Idle envelope that includes the static hinge rotations. Its wing-only
material uses diffuse transmission 0.5 so the membrane pattern remains
readable while the opaque body retains its existing material.

The verified local pack identifies Butterfree's icon as `ICON_MOTH`, backed by
`gfx/icons/moth.png`. Its two 16px frames have non-background extents of
`(0,2)..(15,13)` and `(1,1)..(14,14)`: heights 12px and 14px. NPCs published as
`species:BUTTERFREE:icon_moth` now use the maximum 14/16 height relative to
their source card, divided by the canonical model height. This gives a stable
scale across icon animation without changing the source actor position,
facing, dialogue, collision or follower scale.

The previous native Route34IlexForestGate comparison at (5,4) confirmed readable
folded wing patterns with preserved state, but still showed crowding beside
the teacher. The focused resting-NPC regression passes with the source-sized scale: fixed
feet, complete folded Idle bounds, unchanged follower scale, shared wing joints,
material separation and recursive retirement. The production rebuild passed. A fresh paired capture at (5,4) in
Route34IlexForestGate preserves state (checksum 2940242446, LCD frame 223,
virtual seconds 4.403709791). At the default camera angle the folded membrane
pattern is readable, the body and feet remain supported by the counter, and
the reduced silhouette clears the teacher's face. The alternate orbit 1 pair
also preserves state (checksum 1597392490, LCD frame 398, virtual seconds
7.417859708). Its wings appear edge-on but stay attached and visible; no culling
loss was observed. These two production comparisons accept the resting NPC
pose and source-sized scale. The nameplate slightly overlaps the teacher's
lower body from orbit 1, so crowded caption placement remains open. Broader
source-palette and browser acceptance are still open. Species coverage and the
all-Pokemon completion status are unchanged.
