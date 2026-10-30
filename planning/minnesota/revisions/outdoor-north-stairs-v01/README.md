# Four north-facing gray stairs

The four retained native74 crossings still rendered as pale road caps with no
visible steps. Their direction and collision were correct; retaining that
collision is separate from accepting their art. The source comparison is in
`renders/outdoor-stair-landings-v01/north-ramp-materials-v76/`. Earlier failed
preview attempts remain saved.

`plan.json` guards the three exact native map modules and four placements.
Private Headwaters87 uses already imported86 gray lower tread glyphs as its
upper rows and native74's paved inside landing as its lower rows. It serves
the three R05 approaches whose actual next inside block is01. Private88 uses
the same treads above native70's brown rock landing for R07, whose actual next
inside block is71. Both retain native74 collision WALL/FLOOR/FLOOR/FLOOR and
allocate no glyphs, palettes or images. No pale cave-floor pixels are imported.
The explicit inside_landing field must match that native inside block.

The all-paved v70 preview was wrong for R07, and the all-rock v75 preview was
wrong for the three R05 roads. Both remain retained; v76 matches each actual
landing. Do not generalize one reference tile or material to every flight.

R05's two maps already use mn_outdoor_headwaters_stairs. R07 still uses shared
mn_headwaters and adopts the existing private alias; original prefix collision
is required to agree. Existing Cut/Whirlpool replacements and animation
registrations are inherited by that alias. Shared Johto/Kanto/Headwaters art
remains untouched. Native family hashes and whole-map guards must pass before
publication.

The public Rust builder is revise_minnesota_outdoor_north_stairs; its CLI is
revise_outdoor_north_stairs PACK SHA256 PLAN OUTPUT. The baseline
is ten-boulder v02, but neither Mina's turn nor the seven-corner phase changes
these three map modules, so the composed terrain child can be used with the
same guarded native preimages. The old MCP results prove native74 movement
only; the new publication and input evidence below applies to private87/88.

The native CLI has now built and published
`content-packs/minnesota-current-outdoor-north-stairs-v01.crystalpack`, SHA256
`694e0700d4777b4bc1c06065e92a3ca58f8140ef94a33412120ddec05bf27ca0`.
`renders/outdoor-stair-landings-v01/north-publication-v78.json` binds the exact
parent and source plan. `north-preservation-v78/receipt.json` passes all four
edits across three maps, both native landing recipes, all original glyphs,
palettes, foregrounds, field-move replacements and every unrelated pack root,
including compiled audio. `north-stdio-v78/receipt.json` passes all four new
flights through40 real production MCP calls with no post-input state repair.
All four fresh-source native GPU views now pass with zero pixel mismatches in
`renders/outdoor-stair-landings-v01/north-native-v78/capture-progress.json`.
The complete four-view production contact sheet was inspected. The current
five-alias animation regression also passes, including water/flower frames and
winter clock/icy-rustle behavior. No active world selection changes or earned
regional progression are claimed. Required parent-cohort asset and Bevy
all-targets Clippy now pass in the v66 logs.
