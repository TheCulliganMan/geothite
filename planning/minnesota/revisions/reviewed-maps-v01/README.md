# Reproducible reviewed map continuation


## Current twenty-two-stage recipe

[`recipe-v11.json`](recipe-v11.json), SHA256
`5563e695bcada8127e41e4815cdb923e1029b7738b5f4cddad8aeb0789ffbd4b`,
adds the [Pine Country fork](../../routes/r06-pine-fork-v01/README.md) after R05.
The executed master produces the selected local
`content-packs/minnesota-reviewed-maps-v11.crystalpack`, SHA256
`d3c16929aade3df2e4b26406dd1bd1c0ac61f05b05be5f569c41e1657c3899d9`,
byte-identical to the standalone child, and reproduces v10 exactly.

The native two-lane drop has closed pine flanks, a dry downhill trip and two
six-grass uphill choices. The original northern hop, Cut-gated Ether/stair,
cabins, Super Rod giver, bridge and pier remain intact. A continuous native
lookout rim replaces the old boulder corner. The 39-cell delta preserves all
742 other maps and every existing actor/program, fauna/fishing table, room,
attribute, source pixel and audio program.

[`sources-v11.json`](sources-v11.json), SHA256
`3092d9794519445623d1d8e81c38977233ddba179195419c8b71d2f198a1eaad`,
binds 38 complete native layouts, ten route groups, 27 sources, the retained
26 town anchors, prior garden reward and both new fork lanes. The default auditor
resolves v11; earlier packs require their explicit retained indices. Seven
native courses, 938 actual stdio calls, ten exact full native frames, four
played views and the updated 108-map atlas pass. Eleven publisher and seven
source-index refusals reject; v10 source compatibility passes. Both affected
crates pass all-target Clippy and all 602 audio hash/frame/loop checks pass.
Owned formatting passes; workspace fmt retains the prior offset-allocator issue.
Selection is bound in `renders/selected-r06-pine-fork-v1485/receipt.json`.

The executed build used `cargo build --locked -p crystal-assets --bin
rebuild_minnesota_reviewed_maps` under the shared Cargo lock. To reproduce into
a fresh ignored output after acquiring `.local/minnesota-cargo.slot.lock`:

```sh
.local/r06-reviewed-master-build-20261101T043507/publisher \
  "$PWD" \
  content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack \
  planning/minnesota/revisions/reviewed-maps-v01/recipe-v11.json \
  content-packs/minnesota-reviewed-maps-v11-repro.crystalpack
```

The publisher refuses existing destinations. Current v11 and the exact v10
compatibility result are already retained. No pack or render is tracked.

## Retained twenty-one-stage recipe

[`recipe-v10.json`](recipe-v10.json), SHA256
`5c8c1c0da2750306133b917b8f6601f522b68b59419a9c12bfe366f20453bcef`,
adds the [River Bluffs wooded terraces](../../routes/r05-wooded-terraces-v01/README.md)
after the R02 garden. The executed master produced the preceding local
`content-packs/minnesota-reviewed-maps-v10.crystalpack`, SHA256
`97f3cba339e7b6e6ed8736f191b90a9cd03a10f6ff6cd5756c2cfb754469c591`,
byte-identical to the standalone v02 child, and reproduces v09 exactly.

The full route retains its five outdoor stair flights, two cave mouths,
original trainers, river/fair/city links and fishing. Winding paths, pine/grass
pockets and a four-step grass approach to the existing Great Ball replace bare
terrace shoulders. Continuous native cliff faces replace boulder-built rims;
four interior boulders have clear setbacks. The 135-cell delta leaves all
741 other maps and existing actors, programs, fauna, attributes, art and audio
exact. Eight native courses, 929 terrace stdio calls plus the separate cave
stdio journey, ten exact native frames and three actual played views pass.

[`sources-v10.json`](sources-v10.json), SHA256
`d49df6ab6cee06916ec6c4c2193a426a08fe79f85eafe12ee460de3b0c54ea53`,
binds 36 complete native layouts, nine route groups, 26 sources, 26 retained
town anchors and the prior garden actor/reward. The default source auditor
resolves v10; use an explicit earlier index for retained packs. Eleven publisher
and six source-index refusals reject; v09 source compatibility passes. Both
affected crates pass all-target Clippy. All 602 audio hashes/frame counts/loop
ranges pass. Owned formatting passes; workspace fmt retains unrelated
`offset-allocator` differences. Selection evidence is
`renders/selected-r05-wooded-terraces-v1464/receipt.json`.

