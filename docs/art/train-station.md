# Stationary Magnet Train shell

Both `GoldenrodMagnetTrainStation` and `SaffronMagnetTrainStation` use the same
complete five-block train drawing: `$10,$11,$15,$12,$13`, at source-cell `(8,8)`.
The original faceted kit reconstructs the tapered double-ended train as one
coherent 160×32-source-pixel object with a 26.2-pixel rise. Its ivory folded cab
ends, dark sloped windscreens, red lower belt, framed blue glazing, suspended
magnetic shoes, rear maintenance sides and quiet roof vents have real closed
geometry. All 82 named parts are independently editable. No source pixels,
textures, imported meshes or game data are embedded in the asset.

The earlier 128-cell residual was a plot count for the four end blocks across
two maps, not 128 train/body cells. Per station the full 80-cell plot contains:

- 60 true body/detail cells, now owned by the stationary shell
- 8 live source door-art cells, two complete 2×2 drawings, retained unowned
- 12 native rail/backing cells, retained unowned

Across both stations this is 120 new body cells, 16 retained door cells and 24
retained rail/backing cells. The center block omitted by the old residual adds
24 body and 8 backing cells across the pair. Do not report these as 160 new
objects or as 160 newly modeled cells.

## Source and gameplay boundaries

A single whole-object guard checks the 20×8 rectangle beginning at `(8,8)`:
the complete train drawing plus the adjoining four platform rows. It checks the
map name, atlas, every metatile, subtile phase and tile identity, source-map
anchor, complete viewport inclusion and an exact native rail-ground sample
(`train_station`, block `$14`, row 0, art `$1f`). No broad floor fallback is used.
Every custom profile or existing reservation in that rectangle rejects the
whole model, including overlaps in unowned doors or platform approaches. The
append pass rechecks the complete guard and the native ground before mutation.

The two actual door/warp positions are movement tiles `(6,5)` and `(11,5)`.
Within the model these are the 16×16 footprints `x32..48` and `x112..128`, both
`z16..32`. Every triangle is outside those footprints at every height. This
includes the roof, underside and all jambs. The rear half continues behind the
open recesses; the roof is deliberately cut back above the source door areas so
that the original live door drawing stays visible. No solid threshold, opaque
panel or glass spans either opening. The central gate/platform aisle is outside
ownership and geometry. Native door and rail textures pass through the ordinary
source renderer at their original position; they are never baked into this kit.

Only successful body cells receive coplanar sampled rail backing and an authored
coverage label. Footing heights are unchanged. The resolver and generator cannot
change collision, NPCs, scripts, scene state, movement or warps. Both inspected
maps have no script block changes. The existing power-restoration/PASS gates,
officer and player walking sequences, reciprocal warps, arrival coordinate
event, `special MagnetTrain`, `warpcheck` and `MAPSETUP_TRAIN` remain authoritative.
The separate `VisibleMagnetTrain` travel animation is not changed or replaced.

## Files and checks

- `tools/build-train-station.py` generates the runtime model and editable scene
  from the same original parts, reusing the existing Cable Club geometry/export
  infrastructure and deterministic gzip/base64 runtime format
- `crates/crystal-voxel-view/src/mesh/train_station_source.rs` is the pure matcher;
  `train_station_scenery.rs` handles custom profiles, reservation and mesh append
- `crates/crystal-voxel-view/models/train_station/magnet_train_shell.mesh.json`
  is decoded and cached once by `train_station_models.rs`
- `art/johto/source/manifest.json` stores the hash-verified editable
  `train-station.blend` through bounded ordinary text chunks

Run the focused geometry/source checks:

```sh
python tools/check-train-station-models.py
python tools/check-train-station-source.py \
  --pack content-packs/core-modular.browser.crystalpack
cargo test --locked -p crystal-voxel-view --lib train_station_scenery
```

The geometry check proves 2,232 triangles, 82 positive-volume watertight named
parts, both fully empty boarding footprints, generator/runtime identity and
editable-source reconstruction. The external-pack check compiles the production
pure matcher with `rustc`, tests every guard cell against art, block, phase,
atlas and reservation changes, and covers wrong maps, missing/custom-owned native
ground, clipping, padding and source origin. It also verifies the actual source
collision and boarding script sequence. No pack or generated fixture is retained.
The adapter tests cover unchanged footing, retained native doors/backing,
exclusive ownership, customized door/platform profiles and stale append rejection.

Source/script invariants are not executed gameplay proof. Production controller
boarding/arrival and native map views remain integration gates. Verify both
stations, their door approaches, the two power/PASS denial paths and a successful
trip in each direction through `VisibleShellController`; retain the existing live
travel animation. A model preview alone does not close those checks.
