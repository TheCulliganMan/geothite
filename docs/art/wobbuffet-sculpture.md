# Wobbuffet sculpture

`tools/wobbuffet_sculpt.py` authors the original canonical Wobbuffet mesh. The
species builder dispatches it before Blender batching; the obsolete generic
oval/body/arm recipe has been removed. Its 1.14 height and floor origin remain
unchanged.

The continuous blue body has a broad rounded upper silhouette and narrow lower
trunk. Two curved elastic arms lift above the head, with one hand bent into a
salute. Four soft basal lobes replace the former separate humanoid legs and
flat feet. A wide fitted burgundy mouth has a jagged upper edge and rounded
pink lower tongue. The inward squeezed eyelids fit the actual body triangles.
The black tail grows from the lower back, curves outward and ends in a smooth
tilted paddle with two ivory eye ovals and dark pupils. All features use closed
geometry on the native static mesh/base-color path; no rig or animation is
claimed.

The tail's outward curve keeps its eyes visible at an oblique battle angle;
its root still joins the lower back with actual volume overlap. The asset has
19 closed parts and 19,260 triangles.

Silhouette, expression and palette were checked against the
[official Wobbuffet artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/202.png).
An external model search did not establish a downloadable original open rig
for Wobbuffet. The [gre_san listing](https://sketchfab.com/3d-models/wobbuffet-20618e7d41224126b2a8715ab5ae35ea)
had no download/license or rig evidence, so no external model was imported.

`python3 -m unittest discover -s tools -p test_wobbuffet_sculpt.py` checks
stored closed topology, winding, connectivity, finite unit normals, exact
regeneration, height, budget, volumetric arm/hand/foot/tail attachments, fitted
expression and eye layers, and front mouth/tongue triangle clearance over the
actual supporting meshes. A projected stored-mesh ray check also requires both
tail eye centers to clear the body at an oblique front angle. Multi-angle
software views and the native battle
renderer check presentation, rather than gameplay behavior.

All nine sculpture regressions passed for the final stored asset. The Rust
test covering grounded volumetric bounds for the 220-species battle catalog passed, as did
the canonical layout check and storage validation of 571 JSON documents. The
rebuilt native battle renderer was inspected with the local external content
pack; both tail eyes remain visible in its final capture.
