# Murkrow sculpture and standard skin

`tools/murkrow_sculpt.py` authors the original witch-crow anatomy, and
`tools/murkrow_glb.py` exports canonical `battle_species/murkrow.glb`. The species
builder emits this skin and removes duplicate static JSON. The production
species registry, immutable mesh lookup and generic GPU skinning share this GLB.

All 62 original closed parts retain exact stored f32 positions, normals and
triangle order: 13,359 vertices and 16,616 triangles. The 533,448-byte GLB has
11 joints and 239 accessors, within the existing loader budgets. Its source
geometry/material digest is
`1eb3ca435d7fe293a9371188874d4c5e7007e7f2ad87cddd52b1e3202fbbf132`.
The normalized 0.78 height and floor origin are unchanged. Local pack height
108 is 1′8″ (0.508 m), applied through the standard uniform scale.

The actual local normal front/back sprites and RGB555 palette were inspected.
Feather violet (7,7,17) becomes sRGB8 (57,57,140); gold (26,26,3) becomes
(214,214,24), alongside white and black. Violet replaces the earlier invented
blue shades; gold also supplies the iris, broom binding and talon tips.
Historical part labels retain red/white wording for migration continuity.
No pack sprites or generated reference images are shipped with the model.

The torso yaws above fixed feet, eight toes and eight talon tips. Shins blend
from the fixed root into the torso. The narrow neck and pointed throat feathers
share a height-based transition into the head. The fitted head, sockets,
whites, iris, pupils, glints, upper bill, nostrils and hat brim stay coherent;
the lower bill and dark seam share a separate owner. Each folded wing carries
all three feather vanes rigidly. Three hat feathers settle independently.
The tail stem blends along its length into a base joint and delayed broom fan;
the binding follows the base and every fitted fan crease follows the fan.
An initially short weight transition pinched the stem during attacks; widening
it along the stem removes that strain without changing its geometry.

Idle, attack and hit clips use rotations only, an identity root and neutral
endpoints. Authoritative battle cues own timing. Six stored-skin and nine
migrated anatomy checks pass, covering canonical reconstruction, bind identity,
source colors, rigid feature ownership, planted digits, winding and edge strain,
closed geometry, fitted eyes and nostrils, wing layers and all physical joins.
Front and rear neutral software views were inspected. Both focused production Rust checks pass: GPU skinning at pack size and
off-grid planted-digit, feature-motion and culling checks. Final focused Blender
output is byte-identical and emits no static JSON. The native build passes. Matched normal 3D/2D review against Pidgey shows
planted gold toes and readable hat, folded wing layers and bound broom tail
against the taller grass. Near-side details read clearly; rear details were
inspected in neutral software views. The paired receipt preserves core frame
2544, hash 1815780038, LCD frame 3204 and time 57.765340872
(`state_preserved=true`). Captures remain ignored local QA artifacts.
Exact sprite-frame motion, field gaits and browser review remain open.
Shiny verification and captures are skipped at the user's request.

The wider `every_exact_species_is_grounded_and_volumetric` and
`body_pose_uses_elapsed_idle_time_and_exact_current_attack_progress` regressions
also pass on this asset. They cover catalog geometry and elapsed pose sampling,
without establishing visual acceptance for unrelated models.
