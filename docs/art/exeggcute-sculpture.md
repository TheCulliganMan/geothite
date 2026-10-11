# Exeggcute sculpture

`tools/exeggcute_sculpt.py` authors six original psychic eggs. Five rounded
shells carry distinct fitted pale eyes, fine dark pupils, supported lids,
angular cracks and angry, smirking or tired mouths. A sixth egg is a real
hollow shell with a jagged broken rim and a golden yolk joined to its floor.
The rear eggs form a supported pile behind the front eggs; raised expressions
keep their faces visible. Authoring height remains 1.22 and normal runtime
species scale applies. Exeggutor retains its separate current recipe and asset.

The [official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/102.png)
was inspected for the six identities, expressions and broken shell and is not
shipped. Searches did not establish an accessible, usable original Exeggcute
rig. No external geometry was imported. This original authored mesh is static.

Nine stored-mesh checks cover closed outward parts, unit normals and actual
winding, recipe parity, grounding and height, budgets, six eggs and five faces,
a real shell cavity, a yolk joined to the shell floor, supporting intersections
under raised eggs, actual parent fitting, marking clearance and unoccluded
mouth samples on actual front triangles. Concave mouths are tested through
stored triangle centroids; their bounding-box centers can lie outside the
curved bands and are not valid visibility samples.

The first layout hid the broken egg and rear faces. The revised pile reveals
all six identities, and the yolk was enlarged upward while retaining the same
lower root. All nine final stored-mesh regressions passed. The model has 46 parts and
55,348 triangles across the six eggs. Final front, rear and oblique software
views were inspected, and the canonical copy matches the tested candidate
bytes exactly. The main builder dispatches Exeggcute through the pure recipe;
Exeggutor's canonical asset was not changed. Layout and storage checks passed
for 569 raw JSON model documents. Temporary candidates, reference artwork,
software renders and test logs were removed.

Fresh native presentation remains incomplete: the rebuilt reviewer returned
blank battle captures, as documented in [native review notes](native-pokemon-review.md).
The separate Rust all-251 loaded-mesh check passed after these replacements.
Software previews do not establish native integration or gameplay. The full visual
goal and usable rig search remain active. Temporary species candidates,
reference artwork, software renders and test logs were removed.
