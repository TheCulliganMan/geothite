# The Northstar Survey story

## Premise and cast

The player is a new trainer from Winona carrying original field readings to Rochester for Dr Tamarack. The instruments that should accompany them never arrive. A shipping error becomes a trail of diverted equipment, copied readings and intimidation leading to **Team Lode**, which is concealing the risks of a fictional copper-nickel mine. The parcel from the opening errand contains the originals its leaders need erased.

The mystery grows into a Pokémon adventure: Lode's DEEP BORE tests are disturbing the hidden returning current of **Kettlurk**, the existing Water/Ghost legendary design associated with Devil's Kettle. **Team Lode and its ally Team ICE want to drain the Boundary Waters for copper and nickel beneath the lakebeds.** Lode builds the drainage network and plans to capture Kettlurk to power its pumps. ICE protects the operation through intimidation and fraudulent police authority. The stolen instruments locate the current; falsified reports conceal the tests. The team needs both the machinery and the cover-up, so the investigation and legendary threat form one plot.

The journey follows the existing Winona-to-Ely gym circuit. Each chapter changes what the player understands: an apparent mistake, evidence of theft, evidence of fraud, evidence of deliberate harm, then the intended captive. Winning the final chief battle clears the desk. **Restoring the receiver is a separate player action:** restore the public link, send the true readings and drainage plan to the harbor, and coordinate crews stopping the drainage pumps before the lakes are emptied. Kettlurk escapes being trapped, the survey resumes, and the League trail reopens. Catching a legendary is never required.

Dr Tamarack supplies the recurring investigative thread. The guardian makes the departure personal. The courier recognizes the opening delivery and demands to know who kept its records; the foreman knowingly orders another test despite the warnings; the chief admits that he changed the reports. Residents have their own lives, useful directions and local problems. Fishing, the Fair, Root Beer hospitality and the gym challenge provide room to enjoy Minnesota between confrontations.

The [connected Rochester chapter](playable/rochester.md) now earns the guardian
scene, starter, original readings and five balls from production New Game,
walks R01, and delivers the originals at the clinic before returning home.
The Lode-stamped shipment clue and separate gym prerequisite execute through
native dialogue. It retains the integrated ICE/northern story and Ryan's cabin.
Gym1's battles/reward have separate local combat proof and one complete earned
Cyndaquil/caught-partner run through Care and Continue. Early-party balance/retry
coverage, R02/Mankato and the full regional journey remain unfinished.

## Villain direction

**Nuanced villains, still villains; restrained rather than cartoonish.** Lode's leaders can explain why they want the mine, and they understand the technology. They also understand the evidence and choose to suppress it. Their belief that the region needs metal and work becomes a claim that they may decide whose water, safety and Pokémon can be sacrificed. Their culpability comes from informed choices, not an accident, possession, misunderstanding or a good plan that the player fails to appreciate.

- **Courier:** a controlled fixer who calls a theft a shipping correction. He threatens the player to identify who kept the delivered originals. Losing exposes the manifest; it does not redeem him.
- **Foreman:** an effective operator who values output over the people and Pokémon doing the work. He has read the warnings and orders the next test anyway.
- **Chief:** a survey-trained project leader who regards independent scrutiny as an obstacle to be removed. He intends to make Kettlurk an industrial asset and retain control of the project. After defeat he yields access, not his beliefs.

Give the chief a line such as: “Yes, I changed the reports. I will not stop this mine.” Avoid gleeful declarations that pollution is desirable, comic ownership speeches, a sympathetic deathbed confession, or a final claim that everyone was equally right. Ordinary mine workers and mining towns are not Team Lode; some help expose it. The story can respect those communities while making the antagonists' coercion and concealment unambiguous.

**Team ICE is Lode's Minneapolis ally.** Its leadership actively wants the destruction: emptied lakes, displaced residents and broken resistance. Lode wants the exposed ore; ICE wants to destroy the places people defend. ICE kidnaps anyone it suspects of opposition, without needing evidence. Lode knowingly shields and benefits from the abductions. They are openly cruel
bullies posing as police: counterfeit badges, invented inspection powers,
confiscated records and threats against people who ask questions. Lode pays
them to protect diverted shipments, seize evidence and force residents and
witnesses out of the operation's way. Their menace comes
from abusing claimed authority, not jokes, ice puns or comic incompetence.
The Minneapolis dispatcher is a captive. The player defeats their enforcer, releases the dispatcher through a separate interaction, and then reads a separate dispatch copy
linking both teams. Defeat costs ICE access to the records; they remain hostile.
MS03_ICE, MS03_RESCUE and MS03 now execute as separate native milestones.
Actual controller and stdio MCP verify battle outcomes, refusal/retry, the
physical cell opening, acknowledged reading and return home. The
[Minneapolis runbook](playable/minneapolis.md) records the verified integrated story/gym/cabin candidate;
earlier chapters, earned regional arrival and the remaining finale work stay open.

