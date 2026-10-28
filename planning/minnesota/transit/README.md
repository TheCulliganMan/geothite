# Glacier Line

Current native status: the composed Detroit Lakes child contains the Detroit
Lakes, St. Cloud and St. Paul public station halls with real city entrances.
Detroit Lakes publication and verification are recorded in
[its runbook](../interior-life/DETROIT-LAKES.md). No train service or visited
callbacks are installed. Staples, Red Wing and Winona still need distinct
station/outpost sources and native publication. The full six-stop service
contract below remains unfinished. Earlier source-only notes describe the
original depot proposal; current St. Paul exterior coordinates come from its
[repair contract](../towns/st-paul-repair/contract.json).

The line connects **Detroit Lakes — Staples — St. Cloud — St. Paul — Red Wing —
Winona** in both directions. This follows the Minnesota stops in Amtrak's
[Empire Builder timetable](https://content.amtrak.com/content/timetable/Empire%20Builder.pdf).
Glacier Line is this mod's fictional service name. Moorhead connects to Detroit
Lakes by the existing prairie route; Minneapolis connects through St. Paul.
Neither needs a duplicate rail stop. Staples and Red Wing are rail outposts,
separate from the nineteen main hub reservations and eight gyms.

The source of truth is [glacier-line.json](glacier-line.json). The first complete
station interior is [MnStPaulUnionDepot.json](MnStPaulUnionDepot.json), a hand-set
16×12 metatile room using the unchanged native `train_station` art. Its source
render is [the complete depot](../renders/glacier-line/st-paul-union-depot.png),
with [160×144 walking views](../renders/glacier-line/st-paul-player-views.png).
These original images are source reviews. The current three public halls are
installed, with their own later native review; the six-stop ride service is not.

The original depot pass left an empty blue central concourse. The second pass
adds a planted seating island and an actual cabinet edge near the courier,
leaving connected circulation on both sides. The ticket room opens off the
west; a sheltered garden opens off the east; the red exit carpet returns south
to the future city approach. The whole-map render includes the entire platform
and train. Review its proportions against the saved Goldenrod/Saffron source
station references in the same render directory.

## Physical implementation contract

Coordinates below are 16px core tiles. Rows in the JSON are 32px metatiles.

| Purpose | Exact source anchor |
| --- | --- |
| City-door arrival inside | `(15,21)`, Floor |
| Exit carpet triggers | `(14,23)`, `(15,23)`, `(16,23)`, `(17,23)` |
| Safe passive indoor arrival | `(15,21)`, Floor, proposed native warp5 |
| Conductor | `(17,9)`, Floor on the only public platform throat |
| Boarding approach | `(17,10)`, Floor |
| Protected ride arrival | `(17,11)`, Floor in public hall |
| Platform | `(17,7)`, Floor |
| Native train doors | `(12,5)` and `(17,5)`, Door, inaccessible before boarding |
| Q03 courier | `(20,16)`, Floor beside the timetable cabinet |
| Readable timetable cabinet | `(21,15)`, Wall, public approach from `(20,15)` |
| Ticket clerk | `(7,13)`, Floor in the wooden service room |

The existing native fence blocks close every other platform approach. With the
conductor present, a public collision flood cannot reach the platform or either
train door. Removing only that conductor opens the intended throat. This is a
source geometry check, not controller proof of the boarding movement.

Before boarding, move the conductor up twice and right twice to `(19,7)`. From
the fixed `(17,10)` approach, the player walks up three, left five, up two to the
left train door `(12,5)`. Restore the conductor left twice and down twice before
leaving the map so repeat visits cannot inherit an open throat. Native collision
and object movement must be checked through `VisibleShellController`; these
authored steps have not been executed yet.

The St. Paul source author reserves exterior door `(44,37)` and passive Floor
return `(44,38)`. Freeze the proposed native ordering explicitly: four indoor
carpet exits are warps1–4, then the safe Floor arrival `(15,21)` is warp5.
Exterior door warp1 targets indoor5; exterior passive return is warp2; all four
indoor carpets target exterior2. These are source bindings awaiting actual
Rust publication and controller proof. They are not an existing native module.
The exterior door must target the indoor safe Floor arrival, not the carpet,
platform or train doors. Source compilation must reject missing bindings.

## Service and story implementation contract

Q03's courier win sets `EVENT_MN_COURIER_DEFEATED`, which activates service.
This is the St. Paul courier event; Grand Rapids' Q06 depot flags are unrelated.
The canonical courier encounter is now bound to the native depot; retain its
existing upstream prerequisites, party and acknowledged post-battle event.
Ordinary conversation must never substitute for that win.

A native first-entry callback records each station's own visited flag. Players
can ride back to stations they have physically reached. Passing an intermediate
stop does not unlock it. Keep the destination menu in geographic line order and
show unvisited stops as unavailable. Recheck both service activation and the
selected destination's flag immediately before movement; menu presentation is
not access control. No fare, new mandatory PASS quest or granted badge is needed.

Use the existing production mechanisms: `loadmenu` / `verticalmenu` /
`closewindow` for destination selection, `checkevent` for both guards,
`applymovement` for boarding, `setval` plus `special MagnetTrain` for the native
animated ride, then the explicit destination map-load path and a Floor arrival
south of the destination conductor. The engine already settles both train
directions and preserves the source script across train map loads. Inspect
[the production train regression](../../../crates/crystal-bevy/src/bevy_shell/tests/audio_and_battle_ui.rs:393)
and the script-menu parsers before lowering the authored contract. Do not make a
second travel dispatcher or substitute a captioned teleport for the ride.

Freeze each remaining station's geometry separately: timber/lake district at
Detroit Lakes, compact freight junction at Staples, granite/riverbank district
at St. Cloud, bluff-foot lookout at Red Wing and levee district at Winona.
No repeated six-station template. Complete each public approach and its honest
city or route linkage before describing that stop as playable.

## Fast source loop and acceptance

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_glacier_line.py
```

On the current warm environment, the actual render/check stage measured about
0.10 seconds after pack loading. It checks row extents, actor approaches and
native sprite availability, public doors, a sealed platform throat, Floor
arrivals, boundary closure and unique station flags. Four deliberately broken
variants reject a second throat, an arrival behind the conductor, an arrival
on the conductor and shared visit flags. Exact station/network source copies,
whole terrain, actor composite, conversion stages, all camera crops and nearest
previews remain under immutable hash directories; latest aliases are extra copies.

Before publishing service, test all thirty directed stop bindings and actual
controller rides in both directions, pre-courier denial, unvisited destination
denial, B cancellation, repeat visits, free departure/arrival, city-door returns,
Save/Continue and unaffected money/party/story flags. Retain actual native train
captures and empty-stderr MCP evidence. These checks are outstanding.
