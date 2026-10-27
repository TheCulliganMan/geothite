# Minnesota gym circuit

Finish one gym before designing the next. Give it a simple physical puzzle,
make its trainers part of the route to the leader, inspect the complete room
and the actual player view, and prove its battles, gates, exits and persistence.
Vary the mechanic between gyms. Do not fill rooms with book sprites, quiz
stations or interchangeable attendants.

**The user rejected the old gym visuals.** All eight rooms now have individual
remakes reviewed against actual original Crystal maps and native player views.
The current 480-map interior pack passes the combined eight-room controller
course. These are revised candidates, not inferred user approval. Old renders
remain archived. The current 541-map Leech shore story successor retains all eight
reviewed rooms with their actual city facades and passes every local gym course.
Continuous earned regional travel is still incomplete.
[City progression](PROGRESSION.md) describes the regional journey separately.
[Implementation audit](gyms/implementation-audit.md) records the complete room checks and their limits.

The user rejected the subsequent planter revision too: its walls were unclear
and forced a walk without a useful decision. Rochester now uses
[a smaller two-switch lane course](gyms/rochester-gates.md). The other seven
retain their prior designs pending this review. Earlier test passes do not
finish the improvement request or establish user visual approval.

| Order | City / leader | Type / badge | Next design direction | Status |
| --- | --- | --- | --- | --- |
| 1 | Rochester / Varel | Normal / Care | Two switches; left bay, upper gate, return to right lane | Published7×7 candidate; literal controls and native gate-state review pass |
| 2 | Mankato / Torven | Ground / Prairie | Native rock course with three one-way drops | Rebuilt candidate; native city facade, full course and service checks pass |
| 3 | Minneapolis / Avrin | Electric / Current | Six cans and a fixed adjacent switch pair | Rebuilt candidate; native facade and gameplay checks pass |
| 4 | St. Paul / Brannic | Steel / Capitol | Four linked pad rooms and a gated court | Rebuilt candidate; native facade, all pad pairs, city services and R04 crossing pass |
| 5 | St. Cloud / Saven | Flying / Wind | Blue walkways and paired wind lanes | Rebuilt candidate; native city facade, full wind course, services and R05/R06 crossings pass |
| 6 | Brainerd / Sivren | Water / Lake | Original-style pool and shifting piers | Rebuilt candidate; native course and four focused checks pass |
| 7 | Bemidji / Orlis | Grass / Pine | Two native Cut hedges in a greenhouse | Rebuilt candidate; native facade and gameplay checks pass |
| 8 | Duluth / Norek | Ice / Harbor | Two ice courses and an opening cross-passage | Rebuilt candidate; standalone and native-facade gameplay checks pass |

These directions preserve [story.json](story.json)'s leaders, levels and badge
order. Each room was refined individually. Exact authored data is in
[gyms/current.json](gyms/current.json), which layers revised rooms over the
pinned historical `interiors.json`; implementation status and commands
are in [gyms/README.md](gyms/README.md).

## Rochester: switch the lanes

A7×7 training room has two blue wall switches and three gold gates. Fixed
partitions separate a left training bay from the right lane and leader court.

Battle Tavel, then press the lower switch to open the left lane. Battle Elvik
in that bay and press the upper switch: it opens the gate below Varel, across
the divider. Return to the first switch and flip it again, closing the left
lane and opening the right. That return is required by the physical geometry.
The switches change immediately on A, with no quiz or menu.

[Gate-state sequence](renders/gyms/rochester-gates/gate-sequence.png) ·
[Animation](renders/gyms/rochester-gates/gate-sequence.gif) ·
[Native entrance](renders/gyms/rochester-gates/native-entry.png) ·
[Exact contracts and proof](gyms/rochester-gates.md)

The literal course passes with Totodile16/Hoothoot14 and the initial parcel
fixture. It earns both trainers and Varel, tests the wrong lane and disabled
switches, checks gate reversal and Continue, earns one badge/TM and uses both
real city carpets. Every other map, battle, trainer number and unrelated art
stays exact. These checks prove this candidate's behavior; user acceptance is
still open. The rejected planter generations remain archived.

## Mankato: the low banks

An 8 × 10 metatile gym uses the original Pewter gym's wooden floor, low rock
walls and paired statues. Small sandstone banks mark the three one-way drops.
The former dirt quarry and thick outer boulder walls are rejected versions.

