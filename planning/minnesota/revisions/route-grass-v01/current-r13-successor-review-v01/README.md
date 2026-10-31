# Route 13 southern winter pine grass

Four explicit native83 floor metatiles become native84 winter grass in
MnR13C00R02. Sixteen grass quadrants are added, none removed. The bend and
narrow passage now require ten grass steps each way rather than six. All
sixteen pre-existing inland ice quadrants, the Cut gate, water, stairs, actors,
rewards and unrelated maps remain unchanged. No North Shore ice is added.

The full source route and before/after detail were inspected. Both native 2D
cameras match source pixels exactly and retain winter grass foreground holes.
404 production calls verify encounters in the added grass, RUN, walking both
ways, menus and actual Save/Continue with immutable source/input saves and empty
stderr. Native Cut regressions pass on both parent and child: missing-move and
missing-badge denials, authorized opening, Revive once, Continue and return.
Initial partner, position and badges are local QA setup, not earned progress.

Rust publisher: .local/route-grass-review/portable-r13-pine-grass-tools-v01/
revise_r13_pine_grass PACK SHA256 pine-grass-before-v01.layout.json
pine-grass-after-v01.layout.json FRESH_OUTPUT. Exact commands and hashes are
retained in preservation and build receipts. Affected-crate all-target Clippy
and owned formatting pass; workspace formatting has unrelated existing diffs.

The original scoped acceptance above is supplemented by the full current land
course and restored northern apron below. Statewide completion remains open.


## Current Route13 land course and northern apron

Selected `route_grass_r13_boundary_apron_revision` restores the existing authored
snow-floor apron at MnR13C00R00 metatile (16,0), native8b to83. This joins the
route lane30/31 to Boundary Waters lane32/33 without changing connection offsets
or native gate requirements. The offset-only candidate was rejected by connected
geography validation and was never published or selected.

`boundary-child-preservation-v03.json` proves only that metatile and its raw block
mirror changed on the newer Iron Ventilation parent. All other maps, Bemidji's
plateau stairs, State Fair, coastal terrain, art, audio, encounters, trainers,
actors and scripts are exact. The two inspected native2D seam views have zero
source/native pixel mismatches. All2048 route source metatiles are bound exactly.

`full-route-production-v12.json` records6454 actual production calls through all
three chunks, both neighboring town connection round trips, two-way terrace
stairs, both inland ice slides, wild RUN, menus and Save/Continue. Four real nurse
visits in Ely heal between long legs. `apron-cut-native-v04.json` independently
verifies Cut permissions, opening, Revive once, regrowth after Continue and return.
The initial Voyagear60/Cut/eight-badge party and position are explicit local QA
setup, not earned campaign progress. Surf and paid preserve access are outside
this current course. Existing inland winter is intentional; no coast ice changes.

Earlier Mew run-only checks blacked out after failed escapes; another unhealed
long course carried poison into Continue and blacked out. Those diagnostic logs
and saves remain retained. The final course uses a regional native Cut fixture
and actual nurse healing; no save is retagged, teleported, healed or edited after
play begins. The preliminary parent-child full course also remains retained.

The Rust publisher is portable-r13-boundary-alignment-tools-v03/align_r13_boundary
under .local/route-grass-review. Its exact-parent SHA guard, native preimage/mirror
checks, runtime/geography verification and readback pass. Affected-crate all-target
Clippy and owned formatting pass; whole-workspace formatting reports unrelated
offset-allocator/exporter differences. Full source/build/publication commands and
hashes remain in the corresponding receipts and scripts.
