# Hoppip wind-plant skin

The canonical runtime source is
`crates/crystal-voxel-view/models/battle_species/hoppip.glb`, authored by
`tools/hoppip_glb.py` over the existing `tools/wind_plant_sculpt.py` sculpture.
The retired static JSON was compared before removal: positions, normals,
material colors and triangle order have the same f32 neutral digest,
`01a26a1397af9b2999e457e68761ebc3373359dc53a6e9ee5ff6cf0d07f4c5e9`.

At the initial skin migration, the model had 21 named parts, 3,923 vertices, 5,740 triangles, eleven joints,
122 accessors and 202,816 bytes. Root-owned feet remain fixed. The body blends
from the root to the torso at its support transition; fitted face parts move
with the upper torso. Each ear's shell and inset share a rigid owner. Both
serrated crown blades have root and tip joints, with their tapered veins using
the identical height-based weight field. Front flippers and the curved tail
have separate owners. No root translation or scaling is authored.

Idle provides a small body sway and delayed leaf-tip/tail response. Attack
braces at 18% of the cue and settles its leaf tips later; hit recoils. Clips
use the existing rotation-only cue contract and neutral endpoints. These are
original anatomical motions; detailed source-frame motion fidelity is not yet
accepted. The supplied verified local pack has height digits `104` (1′4″,
0.4064 m), used by the production GPU integration regression. Shiny appearance
continues through the existing palette/flag path on the same skin and geometry.

Four independent stored-skin checks and six stored family-anatomy checks pass.
They cover decoded hierarchy and inverse binds, neutral digest, weights,
leaf/vein coherence, ear inset ownership, fixed feet, floor clearance, winding,
strain and delayed tip response. Both focused Rust integration checks pass, as do all 75 standard variant
schema checks and all 49 GPU checks. The focused Blender builder reproduces the
canonical GLB byte-for-byte; mouth-center signed zeros are normalized across
interpreters without changing the original digest. The production build passes.
Full art validation passes for 60 editable sources, 609 runtime models in
609 files and 749 embedded references.
Browser acceptance and the remaining wind-plant rigs are not established.


Native normal and shiny production pairs were inspected against 2D at identical
state and clock within each pair. Both preserve core frame 0: normal checksum
2222156885, LCD frame 56, virtual seconds 1.424398625; shiny checksum 4237965123,
LCD frame 53, virtual seconds 1.233750541. The body, supporting feet, ears,
flippers, tail and serrated crown remain attached at the pack-owned scale.
The shiny flag recolors this same skin to olive with bright green leaves;
no second model or rig is used. The normal body remains too pale pink compared
with the source sprite's deeper red, and the crown is narrow from the current
edge-on battle view. These pairs accept skin integration, not final visual
fidelity. Source-palette correction and leaf-readability review are next work.

## Pack palette correction (2026-10-14)

The verified supplied pack's normal, front and back palettes agree: leaf green
RGB555 `(11,23,6)` becomes RGB8 `(90,189,49)`, and body red `(21,6,7)` becomes
`(173,49,57)`. Hoppip's body, support feet, flippers, outer ears and curved tail
now use this red; both serrated blades use the green. Fitted face, inner ear,
tail-tip and vein accent materials remain authored detail. Other wind plants
retain their existing colors.

The complete embedded binary buffer is identical to the pre-correction GLB,
as are nodes, skins, mesh schema, accessors, buffer views and animation data.
Only intended material factors and the source digest changed. The current
asset is 202,820 bytes with neutral digest
`008e60652d5067fa358c74a2d3ee74c563c22b5b193c988c1b21fda06b224da8`.
Five stored-skin checks, including independent expected linearized body/leaf
colors, and six family anatomy checks pass. Blender regenerates the corrected
asset byte-for-byte. Both focused Rust checks and the production build pass.
Fresh paired production views preserve state and were inspected against 2D:


| Corrected pair | Checksum (core frame 0) | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: |
| Normal / Sudowoodo | 2093017611 | 53 | 1.219221958 |
| Shiny / Sudowoodo | 414192824 | 56 | 1.345555583 |
| Normal / Pidgeotto fixture | 2119855993 | 55 | 1.368006667 |

