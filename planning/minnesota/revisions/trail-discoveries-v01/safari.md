# The Boundary Waters preserve

A paid visitor can see all four camps, all three named lookouts and the inside of Dorothy’s cabin on one 500-step permit. The measured route spends 493 travel steps and finishes beside the Quiet Island ranger. Someone who simply wants to see the cabin can reach its doorstep in 113. The entire survey rescue, including the radio and the intake, is on the free side of the preserve. [Compiled route, encounter and rescue checks](check-safari.py)

## What a visitor finds

| Place | Previous scene | Revised scene |
| --- | --- | --- |
| Reed Landing | In the free court, the return ranger repeatedly promised to take an unpaid visitor back. Only active expeditions get the retirement prompt. | Unpaid visitors get directions south down the pine path. Once they check in, the ranger can take them back with their catches. The same free directions apply on the next visit after their permit expires. |
| Granite Landing | The sign at (48,36) sent visitors looking for a drop. That old ledge is now a closed cliff, and the real way down has stairs. | The sign identifies the two-way stair northwest at (46–47,32–33). The ranger who can take you back to basecamp stands northeast at (51,35). Both are visible from the sign. |
| Isle of Pines | Dorothy introduced herself on the landing and again inside the cabin, where a second copy of her character stood permanently. | Dorothy stays outdoors with her free first bottle, shop and team care. A camp medic with an existing nurse sprite tends to tired partners inside. Healing, the cabin entrance and the return to the landing work through their original events. |

## One permit

From the entrance, the circuit visits Reed Landing Camp, Palisade Lookout, Pine Portage Camp, Northwind Crown and Granite Bay Camp. It then visits Dorothy’s cabin and Eagle Point, crosses to Quiet Island Camp and meets the return ranger. Its count includes entering the cabin, walking up to the medic and exiting on the south carpet. The last seven steps give a small margin for wandering. Shorter visits leave far more room to explore.

| Destination from entry | Travel steps |
| --- | ---: |
| Reed Landing Camp | 31 |
| Dorothy’s doorstep | 113 |
| Pine Portage Camp | 118 |
| Quiet Island Camp | 121 |
| First encounter grass inside the Cut thicket | 154 |
| Granite Bay Camp | 179 |

The thicket makes a good separate outing. The sign tells the player to cut in from the lake. All 52 grass squares lie behind the one Cut tree. Of the encounters rolled there, 45% are Crookveil and 30% are Latchquill. Across the six outdoor maps, seven habitats offer 43 distinct species on grass and water. The cabin has no Safari encounter table. Rod encounters are separate.

## The portages

The first two portages connect isolated pockets. At the other four, carrying the canoe saves travel. Costs below compare water to water, with a step to land and another to launch. Each crossing works in both directions.

| Crossing | Carry | Paddle |
| --- | ---: | ---: |
| Reed Pocket | 14 | Separate lakes |
| Pine Pocket | 9 | Separate lakes |
| North Pine Arms | 25 | 39 |
| Isle of Pines | 34 | 44 |
| Palisade | 47 | 53 |
| Northwind | 27 | 45 |

The static check starts from the registered paid arrival. It follows the six chunk connections, respects actor bodies and directional cliff edges, and reads the actual signs, doorways, healing and shop events. It checks the 54-square unpaid court with both permit guards present, then follows the free shelter and rescue routes through their actual story gates. The selected `northstar_intro_revision` and Field Headwaters parents produced the same results. The figures come from static analysis of the installed maps and travel budgets.

The original Fuchsia Safari gate offers a paid expedition and a separate early-return conversation. The Boundary Waters rangers use the same distinction. Reed Landing also needs a useful answer for people who have walked there for free. [Original Fuchsia gate](https://github.com/pret/pokered/blob/master/scripts/SafariZoneGate.asm) · [Native Minnesota admission and return](../../../../crates/crystal-assets/src/safari_admission.rs)
