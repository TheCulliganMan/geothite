# Published northern winter geography

The current full-state atlas is
`renders/ice/native-geography-v03/atlas/full-state-native-current-preview.png`,
with its full-resolution original and all 99 native map originals retained.
It uses the roof-corrected `minnesota-northern-winter-v02.crystalpack`, file SHA256
`92bb645535db23c78491b6982ddf5fa17b72b1f0434dfb6456a395fbf8fdbd33`,
content identity `043b57b8814714bcf7b14f1571931eadbd139c12ad3e9afba596c193034c358f`.
Its fresh `receipt.json` verifies 683 maps, 99 exteriors, one component, no
overlaps/conflicts and exact placement/seam/warp/Water geometry equality to the
preceding audit. The current preview was inspected by sight. No terrain was
added in atlas gaps, and all earlier epochs remain unchanged.

The detailed read-only boundary audit below uses its retained native v01 parent,
`content-packs/minnesota-northern-winter-v01.crystalpack`, file SHA256
`aa1aa251f7383640412b82b9b3ed9d6e0bb1304569230a79624c75274db60e5a`.
It contains 683 native maps and 99 Minnesota exteriors. All 99 are in one
consistent direct-seam component, with no overlapping footprints or connection
placement conflicts. All 99 also share Winona's explicit seam/warp graph. This
is static topology evidence; authored gates, earned prerequisites and controller
play remain separate checks.

The actual whole-state native atlas and a 16-map northwest connected subset are
under `renders/ice/native-geography-v02/`. The main review images are
`atlas/full-state-native-current-preview.png` and
`northwest/northwest-native-display.png`. Full-resolution originals and all 99
individual native scenes remain saved. The northwest subset keeps native 32
pixels per metatile and exact native placements. It adds no terrain in dark gaps
and invents no International Falls town or direct Rainy River–R17 connection.
The earlier mixed-art parent is retained unchanged under `native-geography-v01`.
Both final display images were inspected by sight.

`receipt.json` verifies the new region's reciprocal east/west connection with
zero offset. Rainy River core `(63,44)` and `(63,45)` meet R20C00R00 core
`(0,44)` and `(0,45)`. All four cells are actual unoccupied `FLOOR`. Other region
boundary cells are visibly snowy pine walls.

The Surf audit respects native bridge travel axes and tests the packed-closed
Minneapolis gate and an in-memory application of its three existing native
open-gate block changes. Both states have 2,020 directed Surf seam lanes and
166 actual neighboring shore closures. Exactly eight unmatched outer Water
records remain: R02C00R00 north `(30..33,0)` and south `(30..33,43)`. These belong
to separate decorative drainage components of 88 and 64 core cells; each has
zero mounting banks and no conservative Foot launch witness. No reachable
winter Water boundary defect was found. The atlas's broader 174-record ordinary
Water inventory is not a leak count: it also includes nonwater neighboring
shores and omits special bridge traversal classification.

The North Shore's blue far-east edge is visibly closed. The full native coast,
exact native pixel crops, 4× nearest-neighbor crops and inspected review sheet
are under `north-shore-closure/`. Every one of the 192 far-east core cells is
`BUOY` at x47 across R11C01R00..03, using actual native blocks32/36/3a. Water at
x46 lies inside the visible white float/rock-shaped line. Northeast/southeast
caps and an internal map join are saved explicitly; there is no Water-looking
`WALL` substituting for this border. A conservative local witness starts at
R11C00R02's actual outdoor room warp `(51,4)`, follows 14 ordinary Foot steps to
R11C00R01 bank `(55,42)`, mounts at Water `(56,42)`, and follows 72 Water steps to
R11C01R01 `(46,24)`, immediately inside the blocked buoy `(47,24)`. The complete
coordinate witness is in `north-shore-closure/proof.json`; these inputs were not
executed through the controller in this audit.

The source-world towns Ely, Detroit Lakes, Moorhead, International Falls and
Mille Lacs remain absent as native town maps. Existing connecting routes do not
establish those towns as finished content. Native GPU review and earned campaign
acceptance are also outside this geography audit.

Reproduce the audit without publishing or modifying a pack:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/audit_outdoor_connections.py --pack content-packs/minnesota-northern-winter-v01.crystalpack --output planning/minnesota/renders/ice/NEW_EPOCH/atlas --scale 8
.local/minnesota-review-env/bin/python planning/minnesota/tools/audit_reachable_surf_boundaries.py --pack content-packs/minnesota-northern-winter-v01.crystalpack --output planning/minnesota/renders/ice/NEW_EPOCH/surf-boundaries
.local/minnesota-review-env/bin/python planning/minnesota/regions/ice/render_native_geography_subset.py --atlas planning/minnesota/renders/ice/NEW_EPOCH/atlas --output planning/minnesota/renders/ice/NEW_EPOCH/northwest
.local/minnesota-review-env/bin/python planning/minnesota/regions/ice/audit_native_north_shore_closure.py --pack content-packs/minnesota-northern-winter-v01.crystalpack --atlas planning/minnesota/renders/ice/NEW_EPOCH/atlas --output planning/minnesota/renders/ice/NEW_EPOCH/north-shore-closure
```

Choose a new epoch for each run. Both focused tools preserve every generated
image using `render_archive.save_render`; neither writes to the pack.
