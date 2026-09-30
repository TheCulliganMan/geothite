# Original attack fidelity in the immersive 3D game

The intended presentation is an immersive 3D battlefield and artistic modeled
world. Original Crystal is authoritative for attack choreography, timing,
colors, flashes, object motion and sound cues. It does not require replacing the
3D arena, camera, models, lighting or compact HUD with a copy of the 2D screen.
Map spacing, collision, warps, actor identity and gameplay remain authoritative.

A temporary original-LCD composition was used to compare source sequences. It
is a diagnostic reference, not the requested art direction. The following matrix
separates verified source behavior from remaining work.

| Area | Evidence | Remaining work |
| --- | --- | --- |
| Commands, PP, HP, damage and recharge | Production controller and battle engine; fixture regressions and actual damage captures | Keep rendering outside these rules |
| Attack clock and live objects | Existing animation interpreter supplies source frames, OAM, BGP/OBP, displacement and SCX/SCY | Compare projected 3D effects on matching authoritative frames |
| Arena, camera and HUD | Artistic environments, perspective camera, modeled actors and compact backed panels | Retain this immersive presentation while correcting attacks |
| Psychic and Hyper Beam | Original source objects, palette changes, row deformation and screen displacement retained | Verify projected positions, Full/Reduced, interruption and native capture; unverified companion effects are suppressed |
| Surf | Corrected vertical SCY path, LCD-register lifetime and source-object clipping; source script has 185 presented frames | Recheck immersive projection and source sound-tail wait |
| Global BG scroll | Source register sign corrected; OAM remains outside the BG sampling pass | Verify both signs and zero crossings with the immersive camera |
| Sound | Existing Rust source sound engine; animation stereo metadata and Surf's final channel-busy wait are staged separately | Integrate and validate audio without changing canonical PCM or game state |
| Species models | All 251 normal species have first-pass geometry | Continue individual visual review and sculpt refinement; counts do not establish polish |
| People and world assets | Source-aware complete-object bindings preserve ownership, spacing and footing | Finish residual families and review recognizability, proportions and occlusion without requiring pixel-copy palettes |
| Special visual states | Shiny, Substitute, Minimize and some clipping/reveal phases retain source art | Validate each state before replacing it with a modeled appearance |

Attack review needs a source reference, a documented mapping into the 3D scene,
matching-frame comparison and an explicit list of remaining differences. A
successful mesh test or positive map count alone cannot close that review.
Source artwork, content packs and generated reference captures remain external
or ignored; they are not bundled game content.

The unpublished special-room batch also has confirmed object-identity errors:
three department-store roof binoculars were modeled as plants, and Rocket Base
instrument equipment as a bookcase. Correcting those identities is compatible
with artistic 3D design; it does not require copying every original pixel.

## Recorded diagnostic reference

The corrected original-LCD Surf comparison retained 336 actual modeled frames
over eight seconds. All 125 sampled script frames 0–184 used modeled battlers
and vertical-only Surf sampling. Across 104 matching source frame indices,
original-view and modeled-view traces agreed on BGP, OAM, SCX/SCY and cues.
Median update was 23.65 ms, p95 29.80 ms and maximum 40.39 ms; no gap exceeded
100 ms. These measurements describe that diagnostic composition on the cloud
software renderer, not the restored immersive arena or a universal frame rate.

The original wrapper's final sound-busy wait remains a separate pending
correction. Matching the 185-frame animation script does not establish complete
sound-to-HP timing. Silent preview videos do not verify audio fidelity.

Before the immersive restoration, the integrated working tree passed 612 voxel
tests (two existing benchmarks ignored), 21 render-API tests and focused source
scroll, clipping, fixture and recorder checks. The final OAM ownership change
passed 31 battle-view tests, 26 immersive-battle tests and a WebAssembly check.
These historical checks are not validation of subsequent camera/layout changes;
each published checkpoint requires its own relevant tests and exact-tree CI.

The restored immersive composition passes 35 battle-renderer tests and 27
bridge/controller/recorder tests, including projected OAM, resize and repeated
F3 restoration. Native rendering, frame pacing and exact-tree CI are recorded
separately for its publication checkpoint.

The restored immersive Surf capture was inspected at entry, crest rise, return
and HP damage. It retains 249 actual frames over eight seconds (about 31 fps),
with median 31.56 ms, p95 39.32 ms and maximum 62.62 ms between samples. No gap
exceeds 100 ms. All 91 sampled source frames remain modeled; 71 shared source
frame indices match the original view's BGP, OAM, SCX/SCY and cues exactly.
Mesh/material counts stay fixed at 9/33. The video preserves measured timestamps
within 0.51 ms and adds no interpolated frames. Native and WebAssembly builds
pass. Sound and other unreviewed attack families retain the gaps above.

Native QA also caught a stale render-target texture binding that the scene-only
tests could not see. Resizing now invalidates the composite material's GPU bind
group, with image preparation ordered first. A neutral-frame resize regression
checks invalidation and stable subsequent frames; an actual native capture
verifies that the restored arena is visible. Optional
`CRYSTAL_BATTLE_VISIBILITY_TRACE=1` records one camera/mesh visibility snapshot
for diagnosing future GPU differences without changing gameplay.
