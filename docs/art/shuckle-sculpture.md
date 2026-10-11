# Shuckle perforated-shell sculpture

`tools/shuckle_sculpt.py` authors the canonical static Shuckle mesh with standard-library geometry helpers. The catalog's Blender entry point dispatches only Shuckle to this recipe. No other species is regenerated, and this mesh does not claim a skeletal rig or authored animation.

The previous solid red ellipsoid had decorative rings, straight capped limbs and a narrow vertical neck. The replacement has a continuous hollow red shell with six actual apertures: four limb exits, the neck exit and an empty upper opening. Six curved cream collars follow the shell around those apertures. A recessed dark soft-body core supports the four flexible yellow limbs and curved neck; rounded feet meet the ground. A broad yellow head carries two narrow fitted eyes and a small worried expression. Cream flecks follow the stored shell on the front and rear.

The shell uses oriented cube charts projected onto an ellipsoid. Neighboring chart cells share deterministic perimeter samples, outer and inner skins join only at hole rims, and the result is a closed connected shell solid. Annular collars preserve open centers. This avoids placing opaque decorative caps over a solid sphere.

The official [Shuckle artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/213.png) was inspected for colors, proportions, openings and soft-body features. The temporary reference image is not shipped. Searches found printable assets, a themed Cobblemon model and a [Free3D Shuckle listed as unrigged](https://free3d.com/3d-model/shuckle-pokemon-31766.html), but did not establish an available original rigged model suitable for import. No external model or extracted game geometry was adopted.

## Verification

```sh
python3 -m unittest discover -s tools -p test_shuckle_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo build --locked -p crystal-bevy --example immersive_battle_3d --features location-tester
```

Eight independent stored-geometry tests cover closed oriented connected parts, normals, reproducible geometry and preserved height, anatomy counts and triangle budget, real limb/core/foot and neck/head contact, grounded feet, rays through all six shell apertures and collar centers, cream-rim contact with the shell, fitted front and rear spots, and fitted eyes and mouth.

The asset has 27 material parts and 19,836 triangles, below its 21,000-triangle regression budget. The previous catalog height of 0.91 is preserved. Oblique and rear software views and native battle presentation are inspected for silhouette, colors, depth rendering, grounding and camera fit. The native renderer uses the existing ignored `content-packs/realtime-clock.browser.crystalpack`; screenshots verify presentation only, not gameplay or TUI/MCP behavior.

Review renders and reference images are temporary and are removed after verification. No content pack, exported game source, command dump, PCM or build product is delivered. The full 251-species visual review remains active. The Rust catalog bounds test alone does not establish visual quality for every species.