## Mystery, spectacle and geography

The existing **Kettlurk / MN_096 / MN_F045** supplies the supernatural thread without inventing another legendary or moving the prairie pair into the Northwoods. Before the reveal, people describe a current that disappears and returns and a deep pulse beneath the water. A future native set piece can show the pump struggling against that current. Construction platforms, exposed shoreline test cuts and maps of the planned lakebed pits can show what the allies intend. Stop the operation before the lakes are emptied; the ending cannot instantly refill or purify them. The player stops the machinery and its operators; Kettlurk does not clean up the whole region or win the trainer battle for them.

Preserve Kettlurk's optional Devil's Kettle encounter near Grand Marais. The mandatory story may establish its presence and thwart its capture without requiring that side branch, Pokédex registration, a catch, or a paid Safari visit. The current native chief/receiver scene names the target and reports the stopped drainage pumps; an apparition, animated machine and actual Kettlurk encounter are **still implementation work**.

Itasca is an independent reference used to expose a single instrument log relabeled as several station surveys, not a pipe carrying water to every northern destination. The Bemidji officer has comparison copies, so missing the optional Mankato notebook does not require a new story gate. Minnesota has distinct drainage basins. The pulse equipment and supernatural returning current are fictional; do not use them to explain actual Devil's Kettle hydrology. Distinguish the fictional copper-nickel project from iron/taconite mining, and do not borrow a real company's name or imply that a real community committed the story's crimes. Background: [Minnesota DNR mining overview](https://www.dnr.state.mn.us/lands_minerals/mining.html) and [USGS copper-nickel bedrock water-quality study](https://pubs.usgs.gov/sir/2020/5039/sir20205039.pdf).

`story.json` is authoritative for the narrative, quest prerequisites, completion flags, map names, repeat guards, gym teams and badge slots. Its `milestones` attach the reveal and sample dialogue to each existing producing scene. MS10 now supplies the exact three native harbor text bodies through `duluth_story.rs`: ten pages name Kettlurk, identify the lake-drainage scheme, ICE's destructive motive and its kidnappings and establish that leadership read the warnings before continuing. Production input verifies that every page precedes the signal; all eight badges and acknowledged depot clearance remain required. The depot retains the meter, so this scene reads the copied log and does not pretend the player still carries the instrument. MS11/MS12 specify the exact native chief/receiver texts, checked against the lowered shelter by the preservation regression. The Duluth receiver sets the signal before North Shore travel; survey restoration happens later in Boundary Waters. `actors.json` and `interiors.json` reserve actors and navigation. Duluth's R10/R11 walking links are verified on the expanded Northwoods composition; the earned regional journey remains open. Authored direction and symbolic progression are not evidence of an implemented regional playthrough.

## Play sequence

| Quest | Playable sequence | Completion and next gate |
| --- | --- | --- |
| Q00 | Winona departure, starter and original survey parcel | One level-5 starter; five Poké Balls once; Route 1 opens |
| Q01 | Deliver the originals in Rochester; matching instruments are missing | Delivery and badge 1 permit the prairie route |
| Q02 | Mankato notebook has copied readings; defeat Team ICE's counterfeit police enforcer in Minneapolis, free the captive dispatcher and read the separate dispatch copy | Gyms 2/3; evidence points to St Paul |
| Q03 | Courier threatens the keeper of the originals; his manifest names DEEP BORE | Courier win persists; gyms 4/5; north gate checks both city badges |
| Q04 | Lake-country witnesses contradict the reports; obtain an independent reference assignment | Gyms 6/7; optional Surf; main travel remains on land |
| Q05 | Itasca reference and Bemidji comparison expose tampered records | Completed report opens Iron Range; raw reading alone does not |
| Q06 | Foreman guards stolen instruments; recover the meter and test log from the separate cache | Actual win, acknowledged cache recovery, then Grand Rapids handover open Duluth |
| Q07 | Harbor log exposes the plan to drain the Boundary Waters; follow the North Shore, defeat the chief and cut the drainage relay | Gym 8 and signal precede travel; chief win precedes receiver restoration and League release |
| Q08 | Ely League, Hall of Fame and return to a region whose recovery continues | Champion completion persists; later legendary encounter remains optional |

