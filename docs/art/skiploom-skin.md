# Skiploom leaf-ear and flower skin

The canonical runtime source is
`crates/crystal-voxel-view/models/battle_species/skiploom.glb`, authored by
`tools/skiploom_glb.py` over the existing `tools/wind_plant_sculpt.py` geometry.
The initial retirement audit confirmed every part's f32 positions, normals
and triangle order matched the former static JSON. Subsequent ear changes are
recorded below. The body, support feet, flippers,
leaf ears and stalk now use verified pack green; petals use pack gold. The
remaining fitted accents stay authored detail. Hoppip regenerates byte-for-byte
and Jumpluff's stored anatomy remains unchanged.

The supplied verified pack's front, back and normal palettes agree: green
RGB555 `(8,18,7)` becomes RGB8 `(66,148,57)`, and gold `(31,22,5)` becomes
`(255,181,41)`. Its height digits `200` establish 2′0″, or 0.6096 m. Physical
height remains driven by the pack, separately from the authored animation bounds.

The model has 24 named parts, 5,182 vertices, 10,028 triangles, eighteen joints,
145 accessors and 265,012 bytes. Its neutral digest is
`d29a5c2236032d30afba0c5c73b696928dd169c36eb8225b2cd27c43b468fcb1`.
Root-owned feet stay fixed. The body transitions from root to torso near its
support; the fitted face follows the upper torso. Leaf ears and their fitted
folds share the same body/base/tip field. The stalk bends into a separate flower
center, with each of the eight radial petals owning a hinge inside its base.

Idle gives small leaf-ear and flower responses. Attack braces at 18% of its cue
with delayed ear-tip and flower settling; hit recoils. Three rotation-only
clips have neutral endpoints and no root motion. They are original anatomical
responses; detailed source-frame motion fidelity is not established.
Shininess uses the existing appearance flag and palette on this same geometry,
skin and clips.

Four decoded stored-skin checks and six family anatomy checks pass. They cover
hierarchy, inverse binds, materials, owner fields, fixed feet, floor clearance,
winding, strain, motion onset and neutral endpoints. Blender reproduces the
canonical GLB byte-for-byte. Both focused Rust checks pass, including all
vertices sampled off-grid through all three clips against floor and computed
culling bounds, production registry/neutral routing, and GPU skinning at the
verified pack height. The production build, all 76 standard variant schema checks and full art
validation pass. Full validation covers 60 editable sources, 609 runtime models
in 609 files and 751 embedded references. Browser acceptance and the remaining
wind-plant/all-species rigs remain open.


Native production normal and shiny pairs were personally inspected against
2D at identical game state and clock within each pair. Both preserve state:

| Pair | Checksum (core frame 0) | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: |
| Normal | 3084625203 | 62 | 1.636727750 |
| Shiny | 2295057978 | 55 | 1.344080292 |

The gold flower, green normal body and violet/yellow shiny appearance remain
attached and readable at the pack-owned height. The held views establish rig
integration and appearance-flag reuse on the same geometry, not final shape
acceptance. The near leaf ear overlaps the body in this oblique view and the
body reads narrower than the source back sprite; broader silhouette and ear
presentation need refinement. Detailed source-frame motion, gaits and browser
acceptance are still open. Pack readers and focused builder outputs were
removed; native QA images and logs remain ignored. No pack, audio or gameplay
content was changed.

## Leaf-ear splay and matching rest hinges (2026-10-14)

The leaf blades and their fitted fold lines now share a fixed authored
-35-degree rotation around each ear root. Their tip hinges use the same
transform. Skin weights are evaluated in the inverse ear frame so the
body/base/tip transition remains attached after the geometry moves. Body,
flower, support geometry, materials and physical height remain unchanged.
Hoppip still regenerates byte-for-byte.

The export remains 24 parts, 5,182 vertices, 10,028 triangles, eighteen joints,
145 accessors and 265,012 bytes. Its current neutral digest is
`9541ae752be3d0669e5836358537f971a6bf1d42ca2d2d72db639008e0be4987`.
Five stored-skin checks and six family anatomy checks pass. The new check
places the stored tip hinges inside the closed blades and checks ear spread
relative to body width in front and side projections. Both fresh Rust checks
pass, including off-grid culling/floor coverage and GPU physical scale. Blender
reproduces the new asset byte-for-byte, and the production build passes.
Fresh normal and shiny pairs were personally inspected against 2D at identical
game state and clock within each pair:

| Splayed-ear pair | Checksum (core frame 0) | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: |
| Normal | 1122343596 | 63 | 1.563403209 |
| Shiny | 3351867875 | 55 | 1.337014709 |

Both receipts preserve state. The leaf ears now project beyond the body and
their fitted folds stay readable, while the flower and feet remain attached.
The violet/yellow shiny form uses the same mesh and skin. These views accept
the leaf-ear readability correction in the held native battle view. Broader
body proportions, other views, detailed source-frame motion, gaits and browser
acceptance remain open. Focused builder scratch output was removed; native QA
images and logs remain ignored. No pack, audio or gameplay data was changed.
