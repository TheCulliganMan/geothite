# League Journey

Route 13 narrows to the two squares that actually cross into Ely. Snow thins into the gravel at the town line. The little western path reaches a pair of snowy Headbutt pines. Tomas still spots passing trainers, and the item behind Cut keeps its own approach. The guide on the other road gives the actual directions to Ely and Split Rock. [Both roads to Ely](approach.md)

Veil and Casey now have a south exit that works from either side of the carpet. A returning Champion can decline Casey, reconsider and talk again, or walk back through the chambers. Leaving the League at the lobby still ends the run. Ely’s Mart now sells Full Restores for ₽3,000, alongside the cheaper single-purpose medicines. The pine camp still gives the explorer one for free. In Ely and Split Rock the clinic PC now responds where its screen stands. [League rooms and prizes](rooms.md) · [Preparation and Fly landings](check-preparation.py)

After the Hall of Fame, the Ely ranger can point the player to Reed at the St. Cloud station. The Boundary Waters ranger knows whether the skier still needs to cross, has a reward waiting in International Falls, or has already collected it. [After the Hall of Fame](after-champion.md)

Crystal and Polished Crystal both put a nurse and an item clerk at Indigo Plateau before the first League battle. Minnesota already has a clinic and a Mart in Ely, and its registrar points to both. This revision builds on those two town locations. The check follows both street exits from sign-in to the shop and clinic, including the clerk’s counter, the nurse and the party PC. [Crystal’s Indigo Plateau](https://github.com/pret/pokecrystal/blob/master/maps/IndigoPlateauPokecenter1F.asm) · [Polished Crystal’s Indigo Plateau](https://github.com/Rangi42/polishedcrystal/blob/master/maps/IndigoPlateauPokecenter1F.asm)

## Build and verification

Use the Fen Clearings parent for the desired edition and an unused output path. The receipts identify the exact input and output bytes. The command rereads and validates the resulting archive. [Field](field-build.json) · [Intro](intro-build.json)

```sh
CARGO_INCREMENTAL=0 cargo build --locked -j1 -p crystal-assets --example pack_league_journey
target/debug/examples/pack_league_journey "$parent" "$child"
export MN_LEAGUE_JOURNEY_PACK="$child" MN_LAKE_OBSERVATIONS_PACK="$child"
RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0 cargo test --locked -j1 -p crystal-assets --lib -- --ignored --test-threads=1 league_journey_ lake_observations_five_league_doors lake_observations_level_fifty_starter_league_resources
```

Run the following beside this file with Python, cbor2 and Pillow. The checks read the archived maps, art, collision, scripts and teams. They also compare the two editions independently with their own parents. Player-facing stills were rendered in native 2D. The tests cover five-battle move resources and actual return permissions. They do not measure how often first-time players win.

```sh
python check-preservation.py "$parent" "$child"
python check-preparation.py "$child"
python check-approach.py "$parent" "$child"
python check-league-rooms.py "$parent" "$child"
python check-after-champion.py "$parent" "$child"
```
