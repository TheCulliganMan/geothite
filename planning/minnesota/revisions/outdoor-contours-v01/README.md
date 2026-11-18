# Minnesota shorelines, piers and outdoor ledges

Brainerd’s public fishing pier is two walking tiles wide. It starts at the street, leaves a passing space beside the fisher, and ends over the lake. The open tip and sides lead straight to water. With a rod, a player can fish there before earning Surf. With Surf, the player can leave the boards and land on them again. An unmounted walker stops at the water.

The creek dock on Driftless Trail and the Shiner Lake fishing dock now extend from land over water on both sides. At Shiner Lake, the open-water glints and shallow-water ripples use four frames from the lake’s own tile drawings. The deck stays still. Bemidji and Detroit Lakes have open fishing piers. Two more extend from the Mille Lacs shore. The River Bluffs, Needle Lake, Leech Lake, Northern Lakes, Red River and Border Lake piers also have open sides. Duluth’s high quay leads to a signed, low cove where the player can launch south. Nine signs, fishing conversations and a hidden-item clue now describe those shores. Anglers and pickups keep their positions.

| Place | Terrain and traversal |
|---|---|
| Driftless Trail | The limestone stair connects the Potion shelf with the fossil bench. The creek dock extends west from the path to the farmhouse. |
| River Valley and River Bluffs | The rock line follows the bank. The high lookout is closed to Surf along its cliff face. The fishing pier farther south has an open edge. |
| Iron Range | The mine entrance sits in its bench and the western stairs meet their terraces. Surf cannot climb the four exposed faces of the rock point. The adjacent low landing remains open. |
| North Shore | Below both falls, the cliff face runs down to the pool. The low east bank of each pool still permits landing. |
| Gunflint Trail and Wind Tor | Raven Rock keeps its ice approach and Rock Smash return route. Wind Tor keeps its southward drop and its eastern way back up. Both retain their original mountain grass. |
| Lake Country | The high meadow faces the lake across a continuous rock wall. The low shore south of the meadow and the timber dock on the opposite bank remain accessible. |

## Design references

Crystal’s Routes 45 and 46 use short downhill lips between longer cliff faces. They informed the placement of Minnesota’s jump points and uphill stairs. Polished Crystal’s Route 45 shows both ends of a bridge seated against cut banks, a reference for making the Minnesota landings meet their surfaces. Pokémon Orange’s Butwal Island West changes the deck and the collision of five blocks together while keeping the landings recognizable from either height. The Minnesota shore layouts and mine entrance were authored here. The wooden boards reuse the pier artwork in the supplied pack. [Crystal Route 45](https://github.com/pret/pokecrystal/blob/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/maps/Route45.blk) · [Crystal Route 46](https://github.com/pret/pokecrystal/blob/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/maps/Route46.blk) · [Polished Crystal Route 45](https://github.com/Rangi42/polishedcrystal/blob/f8e67fbd53ad7a99870d8ab87fd73612e6c6a679/screenshots/route-45.png) · [Pokémon Orange bridge](https://github.com/PiaCarrot/pokeorange/blob/5b4a95ec2d4638c7b8f125b6e1644390d52d4a5b/maps/ButwalIslandWest.asm#L13-L45)

## Build and verification

The workspace with the local lodge selects the Mothlight descendant. The committed regional selection follows Field Review v02, which includes the previous five improvements. The two verified input/output pairs are [field-build.json](field-build.json) and [mothlight-build.json](mothlight-build.json). Each is a fresh review identity. Existing player saves stay with their original pack.

The three verified parent builds each have 13 open fishing piers, 572 revised metatiles across 24 maps, and all 3,000 original encounter squares in those maps.

To reproduce the field build in an ignored, fresh output location, run these commands from the repository root.

```sh
source_dir=planning/minnesota/revisions/outdoor-contours-v01
parent_pack=content-packs/minnesota-field-review-v02.crystalpack
output_pack=target/outdoor-contours-rebuilt.crystalpack
CARGO_INCREMENTAL=0 cargo run --locked -j 1 -p crystal-assets --example pack_outdoor_contours -- "$parent_pack" "$source_dir" "$output_pack"
uv run --with cbor2 --with pillow python "$source_dir/review.py" "$parent_pack" "$output_pack"
uv run --with cbor2 --with pillow python "$source_dir/check-north.py" "$parent_pack" "$output_pack"
uv run --with cbor2 --with pillow python "$source_dir/check-brainerd-pier.py" "$parent_pack" "$output_pack"
```

The independent comparison reads the compiled maps, their block mirrors, actors, warps, dock dialogue, encounter tables, Pokémon, moves and audio. For every grass square in a changed map it compares its position and artwork in morning, daytime and night palettes. The pack builder exercises the normal movement, fishing and Surf functions on the installed dock and cliff coordinates. All checks run without gameplay inputs.

The native animation regression loads the actual pack, decodes the game’s frames and checks moving water next to stationary wood. For the renderer tests, set `CRYSTAL_SHINER_WATER_TEST_PACK` to the absolute revision pack path and `CRYSTAL_HEADWATERS_TEST_PACK` to the original Headwaters fixture. Run `cargo test --locked -j 1 -p crystal-bevy --lib headwaters_art:: -- --ignored --test-threads 1`. Local comparison images are in `target/outdoor-contours-20261118/`.
