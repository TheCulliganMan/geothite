# Yellow prairie grass

R02 Prairie Farm and R16 Red River Prairie now use golden field grass in
`content-packs/minnesota-prairie-review.crystalpack`. This is a regional art
variant of the preceding 471-map geography pack. Trees, wet banks, water and
the other routes retain their existing art.

Content identity: `1e6310ded1e3c1a8383e9906e0d3dcb16d2f152c2daad4890f5b0f1d0b22c38c`.
File SHA-256: `6bb178696c2b9126936770302dfacbd52db7a10c4274d8469cb3f64e34f72312`.

The two new tilesets copy the original native grass glyphs and assign the
existing background palette7. Dark stems distinguish encounter grass from
short lawn. Original source files and palettes are retained; this does not
recolor every object sharing a green source tile. R02 explicitly colors its
two dry farm fields; R16 colors its prairie plots. All encounter grass on both
routes uses the yellow variant. The explicit `block_remap` strokes select
existing grass only, leaving paths, fences, water and ledges in place.

Root inspected both whole-route source renders and the actual noon native
captures: [Prairie Farm](../renders/prairie-grass/native-r02-golden-640.png) and
[Red River Prairie](../renders/prairie-grass/native-r16-golden-640.png).
The production captures are 640×576; their encoded originals and raw readback
bytes remain archived, along with the earlier 1280×800 captures and palette
comparison. These images are a review candidate, with user acceptance open.

The Rust publisher validates the previous installed layouts, exact art
recipes and every changed block pair. Each new block must retain all four
original collision quadrants. Actors, scripts, encounters, rooms, parties,
connections and HOME are preserved. Separate regional tileset aliases retain
the native Cut/Whirlpool replacements and flower/water animation registration.

## Verified delta

The read-only [comparison](../renders/prairie-grass/proof/f9b04e67f2bf-to-1e6310ded1e3/comparison-a2226105b1921af36a49c9417caf8c1aab6df450877387e1f008887876b59369.json)
passes on the final file. Exactly419 metatiles change:106 in R02,216 in R16's
western chunk and97 in its eastern chunk. All1812 metatiles /7248 collision
quadrants are equivalent. The whole decoded pack matches an explicitly bounded
expected result: those three block arrays/base64 payloads and paired tileset
attributes, two added tilesets, ten added runtime files, two appended manifests,
exact inherited field-move aliases and derived identity. Every other value and
scalar type remains exact. All four before/after whole source/native terrain
images are pixel-identical. Eleven adversarial mutations reject unrelated
actor, party, audio, original art, collision, alias, palette and block drift.
Exact sources and all render stages are retained beside the report.

Three native art tests pass, checking copied pixels, original art/collision
preservation and failure without mutating the borrowed base. The rebuilt Rust
publisher also rejects a wrong source palette and an unrelated terrain edit
before writing an output (`.local/prairie-grass-negative-inputs/report.json`).
All377 audio programs match PCM hashes, frame counts and loop ranges
(`.local/prairie-grass-audio.log`). Both fast source loops pass published
terrain/event equality; R02's three water-gap mutations reject. The refreshed
[connected atlas](../renders/atlas/minnesota-current-connected.png) matches all
twenty route rasters against this pack. Its two native towns and25 interiors
remain the prior inventory; city sources are still separate integration work.

Four current production-controller tests pass: Prairie Farm geometry/rooms and
its populated three-trainer journey; Red River's both-chunk geometry and real
Camper battle. They verify ledges/bridges, interactions/resources, selected PP
mutation and save/Continue on the final pack. Logs:
`.local/prairie-grass-r02-controller-final.log` and
`.local/prairie-grass-r16-controller.log`. Position, party/badges and RNG are
explicit local fixtures; these cases do not establish earned regional travel.
An initial overly broad `prairie_` filter also selected three unrelated R19
tests without their external-pack environment; those fail at fixture loading.
The narrowed R02 run and separate R16 run both pass completely. No runtime
change was made to resolve that test-command mistake.

Scoped Rust formatting and Python compilation checks pass. Full-workspace
`cargo fmt --all -- --check` still reports unrelated differences beginning in
`crates/offset-allocator/src/lib.rs:408` (`.local/prairie-grass-fmt-all.log`).
Strict asset all-target Clippy stops at the existing `bad_bit_mask` error in
`crates/crystal-core/src/multiplayer/mod.rs:1332` before checking this crate
(`.local/prairie-grass-assets-clippy-strict.log`). The existing baseline lint
allowances are recorded separately from that strict result.
Asset and Bevy all-target Clippy both complete with the prior `never_loop`,
`bad_bit_mask` and `approx_constant` allowances; existing warnings remain
(`.local/prairie-grass-assets-clippy-baseline-allowed.log` and
`.local/prairie-grass-bevy-clippy-baseline-allowed.log`). No warning points to
the new prairie asset modules or the changed fixture-selection expressions.

## Movement grass sprites — 2026-10-26

The production rustle loader now selects native background palette7 for the
exact `mn_prairie_kanto` and `mn_prairie_headwaters` aliases. Previously it used
ordinary NPC palette6 and flashed green/day or blue/night over golden grass.
All other tilesets retain their original NPC palette. Source PNG pixels, alpha,
OAM placement and timing are unchanged. Cache and failed-asset keys include
style and normalized time so crossing regional tilesets cannot reuse the wrong
sprite or poison another style's load.

All three production-loader regressions pass on the repaired498-map city pack
(which retains both regional tilesets). They check exact native colors for all
four times, every pixel/alpha/OAM position, shared ordinary-cache behavior,
regional cache transitions and isolated missing-asset errors. Log:
`.local/minnesota-cities-rustle-887-review-profile.log`.

Root inspected the [actual loader contact sheet](../renders/prairie-grass/native-rustle/actual-loader-contact-sheet.png).
All24 original18×9 RGBA frames are retained beside it. These are production Rust
loader outputs enlarged with nearest-neighbor for comparison; a fresh walking
GPU capture with the changed loader remains pending.

## Reproduce

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo build --profile wasm-dev --locked \
  -p crystal-assets --bin pack_prairie_grass
target/runtime-fps/wasm-dev/pack_prairie_grass \
  content-packs/minnesota-geography-review.crystalpack \
  .local/r02-prairie-before.layout.json \
  planning/minnesota/routes/MN_R02.layout.json \
  planning/minnesota/routes/MN_R02.json \
  .local/r16-prairie-before.layout.json \
  planning/minnesota/routes/MN_R16.layout.json \
  planning/minnesota/routes/MN_R16.json \
  content-packs/minnesota-prairie-review.crystalpack
```

The fast source loops use the resulting pack explicitly:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_prairie_farm.py \
  --pack content-packs/minnesota-prairie-review.crystalpack --verify-published --negative-checks
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_red_river_prairie.py \
  --pack content-packs/minnesota-prairie-review.crystalpack --verify-published
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_prairie_grass_revision.py \
  --self-test

export CRYSTAL_PRAIRIE_FARM_TEST_PACK="$PWD/content-packs/minnesota-prairie-review.crystalpack"
export CRYSTAL_RED_RIVER_PRAIRIE_TEST_PACK="$CRYSTAL_PRAIRIE_FARM_TEST_PACK"
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy --lib prairie_farm -- --ignored --test-threads=1
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy --lib red_river_controller -- --ignored --test-threads=1
```

Packs and generated art stay ignored. The full Minnesota map-quality and
earned regional progression work remains open.
