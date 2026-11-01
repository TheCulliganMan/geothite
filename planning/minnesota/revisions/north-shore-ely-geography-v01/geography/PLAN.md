> SUPERSEDED: the user requires North Shore to enter Grand Marais from the west. This document and the old southern-quay/R11-seven-cell/Cedar8-wide-neck source epochs are immutable design history, not the active plan. Active geometry is `west-entry-placement-v02/proof.json`: GM[246,-400], Cedar[227,-419], a65×19 inland connector, town north port18/19. See `WEST_ENTRY.md`.

# Direct Superior coast and inland winter

Status: detached authoring proposal. The exact warm parent is `minnesota-superior-warm-coast-v01.crystalpack`, SHA `ebea77ccb9b9dbe54aa5101e32590c1b075ed8130f6e7fcfe2d8c8451dab11d8`. Native publication, controller traversal and actual player-camera acceptance remain separate. The approved cold-pine correction and additive Ely publication must land before the final contracts are rebound and frozen.

## Outdoor placement

All coordinates below are **metatiles**, with Minneapolis anchored at `(0,0)`. Core collision/event coordinates are twice this scale.

| Map | Origin | Size | Physical role |
| --- | --- | --- | --- |
| R11 whole | 190,-400 | 56×96 | Warm Superior coast; existing cliffs, falls, outdoor stairs and forest retained |
| Grand Marais | 202,-419 | 20×19 | Warm town immediately north of R11; lighthouse headland, fenced quay and Superior bay |
| Cedar Approach | 170,-419 | 32×19 | Inland west branch; warm eastern eight columns, then snow-covered ridge and paired pines |
| New Inland Neck | 162,-419 | 8×19 | Short snowy forest saddle joining Cedar to the existing Gunflint east entrance |
| R12 whole | 98,-443 | 64×56 | Substantial snowy Gunflint crossing with lakes, portages, shelters and intact Cut cache |
| Boundary Waters | 66,-427 | 32×24 | Pine-and-lake valley; existing preserve and safari journey retained |
| R13 whole | 66,-403 | 32×64 | Second substantial snowy ridge ending at Ely |
| Ely | 71,-339 | 20×18 | Existing authored rest camp and five-member League, added through its guarded native builder |

`placement-v03/proof.json` independently proves all 100 exterior maps after the coast move, and all 101 after adding Ely: one reached component, zero inconsistent connection cycles, zero overlapping rectangles, every reciprocal offset exact. Only Grand Marais and Cedar move; the eight-wide Neck is new. All other 97 original exterior footprints remain fixed, including the Boundary Waters/R08/Duluth branch. This proves geometry, not collision or gameplay.

## Actual coastal ports

R11 `MnR11C00R00` north connects directly to Grand Marais south. The offsets are `+12` from R11 and `−12` from town. A single paired direction carries both walking and surfing:

- Foot: R11 core `(40/41,0)` ↔ town `(16/17,37)`. Existing R11 Foot terrain stays exact; town author supplies a fenced stone quay from its mainland to this port.
- Superior: R11 core `x48..61,y0` ↔ town `x24..37,y37`. Seven exact R11 metatiles open the existing reef: `(24,0)30→58`, `(25..30,0)31→35`. The left bank uses the exact actual water-bank art beneath it. Meta31 remains a visible buoy cap; the next eastern chunk's north reef stays closed.
- Town's former east exit closes visibly with native reef/rail art. Its west lake edge closes because the new inland neighbor is forest. Six real building doors stay paired with their actual interiors.

The town author owns the prerequisite officers and all new water gate lanes. Both the Foot and Surf departures must honor the lighthouse prerequisites; returning from the coast for care remains possible. Gate visibility and directional interactions require production tests, not a connectivity graph.

## Inland play sequence

Leaving town west enters Cedar at core `(63,8/9)`, then follows a winding two-core-tile trail through the warm pines. The snowline occurs after travelling inland, entirely outside Grand Marais. The eastern eight Cedar columns remain warm, and the northern edge of the snowline steps through the trees instead of covering the coast.

