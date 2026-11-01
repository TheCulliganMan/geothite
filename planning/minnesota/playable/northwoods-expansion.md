# Expanded North Shore and Boundary Waters

The hand-authored expansions are installed in the ignored local
`content-packs/minnesota-northwoods-expanded-review.crystalpack` (505 maps).
File SHA256: `3949ad8dff81537f9b664f914f5358825da40512fbfdae52c2fffa1a049441b7`.
Content identity: `32a863fcf8ed8e778146a214ccfa84416bfd52e6b99f38cff628e5695be73c12`.
The original498-map pack and every rejected source/render pass remain retained.
This is a review build; final user approval and earned regional travel remain open.

## Geometry and play contracts

| Place | Installed extent | Useful changes |
|---|---|---|
| North Shore |56×96 metatiles; eight clipped native maps| Lake Superior dominates the coast; narrow tributaries and five river catchments join it. Additional falls, offset plunge pools and viewpoints accompany winding approaches. Three short ledge returns, mandatory grass crossings and exact native cave12/30 outdoor stairs shape traversal. Lighthouse room, original parties, Q07 conditions and old pickups remain. Two named ridge caches and three signs are added. |
| Boundary Waters preserve |64×56; six clipped native maps| Connected lakes, pine shores, four campsites, six grassy portages, five short cliff returns and five cave12 outdoor stairs. Staggered Northwind tiers and the bent Eagle Point saddle replace repeated cliff courtyards. Dorothy and her cabin move with their actual gift/shop/rest and paired returns. |

Current authoring is in `routes/MN_R11.json/.layout.json` and
`zones/boundary-waters-safari.json/.layout.json`. Before/after source epochs are
retained under `revisions/northwoods-expansion/`. Source rows/coordinates are
explicit, with no random geography. Fast preview loops take approximately4s;
new art uses existing native pixels and collision quadrants.

North Shore keeps group41 IDs1–3 and lighthouse4; appended chunks use5–9.
Safari keeps group61 IDs1–4 and cabin5; appended southern chunks use6/7.
The cabin exterior is global DOOR1 `(71,55)` / passive FLOOR2 `(71,56)` in
C01R01; both cabin carpets return to that passive record. Dorothy stays outside
at `(69,59)`. Entry arrival is `(15,105)`, with blocking guards `(11,101)` and
`(12,101)`. Safari admission remains closed: there is no fee, active expedition,
ball/step budget or Safari capture implementation. Free basecamp/story terrain
and its three native walking seams remain separate.

The Duluth planned south port moves to R11 C00R03, local metatile `(23,22)`,
landing `(23,20)`. This source reservation does not install a Duluth walking
connection. Regional hub links remain dashed in the [connected atlas](../ATLAS.md).

## Executed evidence

All ten expansion asset tests pass across the final component runs (five each),
plus two native-grass ledge validator regressions. The actual written505 pack
passes [1,062 bounded preservation checks](../renders/northwoods-native/proof/README.md).
All490 unrelated old modules, audio, original art, catalogs, parties, HOME and
recovery records remain exact. Seven new numeric identities, two new pickup
flags and ten bounded destination-art files are the declared additions.
The proof explicitly records two unreachable local placeholder dialogue labels
pruned by canonical reassembly; both original global labels remain exact and
all actual service programs/menu/runtime indexes are preserved.

Native/source comparisons pass for all5,376 North Shore and3,584 preserve blocks
and pixels, actors, room pairings, stable IDs and reciprocal connections. Boundary
checks include all seven water seams,476 closed perimeter tiles and42 closed
forecourt cells. Fourteen actual headless production captures were inspected by
Root and the author, including the weakest road/clearing frames:
[production contact sheet](../renders/northwoods-native/3949ad8dff81537f/production-1793064804698539000/production-decision-cameras.png).
The original640×576 readbacks, raw GPU bytes,160×144 crops and every conversion
remain retained. Actual measured CHRIS pose uses LCD `(80,72)`; the earlier
planning overlay's four-pixel lift is not imposed on native captures.

Fresh Rust synthesis verifies all377 audio PCM hashes, frame counts and loop
ranges (`.local/northwoods-505-audio.log`). The promoted source plan passes:
20 routes,19 hubs,57 regional chunks and86 planned adjacency/gate contracts.
Symbolic progression is separate from production input.

All eight expansion production-controller cases now pass across the corrected
six-case run and two targeted geometry retries. They prove actual hospitality,
closed admission, both cabin returns, six/eight-chunk Surf, waterfall ascents,
all stairs, real ledge jumps/reverse rejection/ordinary returns, grassy portages,
walking seams and Continue. The first two failed geometry fixtures assumed the
wrong stair lane or attempted ordinary BFS onto HOP; corrections use exact
source lanes and production input from the upper approach. No terrain or engine
behavior changed for those fixture errors. Logs:
`.local/northwoods-505-controller-first.log`,
`.local/northwoods-r11-geometry-corrected.log` and
`.local/northwoods-safari-geometry-corrected.log`. All party/badge/position/RNG
fixtures are explicit and do not establish earned regional progression.

R11's broad shore fork remains the weakest composition; publication and matching
pixels do not establish universal Johto quality. Other caves remain to be reviewed.
The separate [R10 gallery candidate](../caves/MnR10Cave1F.review.md) is now in native
verification and does not silently replace this pack's original cave.

## Reproduce publication

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=1 cargo build \
  --profile wasm-dev --locked --config profile.wasm-dev.opt-level=0 \
  --config profile.wasm-dev.package.offset-allocator.opt-level=2 \
  -p crystal-assets --bin expand_minnesota_northwoods

target/runtime-fps/wasm-dev/expand_minnesota_northwoods \
  content-packs/minnesota-cities-review.crystalpack \
  planning/minnesota/revisions/northwoods-expansion/before/MN_R11.layout.json \
  planning/minnesota/revisions/northwoods-expansion/after/MN_R11.layout.json \
  planning/minnesota/revisions/northwoods-expansion/before/MN_R11.json \
  planning/minnesota/revisions/northwoods-expansion/after/MN_R11.json \
  planning/minnesota/revisions/northwoods-expansion/before/boundary-waters-safari.layout.json \
  planning/minnesota/revisions/northwoods-expansion/after/boundary-waters-safari.layout.json \
  planning/minnesota/revisions/northwoods-expansion/after/boundary-waters-safari.json \
  content-packs/minnesota-northwoods-expanded-review.crystalpack
```

See the [implementation contract](../reviews/northwoods-expansion-native-contract.md),
[North Shore handoff](../reviews/north-shore-expanded-handoff.md),
[Boundary comparison](../reviews/boundary-native-comparison.md) and
[actual capture commands](../reviews/northwoods-production-capture.md).
