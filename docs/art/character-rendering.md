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
shared weighted renderer. Each stance contact travels backward at the actor's
published speed. The shoe settles from heel to flat sole and rolls onto the
toe before its lifted return; the ankle moves around those contacts. Forward
knee bend, restrained pelvis
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
Their hip-width stance preserves each ankle's anatomical depth, keeping the
body centered over the actual soles. Level soles stay planted through breathing
and slow
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
silhouettes, balance over the actual heel-to-toe support, foot contact,
continuous clocks and distinct poses, then checks
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

Skirt, robe, kimono and coat wearers now rest their arms slightly outward and
forward. Each character retains its asymmetric elbow bends and idle clock.
This gives the palms more room around the hips without changing the fingers,
leg articulation or authoritative movement. All 29 affected paintings have
fresh eight-direction idle and Run 0.25 captures. Fingertip/fabric contacts
remain, so this pose change does not establish garment clearance. The idle,
gait, hand and production instance validators pass. A fresh offscreen room
pair at the same location reports `state_preserved=true`; the experimental
cloth described below is absent from that production scene.

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
the cleaner chin is retained. Her painting,
UVs, joint influences and topology remain the same. The downstream hair
finishing responds slightly to the changed head bounds. The combined native
reviewer/client build and the final two-rig profile regression passed. A fresh
offscreen `GoldenrodDeptStore1F` pair at `(8, 6)`, noon reports
`state_preserved=true`, with the same five human identities and Totodile
follower. This verifies production integration; the close-up comparisons
assess the profile, which is too small in the room to inspect in detail.

The bare-body shoulder investigation checked the final indexed production
meshes for Bruno, Chuck and both Swimmers. Their coincident bind-position
copies remain joined in the sampled native idle pose, with no open geometric
edges in the shoulder region. Camera rays through Swimmer Guy's bright
underarm triangles miss the closed surface: these are gaps between arm and
torso, rather than missing faces. No geometry was added to cover that space.

`CRYSTAL_PEOPLE_REVIEW_DRAPE_TRIAL` enables a native frozen-pose experiment.
Continuous, length-limited columns attach to the waist's skin field. The
original whole-cage envelope produced overly broad bells and was rejected.
Set the variable to `local` for the subsequent body-envelope study. It uses
the final prepared and posed body, retains complete lower-body surfaces,
selects the outer fitted layer and maps Karen's exposed legs to an existing
skin island. Ordinary clients retain their existing body masks and UVs.

The local study clips collision geometry to the reachable hem and fitted
waist, then converts convex support planes into radial intersections. A
directional support distance is not a radial distance; confusing these values
inflated diagonal silhouettes. The reviewer's `--validate-drape` checks that
distinction, reachable-hem clipping and the original projected hull against
independent all-pairs slice supports. Native captures also require finite,
positive deformation Jacobians and reproduce candidate positions and normals
through the actual Bevy skin palette.

Nurse, Karen and Kurt have native eight-direction frozen captures at idle and
four running phases. The current idle and Run 0.25 samples also use the new
resting arms. The study covers raised-knee holes more successfully, but hand,
waist and inner-hem contacts remain and the running silhouette still flares.
An independent strict triangle-crossing check of the actual final meshes at
Run 0.25 finds zero crossing pairs wholly below bind height 1.2 for Nurse and
Karen; Kurt retains 37 on the inner hem. This threshold excludes upper thighs,
and surface-crossing counts do not measure buried fingertips. Neither those
counts nor the frozen captures establish collision-free animation. No drape
candidate has been installed in gameplay.

The subsequent native `live` study drives the same field through each actor's
GPU skin palette. It retains shared, immutable bind geometry and gives every
instance separate affine control joints, published after transform propagation
so the field's shear is preserved. Upper-seam body joints are compacted;
remaining palette space resolves the waist, hanging panels and rolled hem
within Bevy's 256-joint limit. Source body joints and authoritative movement
remain unchanged. This path is native, opt-in and disabled in ordinary play.

Kurt's thin apron closing faces exposed a shading defect under this stronger
deformation. Live preparation gives those edges separate normals while copying
their exact positions, UVs, colours and weights. Coincident boundary points
still move together. The broad apron faces retain their existing shading;
ordinary preparation remains unchanged.

The initial live study's `--validate-live-drape` passed for Nurse, Karen and Kurt
at idle and 32 running samples, including both loop endpoints. Maximum error
against the detailed field is respectively 0.027228, 0.031156 and 0.033643 source
units, below the unchanged 0.035 limit. Lower vertical fabric panels retain
outward shading and positive finite skin derivatives. Two Nurse instances use
actual Bevy ECS joint and bind transforms under translated, rotated and scaled
roots. They reproduce the field, share their mesh, move independently and remove
their owned control joints when despawned. The default production instance validator
also passes with the experiment disabled.

```sh
CARGO_TARGET_DIR=target/people-motion-20261020 cargo build --locked --jobs 2 \
  -p crystal-voxel-view -p crystal-bevy \
  --features crystal-voxel-view/people-review,crystal-bevy/location-tester \
  --bin render_people_review --example render_at_location \
  --config profile.dev.package.crystal-voxel-view.opt-level=1 \
  --config profile.dev.package.crystal-bevy.opt-level=1

CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
CRYSTAL_PEOPLE_REVIEW_DRAPE_TRIAL=live \
target/people-motion-20261020/debug/render_people_review --validate-live-drape

CRYSTAL_PEOPLE_REVIEW_HEADLESS=1 \
CRYSTAL_PEOPLE_REVIEW_IMAGE_TARGET=1 \
CRYSTAL_PEOPLE_REVIEW_ALL_ANGLES=1 \
CRYSTAL_PEOPLE_REVIEW_RUN_BLEND=1 \
CRYSTAL_PEOPLE_REVIEW_DRAPE_TRIAL=live \
CRYSTAL_PREVIEW_MUTE=1 \
target/people-motion-20261020/debug/render_people_review \
  target/people-review/live-run nurse,karen,kurt 0.25
```

These commands use the `people-review` binary built by the command above.
The before/after study contains 54 native sheets: all three fixtures at idle
and eight run phases, eight directions per pose, with the same camera and
lighting. Front/profile motion sequences, idle and opposite-stride eight-angle
views were inspected. The original external sprite sheets supplied front,
back and profile references; Kurt also has walking frames. This pass preserves
the existing proportions and garment lengths. It does not approve their broader
likeness or styling.

Raised-knee holes are covered more successfully, while the running hems still
flare and hand/waist contacts remain. Independent triangle checks of complete
body surfaces at idle, Run 0.25 and Run 0.75 find no leg crossings on radially
outward garment faces in these nine samples. The bind-height-below-1.2 subset
has zero pairs for Nurse and Karen; Kurt retains four inner-hem pairs in each
running sample. These counts exclude coplanar/edge contacts, and neither this
sampling nor the native validator proves collision-free animation.

A fresh offscreen `GoldenrodDeptStore1F` pair at `(8, 6)`, noon, uses the live
experiment and reports `state_preserved=true` for the same five people and
Totodile follower. It verifies integration and matching presentation state;
counter occlusion and gameplay size limit close garment inspection. The
detailed solver initially cost about 32–34 ms per new pose in this development
build (Receptionist/Pokefan F measured roughly 27–34 ms in the room). The
follow-up below improves its cost and broadens the numerical coverage. Contact,
silhouette and browser review are still required before normal play.

The follow-up retains all 1,656 support planes, 257 height samples and 48 angular
columns. Projected body hulls identify the support candidates, and each fixed
radial column precomputes the lower envelope of its plane bounds. Independent
full-point and full-plane scans remain the numerical oracle. Nine native mesh
fixtures agree within 0.000007 source units, with normal cosine above 0.9999996.
Across 128 movement palettes each, Nurse/Karen/Kurt's median field calculation
fell from 28.6/29.9/31.7 ms to 5.7/6.7/7.4 ms in the same local comparison. These
are development CPU measurements under shared machine load, not gameplay FPS.

Authored torso attachments, including the kimono obi, retain their original
skin rather than entering the hanging-cloth field merely because they are
below the waist. The control frames now split the residual derivative with
interpolation, which avoids counting curvature twice while preserving the
full waist affine. The control count, garment lengths, complete body surfaces,
camera, paintings and source gait remain unchanged by this follow-up.

`--validate-live-roster` now passes all 25 closed-garment identities at idle
and 32 running samples, including both loop endpoints. Its largest field
error is 0.024311 source units, below the unchanged 0.035 limit; lower vertical
panels have positive finite skin derivatives and outward shading. Each identity
also passes the actual Bevy ECS test with two independent instances, translated,
rotated and nonuniformly scaled roots, shared immutable geometry, separate
palettes, production culling bounds at idle and Run 0.25, and owned joint
cleanup. The explicit obi regression preserves every authored sash vertex,
while hanging cloth must still differ from its old skin. A separate calculation
using the actual production update's 32 phases at four movement weights passes
all 3,200 palettes, with maximum field error 0.024127. The default production
instance validator and the radial-envelope validator also pass.

```sh
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
CRYSTAL_PEOPLE_REVIEW_DRAPE_TRIAL=live \
target/people-motion-20261020/debug/render_people_review --validate-live-roster
```

This broadens the experiment's numerical coverage. It does not accept the
characters' likeness, eliminate hand/waist contacts, establish collision-free
animation, or enable the native experiment in ordinary or browser play.

The fresh angle review contains 150 normal-model sheets (75 identities at idle
and Run 0.25) and 225 experimental sheets (25 closed garments at idle and eight
run phases). All 3,000 captured views pass foreground/framing checks. The
25-character front/profile motion sequences and quarter-stride angle sheets
were inspected. Long hems still flare; repeated rear hair shapes, plain rear
costumes and hand contacts remain visible limitations.

