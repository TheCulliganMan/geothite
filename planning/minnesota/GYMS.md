# Minnesota gym circuit

Finish one gym before designing the next. Give it a simple physical puzzle,
make its trainers part of the route to the leader, inspect the complete room
and the actual player view, and prove its battles, gates, exits and persistence.
Vary the mechanic between gyms. Do not fill rooms with book sprites, quiz
stations or interchangeable attendants.

**The user rejected the old gym visuals.** Earlier remakes of all eight rooms
had native-art and controller checks on the480-map room pack and541-map story
successor. Those checks describe historical candidates, not user acceptance.
The current improvement pass has revised all eight rooms on the cumulative541-map
candidate and now rebuilds in the [554-map story and 660-map compact-city branches](gyms/current-regional.md).
Continuous earned regional travel remains incomplete. Earlier renders remain archived.
[City progression](PROGRESSION.md) describes the regional journey separately.
[Implementation audit](gyms/implementation-audit.md) records the complete room checks and their limits.

The user rejected the subsequent planter revision too: its walls were unclear
and forced a walk without a useful decision. Rochester now uses
[a smaller two-switch lane course](gyms/rochester-gates.md). The next seven gyms
also have new revisions below. Earlier test passes do not
finish the improvement request or establish user visual approval.

| Order | City / leader | Type / badge | Next design direction | Status |
| --- | --- | --- | --- | --- |
| 1 | Rochester / Varel | Normal / Care | Two switches; left bay, upper gate, return to right lane | Published7×7 candidate; literal controls and native gate-state review pass |
| 2 | Mankato / Torven | Ground / Prairie | Light sand terraces, wooden stair gates and native drops | Revised candidate; ordinary-party literal course, retreat and native hop views pass |
| 3 | Minneapolis / Avrin | Electric / Current | Lenvi, first lock, Jossa, neighboring switch | Staged circuit candidate; ordinary-party literal controls and native barrier views pass |
| 4 | St. Paul / Brannic | Steel / Capitol | Two pads per room form one continuous loop | Revised candidate; ordinary-party literal course, all eight endpoints, reverse order and native views pass |
| 5 | St. Cloud / Saven | Flying / Wind | Visible floor arrows explain four forced-wind lanes | Revised candidate; ordinary-party literal route, wrong-way/counter-steering checks and native views pass |
| 6 | Brainerd / Sivren | Water / Lake | One visible blue control toggles center/left piers | Revised candidate; ordinary-team literal route, no-menu control, retraction and native state views pass |
| 7 | Bemidji / Orlis | Grass / Pine | Two native Cut hedges; gold victory return gate | Revised candidate; ordinary-team literal course, Cut/trainer denials and native state views pass |
| 8 | Duluth / Norek | Ice / Harbor | Two ice courses; silver stops and gold trainer gates | Revised candidate; native slide/trainer/persistence cases and five native views pass |

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

## Mankato: sand terraces and native drops

Light sand marks the raised western and leader terraces; darker soil fills
the central arena. Wood is confined to the entrance and stair gates. Empty
northwest floor and a redundant boulder were removed. Both gray stairs are
visible before their gates open.

Beat Caldren and climb the left stair, then hop down to Jorel. The native bank
blocks the reverse; a lower drop lets the player retreat before that fight.
Jorel's win opens the upper stair to Torven. After the leader, two short drops
make the return quicker. Both trainers remain required, with no switches,
quizzes or field move.

[Course animation](renders/gyms/mankato-ledge-review/sequence/mankato-course.gif) ·
[Whole room](renders/gyms/mankato-ledge-review/current-circuit/MnMankatoGym-whole.png) ·
[Native hop approach](renders/gyms/mankato-ledge-review/native-hop-top.png) ·
[Exact contracts and proof](gyms/mankato-terraces.md)

The fixed control run passes with Bayleef19/Pidgey17 and the initial Care Badge
fixture. It earns all three wins, spends native PP, checks every two-tile jump
and blocked reverse, retreats before Jorel, verifies Continue/reward protection
and uses both city carpets. The cumulative pack preserves Rochester's gate
revision, the other six rooms and all other maps/battles. Current source and
native art match. User visual acceptance remains open.

## Minneapolis: trainers between the locks

The small5×9 room keeps the pale floor, blue trim and six cans. Red electrical
barriers replace the gray blocks. Jossa now stands between the locks, so
trainer battles and switch decisions alternate on the way to Avrin.

1. Beat Lenvi. Her clue identifies the left can in the top row.
2. Press that can. The lower barrier disappears, revealing the route to Jossa.
3. Beat Jossa. Her clue puts the second switch beside the first, in the same row.
4. Press the neighboring can to the right. The upper barrier disappears.

A wrong second choice resets both locks while keeping both trainer wins. Before
Jossa's battle, the second circuit refuses input without losing the first lock.
Once solved, all cans leave the circuit open. Badge2 still gates Avrin's battle.

[Room-state animation](renders/gyms/minneapolis-circuit/sequence/minneapolis-course.gif) ·
[Native closed barriers](renders/gyms/minneapolis-circuit/native/locked.png) ·
[Native first barrier open](renders/gyms/minneapolis-circuit/native/first-open.png) ·
[Exact contracts and proof](gyms/minneapolis-circuit.md)

