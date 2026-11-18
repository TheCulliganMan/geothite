# Independent submission A · Minnesota regional design

**Reviewed November 18, 2026.** The starting release was `minnesota-story-flow-v01`, selected by `planning/minnesota/world.json`. Its local pack SHA-256 was `02e18ba71203d8aa2e10658c6babc53512daeef592ee53fb2900630f91aa57cb`. I rendered the installed daytime terrain for all 20 numbered roads, inspected 17 cave and mine maps, checked all 125 trainers on the numbered road chunks, and examined the native 160×144 image around each numbered road's hidden pickup. These were static inspections. Save behavior, battle difficulty, dialogue timing and player perceptions were outside the evidence collected.

## 1. A player following the artwork can miss five hidden items

Five pickups sit on 16×16 squares that are pixel-for-pixel identical to plain grass or snow. Each has a single visible sapling elsewhere in the same block. The concrete mismatch is easy to see on Route 17. Its landmark is called “the fir standing alone above the ice,” but investigating the graphic at (25,2) misses the target at (24,2). The other four crops show the same displacement.

| Road | Pickup | Current target | Drawn landmark |
|---|---|---|---|
| 6 | Parlyz Heal | (8,30) | (9,31) |
| 7 | Mystic Water | (24,24) | (24,25) |
| 12 | PP Up | (12,39) | (13,38) |
| 17 | Repel | (24,2) | (25,2) |
| 19 | Great Ball | (54,21) | (55,20) |

**Repair the clue at the interaction square.** Each visible tree also accepts Headbutt. Fallen needles leading to a scuffed patch at its foot, a blue shell by the lake rail, or exposed soil in the prairie windbreak can give the pickup its own visual target and preserve both interactions. Route 15 already does this well. Its lone stump and the Ether event occupy the same quarter-block. An acceptance check should compare all four quarters of a placed block and confirm at least one legal adjacent position from which to investigate it. All 20 annotated pickup cameras (`a-evidence/caches.png`) · Native pixel differences (`a-evidence/caches.txt`)

## 2. The Voyageurs basement repeats the first puzzle's answer

The first-floor exit is a good compact introduction. The entry is in the southwest. A long eastward glide along the bottom edge and a northward glide near the east edge lead to the far ladder. The basement has a different outline and different rocks, but the same east-then-north movement leads from its southwest arrival to a square facing its PP Up. The installed ground and ice permit these paths in ten and nine directional inputs respectively. Both require just two long glides. This is a static path result under conservative passability, with walls and ledges excluded. It establishes that both paths exist.

**Give the basement a different idea.** Let the first eastward glide, which players now know, end with a view of the prize across a narrow crack. Moving one stop into the interior could set up a westward rebound before the final north glide. Preserve the return to the ladder and the hidden-item square. In a static route check, require different stopping sequences on the two floors and a path back from each reachable stopping square. Crystal's Ice Path mixes compact teaching fields, awkwardly positioned stopping rocks and dry land between them. Static paths and tile counts (`a-evidence/ice.txt`) · Installed cave terrain (`a-evidence/caves.png`) · [Crystal Ice Path](https://github.com/pret/pokecrystal/blob/master/maps/IcePath1F.asm)

## 3. Border Pines needs recognizable places between its destinations

On the northern 32×24 block grid one evergreen block occupies 637 of 768 positions (83%). In the 32×8 southern approach a single block occupies 229 of 256 positions (89%). The trees themselves read well. In the overview they form nearly rectangular walls enclosing long, right-angle white avenues. The middle chunk already has ingredients worth building on, including an ice basin, hut and solitary fir. For comparison, even Crystal's long Route 32 breaks up its through-road with an irregular riverbank, a fishing pier, grass and rough ground. Its most frequent block accounts for 22% of the map. The difference in composition matters more than any percentage target.

**Give each fork a specific view.** Make the hut overlook the narrow outlet of the ice basin, let the main path follow one bank to a visible granite face, and curve a shorter tree-lined trail back onto it. Keep the existing den mouth and boundary crossings. At each of the resulting forks, inspect a single handheld-sized camera to see whether its destinations can be distinguished. 20 route overviews (`a-evidence/routes.png`) · Local camera comparisons (`a-evidence/cameras.png`) · [Crystal Route 32](https://github.com/pret/pokecrystal/blob/master/maps/Route32.asm)

## 4. Give trainers individual positions and reasons to look in a direction

84 of 125 trainers on numbered road chunks have a sight range of one tile. All ten trainers on Route 11, all ten on Route 20, all eight on Route 7 and all six on Route 10 share that value. Route 12 gives all ten trainers range two. These counts describe authored vision; they do not establish which battles are unavoidable. Crystal's Route 32 places eight trainers with both one- and three-tile ranges. Its fishing pier gives that difference a local meaning. A shoreline trainer can mind the water while another watches the approach.

**Rework a road by drawing sightlines onto its walkable terrain.** On Route 11, place one lookout at a genuine open crossing, put a second on a short side spur guarding its reward, and let the returning player see and choose a longer lane around one of them. Apply that judgment to each of the four homogeneous roads. Check directions and obstructions per crossing, including an alternate route if one is intended. Preserve automatic encounters. Installed ranges and reference positions (`a-evidence/detail.txt`) · [Crystal Route 32](https://github.com/pret/pokecrystal/blob/master/maps/Route32.asm)

## 5. Use the mine's destinations to distinguish similar junctions

The mine already has an interesting return network. Haulage has five stair positions leading to three destinations. The Sump also has five, including two separate links back to Haulage. In both rooms one rock-fill block accounts for just over half of the terrain. Similar gravel floors, facing rock edges and busy stair graphics make those alternative routes difficult to identify from a still image. The Drill Core's row of machine wheels is a strong counterexample. It identifies the room at a glance.

**Tie exits to landmarks belonging to their destination.** The drain exit could follow visible run-off; a wheel housing could identify the pump approach one camera before the stairs; a broken ore trolley could frame the Haulage shortcut. Keep both alternate connections. Crystal's Union Cave links to two different B1 locations in a compact floor, with pools, pockets, actors and items helping distinguish the paths. Mine and reference cameras (`a-evidence/cameras.png`) · [Crystal Union Cave](https://github.com/pret/pokecrystal/blob/master/maps/UnionCave1F.asm)

**Priority order.** Fix the five misleading cache targets first. Revise Voyageurs B1 next because it duplicates a puzzle solution despite having distinct art. Use Border Pines and Route 11 as the first maps for the wider route-art and trainer-placement passes.

Signed, independent regional reviewer A.
