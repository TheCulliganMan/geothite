# Driftless Hollow

Two ladders led outdoors and a third led down to the shell bed. At the west fork the outdoor ladder and the gallery ladder were visible together, directly below a guide who sent players to “the ladder.” The two Route 1 exits now show daylight through the south wall. The guide points to the ladder below the east stone steps when giving FLASH and on later visits.

| From inside the Hollow | From Route 1 or the lower gallery |
| --- | --- |
| From (9,10), walk south onto the lit floor at (9,11) and continue south to Route 1 at (18,18). | The mouth at (18,17) returns to the west exit. |
| From (31,12), walk south onto the lit floor at (31,13) and continue south to Route 1 at (36,20). | The mouth at (36,19) returns to the east exit. |
| The stone steps at (10,7) lead to the ladder at (11,9). It descends to the gallery at (18,17). | The ladder at (19,17) brings the player back to clear floor at (10,9). |

Crystal's Union Cave uses a south-facing floor exit with light at the edge of the screen for both outdoor connections. The passages to B1F use ladders. Polished Crystal rearranges the room and keeps that distinction at all four warps. Driftless Hollow reuses Crystal’s two daylight tiles in the local limestone. [Crystal’s Union Cave](https://github.com/pret/pokecrystal/blob/master/maps/UnionCave1F.asm) · [Polished Crystal’s Union Cave](https://github.com/Rangi42/polishedcrystal/blob/master/maps/UnionCave1F.asm)

Run the static archive check once per edition, supplying the matching Shore Watch parent and Badge Trails child.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/revisions/badge-trails-v01/check-hollow.py \
  content-packs/minnesota-intro-shore-watch-v01.crystalpack \
  content-packs/minnesota-intro-badge-trails-v01.crystalpack
```

The checker measures 26 ordinary steps between the two indoor approaches in either direction with every actor present. It checks the gallery round trip, a reachable stance at the fossil wall, the guide’s installed pages and the existing badge, item and full-pocket branches. It also checks every changed native block and permission, the two map-block copies, seven original Crystal surface exits and all 14 Minnesota surface cave mouths. These results come from the installed archives and their movement data.
