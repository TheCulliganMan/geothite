# Pokémon travel

`pokemon_travel.rs` assigns a travel style explicitly to every species. The
registry includes the 251-species roster and all Unown forms; an unknown identity
cannot silently inherit a human walk. This is renderer animation: controller
position, collision, scripts, battle state, and audio are unchanged.

The styles distinguish planted biped steps, heavy steps, quick scurries,
diagonal trots, paired-leg bounds, toad bounds, hops, staggered egg hops,
waddles, crawling, scuttling, serpentine waves, belly sliding, burrowing,
rolling, spinning, swimming, grounded fish flops, hovering, flight, and gliding.

- Exeggcute's six complete egg assemblies hop at staggered phases.
- Bulbasaur and Ivysaur pair their hind-leg push and foreleg catch. Venusaur
  uses a smaller rise suited to its large plant and low body.
- Ekans, Arbok, Dratini, Dragonair, and Dunsparce receive blended joints in
  their fixed lower bodies. A lateral wave continues through the upper body;
  the original coiled geometry is retained. Onix and Steelix retain rigid rock
  and armor assemblies with successive lateral offsets along their chains.
- Quadruped bounds differ from diagonal trots; Doduo and Dodrio use longer,
  quicker running steps. Heavy animals use shorter strides and longer stance.
  The upright Meowth, Furret, and Raticate skins use biped hip/foot placement;
  their classification does not assume four ground supports from their names.
- Crab and insect pincers sweep sideways rather than pitching below the feet.
  In particular, Parasect no longer raises its entire body to clear swinging
  pincers.
- Voltorb's and Electrode's rolling angle and Hitmontop's spinning angle follow
  accumulated travel and remain at their current orientation when stopped.
- Diglett peeks above its rooted soil; Dugtrio's three bodies peek at staggered
  phases without lifting the earth mound.
- Magikarp travels on land on its side, flexing its tail and pectoral fins
  between short hops. Its entire posed surface is grounded before the hop arc
  is applied; its authored battle presentation is preserved.
- Swimming adds fin, flipper, and tail strokes. Existing authored wing beats,
  larval body waves, tentacle strokes, and battle clips remain in use.

Steps and travelling waves follow distance phase, with style-specific cadence
limits. Hops have a grounded interval and smooth takeoff/landing. Ground
settling checks the posed skin; rolling checks the whole surface because any
side can become its lowest point. Canonical battle meshes and bindings are
not replaced by the world-only travel rigs.

## Verification

```sh
CARGO_TARGET_DIR=target/human-walk cargo test --locked \
  -p crystal-voxel-view --lib species_rig::motion

CRYSTAL_POKEMON_REVIEW_DIR="$PWD/target/pokemon-motion-review" \
CRYSTAL_POKEMON_REVIEW_ALL=1 CARGO_TARGET_DIR=target/human-walk \
  cargo test --locked -p crystal-voxel-view --lib \
  pokemon_motion_geometry_review -- --ignored

CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_POKEMON_REVIEW_DIR="$PWD/target/pokemon-motion-gpu" \
CRYSTAL_POKEMON_REVIEW_SPECIES=EXEGGCUTE,BULBASAUR,EKANS,ONIX,VOLTORB,HITMONTOP \
CARGO_TARGET_DIR=target/human-walk cargo test --locked \
  -p crystal-voxel-view --lib pokemon_motion_gpu_review -- --ignored
```

The motion tests check support lengths, level feet, takeoff and landing,
independent egg hops, nonrigid serpent movement, stopped rolling orientation,
loop continuity, movement blending, grounding, bounds, and connected surfaces.
Geometry sheets include eight travel phases from two angles and two battle
poses. GPU sheets show the canonical rest pose, two travel phases, and an
authored attack. These are studio checks, not a substitute for a matched-state
2D/3D gameplay review. Review images and CSV output are ignored scratch files.

The completed pass ran 77 motion tests successfully, checked 277 roster/form
entries over 64 geometry phases with no detected penetration or split seams,
and visually inspected 276 GPU captures covering every species and Unown form.
The six species adjusted after that inspection were captured and inspected
again. A matched-state 2D/3D gameplay review was not performed in this pass.

## Follow-up field review

Exeggcute and Voltorb were subsequently inspected in muted headless production
captures at Ecruteak `(18,21)`, noon, using their normal appearances and original
pack sizes. Each final 3D view was paired with the same authoritative 2D state;
an actual moving frame was inspected separately. Both retained readable whole
bodies and ground contact in the sampled field poses. Each movement record has
32 synthetic-input samples over `(18,21)` through `(18,25)`. The focused hop,
serpent, and stopped rolling/spinning regressions also passed. These samples
extend the studio review; they do not establish every-frame field acceptance.

Onix's initial short town route did not show
the companion; a longer route reached an NPC, and a park route reached a fence.
Those captures are not visual passes. A successful map capture and player
movement alone do not prove that the target model was presented.
The subsequent National Park `(20,30)` diagnostic recorded 32 inputs and reached
`(20,33)`. Its actual 3D actor list contains no `PartyFollower`, establishing
that the missing Onix in that scene was a publication/placement condition rather
than camera framing. The studio serpent regression passed separately.

