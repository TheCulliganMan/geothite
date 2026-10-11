# Girafarig and Stantler sculptures

`tools/hoofed_sculpt.py` authors these two canonical static battle meshes directly. The Blender catalog entry point dispatches Girafarig and Stantler to this recipe, preserving Tauros and other species. Neither mesh claims a skeletal rig or authored animation.

Girafarig has long supporting legs, a sculpted neck and tapered face, a pink muzzle and ear interiors, ivory ossicones, an open smile, and seven triangular pink mane plates. Its golden forequarters and brown hindquarters share exactly the same seam ring. Brown shoulder and neck spots and gold hindquarter markings follow the stored side surfaces. A slender brown tail rises into a second head with gold eyes and an ivory grin facing toward the rear.

Stantler has a lean tan trunk, a shaped upright neck, stern eyes and brows, a broad brown nose, pale forehead and flank markings, a cream chest patch and a large soft tan tail. Its pale antlers use closed periodic tube loops with three rounded branches per side. Dark violet orbs contact the lower loop, and the loop centers remain hollow. Both species have dark grounded cloven hooves with fitted clefts.

The official [Girafarig artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/203.png) and [Stantler artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/234.png) were inspected for proportions, colors and species features. Reference images are temporary and are not shipped. Searches found a [Cobblemon Girafarig listing](https://sketchfab.com/3d-models/girafarig-line-cobblemon-25423ef2ebd043749e7421751a6a86a8) and a Stantler viewer, but did not establish an available original rigged model suitable for import. No external or extracted game geometry was adopted.

## Verification

```sh
python3 -m unittest discover -s tools -p test_hoofed_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo build --locked -p crystal-bevy --example immersive_battle_3d --features location-tester
```

Nine independent stored-geometry tests cover closed connected oriented parts, normals, deterministic regeneration and preserved height, distinct anatomy and triangle budgets, grounded hooves and real leg/body/head/neck/muzzle contacts, shared body-color seam vertices, mane and tail support, ear/horn/antler/branch/orb support, hollow antler centers, fitted side markings and fitted front and rear faces. A winding error in the first mane prisms was caught by the topology checks and corrected in the authoring recipe.

Girafarig has 64 material parts and 20,512 triangles; Stantler has 51 parts and 20,548 triangles. Both preserve their previous catalog height of 1.28 and stay below 23,000 triangles. Oblique and rear software views and native battle presentation are inspected for silhouette, colors, depth rendering, grounding and camera fit. Native presentation uses the existing ignored `content-packs/realtime-clock.browser.crystalpack`; screenshots are not gameplay or TUI/MCP evidence.

Review renders and reference images are removed after verification. No content pack, exported game source, command dump, PCM or build product is delivered. The full 251-species visual review remains active; catalog bounds checks do not establish visual quality for every species.
