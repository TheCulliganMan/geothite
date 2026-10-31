# Minneapolis Game Corner

The Game Corner occupies the former Corner Home facade in southern Minneapolis.
Enter at walking tile **(56, 51)**; the safe outside return is **(56, 52)**.
Read the wall immediately left of the entrance for the Game Corner sign.
The internal map identity remains `MnMinneapolisCornerHome`; the playable room
is a native 2D, 10-by-7-metatile Game Corner. Its arrival is (3, 12), with paired
carpet exits at (2, 13) and (3, 13). The publisher checks a walking connection
from the Minneapolis clinic, every game approach and all three counters.

The attendant at (2, 9) provides a free Coin Case and **20 starter coins once**.
The left counter sells 50 coins for 1,000 money or 500 for 10,000. Purchases
require the case, sufficient money and room below the 9,999-coin limit.

The left machines run native **slots**, with one-to-three-coin bets and three
individually stopped reels. The right machines run native **Card Flip**, with
a three-coin stake, a shuffled 24-card deck and the original group/card wagers.
Both use the authoritative coin balance and return control when quitting.

The two right counters offer repeatable prizes:

| Counter | Prize | Coins |
| --- | --- | ---: |
| Items | Moon Stone | 300 |
| Items | Rare Candy | 500 |
| Items | PP Up | 1,000 |
| TMs | TM25 Thunder | 5,500 |
| TMs | TM14 Blizzard | 5,500 |
| TMs | TM38 Fire Blast | 5,500 |

Each redemption delivers one item. Declining, insufficient coins, a full Item
pocket or a full TM stack leaves the coins untouched. The room's two former
residents become the attendant and a patron. Native art and machine geometry
are read from the supplied pack's Goldenrod Game Corner; Johto/Kanto maps,
regional encounters, trainers, Pokemon, art and audio are preserved.

## Source and publication

The authored contract is [game-corner.json](game-corner.json). Apply this Rust
stage after rebuilding Minneapolis interiors. Its current local result is
`content-packs/minnesota-game-corner-v06.crystalpack`, published over
`content-packs/minnesota-current-r16-bank-guidance-v02.crystalpack`. The selected
regional revision is recorded in [world.json](world.json); its cumulative source
index is [revisions/game-corner-v01/sources.json](revisions/game-corner-v01/sources.json).

Executed local build and publication commands:

```sh
CARGO_TARGET_DIR=target/game-corner-ui CARGO_INCREMENTAL=0 cargo build --locked \
  -p geothite -p crystal-assets -p crystal-bevy \
  --features crystal-bevy/location-tester \
  --bin geothite --bin crystal-bevy --bin pack_minnesota_game_corner \
  --example game_corner_fixture --example render_at_location
target/game-corner-ui/debug/pack_minnesota_game_corner \
  content-packs/minnesota-current-r16-bank-guidance-v02.crystalpack \
  planning/minnesota/game-corner.json \
  content-packs/minnesota-game-corner-v06.crystalpack
```

The publisher requires a fresh, ignored output filename; choose another name
when replaying it. Packs, captures, fixture saves and build products are ignored.
It does not deploy the browser release or migrate saves between pack identities.

Card Flip's renderer reads the existing compiled 2bpp banks. Native tile IDs
address deduplicated 62/52-tile banks, while the source PNGs contain 64/60 tiles.
The board also requires 11 columns and 12 rows. These fixes retain every source
asset and restore the native board, portraits, cards and readable prompts.
Renderer-neutral input advances the same slot/card animation clocks once per
frame; the graphical client retains its regular update schedule.

## Verification

Local verification output is retained under
`planning/minnesota/renders/game-corner-v01/`. Position and wallet fixtures are
explicit test setup; they do not establish earned regional story progression.
All subsequent actions use `VisibleShellController::press` or production stdio
MCP inputs. The Rust `game_corner_fixture` example creates those isolated saves
and inspects results without editing a live game.

