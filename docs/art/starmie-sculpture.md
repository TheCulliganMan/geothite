# Starmie sculpture and rig

The original sculpture replaces the generic four-part mesh with two continuous,
offset five-arm purple stars, ten seated radial ridges, an octagonal gold bezel,
eight beveled gold rays, and a large faceted rose core with a fitted pale glint.
The reference is the [official Starmie artwork](https://sg.portal-pokemon.com/pokedex/0121/).
No reference pixels are included in the model.

`tools/starmie_sculpt.py` authors the geometry and `tools/starmie_glb.py` exports
the canonical skin. The existing Staryu star-body helper is reused unchanged.
The asset has 23 closed parts, 2,966 vertices, 3,344 triangles and 24 joints,
and occupies 215,488 bytes. The neutral geometry digest is
`d9e6623d83cb207c09a01879bc85d5124e8a3a87fcdb6bc93ca79084369a6897`.
The authoring height is 1.03; the local content pack supplies height digits 307
(1.0922 metres) and classification MYSTERIOUS.

The core owns its entire mount and eight rays. Each star has its own layer
carrier and five two-joint radial chains with smoothly blended weights.
The lowest rear tip retains its authoring floor. Idle, attack and hit clips
articulate the layers and arms while preserving the coherent gemstone assembly.
The existing Float presentation supplies Starmie's hovering clearance.

Eight Python checks cover canonical bytes, limits, closed outward solids,
actual face normals, all ten chains, occupied core and ray joins with a detached
negative control, floor, neutral endpoints, winding, strain and fitted glint.
Front and rear software views of all three clips were inspected. Blender's
dedicated builder produces byte-identical output. Shiny verification is skipped
at the user's request.

Rebuild the asset with `python3 tools/starmie_glb.py`. Run its stored asset
checks with `python3 -m unittest discover -s tools -p test_starmie_glb.py`.
The focused Rust filter is `starmie` in `crystal-voxel-view`; it covers both
production rig lookup and GPU skin activation at the verified pack height.

Both focused Rust checks and the shared volumetric/model-timing regressions
pass. The native example builds with the existing `target/pokemon-models`
cache and `opt-level=0` overrides for crystal-core/assets/voxel-view/bevy.
The production Route 29 walking encounter against Pidgey passes terrain
admission. The inspected 3D/2D pair preserves state at core frame 2391,
checksum 2002455108, LCD animation frame 3021 and virtual time 54.047793501.
Both complete star layers fit at original pack scale; the core remains clear,
the intended hover has a ground shadow, and the HUD names both species.
No shader errors were reported. Browser visual acceptance remains open.
Final ignored evidence uses prefix `target/pokemon-native-review/starmie-normal`.
