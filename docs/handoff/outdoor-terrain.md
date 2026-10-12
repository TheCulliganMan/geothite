# Outdoor terrain transfer checkpoint — 2026-10-12

## Goal and stopping point

Original goal: **“Make outside rendering perfect up to the standards I set.”** This goal is incomplete. The human requested all work committed for transfer and explicitly instructed this thread to listen to the coordinating agent. Work stops after this checkpoint; resume improvements on the new host after shared integration is committed.

Source checkpoint: `4d8d5824bd36109655493f28506c9623687783ab`. The commit adding this note can be found with `git log -1 -- docs/handoff/outdoor-terrain.md`. Neither this scope commit alone nor an old screenshot establishes a complete transferable build: the coordinator must capture the shared integration paths below.

The target is the supplied warm, nuanced forest remake: natural green variation, dappled light, alive grass, convincing trunks and mineral forms, and blended ground contacts. The supplied forest attachment is external to Git and is not available locally for this handoff. Saved National Park art is tracked at `docs/art/map-concepts/national-park-remake-v1.png` with its companion Markdown. Preserve its real image bytes through Git LFS transfer where applicable. The full review history and area requirements are in `docs/art/outdoor-terrain-review.md`.

## Current implementation

The latest change composes mineral-rim sculpting and rock-base contact rounding before one fracture-aware normal calculation. Previously the base pass erased cap fracture shading. Outdoor caps reuse the existing bedrock texture at the cave top-projection scale; no duplicate rock asset was created. Larger source-coordinate mineral beds replace short-period height ripples. Source floor support, closed corners and tree seats remain requirements.

Lake cabin roofs use their own source colors and the existing closed curved roof courses. Corner trees use exact source-phase bindings. Ordinary lawn/water contacts reuse shared soil relief and a narrow map-anchored damp color band without moving native floor corners. Buoy courses follow exact source ownership. Required tree loading has no alternate 3D fallback; original 2D is permitted to reveal missing coverage. Ordinary grove, Park and intended forest families must stay distinct. Shared route signs reuse one resized authored model.

Latest action: inspected muted Route26 45-degree and Lake clearing 135-degree captures, then completed the extended production winding diagnostic. **The diagnostic failed.** No geometry fix was attempted after the transfer instruction. Its failing regression remains in the checkpoint so the defect is reproducible.

## Checks and unresolved defects

- Current combined native build passed (`target/outdoor-final-rock-pipeline-native-build.log`, 12m16s). This was the full dirty worktree, including shared integrations, not the scope commit in isolation.
- Before the winding extension: 19 stone tests and six base-contact tests passed. Seven ledge-approach tests passed. Five Python asset-storage tests passed at transfer.
- Earlier scoped checks passed: nine cabin, three roof-course, three shore-blend, five ground-blend, two soil-contact, four required-loader and one missing-forest tests. These were not all rerun after the final diagnostic extension.
- **FAIL:** `production_rock_finish_keeps_fractures_after_rounding_the_contact_base`, including corrected native per-tile tessellation. `target/outdoor-final-rock-pipeline-native-topology-test.log` reports an inverted cap triangle at (9.220304,4,14.89865), (8.822326,5.3333335,13.378186), (8.905096,4,13.626882). Do not claim earlier independent winding checks cover the combined sculpt/base pass.
- Route26 (10,28), orbit2/45 degrees: latest inspected image completed in 202.16s. Fracture shading survives, but cliffs are still swollen/lumpy. Side-view bank caps rise; requested saw-blade presentation and every floor-Y seam remain unaccepted.
- LakeOfRage (23,30), orbit6/135 degrees: latest inspected image completed in 228.83s. Quiet cyan curved roof and damp soil contact are visible. Shore geometry stays straight, water retains rectangular glints, trees repeat. This angle leaves the clearing visible but does not resolve canopy overlap in earlier 45-degree views. Earlier Lake view exposed a permitted 2D creature marker; missing Gyarados coverage remains to verify.
- Whole-area review remains: New Bark through Cherrygrove and other routes/cities, Lake/Route43, stone/canyon surfaces, all source floor contacts. National Park still needs exact outer tree families, benches, two grass types and centerpiece shape. Ilex appearance is unaccepted; do not repair it by changing every tree globally.
- Cherrygrove's unwanted lake was overridden in ignored local content: Cherrygrove blocks 17/18 row0 and Route30 x9 rows25/26 water changed to trees. Transfer the actual modified pack and recheck source/native appearance; do not commit packs or restore them from Git history.

Investigate cap deformation/winding first on the new host. Per-vertex height-dependent bevel noise may need an edge-anchored radius; that is an unimplemented investigation, not an accepted repair. Then compare fresh 2D/native pairs at 45 and 90 degrees, correct shape and floor seams, and return to Lake cabins/shore/water. Keep source collision, events, occupancy, claims and floor datums authoritative.

