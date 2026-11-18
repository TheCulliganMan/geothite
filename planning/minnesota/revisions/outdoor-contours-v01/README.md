# Minnesota outdoor contours

The six revised routes use the south face of a cliff to show the drop, a break in that face for a stair or hop, and a recognizable surface at the landing. Separate local builds carry the same terrain on the Field Review and Mothlight Lodge branches. The public release at the time of this review is Lakeside.

| Route | What the terrain now tells the player |
|---|---|
| 1 · Driftless Trail | The Potion occupies the high western shelf. Its short stair is visible from the fossil bench. The eastern wall follows a single elevation. Both cave mouths, the fossil and the earlier jump retain their approaches. |
| 5 · River Bluffs North | Both southbound paths continue over short, visible drops. Each hill has an uphill stair. The southern hidden item has a rock face and a standing space immediately below it. |
| 9 · Iron Range West | A timber entrance is set into the front of the mine bench. Its snow-covered approach lies below the rock shelf. The upper flight has a single landing, and the encounter grass looks like dry sedge growing in rock. |
| 10 · Iron Range Descent | The western stairs meet continuous ore terraces. Dry sedge follows the contours across their exposed tops. Grass in the snowy valley retains its winter appearance. The eastern meadow drops to the lakeshore path and has a walkable return around its end. |
| 12 · Gunflint Trail | The Raven Rock brow runs continuously to the lookout. Ice reaches the lookout. Breaking the rock opens the dry climb to the summit and its return ledge. The shelter remains accessible. |
| 13 · League Approach | Wind Tor's granite and vegetation continue to its low front lip. The eastern stair leads back uphill. Farther along, a single cliff face runs across the map seam and holds the Cedar Shelter mouth. |

## Design references

Crystal’s Routes 45 and 46 use short downhill lips between longer cliff faces. They informed the placement of Minnesota’s jump points and uphill stairs. Polished Crystal’s Route 45 shows both ends of a bridge seated against cut banks, a reference for making the Minnesota landings meet their surfaces. Pokémon Orange’s Butwal Island West changes the deck and the collision of five blocks together while keeping the landings recognizable from either height. The Minnesota layouts and sedge and mine artwork were authored here. [Crystal Route 45](https://github.com/pret/pokecrystal/blob/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/maps/Route45.blk) · [Crystal Route 46](https://github.com/pret/pokecrystal/blob/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/maps/Route46.blk) · [Polished Crystal Route 45](https://github.com/Rangi42/polishedcrystal/blob/f8e67fbd53ad7a99870d8ab87fd73612e6c6a679/screenshots/route-45.png) · [Pokémon Orange bridge](https://github.com/PiaCarrot/pokeorange/blob/5b4a95ec2d4638c7b8f125b6e1644390d52d4a5b/maps/ButwalIslandWest.asm#L13-L45)

## Build and verification

The workspace with the local lodge selects the Mothlight descendant. The committed regional selection follows Field Review v02, which includes the previous five improvements. The two verified input/output pairs are [field-build.json](field-build.json) and [mothlight-build.json](mothlight-build.json). Each is a fresh review identity. Existing player saves stay with their original pack.

To reproduce the field build in an ignored, fresh output location, run these commands from the repository root.

```sh
source_dir=planning/minnesota/revisions/outdoor-contours-v01
parent_pack=content-packs/minnesota-field-review-v02.crystalpack
output_pack=target/outdoor-contours-rebuilt.crystalpack
CARGO_INCREMENTAL=0 cargo run --locked -j 1 -p crystal-assets --example pack_outdoor_contours -- "$parent_pack" "$source_dir" "$output_pack"
uv run --with cbor2 --with pillow python "$source_dir/review.py" "$parent_pack" "$output_pack"
uv run --with cbor2 --with pillow python "$source_dir/check-north.py" "$parent_pack" "$output_pack"
```

All three tested parents (Field Review, Mothlight Lodge and Lakeside) produced the same 291 authored metatile replacements across ten map chunks. The builder checks the six southward hops with the core movement functions and checks the walking approaches. The independent audit checks each compiled map and its block mirror, every event and actor, the unchanged encounter data, and the exact Cut results for the new grass variants. Different parents can already have different hidden-item art in the same tileset. The builder appends new art after the installed bank and resolves the design's IDs at build time.

Route 5's uphill walks take 66 and 90 steps. On eastern Route 10 the item meadow to shore takes four actions, and walking back takes eleven. Wind Tor's return takes eighteen steps. These are terrain measurements with occupied map positions. The visual review used whole exteriors and nine pairs of headless native 2D captures from identical camera positions at noon. No gameplay inputs were sent. The local comparison gallery is in `target/outdoor-contours-20261118/gallery.html`.
