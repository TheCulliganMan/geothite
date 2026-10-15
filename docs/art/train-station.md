# Stationary Magnet Train shell

Both `GoldenrodMagnetTrainStation` and `SaffronMagnetTrainStation` use the same
complete five-block train drawing: `$10,$11,$15,$12,$13`, at source-cell `(8,8)`.
The original faceted kit reconstructs the tapered double-ended train as one
coherent 160×32-source-pixel object with a 26.2-pixel rise. Its ivory rounded cab
ends, dark sloped windscreens, red lower belt, framed blue glazing, suspended
magnetic shoes, rear maintenance sides and quiet roof vents have real closed
geometry. All 242 parts are independently editable. No source pixels,
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
`z16..32`. Geometry stays outside these footprints below the 21-pixel door
header. A continuous curved crown spans both portals, preserving the tubular
body silhouette. Thin seals replace the old full-depth pillars. Four flush
plug-door leaves sit slid sideways alongside the apertures on slim tracks;
this is the stationary open-door pose, not timed opening/closing animation.
No threshold or closed door panel covers either boarding opening. The central gate/platform aisle is outside
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

Goldenrod's blue fixtures are four ticket boxes. The complete east-wall bank
across source blocks `$2c/$2d` resolves as one guarded drawing, with four upright
blue cabinets facing west into the waiting room. Screens, controls and ticket
hatches use live source pigments; beverage bodies and labels are omitted from
the shared cabinet geometry. An incomplete source drawing or unknown material
keeps the original source rendering. Saffron does not acquire this bank.
The paired Goldenrod capture in `target/kanto-resume/station-ticket-review/`
shows the four cabinets and records `state_preserved=true`; footing and
collision are unchanged.

- `tools/build-train-station.py` generates the runtime model and editable scene
  from the same original parts, reusing the existing Cable Club geometry/export
  infrastructure and canonical plain JSON model format
- `crates/crystal-voxel-view/src/mesh/train_station_source.rs` is the pure matcher;
  `train_station_scenery.rs` handles custom profiles, reservation and mesh append
- `crates/crystal-voxel-view/models/train_station/magnet_train_shell.mesh.json`
  is decoded and cached once by `train_station_models.rs`
- `art/johto/source/manifest.json` records the hash-verified editable native
  gzip `train-station.blend` file and its canonical runtime-model hash

Run the focused geometry/source checks:

```sh
python tools/check-train-station-models.py
python tools/check-train-station-source.py \
  --pack content-packs/core-modular.browser.crystalpack
cargo test --locked -p crystal-voxel-view --lib train_station_scenery
```

The runtime geometry check proves 4,544 triangles, 242 positive-volume watertight
named parts, clear boarding portals below the continuous crown, four sideways
door leaves and generator/runtime identity. Run it with `--runtime-only` for
the latest tubular model. The editable scene was regenerated and opened in
Blender: all242 meshes, vertices, faces, material bindings and colors were
verified against the generator. The generated runtime document differs only
in float64 normal rounding (maximum2.23e-16); every runtime float32 normal is
identical. The tracked runtime model is unchanged. The scene's native gzip
archive and runtime-model hash pass the full checker. Timed door motion remains
pending; the scene preserves the stationary open pose.
The external-pack check compiles the production
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
