# Joined gate counters and telephones

The gate kit supplies original full-volume timber cabinetry and teal telephone
consoles. Sixteen NESW edge modules join at exact shared bounds. Exposed edges
have an eased worktop, inset framed panel, carved vertical flutes and a recessed
plinth. The phone has a separate receiver, cradle, twelve tactile keys, display,
return recess and solid coiled cord. No source pixels or game models are imported.

The source adapter matches 14 inspected connected drawings across 28 base maps.
The matching envelope includes one surrounding cell wherever available and every
internal opening. It checks tileset, metatile, source phase and tile identity
before taking any cells. The sparse ownership mask excludes floor, shadows,
walls, doors, approach paths and service gaps. Source boundary flags distinguish
intentional map-edge cuts from clipped unknown drawings. A missing floor,
changed source, cropped network, occupied cell or overlapping custom live
profile leaves the whole connected drawing on its existing rendering path.
The actual same-atlas gate floor tile 01 supplies every vacated source cell.
The original per-cell footing, collision, warps and controller state remain intact.

The checked source scope is 57 connected networks containing 1,482 counter cells
from the remaining-family checklist, 96 phone cells (24 consoles), and 52 extra
counter continuation/cap cells needed to keep those networks connected. Those
52 comprise 48 cells in blocks 2b/34 and four counter cells around the National
Park telephone in block 28. Counts are source 8px cells, not object or mesh counts.

Runtime and source validation:

- `python3 tools/check-gate-counter-models.py` validates all 17 exports, full-volume
  bounds, materials, normals, embedded references and shared flush module extents
- `cargo test -p crystal-voxel-view gate_counters` checks identity changes,
  clipping, sparse topology, custom profile priority, whole-network overlap
  rejection, same-atlas underlay UVs and unchanged footing
- `python3 tools/johto_art_sources.py --check` verifies the editable source chunks
- The ordinary all-map authored audit must verify the exact new source claims;
  base-map counts alone do not certify dynamic map states or art quality

Run Blender in background with `tools/build-gate-counters.py` to rebuild assets
and the two real geometry previews. The editable source preserves named pieces,
material assignments, all edge variants and assembled counter examples. Native Route2, Route29/46, Route35/National Park and Ilex gate captures verify
joined surfaces and retained openings. Individual underground booths, Victory
Road and other layouts still need their own art review.

Native padded frames exposed a boundary-origin mismatch after the initial
base-map audit. Boundary checks now use the source map origin and verified
source dimensions, not the surrounding canvas extent. Base, padded and scrolled
fixtures accept all 57 networks; complete crops remain valid, while 228 partial
crops, 78 counterfeit copies and 171 overlapping custom layouts are rejected.
The production audit confirms 1,630 owned cells without changing floor footing.
