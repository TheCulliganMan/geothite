# Omanyte and Omastar canonical skins

The nautilus pair now uses canonical GLBs through the normal background builder,
direct sculpture CLIs, strict logical readers and Rust dispatch. Registration
reaches 235 skin variants and 40 static species files. Full-roster model and rig
improvement remains active.

| Species | Vertices / triangles | Logical / runtime parts | Joints | Accessors | Bytes |
| --- | --- | --- | --- | --- | --- |
| Omanyte | 12,587 / 18,848 | 32 / 20 | 15 | 136 | 569,656 |
| Omastar | 14,178 / 21,292 | 43 / 26 | 19 | 156 | 621,360 |

Source shape, ribbing, spirals, palettes, eyes and all ten tentacles remain.
Omastar retains six spines and four solid beak quadrants around a real empty
cross. Packing combines only shell seams, paired spirals and adult spines on
their common shell control. Exact logical ranges restore each original part.
No vertex/triangle/part/joint/accessor/byte limit is increased.

The new all-triangle pose check found an existing reversed triangle at the
first polar cap of both shells. Ribbing makes the polar outline slightly
concave; the old fan triangulation folded one ear backward. The source now uses
the existing ear-clipping helper on the actual YZ cap outline. Shell vertices
and triangle counts remain, while cap triangulation and affected smooth normals
are corrected. This repairs the geometry rather than exempting small faces from
the positive-winding check. Signed zero values are normalized for stable hashes.
Before the cap repair, all numeric geometry matched the older exports; their
only byte differences were 69/106 signed zeros.

Both skins have a structural identity root, mantle, shell, two gaze controls and
ten independent tentacle hinges. Omastar adds four beak controls. Shell ribs,
spirals and spines follow the shell. Eye surfaces and their fitted pupils/glints
share each gaze carrier. Tentacle hinges use the actual eighteen-sided root cap
centres, including the juvenile fans' individual source floor translations.
These centres are read from cap triangles rather than inferred from bounds.
No parallel copy of source tentacle paths is introduced.

Idle lasts 3.2 seconds with 81 keys. Attack/hit last one second with 65 keys,
rotation-only source cues and neutral endpoints. The mantle, shell, gaze and
tentacles rotate about Y, preserving their authored vertical coordinates.
Grounded tentacles sweep along the floor; no clamp or hover offset is added.
All non-root controls move weighted geometry independently. The structural
root has no directly weighted vertices. Adult beak quadrants articulate while
preserving their empty cross and occupied mantle roots.

World crawling reuses the ten canonical hinges and their exact bindings instead
of creating replacement joints. Distance-driven crawl strokes compose with the
authored idle clock. At rest the canonical motion remains; during travel every
tentacle supplies a stroke and clock motion continues even at a fixed travel
phase. The older procedural fallback retains its previous behavior. No leg IK
is added to these curled appendages. Complete world bounds and source ground
contacts remain valid throughout the sampled cycle.

Thirty-one checks pass against the final stored GLBs in 21.220 seconds: 23 source
anatomy checks and eight independently decoded skin checks. Coverage includes
recipe/packing identity, hierarchy/inverse binds, standard budgets, closed
outward solids, actual corner normals, all controls and rigid carriers,
ground contact, every triangle's winding and rigid strain, neutral endpoints,
occupied shell/eye/tentacle/beak roots, fitted gaze support, unobscured pupils,
the genuinely empty beak cross, negative controls and byte/species/source
mutation rejection. Palette comparisons use float32 bits; source eye-visibility
and adult-cross rays use the existing indexed oracle with unchanged mathematics
and thresholds.

Four focused Rust checks pass in 1.01 seconds: original-size GPU activation for
both species, canonical shell/eyes/ground contacts/controls, and complete world
crawling with exactly preserved weights, all ten strokes and continuing clock
motion. Original pack dimensions are 0.4064 m and 0.9906 m against standing
height; authoring heights remain 0.80 and 1.08. The normal background builder,
direct CLIs and reviewed candidates match canonical GLB bytes exactly. Uniform
scale, complete animated bounds and stage gates remain intact.

The ignored `target/pokemon-native-review/nautilus-skin-review.png` was inspected
inline after the cap correction. Each named row compares previous static,
decoded neutral, front/rear/side and attack at 0.34 using the same fixed neutral
fit. The broad juvenile gaze and fan, adult slit pupils, spiked shell, ten curls,
fine ribs/spirals and actual beak cross remain distinct. Authoring views alone
do not establish production acceptance.

Reproduction:

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  crates/crystal-voxel-view/models/battle_species --only omanyte,omastar
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools \
  python3 -m unittest -v test_omanyte_sculpt test_omastar_sculpt test_nautilus_glb
