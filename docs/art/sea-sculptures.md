# Corsola and Mantine sculptures

`tools/sea_sculpt.py` replaces the shared primitive recipes for these two species with original authored geometry. The canonical outputs remain `corsola.mesh.json` and `mantine.mesh.json`; the existing battle builder dispatches to the same recipe. Their prior normalized heights (0.93 and 1.02) remain unchanged.

Corsola has five thick coral stems with lateral forks and rounded tips, a separate white lower bowl, four supporting white feet, rounded pink arms, fitted diamond flecks and a surface-following smile. Mantine has broad swept wings with smooth curved cross sections and shared blue/pale material boundaries, a fitted blue dorsal mantle, curved cephalic paddles, tapered tail, gill slits and an attached sea-green remora. These are static meshes; no skeletal rig or swimming animation is claimed.

Official design references were inspected from [Corsola artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/222.png) and [Mantine artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/226.png). Those images are temporary review references and are not shipped. No external mesh, texture, game extraction or content pack is included in this change.

External asset search found printable models and [Cobblemon assets](https://gitlab.com/cable-mc/cobblemon-assets/-/tree/master/blockbench/pokemon/gen2). The latter's [license page](https://gitlab.com/cable-mc/cobblemon-assets/-/blob/master/LICENSE) declares CC BY-NC 3.0. These have not been imported or represented as verified compatible rigs. Original geometry keeps the existing authoring pipeline consistent without adding those external licensing restrictions.

Validation:

```sh
python3 -m unittest discover -s tools -p test_sea_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
```

The seven geometry regressions check closed, connected and outward-oriented parts; actual triangle shading; deterministic regeneration and bounds; shared curved wing boundaries and root/body volume contact; fitted facial details; and four grounded feet overlapping the lower bowl. Front and rear software renders inspect stored geometry. Native battle screenshots use the existing ignored `content-packs/realtime-clock.browser.crystalpack` with the `immersive_battle_3d` example. Rendering checks establish model appearance and camera fit, not gameplay or controller behavior.

Verified this pass: all seven geometry regressions, canonical layout, 571 JSON document validation, the Rust full-catalog grounded/volumetric regression, and both native battle captures passed. Corsola contains 14,180 triangles (606,046 bytes); Mantine contains 11,108 triangles (677,539 bytes). Temporary captures and reference images were removed after inspection.

This is one incremental family pass within the broader 251-species visual audit; it does not establish completion of the full catalog.

## Mantine detail refinement

Mantine's later review replaces pointed cephalic tubes with rounded curved
leaf paddles, adds two fitted zigzag blue borders to the pale wing undersides,
and extends the tail into a swept terminal fin with pale lower and blue upper
layers. The layers seat in each other as closed solids. The attached Remoraid
has a rounded muzzle, fitted dark sucker mouth, fitted eyes and glints, a dorsal
attachment fin, a curved tail stalk and a two-lobed trailing fin. Mantine now
has 32 parts and 27,416 triangles. Its height remains 1.02. Corsola's canonical
asset and authored recipe are preserved.

The new tests check stored zigzag/Remoraid surface fit and triangle clearance,
cephalic paddle roots, tail stalk/leaf/layer joins, Remoraid body/muzzle/fin/tail
joins and attachment to the wing. Physical joins use actual occupied ray
intervals. The existing seam, Corsola grounding, topology, shading and recipe
parity regressions remain active. Corsola retains its 18,000-triangle budget;
Mantine has a 40,000-triangle limit for the subdivided surface borders.

External research inspected the CC Attribution
[Mantine listing](https://sketchfab.com/3d-models/mantine-f46f0b92748f4bf49f0723ccc2d1ece7)
and [Mantine surf listing](https://sketchfab.com/3d-models/mantine-surf-6ebc8759055f4f3994540dd0bd67edc0).
Their metadata did not establish a usable original rig. No external model or
texture was imported. The official artwork above remains the temporary design
reference.

Final Mantine refinement verification passed all ten sea geometry regressions,
including exact Corsola recipe parity. The first surface-clearance run found
a wing-border triangle slightly below its support; increasing the shallow
border thickness to 0.006 corrected it, and the full suite passed on the final
asset. Final underside and rear software views and the final native battle
capture were visually inspected. The native build and Rust 220-species
grounding/volume regression passed, as did layout/storage validation for all
571 JSON assets. Old completed-test object intermediates were removed to
provide build space. Temporary reference art, renders and logs were removed
after inspection. This refinement remains part of the ongoing full visual
review rather than proof that every species is finished.
