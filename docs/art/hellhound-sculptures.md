# Houndour and Houndoom sculptures

`tools/hellhound_sculpt.py` authors original static canine geometry for the two species. The battle builder dispatches to this recipe, preserving the existing normalized height of 1.20 for each. Only their canonical mesh JSON assets are regenerated; Growlithe and Arcanine retain their existing generator and assets.

Both have long canine trunks, upright contoured necks, shaped brows, projecting orange snouts and fitted angled eyes. Four curved supports connect grounded paw pads to the body, with toe grooves fitted to the actual paw fronts. Broad ivory cuffs follow the authored leg centerlines. Bone back bands conform to the stored trunk surface rather than floating above it.

Houndour has two pointed ears, a fitted forehead bone shield with dark inset, two back bands, four single ankle cuffs, small canines and a short tapered tail. Houndoom has two broad hooked ivory horns, three back bands, eight ankle cuffs, twelve pale toe claws, an orange lower jaw and recessed oral cavity with tongue and upper canines. Its ivory neck collar supports a small skull pendant with fitted dark eye sockets. A long thin looping tail ends in a separate attached spade.

Official [Houndour artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/228.png) and [Houndoom artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/229.png) were inspected as temporary design references and are not shipped. External searches found printing-oriented listings, but no compatible original open rig was verified or imported. These are static authored meshes; skeletal animation is not claimed.

Verification commands:

```sh
python3 -m unittest discover -s tools -p test_hellhound_sculpt.py
python3 tools/check-model-layout.py
python3 tools/model_asset_storage.py validate crates/crystal-voxel-view/models
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
cargo build --locked -p crystal-bevy --example immersive_battle_3d --features location-tester
```

Eight geometry checks verify closed connected outward-oriented parts, normals agreeing with stored triangle winding, deterministic regeneration and height/triangle bounds, anatomy counts, real paw/leg/body support, fitted armor/eye/marking surfaces, cuff/claw/spade contacts, muzzle/nose/collar/pendant contact, and rooted head/neck/ears/horns/tails. The first seven tests passed together and the added muzzle/collar contact regression passed separately against the same final meshes. Canonical layout, validation of 571 raw JSON model documents and the Rust full-catalog grounded/volumetric regression passed.

Front, rear and oblique software reviews inspect stored geometry. Both final native battle captures were inspected using the existing ignored `content-packs/realtime-clock.browser.crystalpack`, confirming static appearance and camera fit. This is not gameplay/controller verification. Houndour contains 14,248 triangles (1,166,860 bytes); Houndoom contains 22,736 triangles (1,696,161 bytes). Temporary artwork and captures were removed after inspection.

This is incremental progress within the ongoing 251-species visual goal; full-catalog completion remains unproven.
