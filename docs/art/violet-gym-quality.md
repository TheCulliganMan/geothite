# Violet Gym quality checkpoint

This local quality checkpoint was validated on 2026-10-04. It is an incremental
scene improvement, not a claim that the game, all species, or every attack has
finished conversion. The current shared-camera experiment has only received
native effect review with Ember and must not be described as an all-move release.
Ordinary play therefore retains the established camera. The experimental lower
camera is available only through the explicit native startup flag
`CRYSTAL_BATTLE_CAMERA_TRIAL=1`; browser builds keep it disabled.

## Implemented scope

- Violet Gym's two source-defined recesses have vertical masonry faces and a
  recessed floor. The matcher owns 176 blocked source cells and leaves the
  original walkways, collision, warps, and actor support heights intact.
- Cyndaquil has a rebuilt folded body, fitted closed eyes, planted limbs, and
  thinner flame panels. Spearow has a distinct face, crest, feathered wings,
  beak, and feet, with ten-joint idle, attack, and hit clips.
- Both species retain canonical GLB assets with named anatomy, materials,
  skinning, and clips. Their generators reproduce the tracked GLB bytes exactly.
  There is no parallel Spearow JSON mesh, STL preview, or encoded asset wrapper.
- The encounter grades only authenticated Violet floor rectangles in its
  private atlas copy. The original world atlas, props, pit art, actor colors, and
  attack images are unchanged. This work happens once when the encounter is
  frozen, with no additional image allocation or per-frame geometry rebuild.
- Actor-only baked fill reveals shaded paper faces on the software renderer.
  The existing arena/ball lighting and hardware PBR path are unchanged. Black
  details, directional shading, and the original full-flash colors are retained.
- Audited Ember objects use body-relative registration and a coherent travel
  transform. Their original object layout, tile adjacency, source frame timing,
  palettes, and impact sequence remain authoritative. Unsupported effects retain
  the existing projection; this registration is not yet verified for all moves.

## Validation

At code commit `e3c0ec11d1146f23d059c73f9cdc1f4c7077bae0`:

- 949 renderer tests passed, with 3 existing ignored tests
- The 7 focused source-placement/overlay tests passed on the unchanged effect
  implementation, and all 19 decoded Cyndaquil/Spearow model tests passed
- Native build and WebAssembly check passed with
  `fullscreen-scaling,voxel-view,location-tester`
- Canonical model layout and both model catalogs passed their source checks
- Fresh generator output matched Cyndaquil's 220,804-byte GLB and Spearow's
  148,832-byte GLB byte for byte
- Native walking and production-controller Abe encounters were captured at
  800×600. The comparisons used the same camera, model scales, positions,
  shadow setting, and software-renderer profile. The rendered battle target
  was 600×450 with vertex lighting and balanced quality
- Matched floor frames used original Ember tick 37; matched lighting frames
  used tick 32. Source object positions and palettes matched in each pair

The final eight-second capture recorded 189 frames over 8.09 seconds, with
38.95 ms median and 62.64 ms p95 update intervals on llvmpipe software OpenGL.
The video retains actual variable timestamps, with no interpolation or retiming.
This is one capture observation, not a hardware performance benchmark or a
smoothness guarantee. The recording is silent.

## Remaining visual limits

Independent review retained the deeper pits, quieter floor, and softer actor
fill as useful improvements. The continuous battle composition is still weak:
the fixed separation leaves Spearow small, and the floor/HUD dominate the frame.
The tested physical scales and source support positions have not been altered
to conceal this limitation. Longer-lens calculations did not establish a useful
solution; no additional lens change was applied.

The two sculpts and five-species articulated runtime do not imply that all 251
species are polished or animated. Broader attack registration, presentation
quality, and hardware/browser visual review remain separate work.
