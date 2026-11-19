# Supplies on the way to Ely

At Duluth Dock Supplies the clerk opens the shop by asking how much rope the player wants. A customer beside the counter orders one Potion for their boat. Both Gym Walk editions offer Full Heal, Hyper Potion and Revive at that counter. The new stock adds the two items the people in the room actually name. An Escape Rope costs ₽550 and a basic Potion costs ₽300 for 20 HP. The original Hyper Potion remains available for ₽1,200.

The dock shop is reached from Norek’s gym by walking 100 outdoor squares, then entering the door at (12,77). The indoor landing at (6,9) is five steps from the clerk’s counter and three from a conversation stance beside the customer. The street return leads back outside at (12,78). The counter uses the game’s usual money and Bag handling.

## The two approaches to the League

| From | Walking steps to Ely's League entrance | Tall-grass steps along that shortest path |
| --- | ---: | ---: |
| Boundary Waters, at the southern gate | 225 | 30 |
| Duluth, outside the eighth gym | 243 | 24 |

These measurements use the installed road connections, actors and open gate callbacks. Tied shortest paths can cross different amounts of grass. The three Route 13 sections use a base grass encounter rate of 10 at morning, day and night. Their grass slots range from level 43 to 48. Crystal’s Victory Road has a base rate of 15 in its cave and encounter slots from level 32 to 40. Those rates describe eligible encounter rolls. A trainer battle, Repel, lead level and other field state can change what a player actually meets.

Ely already sells the seven items in Crystal’s Indigo Plateau shop. Its clinic and its Mart are each a 22-step street walk from the League’s west entrance, and the registrar gives both directions. Bemidji and Duluth already stock a Hyper Potion, Full Heal, Revive and Max Repel before that point. A restored team and a bag of late-game supplies are available without a remote shopping trip. Crystal’s Cianwood pharmacy also carries basic and Hyper Potions alongside Full Heal and Revive. [Crystal’s shop inventories](https://github.com/pret/pokecrystal/blob/master/data/items/marts.asm) · [Indigo Plateau’s nurse and clerk](https://github.com/pret/pokecrystal/blob/master/maps/IndigoPlateauPokecenter1F.asm)

Run `provisions.py GYM_WALK_PACK` to reproduce the original shop and the two approaches. Add the corresponding Cave Returns pack as a second argument to verify the corrected clerk, the two conversations, the shop approach and the full shop-catalogue difference. The native test `cave_returns_dock_clerk_sells_the_rope_and_potion_from_the_room` opens this exact actor's registered sale. It checks refusal at ₽549, then buys both goods for exactly ₽850 and verifies both Bag counts.
