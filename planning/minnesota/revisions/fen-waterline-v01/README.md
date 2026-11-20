# Walking and canoeing at the Layered Fen

The stones follow the land around both Fen pools. Every square where the canoe can stop carries the original animated Johto water, including the narrow part of the south lake and the tip of the north pool. The walker can follow the low pebble bank, turn to face the lake, and continue along the shore. With the canoe in the Pack, stepping into the lake launches it.

The earlier layout had water permissions beneath a band of solid stone. At the south lake narrows, the camera showed the canoe settling on a gold pixel at (24,20). The regression samples the rendered ground directly beneath the keel on every mounted frame. A second check reads the finished pack and tests both walkers' soles and canoe hulls around both pools.

The two pools have 85 accessible bank-to-water contacts, 68 around the south lake and 17 around the north inlet. The pack check covers 205 water squares and 787 adjacent walking or paddling steps, including motion between squares. It also checks the encounter grass along the shore. Opening Mitwist’s hollow changes a separate block at (26,8). All six clue approaches and the path to the hidden Ultra Ball remain walkable before and after the reveal.

## Reproduce the check

Use the matching Cave Returns parent from the [Field](field-build.json) or [Intro](intro-build.json) receipt. Choose a fresh output path and use absolute paths for the pack and optional capture directory.

```sh
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo build --locked -j1 -p crystal-assets --example pack_fen_waterline
target/debug/examples/pack_fen_waterline "$parent" "$child"
python planning/minnesota/revisions/fen-waterline-v01/check-footing.py "$parent" "$child"
export CRYSTAL_CANOE_TEST_PACK="$child" MN_SHORE_MOTION_DIR="$frames"
cargo test --locked -j1 -p crystal-bevy --lib -- --ignored --test-threads=1 fen_south_lake_ fen_canoe_grass:: fen_grassy_banks::
```

The grassy landing at (19,15) stirs the blades when the boots touch down. In Crystal, the lower two sprite pieces can sit behind grass while the upper two stay visible. The renderer follows that rule for Minnesota maps using the original Johto grass and for the matching prairie grass at Former Shores. The rocky half of a grass square keeps its place beneath the trainer and canoe. [Crystal walking sprite pieces](https://github.com/pret/pokecrystal/blob/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/data/sprites/facings.asm#L42-L114) · [Crystal grass priority](https://github.com/pret/pokecrystal/blob/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/engine/overworld/map_objects.asm#L2884-L2892)

The four native camera fixtures follow the north, south, east and west approaches to the southern lake, paddle a second square, and land on the same bank. The shore walk tests turning to look at the lake with and without the canoe. Each fixture starts at the named location and suppresses random battles while capturing the movement. The archive check requires Python, Pillow and cbor2.

Additional camera fixtures compare the visible head and boots to the native sprite as Chris and Kris cross ordinary grass at the Detroit Lakes approach and the grass-and-gravel edge at Former Shores.

The southern lake has 14 contacts between tall grass and water. Four held-direction camera fixtures cover all four grass-and-gravel arrangements used at those contacts. Each follows a continuous walk from grass into water, paddles farther in, and returns to the grass. They check the displayed trainer and hull against the active sprite on every tick. To capture them, set `MN_SHORE_MOTION_DIR` and run the command above. The `grass-east`, `grass-west`, `grass-north` and `grass-south` folders are named for the direction of launch.

The same four crossings also run with two display updates per game tick. The test measures the position of the boots or keel against the scrolling map. The midway frame must advance one native pixel in the direction of travel. It also checks which side of the bank the rider occupies, the grass over the boots and the visible canoe. To repeat this check alone, run `fen_grassy_banks_walk_launch_and_land_between_game_ticks` with the selected pack in `CRYSTAL_CANOE_TEST_PACK`. Its captures go in the `between-ticks` subfolder.

On a long northbound or southbound paddle, the two native canoe poses put the blade on opposite sides. The walking trainer mirrors alternating strides. Mirroring a canoe stroke made it look exactly like the resting hull for both riders. The straight-water check compares the 16×16 sprite pixels on both paddle beats for Chris and Kris across five squares in each direction. Its `paddle-chris` and `paddle-kris` captures include the walk to the launch.

The north pool has a one-square tip at (17,5). The native camera follows Kris from the grass at (17,9), across the pebble bank and into that tip. She turns west on the water, turns back, lands and returns over the same grass. The test checks the visible trainer on every update and samples the water beneath the mounted keel. Set `MN_SHORE_MOTION_DIR` and run `fen_north_inlet_walk_launch_turn_at_the_tip_and_return` for the full sequence of 160 × 144 frames.

The two-pool portage starts on the dry approach to the north tip at (17,6). Launch at (17,5), paddle around the corner and land at (16,7). Walk south through the grass, turn east along the bank, and launch into the south lake at (19,15). Circle back and land at (18,15). The `fen_portage_between_both_lakes_preserves_footing_and_water_motion` fixture follows this route in one session with Chris at noon and Kris at night. It pauses on each lake to compare the ripples beside the canoe with a patch of the stationary bank. With `MN_SHORE_MOTION_DIR` set, the two native camera sequences are written to `fen-portage-chris` and `fen-portage-kris`.
