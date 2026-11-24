# Iron Range cave details

The [Crystal atlas](../crystal-reference.html) separates three ideas that are useful in the Iron Range. Mt. Mortar gives its side trips a distinct terrain and uses drops to make the return different. Slowpoke Well puts one strong feature in view at each landing. Rock Tunnel repeatedly changes the direction of a through trip as it winds around solid rock. Those ideas inform the rooms below, using Geothite’s own layouts and native art.

| Room | Landscape and actual route |
| --- | --- |
| Route 9 adit | The lower arrival’s stone shoulders lead toward a usable western stair. A layer of ore near the foreman ties the survey bench to the passage below. The descent at walking squares (10,8) and (11,8) jumps the blocking face and rejoins the ordinary grade two squares south. Going back up takes the stairs. Nell still watches the separate shaft that opens after the foreman. |
| Drill Approach | The central rock rib now reaches into the open ascent. Its exposed vein links the grade to the adit without making the climb look like a work floor. The Ether sits in a shallow recess beside the upper stair. The upper and lower stairs have matching floor landings and the existing trainer rays still have a clear line. |
| Haulage | Two connected sidings run from the actual lift landings into their own workings. The west track passes the weigh guard’s sightline. The east track ends at a parked ore car whose tile blocks movement, before the drainage cut. The pressure shutter still closes the central crosscut between the sidings. Metal approach pads align across that opening without suggesting one long rail across a closed gate. |
| Sump | Rock faces contain the dark west trench. A timber footing around the lower return turns toward the dry towpath. The hidden waterline cache on the isolated rock still requires reaching it across the water. The other basin has a short service walk beside the valve, with the discharge joining the actual east rock face. The upper pump shutter opens only when both pressure circuits are off. |
| Ventilation | The upper fan bank meets an attached rock shoulder. Below, a continuous metal grating runs past the pressure control and ends at the existing crosscut shutter. The fan, pressure station and lower return are visible at separate landings rather than looking like one interchangeable square room. |
| Pump Gallery | The inspection walk drops around an attached rock spur and climbs back up toward the feed control. Two decorative grilles previously looked like operating shutters. They have become a continuous stone face. The true lower shutter opens a shorter return after feed isolation, and the other live shutter opens the approach to the Drill Core. |
| Drill Core | Gray floor panels lead from the arrival toward a painted observation line. The paint stays flat against the original day-lit stone. The wide front of the machine supports the existing walk, observation and rescue choreography. The upper panel stays accessible from its retained turning branch and the final machinery frames keep their authored terrain. |

The [layout rows](mines.json) use walking collision from the selected native tilesets and the [art additions](mines-art.json) describe original two-bit glyphs and metatile compositions. All existing object coordinates, warp records, scene triggers, mutation positions, and script opcodes carry through from the wall finish.

Run the [mine flow audit](review_mine_flow.py) against the selected wall finish and installed detail pack. The audit reads the compiled callback flags and ladder target records, checks both movement capabilities across 32 story states, verifies the two pressure controls and guarded lift sequence, and checks the exact Drill Core movement programs on three machinery terrain states. It also decodes the native two-bit rail glyphs against the published grayscale sheet, checks both rails at each join, and measures the two ordinary uphill returns and the opened Pump Gallery shortcut.

```sh
uv run --with cbor2 --with pillow python planning/minnesota/caves/crystal-detail-v04/review_mine_flow.py \
  content-packs/minnesota-cave-wall-finish-v03.crystalpack \
  content-packs/minnesota-cave-detail-v04.crystalpack
```

The native-scale reviews use the actual source-art palette and the two-bit glyph bank with retained actor positions. See the retained [landing, room, and shutter comparisons](../../../../target/cave-detail-mines-v04) for the visual iterations.
