# Minnesota cave walls

Crystal's Union Cave, Slowpoke Well and Diglett's Cave use low-contrast
stipple beyond their cliff faces. The faces turn with the floor and leave
room for distinct boulder clusters. Ice Path 1F uses a quieter outer field
than the boulder-puzzle floor below. See the [cave reference atlas](../crystal-reference.html)
for the original maps.

All 19 Minnesota cave, mine and ice interiors now have quieter backgrounds.
The blocked field uses each native tileset's stipple art with solid collision.
The 168 contour edits turn long boulder rows into continuous cliff faces,
with boulders kept at selected bends and outcrops. Mines keep the rails,
stair mouths and workings. Ice keeps its slide lanes and low ice faces.
The [replacement floor plans](../crystal-replacement-v02/README.md) supply
the routes, elevations, signs, switches and warp positions.

## Rebuild and review

The [build recipe](build.json) identifies the exact local parent, edge sources
and output. The builder requires a fresh, ignored output path.

```sh
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets --features pack-authoring \
  --bin pack_northstar_cave_wall_finish --bin verify_crystalpack
target/debug/pack_northstar_cave_wall_finish \
  content-packs/minnesota-cave-replacement-v02.crystalpack \
  planning/minnesota/caves/crystal-wall-finish-v03 \
  content-packs/minnesota-cave-wall-finish-v03.crystalpack
target/debug/verify_crystalpack \
  content-packs/minnesota-cave-wall-finish-v03.crystalpack --audio --desktop
uv run --with cbor2 --with pillow python planning/minnesota/tools/review_current_caves.py \
  --revision cave_wall_finish_revision --output target/cave-wall-study/source-review
```

The source review composes the original floor plans with the wall finish,
compares the rows with the compiled maps, and resolves all interior warp
destinations. The wall finish has its own local save identity.
