# Rocky shores and cave entrances

This revision changes 36 Minnesota outdoor map sections. Its 64 shoreline
placements put rock on 18 numbered routes and 11 town, lake and landmark
maps. The stone follows the landform. Low slabs sit along the Driftless
shore, rock breaks through the snow on Gunflint, and granite shelves wrap
around the Boundary Waters portage. Each placement keeps its original four
movement permissions.

At low landings, the old water tiles sometimes drew a brown cliff or timber
edge across a place where the player could already Surf. The revision opens
95 water tiles, including the Grand Marais beach, Boundary Waters
portage, Driftless Trail and Gunflint inlet. The native water animation runs
right up to the bank. Shiner Narrows uses its own local ripple frames. Rocky
edges continue along adjoining high ground.

Fourteen exterior cave mouths were checked against the installed map graphics
and warp tables. Six needed new shoulders. The doors at Former Shores, western
Route 1, northern Rock Cut and Border Pines gained continuous cliff at their
existing positions. Two doors and their outdoor return squares moved. Positions
below are in 16-pixel movement squares.

| Entrance | Previous door → new door | Previous landing → new landing |
| --- | --- | --- |
| Eastern Route 1, fossil bluff | (40, 19) → (36, 19) | (40, 20) → (36, 20) |
| Western Granite Passage, Route 20 | (60, 33) → (62, 31) | (60, 34) → (62, 32) |

The Route 1 stair remains at the eastern end of the cliff. The Granite Passage
shoulder continues into the neighboring route section. Each door has at least
32 pixels of genuine cliff on both sides, a trail approach and a matching
interior return.

## Build and verify

Start from the matching external cliff-faces pack. The Field and Mothlight
build receipts identify each parent and result by SHA-256. The example requires
a fresh output path and verifies the serialized pack before reporting success.
Run the Field build and its independent checks from the repository root.

```sh
revision=planning/minnesota/revisions/shore-rocks-cave-mouths-v01
parent=content-packs/minnesota-cliff-faces-v01.crystalpack
result=content-packs/minnesota-rocky-shores-v01.crystalpack
cargo run --locked -j1 -p crystal-assets --example pack_outdoor_contours -- \
  --rocky-shores "$parent" "$revision" "$result"
uv run --with pillow --with cbor2 python "$revision/review.py" "$parent" "$result"
uv run --with pillow --with cbor2 python "$revision/check-caves.py" "$result"
```

The builder checks 267 distinct public shore positions using the game's walking,
fishing, Surf and landing routines. It also checks each cave transition and
15 trail and stair approaches. The independent audits compare the original and
revised native art, maps, events and encounter terrain. They verify that
both 8-pixel water tiles facing the player at each of the 187 newly opened
contacts use animated water. Eight water squares at bends and beside grass are
checked for stray pieces of the old cliff. The cave audit enumerates every exterior
door and both ends of its warp links.
