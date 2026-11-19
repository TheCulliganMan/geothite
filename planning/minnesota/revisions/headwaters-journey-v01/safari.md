# Reed Landing’s unpaid visitor

The Reed Landing ranger promises to take a visitor back to basecamp with their catches. An unpaid visitor can reach him, but talking to him again produces the same promise. The paid expedition already has its own working return prompt.

The ranger stands at (10,7) in `MnBoundarySafariC00R02`. Starting at the southern court entrance (14,15), an unpaid visitor can walk 11 steps to (10,8) and face him. The native script `BWS_REED_RETURN_RANGER` goes directly to `BWS_REED_RETURN_RANGERText`. The authored text is in `dialogue_overhaul.texts.MnBoundarySafariC00R02.BWS_REED_RETURN_RANGERText`. The interaction opens the separate retirement prompt only during an active expedition’s exploring phase. After a permit ends, the same unpaid directions would apply again. [Authored story](../../story.json) · [Ranger roles](../../../../crates/crystal-assets/src/safari_prompts.rs) · [Field interaction](../../../../crates/crystal-bevy/src/bevy_shell/battle_results.rs)

## Proposed unpaid reply

| Page | First line | Second line |
| --- | --- | --- |
| 1 | Take the pine path | south to basecamp. |
| 2 | Want the shelter? | Look for SURVEY. |

From the public court, south through Pine Entry leads back to basecamp. The admission ranger is 65 walking steps from the court entrance. The shelter is 97 steps away, reached through its door at (43,20). The basecamp trail sign at (34,26) points east for SURVEY, and the sign at (42,22) names the shelter. This text is a proposed follow-up. The paid ranger prompt already asks whether to return to basecamp and keep the catches.

## Admission, rescue and return

| Player state | Installed behavior |
| --- | --- |
| Never paid, including before becoming Champion | Public trails, 54 walkable court squares and the survey shelter are free. The two permit guards at (11,5) and (12,5) stop visitors at the preserve gate. The ranger, medic and radio can be approached independently of Safari. |
| Has the Duluth signal | Defeating Hale and accepting the radio call opens the intake. The landing, Reed, drill, outlet and bypass can all be reached on foot. Restoring the water and releasing Kettlurk requires no money, equipment for Safari travel or expedition flags. |
| Checks in at basecamp | The ranger at (30,9) offers 30 Safari Balls and 500 travel steps for 500. The player needs a canoe or a Surf-capable partner with field permission and room for a capture. A refusal, missing equipment or funds, or a full party and current box leaves the money alone. Admission has no Champion requirement. |
| Takes the trip | Entry is Reed Landing (15,9). A committed step counts once, including boarding by Surf. Turning or meeting a wall costs nothing. Only throwing a ball spends one. After the last catch, the player receives its result before returning. |
| Finishes, retires or walks out | The three return rangers and exhausted budget lead to the basecamp ranger at (30,10). Walking south out of the preserve ends the permit at the actual destination. Catches are retained. A subsequent trip starts a fresh permit. |

## Habitat check

On both Station Return editions, the seven native habitat regions contain 316 tall-grass and 6,155 water squares. Their usable field tables list 43 distinct species, with 27 on grass and 18 on water. Rod encounters are separate. The Cut-only pine thicket contains 52 grass squares and no water. Its grass favors Crookveil (45%) and Latchquill (30%). Reed Marsh has a 3% level-55 Mosquitan slot. Quiet Water has a 3% level-44 Latchusk slot on water. These percentages are conditional on an encounter in that habitat.

The dry and water paths reach grass in six habitats without Cut, within 124 travel steps from entry. The route into the thicket reaches its first grass within 154 steps after cutting its tree. These are static walking and water routes with actors in place. Both editions returned identical habitat, court and rescue results. Admission, counting and settlement were traced in the shared runtime. [Admission and safe return](../../../../crates/crystal-assets/src/safari_admission.rs) · [Travel budgets](../../../../crates/crystal-core/src/systems/safari.rs) · [Capture and inventories](../../../../crates/crystal-core/src/systems/safari_encounter.rs)

Pokémon Red’s Fuchsia gate asks before charging, gives 30 balls and offers separate dialogue for leaving early. Minnesota already makes the same distinction inside a trip. The proposed line gives unpaid visitors at Reed Landing an equally clear way back. [Original Fuchsia Safari gate script](https://github.com/pret/pokered/blob/master/scripts/SafariZoneGate.asm)
