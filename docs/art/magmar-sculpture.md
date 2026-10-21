# Magmar sculpture and skin

The current [volume and posture audit](elemental-fidelity.md) records references,
full orbits, pose samples and verification. The canonical asset is
`crates/crystal-voxel-view/models/battle_species/magmar.glb`, authored by
`tools/magmar_sculpt.py` and exported/strictly read by `tools/magmar_glb.py`.
No downloaded geometry, texture, rig or animation is an authoring input.

Magmar has a deep pear body and fuller low yellow haunches above short upright
dark shins. Both arms bend down and forward, with jagged cuffs following their
actual curved forearms and three claws on each hand. The yellow lower torso
wraps through the sides, rear and underside, with rising flame edges clipped
from the actual torso triangles. This avoids the visible intersections of an
independently triangulated color loft.

The broad golden bill has a real opening, dark backing and tongue. Twin brows,
fitted narrow eyes and three dorsal points remain. Both forked crown flames and
the tail flame swell into actual profile volume, with fitted front and rear
cores. Their full cross-section thickness replaces the previous flat banks.
Seventeen joints follow the revised shoulders, elbows, wrists, legs, crown and
tail. Fitted layers share their parents' fields or complete carriers. Battle
feet and toe claws stay fixed; the world walking adapter builds two complete
support chains without rebinding upper anatomy.

| Logical/runtime parts | Vertices | Triangles | Joints | Accessors | Bytes |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 55 / 39 | 19,024 | 29,200 | 17 | 219 | 803,244 |

Authoring height remains 1.30 and production pack height remains 1.2954 m.
Existing limits remain 20,000 vertices, 120,000 indices, 64 runtime parts,
256 accessors and 1 MiB. Sampling reductions target broad round surfaces;
flame silhouettes, actual thickness and fitted details retain focused topology.
