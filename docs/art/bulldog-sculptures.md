# Snubbull and Granbull sculptures

`tools/bulldog_sculpt.py` authors distinct canonical static models for both
bulldogs. The main species builder dispatches these recipes before Blender
batching. Both preserve the registry's 1.06 height and floor origin.

Snubbull has a continuous pink dress-shaped torso with a scalloped hem, four
fitted blue oval markings, a blue neck ruffle, raised short arms, cream feet,
and a large round head. Broad pink jowls meet a projecting lower lip. Two small
ivory canines point downward in front of that lip. Folded broad ears carry dark
outer markings. Tilted ivory eyes and small pupils replace the generic round
eyes. A short tail remains visible from the rear.

Granbull has a heavier purple chest, belly and hips, muscular bent legs, broad
paws and outstretched arms. Dark wrist cuffs and a fitted triangular chest bib
replace the former thin neck disc. A short purple upper muzzle, broad black
nose, heavy lower jaw, dark open mouth and rosy tongue support its face. The
large curved ivory lower canines rise past the upper muzzle; two smaller upper
fangs point down. Its folded ears remain purple.

The shared helpers provide geometry rather than shared species anatomy.
Ear folds deform their actual surfaces and recompute normals. Eyes, dress
markings, bib, tongue and toe/knuckle clefts fit their supporting triangles.
All parts are closed solids; teeth, limbs, ears, jaws and tails have tested
physical contacts. The palette uses the native base-color rendering path.
These assets are static and do not introduce animation or rigs.

Silhouettes and colors were checked against official
[Snubbull artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/209.png)
and [Granbull artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/210.png).
The models are original authored shapes without game textures or copied
meshes. A [CC Attribution Snubbull listing](https://sketchfab.com/3d-models/snubbull-51213d9e496d416980f0b21bc1546b79)
was found, but the listing did not establish a usable original rig for import.
No external model was imported.

`python3 -m unittest discover -s tools -p test_bulldog_sculpt.py` checks stored
closed topology, winding and normals, reproducible generation, height and
triangle budgets, grounded feet, real body/head/limb/tail/jaw/ear/tooth contacts,
fitted feature depths and the large canines' reach beyond the muzzle. Software
views and native battle captures check presentation, not gameplay behavior.

Verification for this pass: seven stored-geometry tests passed; the Rust
`every_exact_species_is_grounded_and_volumetric` catalog regression passed
across all 251 species; canonical model layout and 571 JSON storage documents
validated. Snubbull has 39 closed parts and 19,516 triangles; Granbull has
42 closed parts and 19,048 triangles. Front, oblique and rear software views
and both native battle captures were inspected. Temporary review artifacts
were removed afterward. Existing Sudowoodo geometry was inspected and preserved.
