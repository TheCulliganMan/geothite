# Exeggcute and Exeggutor articulated skins

The canonical production assets are `exeggcute.glb` and `exeggutor.glb` in
`crates/crystal-voxel-view/models/battle_species`. Both use the shared Rust
`SpeciesRig` path. `tools/psychic_palm_glb.py` exports and strictly reads the
skins; the build dispatcher and roster reader select those same assets. The
obsolete static JSON copies are retired.

| Species | Vertices | Triangles | Logical / packed parts | Joints | GLB bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Exeggcute | 19,868 | 35,472 | 46 / 17 | 7 | 826,108 |
| Exeggutor | 19,278 | 30,396 | 70 / 24 | 30 | 882,084 |

Both retain all six original neutral bounds at authoring height 1.22. The
source pack records heightDigits 104 / 607, giving physical heights of
0.4064 / 2.0066 metres. Scaling remains uniform and independent of animation.
Material packing retains every named source range and its original f32
geometry, normal, color and triangle order.

The opt-in `rig_ready` sculpture recipes keep the original six rounded shells,
hollow jagged broken shell and rooted yolk, plus the palm's trunk, ankles,
feet, claws, coconut heads and folded fronds. Adaptive facial caps remove
unnecessary subdivision. Round eyelids, cracks and bands share radial normals.
Conforming foot pads retain the original outward extent with positive
clearance. Palm midribs widen from .0012 to .0032 with .0026 clearance so they
remain continuous at gameplay scale; leaf blades and neutral bounds stay fixed.
Default sculpture recipes remain available as the historical authoring baseline.

Three grounded front eggs support three independently moving rear eggs. Each
expression follows its own shell; the broken shell and yolk share one frame.
The palm has a trunk/crown chain, three independent coconut faces, two ankles
and eleven frond base/tip chains. Each midrib shares its blade's longitudinal
field. Complete feet, pads and claws remain fixed during battle. Idle, attack
and hit clips use rotation-only source-relative motion and neutral endpoints.
Game programs continue to own battle events, timing, projectiles and audio.

Thirty-five scoped checks passed: 17 installed-asset anatomy checks, 14 decoded
skin checks, two focused Rust integration checks and two shared schema/current
cue checks. They cover actual solid roots, hollow shell/yolk, distinct visible
expressions, coherent fitted detail, independent controls, fixed supports,
floor clearance, winding, bounded strain, reader limits and canonical rejection.
The background Blender build dispatcher reproduced both GLBs byte for byte.

Six selected native 2D/3D pairs and both walking frames were inspected offscreen.
Field views use EcruteakCity (18,21), noon and matching facing/state. Each field
capture retains 32 authoritative movement rows over five origins. Exeggcute's
battle pair uses Route29; Exeggutor's selected battle pair uses NationalPark.
Its Route29 attempt was rejected because no readable higher shot fit checked
source ground. NationalPark passed all admission guards at the original size,
with no fallback. The latter search took 714.805 seconds. The six selected
receipts, current GLB hashes, uniform physical sizes and stable native binary
identities passed an independent audit in ignored review space.

Front views retain all six eggs, five expressions and the broken shell/yolk,
and the palm's three faces and complete clawed feet. Both walking views show
grounded bodies. The retained palm canopy remains narrow at a precise side
angle; broader model fidelity work remains active. Intro transition captures
are diagnostic and are excluded from motion acceptance. No dedicated shiny
checks, pack, terrain, camera or audio changes were made in this migration.

Re-export into ignored review space:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/psychic_palm_glb.py \
  --output target/pokemon-native-review/psychic-palm-rig-candidates
PSYCHIC_PALM_SKIN_MODELS="$PWD/target/pokemon-native-review/psychic-palm-rig-candidates" \
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools \
  -p test_psychic_palm_glb.py -v
```

Pinned SHA256 values:

exeggcute:

- source_sha256: `ade3f296f60113b7cc3d155aa94d30b0ba2782b42af8e9f50d5986688d63eb73`
- packed_sha256: `627e7883a07875496f6221bad5ca019ee5a4904ed8071cc21d714d0d60a3df7b`
- glb_sha256: `2cb9ba417e5d0e57dbc2595ee90dc447fd058126d345f9443c48bcdc27282098`

exeggutor:

- source_sha256: `ec29d1bd63c9e9a5f88998f1e0ac8a846f51f2632680fc277bd31aeeb0126364`
- packed_sha256: `22e3a7b6c82b33b797f06d8e8d1e143dfbea7063adfe20f3c345da215d7053ea`
- glb_sha256: `c1bc1439cf9b2b91ff31f9dee85463318d4e9f89c248d8ca4cab8558b5a08a87`
