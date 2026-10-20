# Ampharos body and surface fitting

The older muzzle used free-standing nostril ellipsoids about 0.0098 model units
above the stored body. Its smile also lacked actual body contact, and the chest
panel used an analytic loft surface that differed from the rendered triangles.
The current recipe fits the eyes, catchlights, nostrils, forehead beacon,
highlight, and chest panel directly to the stored mesh. The mouth tube is
rooted into that surface. A smooth pear-shaped outline replaces the coarse
chest boundary.

The original body profiles remain the authored knots. Monotone interpolation
adds one intermediate ring between each pair, with 32 circumference samples
instead of 24. This rounds the torso, neck, and muzzle transitions without
inventing another body. Marking edges use split crease normals; the beacon
outline avoids an exact ring tangency that would collapse under float32
rounding. Every solid still passes the existing closed-surface, outward-winding,
and nondegenerate-triangle checks.

Ampharos retains 26 parts, nine joints, the 1.35 reference height, and the exact
normal palette. The pack still determines its physical game height. The new
asset has 3,494 vertices, 5,636 triangles, and 192,264 bytes, within its existing
6,000-triangle and 650,000-byte checks. Feet, flippers, horns, collars, tail
geometry, inverse binds, hierarchy, and animation keys remain unchanged.
Mareep and Flaaffy remain byte-identical to their existing canonical assets.

Four anatomy checks and ten shared skin checks passed against the candidate.
They retain the actual foot/body and collar/tail contacts, normal palettes,
winding, edge strain, fixed supports, and source-cue motion checks. New tests
measure fitted front-cap clearance at triangle centers and edges through all
three source clips, and require actual smile/body overlap. They reject the
earlier detached nostrils and smile rather than accepting their bounding boxes.

The existing CLI can reproduce the asset into ignored verification output:

```sh
python3 tools/electric_sheep_glb.py --species ampharos --output target/pokemon-native-review/ampharos-refit-candidate/ampharos.glb
python3 -m unittest discover -s tools -p 'test_ampharos_sculpt.py'
python3 -m unittest discover -s tools -p 'test_electric_sheep_glb.py'
```

The CLI reproduced the integrated asset byte for byte. The Rust electric-sheep
family regression and the 276-variant schema regression also passed, bringing
this refinement to 16 focused checks. Negative controls reintroduced an old
nostril and the old smile and were rejected by the new clearance/contact tests.

Muted headless native pairs at Ecruteak `(18,21)`, noon, were inspected for the
front and walking world views alongside their same-state 2D map references.
An actual moving field frame was also inspected. Face markings remain fitted,
the chest reads as a rounded panel, and feet and tail remain grounded/rooted.
The walking record contains 32 samples across the expected five positions.
The current compressed GLB was verified in the binary, which stayed unchanged
through both pairs. This is the review scope for this refit, not every-angle
acceptance of the species or completion of the whole Pokémon roster.
