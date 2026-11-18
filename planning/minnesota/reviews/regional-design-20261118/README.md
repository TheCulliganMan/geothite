# Minnesota regional design review, November 18

Four reviewers independently examined the November 17 story release. Each chose their own examples and Pokémon Crystal comparisons. Their original submissions are [A](audit-a.md), [B](audit-b.md), [C](audit-c.md) and [D](audit-d.md). Each distinguishes its coverage and assumptions. The selected parent has 355 Minnesota maps. The reviewers inspected every numbered outdoor trainer and all 20 numbered-route hidden items, with closer visual and collision work on a sample of routes, caves, mine rooms, ice floors and the State Fair. This was a static design review. No gameplay inputs were sent.

## Reading the findings together

On **trainer sight**, A and D measured the authored starting pose and range. B and C also read the movement scheduler. Both observations matter. In the released game, a regional standing trainer without a direction override turns as a fast spinner and sees at least eight squares. The one-tile figure measures a map record. D’s Route 6 diagram therefore describes a choice suggested by the original layout under fixed facings. Its interception counts cannot establish what a player experienced in that release. Eight of the nine unusable initial facings point into adjacent walls. The ninth, on Route 12, points off the edge. Crystal’s Route 32 fishers show how fixed positions can make individual stretches of shore play differently. [Route 32 events](https://github.com/pret/pokecrystal/blob/master/maps/Route32.asm)

On **Voyageurs**, the reviewers chose different start squares, items and stopping conditions. They found an east-then-north pattern shared by the floors, a functioning slide requirement upstairs, and an easy walking approach to the basement Revive. The revision keeps the first-floor puzzle and the dry Revive route. It gives the optional PP Up its own visible rock and three changes of direction on the way to the shelf. Crystal’s Ice Path provided a reference for distinguishing the main crossing from optional item routes. [Ice Path 1F](https://github.com/pret/pokecrystal/blob/master/maps/IcePath1F.asm)

On **hidden items**, five caches were installed on 16-pixel squares identical to plain ground. In each case the tree named by the landmark occupied a different square inside the same 32-pixel block. One reviewer raised the possibility that Headbutt could intercept an item placed on the tree. The game resolves an unclaimed background item first. After collection the saved flag suppresses that event, freeing the normal contextual field move on the same tree. The regional builder tests both flag states using the production resolver.

## Twenty design decisions

| # | Place | Decision and reason | Status |
|---|---|---|---|
| 1 | Numbered routes | Let a road trainer's visible stance tell the player which lane starts a battle. | Implemented on all 125 |
| 2 | Routes 2, 5, 7, 10, 12 and 20 | Turn the nine trainers facing a wall or the map edge toward a real crossing. | Implemented |
| 3 | Routes 6, 8 and 13 | Give the trainers behind Cut a real sight challenge inside the pocket. Retain the tree and the reason to return. | Implemented |
| 4 | Border Pines, north | Put a frozen kettle lake and a grove on the den approach so the fork has a recognizable destination. | Implemented |
| 5 | Border Pines, middle | Open a second shore trail for players who want to avoid the long ice crossing. | Implemented |
| 6 | Border Pines, south | Give the Bemidji approach a choice of grass and shelter trails while preserving the earned Cut shortcut. | Implemented |
| 7 | Lake cabin, Route 15 | Connect the porch to the fishing terrace through the windbreak. A watcher and a visible item now have a place on that approach. | Implemented |
| 8 | Gunflint snow, Route 12 | Use snowy ground beneath the Cut sapling and the evergreen that remains after it is cut. | Implemented |
| 9 | Routes 6, 7, 12, 17 and 19 | Reward inspecting the actual spruce or sapling, with a reachable square from which to face it. | Implemented |
| 10 | Granite Den | Put the Spell Tag at the western lichen boulder. The eastern shelf keeps its visible Revive. | Implemented |
| 11 | Gunflint Grotto | Frame the outdoor return with stone shoulders and put the staircase graphic directly on the warp square. | Implemented |
| 12 | Voyageurs B1 | Give the optional PP Up an east-arm ice approach with a visible stopping rock and a safe return. | Implemented |
| 13 | State Fair | Establish a paved east-gate-to-grandstand avenue with room to pass. | Implemented |
| 14 | State Fair crowds | Group optional visitors by time and activity. Essential people and the 35 trainers keep their schedules. | Implemented |
| 15 | State Fair show ring | Reward leaving the avenue and searching the drinking fountain beside the benches. | Implemented |
| 16 | Iron Range mine | Signal the destination before each stair with a different landmark, such as water draining toward the sump or a wheel housing near the pumps. | Open |
| 17 | Route 10 survey cave | Restore the intended two-input downhill return on the exterior carrier ridge. The 37-step uphill route remains. A proposed second cave mouth would exit close to the first, bypass the pine approach, or leave the player on an isolated bluff. | Ledge repaired; extra mouth declined |
| 18 | Hidden landmarks on Routes 11, 14 and 20 | Match Route 11's headland clue to the shore steps and vary the later snow tracks so their endpoint is intelligible. | Open |
| 19 | Smaller caves on Routes 3, 13 and 20 | Give the riverbank stream, shelter pool and lower seep different shapes and relationships to the walking path. | Open |
| 20 | Routes beyond the numbered outdoor chunks | Review the remaining automatic spinners separately. Choose purposeful rotating lookouts where waiting or taking another trail makes a difference. | Open |

The physical map improvements are intentionally uneven in scope. Border Pines received changes across its connected route. The Route 15 work addresses the cabin approach. The other numbered routes received encounter or discovery changes according to the evidence. The individual reports also identify designs worth preserving, including the Route 8 thaw transition, Route 13 survey marker, Route 12 lake and healer, and the two-ended Route 20 cave. Reading static evidence cannot establish how difficult the battles feel or whether the full campaign now matches a mainline release.
