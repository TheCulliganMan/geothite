# Larvitar and Pupitar canonical skins

The current [Larvitar volume review](rhino-larva-fidelity.md) replaces the narrow
bill, invented palms/fingers and three upward tail leaves. Pupitar's authored
mesh, rig, clips and canonical bytes are unchanged by this revision.

| Model | Logical / runtime parts | Vertices / triangles | Joints | Accessors | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Larvitar | 43 / 21 | 7,394 / 10,816 | 11 | 127 | 340,684 |
| Pupitar | 28 / 9 | 4,503 / 5,224 | 4 | 58 | 204,580 |

`tools/rock_pupa_sculpt.py` authors both meshes, and `tools/rock_pupa_glb.py`
exports their canonical skins. The regular Blender builder and strict readers
use the same bytes. No game geometry or textures enter these original assets.

Larvitar has a deep pear-shaped skull and fuller neck/body, shorter relative
crown, fitted angular red gaze, black cheek tears, a closed mouth, red belly
diamond, front vents and rear-flank diamonds. Two short tapered arms replace
palms and fingers. Compact haunches, flat planted pads and six short toes provide
support. The thick short tail ends in six angular radial teeth around a solid
green hub with a shallow rear depression. The green hub deliberately omits a
stray white eye island in the extracted reference's texture.

Larvitar's eleven controls comprise a ground root, torso, skull, two shoulders,
two wrists, two hind limbs and two tail controls. Facial details and horn share
the skull. Spatial arm blends retain independent useful wrists inside the
conical stubs. Front/rear markings share the body. Pads/toes stay fixed in
source clips; thigh blends lead into the hind controls. The stem has a base
control and the fan teeth/hub share the tip. Whole named supports walk in the
production locomotion rig without rebinding the face, arms or tail.

Pupitar retains a faceted tapered blue core, angular visor with visible fitted
gaze, three crown points, six side points, front/back chevron seams and pointed
lower fold. Its four controls are ground root, core, visor and lower fold.
Crown/upper side points share the visor; lower side points and seams share the
core. Core and keel blends preserve their grounded lower vertices. Pupitar
has no invented arms or feet, and retains the established grounded locomotion.

Both skins use rotation-only idle (3.2 seconds/81 keys), attack and hit (one
second/65 keys) with source-cue timing and neutral endpoints. Lossless material
packing retains ordered logical ranges. Existing limits remain 20k vertices,
40k triangles, 64 runtime parts/joints, 256 accessors and 1 MiB. Authoring heights
remain 1.04/1.20; physical pack heights remain 0.6096/1.1938 m.

| Model | Logical float32 digest | Packed float32 digest | GLB SHA-256 |
| --- | --- | --- | --- |
| Larvitar | `427f80e604f64b3939804ea5f70602622dbeb7d3c322beaada67965d9d3e1111` | `3290dc9d6395dccea910d5c48674b1c4a4d4d11dbc9fc5a0a86c48fda58ae33b` | `985b4cdb811c8584530b4d2d793b47eb9a17c5814df78b873cf2e7ee3f9bfecd` |
| Pupitar | `1905fcbbb5bdf9066860f8fb4d6b5c2a3646da2fd4098cd3fc75a43c59d1e6b7` | `b972d03ec14adb898e754bb68511dad4e7f7f131aee538229c455b77ecc3a492` | `bec27e6a6b7bb2d3ee42a7b7581284c0abff812d34d20a85e0b80900e032da34` |

The fidelity review records the current eight-angle references/candidates,
actual pack front/back pictures, Python checks, sampled poses, native evidence
and remaining limits. Source and skin checks retain closed outward solids,
actual roots, fitted front/rear marks, visible gazes, all independent controls,
positive winding/strain, fixed supports and strict canonical-byte rejection.
Independent fidelity guards measure skull/crown ratios, the arm volume and
six-direction tail distribution; the former Larvitar fails those guards.

```sh
PYTHONPATH=tools python3 -m unittest -v \
  test_rock_pupa_sculpt test_rock_pupa_glb test_rhino_larva_fidelity
/Applications/Blender.app/Contents/MacOS/Blender --background \
  --factory-startup --threads 2 --python-exit-code 1 \
  --python tools/build-battle-species.py -- \
  target/rock-pupa-builder-review --only larvitar,pupitar
```
