# Immersive battles with original attack sequences

The game keeps its map-based 3D battle scene, perspective camera, modeled creatures and
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
Source clipping and some faint/withdraw/capture phases retain the original presentation as explicit remaining 3D gaps. Send-out now keeps the original two picture-bank reveal frames and four smoke frames over the 3D map, switching to the model when the full picture appears at tick 6. Shiny send-out reuses the original source animation. Intro trainer portraits still use the original presentation until that source transition finishes.

The arena renders into an offscreen target. One camera-facing similarity maps
the complete source effect canvas between the projected battler hit anchors.
It is fitted in physical pixels, then normalized to viewport coordinates, so
circles stay round at every aspect ratio and separately allocated OAM tiles keep
their shared edges. Source objects remain above background scroll sampling.
This preserves their source sequence, shape and palette along the battler axis. F3 changes presentation without another interpreter, command
path or reset of the authoritative frame.

## Location scenery and battle shots

Checked field encounters reuse the existing rendered map meshes, textures,
footing and props. A city retains its own architecture; a route retains its
actual trees, paths and terrain; caves and gyms retain their modeled interiors.
There is no separate city or route asset catalogue. The encounter holds one
immutable copy of mutable map surfaces while sharing immutable meshes.

The preferred composition is a lower three-quarter view with both Pokémon
facing each other. Visible move, impact, send-out and faint cues direct a smooth
camera sweep into a lower shot and back. Shots use source animation progress,
so pausing a frame or switching to its matched 2D view does not advance the
camera. HP/name panels follow the established screen order, keeping player and
enemy identities fixed when the clearest map view is on the opposite side.
Both bodies, complete animation envelopes, feet, physical sizes and HUD
clearance remain fixed. Source rows retain their registration. Pokémon using
source sprite art also share checked land: an upright billboard follows the
camera inside a conservative ground envelope, without claiming a 3D model.
Each map shot passes the same authentic-ground, omitted-geometry, occlusion
and camera-frustum checks as the establishing view. A failed shot keeps the
proven map view; it cannot extend a map or invent missing ground.

Static terrain batches publish bounded original-triangle coverage before their
CPU buffers retire. Battles share the original GPU meshes; crossing coverage
boxes distinguish the known interior from uncertain exterior pieces without
cutting faces. Every exterior piece still passes the complete frustum and
shadow rejection checks. Camera candidates also reject static walls that hide
a battler's center, upper body or neutral body edges, including wings and tails.
This prevents a shelf from passing a center-ray check while clipping a fighter.
Ledge finishing keeps the source soil face planar. Displaced subdivisions and repeated decorative fringe produced grass-colored spike curtains in the Route 31 battle view; the finish now preserves the original corners without that extra geometry. Source-authored encounter grass remains separate. A native paired 3D/2D capture verified the fix with the battle state preserved. Meadow blades are shorter, with varied bends and staggered shared-edge roots; their verified encounter cells and path boundaries stay fixed. Current native Route 29 wild and Violet Gym trainer pairs preserve the battle state and show readable Pokémon on the actual map without the generic arena.

Tagged native grass remains in the original geometry/shadow coverage. During
modeled battles, blades in front of either battler's depth envelope are invisible
across the full view. This keeps tiny Pokémon clear without silhouette-shaped
holes or clipped grass stumps around their feet. Grass behind the pair, the
original floor, and overworld grass remain unchanged by this battle visibility
rule. Grass encounters prefer staging anchors on the map's actual drawn grass
cells; the same floor, complete-body and scenery proofs still apply.
A native shader-review pair verifies the foreground rule with Cyndaquil and
Sentret fully visible, with the same battle checksum in 3D and 2D. That capture
does not validate the newer grass-height normalization or browser build.

Streamed routes warm the existing grove recipes across the authentic encounter
neighborhood before freezing it, using the normal bounded worker queue. Trees
outside the old overhead camera therefore remain available to lower battle
shots. Deferred sorted tree pieces publish through the existing shared mesh cache before battle freezing, even when the overworld has only needed their opaque combined mesh. They stay hidden in the overworld; the frozen scene retains their exact source granularity. Shot ranking uses visible upper-frame scenery coverage, including partially visible canopies, rather than world dimensions alone. A native Route 31 paired capture verifies retained trees, readable battlers, and preserved battle state. Unloaded source chunks retain conservative omission/shadow bounds; their
absence cannot silently become an empty forest background.

