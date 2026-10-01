# Quiet Gym floor finishes

Azalea and Goldenrod's native pink paths use broad, muted blush-and-cream
ceramic modules. Azalea's green ground and Celadon's green garden floor use a
subdued continuous green material. Celadon's ochre walkway uses broad timber
boards with staggered joints. All three finishes replace high-frequency source
pixels with low-contrast, world-aligned coplanar partitions. Viridian retains
its existing stone material without a palette or geometry change.

These are surface finishes, not authored objects. They add no model coverage,
collision, elevation, actor support, event, warp, or gameplay changes. Every
partition stays within the original quad at its original height. Negative world
origins and viewport crops preserve the same material phase. No edge trim is
inferred from a viewport boundary or an unrecognized neighboring cell.

## Source and destination scope

`gym_floor.rs` admits only the reviewed map, tileset, metatile, subtile phase,
and exact tile. Its native masks contain only `FLOOR` collision quadrants in
the inspected external pack. Azalea `$12` is green floor, not tall grass.
Azalea `$21` permits only the green south skirt; its two northeast green source
cells have `WALL` collision and stay on their original path. Exit carpets,
statues, Celadon flowerbeds, wall art, and unrelated maps sharing either atlas
are outside the new material scope.

A native floor quad must sample its own entire atlas cell with four intact UV
corners. Borrowing a neighboring floor sample cannot recolor a different
surface. The original flat-shape and full-grid-quad checks remain in force.
Native floor destinations with an authored object label are excluded.

Backing beneath the existing Gym scenery is handled separately. The existing
whole-drawing source resolver must succeed again, every owned cell must carry
the exact successful model label, and the floor quad must sample that resolved
model's exact native ground cell. Missing ownership, stale or altered source,
and cropped drawings reject the backing finish atomically. Resolving live
profiles have first refusal over both native floor and model backing, including
custom profiles made from otherwise recognized floor art.

## Inspected external-pack surface counts

| Map | Rose floor | Green floor | Timber floor | Existing model backing | Total finished cells |
| --- | ---: | ---: | ---: | ---: | ---: |
| AzaleaGym | 128 | 261 | 0 | 225 | 614 |
| GoldenrodGym | 912 | 0 | 0 | 504 | 1,416 |
| CeladonGym | 0 | 152 | 216 | 168 | 536 |
| ViridianGym | 0 | 0 | 0 | 0 | 0 new |

Backing is a surface beneath already-counted scenery, never additional modeled
objects. These figures describe full native maps with the relevant complete
scenery present. Crops, changed source and custom profiles can reduce the
number safely. They are not a claim that all scenery in each Gym is complete.
No source images, map/collision catalogs, content packs or compiled outputs are
shipped with this material work.

## Verification

```sh
python tools/check-gym-floor.py --pack content-packs/core-modular.browser.crystalpack
cargo test -p crystal-voxel-view gym_floor
```

The Python check uses `rustc` to exercise the production floor functions, UV
filter, live-profile resolver and Gym source matcher. It materializes native
identity fixtures only in a temporary directory and removes them afterward.
It audits selected cells against independent pack collision permissions and
checks the surface/backing totals above. Its minimal renderer harness supplies
a flat shape shim; native crate tests and actual renderer QA remain separate.

Regressions cover exact source/phase masks, altered and cropped model drawings,
partial or mismatched ownership, full and distorted UV samples, wrong-material
neighbors, custom floor profiles, unchanged warp neighbors, exact coplanar area,
footing/authorship/animation/cutaway preservation, crop/negative-origin phase,
and unchanged existing Viridian stone. After integration, inspect Azalea,
Goldenrod and Celadon at the same native camera positions used for the scenery
review, plus a reversed orbit and a crop through a material boundary.
