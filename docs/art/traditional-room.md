# Traditional-room scenery

Original papercraft wood and paper geometry replaces complete source drawings
in WiseTriosRoom, Route39Barn and DanceTheater. The generator imports no game
pixels, textures or meshes. Three compressed runtime prototypes use the
existing bounded `model_storage` decoder; every closed component is separately
editable in `traditional-room-scenery.blend` through the source-chunk manifest.

## Source roles and honest counts

- Wise Trio: six separate horizontal divider panels, 40 source cells. Block 37
  contains a four-cell-wide north panel and a separate two-cell-wide southeast
  return. Its southwest floor opening stays clear. Block 38 has only the lower
  right return. The west stair/portal and east warp carpet are not claimed
- Barn: three four-cell-wide stall rails, 24 source cells. The central passage
  and north floor halves of block 26 remain clear
- The two maps share the exact 40/41 source drawing and therefore reuse one
  8px slatted rail bay. There are 20 Wise Trio and 12 barn bay instances
- Dance Theater: one atomic 24×10 guard consumes all 240 candidate cells. Its
  rear 48 cells become a twelve-bay paper screen and its 24 front fascia cells
  become a joined stage front. The 168 intervening source-50 cells are the
  continuous stage floor and receive no authored-object credit
- Total new object coverage: 136 cells (40 + 24 + 48 + 24). Stage surface
  replacement: 168 cells, deliberately separate from that object denominator
- Continuous zero-height tatami: 176 block-04 FLOOR cells in KurtsHouse and
  DanceTheater. The 44 source-46/56 edge cells now receive the same finish as
  the other 132. These are floor finishes, never cushions or new volumes

The original theater stage already supports actors at 8px on all 240 cells.
The replacement keeps exactly that top datum, including the four native FLOOR
access cells at source columns 2, 3, 20 and 21 in row 9. Their 5f drawing becomes
horizontal tread relief on the front, while the other 20 fascia cells retain
vertical trim. These are visual access cues, not new ramps or collision edits.
The rear screen is 16px above that datum, wholly inside the two source WALL rows.
The interactive fancy panel below the stage is outside this package.

## Matching and fallback

Bindings require the exact map, world anchor, tileset, metatile, source-art
identity, subtile phase, complete guard and a real zero-height ground sample.
Every source guard cell is checked, including unowned floor openings. A custom
profile touching any guard cell rejects the entire relevant placement. Clipped,
rearranged, altered or unsupported drawings retain the established renderer.
The appender repeats the complete guard before writing and rejects overlapping
claims without emitting partial geometry. Original footing, actors, collision,
warps, scripts, cutaway policy and NPC behavior are untouched.

Tatami finishing requires a complete native block 04 at the correct world
phase, all 16 source identities, zero-height flat shape, no custom profile and
an exact own-cell atlas quad. Wrong UV corners, borrowed UVs, elevated quads,
changed art and partial blocks keep source art. The old cushion grouping is
explicitly disabled on these two maps, even if a future scene supplies flat
source 50; the global missing-50 requirement is never widened.

## Rebuild and validation

Run the generator with ordinary Python and `--runtime-only`, or run it through
headless Blender for the editable source and front/rear previews. The runtime
bytes must match across both paths. Use `check-traditional-room-models.py` for
closed-part topology, consistent winding, positive volume, finite bounds,
opaque materials and unit normals. `check-traditional-room-source.py --pack
PATH --rustc PATH` checks the external ignored pack in memory and compiles the
exact source resolver with temporary fixtures; it needs no Cargo build.

The source checker tests every guard cell against changed art, block, atlas,
x/y phase and custom reservation, plus crop, translated viewport, bad origin,
missing ground and unrelated maps. It verifies native source collision roles,
warp exclusion and fixture separation from actor footprints.

After integration, run the voxel regression suite and native views of all
three maps. Inspect front and rear silhouettes, joined rail ends, both theater
access strips, screen height, actor footing at 8px and zero-height tatami.
Exercise both Wise Trio scripts/warp approaches, barn interaction and the
Kimono Girl/fancy-panel approaches. Staging checks do not certify native UI.
