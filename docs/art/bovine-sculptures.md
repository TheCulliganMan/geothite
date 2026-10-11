# Miltank and Tauros sculptures

These original, reproducible sculptures replace the generic cow and bull in the canonical battle-species catalog. `tools/bovine_sculpt.py` uses the existing stored triangle-mesh schema; the Blender authoring entry point dispatches these two species to that recipe. Girafarig and Stantler retain their existing assets.

Miltank has a plump pink body, dark cow hood and broad ears, cream ear insets, short ivory horns, blue eyes, raised forelimbs with dark hoof hands, grounded hind hooves, a broad cream udder with four attached pink teats, fitted rear spots, and a cream whip ending in a dark ball. The face mask has sufficient shell depth to remain visible over the curved head in the native renderer.

Tauros has a heavy golden quadruped trunk, a continuous gray shoulder mane with a scalloped lower edge, a tapered lowered forehead, swept gray horns, three fitted forehead studs, narrow eyes, gray muzzle, four curved supporting legs with cloven blue-gray hooves, and three individually curved brown tails with pointed blue-gray tufts.

Reference images were inspected locally from the official [Miltank artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/241.png) and [Tauros artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/128.png). Those images are not shipped.

## External model research

[Eleanie's Tauros](https://sketchfab.com/3d-models/tauros-de524ca243b642149ecca49d1a840d2d) is listed as an original fully rigged model under CC Attribution. The official Sketchfab download endpoint returned HTTP 401 without an authenticated account during this pass. No viewer geometry was extracted and no external model was imported. These canonical JSON sculptures are static meshes; they do not claim skeletal rigs or animation support.

## Verification

```sh
python3 -m unittest discover -s tools -p test_bovine_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo build --locked -p crystal-bevy --example immersive_battle_3d --features location-tester
```

Eight independent stored-geometry tests cover closed oriented connected parts, normals, reproducible assets and preserved height, species-specific anatomy counts, actual udder/teat contact, grounded hoof/leg and arm support, head/mane/horn/tail connections, ears and muzzles, rear markings, and fitted eyes and forehead studs. Both meshes stay below 21,000 triangles: Miltank has 17,976 and Tauros 16,504. Their previous catalog heights remain 1.12 and 1.28 respectively.

Front/oblique and rear software views were inspected, followed by native battle screenshots using the existing ignored `content-packs/realtime-clock.browser.crystalpack`. Native checks assess the stored art, depth rendering, grounding, silhouette and camera fit; they are not gameplay or TUI/MCP regressions. Scratch screenshots and downloaded reference images are removed after verification. No content pack, PCM, exported game commands, or build products are delivered.

The full 251-species visual review remains in progress. Catalog bounds and normals checks do not establish visual quality for every species.
