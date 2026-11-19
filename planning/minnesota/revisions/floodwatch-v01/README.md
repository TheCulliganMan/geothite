# Moorhead garden, Prairie Run and rescue routes

## The porch-swap garden

The two brown rectangles south of Moorhead’s Mart were the left roof half of Crystal’s Route 34 gatehouse, placed twice by the original town layout to represent seedling trays. The gardener and the “PORCH-SWAP STARTS” notice establish what belongs here. The plot occupies the same two metatiles at (11,8) and (12,8). The notice belongs to the northwest walking square at (22,16) and can be read from (21,16). [Original placement](../../towns/MnMoorhead.layout.json) · [Resident, notice and approach](../../towns/MnMoorhead.json) · [Garden art](../../../../crates/crystal-assets/src/northstar_porch_garden.rs)

The replacement has a native sign, spare pots and four young plants on a timber table. Its 19 original 8×8 tiles use four shades apiece. Comparing morning, day and night preserved all 128 original Johto Modern terrain blocks. A sweep across all 108 Minnesota exteriors found nine remaining uses of the same left roof in each Lake Observations edition. Each has its matching right half on a gatehouse or lighthouse. The preservation check now compares the native tile artwork across tilesets and follows eastward map connections when a roof crosses an edge. [Native pixel recipe](porch-garden.json) · [Regional roof check](check_garden_roofs.py) · [Preservation checks](check-preservation.py)

## An optional prairie return

From Detroit Lakes, head west past the seed plot notice on Route 16. Finch watches the flowers in the east. June stands on the hay lane to the south. Lyle fishes from the pier north of the road bridge. Each starts a regular trainer battle on sight and the player can visit them in any order. In Moorhead, read the marker on the Red River levee. The helper between the Center and Mart names each trainer you still need and pays one Nugget when you have the Lake Badge, all three wins and the reading. A full Pack leaves the prize waiting. The marker also leads to the original, independent Full Heal from the clinic volunteer.

Crystal’s Route 25 gives an item after several trainer battles. Polished Crystal retains the trainer challenge and prize. On the Minnesota road each fight has an established landmark and the circuit is optional in both directions. Lyle opens with Jiglet, a metal fishing lure, and Finch has three distinct bird partners. Levels and party sizes follow the edition’s original teams. [Crystal Route 25](https://github.com/pret/pokecrystal/blob/master/maps/Route25.asm) · [Polished Crystal Route 25](https://github.com/Rangi42/polishedcrystal/blob/master/maps/Route25.asm) · [Quest and text](../../story.json) · [Trainer and reward scripts](../../../../crates/crystal-assets/src/northstar_prairie_run.rs)

## Minneapolis and the Iron Range

The dispatch-office scene now takes the open aisle around Reed from either entry square. To reach Ada, take the stairs north from Operations, cross Records to its stairs, and take Maintenance pad A followed by Transfer pad B. Once Ada gives the Card Key, pad H leads back toward the Return shutter. The second shutter opens the route through Operations to the relay. The independent check traces the path out with Ellis, the evidence desk and the street to his home. [ICE staging](../../../../crates/crystal-assets/src/northstar_ice_handoff.rs) · [ICE route check](check-ice.py)

Before this revision the mine could mark Finn’s meter as delivered simply by hiding him when the player entered the drill room. Showing him could undo a real delivery. The Grand Rapids technician now owns the delivery receipt. The room checks the receipt to determine whether Finn is present. Both sides of the motor lead the player safely back to the crew after shutdown. A previously incorrect receipt in an older save cannot be distinguished from a genuine delivery by inspecting that flag alone. [Drill room](../../../../crates/crystal-assets/src/northstar_mine_handoff.rs) · [Mine and depot check](check-mine.py)

## Build and verify

Each receipt specifies the exact external parent, fresh output path and expected file digest for its edition. Set `parent` and `child` to the two paths in that receipt. The builder validates the result after reading it back from disk. The checks use compiled pack data, the game’s own map setup and trainer systems, and static renders.

```sh
src=planning/minnesota/revisions/floodwatch-v01
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo run --locked -j1 -p crystal-assets --example pack_floodwatch -- "$parent" "$child"
uv run --with cbor2 --with pillow python "$src/check-moorhead.py" "$parent" "$child"
uv run --with cbor2 --with pillow python "$src/check-ice.py" "$child"
uv run --with cbor2 --with pillow python "$src/check-mine.py" "$child"
uv run --with cbor2 --with pillow python "$src/check-preservation.py" "$parent" "$child"
MN_FLOODWATCH_PACK="$PWD/$child" cargo test --locked -j1 -p crystal-assets --lib floodwatch -- --ignored --test-threads=1
```

The Field build covers 743 maps and 876 trainer teams. The local Mothlight build covers 745 maps and 877 teams. Independent parent comparisons found seven changed rooms, two revised parties and one new reward flag in each. Second builds reproduced both archives byte for byte. See the [Field receipt](field-build.json) and [Mothlight receipt](mothlight-build.json).

The route check includes two-way travel from Brainerd through Detroit Lakes to Moorhead, all three trainer approaches with the others undefeated, both clinic exits and a return to the public road. The reward check varies the badge, the reading, individual wins, each receipt and item-pocket capacity. The mine check follows both valve orders, both sides of the motor and the walk to the depot and Route 10. The separate [campaign ledger](../league-and-fair-v01/coverage.md) tracks the stages beyond this revision, including the Route 19 observation loop, the Duluth-to-intake journey and the full five-battle League curve.
