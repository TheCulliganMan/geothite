# Verified render positions

Run from the repository root with the existing executable. These positions
were visually reviewed; automatic nearest-walkable placement can overlap
building artwork or start on a blocked ledge. Captures belong in `/tmp`.

| Map | X | Y | Purpose | Remaining visible issue |
| --- | ---: | ---: | --- | --- |
| PalletTown | 8 | 12 | Street, homes, lab, shore | Roof joins |
| CeruleanCity | 28 | 18 | Street, house, gym foreground | Repeated roof details |
| CherrygroveCity | 20 | 9 | Buildings and trees | Shore framing |
| Route24 | 8 | 9 | UUDD bridge movement | Flat deck; post shape varies |
| Route4 | 20 | 12 | RRLL movement across grass path | Abrupt ledge ends |
| Route15 | 20 | 9 | RRLL movement | Vegetation repetition |
| ElmsLab | 5 | 6 | Furniture; live orbit2 and4 at zoom0 | Repeated cabinet backs |
| TrainerHouse1F | 5 | 8 | Large-table stool variants | Stair and desk detail |

## Small cross-region pass

```sh
export CRYSTAL_VOXEL_PROFILES=modpacks/voxel-view/profiles.json
RENDER=target/debug/examples/render_at_location
PACK=content-packs/core-modular.browser.crystalpack
"$RENDER" --pack "$PACK" --map PalletTown --x 8 --y 12 --view both --screenshot /tmp/preset-pallet.png
"$RENDER" --pack "$PACK" --map CeruleanCity --x 28 --y 18 --view both --screenshot /tmp/preset-cerulean.png
"$RENDER" --pack "$PACK" --map ElmsLab --x 5 --y 6 --view both --screenshot /tmp/preset-elm.png
"$RENDER" --pack "$PACK" --map Route4 --x 20 --y 12 --view 2.5d --walk RRLL --screenshot /tmp/preset-route4.png
```

Inspect each image and exit status. Do not infer success of earlier commands
from the final command's status. The movement run must report real translation
and zero additional terrain builds during same-map travel. This is a short
regression pass, not complete world coverage. Use `--live` for repeated camera
or profile iteration in one location; see `RENDER_TOOLKIT.md`.

Avoid previously misleading starts: PalletTown(10,9), CeruleanCity(20,18),
Route4(20,9). The first two overlap roof artwork; Route4 RRLL only turns in
place there and correctly fails the movement assertion.