[Complete review](renders/gyms/mankato-og/Mankato-review.png) ·
[Whole room](renders/gyms/mankato-og/MnMankatoGym-whole.png) ·
[Native entrance](renders/gyms/mankato-og/native-entrance.png) ·
[Native west drop](renders/gyms/mankato-og/native-west-bank.png)

Battle Caldren beside the lower stair. Climb the west side, then jump down the
low bank into Jorel's area. Her victory clears the upper stair to Torven.
The ledges block climbing back up. The low edge at the bottom lets you retreat
to the entrance before fighting Jorel; you can always try again via the stair.
After Torven, two short drops make the return quicker than retracing the stairs.
No switches, quizzes, floor pads or field moves are required.

| Stage | Challenge | Physical change |
| --- | --- | --- |
| Arrival `(6,17)` | Caldren `(4,17)`, Sandshrew 12 | Both stairs blocked |
| Caldren defeated | Climb the west side | Lower stair `(1,7)` opens |
| West bank | Jump `(6,8)` → `(6,10)` | Only entry to Jorel's area |
| Jorel `(11,11)`, Wooper 13 defeated | Climb upper stair | Stair `(6,3)` opens |
| Torven `(11,3)` defeated | Prairie Badge and one Mud-Slap TM | Return drops `(10,6)` → `(10,8)`, then `(10,14)` → `(10,16)` |

Actor and jump coordinates are 16px walking tiles; stairs use 32px metatiles.
Both carpets `(6,19)` and `(7,19)` exit; the guide is `(9,16)`. Native collision
checks require the real two-tile jumps, blocked reverse approach, necessary
stairs and useful return loops of at least 24, 6 and 28 walking tiles.
The first attempt exposed a six-tile walk around the west bank; the right-hand
boulders now close that bypass, and the upper stair has its own lane.

Torven keeps the story party: Wooper 14, Sandshrew 15 and Phanpy 16. The first
badge is required. Caldren and Jorel use generic Camper and Picnicker classes;
all three names are original. The controller fixture seeds the test party,
prior badge and initial location. This verifies a standalone interior, not an
earned regional journey or the eventual city-facade connection.


## Minneapolis: the adjacent switches

A 5 × 9 metatile room uses the original electric gym's pale floor, blue wall
trim, six switch cans and paired entrance statues. Two visible metal barriers
close the short passage to Avrin. The former large mill powerhouse is a rejected
version. This is a fixed spatial puzzle with trainer hints, no quiz menus or
random switch locations.

[Complete review](renders/gyms/minneapolis-og/Minneapolis-review.png) ·
[Switch states](renders/gyms/minneapolis-og/native-switch-comparison.png) ·
[City facade](renders/gyms/minneapolis-og/native-city-facade.png)

Battle Lenvi, then Jossa. Lenvi points to the left can in the upper row; Jossa
explains that the second switch is in the next can to the right. Both wins are
required before the switches operate. Press the first switch to release the
lower barrier, then the adjacent second switch to release the upper barrier.
A wrong second choice, including pressing the first can again, resets both
switches and physically restores the barriers. Trainer wins stay saved. Once
both locks are released, inspecting any can keeps the circuit open.

| Stage | Required action | Physical result |
| --- | --- | --- |
| Arrival `(4,15)` | Lenvi `(1,13)`, Mareep 17 | Both barriers closed; switches refuse |
| Lenvi defeated | Jossa `(8,10)`, Magnemite 18 and Voltorb 18 | Jossa accepts the battle |
| Both trainers defeated | Upper-left can `(3,9)` | Lower barrier `(2,3)` opens |
| First switch on | Upper-middle can `(5,9)` | Upper barrier `(2,2)` opens; Avrin reachable |
| Avrin `(4,3)` defeated | Current Badge and one ThunderPunch TM | Solved circuit and rewards persist |

Actor/control coordinates are 16px walking tiles; barriers use 32px metatiles.
The other cans are `(7,9)`, `(3,11)`, `(5,11)` and `(7,11)`. The guide is `(7,15)`.
Avrin keeps Mareep 19, Magnemite 20 and Flaaffy 21; badge 2 remains required.
Her repeat text directs the player to the dispatch note before St. Paul.

