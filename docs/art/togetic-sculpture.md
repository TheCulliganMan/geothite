# Togetic sculpture

`tools/togetic_sculpt.py` authors the anatomy and `tools/egg_fairy_glb.py` exports
the canonical `togetic.glb` standard skin. The battle builder dispatches this rig
before generic Blender batching. The complete sculpted geometry, normals,
colors and triangle order are preserved at the original 0.94 authoring height.
The duplicate static JSON is removed.

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
imported. This original native base-color sculpture now has an authored standard glTF skin.

`python3 -m unittest discover -s tools -p test_togetic_sculpt.py` checks closed
connected topology, winding, unit normals, deterministic recipe parity,
height and triangle budget, actual open ring centers, fitted torso/face/foot
vertices and triangle clearance, wing surface seams, and physical joins of
head, hands, feathers, legs and pointed feet. Join checks compare occupied
ray intervals rather than bounding boxes.

Ten joints articulate the torso, long neck, head, wings, arms and leg middles. The torso and every fitted ring use the same spatial weight field. Each wing and its fitted seams share one owner; the facial details follow the head. Feet, pointed toes and toe creases stay on the identity floor root. The 37 parts retain 16,755 vertices and 26,728 triangles.

Idle loops over 3.2 seconds; attack and hit clips use the shared battle cue timing and return to neutral. The shared skin regressions check floor contact, fitted-detail ownership, posed physical joins, winding, strain, canonical bytes and the existing loader budgets. Shiny checks are intentionally skipped.

Canonical neutral f32 digest: `5971f35977b196a3a2a62472e147b9b0743a53573c66770e9b4bdbc892dea912`. The skin uses 710840 bytes and 178 accessors, within the existing 1 MiB and 256-accessor limits.

All 18 sculpture tests and six shared skin tests passed, as did the two focused Rust rig/GPU regressions and the two catalog/timing regressions. Blender regeneration is byte-identical for both GLBs. The production native client built successfully; normal 3D/2D battles against Pidgey were visually inspected with matching game state and accepted, visibly rendered terrain. Shiny checks were skipped as requested.

Final normal pair: core frame 3708, hash 1318871937, LCD frame 4500, virtual time 84.546857166 seconds; `state_preserved=true`. Review prefix: `target/pokemon-native-review/togetic-normal`.
