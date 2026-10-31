# Current North Shore mainland verification

## Direct whole-map review on the current Iron crosscut successor

On 2026-10-31, read the entire installed route, all seven crossings and both
actual town connections by sight on `iron_crosscut_bends_revision`:
`content-packs/minnesota-iron-crosscut-bends-v01.crystalpack`, SHA256
`a69310b47d2e3583d9fb8378784c919584591d6e260d5f548ae10ea0cd3409ce`.
A fresh production MCP round trip again passed 1,853 calls. Six native save
checkpoints show Normal mode and zero Surf moves. The initial fixture supplies
eight badges and the existing lighthouse signal; this verifies local walking
access rather than earned campaign progression. No terrain edits were made.

The fresh connected render uses actual map connection offsets, includes Duluth
and Grand Marais, and marks positions recorded from real production input.
Source-only originals, seven crossing crops and both full-resolution town
junction views are retained separately. The selected pack was hash-checked
after the replay.

- [Whole connected route, actual played path](../../renders/north-shore-read-map-v954/actual-played-course-original.png)
- [Whole connected native source without overlay](../../renders/north-shore-read-map-v954/duluth-coast-grand-marais-original.png)
- [All seven crossings](../../renders/north-shore-read-map-v954/all-crossings.png)
- [Grand Marais west entrance](../../renders/north-shore-read-map-v954/grand-marais-west-entrance-2x.png)
- [Current acceptance and limitations](../../renders/north-shore-read-map-v954/acceptance-current-v956.json)
- [Complete production input record](../../renders/north-shore-read-map-v954/no-surf-replay-v553.json)

Freshly replayed on 2026-10-31 against the selected
`leech_lake_mainland_pines_revision`,
`content-packs/minnesota-leech-mainland-v01.crystalpack`, SHA256
`bb416c46842b7a32bb213ed2a8e9c5eebdf384150c7dfab4ef8a341458833a87`.

Duluth → North Shore → Grand Marais’s west entrance → Duluth passed in
1,853 production stdio-MCP calls with empty stderr. All six native save
checkpoints confirm Normal walking mode and zero Surf moves. The initial
fixture supplies the existing eight-badge/lighthouse-signal prerequisites,
a local Duluth position, two Minnesota partners and supplies. Subsequent
movement, encounters and menus use production input. This proves local
traversal; it does not prove earned campaign progression.

The seven installed railed footbridges connect the entire coastal land route.
The fresh static graph has 292 ordinary walking steps. The full route and all
seven crossing crops were visually inspected. No further terrain change was
needed, and the selected revision and pack hash were checked again after play.

- [Fresh full dry course](../../renders/north-shore-current-confirmation-1793457568770371000/whole-route/dry-course-overview.png)
- [Fresh seven crossings](../../renders/north-shore-current-confirmation-1793457568770371000/whole-route/all-crossings.png)
- [Fresh acceptance](../../renders/north-shore-current-confirmation-1793457568770371000/acceptance.json)
- [Production input and native save checks](../../renders/north-shore-current-confirmation-1793457568770371000/no-surf-replay-v553.json)

Earlier native/source pixel comparisons and all earlier renders remain retained.

## Previous selected-pack verification

Freshly replayed on 2026-10-31 against the selected
`international_falls_windbreak_revision`,
`content-packs/minnesota-international-falls-windbreak-v01.crystalpack`, SHA256
`7066f05448101c79b4dc761963b0f39a570bad07c4f80e3e777901ac2e207507`.

Duluth → North Shore → Grand Marais's west entrance → Duluth passed through
the production stdio-MCP controller in 1,878 calls with empty stderr. All six
native save checkpoints confirm Normal walking mode and zero Surf moves.
The initial fixture supplies the existing badge/signal prerequisites, local
Duluth position, two Minnesota partners and supplies; this is traversal proof,
not earned campaign progression. All later actions use production input.

All eight North Shore maps, Duluth, Grand Marais and the coastal tileset remain
exactly equal to the installed footbridge revision. No further terrain change
was needed. The fresh whole-route render and all seven crossing crops were
visually inspected; the static mainland course has 292 ordinary walking steps.

- [Current full dry course](../../renders/north-shore-current-dry-trip-1793431094086967000/whole-route/dry-course-overview.png)
- [Current seven crossings](../../renders/north-shore-current-dry-trip-1793431094086967000/whole-route/all-crossings.png)
- [Current acceptance and inheritance](../../renders/north-shore-current-dry-trip-1793431094086967000/acceptance.json)
- [Current production replay and native save checks](../../renders/north-shore-current-dry-trip-1793431094086967000/no-surf-replay-v553.json)

## Earlier native capture verification

Verified again on 2026-10-31 against the selected `r10_survey_spurs_revision` pack,
`content-packs/minnesota-r10-survey-spurs-v02.crystalpack`, SHA256
`8ae24f320b31423a8824d943c98bdd5548aac2fecc9394e7679bcd48a0db5b33`.

The existing seven native railed footbridges are installed. All eight North
Shore chunks, Duluth, Grand Marais, and the coastal tileset match the previously
verified footbridge revision exactly. No additional terrain change was needed
for this selected pack.

A fresh production stdio-MCP replay walked from Duluth through its actual shore
checkpoint, across North Shore to Grand Marais's west entrance, then returned
to Duluth. It passed 1,953 RPC calls with empty stderr. Native save reads at all
six checkpoints show Normal walking mode and zero Surf moves. Initial setup
provided the existing eight-badge/lighthouse-signal prerequisites, two level80
Minnesota partners and supplies; this proves local traversal, not earned
campaign progression. After initial setup, movement and encounters used
production controller input without changing game state directly.

Fresh artifacts, retained with all earlier renders:

- [Whole route and dry walking course](../../renders/north-shore-to-grand-marais-1793423775321179000/whole-route/dry-course-overview.png)
- [All seven crossing views](../../renders/north-shore-to-grand-marais-1793423775321179000/whole-route/all-crossings.png)
- [Production input and native save verification](../../renders/north-shore-to-grand-marais-1793423775321179000/no-surf-replay-v553.json)
- [Exact inheritance verification](../../renders/north-shore-to-grand-marais-1793423775321179000/current-route-inheritance.json)
- [Nine actual native player views](../../renders/north-shore-to-grand-marais-1793423775321179000/connected-native/production-decisions.png)
- [Connected-map full-frame pixel checks](../../renders/north-shore-to-grand-marais-1793423775321179000/connected-native/capture-progress.json)

The whole route, all seven source crossing views and all nine native player
views were visually inspected. Each native frame matches all 368,640 pixels
of the connected source maps, including the crossing across a chunk seam and
both town approaches. The connected-map compositor includes actual neighboring terrain at chunk
seams and both town approaches. Earlier review folders, including the initial
single-map comparator diagnostics, remain retained.

The selected revision and hash were checked again after playback. Earlier
verification folders and their native captures remain retained. No additional
terrain change or pack selection was needed: this successor retains the
selected footbridge repair exactly.
