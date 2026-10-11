# Hoppip, Skiploom and Jumpluff sculptures

`tools/wind_plant_sculpt.py` replaces the shared arms, eye whites and crown
approximations with distinct original botanical anatomy. Hoppip has two closed
serrated crown blades with tapered veins, pointed pink ears with fitted dark
interiors, yellow eyes, a wide smile and a curved tail. Skiploom has smooth
folded leaf ears, red eyes, a gentle smile and eight radial spoon petals around
a pale central disk. Jumpluff has red eyes, a small open smile, crown bracts
and three spherical seed heads connected by visible stems. Each seed head is
a continuous sphere with short sparse bristles, replacing clusters of
intersecting cotton balls. All retain small front flippers and rounded feet.

Anatomy was checked against the official artwork for
[Hoppip](https://assets.pokemon.com/assets/cms2/img/pokedex/full/187.png),
[Skiploom](https://assets.pokemon.com/assets/cms2/img/pokedex/full/188.png) and
[Jumpluff](https://assets.pokemon.com/assets/cms2/img/pokedex/full/189.png).
Those images were temporary visual references in ignored scratch output;
they are not imported into the models or shipped bundle.

Each species preserves its previous normalized height of 0.85, one canonical
static JSON, and the pack-driven runtime species scale. The standard authoring
entry point dispatches to the same recipe. Geometry helpers provide closed
leaf membranes and fitted facial patches without consuming source sprites or
pack content. This pass adds no downloaded mesh, rig, animation or duplicate
model catalog.

```sh
python3 tools/wind_plant_sculpt.py --output crates/crystal-voxel-view/models/battle_species
python3 tools/test_wind_plant_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo run --locked -p crystal-bevy --example immersive_battle_3d --features location-tester -- content-packs/realtime-clock.browser.crystalpack --enemy-gust --species hoppip --size 1000x700 --screenshot target/hoppip-review.png
# Repeat the native fixture with --species skiploom and --species jumpluff.
```

Six stored-model checks cover exact regeneration, physical dimensions and
budgets, closed connected oriented surfaces, shading normals, species anatomy
and eye palettes, fitted facial details and ear insets, actual foot/body
overlap and actual seed-stem/globe overlap. Contact tests intersect real stored
triangles within the shared cross-section instead of assuming that a sloping
stem crosses a globe's center line. Hoppip uses 5,740 triangles and 300,006 raw
JSON bytes; Skiploom uses 10,028 triangles and 427,020 bytes; Jumpluff uses
10,380 triangles and 479,515 bytes.

Before/after oblique and new rear software views were inspected. All three
were inspected in native battle fixtures with the ignored local pack above.
The crown silhouettes fit the camera, Skiploom's petals and leaf ears remain
clear, and Jumpluff's three seed heads remain distinct. Hoppip retains its very
small runtime species scale. All six model checks, asset layout and storage
validation, and the Rust species-loader regression passed. Native captures
verify static presentation only, not animation or gameplay correctness.

This family is an increment toward the full 251-species visual objective;
final quality and rig coverage for the entire catalog remain open.
