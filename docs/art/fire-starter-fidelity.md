# Cyndaquil family reference and posture revision

The previous Cyndaquil stood on four small paws with a cream rear end cap
showing through its coat. Quilava's old four-legged stance was chosen from
its back picture without inspecting the pack's front animation. Typhlosion
retained a long level bill, round ears, long forearms, a short dorsal coat and
a narrow vertical flame bank.

## Reference decision and original geometry

The Battle Revolution models for
[Cyndaquil](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339468/),
[Quilava](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/303321/)
and [Typhlosion](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339469/)
were inspected from all eight angles with matched studio cameras. The actual
provided pack's first four front frames and back picture were inspected for
each species. Four frames each from the separate Showdown
[front](https://play.pokemonshowdown.com/sprites/ani/quilava.gif) and
[back](https://play.pokemonshowdown.com/sprites/ani-back/quilava.gif) animations
provide supplemental later-game posture evidence; their game generation is not
established here.

The Crystal front frames show **upright Quilava**, agreeing with the Battle
Revolution model. The later pixel animation is quadrupedal. This revision uses
the Crystal stance rather than averaging those incompatible poses. Cyndaquil
keeps its low, downward-pointing muzzle, tucked forepaws and broad rear supports.
The game reference's flat flame cards do not become authored flat cards.

All shipped geometry remains original procedural work in
`tools/cyndaquil_sculpt.py` and `tools/fire_badger_sculpt.py`. Downloaded meshes
and textures are external inspection references, never authoring inputs or
runtime assets.

- Cyndaquil has a shorter curved snout, fuller rear haunches and short raised
  forepaws. A closed dorsal patch covers the actual rear body end cap. Four
  fitted ignition spots follow the coat and its skin field. Five thick folded
  fire blades spread across the back, with their pivots at the revised roots.
- Quilava has an upright pear-shaped trunk, short forearms with complete
  palms and fingers, two grounded rear feet and six blunt toes. Short pointed
  ears, cream cheeks and a narrow slanted gaze replace its generic mask.
  Three neck blades sweep backward above a separate low rump fan.
- Typhlosion has a flared lower trunk, continuous sloping back and a long blue
  coat with a jagged rear hem. A compact downward muzzle, cream cheeks,
  triangular ears, narrow gaze and real backed mouth replace its level bill.
  Short forearms rest beside the upper body. Seven thick folded blades form a
  wide shoulder mantle. The lower coat transitions to the ground root so its
  hem stays seated while the upper body articulates.

Body/head color halves share exact boundary positions and normals. The eyes
cross that color boundary and fit to the actual complete stored skull, rather
than one color half. Mouth probes derive their positions between the current
stored jaws and require occupied backing; old fixed world-space probes would
miss the revised cavity.

| Species | Vertices / triangles | Logical / packed parts | Joints | Accessors | Bytes | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Cyndaquil | 16,559 / 8,610 | 38 / 38 | 12 | 227 | 660,332 | 0.508 m body |
| Quilava | 10,428 / 13,428 | 68 / 39 | 13 | 161 | 417,244 | 0.889 m |
| Typhlosion | 11,439 / 15,262 | 76 / 44 | 17 | 189 | 472,532 | 1.7018 m |

The actual verified compiled-pack reader reports height digits 108, 211 and
507 respectively. Authoring heights remain 1.25, 1.20 and 1.45; Cyndaquil's
unchanged 0.716729 body datum excludes flame tips. Physical sizes, loader
budgets and admission tolerances are unchanged.

## Executed authoring checks

All 45 Python checks pass against the final canonical GLBs: fifteen Cyndaquil,
twenty-one fire-badger and nine independent fidelity checks. The nine independent
checks reject all nine corresponding defects in the immutable previous assets.
They read real stored buffers without requiring current authoring pins.
Closed outward solids, corner normals, fitted surfaces, occupied roots, complete
digits, mouth space, skin carriers, floor contact, winding, strain, deterministic
bytes, hierarchy and bounded loaders retain their strict checks.

The actual background Blender builders and regular Python exporters produce
identical GLBs. Only these three of the 276 canonical battle models changed;
the other 273 retain their pre-review hashes. All eight final neutral angles
and 27 decoded pose/angle views were inspected: idle 0.25, attack 0.34 and hit
0.18, each at quarter, profile and rear for all three species. These are sampled
animation checks, not every-frame visual acceptance.

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
  --threads 2 --python-exit-code 1 --python tools/build-actor-props.py -- \
  target/pokemon-native-review/fire-starters-fidelity-20261021/builder-actor \
  --only=battle_cyndaquil --skip-preview
/Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
  --threads 2 --python-exit-code 1 --python tools/build-battle-species.py -- \
  target/pokemon-native-review/fire-starters-fidelity-20261021/builder-badger \
  --only quilava,typhlosion
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools python3 -m unittest -v \
  test_cyndaquil_glb test_fire_badger_sculpt test_fire_badger_glb \
  test_fire_starter_fidelity
```

The personal `game-model-fidelity-audit` skill now requires a source-game pose
decision when front/back references disagree, and verification of production
travel styles and support classification after a stance change. Its turntable
helper keeps all eight neutral angles by default and supports selected extra
angles for decoded poses.

| Species | Logical geometry SHA-256 | Packed geometry SHA-256 | GLB SHA-256 |
| --- | --- | --- | --- |
| Cyndaquil | `32786eb0607cde77e469338e2d262ff91e86b0e77c710ce72bcb5c96d1179d53` | same | `0a84ed78807c61066121070362b12ec3b51986dd5349a491f3408e175f5fbb84` |
| Quilava | `6e57c783a464cfc59e4d89808377df65f11b019c17b08c180c002f41f3396904` | `05c57f5d165f3db779b2a34132f1de16b2c01efcbef27c7f45a66ce6c8c59708` | `0ceb00918789e7d642a66594ecf8d238e318bf2ddc3d40cc79f72db588e46bcb` |
| Typhlosion | `464c12449f8fc3c7371ff95c8684cc6f3938cbcfa803bfaaa8b00dd1cf395e1a` | `2eb6d40426fe2de77970105d100433fca35a127caf4f385e7af7758ae25f6813` | `2f3e88863d338279c98e0155477fa88a2ef285037a931bfd463c569d1dd725ac` |

## Rust and production verification

Eleven Rust checks pass with the final embedded models: source-size GPU
activation for all three, two-support walking for the whole family, badger
skin/root/coat/pose checks, exact embedded bytes for every catalog model,
the complete species schema/physical-size check, weighted neutral endpoints,
current visibility bounds and both malformed-input rejection suites.
Cyndaquil and Quilava are removed from the quadruped classifier and the trot
travel style. The real walking rig retains upper bindings and keeps their
forepaws raised throughout 64 sampled gait phases. All three retain strict
floor, support-contact and locomotion-bound checks.

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked --jobs 2 \
  -p crystal-voxel-view --lib --no-run --message-format=json
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example render_at_location --example immersive_battle_3d \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

The Rust test executable SHA-256 is
`32652bc77bcaab08da87cf2f8a7fbad8035eedbf12c4b6277a5d51b0f28abfd0`.
The exact compressed final assets were located in that executable and in both
rebuilt preview executables before capture. The preview SHA-256 values are
`fbeb66e42ea10b78a52a6c8fcf983625892e40e808bce158813376a7f68547f0`
(`render_at_location`) and
`8edc80ee2c80635c7993c6bd3a57b5b6d970219c4f14fddf6da6bc2d2c7e3537`
(`immersive_battle_3d`).

Twelve headless native/classic field pairs were inspected at 1000 × 700, noon,
normal palette and actual pack size. Each receipt preserves state and includes
the intended `PartyFollower` model. All cases use EcruteakCity:

| View | Player start | Camera orbit / pitch | Input |
| --- | --- | --- | --- |
| Clear front quarter | (18, 23) | `1 35` | None |
| Elevated profile | (18, 23) | `6 50` | None |
| Rear | (20, 27) | `4 35` | None |
| Ordinary field walking | (18, 23) | `0 35` | `RR` |

Camera zoom is 4 for Cyndaquil, 3 for Quilava and 2 for Typhlosion; this changes
only framing, never physical model size. The first straight-on Cyndaquil trial
hid its body behind the trainer and was discarded. The selected front-quarter
view shows the complete creature. Classic field views omit the added follower,
so those pairs establish matching state; separate actual sprites and studio
views establish likeness. The walking screenshots show the settled field after
ordinary controller input; Rust separately checks the complete sampled gait.

Three actual admitted battle pairs also preserve state with intended modeled
identities, no source-card bodies and `fallback=None`:

- Cyndaquil/Pidgey in the actual Falkner trainer fixture in VioletGym.
- Quilava/Sentret in a real Route 29 walking encounter.
- Typhlosion/Sentret in a real Route 29 walking encounter.

All six battle PNGs were inspected. The move menu covers the classic player's
back picture, so the pack back sprite and separate rear views supply that
likeness evidence. Cyndaquil's existing battle fire effects overlap its authored
fire blades in that presentation; the studio and field views show the folded
blade geometry separately. Battle scenery admission reports
`accepted=true reason=None`; no controller, witness or size rule is waived.

For the field front-quarter pair, set `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`, `CRYSTAL_PREVIEW_MUTE=1`,
`CRYSTAL_RENDER_TEST_PARTY_SPECIES=CYNDAQUIL`,
`CRYSTAL_RENDER_TEST_WINDOW_SIZE=1000x700`,
`CRYSTAL_RENDER_TEST_SECOND_VIEW=toggle` and
`CRYSTAL_RENDER_TEST_CAMERA='4 1 35 0'`, then run:

```sh
target/pokemon-models/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map EcruteakCity --x 18 --y 23 --view both --hour 12 \
  --screenshot target/pokemon-native-review/fire-starters-field.png
```

Battle captures use the same completed pack with `immersive_battle_3d`,
`--size 1000x700`, `--screenshot` and `--second` paths, headless image capture,
`CRYSTAL_RENDER_TEST_HOLD_BATTLE=0`,
`CRYSTAL_RENDER_TEST_HOLD_MOVE_MENU=1`,
`CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic`,
`CRYSTAL_RENDER_TEST_WALK_STEPS=1` and `CRYSTAL_ENCOUNTER_TRACE=1`.
The Cyndaquil fixture uses `--falkner-encounter` and sixty `A` presses. The other
two use `--walking-encounter`, their party-species override and twenty `RALA`
repetitions. The trainer fixture directly supplies Cyndaquil.

The ignored gallery is
`target/pokemon-fidelity-review-20261021-fire-starters/index.html`. It retains
the selected source/baseline/final turnarounds, sampled poses, native/classic
PNGs, fifteen paired-state receipts and verification results. Temporary
reference archives, extracted meshes/textures, draft renders and helper binaries
are removed. Shared packs and build caches remain. Nothing was staged or
committed. Full-roster, shiny, browser and every-frame visual acceptance remain
outside this batch's coverage.
