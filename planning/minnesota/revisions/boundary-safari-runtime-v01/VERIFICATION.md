# Native core verification

## Saved native catch screens and trainer-backed battle review

The unchanged, unselected R15 review pack now runs the complete native catch
presentation: caught text, a newly owned Pokédex entry when applicable, the
original nickname question and naming grid, and party/selected-PC-box delivery.
Each screen, text page, YES/NO choice and naming draft/cursor/case survives
binary Continue. Renaming updates the actual stored catch and retained result
atomically. Final acknowledgement releases the encounter and any pending
budget exhaustion; revisiting a screen never repeats capture or storage.

Fresh regressions use the default test profile, without profile overrides:

- 13 self-contained Safari core cases pass; the external legacy-save case is
  explicitly ignored in this run, rather than counted as verified again.
- 11 preserve component cases pass on the named habitat/cyclist parent,
  including strict active-Minnesota habitat, native rod/time/swarm-table and
  evolution validation. Non-Minnesota WOOPER and retired MORLIT are rejected.
  The separate CANOE case still requires a pack containing that item.
- The production-controller catch-screen case passes for both party and PC
  delivery (9.46 seconds), including the actual caught species' Dex pages,
  saved NO, lowercase `Ab` naming, selected BOX3 metadata and final release.
- The ranger-to-retirement case (5.21 seconds), native Good Rod capture case
  (4.25 seconds), and Chris/Kris source-art compositor case (2.86 seconds) pass.
- The real stdio MCP resumes the reached naming save, edits the native draft,
  completes PC transfer and returns to the preserve (3.88 seconds). All nine
  replies succeed and stderr remains byte-for-byte empty.

The first component invocation accidentally used the enabled review as the
disabled-parent fixture: two explicit closed-admission assertions failed.
The corrected invocation uses its exact unmodified parent and passes all eleven
cases; neither the assertions nor the feature gate were weakened. An earlier
misspelled test filter ran zero cases and provides no coverage.

The native nickname review exposed BALLS/STEPS labels clipped by the YES/NO
window. Those counters are now hidden while that window owns their area.
Fresh headless 640×576 images were inspected: [battle commands](../../renders/boundary-safari-runtime-v07/native-v02/wildlife-menu.png)
and [corrected nickname choice](../../renders/boundary-safari-runtime-v07/native-v02/pc-nickname-no.png).
Both trainer backpics were inspected in the fresh compositor output. All
earlier rejected renders, reached saves and test logs remain retained.
The fresh GPU command image matches every one of the 368,640 pixels in the
source-art compositor for the same reached save.

This supersedes the older PC/nickname/Dex coverage gaps below. Remaining work
includes throw/shake animation and full audio presentation, remaining native
movement/travel/exhaustion journeys, canoe input and a rebase onto the selected
main lineage before promotion. Paid admission remains closed in the main pack.
The renderer uses the playable `wasm-dev` build profile; the regression results
above use the default test profile. Scoped formatting passes; whole-workspace
formatting still reports unrelated existing differences beginning with
`offset-allocator`.
All-target Clippy passes for core/assets/runtime/Bevy/TUI-view/geothite with
`crystal-bevy/location-tester`. The initial pass identified one new collapsible
conditional in the read-only TUI projection; it was fixed and both Clippy and
the real stdio regression rerun. Existing unrelated warnings remain; the final
log reports none on the Safari/name-entry leaves. Exact binaries, source hashes,
retained captures and logs are bound in the [v07 receipt](../../renders/boundary-safari-runtime-v07/postcatch-default-receipt-v01.json).

## Current-parent rod capture and review publication

The current unselected review pack is
`content-packs/minnesota-safari-runtime-review-r15-v02.crystalpack`, SHA256
`2474bcd7d57db298514dec4f96eee4eaea9a7f5d171d3ff36bb3fa8abb5abdce`.
Its exact parent is the730-map habitat/cyclist R15 pack, SHA256
`25b7c1dfd8715a90a0675a07e992f0d10a7432e745e0c19e4ebab5890f4957cf`.
The Rust publication/journey case verifies all unrelated data, runtime art and
compiled audio remain exact and that strict readback equals the candidate.
No main selector changed. Earlier sections below retain their original inputs;
their pending caught-area/rod notes are superseded by these current results.

