# Human character quality checkpoint

## Scope and acceptance

The goal remains unfinished. Scope is all 75 human identities, both players,
clothing, hands, shoes, bicycles and production contexts. No whole character is
approved. Passing geometry tests and isolated renders establish only the checks
actually performed.

Accept each character only after inspecting its head, body, hair, clothing,
hands and feet from all angles in Idle and motion. Inspect players mounted
through the bicycle cycle. Compare production 2D and 3D at the same position,
time, facing and game state. Evaluate silhouette, anime styling, UV/material
readability, shading, scale and contact with scenery. Preserve authoritative
controller state and unrelated content.

## Completed implementation and verification

The production path uses the supplied human/skater rigs and required identity
paintings. Strict missing/malformed asset checks reject substitution.

- Nurse has a folded cap, flat red cross, exposed-leg skin UVs and a gradual
  skirt fitting fade. Retained leg geometry continues above the neutral hem;
  fitted skirt weights are smoothed below the pinned waist and shared across
  the rolled hem. The all-role Run winding, animated waist fit, closed garment
  body preservation and Nurse 61-phase leg continuity tests each ran one test
  and passed. Fresh muted GPU Idle, Run .125 and Run .625 renders were inspected
  in front, quarter, side and back views. The latest skirt improvement is
  retained; this does not approve her entire character.
- Hands have refined joined fingers, fuller tips, angle-weighted normals and
  role-aware bare/gloved UVs. Attribute, topology, motion bounds and material
  regressions passed. Bare-hand mapping now crosses adjoining wrist faces,
  removing the painted crease band while preserving authored glove cuffs.
  Three material tests passed, covering Chris, Kris, Nurse, Will and dark glove
  roles; the full mounted-contact and digit/bounds regressions also passed.
  Fourteen fresh Idle/Run close-ups were individually inspected. Geometric
  wrist transitions and overall hand shape remain unfinished. Fingertip caps
  now use a rounded arc scaled to each finger's measured radius. The four-rig
  cap/topology/attribute/61-phase bounds regression and full mounted-contact
  regression pass. Fourteen fresh cap close-ups and four bicycle phase sheets
  were individually inspected; the rounded cap improvement is retained.
- Chris, Kris and Nurse shoes have a slimmer fitted footprint, preserved ankle
  seams and floor height, plain authored palette samples and modeled laces.
  Shoe geometry, albedo protection, lace attachment and mounted production
  hand/shoe contact regressions passed. Running and mounted renders were
  individually inspected. Other roles retain their authored shoes/bare feet.
- Swimmer feet now refine adjacent boundary triangles along with the foot,
  and continue the production ankle skin sample instead of a separate hand
  patch. Both the skin/floor/pinned-ankle regression and production ankle-palette
  regression passed. Eight fresh Idle/Run close-ups were individually inspected.
  The continuity changes are retained; toe shapes, heel/ankle contours and
  shading remain unfinished, and neither swimmer is approved.
- The reviewer removes only its expected previous PNGs, propagates Bevy's exit
  result and requires every expected capture to exist. A failed native render
  returns failure and cannot credit an old image. Successful fresh DX12
  captures verified this path on the Windows host.
- Clair's declared cape now uses the existing closed cape renderer, with her
  authored blue palette and a shorter hem. Lance retains his existing red cape.
  The production-dispatch, closed-edge, outward-normal and 61-phase animation
  bounds regression passed for both rigs. Six fresh Idle/Run sheets were
  individually inspected from front, quarter, side and back. Clair's cape
  addition is retained; her complete character is still unapproved.

## Remaining acceptance work

Nurse still needs cleaner underarms, hands/wrists, hair/buns, jaw/nape and overall
styling. Kurt/KurtOutside robe clearance remains unresolved in motion. Their
contact regression now checks both identities with actual refined shoes over
32 Run phases and also checks the shoe floor. Any replacement deformation must
also preserve outward panel winding and animated waist fit. Continue
individual evaluation and improvement across the complete roster; earlier
75-identity baseline inspection does not approve current models.
The latest 75-identity Idle gallery was freshly generated with the rounded
caps and every sheet was individually inspected in four views. Remaining
visible issues include exposed shoulder/underarm joins, angular bare feet,
sparse hair coverage and clothing seams. The declared canes for Elder,
Gentleman and Pryce, and KimonoGirl's fan/hairpin, have no renderer implementation.
Motion review, fixes and production comparisons are still required before
roster acceptance.

Production comparison remains incomplete. A fresh classic SilverCaveRoom2
capture displays a ball-like player at both (15,18) and adjacent (14,18), while
the pack's mounted Chris source sheet depicts the correct human. Loaded pack
checks establish floor collision and Normal mode at both positions. The actual
classic player-entity image association regression passes after map objects
populate the image store, and New Bark renders Chris correctly. Resolve the
remaining cave GPU presentation mismatch before using these captures as
character fidelity evidence.
The shared pack is still explicitly unvalidated; a successful character render
must not be described as full pack playability or title-flow certification.

## Successful local review procedure

Hydrate Git LFS rigs and paintings under assets/open-models/people. Keep all
content packs ignored and external to tracked source. Use a separate Cargo
target directory, leave other goals' caches and changes intact, and keep audio
muted. The pack/export goal owns the external exporter.

On this Windows host, use PowerShell:

```powershell
$env:CRYSTAL_PREVIEW_MUTE = '1'
$env:CRYSTAL_OPEN_MODEL_ROOT = Join-Path (Get-Location) 'assets/open-models'
$env:WGPU_BACKEND = 'dx12'
$env:CRYSTAL_PEOPLE_REVIEW_IMAGE_TARGET = '1'
cargo build --locked --target-dir target/character-quality --profile wasm-dev `
  -p crystal-voxel-view --features people-review --bin render_people_review
# Set $reviewDirectory to an absolute directory outside tracked source.
& target/character-quality/wasm-dev/render_people_review.exe $reviewDirectory nurse
& target/character-quality/wasm-dev/render_people_review.exe $reviewDirectory nurse 0.125
& target/character-quality/wasm-dev/render_people_review.exe $reviewDirectory nurse 0.625
cargo test --locked --target-dir target/character-quality --profile wasm-dev `
  -p crystal-voxel-view --features people-review,crystal-assets/test-fixtures `
  supplied_closed_garment_panels_keep_outward_winding_through_run -- --ignored
```

Use distinct output directories for each view/phase so evidence remains fresh.
Reviewer single-view options are CRYSTAL_PEOPLE_REVIEW_SINGLE_VIEW=0..3,
CRYSTAL_PEOPLE_REVIEW_DETAIL=hands|feet and
CRYSTAL_PEOPLE_REVIEW_BICYCLE=1 for Chris/Kris. Inspect each PNG manually.
Use ImageGen for bitmap edits; continue geometry and animation work in Rust.
Do not commit screenshots, logs, scratch outputs, generated packs or binaries.
