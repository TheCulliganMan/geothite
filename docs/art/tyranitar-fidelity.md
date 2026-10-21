# Tyranitar reference, volume and posture revision

The previous model passed stored-mesh checks but retained a bottle-shaped
torso, a long muzzle, diagonal ankles and projecting pads. Four central dorsal
cones and three tall fins along a high curled tail gave its profile and rear
the wrong structure. Two chest vents, two thigh vents and no rear openings
also omitted visible reference features.

## Inspected references and corrections

The [Battle Revolution reference](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339557/)
was inspected at 0, 45, 90, 135, 180, 225, 270 and 315 degrees, with matched
studio elevation, lighting and orthographic scale. Four frames each from the
captured [front](https://play.pokemonshowdown.com/sprites/ani/tyranitar.gif) and
[back](https://play.pokemonshowdown.com/sprites/ani-back/tyranitar.gif) idle
animations separately establish stance and tail motion. The supplied pack's
first four front frames and back picture retain the actual Crystal evidence.
The extracted game's neutral arms differ from the captured idle; their spread
is not imposed as the authored battle stance.

All geometry remains original procedural work in `tools/tyranitar_sculpt.py`.
Downloaded meshes and textures are disposable external references, not export
inputs or runtime assets. The reference export's striped triangular texture
artifact is not copied into the authored dark vents.

- A deeper lower barrel and broader chest replace the narrow bottle shoulder.
  Thick fitted front and rear mantle plates descend around a long diamond
  belly with three narrow segment seams.
- A compact tapering skull, descending forehead keel and swept crown replace
  the long flat bill. Narrow fitted gazes, short teeth and a real backed mouth
  remain rooted to their moving carriers.
- Short sloping upper arms and bent forearms carry small complete palms and
  three downward claws beside the chest. Wrists and arm pivots follow them.
- Broad haunches stand over nearly vertical ankle shafts and flat full heels.
  Forward toes remain distinct from the direction of the leg itself.
- Two complete rows of four solid angular dorsal wedges frame the back armor.
  Paired chest/thigh openings and two rear openings follow their actual parents.
- The heavy tail lies low, with four small radial points around its blunt end cap. It no
  longer carries a series of tall upper fins or a high curled tip. Its sampled
  ring weights and terminal-point carriers follow the revised path.

The coat uses a softer olive and the belly a bluer slate, matching the game
render's treatment within the existing lit material style. No emission or
unlit-material workaround is introduced. Authoring height remains 1.40;
production height remains the pack's 2.0066 m. A direct read of the current
local pack confirms `TYRANITAR height_digits=607 height_m=2.0066`.
Loader budgets are unchanged.

| Logical parts | Packed parts | Vertices | Triangles | Joints | Accessors | Bytes |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 87 | 41 | 12,272 | 18,056 | 14 | 225 | 564,076 |

## Stored geometry and motion verification

Thirty Python checks pass: thirteen sculpture checks, eight decoded-skin
checks and nine independent fidelity regressions. They cover closed outward
solids, shading, actual parent support at vertices/edges/face centers, occupied
roots, all controls, floor contact, winding, strain, genuine mouth volume and
unchanged loader limits. Mouth samples are derived between the current stored
lips and must have an occupied throat behind them; former world positions
would test the wrong cavity after the head edit.

The independent fidelity tests read stored buffers without the authoring pin.
Eight reject the immutable previous GLB: shin direction/full heels, body
depth, muzzle/keel, hand projection, two dorsal rows, vent distribution,
segmented diamond and low radial tail. The old model already passes the
backed-mouth guard; that is not presented as a newly found defect.

Four thigh vents now parse their side separately from their vent index. The
posed parent-support test caught a left marking following the opposite leg;
all four now have explicit correct-carrier assertions. No fitting, winding,
floor or strain tolerance was relaxed.
The personal `game-model-fidelity-audit` skill now explicitly checks side
identity separately from repeated-detail indices. It also distinguishes the
axis of a short thick ankle from horizontal bounds displaced by its end caps.
The actual stored cap planes measure about 37.7 degrees of forward shaft
inclination in the prior model and 4.3 degrees in the revision. Heel extent is
checked independently; a projecting toe is not treated as the leg direction.

Regular Python and the actual background Blender builder produce identical
canonical bytes. All eight revised studio views and nine stored pose/angle
views were inspected: idle 0.25, attack 0.34 and hit 0.18, each at quarter,
profile and rear. These sampled clips do not establish every animation phase.

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
  --threads 2 --python-exit-code 1 --python tools/build-battle-species.py -- \
  target/pokemon-native-review/tyranitar-fidelity-20261021/blender --only tyranitar
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools python3 -m unittest -v \
  test_tyranitar_sculpt test_tyranitar_glb test_tyranitar_fidelity
```

Canonical source geometry SHA-256:
`143d10ba16eaa62f9cab0c83a4b73bd310cee49c2932f042a43a86adb076101e`.
Packed geometry SHA-256:
`2c526189fa887aa48b726147687c9810621867301324d38e41cd011df15bf6c5`.
GLB SHA-256:
`8bb7778edd1acee66cd35d571abb1f0f907945054e0009a612a7374af8c9582a`.

The reviewed Blender output was compared byte for byte with the regular
Python candidate before copying it to the canonical model path. The other
275 canonical battle-species GLBs match their pre-review hashes.

## Native verification and limits

Four exact Rust checks pass with the final model bytes embedded in the tested
executable:

```text
battle_view::skinning::tests::tyranitar_skin_activates_gpu_at_source_size
species_rig::motion::tests::tyranitar_skin_walks_on_two_complete_supports_and_preserves_upper_armor_and_tail
species_rig::tests::species_schema_retains_anatomy_skin_order_and_physical_size
species_rig::tests::tyranitar_skin_preserves_armor_jaws_grounded_claws_and_independent_controls
```

The unit executable was built using:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked --jobs 2 \
  -p crystal-voxel-view --lib --no-run \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --message-format=json
```

Each listed check was run with `--exact --nocapture` and
`CRYSTAL_RENDER_TEST_PACK` set to the absolute local
`content-packs/text-tui.crystalpack` path. The tested executable's SHA-256 is
`70b983727b3024879dd4ef0f32dd6dbd858fd7775c3d23a45369a9b43852629f`.
The exact current compressed Tyranitar asset was also located in both native
preview executables before capture. This distinguishes the reviewed model
from an executable containing an earlier revision.

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example render_at_location --example immersive_battle_3d \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

Four headless native/classic field pairs preserve the actual authoritative
state and include the intended `PartyFollower` with source
`species:TYRANITAR:icon_monster`. All use the completed local
`content-packs/core-modular.browser.crystalpack`, noon, normal palette,
1000 × 700 image capture and the production controller.

| View | Map | Player start | Camera setting | Input |
| --- | --- | --- | --- | --- |
| Clear front | EcruteakCity | (18, 23) | `2 0 50 0` | None |
| Elevated side | EcruteakCity | (18, 23) | `2 6 50 0` | None |
| Rear | EcruteakCity | (20, 27) | `1 4 35 0` | None |
| Ordinary walking | NewBarkTown | (7, 12) | `3 1 35 0` | `RR` |

The first pair, for example, was captured with these settings; the remaining
pairs use the table's map, position, camera and optional `--walk` input:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 \
CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image CRYSTAL_PREVIEW_MUTE=1 \
CRYSTAL_RENDER_TEST_PARTY_SPECIES=TYRANITAR \
CRYSTAL_RENDER_TEST_PARTY_SHINY=0 \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=1000x700 \
CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=180 \
CRYSTAL_RENDER_TEST_HOLD_BATTLE=0 CRYSTAL_RENDER_TEST_HOLD_MOVE_MENU=0 \
CRYSTAL_RENDER_TEST_SECOND_VIEW=toggle \
CRYSTAL_RENDER_TEST_CAMERA='2 0 50 0' \
target/pokemon-models/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map EcruteakCity --x 18 --y 23 --view both --hour 12 \
  --screenshot target/pokemon-native-review/tyranitar-fidelity-20261021/native/world-front.png
```

The front, elevated side and rear were inspected with the full creature
visible. The walking view establishes ordinary field movement and scale;
a sign overlaps its head projection, so that image is not facial evidence.
Classic field views omit the added companion: the paired screenshots establish
matching game state, while the separate sprite and studio views establish
complete likeness coverage. No source-size override was used.

Capture placement must use actual source coordinates and occupied follower
seeds. World Y is inverted relative to map Y. At this species' scale the
initial follower check spans three source tiles; the apparently clear
Ecruteak player tile (18, 18) puts the third seed on an NPC and omits the
follower. That rejected trial and roof/tree-obscured trials were replaced,
not counted as successful model inspection.

### Initial native battle capture failures

Both Route 29 walking variants and the real Falkner trainer fixture failed
the production scenery witness before model admission. The walking command
was `immersive_battle_3d` with the local pack, `--walking-encounter`,
`--walk` set to twenty repetitions of `RALA`, `--size 1000x700`, and native
and classic screenshot destinations. The first variant used
`CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic` with
`CRYSTAL_RENDER_TEST_WALK_STEPS=1`; the second used ordinary held input with
`CRYSTAL_RENDER_TEST_WALK_STEPS=0`. Both used headless
image capture, Tyranitar, normal palette, `CRYSTAL_RENDER_TEST_HOLD_BATTLE=0`,
`CRYSTAL_RENDER_TEST_HOLD_MOVE_MENU=1` and `CRYSTAL_ENCOUNTER_TRACE=1`.
The trainer attempt used `--falkner-encounter` and sixty `A` presses, with
the same normal admission and size rules.

All three exited with an error after reporting:

```text
anchored=false
accepted=false reason=Some("encounter has no checked terrain anchors")
capture timed out: Some("waiting for an active world frame")
```

The two walking logs show a rendered source revision one behind the minimum
snapshot revision. That is an observation, not an established cause: the
source correspondence code intentionally handles a coherent preceding frame.
The trainer also failed while stationary. No witness, admission, controller,
camera-size or loader rule was bypassed, and no failed battle screenshot is
included as acceptance evidence.

The [retained-terrain follow-up](pokemon-source-frame-fidelity.md) now repairs
the native witness's incorrect centered-padding assumption. Two real Route 29
Tyranitar battles, with paced and held input, have admitted scenery, preserved
2D/3D state and no fallback. Both PNGs and the receipt for each case are
retained; all four original PNGs were inspected. The full body,
supports, low tail and armor are visible at the same 2.0066 m pack size.
The real Falkner case also renders successfully with its fixed Cyndaquil
fixture. That pair verifies the scripted trainer renderer; it is not labeled
as Tyranitar evidence. All 52 native-feature and 47 applicable classic
encounter-frame regressions pass, including the previously failing native
continuous blocked-approach trainer test.

## Review output and cleanup

The ignored review gallery is
`target/pokemon-fidelity-review-20261021-tyranitar/index.html`. It retains
29 selected PNGs, seven paired-state receipts, model/executable identities,
metrics, positive test results and the immutable-baseline negative results.
The selected evidence covers eight studio angles, nine stored pose/angle
views, four production field cases and two admitted Tyranitar walking battles.
The Cyndaquil trainer pair separately checks the repaired renderer. It does
not establish browser, shiny, every animation phase, every map or full-roster
acceptance. Broader roster work remains active.

Disposable game-reference downloads, extracted meshes/textures, intermediate
renders, failed trials and task-local helper binaries are removed after
retaining this gallery. The shared packs and persistent build cache remain.
No content pack was rebuilt, and nothing was staged or committed.
