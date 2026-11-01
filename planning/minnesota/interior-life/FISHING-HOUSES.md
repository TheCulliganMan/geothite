# Three Minnesota fishing homes

Publication lineage correction: the pack receipts below are retained history.
Current source refuses their obsolete coastal geometry; preserve the rooms and
[rebase onto the compact coast](COMPACT-REBASE.md) before further publication.

The current composed interior pack is
`content-packs/minnesota-interiors-fishing-life-v02.crystalpack`:
716 maps, 475 species, 270 moves, 377 audio programs and 215 non-gym interiors.
Its runtime content hash is
`2a23b30e96a5c25be19bce8a0686d08ab7f1fa8b95e4e63c4087f1e64b82f29d`;
file SHA-256 is
`2eb3396092e89df300e93aa524205c7a0e6b4bf852c6c996654514c4058dbad9`.
Native 2D is the current visual direction.

The three fishing-guru houses are composed onto the protected
[shore gate v04](SHORE-GATE.md), retaining its eight-patch Cedar frontage and
all earlier rooms and canonical story. The [fishing source](../fishing-gurus.json)
installs their real route doors and Old/Good/Super Rod handoffs. The subsequent
[furniture contract](fishing-layouts.json) gives each home a separate native
layout and two readable local details without replacing its guru or service.

| Home | Purpose and local story |
| --- | --- |
| Driftless Trail | A complete teaching table, creek notebook and family float connect the Old Rod teacher to grandchildren and the cold-water creek. |
| River Valley | A repair/writing bench and family flood ledger connect the Good Rod teacher to river safety and cousins in Winona. The native wall map retains its separate interaction. |
| Pine Country Road | A timber cabin with a real native bed, spare-gear cupboard and ice-out photographs connects the Super Rod teacher to family fishing trips. |

The rooms remain 4×4 metatiles. Each retains its guru at `(5,3)`, approach
`(5,4)`, passive arrival `(2,6)` and both active exit carpets `(2,7)/(3,7)`.
The complete tables, writing bench and bed use original native blocks. No new
source graphics, audio, reward, fee, story flag or recovery target is introduced
by the furniture pass. Every new read target is solid reachable furniture;
it does not intercept the wall map, PC or radio service.

## Executed publication

From this checkout, using the pinned toolchain and existing Rust publishers:

```sh
export CARGO_TARGET_DIR=target/interior-life
export CARGO_BUILD_JOBS=4
cargo build --locked --profile wasm-dev -p crystal-assets \
  --bin pack_minnesota_fishing_rods --bin pack_minnesota_interior_layouts \
  --config 'profile.wasm-dev.opt-level=1' \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0'
target/interior-life/wasm-dev/pack_minnesota_fishing_rods \
  content-packs/minnesota-shore-gate-v04.crystalpack \
  content-packs/minnesota-interiors-fishing-v01.crystalpack
target/interior-life/wasm-dev/pack_minnesota_interior_layouts \
  content-packs/minnesota-interiors-fishing-v01.crystalpack \
  planning/minnesota/interior-life/fishing-layouts.json \
  content-packs/minnesota-interiors-fishing-life-v02.crystalpack
```

Both writes were reread by the strict Rust verifier. Choose fresh output names
when repeating publication; these existing packs and every render are retained.
Use a new save for the output identity. The publishers were built by the fresh
asset/test build chain; the explicit build command above selects the same
executables and configuration.

## Verification

The route-house preservation regression passes over the actual gate-v04 parent.
It checks six explicit facade/porch cells per route, both main corridor edges
reachable without an HM, all original actors/events and original dialogue
prefixes, all 27 fishermen's directions, native fishing tables/items and exact
source assets/audio. As documented by the fishing publisher, touched fishermen
use legal active Minnesota parties while retaining their levels, DVs, items,
rewards and defeat/callback metadata. Other trainers are preserved.

The furniture preservation regression passes on the actual written final child
(45.86 seconds): deterministic rebuild identity; every other map, including all
gyms, unchanged; old actor/warp/script/typed command tables retained; unrelated
content, flags and recovery exact. Duplicate installation, stale rows, buried
residents, blocked carpets, floor read targets, native service interception and
gym editing are rejected.

Six production-controller cases pass on the final output:

- Three fishing cases (70.83 seconds): real entries, both exit lanes, re-entry,
  free repeat-safe gifts, Continue, full-pocket failure/retry, independent rods,
  and an actual fisherman victory followed by route/guru directions. The pocket
  test explicitly frees one fixture slot before retrying; it is not a Pack-menu
  disposal test.
- The furniture case (27.29 seconds): all three gurus, all six native read
  targets, visible text/input ownership and real room returns.
- Both gate cases (49.00 seconds): both mouths/lanes, counter and paddler
  conversations, Start/Continue and the actual road seams remain operational.

Position, party, badge and pocket setup are explicit fixtures; these checks do
not establish earned campaign progression. An initial run using an older test
executable failed an exit assertion. Its log remains retained. A fresh build of
the current default 2D test executable supplied the passing final receipts.

Six complete 640×576 headless native 2D frames match independent source
composition with zero differing pixels across 2,211,840 pixels and no exclusions.
All six frames, all three whole rooms and the complete affected route chunks
were personally inspected. All 377 audio PCM hashes, frame counts and loop ranges
match. The pack stayed unchanged during capture; the private executable was
removed.

Current immutable render history is under
`planning/minnesota/renders/interior-life/fishing-houses/`: the original three
identical rooms in `source-v01`, intermediate layouts/captures in `source-v02`,
`source-v03` and `native-2d-v01`, and accepted `source-v04`/`native-2d-v02`.
The v1 contract is retained in `revisions/fishing-layouts-v01.json`. Final logs
are `target/interior-life/interiors-fishing-preservation-v01.log`,
`interiors-fishing-life-preservation-v02.log`,
`interiors-fishing-life-controller-v03.log`,
`interiors-fishing-life-source-comparison-v02.log` and
`interiors-fishing-life-audio-v02.log`.

The current inventory is 223 registered rooms including eight gyms. Of its
215 non-gym reservations, 214 are installed; the additional Rainy River warming
hut brings this child to 215 installed interiors. The separately published
Split Rock clinic and two planned station halls preserve a minimum 218-room
scope. Their integration, canonical Cedar rebasing, full Glacier Line behavior
and regional native 2D quality/service audit remain open. This local acceptance
does not complete the all-interiors objective.
