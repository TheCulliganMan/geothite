# Donphan sculpture

`tools/donphan_sculpt.py` authors the original canonical Donphan mesh. The battle
species builder dispatches both elephant recipes before Blender batching;
the previous generic elephant body is removed. Donphan retains its 1.02
presentation height and grounded origin, with 49 closed parts and 29,596
triangles.

A broad pale barrel body supports a lowered rounded head and four stocky legs.
One continuous charcoal tire follows the rear back, crown and hanging trunk,
ending in a small upward curl. Nine separate rounded transverse tread plates
seat in the continuous tire. The dorsal band intersects the actual body at
sampled cross-sections rather than hovering over its silhouette.

Two broad swept charcoal ears have fitted burgundy lining. Their roots extend
into the head. Two long curved ivory tusks sweep outward from the cheeks.
Narrow fitted ivory eyes with dark slits give the elephant a watchful expression.
The fitted sockets and whites use subdivided contours to follow the steep
outer cranium without sinking their faces beneath it. Each leg has a fitted
charcoal ankle cuff and supports a grounded pale foot with three ivory toes.
A short armored tail joins the rear body.

Silhouette and palette were checked against the
[official Donphan artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/232.png).
The downloadable CC Attribution
[Donphan listing](https://sketchfab.com/3d-models/donphan-5b6b37d1d2a140a4a3442888860cda7e)
did not establish a usable original rig. Another
[Donphan listing](https://sketchfab.com/3d-models/donphan-f199887b29524cdb919a3826374acb42)
explicitly describes a Game Freak model; it was not imported. The animated
[Great Tusk listing](https://sketchfab.com/3d-models/great-tuskparadox-donphan-44a3695858494e948c3c37e996f4c949)
is a different species. No external meshes or textures were imported. This is
an original static native base-color sculpture.

`python3 -m unittest discover -s tools -p test_donphan_sculpt.py` checks closed
connected topology, outward winding, finite unit normals, recipe parity,
height and budget, fitted eye/ear/toe vertices and front triangle clearance,
head/tire/ear/tusk joins, all four leg/cuff/foot supports, all nine tread plate
joins, actual dorsal tire seating and the rooted tail. Join tests compare
occupied ray intervals. The first run identified ear roots outside the head
and eye socket faces below the head surface; the final geometry fixes both.

Final verification passed all nine Donphan geometry regressions. Final front
and rear software views, the side view and native battle capture were visually
inspected. The final native build succeeded. The final Rust catalog regression
passed for the 220-species battle catalog, and layout/storage validation passed for all 571
JSON assets. Catalog tests establish grounding and volume rather than visual
completion of all species. Temporary reference artwork, renders and logs were
removed after inspection.
