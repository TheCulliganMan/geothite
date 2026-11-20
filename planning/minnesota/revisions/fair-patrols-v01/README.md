# Fair Patrols

The [20-point game audit](audit.md) covers the story, exploration, battles and regional services. It records which findings this revision addresses and what remains open.

Six challengers walk circuits through the State Fair. Midway C and the Barn Kid run. Midway D, the stage trainer, the cheese-curds trainer and the garden trainer walk. Watch which way they face as they turn. There is room for an eight-square sightline at multiple points on each circuit. The 32 vendor counters keep a clear approach from the Route 5 entrance. Ada the Baker’s Doughlin carries a blueberry pie slice that can restore up to 50 HP in battle.

On Route 15, the sapling beside the lake cabin opens a path through the evergreens to the rock with a hidden Ether. From the cabin doorstep, the eastern approach to the rock takes 24 steps around the wood or 12 after Cut. The same shortcut works on the return. The public pier still leads to a Revive, and the creel on the far landing holds a Lure Ball.

The three Winona Field Station benches reflect the starter you chose and the partners you subsequently adopted. The regional Pokédex numbers its 225 active species in the published family order.

The AREA page lists places, methods and time of day on the Minnesota map. For example, Paddarray lives in Winona’s Super Rod table and Wrigglet lives in the grass at Reed Marsh. The Trainer Card and Tamarack’s Hall of Fame report count regional species.

## Build and verify

The Field and Intro editions each require their own Fair Table archive. The recipes specify the expected parent and output checksums. The builder writes to a fresh filename and reads the archive back to verify it. [Field recipe](field-build.json) · [Intro recipe](intro-build.json)

```sh
cargo build --locked -j1 -p crystal-assets --example pack_ely_challenge
for edition in minnesota minnesota-intro; do
    parent="content-packs/$edition-fair-table-v01.crystalpack"
    child="content-packs/$edition-fair-patrols-v01.crystalpack"
    target/debug/examples/pack_ely_challenge --fair-patrols "$parent" "$child" || exit 1
    uv run --with cbor2 python planning/minnesota/revisions/fair-patrols-v01/check-patrols.py "$parent" "$child" || exit 1
    uv run --with cbor2 python planning/minnesota/revisions/fair-table-v01/check-vendors.py "$child" || exit 1
done
```

To run the focused engine and Pokédex checks on the Intro edition, use the following commands. The last check renders the native AREA screen directly from a menu fixture.

```sh
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
export MN_STARTER_BENCH_PARENT="$PWD/content-packs/minnesota-intro-fair-table-v01.crystalpack"
export MN_FIELD_GUIDE_PACK="$PWD/content-packs/minnesota-intro-fair-patrols-v01.crystalpack"
export CRYSTAL_TOWN_MAP_RENDER_DIR="$PWD/target/fair-patrols-review"
cargo test --locked -j1 -p crystal-core --lib regional_trainers
cargo test --locked -j1 -p crystal-assets --lib winona_benches_follow_each_starter_choice_and_adoption -- --ignored
cargo test --locked -j1 -p crystal-assets --lib fair_table_dex_numbers_are_dense -- --ignored
cargo test --locked -j1 -p crystal-bevy --lib ely_recognizes_every_regional_partner
cargo test --locked -j1 -p crystal-bevy --lib pokedex_habitats -- --include-ignored --test-threads=1
cargo test --locked -j1 -p crystal-bevy --lib regional_pokedex -- --ignored --test-threads=1
```

Both written editions passed the static checks. The six trainers have 40 positions where they can see eight squares along their current facing. Reserving all 254 squares on the six trainer circuits and 28 visitor circuits simultaneously still leaves an ordinary-floor path from the gate to each of the 32 counters. All 13 specialty sellers and the three public food rooms outside the Fair retain their stock. The Route 15 check compares every terrain block and event with the matching parent, including the pier, bridge and creel.

The archive comparison also checks the three changed maps, Ada’s single held-item change across all trainer teams, and all 225 regional Pokédex numbers against the family order. It compares the item catalog, encounters, compiled audio and runtime artwork with the matching Fair Table edition.
