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
candidate. Fishing contact metadata is retained, but fishing scene placement is
not yet enabled by this bridge.

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
confirmation controls to enter the actual source battle. Its optional recorder
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
separate gap. The terrain increment passes 900 voxel regressions, 24 render-API
checks and focused controller/recorder checks, plus native compilation and the
full-feature WebAssembly check. Native visual evidence is tracked separately
from these tests. An 800×600 native run entered the real Route36 encounter through
the registered SquirtBottle and showed both creatures on the original tree-lined
corridor, then executed Ember and its actual damage response. The unretimed
12.17-second recording contains 247 frames; update median was 42.91 ms and p95
79.80 ms on the cloud software renderer. That is functional incremental evidence,
not a fluid-60-fps claim or a matched performance improvement.

Browser GPU review, fishing/Surf scene placement, trainer
encounters, gyms, caves and ice rooms remain separate visual and provenance
gates; this is not whole-game location coverage.
