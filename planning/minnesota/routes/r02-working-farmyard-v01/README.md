# Prairie Farm Road working garden

The western farmhouse now has a working garden: golden crop beds, short native
stake divisions, a two-tile approach and a grower who gives one useful Berry.
The complete30×22route keeps its original two town roads, bank drop, bridge,
fenced drain, farmhouse, tool shed, three trainers, two residents and two caches.
There is no new required detour, badge gate or field move.

## Current authoring source

[`source-v03.json`](source-v03.json), SHA256
`1348ff5a00f57b5b2add40945585f68fdf4c1e4c2cda638e840c4be1cf4cc875`,
binds the exact selected v08 parent and whole native R02 module. Seventeen
metatiles change; native81 supplies golden encounter crops,2c worked soil and77
real wooden stakes. No pixels, palettes, collision definitions or aliases change.
The original map module preimage is
`605b65ff576d0bce7d082932eae900d733664727ea862c5edacea037c1284af4`.

Coordinates are16px walking tiles unless stated otherwise.

| Feature | Native contract |
| --- | --- |
| Grower | `MN_R02_GARDEN_GROWER`, native GRANNY at(9,19), approach(10,19), face left |
| Working approach | Both columns12/13 remain ordinary FLOOR fromy12through19 |
| Stake divisions | Native77 at metatile(5,8)/(7,8); lower walking cells10/11,17 and14/15,17 really block |
| Golden crop beds | Native81 around the northern farmhouse shoulder and lower garden; native regional grass encounters and priority holes |
| Farmhouse | Original door(11,11), passive return(11,12), both inside exit carpets preserved |
| Tool shed | Original door(52,33), passive return(52,34), both inside exit carpets preserved |
| Western road | R02(0,30/31) ↔ Mankato(47,18/19), native west connection offset6 |
| Eastern road | R02(59,14/15) ↔ Rochester(0,18/19), native east connection offset−2 |
| Original bank drop | All four upper48,y14–17, lip47, wall46, landing45 lanes and ordinary returns preserved |
| Main encounter cost | Current native graph remains18 west→east and16 east→west grass entries |

The old exact-seven-grass nursery proposal belongs to a retired pack. Its
prices are not the selected map's current contract. The new source review
measures the actual before/after graph with native HOP_LEFT behavior and actor
bodies; neither route direction gets a new grass toll. Both real room entries,
all existing actor/post approaches and the entire drain/bridge remain intact.
Historical `MN_R02.json` trainer/wild parties include Johto species. They must
not be used to regenerate current Minnesota teams. The selected native parties,
wild/fishing tables and programs remain exact and pass the225-original-species
regional policy audit.

## Grower and resource behavior

Acknowledging the grower's original garden offer gives native BERRY×1. The
claim `EVENT_MN_R02_GARDEN_BERRY` is set only when the native grant succeeds.
A full Items pocket prints the refusal and preserves the claim; native Toss
permits retry. Repeat dialogue and Continue cannot duplicate it. The NPC stays
in the garden after giving the Berry. No money, badge, quest progress or recovery
point changes. No badge, Cut or Surf is required for the gift or its dry approach.

Rust compiles only the seven new program/text labels and appends their typed
metadata. Every original R02 program and command remains exact. Both the native
event wire and NPC table include the new body. One new event ID is appended;
existing event IDs are never renumbered. Existing saves are not migrated.

## Render loop and retained candidates

`review_r02_working_farmyard.py` renders the complete route and six decision
cameras without compilation. The current warm source render/check took0.169s
following pack loading. All originals, cropped views and rejected passes remain.
v01's broad bare-soil apron was replaced by golden crops in v02. v03 adds the
explicit received-gift text and keeps v02 terrain exactly.

- Whole/source decisions: `renders/r02-working-farmyard-source-v1425/`
- Six exact640×576native source-frame matches: `renders/r02-working-farmyard-native-v1427/`
- Three actual played night/rain views: `renders/r02-working-farmyard-played-native-v1431/`
- Whole-root preservation: `renders/r02-working-farmyard-preservation-v1426/receipt.json`
- Full108-map connected source atlas: `renders/r02-working-farmyard-state-v1432/`