The MCP course is `tools/check_game_corner_mcp.py`, covering the attendant,
both exits, Continue, all six prizes, declined purchases, insufficient coins,
full-pocket/full-TM refusals, coin purchases and capacity/funds guards, complete
slot and Card Flip rounds, payouts, and returning to movement. Every spawned
client must leave stderr empty. Native screenshots and the final regression
results are retained alongside its receipt.

The retained local native client includes the current renderer and controller:

```sh
.local/minnesota-game-corner-client/crystal-bevy \
  --pack content-packs/minnesota-game-corner-v06.crystalpack \
  --save-path .local/minnesota-game-corner-client/play.crystalsave
```

Verification passed: the four pack/controller regressions, the tile-bank decoder,
11 baseline production-controller regressions (including battle PP mutation and
Mom/New Bark), the production MCP movement/Start regression, and clean stderr.
The Card Flip controller case selects an exact-match fixture wager, receives 72
coins (20 minus the 3-coin stake plus 72 = 89), quits, moves, saves and Continues
with 89 coins. Seventeen MCP scenarios cover games, prizes and the coin service.
Their receipts are `mcp-final-v08/receipt.json` (16 scenarios) and
`mcp-games-final-v10/receipt.json` (the completed game course). The v06 room is
byte-for-byte identical to that tested v04 room, and `current-fresh-v11` repeats
entry, attendant, refusal, both exits and Continue on v05. The complete 17-scenario course is repeated on v06 in
`current-all-v12/receipt.json`. The current-parent preservation comparison is
`current-r16-parent-preservation.json`.

Actual headless native 2D review includes the room, entrance, slots, idle Card
Flip choice, dealing/selection, quitting back to the room, and the prize menu and
confirmation. `native-prize-menu-v06.png` shows the actual item counter. The idle YES/NO
and message windows retain their own input-screen ownership; pixel checks in
`screen-contract-v03.json` catch disappearing panels, missing text and absent
cards. All earlier/rejected captures are retained. Run
`tools/check_game_corner_screens.py` with the accepted prompt/deal PNGs to repeat
those checks. The native review client was also built with Cargo's test profile
for these offscreen captures; the ordinary build command above remains available.

All 377 audio programs pass PCM hashes, frame counts and loop ranges. The
preservation comparison allows changes only to the room, its entrance sign and
their compiled mirrors/event constant. Current route/cave work, encounters,
trainers, source files, HOME and all other game data remain intact.

Clippy for both affected crates and all targets passes with existing warnings.
Changed leaf Rust files pass rustfmt. The workspace-wide formatting check still
reports pre-existing formatting differences. The focused roster audit passes for
both touched maps, 319 regional trainers and the 225-species active evolution
closure. The broader regional audit still reports pre-existing foreign-species
fishing-group references on other maps; those unrelated tables are unchanged.

To repeat the external-pack regression, set `RUST_MIN_STACK=67108864`,
`CRYSTAL_INTERIOR_PACK` to the completed Game Corner pack and
`CRYSTAL_GAME_CORNER_PARENT` to its parent. Run:

```sh
cargo test --locked -p crystal-assets -p crystal-bevy --lib \
  --features crystal-bevy/location-tester minnesota_game_corner \
  -- --ignored --nocapture --test-threads=1
python3 planning/minnesota/tools/check_game_corner_mcp.py \
  --pack content-packs/minnesota-game-corner-v06.crystalpack \
  --client .local/minnesota-game-corner-client/geothite \
  --fixture-tool target/game-corner-ui/debug/examples/game_corner_fixture \
  --output planning/minnesota/renders/game-corner-v01/new-review --case all
```

The Python inspection harness requires Pillow and cbor2. Build the fixture
example with `cargo build --locked -p crystal-assets --example game_corner_fixture`
and the same `CARGO_TARGET_DIR` first. Saved review clients remain under `.local`;
disposable task Cargo caches are removed after verification.
