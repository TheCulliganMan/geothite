# International Falls: winter care and the ranger circuit

`content-packs/minnesota-international-falls-v02.crystalpack` is the written and
strictly reread publication over [Moorhead](MOORHEAD.md): 711 maps, 475 species,
270 moves, 377 audio programs and 210 Minnesota non-gym interiors. Its asset
preservation, source review, thirteen native frames and audio verification pass.
All four controller cases pass (10.50 seconds), including both arrival lanes and
the full ranger loop. Gameplay and native 2D acceptance pass; regional 3D review
remains open. The all-interiors objective is active.

## Two rooms with a public purpose

The [town source](../towns/MnInternationalFalls.json) and
[explicit layout](../towns/MnInternationalFalls.layout.json) author a compact
16×14 winter neighborhood. The earlier uninstalled 24×24 reservation is retained
under `revisions/international-falls-source-v1/`.

| Place | People, purpose and native behavior |
| --- | --- |
| Clinic and ranger desk | Nurse, ranger and a neighbor carrying the family's soup-weather joke. Free HP/PP/status care, actual PC, recovery at `(3,8)` and the SQ03 check-in/report authority. The 5×5 clinic keeps a clear aisle and waiting benches. |
| Winter-supply Mart | Clerk, a shopper buying the spare mittens her brother forgets, and priced Ultra Ball, Hyper Potion, Full Heal, Ice Heal, Escape Rope and Max Repel stock. |
| Plowed streets | Plow driver, librarian, thermometer keeper and night skier, with readable weather, town and trail notices. Conversations connect cocoa, scarves, soup and checking that a neighbor gets home. |

The complete native clinic frontage has door `(9,7)` and passive return `(9,8)`.
The complete Mart frontage has door `(23,9)` and passive return `(23,10)`.
Exterior active/passive pairs are 1/2 and 3/4. Each room has two native exit
carpets and passive arrival index 3. Every resident has a reachable approach;
the nurse and clerk use their actual counter tiles.

The town uses a private `mn_international_falls` art alias. Original JohtoModern
building/paving/sign art stays intact, while exact snow and evergreen blocks
are imported from the installed winter region. The shared Rust art importer
now preserves a donor glyph's private foreground colors when assigning unused
destination glyphs. Original graphics, palettes, collision and field-move rules
remain exact. No exported art or generated pack is a tracked deliverable.

## Roads and the existing icefields

| Town edge | Actual neighbor and reciprocal projection |
| --- | --- |
| South | Town `(20/21,27)` ↔ R17 `(36/37,0)`, town offset -8, route +8. |
| East | Town `(31,14/15)` ↔ Rainy River `(0,28/29)`, town offset -7, icefield +7. |

The old source's direct east-to-R20 diagram would overwrite a working
connection. This publication retains R20's existing western Rainy River link
and authors a small western snow approach instead: exactly three icefield
metatiles `(0..2,14)` change from snowy forest `3f` to snow Floor `3c`, meeting
the existing warming-hut approach at column 3. Sliding ice, cache access,
trainers, hut, ordinary return paths and the R20 exit remain intact.

All other original terrain stays exact. R17 receives only a new northern
connection and its attribute mirror. Boundary Waters receives one guarded NEWMAP callback and its three native
programs, preserving its original coordinates,
actors, surfaces, gates, services and programs. No gym or HOME spawn changes.

## SQ03, through native scripts

The ranger asks whether to register after the seventh badge. “No” leaves the
report unset. “Yes” records `EVENT_MN_BORDER_REPORT`; it grants no badge, harbor
signal or League-survey progression. The eastern trail checks all eight native
badges, that report and `EVENT_MN_LIGHTHOUSE_SIGNAL`. Denial owns dialogue input
and steps the player safely back; westbound travelers retain public care access.

A native `MAPCALLBACK_NEWMAP` records
`EVENT_MN_FALLS_VOYAGEURS_ARRIVAL` only at Boundary Waters `(0,24/25)`,
facing east, with the same complete prerequisites. A westbound approach cannot
substitute for arrival through the real R20 connection. Return to the ranger
for one PP Up. `EVENT_MN_BORDER_CIRCUIT_REWARDED` is set only after the native
item grant succeeds. A full pocket retains the reward for a native Toss/retry;
check-in, arrival and reward flags survive Save/Continue. Existing League
registration still requires its original restored-survey chapter flag.

