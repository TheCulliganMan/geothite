# Tyrogue, Smoochum and Jynx volume and posture revision

The October 20 review replaces three weak silhouettes: Tyrogue's plate-like
head, long diagonal legs and paired low hands; Smoochum's large lower body,
projecting fringe and vertical kiss ring; and Jynx's narrow cone skirt and
symmetric hand placement. The full Pokémon fidelity goal remains open.

## Reference evidence

Eight-angle Battle Revolution model views were inspected for
[Tyrogue](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339545/),
[Smoochum](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339547/)
and [Jynx](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339441/).
These establish both profiles, rear mass and appendage depth. Captured
[Tyrogue](https://pokemondb.net/sprites/tyrogue),
[Smoochum](https://pokemondb.net/sprites/smoochum) and
[Jynx](https://pokemondb.net/sprites/jynx) Crystal front/back, Black/White idle
front/back frames and X/Y front views establish the intended expression and
stance. The supplied pack's actual two front frames and back picture were
also inspected for each species.

The reference OBJ textures contain misplaced islands: extra Tyrogue brow/ear
marks and extra Smoochum facial marks and body bands. Captured sprites resolve
those artifacts. Extracted model poses also differ from Crystal's idle poses;
arm span alone is not a proportion measurement. The local pack retains its
normal palettes, including Smoochum's gold lips and Jynx's white hair/trim and
pink gown. Modern reference colors do not replace those palettes.

Reference geometry and textures are temporary external viewing inputs. The
original recipes, exporters and builder do not read them. No downloaded mesh,
texture, rig or animation enters the authored GLBs or bundle.

## Original geometry corrections

Tyrogue's three points now grow from a deep rounded skull. Its ears occupy
both profiles; narrow fitted eyes and a small determined mouth follow the
actual skull triangles. Broad poofy shorts sit above short, stout bent legs
and complete flat-soled shoes. Ankle-to-knee sections avoid the former shared
forward slope while retaining the reference's outward thighs and unequal
support depths. One wrapped hand is beside the face and the other rests low
and outward. Both palms and all six digits have occupied roots. Shoulder,
forearm, ear, head and supporting-leg pivots move with the new anatomy;
forearm blending follows the arm path rather than assuming both arms descend.

Smoochum's head and deep bob dominate its silhouette above a compact lower
body, short supports and tiny feet. The bob has a flared lower edge, rounded
crown, swept notched fringe and four compact leaves. A rear-quarter gap at the
temples required a fuller crown behind the fringe; the lower center is recessed
to preserve the notch. The two redundant purple iris overlays are removed from
the local four-color gaze. The gold lips form a horizontal pout around an
actual recessed opening. Neck, collar, arms, hair and leg controls follow the
revised proportions.

Jynx has a broad heavy bell skirt with seven folds, fuller bodice, a larger
face, swept hair curtains and pointed rear mantle. Its hands occupy different
heights and depths, following the Crystal guard instead of the extracted
model's spread arms. A sleeve bend was rebuilt after the self-intersection
check found crossed elbow faces. Layered hair tips meet the main curtains.
Both arm chains, all ten finger pivots, hair controls and head pivots follow
the revised geometry. The hem stays pinned.

Both kiss apertures have real depressions in the closed heads. Dark linings
follow those actual recessed triangles; projecting annular lips surround empty
air in front of the occupied backing. Depth probes derive their positions
from the stored mouths, compare the backing with adjacent skull tissue and
remain within the actual revised openings.

## Stored assets and executed checks

| Model | Runtime / logical parts | Vertices | Triangles | Joints | Accessors | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Tyrogue | 40 / 40 | 13,839 | 21,596 | 12 | 211 | 589,984 |
| Smoochum | 38 / 38 | 17,939 | 30,344 | 11 | 191 | 759,748 |
| Jynx | 35 / 45 | 18,473 | 30,004 | 22 | 220 | 846,100 |

Authoring heights remain 0.86, 0.78 and 1.28. Verified production pack heights
remain 0.7112 m, 0.4064 m and 1.397 m. Jynx's hand packing preserves the exact
45 logical anatomy ranges in 35 runtime primitives. Strict readers, source
pins, model budgets and the physical scale contract remain enforced.

Full canonical GLB SHA-256 values:

```text
tyrogue  aa5f37508f20315ba48d340b7b9601b33ef30d4286c2ac123eaa4c890af9838e
smoochum 8bd8b2b6b20cd4dd1ab45d8e11a286b2f31ab67a6632221337bd4d40b0a002d7
jynx     7a7a262314186adb5e6d3c8f9a78687c8023a50a6e91b0a94e4af190d7a009ec
```

The 51 focused Python checks cover closed outward geometry, normals, fitted
surfaces, occupied neutral and posed joins, self-intersection, exact source
parity, logical packing, binds, ownership, fixed supports, clip endpoints,
positive posed winding and bounded strain. Seven independent fidelity tests
cut the actual stored skull, shins, shoes, fringe, crown seams, mouth recesses
and lower skirt. The immutable baseline fails all seven tests. No existing
geometry, support, admission or budget tolerance was relaxed.

Blender's regular builder reproduces all three GLBs byte for byte. Every angle
of the final neutral orbit was inspected against the baseline and game orbit.
Fixed-camera 45/90-degree views cover idle at 0.25, action at 0.34 and recoil at
0.18. These sampled poses preserve the corrected supports, fitted faces, hair,
clothing and hand joins. They do not establish every move or animation frame.

```sh
python3 -m unittest discover -s tools -p 'test_tyrogue*.py'
python3 -m unittest discover -s tools -p test_smoochum_glb.py
python3 -m unittest discover -s tools -p test_jynx_glb.py
python3 -m unittest discover -s tools -p test_ice_dancers_sculpt.py
python3 -m unittest discover -s tools -p test_ice_fighter_fidelity.py
```

The background builder used `tools/build-battle-species.py` with an ignored
output directory and `--only TYROGUE,SMOOCHUM,JYNX`. Reference archives and
scratch verification files are removed after production review. Shared packs,
the task's persistent build cache and the selected ignored gallery remain.
Nothing is staged or committed.

## Production verification and limits

Ten focused Rust checks pass: three original-pack-size GPU selections, the
three canonical skin tests, Jynx's composed world hem/bindings, the new complete
Tyrogue/Smoochum walking-support test, biped hip centering and the full species
schema. Both small bipeds retain canonical upper anatomy while their complete
legs, shoes/toes and Tyrogue's shorts hems participate in walking. The checks
sample 64 travel phases with floor, contact and full culling bounds enforced.

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked --jobs 2 \
  -p crystal-voxel-view --lib --no-run \
  --config profile.dev.package.crystal-voxel-view.opt-level=0
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example immersive_battle_3d --example render_at_location \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

The resulting unit executable was run with the ten exact test names selected
by `tyrogue`, `smoochum`, `jynx`,
`species_schema_retains_anatomy_skin_order_and_physical_size` and
`pokemon_motion_biped_steps_are_centered_under_hips_not_toes`. Exact current
compressed GLBs were verified inside the test and both native executables.
The native executable hashes remained unchanged through the captures; the
ignored gallery retains those executable and canonical-asset proofs.

Fifteen selected native/classic pairs were inspected, including all 30 PNGs
and their preserved-state receipts. Real Route 29 walking encounters used
`RALA` repeated 20 times and the ordinary production move-menu hold. Their
385/385/449 witnessed movement frames lead to modeled Tyrogue/Smoochum/Jynx
and Pidgey, with no source-art species and `fallback=None` in every receipt.
The battle menu covers the classic player's back picture, so those pictures
establish state and presentation rather than visible back-sprite likeness.

Field captures used NewBarkTown, start `(7,12)`, hour 12 and `RR`, with cameras
`2 1 35 0`, `5 1 35 0` and `5 5 35 0`. The first value is zoom, the second
orbit and the third pitch. Reverse walking into the town trees obscured the
rear, so the retained back views use static EcruteakCity `(20,21)`, hour 12,
camera `5 5 35 0`. Those views expose the backs without foliage over the target;
building shade limits their material/readability comparison. The static city
fixture also places an NPC at the player coordinate. The Pokémon remain
separate and visible; this is model coverage, not acceptance of that NPC setup.
Different camera-relative walk runs can finish at different positions. State
and timing equality apply within each captured pair.

Tyrogue's skull, full shorts, shoes and high/low guard remain readable at
0.7112 m. Smoochum's large bob and compact body read at its small 0.4064 m size;
the wider field view does not prove tiny facial details. Jynx retains its
broader skirt, distinct hands, full hair and grounded hem at 1.397 m. The lit
field views and studio orbits establish surfaces obscured by tree/building
shade in the opposite-camera shots. The classic field omits the added follower.
Its ownership is independently recorded as the requested PartyFollower in each
receipt. None of these pairs substitutes a different physical size.

Every capture used `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`, `CRYSTAL_PREVIEW_MUTE=1`,
`CRYSTAL_RENDER_TEST_WINDOW_SIZE=1000x700`, normal palettes, capture timeout 180,
second-view toggle and hold-battle 0. Battle captures used hold-move-menu 1;
field captures used 0. All use the completed ignored
`content-packs/core-modular.browser.crystalpack`. No pack rebuild was required.

The [ignored local review gallery](../../target/pokemon-fidelity-review-20261020-ice-fighter/index.html)
retains 48 selected PNGs, 15 paired-state receipts and executable/encounter
proofs. All other 273 canonical GLBs remain byte-identical to the initial
snapshot. Temporary references, draft exports and unselected capture output
are removed; shared packs and build caches remain. The audit skill now calls
for occupied seam sections and rear-quarter review in addition to attachment
checks. Nothing is staged or committed.

Normal native samples and the inspected neutral orbits do not establish every
animation frame, move effect, camera, shiny variant, browser presentation or
the rest of the roster. The overall Pokémon fidelity goal remains active.
