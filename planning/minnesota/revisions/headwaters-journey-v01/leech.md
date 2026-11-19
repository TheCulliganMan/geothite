# Brainerd and the road through Leech Lake

Sivren and the ranger-station archivist send the player to Brainerd's fishing pier after the Lake Badge. The ranger was standing at (25,19), on the inland plaza. She now stands on the bank at (49,20), beside the pier entrance. From (49,21) the player can talk to her with the lake and the angler in view. Both boardwalk lanes stay open. Crystal stages Lance beside the actual body of water at the Lake of Rage, where the location helps identify the person the player came to find. [Lake of Rage map and script](https://github.com/pret/pokecrystal/blob/master/maps/LakeOfRage.asm)

On Route 7 the birder at (13,39) previously described a hop down to the sandbank. Neither route section contains a jump ledge. She now points southeast to the stone steps. The three steps from (17,41) to (17,44) work in both directions.

The Field edition had four headbuttable saplings in the Leech Lake fishing platform and five in the Mille Lacs platform. Its two platforms now use the same four passable timber quadrants as its own Route 18 dock. Mothlight already had that timber. The genuine Route 7 sapling at block (12,12) remains usable. The checker reads the actual board and end-grain pixels and palette from each edition and compares them to the source Kanto timber. [Original landing placements](../cliff-faces-v01/places.json)

At Leech Lake, the fisher's conversation square (35,20) and the water-facing board (38,21) are accessible from the south road at (32,47). At Mille Lacs, both the angler's conversation square (27,22) and the end of the dock at (29,23) are accessible from the west road at (0,12). Both piers let the player walk back from the water's edge.

## Compiled checks

- The chapter road runs from St. Cloud Gym through Route 6, Brainerd, Route 7, Leech Lake and Route 8 to Bemidji's field officer. The checker traces it in both directions on foot, accounts for actors and map-edge offsets, and checks the ranger from all four public entrances to Brainerd. It verifies both exit carpets for Brainerd Gym, the ranger station, Leech's care cabin and the fisher's house.
- Of the 23 chapter trainers with sight triggers, 20 can see the player on a dry public path, two beyond functional Cut trees and one on the Surf island. The lake's eastern cache still requires Whirlpool. None of those field moves is needed for the main road.
- The ranger requires the Lake Badge for her reading. A pre-owned Surf HM counts as delivery without giving a second copy. The checker traces all 16 combinations of the badge, report, gift and comparison flags with zero, one or 99 HMs. A full ordinary Items pocket leaves the dedicated TM/HM slot available. A refused delivery preserves the reading so the player can retry. Teaching Surf and using it in the field retain their original move and badge requirements.

Run the [compiled chapter checker](check-leech.py) with a Headwaters Journey pack and `--parent` followed by its Station Return edition. For the original positions and Field saplings, run it against Station Return with `--baseline`. These are script, art and movement checks on the archive.
