# Minnesota cave entrances

Mt. Mortar’s western doorway on Route 42 occupies a 16×16 walking square. In Crystal the brown wall beside the opening is also 16 pixels high. Two rows of stone continue to each side, and the lighter rock above them extends back into the hill. Dark Cave on Route 31 shows how the same front turns into a sidewall near a doorway. Both original maps can be inspected in the parent pack. [Mt. Mortar map events](https://github.com/pret/pokecrystal/blob/master/maps/Route42.asm) · [Dark Cave map events](https://github.com/pret/pokecrystal/blob/master/maps/Route31.asm)

| Approach | Change |
| --- | --- |
| Rock Cut, north | Both flanks of the arch have a full 16-pixel face. The western corner and eastern sidewall follow the same height into the wooded hillside. The eastern stair and the trainer lane retain their original walking routes. |
| Rock Cut, south | The rock rises farther back on the east side of the opening. From the fork, the stair is visible to the left and the cave is straight ahead. |
| Route 1, eastern mouth | A short worn path curves from the cave landing toward the grass and the stair. The grass keeps its original encounters. |
| Former Shores | The fossil ridge turns back beyond the entrance, exposing its right side and carrying the rock face around the bend. |
| Iron Range mine | A lintel and upright posts frame the dark shaft. The rock projects beside it and the working trail reaches the opening across the snow. |

The audit finds entrances from warp destinations, movement permissions and native doorway pictures. It checks every Minnesota map, including terrain that a story event or field move can change. It follows both sides of the Granite Passage cliff across the Route 20 map boundary. Indoor ladders, the two Rock Cut floor exits and the story-controlled Iron Drill passage have their own movement rules.

Across both editions the audit found 14 exterior mouths on 108 outdoor maps. Five approaches use 28 changed metatiles. The other nine already had supported cliff faces. The final packs passed all 26 exterior-to-cave warp bindings, 15 return bindings, and 15 two-way walking routes to the entrances and the Route 1 fossil stair. The decoded mine doorway is identical in both parents, though Field stores it at block 0x89 and Mothlight at 0x88. The writer verifies its shape and movement rules before adding the timber.

## Reproduce the Field pack

Use the parent in `field-build.json` and a fresh output path. The Mothlight recipe gives the matching paths and hashes for the local Lodge edition.

```sh
src=planning/minnesota/revisions/crystal-cave-faces-v01
parent=content-packs/minnesota-rock-cut-north-face-v01.crystalpack
output=content-packs/minnesota-crystal-cave-faces-v01.crystalpack
CARGO_INCREMENTAL=0 nice -n 14 cargo run --locked -j1 -p crystal-assets \
  --example pack_outdoor_contours -- --crystal-cave-faces "$parent" "$src" "$output"
uv run --with cbor2 --with pillow python "$src/review.py" "$parent" "$output"
```

The writer tests entering the caves, resolves the return events, and checks the Route 5 stairs, ledges, bridge and exits. It then reads back the complete output. The independent audit measures the installed native rock and checks the map and artwork changes against the declared placements. Both checks reject the earlier one-row Rock Cut north wall.
