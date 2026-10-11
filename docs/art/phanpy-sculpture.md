# Phanpy sculpture

`tools/phanpy_sculpt.py` authors the original canonical Phanpy mesh. The battle
species builder dispatches it before Blender batching and rejects the previous
Phanpy path. Donphan retains its existing recipe pending a separate review.
Phanpy preserves its 1.02 presentation height and floor origin. The asset has
32 closed parts and 40,500 triangles.

The rounded horizontal torso supports a broad baby elephant head with small
fitted vertical eyes and ivory reflections. A short, continuously curved trunk
curls upward into a rounded tip with two fitted dark nostrils. A warm fitted
forehead saddle replaces the two detached orange nose beads. A fitted open
smile and rosy interior sit below the trunk.

Two drooping rounded blue ears each carry two curved warm lower lobes fitted
to their actual surfaces. Three planted rounded legs taper into the torso;
a fourth bent foreleg lifts a rounded foot in a playful pose. Each of the four
feet has a shallow ivory toenail. A short tapered tail joins the rear torso.

Silhouette and palette were checked against the
[official Phanpy artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/231.png).
External research inspected the CC Attribution
[Phanpy listing](https://sketchfab.com/3d-models/phanpy-da39b4e9532e4640acffe9e19d999a53),
[a second Phanpy](https://sketchfab.com/3d-models/phanpy-8e83653554f447d6bd7d7957953c2a9c),
and a Blender/ZBrush-authored
[KAWAI Elephant](https://sketchfab.com/3d-models/kawai-elephant-phanpy-pokemon-3d-model-0c1d4e69ba17471b8a75db2aabd62464).
The latter two were not downloadable. The available listing's metadata did
not establish a usable original rig. No external meshes or textures were
imported. This is an original static native base-color sculpture.

`python3 -m unittest discover -s tools -p test_phanpy_sculpt.py` checks closed
connected topology, outward winding, finite unit normals, deterministic recipe
parity, height and triangle budget, fitted ear/face/nail vertices and triangle
clearance, head/trunk/tip/ear joins, planted and lifted leg/foot support, and the
rooted tail and raised trunk tip. Support checks compare actual occupied ray
intervals. A test suffix-matching error originally confused right rear parts
with left rear parts; complete suffix boundaries now keep those checks exact.

Final verification passed all eight Phanpy geometry regressions, plus layout
and storage validation for all 571 JSON assets. Front, rear and three-quarter
software views and the final native battle capture were visually inspected.
The final native build succeeded. A catalog regression passed before the last
upper-leg taper adjustment; the final catalog rerun could not compile because
a concurrently authored test initializer in `src/apron.rs:217` omitted the new
`SurfaceMeshData.cutaway_ranges` field. That unrelated source was preserved.
The final Python geometry regressions cover the actual final Phanpy asset; no
claim is made that the final Rust catalog rerun passed. Review renders, reference
artwork and temporary logs were removed after inspection.

A subsequent catalog rerun during the Donphan review passed for the 220-species battle catalog
with the final Phanpy asset present, after the concurrent apron test initializer
was repaired. This clears the previously recorded catalog verification limit.
