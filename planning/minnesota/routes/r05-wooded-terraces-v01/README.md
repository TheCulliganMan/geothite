# River Bluffs North: wooded terraces

The complete 32×56 route keeps its two hills, five outdoor stair flights,
Rock Cut, overlook, fishing pier, five trainers and real city/river/fair
connections. The broad bare terraces now have winding two-walking-tile paths,
encounter grass and pine shelter. Repeated boulder tiles that had substituted
for cliff rims now form continuous native faces and corners. Four remaining
interior boulders sit inside the plateaus with clear setbacks.

The verified and selected local revision is
`content-packs/minnesota-r05-wooded-terraces-v02.crystalpack`, SHA256
`97f3cba339e7b6e6ed8736f191b90a9cd03a10f6ff6cd5756c2cfb754469c591`,
content identity
`408b2944618db7e6ba1497e29c6e7f82b9d6f5e93eeb3c65f5995aeaddea5d60`.
The twenty-one-stage reviewed build reproduces these bytes as
`content-packs/minnesota-reviewed-maps-v10.crystalpack` and reproduces v09 exactly.
`world.json` now selects `r05_wooded_terraces_revision`; the exact pre-selection
world and evidence hashes are retained in
`renders/selected-r05-wooded-terraces-v1464/receipt.json`. This is a local map
review selection, with broader campaign work still open.

[Whole route](../../renders/r05-wooded-terraces-source-v1463/after-whole-original.png)
· [whole before/after](../../renders/r05-wooded-terraces-source-v1463/before-after-whole-original.png)
· [ten native cameras](../../renders/r05-wooded-terraces-native-v1457/production-decisions.png)
· [actual played night/rain views](../../renders/r05-wooded-terraces-played-native-v1458/actual-played-cameras.png)
· [connected state](../../renders/r05-wooded-terraces-state-v1459/full-state-native-current.png).

## Authoritative inputs and limits

[`source-v05.json`](source-v05.json), SHA256
`926107619b1366f4535bc4f237e4d53a0c2790161096e4371776941c93241bc8`,
binds every native R05 chunk to the exact v09 parent. It changes 90 metatiles
in `MnR05C00R00` and 45 in `MnR05C00R01`; `MnR05C00R02` remains exact.
The 39-cell `cliff_repairs` ownership list records old/new tile IDs, including
the retained interior rock needed to describe its cluster. Every other change
is confined to the explicit walkable terrace surfaces. No source pixels or
palettes are edited; all shapes use installed native tiles.

Cliff corners use 68–73 with their real collision. Stairs retain the existing
86/87 recipes, palette and approach geometry. Ordinary pines use 05, encounter
grass 03, path 01, short grass 02 and flowers 04. No Cut gate or new ice is added.
The ordinary through route remains available without adding a grass toll;
the Great Ball side discovery requires four grass steps from the upper stair,
including when the existing native downward hop is allowed. Its old shortest
ordinary approach cost two grass steps.

The older `MN_R05.json` dossier contains historical Johto parties and encounter
names. It is not current roster or regeneration authority. This revision keeps
the selected native Minnesota parties, encounters, fishing and all programs
exact; it must never republish the historical fauna. The active source index
resolves the native route as all three current chunks.

## Native coordinates and play contract

All following coordinates are 16px walking tiles. Route y is global; chunk rows
start at 0, 48 and 96. The source's `metatile` fields instead use local 32px cells.

| Feature | Current contract |
| --- | --- |
| Upper winding climb | From the original south stair around (15,41), through the planted terrace and (20,30)/(20,26), to the north stair at (25,13–16). Pine collision divides the shoulders; both path lanes remain available. |
| Middle trail | The west stair at (13,75–78) and north stair at (29,51–54) remain connected by the open trail, grass pockets and existing trainer at (31,65). The other road lane at y64 lets the player pass. |
| Five retained stair flights | x13 y75–78; x29 y51–54; x15 y39–42; x25 y13–16; x37 y25–28. All pass in both directions. The closed fronts near (37,40)/(37,76) are cliff walls, not additional stairs. |
| Closed repaired faces | Down from (25,34) and (23,68), right from (18,71) and (40,32), and up from (33,28) are blocked by actual native cliff collision. No road exits through a side face. |
| Existing Great Ball | `MN_R05_ITEM_02` at (37,19), flag `EVENT_MN_R05_ITEM_02`. Approach from (37,24) through the grass to (37,20), face north and interact. The original program grants one Great Ball, persists through Continue and cannot be collected twice. |
| Upper Rock Cut mouth | Exterior active/passive (30,27)/(30,28); interior mouth/clear arrival (11,5)/(11,4). |
| Lower Rock Cut mouth | Exterior active/passive (16,77)/(16,78), local row1 (16,29)/(16,30); interior mouth/clear arrival (5,17)/(5,16). |
| Cave services/story | Original healer, both acknowledged pages, HP/PP/status restoration, Repel pickup and 100-step native Pack effect remain exact. Cave terrain and story are unchanged. |
| North/south | Real north connection to `MnStCloud`, offset 0; south to `MnMinneapolis`, offset −9. Original guards and progression programs remain exact. |
| River/fair connections | East R00/R02 connections to the corresponding `MnMississippiUpstreamWoods` chunks retain offset −18. The R02 west connection to `MnStateFairgrounds` retains offset −28. |
| Fishing and water | Original contained/connected river boundaries, pier, rails, bank approaches, fishing groups and water lanes remain exact. Early Winona/R01 fishing is preserved separately. |

