# Minneapolis waterfront frontages

Four west-bank buildings now have distinct, complete native exteriors: a slate
meeting-house roof, a shallow pitched angler roof, a weatherboard family house
and a low timber lake cabin. This follows the earlier city-wide repair of
internal wall edges and second-floor bottoms; it does not replace those joins
with repeated roof or ground-floor fragments.

The published local candidate is the ignored
`content-packs/minnesota-waterfront-frontages-v01.crystalpack`, SHA256
`3862c34b044a456721b5f7ee5464b42dd123f07e3bd12d16e7e503bec972ffa2`.
It retains the compact Superior coast, snowy pine Ely, Split Rock clinic and
pine-knoll back route from its exact parent. See [PUBLICATION.md](PUBLICATION.md)
for commands, identities and verification limits.

## Whole place and player cameras

- [Complete native city, actors included](../../renders/minneapolis-waterfront-v01/native-v02/whole-city-day-actors.png)
- [Whole waterfront before and after](../../renders/minneapolis-waterfront-v01/source-v06/waterfront-before-candidate.png)
- [Native waterfront at night](../../renders/minneapolis-waterfront-v01/native-v02/whole-waterfront-nite.png)
- Actual native captures: [boathouse](../../renders/minneapolis-waterfront-v01/native-fixtures-v02/boathouse.png), [angler](../../renders/minneapolis-waterfront-v01/native-fixtures-v02/angler.png), [dispatcher](../../renders/minneapolis-waterfront-v01/native-fixtures-v02/dispatcher.png), [cabin](../../renders/minneapolis-waterfront-v01/native-fixtures-v02/cabin.png).
- [All four actual GPU cameras, before and after](../../renders/minneapolis-waterfront-v01/native-paired-v01/all-four-before-after.png)
- [All current outdoor maps connected at one scale](../../renders/minneapolis-waterfront-v01/state-continuity-v01/full-state-native-current.png)

The whole city, paired waterfront, four before/after native cameras and night
waterfront were inspected. The cameras use the same map, passive door return,
noon clock, default facing and 640×576 viewport. These are explicit positioned
visual fixtures, not an earned journey. Day, morning, night and dark source
banks are preserved and reviewed separately from GPU evidence.

## Exact implementation contract

All positions below are 16px gameplay tiles. Footprints are 32px metatiles,
with upper bounds exclusive. No room, actor, story flag, reward or service is
added by changing these fronts.

| Building | Metatile footprint | Real door | Passive return | Existing purpose |
| --- | --- | --- | --- | --- |
| BoatHouse | (2,12)–(6,15) | (9,29) | (9,30) | Host and meeting house; no rental service |
| AnglerHome | (8,13)–(10,15) | (19,29) | (19,30) | Inhabited home and existing fishing advice |
| DispatcherHouse | (11,19)–(13,21) | (23,41) | (23,42) | Family home and existing rescue witness |
| LakeCabin | (2,21)–(5,23) | (7,45) | (7,46) | Existing cabin host; no inn service |

The meeting house keeps its joined cyan-brick body and reuses existing native
slate blocks80/82/82/81. The angler keeps its brick body and receives the two
native Pallet roof blocks38/39. The dispatcher uses the complete Pallet house
38/39 above3c/3d, whose door already matches the actual entrance. The cabin
uses Johto roof2c/2a/2d and frontage26/27/2f. Its middle false doorway is removed;
the complete native2e doorway quadrant is placed at its one actual door.

The frozen source is [source-v06/art.contract.json](source-v06/art.contract.json),
lowered to [source-v06/native.contract.json](source-v06/native.contract.json).
Exactly16 exterior block IDs change; every one of the city's5,120 collision
quadrants remains exact. Twenty-five exact native glyphs and ten recipes are
appended to the private city art. Old bank addressing is repacked without
changing any of the169 original recipes' pixels in any native palette bank.
All690 other map modules, rooms, doors, events, species and audio remain exact.

## Fast review loop

Run `author.py --epoch NEW_NAME` with the local Pillow/cbor2 dependencies. It
reads the exact external parent, authors explicit recipes and renders the whole
city plus all four fronts; it never writes a pack. The accepted warm render
loop took0.308 seconds after loading the parent. Freeze a reviewed epoch with
`freeze.py`, then publish through the Rust CLI. Use fresh epoch/output names.

All source-v01 through source-v06 and their renders remain saved. Rejected
boathouse passes included repeated roof strips and oversized blank walls. The
accepted pass preserves the joined inhabited body and changes only its roof.
The first GPU attempt exposed an overbroad waterfall-art guard; the corrected
guard preserves the waterfall glyph's exclusive allocation while allowing the
verified static native imports. Both failures and final outputs are retained.

This is a local frontage improvement. The remaining regional requirements are
listed in [REVIEW.md](REVIEW.md); the full Minnesota objective remains active.

The [current river review](../../connections/current-river-v01/README.md) now
verifies this exact pack's complete Winona–St Cloud Surf round trip, all seven
bridge passages, native Falls, badge restrictions and eight reached 2D frames.
Its whole26-map river render is separate from the97-map state atlas above.