Current-parent evidence:

- 11 preserve component cases pass (8.69seconds), including all three native
  rods by day/night. Wildlife uses the actual rod table/time/slot and records its
  real bank and separate water contact. No ordinary combat/party mutation or
  travel-step debit occurs; another cast and invalid source group are rejected.
- The actual Good Rod controller journey passes (8.97seconds): ranger entry,
  grass encounters, the north seam, real shore approach, casting/hook/pause and
  text acknowledgement, saved fishing source, Safari Ball capture/storage and
  Continue. The observed fish is SCUTLET Lv20; its catch uses one session ball.
  Input and authoritative RNG remain frozen during the timed casting display.
- The ranger-to-retirement regression passes (26.40seconds), including all four
  commands, saved choice/result Continue and ignored Select while captured.
- The native compositor regression passes (8.53seconds): saved cursor pixels,
  distinct Chris/Kris backpics, retained wild art and battle cleanup after Run.
- 25 native core fishing tests and12 Safari core/save checks pass. The legacy
  binary save retains its checksum and exact bytes. Ordinary fishing still uses
  the same native slot resolver; Safari bypasses ordinary combat/scaling only
  when an actual expedition permit is exploring.

The input owner permits draining committed presentation notifications while
still rejecting other gameplay acts. Native casting keeps the field display
until its text releases input; the battle canvas does not cover the cast.
Strict all-target Clippy passes for core/assets/runtime/Bevy with location-tester;
existing warnings remain, with no warnings on the Safari leaves. Whole-workspace
formatting still fails in unrelated existing files; scoped formatting passes.
The first broad Clippy attempt hit a concurrent courier-test `str.as_str` error;
that was already corrected in shared source before our attempted patch, which
did not apply or alter that file.

Reviewed native images:
[rod encounter](../../renders/boundary-safari-runtime-v06/native-current-parent-v01/rod-wildlife.png),
[rod catch](../../renders/boundary-safari-runtime-v06/native-current-parent-v01/rod-caught-result.png),
and the earlier-parent
[generated-grass catch](../../renders/boundary-safari-runtime-v06/native-v01/grass-caught-result.png).
All rejected test attempts, renders and reached saves remain in
[the v06 evidence](../../renders/boundary-safari-runtime-v06/fishing-current-parent-v01.json).
The first rod itinerary assumed a bank in the court chunk; the corrected journey
walks the actual north seam. A later rejection exposed the notification-drain
guard; another exposed the test's one-A text assumption. The passing case uses
actual native text reveal/acknowledgement, without bypasses or injected wildlife.

Still incomplete: native PC-delivery controller coverage, remaining movement/
travel/exhaustion acceptance, canoe input coverage and complete nickname/Dex/
throw animation/audio presentation. These results do not enable paid admission
or establish earned statewide progression. No377-program PCM rerender was done
for this data-only review overlay; compiled audio bytes are preserved exactly.

During verification, the main selector advanced independently to the HM R15
descendant, SHA256
`5fb635ffdf9938622bba5d5fccae585a1df02a26081567b304cc2649e09a18af`.
Its11 ordinary preserve component cases also pass. Running the full12-case filter
exposes the optional canoe fixture's missing `CANOE` catalog item at
`safari_zone_tests.rs:642`; no item was fabricated or failure converted to a
runtime skip. The unselected review remains pinned to its named habitat/cyclist
parent and must be rebased again before any future promotion to a newer main
lineage. No main selector or canonical regional dossier was changed here.

## Correct native caught area and successful controller capture

The caught-area adapter now resolves the actual native Town Map landmark, with
strict seven-bit validation. It no longer parses a textual map ID and silently
falls back to0. Generation, configuration validation and saved-wildlife validation
share the same lookup. Missing/oversized landmarks and the old unknown-area
fallback are rejected. Ten preserve component cases pass on the selected R15
pack (9.03seconds), with all four grass/Surf habitat origins checked.

