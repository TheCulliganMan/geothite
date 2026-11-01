# Aipom sculpture

`tools/aipom_sculpt.py` authors the canonical static Aipom mesh with standard
library geometry. The main species builder dispatches this recipe before
Blender batching. The original registry height of 1.02 and floor origin remain.

Aipom has a slim rounded purple torso, short neck, large rounded head, generous
upright tilted ears, and a cream chest oval. Its cream face follows the stored
head surface and has a shallow crown notch. Layered round eyes have ivory
whites, blue grey irises, dark pupils and small glints. A closed fitted crescent
grin carries one ivory tooth band with four divisions. Ear-clipped caps with two levels of subdivision
keep both face and grin fitted to their curved supports without folding across
the smile or crown notch.

Two arms reach in different directions, with purple mittens and cream palms.
Bent legs meet broad grounded cream feet with fine fitted toe folds. Three soft
pointed crown strands replace the previous flat triangles. The long prehensile
tail curls behind the body to a broad cream palm. Three short fingers radiate
in different directions and end in bulbous pads, replacing the former parallel
fingers. The tail, palm, digits and pads have real geometric overlap.

Colors and silhouette were checked against the official
[Aipom artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/190.png).
These are original authored shapes, without copied meshes or game textures.
A targeted model search did not establish an accessible licensed original
Aipom rig for import. This asset uses the native static mesh and base-color
path; it does not introduce a rig or animation.

`python3 -m unittest discover -s tools -p test_aipom_sculpt.py` checks stored
closed topology, normals and winding, reproducible export, grounded height,
triangle budget, body/head/limb contacts, the curled tail and radial finger
contacts, fitted masks and eyes, and front-triangle clearance for face and
teeth. Multi-angle software renders and a native battle capture check visible
presentation. A capture is a model review, not proof of gameplay behavior.

Verified this pass: seven stored-geometry tests passed; the Rust catalog
regression passed across the 220-species battle catalog; canonical model layout and 571 JSON
storage documents validated. The final sculpture has 50 closed parts and
30,016 triangles. Front, oblique and rear software views and a native battle
capture were inspected. Temporary references, renders and review logs were
removed afterward.
