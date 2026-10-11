# Qwilfish sculpture

`tools/qwilfish_sculpt.py` authors the original canonical Qwilfish mesh. The
species builder dispatches it before Blender batching; the former overlapping
belly sphere, single planar spine ring, generic leaf tail and belly dots have
been removed. The 0.91 presentation height and floor origin remain unchanged.

Two closed hemispheres share the complete equator of one inflated ellipsoid:
teal above and pale cream below. Their side normals follow the same ellipsoid,
while their internal closing disks retain separate normals. Fourteen tapered
spines point in authored three-dimensional directions, with eight teal upper
cones and six cream lower cones. Every root overlaps its matching hemisphere.
Pointed ivory eyes, small high pupils and a pursed pink lip oval give the face
its wary expression. Small raised cream pectoral fins have fitted V folds.

A curved teal caudal peduncle joins the lower rear body to a rounded paddle
fin. Its teal rim surrounds fitted cream centers on both faces, each with two
oblong teal markings. This replaces the unrelated spots on the old belly. The
asset has 36 closed parts and 13,644 triangles. It uses the native static mesh
and ordinary base-color path; no external mesh, rig or animation was imported.

Silhouette, palette and features were checked against the
[official Qwilfish artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/211.png).
A [CC Attribution Qwilfish listing](https://sketchfab.com/3d-models/qwilfish-175582eff9c04bfd83b48d29c1d18db1)
was found through the official model search. Its search metadata listed zero
animations and did not establish an original usable rig, so it was not adopted.

`python3 -m unittest discover -s tools -p test_qwilfish_sculpt.py` checks
stored closed topology, connectivity, winding, finite unit normals, exact
regeneration, height, budget, the complete shared equator and matching side
normals, every spine root, actual lip and tail overlap, fitted face/fins/markings,
and front triangle clearance over the eye sockets and both cream tail faces.
Multi-angle software views and a native battle capture check model presentation,
rather than gameplay behavior.

All eight sculpture regressions passed for the final stored asset. Front,
rear and oblique software views and the rebuilt native battle renderer were
inspected with the local external content pack. The Rust test covering grounded
volumetric bounds for the 220-species battle catalog passed, together with canonical layout
checks and storage validation of 571 JSON model documents.
