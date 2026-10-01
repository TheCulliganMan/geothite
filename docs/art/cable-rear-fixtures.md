# Cable Club rear fixtures and actual door roles

The upstairs room now gives the two link receivers a proper powder-blue cabinet,
rounded ivory monitor housing, recessed dark display, physical ready indicator,
keys, side vents, brass latch, inset counter front and a narrow raised cable
return. The original model has 29 named closed parts and 812 triangles. Its
source-pixel bounds are `[0,0,0]` to `[16,20,32]`. The upper-left 8 by 8 corner is
empty in every component; the taller apparatus starts behind that negative
space. Original colors and geometry are shared by the runtime cache and editable
`cable-backwalls.blend`; no game pixels or imported geometry are baked in.

The exact existing Computer and PictureFrame assets now bind to all eleven
inspected upstairs maps. This also corrects an earlier semantic mistake: each
four-column “wall displays” source profile contains a two-column sign and a
two-column DOOR. Only the sign receives PictureFrame. The adjoining Trade and
Colosseum doors, and the separate Time Capsule door, receive the existing open
sliding-door frame model. Their complete live source artwork remains on its
original zero-height footprint. The frame has a real empty central aperture,
not an opaque picture backing. The narrow rear strip above the PC uses the
existing clinical wall component, matching its adjacent wall.

## Exact ownership and conservative retained art

Six new source fingerprint groups extend the existing eight Cable Club groups.
Their guards include map-space anchor, tileset, block, source phase, every tile
identity, neighboring cells and the unowned openings. Each map adds:

- One four-cell rear wall at source `(4..6, 0..2)`
- Two seven-cell console silhouettes: column `13` or `21` at row `2`, followed by
  both columns `12..14` or `20..22` at rows `3..6`
- Three four-cell open doorway frames at `(10..12,0..2)`, `(18..20,0..2)` and
  `(26..28,4..6)`

The complete PC still owns six source cells. The two framed signs own four each,
with the full original sign-plus-door source drawing retained as their guard.
Only eleven explicit map names qualify; arbitrary names ending in `2FBeta` do
not. Missing ground, source edits, wrong phase, clipping, custom profile overlap
or prior model ownership retains the source renderer. Exact unchanged bundled
sign/partition profiles yield to their replacements; edited profiles do not.

The main `Pokecenter2F` has six warps, including two mobile warp footprints in
source columns `12..14` and `20..22`, rows `0..2`. The beta maps share the visual
layout but have only their exit warp. Neither shared source art nor coarse WALL
collision licenses filling those mobile openings. All eight cells in those two
footprints remain native in every room, including six striped-wall cells and
the two upper cable-return cells. Each console begins below that boundary.
The apparent floor at `(12,2)` and `(20,2)` is native negative space inside a
coarse WALL quadrant; it stays untouched and is never counted as an object or
repainted as invented floor. Existing actor footprints stay clear. Collision,
warp metadata, actors and controller behavior are not edited.

## Rebuild and checks

```sh
python3 tools/build-cable-backwalls.py target/cable-backwalls --runtime-only
blender -b --threads 2 --python tools/build-cable-backwalls.py -- target/cable-backwalls
python3 tools/check-cable-backwalls.py
cargo test -p crystal-voxel-view cable_
cargo test -p crystal-voxel-view upstairs_
```

The runtime checker verifies deterministic regeneration, finite geometry,
outward winding, positive closed volume and each part's exclusion from the
source-negative corner. It also checks every triangle of the reused door frame
against the central aperture and validates the editable source's hashes.
Focused Rust regressions cover beta PC/sign reuse, original door role, all
source-identity fields, complete drawing guards, unknown maps, source clipping,
custom ownership, atomic append, live source door UVs and unchanged footing.
The existing Cable Club tests continue to cover signed map anchors and scrolled
or padded frames.

Read-only external-pack verification checks all 154 guarded groups across eleven
maps in each local pack. New groups cover 44 rear-wall cells, 154 terminal cells
and 132 DOOR cells per pack. Every claimed non-floor cell has its expected WALL
or DOOR collision role; non-door geometry has no warp/actor intersection. Integrated Rust tests and main/beta native captures now pass, along with an
ordinary beta-room stair transition through the production controller. The
source counts are ownership evidence; further link-service interactions and
mobile-entry routes still require their own controller checks.
