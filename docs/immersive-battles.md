# Immersive battles with original attack sequences

The game keeps its artistic 3D arena, perspective camera, modeled creatures and
compact HUD. Original Crystal is the reference for attacks: their choreography,
timing, colors, flashes, object motion and sound cues. The production
`VisibleShellController` owns commands, PP, HP, damage, status, switching,
capture, rewards, recharge, dialogue and return to the map. The renderer consumes
the currently presented scene and cannot advance combat or reveal future state.
See [the attack fidelity matrix](original-2d-fidelity.md) for remaining work.

## Immersive presentation

Biome scenery, ground, lighting and contact shadows frame the two combatants.
The existing production HUD glyphs are placed in compact backed panels, while
commands and dialogue dock below the arena. Values and input semantics are not
recreated. Stored source transforms make F3 reversible and keep full-screen
party/item surfaces on their existing presentation path.

All 251 normal species have first-pass geometry; individual sculptures still
need visual review. Unknown species, shiny palettes, Substitute and Minimize
retain their actual source art where a correct modeled appearance is unavailable.
Source clipping, extracted rows and some faint/withdraw/reveal/capture phases
currently retain the original presentation as explicit remaining 3D gaps.

The arena renders into a full-window target. Source attack objects are projected
through its camera into the native overlay, outside background scroll sampling.
This keeps their source sequence and palette while placing effects along the
3D battler axis. F3 changes presentation without another interpreter, command
path or reset of the authoritative frame.

## Physical size and shared framing

Modeled battlers take their physical dimension from the loaded pack's Pokédex
feet/inches field, converted to meters at the presentation boundary. One common
world-unit conversion and uniform per-model scaling preserve relative body sizes
and proportions. Authored body references exclude Cyndaquil's flames and Diglett's
soil, follow the current serpent centerlines, and use body length for the explicit
fish/insect cases. These references describe original geometry; no duplicate
species-size catalog is stored. Missing or invalid dimensions retain source art.

One layout fits both complete neutral model bounds, adjusts separation for large
bodies, and projects actors, source OAM, row effects and hit anchors through the
same camera. Visibility changes or source offsets do not resize individual
participants or reframe a held attack. Native Cyndaquil/Sudowoodo and Onix/Diglett
encounters have been reviewed; every species pairing and move still needs its
appropriate visual check.

## Original effects and row sampling

`VisualBattleSourceFrame` retains current source frame, BGP/OBP-derived artwork,
battler displacement, screen displacement, live object slots and SCX/SCY buffers.
No move-name text parsing, guessed hit or renderer-owned duration is used.

Surf and Psychic keep modeled battlers during source scanline effects. A Material2d
quad samples the arena target in the same native HUD pass. A shared homography
maps the source attack plane into the perspective view. Output pixels are
inverse-mapped to source coordinates, then `(x, y)` samples
`(x + SCX[y], y + SCY[y])` before reprojection. Rows outside the source effect
buffer stay unwarped. Source tilemap wrapping applies inside the projected LCD
footprint; the artistic arena continues beyond it. Exposed samples use the
palette-adjusted arena sky. OAM anchors share this mapping while their original
silhouettes remain camera-facing rectangles. The restored native Surf capture verifies the crest, displaced background and
neutral return. Other attack mappings still need their own native review.
Projected source OAM remains separate above this sampled background. The renderer
must not distort that OAM a second time or bend geometry with the opposite sign.

The classic adapter was corrected at the same boundary: Surf supplies vertical
SCY sampling, not horizontal displacement; horizontal row positions subtract raw
SCX. Existing original-ROM object oracles establish Surf's LCD-register selection
and object boundary state. They do not contain a final background framebuffer,
so source/hardware sampling regressions and native comparisons are distinct evidence.

Full source flashes remain the default. F4 or `--reduced-flashes` reduces visual
contrast while keeping source ticks, positions, object lifetimes, sounds and battle
rules. The presentation resources are never read by the controller. Current native
comparison and regression results must be checked before claiming every effect
or interruption is faithful.

## Experimental extracted battler rows

