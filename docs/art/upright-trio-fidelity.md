# Ampharos, Granbull and Mewtwo posture revision

A subsequent [head-volume revision](head-volume-fidelity.md) supersedes the Ampharos/Granbull head and paw geometry, counts and open findings below. This document records the preceding leg/posture batch; its Mewtwo asset remains unchanged by that follow-up.

The October 21 leg recheck covers **all three current models**. Fresh eight-angle
renders confirm upright Ampharos haunches, short upright Granbull shins and
Mewtwo ankles beneath the knees, with toe projection measured separately.
This recheck retains the existing corrected geometry: all 276 canonical GLBs
are byte-identical to the start of the recheck. It does not claim another
sculpting revision. The current Ampharos and Granbull heads are the later
head-volume assets, rather than the older images in the original posture batch.

Forty-five Python checks and nine focused Rust checks pass. Directly shearing
each model's actual leg shafts forward makes its posture guard fail, without
depending on a source digest rejection. Rust again checks two complete walking
supports, floor contact over 64 phases, preserved upper anatomy, and biped
steps centered beneath the hips. Exact decompressed GLB equality and compressed
payload presence were verified in the test executable and both production
examples; their executable hashes remained unchanged through capture.

Nine fresh native/classic pairs cover an unobstructed Ecruteak profile, ordinary
`RRRR` walking and an admitted Route 36 move-menu battle for each species.
All nine receipts preserve state and identify the intended model; all three
battles report no fallback. The profile camera is `2 6 50 0`; walking uses
`2 0 35 0`, with Mewtwo widened to `1 0 35 0` after its tail clipped the frame.
The existing physical pack heights and admission rules remain in effect.
The classic field omits the added follower and the move menu hides the player's
back picture. Retained full game orbits and front/back animation frames supply
those likeness views. Archived before pictures have their original framing
and are labeled separately from the fresh renders. The scoped recheck is at
`target/pokemon-fidelity-review-20261021-upright/index.html`; it does not accept
the rest of the roster. Nothing was staged or committed.

The October 20 review replaced weak original proportions and stiff poses in
these three models. The user's subsequent leg-angle criticism reopened the
first candidates: their feet had moved forward as the legs descended, and
Mewtwo had acquired an exaggerated lower-leg kink and a whole-body shear.
Widening those stances had not corrected their mechanics.

