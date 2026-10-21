# Hitmonchan, Hitmonlee and Hitmontop: depth and posture audit

Hitmonchan now has a rounded skull behind its crown, a smooth rear head,
pointed shoulder guards, an open pleated hem, upright supporting shafts and
full boots. Its unequal bent guards carry padded teardrop mitts. Hitmonlee has
a broader headless upper body, outward-rising angry eyes, heavier upright shin
coils, larger clawed feet and curled hands on unequal bent guards. Hitmontop
keeps Crystal's inverted spin with a deep helmet, swept side flanges, six fitted
folds, a fuller compact body and shorter raised limbs and tail.

The three original `tools/hitmon*_sculpt.py` recipes and `*_glb.py` exporters
remain authoritative. Reference meshes, textures, rigs and animations are not
authoring inputs. Authoring heights (1.30, 1.24, 1.25), pack sizes (1.397,
1.4986, 1.397 m), normal palettes and loader budgets are preserved. Crystal's
pink Hitmonchan kit and gray Hitmonlee coils differ from Battle Revolution's
reference palette. All other 273 canonical GLBs remain byte-identical to the
start of this revision.

Full eight-angle Battle Revolution reference volumes were inspected for
[Hitmonchan](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/303322/),
[Hitmonlee](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/297949/)
and [Hitmontop](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339546/).
Captured BW front/back idle frames, Crystal front/back pictures and X/Y front
pictures were inspected through their [Hitmonchan](https://pokemondb.net/sprites/hitmonchan),
[Hitmonlee](https://pokemondb.net/sprites/hitmonlee) and
[Hitmontop](https://pokemondb.net/sprites/hitmontop) galleries. Two front frames
and the back picture were also read from the completed local pack for each.
Neutral extracted arm spreads establish volume; captured idles establish the
bent guard and inverted spin. Hitmontop's exported texture islands put extra
eyes and patches on the reference; captured pictures establish the actual face
and blue body, and those UV artifacts were not reproduced.

The review used matching eight-angle 4-by-2 sheets for baseline, reference and
candidate, with shared 1.6 orthographic framing. Each final orbit was inspected.
An additional eight-angle underside orbit verifies Hitmonchan's open hem.
Stored idle, attack and recoil samples were rendered from 45 and 90 degrees:
three Hitmonchan samples, four Hitmonlee samples including early and peak kick,
and six Hitmontop samples including four idle phases. The 26 posed views retain
their neutral model frame rather than resizing each pose independently.

The audit found and repaired defects in intermediate candidates: hip roots
showing through the skirt, crossed elbow faces, cropped eye layers, early-kick
heel penetration and lowered raised feet during recoil. Curving an ear-clipped
crown also exposed long sliver fans as normal streaks. Improving its internal
planar diagonals before curving, and separating cap/wall normals, produces the
rounded skull without excessive subdivision. The personal fidelity-audit skill
now includes these triangulation and early-kick checks.

The 61 Python sculpture, skin and fidelity tests pass. They check exact source
parity, closed outward solids, normals, occupied roots, fitted layers, floor
clearance, fixed supports, winding and strain, coherent owners, neutral
endpoints, the lifted kick and complete smooth idle revolution. Six new
stored-volume tests measure skull/boot depth, actual shin sections, the empty
hem interior, eye slope, body/coil mass and compact raised limbs. The unchanged
baseline fails all six new tests. No geometry, support or budget tolerance was
relaxed to accept the candidates.

Blender's normal builder reproduces all three canonical GLBs byte for byte.
Hitmonlee normalizes signed zero in rounded normals so Python 3.9 and Blender's
Python produce identical bytes. This does not change its geometry or shading.

Production travel also needed a posture correction: walking IK was rebinding
Hitmontop's raised kicking legs as ground supports. Spin travel now retains the
authored assembly and uses its cached crown contact to center the turn. The
other two fighters keep two complete walking supports. A focused regression
checks preserved upper bindings, all vertex floor/culling bounds, contact and
the spin point's position over 64 travel phases.

| Model | Parts | Vertices | Triangles | Joints | Accessors | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Hitmonchan | 34 | 10,981 | 16,084 | 12 | 184 | 487,340 |
| Hitmonlee | 33 | 15,445 | 25,744 | 10 | 164 | 639,368 |
| Hitmontop | 41 | 10,934 | 17,604 | 11 | 191 | 447,532 |

Source pins are recorded in the [Hitmonchan](hitmonchan-sculpture.md),
[Hitmonlee](hitmonlee-sculpture.md) and [Hitmontop](hitmontop-sculpture.md) guides
and checked by the strict exporters and Rust schema. Full GLB SHA-256 values:

```text
hitmonchan 0ac86334c1c937bfde41432a2368b967c02e3c1422225c2ce7d85dd361c1054a
hitmonlee  ea72ad74032367a915cba9d721680831da343599d2417c7ffd0b09095db90aac
hitmontop  0d4bd967f5d37a98b4f63a01e06cbc82cc87807de6c4e5dcb494132f5c895c79
```

Run focused Python verification with:

```sh
python3 -m unittest discover -s tools -p 'test_hitmon*.py'
```

The 11 focused Rust checks pass: eight Hitmon skin, GPU selection and travel
checks, the full species schema, biped hip centering and stopped spin/roll
orientation. The test executable's three exact compressed GLBs were verified
before execution. The native examples were rebuilt and their three exact GLBs
verified before capture as well.

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked --jobs 2 \
  -p crystal-voxel-view --lib --no-run \
  --config profile.dev.package.crystal-voxel-view.opt-level=0
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example immersive_battle_3d --example render_at_location \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

The resulting test executable was run with filters `hitmon`,
`species_schema_retains_anatomy_skin_order_and_physical_size`,
`pokemon_motion_biped_steps_are_centered_under_hips_not_toes` and
`travel_roll_and_spin_keep_orientation_when_stopping`. Blender regeneration
used `tools/build-battle-species.py` with the ignored review output directory
and `--only HITMONCHAN,HITMONLEE,HITMONTOP`.

Ten native 3D/classic pairs were inspected, including all 20 PNGs and their
preserved-state receipts. Three real Route 29 walking encounters used `RALA`
repeated 20 times, stopped at the move menu, passed scene admission and exposed
the requested modeled identities with no fallback. Field captures used
`NewBarkTown`, starting `(7,12)` at hour 12 with `RR`, zooms 2/5, orbit 1 and
pitch 35 degrees. Hitmontop also has an opposite-camera pair at zoom 5, orbit 5.
That extra view shows its face and helmet flanges under tree shade; the earlier
native captures show its back. Its crown contact and complete raised limbs
remain coherent during travel. Hitmonchan's deeper skull, open hem and complete
boots and Hitmonlee's upper mass, eyes, coils and claws remain readable at pack
size. Separate camera-relative walk runs can finish at different positions;
state equality applies within each matched pair.

Captures used `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`, `CRYSTAL_PREVIEW_MUTE=1`,
`CRYSTAL_RENDER_TEST_WINDOW_SIZE=1000x700`, normal palettes and hold-battle 0.
Battle captures used hold-move-menu 1; field captures used 0. The classic field
omits the added follower, and the battle move menu covers the player's back
picture. Those pairs establish state and presentation; the separately inspected
pack front/back sheets and complete orbits establish the obscured anatomy.

The ignored [local review gallery](../../target/pokemon-fidelity-review-20261020-hitmon/index.html)
retains 41 selected PNGs, ten paired-state receipts and executable asset proofs.
Reference archives, draft exports, unselected moving frames and temporary
verification files are removed after review. Shared packs and caches remain.
Nothing is staged or committed.

The complete Pokémon fidelity goal remains open. These sampled neutral and
action views do not establish every animation, move effect, camera, shiny
variant, browser presentation or the rest of the roster.
