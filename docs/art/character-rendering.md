# Required character rendering

The 3D world uses `open_people_skin::PeopleGpu` for every resolved human source,
indoors and outdoors, including remote players and bicycle riders. Each actor
instance owns a weighted `Person`; there is no optional legacy character field,
rigid joint renderer, or embedded human catalog.

Native applications load the shared artwork from `assets/open-models` by
default. `CRYSTAL_OPEN_MODEL_ROOT` overrides that directory. The required
`human-male` and `human-female` rigs must be present and valid. Optional unused
rig variants may be absent. UV paintings use the same default root, so the
production client and character reviewer load the same paintings.

Browser startup requires a complete production `open-people.json` bundle.
Missing downloads, timeouts, malformed bundles, or missing production rigs stop
startup. The web build script always bundles character artwork and does not
silently create a preview without it. Scenery remains independently optional.

The retired `catalog.glb`, rigid human decoder and animation implementation,
character generator, human GLB exporter, geometry compactor, and legacy
`characters.blend` source have been removed. Static bicycle files containing
baked old riders have also been removed. Bicycle frames are native Rust meshes;
riders use the same weighted character skin as walking characters. Their seated
pose has not yet received the full costume/limb collision and multi-angle
review required by the broader character quality work.

This removal does not mean every character is artistically finished. The UV
painting and anatomy review remains ongoing; see [character review](character-review.md).
It does not change gameplay, content packs, or audio.

Overworld 3D bodies are rooted at the center of the gameplay tile. The 2D
sprite bottom remains the source presentation anchor; placing a 3D body's
feet there would shift it half a tile south and leave the final map row
overhanging the floor. Terrain support, contact effects, first-person cameras
and cutaways use the centered ground point. Source sprites and map bounds
remain unchanged.

## Human locomotion

Shared arm and leg surfaces receive two connected refinement passes before
costumes are fitted. The neck, torso, hands and ankle/sole boundaries remain
pinned; UV islands stay separate and new vertices interpolate the existing
skin weights. This preparation is cached per supplied body rig, with no
per-frame CPU mesh deformation or additional artwork catalog.

`human_locomotion` supplies distinct distance-driven walk and run poses to the
shared weighted renderer. Each stance foot travels backward at the actor's
published speed; a smooth lifted return, forward knee bend, restrained pelvis
motion and opposing arm swing complete the cycle. Stride scales with character
height. The production motion tracker passes its smoothed speed blend to the
skin, including NPCs and remote players. Idle, jumping, sitting, surfing and
bicycle poses retain their separate presentation paths.

Walking retains the character's idle shoulder spacing. Children, older people,
composed staff, athletes and confident trainers have different arm amplitudes,
elbow bends, torso twist and lean. Arm swing opposes the actual foot trajectory,
including the short running stance; changing elbow flexion and delayed wrists
provide follow-through. Small pelvis/chest counter-roll keeps the head steady.
The distance-driven feet and authoritative game position remain unchanged.
The complete art pass remains open: individual head proportions, hairstyles,
hand anatomy and garment silhouettes need further work. Skirts still
intersect raised knees in Run. Passing
contact/continuity checks does not approve that artwork.

Overworld `biker` NPCs use the shared bicycle frame, articulated pedals and
mounted human pose. Bikes follow each NPC's ground anchor, smoothed facing and
distance-driven pedal phase; stopped riders keep their feet on the pedals.
The Biker's mounted body is included in its culling bounds. The people reviewer
shows Biker on the bike by default, including when a cycle phase is supplied.
The supplied-rig regressions pass for mounted hand/sole contact and body bounds
over 32 pedal phases, shared rider/bike turns and anchors, independent NPC
pedals, sprite replacement/removal, and all 78 initial character pose cases.
Four-angle renders at pedal phases 0 and 0.5 and a production Route 8 comparison
at `(14, 9)`, noon, were inspected. All three Bikers are mounted at their source
positions and facing; the paired 2D/3D capture preserved the game state.

