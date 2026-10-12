# Map scenery and canonical Pokemon restart

Checkpoint: October 12, 2026. Source checkpoint: `0a54f7c` (shoreline ecology,
opt-in scenery alpha tracing, and `docs/art/map-interest-review.md`). This note
is committed separately; locate its commit with
`git log -1 -- docs/handoff/map-scenery-and-canonical-pokemon.md`.

## Goal and current priority

The original goal is to make every Pokemon model look amazing, using openly
available rigged models where suitable. Shiny support is required. Subsequent
direct instructions prioritize making all maps visually interesting instead
of further Pokemon sculpting. All previews must remain muted. Neither full
goal is complete. The transfer request stops new improvements in this checkout;
resume development only after the handoff is complete.

The local verified pack has 388 maps in 35 tileset families. Nine unique maps
have earlier native image captures: LakeOfRage, IlexForest, Route42,
CherrygroveCity, UnionCave1F, IcePath1F, PlayersHouse1F, NationalPark, and
CherrygrovePokecenter1F. These are selected views, not whole-map coverage.
Several early captures omitted the canonical model root and cannot establish
canonical tree-art quality. Later Lake and forest captures used it explicitly.

## Implementation, ownership, and last action

This pass owns `crates/crystal-voxel-view/src/mesh/shore_ecology.rs`: sparse,
location-seeded reeds, freshwater seed heads, and notched lily pads inside
source-authenticated water cells beside verified lawn or natural banks.
Regular and Bug-Catching Contest National Park share a location seed. Paths,
room pools, unresolved art, claims, collision, support heights, and simulation
time are preserved. The final version retains nine shoots and three pads per
eligible cluster. A later count-variation experiment was reverted because its
verification had not finished when the transfer checkpoint was requested.

This pass also added Forest ground/ledge finishing and narrow renderer
diagnostics in shared files. The coordinating chat is responsible for capturing
their remaining shared integrations; do not stage these files wholesale as this
pass's exclusive work:

- `crates/crystal-voxel-view/src/mesh.rs` and `mesh/build_scene.rs`: shoreline
  module registration and append call.
- `crates/crystal-voxel-view/src/mesh/new_bark.rs`: source-verified Forest
  ledge caps, earthen relief, and integrated regressions. Existing
  `mesh/forest_floor.rs` and `mesh/ledge_finish.rs` provide the floor detail.
- `crates/crystal-voxel-view/src/mesh/shore_blend.rs`: shared shoreline finish;
  this pass's guard handles excluded maps with empty candidate indexes and
  returns the original color. Its regression covers Forest, cave, and gate.
- `crates/crystal-voxel-view/src/lib.rs`: opt-in pipeline-key/fragment-def trace.
- `crates/crystal-bevy/src/bevy_shell/render_capture.rs`: opt-in actual RGBA
  readback preservation; default output remains RGB.

The committed `scenery_surfaces.rs` change logs material alpha modes only when
`CRYSTAL_VOXEL_PIPELINE_TRACE` is present. It does not change material behavior.
Redo Outside owns shared tree geometry/dispatch and shoreline terrain.
Lighting/effects owns shader, foliage alpha, water appearance, and performance.
Do not send further coordination messages to Redo Outside: that authorization
was revoked. Preserve all owners' current work.

Earlier Pokemon work includes Tangela and Kangaskhan sculptures in
`crates/crystal-voxel-view/models/battle_species/` and their generators/tests in
`tools/{tangela,kangaskhan}_sculpt.py` and `tools/test_*_sculpt.py`.
The Omanyte candidate generator exists, but its latest candidate was not
visually approved. Do not treat a generator or geometry check as art approval.
Canonical routing is in `battle_species_models.rs`, `johto_actor_props.rs`,
`battle_view.rs`, `new_bark_actors.rs`, and `encounter_terrain/water_stance.rs`.
Shiny presentation is also in `pokemon_appearance.rs` and the render API.
The current dedicated-body catalog assertion is 225. Previous 251 claims
included source-icon substitutions and are superseded. Three actual starter
rigs have direct canonical registration. Missing bodies retain original 2D
source art; do not reintroduce substitute 3D models. No new external rigged
asset was acquired in this scenery pass.

Last action: stopped this pass's still-live count-experiment test build
(exit 130), reverted only those experiment edits, and committed the previously
verified shoreline implementation and review as `0a54f7c`. No task-owned
render or verification process remains active. No new improvement cycle is
needed to finish this handoff.

## Verified evidence and defects

Old-host evidence lives in ignored `target/map-interest-review-20261012/` and
is not a deliverable or prerequisite on the new host. The committed review
records its provenance. Build caches and captures must not be committed.

Verified on the old host:

- `cargo test --locked --profile wasm-dev -p crystal-voxel-view --all-features
  shore_ecology::tests`: two passing geometry/containment/ownership tests,
  including three place seeds and four bank directions.
- The same Cargo test configuration with filter `shore_blend::tests`: four
  passing tests, including the excluded-map crash regression.
- The production Forest ledge filter `forest_ledge`: two passing integrated
  tests preserving native corners, floor contacts, and claims.
- Canonical routing filter `exact_visible_species_tests`: five passing tests;
  bounded reusable geometry covers the 225-body catalog, not visual quality.
- A fresh native location-renderer build completed in 5m45s. Explicit
  image-target, canonical-root, muted RGBA IlexForest capture at `(15,27)`
  completed with exit 0 in 25.24s at 2560x1600. Masked forward and prepass
  pipelines both received `MAY_DISCARD`; a missing discard flag is not proven.
