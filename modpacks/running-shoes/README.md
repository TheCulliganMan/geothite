# Running shoes (default)

The default core modpack enables running shoes from the start, including for
existing external crystalpacks and saves. Hold B (`X` on the graphical keyboard)
and a direction to move on foot at twice walking speed. Release B to walk.
Running is available indoors and outdoors without an item or story unlock.

The Rust core keeps one tile per step and ordinary collision, object occupancy,
encounters, step counters, coordinate events, trainer sight, warps and map
connections. Bike/surf speed, ledge timing, forced movement and authored script
movement are unchanged. Menus, dialogue and battles retain B's normal action.
The existing walking sprite frames animate at the faster step cadence.

Modpack manifests can set `payload.running_shoes` to `false` to disable it, or
`true` to explicitly enable it. Omitting it inherits the enabled core default.
This optional setting leaves legacy pack serialization and pack identities
unchanged when omitted; no game content or generated pack is shipped here.

The text client uses Shift + arrows for a simultaneous B + direction chord
(terminals do not reliably expose independent held keys). The stdio MCP `move`
tool accepts `run: true`, for example `{ "direction": "right", "steps": 3,
"run": true }`. Both call `VisibleShellController::press_buttons`, which shares
the production input ownership and scene continuation with `press`.
