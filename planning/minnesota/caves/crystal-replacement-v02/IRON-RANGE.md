# Iron Range cave replacement

The network begins at Route 9's existing surface adit and still finishes at
the Drill Core. Every underground floor has a new footprint. The two pressure
stations and the feed isolation still control the original stage transitions.
The wagon routes, maintenance walks, intake pool and final observation line
give the rooms different work to do.

| Interior | New size in metatiles | Route |
| --- | --- | --- |
| Route 9 adit | 16 × 11 | The entry climb follows an inclined seam to a survey stair. The foreman overlooks the fork and opens the actual shaft on the far shelf. Nell and the cache retain their original roles. |
| Drill Approach | 12 × 17 | A narrow rising drift folds around a thick central rock rib. A stair at each bend connects its levels. The Ether sits where the west wall opens near the upper stair. |
| Haulage | 24 × 10 | Two long sidings meet at a pressure shutter in the cut. The west station groups the weigh sample and two lifts. The east station has its own control and a narrow runoff trench. |
| Sump | 15 × 15 | Walk a west towpath past a long flooded cut. The hidden cache faces water from an isolated rock. The separated dry shaft leads to the pump shutter and can be reached from either landing on foot. |
| Ventilation | 14 × 17 | Parallel maintenance ducts turn around a central rock mass. The fans sit on the upper landing. The panel opens the only local crossover. |
| Pump Gallery | 20 × 11 | A long inspection loop links the inlet to the feed control. Feed isolation opens the lower service crossing and the separate shutter to the core lift. |
| Drill Core | 16 × 15 | A narrow passage climbs from the lift toward a brightly lit drill theatre. Every approach crosses the observation line. The translated machine and actor movements end on their existing corresponding positions. |

The [layout contract](mines.json) records rows, original block-input
hashes, every moved shaft or actor, and the guarded script operand changes.
Warp record IDs retain their destination pairings. Shaft drawing and the
passive landing are separate, as in the Fair Presentation map records.

Use the static mine verifier on the parent and installed candidate:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/caves/crystal-replacement-v02/review_mines.py \
  content-packs/minnesota-fair-presentation-v01.crystalpack \
  content-packs/minnesota-cave-replacement-v02.crystalpack
```

It checks both modes of traversal at every paired arrival, including partial
shutter states, guarded control and item exposure, closed perimeters, every
trainer ray, and the original source commands from each translated Drill Core
trigger. The source review used the same collision tables with the new rows
substituted in memory. Native source renders were made for the complete floors
and the 160×144 approach cameras.
