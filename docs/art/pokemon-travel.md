# Pokémon travel

`pokemon_travel.rs` assigns a deliberate travel style to all 251 species and
all Unown forms. Unknown identities cannot silently inherit a human walk.
These rigs change presentation; controller position, collisions, scripts,
battle state, and audio remain authoritative and unchanged.

## Land and air motion

The registry distinguishes planted steps, heavy steps, scurries, diagonal
trots, paired-leg bounds, toad bounds, hops, staggered egg hops, waddles,
crawling, scuttling, serpentine waves, belly sliding, burrowing, rolling,
spinning, swimming, fish flops, hovering, flight, and gliding.

- Exeggcute's complete egg assemblies hop at staggered phases.
- Bulbasaur and Ivysaur pair a hind-leg push with a foreleg catch. Venusaur
  uses a smaller rise suited to its plant and low body.
- Quadruped bounds differ from diagonal trots. Doduo and Dodrio use longer,
  quicker steps; heavy animals use shorter strides and longer stance.
- The authored upright Meowth and Raticate bodies use biped supports.
- Furret's new world clone reposes its body horizontally, levels the head,
  lowers the long tail, and fits all four complete paws to the substrate.
  The shared quadruped IK supplies diagonal travelling steps. Its pack
  dimension follows nose-to-tail length instead of the new shoulder height.
  The compiled front/side studio views show the low posture and forward gaze;
  production side views and the standing-view spacing regression pass.
  The canonical upright battle sculpture is retained.
- Crab and insect pincers sweep sideways. Parasect no longer raises its
  entire body to clear a downward pincer swing.
- Ekans, Arbok, Dratini, Dragonair, and Dunsparce receive blended joints in
  their lower bodies. Onix and Steelix retain rigid stone/armor assemblies
  with successive lateral offsets. Original coiled silhouettes are retained.
- Voltorb/Electrode rolling and Hitmontop spinning follow accumulated travel;
  their orientation is retained when motion stops.
- Diglett peeks above rooted soil; Dugtrio's bodies peek at staggered phases.
- Magikarp rests on its side on land, flexing its tail and pectoral fins
  between short hops. Goldeen, Seaking, and Remoraid now use that stranded-fish
  motion too: their opened dry-ground renders showed unsupported air swimming.
  Fresh front/side studio views show flank contact and a separate hop gap.
  The complete posed surface is grounded before adding the hop arc; explicit water sampling
  bypasses the flop and canonical battle presentation remains unchanged.

Steps and waves follow distance phase with style-specific cadence limits.
Hops include a grounded interval and smooth takeoff/landing. Ground settling
checks actual posed surfaces; rolling checks every vertex. World rigs preserve
canonical battle assets and clips. Furret deliberately reposes only its
world clone; other existing body vertices retain their canonical rest pose.

Seventeen bird/dragon fliers use their measured three-dimensional wing span
to centre a complete world-flight stroke. The correction is expressed in the
shoulder parent's frame, so folded wings can extend above and below their
shoulders. The species are Pidgey, Pidgeotto, Pidgeot, Spearow, Fearow, Noctowl,
Murkrow, Skarmory, Articuno, Zapdos, Moltres, Ho-Oh, Lugia, Togetic,
Aerodactyl, Charizard, and Dragonite. Flying rigs target an air gap beneath
the lowest animated surface. Existing bat and other insect motion is retained.

Ledyba's canonical asset has closed cases and no underwings. Its world rig
adds two shallow flight membranes; Ledian retains its authored membranes.
Both open their cases and use an insect cadence. The two intentional case
cuts have separate welding domains, while paint/normal splits within a case
remain connected. Ordinary living actor meshes use the selected world rig's
geometry, so added membranes and skin attributes have matching vertex counts.
Canonical battle bodies are unchanged.

## Water motion

Actual Surf mounts and living intrinsic aquatic actors on published water
cells use `SpeciesRig::sample_swimming`. Terrain selection checks tile identity
and resolved support: raised bridges/tables and unknown terrain do not imply
water. Dolls remain inert. Nonaquatic fliers and ghosts keep their air motion.
No map positions or movement permissions are changed by this decision.

Swimming bypasses land IK, planting, hover clearance, and fish flops. Distance
phase drives tails, fins, flippers, and arms. Legs are stroked separately
because the land limb registry deliberately excludes them. Top-level leg
roots, including generated world hips, supply alternating kicks. Two-legged
paddlers lean into propulsion; their heads counter the pitch to keep a forward
gaze. Resting posture returns as propulsion stops.

