# Central Lakes — bounded water-boundary repair

The existing 32×18 route, battle/meadow choice, required grass, useful ledge,
protected bridge/dock, cabin, actors and story conditions are retained. The
current [whole source map](../renders/central-lakes/whole-route-73246547.png)
closes the two previously exposed water exits. This closure is a **source review
candidate**; the older native captures and fixtures below predate it.

The whole old map, native bridge/dock views and player cameras were inspected
before changing terrain. Their composition already supports the compact play
choices, so no broad redesign was warranted. Both new shoreline joins and the
southern outlet were inspected at source player scale:
[northwest join](../renders/central-lakes/boundary-north-west-join-2e85b982.png),
[northeast join](../renders/central-lakes/boundary-north-east-join-e25a7001.png),
[south outlet](../renders/central-lakes/boundary-south-rocks-33a3effa.png).
These terrain detail crops are source pixels, not production Surf captures.

## Exact source recipe and migration

- North cap: existing native Kanto `0f` at meta x=12–19, y=0, closes all actual
  boundary WATER. Below it, Kanto `6b` at x=12–19, y=1 supplies visible water rocks.
  Its collision is `[WATER,WATER,WALL,WALL]`; the lower core row y=3 closes passage
  from the lake below. The small water strip above those rocks is also sealed
  against entry from any route land/Surf tile.
- South outlet: Kanto `6b` at x=15–17, y=17 closes the bottom core row y=35.
  There is no reciprocal outer water neighbor at either end.
- `additional_water_blocks:["6b"]` preserves source bank fitting beside these
  water-rock blocks. `reject_cliff_cuts:true` is now explicit. No new tileset,
  imported art, facade recipe, room, event identity or native controller is added.
- Exactly 19 metatiles change. A read-only comparison against the historical
  native Central Lakes pack finds 54 changed collision quadrants, all outside
  walking/grass/ledge/door collision. Every ordinary walking, encounter grass,
  HOP_DOWN, ladder and door quadrant is unchanged.
- Map dimensions, chunk ID `MnR15C00R00`, both regional port coordinates,
  BADGE_6/SQ02, parties, pickups, warps and interior rows are unchanged. Root only
  needs to republish this bounded terrain change, preserve unrelated assets/audio,
  and run current native boundary captures/Surf denial plus existing regressions.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_central_lakes.py \
  --negative-checks \
  --compare-walking-baseline content-packs/minnesota-central-lakes.crystalpack
