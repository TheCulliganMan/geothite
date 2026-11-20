# Lakeside Stops

## Minneapolis Department Store

The street door opens onto the lobby carpet. The directory beside the stairs and the plate next to the lift both list the departments. Each upper floor has its own sign. The stairs and the lift serve all five floors.

| Floor | Department |
| --- | --- |
| 1F | Balls and Repels |
| 2F | Medicine |
| 3F | Battle gear |
| 4F | Vitamins |
| 5F | Move machines |

The fifth-floor counter has 13 TMs, including all three elemental punches. Every TM has at least one compatible partner among the 225 active Minnesota species.

Walk into the lift in the upper left corner and face the panel beside the door. Choose a floor, ride, then step out through either half of the doorway. Cancelling leaves the lift on its current floor. The game remembers the destination if you save inside the cab. The ride follows the floor selection and return-door behavior of Crystal's department stores. [Crystal elevator](https://github.com/pret/pokecrystal/blob/master/engine/events/elevator.asm) · [Celadon lift](https://github.com/pret/pokecrystal/blob/master/maps/CeladonDeptStoreElevator.asm)

## Route 11 and St. Paul

- **North Shore pie shop.** The baker serves blueberry pie across the counter. A slice costs 350 and restores up to 50 HP to one Pokémon in the field or in battle. The cracked pie on the table has a note from the baker about her mother.
- **Capitol Library.** The librarian identifies the chart by the far window. Read it from the aisle or the foot of the desk for the two canoe passages, the route across the west bridge and directions to the canoe shop. The bird sketch has its own shelf.
- **Union Depot.** There's a second way into the waiting garden above the timetable. Reed can return to his spot without cutting off the gardener. The clerk points to Mina beside the timetable.
- **Capitol Hall.** The wall behind the records officer is continuous. The staircase that drew people behind the desk had no destination.
- **City storefronts.** Six Duluth shops, four Minneapolis shops and nine St. Paul storefronts now open onto the first row of floor inside the entrance. The two exit carpet tiles lead to the same street doorstep.

## Editions and verification

Start with the matching **Arcade Frontage** archive. Run the builder from the repository root with a fresh output filename. The Field and Intro recipes identify the parent, the output archive and its content identity. Each edition keeps its own saves. [Field recipe](field-build.json) · [Intro recipe](intro-build.json)

```sh
cargo build --locked -j1 -p crystal-assets --example pack_ely_challenge
parent=content-packs/minnesota-arcade-frontage-v01.crystalpack
child=content-packs/minnesota-lakeside-stops-v01.crystalpack
target/debug/examples/pack_ely_challenge --lakeside-stops "$parent" "$child"
uv run --with cbor2 --with pillow python planning/minnesota/revisions/lakeside-stops-v01/check-pack.py "$parent" "$child"

export RUST_MIN_STACK=67108864
export CRYSTAL_WINONA_SAVE_DIR="$PWD/target/lakeside-stops-check/saves"
export CRYSTAL_INTERIOR_PACK="$PWD/$child"
for case in store_elevator:: store_landings:: shore_bakery:: capitol_river_chart:: st_paul_public_rooms::; do
    cargo test --locked -j1 -p crystal-bevy --lib "$case" -- --ignored --test-threads=1 || exit 1
done
```

The archive check compares map events, shop inventories, item behavior, private artwork and the rest of the game data with each parent. The controller cases begin with an explicit location and, for the bakery, a specified party, wallet or full bag. The visitor enters from the street, takes both exits in each of 19 storefronts, and buys two slices. One heals an injured Bankwing through the Pack. The second takes a battle turn when Bird Keeper Ivo spots the visitor. The purchase is also tested with no money, a full pocket and a declined offer.

Both editions passed the ten Lakeside Stops controller cases and the five inherited Game Corner cases. Exactly 27 existing maps changed and one elevator cabin was added. The comparison covers the other 716 Field maps and 718 Intro maps.
