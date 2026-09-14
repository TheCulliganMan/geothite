# Crystal voxel rendering methods

Reference: [DramaticShapeVoxelMod-latest](https://github.com/scottcandy34/DramaticShapeVoxelMod-latest), revision `cd10ac3158db9a53e2e33efa3651935723715c9b`.

The reference was downloaded and studied locally. Crystal's native 2.5D implementation is Rust/Bevy code using Crystal's own exported artwork and profiles; it does not require the Lua mod or a Gen 1 runtime.

## Rendering correspondence

| Reference module | Crystal implementation |
| --- | --- |
| `Structures.lua` round-object construction | `mesh/hull.rs`: silhouette rows become integer circular depth chords; emit only exposed faces, including steps between different neighboring chords; merge depth runs sharing one source texel |
| `Structures.lua` object segmentation | `mesh.rs`: flood the complete drawing outside its dark outline, preserving enclosed colors; use the darker two palette shades for dithered silhouettes |
| `Buildings.lua` | `mesh/house_shell.rs` constructs Johto houses as occupied source-pixel volumes, with measured descending roof ends, recessed facade pixels and culled shell faces. `mesh.rs` handles the other complete building drawings and their native bands. |
| `ChunkMesher.lua` | `mesh.rs`: combined terrain meshes, source texture coordinates, exposed terrain sides and per-face shade; asynchronous scene builds in `lib.rs` |
| `SpriteBillboards.lua` | `camera.rs` and `lib.rs`: current actor frames on alpha-tested cards, facing the camera around the feet; scenery remains fixed in world space |
| `Voxel3D.lua` and `ShadowMap.lua` | `voxel_surface.wgsl`: palette color multiplied by authored face shade and sun visibility; Bevy supplies the depth shadow pass, shared by terrain and actors |

World coordinates are east (+X), height (+Y), and south (+Z). Geometry stays in source-pixel proportions. Presentation does not change collision, warps, NPC scripts, or simulation.

Complete recognized objects must not silently revert to flat tiles when source data is absent. Such errors are returned to the renderer for diagnosis.

## Verification

Run `cargo test -p crystal-voxel-view --lib` for geometry and renderer regressions. Use the location tester described in `RENDER_AT_LOCATION.md` to inspect actual GPU output at the same map coordinates in both modes.

The `voxel-view` feature supports both native Bevy and the WebGL browser build; players select it through the view toggle.

Original geometry validation: 428 voxel-view tests passed. The tree-volume regression was observed failing before the change; the modern-building regression was also run against the original building function and failed on its plain side walls. Native GPU captures cover New Bark Town, Goldenrod City, National Park, Players House 1F, Union Cave 1F, and Ice Path 1F.

## Terrain and player-home correction

The earlier regional trapezoid override was not the reference terrain method. Rock runs now fold their native eight-pixel face courses vertically without expanding into neighboring paths. Complete rock formations fold their outer sixteen-pixel side strips away from the cap and use the two drawn front courses (sixteen pixels) as height. Ice Path small boulders use the same silhouette/chord hull as other round rocks, removing the separate wedge renderer.

Johto houses now occupy the matched drawing footprint, anchored at the source doorway. The roof descends according to the measured column silhouette and overrides the wall volume where they intersect; it is not forced into a generic center ridge. Only exposed voxel faces are emitted, including the one-pixel window recesses. Player-home cabinets, TV and bedroom fixtures have eight-pixel depth; their generated sides sample inward from the outline to avoid solid black flanks.

The player-home wall pass leaves fixture cells to their volume renderers and supplies architectural backing behind them. A regression demonstrates that wall ownership previously suppressed the TV and cabinet meshes entirely. Ice rocks close interior silhouette spans so bright surface cracks do not become through-holes; loose boulders use the drawn eight-pixel front-course height, while perimeter rocks have solid depth.

Kitchen drawings are folded at the native eight-pixel band boundary: burners and basin face upward, cabinet doors face south, and the shell has closed sides and bottom. The room partition folds its six plan-view rows into a horizontal wall top and its two southern courses into the front face. Block `$0f` is matched as the full three-band refrigerator; its upper two bands were previously misclassified as a staircase. Stair openings remain clear of the architectural wall on both floors. The bedroom matcher includes the undecorated stair block and all four decorated variants, so hanging a poster cannot turn the stairwell back into wall artwork. Stair flights have closed painted side courses and risers, with tread color sampled from the source band at each step height. Perimeter ice rocks use the circular-chord hull as well as the loose boulders.

## Movement and room-transition correction

The host owns a mutable viewport atlas, which is overwritten when scrolling or changing maps. Each terrain build now copies that image and keeps its own atlas for the lifetime of its mesh. A frame publishes its map-grid origin separately from camera interpolation. Retained geometry translates by the difference between its built origin and the live origin, so UV slots remain attached to world objects.

Overlapping unchanged terrain is reused within half the available halo. Animated ground pixels update the original built atlas slots. Actor footing heights are remapped by world coordinates. Map identity participates in the cache key, and crossing rooms cancels builds from the previous map and hides its geometry before accepting the new scene.

Current regression validation: 435 voxel-view tests pass, including the reproduced viewport-step offset, mutable-atlas isolation, animated pixel placement, footing remapping, and upstairs/downstairs/outdoor invalidation. Native movement QA (`--walk RDLU`) crossed eight viewport origins in 128 frames with one additional mesh build. Its measured median was 38.55 ms and p95 63.08 ms at 2560×1440 while compiling the browser build; this is not a 60 FPS performance claim. A single browser session walked from upstairs through downstairs dialogue and out into New Bark Town, then reentered the house and walked upstairs again. No fatal runtime error or cross-map texture reuse occurred. See `output/playwright/` for captures and the continuous exit trace.

Animated flowers now have separate textured and solid mesh buffers, sharing the retained atlas and world transform. Their silhouette changes rebuild only these small buffers. Public combined mesh export still includes those buffers. Static geometry and footing heights are covered by an animation-invariance regression.

A native `--walk UD` stair run crossed PlayersHouse2F → PlayersHouse1F in one process, recorded map identity every frame, and passed the screenshot/active-renderer checks. Its 64-frame trace measured 18.26 ms median and 38.06 ms p95; raw data is in `output/movement-qa/house-stairs-verified.movement.csv`.

The browser autosave filename is derived from the composed pack id. Save-slot validation now accepts the `+` separator already used by those ids, while continuing to reject whitespace and path separators. Its regression failed against the previous validator; all 27 save tests pass after the correction.

Final browser movement verification: 284 sampled animation frames during eight directional inputs, median 16.7 ms, p95 16.8 ms, maximum 33.4 ms. Six additional captures taken while ArrowRight was held show unchanged texture attachment through camera motion. No errors or panics were logged in that session; primary and backup autosaves were written successfully. The tested build is installed in `web-dist`, including regenerated compressed JavaScript/Wasm assets. Details and traces: `output/movement-qa/README.md`.