For four-angle motion reviews, pass a normalized cycle phase as the reviewer's
third argument. `CRYSTAL_PEOPLE_REVIEW_RUN_BLEND=0` selects walking (the default),
`1` selects running, and intermediate values inspect the transition.
`CRYSTAL_PEOPLE_REVIEW_RUN_WEIGHT` still controls the blend from idle to motion.
The reviewer's `--validate-walk` mode runs the same CPU gait regressions as the
unit suite against the supplied rigs, without opening a graphics window. It
also checks rounded limb seams, protected surface landmarks, distinct gait
poses, smooth animation joins, arm bone lengths and visible counterbalance
while the legs are spread.

Set `CRYSTAL_PEOPLE_REVIEW_ALL_ANGLES=1` for an eight-direction sheet per
character. The top row is front, front-left, left and rear-left; the bottom is
back, rear-right, right and front-right. Omit the character filter to render
all 75 identities. This option can also capture a fixed walk/run phase:

```sh
CRYSTAL_PEOPLE_REVIEW_HEADLESS=1 \
CRYSTAL_PEOPLE_REVIEW_IMAGE_TARGET=1 \
CRYSTAL_PEOPLE_REVIEW_ALL_ANGLES=1 \
CRYSTAL_PREVIEW_MUTE=1 \
cargo run --locked -p crystal-voxel-view --features people-review \
  --bin render_people_review -- target/people-review/all-angles
```

## Relaxed human idles

Standing actors use `human_idle` rather than the supplied crouched Idle clip.
Their hip-width stance keeps level soles planted through breathing and slow
weight shifts. Shoulders and open hands hang naturally, with restrained head
glances. Children, family characters, staff, scholars, older characters,
confident trainers, broad builds and composed characters have different
timing, arm bends and weight distribution. Stable identity phases and the
world's instance clocks keep shared rigs from moving in unison.

Spawn and subsequent animation updates sample the same pose. Walk transitions
reuse the production leg contact solver; mounted and seated poses keep their
existing fitting paths. Idle sampling changes only skin transforms, without
per-frame allocation or changes to gameplay position or facing. Culling bounds
include the new poses.

The reviewer's `--validate-idle` mode checks all 75 identities for relaxed
silhouettes, foot contact, continuous clocks and distinct poses, then checks
skinned shoe floors and bounds through idle/walk/run blends. Use
`--validate-instances` to run the production spawn, independent-skin and mounted
contact regressions directly, without compiling unrelated game-pack fixtures;
set `CRYSTAL_OPEN_MODEL_ROOT` to the supplied artwork directory for these checks. Use
`CRYSTAL_PEOPLE_REVIEW_IDLE_SECONDS` to inspect later moments in a four-angle
capture. `CRYSTAL_PEOPLE_REVIEW_HEADLESS=1` together with
`CRYSTAL_PEOPLE_REVIEW_IMAGE_TARGET=1` captures without a desktop window.
Keep `CRYSTAL_PREVIEW_MUTE=1` set for visual reviews.
Use the `wasm-dev` profile's default optimization for full-roster instance
checks; preparing every outfit with a per-crate optimization level of zero is slow.

The idle and gait validators and production instance/contact regressions passed
with the supplied artwork. Eleven characters were inspected from four angles,
with four also inspected at a later idle time. A held-state 2D/3D comparison in
`GoldenrodDeptStore1F` at `(8, 6)`, noon, preserved actor positions and facing.
This verifies the relaxed posing; the broader anatomy and painting review remains open.

## Finger posing

Native standing and locomotion poses cache finger rotations from the supplied
grip, with a mild resting curl, a relaxed walking curl and a loose running fist.
Thumb opposition stays gentler than the index-chain bend. Small continuous
changes in tension follow the idle breath or step cycle. Sampling changes only
finger rotations; wrists, bone lengths and root placement stay under their
existing controls. Idle/movement blending also blends the hand shape.

