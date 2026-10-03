# Frozen battle location provenance

`VisibleShellController::battle_presentation_origin()` and the read-only
`BattlePresentationOriginFrame` expose one immutable source value for a battle.
This is presentation metadata; it does not yet replace the battle arena with the
actual encounter map.

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

The renderer still needs to bridge this value through `crystal-render-api`, pin
matching built terrain and footing data, resolve the visible landed pose, and
frame an actual encounter scene. An asynchronous desired cache key is not proof
that the matching geometry was built. The first planned visual gate is the real
Route36 SquirtBottle interaction with the overworld geometry warmed beforehand.

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
separate gap. Actual encounter-map rendering and browser GPU review remain
pending.
