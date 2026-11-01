# Red Wing station removed

## Current compact composition

The selected 730-map union is
`content-packs/minnesota-interiors-selected-r12-v01.crystalpack`.
Its full native map-name and visible-text scans find no Red Wing station or
listing. All 26 targeted controller cases and 377 audio programs pass, including
three fresh St Paul cases covering 39 ordinary door pairs, both clinics/PCs,
bridges, Surf seams and Continue. Three fresh city frames have zero native pixel
differences and were personally inspected. See the
[current selection and exact evidence](selected-union-v01/README.md).

## Earlier compact stages

The correction remains installed in the 729-map local story/interior child,
`content-packs/minnesota-interiors-r03-current-story-v01.crystalpack`.
The complete repaired St Paul exterior remains exact; all three fresh city
controller cases pass, including 39 ordinary door pairs and both Centers.
The retained `current-story-composition-v01/city-and-rail-preservation.json`
receipt scans the story parent for a Red Wing station map or station/timetable
reference and finds neither. Its final R03 child changes only one cave module.
Native 2D captures and the build chain are recorded in
[the current composition](current-story-v01/README.md).

The latest local correction is
`content-packs/minnesota-interiors-current-no-red-wing-v01.crystalpack`, built
from the 712-map / 214-interior terrain-sync v04 candidate using the unchanged
Rust publisher and `rail-stop-correction.json`. A full visible-text scan caught
one stale St Paul timetable inherited from the compact parent. This correction
removes that listing; Detroit's already-correct five-stop timetable stays exact.
No Red Wing station map or visible Minnesota Red Wing reference remains.

Only `MnStPaulUnionDepot`'s `MN_GLACIER_LINE_TIMETABLEText` changes, in its typed
body and raw native script payload. All callers, objects, warps, terrain, other
stories, catalogs and native art/audio remain exact. File SHA-256:
`3dbd5f7908c2d3416a166b87928a8ea6ceae6156e1bb5835cca8ae0b9dc317b0`;
content identity:
`ab00a0a34249535a7ba43e1b7879540c11f89e0005044b5ffb355f51b67e92d4`.
The retained receipt is
`renders/interior-life/current-no-red-wing-v01/preservation.json`.
All three St Paul controller cases pass on this exact corrected child (44.23s),
including entrances/returns, both clinics and PCs, bridges, Surf river seams and
Continue. Its 377 audio programs also pass PCM/hash/frame/loop verification.

Executed from the repository root:

```sh
target/interior-life/wasm-dev/pack_minnesota_interiors \
  content-packs/minnesota-interiors-terrain-sync-v04.crystalpack \
  planning/minnesota/interior-life/rail-stop-correction.json \
  content-packs/minnesota-interiors-current-no-red-wing-v01.crystalpack
```

The shared terrain selector is preserved. Native 2D review remains the only
current visual direction. The three fair rooms are subsequently installed and
verified; International Falls' full regional circuit, passenger service and
wider regional acceptance remain open. See the current composition above and
[the terrain composition record](terrain-sync-v01/README.md).

## Historical branch

Publication lineage correction: the pack receipts below are retained history.
Current source refuses their obsolete coastal geometry; preserve the rooms and
[rebase onto the compact coast](COMPACT-REBASE.md) before further publication.

The user's direction is **no Red Wing train station**, with **native 2D only**
for visual work. Red Wing is removed from the active room registry, actor
registry and Glacier Line stop plan. The depot publisher, production tests and
source have been removed; rejected packs, renders and source snapshots remain
historical evidence under `renders/interior-life/red-wing/`. Do not reintroduce
that station from those artifacts.

The current ignored pack is
`content-packs/minnesota-interiors-no-red-wing-v01.crystalpack`:
719 maps and 217 installed non-gym interiors. The approved stops are Detroit
Lakes, Staples, St. Cloud, St. Paul and Winona. The registry has 224 entries,
including eight gyms; its 216 non-gym rooms plus the warming hut are installed.

