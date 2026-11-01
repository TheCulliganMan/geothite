# Misdreavus sculpture and standard skin

`tools/misdreavus_sculpt.py` authors the flowing ghost anatomy, and
`tools/misdreavus_glb.py` exports canonical `battle_species/misdreavus.glb`.
The species builder emits this standard skin and retires duplicate static JSON.
Production mesh lookup and GPU animation share its exact neutral geometry.

All 41 original closed parts preserve exact stored f32 positions, normals and
triangle order: 14,175 vertices and 23,794 triangles. The GLB is 567,252 bytes,
with 15 joints and 168 accessors, inside the existing loader budgets. Its pinned
geometry/material digest is
`65520ddfee63c5e0fa3d39643396f88dfe2b9e2fbefd7bc24355cc2491b59935`.
Accessor reuse compares packed bytes, preserving signed-zero normals in the
five similar gem meshes. The normalized 1.02 height and floor origin remain
unchanged; local pack height 204 is 2′4″ (0.7112 m), applied uniformly.

Actual local normal front/back sprites and the RGB555 palette were inspected.
Blue (9,9,17) becomes sRGB8 (74,74,140), and pink (30,6,19) becomes
(247,49,156), with white and black. This replaces the earlier invented teal,
cream and ruby colors. White sclera surround pink irises; the locks, gems and
highlights use the source palette. Historical part labels retain warm gold,
ruby and rose wording for continuity. No extracted sprites ship with the model.

An identity root preserves the five low hem lobes. The body blends upward into
a yawing body joint and coherent head. The fitted eye layers and smile, all
five graduated gems, their fitted highlights and both necklace cords share the
head owner. Each of six asymmetric hair locks has a base and delayed curl
joint. Blue lock and pink tip weights use the same position-based field,
keeping all five complete 24-vertex color seams closed throughout motion.
A wider base transition prevents attack strain without simplifying the curls.

Idle, attack and hit clips use rotations only, neutral endpoints and source
battle-cue timing. All seven stored-skin and nine migrated anatomy checks pass,
including canonical reconstruction, bind identity, colors, hem support, seam
closure, fitted feature ownership, winding, edge strain, closed topology,
physical joins, expression fitting and real graduated necklace geometry.
Front and rear neutral software views were inspected. Focused Blender output
is byte-identical and emits no duplicate static JSON. Both focused production Rust checks pass: GPU skinning at pack size and
off-grid seam closure, coherent necklace, hem support and culling checks.
Matched normal native 3D/2D review against Sentret shows a clear spectral
face, graduated necklace and swept curls against tall grass; no color-seam gaps
are visible. Rear curls were also inspected in neutral software views. The
paired capture preserves core frame 3135, hash 3238905079, LCD frame 3849 and
time 71.5263535 (`state_preserved=true`). Images and receipts remain ignored
local QA artifacts. Exact sprite-frame motion,
field gaits and browser review remain open. Shiny checks/captures are skipped
at the user's request.

The wider `every_exact_species_is_grounded_and_volumetric` and
`body_pose_uses_elapsed_idle_time_and_exact_current_attack_progress` regressions
also pass on this asset. They establish catalog geometry and elapsed-pose
coverage, without claiming visual acceptance for unrelated models. The native
client build passes.
