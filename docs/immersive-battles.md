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

The arena renders into an offscreen target. One camera-facing similarity maps
the complete source effect canvas between the projected battler hit anchors.
It is fitted in physical pixels, then normalized to viewport coordinates, so
circles stay round at every aspect ratio and separately allocated OAM tiles keep
their shared edges. Source objects remain above background scroll sampling.
This preserves their source sequence, shape and palette along the battler axis. F3 changes presentation without another interpreter, command
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
quad samples the arena target in the same native HUD pass. The same global
camera-facing similarity maps source OAM and background register sampling.
Output pixels are inverse-mapped to source coordinates, then `(x, y)` samples
`(x + SCX[y], y + SCY[y])` before reprojection. Rows outside the source effect
buffer stay unwarped. Source tilemap wrapping applies inside the projected LCD
footprint; the artistic arena continues beyond it. Exposed samples use the
palette-adjusted arena sky. Both original anchors, `(40,72)` and `(124,32)`,
land exactly on the projected model hit points. Off-anchor paths intentionally
follow the original source geometry instead of the old sheared world-plane map.
The earlier Surf capture predates this shape correction; Surf and other row
mappings need renewed native review. Model root/contact-shadow displacement is
still a separate world-space path and is not certified by the OAM correction.
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
The matched Water Gun pair also retains identical actor/field pixels on eight
shared ticks, including its ripple; its overall cached timing is worse. Alpha-edge,
resized attack, interruption and broader source-OAM review remain open. Neither
pilot closes the general extracted-row/reveal gap.

## Experimental native capture images

`CRYSTAL_BATTLE_CAPTURE_PROTOTYPE=1` enables a native-only, first-command capture
pilot. It independently prewarms a static enemy image and admits it only after
the corresponding draw, output blit and submission are acknowledged. A pending
or incompatible image uses the classic renderer for the complete capture.
The default remains classic capture; WebAssembly and unsupported appearances
retain that path. Normal, nonshiny, untransformed static enemies are eligible;
articulated targets, Substitute, later-turn appearance state and tutorial captures
are outside this first slice. Future queued enemy responses do not count as an
active move and must not prevent an otherwise eligible failed throw.

The image lease uses the source ReturnMon/EnterMon whole-tile selections, not a
center crop or a uniform shrinking model. Enemy pictures select 7/5/3 tile rows
and columns with the original four-tick phases, then hide or restore. Original
ball OAM, current-item palette, object clock, shakes and outcome remain controller
owned. The caught ball stays through the Gotcha page; failed catches restore the
model without leaking future response HP. F3 or an incompatible view retires the
lease and latches classic presentation until that capture ends.

At the current checkpoint, real Master Ball success and ordinary failure remain
in the modeled arena; the failed attempt retains its queued Mimic response.
An actual F3 interruption safely falls back. Full captures were checked at
800x600; both Full and Reduced captures completed at1180x812. Resize invalidation passes renderer tests, but a new
actual native resize during capture has not been verified. The viewed fixture
places the final ball inside the enemy foot ring; that is not a general
pixel-perfect registration proof between model compaction and source OAM.

Gust's shared wait/return clock, original OAM assembly and foreground priority
were checked against the pinned original source in both directions. After the
shape correction, native wind and both hit phases were visually compared with
that independent assembly. Those newer captures did not sample terminal tick90;
earlier classic captures and source-clock tests cover it. Do not turn a selected
visual comparison into an all-frame or all-move fidelity claim.

## Specific source sequences

- Shadow Ball uses BGP `$1b`, a blue Barrage Ball with `WAVE_TO_TARGET`, and a
  later poof. Its source does not request a repeated flashing background effect.
- Psychic uses `PSYCHIC_M`: eight WAVE objects, source palette changes and
  horizontal row deformation. The added target aura is removed.
- Hyper Beam uses the original beam segments, tip, flashes and source screen
  displacement. Recharge is a production battle rule, not a renderer delay.
- Surf uses move57 and `BattleAnim_Surf`: its blue 22-piece OAM crest follows
  the source object VM with vertical background sampling. No extra rolling water
  wall, foam, spray or impact object is added.

The older per-move frame-number examples predated the shared N+1 wait/return
correction. Exact current timing comes from the source program and controller
trace; those historical numbers are not a current validation result.

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
