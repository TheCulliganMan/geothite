# Whole-interior scope and remaining work

The objective remains every Minnesota interior except the eight gyms. The
baseline source reservations contained 210 distinct non-gym rooms. The installed
Rainy River warming hut is additional to that list; its purpose and residents
are already covered by [stories.json](stories.json). The combined scope is
therefore 211 registered/installed room contracts, including gates and caves as
well as ordinary buildings. The Glacier Line source additionally names three
unregistered stations: Staples, Red Wing and Winona. The objective retains those
planned interiors too, for at least 214 rooms; the reservation catalog alone is
not the full completion boundary. These counts do not claim visual/gameplay acceptance.

The [International Falls child](INTERNATIONAL-FALLS.md),
`minnesota-international-falls-v02.crystalpack`, contains 210 of those rooms: the
187-room narrative parent, Ely's nine interiors, three fair rooms, two ice
floors, five Detroit Lakes interiors, two Moorhead interiors and two Falls rooms.
Falls' four production cases, strict preservation and thirteen native 2D checks pass;
the earlier regional checks remain recorded in their publication runbooks.
The current native indoor/gate/dungeon inventory contains no additional
Minnesota rooms outside this union. Every installed room except the deliberate
lower ice-puzzle floor has a script/trainer actor; that floor has two native
discoveries and shares the upper floor's guide. The narrative contract records
the first 187 purposes and conversations. Ely's existing town/League sources supply its services, people
and chapter. Physical refinement currently covers 40 room contracts and 47
furniture details; those counts do not imply that all other rooms have passed
the same physical review.

## Completed branch composition

The existing fairgrounds builder supplies `MnFairGrandstand`, `MnFairBarn` and
`MnFairDairy`, each with residents and activities. Its documented prerequisite
is the regional species, move and animation pipeline. The compact St. Paul
connection is explicitly authored in [fair-city-connection.json](fair-city-connection.json)
and must preserve every existing facade and river seam. Publication, preservation,
production seam/room/activity checks and native review now pass on the composed
output; their exact inputs and results are recorded in the integration runbook.

The existing ice publisher supplies `MnVoyageursIce1F` and
`MnVoyageursIceB1F`, with their native sliding, rewards, ladder and return
contracts. The two floors now pass composition preservation, actual controller
replay and full native camera comparison on the same lineage. The final output still
lacks the older shore-gate contract and three planned
stations below.

## Remaining registered rooms and transit source contracts

| Place | Rooms | Authoritative state and work still required |
| --- | --- | --- |
| Detroit Lakes | Published clinic, Mart, station, canoe cabin and ranger cabin | All five rooms, residents, carpets and three reciprocal roads pass production input. Care/PC/poison recovery, priced shop and SQ01 retry/persistence pass. Full Glacier Line service remains unfinished. See the separate runbook. |
| Moorhead | Published clinic and Mart | Both rooms, ten town/indoor residents, free care/PC/recovery, priced stock, the remembered flood-notebook gift and both R16 lanes pass production input. Thirteen full native frames and the complete levee Surf barrier pass. See [Moorhead](MOORHEAD.md). |
| International Falls | Published clinic/ranger desk and Mart | Nine residents, both service rooms, all carpets, both road lanes, care/PC/recovery and priced stock pass. The complete actual icefield/Voyageurs round trip, both far-camp entry lanes, full-pocket reward retry and Save/Continue pass. See [Falls](INTERNATIONAL-FALLS.md). Regional 3D review remains open. |
| Glacier Line | Staples, Red Wing and Winona stations | These three names are present in the six-stop network source but absent from the shared room reservations and current native pack. Distinct station/outpost geometry, real entrances, residents, visited callbacks and the full guarded ride network remain to author and verify. |
| Grand Marais | Older shore-gate contract | `MnGrandMaraisShoreGate` is absent from the current pack. Concurrent source revisions have removed it from the live reservation list and replaced the old connection dossier with direct-seam geography. This does not itself resolve the baseline interior requirement. Reconcile the physical/story contract explicitly against the actual town, entrances and route connections. |

Sources are [the shared reservations](../interiors.json),
[Detroit Lakes](../towns/MnDetroitLakes.json),
[its station](../transit/MnDetroitLakesStation.json),
[Moorhead](../towns/MnMoorhead.json),
[International Falls](../towns/MnInternationalFalls.json) and
[the older shore-gate dossier](../towns/MnGrandMarais.connections.json), and
[the six-stop transit contract](../transit/glacier-line.json).
Missing rooms are implementation work, not a request for the user to supply a
different pack. Preserve the existing regional gates, story, services and audio
while integrating them.

The original 705-map endpoint audit found R16's western connection unbound;
Moorhead now binds both lanes while preserving its eastern Detroit Lakes
approach. Falls now binds R17's north and a bounded western icefield approach.
R20's west edge
already connects to `MnRainyRiverIcefields`; do not overwrite that working
connection to fit the older International Falls source diagram. The published
town fits the actual northern/icefield topology and preserves its gates.

## Remaining acceptance

Each real building needs reachable native entrances/returns, residents and a
concrete Minnesota purpose, with useful services, conversations, discoveries or
story behavior. Test the production controller, including input ownership,
money/items/HP/PP where relevant and persistence. Inspect complete source rooms
and actual native cameras. Existing dialogue coverage, map counts or a successful
pack read alone do not establish that acceptance.

The [harbor-care continuation](HARBOR-CARE.md) repairs the Duluth field clinic,
beach shelter and original harbor workshop. All 47 installed Duluth non-gym
rooms now have a whole-room source and native arrival review; services, shops
and optional matches retain production-controller checks. The
bedroom camera comparison now passes with independently derived 4× room-fit
geometry on the 708-map child. St. Paul's separate off-map comparison also
passes after including the native compositor's second-hop Mississippi scenery;
both fresh full frames were personally inspected. Missing-town publication,
room-quality review and the final completion audit remain active.

The current live registry has 217 distinct entries, including eight gyms:
209 non-gym reservations, all now installed in the current child.
The additional warming hut, baseline shore-gate requirement and three planned
stations retain the full minimum 214-room scope above. Removing a reservation
from a concurrent source file is not evidence of completed gameplay or review.

Fresh regional 3D acceptance is still open. The headless St. Paul capture
times out while building authored terrain and repeatedly reports a neighboring
map exceeding the connected-actor 16-slot presentation namespace. The actual
command and log are retained in `renders/interior-life/voxel-follow-up-v01/`.
Authored art aliases also need their complete native building provenance checked
in 3D. Passing 2D pixels and room counts do not close those defects.