The subsequent investigation found valid collision blockers in the attempted
seed paths. Onix requires a 630.110-pixel trail at its original pack dimensions.
Goldenrod `(18,26)` provides a long clear seed strip and successfully publishes
the companion, although buildings obscure its face. Ecruteak `(17,26)` provides
the better review position. Its final paired view and actual moving frame were
inspected: the complete rock silhouette, face, forehead horn, and low tail are
visible, with the tail meeting the plaza. The 32-input record traverses
`(17,26)` through `(17,30)`, preserves the paired game state, and includes
`species:ONIX:icon_monster` as the actual `PartyFollower`. The current canonical
GLB was verified inside the unchanged capture binary. Size, rig, and collision
rules were preserved; no model shrink or invented trail was necessary.

Eight follower regressions passed, covering corners, blocked placement, large
body spacing, fractional gaps at rest, bounded history, and preservation of
authoritative gameplay. Optional `CRYSTAL_POKEMON_FOLLOW_TRACE=1` now reports
changes between waiting for history, frame-loading failures, and successful
publication. It stays silent by default and retains only its last reason.
Both reviewed Onix pairs were muted and headless; shiny checks were skipped.

Paired capture receipts now include `world_actor_sources` from the actual 3D
readback request, including presentation-only followers. The existing
`actor_sources` still describes the authoritative comparison baseline, which
deliberately excludes followers. Reviewers can distinguish a missing published
companion from one outside the camera view before changing model scale or rigs.
Actor publication evidence still requires visual inspection of the target.

## Magikarp land travel

The grounded flop follow-up passed all 78 motion tests (two explicit review
utilities remain ignored in the normal suite). A separate 64-phase geometry
review detected no floor penetration, and the offscreen GPU sheet was visually
inspected for its grounded and airborne poses. These are studio checks; a
matched-state gameplay review and water-specific locomotion remain outstanding.

## Water travel

Surf mounts now use `SpeciesRig::sample_swimming` instead of replaying only an
idle clip. Distance phase drives tail, fin, flipper, and limb strokes, blended
by the same movement weight as land travel. Water sampling bypasses foot
planting, ground settling, hover clearance, and Magikarp's land flop. The
existing body-local Surf seat and waterline still anchor mount and rider.
Authored idle and battle clips are retained. This applies to actual Surf
mounts; terrain selection for other world Pokémon is separate work.

The offscreen GPU review accepts `CRYSTAL_POKEMON_REVIEW_SWIMMING=1` to show
swimming poses in its two travel columns, retaining the neutral and battle
columns as references.

The water-travel follow-up passed 79 motion tests and the explicitly invoked
`surf_riders_stay_on_the_animated_mount_through_turns_and_dismount` regression.
Eight offscreen studio sheets were inspected: Magikarp, Goldeen, Gyarados,
Lapras, Seel, Dewgong, Squirtle, and Totodile. They establish stroke sampling
and separation from land motion, not final swimming acceptance in a water
scene. Upright paddlers retain their authored torso poses, and Gyarados still
needs a travelling body wave beyond its tail stroke. Neither broader model
fidelity nor every-species gameplay visual acceptance is established here.

## Gyarados body-wave follow-up

The world rig now blends the fixed body through twelve extra joints, with a
phase-delayed lateral wave continuing into the existing neck and tail branches.
The canonical mesh and battle skeleton remain unchanged. Surf seat fitting
recognizes these body influences, preserving the neutral seat and waterline;
the rider follows the resulting deformed skin.

All 80 motion tests passed. The animated-seat and production Surf turn/dismount
regressions also passed. The rendered GPU sheet and the geometry phase sheet
were opened and visually inspected, including front and side phases. The coil
bends and the neck shifts laterally without visible opened belly seams,
detached fins, or a disconnected jaw in those sampled views. The geometry
review detected no floor penetration. Its seam-pair count is zero, so it is
not independent numerical proof of seam integrity.

The upright coiled silhouette remains; a horizontal swimming posture and a
matched-state production water scene have not been accepted. This follow-up
is partial swimming improvement, not acceptance of every model or animation.

## Forward swimming posture

Two-legged world rigs now lean into swimming propulsion, with rounded arm
strokes and alternating leg kicks. Gyarados uses a smaller body lean. Head
joints counter the body pitch so the face continues looking forward. Land
sampling and canonical battle clips are unchanged.

The first front/side renders were opened and rejected because the head followed
the body pitch and looked downward. After the head correction, all ten new
front/side sheets for Squirtle, Totodile, Blastoise, Feraligatr, and Gyarados
were opened and visually inspected. The sampled faces remain forward while
the torso leans; no opened neck seam was visible in those views. These are
studio views with a ground plane, not a production water scene. They do not
establish water immersion or every-frame visual acceptance.

The corrected build passed all 81 motion tests, plus the animated Surf-seat
and production Surf turn/dismount regressions. Upright resting poses remain
when propulsion stops; the land and water paths remain separate.

## Fresh neutral roster inspection (in progress)

The explicit model paths in the current rig registry resolve to 276 distinct
assets: 271 files in `models/battle_species` and five in `models/actor_props`.
Every referenced file exists, and the battle-species folder has no additional
model outside that path set. The extra bodies are Chikorita, Cyndaquil,
Totodile, Clefairy, and Jigglypuff.

The first five alphabetical contact sheets (Abra through Mantine, 120 files)
and the separate five-body sheet were opened and inspected. This pass has not
found an obvious absent body part or broken silhouette in those neutral views.
It does not establish animation, every-angle anatomy, production lighting,
water immersion, or visual acceptance of the uninspected remainder. The full
neutral render and fresh production capture build are still running.