The first expanded strict contact check failed for the three kimono identities.
At Run 0.25, radially outward panels had 26/68/26 leg crossing pairs for
Erika/Falkner/Kimono Girl; Run 0.75 had 26/74/26. Nurse and Karen had no strict
leg crossings in these samples. Kurt had no radially outward crossings, but
49/45 lower-leg pairs on all panels at the two strides, increased from four
in the previous Run 0.25 study. That detailed field target also had
13/54/13 outward pairs for the kimono fixtures at Run 0.25, so closer control
interpolation alone cannot fix the contact target. These checks exclude
coplanar and exact edge contacts and do not inspect hand or waist contact.
At idle, all six have zero outward leg pairs and zero lower-leg pairs on all
panels. This hem/footwear regression prevented acceptance of that prototype.

A rebuilt offscreen `DanceTheater` pair at `(6, 3)`, noon, reports
`state_preserved=true`, with the same ten actors including all five Kimono
Girls. The performers remain readable above the stage; the audience is
partially cropped by this camera. Initial Kimono Girl solves measure roughly
8–9 ms and Granny 7.5–8 ms in this development capture. This verifies matching
game state and native integration, not gameplay frame rate or garment approval.

The subsequent hem-clearance correction adds room beside and above a shoe,
using four contact corners at each body point. The existing radial offset
alone did not protect the rolled hem from the foot's sides and top. Clearance
still fades to zero at the authored waist, and feet below the reachable hem
are clipped before fitting. Garment lengths, original body surfaces, source
gaits, palette size and paintings remain unchanged by this correction.

The native `--validate-live-roster` now checks strict lower-leg/fabric triangle
crossings at every one of its 33 poses, across all 25 closed garments. It
passes all 825 posed states and the existing per-identity two-instance ECS
checks. Maximum error against the detailed field is 0.022115, below the
unchanged 0.035 limit; the minimum checked normal alignment is 0.084256.
Separate sweeps of the actual production update pass 3,200 running palettes
and 3,200 walking palettes, each using 32 phases and four movement weights,
with zero strict lower-leg crossings on either fabric layer. Their maximum
field errors are 0.021506 and 0.016214 respectively. The new detector also
finds the old Run 0.25 failures: 49/111/182/110 lower-leg pairs for
Kurt/Erika/Falkner/Kimono Girl, all reduced to zero in the corrected fixtures.
Contact scope is body triangles below bind height 1.2 with leg influences;
coplanar, exact edge, hand, waist and garment self-contact are outside this
check. The radial-envelope/clearance and strict-crossing unit checks pass,
as does the default production instance validator.

Exact duplicate body corners are now removed before clearance expansion.
This avoids repeatedly allocating and sorting the same expanded points for
incident triangles. Eighteen native mesh/pose fixtures produce identical
positions and normals with and without the optimization. Their median field
calculation falls from 17.4 to 14.9 ms in the local paired development run.
A fresh `DanceTheater` pair at `(6, 3)`, noon, again preserves the same ten
actors and reports `state_preserved=true`; initial Kimono Girl solves measure
9.4–10.7 ms and Granny 7.9–8.9 ms. These measurements are subject to shared
machine load and do not establish gameplay FPS.

The updated review captures all 25 closed garments from eight directions at
idle, eight run phases and a walking pose, with matched before/after motion
studies: 250 fresh sheets and 2,000 views. All views pass foreground/framing
checks. An independent triangle checker also passes 100 native exported poses
(idle, both quarter strides and walking), using the actual control palettes
and complete prepared body meshes. The foot emerges beneath the hem more
cleanly. Running robes still
flare into rigid bell shapes, and hand/waist overlap and broad likeness remain
unfinished. The pinned external source's `gfx/trainers/falkner.png` also
confirms separate wide trouser legs and an asymmetric fringe; the
single closed garment and central quiff at that stage needed a modeling pass. Battle and
overworld palette contexts must be kept distinct when applying that reference.
The complete cast remains unapproved, and this experiment remains disabled
in normal and browser play.

The following Falkner pass replaces the central quiff with a lower side part
and one connected, two-layer fringe over the left eye. It is constructed on
the final prepared scalp and follows the existing head joint. Its entire root
enters occupied scalp; the opposite eye stays clear. Separate closed locks
were rejected after native close-ups exposed a plate-like appearance and skin
gaps. The connected surface also needed a shorter temple edge after inspecting
the right profile. The source painting and physical width/height settings are
unchanged. Falkner now uses two scalp refinement passes instead of three;
the complete prepared body has 32,798 vertices and 42,804 triangles before
ordinary garment coverage, compared with 46,218/69,012 in the earlier fixture.

The native head-detail reviewer now exposes the same eight directions as the
full-body orbit. `CRYSTAL_PEOPLE_REVIEW_SINGLE_VIEW` uses 0 front, 1 front left,
2 left, 3 rear left, 4 back, 5 rear right, 6 right and 7 front right. Both the
actor and the posed-head camera use that shared orbit. Older four-view detail
captures used index 3 for the back. Fresh native head views cover all eight
directions; full-body captures include idle and both quarter strides with
`CRYSTAL_PEOPLE_REVIEW_RUN_BLEND=1` for running.

With the trouser trial disabled, the rebuilt live-garment roster passes all
825 posed states and 25 independent-instance checks. The maximum field error
remains 0.022115 and the minimum checked normal alignment 0.084256. The
ordinary production instance validator also passes. The fringe check covers
buried roots, asymmetric eye coverage, forehead distance, closed volume and
finite outward shading. A production `VioletGym` pair at `(5, 3)`, noon,
preserves the same five actors, frame 69 and state hash 2609560824, including
Falkner. His placement and blue silhouette are readable against the rear wall;
this room camera is too distant to certify facial detail. Native close-ups
remain the evidence for the fringe. These are native checks, not browser or
whole-cast visual acceptance.

`CRYSTAL_PEOPLE_REVIEW_TROUSERS_TRIAL=1` selects a separate articulated-trouser
prototype only alongside `CRYSTAL_PEOPLE_REVIEW_DRAPE_TRIAL=local` or `live`.
That requirement retains the complete underlying body during review. Its
two-layer surface has a connected crotch and two leg openings, with source
leg influences and separate normals at the waist/cuff cuts. The neutral
topology and shading checks pass, but the native moving test fails for
Falkner at frame 1 (normal alignment -0.14770624). The separate production
palette sweep also finds running leg/fabric crossings. The roster still
includes him in all 25 checks; the trial failure is retained. This prototype
is disabled during ordinary and browser play. Falkner's calf fullness, knee/crotch
fit, rear hair paint seam, flat facial profile and the correct upper clothing
remain unfinished; the original lower garment also remains visually unapproved.

The next trouser cut replaces the broadened copy of the body's folds with
joined panels, a shared crotch seam and two broad hems. Its upper edge samples
the actual torso section beneath the obi, removing the detached bucket-like
waist seen in the first native trial. Source skin weights are averaged over
horizontal body-section segments; nearest-surface transfer had incorrectly
assigned wide hip panels to lower thigh corners. The source painting, physical
proportions, authored obi and gait remain unchanged. The final costume,
including the obi, has 5,288 vertices and 9,216 triangles. The complete retained
body remains 32,798 vertices and 42,804 triangles.

Closed geometry, finite shading, skin bounds and neutral lower-leg clearance
pass. New waist-attachment and cuff-to-calf width guards also pass and reject
the old copied-body cut: its cuff width was 0.215632 against a 0.500002 calf.
This is a neutral-shape improvement, not posed-surface acceptance. Native
validation still fails at the idle sample, frame 0, with normal alignment
-0.12592241. A separate sweep over 256 retained production palettes finds up
to 646 running and 168 walking lower-leg/fabric crossing pairs. The body,
gait, numerical tolerances and all 25 roster entries remain in these checks.

A 32-ring affine carrier experiment was evaluated in the actual native
skinning path and removed after the running views exposed a trailing cuff
stretched into a broad plate. It reduced some running contacts but retained
posed folds, strict fabric self-crossings and inward shading; a lower contact
count did not establish a better moving model. The kept panel cut still uses
the transferred source skin, and the trouser trial remains disabled in normal
and browser play. The default production instance validator and the existing
825-pose live-garment roster continue to pass with the trouser trial disabled.

The new review includes 64 fresh full-body views (kept and rejected variants at
idle, walking and both running quarter strides), four native trailing-cuff
detail crops and matched comparisons against three retained eight-angle
baselines. The detail crops intentionally omit part of the raised foot.
A fresh production `VioletGym` pair at `(5, 3)`, noon, enables the kept panel
trial and preserves the same five actors, frame 247 and state hash 6423770.
The waist and straighter hems read more cleanly, while crotch/inner-knee fit,
upper clothing, face depth and the complete cast remain unfinished.

The native `head` detail camera now follows the currently skinned head,
including the presented proportions and knee compression. Fresh running
side views of Receptionist, Youngster and Granny retain the whole chin and
crown. The earlier fixed-camera running close-ups were cropped and discarded.

Older characters now use an 82% stride and 80% swing lift; staff and formal
walks use 90% stride and 88% lift. The distance publisher advances the matching
character cycle, preserving authoritative travel and planted foot speed.
The full native gait validator passes, including the actual publisher plus
posed ankle contacts for every identity at walking and running speeds. This
keeps articulated knees; it is not a replacement for the unfinished garment
deformation. The final two-rig profile regression also passes after this change.

The updated native gallery contains all 75 paintings in idle and Run 0.25,
eight directions each (150 sheets, 1,200 views). Foreground and framing checks
pass in every cell. The 27 distinct paintings affected by the stride families
were also reviewed from front, left and back, with before/after comparisons
and eight-pose motion samples for Granny, Kurt, Nurse, Receptionist, Officer
and Mom. Knees still articulate; robe/apron/skirt intersections remain visible.
The current matched room pair again reports `state_preserved=true`.

The subsequent garment-pose gallery refreshes the 29 affected identities in
both poses and their eight-view idle turntables; the other 46 retain their
unchanged earlier captures. It includes before/after production idle views
for Nurse, Karen and Kurt and a separate frozen cloth study. Every delivered
PNG cell has a framed foreground; all 75 turntables contain eight frames.
The complete cast remains under artistic review.

## Shared anime face volume and angle review (20 October 2026)

