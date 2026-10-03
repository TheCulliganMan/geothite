# Optional local static model previews

STL previews duplicate the canonical model geometry and cannot carry animation,
rigs, materials or smooth vertex normals. They are not runtime inputs and are no
longer tracked or required by CI. The canonical human GLB, raw JSON static models, and editable Blender
sources remain in the repository with their full data.

The optional exporter writes deterministic binary STL files to ignored
`output/model-stl/` for local geometry inspection. Its existing 10 MB per-model
limit is retained; it refuses oversized models rather than splitting geometry.
All `.stl` output is ignored by Git, including custom output locations.

## Geometry fidelity

- Every source triangle is retained in its original primitive/index order,
  including duplicate or degenerate source triangles. There is no remeshing,
  decimation, voxelization, fitting, repair, or arbitrary rescaling.
- Static models have their part transforms baked into the stored positions.
  Fourteen lighthouse, ship, and traditional-room files explicitly mark
  `dimensions_pixels`: their authoring helpers divide each position component
  by its corresponding dimension. The exporter reverses this storage transform
  to recover native authored pixel units and proportions. Both the input and
  componentwise product round to float32, matching the runtime's precision.
  Generic `dimensions` and `bounds` metadata never imply a scale. World-instance
  footprint fitting, clearances, elevation, and visual wall height are not baked
  into the standalone authored asset.
- Human previews read named scenes from `johto_characters/catalog.glb`.
  The `.rig.json` CLI names are virtual scene selectors, not stored sidecars. All sixteen joints are placed in their unanimated
  bind pose by accumulating parent-relative translations, matching
  `johto_characters.rs`. Float32 conversion and each translation addition use
  the runtime's precision. No animation or world instance placement is applied.
- Runtime coordinates use +Y up and front +Z. STL coordinates are +Z up and
  front -Y: `(x, y, z) -> (x, -z, y)`, a proper +90-degree X-axis rotation with
  determinant +1. This changes neither shape nor handedness and preserves
  winding. Original units and origin are retained.
- Standard STL stores one facet normal per triangle, not smooth per-vertex
  normals, colors, materials, named parts, or rigs. Facet normals are computed
  from the transformed indexed vertices and source winding. A degenerate
  triangle gets a zero normal. The canonical GLB, raw JSON, and Blender sources retain
  the full smooth-normal, color/material, and articulation data.
- The exporter reads every written STL back and checks its count, byte length,
  every vertex/facet-normal float32 bit, and empty attribute bytes against the
  resolved source. Output is deterministic and has no timestamps.

## Reproduce and verify

From the repository root:

```sh
python3 tools/export-model-stl.py
python3 tools/export-model-stl.py --check
python3 tools/test_export_model_stl.py
```

The default writes to `output/model-stl/`, preserving the family/stem layout.
`--output-root /path/to/stl` selects another local output directory. To export
just one:

```sh
python3 tools/export-model-stl.py --model battle_species/gengar.glb
python3 tools/export-model-stl.py --model johto_characters/trainer.rig.json
```

`--models-root` can point at another canonical model tree. The exporter uses
only the Python standard library; it reads canonical GLB or raw JSON and does not need
Blender or the game's runtime bundle.

A full-tree run rejects unexpected/orphan `.stl` files in the selected output
directory, and `--check` fails for any missing, stale, or damaged expected local
export. This is an opt-in check after generating previews, not a clean-checkout
requirement. To list verified exports, use `--check --list-output-paths`. Paths
on stdout are relative to the output root; verification summaries go to stderr.
No path list is emitted if any model fails verification. Targeted `--model`
runs verify only those requested models and do not classify unrelated STLs as
orphans. CI checks the canonical model layout instead, rejecting duplicate STL
previews, same-asset GLB/JSON pairs, and a human catalog alongside the retired
rig JSON family in the runtime model tree.
