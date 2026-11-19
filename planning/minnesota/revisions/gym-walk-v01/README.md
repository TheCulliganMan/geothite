# The first visit to six Minnesota gyms

The Rochester court, Mankato quarry, Minneapolis power station, St. Cloud airworks, Brainerd beach and Bemidji garden were reviewed from their actual entrances on the Badge Trails editions. St. Paul and Duluth have their own [court and harbor review](../badge-trails-v01/gyms.md).

| Gym | What changes for the player |
| --- | --- |
| Rochester | Both flights meet Varel’s raised court. Elvik watches the short east stair, and the long route to the west stair gives the player a choice. |
| Mankato | The one-way drop has a sandstone lip. The walls meet the terrace, and Jorel’s gate reveals the stairs that actually lead up to Torven. |
| Minneapolis | A works on the visible faces of both breakers. A locked breaker names the trainer who controls it. |
| St. Cloud | Both halves of the four console fronts respond. The guide stands clear of the lower dial. The player can still cross and return without battling Hallen. |
| Brainerd | Follow the pale sand straight to Sivren on foot. The darker water forms three canoe pools. The two regular trainers stand on sand and challenge by sight. |
| Bemidji | The guide hands over Cut, points out the two approaches and remembers which keeper the player has beaten. After the badge, the guide identifies the return gate. |

The room reviews record the views, routes, choices and original game references. [Court and quarry](rochester.md) · [Power station and airworks](relays.md) · [Beach and garden](north.md)

## Build and verify

Use the matching Badge Trails parent and a fresh output path. Intro restores the published Brainerd beach art. Field already had the beach. Both editions receive the pool depths and animation. [Field receipt](field-build.json) · [Intro receipt](intro-build.json)

```sh
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo build --locked -j1 -p crystal-assets --example pack_gym_walk
target/debug/examples/pack_gym_walk "$parent" "$child"
MN_BADGE_TRAILS_PACK="$child" cargo test --locked -j1 -p crystal-bevy --lib -- \
  --ignored --test-threads=1 gym_walk_ badge_trails_all_eighteen
```

The native tests use disposable party, position and puzzle-state fixtures. They enter Rochester and Mankato, try the former sideways approaches, walk the stairs and make the quarry drop. They operate the power breakers and airworks dials, visit the garden guide with different wins, take the straight wade with and without the canoe, paddle all 19 deep squares and Continue afloat. Each of the 18 regular gym trainers also starts its own battle on sight.

Set `MN_GYM_WALK_BEACH_FRAMES` to an empty directory to capture the native 160×144 screen every second update on the central crossing and the three canoe trips. The wading test checks the two trainers' rendered feet on sand, in water and at the mixed bank. It also checks the change from wet boots to the canoe and back.

Run these archive checks from this directory using Python with cbor2 and Pillow.

```sh
python rochester.py "$parent" "$child"
python relays.py "$child"
python north.py "$parent" "$child"
python check-pools.py "$parent" "$child"
```