The publication starts from the protected **pre-depot Staples v02 parent**.
It corrects only the Detroit Lakes and St. Paul timetable text bodies through
the existing Rust interior publisher. Their labels, callers and interactions
remain intact. The entire lower-Mississippi bank is restored to its pre-depot
map. All other maps, including St. Paul's repaired buildings, clinics, river
banks, bridges and quay, remain exact. No replacement building is introduced.
All game catalogs, source art and audio remain exact.

## Executed publication

From the repository root, using the existing Rust publisher:

```sh
target/interior-life/wasm-dev/pack_minnesota_interiors \
  content-packs/minnesota-interiors-staples-v02.crystalpack \
  planning/minnesota/interior-life/rail-stop-correction.json \
  content-packs/minnesota-interiors-no-red-wing-v01.crystalpack
```

The publisher strictly reread the resulting pack. The binary's publisher source
was unchanged by this correction. Use a fresh output path for regeneration;
retain the existing ignored pack and historical rejected packs.

| Identity | SHA-256 |
| --- | --- |
| Parent file | `f8d09d84f1d16d34c4ba84373d4b35e3e3eb1e7844212a9f56062aee79db1ff0` |
| Current file | `6e2c0d68a25318de84f8e9da5d29c393704dfc25ba14ba07f44d26ba850b285a` |
| Current runtime content | `de8dce386b3903ee01ca2a4ada21d12395a6422af8fbe8517f8845977a09508d` |

## Verification

- Strict runtime/desktop validation and all 377 audio PCM hashes, frame counts
  and loop ranges pass. Log: `target/interior-life/no-red-wing-audio-v01.log`.
- Read-only full-pack comparison proves that only the two timetable bodies and
  their corresponding raw text payloads change from Staples v02, plus required
  report/identity bookkeeping. No Red Wing station or visible timetable text
  remains. All other game data and native source files remain exact. Receipt:
  `renders/interior-life/no-red-wing-v01/preservation.json`.
- Three existing Staples production-controller cases pass (6.43s): actual
  spur/door/return and visited flag, priced purchase/denial/Continue, and native
  regional wild battle/RUN.
- Three existing St. Paul production-controller cases pass (15.05s): all 39
  ordinary doors and both bridges/Continue, both clinics' free care/native PC,
  and Surf through both bridges and river seams/Continue. Initial location,
  party and other setup are explicit fixtures, not earned regional progression.
- Those six cases used the existing unchanged production test executable
  `target/interior-life/wasm-dev/deps/crystal_bevy-8e2ff9039d737c59` with
  `CRYSTAL_INTERIOR_PACK` set to the current pack, `RUST_MIN_STACK=67108864`,
  filters `staples_station_controller_` and `st_paul_refinement_`, and
  `--ignored --nocapture`. Logs: `no-red-wing-staples-controller-v01.log` and
  `no-red-wing-city-controller-v01.log` under `target/interior-life/`.
- A fresh headless native 640×576 2D frame at the original river landing
  `(40,39)`, facing south at noon, was personally inspected. All 368,640 pixels
  match the independent native-source composition, with no exclusions. Capture,
  commands and comparison are retained under
  `renders/interior-life/no-red-wing-v01/`. The private capture binary was removed.
- Current source passes both affected crates' all-target Clippy checks only
  when the two previously identified unrelated denied lints are allowed:
  `-A clippy::bad_bit_mask -A clippy::never_loop` (54.19s). Normal Clippy still
  fails in `crystal-core` on the existing always-false bit comparison.
  Required workspace `cargo fmt --all -- --check` still fails first in the
  unrelated `offset-allocator/src/lib.rs:408`. Logs are
  `no-red-wing-clippy-v01.log`,
  `no-red-wing-clippy-existing-denials-allowed-v01.log` and
  `no-red-wing-fmt-v01.log` under `target/interior-life/`.
- Scoped `git diff --check` passes. No packs, captures, binaries or scratch
  programs are staged or committed.

Full five-stop passenger service (twenty directed bindings), the Staples ledger
wall-position repair, canonical coast/Cedar and Ely geography composition, and
regional room-quality/service acceptance remain open. Counts do not complete
that broader objective. This correction adds no voxel work.