The standalone candidate is
`content-packs/minnesota-r02-working-farmyard-v01.crystalpack`, SHA256
`60d9f65efb97ae2363bf7885fff9fd860ab16614c38635d1c05ed53b4de63583`,
content identity`229973e1cc11df7932bde0344ce4776fe4fb67d04f5fafee8f83142447f3a5ec`.
The twenty-stage recipe-v09 reproduces these bytes as
`content-packs/minnesota-reviewed-maps-v09.crystalpack`, now selected locally in
`world.json`. Its source index binds the complete R02 terrain and new native
grower/grant; the previous v08 rebuild is byte-identical. The selection receipt
is `renders/selected-r02-working-farmyard-v1438/receipt.json`.

## Executed native verification

Four controller courses pass in `renders/r02-working-farmyard-controller-v1428/`:
held-offer Continue and input lock, the actual gift, native Pack use healing10HP,
no duplicate after repeat/Continue, full-pocket refusal, native Toss/retry,
both working approach lanes, stake collision, a real encounter in the new crops,
native Run/Continue, all original bank lanes and returns, bridge, original
resident/post/pickup interactions and both rooms. The local original Birchit10
has a declared12HP injury and Smoke Ball; capacity/navigation fixtures are
explicit. They do not establish an earned route arrival or intended-level balance.
No party, bag, position, flag, HP/PP or RNG is rewritten after initial setup.

The fifth course in `renders/r02-working-farmyard-town-seams-v1436/` supplies
only prior Care and an empty party, then walks both actual lanes through both
Rochester and Mankato seams in both directions, settles arrivals and continues.
It preserves the original departure conditions and grants no garden reward.
The corrected final test build and all-target Bevy Clippy pass; assets Clippy
and all602audio checks pass too. Owned Rust formatting passes; workspace
formatting retains the unrelated offset-allocator differences.

Both early zero-badge Old Rod/guru courses pass again in
`renders/r02-working-farmyard-early-fishing-v1434/`. Original Winona and R01 dry
banks, rod rewards and every regional fishing table remain exact.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin revise_r02_working_farmyard
target/verdigrim-scene/debug/revise_r02_working_farmyard \
  content-packs/minnesota-reviewed-maps-v08.crystalpack \
  38366f43092e040d40e1084ac4437cef967d812029d4e0c9e04d160b06139df7 \
  planning/minnesota/routes/r02-working-farmyard-v01/source-v03.json \
  content-packs/minnesota-r02-working-farmyard-FRESH.crystalpack
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_r02_working_farmyard.py \
  --source planning/minnesota/routes/r02-working-farmyard-v01/source-v03.json \
  --output planning/minnesota/renders/r02-working-farmyard-FRESH
```

Serialize Cargo/publication with`.local/minnesota-cargo.slot.lock` and use fresh
ignored outputs. The full regional map-quality goal remains active.

Four actual production stdio processes pass496calls with byte-empty stderr in
`renders/r02-working-farmyard-stdio-v1437/`. They continue the held offer,
check the actual Pack shows exactly one Berry, replay both garden lanes/stakes,
verify repeat/fresh-process persistence, walk both lanes through both town seams
in both directions and check Start/Continue. Earlier v1430 expected unpadded
`x1`; the native display is`x01`. That harness failure and the successful
intermediate two-process v1433 replay are retained separately. No gameplay or
quantity expectation was changed.

```sh
CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-reviewed-maps-v09.crystalpack" \
CRYSTAL_WORKING_FARMYARD_REVIEW_DIR="$PWD/planning/minnesota/renders/r02-native-FRESH" \
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo test --locked -p crystal-bevy --lib working_farmyard_controller_ \
  -- --ignored --nocapture --test-threads=1
```