- A same-executable `zero-radius` reveal comparison completed with exit 0 in
  86.33s. It left the gaps and obscured the player more. Baseline fully
  transparent pixels: 455,879; zero-radius: 517,742. Removing player reveal
  is not supported as a fix. Tree placement already emits ground backing;
  source inspection does not justify adding a second floor.

Failed or incomplete:

- Forest images execute but fail art review: foliage gaps and excessive
  overlapping canopies remain. RGBA evidence shows most apparent black
  backgrounds are fully transparent final-frame regions; exact cause is
  unresolved. Do not label them opaque pixels from an RGB export.
- Default window-target Lake capture was all black and exited with a blank
  frame error. Separate image-target success does not fix or verify it.
- An initial Forest attempt panicked on empty shoreline candidates. The
  guard and subsequent RGBA capture verified the repair.
- GoldenrodPokecenter1F fails before capture on an authored receptionist at
  `(16,8)` outside a 10x8 map. This is present in the pack; a callback moving
  the actor was not established. Do not bypass validation without a proper
  source/visibility-aware regression.
- `python3 tools/check-actor-props.py` previously failed on refined
  `icon_clefairy` at 38,760 triangles against the existing 25,000 limit.
  No passing full actor audit is claimed; do not blindly weaken the checker.
- The reverted cluster-count experiment's Cargo build was canceled, not a
  test pass or failure. Initial trace compilation also encountered another
  owner's missing cave imports; those were fixed before the successful build.
- All-map, browser, ordinary window, gameplay, every-species art/rig, and
  full shiny coverage remain unverified. A screenshot/dump is not gameplay
  evidence. Earlier normal/shiny captures do not prove current global coverage.

## Dependencies and exact restart commands

Follow `AGENTS.md`, `docs/cold-start-3d.md`, and the coordinating chat's
`scripts/cold-start-3d.sh`. Transfer the Git bundle, Git LFS objects, and ignored
content packs separately as that document describes. The actual pack used was
`content-packs/realtime-clock.browser.crystalpack`; never commit it or rebuild
it from the incompatible legacy export workspace as an assumed success.
Restore real LFS artwork, not pointer files. No source audio/catalog duplication,
PCM export, ROM, external exporter, or extra clock is required by this pass.

From the repository root on the new host:

```sh
export CRYSTAL_PREVIEW_MUTE=1
export CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models"
export CRYSTAL_PACK="$PWD/content-packs/realtime-clock.browser.crystalpack"
export CARGO_BUILD_JOBS=2
scripts/cold-start-3d.sh check
```

That script's check is pack/art validation, not a GPU or gameplay pass. Then
rerun the focused production tests against the fully integrated transfer:

```sh
export CARGO_TARGET_DIR="$PWD/target/3d-handoff-verification"
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_WASM_DEV_DEBUG=0
cargo test --locked --profile wasm-dev -p crystal-voxel-view --all-features shore_ecology::tests
cargo test --locked --profile wasm-dev -p crystal-voxel-view --all-features shore_blend::tests
cargo test --locked --profile wasm-dev -p crystal-voxel-view --all-features forest_ledge
cargo test --locked --profile wasm-dev -p crystal-voxel-view --all-features exact_visible_species_tests
cargo test --locked --profile wasm-dev -p crystal-voxel-view --all-features exact_species_props_share_canonical_bodies_without_icon_substitution
python3 tools/check-actor-props.py
python3 tools/check-battle-species.py
cargo build --locked --profile wasm-dev -p geothite -p crystal-bevy -p crystal-voxel-view -p crystal-render-api --all-features --example render_at_location
```

The two Python commands are new-host rechecks, not claimed passing full audits
at this checkpoint. Inspect actual failures. Do not run asset generators over
approved canonical files without reviewing their output first.

Reproduce the forest diagnostic using the newly built executable:

```sh
mkdir -p target/map-interest-resume
CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_CAPTURE_CHANNELS=rgba \
CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=300 \
CRYSTAL_VOXEL_PIPELINE_TRACE=1 \
"$CARGO_TARGET_DIR/wasm-dev/examples/render_at_location" \
  --pack "$CRYSTAL_PACK" --map IlexForest --x 15 --y 27 --view 2.5d --hour 11 \
  --screenshot "$PWD/target/map-interest-resume/forest-rgba.png" \
  > target/map-interest-resume/forest-rgba.log 2>&1
```

Repeat that command with `--map LakeOfRage --x 20 --y 18` and a distinct Lake
output filename to check the shoreline. The explicit image target is a
diagnostic mode. Separately test the normal window target and the browser;
preserve failed captures rather than reporting another target as a retry pass.

Next steps after transfer: verify the shared module hooks and crash guard are
actually committed; reproduce and repair forest gaps and window capture with
the appropriate owners; visually review all 35 map families and then remaining
maps without changing navigation/events; resolve the Goldenrod source defect.
When the map priority is complete or the user reprioritizes, resume Pokemon
work: inspect every canonical body at multiple angles in normal and shiny
native/browser battle views, finish Omanyte review, inventory missing dedicated
bodies, and evaluate openly licensed rigged assets with provenance. Preserve
original 2D for missing coverage. Tests, catalogs, and source-icon routing
alone cannot establish that every Pokemon model looks finished.
