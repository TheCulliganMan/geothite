# Castorock gallery: fitted banks and a dry circuit

This revises the complete installed14×8R03 cave, including its newer Castorock
extension. The older8×6canonical room is a historical preimage, not a valid
publication source for the current game. `site-before.json` freezes the installed
acquisition site; `site-v02.json` is the reviewed candidate. `site-v01.json` is
the retained first contour draft. The original western cave is unchanged.

The eastern chamber previously had a broad, square upper room and an open lower
floor around a one-block seep. The candidate brings the northern and eastern
walls inward, gives the seep a complete four-block native shoreline, and fits
dry west/east banks around it. The sheltered upper observation and lower river
bed read as different destinations. There is no new field-move requirement or
invented entrance, service, encounter, staircase or cliff hop.

## Native gameplay contract

All coordinates are16px gameplay tiles unless stated otherwise.

| Feature | Retained position and required behavior |
| --- | --- |
| Original mouth / clear floor | (7,9), LADDER / (7,8), FLOOR |
| Real exterior | MnR03C00R00, active(8,37), passive(8,38) |
| Walker / original Great Ball | (5,5) / (12,5), with original scripts and item flag |
| Strength approach / boulder | (12,6) / (13,6) |
| Forward pushes | Boulder(14,6), then(15,6); player can use the retained southern walking buffer |
| Reverse return buffer | Native west pushes through(12,6),(11,6),(10,6) still permit an exit without taking the Great Ball |
| Old waterline observation | Marker(2,5), approach(3,5), face left |
| Split pillar observation | Marker(20,4), approach(19,4), face right |
| Ancient bed observation | Marker(23,13), approach(23,12), face down |
| Original encounter | CASTOROCK24at(21,11), approach(21,10), face down |
| Seep | Metatiles(9..10,3..4), native3e/3d/32/33; enclosed water, dry banks on both sides |
| West-bank bend | (17,8)→(17,9)→(17,10)→(18,10) |
| East bank | x22,y5..10; ordinary dry connection to the lower encounter approach |
| Palette / border | Original forced NITE / solid native cave2e |

Strength must remain denied without its existing badge/move prerequisites. The
closed boulder must seal both eastern observations and the encounter even if
Surf is available. Three actually acknowledged observations reveal the creature;
the native choice, battle, capture, retry and Continue rules remain unchanged.
Every ordinary dry cell is reachable after opening the throat with the original
actor bodies present. Retain both exit/reentry and the once-only original item.

## Fast review and publication

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_castorock_gallery.py \
  --pack content-packs/minnesota-r03-fitted-gallery-v02.crystalpack \
  --before planning/minnesota/caves/current-r03-gallery-v02/site-before.json \
  --site planning/minnesota/caves/current-r03-gallery-v02/site-v02.json \
  --published \
  --output planning/minnesota/renders/caves/r03-gallery-next-review
```

Add `--watch` to keep pack/art loaded and retain a fresh source snapshot and
complete-room render per edit. The measured warm v02source review takes0.226s.
Whole-room originals, converted/enlarged images and both rejected and reviewed
drafts remain under `renders/caves/r03-gallery-v237/`. Initial-visit images keep
Castorock hidden; separately labeled source-anchor images show the revealed
position. No generated image changes native geometry or conceals a defect.

Rust publication is `revise_castorock_gallery BASE BEFORE_SITE AFTER_SITE OUTPUT`.
It binds the exact source preimage and native block payload, preserves all
nonterrain site fields and existing programs, and validates closed borders,
Strength denial geometry, both pushed return states and every dry cell. The
external-fixture regression reads its pack from
`CRYSTAL_CASTOROCK_REVISION_PACK`; inputs are loaded at runtime.

## Native acceptance

The final candidate is `content-packs/minnesota-r03-fitted-gallery-v02.crystalpack`,
SHA256`2e23b03fdcfedc0b93a3a03ac347f2a4440965693a52db2b401a0daeb2664580`.
Its parent is the newer Winona Mart repair, preserving the complete current
St.Paul, Duluth plateau/harbor, Mankato, shoreline and prior cave work.
The earlier St.Paul-parent v01publication remains a retained preimage.

Only20native blocks, their one encoded payload and the derived identity change.
All731other complete maps, every event, script, encounter, trainer, species,
native resource, HOME, recovery record and report remain exact. The western
8×6portion and its original rewards/mouth are untouched. `terrain-ledger-v02.json`
is the final20-cell ledger; the separately named draft-v01ledger is historical.

All four original production-controller cases pass on this exact final pack:
Strength denial/real HM teaching/pushes/three acknowledged observations; refusal,
actual capture and Continue; level32evolution; and RUN/defeat/reverse pushes/
original exit/reentry. A separate121-call production stdio course walks both
dry banks and the lower bed, reads both eastern observations with real input
ownership, then Continues the actually reached save and walks again. Stderr is
byte-empty. Initial parties, badge/items in controller tests and the stdio
placement behind the original gate are explicit QA fixtures, not earned access.

Three fresh offscreen native2D frames have zero differing pixels against the
installed source, with a coherent native cave-water animation phase. The whole
room, its initial-visit source image and all three unmodified native views were
personally inspected. Castorock is correctly hidden in initial-visit comparisons;
the separately retained revealed-anchor images are labeled source fixtures.
All377audio PCM hashes, frame counts and loop ranges pass.

The new guarded Rust publisher and seven rejection cases pass. Assets Clippy
with `--all-targets` passes with existing warnings and no warnings from the new
module or CLI. Changed Rust formatting passes; full-workspace fmt still reports
pre-existing differences beginning in `offset-allocator`. Source publication
review takes0.179s after native pack/art loading.

Evidence is bound in `renders/caves/r03-gallery-v237/accepted-v249.json` and the
adjacent preservation, controller, stdio, native, audio and build records.
`plan.json` and `sources.json` bind exact inputs and inherited authoring sources.
Selection is recorded separately in that render folder. The broader city/route/
cave goal remains active; this does not claim a full campaign, browser deployment,
save migration or final user visual acceptance.