The executed build used `cargo build --locked -p crystal-assets --bin
rebuild_minnesota_reviewed_maps`, with `RUST_MIN_STACK=67108864` and
`CARGO_TARGET_DIR=target/verdigrim-scene`. To reproduce into a fresh ignored
output after acquiring `.local/minnesota-cargo.slot.lock`:

```sh
.local/r05-reviewed-master-build-20261101T035707/publisher \
  "$PWD" \
  content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack \
  planning/minnesota/revisions/reviewed-maps-v01/recipe-v10.json \
  content-packs/minnesota-reviewed-maps-v10-repro.crystalpack
```

The publisher refuses existing destinations. The selected v10 and byte-identical
v09 compatibility output are already retained; no content pack is tracked.

## Retained twenty-stage recipe

[`recipe-v09.json`](recipe-v09.json), SHA256
`e1d8b228d9e4850b2ae009f0f7140fcc5ea669fb179c3c9b2b46c9eb49911b1e`,
adds the [R02 working garden](../../routes/r02-working-farmyard-v01/README.md)
after the R16 seed bank. The executed Rust rebuild produced the preceding local
`content-packs/minnesota-reviewed-maps-v09.crystalpack`, SHA256
`60d9f65efb97ae2363bf7885fff9fd860ab16614c38635d1c05ed53b4de63583`,
byte-identical to the standalone child. Golden crops and working stakes replace
an empty pasture shoulder; one native grower gives a transactional, once-only
Berry. Both roads, original bridge/bank/rooms/programs, regional fauna, fishing
and all742other maps remain exact. The master also reproduces v08 bytes exactly.

[`sources-v09.json`](sources-v09.json), SHA256
`085cab8f04c330536b8053ef5f25b5e09cfe7d3b992f64556de732b8a5e099de`,
binds33native layouts, eight route groups,25sources,26retained town anchors
and the new route grower/reward. The default auditor now resolves v09; earlier
indices remain supported explicitly. Ten current publisher refusals and six
corrupted-index cases reject. All602audio checks and affected assets/Bevy
all-target Clippy pass. Owned formatting passes; workspace formatting retains
unrelated offset-allocator differences.

Five native cases cover held-offer Continue/input lock, gift/10HP item use,
repeat, full-pocket Toss/retry, both working lanes, new-crop wild Run, original
bridge/bank/rooms/resources and both lanes of both real town seams in both
directions. Four actual stdio processes pass496calls with empty stderr. Six
complete native frames match source exactly; the whole route, three actual
played night/rain saves and108-map connected atlas were inspected. Both early
zero-badge fishing courses pass again. The
[selection receipt](../../renders/selected-r02-working-farmyard-v1438/receipt.json)
binds exact bytes/evidence and the prior world. Local fixtures do not establish
earned regional access or intended-level balance; the wider map goal remains open.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin rebuild_minnesota_reviewed_maps
target/verdigrim-scene/debug/rebuild_minnesota_reviewed_maps \
  "$PWD" \
  content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack \
  planning/minnesota/revisions/reviewed-maps-v01/recipe-v09.json \
  content-packs/minnesota-reviewed-maps-v09-FRESH.crystalpack
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/audit_reviewed_map_sources.py \
  --pack content-packs/minnesota-reviewed-maps-v09.crystalpack \
  --output planning/minnesota/renders/reviewed-v09-sources-FRESH
