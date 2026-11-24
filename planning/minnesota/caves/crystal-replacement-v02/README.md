# Minnesota cave replacements

These 19 interiors replace the rejected cave pass. Each starts from a new
outline with its own route. The [Crystal reference atlas](../crystal-reference.html)
informs the scale and wayfinding. New terrain, warp positions and event anchors
are authored together. The Fair Presentation pack supplies original Northstar
assets, map identities and outside trail connections.

The [six southern rooms](SOUTH.md) lead through a limestone shelf, a dark
fossil switchback, a flooded spring, a narrow fault and two distinct shelters.
The [four northern rooms](north.md) follow faults, spring banks and a spillway.
The [seven Iron Range rooms](IRON-RANGE.md) follow the working mine from the
surface adit, through its sidings, ducts and pump stations, to the drill floor.

| Room | What the new route asks of the player |
| --- | --- |
| Ore Line | Work up a steep fault from the lower landing, with a rough upper branch and a separate exit from the fracture pocket. |
| Gunflint Grotto | Follow the north bank past the shelter. The far spring cache requires crossing the water. |
| Winding Cleft | Follow the rock fissure from one mouth to the other. The westward drops provide a distinct, shorter return. |
| Inner Basin | Work the controls around the rim, then cross the exposed bed to the lower discovery when the drain opens. |
| Voyageurs Ice 1F | Slide east along a frozen creek, break at its middle shelf, and climb to a far ladder. |
| Voyageurs Ice B1F | Descend a narrow ice cleft to a wide bottom shelf. The long slides, stone rib and raised supply bay give separate destinations. |

The [build recipe](build.json) binds the exact Fair Presentation parent and the
replacement sources. The builder preserves the exterior routes and keeps the
save flags for rewards and story events. Story coordinates, native warp
records and directions that refer to the interior are updated together. The
local pack is selected for review and has a separate save slot for its new
layouts.

## Build and verification

```sh
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets --features pack-authoring \
  --bin pack_northstar_cave_redesign --bin verify_crystalpack
target/debug/pack_northstar_cave_redesign \
  content-packs/minnesota-fair-presentation-v01.crystalpack \
  planning/minnesota/caves/crystal-replacement-v02 \
  content-packs/minnesota-cave-replacement-v02.crystalpack
target/debug/verify_crystalpack content-packs/minnesota-cave-replacement-v02.crystalpack --audio --desktop
```

`review.py` compares the Fair parent and the replacement, confirms all 19
footprints and 64 indexed warps, and writes the full floorplans. The three
regional notes give the corresponding cave checks. Run `review_ice.py` on
the replacement pack to check both Voyageurs floors, every combination of
collected items, and an exit from each reachable stopping square. The compiled
mine review covers 90 arrival and gate cases and 63 versions of the Drill Core
actor movements. All 602 inherited audio programs match their PCM hashes, frame
counts and loop ranges.

The whole-room source renders and 38 native images are retained under
`target/crystal-cave-replacement/`. The native pairs show the same route beats
from each layout's own coordinates. The source floorplans show the Lamp Gallery
lit so the new switchback is readable.
