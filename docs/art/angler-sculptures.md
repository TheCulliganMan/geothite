# Chinchou and Lanturn sculptures

The canonical static meshes are original standard-library geometry authored in
`tools/angler_sculpt.py`. `tools/build-battle-species.py` dispatches both recipes
before Blender batching. Both retain the registry's 0.95 height and floor origin.

Chinchou has a wide blue oval body, large yellow eyes with black cross pupils,
two soft lower paddles, rounded pale side fins, and separate sweeping antennae
ending in hanging yellow teardrops with pale centers. Its tiny mouth remains
separate from the eyes; the former generic yellow face is removed.

Lanturn has a deeper, longer blue fish body, a fitted golden eye saddle, ivory
and red eyes, an open mouth, a tapered caudal peduncle, and rounded upper blue
and lower yellow tail lobes. One arched main stalk carries the larger light;
a short branch carries the smaller light. Twelve radial rings on the broad
mask sample the stored cheek surface rather than interpolating a flat decal.

The meshes use closed parts and base colors supported by the native renderer.
Pale lure centers suggest light through color; these assets do not add emission,
rigs, or animation. Fin folds and facial features follow their supporting
triangles. No external model, extracted game geometry, or textures are included.
The reference silhouettes and colors were checked against the official
[Chinchou artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/170.png)
and [Lanturn artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/171.png).
A targeted rigged-model search did not establish a usable licensed original rig
for these species, so this pass uses the original authored meshes.

`python3 -m unittest discover -s tools -p test_angler_sculpt.py` verifies stored
closed topology, orientation, normals, regeneration, bounds, triangle budgets,
real body/fin/stalk/lure/tail contacts, fitted feature depths, and mask triangle
clearance above the cheek. Multi-angle software views and native battle captures
check presentation. Native captures are visual checks, not gameplay evidence.

Verification for this pass: all eight stored-geometry tests passed, the Rust
`every_exact_species_is_grounded_and_volumetric` catalog regression passed,
canonical layout and all 571 JSON storage documents validated, and both native
battle captures were inspected. Temporary artwork references and renders were
removed after review.
