# Frozen inland water with native cave-rock boundaries

**North Shore remains unfrozen.** The current frozen-water overlay covers the existing inland lakes in R12,
R13, R17 and Rainy River: seven chunks. It retains every map footprint, port,
original walking tile and event. North Shore exterior modules remain exact to
the unfrozen parent.
All eight R11 chunks must use warm `mn_north_shore` with no ICE collision.

The contract `frozen-water-compact-v02.json` records exact preimages on the
current compact fruit pack. Native Ice Path `19` rock pixels provide solid
lake rims and closed map edges. Water becomes native sliding ICE. Original
walking paths, trainers, trees and gates remain intact; the reviewed two-tile
R17 snow ramp opens the lake. Only real in-range connections permit edge ice.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/author_frozen_water.py \
  --base content-packs/minnesota-fruit-bank-compact-v01.crystalpack
target/river-fun/wasm-dev/pack_frozen_water \
  content-packs/minnesota-fruit-bank-compact-v01.crystalpack \
  planning/minnesota/regions/ice/frozen-water-compact-v02.json \
  content-packs/minnesota-fruit-ice-compact-v02.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_frozen_water.py \
  --written content-packs/minnesota-fruit-ice-compact-v02.crystalpack \
  --output planning/minnesota/renders/frozen-compact-v02/written-v01
```

The Rust publication validates current compact northern geometry, native art,
collision and pack identity, then verifies the actual written readback. The
read-only review compares source pixels, every map and unrelated catalogs/audio;
it records exact native slide/rock-stop and edge controller courses.

All historical renders remain. [Earlier receipts](FROZEN-WATER-HISTORICAL-v01.md)
are superseded because their icy North Shore and oversized parent were rejected.
Those packs are retained with `.retired` suffixes outside current pack selection.

Verified final content identity: `b27de81ba40675ab2e7e336899920dcdbb451627dcb0740af0d3ad540ce77b8d`.
File SHA256: `344775b2a985e21b2d10be8a5d4737dd704feb286846c132ceafe78127357331`.
The written review verifies 1,380 water quadrants converted to ICE, 393 native
rock quadrants and three solid border recipes. All ten snowy exterior modules
contain no liquid water. All eight warm R11 modules remain byte-identical to
the fruit/bank parent. The four production controller courses pass: R12 eight
tiles across one seam, R13 sixteen tiles, R17 eight tiles and Rainy River 24
tiles, including blocked rock input and Continue. Reached R12/R13 native2D
frames are retained under `renders/frozen-compact-v02/native-v01/`.
