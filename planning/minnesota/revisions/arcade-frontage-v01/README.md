# Minneapolis Game Corner and Cedar Inlet

The Game Corner has a three-bay brick frontage and a yellow 7–7–7 marquee. Read the marquee from either sidewalk space beneath it. Walk through the door beside the reels and talk to the attendant near the entrance for a Coin Case and 20 coins. Check it any time with **Pack → Key Items → Coin Case → Use**.

| Inside the arcade | What the player can do |
| --- | --- |
| Back-left cashier | Buy 50 coins for 1,000 or 500 for 10,000. The case holds 9,999 coins. |
| Left machine banks | Bet one to three coins and stop the three slot reels. |
| Right machines | Bet three coins on Card Flip and collect any payout. |
| Back-right item counter | Buy a Quick Claw for 300, Rare Candy for 500 or PP Up for 1,000. |
| Adjacent TM counter | Buy Thunder, Blizzard or Fire Blast for 5,500 apiece. Each menu entry shows the full move name. |

All six purchases show the prize and price together on one confirmation page. All three counters and both games share the same floor. The room uses the native Goldenrod Game Corner layout. Its greeter and patron give directions that match the aisles. The Quick Claw replaces a Moon Stone that had no eligible partner in the active regional roster. Thunder is compatible with 22 Minnesota species, Blizzard with 23 and Fire Blast with 31. [Crystal’s Goldenrod Game Corner](https://github.com/pret/pokecrystal/blob/master/maps/GoldenrodGameCorner.asm)

## Reaching the services

- At Cedar Inlet, a freestanding sign marks the west bridgehead. There is room to pass on each side. The bridge and shore trails still meet there.
- The South Arrival Desk has a visitor menu. Directions to the Game Corner use the inn and marquee as landmarks. The South Market is two doors east. The clinic route passes the first river bridge and turns at the second one.
- The Mill Workers Inn has a single bed along the east wall. Speak to the host or face either half of the bed. Accepting restores HP, PP and status across the party, including fainted partners. The inn charges nothing and preserves the player's registered clinic.

An inventory of actors and readable scenery covered 357 Minnesota maps in the parent. Direct checks focused on the services above. Two further gaps are the Route 11 pie shop, where the sign advertises fresh pies and the staff only talk, and the Capitol Library, where the librarian describes charts the player cannot read. Both services were completed in [Lakeside Stops](../lakeside-stops-v01/README.md).

## Rebuilding and verifying an edition

Run from the repository root. The builder takes either matching Wind Lesson edition and writes a fresh external archive. The Field and Intro recipes record the parent and result hashes. Saves use their edition's content identity. [Field build](field-build.json) · [Intro build](intro-build.json)

```sh
cargo build --locked -j1 -p crystal-assets --example pack_ely_challenge --example game_corner_fixture
cargo build --locked -j1 -p geothite --bin geothite
parent=content-packs/minnesota-wind-lesson-v01.crystalpack
child=content-packs/minnesota-arcade-frontage-v01.crystalpack
target/debug/examples/pack_ely_challenge --arcade-frontage "$parent" "$child"
uv run --with cbor2 --with pillow python planning/minnesota/revisions/arcade-frontage-v01/check-pack.py "$parent" "$child"
uv run --with cbor2 --with pillow python planning/minnesota/tools/check_game_corner_mcp.py \
  --pack "$child" --client target/debug/geothite --fixture-tool target/debug/examples/game_corner_fixture \
  --output target/arcade-course-new
```

For the native tests, set `CRYSTAL_INTERIOR_PACK` and `MN_BADGE_TRAILS_PACK` to the absolute path of the child archive, and set `RUST_MIN_STACK=67108864`. The test filters are `game_corner::`, `regional_direction_boards` and `mill_workers_inn` in `cargo test --locked -j1 -p crystal-bevy --lib FILTER -- --ignored --test-threads=1`. The external fixtures supply a location, money or party when needed. Getting a case, spending the coins, giving the prize to a partner and teaching a prize TM all use the game's own controls. [Wallet and partner tests](../../../../crates/crystal-bevy/src/bevy_shell/tests/game_corner_wallet.rs) · [Cashier, redemption and machine tests](../../tools/check_game_corner_mcp.py)

## Verified outcomes

Both written editions passed the 17 Game Corner scenarios and ten native controller checks for the machines, wallet, TM use, street and trail signs, directory and inn. In the wallet check a visitor started with no coins, received 20, bought 500, redeemed the Quick Claw and saw 220 in the Coin Case. Pouchid still held the claw after Continue. A separate visitor redeemed Fire Blast, chose an incompatible Pouchid, then successfully taught Kindlit. After Continue, Kindlit still knew the move and the visitor could buy another copy.

The native Card Flip test won 72 coins on an exact-card wager, quit the game and reloaded the resulting 89-coin wallet. Both the item counter and the TM counter preserved the full wallet on refusal. The archive comparison verified that five maps changed and the other 738 matched their parents.
