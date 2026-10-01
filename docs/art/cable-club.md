# Cable Club room kit

Three original full-volume models form four long partitions, two shorter Time
Capsule vestibule returns, and the Time Capsule machine/backdrop. Powder-blue
recessed panels, ivory crowns and dark toe plinths unify the room. The capsule
adds two jade receiver columns with brass bands, a dimensional hourglass
chronometer, an inset control screen, physical keys and a closed maintenance
panel. Every named model part is an editable closed solid with an underside.
The geometry uses authored material colors and contains no copied game art,
ROM, textures or imported meshes.

```sh
python3 tools/build-cable-club.py target/cable-club --runtime-only
blender -b --threads 2 --python tools/build-cable-club.py -- target/cable-club
python3 tools/check-cable-club.py
cargo test -p crystal-voxel-view cable_club
```

The Python-only path generates the exact cached model documents without
Blender. `--skip-preview` writes the same editable source without rendering.
The source manifest retains `cable-club.blend` in three bounded, hash-verified
gzip/base64 chunks. Its named asset collections and visible studio assembly
use the identical polygon definitions as the runtime exports.

## Source ownership and openings

The adapter accepts eleven explicitly named upstairs maps with the inspected
32 by 16 source-cell layout. Three exact source fingerprints guard independent
west, record-sign and connected east partition networks. Five additional
fingerprints guard the capsule and four sparse floor groups. Fingerprints
include atlas, metatile identity, source tile and phase plus unowned guard
cells and openings. Map-space anchors use the published signed grid origin,
so padding, scrolling and viewport clipping cannot relocate or partially match
a drawing. Missing native block-04 floor, any modified source cell, incompatible
phase or any custom live-profile overlap rejects the complete affected network.
Only the three unchanged bundled partition profiles yield to this kit. Failed
matches retain the prior source renderer.

Each map owns 88 partition cells: four 2 by 8 long partitions and two 2 by 6
short returns. The capsule owns eight cells, source columns 26–27 and rows 0–3.
Its 16 by 32 source-pixel footprint ends before the separate four-cell source
doorway at rows 4–5. No shell spans a source opening. All 96 model-owned collision
quadrants in the inspected packs are WALL; every actor and warp's full 2 by 2
source footprint remains outside model ownership. Source inscriptions on the
record-sign partition stay as live textured lettering on its south end.
Appending uses native zero-height floor underlays and leaves footing, collision,
actors, warps and controller data alone. Partition height is 16 source pixels;
the capsule crown rises to 22 source pixels within its own footprint.

The four floor groups finish 34 source cells per room. Only the inspected
source-tile-01 strips with FLOOR collision qualify. They use the established
ceramic palette and signed world-grid seam phase at exactly zero height.
The guards include adjacent counter and opening identities; a custom profile
or reserved overlap rejects the complete affected floor group. These finishes
are architectural surfaces, never added object-coverage credit. Two floor-like
source strips at `(12, 2)` and `(20, 2)` are in WALL quadrants and remain source
art. The native capsule doorway also remains source art.

## Cost and verification limits

The three cached prototypes contain 728, 616 and 1,176 triangles and together
occupy 29,521 stored bytes. Their exact bounds in X/up/depth source pixels are
`[0,0,0]` to `[16,16,64]`, `[16,16,48]` and `[16,22,32]`. A full room assembles
5,320 model triangles and four extra record-sign triangles into existing
terrain batches, plus the sparse zero-height floor finish. There are no
per-part runtime entities or new texture assets.

Read-only inspection of both local external packs verifies 88 complete guarded
networks across eleven maps: 968 partition cells, 88 capsule cells and 374 floor
finish cells in each pack. Collision, nonoverlap, actor/warp footprints and
mutation rejection were checked against those source identities. The geometry
checker verifies deterministic cache regeneration, finite unit normals, winding,
nondegenerate triangles, positive volume and watertightness for all 90 named
prototype parts. Focused Rust regressions cover source identity guards, sparse
ownership, custom-profile/reservation fallback, padded/scrolled frames, native
bounds, atomic append, zero-height flooring and unchanged footing. A standalone
Rust surface check also verifies the new helper exactly matches existing room
ceramic over positive and negative grid origins.

Actual Blender assembly and close-up renders were inspected. Integrated native
Cable Club and Celadon beta-room captures verify the capsule, floor continuity,
actor openings and retained source doorway. The all-map production audit
retains 388 verified maps without errors and adds exactly 88 capsule cells;
floor finishes receive no object-coverage credit. This pass
finishes the capsule and verified floor strips, not every source-art element or
every room in the game.
