# Five-stop native passenger service

The current integration candidate is
`content-packs/minnesota-glacier-line-service-r14-v01.crystalpack` (730 maps),
SHA-256 `0fcf972dde061fa55300a490803b95e7e68a2b0c669b46302bd89617abb9c672`,
content identity
`547999cc4ecd67b240d1f627b7788d3f8abbff14392e73a3f8bde1553eb11140`.
Its [R14 contract](glacier-service-r14-v01.json) changes only the verified
parent identity; all five station contracts match the original source exactly.
The existing Rust publisher passes strict geometry, preservation and written
readback. An independent comparison confirms exact accepted R13/R14 blocks and
their mirrors in four maps, with every service program and all other game data
unchanged. The receipt is
`renders/interior-life/glacier-service-r14-v01/r14-parent-preservation-v01.json`.
Controller acceptance and fresh 2D LCD review remain pending. It is not selected.

Executed publication:

```sh
target/interior-life/debug/pack_glacier_line_service \
  content-packs/minnesota-current-grass-r14-v01.crystalpack \
  planning/minnesota/transit/glacier-service-r14-v01.json \
  content-packs/minnesota-glacier-line-service-r14-v01.crystalpack
```

The preceding R12-based service candidate is
`content-packs/minnesota-glacier-line-service-v02.crystalpack` (730 maps), built
over the selected 230-interior composition. Its SHA-256 is
`77ef78c7bc26e709ef1254db0ca81c10b564cc5c678c5b8ec7c4647cd31d9b3c`;
content identity is
`8210918d13c0f496877d89a0ff4c8bcc0cfdd149d8d82898e43edaab1d6d23cc`.
The shared selector has advanced through the interior union to
`route_grass_r14_revision`. This R12-based service candidate is not selected;
integration must retain the newer accepted terrain after controller and native
LCD acceptance. Review remains native 2D; Red Wing is excluded.

Opt-in regressions read `CRYSTAL_GLACIER_SOURCE` when set, so both historical
and current exact-parent contracts can be replayed without rewriting an older
source. Set it to the absolute R14 contract for final integration checks, along
with `CRYSTAL_GLACIER_BASE` for the R14 parent, `CRYSTAL_GLACIER_PACK` for the
service child and `CRYSTAL_INTERIOR_PACK` for the controller's service child.

The five stops are Detroit Lakes, Staples, St Cloud, St Paul and Winona, in that
order. Existing halls, conductors, public doors and their individual stories
remain intact. Travel is free after the existing Q03 courier victory,
`EVENT_MN_COURIER_DEFEATED`. Grand Rapids' Q06 depot event does not unlock trains.
Each destination must already have been visited through its public hall.

## Native implementation

[glacier-service-v01.json](glacier-service-v01.json) pins the exact completed
parent, native station block hashes, conductors, public door indices, protected
arrivals and explicit boarding/reset steps. The Rust publisher rejects a changed
stop roster, wrong authority/visit flags, unsafe arrivals, unbound public exits,
walkable platform bypasses, missing movements and conductor resets that fail to
return home facing south. It retains the existing Winona/Staples visit callbacks
and allocates only the three missing station visit events. Existing St Paul and
St Cloud story/object callbacks stay in their original order.

Each hall has 16 bounded menu-data variants for the four destination-visit bits.
The zero-destination case explains the foot-visit rule without opening a menu;
the other 15 masks use normal native five-row menus. The four other stops remain
in line order, with ` X` marking unknown stops and CANCEL last. B and CANCEL
close the ticket surface. The selected branch checks service and destination
again before moving anyone. This uses existing native menu commands and the
production input owner.

All 20 directed branches restore the source conductor before `MagnetTrain`, set
its direction from the ordered stop indices, and issue explicit `warpfacing`
immediately followed by `newloadmap MAPSETUP_TRAIN`. Arrival is ordinary Floor
south of the destination's blocked throat, outside all existing events. Only
PLAYER movement, the original native delay and end remain after map load.
There is no replacement gameplay dispatcher or new train art/audio catalog.