The reference-only Battle Revolution volumes were inspected at all eight
45-degree angles: [Ampharos](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339492/),
[Granbull](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/339520/)
and [Mewtwo](https://models.spriters-resource.com/wii/pokemonbattlerevolution/asset/296506/).
Four captured game-animation frames from each front and back supplied a
separate idle-pose check. Those animations are available from Pokémon
Showdown's [front](https://play.pokemonshowdown.com/sprites/ani/)
and [back](https://play.pokemonshowdown.com/sprites/ani-back/) galleries.
Extracted Ampharos/Mewtwo arm spreads were treated as neutral reference poses,
not the desired battle idle. Granbull's [official artwork](https://www.pokemon.com/us/pokedex/granbull)
also confirms that the large upward lower-jaw points are body-colored;
the small downward fangs are white.

The six actual front/back PNGs in the local browser pack were also read with
Rust's `read_verified_compiled_game_pack` and inspected at nearest-neighbor
enlargement. The first neutral front frames and complete back pictures confirm
the local palettes and provide the production source-picture comparison.

All runtime geometry remains original procedural work. Downloaded meshes and
textures stayed in a disposable external directory and were never inputs to
the authoring recipes or canonical exports.

Ampharos now has a broader pear torso, longer fitted chest panel, three neck
and three tail bands, descending flippers and a thicker, longer tail. Separate
rounded haunches bow outward in front view while remaining upright in profile.
The thinner feet sit beneath the haunches and project forward at the toes.
The former forward-slanting support volumes are gone.

Granbull's arms form a bent guard, its ears fold lower and its large jaw points
use the body palette. Its low hips now cover the leg roots, avoiding exposed
upper-leg caps. Short upright shins descend to flatter paws; forward reach
belongs to the paws rather than diagonal legs. The associated leg pivots and
the continuous bent-arm weight field follow the revised anatomy.

Mewtwo has narrower shoulders, lowered arms and hands, wider haunches and a
long low tail with a sweeping terminal curl. The whole-body forward shear was
removed. Its lower legs descend nearly vertically beneath the knees, and the
long toe pads extend ahead of the ankles. Knee pivots moved with the shaft.

Canonical authoring heights remain 1.35, 1.06 and 1.45 respectively; production
scale still comes from the supplied pack. Existing geometry/loader budgets
are unchanged, including Ampharos's stricter 6,000-triangle authoring budget.

| Model | Vertices | Triangles | Parts | Joints | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Ampharos | 3,822 | 5,868 | 30 | 9 | 204,488 |
| Granbull | 14,203 | 19,048 | 42 | 13 | 583,344 |
| Mewtwo | 15,722 | 27,880 | 52 | 17 | 633,412 |

Before/candidate neutral studio renders use shared orthographic framing and
lighting. All eight final angles were inspected, plus idle 0.25, attack 0.34
and hit 0.18 at quarter and profile views. New posture checks intersect the
stored triangles with several horizontal planes and measure the leg's depth
at those heights. They reject all three pre-correction candidates. Toe reach,
sole thickness, occupied hip/ankle contacts, closed solids, shading, deformation
strain and canonical source/byte parity are checked separately.

Forty-two focused Python checks pass across the final assets. The independent
species geometry and storage validators pass for all three. The other 273
canonical GLBs stayed byte-identical during this revision, including Mareep,
Flaaffy and Snubbull. No pack regeneration or audio changes were required.

Nine Rust checks pass against a test executable verified to contain the exact
current compressed GLB payloads: three family/species GPU checks at pack size,
three authored support/animation checks, the complete schema, the new
three-species walking regression and the expanded step-centering regression.
Walking checks cover 64 phases, two complete support limbs, floor contact,
culling bounds and preservation of upper-body ownership.

Both production examples built successfully, and each executable was verified
to embed all three exact current compressed GLBs before capture. Three headless
Route 29 walking encounters reached the real move menu with the requested
models and `fallback=None`. Wider New Bark walking views and closer field
views were also inspected alongside their classic counterparts. The nine
retained pairs all report `state_preserved=true` and identify the intended
party followers. Battle profiles show the revised legs under the hips;
field views retain the stance and planted support during ordinary movement.

The closer Ampharos/Granbull field sign covers part of a near foot, so those
frames alone do not prove the complete sole. The wider field, battle and
studio views supply that coverage. Mewtwo's rotated close view shows both
lower legs but crops the tail; its wider field and studio orbit show the full
tail. The classic move menu hides the player's back picture, and the classic
field omits the added follower. Those pairs prove state correspondence; the
separately inspected source pictures establish the front/back comparison.

The executed native build command was:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester \
  --example immersive_battle_3d --example render_at_location \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

Captures used `core-modular.browser.crystalpack`, headless image targets and
muted preview audio. Each battle used `--walking-encounter`, ordinary `RALA`
inputs repeated twenty times, `CRYSTAL_RENDER_TEST_HOLD_BATTLE=0` and
`CRYSTAL_RENDER_TEST_HOLD_MOVE_MENU=1`. Each field started at New Bark `(7,12)`
at hour 12 and used ordinary `RR` inputs with both hold flags zero. Field
cameras were `2 1 35 0` and `5 0 20 0`; Mewtwo's retained close angle used
`5 2 20 0`. Input directions are camera-relative, so different camera runs
need not end at the same location. Each individual 2D/3D pair retains its own
identical position, time, facing and game state. An initial field capture
incorrectly inherited the battle move-menu hold; it was discarded and rerun
with that battle-only option disabled.

The reusable `game-model-fidelity-audit` skill now explicitly requires tracing
hips, knees, ankles and soles in profiles. It distinguishes toe projection
from leg slope and rejects generic forward leans or invented crouches. The
skill was applied to this revision and its validator passes.

The retained local review is
`target/pokemon-fidelity-review-20261020-upright/index.html`.
The neutral orbit, sampled clips and native views do not establish every
animation, shiny variant, browser camera or acceptance of the remaining
roster. Ampharos's head still needs a more drooping muzzle silhouette, and
Granbull's cheeks/paws remain smoother than the angular game reference.
Task-owned external reference archives, meshes and textures, draft exports,
raw capture output and temporary verification tools were removed after the
final gallery was preserved. Shared packs and Cargo caches remain untouched.
Nothing was staged or committed.
