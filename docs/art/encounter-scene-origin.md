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

## Actual encounter geometry

The renderer pins the matching built terrain, footing and source texture for an
encounter generation. A desired asynchronous cache key alone is insufficient.
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

Browser GPU review, Surf scene placement, broad walking-location visual coverage, trainer
encounters, gyms, caves and ice rooms remain separate visual and provenance
gates; this is not whole-game location coverage.
