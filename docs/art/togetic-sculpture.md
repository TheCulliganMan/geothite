# Togetic sculpture

`tools/togetic_sculpt.py` authors the original canonical Togetic asset. The
battle-species builder dispatches it before Blender batching. It preserves the
0.94 presentation height and grounded origin, with 37 closed parts and 26,728
triangles.

A single pear-shaped torso continues into the slender neck. The rounded head
has three continuous swept crown points, small vertical eyes, ivory glints,
soft brows and a delighted rosy smile. Six flattened curved wing feathers
sweep upward from the torso; four subtle surface seams describe their shape.
Two asymmetric raised leaf hands, short supporting legs and pointed ivory feet
replace the previous generic arms, beige feet and short triangular wings.

Six fitted red and blue annular triangle/lozenge markings have actual empty
centers exposing the torso. A smaller blue teardrop fits the front body.
Front and rear markings follow the stored torso geometry.

Silhouette and palette were checked against the
[official Togetic artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/176.png).
The CC Attribution
[Togetic listing](https://sketchfab.com/3d-models/togetic-9ac8a53c471a49899dddd97b89da969b)
did not establish a usable original rig. No external mesh or texture was
imported. This original native base-color sculpture is static.

`python3 -m unittest discover -s tools -p test_togetic_sculpt.py` checks closed
connected topology, winding, unit normals, deterministic recipe parity,
height and triangle budget, actual open ring centers, fitted torso/face/foot
vertices and triangle clearance, wing surface seams, and physical joins of
head, hands, feathers, legs and pointed feet. Join checks compare occupied
ray intervals rather than bounding boxes.

Final verification passed all nine Togetic geometry regressions. Front, rear
and three-quarter software renders and a native battle capture were visually
inspected. The Rust grounding/volume regression passed for the 220-species battle catalog,
and layout/storage validation passed for all 571 JSON assets. Those catalog
checks establish geometry integrity rather than completion of the visual
review. The native build succeeded after old completed-test object files were
removed to provide disk space. Temporary reference artwork, renders and logs
were removed after verification.