The production preparation path now gives all 75 people a shallow nose and a
receding chin. It preserves frontal feature coordinates and the existing
painting, while recessing the surrounding lower face. The eye plane, crown,
body and source collar loop remain fixed. Linear adaptive refinement supplies
detail only where the nose still crosses long triangles. The mixed head/jaw
influences follow the field continuously; preserving every intermediate
mixed-weight point had made an inward ridge above the pinned neck loop.
Fitted facial accessories receive the same deformation in head-joint space.
Falkner's fringe already fits the prepared head and is not deformed twice.

Authentic Crystal front/back/profile overworld frames were rechecked from the
external source workspace, including the Gym Guide's rectangular glasses.
His appearance now includes those frames, with a larger rectangular lens
opening and thicker wire than the small round spectacles. The shared head
volume is an authored anime interpretation of the supplied paintings, not a
claim that the 2D game provides a canonical 3D head.

The first stronger volume passed geometry checks but made a dark nose blob
in the actual front render. It was rejected. The kept version narrows and
softens the relief and transports a quarter of the new curvature into the
painted face's lighting normals. The full geometric deformation remains in
the surface. This intentionally quiet shading avoids doubling the nose
shadow already drawn in the artwork. All changed triangles still require
positive alignment with their shading normals, finite unit normals, retained
area and a positive deformation Jacobian.

Head and hair subdivision now stop when their existing edge detail is
sufficient. Final body preparation also merges exact duplicate attribute
records and removes unused records. It preserves every indexed triangle
corner's position, normal, color, UV, joints, weights and cutaway ownership;
coincident UV, normal and skin seams remain separate. Actual prepared bodies
in this review changed from 37,031 to 17,714 vertices for Chris, from 65,970
to 31,980 for Nurse, and from 209,527 to 37,458 for Standing Youngster. These
are geometry counts, not frame-rate measurements. Compaction itself retains
the triangle count; adaptive refinement accounts for topology differences.

The native `--validate-faces` path now prepares the actual entire cast in
addition to its older two-rig reference checks. It verifies the changed
surface, preserved artwork and anchors, nose/chin depth, finite outward
shading and a prepared body budget below 120,000 vertices for every identity.
The production instance, idle and locomotion validators pass. The existing
25-identity live-garment roster also passes with the tailored-trouser trial
disabled. The Cargo test
`people_mesh::tests::duplicate_records_merge_without_merging_uv_normal_or_skin_seams`
passes, including the cutaway ownership seam.

All 75 people were freshly rendered in idle and running phase 0.25, eight
directions each, plus all eight native head directions at 1000 pixels.
Eleven representative people also have four walking and four running phases;
their running quarter phase is shared with the cast set. That is 2,416 unique
current angle/pose views. Framing checks pass for all 1,816 body cells and
600 head views. Labeled contact sheets derive from those actual images and
preserve their camera scale. Head busts intentionally crop the shoulders.
Matched before/after head sheets cover all eight directions for those eleven
people; all-cast baseline body sheets were also captured before editing.

The room client was rebuilt against the current renderer changes after an
older executable timed out waiting for an active world frame. Fresh successful
production comparisons use the local `core-modular.crystalpack` at noon:
`VioletGym`, player `(7, 14)`, reports `state_preserved=true`, five source
actors, frame/LCD frame 67 and hash 561317937; `VioletPokecenter1F`, player
`(3, 4)`, reports the same state-preservation result, five source actors,
frame 58, LCD frame 62 and hash 2511709195. Both pairs were visually inspected.
They establish placement, scale, occlusion and identity in the room; the
native head images establish facial detail. These checks do not establish
browser visual acceptance.

The shared face pass is a scoped improvement. Side/rear jaw facets, hair
paint seams, body/outfit identity, moving garment fit and the separate
disabled trouser prototype remain unfinished. The current gallery is outside
the repository at
`~/.codex/artifacts/people-all-angles-face-volume-2026-10-20/index.html`.
No reference sprites, generated meshes, packs or captures are shipped with
the source.

## Trouser seam construction and motion review (20 October 2026)

The opt-in Falkner trouser cut now keeps thickness inside its authored outer
surface. The previous outward offsets consumed the narrow medial space and
produced 278 strict neutral fabric self-crossing pairs despite passing closed
manifold and winding checks. The inset cut has zero such neutral pairs. The
complete body, painted appearance, waist, cuff height and gait remain present.

The connected upper panels also receive a harmonic skin-weight field. Shared
crotch vertices merge opposite-shin influence into the corresponding thigh
before reducing to four joint slots; an arbitrary smallest-weight drop had
made the adjacent panels disagree. A broader thigh/shin transition removes
the previous two local idle triangle inversions. The actual native idle
surface has positive shading alignment (minimum approximately 0.5675).

These improvements do not establish a valid moving garment. A new strict
fabric self-contact check finds 433 unordered pairs in the actual native idle
trouser surface, down from 614 before. The separate authored obi/bow has
occupied joins and retains its own skin and attachment checks. Trouser
self-contact excludes that distinct component while retaining every trouser
face. The neutral checker also rejects the previous cut. A new hip-inclusive
body-contact selection keeps whole triangles straddling the waist; the older
below-knee selection remains unchanged for the other garment checks. Its
synthetic regression verifies both selections and actual skin-palette motion.

The existing external-rig Cargo test
`supplied_trousers_have_two_articulated_closed_legs` passes the closed cut,
waist/cuff constraints and actual idle triangle shading. Both
`contact_validation::tests` pass. A supplementary actual-source probe over 256
retained production walking/running palettes still finds up to 573 lower-leg
crossing pairs and negative moving normals. A walking self-contact probe
also fails at phase zero with 194 unordered pairs. These probe harnesses use
rig shims and supplement the native integration checks; they are not passing
motion evidence. Shin-only hems, hip pinning, a shortened cuff and an extra
bilateral support control were evaluated and rejected.

The rebuilt native production-instance validator and all 825 default
live-garment pose samples pass with the trousers disabled. Enabling them
retains the full roster and rejects only Falkner at idle: 433 fabric
self-crossing pairs and 192 leg/hip-body pairs. The latter confirms the
visible hip/crotch poke-through missed by the old below-knee contact scope.

The scoped review contains ten fresh current native eight-direction sheets
(80 full-body views), four matched pre-change sheets, four intermediate
rejected sheets and eighteen 1000-pixel hip/lower-leg detail crops. All 144
full-body cells have foreground clear of their edges. Detail crops
intentionally omit the head/shoes or waist, and can clip part of a raised
foot. Every current sheet and the labeled detail contacts were visually
inspected. Body poke-through at the crotch, inner-knee folds and cuff fit
remain visible. The trousers stay disabled during ordinary and browser play.

A fresh production pair from the rebuilt client uses the local
`core-modular.crystalpack`, `VioletGym`, player `(5, 3)`, noon, with both trial
flags enabled. Its receipt reports `state_preserved=true`, five source actors,
frame/LCD frame 64 and hash 2493642372. Both views were inspected: Falkner
occupies the expected rear platform with a readable blue/white silhouette;
the broad room camera does not resolve his crotch/cuff defects or face detail.
The close-up native views remain the evidence for those surfaces.

The review is outside the repository at
`~/.codex/artifacts/people-trouser-seam-study-2026-10-20/index.html`.
The earlier all-75 face/angle gallery remains a separate snapshot; it was not
regenerated by this scoped trouser study. Neither gallery establishes full
animation, frame-rate or browser visual acceptance.

## Inner leg cut and hip fit (20 October 2026)

The next opt-in trouser revision fits the medial leg section to the retained
body while preserving the wider outer panel. The old round sections occupied
the space between the knees when the thighs turned inward. Opening the fork
directly from the shared saddle removes the remaining near-coincident upper
panels; the previous zero-tangent width left them crossing despite positive
shading normals. Hip sections now have enough clearance for the complete
source body. The painting, cuff height, outer fullness and gait are retained.

The supplied-rig Cargo regression now also checks the actual idle surface for
strict fabric self-crossings. It passes. The rebuilt native roster confirms
zero idle fabric self-crossing pairs, down from 433, and 20 leg/hip-body pairs,
down from 192. Those body contacts still reject the prototype. They lie near
the front hip/crotch transition; the body has not been removed to hide them.

A supplementary comparison uses the same complete prepared body and 256
retained production palettes for both cuts. In its 128 walking samples, the
new cut has zero fabric self-crossing pairs (previous maximum 465), positive
local normal alignment, and up to 120 body-contact pairs (previous maximum
280). In 128 running samples, the maxima change from 476 to 236 fabric
self-crossing pairs and from 942 to 802 body-contact pairs. Running normals
still turn inward, with a minimum alignment around -0.717. These are
actual-source probes with rig shims, supplementing the native integration
checks; they do not establish motion acceptance. Direct directional skin
transfer was rejected because it worsened running folds.

The first native views caught a new waist/sash intersection introduced by the
fuller hips. The sash now fits the actual trouser surface beneath it. A
regression samples the real inner barrel triangles between their authored
rings and directions, preserving a strict clearance requirement and the
original torso skin. The initial fitted margin was insufficient between
rings, so the geometry clearance was increased. The source body, gait and
numerical crossing tolerances remain intact.

Both supplied-rig Cargo tests pass, including the new sash's consistently
closed winding, outward triangle shading, unchanged torso joints/weights and
clearance measured between its authored rings and columns. The supplementary
mesh probe measures a minimum inner gap of about 0.016, versus approximately
-0.024 for the old sash over the fuller trousers.

The first per-height fit made an unwanted raised horizontal band in the native
close-up and was rejected. The kept sash uses one tapered height profile:
its top stays fixed and its lower radius encloses 24 sampled body/cloth
sections. The rear knot and short ends follow the new sash at their own
heights. The existing three-character obi regression also passes; ordinary
Erika, Falkner and Kimono Girl sashes keep their original fit path.

The final review has ten fresh native eight-direction sheets (80 current
views), ten matched retained pre-change sheets, and 21 fresh 1000-pixel
details: front/left/back/right hips and front/left/right lower legs at idle,
Run 0.25 and Run 0.8125. All 160 full-body cells pass framing checks. Every
current sheet and both complete detail contacts were inspected. The sash has
a continuous lower edge and visible occupied knot/tails in these sampled
poses; hip poke-through, inner-knee folds and cuff spurs remain visible.
Detail cameras intentionally crop other body regions and can omit part of a
raised foot.

