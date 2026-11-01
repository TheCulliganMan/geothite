# Wooper and Quagsire sculptures

`tools/axolotl_sculpt.py` replaces the shared sphere-body recipe with two
distinct original silhouettes. Wooper has a small pear belly, a broad head,
tapered branching pink gills and three fitted blue belly bars. Quagsire has a
continuous bulky body and cranium, fleshy hanging arms, a wide shallow smile
and a lavender fin that follows its back into the rising tail. Both have
flattened paddle tails, tiny fitted black eyes and rounded supporting legs
that overlap the body and grounded three-lobed soles.

Each species keeps one canonical static JSON mesh, dispatched by the standard
battle-species authoring entry point. The previous physical heights remain
0.70 units for Wooper and 1.13 units for Quagsire. No source sprites, pack
content, downloaded meshes or duplicate geometry catalogs are consumed. This
increment does not add skinning or animation.

```sh
python3 tools/axolotl_sculpt.py --output crates/crystal-voxel-view/models/battle_species
python3 tools/test_axolotl_sculpt.py
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo run --locked -p crystal-bevy --example immersive_battle_3d --features location-tester -- content-packs/realtime-clock.browser.crystalpack --enemy-gust --species wooper --size 1000x700 --screenshot target/wooper-review.png
# Repeat the native fixture with --species quagsire.
```

The six stored-model checks cover exact regeneration, physical dimensions,
closed connected surfaces, oriented edges, normals and winding, fitted facial
and belly markings, distinct head/body proportions, real paddle cross-sections
and volumetric overlap between body, legs and soles. The contact regression
catches the floating ankle/foot gap found during initial art inspection.
Both models stay below 7,000 triangles and 400,000 raw JSON bytes. Software
raster inspection compares the previous and new neutral geometry, including
rear views of the tail and dorsal fin. Native fixtures verify static
presentation; they do not establish animation or gameplay correctness.
Both final models were inspected in native 1000×700 battle fixtures using the
ignored local pack above. The eyes, gills, belly bars, smile, tail and fins
remain visible; feet contact the arena and the tails fit the battle camera.
Wooper retains its smaller species-based runtime scale. Source-storage and
single-model-layout validation, all six model checks, and the Rust species
loader regression passed. The broader Python species validator
still encounters the previously documented macOS Bayleef regeneration mismatch.

## External rig search

[Wooper Rigged And Unrigged by sjejejej10](https://sketchfab.com/3d-models/wooper-rigged-and-unrigged-9f8a50a877234fdb958d4b0ee49a53d3)
is listed under CC Attribution and describes a rigged and an unrigged copy.
The official download endpoint returned HTTP 401 on October 10, 2026, requiring
authentication. No asset was downloaded or imported. Original authorship,
rig contents, textures and suitability for the game remain unverified.
An available authenticated download still needs source and license review,
attribution, a supported rig import and a comparison in the production renderer
before adoption. A search result is not proof of a usable runtime rig.

The full objective still covers every Pokémon model. This two-species pass
does not establish final visual quality or rig coverage for all 251 species.
