# Frozen battle location provenance

`VisibleShellController::battle_presentation_origin()` and the read-only
`BattlePresentationOriginFrame` expose one immutable source value for a battle.
This is presentation metadata. The first checked encounter scene can reuse the
actual built overworld geometry; unsupported or incomplete evidence retains the
existing arena or source presentation.

The source value retains the canonical map ID, authoritative player tile,
facing, movement mode and source frame. Normal overworld encounter results also
retain their checked map, tile and surface. Fishing carries the already validated
facing water target through the transient cast result and freezes it in the
shared cast-presentation helper used by Pack USE and registered-item actions.
The player and core fishing encounter remain on the shore tile.

Other entry paths preserve the source pose at battle entry. Their checked
contact remains absent when no authoritative result supplies it. In particular,
this checkpoint does not establish Headbutt, Rock Smash or Sweet Scent target
provenance, nor a complete Surf playthrough. Map names are never treated as
proof of water contact.

Pending provenance survives the existing entry presentation reset. Repeated
entry preparation reuses the same value, and visible exit clears it only after
retained narration drains. Save load and blackout clear it. The controller-local
generation survives reset but is not a gameplay or replay identity. The complete
runtime session, RNG, save schema and journal command/result payloads are
unchanged. A stable frame does not trigger a Bevy resource change notification.

The checked SquirtBottle path also freezes the actual object identifier, object
script, visible feet, dispatched interaction script and executed `startbattle`
command. It binds only when those witnesses agree with the resulting static
encounter. Refused, cancelled, stale or unrelated interactions cannot reuse a
candidate. Fishing supplies a separately typed checked-water target, with no
invented object identifier or enemy actor.

## Fishing from the actual shore

Registered-rod and Pack USE casts share the existing authoritative cast result.
Only a cast that really commits a wild battle can bind its checked water tile.
The player and gameplay encounter remain on the original shore tile. The
presentation uses the actual settled player support and the checked adjacent
water support, both inside the verified source map and completed terrain grid.
The water pose is a neutral supported battle stance, not a swimming animation.

Pack is a full-screen source surface. When a carried rod is available, its opening
boundary can retain one validated source-only terrain frame. That private value
does not publish a battle. A successful cast consumes it only if the original
pose, extent and terrain revision still agree. Closing Pack, navigation, load,
or stale evidence clears it; a cold opening cannot adopt a later frame. The
registered shortcut uses its immediate pre-cast field frame.

The committed battle remains hidden through the rod animation and bite notice.
The shared scene selector keeps the retained field until acknowledgment, then
releases the ordinary source battle entry. This fixes a premature battle-HUD
frame without changing fishing phases, gameplay ticks, DIV reads or outcomes.
The 3D world publisher also restores the validated field after Pack has hidden
its prior publication; an already active fishing field remains held unchanged.

## Ordinary walking grass encounters

A normal one-tile grass step can bind its committed source contact without
changing encounter rolls, walk interpolation, entry timing or command replay.
The frozen witness comes from the exact settled moved-from sprite or its
original interpolation segment. Missing or stale initial evidence is not
repaired from an unrelated later frame. The landed source support is resolved
inside that genuinely witnessed terrain grid.

During continuous walking, core battle commitment does not clear the optional
world frame before the visible step has landed. Extraction continues only for
the current bound encounter while the existing field presentation owns the map;
actual battle/full-screen entities and the original replacement boundary still
clear it. The battle LCD flag and source clocks retain their original meaning.

Random encounters have no overworld enemy actor. The enemy support is explicitly
derived: two source tiles away in facing, right, left, then back order. Both
corridor cells must be unoccupied plain land. A bounded 7×7 source collision
sample records the precise approved cells; it is not a claim that any species
will fit there. The renderer separately requires the complete animated body
footprint to fit the union of those cells and the matching built elevation.
Holes, absent built ground, excessive body size and incompatible heights reject
the location. Source move offsets are checked against the same fixed supports
without camera movement, species resizing or per-frame mesh reconstruction.
An offset outside the proven region retains the original source presentation.

The disposable `--walking-encounter` fixture starts on a checked Route29 grass
edge and awaits ordinary movement. It never starts or forces an encounter.

## Ordinary cave-floor encounters

The same walking bridge asks the runtime's checked encounter-surface query,
which uses verified cave/dungeon metadata for ordinary land encounters. Both
the moved-from and landed collision must still be plain floor or grass;
water, ice, forced movement, doors, ladders and other special surfaces remain
outside this increment. A cave-looking map name or ordinary route floor is
insufficient. Source collision, occupancy, landing evidence and full animated
body clearance remain independent checks.

The disposable `--cave-encounter` option selects a checked UnionCave1F floor step
through the shared walking preview and awaits ordinary movement/encounter rolls.
It does not start a battle or alter encounter odds.