A fresh rebuilt production pair uses the local `core-modular.crystalpack`,
`VioletGym`, player `(5, 3)`, noon, with both trial flags enabled. It reports
`state_preserved=true`, five source actors, frame/LCD frame 325 and hash
387755716. Both views were inspected. The blue/white silhouette reads on the
rear platform at the expected scale; the wide room camera cannot resolve the
hip defect or establish close-up cloth acceptance. The production-instance
validator and default 825-pose garment roster pass; the trouser-enabled roster
still rejects the 20 idle body contacts.

The review is outside the repository at
`~/.codex/artifacts/people-falkner-fit-2026-10-20/index.html`. The all-75
gallery remains a separate retained snapshot. No frame-rate, full-animation
or browser acceptance is claimed, and no raw references, packs or generated
meshes are shipped with the source.

The trouser trial remains disabled during ordinary and browser play. Running
folds, front hip contact, upper clothing, hair seams and the broader cast's
model and animation work remain unfinished.

## Shared shoulder skin and complete angle review (21 October 2026)

The final prepared bodies now spread shoulder and elbow influence continuously
instead of jumping from a chest/shoulder mix to a rigid upper arm over one
short edge. The old field turned underarm faces against their shading when
the arms lowered. `arm_skin` derives the joints and dimensions from the
supplied rig, refines coarse shoulder triangles with joined linear edges,
then fits the skin field. Linear refinement preserves the bind surface and
paint interpolation. The rig, gait, physical sizes, wrists, hands and unrelated
skin influences retain their existing paths.

The kimono sleeve refinement also joins neighboring torso triangles. The
completed Falkner and Erika body exports each have zero open upper-body
edges, compared with 192 before this correction. The supplied-rig regression
checks the closed sleeve/torso boundary for Erika, Falkner and Kimono Girl,
alongside the existing surface, skin and unrelated-character protections.

Bruno, Chuck and Swimmer Guy rebuild their torso normals from the final
prepared surface, after all arm refinement. Rebuilding earlier retained
coarse interpolated normals on the new underarm triangles. The normal finish
changes neither their geometry nor the animation. Other identities retain
their normal path.

The rebuilt native `--validate-arms` check uses `Person::update` on all 75
final appearances. It exercises idle clocks, 32 walking/running phases,
55-percent transitions and the three supported riders, with mounted poses
limited to those riders. All 12,320 pose checks pass. The minimum geometric
face/shading agreement is 0.11234, above the unchanged 0.05 requirement; the
minimum posed/bind area ratio is 0.10614, above the unchanged 0.015 limit.
Six supplied-rig Cargo regressions pass: arm surface/skin protections, the
kimono boundary check and four exposed-body checks. Native instance, idle,
walk, face/budget and default 825-pose live-garment checks also pass.

The current gallery contains all 75 people in eight labeled directions at
idle and Run 0.25: 1,200 full-body views. Biker uses mounted idle and pedal
phase 0.25. Eleven matched upper-body studies provide 176 native 1000-pixel
images; a matched Swimmer Guy Run 0.375 profile adds two more. All 1,376
current/baseline full-body cells pass framing checks. Every full-body angle
and the complete upper-body contacts were inspected. The gallery and its ZIP
remain outside the repository at
`~/.codex/artifacts/people-all-angles-arm-skin-2026-10-21/index.html`.

Fresh production captures use the local `core-modular.crystalpack`, noon,
`VioletGym` at `(5, 3)` and `VioletPokecenter1F` at `(3, 4)`. Both source/current
pairs report `state_preserved=true` and five source actors. The Gym receipt
reports frame/LCD frame 323 and hash 4175536180; the clinic reports frame 82,
LCD animation frame 89 and hash 536525370. The triptychs contain the 2D
reference, a retained previous 3D capture and the current 3D capture; the
previous image is historical. The current blue/white gym silhouette and pink
haired clinic nurse remain readable at their expected positions and scale.
The wide room views do not resolve the shoulder corrections.

The review still exposes pinched armpits, jagged sleeve cuffs, pale artwork
fragments at Kimono Girl's rear shoulder, abrupt raised-knee garment cuts and
hair/jaw seams. These are unfinished art, not visual acceptance of the cast.
Visual review of jumping, surfing, sitting and complete animation cycles,
browser appearance and frame rate are outside this review's coverage. The trouser trial remains
disabled during ordinary and browser play.

The executed native review commands use the supplied `assets/open-models`
and the existing ignored local pack:

```sh
export CARGO_TARGET_DIR=target/people-motion-20261020
export CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models"
cargo build --locked --jobs 2 -p crystal-voxel-view -p crystal-bevy \
  --features crystal-voxel-view/people-review,crystal-bevy/location-tester \
  --bin render_people_review --example render_at_location \
  --config profile.dev.package.crystal-voxel-view.opt-level=1 \
  --config profile.dev.package.crystal-bevy.opt-level=1
target/people-motion-20261020/debug/render_people_review --validate-arms
```

The angle renderer uses `CRYSTAL_PEOPLE_REVIEW_HEADLESS=1`,
`CRYSTAL_PEOPLE_REVIEW_IMAGE_TARGET=1`,
`CRYSTAL_PEOPLE_REVIEW_ALL_ANGLES=1` and `CRYSTAL_PREVIEW_MUTE=1`.
`CRYSTAL_PEOPLE_REVIEW_DETAIL=torso` with a single view selects the posed
upper-body camera. The native review-only
`CRYSTAL_PEOPLE_REVIEW_ARM_SKIN_BASELINE=1` bypasses the new arm field and its
refinement for comparison; it is not a production setting. Packs, exported
diagnostic meshes, captures and generated reports are not added to the shipped
bundle.

## Sleeve artwork follow-up (21 October 2026)

Kimono Girl's rear underarm sampled part of the front blossom painting. The
shoulder mapping stopped above that recess. `painted_torso::kimono_shoulders`
now maps the complete sleeve-root faces to uninterrupted red fabric while
preserving the central front painting and collar. Matched lit and unlit native
views confirm the duplicated blossom is gone. The underarm shape is unchanged;
its pinching and angular folds remain unfinished.

The new supplied-rig regression samples the actual prepared underarm faces on
both sides, including triangle interiors, and checks the untouched central
painting, geometry, normals, winding and skin weights. All eight
`painted_torso::tests` pass. Indexed idle and Run 0.25 exports retain exact
positions, normals, joints, weights and animation palettes before/current.
The sampled underarm region drops from 453 non-cloth triangle centroids to
zero. Erika, Falkner and Nurse retain their indexed surfaces and UVs exactly.
The six native arm, instance, idle, walk, face/budget and live-roster gates also
pass, including the unchanged 12,320-pose arm check and 825 garment poses.

The new external review is
`~/.codex/artifacts/people-all-angles-sleeve-uv-2026-10-21/index.html`, with a
sibling ZIP. It contains 1,200 fresh full-body views of all 75 people, eight
directions at idle and Run 0.25, plus 64 native 1000-pixel before/current upper
body images and an unlit rear pair. Biker uses mounted idle and pedal phase
0.25. All 1,328 current/comparison full-body cells pass framing checks. The
corrected character's eight detail directions and representative cast sheets
were inspected; this does not establish visual acceptance of every animation.

Fresh production pairs show the 2D reference beside current 3D, using the
existing local pack at noon. `VioletGym` at `(5, 3)` reports preserved state,
five source actors, checksum frame 74/hash 2821199722 and LCD frame 74.
`VioletPokecenter1F` at `(3, 4)` reports preserved state, five source actors,
checksum frame 42/hash 885503962 and LCD animation frame 46. The people remain
readable at their expected positions and scale. These room views do not resolve
the sleeve-root UV correction. Raised-knee garment cuts, cuffs, underarm shape,
hair/jaw seams, complete animation cycles, browser appearance and frame rate
remain open. Experimental trousers stay disabled.

The build and headless review settings above were used again. The executed
clothing regression command was:

```sh
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  CARGO_TARGET_DIR=target/people-motion-20261020 \
  cargo test --locked --jobs 2 -p crystal-voxel-view \
  --features people-review --lib painted_torso::tests \
  --config profile.dev.package.crystal-voxel-view.opt-level=1 \
  --config profile.dev.package.crystal-bevy.opt-level=1 -- --ignored
```

## Validation of legacy human removal

The regression exercises all 77 resolved human source identities through the production weighted body/costume renderer and passes. Actor lifecycle, motion and prop checks pass (16 checks), mandatory-rig failure checks pass (2 checks), and bicycle triangle normals pass. The browser target compiled with mandatory person bundle loading. These checks cover renderer integration; they do not establish that the character art is approved or that gameplay is playable.

The subsequent Chris anime head refinement has its own passing regression for UVs, skinning, non-head body positions, accessory anchors and finite normals. Actual native four-angle idle/run renders and local-pack room screenshots are recorded in the [current art direction](stylized-character-faces.md). The rest of the character art remains intermediate and unapproved.

## Moving arm volume and eight-angle review (21 October 2026)

The shared weighted renderer now interpolates rigid dual-quaternion arm blends
through a sparse set of additional GPU joints. Each person owns the moving
joint entities and pose buffer; prepared appearances share an immutable recipe.
The source animation joints remain the exact palette prefix, and separate
costume meshes retain their original bindings. Preparation includes mixed
Spine/chest/arm corners that the preceding five-carrier field skipped. The
final hanging-sleeve surface receives welded angle-weighted normals before
the arm weights are fitted.

The recipe changes skin bindings without changing bind positions, UVs or
topology. The browser worker carries and validates that recipe alongside the
mesh; actor response protocol version 2 rejects stale response layouts. The
source fingerprint includes the new Rust modules. All 75 appearances use
lattice resolution 12, with at most 252 of Bevy's 256 joint slots. The ordinary
native renderer evaluates these joints from the authoritative live pose.
`CRYSTAL_PEOPLE_REVIEW_ARM_MATRIX_SKIN=1` provides a native comparison with the
current torso field; `CRYSTAL_PEOPLE_REVIEW_DQ_SKIN=1` freezes an exact full blend
at a single review pose. Neither comparison flag is a production setting.

