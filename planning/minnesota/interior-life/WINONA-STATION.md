# Winona depot: winter register and late-train shelter

Publication lineage correction: the pack receipts below are retained history.
Current source refuses their obsolete coastal geometry; preserve the rooms and
[rebase onto the compact coast](COMPACT-REBASE.md) before further publication.

Current scope correction: Red Wing's train station is excluded by user direction.
The [current five-stop publication](RAIL-STOP-CORRECTION.md) supersedes earlier
station reservations below; historical pack/test receipts remain unchanged.

The written, strictly reread child is
`content-packs/minnesota-winona-station-v03.crystalpack`: 712 maps, 475 species,
270 moves, 377 audio programs and 211 installed Minnesota non-gym interiors.
Its runtime content hash is
`d09cff71ce8b879b5226347e6f430fd7d3ddc2458348a242b5ef8eb36581bf14`;
file SHA-256 is
`d4d199a347e5e0687726da31c5d01cd40971fb8f6d1422f348481caf4680bc5d`.
Use a fresh save for this pack identity. The all-interiors goal remains active.

The [station source](../transit/MnWinonaStation.json) adds a compact 10×9 public
hall: timber reading nook, paired winter notices, four waiting chairs, planted
east wall and a conductor closing the platform throat. The mason remembers
setting local limestone with his father; the paddler waits out a headwind;
the winter register recalls the depot staying open when river ice carried the
mail. After reading the log and hearing the mason in either order, the porter
shares one ICE HEAL for the road north. An actually full pocket leaves the gift
available for retry. Returning or Save/Continue never duplicates it.

The conductor explains that passenger service is not open. This hall grants no
courier, route, gym or other regional progression flag. Its native NEWMAP
callback sets only `EVENT_MN_STATION_WINONA_VISITED`. Continue and speculative
scene preparation cannot invent a first visit. The full six-stop Glacier Line
remains unfinished.

## Real door and protected town geometry

The town keeps its original four facades, five rooms, four residents, first
eight warps, scripts and roads. Exactly five original lawn blocks change:
`(9,5)18`, `(10,5)19`, `(9,6)16`, `(10,6)1e`, `(11,6)45` (hex blocks).
The new depot door is core `(19,13)`, warp 9, to station passive arrival 3 at
`(11,15)`. Both exit carpets `(10,17)`/`(11,17)` return through town passive 10
at `(19,14)`, clear of the door. The placard at `(22,12)` has approach `(22,13)`.
Train doors `(6,5)`/`(11,5)` are sealed and have no departure warp.

The original town JSON/layout remain the base authoring input. Its explicit
`transit_overlays` metadata links this separately published facade and room;
the Rust publisher validates the combined geometry in memory. The shared world
and room reservations now register the same source/door/arrival contract.
The plan checker validates that link without pretending the base town contains
the overlay's terrain or appending a duplicate station on an old bootstrap.

The existing west R01 link retains offset −3 and both walking lanes. The north
Mississippi mouth retains offset 0 and all four water lanes. The native edge
validator projects all 12 directed transitions before temporarily removing the
links for the shared unbound-map geometry checker; it restores the identical
links before publication. The checker was not weakened and no road/river link
was removed from the written town.

## Executed build and verification

The executed publication command, after building the Rust publisher with the
pinned toolchain and `CARGO_TARGET_DIR=target/interior-life`, was:

```sh
target/interior-life/wasm-dev/pack_winona_station \
  content-packs/minnesota-international-falls-v02.crystalpack \
  planning/minnesota/towns/MnWinona.json \
  planning/minnesota/transit/MnWinonaStation.json \
  content-packs/minnesota-winona-station-v03.crystalpack
```

The publisher refuses to overwrite an existing pack. The final preservation
regression rebuilds from the current metadata/source and requires identical
child identity. It preserves all old rooms, actors, services, chapter scripts,
source art, catalogs, audio, spawn/recovery records and landmark entries; the
only bounded exterior differences are the five declared blocks, two warps and
placard. Native typed/raw map sections are reconstructed and compared. Eight
unsafe-source cases and duplicate installation are rejected.

Four current production-controller cases pass in 4.11s, including real R01
entry, placard, door, four residents, both carpet returns, blocked boarding,
Start/movement recovery, both clue orders, Save/Continue, once-only gift and
full-pocket retry after actual Pack/Toss input. The fourth case builds all four
real station halls with source images and actual modeled furniture without
advancing state or granting Winona's visit. Initial position/starter/party/bag
fixtures are explicit setup; subsequent visits and story actions use input.
Final logs are `winona-station-preservation-v03.log` and
`winona-station-controller-v05.log` under `target/interior-life/`.

All 377 supplied audio PCM hashes, frame counts and loop ranges match
(`winona-station-audio-v02.log`). The final seven 640×576 native 2D comparisons
pass with zero differing pixels. Their corresponding production 3D frames were
personally inspected: town door, public arrival, winter log, mason, porter,
conductor and paddler. Native plants, chairs, fences, turnstile, notices and
closed blue masonry walls are modeled; the existing background train art is
not a completed regional train-model/service review.

Every source/native original, GPU descriptor, conversion, difference and
rejected pass is retained under
`planning/minnesota/renders/interior-life/glacier-stations/`. The accepted local
public-hall sequence is `native-v04`; its executable SHA-256 is
`09692764f1883670213070e78ff03e7b7dac4a7e5a549d0a7dfd7626cb4cc2cd`.
Earlier candidates remain: pack v01 lacked the refreshed exterior raw event
section; pack v02 used unnecessary decorative statues; native-v03 exposed an
inward wall front, corrected before native-v04. These were not accepted final
artifacts. Public-hall visual acceptance does not close the broader regional
3D audit in [COVERAGE.md](COVERAGE.md).

The whole-plan checker still encounters the same pre-existing city
recomposition mismatch (`recomposition changed one of 44 original source rooms`)
before and after registration. Winona's preceding overlay checks pass; strict
Rust pack preservation supplies the actual geometry/runtime evidence. New Rust
leaves pass rustfmt; whole-workspace formatting/lint baseline limitations are
recorded in [the renderer notes](CITY-COAST-FOLLOW-UP.md).

Four unsafe overlay registrations (stale world link, wrong arrival, wrong
parent and duplicate door) are independently rejected before the unrelated
whole-plan failure; see `winona-overlay-plan-probes.log`. The current registry
inventory resolves all 211 non-gym room IDs against the written pack. The
remaining minimum scope includes the older shore gate and two missing halls,
plus their actual stories, service and regional quality acceptance.
