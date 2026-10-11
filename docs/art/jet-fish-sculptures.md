# Remoraid and Octillery sculptures

`tools/jet_fish_sculpt.py` authors original static geometry for Remoraid and Octillery. The main battle builder dispatches these two species to the recipe and retains the existing Goldeen, Seaking and Magikarp recipes. Only the two canonical mesh JSON assets are regenerated. The prior normalized height of 1.00 remains unchanged.

Remoraid has a continuous tapered mint body, swept upper and scalloped lower tail lobes, a forward dorsal crest, lower pelvic fin, pale pectoral paddles, paired fitted lateral bars and large ivory eyes. The caudal lobes sit behind the body rather than crossing its face. Octillery has a continuous mantle linking its rounded head to a narrow neck and flared arm base. Eight individually curved solid arms carry sixteen raised annular yellow sucker rims and recessed gold wells. Its three crown spots follow the actual head surface. Heavy red eyelids and a hollow upward siphon replace the previous round stare and capped mouth.

Official [Remoraid artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/223.png) and [Octillery artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/224.png) were inspected as temporary design references. No reference image, downloaded mesh, texture or extracted game model is shipped. Searches found print-oriented assets and game extractions, but no compatible original open rig was verified or imported. The canonical assets remain static; skeletal animation is not claimed.

Verification commands:

```sh
python3 -m unittest discover -s tools -p test_jet_fish_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo build --locked -p crystal-bevy --example immersive_battle_3d --features location-tester
```

Eight geometry tests check closed connected outward-oriented primitives, shading against stored triangles, regeneration and height/triangle bounds, fish proportions and tail placement, eight arms with actual mantle contact, fitted facial and lateral details, sucker support on actual arm surfaces, and an open siphon lip with real depth. Software front, oblique and rear views inspect stored geometry. Native battle captures use the ignored local `content-packs/realtime-clock.browser.crystalpack` and verify static appearance and camera fit. They do not establish controller or gameplay behavior.

Verified this pass: eight geometry regressions, canonical layout, validation of 571 raw JSON model documents, the full-catalog Rust grounded/volumetric regression, and native battle captures for both species passed. Remoraid contains 7,724 triangles (506,355 bytes); Octillery contains 21,740 triangles (1,443,244 bytes). Temporary captures and reference images were removed after inspection.

This is an incremental pass within the full 251-species goal; the complete catalog still requires visual review and improvements.
