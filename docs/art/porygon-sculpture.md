# Porygon sculpture

`tools/porygon_sculpt.py` authors the original canonical Porygon mesh. The battle
species builder dispatches both virtual ducks before Blender batching; their
previous shared primitive recipe is removed. Porygon keeps its 1.05 presentation
height and grounded origin, with 16 closed parts and 2,008 triangles.

Authored transverse polygon cages form the rose hull, short angular neck,
long sloping head and tapered cyan bill. The cyan breast seats in the forward
lower hull and rises toward the neck. Two grounded block-shaped paddles splay
outward, and a narrow rooted cyan blade forms the upright tail. These cages use
flat face normals to retain the original virtual duck's crisp panels rather
than generic triangulated ellipsoid shapes.

Hexagonal ivory eye panels with thin dark borders follow the actual head side
panels. Small circular dark pupils and ivory glints fit those stored surfaces.
The initial front-projected eyes crossed the head's silhouette bevels; fitting
in each side panel's rotated coordinate frame keeps them flat and readable.
The initial cyan breast left the lower red hull exposed, so its authored cage
was expanded to cover the forward hull with the intended cyan chest.

Silhouette and palette were checked against the
[official Porygon artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/137.png).
External research inspected the CC Attribution
[Porygon listing](https://sketchfab.com/3d-models/porygon-aeb8823602fa4733a26d0214bf6c29a2),
the CC Attribution-ShareAlike
[Pokémon 137 listing](https://sketchfab.com/3d-models/pokemon-137-porygon-5ccc6c23e0b64e72b967b67851314ee1),
and [another Porygon listing](https://sketchfab.com/3d-models/porygon-d468bad5f0da414b80a7a10e0ce06963).
Their metadata did not establish a usable original rig. No external meshes or
textures were imported. This is original static native base-color geometry.

`python3 -m unittest discover -s tools -p test_porygon_sculpt.py` checks closed
connected topology, outward winding, finite unit normals, exact recipe parity,
height and budget, fitted side-eye vertices and triangle clearance, all
hull/neck/head/bill/tail/foot joins, grounded feet, actual cyan coverage of the
forward hull and flat normals on the main angular cages. Physical joins use
occupied ray intervals rather than bounding boxes.

Final verification passed all eight Porygon geometry regressions. Final front
and rear software renders and the native battle capture were visually
inspected. The Rust 220-species grounding/volume regression passed, as did
layout/storage validation for all 571 JSON assets. The initial native link
failed because the disk filled; removing old completed-test object
intermediates provided space, and the final native build succeeded. Temporary
reference artwork, renders and logs were removed after inspection. The full
species visual review remains ongoing.
