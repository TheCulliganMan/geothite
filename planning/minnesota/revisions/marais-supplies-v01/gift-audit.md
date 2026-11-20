# Minnesota gift audit

Tamarack’s field-station adoption was the only regional map gift we found that could permanently skip part of a promised bundle. On Ely Return, accepting Birchit attempted five Poké Balls and three Potions. Either pocket could reject its whole stack. Neither item result was checked before the station marked the adoption complete. The ordinary return conversation then discussed the lighthouse or the skipper. A player could get Birchit and neither supply, or get only one supply, with no way to collect the rest.

| Ely Return edition | Minnesota maps | Item gifts | Pokémon gifts | Egg gifts | Unchecked results |
| --- | ---: | ---: | ---: | ---: | ---: |
| Field | 355 | 94 | 10 | 1 | 2 |
| Intro | 357 | 96 | 10 | 1 | 2 |

Both Marais Supplies archives retain those gift counts. All 105 Field and 107 Intro gift instructions now branch on the delivery result. Tamarack records and confirms the Balls and Potions separately, so a player can return for whichever pocket was full.

Each count is a gift instruction with a corresponding compiled runtime entry and a local path from an authored object, tile, scene or map callback. An instruction may represent a choice, a retry or a condition that only applies later in the story. The inventory covers direct map gifts. Shops, capture, item balls, hidden-item searches and scripted coin payments use separate operations. The regional maps were identified by their Minnesota map groups. [Reproducible census](audit-gifts.py)

- **Other partner handoffs.** The Winona starter balls, the aide’s later partners, the Rochester patient’s Pipulse, and the researcher’s two fossil revivals all branch on the native party-and-PC-full result before granting progress. Winona’s Loobit Egg uses the Egg-specific success value. Winona’s five travel Balls have their own full-pocket branch and receipt. The parent archive contains a separate physical entry for each starter pedestal, and both fossil-restoration branches start at the named researcher.
- **Fossil exchange.** On Route 1 and in Rochester’s lab, the two possible fossils at each location jump to a shared result handler. Its first instruction tests the item result. Only a successful delivery sets the exchange receipt. Looking only at the instruction immediately after each gift would incorrectly identify four failures.
- **Lighthouse.** The keeper consumes the Machine Part and repairs the light before attempting one Super Potion. If making space is necessary, he sets a separate pending flag. Talking to him again retries the potion while keeping the repaired lighthouse and spending no further parts. [Archive checks for the station and keeper](check-outcomes.py)
- **Open follow-up in the Driftless Lamp Gallery.** If the Items pocket cannot hold the Rare Candy, the cache closes without a message. The item remains visible and its collection flag stays clear, so it can be collected later. The success path alone plays the pickup sound, marks it taken and hides it. A short “make room” message would tell the player why nothing happened. The field object is `MN_LAMP_GALLERY_CACHE`; its failure branch is `MnLampCacheClose`.
