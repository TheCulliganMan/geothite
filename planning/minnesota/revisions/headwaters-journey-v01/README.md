# The headwaters journey

At Lake Itasca, a marked staff stands against the west bank just above the timber footbridge. Face it from the west, or read the nearby posted notice. Both give the same dated reading. A player who goes to Itasca first can take that reading straight to the Bemidji officer. A player who visits Brainerd first recognizes the ranger’s date. The officer can sign either account before the Pine Badge. The Iron Range foreman checks for both the signature and the badge.

The bridge carries two walking lanes east and west. A Pokémon using Surf can pass underneath in either direction along the brook. The water has a continuous brown stone edge where it narrows below the lake. The distance check starts separately at Bemidji Gym, at the field officer and at the gatehouse. It follows the two gatehouse carpets, the west-bank notice and gauge, the return to the officer and the road to the mine. There are 16 combinations of ranger, notebook, badge and visit order. [Itasca geometry and quest check](check-itasca.py) · [Native bridge and gauge](../../../../crates/crystal-assets/src/northstar_itasca_crossing.rs)

| Elsewhere along the journey | What the player finds |
| --- | --- |
| St. Paul and St. Cloud | Mina recognizes a prior ranger visit. After the player helps Sera, she walks around the trees to the station forecourt. The player can go inside, radio Nell and bring the reply back to the person waiting outside. [Mina and Sera](central.md) |
| Brainerd and the fishing lakes | The ranger stands on the bank beside the pier named by Sivren. Route 7’s birder points to the two-way stone steps. Leech Lake and Mille Lacs have walkable fishing platforms in both editions. [Lake road and two piers](leech.md) |
| Route 20 | Signs at opposite approaches tell travelers that the outer cave mouths connect. The middle mouth leads to a two-floor ice cavern. A traveler can use the surface road, cross underground, or stop to explore. [Granite Narrows](passage.md) |
| Glacier Line | A first stop registers itself. Visit a second station to unlock the free ride back. Five public roads connect to the halls and all 20 directed rides return to a walkable platform. [Station visits and menus](rail.md) |

## Rebuild and check an edition

Use either Station Return parent specified in the [Field receipt](field-build.json) or [Mothlight receipt](mothlight-build.json). Each receipt binds its own parent and result to their full SHA-256 hashes. The compiler requires a fresh output path. The verification scripts read compiled game data and need Python with cbor2 and Pillow.

```sh
src=planning/minnesota/revisions/headwaters-journey-v01
edition=field
parent=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["parent_pack"])' "$src/$edition-build.json")
mkdir -p target/headwaters-review
child=target/headwaters-review/$edition.crystalpack
CARGO_INCREMENTAL=0 cargo run --locked -j1 -p crystal-assets --example pack_headwaters_journey -- "$parent" "$child"
python3 "$src/check-preservation.py" "$parent" "$child"
python3 "$src/check-itasca.py" "$child"
python3 "$src/check-central.py" "$child"
python3 "$src/check-leech.py" "$child" --parent "$parent"
python3 "$src/check-passage.py" "$child"
python3 "$src/check-rail.py" "$child"
MN_HEADWATERS_PACK="$PWD/$child" RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0 cargo test --locked -j1 -p crystal-assets --lib headwaters_reading_and_bridge -- --ignored --test-threads=1
```

The preservation check compares every map, the original headwaters terrain in morning, day and night palettes, animations, sound and the rest of the pack with the corresponding parent. Both editions share the new gauge and bridge. Only Field needed new timber on the fishing platforms. The stock bridge planks live at different positions in the two editions’ tilesets. The compiler matches the original artwork and its walking permissions before placing it.

Native camera stills document the appearance. Isolated movement and interaction checks establish what an individual step or button press does. The route traces measure travel on the compiled map and the scene checks follow its branches. A continuous player journey and a public release are outside this review. [Campaign coverage](../league-and-fair-v01/coverage.md) · [Safari admission and the unpaid ranger](safari.md)
