# Historical interior, fair and ice checkpoint

This v01 checkpoint is superseded by [harbor-care v02](HARBOR-CARE.md). Its
20 passing targeted cases did not test the overwritten receiver reveal. The
corrected publication restores canonical chapter dialogue before rebuilding all
stages, protects reviewed text labels and adds three physical room repairs.
The commands and results below remain the executed historical record.


The completed local output is
`content-packs/minnesota-interiors-fair-ice-v01.crystalpack`: **699 maps, 475
species, 270 moves and 377 audio programs**. It carries 201 Minnesota non-gym
interiors in one lineage: the 187-room narrative parent, nine Ely interiors,
three fair rooms and two ice floors. The current canonical species inputs supply
224 regional species in addition to the original 251. This composition does not
install the separate Minnesota cry-revision branch.

The [coverage record](COVERAGE.md) retains the ten missing room reservations and
remaining physical/visual work. This checkpoint does not complete the
all-interiors objective or prove earned campaign progression.

## Publication

The preceding [club publication](CLUBS.md) already includes the v05 furniture
corrections, Ely and the repaired St. Paul exterior. The fair's existing food and
carnival species require the documented regional species/move/animation stages.
All stages below were executed with the Rust publishers; each writes and
rereads an ignored pack through the strict verifier. No source exporter or
shared external content workspace was changed.

The authoring cache was built with these executed integration-test build
commands, which also compile the package's publisher binaries:

```sh
export CARGO_TARGET_DIR=target/interior-life
export CARGO_BUILD_JOBS=4
cargo test --locked --profile wasm-dev -p crystal-assets --features test-fixtures \
  --test state_fair_modpack --test voyageurs_ice_composition --no-run \
  --config 'profile.wasm-dev.opt-level=1'
```

Publication used fresh output names:

```sh
target/interior-life/wasm-dev/pack_fairy_type types \
  content-packs/minnesota-interior-clubs-v02.crystalpack \
  content-packs/minnesota-interiors-types-v02.crystalpack
target/interior-life/wasm-dev/pack_minnesota_pokemon \
  content-packs/minnesota-interiors-types-v02.crystalpack \
  planning/minnesota/pokemon target/minnesota-pokemon/native \
  planning/minnesota/pokemon/species-profiles.json \
  content-packs/minnesota-interiors-species-v02.crystalpack
target/interior-life/wasm-dev/pack_northwoods_moves \
  content-packs/minnesota-interiors-species-v02.crystalpack \
  content-packs/minnesota-interiors-moves-v02.crystalpack
target/interior-life/wasm-dev/pack_fairy_type moves \
  content-packs/minnesota-interiors-moves-v02.crystalpack \
  content-packs/minnesota-interiors-fairy-v02.crystalpack \
  planning/minnesota/pokemon/fairy-moves.json
target/interior-life/wasm-dev/pack_northwoods_animations \
  content-packs/minnesota-interiors-fairy-v02.crystalpack \
  content-packs/minnesota-interiors-animated-v02.crystalpack
target/interior-life/wasm-dev/pack_state_fair \
  content-packs/minnesota-interiors-animated-v02.crystalpack \
  planning/minnesota/zones/state-fairgrounds.layout.json \
  planning/minnesota/zones/state-fairgrounds.json \
  content-packs/minnesota-interiors-fair-v01.crystalpack \
  planning/minnesota/interior-life/fair-city-connection.json
target/interior-life/wasm-dev/pack_voyageurs_ice \
  content-packs/minnesota-interiors-fair-v01.crystalpack \
  planning/minnesota/ice-caves/voyageurs-ice.json \
  content-packs/minnesota-interiors-fair-ice-v01.crystalpack
```

The compact connection changes exactly five St. Paul metatiles at `(5,0..4)`
into the existing native brick street, joining the street at `(5,5)`. It preserves
all facades, indexed doors, people and river connections. City north offset -3
and fair south offset +3 pair city core `(10,0)` with fair core `(16,71)`.
The older 64×48 city's frozen eight-block road is not applied to this 40×34 city.

Packs, executables, saves, raw source studies and captures remain ignored.
Regeneration needs new immutable output names and a save matching that pack's
identity. The final content hash is
`e78af6197005f39e3f0145684704e180175e450bf35f08ab488180c9db41f806`.

## Production verification

All **20 controller cases pass on the exact final 699-map pack**:

