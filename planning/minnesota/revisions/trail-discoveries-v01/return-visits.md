# Returning to town after the survey and the League

The Field archive and the Intro archive agree on the main handoffs through Ely and on most return visits. The Rochester clinic and a Duluth errand still have incorrect return dialogue on both editions. In addition, Intro is missing four previously written resident conversations present in Field. Three of those change when the survey is restored. The fourth changes earlier, when Reed releases the worker.

## Rochester clinic

The player sees Loobit at (1,6). The aide says it climbed into the basin from a dry bank. After Ivy leaves to handle the emergency, the aide says Ivy asked her to keep Loobit there until the bank can be checked. Interacting with the Pokémon uses its correct Loobit sprite and text, but plays the retired Gillpup cry. Both the map program and the compiled audio command specify `GILLPUP`. Gillpup is retired family MN_F140, while Loobit is active family MN_F014.

Restoration hides the creature. The aide at (8,6) then switches to `MnArcHabitatAfterText`, which says that Loobit keeps nudging its basin toward the door and that they will try the bank when the water rises. The aide is still available but the clinic animal she is describing has gone. Match the single cry operand to Loobit and let her describe taking it back to the bank. The underlying episode and Ivy's departure already line up. Keep the historical object ID `MN_ARC_GILLPUP` and the existing disappearance on restoration.

## Duluth Union Hall

| Actual visit | Ground-floor host at (5,5) | Upstairs recorder at (5,5) |
| --- | --- | --- |
| Before receiving the names | Describes a boy looking for his mother's name and sends the player to the tally keeper upstairs at the Ore Warehouse | Explains that the record needs the names |
| After obtaining the list | Tells the player to take it upstairs | Receives the names, gives one Ether and sets `EVENT_MN_INTERIOR_UNION_REWARD` only if the item was accepted |
| After the Ether, including after becoming Champion | Still says “If you've got them already, take them upstairs here.” | Says the boy can now find her name and leaves a pencil by the book |

The first-floor script `MN_DULUTH_X_UNIONHALL_F1_HOST` on `MnDuluthUnionHall1F` only checks `EVENT_MN_INTERIOR_UNION_WELCOME`. The upstairs recorder checks completion first. The Ore Warehouse tally keeper on `MnDuluthOreWarehouse2F` likewise checks only `EVENT_MN_INTERIOR_SHIFT_NAMES`, then asks if the player still has the page and points back to the recorder. These are separately reachable, permanently staffed desks in both editions. Both can acknowledge the recorder once the existing reward flag is set. A player with a full bag should still be able to return for the Ether.

## Four resident conversations already in Field

All four have the same identity, sprite, interaction position and visibility in both archives. Intro has one unconditional conversation per person. Field already has the eight reviewed texts below, with short first encounters and distinct returns. Their source remains [the authored resident dialogue](../town-responses-v01/dialogue.json).

| Room and resident script | Position | First text | Field return text | Event |
| --- | --- | --- | --- | --- |
| `MnStCloud` · `MN_ST_CLOUD_STONECUTTER` | (18,13) | `MN_ST_CLOUD_STONECUTTERText` | `MnTownStonecutterAfterText` | `EVENT_MN_REED_WORKER_RELEASED` |
| `MnStCloud` · `MN_ST_CLOUD_RIVER_WATCHER` | (29,24) | `MN_ST_CLOUD_RIVER_WATCHERText` | `MnTownRiverWatcherAfterText` | `EVENT_MN_SURVEY_RESTORED` |
| `MnDuluth` · `MN_DULUTH_X_DOCK_HAND` | (20,73) | `MN_DULUTH_X_DOCK_HANDText` | `MnTownDockHandAfterText` | `EVENT_MN_SURVEY_RESTORED` |
| `MnDuluth` · `MnDuluth_MN_DULUTH_ISLAND_ARTIST` | (76,58) | `MnDuluth_MN_DULUTH_ISLAND_ARTISTText` | `MnTownPainterAfterText` | `EVENT_MN_SURVEY_RESTORED` |

The clearest discrepancy is the Duluth dock hand. Intro still says “The dock gets higher every day” after the water returns. Field describes Lakernaut reaching the boats again. Intro’s island artist continues talking about painting a spoon, whereas Field’s resident notices the Loobit diving again. In St. Cloud, Field’s river watcher gets to race Birchit again and the stonecutter lets Pendulop recover from the shouting. The four matching actor records let Intro reuse these specific conversations.

## Verified working returns

| Milestone | Examples of residents and responses present in both editions |
| --- | --- |
| Rescue and restored survey | Winona's river watcher sees the water reach their child's stick. The Winona and Leech Lake workers offer separate habitat tasks that really do become available. Ellis and Jo in Minneapolis react to Finn’s call. Mina at St. Paul and Sera in St. Cloud report hearing from Finn. Bemidji’s officer requests fresh readings from the gauge crews. The Grand Rapids technician keeps both original readings. The Duluth harbor engineer plans a new sounding. Nell at the mine identifies the last injured worker as safe. |
| Restored rescue site | The Boundary ranger sends the player down the south trail to Ely. The shelter assistant clears the radio for the medic. Inside, Ivy and Finn discuss the open cage and the injured worker. The ranger at Devil’s Kettle notices the restored current. |
| League title | Mom and the Winona cousin acknowledge the team. Tamarack explains the gym rematches after the one-time restoration gift. Rochester's city guide points to Varel. All eight gym guides and leaders have circuit or rematch branches. Casey has a real rematch offer. Reed is available at St. Cloud station; the Boundary ranger can now describe the Voyageurs watch and International Falls clinic. |

Two apparent stale directions disappear with their speakers. Ivy at the Rochester clinic is hidden once the lighthouse signal is sent. Reed at the St. Paul depot is hidden by its arrival callback after defeat. Neither keeps telling returning Champions to redo an earlier stage.

Evidence comes from the installed map actors, ownership flags, local instructions and native text pages of both Trail Discoveries v01 editions. The checks distinguish having restored the water or won the title from also completing a separate errand. The findings cover the named return branches and directly observable contradictions. No gameplay or timing-sensitive scene replay was used. Related independent checks trace [the central handoff](../headwaters-journey-v01/check-central.py), [the harbor and intake](../lake-observations-v01/check-harbor.py), [home and registration](../campaign-quality-v01/check-journey.py), and [the title and rematch](../lake-observations-v01/check-league.py).
