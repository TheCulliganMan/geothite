# Phanpy trunk and lifted-foreleg skin

`tools/phanpy_sculpt.py` authors the baby elephant; `tools/elephant_glb.py`
exports its canonical standard skin. The common production reader and GPU path
consume `models/battle_species/phanpy.glb`; the parallel static JSON is retired.
The battle builder now dispatches to this exporter before legacy static hooks.

The rounded body/head, upturned trunk, raised nose, drooping ears, three planted
pads and playful lifted foreleg retain their original anatomy. Twenty-eight
parts preserve exact positions, normals and indices. Four fitted ear markings
use two subdivision levels instead of three, reducing vertices from 24,269
to 18,472 and triangles from 40,500 to 31,284. The stored-surface and front-face
clearance checks pass at the existing marking offset. This fits the existing
20,000-vertex/120,000-index limits without raising them. Original ordered patch
centroids remain stable across Python versions.

Normal materials use the local pack's RGB555 blue (17,20,26), warm marking
(23,9,6), white and black. The forehead saddle, ear markings and tongue share
the warm color; pupils, nostrils and mouth are black. The canonical asset has
32 parts, 13 joints, 163 accessors and 729,772 bytes. Its neutral digest is
`4f4d743119d23709e102ad164adb17c52d236d9dca587c4ef10b3176e6fab0f4`.
Normalized height remains 1.02; pack height 108 means 1 foot 8 inches (0.508 m).

Three grounded pads and their fitted nails stay on the identity root. The
lifted foreleg, pad and nail share a joint parented to the torso. Other leg
middles blend between fixed supports and the torso. Head, gaze, saddle and
smile move together; each ear owns both fitted markings. Trunk base/tip stages
use a monotone field across the thick upturned bend, avoiding jumps from
nearest-segment ownership. The raised nose and nostrils share the tip joint.
A two-stage tail follows with delayed motion. Idle, attack and hit use rotations
only, neutral endpoints and the production source-cue timing boundary.

All eight anatomy checks and seven shared decoded-skin checks pass. They cover
closed outward surfaces, support joins, fitted marking vertices and triangle
clearance, canonical bind/budgets, normal materials, coherent owners, fixed
pads, lifted-foot motion, winding/edge strain and delayed trunk-tip motion.
Both focused Blender exports match native bytes. Front/rear software views
were inspected. Both focused production Rust checks pass, including GPU activation at the
verified pack heights, fixed supports, coherent owners and animated bounds.
Matched normal native review passes.
Shiny checks remain skipped; exact sprite-frame motion, field gaits and browser
review remain open.

```sh
python3 tools/elephant_glb.py --species phanpy --output crates/crystal-voxel-view/models/battle_species/phanpy.glb
python3 -m unittest discover -s tools -p test_phanpy_sculpt.py
python3 -m unittest discover -s tools -p test_elephant_glb.py
cargo test --locked -p crystal-voxel-view elephant_family_ --lib
```


Phanpy's matched normal native 3D/2D review against Pidgey passes. Its small
pack-based scale, grounded pads, lifted forefoot, upturned trunk and fitted ear
markings read clearly against the retained grass. The paired receipt preserves
core frame 2387, hash 2681977306, LCD frame 2965 and time 52.322835457
(`state_preserved=true`). The native build and wider catalog grounding/actual
elapsed-pose regressions pass. Shiny checks remain skipped.
