# Fruit tree refinement

The 3D `fruit_tree` actor uses a continuous, fluted trunk with five staggered
boughs, recessed foliage, 300 closed cupped leaves, and nine hanging berries.
The berries have depressed tops, small stems and shoot leaves. Their placement
leaves fruit visible from both sides of the crown. This replaces the former
five bare faceted lobes and surface-mounted fruit dots.

The model retains its 1.08-unit height and floor-centered root. Its new width
and depth are 0.810606 × 0.727528 units, within the previous
0.874760 × 0.802672 footprint. The actor registry, presentation scale, source
visibility rules, picked-tree behavior, interaction and collision are unchanged.
The pack and original 2D sprite are unchanged.

The model has 8,132 triangles, up from 2,212. All components still merge into
the existing single actor mesh; the individual leaves add no render entities
or draw calls. No textures or external model dependencies are introduced.

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
winding, closed-volume orientation, the existing size contract, leaf detail,
fruit stems and the triangle budget. The broader actor checker currently
expects the retired `slowpoke.mesh.json`; that independent migration prevents
its full-catalog run from completing.

The native location tester was rebuilt with one Cargo job and the actual
verified browser pack. Paired 2D/3D captures at `Route30 (9, 6)`, noon, preserve
the game state (`state_preserved=true`). The before/after review confirms
readable berries and layered leaves, a continuous grounded trunk, and unchanged
placement beside the house. Front and rear Blender previews were also inspected;
the first preview's trunk seams and detached leaf shell were corrected before
the production rebuild. The focused validator and native build pass.
