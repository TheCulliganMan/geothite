# Pineco and Forretress armor sculptures

`tools/armored_bug_sculpt.py` authors these two canonical static battle meshes directly. The Blender catalog entry point dispatches their names to the standard-library recipe. Other species are not regenerated. Neither mesh claims skeletal rigging or authored animation.

Pineco replaces scattered shards with a tapered dark pine-cone core and three rings of overlapping radial plates. Every plate has a teal upper armor layer and navy underside sharing the same seam vertices. Five tall crown plates surround a tapered stem spike; the two front plates frame the fitted ivory eyes and narrow red irises. The crown was raised after the first visual check to expose the eyes more clearly.

Forretress replaces the plain pink body and gray pipes with a continuous red core inside two pale rocky shell hemispheres. Both hemispheres have scalloped equatorial edges, thickness, inner surfaces and closed poles. Their lower rims overlap the red core. A dark seam circles the full equator. Large dark eye sockets hold ivory annular rims, with open centers exposing the dark socket underneath. Four tapered red ports are actual annular solids with recessed dark interiors. Small fitted stone chips appear on the front and rear of both shell halves.

The official [Pineco artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/204.png) and [Forretress artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/205.png) were inspected for proportions, armor structure and colors. Temporary reference images are not shipped. Searches found printable models and a [Pineco model listing](https://sketchfab.com/3d-models/pineco-f2cbcd344ec14545897dabb85f6b8b20), but did not establish an available original rigged model suitable for import. No external or extracted game model was adopted.

## Verification

```sh
python3 -m unittest discover -s tools -p test_armored_bug_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo build --locked -p crystal-bevy --example immersive_battle_3d --features location-tester
```

Eight independent stored-geometry regressions check closed connected oriented solids, normals, deterministic regeneration and preserved heights, distinct anatomy and triangle budgets, Pineco's shared armor seams and rooted plates, Forretress shell/core/belt/port connections, recessed bore support, closed shell poles and open equator, genuine port and eye-ring apertures, and fitted eyes and stone chips on the actual stored surfaces.

Pineco has 54 material parts and 3,900 triangles; Forretress has 48 parts and 14,524 triangles. Both preserve the previous catalog height of 0.93 and stay below the 15,000-triangle regression budget. Front, oblique and rear software views and native battle presentations are inspected for silhouette, colors, depth rendering, grounding and camera fit. The native renderer uses the existing ignored `content-packs/realtime-clock.browser.crystalpack`; screenshots are presentation evidence, not gameplay or TUI/MCP verification.

Review renders and reference images are temporary and are removed after verification. No content pack, exported game source, command dump, PCM or build product is delivered. The full 251-species visual review remains active; bounds tests do not establish visual quality for every model.
