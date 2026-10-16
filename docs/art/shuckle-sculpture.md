# Shuckle perforated-shell sculpture and standard skin

`tools/shuckle_sculpt.py` authors the anatomy and `tools/shuckle_glb.py` exports
canonical `models/battle_species/shuckle.glb`. A dedicated species-builder branch
now emits the GLB and removes duplicate static JSON. All 27 parts preserve the
previous asset's exact stored f32 positions, normals and triangle order:
14,875 vertices and 19,836 triangles. The normalized 0.91 span and floor origin
are unchanged. The local pack's height 200 means 2′0″ (0.6096 m).

The shell remains one connected outer/inner skin with six real openings and
annular collars. Four soft limbs emerge through their authored holes and reach
rounded grounded feet. The neck passes through its own opening to a rounded
head with fitted eyes and mouth. Front/rear spots stay on actual shell triangles.
Actual local front/back sprites and normal palettes were inspected. Shell red
is RGB555 (21,4,3), or sRGB8 (173,33,24); soft-body gold is (31,22,5), or
(255,181,41). Collars and spots now use white, with black eyes, mouth and recessed
interior. Geometry helpers perform the only conversion to linear colors.

Eight joints follow the common GPU skinning path. Shell, collars, spots, recessed
core and all four feet remain rigid to the identity root. Each limb's middle
blends to its own joint, fading back to fixed weights at both ends. Yaw-only
limb motion retains every limb vertex's height. The neck blends through base,
bend and head joints, with fitted eyes/mouth rigid to the head. Rotation-only
idle/attack/hit clips have neutral endpoints and delayed upper-neck/head
response; authoritative source battle cues own playback timing. The shell is
never deformed to move the soft body. Exact source-frame retraction and field
gaits remain open.

The GLB is 594,848 bytes with 132 accessors, within existing loader limits.
Its rest digest is `2a7a21948c474962c403086b8e508fc560f2c001c1448311316a0dbb46bc2206`.
Six independent stored-skin and eight migrated anatomy checks pass, including
bind identity, canonical regeneration, source colors, fixed shell/collar/spot
geometry, all six shell apertures, planted feet, coherent face, soft motion,
winding/edge strain and actual volumetric/surface support. Focused Blender
builder output is byte-identical and emits no static JSON. Neutral front and
rear software previews were inspected. Both focused production Rust checks pass: GPU skinning at the pack height,
and off-grid clip sampling for fixed shell/feet, coherent face and culling
bounds. The native build and matched normal Shuckle/Pidgey review pass: core 1959,
checksum 3950609093, LCD 2608, time 46.914735668, state preserved. The fixed
shell, seated neck/collars, complete head and visible grounded feet remain
readable, with floor contact shading. Native rest-pose/rig/color integration is
accepted; exact source-frame retraction remains open. Shiny-specific checks and captures are skipped at
the user's request.
