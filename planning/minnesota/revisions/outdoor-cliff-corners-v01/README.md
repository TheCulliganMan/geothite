# Restore retaining corners

Seven rim boulders replace actual cliff corners in five Minnesota exteriors.
The reviewed source candidate restores native6c at western corners and native6d
at eastern corners. Five rocks move one metatile inward and one north onto
flat ground. The two rocks at the end of the narrow Iron Range shelf are
removed; putting either inside would block the entire shelf.

`source-plan.json` lists all12 exact terrain edits and predicted map guards
after `outdoor-boulder-setbacks-v01`. Its native parent is not yet published;
these guards must be checked against the actual parent before implementation.
No active map, terrain payload, event, actor, doorway, encounter, field move or
world pointer has been changed by this source review.

Full original maps, before/after crops and topology evidence are retained at
`renders/outdoor-stair-landings-v01/cliff-corners-v52/`. All preexisting ordinary
FLOOR/TALL_GRASS walking components remain connected with NPC bodies included.
That static result is not a native input proof. The winter alias retains its
icy grass, snow and pines.

The western R10 boulder stack at(11,5) remains under individual review. Its
adjacent and lower rock turns are more complex than an outer corner; blindly
replacing it with6c would create a new incorrect join.

`plan.json` is the executable corner-only Rust input. It uses seven typed
`corner_boulders` records, requires native68/69 immediately above each source
and native72 on its interior front neighbor, permits only flat unoccupied
diagonal destinations, and retains exact whole-map guards. The Rust builder
uses a separate `minnesota-outdoor-cliff-corners-v1` manifest and rejects repeated
installation. `source-plan.json` keeps the equivalent12-cell review ledger.

Native Rust publication, strict whole-pack preservation, native render review
and production traversal are queued after the exact boulder-setbacks parent.
The dedicated production-controller case is
`duluth_city::cliff_corners::outdoor_restored_corners_keep_inside_rims_and_deny_crossing_the_face`.
It checks every inside rim and adjacent interior step, rejects crossing the
outer side/front, and saves/Continues each actual reached corner. Its initial
local placement and legal partners are setup, not earned access.
This candidate is not selected.

The composed native terrain v02 child now includes Mina's separate turn and
these seven corners. `renders/outdoor-stair-landings-v01/terrain-publication-v69.json`
binds the actual completed Cargo-built public API library, source plans and
pack hashes. `corner-preservation-v69/receipt.json` passes all12 exact terrain
edits across five maps, preserving all other data/runtime/art/audio bytes
including Mina's already corrected facing. All377 audio checks pass in
`terrain-audio-v69.log`. The candidate remains unselected pending native input,
new GPU views and renderer animation/icy-rustle verification.

The composed terrain v02 production MCP run now passes all17 boulder/corner
cases with210 actual calls and empty stderr, including Mina's formerly failing
upper-rim crossing. Evidence: `current-terrain-stdio-v72/receipt.json` and
`current-terrain-mcp-proof-v72.json` under the outdoor-stair render folder.
Initial local position/partners/Repel are explicit setup. No trainer is marked
defeated and no state is repaired after input. Mina's retained lower-bench
battle, scaled roster and fresh renderer checks remain separately queued.
