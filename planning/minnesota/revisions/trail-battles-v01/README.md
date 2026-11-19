# Layered Fen and the Northwoods turnoff

The eastern trail enters a wetland between two pools. The path crosses between the western saplings and the northern inlet, follows the west cove and turns along the south pool toward the fir at the foot of the eastern pines. On the way back the player can read the snowy Northwoods sign for Route 17 or Bemidji. The mark in the saplings and the reflective bank use the familiar Johto artwork. The eastern mark sits on a fir branch.

The 40-by-26-square water outline is hand-authored in `fen-water.txt`. Each wet square carries the WATER permission. Edge tiles follow its actual neighbors, including every inward and outward bend. On the south side a low horizontal row of sunlit stones meets the existing side bank using four native-scale corner shapes. The blue water runs through the neck connecting the southern coves. The two shores permit boarding and landing where public ground meets water. The encounter grass around the pools remains encounter terrain. The hidden hollow opens on foot.

All three observations count on the first visit in any order. The observer points to a missing mark if asked. The third mark opens the hollow and reveals Mitwist. The old secondary search positions and partial story flags still work. A player can decline the encounter and return. The story only records the catch after a successful capture. Northwoods, the eastern exit, is the return to the main road.

We compared the donor lakes in the supplied Johto archive, including Lake of Rage, and inspected the searchable Route 35, 38 and 43 signs. Ilex Forest's changing dialogue and persistent search, and the clearly marked destinations on Route 31, informed this walk. These maps and Polished Crystal's version of Route 31 can be inspected in their original authors' sources. [Crystal Ilex Forest](https://github.com/pret/pokecrystal/blob/master/maps/IlexForest.asm) · [Crystal Route 31](https://github.com/pret/pokecrystal/blob/master/maps/Route31.asm) · [Polished Crystal Route 31](https://github.com/Rangi42/polishedcrystal/blob/master/maps/Route31.asm)

The two compiled pools occupy 85 and 120 water squares. From public land they have 17 and 64 two-way Surf approaches respectively, after excluding examination points. All 48 combinations of landmark order and old or primary interaction squares complete the investigation. Following the observer’s suggested order takes 83 steps from the east entrance to Mitwist, down from 131. The trip uses the real collision grid and respects the closed hollow.

## Other campaign corrections

- Two ICE guards now see the player along their real basement and relay-floor corridors. Gym students keep their sight battles and all eight badge leaders await conversation.
- After Minneapolis, Avrin gives the dispatch lead the player has actually reached. On Route 5 the warden directs the player down the three south-facing shelves.
- Ten Minneapolis cyclists and fifteen State Fair trainers have individually authored teams. Moves come from the active partner’s native level-up and TM lists. The fixed-level regional editions keep their authored levels. In the optional collection-scaling rules a leader retains the awarded TM on an original user when every copy would have dropped out of the four move slots.
- Dig and Escape Rope retain the particular cave mouth used to enter after an internal stair. The entrance survives save restoration and the outdoor arrival steps away from the door. [Crystal warp handling](https://github.com/pret/pokecrystal/blob/master/engine/overworld/warp_connection.asm)

## Rebuild and check

Use the parent listed in the edition's build receipt, with a fresh output path. The builder verifies both the parent and the written archive. Keep packs outside version control.

```sh
CARGO_INCREMENTAL=0 cargo build --locked -j1 -p crystal-assets --features pack-authoring --example pack_trail_battles
target/debug/examples/pack_trail_battles "$parent" "$child"
```

Run the checks beside this file with Python, Pillow and cbor2. The geometry and script checks operate on the compiled archive and require no play session. The Fen journey check traces all six search orders through the room and the post-reveal return. The preservation check compares both map registries, the native art at three times of day, trainer settings and regional data.

```sh
python check-preservation.py "$parent" "$child"
python check-trainers.py "$child"
python check-caves.py "$child"
python check-handoffs.py "$parent" "$child"
python check-fen-bank.py "$parent" "$child" "$town_returns_edition"
python check-fen-journey.py "$child"
python check-fen-dialogue.py "$parent" "$child"
```

The three external-archive regressions exercise the production trainer detector, battle-party construction, native cave transitions and save restoration. Run them for each edition from the repository root.

```sh
export MN_TRAIL_BATTLES_PACK="$child" MN_CAVE_RETURNS_PACK="$child"
export RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0
cargo test --locked -j1 -p crystal-assets --lib -- --ignored --test-threads=1 northstar_trail_battles:: northstar_trainer_lanes::
cargo test --locked -j1 -p crystal-core --lib dig_warp_memory_
```