A fresh production-controller test also delivers a successful native capture
(9.72seconds): real ranger entry, generated wildlife and Ball input, correct
stored location/level/time/trainer, unchanged original partner, party/Pokedex
delivery, pending-result Continue, blocked duplicate Start, acknowledgement and
a second Continue with the same stored catch. The explicit party/badge/divider
fixture ends before the first input. This supersedes the earlier journey's
failed-throw-only delivery limit; it does not yet prove PC delivery or completed
nickname/Dex/animation/audio presentation.

Exact input and binary hashes, logs and reached saves are retained in the
[caught-area receipt](../../renders/boundary-safari-runtime-v06/caught-location-v01.json).
The native controller used the unchanged unselected review pack. Old prototype
encounter saves that contain location0 remain preserved, but strict current
validation rejects their incorrect location. Main-pack admission remains closed.

## Safari battle display review

The capture surface now uses the full native battle display and the selected
trainer's Chris/Kris back sprite in the standard lower-left slot. The actual
wild Pokemon uses the standard upper-right slot and native enemy HUD. Counters
occupy the player HUD area; Ball/Bait/Rock/Run and result text share the lower
battle window. Exploration releases fullscreen ownership and removes the back
sprite. The actual saved capture engine still resolves these commands.

Native headless640x576 captures show the command screen and an eating result.
The command capture matches all368,640 pixels of the source-art ECS compositor.
The first GPU pass exposed a projection-rounding error: an exact4x fit became
3.9999998 and floored to3x. The shared fullscreen fitter now absorbs only
floating-point error before flooring. All8 fullscreen regressions pass, including
the new exact-fit check and a real fractional fit that must still round down.
The compositor regression passes saved-cursor pixel changes, distinct Chris/Kris
backpics, unchanged wild art and removal of the battle portrait on Run.
The fresh final controller binary also passes the ranger-to-retirement journey
in17.38seconds. Strict affected-crate all-target Clippy passes with
`--features location-tester`; existing workspace/dependency warnings remain,
with no warnings pointing to either Safari presentation/test leaf.

Reviewed images: [native commands](../../renders/boundary-safari-runtime-v05/native-v02/wildlife-menu.png),
[native result](../../renders/boundary-safari-runtime-v05/native-v02/bait-result.png),
and [Kris source-art fixture](../../renders/boundary-safari-runtime-v05/source-v02/kris-rock.png).
Both rejected border passes and every reached save are retained. Evidence and
exact binaries are recorded in [the v05 receipt](../../renders/boundary-safari-runtime-v05/receipt.json).
The candidate pack is unchanged at SHA256
`2f1e6bc160fcd353e8068410049b717346b01e6c94f6b2cb20d66d01c6df4835`.
It is an unselected review candidate, not a descendant of the latest R14 route
selection. No regional selector or pack was changed for this presentation work.

This does not finish caught-area metadata, capture-only fishing, every movement
and travel path, or complete post-catch nickname/Dex/animation/audio presentation.
Paid admission remains closed in the selected pack. Whole-workspace formatting
still fails in existing unrelated files beginning with offset-allocator; scoped
Safari leaves format cleanly. Clippy results and existing warnings are bound to
the receipt rather than described as a warning-free workspace.

## First production-controller expedition journey

The first ranger-to-retirement journey now passes through
`VisibleShellController::press` (16.29seconds). The unselected review candidate is
`content-packs/minnesota-safari-runtime-review-v01.crystalpack`, published in Rust
over the730-map interior union. Publication verified strict readback and exact
preservation of all unrelated data, runtime art and compiled audio bytes.

Actual input opens the facing ranger's offer; declines without paying; resumes
the saved NO choice; confirms and pays500 once; enters with30 balls/500 steps;
walks to generated Minnesota wildlife; verifies capture ownership and saved
cursor/cache invalidation; uses Run, Bait, Rock and Ball; continues each pending
result; and retires through a real return ranger. There is no ordinary combat
battle. The throw observed in this run was a failed MOSQUITAN capture followed
by flee: this case does **not** prove successful catch delivery through the
controller. That still has separate core/component coverage only.

