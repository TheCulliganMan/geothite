# Native 2D city review

The current city-render direction is **2D only**. Continue building and coastline
work in the native map layout and source-art assemblies. The review uses
`render_at_location --view 2d`, which sets `voxel_view_enabled=false`.

The selected current composition is
`content-packs/minnesota-interiors-selected-r12-v01.crystalpack` (730 maps),
SHA-256 `1d60940da0f42dbce7bd5a595ccd8d8b8d68c2e84dcfa4ca505c67d5abd6595b`.
It preserves the complete repaired St Paul module. All three city controller
cases pass again (97.66s): 39 ordinary door pairs, both clinics' free care and
native PCs, both bridges, Surf underpasses, river seams and Continue. Both
clinics and the west river bend have fresh 640×576 native captures with zero
pixel differences and no exclusions. Those frames and the whole city were
personally inspected. All 377 audio programs pass, and Red Wing has no station
or timetable listing. See the
[selected union and retained evidence](../../interior-life/selected-union-v01/README.md).

The preceding composed local interior candidate is
`content-packs/minnesota-interiors-r03-current-story-v01.crystalpack` (729 maps),
SHA-256 `86250e01158135ef6e26238533d30edcf97dbea03d90491cc9aded701cb1a816`.
Its complete St Paul module equals the accepted repaired city exactly. Ten
fresh facade captures on its immediate story parent and four river captures on
this final child match all native source pixels, with no exclusions. All
fourteen frames and the whole city were personally inspected. Complete house
and Center assemblies, indexed doors, straight banks, joined corners, both
bridges, the southern quay and working basin remain intact.

All three city controller cases pass on that final child (71.66 seconds): 39
ordinary entrances and returns, both Centers' free care and native PCs, bridge
walks, Surf underpasses, reciprocal river seams and Continue. All 377 audio
programs pass again. No Red Wing station or station timetable listing exists.
Fixtures declare placement, party and optional prerequisites; they do not
establish earned campaign progression.

Images and receipts are retained under
`planning/minnesota/renders/interior-life/current-story-city-native-v01/`,
`current-story-shore-native-v01/` and `current-story-composition-v01/`.
See [the exact-parent composition](../../interior-life/current-story-v01/README.md).
The shared regional selector is preserved; this local integration does not
complete the all-interiors objective.

A preceding verified local interior candidate is
`content-packs/minnesota-current-voyageurs-ice-v01.crystalpack` (709 maps), over
the accepted R12 shelter parent. Fourteen fresh city frames again match every
pixel and were personally inspected through retained native-size review pages.
The same three St Paul controller cases pass (26.57s), within 37 passing local
gameplay cases. The source module preimage and repaired facade/river geometry
below remain exact. New receipts are under
`renders/st-paul/current-root-native-v01/`; see
[the current composition status](../../interior-life/COMPACT-REBASE.md).
This is a verified local candidate, with later R03/R13 terrain, the fair and
International Falls still to compose. It does not replace the shared selector.

## Initial compact replay

The compact rebase now restores this repair over
`content-packs/minnesota-compact-shore-gate-v01.crystalpack`, producing
`content-packs/minnesota-compact-st-paul-repair-v01.crystalpack` (695 maps).
[compact-contract-v01.json](compact-contract-v01.json) pins the actual current
St. Paul module (`07d6f4d8aeba275f4267a375b4a1583fa14425a30710a22d5fbf2973364f708d`)
and retains every indexed preimage. It changes 164 metatiles and 20 warp
positions; five of the original repair's 169 cells were already revised in the
compact parent. All city attributes, regional fishing, reciprocal river
connections, other maps, interiors, native art and audio remain exact.

Fourteen fresh headless production 2D frames match all 5,160,960 compared pixels.
All fourteen frames and both complete city images have been personally viewed.
The rebuilt renderer and pack hashes are retained in
`renders/st-paul/compact-repair-native-v02/capture-progress.json`. The first
attempt used an older renderer which rejected newer Minneapolis connection art;
that failed capture remains under `compact-repair-native-v01/`.

The four applicable asset cases pass (2.26s). All three current production
controller cases pass (35.21s): 39 ordinary entrances and returns, clinic care
and PCs, both walking bridges, both Surf underpasses, reciprocal river seams
and Save/Continue. All 377 audio programs pass PCM/hash/frame/loop verification.
This establishes the repaired city's local behavior, not earned campaign access
or completion of the wider Minnesota interior objective.