| Cases | Observed behavior |
| --- | --- |
| Six interior cases, 78.74 s | All club details/residents/returns, hostel care/stairs, all 117 added residents, Dorothy's HP/PP/status care, full-pocket gift retry, weather/labor errands and Save/Continue. |
| Two Duluth cases, 115.34 s + 15.85 s | Every original/expanded facade and stair, distinct priced shops with actual money/items, optional battles spending PP, saved defeated flags and repeat denial. |
| Eight fair cases, 140.28 s | All three rooms, actual city seam both ways, counters/stamps/prize, all sixteen food effects, full-bag retry, 22 real trainer battles, overlapping sight, four patrols, beer/confusion and all 29 encounter species. |
| One ice case, 9.28 s | Guide page/input ownership, native sliding stops, all rewards, both ladder directions, exterior return and Continue. |
| Three St. Paul cases, 17.36 s | All 39 ordinary facades/returns, both walking bridges, both clinics' free care/PCs, Surf under bridges, both river seams and saved under-bridge state. |

The optimized harness is built with:

```sh
export RUST_MIN_STACK=67108864
cargo test --locked --profile wasm-dev -p crystal-bevy --lib --no-run \
  --config 'profile.wasm-dev.opt-level=1'
```

Use the final pack with `CRYSTAL_INTERIOR_PACK` and
`CRYSTAL_INTERIOR_LAYOUT_SOURCE=planning/minnesota/interior-life/club-layouts.json`
for `interior_l`; `CRYSTAL_GYM_TEST_PACK` plus Duluth's expansion contract for
the two `city_expansion_controller_...` cases; `CRYSTAL_FAIR_TEST_PACK` for
`state_fair`; `CRYSTAL_ICE_DUNGEON_PACK` for `voyageurs_ice_controller`; and
`CRYSTAL_INTERIOR_PACK` for `st_paul_refinement`. Use absolute environment paths,
`--ignored --test-threads=1`, and confirm each filter runs its expected cases.
Explicit position, party, money, badge and prior-access fixtures are setup;
subsequent traversal and state changes use `VisibleShellController::press`.

The fair preservation/deterministic-build case passes (5.58 s), including compact
road preimages and rejected offset/stale-road fixtures. The new ice composition
case passes (2.18 s): only two declared entrance blocks and two appended warps
change on the old R20 map. All original maps/programs, indexes, NPCs, source
files, audio, HOME and recovery are retained. Stale entrance terrain, duplicate
publication and a buried guide are rejected. The preceding v05, club, Ely and
city asset cases are recorded in their respective runbooks.

```sh
export CRYSTAL_FAIR_BASE="$PWD/content-packs/minnesota-interiors-animated-v02.crystalpack"
export CRYSTAL_FAIR_TEST_PACK="$PWD/content-packs/minnesota-interiors-fair-v01.crystalpack"
export CRYSTAL_FAIR_CITY_CONNECTION="$PWD/planning/minnesota/interior-life/fair-city-connection.json"
export CRYSTAL_ICE_DUNGEON_BASE="$PWD/content-packs/minnesota-interiors-fair-v01.crystalpack"
export CRYSTAL_ICE_DUNGEON_PACK="$PWD/content-packs/minnesota-interiors-fair-ice-v01.crystalpack"
cargo test --locked --profile wasm-dev -p crystal-assets --features test-fixtures \
  --test state_fair_modpack --test voyageurs_ice_composition \
  --config 'profile.wasm-dev.opt-level=1' -- --ignored --test-threads=1
target/agent-northwoods-overworld-art-rebuild/debug/verify_crystalpack \
  content-packs/minnesota-interiors-fair-ice-v01.crystalpack --audio --desktop
```

All 377 PCM hashes, frame counts and loop ranges pass on the final pack. Logs
are retained under ignored `target/interior-life/integration-*`.

## Native review

All five added room cameras and three city/fair seam cameras match full frames
exactly, without excluding moving regions. They were visually inspected together
with all five complete source rooms. The preceding 22 club/furniture cameras
remain applicable because the composition preserves those room modules exactly.

Views are [branch-views.json](branch-views.json) and
[fair-seam-views.json](fair-seam-views.json). The retained review includes
[whole rooms](../renders/interior-life/branches-v02-native-review/whole-source-rooms.png),
[first native page](../renders/interior-life/branches-v02-native-review/native-page-1.png),
[second native page](../renders/interior-life/branches-v02-native-review/native-page-2.png)
and [the current complete St. Paul](../renders/st-paul/repair-composed-v02/st-paul-current-whole.png).

Initial source-comparison attempts exposed two review-helper limitations: the
barn's Tauros resolves through its existing native Equine menu icon, and a
clockwise patrol's authored initial direction is right. The helpers now follow
those production rules; no game asset or GPU image was changed. All initial
captures, failed comparison records and later successful originals remain in
`branches-v01-native`, `branches-v02-native`, `fair-seam-v01-native` and
`fair-seam-v02-native`. Full native frames and all conversions are archived.

Changed Rust leaves pass formatting. Workspace formatting and ordinary Clippy
still have the recorded unrelated baselines; the all-target assets/Bevy Clippy
check with only `bad_bit_mask` and `never_loop` allowed completes successfully.
A separate newly added opening module initially failed on redundant unstable
`str::as_str`; removing that redundant conversion restored compilation without
changing its collision check.
