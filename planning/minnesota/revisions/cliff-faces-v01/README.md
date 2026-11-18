# Minnesota waterfronts and cliff faces

The public lake pier in Duluth, the lighthouse shore in Grand Marais and the Cedar Lake fishing bank now let a player walk up to the water, cast a rod, launch Surf and land on the same spot. Water itself stops a player on foot. Duluth’s pier and fishing floats use native timber, with the fishers and pickup on the boards. Nearby street and waterfront signs give directions that match the revised maps.

On Route 16 the Red River and drainage bridges carry walkers east–west and swimmers north–south. The smaller bridge crosses a map boundary. At St. Cloud the public river walk meets water below an inland granite overlook with working stairs. The Route 10 cliff meets the trail at its foot, and the Granite Cave mouth on Route 20 has a continuous sidewall. Cedar Lake’s low shore stays open below its inland birch ridge.

Crystal’s Route 45 was the reference for separating long rock faces from a low lip that a player can jump. Polished Crystal’s Route 45 bridge informed how to seat a deck between two banks. Minnesota uses the native art in its own supplied pack. [Crystal Route 45](https://github.com/pret/pokecrystal/blob/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/maps/Route45.blk) · [Polished Crystal Route 45](https://github.com/Rangi42/polishedcrystal/blob/f8e67fbd53ad7a99870d8ab87fd73612e6c6a679/screenshots/route-45.png)

## Build and check

The Field and Mothlight recipes name their respective outdoor-contours v04 parents and expected SHA-256 hashes. Cedar Lake has one bank tile with a different native index on those two parents. Both end up with the same open water there. Each output needs a fresh filename.

```sh
src=planning/minnesota/revisions/cliff-faces-v01
parent=content-packs/minnesota-outdoor-contours-v04.crystalpack
output=content-packs/minnesota-cliff-faces-v01.crystalpack
CARGO_INCREMENTAL=0 nice -n 14 cargo build --locked -j1 -p crystal-assets --example pack_outdoor_contours
target/debug/examples/pack_outdoor_contours --cliffs "$parent" "$src" "$output"
uv run --with cbor2 --with pillow python "$src/review.py" "$parent" "$output"
```

Both builds exercise 81 named shoreline positions with the game's walking, fishing, Surf and dismount functions. They also check the Route 16 bridge crossings, stairs, inherited ledges and previously opened piers. The independent comparison covers all 473 placements on 37 exteriors, all 14 dialogue replacements, the original actor and warp positions, and all 9,468 encounter squares in morning, daytime and night palettes. The day/night renderer regression for Duluth and Boundary Waters is `boundary_waters_and_duluth_docks_keep_native_water_on_three_open_sides`; set `CRYSTAL_SHINER_WATER_TEST_PACK` to this output for that ignored test.
