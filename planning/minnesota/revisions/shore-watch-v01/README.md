# Route 18 and Mille Lacs Shore Watch

Both western fishing piers face open water. The trail east follows a brown stone bank to a sandy launch. Across the water, an open beach offers a place to land for the Revive. At Mille Lacs the picnic trail curls around an inlet to a flowering point. A picnicker tells how Dartduck caught her windblown lettuce, then offers the Lure Ball she lost in the flowers. The point can be reached on foot from the Fly landing, or by water. A canoe or Surf brings the offshore PP Up within reach. [Route and island](check-route.py) · [Picnic cove](check-picnic.py)

The sand is drawn around the trainer's boots and the canoe's keel. The walking and canoe sprites meet at their visible point of contact with the bank. That point is low in the 16-pixel sprite, so a southward launch happens earlier in the stride than a northward launch. Landing reverses the same crossing. The native motion check samples both characters on the western mainland, Revive island, eastern beach and picnic cove. Fixed controller inputs walk to the bank, launch, paddle, turn and land. An optional capture exports the real visible sprites at the native LCD resolution, including walk and paddle action frames. [Motion regression](../../../../crates/crystal-bevy/src/bevy_shell/tests/shore_watch_canoe.rs)

The dock ranger introduces the three volunteers at the reeds, meadow and beach. On a partial return he names a volunteer the player still needs. The park ranger points players with a complete report back to the dock. With the Lake Badge, players carrying an Old Rod or no rod earn the Good Rod. Players who already own a Good or Super Rod earn three Lure Balls. A full pocket defers the entire gift. Previously completed reports keep their existing receipt. Crystal and Polished Crystal both remember the angler's Good Rod gift at Olivine. Minnesota's extra tackle gives the trip a purpose even after visiting its other rod houses. [Report and trainer checks](check-reports.py) · [Crystal](https://github.com/pret/pokecrystal/blob/master/maps/OlivineGoodRodHouse.asm) · [Polished Crystal](https://github.com/Rangi42/polishedcrystal/blob/master/maps/OlivineGoodRodHouse.asm)

A native nighttime Good Rod cast at the Route 18 pier could start a level-20 Tapduck battle and then fail save validation because Tapduck at that level came from the lake swarm. The save validator and runtime encounter list now recognize the swarm tables assigned to the originating map. The regression uses a seeded swarm, saves during that battle, reopens it, catches the fish with a Lure Ball and reopens the completed catch. It also checks a daytime Scutlet, a missed bite, a cast toward land, and every rod-table slot at morning, day and night on maps with swarm rules. [Fishing regression](../../../../crates/crystal-runtime/tests/shore_watch_fishing.rs)

## Reproduce the local editions

Select the exact League Journey parent for the desired edition and a fresh output path. The builder verifies the archived result after writing it. [Field](field-build.json) · [Intro](intro-build.json)

```sh
CARGO_INCREMENTAL=0 cargo build --locked -j1 -p crystal-assets --example pack_shore_watch
target/debug/examples/pack_shore_watch "$parent" "$child"
export MN_SHORE_WATCH_PACK="$child" CRYSTAL_WINTER_RENDER_TEST_PACK="$child"
export CRYSTAL_CANOE_TEST_PACK="$child"
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo test --locked -j1 -p crystal-runtime --test shore_watch_fishing -- --ignored --test-threads=1
cargo test --locked -j1 -p crystal-assets --lib -- --ignored --test-threads=1 shore_watch_lure_reward the_flower_point
cargo test --locked -j1 -p crystal-bevy --lib shore_watch_ -- --ignored --test-threads=1
cargo test --locked -j1 -p crystal-bevy --lib canoe_shore
cargo test --locked -j1 -p crystal-bevy --lib -- --ignored --test-threads=1 \
  canoe_controller_automatic_launch_portage_and_water_continue \
  canoe_fishing_controller_registered_rod_catch_and_continue_afloat \
  canoe_controller_paddling_triggers_wild_battle_and_capture
```

Set `MN_SHORE_MOTION_DIR` to an empty review directory when running the Bevy motion test to keep native frames and their tile, timing and sprite records. The test uses an isolated equipped-canoe fixture and disables random encounters for the short crossings.

Run these checks from this directory with Python, cbor2 and Pillow. The first compares archive contents with the parent, including original terrain pixels, habitats, trainer teams and ambient animation. The other checks trace the actual maps and their event scripts.

```sh
python check-preservation.py "$parent" "$child"
python check-route.py "$parent" "$child"
python check-picnic.py "$parent" "$child"
python check-reports.py "$child"
```
