# Voltorb and Electrode sculptures

`tools/orb_sculpt.py` authors original smooth red and ivory molded spheres.
Each hemisphere is a closed outward solid with a shared 64-point equator.
The color order is reversed for Electrode. Voltorb preserves its shared actor
asset identity and 0.82 height; Electrode preserves its battle identity and
0.94 height. Normal runtime species scale applies.

Voltorb has fitted dark sockets, large pale angular eyes and fine stern pupils.
Electrode has narrow dark pupils, supported raised brows and a broad curved
pale grin with two short tooth divisions. The details follow actual stored
shell or marking triangles rather than floating in front of a coarse sphere.

Official [Voltorb artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/100.png)
and [Electrode artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/101.png)
were inspected for expressions and colors and are not shipped. The public
[Voltorb listing](https://sketchfab.com/3d-models/voltorb-c474c0d3b97f4f888043fe7e25de8f8e)
advertises CC Attribution and zero animations, but does not establish a usable
original rig. Zero animations does not establish an absent skeleton. Its
official download endpoint returned HTTP 401 without authentication.
No external geometry was imported. These original authored meshes are static.

Six stored-mesh checks cover closed geometry, outward winding and unit
normals, exact recipe parity, grounding and heights, budgets, identical shell
equators, distinct expressions, actual parent fitting and front clearance.
All six final stored-mesh regressions passed. Voltorb has 8 parts and 8,248
triangles; Electrode has 10 parts and 23,480 triangles. Final front, rear and
oblique software views were inspected. The Voltorb candidate was regenerated
after checking the existing canonical height: the original ball dimensions
were 0.64, but its shared export normalization was 0.82. The replacement
preserves that actual 0.82 height.

Canonical copies match the tested candidate bytes exactly. Both builders
route their respective identities through the pure recipe. Layout and storage
checks passed for 569 raw JSON model documents. Temporary candidates,
reference artwork, software renders and logs were removed.

Fresh native presentation remains incomplete: the rebuilt reviewer returned
blank battle captures, as documented in [native review notes](native-pokemon-review.md).
The separate Rust all-251 loaded-mesh check passed after these replacements.
Software previews do not establish native integration or gameplay. The full visual
goal and usable rig search remain active. Temporary species candidates,
reference artwork, software renders and test logs were removed.
