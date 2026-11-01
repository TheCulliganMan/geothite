# Bellsprout family sculptures

The three canonical static meshes now come from `tools/pitcher_sculpt.py`.
Bellsprout has a sinuous stalk, spreading roots and a tilted hollow trumpet.
Weepinbell retains its hanging bell silhouette and forward-facing mouth.
Victreebel has a tall tapered pitcher, upward opening, hood, hooked vine and
paired teeth. Curved leaf blades include raised midribs; green markings follow
the pitcher surface instead of protruding as separate balls. All retain the
previous 1.05-unit height and floor datum.

The standard builder dispatches to this recipe, preserving one canonical mesh
per species. No game content or external geometry is consumed. These are
static models; this increment does not add skinning or animation.

```sh
python3 tools/pitcher_sculpt.py --output crates/crystal-voxel-view/models/battle_species
python3 tools/test_pitcher_sculpt.py
cargo test --locked -p crystal-voxel-view every_exact_species_is_grounded_and_volumetric --lib
```

Stored-mesh tests verify reproducibility, closed connected surfaces, oriented
edges, winding, unit normals, mouth recess depth, fitted pigment, physical
dimensions and a 7,000-triangle ceiling. Software raster inspection covers
neutral front and rear views. Native 1000×700 production battle fixtures were
also captured with the ignored local `realtime-clock.browser.crystalpack`.
The low camera angle exposed Victreebel's bright inner wall hiding its floor;
a fitted dark cavity lining keeps the opening legible from that angle.
The five model checks and the Rust loader regression passed. The all-species
Python validator stopped on the unchanged Bayleef GLB's byte-exact canonical
regeneration check on macOS, before completing its full catalog audit.

## External rig search

Wersaus33's [artist-authored rigged Bellsprout](https://sketchfab.com/3d-models/bellsprout-rigged-pokemon-68338b3a64ef4ed49ee4335af63d23e7)
is listed as downloadable under CC Attribution. On October 10, 2026, its public
page fetch returned 403, and the official model download endpoint returned 401
with an authentication requirement. A subsequent browser visit loaded the
creator's page, confirmed the CC Attribution listing and NoAI tag, and opened
a sign-in dialog when Download was selected. No geometry was downloaded or imported;
the rig, topology, textures and game suitability remain unverified. If an
authorized download becomes available, inspect the actual asset and license,
retain attribution, and compare it in the production renderer before adopting
it. Search results alone are insufficient evidence for an import.

The complete objective still covers every Pokémon model. This family increment
does not establish visual quality or rig coverage for all 251 species.