```


## Retained nineteen-stage recipe

[`recipe-v08.json`](recipe-v08.json), SHA256
`c2d2115413ebebb74e6b45d146fe4575267ec97b0342a6e7af001b8b68fbb0d4`,
adds the [R16 seed bank](../../routes/r16-seed-bank-v01/README.md) after Brainerd.
The executed Rust rebuild produced the prior local selection
`content-packs/minnesota-reviewed-maps-v08.crystalpack`, SHA256
`38366f43092e040d40e1084ac4437cef967d812029d4e0c9e04d160b06139df7`,
byte-identical to the standalone child. Fifteen western prairie metatiles add
a short native downhill hop beside the existing grower/flowers and two grassy
uphill gates. Both rivers/bridges, the original hayfield drop, stories, regional
fauna, early fishing and all742other complete maps remain exact.

[`sources-v08.json`](sources-v08.json), SHA256
`d9336e32b95db0553e1b92a95da855368f9cc56eef72580be2a582e6f0731c04`,
binds32native layouts, seven route groups,24sources and26town anchors.
Both R16 chunks have complete current source owners, including the unchanged
eastern chunk. The auditor at that epoch defaulted to v08; v07 compatibility still passes.
The new master rebuilds the previous v07 bytes exactly. Nine current publisher
refusals and six corrupted-index cases reject; all602audio programs and affected
assets/Bevy all-target Clippy pass. Owned formatting passes; workspace formatting
retains unrelated offset-allocator differences. The initial audio attempt used
an obsolete verifier; the freshly rebuilt current verifier passes.

Three native controller cases pass both new hop lanes, reverse/side refusal,
ordinary grassy returns, grower dialogue, both bridges/hayfield lanes, the chunk
seam, actual wild Run and Continue. Two stdio processes pass186calls with empty
stderr. Six complete640×576native frames match the source exactly; the whole
route, connected108-map atlas and two actual played night/rain views were
inspected. Both early fishing courses pass on identical bytes. The
[selection receipt](../../renders/selected-r16-seed-bank-v1410/receipt.json)
binds exact evidence, fixture limits and the prior world. Full regional quality
and earned progression remain open. No deployment or save migration is claimed.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin rebuild_minnesota_reviewed_maps
target/verdigrim-scene/debug/rebuild_minnesota_reviewed_maps \
  "$PWD" \
  content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack \
  planning/minnesota/revisions/reviewed-maps-v01/recipe-v08.json \
  content-packs/minnesota-reviewed-maps-v08-FRESH.crystalpack
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/audit_reviewed_map_sources.py \
  --pack content-packs/minnesota-reviewed-maps-v08.crystalpack \
  --index planning/minnesota/revisions/reviewed-maps-v01/sources-v08.json \
  --output planning/minnesota/renders/reviewed-v08-sources-FRESH
```


## Retained eighteen-stage recipe