Reached saves, all rejected attempts and exact hashes/limits are retained in
[controller evidence](../../renders/boundary-safari-runtime-v04/controller-pass-v03.json).
The first attempt exposed the concurrent appearance-field binary-layout bug;
the original schema has since been restored in shared source. The exact failed
ranger save also decodes/re-encodes byte-identically with the preceding original
schema. The next rejection was an incoherent test badge setup: Continue hydrated
`ENGINE_FOGBADGE` after setup had set only its badge bits. The fixture now sets
both native representations before gameplay. Preservation assertions remain.

The successful binary is
`target/runtime-fps/wasm-dev/deps/crystal_bevy-2b3810691aa21f55`, with core, assets,
runtime and Bevy opt-level0 for local verification. Buildv06 and flowv03 logs are
preserved with the evidence. The native screenshot renderer is being built;
there is no new native UI visual acceptance yet. Caught-area metadata, rods,
outward travel, remaining movement/storage/exhaustion paths and complete
post-catch presentation still need work. Paid admission remains closed in the
selected pack; this bounded journey does not complete the whole feature.

## Generated wildlife and saved input owners

The fresh component run passes eight actual-pack cases on the R12 pack at
SHA256 `8c9a01edf629270392e43ae8761fd0ea78d57035b0cb4b097bf9eebc3fc38717`.
It includes generated wildlife after committed native walking and Surf in all
four habitats, with valid source species/levels, full HP, no ordinary combat
state and no player-party mutation. The saved ranger choice survives native
resume; it blocks world movement, declining costs nothing, accepting charges
once and a duplicate confirmation fails without a second debit or warp.
These are component fixtures, not production-controller acceptance.

The fresh core binary passes11 Safari cases, including binary Continue of the
capture cursor and pending result, saved ranger selection and unchanged legacy
save bytes/checksum. The first core replay accidentally used the older binary;
its retained v01 log is explicitly superseded by v02. The new core executable
is `target/runtime-fps/wasm-dev/deps/crystal_core-0f9f5c2dadcffa62`.

The broader assets filter reports12 passed/5 failed: four historical geometry
cases consume the expanded64×56 source with old48×40 contracts, and the canoe
admission fixture requires the absent CANOE item in this R12 input. They are
retained as failures, not counted as passing or silently patched out of source.
The eight expedition cases were then selected explicitly with the canoe case
excluded for this input pack. Their result is8 passed/0 failed (17.60seconds).
The core result is11 passed/0 failed (0.04seconds).

The shared Bevy/TUI library code check passed in8m37s. Subsequent display-cache
and text-width edits still need fresh controller/render verification. A full
production-input test now covers the actual ranger, generated wildlife, four
commands, Continue and ranger retirement; it is being built. Its existence is
not evidence that it passes. No new pack has been published or selected.

Logs, exact input/binary hashes, source hashes at receipt and limits are retained
in [v03 evidence](../../renders/boundary-safari-runtime-v03/receipt.json).
Focused Safari-leaf formatting passes. Whole-workspace formatting still reports
unrelated differences beginning in offset-allocator and pack exporters.
Outward-travel settlement, all movement categories, capture nickname/Dex
presentation, native animation/audio and clean stdio/visual checks remain open.
Source review also found that the Safari adapter currently parses the textual
`saved_map_id` as a byte and falls back to0. That loses the real caught area.
Before acceptance, generation and saved-wildlife validation must instead use
`pokegear_landmark_for_map`, validate its native seven-bit ID and verify that
the resulting party/PC catch retains that location. The component tests above
do not establish correct caught-area metadata; their fixtures use the same old
adapter convention. This correction is still pending while the first controller
build finishes.
The current union's six preserve exteriors also retain
`FISHGROUP_MN_QWILFISH`. `cast_fishing_rod_in_session_with_divider` still calls
the ordinary fishing-battle factory after a bite. Safari needs an explicit
capture-only fishing path, including real bank/target coordinates and native
cast presentation; simply excluding ordinary walking encounter tables does
not cover rods. This path must be verified before paid admission opens.

