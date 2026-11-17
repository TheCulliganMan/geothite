# Glacier Line rear-edge depots

The five public stops share a simple arrangement. Streets approach the south
doors and railway space runs behind the buildings at the north edge. The
exterior rail blocks have wall collision in all four quadrants. Boarding stays
inside the halls on the existing Glacier Line service.

| Stop | Door | Return porch | Railway corridor |
| --- | --- | --- | --- |
| Winona | (11, 5) | (11, 6) | West edge to a buffer before the river |
| Detroit Lakes | (11, 5) | (11, 6) | West edge to a buffer before the north gate |
| St. Cloud | (37, 5) | (37, 6) | East edge, clear of the north checkpoint |
| St. Paul Union Depot | (8, 7) | (8, 8) | West edge behind the northwest depot |
| Staples Rail Junction | (13, 7) | (13, 8) | Existing rear tracks extended to both edges |

Door and porch coordinates use 16-pixel gameplay tiles. The St. Cloud forecourt
includes the existing HM Fly guide. St. Paul uses the former riverside depot
site as a landscaped square and its civic guide and wayfinding board point
northwest. The relocated doors retain their warp indices and room targets.

[stations.json](stations.json) holds the native map rows and exact event moves.
[city-access.json](city-access.json) binds each public road fixture to its new
door and return porch. The railway art derives from the Goldenrod track block
in the supplied parent pack. The source assets and 738 other maps remain
byte-identical, including the five station interiors.

## Build and review

Run from the repository root with the selected red-canoe parent available
locally. Use a fresh output path when repeating the build.

```sh
cargo build --locked -p crystal-assets --bin pack_glacier_rear_rail
target/debug/pack_glacier_rear_rail \
  content-packs/minnesota-region-ecology-red-canoe-v01.crystalpack \
  content-packs/minnesota-glacier-rear-rail-v01.crystalpack
target/debug/verify_crystalpack \
  content-packs/minnesota-glacier-rear-rail-v01.crystalpack --audio --desktop
```

The native road regression uses the new city access contract, the existing
Glacier service contract and the compiled pack. It enters all five halls from
their road approaches, checks station visits, saves and continues inside, then
exits to each matching porch and walks back. Winona uses its existing qualified
warden crossing. Road-start positions and prerequisites are test fixtures.

```sh
CRYSTAL_INTERIOR_PACK="$PWD/content-packs/minnesota-glacier-rear-rail-v01.crystalpack" \
CRYSTAL_GLACIER_SOURCE="$PWD/planning/minnesota/transit/glacier-current-v01/service.json" \
CRYSTAL_GLACIER_CITY_ACCESS="$PWD/planning/minnesota/transit/glacier-rear-rail-v01/city-access.json" \
RUST_MIN_STACK=67108864 \
cargo test --locked -p crystal-bevy glacier_line_public_roads -- --ignored --test-threads=1
```

The existing ride regression also passed all 20 directed journeys, and all
602 audio programs matched their PCM hashes, frame counts and loop ranges.

[Full maps, native porch captures, and source references](../../renders/glacier-rear-rail-v01/)
are retained as ignored review images. The track edges depict the off-map rail
corridor. The existing station service supplies the actual intercity travel.
