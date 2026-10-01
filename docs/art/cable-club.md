# Cable Club booth partitions

Two original full-volume models form four long partitions and two shorter Time
Capsule vestibule returns. Powder-blue recessed panels, slim metal stiles,
ivory crowns and continuous dark toe plinths give the long booths a coherent
low-poly construction. Every named part is an editable closed solid, including
its underside. The geometry uses authored material colors and contains no
copied game art, ROM, textures or imported meshes.

```sh
python3 tools/build-cable-club.py target/cable-club --runtime-only
blender -b --threads 2 --python tools/build-cable-club.py -- target/cable-club
python3 tools/check-cable-club.py
cargo test -p crystal-voxel-view cable_club
```

The Python-only path generates the exact cached model documents without
Blender. `--skip-preview` writes the same editable source without rendering.
The source manifest retains `cable-club.blend` in two bounded, hash-verified
gzip/base64 chunks. Its named asset collections and visible studio assemblies
use the identical polygon definitions as the runtime exports.

## Source ownership and openings

The adapter accepts the eleven explicitly named upstairs maps, each with the
inspected 32 by 16 source-cell layout. Three exact source fingerprints guard
independent west, record-sign and connected east networks. Fingerprints include
atlas, metatile identity, source tile and phase plus unowned guard cells and
openings. Map-space anchors use the published signed grid origin, so padding,
scrolling and viewport clipping cannot relocate or partially match a drawing.
Missing native block-04 floor, any modified source cell, incompatible phase or
any custom live-profile overlap rejects the complete network. Only the three
unchanged bundled partition profiles yield to this kit. Failed matches retain
the prior source renderer.

Each map owns 88 source cells: four 2 by 8 long partitions and two 2 by 6 short
returns. No shell spans a source opening. All owned collision quadrants in the
inspected pack are WALL; every actor and warp's full 2 by 2 source footprint
remains outside ownership. Source inscriptions on the record-sign partition
stay as live textured lettering on its south end. Appending uses a native
zero-height floor underlay and leaves footing, collision, actors, warps and
controller data alone. Runtime height is 16 source pixels.

## Cost and verification limits

The two cached prototypes contain 728 and 616 triangles, stored in roughly
16 KB. Their exact bounds are `[0,0,0]` to `[16,16,64]` and `[16,16,48]` in
X/up/depth source-pixel units. A full room assembles 4,144 solid triangles and
four extra record-sign triangles into the existing terrain batches. There are
no per-part runtime entities or new texture assets.

Read-only base-pack inspection verifies 33 complete networks, 66 shells and
968 claimed cells across eleven maps. This includes cap/return art outside the
earlier 730-cell residual estimate and replaces the three prior generic models
on the active upstairs map. The geometry checker verifies deterministic cache
regeneration, finite unit normals, winding, nondegenerate triangles, positive
volume and watertightness for all 48 named prototype parts. Focused Rust tests
cover every identity field and guard cell, sparse topology, native actor/warp
footprints, custom profiles, clipping, padded/scrolled frames, missing floor,
atomic append, physical bounds and unchanged footing.

Integrated native Cable Club and Celadon beta-room captures were inspected,
in addition to the initial front and orbit-two views. The central Time Capsule machine
and backdrop remain eight source cells per map, above a separate four-cell
source door/warp drawing; those are outside the partition family. The source
floor strips beneath counters and booths also still need coordinated material
finishing. Geometry coverage does not make this whole room complete.
