# Route 6 southern woodland grass

Five native metatiles extend the existing woodland grass into its northern
approach: (6..10,13) in MnR06C00R01. Twenty grass quadrants are added, none
removed. The extension requires two additional grass steps in either direction.
Existing ledges, terrace stairs, houses, bridges, water, signs, trainers and
pickups remain unchanged. The earlier rail-side candidate was rejected because
the rail's walking strip provided a dry bypass.

The full route and source before/after views were reviewed. Both production
2D cameras match source pixels exactly and were inspected. A native local
Save/Continue checkpoint passes. Production MCP verifies 373 calls, walking
through the new grass, a wild encounter there, RUN, menus and actual Continue.
The first encounter harness stopped after a battle in older grass; the revised
course stays in the new strip until the intended encounter is observed. Initial
partner, badge and location are explicit local QA fixtures, not earned progress.

Rust publisher: .local/route-grass-review/portable-r06-south-woods-grass-tools-v01/
revise_r06_south_woods_grass PARENT SHA256 south-woods-grass-before-v01.layout.json
south-woods-grass-after-v01.layout.json FRESH_OUTPUT. The publication receipt
records exact parent and pack hashes. Affected-crate all-target Clippy and
owned formatting pass; workspace formatting has unrelated existing differences.

Scoped woodland-crossing acceptance does not establish the full current Route 6
course or complete the statewide grass objective.

Expanded current-route evidence: `full-route-production-v01.json` records 3601
production calls across both chunks, both gate approaches and repeated walking
after Continue. This does not include neighboring-town transitions.
`current-terrace-native-v01.json` verifies Cut permissions, Ether once, Continue
and return through the existing stair approach.

Neighboring-town transitions are now verified by
`connections-production-v01.json`: 1912 production calls enter Brainerd and
St Cloud and return, with both checks repeated after Continue.
