# Chikorita family reference and volume revision

The previous Chikorita had a shallow rear barrel, a sideways crown and dark
button eyes. Bayleef's narrow neck, small skull and thin collar leaves did not
match the game volume. Meganium had a flat projecting muzzle, sideways antennae,
narrow flower petals and a descending tail.

## Reference decision and original geometry

The Battle Revolution models for
[Chikorita](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339465/),
[Bayleef](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339466/)
and [Meganium](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339467/)
were inspected through all eight angles. Reference, actual baseline and final
GLBs use matching labeled 4 × 2 contact sheets, an 8-degree elevation and a
shared 2.2 studio camera scale. Studio normalization aids comparison; runtime
physical sizes remain unchanged. Meganium's male reference matches the normal
male starter fixture; female-form visual acceptance is not established here.

The actual provided pack's first four front frames and back picture were
inspected for all three species. Four frames each from the separate Showdown
[front](https://play.pokemonshowdown.com/sprites/ani/meganium.gif) and
[back](https://play.pokemonshowdown.com/sprites/ani-back/meganium.gif) animations
provide supplemental posture and surface evidence; their game generation is
not established here. These sprites confirm pale petal edges and gold stamens.
Opaque black rectangles around the exported game petals are alpha artifacts.

All production meshes remain original procedural geometry. Downloaded game
meshes and textures were inspection references, never exporter inputs or
shipped assets.

- Chikorita has a deeper rear body, broader centered crown leaf, red irises,
  narrow pupils and twelve small fitted nails. Rear foot centers follow the
  fuller barrel. Its leaf rig pivots follow the revised crown.
- Bayleef has a fuller neck and skull, red eyes, eight broad solid folded collar
  leaves and a centered backward crown with its real edge notch preserved.
  A thick rising tail has a buried root and a pivot at that revised root.
- Meganium has a shorter rounded skull, substantial neck, amber eyes and an
  actual backed mouth opening above its independently hinged lower jaw. Thin
  antennae sweep backward from buried skull roots. Six broad folded petals
  carry pink tissue on both faces, pale rims and fitted gold stamens. The tail
  rises from a thick buried root. Skull, jaw, antenna and tail pivots follow
  the revised anatomy.

All three remain quadrupeds with four ground supports. Their complete source
skin hierarchies and source-relative idle, attack and hit clips remain active.

| Species | Vertices / triangles | Logical / packed parts | Joints | Accessors | Bytes | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Chikorita | 5,108 / 3,004 | 35 / 35 | 11 | 209 | 259,736 | 0.889 m |
| Bayleef | 6,800 / 4,360 | 37 / 37 | 16 | 234 | 344,396 | 1.1938 m |
| Meganium | 13,870 / 7,148 | 50 / 45 | 23 | 231 | 587,372 | 1.8034 m |

The actual verified Rust compiled-pack reader reports height digits 211, 311
and 511 respectively. Full authoring heights remain 1.0, 1.15 and 1.48, with
body datums 0.688, 0.982 and 1.322. Geometry budgets, physical sizes and scene
admission tolerances are unchanged.

## Executed authoring checks

All 51 Python checks pass against the final canonical GLBs: 36 existing
species checks, thirteen independent fidelity checks and two shading checks.
Eleven fidelity checks reject the corresponding previous geometry. The root
support and Chikorita surface-fit checks also accept the previous valid roots
and facial fit; they protect the revisions without inventing historical
failures. These checks read stored buffers without depending on the current
authoring digest.

Closed connected solids, actual corner normals, fitted face clearance, planted
soles and claws, skin ownership, binds, clip endpoints, winding, strain,
quantization, container rejection and loader limits retain their strict tests.
The new checks measure barrel depth, centered leaf sections, real collar-leaf
volume, skull/neck depth, rising tail mass, backward antennae, empty mouth
space with occupied backing, complete buried root caps and both petal faces.
Meganium's free lower petal clears the chest by at least 0.00933259 authoring
units across the sampled clips; the existing 0.002 minimum remains intact.

Three issues found during revision are now explicit in the personal audit
skill. Whole-body normalization had shifted the mouth cut below the actual
jaw. Separately evaluating a nonlinear skin field on new clipped petal vertices
buried their fitted layers during motion. Checking only an attachment's center
missed exposed portions of its root cap. The final model uses an actual aligned
mouth recess, one coherent hinge per folded petal and occupied complete root
caps through sampled motion. Clearance rays follow each surface's moving
carrier. Bayleef normalizes negligible normal-axis residues so ordinary Python
and Blender serialize the same buffers.

The actual background Blender builders and standard Python exporters produce
identical bytes. Only these three of the 276 canonical battle assets changed;
the other 273 retain their pre-review hashes. All eight final neutral angles
and 27 decoded pose views were visually inspected: idle 0.25, attack 0.34 and
hit 0.18, each at quarter, profile and rear for every species. These are sampled
animation checks, not every-frame visual acceptance. Chikorita's face is now
clipped through the stored skull facets; projecting only the old artwork
vertices left chords too close to its revised curved skull.

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
  --threads 2 --python-exit-code 1 --python tools/build-actor-props.py -- \
  target/pokemon-native-review/grass-starters-fidelity-20261021/builder-actor \
  --only=battle_chikorita --skip-preview
/Applications/Blender.app/Contents/MacOS/Blender --background --factory-startup \
  --threads 2 --python-exit-code 1 --python tools/build-battle-species.py -- \
  target/pokemon-native-review/grass-starters-fidelity-20261021/builder-species \
  --only bayleef,meganium
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools python3 -m unittest -v \
  test_chikorita_glb test_bayleef_glb test_meganium_glb \
  test_grass_starter_fidelity test_leaf_saur_shading
```

| Species | Logical geometry SHA-256 | GLB SHA-256 |
| --- | --- | --- |
| Chikorita | `4fdcc13e8186096f6148567c2b0649e177d42e9de0823144fc8250e6a4f64620` | `c49db177021bb15f074cf085cf25d3c4e43b55c2ac215b6b4e48c2ffb40df63d` |
| Bayleef | `cd8a83f9c8232478b9f017d3732552858fbde65d4565f5d1b812567a1f254d30` | `dde26a2bd6b865495ee3c3e6a947d8e840737f60f704b1398219fb20d7629551` |
| Meganium | `3a0e3933c4a43a945702297640b63c8f21a8ed0bfca7922cf919209860f06120` | `fbffaa7d38de96f3d55a6bf6a1bc22461eaab82f09db67aa201b60372415f76e` |

## Rust and production verification

Fourteen Rust checks pass against the final embedded assets: the complete
species schema/physical-size check, exact embedded bytes for every model,
grass attack onset, neutral bind/clip endpoints, dense visibility bounds,
both malformed-input rejection suites, four-support walking for the whole
family, three registry/contact checks and three pack-size GPU activation checks.
The actual walking rig keeps four complete supports, preserves upper-body and
botanical bindings, stays inside its locomotion bounds and maintains strict
floor/support contact through 64 gait phases per species.

Meganium's runtime identity pins its actual packed geometry,
`afe7d22e5dcd2293457869130b45048cd542d673ce394f59f02c4713ff23222b`.
The source range metadata retains all fifty logical parts while batching the
six stamens into one draw primitive. No loader limit is increased.

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked --jobs 2 \
  -p crystal-voxel-view --lib --no-run --message-format=json
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example render_at_location --example immersive_battle_3d \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

The final Rust test executable SHA-256 is
`4752a3505d63f533275e037b4ab92a33b7d1df35baa18cd047a7c5744ede5eba`.
The exact final compressed GLBs were located in that executable and in both
rebuilt native previews. Their SHA-256 values are
`416f48758e41349e9c66bf20da1a13c8b7270568fdf18074caeb069130dcbe02`
(`render_at_location`) and
`a3478e14dcc97a5a449ec82de9f79df22f5f434ece144037f53fa82821776c38`
(`immersive_battle_3d`).

Twelve headless native/classic field pairs were inspected at 1000 × 700,
noon and the actual pack sizes. Every receipt preserves authoritative state
and identifies the intended `PartyFollower` model. All cases use Ecruteak City:

| View | Player start | Camera orbit / pitch | Ordinary input |
| --- | --- | --- | --- |
| Front quarter | (18, 23) | `1 35` | None |
| Elevated profile | (18, 23) | `6 50` | None |
| Rear | (20, 27) | `4 35` | None |
| Walking | (18, 23) | `0 35` | `RR` for Chikorita/Bayleef, `RRRR` for Meganium |

Camera zoom is 3 for Chikorita/Bayleef and 2 for Meganium. It changes framing,
never model size. Meganium's larger follow distance left it on the preceding
trail segment after two right presses. Four ordinary presses produce the
selected frame with player and follower both facing right. Receipts record
the actual moved actor centers; captures can be midstep. The Rust gait check
separately covers complete sampled walking cycles.

The classic field omits the presentation follower, so these pairs establish
matching state rather than a second follower likeness image. The pack sprites,
genuine game orbits and complete studio rear/profile views supply that evidence.
The native backs retain readable body mass, tail roots and botanical surfaces.

Three actual Route 29 walking-encounter battle pairs were also inspected,
using the fixture's 16:00 lighting and normal palette. All preserve state,
include the requested species and Sentret as modeled bodies, and report
`source=[]`, `fallback=None` and `accepted=true reason=None`. Meganium's
longer silhouette rejects some proposed camera shots; the existing production
search selects an admitted shot with the complete animated body inside the
checked retained neighborhood. No scale, witness or admission rule is waived.
The move menu hides the classic player's back picture, so that capture does
not establish its likeness on its own.

For the front-quarter field pair, set `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`, `CRYSTAL_PREVIEW_MUTE=1`,
`CRYSTAL_RENDER_TEST_PARTY_SPECIES=CHIKORITA`,
`CRYSTAL_RENDER_TEST_PARTY_SHINY=0`,
`CRYSTAL_RENDER_TEST_WINDOW_SIZE=1000x700`,
`CRYSTAL_RENDER_TEST_SECOND_VIEW=toggle`,
`CRYSTAL_RENDER_TEST_HOLD_MOVE_MENU=0` and
`CRYSTAL_RENDER_TEST_CAMERA='3 1 35 0'`, then run:

```sh
target/pokemon-models/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map EcruteakCity --x 18 --y 23 --view both --hour 12 \
  --screenshot target/pokemon-native-review/grass-starter-field.png
```

Walking field captures use `--walk RR` or `--walk RRRR`,
`CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic` and
`CRYSTAL_RENDER_TEST_WALK_STEPS=1`. Scope the move-menu hold to battle review:
setting it for an ordinary field walk makes the reviewer wait for FIGHT there.
The battle captures use `immersive_battle_3d` with the same completed pack,
`--walking-encounter`, twenty `RALA` repetitions for `--walk`, `--size 1000x700`,
`--screenshot` and `--second` paths, the intended party override, synthetic
step input, `CRYSTAL_RENDER_TEST_HOLD_BATTLE=0`,
`CRYSTAL_RENDER_TEST_HOLD_MOVE_MENU=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_SOURCE_FALLBACK=0` and
`CRYSTAL_ENCOUNTER_TRACE=1`.

The selected ignored gallery is
`target/pokemon-fidelity-review-20261021-grass-starters/index.html`. It retains
47 PNGs, fifteen paired-state receipts and concise source/identity/admission
evidence. Reference archives, downloaded meshes/textures, draft exports,
unselected captures and temporary helpers are removed; shared packs and build
caches remain. Nothing is staged or committed. Full-roster, shiny, female-form,
browser and every-frame visual acceptance remain outside this batch's coverage.
