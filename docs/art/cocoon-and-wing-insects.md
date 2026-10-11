# Cocoon and wing insect sculptures

Metapod, Kakuna, Butterfree and Beedrill each retain one canonical static JSON
mesh. Their new standard-library recipes are dispatched by the existing
`build-battle-species.py` authoring entry point. No external meshes, game data
or parallel geometry catalog are required.

`tools/cocoon_sculpt.py` gives Metapod a continuous crescent shell with a
projecting cheek, folded lower body and hooked crown. Kakuna has a distinct
tapered golden shell, folded arm plates and fine abdominal creases. Their
angular eye slits, pupils and plates follow the actual stored shell surface;
there are no protruding white eye spheres. Both preserve their previous
0.80-unit physical height and floor datum.

`tools/wing_insect_sculpt.py` gives Butterfree rounded scalloped upper and lower
wings, black rims and branching veins on both sides. Its blue proboscis,
mittens and long foot pads distinguish it from Beedrill's narrow four-wing
silhouette, slender petiole, striped gaster, rear stinger and long forearm
drills. Both have fitted ruby compound lenses and curved antennae. Wings are
curved closed solids, with inset membranes exposed on front and back. Both
retain the previous 1.04-unit height. All four models remain static; this
increment does not add animation or skinning.

```sh
python3 tools/cocoon_sculpt.py --output crates/crystal-voxel-view/models/battle_species
python3 tools/wing_insect_sculpt.py --output crates/crystal-voxel-view/models/battle_species
python3 tools/test_cocoon_sculpt.py
python3 tools/test_wing_insect_sculpt.py
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
# Native GPU inspection; repeat with beedrill, metapod and kakuna.
cargo run --locked -p crystal-bevy --example immersive_battle_3d --features location-tester -- content-packs/realtime-clock.browser.crystalpack --enemy-gust --species butterfree --size 1000x700 --screenshot target/insect-review.png
```

The ten stored-model tests verify recipe equality, dimensions, topology,
oriented edges, closed connected parts, normals and winding, fitted cocoon
details, a genuine bent Metapod contour, wing curvature, membrane visibility
on both faces, bilateral silhouettes and bounded geometry. Cocoon models
have fewer than 1,000 triangles; flying insects have fewer than 10,000.
Software raster inspection compares the previous and new canonical meshes.
The ten tests, model-layout check, source-storage validation and Rust loader
regression passed. Native 1000×700 battle fixtures using the ignored local
`realtime-clock.browser.crystalpack` were captured for all four species and
inspected at actual GPU output resolution. Their silhouettes, materials,
faces, thin wings and physical scale remain visible without camera clipping.
These captures establish static presentation, not animation or gameplay
correctness. The broad Python species validator remains affected by the
previously documented macOS Bayleef regeneration mismatch.

The broader goal covers every Pokémon model. This is a four-species art
increment, not evidence of complete visual quality or rig coverage for all
251 species. The artist-authored external Bellsprout rig remains unavailable
without an authenticated download; see [the external rig search](pitcher-sculptures.md).