The native samplers own this curl directly, so the source-clip relaxation
cannot flatten it afterward. Jump, bicycle, surfing and seated poses keep their
separate fitting paths. The supplied skeleton has one index chain and one thumb
chain on each hand; the connected fingertip contours do not have independent
four-finger articulation.

The reviewer's `--validate-hands` check covers both production rigs, visible
fingertip closure, preserved wrist/body transforms, finite unit rotations,
unchanged translations/scales and skinned hand bounds through movement. It
does not certify hand self-intersection or finished anatomy.

The selected running index curl is 0.85 of the supplied grip, with gentler
thumb opposition. Offline reviewers can compare other index curls with
`CRYSTAL_PEOPLE_REVIEW_FINGER_CURL` (0.3–0.95); ordinary clients use the fixed
production value. Standing and running closeups of Chris, Kris, Nurse, Will,
Youngster and Blue supplied the hand review, with opposite-stride views of
Chris, Kris and Will. Existing skirt crossings and wrist/costume seams remain
visible in those captures.

The current hands were also checked in a fresh offscreen production pair in
`GoldenrodDeptStore1F`, player tile `(8, 6)`, noon. The paired receipt reports
`state_preserved=true` and matches the positions/facing of Chris, Receptionist,
Pokefan F, Bug Catcher and Gentleman, with the same Totodile follower. Clothing
intersections remain visible; this scene does not approve the broader art pass.

## Current roster and garment review

The October 20 native review revisited all 75 identities in Idle and Run at
phase 0.25, from front, left and back: 450 views drawn from the eight-angle
gallery. This is triage, not individual artistic acceptance. Flat facial
profiles, rigid rear hair, shoulder/armpit material seams, blocky exposed feet
and generic rear clothing remain visible across the cast. The desired anime
direction and the unfinished model-first redesign remain in force.

Receptionist now uses the players' small nose/chin profile correction in
source. Before/after native front, oblique and side views were inspected;
the cleaner chin is retained for production verification. Her painting,
UVs, joint influences and topology remain the same. The downstream hair
finishing responds slightly to the changed head bounds. The fresh combined
reviewer/client build and the final profile regression are still pending;
the earlier hand scene does not verify this subsequent head change.

`CRYSTAL_PEOPLE_REVIEW_DRAPE_TRIAL` enables a native frozen-pose experiment,
not production cloth animation. It fits continuous, length-limited columns
around a convex envelope of the posed leg cage and pins them to the waist's
skin field. The reviewer's `--validate-drape` mode checks the projected hull
against independent all-pairs slice supports. Native captures additionally
require a positive deformation Jacobian and reproduce candidate positions
and normals through the actual Bevy skin palette.

Eight-direction Run 0.25 trials of Nurse, Karen and Kurt were rendered and
inspected. Strict lower-body crossing pairs disappear for Nurse and Karen
in this pose, but upper-body/waist crossings remain, and the skirts/robe flare
too broadly. The shape is rejected as final art. Existing body masks also
remove upper-leg surfaces on many outfits, so crossing counts alone cannot
approve raised hems. No drape candidate has been installed in gameplay.

The fixed `head` detail camera is useful for the adult idle comparison; its
initial running close-ups clipped the chin and were discarded. Complete
full-body motion sheets remain required while that framing is improved.

## Validation of legacy human removal

The regression exercises all 77 resolved human source identities through the production weighted body/costume renderer and passes. Actor lifecycle, motion and prop checks pass (16 checks), mandatory-rig failure checks pass (2 checks), and bicycle triangle normals pass. The browser target compiled with mandatory person bundle loading. These checks cover renderer integration; they do not establish that the character art is approved or that gameplay is playable.

The subsequent Chris anime head refinement has its own passing regression for UVs, skinning, non-head body positions, accessory anchors and finite normals. Actual native four-angle idle/run renders and local-pack room screenshots are recorded in the [current art direction](stylized-character-faces.md). The rest of the character art remains intermediate and unapproved.
