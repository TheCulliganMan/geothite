# Badge Trails

Driftless Hollow’s sunny exits lead back to Route 1. The ladder below the east steps leads to the underground shell bed. The guide explains which way to go when handing over Flash and on return visits. Both trail mouths still face onto dry ground. The indoor path between them takes 26 walking steps. [Cave layout and references](hollow.md)

Brannic in St Paul recognizes a win in either wing. He names Avrin in Minneapolis if the challenger needs the Current Badge. A challenger with the badge who still needs a wing win hears the lift directions. Norek in Duluth points to Orlis in Bemidji if the Pine Badge is missing. Anver remains an optional battle. The gym audit traces both public streets and restored room layouts, including the arrival callbacks that clear Duluth’s old checkpoint. [Eight-gym review](gyms.md)

On a return to St Paul, Brannic can send a player carrying the pump sketch to Mina or explain the road to St Cloud. Once the Wind Badge is earned, he remembers the fans in Saven’s gym. In Brainerd, Sivren recognizes the ranger’s date, the Itasca reading and the signed comparison in turn. After the errand he talks about fishing with Torrout. [Leader and reward conversations](leaders.md)

## Build and verify

Use the Shore Watch parent listed on the matching build receipt and a fresh output path. The source builder checks its output by reopening the archive. [Field edition](field-build.json) · [Intro edition](intro-build.json)

```sh
CARGO_INCREMENTAL=0 cargo build --locked -j1 -p crystal-assets --example pack_badge_trails
target/debug/examples/pack_badge_trails "$parent" "$child"
export MN_BADGE_TRAILS_PACK="$child" CRYSTAL_GYM_TEST_PACK="$child" CRYSTAL_CANOE_TEST_PACK="$child"
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo test --locked -j1 -p crystal-bevy --lib -- --ignored --test-threads=1 \
  badge_trails_ brainerd_nurse_continue_and_battle_whiteout_return_to_last_heal \
  shore_watch_bank_walk_and_one_tick_water_glance_keep_the_player_on_foot
cargo test --locked -j1 -p crystal-bevy --lib shore_watch_walk_launch_paddle_and_land_in_native_frames -- --ignored --test-threads=1
cargo test --locked -j1 -p crystal-assets --lib canonical_dialogue_fits_native_pages
```

From one step outside each of the 18 trainers’ sightlines, a directional input opens the native challenge. The check advances the challenge dialogue and verifies the opponent, party species and levels for every resulting battle. Puzzle flags in these fixtures represent paths traced from each gym’s public door.

The focused native checks enter and leave both cave mouths, save and Continue inside the cave and downstairs, and return along the same route. Nine conversations use ordinary interaction with the three leaders from explicit badge and story states. A separate test heals in Brainerd, saves, loses a real gym battle and returns to the same Pokémon Center. It locates the trainer in the installed room so it works across the changes to Brainerd’s floor plan.

At the Route 18 mainland, Revive island, night beach and Mille Lacs picnic landing, the native renderer produced 312 frames per edition. Both recordings show the trainers taking walking strides, boarding, paddling and stepping ashore. The two editions match frame for frame. A second check follows 22 dry steps along two banks with and without a canoe. A tap turns the trainer to face the lake. After twelve idle frames, a new forward press launches an owned canoe, and a trainer without one stays on shore. [Shore Watch movement checks](../shore-watch-v01/README.md)

From this directory, run the following with Python, cbor2 and Pillow. The preservation check compares all maps and regional data, the shared script table, the native cave tiles, and the remainder of the archive. The other checks examine entrances, movement, trainer sightlines and dialogue branches.

```sh
python check-preservation.py "$parent" "$child"
python check-hollow.py "$parent" "$child"
python check-gyms.py "$child"
python check-leaders.py "$parent" "$child"
```