All six native arm, instance, idle, walk, face/budget and live-roster checks
pass. The final mesh and actual `Person::update` pass 12,320 production poses
across all 75 people and the three supported riders. Minimum outward agreement
is 0.09019455 and minimum posed/bind triangle area ratio is 0.03317699, above
the unchanged 0.05 and 0.015 limits. The garment gate also passes all 825 poses
with experimental trousers disabled. Per-instance checks cover independent
helper joints, movement, costume binding prefixes and entity cleanup.

Three new math regressions pass, including lattice reconstruction and
antipodal quaternion handling. Four supplied-rig tests pass independently:
mixed Spine corner refinement, unrelated surface/skin protections, five real
appearances through paged transport, and a complete paged actor job. A separate
native-data Rust probe compares Nurse, Falkner, Swimmer Guy and Kimono Girl over
128 captured production palettes each. Original bind attributes and source
pose prefixes match exactly. Published helper matrices match the recipe, with
maximum element error zero. Compared with full per-vertex dual-quaternion
blending, maximum position error is 0.000791067 source units and maximum normal
angle error is 0.071327 degrees. This is a math/transport check; native rendered
images provide separate visual evidence. The browser worker target compiles;
browser visuals and frame rate remain unverified.

The external review is
`~/.codex/artifacts/people-volume-skin-2026-10-21/index.html`, with a sibling
ZIP. It contains 1,200 fresh full-body views: all 75 identities, eight labeled
directions, idle and Run 0.25 (mounted idle/pedal phase for Biker). Six
click-to-play running loops contain eight captured phases and eight directions,
adding 384 views; their playback timing is illustrative. Ten matched
eight-direction shoulder studies retain the preceding renderer beside current
captures. Fresh matrix/full-blend comparisons and critical Run 0.375 profiles
are labeled separately. All 1,584 full-body cells pass framing checks; the
gallery has 444 PNGs, six eight-frame WebPs, 81 HTML pages and 1,195 verified
local references. The ZIP passes CRC and exact file-byte comparison.

Fresh noon production pairs use the existing local pack: `VioletGym` at
`(5, 3)` reports preserved state, five source actors, checksum/LCD frame 151 and
hash 1261281087; `VioletPokecenter1F` at `(3, 4)` reports preserved state, five
source actors, checksum/LCD frame 144 and hash 3902769424. The people remain
readable at their expected positions and scale. These room views do not resolve
the arm corrections.

The upper-body studies show less elbow collapse, but also conspicuous angular
underarm recesses, pinched cloth and faceted Kimono Girl shoulders. Bare torso
transitions, jagged cuffs, raised-knee garment cuts and hair/jaw seams remain
unfinished. Passing shading/area limits does not establish collision-free
cloth or visual acceptance. The sampled running previews do not establish
complete animation quality; visual checks of jumping, sitting, surfing and
walking transitions remain open. The cast is intermediate and unapproved.

The native build and headless capture settings above were executed again.
The final review executable ran `--validate-arms`, `--validate-instances`,
`--validate-idle`, `--validate-walk`, `--validate-faces` and
`CRYSTAL_PEOPLE_REVIEW_DRAPE_TRIAL=live --validate-live-roster`. Additional
executed Cargo checks were:

```sh
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  CARGO_TARGET_DIR=target/people-motion-20261020 \
  cargo test --locked --jobs 2 -p crystal-voxel-view \
  --features people-review,actor-worker --lib arm_ \
  --config profile.dev.package.crystal-voxel-view.opt-level=1 \
  --config profile.dev.package.crystal-bevy.opt-level=1
CARGO_TARGET_DIR=target/people-motion-20261020/wasm-review \
  cargo check --locked --jobs 2 -p crystal-voxel-view \
  --target wasm32-unknown-unknown --features actor-worker-browser
```

The four supplied tests were then run with exact test names and `--ignored`
using the freshly built test executable; each ran one test and passed.
Diagnostic source snapshots, captured mesh fixtures and logs remain outside
the repository at `~/.codex/artifacts/people-volume-skin-diagnostics-2026-10-21`.
Generated captures, packs, mesh exports and compiled probes are not shipped.

## Connected arm roots and steering review (21 October 2026)

The shared body preparation now rounds the connected chest/arm junction after
the separate torso and sleeve operations. Two adaptive passes split coarse
edges while retaining dense painted panels. Texture-island copies remain
joined. Refinement includes the long lower chest transition, and the finished
root receives angle-weighted geometric normals before the existing arm field
is fitted. This changes the prepared body surface; unrelated regions, original
UVs and source rig transforms retain their authored values.

Each sparse arm blend now has an occupied surface center. Preparation computes
the exact integral of position times its linearly interpolated influence over
the triangles, so extra texture records and nonuniform triangle density do not
bias that center. Live updates retain the rigid dual-quaternion rotation and
make the center follow the weighted source-joint path. The browser worker
transports and validates these centers; actor response protocol version 3
rejects the preceding layout. The source animation palette remains the exact
prefix used by separate costume meshes.

The mounted pose also removes upper-arm axial roll inherited from combat Idle
and the steering solve. It constructs the arm swing from the bind reference to
the existing elbow, then restores the complete forearm world orientation.
Elbow positions and all hand/finger frames therefore retain their steering
contacts. A supplied-rig regression checks three rider scales across 32 varying
Idle/pedal phases against the uncorrected pose, with the existing saddle,
grip and pedal regression run separately.

The final native build passes all six arm, instance, idle, walk, face/budget and
live-roster gates. The unchanged arm limits are 0.05 outward agreement and
0.015 posed/bind area ratio. Across 12,320 production pose checks, the minima
are 0.1334427 and 0.023883335. Sparse resolution is at least 10 and the largest
palette uses 255 of 256 slots. The live garment gate passes all 25 garment
identities with experimental trousers disabled. These shading and area checks
do not establish collision-free cloth.

The filtered Cargo run passes 13 ordinary tests. Seven supplied tests were
then selected individually with exact names and `--ignored`; each ran one
test and passed: both arm-field protections, joined-root geometry, mounted
contacts, steering frame preservation, five real appearances through paged
transport, and the complete paged actor job. The worker header retains its
8,192-byte bound. Four lattice/anchor math regressions cover reconstruction,
rigid local radius, area weighting and the exact varying-influence integral.

A separate actual-source Rust probe reconstructs seven on-foot appearances
and all three riders from their freshly exported native fitting inputs. Over
128 captured production palettes per case (1,280 total), indexed positions,
normals, UVs, joints and weights match exactly, as do recipes and their surface
centers. Maximum helper-matrix element error is zero. The optional fitting
exports omit colors and cutaway flags, so the probe compares indexed triangle
corners rather than requiring identical compact vertex-record counts. The
final `actor-worker-browser` WASM target check passes. Browser visuals and
frame rate remain unverified.

The external review is
`~/.codex/artifacts/people-root-volume-2026-10-21/index.html`. It contains fresh
idle and Run 0.25 sheets for all 75 people, each with eight labeled directions;
Biker uses mounted poses. Six running loops and three riding loops each retain
eight captured phases and eight directions, with illustrative playback timing.
Ten matched shoulder studies retain the start-of-pass and current 1,000-pixel
views in every direction. Current mounted details and critical Run 0.375
profiles are included. Original native images, source snapshots and numeric
reports are retained in the sibling diagnostics directory outside the repo.
All 1,728 fresh full-body cells pass framing checks with at least 43 pixels of
margin. The gallery contains 383 PNGs, nine eight-frame WebPs, 81 HTML pages and
1,256 verified local references. Its 103,001,986-byte sibling ZIP passes CRC and
exact file-byte comparison.

Fresh headless noon room pairs use the existing local pack. `VioletGym` at
`(5, 3)` reports preserved state, five source actors, checksum frame 83/hash
3896312843 and LCD frame 83. `VioletPokecenter1F` at `(3, 4)` reports preserved
state, five source actors, checksum frame 57/hash 2567152420 and LCD frame 61.
The 1,280 by 800 images retain expected actor placement, orientation and scale;
these room views do not resolve the shoulder details.

The shoulder studies show smaller gaps and smoother joins. Deep rear arm-root
creases, pinched fabric, jagged cuffs, faceted bare torsos, raised-knee robe
cuts, dark Biker torso patches and hair/jaw seams remain visible. The sampled
cycles do not establish complete animation quality; jumping, sitting, surfing
and walking transitions still need visual review. The cast remains
intermediate and unapproved. All captures are offscreen and muted, with no
changes to content packs or audio.

The native build, six reviewer gates and Cargo/WASM commands in the preceding
section were executed again with the final source. Native comparison flags
`CRYSTAL_PEOPLE_REVIEW_ARM_SURFACE_BASELINE=1`,
`CRYSTAL_PEOPLE_REVIEW_ARM_CENTER_STRENGTH=0` and
`CRYSTAL_PEOPLE_REVIEW_BICYCLE_ARM_UNTWIST=0` are offline diagnostics;
production defaults use joined rounding, anchored centers and steering
untwisting.

## Garment stride, closing rims and control fitting (21 October 2026)

The production locomotion style now shortens steps for 24 closed-skirt/robe
identities. Long hems use 0.78 stride, 0.50 lift and 0.88 arm-swing factors;
shorter hems use 0.92, 0.72 and 0.94. The existing distance-driven phase uses
the same stride factor, increasing cadence while preserving authoritative
travel speed. Idle, facing, run/walk blend ownership and unrelated styles
retain their existing behavior. Falkner and the disabled separate-trouser
trial are excluded. The pinned source trainer and overworld sprites show
Falkner wearing separate trousers; his current robe is still an interim model.

All 25 closed garment shells now finish their fitted normals before separate
obi, apron or other authored details are added. Horizontal closing rims keep
separate normal islands from the vertical panels. Nurse retains its previous
normal preparation exactly. For six freshly exported representative appearances,
indexed bind positions, UVs, joints and weights are bit-identical to the
preceding version, as are complete body meshes and arm recipes. Duplicate
records for normal boundaries do not alter the indexed geometric surface.
The supplied regression checks all 25 bind rims, then 2,400 actual gait states
(25 identities, three run/walk blends and 32 phases) for noncollapsed triangles
and outward finite rim shading. The separate obi skin remains authored.

