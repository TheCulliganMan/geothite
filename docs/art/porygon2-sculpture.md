# Porygon2 sculpture

`tools/porygon2_sculpt.py` authors the original canonical Porygon2 mesh. The
battle-species builder dispatches it before Blender batching and rejects the
previous Porygon2 path. Porygon now has its own authored angular-cage recipe. Porygon2 keeps its 1.05 presentation height and grounded
origin, with 17 closed parts and 39,784 triangles.

One continuous rose volume forms the rounded torso, slender curved neck and
head. A seated cyan lower body follows the stored contour rings and has a
curved upper color boundary. The initial evenly spaced belly rings crossed
underlying contour knots and left red slivers showing through; using the actual
torso contour levels fixes those gaps.

Two rounded cyan paddles splay outward and support the lower body. A slender
rooted leaf tail curves backward and upward. The flattened blue spatulate bill
joins the head and slopes gently downward. Large fitted ivory eyes have thin
dark borders, curious vertical pupils and small ivory glints. Subtle seated
pink and pale-cyan reflections provide stylized toy highlights. These are
native static base-color surfaces rather than a claim of a skeletal rig or
custom glossy material.

Silhouette and palette were checked against the
[official Porygon2 artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/233.png).
External research inspected downloadable CC Attribution listings for
[Porygon2](https://sketchfab.com/3d-models/porygon2-76d884ae73f84ca59303a50b3188f0ab),
[Porygon2 Johto](https://sketchfab.com/3d-models/porygon2-johto-aab3755cf2474494bfcf67a792ed0935),
and [another Porygon2](https://sketchfab.com/3d-models/porygon2-bbe8d4b77bb2422b8edaa33329e4787f).
Their metadata did not establish a usable original rig. No external meshes or
textures were imported.

`python3 -m unittest discover -s tools -p test_porygon2_sculpt.py` checks closed
connected topology, outward winding, finite unit normals, recipe parity,
height and budget, fitted eyes/reflections and triangle clearance, physical
belly/bill/tail/paddle joins, both grounded paddles' support by the blue body,
and actual blue coverage of the front and rear lower torso. Join checks use
occupied ray intervals rather than bounding boxes.

Final verification passed all eight Porygon2 regressions, including exact
recipe parity. The first run found averaged tail-cap normals opposing a tiny
cap triangle; crease-aware normals applied after final coordinate quantization
correct that issue. Front/rear software views and the final native battle
capture were visually inspected. The native build and Rust 220-species
grounding/volume regression passed, and layout/storage validation passed for
all 571 JSON assets. Old completed-test object intermediates were removed to
provide build space. Temporary reference art, renders and logs were removed
after inspection. The full species visual review remains ongoing.
