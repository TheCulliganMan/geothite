# Sneasel sculpture

`tools/sneasel_sculpt.py` authors original static Sneasel geometry. The battle builder dispatches to the recipe and writes the established `sneasel.mesh.json`; its existing normalized height of 1.11 is unchanged. Other species are not regenerated.

A slim continuous teal torso supports a tapered muzzle and broad feline crown. Surface-fitted slanted dark sockets, ivory eye whites, narrow red irises and vertical pupils replace protruding round eyes. The forehead and chest carry fitted oval gold gems. A dark fitted grin contains a pink tongue and two small ivory fangs. One pointed teal ear and a long swept rose plume give the correct asymmetry. Three curved rose tail feathers connect through a rooted stem. Reaching forelimbs support broad palms and two hooked ivory talons each; outward-curving hocks support grounded teal feet with two curved ivory toe claws each.

[Official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/215.png) was inspected temporarily and is not shipped. A search located [Sneasel by nguyenlouis32](https://sketchfab.com/3d-models/sneasel-8e675f32390a44d190191e38f1c85dc4), but the page fetch returned HTTP 403 and the official download endpoint returned HTTP 401. Its authorship, rig and suitability were not verified; no external mesh or texture was imported. This canonical mesh is static and does not claim skeletal animation.

Verification commands:

```sh
python3 -m unittest discover -s tools -p test_sneasel_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo build --locked -p crystal-bevy --example immersive_battle_3d --features location-tester
```

Seven geometry checks inspect closed connected outward-oriented parts, normals against stored winding, deterministic regeneration and bounds, anatomy counts, rooted head/ears/forelimbs, all three tail feather contacts, foot/leg/claw support, and actual surface contact for facial details and gems. Front, rear and oblique software views inspect stored geometry. Native appearance and camera fit use the existing ignored `content-packs/realtime-clock.browser.crystalpack` with the immersive battle example. These are static model checks, not gameplay/controller verification.

Verified this pass: seven geometry checks, canonical layout, validation of 571 raw JSON model documents, the Rust full-catalog grounded/volumetric regression and native battle capture passed. The asset contains 9,936 triangles and occupies 644,321 bytes. Temporary references, download response and review captures were removed after inspection.

This is an incremental species pass within the ongoing 251-species goal. Full-catalog visual completion remains unproven.
