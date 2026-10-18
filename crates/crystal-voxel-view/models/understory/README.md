# Woodland plants

Adapted from **Quaternius, Stylized Nature MegaKit (Standard/free edition)**:
https://quaternius.com/packs/stylizednaturemegakit.html
https://quaternius.itch.io/stylized-nature-megakit

The publisher dedicates these models to the public domain under CC0 1.0.
The original notice is preserved in `LICENSE.txt`. The source download is
`Stylized Nature MegaKit[Standard].zip`; only these four plants are used:

| Runtime asset | Original model |
| --- | --- |
| fern | Fern_1 |
| woodland_stem | Plant_1_Big |
| clover | Plant_7_Big |
| wildflower | Flower_3_Single |

The Rust example `import_understory_models` imports the publisher's glTF files,
clips transparent leaf and petal outlines into actual triangles, retains painted
colors and curved normals, adds reverse faces, grounds each model, and normalizes
its horizontal radius. Runtime instances vary rotation, size, height and palette;
no tree assets from this kit are loaded. No texture cards or new renderer material
are required. The JSON meshes are canonical artwork; Cargo compresses them only
in its ignored build output.

To regenerate, extract the free source archive outside tracked source, then run:

```sh
cargo run --locked -p crystal-voxel-view --example import_understory_models -- \
  "$PLANT_SOURCE/glTF" crates/crystal-voxel-view/models/understory
```

`PLANT_SOURCE` is the local extraction directory. Source archives and temporary
verification output must remain outside tracked source.