The corrected normal body is visibly deeper red and the leaves are stronger
green. Shiny presentation remains olive with bright green leaves on the same
mesh, joints and clips. These views accept the palette correction, while
lighting continues to shade the source colors. The alternate Pidgeotto fixture
keeps essentially the same oblique model view and still shows a narrow crown;
it does not establish broadside crown acceptance. Leaf shape/curvature review,
source-frame motion and browser review remain open. Temporary pack-reader,
pre-correction GLB and focused Blender outputs were removed; QA stays ignored.

## Curved crown blades and fitted midribs (2026-10-14)

Both crown blades now bow in opposite depth directions through their middle.
Two levels of midpoint subdivision preserve the serrated XY outline and the
closed blade thickness before curvature is applied; normals are recomputed
for the curved surfaces. Dense midrib samples use the actual blade's horizontal
cross-sections, addressing the old vein path that partly left the silhouette.
The midribs use the same bow as their blades and retain the shared root/tip
weight field. Other wind plants and the body/face/support geometry are unchanged.

The revised model has 21 parts, 6,077 vertices, 8,380 triangles, eleven joints,
122 accessors and 286,536 bytes. Its digest is
`10632e7b0c178569f270d0e36b649a9403b9ddab13be50e7f22bca01bcd7ab4a`.
Six stored-skin checks and six family anatomy checks pass, including a new
ray-based check that all sampled vein vertices stay near the blade surface,
nonzero blade depth, fixed feet, winding, deformation and palette checks.
The Hoppip size check now measures its shipped GLB against 512 KB rather than
serialized decoded JSON; vertex/triangle budgets remain in force. Blender
reproduces the finalized GLB byte-for-byte. Both fresh Rust checks and the
production build pass. A new native pair was inspected against 2D at preserved
state: checksum 2038482523, core frame 0, LCD frame 114, virtual seconds
3.236874334. It shows attached curved leaves and corrected red/green materials,
but the crown still reads too narrowly from the oblique battle view. Final
visual acceptance is rejected; a more clearly splayed blade design is next.
The all-75-variant off-grid culling-envelope regression subsequently passed
for this bowed iteration in 511.63s.
No camera/facing adjustment was used to hide the silhouette issue.

## Splayed crown and aligned leaf pivots (2026-10-14)

The two leaf solids now share a fixed authored -35-degree crown angle, with a
smaller 0.035-unit bow that begins at the root hinge. Both leaf veins follow
that same transform, with geometric normals rebuilt after deformation. The
tip hinges now lie on the fitted midribs rather than the old unfit centerline.
The initial positive-angle prototype was rejected in the actual production
view because it made the blades more edge-on; a neutral side review alone was
insufficient to establish acceptance.

The final export has 21 parts, 5,963 vertices, 8,380 triangles, eleven joints,
123 accessors and 284,556 bytes. Its current neutral digest is
`aaa5bb6c980f1ba8d3327f7f611a059193c29ae7092e35f06acab3b9181507b4`.
Seven stored-model checks and six family anatomy checks pass. The added checks
verify tip pivots against actual stored vein vertices and measure crown width
from the front and side relative to body width. Both fresh Rust checks pass;
the Hoppip registry check now samples every vertex off-grid across all three
clips for floor clearance and computed culling-envelope containment. Blender
reproduces this asset byte-for-byte. The earlier all-75-variant envelope run
also completed successfully for the preceding bowed iteration in 511.63s.
The production build passes. Fresh normal and shiny pairs were personally
inspected against 2D at identical game state and clock within each pair:

| Final splayed pair | Checksum (core frame 0) | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: |
| Normal | 2686567075 | 56 | 1.452777292 |
| Shiny | 3671073830 | 55 | 1.485138209 |

Both receipts preserve state. The final view shows two distinct serrated
blades with fitted veins and attached hinge motion, resolving the narrow
strip silhouette in the preceding views. Red body/green leaves and the olive
shiny body/bright green leaves remain readable at the pack height. This accepts
the crown readability improvement in the held native battle view. Detailed
source-cue motion, gaits, other world/battle angles and browser acceptance
remain open. The shiny flag continues to recolor the same geometry and skin.
Prototype code/models and focused builder output were removed; QA stays ignored.