## Ownership and shared integration

The source checkpoint owns `mesh/outdoor_stone.rs`, `mesh/rock_bases.rs`, `mesh/shore_blend.rs`, `mesh/buoy_course.rs`, `mesh/ledge_finish.rs`, `new_bark_models.rs`, `roof_tiles.rs`, `open_models.rs`, `exterior_models.rs`, `foliage.rs`, `foliage/park_tree.rs`, route-sign JSON, three obsolete tree JSON removals, the foliage/world authoring tools, storage test and outdoor review document. Rust paths are under `crates/crystal-voxel-view/src/`; model paths under its `models/`.

The coordinating agent must preserve these mixed-owner paths in its shared integration commit:

- `crates/crystal-voxel-view/src/mesh.rs`: shore/buoy module wiring.
- `crates/crystal-voxel-view/src/mesh/build_scene.rs`: production `finish_rock_geometry`, source binding/roof integration.
- `crates/crystal-voxel-view/src/mesh/new_bark.rs`: source lawn palette, shoreline finish/tessellation, corner crowns, tree foundations, source floor contacts and common grove integration.
- `crates/crystal-voxel-view/src/mesh/modeled_exteriors.rs`: cabin roof palette extraction, canonical forest loading and exact source-2D missing coverage; preserve other owner's underplanting.
- `crates/crystal-voxel-view/src/mesh/ground_blend.rs`: shared source path/lawn palette integration.
- `crates/crystal-voxel-view/src/voxel_surface.wesl`: outdoor negative-tag rock top projection at 144 units; preserve lighting/cave owners' changes.
- `tools/build-new-bark-models.py`: obsolete tree prototype removal mixed with another owner's NPC work.
- Required-loader adapters, including `interior_models.rs`, `johto_actor_props.rs` and their callers, plus remaining obsolete actor/interior fallback deletions, belong in the coordinator's shared checkpoint. They were deliberately not swept into this scope commit.

No whole-worktree staging was used. No pack, native executable, scratch render, ROM, exported source catalog or generated PCM is a deliverable here.

## Restart on the new host

Use the coordinator's `docs/cold-start-3d.md` and `scripts/cold-start-3d.sh`; do not create a competing startup path. Transfer Git history, separate real Git LFS objects/assets and ignored `content-packs/` contents. Required pack: `content-packs/realtime-clock.browser.crystalpack`. Required canonical model root: `$PWD/assets/open-models`. Git LFS pointer text is not an asset. Follow pinned Rust toolchain and platform/GPU dependencies in the cold-start documentation. Captures need a working native GPU backend even when using image readback.

Run inside the transferred checkout after shared integrations:

```sh
./scripts/cold-start-3d.sh check
python3 tools/test_model_asset_storage.py
cargo test --locked -p crystal-voxel-view --lib \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  production_rock_finish_keeps_fractures
```

The last command currently fails and is the first defect to fix. Relevant focused filters afterward are `mesh::outdoor_stone::`, `mesh::rock_bases::`, `mesh::ledge_approach::`, `mesh::shore_blend::`, `cabin_` and `roof_tiles`.

For a dedicated native build:

```sh
CRYSTAL_PREVIEW_MUTE=1 CARGO_TARGET_DIR="$PWD/target-outdoor-verify" \
CARGO_INCREMENTAL=0 CARGO_PROFILE_WASM_DEV_DEBUG=0 \
cargo build --locked --profile wasm-dev \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 --jobs 2 \
  -p geothite -p crystal-bevy -p crystal-voxel-view -p crystal-render-api \
  --all-features --example new_bark_3d
```

Capture fresh source/native comparison without loud audio:

```sh
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_OPEN_MODEL_ROOT="$PWD/assets/open-models" \
CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=300 \
./target-outdoor-verify/wasm-dev/examples/new_bark_3d \
  content-packs/realtime-clock.browser.crystalpack \
  --map Route26 --x 10 --y 28 --orbit 2 --zoom 1 --size 640x480 \
  --screenshot target/outdoor-route26-new-host-45.png \
  --second target/outdoor-route26-new-host-2d.png
```

Repeat with orbit4 for 90 degrees. For Lake use `--map LakeOfRage --x 23 --y 30 --orbit 6`; also review 45 degrees to expose canopy issues. Keep every preview muted with `CRYSTAL_PREVIEW_MUTE=1`, never by changing system volume. Ignored old-host screenshots/logs are diagnostic history and must be regenerated for current acceptance. A pack dump or render does not prove playable controller/MCP input, and this scope makes no audio or gameplay validation claim.
