# Slugma and Magcargo sculptures

`tools/lava_sculpt.py` replaces the shared oval base and separate sphere neck
with a continuous longitudinal mantle, raised face and tapered tail. The low
body has gently scalloped edges and a flattened underside. Rounded eye stalks
overlap the neck, yellow oval eyes have slim black pupils, and the smile and
molten cheek droplets follow the stored skin. Slugma's two sinuous folds are
fitted to its upper mantle. Magcargo carries a broad stone shell with a full
raised spiral on both sides, small molten seams and a hollow chimney with a
dark inner wall and recessed orange floor.

Both preserve the previous normalized height of 0.98. The runtime continues
to apply the species scale from the external content pack. There is one
canonical static JSON per species, also dispatched by the standard authoring
entry point. This change adds no imported models, pack content, duplicate
catalog, rig or animation.

```sh
python3 tools/lava_sculpt.py --output crates/crystal-voxel-view/models/battle_species
python3 tools/test_lava_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo run --locked -p crystal-bevy --example immersive_battle_3d --features location-tester -- content-packs/realtime-clock.browser.crystalpack --enemy-gust --species slugma --size 1000x700 --screenshot target/slugma-review.png
# Repeat the native fixture with --species magcargo.
```

Six stored-model checks cover exact regeneration, physical dimensions and
budgets, closed connected oriented surfaces, shading normals, distinct shell
anatomy, an uncapped chimney opening, fitted details and real volumetric
contact between the body, eye stalks and shell. The upper-skin contact check
guards against a floating mantle fold found in the initial oblique review.
Slugma uses 4,108 triangles and 183,905 raw JSON bytes; Magcargo uses 10,276
triangles and 453,452 bytes. The extra shell budget supplies two complete
spirals and an annular chimney instead of an opaque cap.

Before/after oblique views and new rear views were inspected with the software
rasterizer. Native battle fixtures check static presentation and camera fit;
they do not establish animation or gameplay correctness. Both final models
were inspected in native fixtures using the ignored local pack above: the
eyes and smile remain readable, the shell spiral and chimney opening are
visible, and both bodies contact the arena within the existing species scale.
All six model checks, canonical layout and storage validation, and the Rust
species-loader regression passed. The complete
251-species visual goal remains open beyond this family.
