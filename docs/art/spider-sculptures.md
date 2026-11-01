# Spinarak and Ariados sculptures

`tools/spider_sculpt.py` replaces the shared spider recipe with distinct
original anatomy. Spinarak has a broad green head, a raised egg-shaped abdomen,
six walking legs, dark teal leg bands, red hooked mandibles, an upright ivory
horn and a fitted dorsal false face. Ariados has a narrower red head, an
elongated red abdomen, four walking legs plus two raised legs, purple and gold
bands, purple eye sockets, ivory mandibles, a forward horn and broad black
chevron markings. Both have black pupils and small ivory glints.

The anatomy was checked against the official
[Spinarak artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/167.png)
and [Ariados artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/168.png).
Those images were used only for visual reference in ignored scratch output;
they are not imported into the models or shipped bundle.

Every band in a limb shares an identical transported ring with its neighbor.
This fixes the gaps at bends produced by independent tube frames. The head
and abdomen overlap, while dorsal markings follow actual stored carapace
triangles. Dense edge sampling keeps their outlines from disappearing into
the curved skin. Each species retains its previous normalized height of 0.90
and one canonical static JSON, dispatched by the standard authoring entry
point. Runtime species scaling remains driven by the external pack. No rig,
animation, downloaded mesh or duplicate source catalog is added.

```sh
python3 tools/spider_sculpt.py --output crates/crystal-voxel-view/models/battle_species
python3 tools/test_spider_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo run --locked -p crystal-bevy --example immersive_battle_3d --features location-tester -- content-packs/realtime-clock.browser.crystalpack --enemy-gust --species spinarak --size 1000x700 --screenshot target/spinarak-review.png
# Repeat the native fixture with --species ariados.
```

Six stored-model checks cover exact regeneration, dimensions and budgets,
closed connected oriented surfaces, shading normals, six-limb anatomy,
identical shared band rings, planted tips, dorsal marking contact, actual
head/abdomen overlap and distinct horn directions. Spinarak uses 7,844
triangles and 564,299 raw JSON bytes; Ariados uses 6,560 triangles and 448,361
bytes. Before/after oblique views and a rear view were inspected with the
software rasterizer. Native fixtures verify static presentation only, not
animation or gameplay correctness. Both final models were inspected in native
battle fixtures with the ignored pack above. The walking tips contact the
arena, the raised Ariados legs fit the camera, and the dorsal false face and
chevrons remain clear. Outer eye highlights were repositioned after the
initial camera review. All six model checks, asset layout and storage
validation, and the Rust species-loader regression passed.

This family is an increment toward the full 251-species visual objective;
final quality and rig coverage for the entire catalog remain unproven.