Gyarados's world body uses twelve blended strips, with a delayed lateral wave
continuing into its neck and tail branches. It retains a partly upright coil.
Surf seat fitting recognizes the additional body influences; the rider follows
the animated skin contact point. Waterline and rider placement were reviewed
in paired production scenes and moving frames, including Gyarados and Vaporeon.

The existing body-local Surf seat/waterline anchors mount and rider. A disposable
Surf capture fixture accepts `CRYSTAL_RENDER_TEST_SURF_SPECIES` (default
`TOTODILE`), teaches Surf through the runtime API, and waits for ordinary A/Yes
mounting on Route44. It does not write a user save.

## Current verification status

The last fully passing motion suite ran **90 tests** (two explicit reviewers
ignored). It includes Furret's four supports and diagonal pairing, continuous
seal belly contact, slug deformation, water tentacle strokes, and four stranded
fish. It also includes Lapras's grounded land pull. This pass includes the final
fish appendage folds, upward tail curl, and grounded moving-body clearance checks. Source/model timestamps were checked
against the compiled review artifact before the focused actor tests and captures.

- Furret's first lowered render exposed a downward gaze, and a regression found
  only two support groups. Complete haunch/paw binding and a levelled head fixed
  those defects. The corrected front/side GPU sheets were opened: the body is
  low, the gaze is forward, the tail stays attached, and paw poses alternate.
  Actor asset-sharing and culling-bound regressions passed. Follower spacing
  now shares the world reference measurement and caches an envelope of the
  lowered gait, instead of measuring the upright battle model. Its standing
  view regression now includes Furret and passes. Production side views and
  a moving frame were opened and show the low body clear of the trainer.
- Seel/Dewgong now pull with complete fans and their folds, with delayed trunk
  and tail flex. The belly is anchored first; paddles and a low muzzle clear
  the substrate independently. The strict 64-phase belly-contact check passes,
  and corrected front/side sheets were opened. This avoids both the old rigid
  slide and transferring the whole animal's support to its head or a paddle.
- Slugma/Magcargo's soft foot wave keeps shell/neck/eyes on authored owners.
  Corrected front/side sheets were opened and contact/stopping tests pass.
- Octillery, Tentacool, Tentacruel, Omanyte, and Omastar have explicit free-water
  tentacle strokes; the jellies pulse their bells. Their opened front/side
  swimming sheets show articulation. Grounded support remains a separate path.
- Goldeen, Seaking, and Remoraid now share Magikarp's stranded-fish motion.
  Closer front/side inspection rejected poses that propped the fish on a fin.
  The latest world rig uses a full side roll and measured trailing folds for
  complete pectoral/pelvic fins and Magikarp's barbels. Goldeen/Seaking's broad
  tail lobes gain separate hinges so each can lie flat. Authored ray/paint
  collections follow their complete fin. Water sampling bypasses these folds,
  and the canonical battle asset is unchanged. The fold revision passed 89
  motion tests and its front/side images were opened, but moving grounded
  poses still levered the body onto the tail. The latest tail curl stays away
  from the substrate, and membrane folds counter the body rock. The regression
  now checks body clearance throughout the grounded portion as well as at
  rest. All 89 motion tests pass with this correction. All eight latest land
  front/side sheets were opened and show flank contact in the sampled grounded
  poses, followed by a distinct airborne gap. All eight fish water sheets were
  also opened: upright fins unfold and tails remain attached. A grounded fin
  alone is not acceptance.

Eight-phase reviews of Quilava, Rhyhorn, and Slowpoke show alternating short
steps that were hard to distinguish in four small GPU frames. All 82 fresh quadruped front/side sheets were opened to review the retained
anatomical front/rear grouping. Full-size Piloswine, Swinub, and Quilava side
views resolve small alternate foot lifts that are faint in contact sheets.
The elevated-water selector regression also passes, but its corrected native
Lake of Rage capture has now been opened and shows water contact.

## Verified evidence and limits

The initial complete motion pass ran 77 tests, checked 277 roster/form entries
over 64 geometry phases, and opened 276 GPU captures covering the registered
species and Unown forms. Six adjusted species were captured again. A subsequent
neutral audit resolved all 276 distinct registered assets (271 battle-species
files and five extra bodies), opened twelve alphabetical contact sheets and
the extra-body sheet, and found no obvious missing body part or broken neutral
silhouette. This does not prove every-angle anatomy or natural motion.