## Actual encounter geometry

The renderer pins the matching built terrain, footing and source texture for an
encounter generation. A desired asynchronous cache key alone is insufficient.
For a walking encounter, an older mesh grid can be reused only after exact
source, texture and priority correspondence across its full overlap with the
witnessed grid. Both grids, source extent and scroll allowance are checked. The terrain
revision is a live-viewport key and can change while the retained grid remains;
exact immutable per-cell correspondence supplies the scrolling proof. For authored
animated textures, two phases are equivalent only when both belong to the same
complete immutable image family. The actual completed build must also prove a
geometry-invariant Flat/Water/Waterfall profile; any applicable live drawing or
ground-sample override disables this exemption. Static/unproven textures and all
source/priority metadata still require equality. Palette mappings and time of
day participate in the art cache identity, so unrelated art cannot inherit a
water-animation exemption. Feet are sampled in the actual built grid and translated back exactly
once; only the shared original-map acreage can support the battle camera.
The rendered frame includes the authoritative map extent separately from its
padded cache grid. Terrain crossing that extent is split into inside/outside
domains while preserving the complete overworld surface. Unsafe mixed-attribute
crossings remain unresolved and are conservatively rejected for the battle.
Mutable flower geometry and image data are copied once; immutable geometry uses
existing handles. A stable encounter does not rebuild meshes every frame.

The battle uses the actual landed feet and a single map-to-battle scale, with four
battle units per core map tile. Species retain their shared 2.65 units-per-meter
physical sizing. The camera fits the complete animated bodies and follows the
source encounter corridor; it never spreads or individually resizes combatants
to make an invalid pairing fit. Clipping, overlap, missing footing, incomplete
terrain or unsupported source presentation keeps the existing fallback.

Nearby world actors are not duplicated into the battle scene. Geometry and
materials are isolated to the battle render layer, source palette flashes affect
only those copied materials, and qualifying whole occluders fade smoothly. The
scene is retired on exit or generation replacement. Combat, collision, source
move clocks and original effect projection remain controller-owned.

The first visual gate is the real Route36 SquirtBottle interaction with the
overworld geometry warmed beforehand. The disposable `immersive_battle_3d`
example accepts `--route36-encounter`; use the registered bottle and normal
confirmation controls to enter the actual source battle. The example
also supports `--fishing-encounter`, which places a fresh session at a compiled
Route32 shoreline with a carried Good Rod. Use either Pack USE or the registered
shortcut; bite and species selection keep their ordinary random path. The recorder
can wait 30–900 seconds for the first requested cue with
`CRYSTAL_CAPTURE_ARM_SECONDS` (default 180). This only bounds the pre-recording
wait and does not change game or recorded time.

## Validation

All ten origin/lifecycle and registered-item input checks pass. The real caught
fish retains the exact shore snapshot and checked water target through the
visible animation and controller A entry. The no-bite case records exactly the
bag cast and ordinary ItemUse event drain; replaying both produces the exact
actual game state without creating encounter provenance.

Six runtime fishing checks pass, including unchanged random-divider traces,
atomic replay and transient target metadata excluded from journal payloads.
Four runtime asset-mount checks, five battle-intro checks and four capture-palette
checks also pass. These 29 focused checks are not a claim that the entire
controller suite was run.

Native compilation and the full-feature WebAssembly check pass. A disposable
native game session booted the existing modeled battle and executed Shadow Ball
through ordinary controls, including its original palette transition and actual
damage. This is a functional smoke check, not a new scene or performance result.

The renderer-neutral controller does not advance the fishing animation itself;
the input tests drive its existing renderer-owned clock before acknowledging
the notice through controller A. Text-client fishing-clock coverage remains a
separate gap. The terrain increment passes 901 voxel regressions, 24 render-API
checks and focused controller/recorder checks, plus native compilation and the
full-feature WebAssembly check. Native visual evidence is tracked separately
from these tests. An 800×600 native run entered the real Route36 encounter through
the registered SquirtBottle and showed both creatures on the original tree-lined
corridor, then executed Ember and its actual damage response. The unretimed
12.17-second recording contains 247 frames; update median was 42.91 ms and p95
79.80 ms on the cloud software renderer. That is functional incremental evidence,
not a fluid-60-fps claim or a matched performance improvement.

Native registered-rod and Pack USE sessions have both reached genuine Route32
water battles. The Pack session also ran from the encounter through the normal
result text and returned to the same complete shoreline. The earlier registered
capture contained 323 frames over 12.07 seconds, with update median 34.94 ms and
p95 48.65 ms; its missed Supersonic is ordinary gameplay, not a substituted hit.
Fishing tests cover both input paths, exact source command replay, no bite,
cancel, stale/missing evidence, reload and retained entry/exit lifetime. Separate
presentation checks require the field, rather than battle HUD/commands, through
the rod's existing phase clock.

