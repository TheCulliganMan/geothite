# Zubat, Golbat and Crobat standard skins

The original `tools/bat_sculpt.py` recipes now publish one canonical standard
skin GLB per species. Their former JSON assets are retired. Geometry, normals,
materials, part names and triangle order retain their original f32 values.
Zubat keeps its eyeless hollow muzzle; Golbat keeps its deep hollow carapace,
recessed throat and four canines; Crobat keeps its angry eyes and two wing pairs.

| Species | Parts | Vertices | Triangles | Bytes | Joints | Accessors | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Zubat | 28 | 2,368 | 3,352 | 150,916 | 10 | 139 | 0.7874 m (2′7″) |
| Golbat | 32 | 3,545 | 4,784 | 193,972 | 10 | 158 | 1.6002 m (5′3″) |
| Crobat | 28 | 4,153 | 4,116 | 232,628 | 14 | 163 | 1.8034 m (5′11″) |

The three height digits were read from the verified local pack: 207, 503 and
511. Authored reference height remains 1.0; no runtime size or reader limit is
changed. Shared accessors keep all three within the existing bounded reader.

Neutral digests:

- Zubat: `737d2dad1c97b82b0fead248d72ac003e5e759ce90debf0c4cabb06d80c851d0`.
- Golbat: `db6b0c4bcb4c3f268b98ca45678c143db637a570e2cacc1867629fa2f2f558d4`.
- Crobat: `d01d5cfb92de35f18629800273962eb8a9719f659891ad7f397d44202ff717bd`.

One body bone owns the complete fitted face, mouth and teeth. Ears and their
insets share rigid owners. Upper wings have shoulder, wrist and tip joints;
Crobat adds shoulder and tip joints for each lower wing. Membranes, leading
edges and branching spars use the same continuous spatial weight field.
Zubat and Golbat's dangling legs and tiny toes retain their root geometry.

Idle lasts 2.4 seconds with 73 keys. Attack and hit last one second with 65
keys each. Clips use rotations only, identity endpoints and the existing source
battle cue clock. The body braces early; wing tips and ears trail. Shoulder
motion folds the wing while wrist/tip bends move out of its plane. A first
in-plane wrist implementation inverted a narrow contour triangle during idle
and was replaced before visual acceptance.

The pure authoring CLI was executed: its three outputs matched the canonical
GLBs byte for byte and removed seeded stale JSON files. The Blender batch
branch was updated and syntax checked; the full Blender batch was not run.

```sh
python3 tools/bat_glb.py
PYTHONPATH=tools python3 -m unittest test_bat_glb test_bat_sculpt -v
PYTHONPATH=tools python3 -m unittest \
  test_battle_model_assets.CanonicalSources.test_bats_resolve_one_canonical_skin_each -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked \
  -p crystal-voxel-view bat --lib
```

The 12 stored bat anatomy and skin checks, canonical routing check, focused
Rust registry check and GPU integration check passed. Broader regressions pass:
33 active rig tests (one existing audit ignored), 29 skinning tests and 13 actor
tests. A fresh native production build was inspected against paired 2D for all
three species; each receipt reports preserved state.

| Battle pair | Hash | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: |
| Zubat | 126865774 | 34 | 0.765732 |
| Golbat | 1078762090 | 34 | 0.733618834 |
| Crobat | 2045930643 | 35 | 0.720177125 |

Zubat’s hollow eyeless muzzle and hanging legs remain distinct from Golbat’s
tall carapace and open mouth. Crobat’s violet body, yellow eyes and lower wings
remain visible. All three retain their pack-derived physical sizes; the larger
wing envelopes cause the production camera to frame Golbat and Crobat farther
back. This side angle compresses the broad wing silhouettes. Zubat and Golbat
still render more teal than their blue source sprites; colours were deliberately
preserved for this rig migration. Wing presentation, palette fidelity, browser
rendering and detailed source-cue visual motion remain open. These previews
verify rig integration and preserved anatomy, not final visual acceptance.

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked -p crystal-bevy \
  --features location-tester --example immersive_battle_3d
# Execute once for each of ZUBAT, GOLBAT and CROBAT, using its lowercase filename.
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
./target/pokemon-models/debug/examples/immersive_battle_3d \
content-packs/core-modular.browser.crystalpack --wild-species ZUBAT --size 1000x700 \
--screenshot target/pokemon-native-review/zubat-battle-3d.png \
--second target/pokemon-native-review/zubat-battle-2d.png
```

An initial broad Python discovery also ran unrelated canonical model tests and
reported a Chikorita writer/GLB mismatch. The bat-focused checks passed. The
Chikorita mismatch was subsequently repaired by canonicalizing tiny geometric
zero components; see `chikorita-papercraft.md`. The initial broad discovery
was not reported as passing. Registering these skins is not proof that all
Pokemon models and rigs are complete.
