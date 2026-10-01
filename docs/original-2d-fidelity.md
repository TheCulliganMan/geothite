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
| Surf | Corrected vertical SCY path, LCD-register lifetime and source-object clipping; source script has 185 presented frames | Immersive projection is captured; straight-root sound-tail wait now tested, wrapped Surf remains open |
| Global BG scroll | Source register sign corrected; OAM remains outside the BG sampling pass | Verify both signs and zero crossings with the immersive camera |
| Sound | Rust source sound engine plus original packed stereo metadata and Surf's channel-busy wait; canonical PCM stays unchanged | Native audible review and general channel replacement/priority/music arbitration remain open |
| Species models | All 251 normal species have first-pass geometry; pack dimensions now drive uniform body scale and shared camera fit, with native Cyndaquil/Sudowoodo and Onix/Diglett checks | Continue individual visual review and sculpt refinement; counts and representative pairings do not establish polish |
| People and world assets | Source-aware complete-object bindings preserve ownership, spacing and footing | Finish residual families and review recognizability, proportions and occlusion without requiring pixel-copy palettes |
| Special visual states | Shiny, Substitute, Minimize and some clipping/reveal phases retain source art | Validate each state before replacing it with a modeled appearance |

Attack review needs a source reference, a documented mapping into the 3D scene,
matching-frame comparison and an explicit list of remaining differences. A
successful mesh test or positive map count alone cannot close that review.
Source artwork, content packs and generated reference captures remain external
or ignored; they are not bundled game content.

The special-room batch corrects three department-store roof binoculars and
Rocket Base instrument equipment. Native captures verify those object identities
while retaining the artistic 3D design.

## Recorded diagnostic reference

The corrected original-LCD Surf comparison retained 336 actual modeled frames
over eight seconds. All 125 sampled script frames 0–184 used modeled battlers
and vertical-only Surf sampling. Across 104 matching source frame indices,
original-view and modeled-view traces agreed on BGP, OAM, SCX/SCY and cues.
Median update was 23.65 ms, p95 29.80 ms and maximum 40.39 ms; no gap exceeded
100 ms. These measurements describe that diagnostic composition on the cloud
software renderer, not the restored immersive arena or a universal frame rate.

That recording predates the sound-busy wait correction below. Matching the
185-frame animation script alone does not establish complete sound-to-HP timing.
Silent preview videos do not verify audible fidelity.

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

## Source sound and wrapper corrections

Visible animation sound events now retain the original packed argument byte and
sound identity through both native and browser command queues. The shared
post-cache channel mask covers 24 inspected source sound programs. Every mode-2/3
cue across 52 move roots is covered; modes 2/3 select static left/right routing in
Crystal, just like modes 0/1, with enemy-side reversal. Their extra duration fields
and flag are written but never read by the source engine. They are not invented
pan sweeps or cutoffs. Mono remains unchanged. Eligibility is sound-routing
coverage, not complete attack or hardware mixer fidelity.

Tests decode the external pack, compare canonical PCM hashes, frame counts,
format and loop metadata, exercise both actors and Mono, and confirm immutable
cache identity. Whole transient replacement, priority filtering and music
preemption remain different from a channel-accurate hardware mixer.

For straight-root Surf, the sound interpreter computes source channel clocks
without generating PCM or using audio-device wall time. The 185-frame visual
script remains unchanged. Its final channel remains busy until tick257, so the
controller holds72 virtual ticks before releasing damage. The original objects
are cleared and the HUD restored during that hold. Muted and audible execution
share the same timing. Wrapped Surf and general sound arbitration still need
review. New silent videos cannot establish audible synchronization.

Kinesis, Softboiled and Milk Drink now initialize their object buffers using the
canonical move identity even when called through wrapper labels. Pack-backed
VM/OAM and source-art rendering tests verify the original enemy-side eight-pixel
correction and prove that presentation text cannot alter attack identity.

Extracted battler rows remain a separate modeled-rendering gap: 70 move roots
can enter a whole-scene source-art fallback. The integrated native-only
Tackle/Water Gun prototype and its separately gated actor-capture cache remain
disabled by default. Exact source structure and full two-row OAM ownership are
required; unsupported roots and partial strips retain source rendering. Source
footprint, object priority, alpha edges and native composition still need visual
review, and cache A/B measurements are pending. See [the opt-in boundaries](immersive-battles.md#experimental-extracted-battler-rows).
