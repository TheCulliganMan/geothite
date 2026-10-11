# Teddiursa and Ursaring sculptures

`tools/bear_sculpt.py` authors original canonical static meshes for the bear
family. The main species builder dispatches both recipes before Blender
batching. Both retain the registry's 1.14 height and floor origin.

Teddiursa has an oversized round orange head, large dark eyes with ivory
glints, broad cream cheeks, a small dark nose, a tiny mouth and tongue, and
round ears with dark centers. One paw reaches toward its mouth, while the other
extends to the side. Short bent legs support round paws with ivory claws. A
small round tail completes the cub silhouette. The cream forehead crescent is
a closed fitted strip with an actual opening and exposed orange skin inside.

Ursaring has a massive pear-shaped torso, thick upright neck, smaller stern
head, broad cream snout, dark nose and frown. Pointed layered fur follows the
chest and neck surface at its shoulders; projecting tips extend the side
silhouette. Heavy arms end in four long claws and a side thumb claw per hand, and broad grounded
feet carry three forward claws each. The cream chest ring samples the actual
curved body; its center exposes the brown skin rather than covering it with
an extra brown disc. The ring and crescent are markings, not raised tube props.

The meshes use the native static base-color path, without animation or rigs.
All parts are closed solids; limbs, paws, claws, heads, ears and tails have
physical support. No copied geometry, game textures, or external model is
included. Official [Teddiursa artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/216.png)
and [Ursaring artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/217.png)
provided silhouette and color references. A
[CC Attribution Teddiursa listing](https://sketchfab.com/3d-models/teddiursa-4d2f736edc704e3c855d6a5ff98202d0)
was found, but did not establish a usable original rig for import.

`python3 -m unittest discover -s tools -p test_bear_sculpt.py` checks stored
closed topology, winding, normals, regeneration, grounded bounds and triangle
budgets; distinct adult/cub proportions; body, head, ear, limb, claw, muzzle,
nose and held-paw contacts; supported shoulder fur; and fitted markings with
real gaps. Multi-angle software renders and native battle captures verify
presentation. Captures are visual checks, not gameplay evidence.

Verified this pass: all nine stored bear regressions passed, including front
fur triangle clearance. The shared concave fitting helper was moved into
`sculpt_geometry.py`; all seven Aipom regressions passed, including unchanged
canonical regeneration. The Rust catalog regression passed across the 220-species battle catalog, canonical layout and 571 JSON documents validated, and final native
captures for both bears were inspected. Teddiursa has 37 closed parts and
23,880 triangles; Ursaring has 48 closed parts and 27,116 triangles. Review
references, captures and logs were removed afterward.
