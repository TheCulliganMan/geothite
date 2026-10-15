# Staryu sculpture and radial skin

Staryu now has one canonical battle body at
`crates/crystal-voxel-view/models/battle_species/staryu.glb`.
The original pure recipe in `tools/staryu_sculpt.py` builds a continuous,
volumetric five-arm lens, five shallow fitted radial ridges, a raised gold
collar and bezel, and a faceted red ruby with a small fitted white glint.
The world `icon_staryu` route shares this body; its old JSON geometry is removed.
Both actor and battle builder branches publish the same canonical GLB without
creating a second icon mesh. The archived Blender sources are unchanged.

The verified local pack supplies height digits 207, or 0.7874 m (2′7″).
Normal, front and back palettes agree: RGB555 `(23,17,11)` maps to
RGB8 `(189,140,90)` for the body, and `(31,5,3)` maps to `(255,41,24)`
for the core. The separate gold setting is an authored accent. Temporary
reference sprites and the Rust pack-reader helper were removed after review.

The GLB contains 10 named parts, 1,982 normal-split vertices, 2,852 triangles,
12 joints, 80 accessors and 132,772 bytes. Authored reference height is 1.0;
existing runtime reader limits remain unchanged. Neutral geometry digest:
`c8ef0c854e722452e6850b79cd7df6b66598e7b21488b69ba818c5dc80e87a67`.

The identity floor root anchors the two lower tips. A coherent core bone owns
the ruby, glint, collar and bezel. Each of the five arms has base and tip bones;
body and ridge weights use the same radial field. Distal lower-arm weights
return to the floor root, so the clips flex the upper arms without moving the
contacts. Idle lasts 3.2 seconds with 81 keys. Attack and hit last one second
with 65 keys, an early brace and a trailing arm-tip flex. Clips use rotations,
identity endpoints and the existing authoritative source battle cue clock.

Nine stored anatomy/skin checks pass. They cover closed outward geometry,
shading, all five radial extrema, both floor contacts, core/glint and ridge
clearance against actual stored triangles, the pack body/core colors, joint
ownership, clip endpoints, articulation, winding and strain. The apparent
marks in the neutral software review prompted a clearance check: every sampled
front ridge triangle is above the parent body, with at least 0.00445 clearance
in authored units. A stored-data regression retains a 0.002 minimum.

Two routing/catalogue checks pass. The pure authoring CLI was executed into a
temporary directory and matched canonical bytes exactly. Both Blender builder
branches were syntax checked; the full Blender batches were not executed.
Fresh Rust suites pass: 37 rig checks with one existing ignored check, 33 GPU
skinning checks and five actor-prop checks. The actor checks confirm that the
world icon and exact species select the same canonical body.

The production `immersive_battle_3d` example was rebuilt and inspected against
paired 2D with the supported Staryu/default-wild fixture. At 1000×700, the
receipt reports preserved state, checksum 3691665458, LCD animation frame 54
and virtual time 1.296023334. The native body, raised gold setting and red core
are visible at the verified physical height. The two lower tips support the
star. The default side camera compresses the planar arm span, so full
composition acceptance and more native camera angles remain open. The front
stored-geometry review shows all five arms. Browser rendering and detailed
source-frame motion acceptance also remain open.

```sh
python3 tools/staryu_glb.py
PYTHONPATH=tools python3 -m unittest test_staryu_glb -v
PYTHONPATH=tools python3 -m unittest \
  test_battle_model_assets.CanonicalSources.test_staryu_resolves_one_new_canonical_skin \
  test_battle_model_assets.CanonicalSources.test_starters_resolve_production_actor_rigs -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view \
  species_rig::tests --lib -j2 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0
```

There are now 34 registered standard skins and 243 dedicated battle bodies.
Eight catalogue gaps and the remaining static-body rigging, motion and visual
requirements remain within the full all-Pokemon goal. No pack or audio changes
were made for this model.
