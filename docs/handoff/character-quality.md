# Human character quality checkpoint

## Original goal and completion status

Make every 3D player and NPC character hand evaluated and improved, with clean
UV maps, readable anime styling and smooth, clear models. Scope is all 75 human
identities, players, bicycles and their production render contexts. The goal is
**unfinished**. No whole character is approved. Local render improvements and
passing narrow tests must not be described as completion of the roster.

## Current implementation

The production path uses the supplied human/skater rigs and required per-identity
paintings; strict missing/malformed asset checks reject substitution. Character
work includes flatter drawn anime faces, fitted head/hair/cap shapes, tailored
front/back clothing UVs, refined joined hands, resting finger poses, modeled
player laces and UV seams, animated bicycle pedals and hand/foot attachments,
conservative animated bounds, and an offline reviewer exporting the actual GPU
body, costume and palette. These changes have different verification states.

The latest local Nurse work replaces the oval cap and raised blob cross with a
folded crown and flat red cross, maps exposed calves to skin, rounds the sleeve
and forearm together, and rebuilds fitted garment normals with distinct flat
rim normal islands. Her complete model remains unfinished. The waist has a
bright band from a short skirt flare; the proposed broader fitting fade is only
an external numeric prediction, not a source change. Current fit radius grows
from .27094 at y=1.62 to .33438 at y=1.47938 while the body barely changes.

The last source change gives Nurse a .40 shoulder-weight span (x=.25–.65) so
it ends where the elbow blend begins. Other exposed-arm roles retain .50.
Fresh muted native build passed (11m22s), and all five capture jobs passed.
All five images were individually reviewed; the underarm remains too square
and is unapproved. The arm regression was intentionally interrupted at the
user request for immediate handoff; no completed test result is credited.
Idle GPU checks preserve positions, normals, indices and UVs exactly; 929 skin
vertices change, maximum weight-sum error is 7.45e-8, and 2,154 selected arm
faces have zero opposed normals (minimum agreement .1469025). This is an Idle
check only, not whole-model approval.

## Verified evidence and failures

- Nurse folded cap: fresh native build and seven capture jobs passed. All 13
  images individually reviewed. Ordinary cap regression ran 1 test and passed.
  Body/pose/costume exports match the preceding Nurse/Cal/Youngster baseline.
- Nurse exposed calves: fresh build and five capture jobs passed; all five
  images reviewed. 372 UV corners change; per-face positions/normals/joints/
  weights exact. Costume and pose exact. No changed corner above y=.785.
- Nurse fitted normals: fresh build and first four jobs passed. Fifth side PNG
  saved and reviewed but process stalled after window closed. Main-thread sample
  showed a condition wait; only its verified task-owned PID was stopped with
  SIGTERM. Fifth job is -15, not a pass. All five images reviewed. All 4,000 cloth
  faces now have outward normal agreement, minimum .8175876. Per-face geometry,
  UVs and skin are exact; body/pose exact. Bright waist band remains unfinished.
- Nurse earlier arm variants: full shoulder blend rejected visually for square
  underarm and failed Run15/32 (-.03778). Elbow-only preserved shoulder contour
  but fresh test failed Idle0/32 (-.9502461). Current shortened-span candidate
  is not covered by those old results. The original exported Idle model also
  contains the same severe shoulder fold; preservation is not acceptance.
- Current retained hand geometry: fresh full-cycle mounted grip/pedal,
  resting-pose and white-glove checks passed in the terminal-domain review.
  The aggregate four-rig digit shape test ran 1 test and failed with 52 defects:
  32 lateral-cap and 20 depth-cap checks. All preceding finite/skin/closed-edge
  and 3 clips x61 phase bounds assertions completed for all four rigs. No hand
  approval. The fixed lateral .015 criterion may imply a sharper tip than a
  circular cap at actual finger scale; independently reassess shape and actual
  renders rather than simply lowering a threshold to obtain green tests.
- Strict loading: four negative checks each ran 1 actual test and passed:
  missing production rigs, missing native rigs, malformed optional people and
  incomplete identity paintings. Loading correctness is not visual approval.
- Player shoes/laces: measured upper attachment and UV protection checked;
  multiple running/mounted views inspected. Heavy faceted sole/heel/collar
  still unfinished. Mounted contacts passed before latest Nurse-only edits.
- Will tailored red front/back atlas and white glove UVs: rendered and protected
  fields checked. Lower crown rendered in Idle/Run .125/.625, head-only changes
  verified and crown preservation test passed. Whole Will still unfinished.
- Kurt/KurtOutside hair/nape improved; robe clearance still fails 6/32 poses.
  A straight-knee trial was rejected for stiff gait and floor penetration.
- Historical baseline/motion review covered all 75 identities. This does not
  approve any complete model or establish current whole-game context quality.

## Owned and shared source

