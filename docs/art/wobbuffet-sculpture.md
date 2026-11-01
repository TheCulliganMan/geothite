# Wobbuffet sculpture and standard skin

`tools/wobbuffet_sculpt.py` authors the anatomy and `tools/wobbuffet_glb.py`
exports canonical `models/battle_species/wobbuffet.glb`. The species builder
now emits that GLB and removes duplicate static JSON. All 19 parts preserve the
previous asset's exact stored f32 positions, normals and triangle order:
11,791 vertices and 19,260 triangles. The 1.14 normalized reference span and
floor origin are unchanged. The local pack's height 403 is 4′3″ (1.2954 m).

The broad upper body tapers into four soft basal lobes. Two continuous curved
arms rise above the head, one ending in a salute. Fitted squeezed eyelids, a
zigzag mouth and lower tongue retain their actual triangle support. A curved
rear stem reaches a rounded paddle with two fitted eyes and pupils. The
existing authored salute pose remains; this migration does not claim an exact
pose match to every source sprite frame.

Actual local Crystal front/back sprites and normal palettes were inspected.
The body now uses source RGB555 (12,21,31), or sRGB8 (99,173,255). The open mouth
and tail are black; the lower mouth uses source shade (7,14,16), or (57,115,132),
in place of burgundy and pink. Tail eyes are white. Geometry helpers perform
the sole sRGB-to-linear conversion.

Eleven joints follow the shared GPU skinning path. All four foot lobes stay
rigid to the identity root, and the lower body blends upward into the torso.
The fitted face remains rigid to the upper torso. Independent shoulder, elbow
and hand chains articulate both arms; rounded hand tips share their terminal
owners. Three stages bend the tail stem, while the complete paddle, eyes and
pupils share its final joint. Rotation-only idle/attack/hit clips have neutral
endpoints and delayed hands/tail response. Authoritative battle cues own
playback timing. Exact source-frame face/arm motion and field gaits remain open.

The GLB is 503,824 bytes with 111 accessors, within existing loader limits.
Its rest digest is `4afb5da2de8fcfaaa070c632289d645dc8b67e7790a2f57b19dc3dc3e5b20a09`.
Six independent stored-skin and nine migrated anatomy checks pass: canonical
regeneration, bind identity, source colors, coherent fitted owners, planted
lobes, clip endpoints, winding/edge strain, closed topology and volumetric
support. The focused Blender species builder produces byte-identical output
and no static JSON. The neutral software preview was inspected. Both focused Rust checks pass: GPU skinning at the pack height, and off-grid
clip sampling for planted support, coherent attachments and culling bounds.
The native build and matched normal Wobbuffet/Pidgey review pass (core 1957,
checksum 1205809066, LCD 2544, time 45.984559665, state preserved). Raised
arms, fitted face, both tail eyes and planted lobes remain readable with floor
contact shading. Native rest-pose/rig/color integration is accepted. Shiny-specific verification is
skipped at the user's request.