The same eight cases were also run against the newly selected730-map interior
union, `content-packs/minnesota-interiors-selected-r12-v01.crystalpack`, SHA256
`1d60940da0f42dbce7bd5a595ccd8d8b8d68c2e84dcfa4ca505c67d5abd6595b`.
All eight pass (22.40seconds). Its separate
[input-bound receipt](../../renders/boundary-safari-runtime-v03/current-union.json)
and log preserve that distinction from the earlier705-map R12 input.

Executed focused commands, using the fresh binaries built by Cargo:

```sh
CRYSTAL_SAFARI_TEST_PACK="$PWD/content-packs/minnesota-interiors-selected-r12-v01.crystalpack" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_assets-51037f981d106d3a \
  safari_zone::tests:: --ignored --skip canoe_safari_admission \
  --nocapture --test-threads=1

CRYSTAL_SAFARI_LEGACY_SAVE="$PWD/planning/minnesota/renders/mille-lacs-park-v01/reached-v05/ledge-landing.crystalsave" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_core-0f9f5c2dadcffa62 \
  safari_ --include-ignored --nocapture
```

## Pack-bound native integration

The subsequent integration passes all6 external-pack cases on
`content-packs/minnesota-current-grass-r11-v01.crystalpack`, plus60 core
Surf/field-move and Safari cases. The runtime library also checks successfully.
The exact sources, input pack, logs and full native-art preserve render are
bound in the newer
[receipt](../../renders/boundary-safari-runtime-v02/receipt.json).
The refreshed [whole preserve](../../renders/boundary-safari-runtime-v02/whole-preserve.png)
uses its reward descendant's exact native art. A separate read-only comparison
confirms all seven preserve maps, the free approach/hub and preserve tileset
remain exact in the subsequently selected R12 pack. Those correspondence checks
do not relabel the R11 component tests as R12 controller runs.

The six actual-pack tests establish configured geography/negative references;
transactional refusal and failed native warp; real entry/return and duplicate
confirmation; saved wildlife origin validation; the existing free court's
native guard bodies, walking budget, Continue and final-step behavior; and
native action RNG/checksum replay with bad-trace rollback and result input
ownership. Explicit placement, party and encounter setups are component
fixtures. They do not prove live ranger dialogue, generated Safari encounters,
earned regional progression or production-controller acceptance.

Rejected passes remain retained. The first exposed the already implemented
free `MnPreservePineEntry` seam, so the actual walk/court was preserved and
included in the contract. A later gate probe caught that visual invisibility
does not remove native object collision; permit guards now use native
disappear/appear object-struct operations. The resume fixture was then corrected
to supply the real pack's music IDs instead of an empty catalog. Gameplay data
was not changed to accommodate that fixture error.

Executed integration commands use the pinned toolchain and core/assets
optimization overrides for local iteration:

```sh
export CARGO_TARGET_DIR=target/runtime-fps
export CARGO_BUILD_JOBS=3
export RUST_MIN_STACK=67108864
export CRYSTAL_SAFARI_TEST_PACK="$PWD/content-packs/minnesota-current-grass-r11-v01.crystalpack"

cargo --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --profile wasm-dev --locked -p crystal-assets --lib safari_ -- \
  --ignored --nocapture

cargo --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  check --profile wasm-dev --locked -p crystal-runtime --lib

cargo --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  test --profile wasm-dev --locked -p crystal-core --lib -- \
  field_moves safari --include-ignored
```

The last command uses the same `CRYSTAL_SAFARI_LEGACY_SAVE` documented below.
Required all-target Clippy was attempted for core/assets/runtime and stops at
the existing `bad_bit_mask` denial. Allowing that one lint exposes11 existing
assets `never_loop` denials. A supplemental all-target check of assets/runtime
with **only those two lints allowed** passes (2m11s); it reports no diagnostics
against the new Safari leaves or runtime adapter. Focused new-leaf rustfmt
passes. Whole-workspace rustfmt still reports existing differences; unrelated
formatting in `field_moves.rs` was preserved.