The default renderer still falls back to source presentation for extracted-row
phases. A native-only pilot is available with
`CRYSTAL_BATTLE_ROW_PROTOTYPE=1` for the exact `TACKLE`/`BattleAnim_Tackle` and
`WATER_GUN`/`BattleAnim_WaterGun` roots. It requires normal modeled battlers,
recognized source event structure and a complete two-row OAM rectangle. Partial
or multiple strips, unsupported sequences and special appearances retain the
source renderer. Live source OAM, including its final deinitialization frame,
remains the lifetime authority.

`CRYSTAL_BATTLE_ROW_CAPTURE_CACHE=1` is a separate opt-in that also requires the
row prototype. It reuses eligible static actor captures only after the matching
render draw, output blit and submission are acknowledged; changed appearance,
scene or target state invalidates reuse. Both options are disabled by default
and unavailable in WebAssembly. A matched native Tackle pair verifies identical
actor/field pixels on shared source frames and removes the measured first-row
hitch; it does not improve the median capture interval. See the
[bounded timing result](battle-3d-performance.md#experimental-actor-capture-reuse).
Water Gun, alpha-edge, resize, interruption and broader source-OAM review remain
open. Neither pilot closes the general extracted-row/reveal gap.

## Specific source sequences

- Shadow Ball changes BGP to `$1b` on frame 1 and spawns its blue Barrage Ball
  with `WAVE_TO_TARGET`; the poof starts at frame 33 and the script returns at 57.
  Its source does not request a repeated flashing background effect.
- Psychic uses `PSYCHIC_M`: eight WAVE objects start at frames 1, 9, 17, 25, 33,
  41, 49 and 57. Source palette changes and horizontal row deformation continue
  through frame 160; the script completes at 165. The added target aura is removed.
- Hyper Beam's segments start at frames 1, 5, 9 and 13, with its tip at 13.
  Original flashes and source screen displacement are retained; the script
  completes at 61. Recharge is a production battle rule, not a renderer delay.
- Surf uses move 57 and `BattleAnim_Surf`. Its blue 22-piece OAM crest follows
  the source object VM with vertical background sampling. Events occur at
  presented frames 1, 33, 65 and 97, and the object enters its exit state at 129.
  Total duration is 185 presented frames including return. No extra rolling water
  wall, foam, spray or impact object is added.

The existing Rust sound engine remains authoritative. The source sound-ID/frame
sequence is retained; animation-command stereo arguments are under a separate
review. Silent preview videos do not establish audio fidelity.

## Reproduce the disposable native fixture

```sh
cargo build --locked -p crystal-bevy \
  --features voxel-view,location-tester,fullscreen-scaling \
  --example immersive_battle_3d
target/debug/examples/immersive_battle_3d \
  content-packs/core-modular.browser.crystalpack --surf \
  --record output/surf-3d --record-on-move --seconds 8
```

Select FIGHT and the first move through normal controls. `--surf` uses level-20
Totodile taught the actual nonconsumable HM03; it does not inject a fabricated move,
force damage or alter enemy HP. Other exclusive fixtures are `--shadow-ball`
(level-40 Gengar and its real TM), `--psychic` (level-20 Kadabra) and `--hyper-beam`
(level-20 Raticate). Their existing compatibility, PP and recharge checks remain.
The Hyper Beam developer fixture retains its documented deterministic DIV replay;
Surf uses ordinary accuracy and damage resolution. No fixture loads or saves a
user save file. Add `--classic` for a source-view comparison.

Recordings retain actual sample timestamps, view mode, source frame/OAM/SCX/SCY,
asset counts and renderer dimensions. `CRYSTAL_CAPTURE_FPS=60` is a maximum,
not a 60-fps claim. Encode `frames.ffconcat` with `-fps_mode vfr`; never interpolate
or retime missing samples. Compare original and modeled captures on shared source
frame indices. The earlier performance checkpoint is documented in
[battle performance checks](battle-3d-performance.md); the restored immersive composition
requires its own native measurements.

## Validation

Run the battle-view and render-API tests, source scroll/timeline regressions,
controller fixture tests with an explicit external pack, and native/browser builds.
Native review must include Surf and Psychic row effects, matching original views,
Full/Reduced, F3, resize, menus/cancel, faint/switch/capture and return to the world.
A model registry count or a successful static mesh test is not a fidelity pass.
