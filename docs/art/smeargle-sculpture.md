# Smeargle painter-dog sculpture

`tools/smeargle_sculpt.py` authors the canonical static Smeargle mesh directly with the existing standard-library geometry helpers. The Blender catalog builder dispatches Smeargle to this recipe. Other species are not regenerated.

The previous generic dog had tiny dark eyes, brown paws and tail, a cylindrical cap tip, and a smooth green oval. The replacement has a continuous cream torso and projecting canine muzzle, large ivory eyes in brown sockets, short floppy brown ears, a soft slanted cream beret with a bent crown point, cream three-toed paws with fitted grooves, brown ankle markings, and two brown bands on each forelimb. One hand holds a cream tail that sweeps across the front of the body and ends in a tapered brush. Cream bristles and green paint meet at the same scalloped ring, with no floating pigment shell. The green tip has a shaped, subtly uneven contour.

The official [Smeargle artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/235.png) was inspected for proportions, colors, features and pose. Reference images are temporary verification material and are not shipped.

## External model research

Searches found a [Cobblemon Smeargle listing](https://sketchfab.com/3d-models/smeargle-cobblemon-2e3990b6ca7942728fd47d86ab2e575b), a Pokémon X/Y extraction, and a [Free3D model listed as unrigged](https://free3d.com/3d-model/smeargle-pokemon-4404.html). This pass did not verify an available original rigged Smeargle suitable for import. No external model or game extraction was imported. This original JSON mesh has no skeletal rig or authored animation.

## Verification

```sh
python3 -m unittest discover -s tools -p test_smeargle_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo build --locked -p crystal-bevy --example immersive_battle_3d --features location-tester
```

Eight independent stored-geometry tests cover closed oriented connected parts, triangle normals, deterministic regeneration and preserved height, anatomy and colors, actual foot/leg/body support, head/ear/hat/tongue connections, tail/body/hand/brush contact, the shared scalloped brush boundary, shared arm-band rings and finger contacts, and fitted eyes, nostrils and paw grooves.

The asset has 65 material parts and 20,276 triangles, below the 22,000-triangle regression budget. Its catalog height remains 1.14. Front/oblique and rear software renders and a native battle screenshot are inspected for silhouette, colors, shading, grounding and camera fit. The existing ignored `content-packs/realtime-clock.browser.crystalpack` is used only for native presentation verification. Screenshots do not establish gameplay or TUI/MCP correctness.

Temporary renders and the reference image are removed after verification. No content packs, exported game source, command dumps, PCM or build products are delivered. The full 251-species visual review remains active; catalog bounds tests alone cannot prove every species looks good.