The actual Minneapolis gym facade now enters this room at `(4,15)`. Both
carpets `(4,17)` and `(5,17)` return to safe exterior floor `(42,24)` through
city return record 4; the exterior door is `(42,23)`. The city builder pins the
native gym module except the three warp bindings and their mirrored event
records. It accepts both the individual and full-eight-room review layouts.
The latter is an integration fixture, not visual acceptance of other drafts.

Four standalone controller checks cover trainer requirements, every wrong
second choice, reset collision, input ownership, Continue inside the partially
open passage, single rewards, loss and missing badge 2. A city controller check
earns both trainer wins, switches and badge through real facade round trips,
including both carpets before and after victory. Prior badges, test party and
initial placement are explicit fixtures; earned regional travel is not proved.
The city preservation/rejection tests, existing city gameplay checks and native
terrain/collision comparisons pass. Original HOME, recovery, maps, source
trainers, catalogs and audio remain preserved.


## St. Paul: four pad rooms

A 7 × 10 metatile gym uses the original Saffron gym's pale diamond floor,
thin blue walls and native pads. Four closely spaced chambers replace the
large separated galleries. Every visible pad has a reciprocal destination;
there are no menu controls, book props or quiz attendants.

[Complete review](renders/gyms/st-paul-og/St-Paul-review.png) ·
[Court before/after](renders/gyms/st-paul-og/native-grille-comparison.png) ·
[Belva's room](renders/gyms/st-paul-og/native-belva-room.png)

Take either side route to Belva and Orven. The second pad in each trainer room
leads around a loop through the other room and the foyer. The middle foyer pad
leads to Brannic's court. Trying it early shows the closed grille and a safe
return pad. Both trainer wins release that grille. The route permits either
trainer order, and all pads keep working after victory.

| Pair | Native positions | Warp records | Route |
| --- | --- | --- | --- |
| West room | `(1,11)` ↔ `(1,3)` | 4 / 5 | Foyer ↔ Belva |
| Cross room | `(5,5)` ↔ `(9,11)` | 6 / 7 | Belva ↔ Orven |
| East return | `(11,15)` ↔ `(5,13)` | 8 / 9 | Orven ↔ foyer |
| Middle pad | `(3,15)` ↔ `(11,7)` | 10 / 11 | Foyer ↔ court landing |

Positions are 16px walking tiles. The grille at metatile `(5,2)` joins the side
wall without a walking bypass. Belva `(3,3)` uses Magnemite 21; Orven `(11,12)`
uses Pineco 22 and Magnemite 22. Brannic `(11,2)` retains Magnemite 23, Pineco 24
and Skarmory 25, badge-3 refusal and one retryable Steel Wing TM after victory.
All three names and their generic trainer classes are asserted in battles.

Arrival is `(2,17)`; carpets `(2,19)` and `(3,19)` exit; guide `(4,16)` explains
the pads. Four controller checks prove both trainer orders, every pair in both
directions, waiting/saving on pads without bouncing, early-court escape,
selected-move PP, victory, repeat reward protection, loss, missing badge 3,
both carpets and Continue. Initial party/prior badges/placement are fixtures.
Source/published pixels and collision match; native player views are inspected.

The town's canonical reference, hashes, glyph map and proposed door contract
are synchronized. Future city integration must bind only records 1–3 to the
safe town return at `(54,16)` and preserve internal records 4–11. St. Paul town,
services, depot and regional travel remain source work; this pack verifies the
standalone gym through its review vestibule.


## St. Cloud: read the wind

A 9 × 11 metatile gym uses the original Flying gym's blue brick, raised edges,
metal caps and entrance statues. Stepped landings replace the large timber
platforms. Four short slatted lanes cross the gaps. The slate art is unchanged;
only those visibly distinct slats receive native forced-walking collision.

[Complete review](renders/gyms/st-cloud-og/St-Cloud-review.png) ·
[Lower crossing](renders/gyms/st-cloud-og/native-lower-crossing.png) ·
[Upper crossing](renders/gyms/st-cloud-og/native-upper-crossing.png)

Each pair follows the same rule: **upper lane east, lower lane west**. Fight
Wyrel, cross east to Hallen, then go north and cross west to Saven. Trying the
wrong lane pushes you back to the same bank. The slats carry you even if you
press against the wind; stone is a safe resting place. After victory, use the
upper eastbound lane and the lower westbound lane to return.

| Stage | Challenge / route | Physical result |
| --- | --- | --- |
| Arrival `(4,19)` | Wyrel `(4,15)`, Pidgeotto 25 | Both outward crossings shut |
| Wyrel defeated | Lower eastbound lane `(7,13)` → `(12,13)` | Mouth at metatile `(3,6)` opens; Hallen reachable |
| Hallen `(14,10)`, Spearow 25 and Noctowl 26 defeated | Upper westbound lane `(12,9)` → `(7,9)` | Mouth `(6,4)` opens; Saven reachable |
| Saven `(3,3)` defeated | Wind Badge and one Swift TM | Return east `(7,5)` → `(12,5)`, then west `(12,17)` → `(7,17)` |

Each lane is four walking tiles long, with ordinary stone on both ends. Actor
and crossing coordinates are 16px tiles; gate coordinates are metatiles.
Guide `(7,18)` explains the rule; carpets `(4,21)` and `(5,21)` exit. Saven keeps
Pidgeotto 28, Noctowl 29 and Fearow 30. Badge 4 is required. His river-lookout
hint remains, and this gym does not grant the city's separate Fly gift.

Three production-controller checks pass: the full wind course, loss and missing
badge 4. They earn both trainer wins, observe live movement, press against an
active gust, wait without more movement input, try both wrong lanes, use every
return, save on landings, win the leader battle with real PP use, verify one TM
and use both carpets. Initial party/prior badges/placement are fixtures.
Source/native pixels and the declared collision overrides match. The 473-map
pack preserves all 471 source maps, files, source trainers, HOME/recovery,
catalogs and audio.

The canonical city reference, glyph map, viewpoints and carpet anchors are
updated. Current non-gym city fields, exterior layout and station are preserved
exactly. The canonical source phases pass; the historical whole-town comparison
baseline already differs from the current city source and was left unchanged.
The 492-map city successor now publishes the four real facades and both route
mouths. The full wind course, trainer/leader wins, both city returns, Continue,
care/PC/priced mart and public station checks pass. The station platform stays
sealed; rail service and earned regional travel remain pending. See
[St. Cloud's native runbook](playable/st-cloud.md).


## Revised style authority and original characters

The user rejected the oversized mixed-tile gym designs and asked for the
original gyms as the reference. Actual complete Crystal gym maps are now saved
under `renders/gyms/originals/`. Match their proportions, coherent materials,
legible paths and trainer placement. Existing controller results establish
behavior; they do not establish that an earlier visual design was accepted.

All authored characters use invented names: no real people, existing Pokémon
leaders or other named canon characters. Minnesota city names stay geographic.
All authored gym leaders and practice trainers use original names. Generic
native trainer sprites and classes provide their appearance. Canon leader
sprites and named leader classes are excluded from these authored characters.
The source game and historical reference images remain unchanged.

## Brainerd: shifting piers, original-style revision

[Reference / revised room](renders/gyms/brainerd-og/original-style-comparison.png) ·
[Native pier comparison](renders/gyms/brainerd-og/native-pier-comparison.png)

A 5 × 8 metatile pool room uses one coherent native Port tile family: gray pool
edging, brick piers, blue water, the original-style entrance statues and carpet.
Its dimensions follow the original water gym. The previous 13 × 14 mixed
machinery/wood/stone design is rejected and retained only in render history.

Sivren is the leader. Lorla and Bravel are the two required practice trainers.
Beat Lorla `(2,10)`, then speak to her again to choose CENTER. The middle
floating walk extends at metatile `(2,4)`, giving access to Bravel `(2,7)`.
After Bravel, return to Lorla and choose LEFT. The middle walk retracts, the
left walk extends at `(1,4)`, and the upper landing `(3,2)` connects Sivren's
pier `(5,3)`. Cancel preserves the setting. Both choices remain available.
There are no machines, scattered book controls or trivia answers.

Arrival is `(4,14)`; carpet quadrants `(4,15)` and `(5,15)` both exit. The guide
is `(7,14)`. Walking the circuit needs no field move. Native dry pool edging
may reach a bounded map edge, as in the original. Every pool phase retains a
dry perimeter and no connections; actual left/right edge input stays inside
the room. A later Surf visit cannot skip the leader's required trainer checks.
The original Port brick includes LADDER collision without a matching warp,
exactly as the reference gym; it behaves as walkable pier, and there are no
extra ladder destinations or altered source collision recipes.

Four production-controller tests cover the course, loss, badge-5 refusal and
an explicitly seeded poolside placement that still cannot bypass the trainer
requirements. The course earns both battles and selected-move PP, changes and
retracts actual collision, saves on each pier position, checks repeat reward
protection and uses both carpets. Initial prior badges, party and placement
are fixtures. This proves the standalone interior; native city rebinding is
still a separate integration step. The remaining city/aggregate audit is separate.

Rochester is the second original-style remake. Its route, loss, parcel refusal,
full-TM retry and source-preservation checks pass. Native entrance, middle and
court views have been inspected. Mankato is the third remake, with three
controller checks and source-preservation checks passing. Continue individually
through the individual remakes below; the older layouts remain rejected.


## Bemidji: the Cut garden

A 6 × 10 metatile greenhouse uses the original Grass gym's lawn, flower borders,
wooden entrance and paired statues. Two native small Cut trees fit into offset
hedges. The former oversized park and SHOOT/BLOOM/SEED quiz are rejected passes.

[Complete review](renders/gyms/bemidji-og/Bemidji-review.png) ·
[Native hedge states](renders/gyms/bemidji-og/native-hedge-comparison.png)

The guide supplies HM01 once if the player does not already own it. Teach Cut
through Pack. Battle Moslen `(7,14)`, cut the west hedge at `(3,12)`, and find
Elvren `(8,9)`. Then cut the east hedge at `(7,6)` to reach Orlis `(5,3)`.
Brindel `(3,9)` offers an optional battle beside the middle court. Elvren refuses
an early challenge until Moslen is beaten; Orlis requires both wins and badge 6.
Cutting ahead cannot replace either battle. The source game's Hive Badge and
party-known Cut requirements still apply.

Walking through a freshly cut opening records that hedge as pruned. Native
coordinate events and tile callbacks keep it clear after battle, Continue and
return visits. Saving on the former tree tile is supported. The other native
collision quadrants remain intact; the tall hedge beside each gap still blocks
movement. The rest of the game's temporary vegetation behavior is unchanged.
The return uses the two cleared openings, with no extra switch or quiz.

Arrival is `(4,17)` and the guide stands at `(9,17)`. Both carpets `(4,19)` and
`(5,19)` return to Bemidji's actual facade, safe street tile `(10,10)`, exterior
warp 2. The standalone review pack instead uses its separate review hall.
Orlis keeps Tangela 38, Jumpluff 39 and Bellossom 40, earning Pine and one Giga
Drain TM. All character names are original and native classes are generic.

The individual pack contains 473 maps; the city-bound pack contains 479. City
lowering preserves the complete canonical room, art, parties, guide gift and
puzzle scripts, changing only three return destinations. Party, prior badges
and starting street placement are explicit fixtures. Native local play is
verified separately from the still-unearned regional journey and Q05 report.


## Duluth: the ice courses

A 7 × 10 metatile room uses the original ice gym's native white ice, small
rough floor patches, metal stops, blue floor, statues and carpet. The large
wood-and-stone twin-bay draft is rejected. Two short slides form the puzzle;
trainer wins change actual collision instead of answering quizzes.

[Complete review](renders/gyms/duluth-og/Duluth-review.png) ·
[Native court states](renders/gyms/duluth-og/native-court-comparison.png) ·
[Native city facade](renders/gyms/duluth-og/native-city-facade.png)

Start the west course at `(2,16)`: UP stops at `(2,8)`, RIGHT reaches the rough
patch `(4,8)`, and UP reaches the dry landing `(4,7)`. Find Livra `(3,4)`.
Her win opens the west return curb and the east launch. Anver `(5,6)` can be
approached on this side, but requires Tordel's victory before battling.

At the east launch `(11,16)`, go UP to `(11,10)`, LEFT to the rough patch
`(9,10)`, then UP to `(9,7)` and Tordel `(9,6)`. A full stop at the eastern
edge permits the reverse trip: return to the patch, slide RIGHT, then DOWN.
The player can retreat before fighting Tordel. His win opens the cross-passage
at metatile `(3,3)` to Anver, avoiding another ice crossing.

Anver's win opens the upper court at metatile `(3,2)`. Norek `(8,3)` requires
all three victories and badge 7. He retains Dewgong 42, Piloswine 43, Sneasel 44
and Lapras 45, awarding Harbor and one Icy Wind TM. Names are original; classes
are generic Cooltrainer and Sailor. The source badge permission remains the
original eighth engine slot.

Arrival is `(6,17)`; the guide is `(10,17)`. Both carpets `(6,19)` / `(7,19)`
return to the actual Duluth street `(16,12)`, passive exterior warp 4. The local
city course earns all three wins and badge 8, then walks to the real workshop
and acknowledges its receiver. Badge 8 alone does not set the signal. Prior
badges, the upstream depot event, initial street position and party are explicit
fixtures; this is not an earned regional campaign.

Four standalone controller checks pass, including live ice and opposing input,
empty-frame travel, blocked stops, turning after Continue on ice/floor, both
return paths, loss, prior-badge refusal, PP use, single reward and both carpets.
Six existing city checks and the new gym-to-receiver course pass on the native
city candidate. The combined eight-room pack also passes its complete course.

## Current circuit and remaining integration

[All eight current rooms at one native tile scale](renders/gyms/remade-circuit/all-gym-interiors.png).

`content-packs/minnesota-gyms-remake.crystalpack` contains all eight reviewed
rooms and their separate review hall, 480 maps total. Every source terrain pixel
and collision quadrant matches publication, including the explicit native wind
exception. The combined controller test earns each room's required battles,
puzzle states, badge, single TM and Continue from explicit local fixtures.
It does not prove a continuous eight-city journey.

The separate 498-map northern candidate binds Brainerd, Bemidji and Duluth to
their actual facades. Brainerd's five-facade/both-carpet check passes; canonical
module comparison preserves each gym's full behavior while changing only its
three return wires. Minneapolis has its separately verified city candidate.
Rochester's current first gym is also bound in its 526-map opening/story pack;
fresh native checks cover delivery, trainers, reward and both city returns.
Mankato's 483-map local candidate binds the second gym, clinic and mart. Its
native terrace hop, full gym course, healing, PC access, priced purchase, all
room returns and Continue pass production input. See [Mankato](playable/mankato.md).
St. Paul's 488-map local successor binds all five public facades, retains every
internal gym pad, and proves the two-lane reciprocal R04 crossing, full gym
course, services and every room carpet. See [St. Paul](playable/st-paul.md).
Its Q03 courier and rail service remain unfinished. St. Cloud's 492-map successor
adds four actual facades, the canonical wind course, services and both-lane
reciprocal R05/R06 crossings. See [St. Cloud](playable/st-cloud.md).
The [538-map regional successor](playable/eight-gyms.md) now consumes these
current rooms, the opening/northern story and all city bindings in one pack.
Preservation checks and a single production suite prove all eight local gym
courses, badges/TMs, Continue and both actual city returns. The caught-location
limit exposed during integration is fixed without renumbering existing areas.
Later successors bind [R02 and its optional notebook](playable/prairie-story.md)
and [R03 with the Prairie Badge gate](playable/river-cities.md). The
[fourth-gym approach](playable/gym-four-approach.md) now opens Minneapolis's east
road through R04 to St Paul with its Current Badge checkpoint.
Its continuous local course earns the rescue and third/fourth gym courses and
walks home; initial party, prior badges and resources remain explicit fixtures.
The courier, lake-country and Leech shore chapters have separate tested
successors. Remaining regional links and the continuous earned badge journey
stay unfinished. Local courses do not substitute for that final audit.

The [connected-border integration audit](playable/gym-border-audit.md) fixes a
production R05 crossing into a visible forest wall, preserving the source art.
The St Cloud local course now earns Wind before its guarded north departure
and traverses the actual outbound/inbound checkpoint scripts on both lanes.

The [final gym approach](playable/gym-eight-approach.md) now passes one
continuous mine/foreman/recovery/handover/R10/Duluth course and return. Its two
level45 review partners, seven prior badges and upstream report are initial
fixtures; Norek's badge and TM are earned through his actual facade and course.