Backdrop ranking samples the bounds of the original sorted tree pieces rather
than the empty space inside a whole-tree bounding box. Camera candidates include
6° and 12° elevations. The native Ilex Forest paired capture preserves the
battle checksum and shows the retained forest canopy with readable fighters;
its visible foreground terrain edge still needs composition work. This is a
native inspection result, not browser or all-location visual acceptance.
The corresponding native Route 29 pair also preserves the checksum and shows
original trees behind readable Cyndaquil and Rattata. Its 185-second diagnostic
capture includes scene setup and camera search; that duration needs profiling
before treating this framing as production-ready performance.
Shot scoring now shares one camera inverse across its scenery pieces and stops
when all 32 backdrop cells are covered. Distant staging positions are skipped
only when the maximum possible score, including their distance penalty, cannot
beat the current shot. Terrain admission remains exhaustive for evaluated shots.
A 1,280-case projection regression matches the previous corner projections;
an isolated 160,000-box CPU control matches checksums. These checks do not
establish a game-loading or frame-rate improvement.
The traced native Route 29 follow-up preserves its paired battle state, but
still takes about 191 seconds overall. Three `battle.establish_camera` spans
take approximately 46, 38 and 38 seconds; a live CPU sample concentrates in
the omitted-terrain visibility/shadow proof inside `constrain_camera`.
The projection optimization has not resolved this setup bottleneck.

The production renderer no longer allocates a generic battle arena. Its 3D camera activates only after the actual encounter map passes admission. While groves, materials, images or bounds publish, the original presentation remains; temporary publication misses retry against the same terrain roots rather than permanently rejecting the encounter. Provenance, support and geometry failures remain strict. A native Route 31 recording verifies this transition and
readable fighters during the subsequent move. Capture logs include
`map_scenery_id` and `map_scenery_fallback_reason` so review can distinguish a
retained location from a pending or rejected map scene.

For native review, the existing `--walk` input sequence also accepts `A` and `B`
for the ordinary confirm/cancel keys, and `S` for the registered field item. This lets the Violet Gym field fixture
walk into Abe's sight and acknowledge the actual dialogue before capturing a
battle. Inputs still pass through the production controller.

Pokémon use a white ambient fill in the battle cameras, with raised shaded-face
fill in the software path. This keeps their actual colors readable in dark
interiors without changing map textures. Native grass has a battle-only body
aperture; the original route blades and floor remain in place outside it.

The source-effect material uses alpha blending and discards transparent OAM
pixels. Native Violet Gym Leer/Peck recordings verify visible Pokémon, a moving
camera, transparent beam edges, and restored HP/menu panels after the move.
The row compositor also blends its transparent pixels over the retained arena.
Native Route 31 and WebGPU Tackle recordings verify that transparent row regions
preserve the Pokémon and scenery rather than covering them in black. The exact
Tackle program now requests modeled rows by default; other row programs retain
their existing admission and fallback rules.
During a presented move, both native 3D HP backings are hidden with their glyphs
so empty cards cannot cover the arena. This is representative visual evidence,
not acceptance of every attack or location.
The native panels use a compact scale ceiling rather than filling the whole
viewport width. A subsequent matched Violet Gym pair verifies legible glyphs,
more room for both bodies, and identical species, levels, HP and command state;
the capture receipt confirms the authoritative state stayed fixed across F3.

Direct source sprites are prepared before Bevy collects visible entities. The
old late preparation could leave freshly rebuilt attack sprites out of the draw
list, producing a blank LCD during animation while idle menus still looked
correct. Native classic Leer and a modeled cave turn's Tackle fallback now show
their sprites and text, followed by the retained cave and updated HP/menu state.
These recordings used different ordinary encounters, not a matched move oracle.

Paired native inspection on macOS uses an explicit image target:

```sh
CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic \
target/debug/examples/immersive_battle_3d \
  content-packs/core-modular.browser.crystalpack --gym-encounter \
  --size 800x600 --walk UAAAAAA \
  --screenshot /tmp/gym-3d.png --second /tmp/gym-2d.png
```

The paired capture freezes the same production state for F3 comparison after
input settles, excluding transient blank source palette flashes. City fishing
review can select an existing map with
`CRYSTAL_RENDER_TEST_FISHING_MAP=OlivineCity` and `--fishing-encounter --walk SAAAA`.
`CRYSTAL_RENDER_TEST_HOLD_BATTLE=1` stops a longer sequence once both Pokémon
are visible in a real, settled field battle; exhausting the inputs without a
battle fails the review instead of accepting an overworld screenshot.
The fixture locates actual unoccupied shore/water cells and uses the ordinary
registered Good Rod input; a bite is not guaranteed. Source cards remain source
cards when no authored model exists. Limited retained acreage, missing ground
or unsupported water stances can still delay or reject 3D admission; no generic arena is substituted.
this is not a claim that every location/species pair has been visually accepted.

Other standing trainers can be inspected with
`CRYSTAL_RENDER_TEST_TRAINER_MAP=GoldenrodGym`, `--gym-encounter` and `--walk`
followed by repeated `A` inputs. Optional `CRYSTAL_RENDER_TEST_TRAINER_OBJECT`
selects an original object identifier. The fresh fixture checks real adjacent
floor outside warps, keeps the trainer's script, party and unbeaten flag, and
waits for ordinary A interaction. Matched native Azalea and Goldenrod captures
verify distinct original gym interiors and unobstructed fighters with preserved
battle state; they do not certify every gym, trainer or subsequent move.

