# Togepi shell sculpture

`tools/togepi_sculpt.py` authors the original canonical Togepi mesh. The battle
species builder dispatches it before Blender batching and rejects the old
Togepi path. Togetic now has its own dedicated sculpture. Togepi keeps its 0.94
presentation height and floor origin. The asset has 27 closed parts and 28,896
triangles.

The ivory egg is a continuous closed shell with ten cracked rim teeth, an inner
wall and a recessed interior floor. Seven red and blue triangle/lozenge rings
have actual open centers exposing the shell beneath, with fitted inner walls
and subdivided annular faces. Two smaller red triangle flecks fit the front and
rear shell surfaces. These replace the old floating solid triangle markers.

The cream head is one rounded volume with five swept crown points. Its lower
cream core extends into the egg and joins the shell floor. The final crown uses
a deformed sphere rather than the early outlined cap, which had visible flat
edges. Small dark vertical eyes have ivory glints and soft side creases. A
fitted delighted open grin has a rosy lower tongue. Two short raised cream
hands and two supporting legs join grounded rounded feet with fitted toe
creases.

Silhouette and palette were checked against the
[official Togepi artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/175.png).
External research found CC Attribution listings for
[Togepi](https://sketchfab.com/3d-models/togepi-f091c97e48a2423d98d3d1b839d6211b),
[Togepi_upload](https://sketchfab.com/3d-models/togepi-upload-575545f917724e22b69744bbf67aa857),
described as made in Blender, and
[a Maya-authored Togepi](https://sketchfab.com/3d-models/togepi-caa6fdac0d7f4f178dc6aeadbc2f1d1b).
Their metadata did not establish a usable original rig. No external meshes or
textures were imported. This original static sculpture uses native base colors
and does not claim rigging or animation.

`python3 -m unittest discover -s tools -p test_togepi_sculpt.py` checks closed
connected topology, winding, finite unit normals, deterministic recipe parity,
height, budget, anatomy counts, actual cracked rim teeth and cavity depth,
actual empty color-ring centers, all fitted face/foot/shell vertices and front
triangle clearance, and physical head/hand/leg/foot joins. Root checks use
actual occupied ray intervals rather than bounds alone. Front/rear software
views and a native battle capture assess model presentation rather than
gameplay behavior.

Final verification passed all nine Togepi geometry regressions. The final
front/rear software views and native battle capture were visually inspected.
The Rust catalog regression passed for the 220-species battle catalog; it verifies grounding
and volume rather than visual quality. Layout and storage validation passed
for all 571 JSON assets. The initial native link exhausted disk space; removing
old completed-test object intermediates allowed the final build to succeed.
Temporary reference artwork, review renders and logs were removed after
inspection.