The stronger drape path remains a native opt-in experiment, disabled in
ordinary and browser play. Its local envelope now derives support planes from
exact projected hull edges at the existing 72 azimuths and 0.001 clearance.
The retained full-point support oracle checks the adaptive planes and radial
columns, including a wedge missed by the preceding fixed slope samples.
Local fields also avoid constructing the unused coarse leg cage.

A tighter detailed target initially exposed eight strict lower-leg crossing
pairs in Falkner's interpolated GPU surface at Run phase 9/31. An actual-source
target probe found zero pairs in the detailed target itself at that pose. The
experimental template now builds immutable area-weighted sparse normal
equations and fits control translations to the target with at most 24
preconditioned conjugate-gradient iterations. This retains the affine shading
frames and GPU joint count. Small diagonal regularization stabilizes dependent
or unused controls. Weights use triangle-corner lumped area; this is not an
exact integral of the continuous fitted field. A rigid-target regression
checks reconstruction and unchanged basis columns.

All seven final native reviewer gates pass: drape, arms, instances, idle, walk,
faces and live roster. The expanded final roster check includes horizontal
closing rims below the control region's top, alongside vertical panels. All
25 identities pass 825 sampled poses and their independent two-instance ECS
checks. Maximum detailed-target position error is 0.012897 and minimum
geometric/shading normal agreement is 0.193219. The position limit remains
0.035 and the lower-leg strict-crossing requirement remains zero. These
contact checks exclude coplanar/edge contacts, hands, waist and fabric
self-contact. The unchanged arm gate still covers 12,320 poses with minima
0.1334427 outward agreement and 0.023883335 posed/bind area ratio; the largest
palette retains 255 slots. These checks do not establish complete cloth
clearance or artistic acceptance.

The executed native build uses the same command and headless settings above.
The following filtered Cargo run passes six ordinary tests; five supplied
tests remain ignored by that filtered run:

```sh
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  CARGO_TARGET_DIR=target/people-motion-20261020 \
  cargo test --locked --jobs 2 -p crystal-voxel-view \
  --features people-review,actor-worker --lib garment \
  --config profile.dev.package.crystal-voxel-view.opt-level=1 \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

Three supplied tests were then selected individually with exact names and
`--ignored` using that freshly built executable. Each ran one test and passed:
`supplied_closed_shells_keep_separate_outward_hem_normals_through_motion`,
`five_real_appearances_preserve_all_mesh_and_skin_bits_through_pages`, and
`complete_actor_job_pages_restore_exact_production_appearance`. The paged
worker header remains within 8,192 bytes and protocol version 3 is unchanged.
The final `actor-worker-browser` WASM check with the preceding command passes.
The subsequent native-only validator extension changes no rendering code.
Browser visuals and frame rate remain unverified.

The external gallery is
`~/.codex/artifacts/people-garment-motion-2026-10-21/index.html`. All 25 closed
garment identities have fresh ordinary idle and eight-phase running sheets,
with eight labeled directions per sheet. Six matched before/current/drape
studies include 18 eight-frame loops and fixed-height 1,000-pixel garment
details. All 25 experimental outfits have Run 0.25 sheets, with complete
eight-phase studies for six representatives and a matched Falkner critical
pose. The preceding complete 75-person gallery remains a separate snapshot.
The new gallery contains 482 PNGs, 18 WebPs, 32 HTML pages and 1,113 verified
local references. Every animated body pixel matches its native capture. All
2,832 full-body cells, including baselines, pass framing checks with at least
68 pixels of margin. The 116,907,673-byte sibling ZIP contains 537 files and
passes CRC and exact file-byte comparison.

Fresh ordinary-production noon pairs preserve source state in `DanceTheater`
at `(6, 3)` (10 source actors, checksum frame 63/hash 4096693750, LCD frame 80)
and `VioletPokecenter1F` at `(3, 4)` (five source actors, checksum frame 56/hash
3513807126, LCD frame 60). Placement, orientation and physical scale remain
readable in the 1,280 by 800 pairs, but the room views do not resolve hems.
The theater reveals a separate fidelity defect: all five modeled performers
share a red outfit while source sprites use distinct palettes. Some audience
members are outside this camera. This remains unfinished.

Ordinary raised-knee cloth cuts remain prominent. Over six clipped/exposed
lower-body fixtures and 128 actual palettes each, strict crossing-pair totals
fall from 97,717 to 62,619; these are diagnostic pair counts, not a percentage
of collision-free poses or a complete-anatomy check. Reversed shading counts
fall to zero. The experimental surface covers knees better but flares into
rigid bell shapes; hand, waist and apron behavior and runtime cost remain
unaccepted. Shoulder creases, cuffs, faceted torsos and hair/jaw seams also
remain open. Jumping, sitting, surfing and walking transitions still need
visual review. No production drape promotion or complete-cast acceptance is
claimed. All captures are offscreen and muted; content packs and audio are
unchanged. Selected source snapshots, native fixtures, captures and verified
logs are retained outside the repository in the sibling
`people-garment-motion-diagnostics-2026-10-21` directory.

## Individual theater clothing palettes (21 October 2026)

Kimono Girl now participates in the existing per-instance source-clothing
path. The verified local pack contains five performers in `DanceTheater`:
two green, two blue and one red. Each actor derives its accent from its own
already paletted sprite. Painted sleeves and the fitted robe use the same
recolored fabric sample, retaining the painting's light/dark folds instead
of resampling the default red atlas for the skirt. The painting's cloth starts
at approximately v=0.47; including that collar-side strip removes red tabs
from the blue and green variants. Pale collars, pink blossom pigment, green
stems, neutral ink and protected skin/hair islands retain their authored
pixels. The separate obi and bow retain their original colors and geometry.
No new source palette catalog or texture asset is shipped.

The native build and headless capture settings in the preceding sections were
executed again. This filtered Cargo command passes 19 ordinary tests, with
six supplied tests ignored by that filtered run:

```sh
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
  CARGO_TARGET_DIR=target/people-motion-20261020 \
  cargo test --locked --jobs 2 -p crystal-voxel-view \
  --features people-review,actor-worker --lib uniform \
  --config profile.dev.package.crystal-voxel-view.opt-level=1 \
  --config profile.dev.package.crystal-bevy.opt-level=1 -- --nocapture
