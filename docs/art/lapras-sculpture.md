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

The stored GLB has 45 named closed parts, 19,471 vertices and 23,332 triangles.
Floor minimum is zero and neutral height is 1.50 authoring units; pack dimensions
own world scale. Its neutral digest is
`e783b1adb22c94e3a0914dde725c5eef44c4197fc93e4736edc49e24e455548e`.
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


## Seated battle contact correction (2026-10-15)

A fresh production Route 29 walking encounter reproduced Lapras hovering
above the source grass. The accepted native 3D/2D baseline preserved state
at core frame 3182, checksum 673262487, LCD animation frame 3994 and virtual
time 80.012000502. The renderer's generic floating-battler adjustment added
0.24 authoring units (16% of its height), despite the canonical skin's fixed
belly and shell. This is separate from grass height and source geometry.

`pokemon_motion::battle_hovers` now distinguishes Lapras's seated battle
presentation from its swimming locomotion. Battle sampling no longer adds
the artificial root lift to Lapras. Ground contact uses the existing cached
floor-connected belly region, excluding the extended flipper tips. Swimming
locomotion, authored geometry, weights, clips, pack dimensions and palette
remain unchanged. Actual hovering species retain their existing clearance.

An independent stored-vertex regression first failed at idle time zero with
floor 0.24. After correction it passes for 33 samples of each idle, attack
and recoil clip. The same regression checks unchanged Cloyster and Porygon
clearance. Four focused Rust checks pass for seated vertices, the production
contact-shadow cache, dedicated Lapras registry and GPU activation at pack
size. The existing complete-shell Cloyster locomotion regression also passes.
Shiny verification remains skipped at the user's request.


The rebuilt native Route 29 pair `lapras-grounded-3d.png` /
`lapras-grounded-2d.png` passes terrain admission (`accepted=true`) on the
original ground. The receipt preserves state at core frame 1453, checksum
1571462043, LCD animation frame 2091, virtual time 36.800318958. The pair was
visually inspected: the tan belly meets the field, the contact shadow sits
under the seated body, and the blue neck, face, shell spikes, curled ears and
four flippers remain readable at unchanged pack size. No shader errors were
reported. This run encountered Pidgey; the earlier baseline encountered Rattata,
so each native view is paired with its own identical-state 2D view rather than
claimed as a frame-identical cross-run comparison.

The catalogue-wide elapsed-idle/source-attack timing regression also passed.
This closes the added battle-hover/contact defect only. Lapras's broader
shape and detailed face/flipper refinement, legacy world props and browser
visual acceptance remain open. No grass-height or shiny changes were made.
