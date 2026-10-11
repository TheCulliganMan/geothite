# Bellossom dancer sculpture

`tools/bellossom_sculpt.py` authors the original canonical Bellossom asset.
The battle species builder dispatches it before Blender batching and rejects
the old shared flower path. Sunkern and Sunflora retain their dedicated recipes.
Bellossom keeps its 0.95 presentation height and floor origin.

The dancer has a continuous small green torso and broad pale green head. Short
curved leaf-like arms form a raised arm and an arm crossing the chest. Glossy
dark eyes carry two ivory catchlights each, fitted orange cheeks frame a rosy
open smile, and the facial patches follow the actual stored head surface.

The skirt has fifteen closed curved leaves: five broad green outer leaves,
five alternating golden inner leaves, and five darker rear leaves. Each leaf
has a rounded transverse section, a narrow root inside the torso, a broad
middle and a tapered pointed end. These replace the old sparse flat triangles.

Two crown flowers have six broad rounded coral/orange petals each. Their faces
are subdivided and gently curled. Thick golden cups have raised annular lips,
descending inner walls and recessed olive floors. The petal roots remain below
the cavity floors, avoiding the initial visible petal intersections. The final
asset contains 45 closed parts and 31,900 triangles.

Silhouette and palette were checked against the
[official Bellossom artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/182.png).
External research found CC Attribution listings for
[Bellossom](https://sketchfab.com/3d-models/bellossom-8b77abfbcc9c472ba68d72609b682bc5)
and [a fan-authored Bellossom](https://sketchfab.com/3d-models/bellossom-9b2fb72a8a3c464aa6427bdad690d90b).
The metadata did not establish an original usable rig. No external mesh or
textures were imported. This original static sculpture uses the native
base-color path; it does not claim rigging or animation.

`python3 -m unittest discover -s tools -p test_bellossom_sculpt.py` checks
closed connected topology, winding, finite unit normals, deterministic recipe
parity, height, budget, part counts, fitted facial geometry and front triangle
clearance, actual torso/arm/skirt roots, all twelve petal roots, both crown
attachments and the actual recessed floors with clearance across each well.
Front/rear software reviews and native battle captures assess model
presentation, not gameplay behavior.

Final verification passed all eight Bellossom regressions. The final front and
rear software views and native battle capture were visually inspected. The Rust
catalog regression passed for the 220-species battle catalog; it verifies grounding and volume
rather than visual quality. Layout and storage validation passed for all 571
JSON assets. Temporary review images, reference artwork and logs were removed
after inspection.
