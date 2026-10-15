# Lapras dedicated sculpture and skin

Lapras now has an original exact body instead of a missing-body fallback.
The sole canonical battle source is
`crates/crystal-voxel-view/models/battle_species/lapras.glb`.
`tools/lapras_sculpt.py` authors its geometry and `tools/lapras_glb.py` authors
its standard glTF skin. Legacy `big_lapras` and `icon_lapras` props remain
separate; neither was imported or relabeled as the exact species.

The inspected [official Lapras artwork](https://sg.portal-pokemon.com/pokedex/0131/)
guides the curved upright neck, cream throat and lower muzzle, short forehead
horn, inward-curled fleshy ears, four broad swimming flippers, blue markings,
rolled shell rim and eight seated blunt shell spikes. An initial model review
caught hollow donut ears; closed fleshy backing fills both curls. Materials are
matte. Shininess uses the existing battler flag and pack palette on the same
geometry, skin and clips; no shiny asset or alternate rig is introduced.

The stored GLB has 42 named closed parts, 19,298 vertices and 21,720 triangles.
Floor minimum is zero and neutral height is 1.50 authoring units; pack dimensions
own world scale. Its neutral digest is
`8695d9198539d60d3aaaaea4daf8dfb27f129c3a57c6ef5259af9fa807ec217e`.
The nine joints keep torso, underside and shell fixed while the two neck joints,
head, four flippers and tail respond to source battle cues. Face, horn and ears
share the head joint. The throat/markings share neck weights. Idle surveys over
four seconds; attack braces at 18% of its source cue with delayed head/flipper
settling; hit recoils. All clips contain rotation keys only.

```sh
python3 tools/lapras_glb.py
python3 tools/test_lapras_glb.py
cargo test --locked -p crystal-voxel-view species_rig --lib
cargo test --locked -p crystal-voxel-view lapras --lib
```

Verification status is recorded in `docs/art/animated-models.md`. Neutral review
images are ignored local verification output under
`target/pokemon-native-review/lapras-final/`; they are not production screenshots.

Before the palette correction, the initial native 3D/2D held pair was inspected at
`target/pokemon-native-review/lapras-3d.png` and `lapras-2d.png`. The state receipt
confirms `state_preserved=true` (hash 2525914295, LCD animation frame 43). The
body is grounded and its shell, flippers, curled ears and face remain attached
and readable beside Sudowoodo. Lighting darkens the authored blue to teal; the
source sprite uses royal blue. The large body follows the pack's dimension
rather than an arbitrary size clamp. This held idle comparison does not prove
source-cue attack/hit motion or browser acceptance.

Six stored-model checks, the focused source-routing test, both Cargo Lapras
integration tests, 18 active rig tests and the 228-species reusable model check
pass. One existing migration audit remains ignored. The initial starter-routing run encountered a Mac Chikorita regeneration
mismatch; the subsequent Chikorita correction and current routing passes are
recorded in `animated-models.md`.


## Pack palette correction (2026-10-13)

A fresh native review reproduced the teal mismatch. The verified local pack's
normal/front/back palettes agree: blue RGB555 `(8,16,28)` becomes RGB8
`(66,132,231)`, and tan `(28,21,13)` becomes `(231,173,107)`. Body, neck,
head, ears, horn, flippers and tail now use this blue. Belly, throat and lower
jaw use the pack tan. Spots and brows use a darker authored accent of the same
blue hue. Gray shell/rim and eye materials remain unchanged. Height digits 802
establish 2.4892 m (8′2″); physical scale is unchanged.

An executed before/after GLB audit proves the complete binary geometry/skin/
animation buffer identical, along with nodes, skins, mesh schema and animation
channels. Only the intended material factors and neutral digest changed.
The asset remains 42 parts, 19,298 vertices, 21,720 triangles, nine joints and
238 accessors, now 825,748 bytes. Six stored-model tests and the new independent
pack blue/tan test pass, as do both focused Rust Lapras integration tests.
No new geometry, rig, clips or shiny variant was introduced for this correction.

The newly rebuilt native paired view at 1000×700 preserves state, checksum
3736814282, LCD animation frame 37, virtual time 0.942495959. It visibly replaces
the teal body with blue and the pale underside with the source tan. The ears,
face, shell and flippers remain attached and readable. This verifies the
palette correction in the held native view; broader shape/composition, detailed
source-frame motion, legacy world props and browser fidelity remain open.
Temporary source sprites and the pack-reading helper were removed after review.