The literal control course passes with Geodude23/Bayleef23 using native level-up
moves, starting with two prior badges. It earns all three wins, checks denial,
wrong choices/reset, Continue, a single badge/TM and both real city returns.
The two inspected native cameras exactly match source pixels at the same
position, facing, time and seeded gate state. The animation illustrates room
states rather than recording play. The cumulative541-map pack preserves the
first two gym revisions; later city/story successors need separate integration.

## St. Paul: cross each room to the far pad

Four small rooms form one continuous pad loop: entrance, Belva, Orven, leader
landing, entrance. The regular rooms put arrival and onward pads in opposing
corners. After Orven, the far pad now goes directly to the leader landing;
there is no extra trip through the foyer. The landing's other pad is the short
return to the entrance. Both trainer wins release its visible metal grille.

[Illustrated route](renders/gyms/st-paul-diagonal/sequence/st-paul-loop.gif) ·
[Native onward pad](renders/gyms/st-paul-diagonal/native/belva-forward.png) ·
[Native opened court](renders/gyms/st-paul-diagonal/native/court-open.png) ·
[Exact contracts and proof](gyms/st-paul-loop.md)

The fixed control course passes with Quilava27/Geodude25 and native level-up
moves. It earns both trainers and Brannic, uses every pad endpoint in both
directions, verifies no arrival/Continue bounce, checks the early solid grille,
one badge/TM and both actual city carpets. Reverse inspection order, missing
badge3 and loss negatives also pass. The cumulative541-map pack preserves the
first three revisions and all other maps/battles. Two native camera comparisons
match every source pixel. Four gyms still await this improvement pass; user
acceptance and continuous earned regional progression remain open.

## St. Cloud: read the arrows

The eastbound and westbound lanes formerly used identical slats. Native arrow
markings now show the direction on every walking tile, before the player steps
onto it. The underlying four-lane course and safe stone landings stay intact.
The guide and defeated trainers give shorter directional hints.

Beat Wyrel, ride the lower eastbound lane to Hallen, then beat Hallen and ride
the upper westbound lane toward Saven. After the leader, return east on the top
lane and west on the bottom lane. Entering from the wrong end returns to the
same bank; steering against the wind cannot override its carry.

[Illustrated route](renders/gyms/st-cloud-arrows/sequence/st-cloud-wind.gif) ·
[Native lower crossing](renders/gyms/st-cloud-arrows/native/lower-east.png) ·
[Native upper crossing](renders/gyms/st-cloud-arrows/native/upper-west.png) ·
[Exact contracts and proof](gyms/st-cloud-arrows.md)

The fixed control course passes with Ampharos31/Graveler30 using native level-up
moves and four initial prior badges. It earns all three wins, exercises every
lane, wrong-way entries, counter-steering, Continue, one reward and both real
city carpets. Loss and missing badge4 checks also pass. All ten revision asset
cases preserve the first four revisions, every collision and all unrelated
content. Both inspected native cameras match every source pixel. The cumulative
pack has541 maps. Three designs, user acceptance and earned regional integration
remain open.

## Brainerd: press the dock control

The small Port-style pool keeps its brick piers and gray edging. A single blue
control replaces the right entrance statue. A changes the pier position
immediately; Lorla's repeat conversation now gives a clue instead of a menu.

Beat Lorla, press the control to center the walk, cross and beat Bravel, then
return and press again. The center walk retracts and the left walk opens the
route to Sivren. An early extra press cannot skip Bravel; another press safely
restores the center. No field move is required for the walking course.

[Illustrated route](renders/gyms/brainerd-winch/sequence/brainerd-winch.gif) ·
[Native control](renders/gyms/brainerd-winch/native/dock-control.png) ·
[Center walk](renders/gyms/brainerd-winch/native/center-walk.png) ·
[Left walk](renders/gyms/brainerd-winch/native/left-walk.png) ·
[Exact contracts and proof](gyms/brainerd-winch.md)

The literal course passes with Sunflora36/Ampharos35/Venusaur36, native level-up
moves, matchup-based PKMN switches and five initial prior badges. It earns all
three battles, checks early presses, reversals, actual water retraction,
Continue, one reward and both real city carpets. Missing badge5, loss and a
separately seeded poolside-denial case pass. All twelve asset cases preserve
the first five revisions and unrelated content. Every collision outside the
new control remains exact. Three inspected native state views match all source
pixels. The cumulative pack has541 maps; two gym designs remain to review.
User acceptance and continuous earned regional integration remain open.

## Bemidji: the Cut garden

The current [victory-return revision](gyms/bemidji-return.md) keeps this course
and opens a gold gate in the south hedge after Orlis. One bush above it becomes
lawn. The return from the leader approach takes18 steps rather than22 through
the already-pruned garden. Its literal ordinary-team route and all original
Cut/required-trainer/persistence cases pass on the cumulative541-map pack;
both native gate-state cameras match source pixels. The details and older pack
counts below describe the historical parent.

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
The historical return uses the two cleared openings. The current victory gate
shortens that walk without adding a switch or quiz.

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

The current [gold-gate revision](gyms/duluth-gold-gates.md) distinguishes the four
moving gates from permanent silver stops. Their native gold panels disappear
after the existing trainer wins; slide routes and collision stay exact. Four
existing controller cases, sixteen revision asset cases and five native-state
camera comparisons pass. The cumulative541-map candidate preserves all seven
prior revisions. Earlier individual/city pack counts below describe the
historical parent rather than the latest regional composition.

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