## Evidence

- `renders/r05-wooded-terraces-preservation-v1451/receipt.json`: the entire child
  equals its parent after restoring only 135 terrain cells, their two encoded
  mirrors, the added manifest marker and derived identity. All 741 other maps,
  actors, events, programs, regional fauna, rooms, art/audio, attributes and
  resources remain exact.
- `renders/r05-wooded-terraces-controller-v1453/`: its first entry passes three
  new courses for winding paths, five stairs, blocking faces, native reward,
  wild encounter/Run and Continue. The later cave invocation selected an older
  source preimage and failed before gameplay; it is not counted.
- `renders/r05-wooded-terraces-cave-fishing-v1456/receipt.json`: the correct
  source is `caves/r05-shelf-drop-v01/after.json`. Three current cave courses
  pass both mouths, stairs/drops, healer input ownership and actual treatment,
  original Repel pickup/Pack effect and Continue. Both early fishing courses
  also pass. That is eight successful native courses across the two receipts.
- `renders/r05-wooded-terraces-stdio-v1455/receipt.json`: two real processes
  complete 929 calls with byte-empty stderr, walking all five stair flights,
  testing repaired faces and continuing the actual reached save.
- `renders/r05-current-mouths-stdio-v1461/receipt.json`: a third real process
  exits/reenters both cave mouths, walks the 18-input floor/stair course and
  exposes the production Start/Pack menu with byte-empty stderr.
- `renders/r05-wooded-terraces-native-v1457/capture-progress.json`: all ten
  complete 640×576 frames match installed native source exactly. Every explicit
  player camera is on unoccupied Floor or Tall Grass. The whole source, weakest
  ordinary stretch, all native cameras and three actual saved night/rain views
  were personally inspected.
- `renders/r05-early-dry-banks-v1460/receipt.json`: ten Winona and six R01 public
  banks are still reachable without water, ice, Cut or occupied actor tiles.
- The 108-map atlas has three direct-seam components, no overlaps or inconsistent
  cycles, 1,796 reciprocal water lanes and the same 110 raw edge mismatches as
  v09. The entire connection report is equal outside pack identity. Prior
  mismatch triage still applies; these counts do not justify fencing banks.
- Affected assets/Bevy all-target Clippy passes, as do all 602 audio PCM hash,
  frame-count and loop-range checks. Owned formatting passes. Workspace fmt
  retains the unrelated offset-allocator formatting differences.
- Six standalone publisher, five master publisher and six source-index refusal
  cases reject while preserving outputs; explicit v09 source compatibility passes. Three in-memory early-bank closure
  probes also reject without modifying the pack.

All controller journeys declare local starting positions, prior trainer flags
and, for wild encounters, one original Minnesota companion with a Smoke Ball.
No position, party, bag, badges, RNG, clock or story state is rewritten after
setup. These prove local mechanics; they do not earn route access or establish
unboosted campaign balance. Existing saves are not migrated. Broader statewide
quality and earned progression remain open; no user visual approval is assumed.

## Reproduce and iterate

The exact publication commands were executed with `RUST_MIN_STACK=67108864`
and `CARGO_TARGET_DIR=target/verdigrim-scene`, serialized under
`.local/minnesota-cargo.slot.lock`. The standalone executable is
`.local/r05-terraces-final-build-20261101T035404/publisher`; the master and its
executed recipe are in `.local/r05-reviewed-master-build-20261101T035707/` and
`revisions/reviewed-maps-v01/recipe-v10.json`. Outputs are ignored and publishers
refuse existing paths, so use a fresh output for another run.

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_r05_wooded_terraces.py \
  --source planning/minnesota/routes/r05-wooded-terraces-v01/source-v05.json \
  --output planning/minnesota/renders/r05-terraces-next-v01
```

The full source render/check loop measured 0.48 seconds after loading the pack.
[`native-cameras-v01.json`](native-cameras-v01.json) contains reviewed walkable
player placements; source `review_views` are landmark crop centers and must not
be converted directly into standing positions.

```sh
CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-reviewed-maps-v10.crystalpack" \
CRYSTAL_R05_TERRACE_REVIEW_DIR="$PWD/planning/minnesota/renders/r05-next-controller/native-saves" \
RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/verdigrim-scene \
  cargo test --locked -p crystal-bevy --lib --features location-tester \
  wooded_terraces_controller_ -- --ignored --nocapture --test-threads=1
```

Every prior source, standalone candidate, whole render, raw native frame,
conversion, rejected camera and failed harness receipt remains retained. In
particular v1446's trainer-overlap camera and v1452's two landmark-centered
standing fixtures are excluded from accepted visual evidence; v1457 corrects
all three. The v1447 harness mistook a closed cliff for a stair. Neither failure
was fixed by making the map walk through a wall or by relaxing a pixel check.
