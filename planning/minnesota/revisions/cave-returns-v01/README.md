# Following Castorock and finding the way back

In Route 3’s River Pocket Cave, Castorock leaves two grooves above the pool, bark in the split pillar and a forked branch on the dry bed. Each mark belongs to the exact stone the player can inspect. The walker beside the water now remembers the traces the player has read and where the search leads next. The original Strength puzzle and optional encounter still use the same three places. Crystal gives its Union Cave visitor a clue to the Pokémon at the water’s edge. Minnesota’s clue follows what Castorock does with its branch. [Castorock’s three tracks](caves.md) · [Crystal’s encounter](https://github.com/pret/pokecrystal/blob/master/maps/UnionCaveB2F.asm) · [Polished Crystal’s encounter](https://github.com/Rangi42/polishedcrystal/blob/master/maps/UnionCaveB2F.asm)

Duluth’s dock clerk now sells the Escape Rope mentioned at the counter and the basic Potion a nearby customer asks for. In Ely, the registrar explains that players can revisit earlier League chambers. Losing a match or returning to the lobby ends the run. The shop and registrar keep their ordinary buy and registration choices. [Stock and routes to Ely](provisions.md) · [League losses and withdrawal](league.md)

## Local build and checks

Build from the matching Gym Walk parent into a fresh external archive. The receipts identify the exact inputs and outputs for each edition. [Field](field-build.json) · [Intro](intro-build.json)

```sh
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo build --locked -j1 -p crystal-assets --example pack_cave_returns
target/debug/examples/pack_cave_returns "$parent" "$child"
export MN_BADGE_TRAILS_PACK="$child" CRYSTAL_WINONA_TEST_PACK="$child" MN_CAVE_RETURNS_PACK="$child"
cargo test --locked -j1 -p crystal-assets --lib cave_returns_dock_clerk -- --ignored --test-threads=1
cargo test --locked -j1 -p crystal-bevy --lib -- --ignored --test-threads=1 \
  cave_escape_trip cave_returns_incisor league_return_trip_ \
  castorock_controller_strength_gate_and_three_acknowledged_clues \
  castorock_controller_decline_actual_capture_and_continue \
  castorock_controller_run_and_defeat_return_paths_and_reentry
```

The cave return check starts on the approach to each of Minnesota’s fourteen outdoor cave mouths. The player enters, saves and continues underground, opens the Pack and uses one of two Escape Ropes. It checks where the player emerges, the step back onto the trail and reentry into the same cave. One trip also descends to the fossil gallery before saving. Another fixture enters the Duluth shop, buys a rope and medicine from the clerk across the counter, and checks that the purchases survive Continue. Encounters are disabled in the cave return fixtures. The separate Castorock tests exercise Strength, the search, encounter decisions and capture. Set `MN_CAVE_TRACE_FRAMES` to an empty directory to retain three native 160×144 views while facing the marked stones.

The archive checks compare the two complete editions with their respective parents. Run these from the revision directory with Python, Pillow and cbor2.

```sh
python check-preservation.py "$parent" "$child"
python provisions.py "$parent" "$child"
python caves.py "$parent" "$child"
```
