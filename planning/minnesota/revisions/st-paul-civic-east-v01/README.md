# St. Paul civic streets, Summit and eastern services

Six complete native facades move to form useful neighboring groups. The museum
and annex share a lower civic frontage. The Summit study house, exhibition club
and press have bounded approaches and retain both original pickups. The eastern
gym, clinic and community hall face one short street; workshop, cafe and freight
face the next. A connected pine boundary replaces the empty eastern service yard.
The city stays40x34, with all40 existing destinations and the connected river.

This continues the selected market/care-court work. It changes194 terrain cells
and12 indexed warp positions, preserving every room, program, opponent, reward,
sign, resident, recovery position and regional connection. It adds no new activity
or story gate. Native urban turf is FLOOR; no encounter grass or incidental Cut
tree is introduced. There is still broader city and regional quality work to do.

`after.layout.json`, `recomposition.contract.json`, `town.dossier.json` and
`sources.json` are the current terrain, anchors, room contracts and lineage.
The dossier gives all40 real entry/return pairs and35 current expansion facade
footprints with exact source rows. Original creation metadata remains historical.

## Complete facade moves

Coordinates are32px metatiles; the listed native rectangles move intact.

| Destination | Before | After | Width x height | Entry warp |
| --- | --- | --- | --- | --- |
| Civic Museum |24,8 |23,9 |4x3 |35 |
| Summit Move Study House |29,2 |29,3 |2x2 |37 |
| Summit Press House |29,10 |29,8 |2x1 |41 |
| Community Hall |31,12 |29,13 |5x3 |59 |
| East Workshop |22,18 |22,17 |4x3 |57 |
| Freight Office |31,17 |29,17 |4x3 |63 |

Ordinary walking distances include authored bodies and native ramp/deck
permissions. They compare passive exterior returns, without Surf or other hops.

| Useful link | Before | After |
| --- | ---: | ---: |
| Annex to museum |14 |10 |
| Eastern clinic to community hall |13 |7 |
| Workshop to cafe |11 |9 |
| Freight office to depot |28 |24 |
| Study house to exhibition club |11 |9 |

The two main eastern frontages are one metatile/two walking tiles deep. The
ordinary side alleys at x26 and x33 join them to the depot approach. The press
approach at x28 remains continuously paved around the community block. The
eastern forest joins the existing map boundary, rather than scattered tree props.

## Bluff and connections

The original low native07 drop remains at metatile(22,7). From core(44,13), walk
onto(44,14), then hop south to(44,16). Reverse input is denied. The existing
Kanto05 civic ramp at(18,7) supplies the19-step ordinary return, with the standing
guide included. The new windbreak at x24..31/y7 closes a longer eastern
walk-around; it does not create the short ramp return or change its distance.
No high cliff, stair art or palette is changed.

The source and MCP planners now include native LADDER as ordinary walking
collision where there is no authored warp. Earlier floor-only comparison
`walking-comparison-v216.json` and the long-detour replay remain rejected planner
evidence; corrected measurements are in `walking-comparison-v219.json`.

All three connection offsets, the complete perimeter, every water/bridge tile
and the original native hop remain exact. The west road still meets
`MnR04C00R00` at offset15; east and south water still meet
`MnMississippiLowerWoodsC00R01` and `MnMississippiConfluenceC03R00` at offsets14
and36. Eight exact reciprocal water lanes are retained in the contract.

## Verified native work

Local candidate pack:
`content-packs/minnesota-st-paul-civic-east-v02.crystalpack`.
Resolve `world.json` for current selection. The parent is
`st_paul_inhabited_streets_revision`; this retains the newer shoreline rocks,
route openings, Duluth inner-buoy clearance, cafes, harbor and R10 cave.

Retained evidence is under `renders/st-paul-civic-east-v211/`:

- `whole-root-preservation-v214.json` independently reconstructs all changed
  event coordinates and mirrors; all731 other maps, art, audio, catalogs,
  trainers, scripts, rewards, spawns and publication records remain exact.
- `production-tour-v218/receipt.json` has1362 real stdio/controller calls:
  both bridge walks, the native drop and denied reverse, the19-step ramp return,
  all40 building entries/returns, service and river boards, Save/Continue and
  Start. Stderr is empty. Nothing is repositioned or granted after input begins.
- `native-gpu-v214/` retains six fresh640x576 native views with zero differing
  pixels. All six, both whole-city source passes and district originals were
  visually inspected. `views.json` contains their exact positions.
- `audio-v215.log` verifies all377 PCM hashes, frame counts and loop ranges.
- The discarded first native pack and every source/render/harness pass remain
  retained. The first hop harness needed the ordinary step onto the lip before
  pressing south to jump; that correction changes no game state or terrain.

Fixtures begin with the original Minnesota POUCHID45/CINDORE12, money10000,
Repel and local placement. This is local geometry/access verification, not
earned progression, gym completion or acceptance of interior transactions.

## Fast source loop and publication

This executed source command keeps the pack loaded, checks native collision and
all indexed anchors, and saves a new full-town original on every edit:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/render_city_streets.py \
  --parent content-packs/minnesota-st-paul-inhabited-streets-v03.crystalpack \
  --directory planning/minnesota/revisions/st-paul-civic-east-v01 \
  --output planning/minnesota/renders/st-paul-civic-next --watch
```

Measured check/render passes take about0.2-0.3seconds after pack loading. The
native publisher is the existing Rust `recompose_authored_city` entry point;
the executed retained executable is `.local/duluth-garden-cached-recompose`,
with arguments parent pack, layout, contract and fresh output pack. No new Rust
implementation or fresh Cargo build is claimed. Always re-resolve the selected
parent before another publication and retain independently accepted changes.