The Detroit Lakes and Moorhead compact v02 children retain the repaired St. Paul
module exactly. Two new connected context captures on the 704-map Moorhead child
also match every pixel; originals and full comparisons are retained under
`renders/st-paul/compact-context-native-v01/`. Red Wing's station is absent from
both inspected packs. The active shared mapping pointer is not changed by this
local composition.

## Historical composition evidence

The earlier composition check used the 717-map
`content-packs/minnesota-interiors-split-rock-v01.crystalpack`, SHA-256
`978ab6e3f674acf72967ca2f6dba799d590b47f34ef855ea4cfb8cfa7c9c5397`.
Its complete St. Paul module, Kanto tileset, all native runtime source files and
audio manifest equal the prior reviewed gate pack exactly. Nine fresh headless
640×576 native 2D captures match the source compositor with zero differing
pixels across all 3,317,760 pixels; none are excluded. All nine views and the
complete city have been personally inspected. The existing production capture
executable is recorded by hash in the capture receipt; the three controller
regressions were rebuilt from the current checkout and pass in 20.01 seconds.
They exercise all 39 ordinary entrances and returns, both bridges, both Centers'
free healing and native PCs, reciprocal river seams and Save/Continue.

This review leaves the pack unchanged. The complete images, raw captures,
source comparisons, preservation receipt and archived review scripts are under
`planning/minnesota/renders/interior-life/city-2d-v03/`. Logs are
`target/interior-life/city-2d-captures-v03.log`,
`city-2d-source-comparison-v03.log`, `city-2d-preservation-v03.log` and
`city-2d-controller-v03.log`. These are explicit visual and gameplay fixtures,
not evidence of earned campaign progression or completion of the regional
interior build-out. No voxel render was used in this review.

A prior composition check uses the 713-map
`content-packs/minnesota-shore-gate-v04.crystalpack`, SHA-256
`d97bb1782b58fbf4a77f71a0f446c5b86911889de0c466549bda2324a526cec0`.
Its complete St. Paul module, native tileset/collision, scenery/NPC source art
and audio manifest equal the previously reviewed city parent exactly.
Nine fresh headless 640×576 production captures match the source compositor
with zero differing pixels across all 3,317,760 pixels. No pixels are excluded;
water and flower animation phases remain coherent across each complete frame.
All nine frames and the whole city have been personally inspected.

All three `st_paul_refinement` production-controller cases pass on this pack
(51.55 seconds): 39 ordinary building entrances and paired returns, both walking
bridges, both Centers' free HP/status/PP recovery and native PCs, Surf under both
bridges, reciprocal river seams and Save/Continue. Initial position, optional
trainer victories, the Surf partner and badge are explicit fixtures.

Current images, raw GPU captures, references, differences, before/after source
details and archived capture/comparison programs are retained under
`planning/minnesota/renders/interior-life/city-2d-v02/`. The before images use
the actual `minnesota-interiors-ely-v03.crystalpack` exterior at the same source
coordinates. The current logs are `target/interior-life/city-2d-captures-v02.log`,
`city-2d-source-comparison-v02.log` and `city-2d-controller-v02.log`.
The pack remains unchanged and the private capture executable was removed.
This city verification does not complete the regional interior build-out.

The original 2D review below remains as a separate historical receipt.

The repaired St. Paul source contains 17 complete native facade replacements,
including both Pokémon Centers, both Marts, ordinary houses and the bakery,
cafe and inn. Their exact indexed doors remain installed in
`content-packs/minnesota-international-falls-v02.crystalpack`. River changes
replace the repeated corner fragments with straight banks and explicit convex
and concave joins, retain both bridge crossings, and form a working basin.

The complete source city is 40 by 34 metatiles. The whole-map native-art review
and player views are retained under
`planning/minnesota/renders/interior-life/city-2d-v01/`.
The capture fixtures place an empty-party trainer facing south at noon; they
are visual setup and do not claim earned campaign progression.

All nine complete 640×576 production frames match the source compositor with
zero differing pixels across 3,317,760 pixels. Native water/flower animation
phases are coherent across each frame; no pixels are excluded. All nine native
views and the full city were personally inspected. The 17 replacement assemblies
also pass exact block, indexed-door, DOOR collision, unoccupied FLOOR return checks.

The pack SHA-256 is `9615a405a730e9f2bd6a56af3dc9b9ea618d9336f4525ce046cc42a81b5642cf` and remained unchanged. The private capture
executable was removed. All originals, source references, differences and script
snapshots remain in the ignored render history. The current run logs are
`target/interior-life/city-2d-captures-v01.log` and
`target/interior-life/city-2d-source-comparison-v01.log`.

Existing rooms, scripts and audio are preserved. This review does not publish a
new content pack.
