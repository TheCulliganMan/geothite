# Delibird sculpture

`tools/delibird_sculpt.py` replaces Delibird's generic bird recipe with original static geometry. The canonical asset remains `delibird.mesh.json`; the battle builder dispatches to the same recipe. Its prior normalized height of 1.10 is preserved. Other bird assets are not regenerated.

A rounded red pear torso supports a broad white head, paired three-point feather fans, large dark eye surrounds and a rounded two-part yellow bill. A subdivided concave white collar follows the actual torso/head surfaces and ends in pointed breast feathers above a single lower button. The two flippers have distinct raised/lowered poses. Short hidden legs support grounded three-toed webs. The sack-shaped tail has a sagging white bowl, curved neck held under the lowered flipper, and rear folds fitted to the bag surface.

[Official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/225.png) was inspected as a temporary design reference and is not shipped. No external rig was verified or imported for this species. The authored mesh is static; no skeletal animation is claimed.

Verification commands:

```sh
python3 -m unittest discover -s tools -p test_delibird_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo build --locked -p crystal-bevy --example immersive_battle_3d --features location-tester
```

Six geometry regressions check closed connected outward-oriented parts, finite unit normals agreeing with stored triangle winding, deterministic regeneration and bounds, fitted collar and bag folds, actual foot/leg/body volume contacts, asymmetric wings, actual flipper/torso volume contact and crest/head contact. Front, rear and oblique software views inspect stored geometry. Native appearance and camera fit are reviewed using the ignored existing `content-packs/realtime-clock.browser.crystalpack`; this is not a gameplay or controller test.

Verified this pass: six geometry tests, canonical model layout, validation of 571 raw JSON model documents, the Rust full-catalog grounded/volumetric regression, and the native battle capture passed. The asset contains 17,808 triangles and occupies 808,314 bytes. Temporary captures and artwork references were removed after inspection.

This is incremental progress toward the 251-species goal; full-catalog visual completion remains unproven.
