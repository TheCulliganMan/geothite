# Mareep and Flaaffy sculptures

`tools/sheep_sculpt.py` replaces the shared wool-ball collar with distinct
original electric sheep anatomy. Mareep has a full cream fleece, four short
tapered blue legs, a joined blue cranium and muzzle, a single curled forelock,
gold ear bands and an orange tail beacon. Flaaffy has an exposed pear-shaped
pink belly, two supporting legs, pink foot pads with fitted ivory toes, tapered
arms, a white shoulder cape, paired crown curls, pink ear bands and a blue
tail beacon. Both have fitted eyes and nostrils and a curved striped tail.

The forms were checked against the official
[Mareep artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/179.png)
and [Flaaffy artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/180.png).
Those images served only as visual references in ignored scratch output and
are not imported into the models or shipped bundle.

Each fleece is one closed surface. Its radial envelope blends rounded wool
contours into a common core instead of intersecting detached sphere primitives.
The crown grooves follow actual stored tuft triangles. Ear and tail bands
share transported rings, using the geometry helper from the spider recipe.
The previous normalized heights remain 0.85 for Mareep and 1.0 for Flaaffy;
runtime species scaling remains driven by the external content pack. Each has
one canonical static JSON, also dispatched by the standard authoring entry
point. Ampharos's existing model is preserved. This pass adds no downloaded
mesh, duplicate catalog, rig or animation.

```sh
python3 tools/sheep_sculpt.py --output crates/crystal-voxel-view/models/battle_species
python3 tools/test_sheep_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo run --locked -p crystal-bevy --example immersive_battle_3d --features location-tester -- content-packs/realtime-clock.browser.crystalpack --enemy-gust --species mareep --size 1000x700 --screenshot target/mareep-review.png
# Repeat the native fixture with --species flaaffy.
```

Six stored-model checks cover exact regeneration, dimensions and budgets,
closed connected oriented surfaces, shading normals, distinct fleece/limb/curl
anatomy, grounded supports, fitted crown grooves, nostrils and toes, identical
ear/tail joint rings and actual volumetric overlap between legs and body. The
contact check caught the shallow Mareep foreleg attachment; roots now extend
into the fleece. Dense toe-outline sampling fixes clipping against Flaaffy's
curved feet. Mareep uses 11,560 triangles and 519,321 raw JSON bytes; Flaaffy
uses 16,356 triangles and 811,902 bytes. The larger fleece sampling budget
retains rounded outlines and avoids pointed initial wool contours.

Software before/after and rear inspection checks the neutral geometry. Native fixtures
establish static appearance and camera fit only, not animation or gameplay
correctness. Both final models were inspected with the ignored local pack
above. The distinct fleece silhouettes, crown grooves, ivory toes and colored
tail beacons remain visible; supports contact the arena and tails fit the
camera. All six model checks, canonical layout and storage validation, and the
Rust species-loader regression passed. This family is an increment toward the full 251-species visual
objective; final quality and rig coverage for the entire catalog remain open.