```

| Species | Logical float32 geometry SHA-256 | Packed float32 geometry SHA-256 | GLB SHA-256 |
| --- | --- | --- | --- |
| Omanyte | `70c60a2c6dbd169dd870c5af37ef35c2c2c96ecc366cfff95eb12ae93835532d` | `9df10630971b62ea925b0d03f556b92a04dcfe75d25ad66bbe7f0c7965f86246` | `fc13e2dc22066cc2e04317d5d8c0df8ff37dd3dceb95802bb1e8de7764adf0f1` |
| Omastar | `598f572bd5aee2e09db4e9b647c6629c3b48a89ba2cf64e9d72e25c8ddb930c4` | `2df6740a96834863a7812b4ca47f7fbf799992bb8eab0541f445325d864fb4ee` | `04c9f7b25d77dbe78caf960c33276d7cd28de5a37f9f49d615c7620e11f11c47` |

Native captures use `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image` and `CRYSTAL_PREVIEW_MUTE=1`.
The production path disables Winit and macOS primary-window retention and uses
the schedule runner. Binary preflight checks headless/mute tags and exact
compressed current GLBs in both examples. No windows, focus changes, shiny
checks, grass-height, AA/filtering, pack, audio or gameplay work is included.

The shared schema/physical-size check passes in 84.69 seconds and the shared
elapsed-idle/exact-current-attack cue check in 84.47 seconds. Together with the
31 stored Python and four focused Rust checks, 37 scoped checks pass. Both
native capture examples built successfully in 49.73 seconds with
`location-tester,bevy/zstd_rust`. These checks do not establish full-suite or
complete gameplay acceptance.

Example offscreen world reproduction from this checkout:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_PARTY_SHINY=0 \
CRYSTAL_RENDER_TEST_PARTY_SPECIES=OMASTAR \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=1000x700 \
CRYSTAL_RENDER_TEST_CAMERA='3 2 35 0' \
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic CRYSTAL_RENDER_TEST_WALK_STEPS=0 \
./target/pokemon-models/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map Route36 --x 20 --y 8 --view both --hour 12 --walk D \
  --screenshot target/pokemon-native-review/nautilus-omastar-world.png
```

## Native paired review

Six current production 3D/2D pairs and two moving-world frames were inspected
inline, all offscreen and muted at 1000×700. Every paired authoritative state
is preserved. Four front/world captures retain 32 actual synthetic-input
movement rows each across five Route36 positions, (20,8) through (20,12).
Both named walking battles admit complete original 0.4064/0.9906 m bodies,
uniform scale and zero support offset, with no fallback or relaxed camera,
authentic-ground, omitted-geometry/shadow or whole-body bounds gates.

Grounded fans/curls remain at the path and arena surface. Juvenile round eyes,
adult yellow slit eyes, cream ribbed shells, side spirals, adult spines and
distinct tentacle arrangements remain recognizable. The frontal adult view
shows the genuinely open beak cross; profile world/battle views hide portions
of the mouth, far gaze and some arms. Small ribs/spirals alias at distance and
the undershell receives a strong shadow. These remain broader lighting/AA and
camera-coverage work. No physical dimension or FOV is changed for readability.
Browser, all world/material combinations and full gameplay remain unverified.

Final captures completed in 35–56 seconds each using the same stable rebuilt
executables throughout. Exact current compressed GLBs and binary identities
were checked before and after. Completed producer scripts, temporary
source/candidate/CLI copies and redundant intermediate frames were removed.
Final raw/paired images, receipts, four movement CSVs, two moving-world frames,
metrics and logs remain ignored under `target/pokemon-native-review`. No shiny
checks, grass-height, AA/filtering, pack, audio or gameplay changes are included.
The full model and rig improvement goal remains active.

| Species / view | Frame / checksum | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| Omanyte front | 1539 / 2599334376 | 1542 | 26.189513209 |
| Omanyte world | 1450 / 4250200729 | 1451 | 24.829035167 |
| Omanyte battle | 1857 / 2947599764 | 2434 | 41.620041792 |
| Omastar front | 1621 / 2550561944 | 1627 | 27.952924208 |
| Omastar world | 1517 / 2663107910 | 1526 | 26.061003875 |
| Omastar battle | 1598 / 4157514186 | 2183 | 37.444929376 |

Stable executable SHA-256:
- `render_at_location`: `2b0e2af83ef4722948bcbb51ceeda31395c4201570ac30d022e5158722a5bba3`.
- `immersive_battle_3d`: `88ee1d6d0adcf38030995f2ceb19e6dee5c274eca402cec4f12824624b1cb296`.