## Executed publication

```sh
export CARGO_TARGET_DIR=target/interior-life
export CARGO_BUILD_JOBS=4
export RUST_MIN_STACK=67108864
cargo build --locked --profile wasm-dev -p crystal-assets \
  --bin pack_international_falls --config 'profile.wasm-dev.opt-level=1'
target/interior-life/wasm-dev/pack_international_falls \
  content-packs/minnesota-moorhead-v01.crystalpack \
  planning/minnesota/towns/MnInternationalFalls.json \
  planning/minnesota/towns/MnInternationalFalls.layout.json \
  content-packs/minnesota-international-falls-v02.crystalpack
```

The completed output already exists; the publisher rejects overwriting it.
Choose a fresh output name for reproduction. Packs, saves, native captures,
source renders and build products remain ignored.

```sh
export CRYSTAL_FALLS_BASE="$PWD/content-packs/minnesota-moorhead-v01.crystalpack"
export CRYSTAL_INTERIOR_PACK="$PWD/content-packs/minnesota-international-falls-v02.crystalpack"
cargo test --locked --profile wasm-dev -p crystal-assets --features test-fixtures \
  --test international_falls --config 'profile.wasm-dev.opt-level=1' -- --ignored
cargo test --locked --profile wasm-dev -p crystal-bevy --lib international_falls_controller \
  --config 'profile.wasm-dev.opt-level=1' -- --ignored --test-threads=1
```

The asset preservation/rebuild/rejection case passes in 3.26 seconds. It checks
exact original catalog/art/audio/programs, bounded map changes, all old recovery
records, split-payload reconstruction and rejected missing/weakened contracts.
It explicitly distinguishes the three new event flags from the new native
group constant. All 377 audio programs pass PCM hashes, frame counts and loops.

Source geometry passes eleven adversarial cases, including rejected coordinate-event
substitution, a wrong-facing arrival and a missing eighth badge. Seven room and six town native captures pass complete
640×576 source comparisons and have been personally inspected. Raw GPU frames,
descriptors, source references, every comparison stage and the earlier rejected
source studies remain under ignored `renders/international-falls/` and
`renders/interior-life/international-falls-v01-native/`.

The initial controller run passes services and people/road returns. Its two
remaining failures were test-helper errors: selecting No before the native
dialogue pages finished, and planning R20 inside only one chunk. The corrected
choice helper waits for the authoritative visible prompt; the corrected walk
planner follows the four real clipped route chunks, while every step still uses
`VisibleShellController::press`. The corrected map-entry callback passes the complete course in 4.61 seconds.
The final four-case rerun passes, additionally checking an unregistered actual
arrival, the second outlet lane, arrival from the wrong region and Continue
at the edge.


The first candidate's edge coordinate event did not execute on a connected-map
arrival. It remains retained as v01. The v02 correction uses the existing native
NEWMAP callback, with X/Y and facing checks, rather than changing the controller.
All thirteen captured town/room map modules and every runtime/art file are exact
between v01 and v02. Their retained `v02-preservation.json` receipts justify
reusing the untouched original GPU captures without relabeling their executable
or pack identity. Only Boundary Waters programs and its callback section change.


Runtime content hash:
`46d469124d65a25dc67876cf1dce4c90cf9424288e2fb65833be3571498dbc26`.
Whole-file SHA-256:
`9615a405a730e9f2bd6a56af3dc9b9ea618d9336f4525ce046cc42a81b5642cf`.
The final source review passes all eleven counterexamples in 190.7 ms after
loading, and compares every installed map block and all decoded source colors.

## Regional 3D acceptance still open

The fresh headless St. Paul 2.5D check did not produce an accepted frame. The
production location tester exited with `capture timed out: Some("building authored
terrain")`, while repeatedly reporting that a neighboring map exceeded the
16-slot connected-actor presentation namespace. The attempted command, executable
hash and full log remain in `renders/interior-life/voxel-follow-up-v01/`.
That failure is separate from the passing 2D/gameplay receipts. New authored
art aliases also need their native building provenance checked in 3D. Neither
issue is counted as visual acceptance or completion of the regional quality audit.