The [native prairie chapter](playable/prairie-story.md) now connects Rochester,
R02 and Mankato. The optional garden notebook gives the first concrete copied-
reading clue; its completion follows both acknowledged pages. It does not gate
Torven. Local production input/Continue and actual stdio pass; the fully earned
chapter journey remains unproven before R02. The [river successor](playable/river-cities.md)
connects Mankato through R03 to Minneapolis's ICE chapter. A continuous local
fixture walks there, wins the enforcer battle, acknowledges the separate rescue
and dispatch reading, and walks back without earning badge3 or the notebook.
The [courier successor](playable/courier-story.md) now connects Current-only
R04 travel to St Paul. The actual dispatch note separately permits the
Magnemite/Drowzee courier confrontation; a distinct paper records the DEEP BORE
clue after all pages are acknowledged. Local populated battle/read/return,
loss recovery, independent gym access and actual stdio pass. The
[lake-country successor](playable/lake-country-story.md) now connects the two-city
R05 gate, St Cloud, the Wind-gated R06 and Brainerd. One local native course
earns Wind/Lake, reads the ranger's contradictory report and receives Surf once;
the courier can remain unfinished throughout. Initial four badges and strong
partner are setup. The [shore successor](playable/leech-shore.md) now connects
Lake-only R07/R08 through the actual Leech Lake settlement to Bemidji. Local
witnesses give the empty nets and ICE's pressure on record-keepers a voice.
A continuous local course earns Pine after a played loss/clinic recovery/retry
and walks home. Six prior badges, partner/Cut and RNG remain setup. The [Itasca successor](playable/itasca-story.md) now binds the actual Ilex-Azalea
gatehouse, early Lake-only R14 outing, Pine-qualified marker and separately
acknowledged officer comparison. Reading alone does not open R09; the completed
report plus Pine does. A continuous local course earns Pine, reading and report
through native input; empty-party road and actual stdio round trips also pass.
Optional branches, rail and the continuous earned regional journey remain open.

Do not gate all exploration behind the main quest. The Itasca/Detroit circuit, prairie route, Mille Lacs shore and border circuit have their own rewards. Their flags cannot stand in for mandatory story completion. Route 20 may reach Boundary Waters after badge 8 and the harbor signal, but Route 13 still requires the restored survey.

## Gym pacing

See [the eight gym designs](GYMS.md) for puzzles, battle move sets and rewards,
and [city by city progression](PROGRESSION.md) for the full journey and optional
branches. The existing JSON remains authoritative for rosters and route gates.

| Slot | Place | Leader | Type | Badge | Ace level |
| --- | --- | --- | --- | --- | ---: |
| 1 | Rochester | Varel | Normal | Care | 12 |
| 2 | Mankato | Torven | Ground | Prairie | 16 |
| 3 | Minneapolis | Avrin | Electric | Current | 21 |
| 4 | St Paul | Brannic | Steel | Capitol | 25 |
| 5 | St Cloud | Saven | Flying | Wind | 30 |
| 6 | Brainerd | Sivren | Water | Lake | 35 |
| 7 | Bemidji | Orlis | Grass | Pine | 40 |
| 8 | Duluth | Norek | Ice | Harbor | 45 |

Full species and level lists are in `story.json`. Resolve moves and held items through existing pack catalogs; use only legal authored move sets. Minnesota badge display names map to the eight existing Johto badge slots. Keep battle rewards, obedience and field permissions coherent with those slots. Do not infer a field permission from the new display name or gym type. Inspect the actual base `field_moves` catalog before assigning HM rewards or shortcuts.

The four League trainers are Fern, Flint, Veil and Reed, followed by champion Casey. Their complete species and level rosters are in `story.json`, together with the courier, foreman and survey-chief parties. Resolve legal move sets, held items, rewards, battle classes and trainer presentation from the base catalogs before the battle milestone; a species/level roster alone is not a compiled trainer definition.

## Persistent state contracts

Use `EVENT_MN_*` names for new authored story flags, trainer victories and item pickups. Allocate their numeric script constants against the real pack catalog. `BADGE_n`, `MOVE_CUT` and `CUT_FIELD_PERMISSION` in the plan are condition expressions, not literal new engine event tokens. Translate them to existing badge/party/field-move checks.

For every once-only reward, check its completion flag before changing the bag or party, run the appropriate authoritative mutation, and set the completion flag only after success. A full party, full bag, cancelled menu or lost battle cannot mark a reward as delivered. Choose a specific recovery branch for each of those outcomes.

Trainer victory requires an actual resolved battle win. Save its flag before the next overworld interaction. A loss uses production blackout behavior and does not leave a defeated trainer or grant a badge. A repeat interaction selects post-win text and does not replay rewards.

Local story scenery changes use existing persisted map block overrides and map-local object visibility. Do not implement water-flow physics or renderer-owned quest flags. A scene that changes both a flag and blocks must restore the same visible state on reload and after returning from another map.

