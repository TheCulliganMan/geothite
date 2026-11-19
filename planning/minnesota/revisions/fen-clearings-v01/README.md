# Returning to the Layered Fen

At the east entrance, snow reaches the verges of the walking trail in patches. The exposed trail continues through to the Bemidji sign. Intact pine silhouettes carry the change from green needles to frost. Only the Fen and Northwoods use the new arrangements. The lakes retain their low brown shores and open water.

The southern walking loop leads to twin firs beside the cove. Looking behind the farther tree from the dry pocket finds one Ultra Ball. The observer mentions where the ball rolled after the hollow opens. It can also be discovered earlier. A full Ball pocket leaves the item in place. Itemfinder tracks the actual collectible.

Mitwist’s hollow has a narrow entrance and a sheltered place beneath the pines. The three dependable clues open the path in any order, and the optional encounter remains available until caught.

The original hollow had two flower stamps in a clearing. The new layout opens 24 walking squares beneath the trees. From the gap, the western side of Mitwist is seven steps away. Players can approach from three sides and can leave or decline the encounter. The pool and grove search points still work.

The two pools belong to the regional peat-water habitat. Morning Surf favors Rondive, daylight favors Reedraft, and after dark Leechurge is common. Crosspout can be found at all three times. The authored base slot levels are 32, 34 and 36, the levels already assigned to Fen Surf by the regional encounter plan. The Surf rate is 3. Rods continue to use the native Minnesota pond group.

## Reproduce the local build

Build from the matching Trail Battles parent specified in the edition's receipt and choose a fresh output path. The exporter validates its input and reads the finished archive back. Both editions use the same source. Generated packs stay outside version control.

```sh
CARGO_INCREMENTAL=0 cargo build --locked -j1 -p crystal-assets --features pack-authoring --example pack_fen_clearings
target/debug/examples/pack_fen_clearings "$parent" "$child"
```

Use absolute archive paths for the native regressions. They operate on isolated state using production pickup, encounter and save code.

```sh
export MN_FEN_CLEARINGS_PACK="$child" MN_FEN_CACHE_PACK="$child"
export RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0
cargo test --locked -j1 -p crystal-assets --lib -- --ignored --test-threads=1 fen_pools_resolve_all_three_surf_slots_at_each_clock southern_fen_spare_ball_full_pocket_itemfinder_and_continue
```

The checks beside this file use the archived game data. Run each against the parent and child with Python, cbor2 and Pillow. The hollow check exercises all six landmark orders with primary and historical positions, returning partial progress and the optional battle outcomes. The shore check compares both sides of the road, the sign, underlying movement and animations.

```sh
python check-preservation.py "$parent" "$child"
python check-thaw.py "$parent" "$child"
python check-hollow.py "$parent" "$child"
python check-southern-cache.py "$parent" "$child"
python check-water.py "$parent" "$child"
```
