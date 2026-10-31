# Route 9 southwest grass pocket

Three existing native metatiles become grass: (6,3), (5,6), (5,7) in
MnR09C00R01. Twelve grass quadrants are added; none removed. The narrow
approach now crosses two grass tiles before the existing Cut gate. The Revive
pocket has two grass steps from the opened gate. Sign access remains clear.
The gate, cliffs, stairs, actors, scripts, encounters, art and audio are unchanged.

Native Cut regression passes missing-move and missing-badge denials, authorized
Cut, Revive once, Continue persistence, tree regrowth and return from inside.
Production MCP v02 verifies 189 calls with an encounter in the added grass,
RUN, menus and actual Save/Continue. Initial partner/badge/position are explicit
local QA fixtures. A first harness attempted to plan through the collected item
actor; its static planner conservatively retained that body. The corrected
course walks beside it. No runtime state or save identity was edited.

Both native 2D views were inspected and match source pixels exactly. Whole-route
and before/after source renders are preserved. All-target crystal-assets Clippy
and owned leaf formatting passed. Workspace formatting has pre-existing diffs.

Rust reproduction: .local/route-grass-review/portable-r09-southwest-grass-tools-v01/
revise_r09_southwest_grass PARENT SHA256 southwest-grass-before-v01.layout.json
southwest-grass-after-v01.layout.json FRESH_OUTPUT. The publication receipt records
the actual parent and hash. Packs and generated renders remain ignored.

This is a scoped grass-pocket acceptance. It does not establish full current
Route 9 traversal or completion of the statewide grass objective.

Expanded course: `full-route-production-v02.json` records 2957 production
calls through all four chunks, real Cut from inside the saved pocket, both
stair courses, Bemidji and Grand Rapids round trips, repeated after Continue.
All native stair tread coordinates occur in production responses. Complete
current route and neighbor maps, route tilesets, encounters, trainer teams and
battle inputs are exact to the tested publication; logs are not relabeled as
fresh runs on the later pack.
