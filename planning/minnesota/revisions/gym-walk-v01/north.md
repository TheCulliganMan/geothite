# Brainerd’s beach and Bemidji’s garden

## Brainerd

The Intro edition of Badge Trails used one blue water pattern and square patches of sand. Lorla mentioned Yodeloon’s starting line, and Bravel asked for his red float. Neither landmark appeared on that map. From the entrance the player could walk through the blue field, but the artwork gave no indication which water was shallow.

Intro now uses the Minnesota swimming-beach art already present in Field. Both editions previously had still water. A line in the west sand marks Lorla’s spot, a red float sits beside Bravel, and the light central shallows lead north to Sivren. Nineteen darker squares form three optional canoe pools. All four characters stand on sand. The center crossing from the city arrival at (8,12) to the leader’s south side at (8,4) takes eight ordinary steps. Taking either side puts a challenger in a regular trainer’s sight. The center route goes straight to the leader.

Eight native ripple glyphs animate 39 spots across both shades of water. Each has four 8×8 frames, 22 ticks apart. The first frame is the installed beach artwork. As Chris or Kris walks through the shallows, the water hides the bottom of each wet boot and catches the light beside it. On a mixed sand-and-water tile, the dry foot stays visible. The red float, dry banks and sand islands keep their shape. The original Crystal Cerulean room made its walking surface identifiable with a raised deck through the pool. The beach uses the color of the lake bottom to answer the same question. [Crystal’s Cerulean gym](https://github.com/pret/pokecrystal/blob/master/maps/CeruleanGym.asm)

## Bemidji

Both little trees can be approached on the first visit. The left tree opens Moslen’s clearing and Brindel’s optional shortcut match. The long path along the right wall reaches Elvren’s tree. The obstacles behind the entrance open after Orlis awards the badge. The earlier guide explained the two required fights during the HM gift and immediately repeated those instructions. On later visits the guide still told the player to win both fights, even after they had done so.

The gift now explains where to teach Cut. The guide then names the two paths. On returning, players hear which keeper remains, where to find Orlis once both have vouched for them, or how the middle gate leads back to the door after winning the badge. The gift’s pocket check and retry path remain intact. Champions still receive the existing rematch circuit.

| From the door at (4,17) | Actual first approach |
| --- | --- |
| Moslen | Five walking steps to (3,13), facing north toward the little tree at (3,12). After cutting, Moslen sees (4,11) or (4,10). |
| Elvren | Seventeen walking steps to (7,7) along the right side, facing north toward the tree at (7,6). After cutting, Elvren sees (8,5). |
| Brindel | Step into (3,8) from the west clearing for his match. Moslen can spot the player without crossing Brindel’s line. |

In both Crystal and Polished Crystal’s Celadon Gym, the opposing students face inward across the gardener’s path and the player talks to Erika at the back. In Bemidji the trees lead to two separate clearings. The guide’s new directions preserve the choice of order and identify the optional side match. [Crystal’s Celadon gym](https://github.com/pret/pokecrystal/blob/master/maps/CeladonGym.asm) · [Polished Crystal’s Celadon gym](https://github.com/Rangi42/polishedcrystal/blob/f8e67fbd53ad7a99870d8ab87fd73612e6c6a679/maps/CeladonGym.asm)

Run `python north.py PARENT.crystalpack CHILD.crystalpack` with Pillow and cbor2. The check reads compiled map-entry scripts, physical movement squares, trainer ranges, guide branches, 2bpp water frames and native sand pixels. Its 16 guide states and eight terrain states cover both keeper orders, the optional match, repeat visits and the earned exit. These are stationary archive checks. The separate native gym sight test starts every teacher’s actual battle on direction-only entry.
