# Swinub and Piloswine sculptures

`tools/fur_pig_sculpt.py` replaces the shared brown sphere with two original
fur silhouettes. Swinub has a low elongated coat, four fitted tapered dorsal
wedges, sleepy curved eyes, a projecting pink snout and small rear wisps.
Piloswine has a tall humped mantle with a scalloped hem, long drooping ears,
hidden eyes, thin fitted front fur creases and two ivory tusks curving down
and forward. Both have fitted tilted nostrils, four supporting shanks and
grounded dark oval hooves. Swinub's lowered coat covers most of the supports
instead of leaving long exposed legs.

The anatomy was checked against the official
[Swinub artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/220.png)
and [Piloswine artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/221.png).
Those images were temporary visual references in ignored scratch output;
they are not imported into the models or shipped bundle.

Each species retains its previous normalized height of 0.98, one canonical
static JSON and the pack-driven runtime species scale. The standard authoring
entry point dispatches to the same recipe. Dorsal markings follow actual
stored coat triangles. Piloswine's continuous mantle supplies the fringe
silhouette, replacing separate orange blade primitives. This pass adds no
downloaded mesh, rig, animation, duplicate catalog or pack content.

```sh
python3 tools/fur_pig_sculpt.py --output crates/crystal-voxel-view/models/battle_species
python3 tools/test_fur_pig_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo run --locked -p crystal-bevy --example immersive_battle_3d --features location-tester -- content-packs/realtime-clock.browser.crystalpack --enemy-gust --species swinub --size 1000x700 --screenshot target/swinub-review.png
# Repeat the native fixture with --species piloswine.
```

Six stored-model checks cover exact regeneration, dimensions and budgets,
closed connected oriented surfaces, shading normals, distinct coat proportions
and anatomy, the downward/forward tusk direction, fitted stripes, eyes,
nostrils and fur creases, grounded hooves and actual volumetric overlap between
hoof, shank and body. Swinub uses 8,496 triangles and 526,858 raw JSON bytes;
Piloswine uses 9,432 triangles and 416,459 bytes. Before/after oblique views and
a new rear view were inspected with the software rasterizer. Native fixtures
verify static appearance and camera fit only, not animation or gameplay
correctness. Both final models were inspected using the ignored local pack
above. Swinub's low coat, stripes and sleepy face remain visible; Piloswine's
ears, snout and curved tusks fit the camera. Piloswine's crown was broadened
after the first native review to remove an overly pointed contour, then
rebuilt and recaptured. Hooves contact the arena. All six model checks, asset
layout and storage validation, and the Rust species-loader regression passed.

This family is an increment toward the full 251-species visual objective;
final quality and rig coverage for the entire catalog remain open.
