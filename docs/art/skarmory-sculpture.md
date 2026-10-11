# Skarmory sculpture

`tools/skarmory_sculpt.py` authors original static geometry for Skarmory, replacing the sphere/rod torso, round head and disconnected flat wing primitives. The established battle builder dispatches to this recipe and writes the canonical `skarmory.mesh.json`. Its existing normalized height of 1.21 remains unchanged; no other species asset is regenerated.

The breast, shoulder and rising neck form a continuous shell. A joined angular skull extends into a sharp upper bill, with a separate lower mandible and dark mouth seam. Surface-fitted yellow eyes sit inside dark angular sockets. Four fitted neck seams define overlapping armor. An upright swept crest and four rear nape plumes join the skull. Each wing has a swept pale steel leading edge and four layered red blade solids with broad roots embedded in the shoulder. Silver haunches support bent darker hocks and three forward plus one rear hooked talon on each grounded foot. The long hooked tail blade connects to the rear body.

The palette and sharp surface planes suggest steel; the recipe does not add reflective PBR materials or a skeletal rig. The existing runtime mesh-color rendering path is retained.

[Official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/227.png) was inspected as a temporary design reference. [Skarmory (Animated) by nguyenlouis32](https://sketchfab.com/3d-models/skarmory-animated-f6bd32a3716848d8895b5dd562be8c79) advertises animation and CC Attribution, but its authorship and rig suitability were not verified. The official download API returned HTTP 401 without authentication. Nothing from that listing was imported, and no external mesh or reference image is shipped.

Verification commands:

```sh
python3 -m unittest discover -s tools -p test_skarmory_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo build --locked -p crystal-bevy --example immersive_battle_3d --features location-tester
```

Seven geometry checks inspect closed connected outward-oriented solids, normals against stored triangles, deterministic regeneration and bounds, blade counts, real wing/head/tail body contacts, crest/plume skull contact, fitted eye/neck surfaces and actual haunch/hock/talon support. Software front, rear and oblique reviews inspect stored geometry. A native battle capture uses the existing ignored `content-packs/realtime-clock.browser.crystalpack` to inspect static appearance and camera fit. This does not establish gameplay or controller behavior.

Verified this pass: seven geometry checks, canonical model layout, validation of 571 raw JSON model documents, the Rust full-catalog grounded/volumetric regression, and a native battle capture passed. The canonical asset contains 4,288 triangles and occupies 260,728 bytes. Temporary artwork, download response and review captures were removed after inspection.

This is an incremental species pass; the full 251-species visual goal remains active and unproven.
