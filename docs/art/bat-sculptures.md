# Zubat, Golbat and Crobat sculptures

`tools/bat_sculpt.py` replaces the shared triangular-wing and sphere-face
recipe with original static anatomy. The upper wings have curved scalloped
edges, thin closed membranes, tapered leading bones and branching spars.
Zubat has an eyeless open muzzle, four inward fangs, folded ears and slender
hanging legs. Golbat has a thick elliptical mouth frame, a dark recessed
throat, fitted ivory eyes and four long canines. Crobat has a broad violet
cranium, narrow fitted yellow eyes, an angular mouth and a second swept wing
pair. Each species retains its previous normalized height of 1.0; runtime
species scaling remains driven by the external pack.

There is one canonical JSON mesh per species. The standard battle-species
authoring entry point dispatches to the same recipe. No pack content,
downloaded assets, duplicate model catalogs, skinning or animation are added.

```sh
python3 tools/bat_sculpt.py --output crates/crystal-voxel-view/models/battle_species
python3 tools/test_bat_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo run --locked -p crystal-bevy --example immersive_battle_3d --features location-tester -- content-packs/realtime-clock.browser.crystalpack --enemy-gust --species zubat --size 1000x700 --screenshot target/zubat-review.png
# Repeat the native fixture with --species golbat and --species crobat.
```

The six model checks cover exact regeneration, dimensions and budgets, closed
connected oriented surfaces, shading normals, species anatomy, genuine open
mouths and eye contact with actual stored face triangles. Edge sampling on the
eye patches prevents the original corner-only outlines from disappearing into
the curved face. Golbat’s ivory wedges are confined to the front lip;
a depth-range regression catches patches crossing back over the rolled rim.
Zubat uses 3,352 triangles, Golbat 4,784 and Crobat 4,116;
each is below 310,000 raw JSON bytes. Front, oblique and rear software views
were inspected. Native fixtures establish static visual presentation only.
All three were inspected in native battle fixtures with the ignored local pack
above. The wings fit the camera, Crobat retains four visible wings and readable
yellow eyes, and Zubat keeps its smaller species scale. Golbat's corrected
front-lip eyes were recaptured after rebuilding. All six model checks, asset
layout and storage validation, and the Rust species-loader regression passed.

This family is an increment toward the full 251-species visual objective;
it does not establish final quality or rig coverage for every species.
