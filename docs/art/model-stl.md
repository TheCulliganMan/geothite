# Native GitHub 3D models

Each canonical `.mesh.json` or `.rig.json` has an adjacent `.stl` containing its
actual indexed triangles. For example, open
`crates/crystal-voxel-view/models/battle_species/gengar.stl` on GitHub to rotate
and inspect the Gengar mesh. These are ordinary raw binary STL files, not
Base64, compressed wrappers, Git LFS pointers, screenshots, or redesigned models.

[GitHub's native 3D viewer supports STL files up to 10 MB](https://docs.github.com/en/repositories/working-with-files/using-files/working-with-non-code-files#3d-file-viewer).
The exporter refuses an oversized model; it never silently splits one.

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
- Character `.rig.json` exports resolve `shared.geometry.json` by its library
  identity and primitive index. All sixteen joints are placed in their unanimated
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
  triangle gets a zero normal. The canonical raw JSON and Blender sources retain
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

The default writes beside the canonical models. `--output-root /path/to/stl`
keeps the same family/stem layout in a separate directory. To export just one:

```sh
python3 tools/export-model-stl.py --model battle_species/gengar.mesh.json
```

`--models-root` can point at another canonical model tree. The exporter uses
only the Python standard library; it reads raw JSON directly and does not need
Blender or the game's runtime bundle.

A full-tree run rejects unexpected/orphan `.stl` files and `--check` also fails
for any missing, stale, or damaged expected export. To obtain an exact staging
list after all exports pass, use `--check --list-output-paths`. Paths on stdout
are relative to the output root; verification summaries go to stderr. No path
list is emitted if any model fails verification. Targeted `--model` runs verify
only those requested models and do not classify unrelated STLs as orphans.
