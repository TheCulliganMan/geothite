# Minneapolis citywide cycling

The existing 18 optional cycling opponents now ride separate streets across both banks. The former east/south cluster is spread through the mill exchange, museum, boathouse, Dispatch waterfront, west cabin, canal homes, gardeners lane, music/print shops, library/northeast, east garden avenue, department-store lane, Gym block, market square, southern homes and kitchen/cycle club. Three riders cross the upper, central and southern main bridges.

`routes.json` is the implementation source. Coordinates use 16px gameplay tiles. Each start is the top-left corner; `size` gives native patrol width and height. Existing `SPRITEMOVEDATA_PATROL_CLOCKWISE` walks east, south, west and north with eight-frame bicycle strides. Circuits range from 10 to 48 steps and contain 440 distinct tiles. No pair shares a tile, so riders cannot jam one another. Door/return records, signs, fixed residents, coordinate events and the regional arrival are excluded. The player still collides with riders; the native loaded viewport schedules movement.

The upper bridge turnaround was moved two walking tiles east after the access check found a two-rider corner obstruction at the mill approach. The revised configuration passed 1,000 shared phases, every individual route position against the other starts, and all four adjacent cross-route pairs: 1,444 configurations. All 40 doors, 40 return tiles and 34 readings retained access in those checks. This is a static graph check, separate from actual gameplay.

The selected local pack is `content-packs/minnesota-minneapolis-citywide-cycling-v02.crystalpack`, SHA256 `504938ec2d3b0230674ddc5ae4f8ec13e9e713dd4189c1f128ff23112959b24a`, identity `a7f6270ac5c7cdc83fe0f7484fd8485990ceb9749a0fde803dc43cbd22b90bad`. It extends `route_grass_st_cloud_fronts_revision`, preserving that revision's St. Cloud work and the dedicated blue/green helmeted cyclists. The complete raw-root comparison allows only the 18 cycling starts/extents and their typed/raw mirrors to change. All 731 other full map modules, terrain, facades, rooms, story gates, teams, dialogue, sprites, runtime files and audio are exact.

The later selected `route_grass_r14_forest_bank_revision` retains this entire Minneapolis module, nonterrain data and runtime assets exactly. Its pack is `content-packs/minnesota-current-r14-forest-bank-v01.crystalpack`, SHA256 `e9f4108d3b0a35f04145e4928446c03845982e1f13d7ff8f34e383bd701757d5`. The current-successor comparison is retained in `current-successor-v379.json`; it preserves the verified cycling work while adding the separately reviewed Route 14 terrain change.

All 18 opponents retain their optional challenges, active original Minnesota parties, scaling, defeat flags and the 18 native `reloadmapafterbattle` dispatches. The roster check covers all 36 original party slots against the current roster and retirement list.

## Executed publication

The Rust publisher and child readback completed with this command. Reruns require a fresh ignored output filename.

```sh
target/runtime-fps/debug/revise_minneapolis_cycling_routes \
  content-packs/minnesota-current-st-cloud-fronts-v02.crystalpack \
  df5e4c47500d84046f542418a203c8d58b0e119e97bc235083434464c8682ef3 \
  planning/minnesota/revisions/minneapolis-citywide-cycling-v01/routes.json \
  content-packs/minnesota-minneapolis-citywide-cycling-v02.crystalpack
```

Publisher source: `crates/crystal-assets/src/minneapolis_cycling_routes.rs`. It binds the entire parent city module by SHA256, requires the exact 18 existing actors, validates track collisions and protected anchors, and updates each existing event mirror. It preserves existing script programs and art.

## Review and gameplay evidence

Evidence is retained under `planning/minnesota/renders/minneapolis-cycling-v311/`:

- `source-v320/whole-city-routes-original.png`: complete city with numbered review tracks; the overlay is review annotation.
- `final-whole-town-v339/whole-town-original.png`: final pack's exact native source-art city and actors, identical to the reviewed source preview.
- `final-midroute-laps-v343/`: final-pack production-controller observations, actual reached saves and eight native 2D captures. All 18 riders completed at least two full laps: 880 forward steps, all 440 route tiles observed, and a reached save/Continue for each rider.
- `native-review-gallery-v347.png`: inspected contact sheet of those eight native views, using the game's actual night palette.
- `production-tour-v327/receipt.json`: 1,030 production MCP calls, both banks, all three bridges, six actual interiors and their returns, Start, reached save and fresh-process Continue; stderr is empty. This tour used the v01 cycling child. Its Minneapolis module, runtime art, parties and all other gameplay inputs are identical in the final v02 child; v02 additionally preserves the newer St. Cloud front correction.
- `midroute-Continue-mcp-v346/receipt.json`: 54 read-only RPC calls loading all 18 actual reached mid-route saves in fresh production stdio processes; every rider remains away from its starting corner and on its own route, all original inputs remain unchanged, and stderr is empty.
- `whole-root-preservation-v335.json`, `traffic-access-v328.json`, `roster-policy-v341.json`: exact final preservation, route access and active roster checks.

The initial local positions and parties are explicit QA fixtures. All later movement, yielding, dialogues, battles and reached saves use `VisibleShellController`; the fixtures do not establish earned regional progression. The final replay saves every rider away from its start and proves exact resumed positions through production Continue. The four challenge regressions also pass, including native PP mutation and battle Continue for riders 01, 13 and 18, optional decline, normal patrol movement, and victory return/input release. The publisher guard case passes valid input and rejects stale, overlapping and blocked route sources.

The reusable controller regression is `minneapolis_citywide_cyclists_complete_distinct_routes_and_continue` in `crates/crystal-bevy/src/bevy_shell/tests/minneapolis_citywide_cycling.rs`. It follows riders with ordinary player inputs, yields rather than occupying their next tiles, checks native route membership, requires every tile to be observed and two full laps, saves away from the starting corner, reloads through production Continue and verifies resumed positions. Set `CRYSTAL_INTERIOR_PACK` to the absolute final pack, `RUST_MIN_STACK=67108864`, and `CRYSTAL_CYCLING_REVIEW_DIR` to a fresh ignored evidence directory before running the ignored test. Previous outputs and rejected passes remain intact.

Both affected-crate all-target Clippy rechecks passed, with existing warnings. The earlier attempts were blocked by a concurrent R14 range-expression compile error; its repaired-source recheck is retained in `code-recheck-v358.json` and the two `clippy-*-v345.log` files. The earlier failures remain archived.

Workspace formatting has existing failures, beginning in `crates/offset-allocator/src/lib.rs:408`; changed leaf files pass scoped rustfmt. This revision does not complete the broader map-quality or regional campaign goal.