The walking handoff has 33 passing focused controller checks, including a real
continuous-input Route29 sequence, exact source journal replay, immutable
witnesses and the field-to-battle extraction boundary. Retained-grid proofs
pass 912 voxel regressions and 24 render-API checks, plus native compilation
and the full-feature WebAssembly check. A native Route29 walk produced Hoppip
through ordinary encounter rolls and accepted the actual grass/path geometry
at landed tile (46,12), facing north. Both original supports and canonical
species scales remained fixed. Ember played with its original cues and real
super-effective damage. Its source-only result presentation remains a fallback.
The short unretimed capture contains 88 frames over 4.13 seconds (update median
29.13 ms, p95 84.63 ms) on software OpenGL. The recorder ended that session. A separate native Pidgey encounter facing
south also accepted the original route terrain; Run and its result text returned
to the complete same field, and ordinary walking controls resumed.

Browser GPU review, Surf scene placement, broad walking-location visual coverage, trainer
encounters, gyms, other cave layouts and ice rooms remain separate visual and provenance
gates; this is not whole-game location coverage.


## Queued faint and withdrawal presentation

A future queued faint or withdrawal does not force the current HP/result scene
back to the source renderer. The fallback now reads only the animation that has
actually started. Original faint row removal and recall clipping still use the
source presentation when their authored sequence starts; capture, send-out and
trainer-exit gates remain independent.

Four focused regressions cover production faint staging, recall and its label
alias, the real terminal KO controller path, and independent source-only gates.
They verify exact runtime state, command history, random-divider trace, source
clock and repeated extraction stability. These passed within 58 focused
controller checks, alongside native compilation and the full-feature Wasm
check. The motivating Route29 Ember recording showed the premature switch at
HP drain before fainting began. A corrected native Ember KO retained the modeled scene through all 48 HP-loss
updates and the critical-hit result text. The unretimed capture contains 172
frames over 6.07 seconds (update median 30.95 ms, p95 39.11 ms) on the same cloud
software renderer. That result uses the existing generic cave arena: actual
location acceptance and visual quality remain a separate gate.


## Cave-floor visual gate and remaining cost

The floor bridge and authenticated texture-phase proof pass 917 voxel checks,
24 render-API checks and 62 focused controller/producer checks, plus native
compilation and full-feature WebAssembly. The producer tests drive the real
compositor across cave water/scroll boundaries, preserve retained-grid
allocation and source clocks, and distinguish palette/time art-cache families.

An ordinary native UnionCave1F step from (7,26) to (8,26), facing east, produced
a real Zubat encounter. Its exact retained floor, rocks, walls and nearby water
passed the scene proof with original supports and canonical species sizes.
Ember applied real damage and KO. The source renderer handled the six recorded
updates of actual FAINT_MON row removal; the retained cave then returned for
the faint-result text. The recorder ended this short session, so this is not a
separate native cave exit test.

Performance remains a material gap: the unretimed 800×600 software-OpenGL
recording captured 58 frames over 6.24 seconds, with update median 83.21 ms and
p95 196.56 ms (about 9 captured frames/second). That capture is substantially slower
than the earlier generic-arena capture; this is functional,
incremental location coverage, not a smooth-performance or polished-cave claim.
Geometry-derived animated profiles, unsupported layouts and incomplete body
clearance continue to use the existing fallback.


A separate no-readback measurement at the same source contact and settings
reached a Sandshrew battle. Its quiet held-menu interval contained 1,882
consecutive samples with median 28.13 ms and p95 72.73 ms; occasional stalls
remained. The opponent and fitted camera differed from the Zubat recording, so
this is not an exact A/B comparison and does not assign all overhead to capture.
The slow capture alone does not establish the steady gameplay rate.

The native location tester now has a default-off
`CRYSTAL_CAPTURE_PACING_PROBE=1` measurement. With image recording,
`--record-on-move` and at least 36 seconds, it holds the ordinary scene and
admits screenshot readbacks only during seconds 14–22 of the real move-triggered
recording clock. Separate baseline, capture and drain intervals record focus,
request/completion counts and callback latency. Analysis excludes pending work
and phase boundaries. Probe images retain actual timestamps and deliberately
have no continuous-video manifest across the unrecorded intervals. This tool
never drives inputs, pauses source animation, changes geometry or changes game
state. Its purpose is to isolate capture cost before choosing an optimization.


A synchronized repeat on the unchanged native executable measured the same
accepted Zubat cave scene before, during and after screenshot admission. The
two eligible no-readback baseline means were both 28.8 ms per update; capture
averaged 35.46 ms. All measured windows stayed focused. Native process/cgroup
sampling recorded no throttle increments, major faults, reclamation or memory
limit events during those windows. This warmed run visited a generic Onix
battle first, so it does not rule out first-use pipeline costs. The earlier
severe stalls remain unresolved; no renderer optimization occurred between
these measurements.

