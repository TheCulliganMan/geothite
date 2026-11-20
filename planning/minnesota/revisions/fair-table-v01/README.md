# Fair Table

At North Shore Pies on Route 11, buy a blueberry slice for 350. In the Pack's Items pocket, choose **Use** to restore up to 50 HP to a living partner, or choose **Give** and select a Pokémon. A partner holding the slice eats it at the between-turn check in battle when its HP is below half. At exactly half it keeps the slice. You can take the slice back from the party menu before a battle. The between-turn timing, half-HP threshold and single-use behavior follow Crystal’s healing berries. [Crystal battle code](https://github.com/pret/pokecrystal/blob/master/engine/battle/core.asm#L4089-L4199)

The 16 State Fair specialties use the same rule, with recovery ranging from 10 HP for a pickle stick to 100 HP for fried walleye. The drinks seller explains the difference between the two beers before taking an order. Fair beer heals up to 100 HP and causes four turns of confusion. Root beer heals up to 60 HP. Water, milk, soda and lemonade keep their familiar Pack uses. [Food, price, healing and counter list](foods.md)

The West Seventh Bakery sells butter rolls and cardamom buns. Duluth Community Kitchen serves mushroom and wild rice soup. The Hillside Hostel cook will give you one butter roll for the road. The cook keeps it for you if you decline or arrive with a full Pack. The host who heals your team still works the original desk.

## Build and check

Each edition starts from its matching Lakeside Stops pack. The Field and Intro recipes identify the expected archives by checksum. The packs are external to the source repository. Use a fresh output name. [Field recipe](field-build.json) · [Intro recipe](intro-build.json)

```sh
cargo build --locked -j1 -p crystal-assets --example pack_ely_challenge
parent=content-packs/minnesota-lakeside-stops-v01.crystalpack
child=content-packs/minnesota-fair-table-v01.crystalpack
target/debug/examples/pack_ely_challenge --fair-table "$parent" "$child"
uv run --with cbor2 python planning/minnesota/revisions/fair-table-v01/check-pack.py "$parent" "$child"
uv run --with cbor2 python planning/minnesota/revisions/fair-table-v01/check-vendors.py "$child"

export CRYSTAL_INTERIOR_PACK="$PWD/$child" CRYSTAL_FAIR_TEST_PACK="$PWD/$child"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/target/fair-table-check/saves" RUST_MIN_STACK=67108864
mkdir -p "$CRYSTAL_WINONA_SAVE_DIR"
cargo test --locked -j1 -p crystal-core --lib held_
for case in held_picnic:: food_vendors:: food_neighbors:: shore_bakery::; do
    cargo test --locked -j1 -p crystal-bevy --lib "$case" -- --ignored --test-threads=1 || exit 1
done
```

The location check starts at the woodland gate with every fairgoer in place. It follows floor tiles to all 32 stalls and matches each seller, counter face, interaction and stock list to the authored fair. There are 13 stands selling the 16 specialties. The comparison also checks the regional items, shop inventories, dialogue, gift flag, map geometry and artwork against the parent edition.

Both editions passed 25 focused checks each, covering the food counters, hostel, bakery, Game Corner and Lakeside services. For each of the 20 foods, two battle turns checked healing, consumption and the empty held slot on the following turn. A separate test bought a slice, gave it to Keypout, took it back, gave it again and saved. After Continue, Keypout ate the slice in Bird Keeper Ivo’s battle and went from 15/180 to 65/180 HP. Its held slot stayed empty after switching and after a post-battle save. Chrome displayed the same purchased item in the Pack and on the second party stats page.