[`recipe-v07.json`](recipe-v07.json), SHA256
`cedc824090bfd3851a2315cca0e38f74a765593ba689f77393f14878f46b0664`,
adds [Brainerd's lakeside loop](../../towns/brainerd-lakeside-loop-v01/README.md)
after Mankato. The executed rebuild produces
`content-packs/minnesota-reviewed-maps-v07.crystalpack`, SHA256
`d94e6ccfa6460f58730fff0e5b23a4167a050cc0709214625af9199718e70a7f`,
byte-identical to the standalone child and selected before v08. The existing
outfitter joins the lake street, both through-road lanes are clear, and the
pier has its proper left/right cap corners. Original Gym cliffs/stairs,
programs, rewards, lake and all 742 other maps remain exact.

[`sources-v07.json`](sources-v07.json), SHA256
`56ad4e5453f1da48296711b3142c4b23412cb42ce83732af617127e8ec577339`,
binds 30 native layouts, six route groups, 23 sources and 26 town anchors
(Rochester8, Grand Rapids9, Mankato7, Brainerd2). The auditor at that epoch defaulted to v07;
all earlier recipes/indices remain supported. Final source-owned rows/anchors
take precedence over historical creation dossiers. Resolve `world.json` before
editing or rendering the selected region.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin rebuild_minnesota_reviewed_maps
target/verdigrim-scene/debug/rebuild_minnesota_reviewed_maps \
  "$PWD" \
  content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack \
  planning/minnesota/revisions/reviewed-maps-v01/recipe-v07.json \
  content-packs/minnesota-reviewed-maps-v07-copy.crystalpack
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/audit_reviewed_map_sources.py \
  --pack content-packs/minnesota-reviewed-maps-v07.crystalpack \
  --index planning/minnesota/revisions/reviewed-maps-v01/sources-v07.json \
  --output planning/minnesota/renders/reviewed-map-sources-FRESH
```

Use fresh ignored outputs and serialize Cargo/publication. Build, exact v07
reproduction, byte-identical v06 compatibility and affected assets/Bevy
all-target Clippy pass. Thirteen unsafe publisher cases (seven standalone,
six master) and eight corrupted-index cases reject. The previous v06 source
audit still passes. All 602 audio/desktop checks and owned Rust formatting pass;
workspace formatting retains its pre-existing offset-allocator differences.

Four current Brainerd controller cases pass five doors/ten exit carpets/seven
posts, both street lanes, rail refusal, both pier columns and actual dry-bank
fishing, native HP/status/PP care, once-only cabin supply gift/repeat and
original Gym cliff/stairs/Continue. Three actual stdio processes pass 845 calls
with empty stderr. Eight complete clear-day native frames match their source
exactly; four actual night/rain saved views were inspected separately. The whole
town and 108-map atlas were inspected. The [selection receipt](../../renders/selected-brainerd-lakeside-loop-v1394/receipt.json)
binds exact successful proof and the preceding world. Early zero-badge rod gift/
casting/encounter/Run/Continue and all guru gifts/repeats pass on these bytes.
The broader quality and earned regional progression goal remains open; this
local selection does not deploy, migrate saves or complete a campaign.

## Retained seventeen-stage recipe

[`recipe-v06.json`](recipe-v06.json), SHA256
`c438671a06dc8625e9603bdc238be7cc4963c2d06e7b9a4bee968dc976822c41`,
adds the [Mankato terrace/pond garden](../../towns/mankato-terrace-pond-v01/README.md)
after Grand Rapids. The executed Rust rebuild produces
`content-packs/minnesota-reviewed-maps-v06.crystalpack`, SHA256
`f4cd87d34fba9696d67c9658052976b9f7ba2d9689371603fa358685b5592547`,
byte-identical to the standalone child. This was selected before Brainerd.
Complete native facades, the existing Gym/ledge course and notebook remain;
services share a tighter street loop, with two open dry casting banks beside
a contained pond. All742 other complete maps and existing resources are exact.

[`sources-v06.json`](sources-v06.json), SHA256
`6689359951c55544d148c8fb06e1d10e8b6c69111a431796fe6714eee1c4d9ff`,
binds29 native layouts, six route groups,22 sources and24 town anchors
(Rochester8, Grand Rapids9, Mankato7). The auditor at that epoch defaulted to v06; earlier
indices and recipes remain supported explicitly. Frozen source status strings
identify authoring epochs; resolve `world.json` for actual local selection.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin rebuild_minnesota_reviewed_maps
target/verdigrim-scene/debug/rebuild_minnesota_reviewed_maps \
  "$PWD" \
  content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack \
  planning/minnesota/revisions/reviewed-maps-v01/recipe-v06.json \
  content-packs/minnesota-reviewed-maps-v06-copy.crystalpack
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/audit_reviewed_map_sources.py \
  --pack content-packs/minnesota-reviewed-maps-v06.crystalpack \
  --index planning/minnesota/revisions/reviewed-maps-v01/sources-v06.json \
  --output planning/minnesota/renders/reviewed-map-sources-FRESH
```

Use fresh ignored outputs and serialize Cargo/publication. Build/reproduction
and affected-crate all-target Clippy pass. The new master reproduces v05 exactly
in `.local/mankato-v05-compatibility-1793516333606132000/`. Thirteen publisher
refusals and ten corrupted-index refusals pass. All602 audio checks and scoped
formatting pass; workspace formatting retains its earlier offset-allocator differences.
Six current controller cases cover doors/carpets/posts, services, notebook,
original Gym/ledge play, both ordinary rod banks and all52 native Surf tiles.
Three actual stdio processes pass403 calls with empty stderr. Five saved native
views and five separate clear-day fixtures were inspected; the latter match
complete source frames with zero differing pixels. The whole town and108-map
atlas were inspected. The [selection receipt](../../renders/selected-mankato-terrace-pond-v1380/receipt.json)
binds exact proofs and the preceding world. Native Old Rod gift/casting/Run/
Continue and all guru gifts/repeats pass on v06, with ten Winona and six R01 dry
banks still reachable. Broader earned progression, visual approval, browser
publication and save migration remain separate.

## Retained sixteen-stage recipe

[`recipe-v05.json`](recipe-v05.json), SHA256
`a2aba4c2ea8fa36ad57299d8bff1a872ac22572332c95ca93a4ba6ad7887082e`,
adds the [Grand Rapids depot/quay revision](../../towns/grand-rapids-depot-quay-v01/README.md)
after Rochester. The executed 29.90-second rebuild produces
`content-packs/minnesota-reviewed-maps-v05.crystalpack`, SHA256
`79e431d36fd47c02904b8c9a92d7871a4d9cead4c3e557ed6a73c98d27837e30`,
byte-identical to the standalone Grand Rapids child. This was the local
review selection before Mankato. The town brings original services onto a tighter inhabited
street/quay loop, retaining complete original facades, grass, water and shore.
All 742 other maps and existing content remain exact.

[`sources-v05.json`](sources-v05.json), SHA256
`5819598c010e9a1b39d728e7834f5a37a851d32f1f86c53c8a70c395cbf3c4ec`,
binds 28 native layouts, six route groups, 21 sources, eight Rochester anchors
and nine Grand Rapids anchors. The auditor at that epoch defaulted to v05 and retains explicit
older-index support. All inherited catalogs and the green Safari configuration
remain unchanged. The frozen proposal/status strings identify authoring epochs;
`world.json` resolves the actual local selection.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin rebuild_minnesota_reviewed_maps
target/verdigrim-scene/debug/rebuild_minnesota_reviewed_maps \
  "$PWD" \
  content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack \
  planning/minnesota/revisions/reviewed-maps-v01/recipe-v05.json \
  content-packs/minnesota-reviewed-maps-v05-copy.crystalpack
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/audit_reviewed_map_sources.py \
  --pack content-packs/minnesota-reviewed-maps-v05.crystalpack \
  --index planning/minnesota/revisions/reviewed-maps-v01/sources-v05.json \
  --output planning/minnesota/renders/reviewed-map-sources-FRESH
```

Use fresh ignored outputs and serialize Cargo/publication. The five build,
reproduction and assets/Bevy all-target Clippy checks pass in
`.local/grand-rapids-reviewed-build-1793511504353838000/`. A v04 compatibility
rebuild remains byte-identical in
`.local/grand-rapids-master-v04-compatibility-1793511730695828000/`.
Twelve publisher refusals and nine corrupted-index refusals pass in
`.local/grand-rapids-publisher-guards-1793511729540912000/` and
`.local/grand-rapids-source-index-guards-1793511608052378000/`.
Scoped Rust formatting passes; the workspace check retains its earlier offset
allocator differences. All 602 audio checks bind the identical standalone child.

Twelve fresh controller cases pass on v05 in
`renders/grand-rapids-reviewed-controller-1793511703430559000/`.
Two stdio sessions pass 269 calls with byte-empty stderr in
`renders/grand-rapids-depot-quay-stdio-v1365/`. The installed whole town and five
actual saved native views were inspected. The new whole-state atlas contains
108 outdoors, with no overlapping footprints, inconsistent cycles or missing
hubs. The [selection receipt](../../renders/selected-grand-rapids-depot-quay-v1366/receipt.json)
preserves the preceding world and binds the exact local proof. These local
fixtures do not prove an uninterrupted earned regional campaign, existing-save
migration, browser publication or final user visual approval.

## Retained fifteen-stage recipe

[`recipe-v04.json`](recipe-v04.json), SHA256
`34ad209b29faac1a5806f42ec9a0b6f105ac20e978a92350785f5ab37d146a00`,
adds the [tighter Rochester care green](../../towns/rochester-care-green-v01/README.md)
after the fourteen regional stages. The executed output is
`content-packs/minnesota-reviewed-maps-v04.crystalpack`, SHA256
`47d45a3bb934e4ad863d3e92950e29dd55bdceb62d5824ff23952d55e11e7b48`.
It is byte-identical to the standalone Rochester child and was the preceding
local review selection in `world.json`. All 742 other maps, existing programs, rooms,
fauna, art and audio remain exact.

[`sources-v04.json`](sources-v04.json), SHA256
`b5d180d890179a4e676ab7668674f6b9e07f49adfa8047abdd64ffa28e9c2df5`,
binds 28 final native layouts, six route groups, twenty source files and eight
relocated Rochester anchors. All inherited catalogs and the optional green
Safari configuration remain bound. The auditor at that epoch defaulted to v04; all older
indices remain supported explicitly. Safari ownership no longer assumes it is
the final stage.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin rebuild_minnesota_reviewed_maps
target/verdigrim-scene/debug/rebuild_minnesota_reviewed_maps \
  "$PWD" \
  content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack \
  planning/minnesota/revisions/reviewed-maps-v01/recipe-v04.json \
  content-packs/minnesota-reviewed-maps-v04-copy.crystalpack
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/audit_reviewed_map_sources.py \
  --pack content-packs/minnesota-reviewed-maps-v04.crystalpack \
  --index planning/minnesota/revisions/reviewed-maps-v01/sources-v04.json \
  --output planning/minnesota/renders/reviewed-map-sources-FRESH
```

The selected v04 already exists. The copy/audit names above must be fresh ignored
paths; existing outputs are refused. The complete fifteen-stage rebuild takes
29.70 seconds. The second standalone run, fourteen-stage compatibility, strict
readback, all 602 audio/desktop checks and assets all-target Clippy pass in
`.local/rochester-reviewed-build-1793507072509902000/`. Eleven publisher refusal
cases and nine corrupted-index cases pass in
`.local/rochester-publisher-guards-1793507273142234000/` and
`.local/rochester-source-index-guards-1793507334819690000/`.

The town runbook records nine successful current controller cases, earned first
delivery/first badge, two real stdio sessions (285 calls, empty stderr), five
inspected native cameras and the complete source view. Failed test-harness
assumptions remain retained and are excluded from the passing count. The
[selection receipt](../../renders/selected-rochester-care-green-v1356/receipt.json)
binds the proof files and preserves the prior world. The refreshed regional
atlas has all 108 outdoors with unchanged real offsets and no footprint/cycle
conflicts. Full campaign and broader visual quality work continue.

The sections below retain each earlier authoring epoch and its evidence.
Their frozen recipe/index status strings are historical; `world.json` resolves
the current selection.

## Retained fourteen-stage recipe

[`recipe-v03.json`](recipe-v03.json), SHA256
`6a371d56959c7b76c578d10e8c0d809c7f4c94c3c4be75776655979b15cbea6e`,
adds the [verified optional Boundary Waters expedition](../boundary-safari-runtime-v01/CURRENT-INTEGRATION.md)
to the thirteen map stages. The executed output is
`content-packs/minnesota-reviewed-maps-v03.crystalpack`, SHA256
`f79f8a3226ef2b4d97d25b546871359797f006265beaa5dcded344024ec5e89d`.
It matches `minnesota-reviewed-safari-v01.crystalpack` byte for byte. Admission
and capture battles are configured in this candidate; the free story junction
and green preserve remain intact. This was the preceding local review selection;
the [combined regional review](REGIONAL-REVIEW.md) records its exact coverage
and remaining opening/story and visual-quality audit.

[`sources-v03.json`](sources-v03.json), SHA256
`771f1fed0109b9ddacff5c15c48a0c5d55f0e7bdfc726e1e39e233c7d29e77dc`,
binds the 27 changed terrain layouts, six route groups, nineteen sources and the
actual Safari configuration/public-guard policy. It also carries every other
parent source catalog, including all 30 route dossiers, so existing source
consumers retain their bindings. The auditor checks those inherited catalogs
and refuses a missing Safari owner. Older recipes/indices remain reproducible.
Their frozen status strings record the preselection authoring epoch;
`world.json` is authoritative for the current local selection.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin rebuild_minnesota_reviewed_maps
target/verdigrim-scene/debug/rebuild_minnesota_reviewed_maps \
  "$PWD" \
  content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack \
  planning/minnesota/revisions/reviewed-maps-v01/recipe-v03.json \
  content-packs/minnesota-reviewed-maps-v03.crystalpack
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/audit_reviewed_map_sources.py \
  --pack content-packs/minnesota-reviewed-maps-v03.crystalpack \
  --index planning/minnesota/revisions/reviewed-maps-v01/sources-v03.json \
  --output planning/minnesota/renders/reviewed-map-sources-v1320
```

These exact outputs exist; choose fresh ignored names for another run. Build,
all-target assets Clippy, the complete fourteen-stage composition and original
twelve-stage compatibility pass in `.local/reviewed-safari-build-v1311/`.
Seven CLI refusals and eight corrupted-index refusals pass in
`.local/reviewed-maps-guards-v1321/` and
`.local/reviewed-source-index-guards-v1321/`. The source/installed audit is
`renders/reviewed-map-sources-v1320/`.

The Safari runbook records 26 current native tests, actual battle/field views,
two stdio sessions, complete preservation and all 602 audio checks. Its few
test-harness corrections retain the repaired cliffs and current ecology;
terrain and encounter rates were not weakened. A further 33 city, train, fair,
river, renderer-policy and weather tests pass on the same pack. These local
fixtures do not claim an earned campaign or cross-pack save migration.

The source preview tools accept this selected descendant only after checking
every actual pack hash, frozen build recipe and declared parent path/hash back
to the coastal ancestor. They still render their explicit source stage. The installed native
atlas is the complete current selection; earlier stage previews remain history.

## Retained thirteen-stage recipe

[`recipe-v02.json`](recipe-v02.json), SHA256
`da0b34a501451333e655e5f5c3b17beed0c183e121fce094641036acb0d86316`,
extends the twelve stages below with the
[upper Haulage boulder clearance](../../caves/haulage-rock-clearance-v01/README.md).
The executed combined output is `content-packs/minnesota-reviewed-maps-v02.crystalpack`,
SHA256 `110e658e66449fd6ca3dc52fb7de5fa3f0f2bd827d8103aa838a32d63193229c`.
It is byte-identical to the separately verified Haulage child. The command still
starts from the selected coastal pack and needs no intermediate packs.

[`sources-v02.json`](sources-v02.json), SHA256
`f6f22c312699dc2e03123ea3a2aa2869237771bd0c5bae0f40f42b72891b860f`,
binds 27 final native layouts, six route groups and eighteen source files.
The auditor now defaults to the fifteen-stage index above. Both older indices
remain supported when supplied explicitly; the twelve-stage output remains
byte-identical.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin rebuild_minnesota_reviewed_maps
target/verdigrim-scene/debug/rebuild_minnesota_reviewed_maps \
  "$PWD" \
  content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack \
  planning/minnesota/revisions/reviewed-maps-v01/recipe-v02.json \
  content-packs/minnesota-reviewed-maps-v02.crystalpack
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/audit_reviewed_map_sources.py \
  --pack content-packs/minnesota-reviewed-maps-v02.crystalpack \
  --index planning/minnesota/revisions/reviewed-maps-v01/sources-v02.json \
  --output planning/minnesota/renders/reviewed-map-sources-v1302
```

These outputs exist; use fresh ignored names. Build, all-target assets Clippy
and both full compositions pass in `.local/reviewed-haulage-build-v1298/`.
Seven updated CLI refusals and six corrupted-index cases pass in
`.local/reviewed-maps-guards-v1303/` and
`.local/reviewed-source-index-guards-v1303/`. Both installed/source audits pass
in `renders/reviewed-map-sources-v1302/` and
`renders/reviewed-map-sources-v1-compatible-v1302/`.

The current candidate remains unselected. It preserves the winter/Ely/Inner
Basin parent, all 742 other maps, early fishing and warm Boundary Waters/
Superior exclusions. The Haulage runbook records actual network/battle/shaft/
Continue, native view and stdio checks. Broader exterior/city quality, combined
regional progression and final verified selection remain open.

## Retained twelve-stage recipe

One Rust command now rebuilds the complete reviewed winter/Ely/Inner Basin
continuation from the selected coastal passing-width pack. It runs the existing
twelve source-bound builders in order, preserving their complete parent,
collision, art and residual guards. It writes only the final ignored pack.
Intermediate packs are no longer required as inputs.

The executed result is `content-packs/minnesota-reviewed-maps-v01.crystalpack`,
SHA256 `83a63e4791f5de5bd72b8b874426b693bf03e3b2067665bfd1a4f7182aba58ba`.
It is **byte-identical** to `minnesota-inner-basin-bedrock-v02.crystalpack`.
This is a reproducible unselected candidate, not a new terrain revision or a
complete earned regional playthrough. `world.json` still selects the coastal
passing-width revision. No client/browser release, save migration or selection
was performed.

## Build inputs and authoritative sources

[`recipe.json`](recipe.json), SHA256
`e00da9d5f66546314ad9cb203dd72fb0ce2dbff9a6ea6494a4cf1a4006a46ed7`,
binds the exact coastal parent, its source index, seventeen authoring files and
every stage's expected content identity. The CLI freezes that recipe hash and
reads/hashes the complete source closure before starting any builder. Source
text is retained in memory through composition. Rust owns pack serialization.

| Order | Existing guarded builder |
| --- | --- |
| 1 | Bemidji complete timber roof and closed plateau face |
| 2 | Ten-map inland winter belt |
| 3 | Iron Range cliff/boulder clearance, approved source-v02 |
| 4 | Tamarack/Rainy River thaw |
| 5 | Iron Range–Duluth descent thaw |
| 6 | Descent cliff/boulder clearance |
| 7 | Native snowy/thawing grass Cut, approved source-v02 |
| 8 | Bemidji–Leech Headwaters snowline |
| 9 | Headwaters stair/boulder clearance |
| 10 | Bemidji–Itasca snowline, approved source-v02 |
| 11 | Ely pine-town refinement against the winter parent |
| 12 | Inner Basin fitted bedrock, approved source-v03 |

[`sources.json`](sources.json), SHA256
`09e69c2c150393f7256e4ba9963c987f1c83f16e04b9cc13dd1fb8eb0f36021c`,
supersedes the old terrain references for all 26 affected native maps and six
route groups. Each final layout points directly into the appropriate frozen
source using `rows_pointer`; there is no duplicate exported block catalog.
`current_native_layouts` is the complete set of changed native map owners.
`current_layouts` retains unrelated parent bindings; a
`current_native_layout_group` resolves to its listed native owners, and
`source_bound_native_rows` resolves through the source hash and JSON pointer.
All other parent contracts remain inherited from the selected source index.

The read-only [source audit](../../tools/audit_reviewed_map_sources.py) consumes
that index, verifies the latest source ownership in recipe order and compares
all installed rows, aliases, dimensions and declared borders. It rejects missing
owners, historical row pointers, stale route groups and unrelated binding drift.
It does not claim gameplay or visual acceptance.

## Executed commands

From the repository root, with the selected external pack present:

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin rebuild_minnesota_reviewed_maps
target/verdigrim-scene/debug/rebuild_minnesota_reviewed_maps \
  "$PWD" \
  content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack \
  planning/minnesota/revisions/reviewed-maps-v01/recipe.json \
  content-packs/minnesota-reviewed-maps-v01.crystalpack
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/audit_reviewed_map_sources.py \
  --pack content-packs/minnesota-reviewed-maps-v01.crystalpack \
  --index planning/minnesota/revisions/reviewed-maps-v01/sources.json \
  --output planning/minnesota/renders/reviewed-map-sources-v1286
```

These exact outputs now exist; use fresh ignored output names for another run.
The parent is SHA256
`e951fddafee3a0c09a2356d4dd683890ac498e23daeb266d08c5f12d79ec1167`.
The CLI accepts another physical copy of those exact parent bytes. It refuses
a later child or historical parent rather than silently dropping reviewed work.
Serialize Cargo with `.local/minnesota-cargo.slot.lock`; preserve the existing
two-job default and check free space according to `AGENTS.md`.

## Verification and remaining work

`.local/reviewed-maps-build-v1284/receipt.json` records successful compilation,
all-target `crystal-assets` Clippy and the complete 23.84-second rebuild. Every
stage identity matches and strict final readback matches the in-memory result.
All final pack bytes match the separately reviewed candidate. Unrelated existing
Clippy warnings remain; none refer to the new CLI. Scoped formatting passes;
workspace formatting retains unrelated differences.

Seven real CLI refusals pass in `.local/reviewed-maps-guards-v1285/`: wrong
parent, changed recipe, existing output, unignored output, changed final source,
missing final source and changed parent source index. Each refuses before stage 1
and leaves output absent or unchanged. Six corrupted-index cases are rejected in
`.local/reviewed-source-index-guards-v1287/`. The actual26-map source/installed
comparison passes in `renders/reviewed-map-sources-v1286/`. The original failed
compile attempts and rejected draft recipe runs remain separate in v1280–v1283;
the guards caught two initial entries pointing at rejected historical sources.

Because the final bytes are identical, the [Inner Basin runbook](../../caves/inner-basin-bedrock-v01/README.md)
and earlier winter/Ely/coast/river verification still apply. The first-route
[free Old Rod and dry fishing bank](../../connections/early-fishing-v01/README.md)
remain available with zero badges and no field moves. Boundary Waters and the
Superior coast remain green; this recipe adds no route-blocking ice.

The eight ordinary mine camera fixtures in `renders/mine-cameras-v1278/` have
been viewed. The fin, pressure crosscut, both climbs and all five/four Haulage/
Ventilation shaft pairs also pass actual controller courses in
`renders/mine-camera-gameplay-v1277/`. Those are explicit local fixtures. The
upper Haulage boulder setback was subsequently completed in the thirteen-stage
successor above; these older views remain the unchanged preimage evidence.

Broader exterior/city quality, combined earned regional progression, final
native acceptance and verified selection remain open. This command rebuilds
the continuation from the existing selected regional parent; it does not replace
the core Crystal initializer or reconstruct every historical regional stage.
