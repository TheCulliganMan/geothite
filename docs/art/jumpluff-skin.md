# Jumpluff three-head seed skin

The canonical runtime source is
`crates/crystal-voxel-view/models/battle_species/jumpluff.glb`, authored by
`tools/jumpluff_glb.py` over `tools/wind_plant_sculpt.py`. The migration audit
expanded the stored model and compared every original component's positions,
normals and triangle order with the retired JSON. All 103 source components,
including all 84 fine bristles, preserve their geometry.

Each seed head's 28 rigid bristles share a render group. The GLB stores component
ranges in global metadata so authoring readers can recover the individual
closed components for anatomy checks. Runtime primitive metadata retains the
existing strict `part` schema. There are 22 runtime anatomy groups, 5,836
vertices, 10,380 triangles, thirteen joints, 125 accessors and 283,816 bytes.
No loader limits were increased. The grouped neutral digest is
`c97857291dcd124502dab863c5adc8940071421d7e68236c8d69471ab04cd845`.

The verified supplied pack's front/back/normal palettes agree on blue RGB555
`(9,8,24)` and tan `(28,23,13)`, expanded to RGB8 `(74,66,198)` and
`(231,189,107)`. The body, support feet and front flippers use the blue;
The initial seed spheres and bristles used tan; the white-surface correction
below supersedes that sphere material. Green stems/bracts and fitted face accents
remain authored detail. Hoppip and Skiploom still regenerate byte-for-byte.
Height digits `207` establish 2′7″, or 0.7874 m, which owns production scale.

Root-owned feet stay fixed. Lower stems lift into separate seed-head joints;
each sphere and its bristles share a rigid owner. The crown has a two-joint
stem, a seed-head joint and independent bracts. Idle sways and lifts lightly;
attack braces at 18% of its cue and settles the crown later; hit recoils.
Three rotation-only clips have neutral endpoints and no root motion. These
are original anatomical responses; detailed source-frame fidelity remains open.

Four independent stored-skin checks and six family anatomy checks pass. Both
focused Rust checks pass for registry/neutral routing, all vertices sampled
off-grid through all clips against floor and culling bounds, rigid head/bristle
ownership, fixed feet and GPU skinning at the verified pack height. Blender
reproduces the complete GLB byte-for-byte; signed zero tongue coordinates are
normalized across authoring runtimes without changing the stored geometry.
The first production build, all 77 variant schemas and full art validation
pass. Full validation covers 61 editable sources, 609 runtime models and 752
embedded references. Initial native normal/shiny comparisons preserve state
but reject the all-tan/all-lilac sphere materials. Browser verification, gaits and the remaining all-species work
remain open.


## White puff surface correction (2026-10-14)

Initial paired views show that the source sprites retain mostly white seed
heads, with tan shading normally and lilac shading when shiny. Coloring the
entire sphere with the shade palette made the normal heads gold and shiny
heads lilac. The sphere material now uses white; bristles retain the secondary
palette color. A before/after audit proves the complete binary geometry,
weights, inverse binds and animation buffer unchanged, with nodes, skins,
accessors, buffer views and animations identical.

The current export is 283,968 bytes and has grouped neutral digest
`70c2f3002f9ec982475c30f2e55a58d41bf5979a66fcb91187473cf80e295e24`.
Four stored-skin/material checks and six family anatomy checks pass. Blender
reproduces the corrected GLB byte-for-byte. Both fresh Rust checks and the production build pass. Corrected native
normal/shiny pairs were personally inspected against 2D at identical game state
and clock within each pair: Seed-head surface detail and far-head visibility
still require broader review before full visual acceptance.


| White-surface pair | Checksum (core frame 0) | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: |
| Normal | 3260994057 | 61 | 1.504502625 |
| Shiny | 1744052987 | 48 | 1.223257084 |