The latest completed motion build passed **90 tests**, including Lapras land
contact, final fish folds and grounded tail correction, with two explicit review
utilities ignored. It included Furret, strict seal contact, slug/tentacle checks, full flight correction, swimming feet,
ladybird case separation, grounding, loop/blend, bounds, and seam checks.
A verification runner checked source and model timestamps against that compiled
artifact and separately executed seven actor/surf tests and all three follower
tests; all passed. They include:

- `ledyba_world_actor_publishes_its_flight_membranes_with_matching_skin`
- `aquatic_presentation_requires_real_water_and_unraised_support`
- `magikarp_switches_between_land_flop_and_water_swim_without_moving_gameplay`
- `surf_seats_use_body_surfaces_and_follow_the_animated_skin`
- `surf_riders_stay_on_the_animated_mount_through_turns_and_dismount`

The water-transition test proves that Magikarp changes presentation without
moving authoritative gameplay. Rider tests establish animated contact-point
behavior numerically; they are not visual proof of immersion.

All 34 front/side sheets for the seventeen corrected fliers were opened.
Distinct up/down strokes and downstroke clearance are visible. Murkrow retains
compact wing geometry. Side views make many broad wings edge-on, so the front
views provide stronger shape evidence. Fresh front/side/rear ladybird sheets
were also opened: the first rear views were rejected because seam welding
stretched the cases into a joined cap. After separating hinge ownership, the
new rear views show two cases and an open lower gap with attached membranes.

Swimming front/side sheets were opened for Magikarp, Squirtle, Totodile,
Quagsire, Lapras, Vaporeon, and Gyarados. Foot phases differ; paddlers keep a
forward gaze; Lapras retains a level body. Earlier downward-looking head poses
were rejected and corrected. These studio floors are deliberately lowered to
show the feet and are not production water surfaces. Fresh front/side water
sheets for Seel, Dewgong, Totodile, Vaporeon, Gyarados and the four stranded
fish were also opened after the final motion pass. The sampled seals retain
attached paddles; Totodile/Vaporeon show separate kicks, and Gyarados changes
its body curve. Actual water placement remains a production check. Gyarados's opened wave
sheets showed no detached fins or open belly seams in sampled views; their
zero detected seam-pair count is not independent proof of seam integrity.

Additional full motion sheets were opened across walkers, crawlers, scuttlers,
hoppers, rollers, swimmers, and fliers. They include the alphabetical groups
Abra–Bellossom, Clefairy–Ditto, Donphan–Entei, Espeon–Geodude,
Girafarig–Heracross, Hitmonchan–Jolteon, Jynx–Larvitar, Lickitung–Marill,
Marowak–Nidoking, Nidoqueen–Paras, and Parasect–Poliwrath, and Ponyta–Zubat, excluding species already reviewed in
focused flight/water passes. Explicit bat/insect extrema were opened for
Zubat, Golbat, Crobat, Butterfree, Celebi, Ledyba, and Ledian. Inconclusive
small-amplitude poses prompted longer phase reviews, not acceptance.

The Parasect–Poliwrath sheets retain low scuttle supports, diagonal feline
steps, and distinct hops. Piloswine's feet remain partly hidden by its coat.
A subsequent eight-phase sheet and a larger side GPU sheet were opened;
short alternate feet are visible beneath the hem. The external pinned source
sprite was also opened and confirms the intentionally hidden eyes/legs and
long coat. That supports retaining its silhouette, without claiming every
frame or production acceptance.

Apparent omissions were checked from another angle: Mewtwo's attached curled
tail was hidden by its front view, and Girafarig's tail eyes/mouth were visible
in a separate rear capture. Neither observation justified speculative geometry
changes. The Mr. Mime review selector was corrected to the registry spelling
`MR__MIME` before rendering the remainder of that group.

## Production scene review

Earlier muted, headless Exeggcute and Voltorb captures at Ecruteak `(18,21)`,
noon, paired 3D with the same authoritative 2D state and separately opened an
actual moving frame. Their 32-input records traversed `(18,21)`–`(18,25)`.
The sampled bodies remained readable and grounded.

