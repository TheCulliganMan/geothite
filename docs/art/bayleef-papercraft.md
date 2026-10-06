# Bayleef papercraft sculpture and animation

Bayleef replaces its separate spherical body pieces, floating face, diamond
crown and flat collar with a connected long-neck quadruped, fitted facial
panels, a notched folded canopy and curled collar buds. The tapered tail and
four grounded legs preserve a distinct silhouette from Chikorita.

The only canonical model is `models/battle_species/bayleef.glb` in
`crates/crystal-voxel-view`: 33 anatomy parts, 3,836 triangles, 9,626 split-normal
vertices and a 16-joint standard glTF skin. The old same-species JSON is removed.
No additional Blender archive, STL or neutral mesh copy is generated in Git.
The existing multi-species archive remains historical authoring material.

`tools/bayleef_sculpt.py` authors the panels; `tools/bayleef_glb.py` writes the
skin, materials and three clips using Python's standard library. Regenerate
and validate without Blender or a content pack:

```sh
python3 tools/bayleef_glb.py
python3 tools/test_bayleef_glb.py
python3 tools/check-battle-species.py
python3 tools/check-johto-models.py
```

Pelvis, chest, neck and head form a dedicated chain. Four leg joints preserve
planted soles; separate collar, tail and crown joints settle behind body
motion. Idle is periodic; attack and hit clips use the existing presentation
cue progress. These are authored body motions, while the original move
programs continue to own projectiles, flashes, sounds, timing and damage.

Full neutral authoring height remains 1.15 units, with minimum Y exactly zero.
The runtime uses the original pack's height digits 311, or 3 feet 11 inches,
to preserve 1.1938 meters. Animated bounds affect visibility without changing
physical scale. Encounter positions and camera policy are unchanged.

The stored-byte tests cover deterministic export, closed connected parts,
normal winding, notched anatomy, fitted facial clearance under animation,
normalized weights, four fixed soles/twelve claws, loop seams, blends, strain
and skin quantization. Runtime tests additionally check shared GPU resources,
actual skin binding, dense animation bounds and canonical world-space size.

The disposable native preview accepts `--enemy-gust --starter bayleef`.
Its natural level-25 learnset responds with Razor Leaf to the original trainer
AI's Gust, through the production controller with real PP/HP updates and no
save destination. The visual review is of actual stored geometry and clips,
not a claim that all 251 species have completed this art/animation pass.