```

The source checks pass the existing two piers, directed Ron/meadow alternative,
31-step ordinary ledge return, required six-step grass and all event/room
approaches. The full perimeter has **zero WATER collision**, including both
shore joins; walking+Surf reaches 932 cells and no outer water edge. Three
malformed copies (north cap hole, north rock gap, south rock gap) are rejected.
Warm whole-map/source checks remain about 110ms, without compilation.

Current source hashes:

- Layout: `5cda48486308bcebef1937ee5a6045af45a6240ca76a44f93fdc5bab2321dd85`
- Dossier: `2d3d16ebd1e4488c378b893e84db134ba0e24054cd8d493c0ee329aaf294e9d8`

Native publication, production Surf boundary denial and final user visual
acceptance remain pending for this closure. The historical native evidence below
still documents the unchanged walking mechanics; it is not evidence that the
new water closure has been published or played.

---

# Historical compact route implementation and native evidence

The user rejected the 96×72-metatile route as too large and uninteresting. This replacement is **32×18 metatiles**, one native exterior, with one 5×4-metatile cabin. Area fell from 6,912 to 576 metatiles (92%). Do not expand it or copy its arrangement across the region before visual/play review. The other giant route passes remain unaccepted.

[Whole map](../renders/central-lakes/whole-route-b1c06d98.png) · [player-scale views](../renders/central-lakes/viewpoints.png) · [anchors](../renders/central-lakes/anchors.png) · [native bridge](../renders/central-lakes/native-bridge.png) · [native ledge](../renders/central-lakes/native-ledge.png) · [native dock](../renders/central-lakes/native-dock.png) · [native cabin](../renders/central-lakes/native-cabin.png).

## Play decisions and physical connections

| Location | Behavior and coordinates in 16px gameplay tiles |
| --- | --- |
| West arrival | Detroit Lakes gate `(2,24)`, landing `(4,24)`. A six-step grass crossing at x=6–11 is unavoidable. |
| First fork | Ron stands `(17,24)` in the short lane, looking south. Coming from the west, battle him or take the longer northern grass detour and collect Hyper Potion `(23,9)`. Coming from the east, the ledge blocks reverse entry into that detour and Ron controls the narrow road. The detour returns through a south-facing ledge at `(22–25,18–19)`: step onto HOP_DOWN at y=18, then hop over the wall at y=19 to floor at y=20. Trees west and lake east close the sides. Reverse ascent is blocked; the western path is the uphill entrance. |
| Outlet bridge | Both walking rows y=28–29 cross x=30–35. Stone bollards above and wooden rail below prevent entering water sideways. Removing the deck disconnects the gates. |
| Fishing dock | Enter through the eastern bank opening `(42,14–15)`. Native rails close its upper/lower edges and west end. Revive `(37,15)` rewards visiting the spur. |
| Cabin | Door `(53,11)` enters `MnR15LakeCabin`; both indoor exits target a separate native return-only warp on exterior floor `(53,12)`. The floor event does not trigger entry when walked over. Eli, a wall chart and Max Ether are inside. Niko offers an optional battle in the small forecourt. |
| East arrival | Brainerd gate `(60,24)`, landing `(56,24)`. Neither Surf nor Cut is needed. |

There is one returning detour, two trainers, two residents, two outdoor pickups, three signs and one enterable cabin. Extra reserved trainer/item flags remain unused. `BADGE_6` and the SQ02 prairie-return story remain unchanged; SQ02's NUGGET belongs at Moorhead, not to a route resident. Real hub gates, earned progression, recovery and a complete playthrough remain integration work.

The terrain uses 27 unchanged Kanto metatile varieties. Ground `31` is pebbled road; `01` is natural trail; `0b` is encounter grass; `74` is flowers; `0f` and source canopy edges are woods. Source `43` is open water: `2d` contains an actual shoreline fragment and must not fill this lake. Signs use the lower-right board in `56`; `09` includes a building fragment and is inappropriate as a freestanding sign. Rails and bollards use their real directional wall quadrants. No new pixel art or content catalog is added.

## Fast review loop

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_central_lakes.py --serve
```

Open `http://127.0.0.1:8770/`. Edit [MN_R15.layout.json](MN_R15.layout.json) for terrain and [MN_R15.json](MN_R15.json) for events. The complete map, viewport sheet, cabin and anchors refresh together; a warm one-shot render/check measured 97ms. The loop does not run Cargo or publish a pack. Republishing changes pack identity: regenerate review saves through the controller fixtures instead of editing their hashes. Invalid geometry remains marked as invalid.

Static checks prove the road/detour polylines, required grass with no dry bypass, actor approaches, dock entrance/closed end, bridge rails, room layout and source coordinates. They do not prove fun or complete gameplay. A directed topology check proves the west-to-east detour exists and its reverse cannot bypass Ron. The Rust publisher independently rejects a dry bypass and a disconnected bridge. Source comparison checks all 576 native exterior metatiles, six outdoor objects, both room returns and exact dialogue/parties.