Blackthorn's original animated painted planes retain fixed geometry only when
the complete authenticated texture family is opaque. A native Blackthorn pair
now retains its real stairs, platforms and lava; a changing cutout still fails
correspondence. Native Cianwood and Ecruteak pairs also preserve their original
interiors and battle state. Ecruteak's dark-edge composition remains under review.

Location-tester browser builds support `preview=falkner-battle` and
`preview=route29-battle`, with `multiplayer=off`. These are fresh, unsaved sessions
requiring original button inputs to start a battle. The preview starts in 3D;
activating 3D only after dialogue has begun can lose the preceding field evidence
and require the conservative arena. Browser verification of that transition
and Retina HUD sizing is still in progress.

Route inspection can select a compiled map with
`CRYSTAL_RENDER_TEST_WALKING_MAP=Route30` and `--walking-encounter`.
The disposable fixture checks an unoccupied grass step and its surrounding
walkable corridor, outside trainer sight. Ordinary movement must still produce
the encounter; the fixture does not force a battle or change encounter rolls.
Maps whose verified cave/dungeon metadata permits ordinary floor encounters
also use that original floor. A native review with
`CRYSTAL_RENDER_TEST_WALKING_MAP=IlexForest` reached an ordinary Weedle battle
and retained the forest's existing trees, flowers, sign structure and ground.
Its paired 2D receipt confirms preserved battle state. Forest startup time and
visual ground contact still need improvement; this is not acceptance of every
forest encounter or camera angle.

The establishing shot compares physically checked nearby floor positions and
lower camera angles, rewarding visible original architecture and trees behind
the fighters. Neutral bodies must remain readable in the available screen band.
The selected position does not move the source map or change the game position.
Matched native Cherrygrove rival and Falkner captures verify visible fighters,
original scenery, species, levels, HP and menus with preserved state across F3.

Classic-only walking reviews warm the actual LCD presentation rather than
waiting for an inactive voxel renderer. This permits `--classic --walk ...`
recordings to exercise the same ordinary controller inputs.

`CRYSTAL_RENDER_TEST_CAVE_APPROACH=left` starts the disposable Union Cave floor
fixture at (11,26), facing left; the default remains (8,26), facing right.
Both validate the original floor and trainer sight and await ordinary encounter
rolls. A matched left-facing Sandshrew encounter and a streamed Route29 Hoppip
encounter verify visible Pokémon with real cave/forest scenery. A left-only walk
that exhausted the floor without an encounter was rejected, not accepted as QA.

On macOS, recording also uses the explicit image target; it does not need a
simultaneous one-shot screenshot. The verified four-second Violet Gym recording
uses the same fixture with `--walk UAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA`,
`--record-on-move --record /tmp/gym-turn --seconds 4`. Development builds with
unoptimized core code used `RUST_MIN_STACK=33554432`. Readbacks are bounded to
two outstanding images, and recordings retain real timestamps and source frames.

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

## Extracted battler rows

The exact `TACKLE`/`BattleAnim_Tackle` root requests modeled rows by default in
3D mode. A native Route 31 recording with the prototype/cache flags removed
verifies its impact, recovery and HP/menu restoration at 1280×720. WebGPU preview
reviews also verify visible actors and impact after resizing the browser from
1280×720 to 960×600. These checks do not certify every move or interruption.

`WATER_GUN`/`BattleAnim_WaterGun` remains experimental. Native review uses
`CRYSTAL_BATTLE_ROW_PROTOTYPE=1`; location-tester browser builds also accept
`battle-row-prototype=on`. All modeled-row admission requires normal modeled battlers,
recognized source event structure and a complete two-row OAM rectangle. Partial
or multiple strips, unsupported sequences and special appearances retain the
source renderer. Live source OAM, including its final deinitialization frame,
remains the lifetime authority.

`CRYSTAL_BATTLE_ROW_CAPTURE_CACHE=1` is a separate opt-in that also requires the
row prototype. It reuses eligible static actor captures only after the matching
render draw, output blit and submission are acknowledged; changed appearance,
scene or target state invalidates reuse. Capture reuse is disabled by default
and unavailable in WebAssembly. A matched native Tackle pair verifies identical
actor/field pixels on shared source frames and removes the measured first-row
hitch; it does not improve the median capture interval. See the
[bounded timing result](battle-3d-performance.md#experimental-actor-capture-reuse).
The matched Water Gun pair also retains identical actor/field pixels on eight
shared ticks, including its ripple; its overall cached timing is worse. Water Gun
still needs independent browser and resized-attack inspection. Broader source-OAM,
reveal and interruption review remain open.

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