## Executed publication and evidence

```sh
CARGO_TARGET_DIR=target/interior-life CARGO_BUILD_JOBS=2 \
  cargo build --locked --profile test -p crystal-assets \
  --bin pack_glacier_line_service
target/interior-life/debug/pack_glacier_line_service \
  content-packs/minnesota-interiors-selected-r12-v01.crystalpack \
  planning/minnesota/transit/glacier-service-v01.json \
  content-packs/minnesota-glacier-line-service-v02.crystalpack
```

The completed output is immutable; choose a fresh name for a replay. The first
build process ended with signal143 before a Rust diagnostic; the second build
passed. The publisher passes native geometry, strict preservation and written
readback. A separate read-only comparison confirms only the five station modules,
their raw program payloads and three new visit constants change. All 725 other
complete maps, NPC records, terrain/events, numeric identities, catalogs and
native art/audio remain exact. Original conductor speech is retained under
separate labels. No Red Wing map or visible reference exists.

All 377 audio programs pass PCM/hash/frame/loop verification on V02. The V01
native-table regression passes all 20 destination bindings, every marked menu row, full
typed/raw module reconstruction, real pack serialization and 12 unsafe-input
rejections. Its final replay reproduced V01 from the then-current Rust source
(27.36s); V02's default-profile replay is pending. Logs are under
`target/interior-life/glacier-service-*`; independent preservation evidence is
under `planning/minnesota/renders/interior-life/glacier-service-v01/`.

The new controller cases exercise five real public foot entries and their own
visit flags; service/zero-destination refusals; all 20 locked rows; both cancel
paths; all 16 visit masks at each hall; all 20 animated rides, a repeat return,
conductor restoration, resources and Continue; and an actual courier battle
followed by an unlocked train without granting the separate manifest or Q06.
The first rebuilt replay passes real foot entries/Continue, all denials and
cancel paths, and all 80 visit masks. The courier battle is won through native
input. Both journey tests then fail a fixture assumption: `press` finishes the
automatic train clock before returning, so an intermediate animation cannot be
sampled afterward. The corrected replay requires the actual terminal arrival
cue, native conductor home memory before the graphics special, the retained
120-frame delay, destinations, resources and Continue. Its execution is pending.
The original failures remain retained. Local service/party/badge/earlier-visit
fixtures are explicit setup, not an earned statewide campaign.

Optional LCD evidence uses production Bevy ticket sprites from the actual
controller state, with the established native UI compositor and an isolated
backdrop. A separate, explicitly labeled train presentation fixture advances
the native animation clock for art review; it does not prove a live trip.
Original full-resolution frames and native160×144 derivatives receive immutable
unique paths. This is CPU LCD composition, not terrain or GPU capture. The first
ticket review finds readable station names/markers but an incomplete remaining
prompt, `headed?`. Current Rust shortens the final welcome sentence to
`Choose a stop.` in the completed V02 pack; another visual review is pending.
See the retained `controller-v02-and-ui-review.json` receipt.

V02 changes only the five welcome bodies and their raw/typed text mirrors.
Every other pack field, except the derived identity, remains exact. Its receipt
is `renders/interior-life/glacier-service-v02/prompt-only-preservation-v01.json`.
V01 remains retained, SHA-256
`ac3e138d7ceb82ab86b283375f9ceb277a6cd5263197e52a8e5b197a0cae9830`,
identity `e4f242a84c82b164a967dc2c6d74569d3721eaf428753f833eb8670609f87177`.

The default affected all-target Clippy command passes (8m19s), with existing
warnings elsewhere and no service-leaf warnings. Scoped Rust formatting passes;
workspace formatting retains its existing offset-allocator baseline. A final
Clippy replay after the fixture's native-unloading correction and full V02
controller/visual acceptance remain pending. The full interior objective,
Falls regional circuit and wider room/story acceptance remain active.
