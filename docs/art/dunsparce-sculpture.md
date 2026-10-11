# Dunsparce land-snake sculpture

`tools/dunsparce_sculpt.py` authors the canonical Dunsparce mesh directly. The catalog authoring entry point dispatches only this species to the recipe. The mesh is an original static sculpture, with no imported rig or authored animation.

The old model used overlapping body and head ellipsoids, tiny line eyes, white fangs, angular wing shards, raised blue strips and an ungrooved cone. The replacement has a continuous broad rounded head and tapering trunk, a blue underside sharing its side seams with the yellow body, large blue oval eye rings with pale closed lids and diagonal creases, two pointed blue jaw barbs, two small curved ivory feather wings, fitted blue back ribbons, and a raised yellow tail ending in a three-turn spiral drill.

The body color regions share exact boundary vertices along the full trunk. Back ribbons use a dense grid sampled from the stored body triangles, avoiding the breaks observed with a broad patch fan in the first native render. Pale eyelids have sufficient shell depth to avoid the blue ring showing through at oblique native viewing angles. The drill spiral follows the authored cone profiles; its proximity to actual stored cone triangles is verified independently.

The official [Dunsparce artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/206.png) was inspected for proportions, colors and features. The reference image is temporary and is not shipped. Searches of downloadable models, rigged-model listings and printable assets did not establish an available original rigged Dunsparce suitable for import. No extracted game geometry was adopted.

## Verification

```sh
python3 -m unittest discover -s tools -p test_dunsparce_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo build --locked -p crystal-bevy --example immersive_battle_3d --features location-tester
```

Eight independent stored-geometry tests cover closed oriented connected parts, normals, deterministic regeneration and scale, anatomy counts, exact body/belly seams, grounded belly, wing/jaw/tail/drill contacts, fitted eyes and feather creases, ribbons on the actual roof, and the spiral's surface proximity and coverage across several heights.

The final asset has 19 material parts and 12,248 triangles, below its 14,000-triangle regression budget. The previous catalog height of 0.85 is preserved. Front/oblique and rear software views and native battle presentation are inspected for silhouette, colors, depth rendering, grounding and camera fit. The existing ignored `content-packs/realtime-clock.browser.crystalpack` is used only for native rendering. These screenshots do not establish gameplay or TUI/MCP behavior.

Temporary renders and reference images are removed after verification. No content pack, game export, command dump, PCM or build product is delivered. The full 251-species visual review remains in progress; the Rust catalog bounds regression does not establish visual quality for all species.
