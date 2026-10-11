# Yanma dragonfly sculpture

`tools/yanma_sculpt.py` authors the original canonical Yanma asset. The battle
species builder dispatches it before Blender batching. Yanma retains its 0.95
presentation height and floor origin. The asset contains 66 closed parts and
27,036 triangles.

Two large green compound-eye housings join a red elongated thorax. Dark glossy
eye lenses carry two ivory catchlights each. Swept red forehead points rise
from a fitted central V saddle. Two rounded red mandibles carry a fitted dark
smile. Six angular jointed legs terminate in twelve pointed forked toes.

Four long curved ivory wings have rounded tips, closed thin sections and three
slender gray veins each. Orange panels fit both the upper and lower wing faces,
leaving ivory tips and a small boundary inset. The veins follow the actual
stored wing surfaces. These use opaque native base colors; transparency,
rigging and animation are not claimed.

A continuous abdomen rises behind the thorax into a rounded terminal cap. Three
closed annular dark-red collars fit the segment joints instead of the original
floating cream rings. Two broad red steering fins and two small ivory terminal
prongs attach to the cap.

Silhouette and palette were checked against the
[official Yanma artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/193.png).
External research found a CC Attribution
[Yanma listing](https://sketchfab.com/3d-models/yanma-0a85cd3a23514b9695e859a24f6d0536),
whose metadata did not establish a usable original rig. No external mesh or
textures were imported.

`python3 -m unittest discover -s tools -p test_yanma_sculpt.py` checks closed
connected topology, winding, finite unit normals, deterministic recipe parity,
height, budget, anatomy counts, fitted eye/forehead/mouth vertices and triangle
clearance, both orange wing faces, all twelve vein paths, wing and head roots,
six leg roots and twelve toe roots, actual annular collar/body intersections,
and the terminal cap, fins and prongs. Root checks compare actual occupied ray
intervals, retaining the empty interiors of annular collars.

Software views from above, in front and behind and the native battle capture
assess model presentation rather than gameplay behavior.

Final verification passed all nine Yanma geometry regressions. The final front,
rear and elevated software views and the native battle capture were visually
inspected. The Rust catalog regression passed for the 220-species battle catalog; it verifies
grounding and volume rather than visual quality. Layout and storage validation
passed for all 571 JSON assets. Temporary reference artwork, review renders and
logs were removed after inspection.
