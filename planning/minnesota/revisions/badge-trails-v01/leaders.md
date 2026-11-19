# After each badge

The two Shore Watch editions have identical installed scripts and pages for all eight leaders and gym guides. The evidence below follows the first victory with room in the TM pocket, then the pre-Champion return. If the pocket is full, the badge is already earned. Speaking to the leader again retries the TM before entering the ordinary return conversation.

| Badge | What the player hears in Shore Watch | Where that leads |
| --- | --- | --- |
| Care, Rochester | Varel names Mankato to the west and distinguishes the optional Flash guide back east. He repeats the farm road to Mankato. | Route 2 west reaches Mankato. The guide in Route 1’s limestone cave accepts the Care Badge for Flash. |
| Prairie, Mankato | Torven names Minneapolis to the north and the tool shed on the east farm road for Cut. He talks about the birds on a return visit. | Route 3 north reaches Minneapolis. The Route 2 shed keeper accepts the Prairie Badge for Cut. |
| Current, Minneapolis | Avrin names the mill foreman for Strength. On return he can direct the player to dispatch, Ellis’s desk or Mina at St. Paul Union Depot. | The foreman checks the badge. A rescued crew unlocks the dispatch desk. The sketch connects to the depot miner and Mina. Once the date is in hand, Avrin retires that lead. |
| Capitol, St. Paul | Brannic awards access to the free canoe at the Angler House. His return gives the same canoe and Surf advice. | The boat and Surf are real. His next opponent is Saven in St. Cloud. Reaching her by road means retracing Route 4 west to Minneapolis before turning north onto Route 5. Mina’s unfinished date can still be collected nearby. |
| Wind, St. Cloud | Saven directs the player to the depot guide for Fly. Her return explains the first flight. | The guide checks the Wind Badge. Outside, a local names Pine Road north to Brainerd. Sera’s return encounter and station relay have their own witnesses. |
| Lake, Brainerd | Sivren names the pier ranger. On every pre-Champion return she still says to take the badge to the ranger before leaving. | The first visit really does earn the ranger’s date, and she can offer Surf. The ranger then names Leech Lake, Bemidji, the Itasca staff and the field officer. Sivren previously repeated the *completed* pier errand even after the officer signed. |
| Pine, Bemidji | Orlis names the fisher on Bemidji pier for Whirlpool. His return also names Lake Itasca to the west. | The fisher checks the Pine Badge. The field officer accepts the Itasca reading. The mine foreman requires both her signature and the badge. A local in town gives the mine directions once signed. |
| Harbor, Duluth | Norek gives the newly opened rink exit and names the Lake Outfitter for Waterfall. His return is a story about skating. | The outfitter checks the Harbor Badge. The harbor worker outside directs the player toward the workshop or the northern shelter according to progress. Ely’s registrar requires all eight badges and the restored survey. |

## Two changed conversations

Brannic at `MnStPaulGymLeader` gives the same Capitol Badge, TM and canoe-shop invitation. On return, a trainer carrying Ellis’s sketch who has yet to receive Mina’s date hears that she is beside the shipment at Union Depot. Otherwise Brannic explains both turns to St. Cloud while Saven’s Wind Badge is still outstanding. Once the player has that badge and no local handoff remains, Brannic remembers what happened when Saven switched on her fans.

Sivren at `MnBrainerdGymLeader` keeps the initial pier invitation. The ranger’s recorded reading now leads to the gauge west of Bemidji, via Leech Lake. If the player saw the Itasca staff first, Sivren sends the mark straight to the Bemidji field officer. After the officer signs, or the survey is restored, Sivren points out the wake of Torrout, her actual lead Pokémon.

Both changes live on the return branch reached after accepting the original TM and before becoming Champion. The public leader, original reward and pocket-full retry, Champion battle offers, rematch gifts and circuit guides remain the entry points. The two gyms’ entrance instructions are reviewed separately in [the court audit](gyms.md).

Run `check-leaders.py PARENT CHILD` with matching editions. It checks the native road connections, recipients, item-retry order, 32 combinations of story progress, and the same 32 states when a Champion declines a return match. It checks each new page against its canonical native lines and checks the local, global and indexed command records that carry those lines.