Onix initially failed publication on short or obstructed seed paths. A
National Park diagnostic had no actual `PartyFollower`; player movement and a
successful screenshot were not treated as proof the model appeared. At its
original pack size, Onix requires a 630.110-pixel history trail. Goldenrod
`(18,26)` published it but obscured its face. Ecruteak `(17,26)` provided the
successful review: paired final views and a moving frame showed the whole rock
silhouette, face, horn, and low tail. The record traversed `(17,26)`–`(17,30)`
and identified `species:ONIX:icon_monster` as the actual follower. No shrinking
or invented trail was used. Eight follower regressions passed.

Paired capture receipts include actual `world_actor_sources`, while
`actor_sources` describes the authoritative comparison baseline. These help
distinguish missing publication from framing, but require opening the image.
`CRYSTAL_POKEMON_FOLLOW_TRACE=1` is opt-in and silent by default.

The latest native capture tools built successfully, and task module timestamps
were checked against their compilation fingerprint. Paired production scenes
were opened at matching state/time/facing. Ecruteak `(17,26)` followed four
ordinary downward inputs to `(17,30)` with a side camera (`2 2 35`): Furret is
low and readable at its pack size; Magikarp lies on its flank clear of the
trainer; Pidgeot has distinct wing poses and a shadow gap; Ledyba publishes its
pale membrane and stays airborne. Final paired views and actual moving frames
were opened for all four. The front camera's previous obscured fish and upright
Furret were not accepted.

The corrected Lake of Rage view from `(20,27)`, noon, camera `0 0 35`, shows
red Gyarados meeting the lake surface. Its opt-in placement trace records
`swimming=true`, support and native water datum both `21.087465`, and immersion
`17.091597`. The old positive-height water rejection is fixed. Both actual 2D
and 3D views were opened after the fix.

Route44 Surf fixtures mounted actual Totodile, Vaporeon, and Gyarados through
ordinary A/Yes input. High (`3 6 50`) and low side (`3 4 18`) final and moving
3D views were opened, together with matching 2D baselines. The rider follows
the carrying surface through turns. Vaporeon's lower body and legs are hidden
by water while head and tail remain readable. Gyarados retains its partly
upright coil, with the low back carrying the rider. Totodile's small pack size
means the seated trainer obscures much of it; its studio strokes provide the
separate articulation evidence. These observations do not claim every limb is
visible through an opaque surface or rider.

A production tentacle review opened Octillery, Tentacruel, and Omastar's final
and moving 3D frames. **Tentacruel was rejected**: its rider sat low among the
tendrils and too much of the creature remained above water. The fitter omitted
its bell and selected pinned root-bound tips instead. Tentacool/Tentacruel now
use their actual bell triangles for rider contact and bell bounds for waterline.
The new 64-phase regression passes upper-bell contact, lower-bell immersion,
and submerged-tendril checks for both species. The existing seat test and rider
turn/dismount test also pass, and that artifact passes all 89 motion tests
(two review utilities ignored). A follow-up anatomy audit found Octillery's
upper body uses a `mantle` joint: the old fitter only considered rooted arm
bases below it. Its opened low carrying pose is also rejected. The fitter now
recognizes its mantle, with a regression for upper-body contact and submerged
arm bases. A 47-species aquatic ownership audit also flagged Staryu/Starmie.
Their existing production final/moving views were opened and rejected: Staryu's
rider sat beside its body, while Starmie intersected the torso. Their water-only
sampler now lays the broad star face upward and centres it on the actor anchor.
The fitter measures buoyancy from that same posed body and places the rider on
the central hard core; upright land/battle poses remain separate. A 64-phase
regression checks the face-up silhouette, immersion, and above-water contact.
All four focused Surf tests pass, followed by all 89 motion tests (two review
utilities ignored), six actor checks and three follower checks. Source timestamps
match that tested artifact. Fresh star front/side GPU sheets were opened: broad
face-up bodies, flexing points, and attached cores are visible. The native rebuild passed with matching task source timestamps. Fresh matched
2D/3D production views and moving frames were opened for Tentacool, Tentacruel,
Octillery, Staryu and Starmie: riders stay above their carrying surfaces, lower
bodies meet the water, and both flat stars remain readable while turning.
The rejected low seats and torso intersections are corrected.

