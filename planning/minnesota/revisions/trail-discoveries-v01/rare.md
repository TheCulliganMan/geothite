# Minnesota’s optional wild battles

Both Field and Intro contain ten battles started by talking to a Pokémon on a Minnesota map. The two Northwoods encounters have their own Tall Tale audit. The other eight previously asked “Step closer?” or “Go closer?” immediately before starting a wild battle. At Lake Pepin the player had just watched a still Barbelisk with its eyes above the water. Choosing to look closer committed the player to its level-55 battle.

All eight now keep the original observation and end with “Battle [name]?” The player can watch the creature, decline and change their mind on the same visit.

| Site | How the creature appears | Creature and level | What the player sees before choosing |
| --- | --- | --- | --- |
| Route 3, Incisor Gallery | Read the pool, split pillar and bed beyond the Strength boulder | Castorock 24 | Pins the branch with a paw and looks up |
| Route 8, Ancient Mire | Read the timber, footprint and moss | Vergloom 40 | Water drops from the moss on its shoulder across the pool |
| Lake Pepin, sheltered inlet | Compare the dock sketch, eddy and reed observations with the boat operator | Barbelisk 55 | Holds still with its eyes above water |
| Field Margin | Correct the notice, then discover the creature along the furrow | Kensigard 50 | Turns its shields toward the player |
| Former Shores, Inner Basin | Compare the shores, set the three dials high/middle/low and open the outlet | Kensigard 60 | Lowers its ridged head as the basin drains |
| Layered Fen | Compare the layers, then follow the old root, pool and living rib | Mitwist 55 | Stays among the roots |
| Boundary Intake | Verdigrim appears during the drill incident. The invitation unlocks when the clean bypass has contained it | Verdigrim 60 | Lifts its head from the copper face, scattering grit |
| Devil’s Kettle | Release Kettlurk at the intake and finish restoring the water | Kettlurk 60 | Opens its freed fin and drifts within reach of the bank |

Two different Kensigard have independent discoveries, levels and catch flags. The old Inner Basin, Layered Fen and Pepin actor labels still contain Strandrak, Flarkindle and Pepivern, respectively. In both editions the field sprites, native cries, written observations and actual battle requests use the living species shown above. None of those three names or Icemyth appeared in the native Minnesota dialogue scan. The State Fair service stair remains usable. Its room has no exhibit actor or encounter.

For every row, the installed script plays the named Pokémon’s cry only after Yes. Catching it records the site’s catch flag and keeps it absent on future visits. An uncaught fight withdraws it for the current visit. Re-entering the map makes it available again with its discovery intact. Castorock and Vergloom use the native object callback for that return. The other sites use their new-map callback. The engine’s postbattle reload and Continue callback lists skip both of those phases. The last two scenes at the intake and falls keep their normal story outcome even if the player never battles either Pokémon.

Run the compiled data audit from the repository root, once per edition. It reads both archives and compares the encounter contracts, printed species observations and outcome branches.

```sh
uv run --offline --python 3.14t --with cbor2 --with pillow python planning/minnesota/revisions/trail-discoveries-v01/check-rare.py content-packs/minnesota-intro-trail-discoveries-v01.crystalpack --parent content-packs/minnesota-northstar-intro-v03.crystalpack
uv run --offline --python 3.14t --with cbor2 --with pillow python planning/minnesota/revisions/trail-discoveries-v01/check-rare.py content-packs/minnesota-trail-discoveries-v01.crystalpack --parent content-packs/minnesota-headwaters-journey-v01.crystalpack
```

The audit reads the installed scripts, sprite assets, native battle definitions, print pages and map callbacks. Permanent capture follows the game’s caught bit. Wild rolls, fishing, gifts and controller play are outside this fixed-encounter measure.
