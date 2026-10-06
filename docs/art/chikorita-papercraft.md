# Chikorita papercraft sculpture and animation

Chikorita now has one connected quadruped body, fitted facial surfaces, a
curved folded leaf and a seated collar. This replaces the separate spherical
head, body and feet and the diamond-shaped leaf in the earlier static model.

The canonical asset is `models/actor_props/battle_chikorita.glb` under
`crates/crystal-voxel-view`. It contains 19 anatomy parts, 3,220 triangles,
7,463 normal-split vertices and an 11-joint standard glTF skin. The previous
same-species JSON is removed. The existing multi-asset Blender archive remains
available; no additional generated Blender or STL copy is added.

`tools/chikorita_sculpt.py` authors the neutral panels and
`tools/chikorita_glb.py` regenerates the complete GLB without Blender or game
content. The Blender actor-kit builder imports those same panels for editing.

The clips provide planted breathing and a small head survey, a chest-driven
attack brace, and a hit recoil with delayed leaf settling. The leaf has three
articulated joints; all four soles remain rooted. Attack and hit progress comes
from the existing battle cues. The model does not change move programs,
projectiles, sounds, damage, encounter positions or camera policy.

The neutral full height remains 1.0 model unit. The original pack's Pokédex
height digits, 211, convert to 2 feet 11 inches or 0.889 meters through the
existing runtime conversion. Animation bounds are used only for visibility;
they do not rescale the body. The body is centered about its own X axis, so the
asymmetric leaf no longer shifts the body sideways during normalization.

The original Crystal front/back references informed the tall dark eye shape,
pale body, green collar and curved crown leaf. The three-dimensional hidden
surfaces and joints are authored geometry, not sprite extrusions.

`tools/test_chikorita_glb.py` reads the stored GLB and checks deterministic
regeneration, topology and winding, neutral scale, joint ownership, fixed
contact patches, clip seams and blends, floor clearance and skin quantization.
Rust regressions cover canonical asset selection, actual skin bindings,
shared resources, dense animation bounds, ground contacts and physical scale.

The disposable native fixture accepts `--enemy-gust --starter chikorita`.
It retains the natural level-25 learnset: Razor Leaf responds to the original
trainer AI's Gust, through the production controller and real PP/HP updates.
The fixture has no save destination.