A wider dry-ground Lapras capture at Ecruteak `(17,26)`–`(17,30)`, camera
`0 2 35`, exposed an air gap beneath its belly. This was rejected. Its world
land path now uses complete flipper pulls with a grounded cream belly, while
explicit water sampling retains free swimming. A new 64-phase regression
checks contact, nonpenetration, full-flipper motion, stopping and separate water
strokes. All 90 motion tests pass, as do the four Surf seat tests, rider-turn
regression and three follower tests. Fresh front/side land and swimming sheets
were opened: the belly stays on the land plane, complete flippers pull, and
water strokes stay free. The native rebuild passed and includes all final task source timestamps.
Fresh matched production views and actual moving frames were opened at
Ecruteak `(17,26)`–`(17,30)`, camera `0 2 35`, and Route44 Surf, camera
`1 6 35`. The cream belly meets the path; in water the lower body is submerged
and the rider stays on the shell through the turn. Both paired receipts preserve
state and publish actual Lapras. This closes the pending visual check.

## Running verification

Use one Cargo build per cache. The currently used incremental review profile is:

```sh
CARGO_TARGET_DIR=target/human-walk CARGO_BUILD_JOBS=1 \
  cargo test --locked -p crystal-voxel-view --lib species_rig::motion \
  --config 'profile.dev.package.crystal-voxel-view.opt-level=1' \
  --config 'profile.dev.package.crystal-voxel-view.debug=0' \
  --config 'profile.dev.package.crystal-voxel-view.incremental=true'
```

Explicit review utilities are `pokemon_motion_geometry_review` and
`pokemon_motion_gpu_review`; invoke them with `--ignored`. Geometry review
samples eight travel poses from two angles plus two battle poses. GPU review
shows canonical rest, two travel poses, and a canonical attack. Its controls are:

- `CRYSTAL_POKEMON_REVIEW_DIR`: ignored output directory.
- `CRYSTAL_POKEMON_REVIEW_SPECIES`: comma-separated registry names.
- `CRYSTAL_POKEMON_REVIEW_YAW`: angle in radians (default `0.45`).
- `CRYSTAL_POKEMON_REVIEW_WING_EXTREMES=1`: opposite actual wingbeat extrema.
- `CRYSTAL_POKEMON_REVIEW_SWIMMING=1`: opposite water stroke/kick phases.

GPU/native captures must use `CRYSTAL_RENDER_TEST_HEADLESS=1` and
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`. Review images, CSV files, logs,
and temporary drivers are ignored scratch output, not shipped deliverables.
Open rendered results before accepting composition, anatomy, or motion.

The production capture tools were built and run with the local core pack:

```sh
CARGO_TARGET_DIR=target/human-walk CARGO_BUILD_JOBS=1 \
  cargo build --locked -p crystal-bevy --features location-tester \
  --example render_at_location --example immersive_battle_3d \
  --config 'profile.dev.package.crystal-voxel-view.opt-level=1' \
  --config 'profile.dev.package.crystal-voxel-view.debug=0' \
  --config 'profile.dev.package.crystal-voxel-view.incremental=true'

CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
  CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_CAMERA='0 2 35' \
  CRYSTAL_RENDER_TEST_WINDOW_SIZE=1280x960 \
  CRYSTAL_RENDER_TEST_PARTY_SPECIES=LAPRAS CRYSTAL_RENDER_TEST_PARTY_SIZE=1 \
  CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic CRYSTAL_RENDER_TEST_WALK_STEPS=1 \
  target/human-walk/debug/examples/render_at_location \
  --pack content-packs/core-modular.crystalpack --map EcruteakCity \
  --x 17 --y 26 --hour 12 --view both --walk DDDD \
  --screenshot target/lapras-land.png

CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
  CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_CAMERA='1 6 35' \
  CRYSTAL_RENDER_TEST_WINDOW_SIZE=1280x960 \
  CRYSTAL_RENDER_TEST_SURF_SPECIES=LAPRAS \
  CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic CRYSTAL_RENDER_TEST_WALK_STEPS=1 \
  target/human-walk/debug/examples/immersive_battle_3d \
  content-packs/core-modular.crystalpack --surf-encounter \
  --screenshot target/lapras-surf-3d.png --second target/lapras-surf-2d.png \
  --walk AAAAAAAAAAAAADDL --size 1280x960
```

Capture generation is never visual acceptance: open both views and the moving
frames. Pack files and build caches remain ignored; review scratch is removed
after inspection.
