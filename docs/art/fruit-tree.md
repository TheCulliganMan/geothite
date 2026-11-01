# Fruit tree refinement

The 3D `fruit_tree` actor uses the surrounding Johto forest's muted linear
foliage and bark palette. Ten unequal scaffold limbs carry 50 leafy twigs and
800 folded leaves. There are no solid foliage cores or spherical crown lobes.
Ten small berries hang from stems anchored to actual shoots.
The actor shares the terrain's solid material handle, including its exterior
sky fill, shadows and day/night response. It no longer uses the character PBR
material, which shaded the narrow leaves almost black beside the grove trees.

The model is 2.4 units tall, with its spreading crown bounded by 2.10 × 1.90
units. Five root flares ground the crooked trunk. The floor-centered interaction
root, source visibility, picked-tree behavior and collision remain unchanged.

The model has 17,192 triangles. Its components merge into the existing single
actor mesh; individual leaves add no render entities or draw calls. There are
no textures or external model dependencies.

The editable recipe is `fruit_tree()` in `tools/build-actor-props.py`.
`art/johto/source/actor-props-fruit-tree-refinement.blend` is the focused editable
source, superseding the fruit-tree collection in the original batch scene.
The runtime source is
`crates/crystal-voxel-view/models/actor_props/fruit_tree.mesh.json`.

Focused regeneration uses the existing Blender exporter:

```sh
blender -b -t 1 --python tools/build-actor-props.py -- \
  target/fruit-tree-authoring --only=fruit_tree --skip-preview
python3 tools/check-fruit-tree.py \
  --model target/fruit-tree-authoring/fruit_tree.mesh.json
```

The focused validator checks finite geometry, normalized normals, triangle
winding, volume orientation, floor root, canopy bounds, all 800 leaves and 50
twigs, fruit stems and an 18,000-triangle budget. It rejects the retired solid
foliage cores. The editable-source integrity check verifies the saved Blender
source against its manifest.

Validation: the focused geometry validator, editable Blender source integrity
check and native `render_at_location` build pass. Production captures were
inspected against 2D at `Route29 (11, 6)`, noon, with matching facing and
`state_preserved=true`. The first capture exposed excessive PBR darkening;
the material was changed to share terrain lighting, and foliage colors were
adjusted after another in-scene comparison. The reviewed image is saved locally
at `.local/art-review/fruit-tree.png` (ignored). This was visual QA under
concurrent build workloads, not a performance measurement.