```

The actual-pack test covers all three Kimono Girl accents in four palette
banks and six sprite frames (72 cases), alongside the preceding seven service
and community roles. Four supplied tests were then selected individually with
exact names and `--ignored`; each ran one test and passed: service palette
cache preservation, affected-actor-only refresh, five real appearances through
paged transport, and the complete paged actor job. Kimono Girl's cache check
compares all non-color costume attributes with the default appearance,
preserves the entire obi/bow color suffix, and checks that the robe matches
the repainted body's plain patch. Body mesh, head mesh, inverse binds and
bounds remain shared. The actor test checks that an accent change refreshes
only that actor and ordinary frame updates keep its instance stable. The
final `actor-worker-browser` WASM target check passes; browser visuals and
frame rate remain unverified.

Actual 512-pixel painting comparisons preserve all 122,880 pixels in the
protected upper rows, every alpha byte, 1,304 identified pale-collar pixels
and 691 identified pink-blossom pixels. Blue and green collar strips retain
zero saturated-red pixels from the former 3,264-pixel red strip. Default
eight-angle idle, walking and running captures match the preceding executable
pixel for pixel. The immutable external painting is unchanged.

The native reviewer accepts `CRYSTAL_PEOPLE_REVIEW_UNIFORM_RGB` for supported
clothing roles and exports the actual recolored painting when requested.
Walking uses run blend zero. Running captures explicitly set both
`CRYSTAL_PEOPLE_REVIEW_RUN_BLEND=1` and
`CRYSTAL_PEOPLE_REVIEW_RUN_WEIGHT=1`. A draft initially mislabeled default
walking samples as running; it was rejected before delivery. Those samples
are retained as walking evidence, and every running phase was captured again
with the explicit production blend. The gallery verifier now checks these
recorded modes and requires the critical run pose to differ from walking.

The review is
`~/.codex/artifacts/people-kimono-palettes-2026-10-21/index.html`. Each of three
colors has idle plus eight walking and eight running phases, all with eight
directions. Six lossless loops retain 48 sampled frames with illustrative
playback timing. All 456 full-body cells, including default comparisons,
pass framing checks with at least 69 pixels of margin. Every labeled or
animated body pixel matches its native capture. The gallery contains 77 PNGs,
10 HTML pages and 190 verified local references. Its 25,395,941-byte sibling
ZIP contains 102 files and passes CRC and exact file-byte comparison.

Four fresh 1,280 by 800 production pairs preserve source state. At `(6, 3)`,
the theater noon pair reports 10 source actors, checksum frame 64/hash
4040650042 and LCD frame 81; 21:00 reports frame 312/hash 3385013444 and LCD
frame 334. The additional stage camera uses `1.2 0 60 0`, reports frame
257/hash 2935603447 and LCD frame 271, and changes no actor scale. The
unrelated `VioletPokecenter1F` pair at `(3, 4)` has five source actors,
checksum frame 64/hash 771357619 and LCD frame 68. The theater models visibly
retain their respective source color groups and expected placement. Audience
cropping and room-scale detail limits remain; the stage camera partly
occludes the outer performers against walls. The preceding before-palette
capture uses a different animation clock and is labeled accordingly.

This resolves the repeated-red theater outfit defect recorded above. It does
not resolve shoulder/arm-root gaps, faceted joins or raised-knee cloth cuts,
which remain conspicuous in the running views. The native drape experiment
remains disabled in normal and browser play. Complete animation and full-cast
artistic acceptance remain open. Captures are offscreen and muted; no content
pack or audio changes were made. Selected source snapshots, raw captures and
verified logs are retained outside the repository in
`people-kimono-palettes-diagnostics-2026-10-21`.

## Current cast angle renders (21 October 2026)

The requested fresh gallery is saved outside the repository at
`~/.codex/artifacts/people-all-angles-current-2026-10-21/index.html`, with a
sibling ZIP. It contains all 75 people in eight directions, at idle clock zero
and running phase 0.25: 150 labeled sheets and 1,200 captured views. Biker
remains mounted. Search, pose selection and individual-angle inspection are
available in the gallery; the full-resolution sheets preserve every native
body pixel. All view cells are populated, with at least 43 pixels of clear
margin. All 76 HTML pages and 1,052 local references were checked.

The review executable was rebuilt using the review/location feature build
above, then copied before capture so concurrent unrelated work could not
change the running renderer. Captures used `CRYSTAL_PEOPLE_REVIEW_HEADLESS=1`,
`CRYSTAL_PEOPLE_REVIEW_IMAGE_TARGET=1`,
`CRYSTAL_PEOPLE_REVIEW_ALL_ANGLES=1`,
`CRYSTAL_PEOPLE_REVIEW_IDLE_SECONDS=0`,
`CRYSTAL_PEOPLE_REVIEW_RUN_BLEND=1`,
`CRYSTAL_PEOPLE_REVIEW_RUN_WEIGHT=1` and `CRYSTAL_PREVIEW_MUTE=1`.
The external diagnostics retain exact commands, source and binary hashes,
unchanged artwork hashes, native captures and image checks.

These are current work-in-progress models, including the unaccepted elbow
rest-pose experiment. Its earlier arm sweep rejected eight identities, and
wide sleeves visibly intersect Kimono Girl's sash. Shoulder creases and
raised-knee cloth cuts remain. This render request introduces no new model or
animation changes and establishes no artistic, complete-animation, browser or
gameplay acceptance.

## Elbow planes through movement (21 October 2026)

Locomotion now carries each character's anatomical elbow hinge through the
shoulder rotation, flexes the elbow around that hinge and rolls the forearm
around its own length. The preceding world-X flexion left the authored elbow
plane whenever the standing manner or shoulder was rolled. Two local hinge
vectors are cached with the neutral anatomy; sampling does not reconstruct
three world-space joint positions for every elbow. Foot trajectories, travel
distance, standing manners and the existing hand follow-through are retained.

The uniform standing elbow swivel has been removed from normal play. Its
fixed wrist frames did not guarantee sleeve clearance: Kimono Girl's hanging
sleeves crossed the sash, and eight appearances failed the shoulder surface
sweep even with the revised forearm hinge. The swivel remains a native review
experiment through `CRYSTAL_PEOPLE_REVIEW_IDLE_ELBOW_TUCK`; its field and helper
are excluded from normal non-test/non-review builds and browser production builds.
All 75 fresh idle silhouettes now exactly match the pre-experiment captures.
The retained torso-carrier normal finishing changes shading without moving
the bind surface. Neither rejected underarm volume expansion is in production.

The complete edge probe for Chris, Nurse, Kimono Girl, Janine, Erika and
Granny found no non-two-use edges with both endpoints in
`0.17 < abs(x) < 1.34`, `1.78 < y < 2.58`, and no separated coincident copies
in the sampled idle and running poses. This region excludes the intentional
neck opening. A preliminary summary based on a truncated broader edge list
incorrectly counted neck edges for Janine and Erika; the corrected complete
probe supersedes it. Visible underarm background is silhouette clearance,
rather than a demonstrated torn seam. Unlit captures also distinguish painted
dress folds from geometric shading; that artwork remains intact.

A new supplied-rig regression measures both elbow planes over 12,000 poses,
all 75 people and five walk/run blends. Maximum `1 - dot(normal, rest_normal)`
is 0.00000023841858, below the 0.00002 limit. Replaying the previous rotation
with `CRYSTAL_PEOPLE_REVIEW_ELBOW_WORLD_AXIS=1` fails the same guard on Beauty's
first walking pose (error 0.008771837). The native arm sweep passes 12,320
production poses, retaining minimum outward agreement 0.1334427, minimum area
ratio 0.023883335, lattice resolution at least 10 and a maximum 255-joint
palette out of 256. Existing foot contact, bone length, floor, loop,
continuity, culling, instance and face checks pass. The isolated swivel study
still preserves hand frames over 2,400 poses, but is not visually accepted.

Cargo checks pass 13 ordinary arm-filtered tests and five exact supplied-rig
or fixture checks: elbow planes, swivel hand frames, five-appearance paged
transport, the complete paged actor job and the roof fixture. An unrelated
private-field compile error in that fixture was repaired by constructing
`TerrainMeshData::default()` before assigning its public footing heights;
its assertions and production behavior are unchanged. The browser worker
target compiles with the new hinge cache (28.45 seconds). These are native
data/transport checks and a browser compile, not browser visual acceptance.
The combined browser-worker/review feature build also passes (2 minutes
43 seconds); its native-only swivel audit is excluded on WebAssembly.

The current gallery is
`~/.codex/artifacts/people-elbow-motion-2026-10-21/index.html`, with a sibling
ZIP. It contains 150 fresh full-cast idle/Run-0.25 sheets, 1,200 populated view
cells, and at least 43 pixels of clear margin. Four people have eight-phase
walking and running comparisons in all eight directions. Eight composite
cycles keep previous/current phases synchronized; the 16 separate source
cycles are retained too. Every animated native body pixel is preserved.
Matched 1,000-pixel torso studies use fixed physical-height framing. The four
compared body exports and arm recipes agree for every exported attribute;
their authoritative pose palettes differ. These exports omit color and
cutaway flags. All 86 HTML pages and 1,342 local references were checked.

Seven fresh production source-2D/current-3D pairs preserve position, facing,
time and game state: theater noon/night, Pokémon Center idle, and two short
keyboard routes at two capture timings. The mid-step frames show the player
moving in the production renderer. The named location-review option
`CRYSTAL_RENDER_TEST_WALK_CAPTURE_TICK=4` selects that earlier frame; its
default remains 12 and the valid range is 0–31. It changes capture timing only.
Studio cycle playback is illustrative, and short room steps include ordinary
animation blending rather than forced full-speed poses.

Shoulder shapes, underarm recesses, jagged cuffs, raised-knee garment cuts,
unfinished tailoring and hair/jaw seams remain open. The sampled motion is
improved; full-cast artistic, complete-animation and browser-performance
acceptance remain unproven. Captures are headless and muted. No pack, audio or
unrelated game content was edited. Rejected studies, source snapshots, native
fixtures, commands and logs are retained outside the repository in
`people-elbow-motion-2026-10-21-diagnostics`.

## Continuous short sleeve cuffs (21 October 2026)

Red, Misty, Brock, Pokéfan M and Nurse now have continuous cloth/skin cuffs.
The preceding whole-triangle material selection left a stepped edge after
arm refinement. The final weighted surface is now split at the existing
`abs(x) = 0.74` sleeve cutoff, before compaction and the continuous arm
palette are prepared. Neighboring UV islands receive the same edge splits,
and material copies share identical positions and fitted skin weights.
Original vertices are retained; new edge vertices interpolate the existing
attributes. No influences are dropped to fit the four-slot skin format.
Newly assigned material samples reuse the original painting coordinates,
including refinement before rounding, rather than projecting the rounded
surface back onto the painting. The original artwork remains unchanged.

The supplied-rig cuff guard passes five people, ten closed loops and 1,590
shared material edges. Each edge has one cloth face and one skin face, all
vertices lie on the cutoff, and coincident copies have exactly matching
final joint IDs and weights. Running the same guard with the native review
control `CRYSTAL_PEOPLE_REVIEW_CUFF_BASELINE=1` rejects Red's previous stepped
boundary at `x = 0.7061703`. The synthetic regression additionally checks
neighboring UV islands, winding, covered area, vertex colors, hidden-body
flags and coincident copies under sixteen posed transforms. The original
short-sleeve source-paint regression, five-appearance paged transport and
complete paged actor-job regression all pass, for five passing Cargo checks.
A first source-paint invocation encountered a concurrent unrelated
background-test helper compile error; the completed rerun passes.

The five upstream body exports are byte-identical between the review control
and current preparation, now including colors and cutaway ranges. Source
joint matrices and arm recipe topology agree. Full bind-surface area differs
by at most `1.311e-10` relatively. The first four appearances add 532
triangles each; Nurse adds 1,052. Existing positions newly assigned cloth
where they previously carried only skin lie within
`0.7146592 <= abs(x) <= 0.7399887`, confined to the cuff band. The other 70
people retain identical idle and Run-0.25 silhouettes against the preceding
gallery. Some independent captures contain small RGB differences despite
unchanged exported data, so full pixel equality is not claimed.

The native arm sweep still passes all 12,320 production poses with minimum
outward agreement 0.1334427, minimum area ratio 0.023883335, lattice
resolution at least 10 and maximum palette 255/256. Face, movement and
instance checks pass. The combined `actor-worker-browser,people-review`
WebAssembly check passes in 15.16 seconds. These establish native data and
transport checks and browser compilation; browser performance and visuals
remain unverified. Rebuild the native review executables with the command
above, then run the additional guard with:

```sh
CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
  target/people-motion-20261020/debug/render_people_review --validate-cuffs