Owned character source/art/review documents are listed in the checkpoint commit.
Shared integrations require the coordinating commit: `src/lib.rs` module wiring,
`src/new_bark_actors.rs` bike/actor hooks, `src/open_people_skin.rs` production
upload/pose/render hooks, and browser/native mute defaults. `src/bare_body.rs`
contains another worker's swimmer/torso work plus this thread's Nurse branch and
arm regression extension; retain all of it in the shared integration checkpoint.
`src/footwear.rs` has separately owned bare-foot contour/arch edits. Only this
thread's player palette, plain shoe UV, modeled lace and player test hunks belong
to this checkpoint. Do not stage other renderer, terrain, creature or pack work.

## New-host dependencies and restart

Use the coordinating `scripts/cold-start-3d.sh` and `docs/cold-start-3d.md` for the
Git bundle, separate Git LFS objects and ignored local content packs. Both people
GLBs and character PNGs use Git LFS; hydrate actual objects, not pointer files.
The four required rigs are under `assets/open-models/people/kenney/`, paintings
under `assets/open-models/people/npc-textures/`. Production source builds the
strict people bundle from them. Never commit ignored packs or copy old compiled
browser products into source. Rust toolchain/Bevy dependencies come from the
repository toolchain and lockfile. A native GPU/window environment is required
for screenshot review; numeric asset tests alone do not replace viewing renders.

Old-host captures/logs/diagnostics are under
`~/.codex/artifacts/anime-character-review/pending-model-review/`. They are
ignored external evidence, not runtime dependencies or committed deliverables.
The isolated-garment-target cache and session IDs are also old-host only.
ImageGen outputs used to produce canonical PNGs are not required to run the app.
Use ImageGen for any further bitmap edits; do not edit textures with Python.

Run from the new checkout (output outside tracked source):

```sh
export CRYSTAL_PREVIEW_MUTE=1
export CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models"
review_dir="$(mktemp -d)"
cargo build --locked --profile wasm-dev -p crystal-voxel-view \
  --features people-review --bin render_people_review
CRYSTAL_PEOPLE_REVIEW_IMAGE_TARGET=1 \
  ./target/wasm-dev/render_people_review "$review_dir/nurse-idle" nurse
CRYSTAL_PEOPLE_REVIEW_IMAGE_TARGET=1 \
  ./target/wasm-dev/render_people_review "$review_dir/nurse-run125" nurse 0.125
CRYSTAL_PEOPLE_REVIEW_IMAGE_TARGET=1 \
  ./target/wasm-dev/render_people_review "$review_dir/nurse-run625" nurse 0.625
cargo test --locked -p crystal-voxel-view --features people-review,crystal-assets/test-fixtures \
  supplied_exposed_arms_keep_shading_on_the_outward_surface -- --ignored
cargo test --locked -p crystal-voxel-view --features people-review,crystal-assets/test-fixtures \
  supplied_digits_preserve_wrists_attributes_and_animation_bounds -- --ignored
cargo test --locked -p crystal-voxel-view --features people-review,crystal-assets/test-fixtures \
  mounted_production_hands_and_shoes_fit_grips_and_pedals_through_full_cycle -- --ignored
```

These restart commands use the repository's current feature declarations and
new default target directory; the old evidence used additional cached Bevy
features and an external target. Confirm actual test counts and successful
compilation on the new host. Do not credit zero tests or a stale test binary.

Reviewer options: `CRYSTAL_PEOPLE_REVIEW_SINGLE_VIEW=0..3` for front/three-quarter/
side/back; `CRYSTAL_PEOPLE_REVIEW_DETAIL=hands|feet` with single view; bicycle mode
`CRYSTAL_PEOPLE_REVIEW_BICYCLE=1` for players; actual GPU diagnostics via
`CRYSTAL_PEOPLE_REVIEW_EXPORT_MESHES` pointing to an external output directory.
Final numeric argument is normalized animation phase. Inspect each PNG manually.
No current hand/shoe/Will/Nurse edits have been deployed to the served browser;
its old engine is not evidence for the checkpoint. Rebuild the complete browser
engine/glue/strict bundle/gzip/index together after stable native verification.

## Next work

First inspect the final shortened-shoulder result below and reproduce its checks.
If it fails or its silhouette is bad, fix the continuous shoulder/elbow field
without hiding the failure. Preserve hand/neck UVs and skin boundaries. Then
smooth Nurse's skirt fitting fade while preserving waist attachment; verify
full angles, motion and surface orientation. Improve her shoes, skirt gait,
jaw/nape and buns. Continue the full 75-identity roster with actual rendered
head/body/hair/clothing/hand/foot evaluation and production context checks.
No goal completion should be claimed until the original full scope is verified.

## Final checkpoint results / commit

Verification stopped at the immediate handoff instruction. No further
improvements or tests are running in this task. Commit ID is in the final
response; this commit contains the durable note. Checks above cover the full
shared working tree, not this partial ownership checkpoint alone. The
coordinator must commit remaining shared integrations before a standalone
new-host build. Keep audio muted. The full character goal remains unfinished.