Both receipts preserve state. The blue normal and pink shiny body share the
same geometry, joints and clips; puff surfaces stay white and fine bristles
retain their shade-color appearance. These views verify the rig integration
and white-surface correction. Full visual acceptance remains open: the spheres
still look too smooth, the far lower seed head is partly hidden by the body,
and source-colored underside shading needs more work. Source-frame motion,
gaits and browser review also remain open. Temporary pack readers, comparison
scripts, pre-correction GLB and focused builder outputs were removed; QA images
and logs remain ignored. No pack, audio or gameplay data was changed.

## Lower seed-head separation (2026-10-14)

Both lower assemblies turn −35° about their stem attachments in the horizontal
plane. Each sphere, its 28 bristles and its stem share the same transform;
the seed-head pivots follow their sphere centers. The stem weight transition
is evaluated in the original local coordinates. This preserves all neutral
heights, crown geometry and support feet while separating the lower heads in
depth. Hoppip and Skiploom still export byte-for-byte unchanged.

The export is 283,964 bytes, with grouped neutral digest
`62a6860a398942b33c5be4dfee51d645e90b2efdc32e9d5c010c9e4b6f0fe00f`.
Five independent stored-model checks pass, including centered pivots and depth
separation relative to body width. Six family checks and both focused Rust
registry/GPU checks pass. Blender reproduces the complete GLB byte-for-byte.
The production build passes. Normal and shiny paired captures were inspected
against their 2D views at the same state and clock within each pair. Both
receipts preserve state: normal checksum 334151549, core frame 0, LCD frame 55,
virtual seconds 1.31781525; shiny checksum 806345474, core frame 0, LCD frame 56,
virtual seconds 1.430568916. The far lower head now projects more clearly past
the body, though it remains partly occluded at this battle angle. All three
heads remain attached, and the feet stay above the floor. This accepts the
visibility improvement only. Smooth puff surfaces, missing tan/lilac underside
shading, body proportions, source-frame motion, gaits and browser review remain
open. Focused Blender output was removed; QA images and logs remain ignored.


## Scalloped seed-head underside (2026-10-14)

Each puff now has a white upper region and tan underside, meeting at a shared
32-point scalloped latitude. The two closed solids have internal caps; both
visible surfaces remain on the same spherical contour. They use existing
untextured materials and the standard rig schema. Each region and all 28
bristles retain the same rigid head joint. Bristle roots are slightly wider
and tips slightly longer. The stem-contact check tests the complete puff union,
including the lower crown region where its stem enters.

The canonical export has 106 source components, 25 runtime anatomy groups,
6,247 vertices, 10,560 triangles, thirteen joints, 136 accessors and 299,068
bytes. Its grouped neutral digest is
`2de06589332b313b7e2264c47f04d319ddb334f09c1caca6632cf7a69b7659fd`.
Six stored-model checks and six family checks pass, including seam equality,
closed topology, material colors, skin ownership, support height and stem
contact. Both focused Rust registry/GPU checks pass. Blender reproduces the
complete GLB byte-for-byte; Hoppip and Skiploom remain byte-identical.
All 77 registered variant schemas and the production native build pass.
Full art validation passes for 61 editable Blender sources, 609 runtime models
in 609 files and 753 embedded references.
Normal and shiny paired views were inspected against 2D at the same state and
clock within each pair. Both receipts preserve state: normal checksum
3326295098, core frame 0, LCD frame 55, virtual seconds 1.360544501; shiny
checksum 3224073217, core frame 0, LCD frame 54, virtual seconds 1.363543083.
White puff tops now visibly retain tan undersides normally and lilac undersides
when shiny, using the existing palette remapping path. The scalloped edge is
readable, the three heads remain attached, and support height is unchanged.
This accepts the material separation. Fine seed texture remains sparse at battle
scale, the body proportions still need review, and source-frame motion, gaits,
browser review and the all-species goal remain open. Focused Blender output
was removed; QA captures/logs remain ignored.
