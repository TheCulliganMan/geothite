# Misdreavus sculpture

`tools/misdreavus_sculpt.py` authors the original canonical Misdreavus mesh.
The species builder dispatches the recipe before Blender batching; the former
straight locks, pink terminal balls and uniform necklace have been removed.
The 1.02 presentation height and floor origin remain unchanged.

A broad rounded blue face sits above a compact continuous body with five soft
hem lobes. Six asymmetric volumetric locks sweep and curl around the head,
including the high crown plume and a rear lock. Five have tapered pink ends.
Each color transition shares a complete elliptical section between two closed
solids, rather than attaching a separate pink ball. Warm gold almond eyes,
large ruby irises, dark inward pupils and ivory glints fit the actual stored
face triangles. A small curved smile follows the same surface.

Five graduated ruby necklace gems follow the lower face, with a larger central
orb, medium neighbors and small end beads. Their closed volumes overlap the
head or body and a curved cord. A rear cord completes the loop. Rose and ivory
highlights fit the stored gem surfaces; they use ordinary base colors rather
than claiming emissive or glass materials. The asset has 41 closed parts and
23,794 triangles. It uses the native static mesh path and has no imported rig
or animation.

Silhouette, features and palette were checked against the
[official Misdreavus artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/200.png).
The [Koof Misdreavus listing](https://sketchfab.com/3d-models/misdreavus-55233fb017cb4d0a9f20dcd1c7daca11)
and its [public metadata](https://api.sketchfab.com/v3/models/55233fb017cb4d0a9f20dcd1c7daca11)
describe a Blender-authored rig, one animation and CC Attribution 4.0 licensing.
Its official download endpoint returned HTTP 401 requiring authentication;
no external model, viewer geometry or textures were imported.

`python3 -m unittest discover -s tools -p test_misdreavus_sculpt.py` checks
closed topology, winding, connectivity, finite unit normals, exact regeneration,
height, budget, real head/body/hair and gem/cord overlap, complete nonzero pink
curl seams, fitted expression and highlight layers, front marking triangle
clearance, and the five-lobed hem. Software views and a native battle capture
check model presentation rather than gameplay behavior.

All nine sculpture regressions passed for the final stored asset. Front,
rear and oblique software views and the rebuilt native battle renderer were
inspected with the local external content pack. The Rust test covering
grounded volumetric bounds for the 220-species battle catalog passed, together with canonical
layout checks and storage validation of 571 JSON model documents.
