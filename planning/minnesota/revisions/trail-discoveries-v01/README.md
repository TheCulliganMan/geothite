# Minnesota trail discoveries

In the Bemidji Northwoods, go through the little forest gate and ask the road worker why the ditch is dry. The pipe lies east of the worker. There is a split hoofprint on the pool’s east bank, and three fallen trunks lie where the road bends. Bring those observations back in any order. The worker can point out whatever is still missing.

The windthrow is holding the water away from the houses. Leave it where it fell. The separate log across the north trail can be rolled aside from either end. Bunjant appears beside the trunk as it moves. The marked timber and the old tally post are inside the camp. Return to the worker with the account to find two independent, voluntary encounters. Both say explicitly that accepting starts a battle. [Northwoods paths, states and return visits](tall-tale.md)

The six landmarks use 32 original Game Boy Color tiles. The scrolling map camera uses at most six of the eight available four-shade background palettes. The water retains its original animation and Surf footprint, and the low brown shore follows the existing bays. The open and closed trail use different movement permissions. The genuine sapling beside the marked wood still responds to Headbutt. Crystal’s Ilex Forest keeps its Farfetch’d at the right location on reentry. Here the map’s own callbacks restore the trunk and keep Bunjant at the right place for its later encounter. [Original drawings](tall-tale-art.json) · [Crystal’s Ilex Forest](https://github.com/pret/pokecrystal/blob/master/maps/IlexForest.asm) · [Polished Crystal’s Ilex Forest](https://github.com/Rangi42/polishedcrystal/blob/master/maps/IlexForest.asm)

| Stop | What changed |
| --- | --- |
| Route 2, Route 8 and Cedar Ridge | The five prairie noticeboards read from their printed faces. A birch walker points to the cones that shelter a Miracle Seed, and the Hyper Potion is marked by a winter cairn. The review traced all 57 hidden items from an entrance to a facing square and back. [Clue review](clues.md) |
| Boundary Waters | An unpaid visitor at Reed Landing gets directions to basecamp and the shelter. Granite Landing gives the bearings to its stairs and return ranger. Dorothy runs her outdoor shop, and a medic tends to partners in the cabin. One 493-step Safari itinerary fits all four camps, three lookouts and the cabin in the 500-step permit. The shelter and rescue remain free. [Safari maps and admission](safari.md) |
| Route 20 and St. Cloud | Kite’s Chorip can attack in its teammate’s rain. Reed’s Champion rematch Greytag has a strong, reliable Normal attack alongside Quick Attack. The battle audit follows installed teams from the first gyms through the League and rematches. [Battle course](battles.md) |
| Optional rare encounters | At all ten fixed wild battles, the player sees a battle question before choosing. A refusal preserves the existing encounter. At sites where a retired species was replaced, the question names the same Minnesota species that appears on screen. [Species, choices and returns](rare.md) |

## Rebuild and verify

The [Field receipt](field-build.json) uses the shared Headwaters Journey edition. The [Intro receipt](intro-build.json) uses the locally selected Northstar opening, a descendant of Mothlight Headwaters. Use each receipt with its own parent. The external packs have separate save identities.

```sh
src=planning/minnesota/revisions/trail-discoveries-v01
edition=field
parent=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["parent_pack"])' "$src/$edition-build.json")
mkdir -p target/discovery-review
child=target/discovery-review/$edition.crystalpack
CARGO_INCREMENTAL=0 cargo run --locked -j1 -p crystal-assets --example pack_trail_discoveries -- "$parent" "$child"
python3 "$src/check-preservation.py" "$parent" "$child"
python3 "$src/check-clues.py" "$child" --parent "$parent"
python3 "$src/check-safari.py" "$child" --parent "$parent"
python3 "$src/check-battles.py" "$child" --parent "$parent"
python3 "$src/check-tall-tale.py" "$child"
python3 "$src/check-rare.py" "$child" --parent "$parent"
MN_TRAIL_DISCOVERIES_PACK="$PWD/$child" RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0 cargo test --locked -j1 -p crystal-assets --lib tall_tale_trunk_survives -- --ignored --test-threads=1
```

Run with Python, cbor2 and Pillow. The native test applies the map-entry callback, resolves the A-button interactions, checks travel across the trunk in each state and applies the object commands that stage Bunjant. The other checks use compiled geometry, scripts, encounter records and native artwork. The [campaign ledger](../league-and-fair-v01/coverage.md) records the wider coverage. The [State Fair audit](fair.md) and [resident return audit](return-visits.md) describe confirmed follow-up work in the two editions.
