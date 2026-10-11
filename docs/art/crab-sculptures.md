# Krabby and Kingler sculptures

`tools/crab_sculpt.py` authors original four-legged crabs with orange domed
carapaces, rounded ivory lower shells, articulated ivory limbs, and open
hooked orange pincers. Krabby has two crown points and round pale eyes.
Kingler has five crown points, angular eyes and one substantially larger claw.
The crown points intersect the carapace, and cheek points grow from the lower
shell. Facial markings fit the actual stored triangles. Authoring height
remains 0.97 for both species, with normal runtime species scale.

Official [Krabby artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/098.png)
and [Kingler artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/099.png)
were inspected for anatomy and colors and are not shipped. The downloadable
[Kingler listing](https://sketchfab.com/3d-models/kingler-34df10aac7984ef681f11f2f7c6c7a7e)
advertises CC Attribution and zero animations. That metadata does not establish
a usable original rig; zero animations does not establish an absent skeleton.
Its official download endpoint returned HTTP 401 without authentication.
No external geometry was imported. These original authored meshes are static.

Stored-mesh checks cover closed connected outward surfaces, normals and
actual winding, exact recipe parity, preserved height and grounding, budgets,
four walking legs, two open claws, crown differences, Kingler's claw asymmetry,
actual solid intersections at articulations and roots, marking fitting and
front-face clearance. The initial rear legs were moved inward after an
independent stored-ray test found their roots outside the ivory lower shell.
All six stored-mesh regressions passed. Krabby has 27 parts and 32,904
triangles; Kingler has 30 parts and 26,412 triangles. Front, rear and oblique
software reviews are complete. Canonical copies match the tested candidates
byte for byte, and the main builder dispatches both through the pure recipe.
Layout and storage checks passed for 569 raw JSON model documents.

Fresh native presentation remains incomplete: the rebuilt reviewer returned
blank battle captures, as documented in [native review notes](native-pokemon-review.md).
The separate Rust all-251 loaded-mesh check passed after these replacements.
Software previews do not establish native integration or gameplay. The full visual
goal and usable rig search remain active. Temporary species candidates,
reference artwork, software renders and test logs were removed.