**Q06 evidence custody.** MS08 records only the foreman's actual defeat.
MS08_CACHE is a separate interaction at the guarded meter: declining leaves it
available; confirming and acknowledging the recovery text sets
`EVENT_MN_MINE_INSTRUMENT_RECOVERED`. This persistent quest payload follows the
same event-based approach as the opening parcel. MS09 requires both defeat and
recovery before the separate depot handover can clear the Duluth route. The
mine does not grant `EVENT_MN_DEPOT_CLEARED` or substitute a win for carrying
the evidence. Its local native binding is `iron_range_story.rs`. Grand Rapids'
native technician checks both prerequisites, offers a retryable yes/no and
records clearance after the full reading is acknowledged. The depot keeps the
meter; the player takes a log copy for Duluth. Recovery remains a historical
milestone, rather than a claim that the player still carries the meter.
`grand_rapids_modpack.rs` binds the handover and ordinary services;
`grand_rapids_connections.rs` binds the guarded R09/town/R10 walking links.
The east checkpoint requires badge 7 and this completed handover. The onward Duluth link is now verified through both checkpoint lanes; the full earned regional journey remains open.

## First chapter script contracts

**Winona guardian interception.** Trigger on the authored ground-floor approach once. Face the guardian and player, open the first page, and reject movement while dialogue owns input. A advances exactly one visible page. Complete the scene and release movement only after its last page. Store a completion event so re-entering the room does not repeat it.

**Starter choice.** Three labelled balls offer Chikorita, Cyndaquil and Totodile through native yes/no, gift and nickname owners. Declining preserves all three choices. The receipt event commits after the real level-5 gift and nickname flow; the originals and five balls remain separate recoverable handovers. The route gate checks that receipt event. Re-entering the field station uses the opening's repeat dialogue.

**Route 1 tutorial and farmer.** Give each interaction one purpose: grass encounters, ledge direction, item pickup, or resting. Keep mandatory route movement independent of the optional farmhouse and cave. The first encounter cannot occur before the player has their starter. The field guide may supply text without forcing a scripted battle if that makes the first slice easier to verify honestly.

**Rochester delivery.** The clinic's separate survey recipient checks the starter and original-readings events. Two acknowledged pages reveal the missing meters and Lode stamp, then set `EVENT_MN_DELIVERED_ROCHESTER_PARCEL`. Delivery adds no bag reward. Repeat dialogue acknowledges the retained copy; Varel checks delivery independently of the two qualifying trainer victories.

**First gym.** The guide, qualifying trainers and Varel each use authored scripts through `VisibleShellController`. Varel's party is the explicit two-Pokémon team in `story.json`. On a win, set native badge slot 1 and the Minnesota victory event. On a loss, preserve the unearned state. Verify the battle engine's move PP mutation and resulting dialogue; a HUD alone does not complete the milestone.

Write final dialogue in page-sized game text, with intentional line breaks and source-font coverage. The current generic trainer lines in route dossiers are script placeholders and need a voice pass; they are not approved final dialogue.

Boundary Waters is the region’s Safari preserve. Its ranger basecamp remains a free route junction; the mandatory survey-chief/receiver scene stays in the public survey shelter. The optional expedition uses capture-only wildlife encounters and a finite ball/step budget. Safari payment, rare captures and exhausted permits cannot replace or block Q07 flags. See [the Safari contract](zones/boundary-waters-safari.json).

The [continuous Iron Range course](playable/northstar-iron-course.md) now earns
Pine, the independent reading/report, foreman victory, separate meter recovery,
depot handover, Harbor and the ten-page drainage reveal in one local session.
A separate real wild loss preserves that earned custody through clinic recovery
and Continue. Initial six badges and Mew45/Cut remain setup. The frozen543-map
parent has no Grand Marais regional links. The [544-map successor](playable/north-shore-story.md)
adds the authentic coast gatehouse, two-lane Gunflint seam, free inbound care
and an optional local lamp repair. It preserves the Duluth signal requirement
and grants no new regional story flag. Ely and the full earned campaign remain
unfinished.

The [earned northern finale](playable/northstar-finale.md) now continues the
actual Boundary arrival through the chief and separate receiver. It checks the
canonical admission and completion pages, actual selected-move PP, refusal of
R13 even after the chief win, retryable decline, acknowledgment before restoration,
repeat/Continue and the full reopened trail to the Ely frontier. Two actual
stdio replays also pass. Initial six badges/Mew45 remain upstream setup; this
does not publish Ely, implement Kettlurk's visible machinery/encounter or prove
a complete fresh Winona playthrough.
