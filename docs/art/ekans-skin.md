# Ekans continuous coil and skin

Ekans replaces the generic tube, spherical head and four thin neck bands with
an original grounded purple coil, tapered snake brow/muzzle, fitted golden
eyes with dark vertical slits and ivory glints, small nostrils, a fitted closed
mouth, one broad collar, continuous golden ventral skin and three overlapping
raised tail segments. The user-supplied pack's normal front/back sprites guide
the anatomy. The neutral mouth is closed; the reference sprite's open-mouth
expression and exact palette parity are not claimed.

| Vertices / triangles | Parts | Joints | Accessors | GLB bytes | Authoring height |
| --- | --- | --- | --- | --- | --- |
| 8,481 / 14,536 | 16 | 5 | 90 | 378,092 | 1.18 |

The old static model had 1,178 vertices, 2,304 triangles and 13 parts.
All existing 20k vertex, 40k triangle, 64 part/joint, 256 accessor and 1 MiB
limits remain unchanged. The original longitudinal size calibration
`2.4854835 / 1.18`, original pack dimension, uniform scale and full animated
bounds remain in force. Raised tail geometry is included in fit bounds.

The five controls are floor root, raised neck/body, head, tail base and tail
tip. A sampled parallel-transport tube forms the complete coil. Longitudinal
weights choose the actual closest ring surface, preserving the lower coil
on the root while blending the raised neck and tail. The collar copies actual
body ring frames and diagonals, with a shallow overlapping back; its weights
follow the same longitudinal query. The face shares the head carrier and the
three tail segments share the terminal carrier. Idle lasts 3.4 seconds/85 keys;
attack and hit last one second/65 keys. All tracks rotate, use source-relative
cues and return to neutral at their endpoints.

The initial thick neck-to-coil bend crossed itself. The actual bend radii
were narrowed while retaining the broad low coil; the independent surface
intersection check now passes, including posed body checks. No parser limit
or geometric tolerance was relaxed.

Sixteen Python checks pass in 11.169 seconds: closed connected outward solids,
finite shading/winding, dimensions/budgets, characteristic eyes/collar/rattles,
uncrossed coil, occupied roots with detached negative controls, fitted feature
vertices/edges/centers, collar/ventral surface support, independently decoded
source parity/hierarchy/inverse binds, all controls moving, coherent face/tail
carriers, fixed floor, positive posed winding/bounded strain, posed roots and
collar support, source cues/endpoints and mutated bytes/source/species rejection.
Three focused Rust checks pass in 0.86 seconds for original-size GPU admission,
canonical geometry/fixed floor across clips, and preserved world bindings,
grounded contacts and full locomotion bounds. The shared anatomy/order/physical-
size schema check passes in 106.68 seconds, making twenty scoped checks overall.
These are not a full-suite or gameplay-completion claim.

The direct sculpture CLI and background Blender builder reproduce the
canonical asset byte for byte. Both native clients build in 1 minute 5 seconds.
The common canonical model reader resolves the GLB without a duplicate JSON:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/ekans_sculpt.py \
  --output target/ekans-rebuild
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  target/ekans-rebuild --only EKANS
```

Source and packed SHA256 are identical:
`eaf2ebad196f288b8c3827336f2ca7c90dd56070ced67699b45e13427dfbb6e4`.
GLB SHA256:
`7d73c37140684a2973ef7c3c9632b0a54cd6433a7b0784a14e55a6c3cb6a1a02`.
The decoded neutral/attack source sheet, three same-state production 2D/3D
pairs and two moving world/front frames were inspected inline, entirely
headless and muted. Front/world capture clocks are 113.612/154.957 seconds;
battle's native second-image completion clock is 191.78 seconds. The battle
admits complete Ekans and Pidgey models without fallback, preserves the
authoritative state and names both species in the HUD. Its original 2.0066 m
length maps through the unchanged longitudinal calibration with uniform scale
2.1394186 and physical support zero.

The first helper postcheck assumed standing height, so it rejected this valid
longitudinal dimension after saving both battle images. The helper now applies
the existing Ekans ratio. The fresh saved images, paired-state receipt, accepted
stage, uniform scale, physical length and binary identities were independently
audited; no render or production sizing change was needed. The recovered
receipt explicitly records this postcheck and its native timing source.

Front and battle views clearly expose the fitted gaze, collar, low coil and
raised tail. The neutral mouth remains closed and visually subtle. The world
camera at rotation step 6 still has foreground branches across the animal;
the moving frame exposes it through existing foliage fading. Broad world
visibility and expression/palette parity remain open.

Both clients retain their exact SHA256 identities throughout all captures:
`render_at_location` is
`b2ef26e2f24e6f72416fecc5fb7d52b9738bd20d60e184ad9705b05e591fc2ec`;
`immersive_battle_3d` is
`035e68b7a4844197da2e560292280f732a8784f00e95adcfb747259adcf5b938`.
Receipts and local review images remain ignored under
`target/pokemon-native-review/`. Registry count is 256 skin variants and
19 static species files.
No shiny, grass-height, AA, audio, pack or gameplay changes belong to this batch.
Full-roster model and rig improvement remains active.
