# Northern cave replacements

The four interiors in `north.json` use the Fair Presentation archive as their
source of existing events and terrain art. Each drawing starts from a new
footprint. Exterior warp records keep their targets while the corresponding
interior landings move with the routes.

| Room | Footprint | Main route |
| --- | --- | --- |
| Winding Cleft | 25 × 11 | The west landing climbs successive shelves above a long flooded fault. The east shore reaches a lower trail that drops onto the west landing. The walker takes 64 steps eastbound and 43 westbound before counting warp entry. |
| Gunflint Grotto | 18 × 14 | The ladder lands inside Otta’s dry camp. A single stair descends to the spring, and the walking route rounds the lower bank before climbing to the far supply shelf. The rock stranded in the spring holds the Surf-only Revive. |
| Ore Line | 16 × 24 | The reader stands at a landing halfway up the fault. The upper cuts lead to the fracture and its reward. Two southward hops provide a return when the Rock Smash boulder has returned. The lower route climbs down to the spring and circles it to the supply. |
| Inner Basin | 20 × 16 | Three indicators occupy different banks around the permanent lower lake. The lever drains a forked upper spillway and opens its sill. The sanctuary and Kensigard are accessible only after that change. |

The low bank in Winding Cleft is a return route within the cave. Its westward
ledge makes crossing that way 21 steps faster than walking back by the crest.
The west entrance still reaches the east entirely on foot. Hidden items in
Gunflint and Winding Cleft have no dry interaction tile even after obtaining the
other supplies. The lower casting banks remain usable.

Inner Basin uses the same nine script block changes at their new positions.
Eight water blocks drain and the ninth change opens the sill. The room has 334
reachable dry tiles before opening and 459 after. The existing three indicator
conditions and the once-only encounter receipt are inherited.

The following read-only verifier checks the actual compiled rows, moved event
records, changed script operands, linked exits, directional ledges, and dry and
Surf approaches with every local item already claimed or still present. It
checks both versions of the fracture and drain gates. It does not run the game.

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/caves/crystal-replacement-v02/check_north.py \
  content-packs/minnesota-fair-presentation-v01.crystalpack \
  content-packs/minnesota-cave-replacement-v02.crystalpack
```