The capture files were effectively uncompressed at 1,440,773 bytes per 800×600
frame. A paired offline test on six actual frames compared the existing
Fast/NoFilter PNG path with Fast/Sub. All 240 roundtrips preserved the exact
RGB bytes, and Sub reduced file size by 76.3%, but its production encoder-to-file
mean rose from 4.093 to 5.262 ms. It was slower in 43 of 48 measured pairs. That
filter change was rejected as a pacing optimization. The game keeps its
geometry, materials, scale, resolution and existing recorder filter.


## Settled trainer-table contact

Ordinary trainer sight can now freeze its completed seen-text scene. The contact
records the actual trainer after approach and `writeobjectxy`, its object/script
identity, facing, trainer class/ID/event and exact trainer-table command. It
binds only when that matching authoritative trainer start succeeds. The prior
sight event's pre-approach tile cannot substitute for the settled contact.
Cold, stale, missing or mismatched player/object/terrain evidence retains the
existing fallback. The candidate does not publish a battle during seen text.

The trainer's occupied tile is separate from a derived Pokémon support. A
bounded set of original-map, unoccupied floor cells supplies the presentation
corridor. The consumer checks exact settled feet, the separate trainer witness,
actual built elevations and every cell under the complete animated body. Source
attack displacements use that same clearance without rescaling or repositioning
a creature. Entry preparation preserves the bound witness; terminal narration
retains it until the ordinary visible exit.

The first native gate used `--gym-encounter`: a fresh player at VioletGym (5,11)
walked Up into Abe's original sight line. Abe moved from (2,10) to (4,10), facing
right; the player stopped at (5,10), facing left. `TrainerBirdKeeperAbe` command 0
is the original trainer-table command for BIRD_KEEPER/ABE and his level-9
Spearow. The Pokémon support was derived at (5,12), not at Abe's field position.
The renderer accepted the actual blue gym floor, openings and entrance statues
with original terrain coordinates and canonical species sizes.

All 920 voxel checks, 24 render-API checks and 63 focused controller checks pass,
along with native compilation and full-feature WebAssembly. The native Ember
recording captured 209 real frames over 8.08 seconds at 800×600, with a 600×450
3D target on software OpenGL: update median 36.34 ms, p95 49.04 ms. Nine updates
used the original faint-row removal before the actual gym returned for result
text. That recorder ended the session. A separate native run then completed
Abe's original victory, experience/reward and post-battle dialogue, restored
the same gym field, and responded to a directional turn after text closed.
It ended before a second movement press, so no extra post-battle tile step is
claimed. Room/model polish and first-use stalls remain open. Falkner's explicit
scripted battle path and other gym layouts are not claimed by this sight gate.


## Ice Path floor fixture

`--ice-encounter` prepares a fresh IcePath1F session at (8,16), facing right,
beside the original sliding ice. It does not start a battle, choose an opponent
or change RNG. Ordinary steps within (8,16)/(9,16) must consume the source
cooldown and encounter rules. This CAVE map already uses the checked ordinary
land path; no production collision or renderer gate is widened for its name.

The fixture checks the mixed drawing of metatile $02 and its complete unoccupied
FLOOR band at x=6..13, y=16..17. Neighboring $1f ICE cells retain permission $23
and their original drawing. The 7×7 area around (9,16) has exactly 24 approved
floor cells, with intervening ICE, WALL and UP_WALL cells excluded. The initial
forward presentation corridor reaches (11,16); a real reverse encounter uses
its independently checked opposite corridor. All final species footprint,
built-height, retained-scene, camera and omission checks remain authoritative.
Sliding encounters, forced movement and cold or stale evidence keep their
existing fallback. The original ice boulders, masses and shelf models are
reused; this fixture adds no generic environment or duplicate geometry.


The native gate then rolled a real level-23 Swinub after a floor step from
(10,16) to (11,16), facing right, with derived enemy support at (13,16). The
operator walked normally after the fixed starting setup; the realized contact
was not forced to be the first pair. Original icy boulders, shelf edges and
adjacent ice passed the complete built-ground, camera and omission checks.
The 17 walking/source regressions and four trainer regressions passed, along
with full-feature WebAssembly and native compilation.

Swinub's priority Endure acted before the selected Ember. Its existing source
fallback remained visible for 87 updates from 0.037930 to 2.206090 seconds,
then the same actual icy scene returned. The unretimed 800×600 capture contains
225 real images over 8.13 seconds; update median was 27.62 ms and p95 42.82 ms.
Those figures mix source and modeled presentation and are not a steady 3D
performance comparison. Endure is not claimed fully 3D by this fixture. Species
art, remaining source fallbacks and sliding-ice coverage remain separate gaps.