No new pack has been published or selected. The current game still has no
enabled Safari config and no live paid admission. Habitat generation, actual
four-command controller presentation, outward-travel settlement, all movement
categories and clean stdio/native visual review remain required before opening.

## Earlier core component baseline

The final core revision passes all10 focused tests, including the ignored
external legacy-save case when explicitly enabled. All40 shared native capture
cases also pass. This proves the component behavior described in
[README.md](README.md), not production admission or a playable Safari menu.

The [receipt](../../renders/boundary-safari-runtime-v01/receipt.json) binds source
hashes, logs, the real legacy input and the unchanged selected pack. Every
attempt's log is retained in that same ignored directory, including rejected
passes. The first pass exposed a zero numeric species identity in the test
fixture; the fixture and pre-persistence compiled-species validation were fixed.
Two new enum-size Clippy warnings were fixed by boxing the stored catch; its
Serde representation and inactive binary encoding are unchanged.

## Executed commands

```sh
export CARGO_TARGET_DIR=target/runtime-fps
export CARGO_BUILD_JOBS=3
export RUST_MIN_STACK=67108864
export CRYSTAL_SAFARI_LEGACY_SAVE="$PWD/planning/minnesota/renders/mille-lacs-park-v01/reached-v05/ledge-landing.crystalsave"

cargo --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  test --profile wasm-dev --locked -p crystal-core --lib safari -- \
  --include-ignored --nocapture

target/runtime-fps/wasm-dev/deps/crystal_core-42bffda1a758433b

cargo --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  clippy --profile wasm-dev --locked -p crystal-core \
  --all-targets --features test-fixtures

cargo --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  clippy --profile wasm-dev --locked -p crystal-core \
  --all-targets --features test-fixtures -- -A clippy::bad_bit_mask

cargo --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  clippy --profile wasm-dev --locked -p crystal-core \
  --lib --features test-fixtures -- -A clippy::bad_bit_mask

cargo fmt --all -- --check
rustfmt --edition 2024 --check \
  crates/crystal-core/src/systems/safari.rs \
  crates/crystal-core/src/systems/safari_encounter.rs \
  crates/crystal-core/src/systems/safari_tests.rs \
  crates/crystal-core/src/battle/capture.rs
```

The explicit core-only optimization override keeps local test rebuilds short;
the pinned toolchain and native implementation remain unchanged. The exact
hashed test executable is evidence for this run; a future build may have a
different filename. Use Cargo to rebuild after code changes.

## Broader limits

The complete core binary reports2260 passed,11 failed and1 ignored. Five
failures read absent `../vendor/pokecrystal` move-effect source files, two need
the external special-routines catalog (the second observes its poisoned lazy
initializer), one invalid-save test expects encoding of a rejected event flag,
and three object-allocation tests disagree with the current shared allocator.
These failures are outside the Safari edits; the full suite is not green.
The complete original names/results are in `safari-core-full-v03.log`.

Required all-target Clippy stops at the existing `multiplayer/mod.rs:1332`
`bad_bit_mask` denial. Allowing only that lint advances to absent radio-text
integration JSON fixtures. The supplemental library check passes with that one
allowance; existing warnings remain. Neither supplemental log has a diagnostic
against the new Safari leaves. Focused rustfmt passes. Whole-workspace rustfmt
reports existing differences starting in `offset-allocator` and exporter leaves;
unrelated formatting was preserved.

At that earlier core baseline, the selected `route_grass_r10_revision` pack was
byte-identical at
SHA256 `f9e3153964a966d610e1ec8c9ecfb45e980e80eab8ea48e336e061863747acd0`.
That earlier run did not publish a pack or claim production-client/UI/MCP
acceptance. The subsequent integration above adds actual pack references,
admission/return transactions and movement hooks. Generated encounters,
four-command controller presentation, remaining travel categories and
free-story regressions are still required as listed in README.md.
