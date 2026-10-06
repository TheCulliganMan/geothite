# Camera-aware wall and roof translucency

Blocking authored walls and roofs remain visible at 20% opacity instead of
losing a capsule-shaped region around the player. The complete source-owned
object or section fades, so other actors and props behind that section remain
readable too. Floor backing, furniture, unmarked scenery, collision, scripts,
warps and actor footing are unchanged.

The renderer separates existing authored vertex ranges before ordinary static
terrain batching. A cached triangle BVH tests nine camera-to-player body/head
rays; bounding boxes are only a broad phase. This avoids treating the empty
interior of a concave U-wall as a solid blocker. Gym maze cells keep separate
source ownership so a single obstructing section cannot fade the whole maze.
Department-store housings and their textured plaques explicitly share a state.
The association comes from the successful source append, never proximity.

Fade-in and restoration use exponential time-based transitions with a 120 ms
exit hold. Returning to the same obstruction retargets the existing opacity.
Fully restored groups use the ordinary opaque material and write depth. Fading
groups use cached blended materials, normal depth testing and no depth writes.
Up to 16 centered render leaves per material domain improve transparent sorting;
all leaves of a source group share one fade state. Geometry is uploaded once
per terrain revision, and map replacement drops its meshes, materials and state.

The forward-only fade leaves StandardMaterial alpha unchanged. Bevy's existing
shadow pass therefore keeps each physical wall's complete shadow through the
transition. Transparency uses ordinary sorted blending, so oblique and
intersecting-wall views remain part of native visual review.

Regression coverage includes the empty U-room false positive, orbit/zoom/rise
comparisons against actual triangles, foreground depth order, continuous and
repeated fades, equivalent time steps across the exit hold, F3 and root movement,
linked plaques, opaque neighbors, indexed geometry reconstruction, fixed asset
counts during movement, and repeated map replacement with real asset cleanup.
Run `cargo test -p crystal-voxel-view occluder_fade` for the focused suite and the
full voxel-view suite for source ownership and footing regressions.

The initial same-camera FastShipB1F review in
`output/world-3d-expanded/wall-translucency-v1/` includes mess-back, mess-side,
mess-front and entrance-back. The back and side views retain faint wall shapes
with multiple actors, stools and tables visible behind them. The front and
unblocked entrance keep solid walls. These still images do not by themselves
establish transition timing or frame performance; use a repeated walk/orbit clip
and paired native frame measurements for those checks.

## Recovery validation

The original implementation passed 818 voxel tests and native/WebAssembly builds
before the execution workspace was replaced. The source was recovered from its
authoring steps, without intentional behavior changes. The original local capture
directory did not survive; the still-image observations above describe that prior
review. The recovered checkout again passes all 818 voxel tests, with two existing
benchmarks ignored. Fresh native/WebAssembly builds, camera/movement/F3 checks
and captures remain required before final signoff.
