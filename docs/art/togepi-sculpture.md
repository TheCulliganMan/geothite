# Togepi shell sculpture

`tools/togepi_sculpt.py` authors the anatomy and `tools/egg_fairy_glb.py` exports
the canonical `togepi.glb` standard skin. The battle builder dispatches this rig
before generic Blender batching. The complete sculpted geometry, normals,
colors and triangle order are preserved at the original 0.94 authoring height.
The duplicate static JSON is removed.

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
textures were imported. This original sculpture uses native base colors and an authored standard glTF skin.

`python3 -m unittest discover -s tools -p test_togepi_sculpt.py` checks closed
connected topology, winding, finite unit normals, deterministic recipe parity,
height, budget, anatomy counts, actual cracked rim teeth and cavity depth,
actual empty color-ring centers, all fitted face/foot/shell vertices and front
triangle clearance, and physical head/hand/leg/foot joins. Root checks use
actual occupied ray intervals rather than bounds alone. Front/rear software
views and a native battle capture assess model presentation rather than
gameplay behavior.

Seven joints articulate the shell, rounded star head, raised hands and leg middles. The shell and all seven open rings share one rigid owner, preserving their cavity and fitted surfaces. Facial details follow the head. Feet and toe creases stay on the identity floor root. The 27 parts retain 17,825 vertices and 28,896 triangles.

Idle loops over 3.2 seconds; attack and hit clips use the shared battle cue timing and return to neutral. The shared skin regressions check floor contact, fitted-detail ownership, posed physical joins, winding, strain, canonical bytes and the existing loader budgets. Shiny checks are intentionally skipped.

Canonical neutral f32 digest: `975cb4f931bddf2545df3621a30a743f70bb3662f218c61eb5ae90879d34b53b`. The skin uses 720576 bytes and 119 accessors, within the existing 1 MiB and 256-accessor limits.

All 18 sculpture tests and six shared skin tests passed, as did the two focused Rust rig/GPU regressions and the two catalog/timing regressions. Blender regeneration is byte-identical for both GLBs. The production native client built successfully; normal 3D/2D battles against Pidgey were visually inspected with matching game state and accepted, visibly rendered terrain. Shiny checks were skipped as requested.

Final normal pair: core frame 2781, hash 1494234955, LCD frame 3504, virtual time 64.27343179 seconds; `state_preserved=true`. Review prefix: `target/pokemon-native-review/togepi-normal`.
