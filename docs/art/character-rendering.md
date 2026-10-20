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
The complete art pass remains open: running hands still read as open palms,
and individual head proportions, hairstyles and garment silhouettes need
further work. Skirts still intersect raised knees in Run. Passing
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

## Validation of legacy human removal

The regression exercises all 77 resolved human source identities through the production weighted body/costume renderer and passes. Actor lifecycle, motion and prop checks pass (16 checks), mandatory-rig failure checks pass (2 checks), and bicycle triangle normals pass. The browser target compiled with mandatory person bundle loading. These checks cover renderer integration; they do not establish that the character art is approved or that gameplay is playable.

The subsequent Chris anime head refinement has its own passing regression for UVs, skinning, non-head body positions, accessory anchors and finite normals. Actual native four-angle idle/run renders and local-pack room screenshots are recorded in the [current art direction](stylized-character-faces.md). The rest of the character art remains intermediate and unapproved.
