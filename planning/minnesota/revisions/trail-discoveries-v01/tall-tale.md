# Bemidji’s Tall Tale Trail

The Northwoods gate can be reached from Bemidji, Route 17 or Layered Fen. Both marked lanes through the gatehouse work in both directions. Its attendant tells the player to check in with the road crew. The trail-side arrival is (3,34). From there the following landmarks can be reached and left on foot. Coordinates use 16-pixel movement squares.

| Landmark | Where to stand | What happens |
| --- | --- | --- |
| Road worker (10,34) | (10,35), eight steps from the gate | Introduces the three observations. After partial progress the worker names a missing clue. |
| Culvert (23,29) | (22,29), facing east | Records the pipe. |
| Hoofprints (17,15) | (16,15), facing east | Records the tracks. |
| Stacked wood (28–29,20) | (28,21) or (29,21), facing north | Either half records the stack. The three observations can be collected in any order. |
| Loose log (28–29,16) | (28,17) or (29,17), facing north | Once the worker compares the clues, either half offers to move the log. Declining leaves the trail closed. Accepting brings Bunjant into view at (29,15) and clears a two-square-wide path. |
| Marked log (28,8) | (28,9), facing north | Records the account the worker needs. The nearby camp post is optional. The living tree immediately east of the log still supports Headbutt. |

Before the log moves, the pond supplies no Surf shortcut into camp. Both walking lanes remain open on return visits. The worker directs players to the marked log until they read it. Bringing the account back reveals Babehemoth (18,13) and Bunjant (30,8). Neither creature blocks the route home.

Both encounters explicitly ask whether to start a battle. They begin at a minimum level of 60. Declining leaves the encounter available immediately. After a successful escape or a knockout, the creature returns on a fresh map visit. Catching it permanently removes that creature and preserves the other encounter. Continuing a save in the clearing honors the current visit.

The archive check traces six clue orders, all eight clue subsets, both encounter orders and 16 reveal/capture combinations. It also compares the saved tile-change instructions with the map callbacks that draw and clear the log. A disagreement there made the first draft display the wrong closed log. These results come from the installed map, scripts and collision data. [Repeatable archive check](check-tall-tale.py) · [Existing controller coverage](../../../../crates/crystal-bevy/src/bevy_shell/tests/northwoods_discovery.rs)