## Native commands

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_central_lakes -- \
  content-packs/text-tui.crystalpack \
  planning/minnesota/routes/MN_R15.layout.json \
  planning/minnesota/routes/MN_R15.json \
  content-packs/minnesota-central-lakes.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_central_lakes.py \
  --pack content-packs/minnesota-central-lakes.crystalpack --verify-published
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-assets --test central_lakes
CRYSTAL_CENTRAL_LAKES_TEST_PACK="$PWD/content-packs/minnesota-central-lakes.crystalpack" \
CRYSTAL_CENTRAL_LAKES_FIXTURE_SAVE="$PWD/.local/central-lakes-review.crystalsave" \
CRYSTAL_CENTRAL_LAKES_JOURNEY_SAVE="$PWD/.local/central-lakes-journey.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --lib --profile wasm-dev --locked \
  -p crystal-bevy --features location-tester \
  central_lakes_ -- --ignored
CRYSTAL_CENTRAL_LAKES_TEST_PACK="$PWD/content-packs/minnesota-central-lakes.crystalpack" \
CRYSTAL_CENTRAL_LAKES_JOURNEY_SAVE="$PWD/.local/central-lakes-journey.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p geothite --test central_lakes -- --ignored
target/runtime-fps/wasm-dev/geothite play \
  content-packs/minnesota-central-lakes.crystalpack \
  --load .local/central-lakes-journey.crystalsave --compact
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-central-lakes.crystalpack --audio --desktop
```

The resulting isolated pack contains 390 maps and preserves all original maps, assets, HOME/recovery and audio. All 377 audio programs pass PCM hash, frame count and loop checks. Headless native captures were inspected at the dock, cabin and route fork. The fixtures use explicit placement, a level-34 Meganium and level-36 Lapras with source-legal retained moves, six badge bits and a deterministic divider replay. They prove the isolated grass/ledge/trainer journey, not earned badges, regional ports or a playable region. Packs, renders and fixture saves stay ignored.

## Verified review evidence and limits

Three production-controller fixtures pass. The meadow HOP_DOWN transition skips
its wall face and blocks reverse ascent. The bridge works in both directions on
both rows, dock rails stop movement, and both cabin exits arrive on safe exterior
floor. The complete isolated journey crosses the shared grass gate, takes the
northern cache detour and drops beyond Ron without defeating him. Returning
from the east encounters Ron on his short lane. Both Ron and Niko are defeated
through visible battle menus, all four partners are observed, and attack PP
actually decreases. This deterministic run escapes two real wild encounters.

All three signs, both outdoor residents, Eli and the wall chart are read through
A. Movement is rejected while their text owns input. Hyper Potion, Revive and
Max Ether are collected; party HP/status/PP, bag and all corresponding defeat
and pickup flags survive Continue. Stdio MCP continues that exact exported
journey, moves `(4,25)` to `(4,26)`, opens/closes production Start/Pack and emits
no stderr. The optimized production TUI also loads the same save, opens Start/Pack, returns to walking and exits cleanly.

The map was trimmed to 32×18 after proving all activity fits above the removed
six rows. Native bridge framing was inspected after the trim. A later source
art correction replaces fifteen incorrect bank/corner blocks; their four WATER
collision quadrants, events, parties, encounters and every other map remain
identical. The final native bridge capture has a continuous bank without the
previous detached brown fragments. Kanto `64`/`65` are east/west banks; `2e`/`2d`
are small northwest/northeast concave corners, not full banks or open water.

Real Brainerd/Detroit Lakes ports, losing/recovery behavior through Minnesota
hubs, and the earned SQ02 story journey remain integration work. This is a
reviewable native route, not a completed region.

All five native asset regressions pass (Central Lakes and the four previous
route overlays). Exact publication comparison, plan graph checks and all 377
audio checks pass. Changed Rust files pass scoped rustfmt. The normal workspace
format check still fails on pre-existing formatting beginning at
`crates/offset-allocator/src/lib.rs:408`; normal all-target Clippy still stops at
pre-existing `clippy::bad_bit_mask` in
`crystal-core/src/multiplayer/mod.rs:1332`. Focused no-dependency Clippy runs with
the existing `never_loop` allowance pass; existing warnings remain outside this
map work. Native screenshots and collision checks are review evidence, not
final visual approval or a claim that the region is playable.
