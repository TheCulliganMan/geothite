# Minnesota gym profiles

The eight gyms follow the Winona-to-Duluth journey. Each has a physical course,
named trainers and a leader battle. This profile describes those courses without
repeating historical test logs or importing obsolete Pokémon teams.

[gyms/current.json](gyms/current.json) selects the authored room revisions.
[story.json](story.json) defines badge slots and base battle contracts;
[story-pokemon.json](story-pokemon.json) and the active curated roster determine
Minnesota species. The base records still contain legacy import names, so they
must not be copied directly as current regional parties. Use the final bound
pack for legal moves, evolution stages, items and field permissions.

## Circuit

| Slot | City | Leader | Type | Badge | Course |
| --- | --- | --- | --- | --- | --- |
| 1 | Rochester | Varel | Normal | Care | Two switches alternate the open lanes |
| 2 | Mankato | Torven | Ground | Prairie | Stair gates and one-way terrace drops |
| 3 | Minneapolis | Avrin | Electric | Current | Trainer victories and neighboring circuit switches |
| 4 | St. Paul | Brannic | Steel | Capitol | Four rooms joined by a continuous transfer-pad loop |
| 5 | St. Cloud | Saven | Flying | Wind | Four forced-wind lanes with visible arrows |
| 6 | Brainerd | Sivren | Water | Lake | One dock control alternates the center and left walks |
| 7 | Bemidji | Orlis | Grass | Pine | Two Cut hedges and a victory return gate |
| 8 | Duluth | Norek | Ice | Harbor | Two ice courses, permanent stops and trainer gates |

The type and badge names identify the authored leaders. Do not invent a
biography or civic role from a puzzle theme. Art direction is recorded in the
[trainer cast](npcs/cast/README.md).

## Rochester: Varel

The first gym is a 7 × 7 metatile training room with two blue wall switches,
three gold gates, a left training bay and a right lane to the leader.

Beat Tavel, then press the lower switch to enter the left bay. Beat Elvik and
press the upper switch to open the gate below Varel. Return to the first switch
and flip it again: the left bay closes and the right lane opens. The physical
return teaches that changing a control can make an earlier route useful.
Switches respond directly to A. Varel also requires Ivy's completed delivery.

[Room contract and historical checks](gyms/rochester-gates.md).

## Mankato: Torven

Sand marks the raised terraces; darker soil separates the central arena.
Wooden gates guard the two ordinary stairs.

Beat Caldren, climb the left stair and drop toward Jorel. The bank blocks the
reverse jump, while a lower drop permits retreat before the battle. Jorel's
win opens the upper stair to Torven. Short return drops make the trip out
quicker. Both trainer wins are required; no field move or quiz replaces them.

[Room contract and historical checks](gyms/mankato-terraces.md).

## Minneapolis: Avrin

A 5 × 9 metatile room uses six cans, pale floor, blue trim and red electrical
barriers. Jossa stands between the two locks.

1. Beat Lenvi and use her clue to press the left can in the top row.
2. Cross the opened lower barrier and beat Jossa.
3. Press the neighboring can to the right to open the upper barrier.

A wrong second choice resets the locks without resetting trainer victories.
The second circuit refuses early input while preserving the first lock. Once
solved, the course stays open. Avrin requires the Prairie Badge.

[Room contract and historical checks](gyms/minneapolis-circuit.md).

## St. Paul: Brannic

The entrance, Belva's room, Orven's room and leader landing form one continuous
pad loop. Arrival and onward pads occupy opposing corners, so each room has a
readable direction of travel.

Cross Belva's room, then Orven's. Both wins release the leader's metal grille.
The far pad after Orven goes directly to Brannic's landing; its other pad is
the short return to the entrance. Pad arrivals must not immediately bounce the
player back, including after Continue. Brannic requires Current.

[Room contract and historical checks](gyms/st-paul-loop.md).

## St. Cloud: Saven

Arrows show the direction of four wind lanes before the player steps onto them.
Ordinary stone landings separate the forced movement.

Beat Wyrel, ride the lower eastbound lane to Hallen, then beat Hallen and take
the upper westbound lane toward Saven. The top eastbound and bottom westbound
lanes provide the return. Entering from the wrong end returns to the same bank;
counter-steering cannot override the wind. Saven requires Capitol.

[Room contract and historical checks](gyms/st-cloud-arrows.md).

## Brainerd: Sivren

Brick piers surround the pool. A visible blue dock control replaces a menu
hidden in a trainer conversation.

Beat Lorla, press the control to center the walk, cross and beat Bravel. Return
to the control and press again: the center retracts and the left walk opens
Sivren's approach. An early extra press cannot skip Bravel, and another press
restores a safe crossing. The walking course needs no field move. Sivren
requires Wind.

[Room contract and historical checks](gyms/brainerd-winch.md).

## Bemidji: Orlis

A 6 × 10 metatile greenhouse uses lawn, flower borders and two small Cut trees
fitted into offset hedges. The guide offers HM01 once if it is missing.

Teach Cut through Pack. Beat Moslen, cut the west hedge, reach Elvren, then cut
the east hedge to approach Orlis. Brindel is an optional middle-court battle.
Elvren requires Moslen's victory; Orlis requires both mandatory trainers and
Lake. Cutting ahead cannot replace those wins. Retain the native badge and
party-known-move checks for Cut.

Walking through a cut opening records that hedge as pruned. It stays clear
after battle, Continue and return visits, including a save on the former tree
tile. After victory, the gold south gate provides a shorter return.

[Course contract](gyms/README.md) and
[victory-return revision](gyms/bemidji-return.md).

## Duluth: Norek

A 7 × 10 metatile ice room separates permanent silver stops from gold gates
opened by trainer victories. Rough patches provide readable turning points.

The west course leads to Livra. Her victory opens the return curb and east
launch. The east course leads to Tordel; his victory opens the cross-passage
to Anver. Anver requires Tordel's win, and his own win opens the upper court.
Norek requires all three victories and Pine. The route permits retreat before
the eastern fight.

Harbor is the eighth badge. It does not complete the harbor crew's separate
conversation or set the northern signal by itself.

[Room contract and historical checks](gyms/duluth-gold-gates.md).

## Rewards and evidence

Leader victory uses the real battle result, native badge slot and once-only TM
reward. A loss grants neither badge nor reward. Completed trainer and puzzle
states persist as authored; repeats cannot duplicate the reward. Keep both
entrance carpets connected to the actual city facade.

The [regional gym publication record](gyms/current-regional.md) describes two
historical parent packs and their local fixture courses. The
[current story composition](revisions/current-story-v01/README.md) and later
world revisions have separate identities. Neither this profile nor an earlier
room screenshot certifies every later composition.

This documentation cleanup adds no battle, visual or balance acceptance.
Recorded local courses must retain their initial party and prior-badge limits;
a continuous ordinary-party journey through all eight cities remains a
separate requirement.