Cedar retains the existing Keeper trainer on its main snowy clearing. The existing Walker trainer moves from `(53,31)` to `(55,17)` on a smaller irregular granite outcrop. Native cave stairs at meta `(27,10)` are the real entrance to this optional battle; the main path continues below the rocks. No new trainer party, actor, event flag or dialogue program is allocated. The sign's two stale directions become `GRAND MARAIS east.` and `GUNFLINT west.`. No cross-map vista is claimed: the outcrop's purpose is the optional existing trainer battle.

The actor-aware source collision view reaches every one of Cedar's 431 walkable quadrants, both trainer approaches, and both edge ports. The current shortest edge crossing is 105 steps. Old south-coast terrain is visibly closed. Eight source-art camera crops cover the Neck, inland seam, west port, widest Keeper clearing, snowline, cave stairs, summit trainer and warm town-side port. They are source crops, not GPU or clamp acceptance.

The eight-wide Neck is a 27-step bend through snowy trees, with mandatory icy-colored grass at the saddle. Its only outer walking tiles are west/east core rows8/9. Every one of its 56 walkable quadrants is connected. It contains no water, sliding ice, NPC, door, callback, trainer, item or encounter program. It needs a real native map registration, not an invented 32-wide chunk.

R12/R13 winter authoring belongs to the separate `inland-winter-r12-r13` proposal. Snow floor remains ordinary Floor; icy-colored grass remains encounter grass. Sliding ICE, if used, needs a deliberately bounded course and native stopping/return proof. Existing enclosed lakes remain surfable water, with their banks, shelter doors, Cut replacement, trainers, cliffs and return routes preserved. Boundary Waters stays a distinct lake valley between the two snowy route regions.

## Native publication contracts

- Cedar/Neck: `cedar-neck-v03/contract.json` is the current source epoch before the precise cold-pine replacement. It changes 486 Cedar metatiles plus the two sign directions and only the Walker's position. Existing events/programs otherwise stay exact. All source epochs and renders remain saved.
- R11 aperture: `r11-aperture-v01/contract.json` records exact warm module preimage, seven terrain cells and one north-connection replacement. All other 761 cells and every other field stay exact.
- Cold pines: parent will append a single native recipe `8b` preserving the narrow paired johto05 pine silhouette. Both interim86/87 canopy placements in Cedar/Neck must become8b; neither old alias prefixes nor shared native glyph references may change.
- New Neck: `MnMaraisInlandNeck`, constant `MN_MARAIS_INLAND_NECK`, native group40 `MINNESOTA`, unused mapId8, dimensions8×19. Reuse existing Grand Marais landmark. Native typed extension inserts ordinary empty script/event transport headers. Temporary recovery280 is removed after registration; this map adds no retained recovery or new-game start.
- Add Ely first: exact warm parent passes all eleven existing Ely module preimages, eight tileset preimages, town/layout byte hashes and five-member League subtree hash. Group81 and all ten map names are unused. Actual next event is2385, max recovery277. Allocate League events2385/2386 and recovery278/279, add ten maps, preserve every unrelated module/art/audio/catalog. `ely-parent-v03/receipt.json` records the independent certification.
- Rebind final coast/Neck/winter contracts to the actual published additive Ely/cold-pine parent. Guard owned modules against exact before hashes and all unrelated fields against full residual equality. Do not weaken the existing Ely source/template/tileset preimages to transplant an alternate pack.

## Acceptance still required

Native read-back must prove every declared cell, atlas/registry mirror and program mutation, with full unrelated residual preservation. Check the new direct Foot and Surf seams in both directions; lighthouse failure and success on every departure lane; all six town room returns; town-side care return; inland Foot travel in both directions; both Cedar trainers and its sign; optional stair approach and return; all R12/R13 doors/Cut/grass routes and cold-port return; and R13/Ely registration. Save/Continue must preserve both map transitions and relevant flags.

Render the entire connected warm coast and inland branch from the actual written pack, then production player cameras at both seams, town quay/lighthouse, winter transition, narrow grass saddle, summit trainer, R12 lake portages and R13/Ely entrance. Compare all applicable time banks. Keep all failed captures and every render epoch. Static source proof is not a claim that the game was played or that the production camera was inspected.