```

The current gallery is
`~/.codex/artifacts/people-cuff-finish-2026-10-21/index.html`, with a sibling
ZIP. All 75 people have fresh eight-direction idle and Run-0.25 sheets:
150 sheets and 1,200 populated view cells, with at least 43 pixels of clear
margin. Five cuff studies provide 80 matched torso comparisons across all
eight directions and both poses. Ten synchronized before/current walking
and running cycles preserve every native body pixel through eight phases;
their 160 source sheets contain 1,280 view cells and at least 68 pixels of
clear margin. Playback timing is illustrative. Four production source-2D/3D
pairs show Nurse and Brock in their respective rooms. Each pair preserves
position, facing, time and game state; independent before/current sessions
can have different animation clocks.

The cuff boundary is improved. Shoulder shapes, underarm recesses,
raised-knee garment cuts, unfinished tailoring and hair/jaw seams remain
open. Full-cast artistic and complete-animation acceptance remain unproven.
Captures are headless and muted. Source snapshots, exact capture commands,
native fixtures, logs and comparisons are preserved outside the repository
in `people-cuff-finish-2026-10-21-diagnostics`. No pack, audio or unrelated
game content was edited.

## Shoulder shading and standing balance (21 October 2026)

The shared shoulder surface now receives a bounded cotangent fairing before
arm refinement. Refinement still makes its edge decisions from the preceding
reference positions, preserving the painted splits and skin attributes.
The inner back fans now include their centerline corners. Smooth normals are
finished on the curved surface before linear subdivision interpolates them;
finishing them afterward had restored the original cage's large triangular
facets. The native `--validate-shoulders` guard includes a planar needle-fan
fixture and the actual centerline fan that the previous selection missed.

The backward-set standing posture came from centering the ankle joints under
the hips. In the supplied anatomy those joints sit near the heel: that change
advanced the whole shoe by approximately 0.18 source units. Standing and
movement now preserve each ankle's original depth. The pelvis stays within
both legs' reach during idle weight shifts; the largest confident-character
foot stagger is reduced from 0.045 to 0.030 reference units. Bone lengths,
physical character sizes, gameplay position and facing are preserved.

Walking settles from heel to flat sole, rolls onto the toe, then returns with
the shoe level before its next heel contact. Contact points keep the existing
distance-driven speed. The cached support profile comes from the final
uploaded body mesh, including adapted shoes and bare feet. It follows the
actual sole's convex profile and integrates rotations across toe bevels;
a fixed bounding box had left a visible gap under the shortened feet.
After pose blending, floor clearance is reapplied using the cached profile.
No mesh vertices are deformed on the CPU during animation. Upload derives the
same small immutable cache from native and worker-restored mesh attributes,
so this adds no worker wire fields or protocol change.

The sole regression includes a raised toe bevel that rejects the preceding
box approximation and checks contact, ankle continuity and absence of sliding.
The existing ground-speed check now observes actual sole contacts while the
ankle rolls. Standing extension, walking support extension, bone-length,
loop, floor-penetration and contact-speed limits are retained.


Fresh standing measurements cover all 74 prepared standing appearances
(Biker stays mounted) and place the hips and head between 0.4733 and 0.5644
of the finished sole depth, measured from heel to toe. The six matching old
stance controls put the hips below 0.30. Those controls retain byte-identical
body exports; only pose/contact behavior changes. The supplied source-rig
standing check also passes 1,800 poses across all 75 identities. The old
stance control rejects the same balance guard.

The native shoulder, cuff, face, walk, idle, arm, instance and live-garment
checks all pass. The live-garment check alone uses `DRAPE_TRIAL=live`; the
gallery uses the default production garments. The arm sweep covers 12,320
poses and 481,825,120 triangle samples, with minimum outward agreement
0.06835382, minimum area ratio 0.081675455, lattice resolution at least 10
and maximum palette 253/256. Finished-mesh floor checks cover 1,536 current
walk/run poses for Chris, Nurse, Kimono Girl, Brock, Janine and Swimmer Guy.
Maximum penetration is 1.539e-6 source units and maximum stance clearance is
3.848e-7, within the existing 0.003/0.005 limits.

The current review is
`~/.codex/artifacts/people-stance-2026-10-21/index.html`, with a sibling ZIP.
It contains 150 eight-direction idle/run sheets for all 75 people: 1,200
populated views with at least 43 pixels of clear margin. Eighteen matched
stance sheets compare idle, walking and running for six people. Twelve
synchronized walking/running cycles use 192 native phase sheets and 1,536
view cells; their lossless playback preserves native body pixels, while
playback timing remains illustrative. Three new production source-2D/3D
pairs show Violet Pokémon Center and Dance Theater at noon and night,
preserving position, facing, time and game state. Both side profiles of the
whole cast and all three production pairs were visually inspected.

The repeated heel-weighted pose is corrected. The production captures also
show remaining room-framing limits, and the studio studies still expose
underarm recesses, garment cuts, tailoring and hair/jaw seams. Full-cast
artistic acceptance, the complete animation set and browser performance
remain open. All captures are headless and muted. Paintings, packs, audio and
unrelated game content are unchanged by this work.

The review and build cache live outside the repository's `target/` directory.
The verified native build uses the following command; it disables debug
symbols to keep the review cache smaller:

```sh
CARGO_TARGET_DIR="$HOME/.cache/codex-people-stance-20261021/cargo-native" \
  cargo build --locked --jobs 4 -p crystal-voxel-view -p crystal-bevy \
  --features crystal-voxel-view/people-review,crystal-bevy/location-tester \
  --bin render_people_review --example render_at_location \
  --config profile.dev.opt-level=1 \
  --config 'profile.dev.package."*".opt-level=1' \
  --config profile.dev.debug=0
```

The combined `actor-worker-browser,people-review` WebAssembly check passes
using a separate `cargo-wasm` cache, one build job, the same profile settings
and `--target wasm32-unknown-unknown`. This confirms compilation only.
Fresh commands, source snapshots, logs, controls, geometry measurements and
capture data are preserved outside the repository in
`people-stance-2026-10-21-diagnostics`. The initial repository `target/`
directory disappeared during work; the delivered captures and validation
records were regenerated in the external cache. Earlier rejected prototype
raw data could not be preserved.

Seven fresh Cargo test selections pass after the rebuild: supplied standing
balance; sole speed/rolling/bevel support; ordinary arm helpers; joined UV
boundary bending; supplied shoulder preservation; five real appearances
through paged transport; and the complete production actor-job page path.
The two transport checks preserve the finished mesh and skin attributes.
Exact invocations and full results are in the external diagnostic archive.

## Individual long hair and closed bob rims (21 October 2026)

Beauty, Clair, Daisy, Jasmine, Karen, Rocket Girl and Sabrina now have distinct
hair lengths, tapers, rear sweeps and ends. Jasmine also has paired raised
gathers with ivory bands. Crystal's front, back and profile sprites, walking
frames where available, and stored trainer pictures guided the silhouettes.
The inspected source is the external `pret/pokecrystal` checkout at
`3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6`; reference paths and hashes remain
in the external review archive. Existing stylized paintings supply the colors.

`hair_curtain` appends the new hair after the body is fitted. Roots follow the
head and lower sections blend smoothly toward the existing upper-back joint.
The neutral shape compensates for head scaling, and a small radial cache of
the finished scalp fits the buried roots. Surface normals follow the presented
shape through that scale transition. The cache conservatively smooths small
scalp bumps without exposing their source triangles. Existing hair pigment
is sampled into vertex albedo; a protected neutral portrait texel prevents the
body material from tinting it a second time. No painting or texture is added.

Each cut adds 5,570 vertices; Jasmine adds 6,492 including her gathers and bands.
They reuse the current skeleton and ordinary GPU skinning. Sampled source,
idle and generated gait poses determine conservative culling bounds. There
are no new worker fields, joints or per-frame vertex deformation. All original
prepared body attributes, skin weights, triangle indices, cutaways and arm
recipes remain exact in the seven matched old-cut controls.

The separate short-bob helper also had inward closing rims. Those quads now
face outward, with duplicated crease normals. This corrects Bugsy, Erika, Mom,
Receptionist and Link Receptionist without changing their cut or pigment.
Both columns in the long-hair comparison retain this rim correction; the
left column is labeled **Previous cut control**, not an untouched baseline.

The supplied native hair guard passes 1,960 poses across all seven identities,
including closed oriented topology, finite weights, normals, carrier ownership,
vertex budgets and culling. Minimum outward normal agreement is 0.701523.
The rigid-cut control fails the same guard. A separate prepared-mesh check
observes 1,792 actual `Person::update` palettes: roots stay fixed to the head
and tips move relative to it. Rear-surface rays below the nape cover 336 poses
against actual body/costume triangles, with minimum clearance 0.10015 source
units. Buried scalp and gather roots are excluded from that clearance claim.

Eight focused Cargo test selections pass: the scale-transition normal,
occupied-scalp fit, material pigment multiplication, ordinary and painted
closing rims, supplied cuts, three long-haired appearances through paged
transport, and the existing five-appearance transport check. The native
shoulder, cuff, face, walk, idle, arm and instance guards also pass. The combined
`actor-worker-browser,people-review` WebAssembly check passes; it establishes
compilation only. The freshly rebuilt renderer exports exactly the same seven
prepared models and all 1,792 palettes as the capture build.

The required workspace formatting check still reports widespread pre-existing
differences; the new leaf modules, worker transport file and changed legacy
hair functions are formatted. Required all-target Clippy encounters existing
`bad_bit_mask` and approximate-constant errors outside this change. The affected
crate's all-target check with `--no-deps`, allowing only its existing
`approx_constant` and `erasing_op` lints, passes with no diagnostics in the three
new hair modules. No persistent lint exceptions were added.

The new review is
`~/.codex/artifacts/people-hair-flow-2026-10-21/index.html`, with a sibling ZIP.
It includes all 75 people in 150 eight-direction idle/run sheets, 112 matching
native hair close-ups, six lossless synchronized walk/run comparisons, and
four fresh production 2D/3D pairs for Karen, Jasmine, Beauty and Erika. All four
pairs preserve position, facing, time and state and contain the intended source
character. All affected idle/run orbits and the seven close-up orbits were
visually inspected. Native body pixels are preserved in labeled sheets and
motion frames; close-up overview sheets explicitly identify their 50% scale.

Cross-build pixel identity with the previous gallery is not claimed: the 126
unaffected sheets have at most 558 changed pixels per 1,680,000-pixel image.
Separate Chris, Nurse and mounted Biker controls preserve every exported mesh
attribute and the sampled production palette exactly. All 75 painting hashes
are unchanged. Commands, source snapshots, raw captures, rejected prototypes,
checks and the small raster differences are retained in
`people-hair-flow-2026-10-21-diagnostics` outside the repository.

This improves the hair silhouettes and their movement. Front-hair transitions,
jaw profiles, garment seams and cuts, finer strands, room framing and gameplay
facial readability still need work. Full-cast artistic acceptance, the complete
animation set and browser performance remain open. Captures are headless and
muted; no packs, audio or unrelated game content were changed for this pass.
