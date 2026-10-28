# Harbor care and protected story publication

This care-stage output is
`content-packs/minnesota-harbor-care-v02.crystalpack`: 699 maps, 475 species,
270 moves, 377 audio programs and 201 non-gym Minnesota interiors. The subsequent
verified [Moorhead child](MOORHEAD.md) contains 708 maps and 208
interiors. International Falls, the older shore-gate contract, three planned stations, full rail
service and the regional quality audit remain in [COVERAGE.md](COVERAGE.md) and
belong to the active all-interiors objective.

## Three physical repairs

[harbor-care-layouts.json](harbor-care-layouts.json) adds three complete room
contracts and six readable objects to the preceding 37 contracts and 41 details.

| Room | Change |
| --- | --- |
| Duluth Field Clinic | Replace isolated facility-counter fragments with a complete native treatment desk, shift display and waiting bench. The original nurse serves the public through a real counter. |
| Duluth Beach Shelter | A native stove, cot, found-mitten cupboard and shared table give the original free-rest service a physical home. |
| Duluth Harbor Workshop | One complete repair bench and signal files replace generic dining furniture. Both the workshop keeper and the main-story receiver remain at their original positions. |

Actors, original service programs, events, door indices and room dimensions
remain exact. The layout builder supports an optional `counter_tile` for a
resident approach. It requires a reachable public floor, a solid native COUNTER
at the exact midpoint, and the original actor immediately behind it. A floor
claimed as a counter, an occupied counter, or a furniture event intercepting
the nurse is rejected. Ordinary adjacent actor approaches retain their existing
validation. The controller fixture faces the counter and presses the same
production A input used by ordinary players.

The initial source study exposed a broken bench middle and a blank wall gap;
both were corrected before publication. Every rejected source image, raw GPU
capture and conversion remains in the ignored render archive. The final source
rooms are under `renders/interior-life/harbor-care-v02-source/` and exact native
views use [harbor-care-views.json](harbor-care-views.json).

## Restore reviewed chapter text

The receiver chapter regression found that the old v03 room-story contract
replaced `MN_HARBOR_RECEIVERText` with ordinary local flavor. Its executable
flags survived, but its Kettlurk reveal did not. The 111 revised labels were
audited against the reviewed story registry; this was the sole collision.

The flavor now belongs to the ordinary workshop keeper. `stories.json` is v4;
the rejected v3 contract is retained in `revisions/`. The story builder now
rejects reviewed chapter labels using the existing canonical scene registry,
including Ely's ending text. The full lineage was rebuilt from a canonical
dialogue refresh of the winter parent, rather than patching only the final
receiver cache. A final canonical refresh reparses both serialized text views.

The old v03 narrative, v05 layout and fair/ice v01 packs are historical review
outputs. Their previously passing room/service checks did not exercise the
receiver reveal; use the corrected lineage below for continued work.

## Executed publication

All publishers are Rust and all outputs remain ignored. The builder profile is
`--profile wasm-dev --config 'profile.wasm-dev.opt-level=1'`, with
`CARGO_TARGET_DIR=target/interior-life` and `CARGO_BUILD_JOBS=4`. Each stage
writes and rereads its output through the strict pack verifier. Regeneration
must choose fresh output names.

The final content hash is
`22b7f259174ab51a81313a7e0b2e760b1604c05b16333251bf0a4298ad91844f`.

| Stage | Input | Output stem under `content-packs/` |
| --- | --- | --- |
| `pack_northstar_dialogue` | northern-winter-v01 + `story.json` | minnesota-interior-canonical-base-v01 |
| `pack_minnesota_interiors` | canonical base + `stories.json` | minnesota-interior-life-v04 |
| `pack_minnesota_interior_layouts` | life-v04 + `layouts.json` v6 | minnesota-interior-layout-v06 |
| `pack_ely_league` | layout-v06 + Ely town/layout + `story.json` | minnesota-interiors-ely-v03 |
| `recompose_authored_city` | Ely-v03 + St. Paul repair layout/contract | minnesota-st-paul-repair-v03 |
| `pack_minnesota_interior_layouts` | city-v03 + `club-layouts.json` | minnesota-interior-clubs-v03 |
| `pack_fairy_type types` | clubs-v03 | minnesota-interiors-types-v03 |
| `pack_minnesota_pokemon` | types-v03 + current canonical regional inputs | minnesota-interiors-species-v03 |
| `pack_northwoods_moves` | species-v03 | minnesota-interiors-moves-v03 |
| `pack_fairy_type moves` | moves-v03 + `fairy-moves.json` | minnesota-interiors-fairy-v03 |
| `pack_northwoods_animations` | fairy-v03 | minnesota-interiors-animated-v03 |
| `pack_state_fair` | animated-v03 + fair layout/dossier + compact connection | minnesota-interiors-fair-v02 |
| `pack_voyageurs_ice` | fair-v02 + `voyageurs-ice.json` | minnesota-interiors-fair-ice-v02 |
| `pack_minnesota_interior_layouts` | fair-ice-v02 + harbor-care contract | minnesota-harbor-care-base-v02 |
| `pack_northstar_dialogue` | care-base-v02 + `story.json` | minnesota-harbor-care-v02 |

The roster, fair and ice invocations use the argument order and source paths in
[INTEGRATION.md](INTEGRATION.md), substituting the stems above. Ely accepts the
exact v06 parent hash while retaining its native module/tileset preimages and
allocation guards. No source exporter or shared external workspace was changed.

The new stages were executed as follows:

```sh
target/interior-life/wasm-dev/pack_northstar_dialogue \
  content-packs/minnesota-northern-winter-v01.crystalpack \
  planning/minnesota/story.json \
  content-packs/minnesota-interior-canonical-base-v01.crystalpack
target/interior-life/wasm-dev/pack_minnesota_interiors \
  content-packs/minnesota-interior-canonical-base-v01.crystalpack \
  planning/minnesota/interior-life/stories.json \
  content-packs/minnesota-interior-life-v04.crystalpack
target/interior-life/wasm-dev/pack_minnesota_interior_layouts \
  content-packs/minnesota-interior-life-v04.crystalpack \
  planning/minnesota/interior-life/layouts.json \
  content-packs/minnesota-interior-layout-v06.crystalpack
target/interior-life/wasm-dev/pack_minnesota_interior_layouts \
  content-packs/minnesota-interiors-fair-ice-v02.crystalpack \
  planning/minnesota/interior-life/harbor-care-layouts.json \
  content-packs/minnesota-harbor-care-base-v02.crystalpack
target/interior-life/wasm-dev/pack_northstar_dialogue \
  content-packs/minnesota-harbor-care-base-v02.crystalpack \
  planning/minnesota/story.json \
  content-packs/minnesota-harbor-care-v02.crystalpack
```

## Verification boundary

The final publication's St. Paul module is exactly equal to the previously
reviewed fair/ice v01 module, including all houses, both clinics, the river and
the five-block fair approach. Across all 699 maps, care-v01 and corrected
care-v02 have identical attributes, blocks, actors, event positions and
executable progression commands. Differences are the approved text/cache
refresh and publication metadata. Earlier geometry captures therefore remain
applicable; final-pack captures and controller checks are recorded separately.

The six care cameras and fifteen retained Duluth room cameras supplement the
earlier sixteen-room and thirteen-room physical reviews. All 47 installed
Duluth non-gym interiors now have a whole-room source and native arrival review.
The retained shop stock aisles and practice floors remain intentionally open
for services and battles; this local review does not close the region-wide
quality audit. All captures are headless, with position/time/facing fixtures
explicitly recorded. Fixtures do not prove earned campaign progression.

All fifteen asset cases pass on the rebased stages: narrative preservation and
negative fixtures; the original, club and care layout contracts; both Ely
cases; four applicable St. Paul cases; fair and ice composition; and three
canonical-dialogue cases. These check deterministic rebuild identities,
unrelated content/program preservation, safe counter approaches, rejection of
reviewed-text overrides and final canonical idempotence. The two city cases
specific to the separate Minneapolis water-gate recipe are excluded; St. Paul's
retained-river case is included. All 377 PCM hashes, frame counts and loop ranges
pass on the final pack.

The physical care preservation test targets the pre-refresh care publication:

```sh
export CARGO_TARGET_DIR=target/interior-life
export CARGO_BUILD_JOBS=4
export RUST_MIN_STACK=67108864
export CRYSTAL_INTERIOR_LAYOUT_BASE="$PWD/content-packs/minnesota-interiors-fair-ice-v02.crystalpack"
export CRYSTAL_INTERIOR_PACK="$PWD/content-packs/minnesota-harbor-care-base-v02.crystalpack"
export CRYSTAL_INTERIOR_LAYOUT_SOURCE="$PWD/planning/minnesota/interior-life/harbor-care-layouts.json"
cargo test --locked --profile wasm-dev -p crystal-assets --features test-fixtures \
  --test minnesota_interior_layouts --config 'profile.wasm-dev.opt-level=1' \
  -- --ignored --test-threads=1
export CRYSTAL_STORY_TEXT_BASE="$CRYSTAL_INTERIOR_PACK"
export CRYSTAL_STORY_TEXT_PACK="$PWD/content-packs/minnesota-harbor-care-v02.crystalpack"
cargo test --locked --profile wasm-dev -p crystal-assets --features test-fixtures \
  --lib northstar_dialogue --config 'profile.wasm-dev.opt-level=1' \
  -- --include-ignored --test-threads=1
```

The canonical test proves that the final refresh changes only approved text
views. Earlier stage tests use their exact adjacent input/output pairs in the
publication table and their existing runbook environment bindings. Logs are
retained under ignored `target/interior-life/harbor-care-v02-*`.

All **22 distinct production controller cases pass on the final care-v02 pack**:

| Cases | Observed result |
| --- | --- |
| Seven `interior_l` cases | All residents and furniture/returns, hostel care/stairs, 117 new conversations, both errands, gift retry, Dorothy's care, and the new clinic-counter/beach-rest care with Continue. |
| Two Duluth city cases | Every facade/stair, distinct priced shop transactions, optional battles spending PP, saved defeated flags and repeat denial. |
| Eight fair cases | All rooms and city seams, 22 trainer battles, patrols/sight, every food effect, prize retry and all 29 encounter species. |
| One ice case | Native sliding, guide pages/input ownership, discoveries, ladders, exterior return and Continue. |
| Three St. Paul cases | All 39 ordinary doors/returns, both clinics' actual HP/PP/status care and PCs, bridge walks, Surf underpasses, river seams and Continue. |
| One Duluth chapter case | Every receiver dialogue page, explicit Kettlurk reveal, signal flag only after completing the scene, and retained state after Continue. |

The new care case injures and poisons a partner, empties its PP, starts with no
money, then uses the actual native nurse counter or shelter host. It repeats
after Save/Continue and a real room departure/return. Runtime checks use the
final pack with the environment/filter bindings in [INTEGRATION.md](INTEGRATION.md),
substituting the care contract for `CRYSTAL_INTERIOR_LAYOUT_SOURCE` and adding
`CRYSTAL_DULUTH_STORY_TEST_PACK` for the chapter filter. The seven-case
`interior_l` filter already includes the new counter-care case.

All six fresh care and fourteen fresh city camera comparisons pass exact full
frames and have been personally inspected. Their current views are
[care rooms](../renders/interior-life/harbor-care-v02-native/production-decisions.png),
[complete city](../renders/st-paul/repair-current-v03-native/MnStPaul-whole-native-source.png)
and [city cameras](../renders/st-paul/repair-current-v03-native/native-review-page-1.png)
([waterfront](../renders/st-paul/repair-current-v03-native/native-review-page-2.png),
[downstream](../renders/st-paul/repair-current-v03-native/native-review-page-3.png)).
The [Moorhead follow-up](MOORHEAD.md#bedroom-camera-follow-up) independently
resolves the bedroom's 4× camera transform, with a fresh full-frame pixel pass.
The St. Paul follow-up also resolves the old off-map comparison: its reference
omitted second-hop Mississippi scenery. Two fresh complete frames pass on the
708-map child; the original rejected frame is retained. See the city runbook.

Changed Rust leaves pass `rustfmt --check`; scoped prose changes pass
`git diff --check`. Whole-workspace `cargo fmt --all -- --check` still fails on
existing formatting beginning in `offset-allocator`. Normal all-target Clippy
stops at the existing `bad_bit_mask` denial in
`crystal-core/src/multiplayer/mod.rs:1332`. The all-target assets/Bevy check with
only the existing `bad_bit_mask` and `never_loop` denials allowed completes;
there are existing warnings elsewhere and none in the changed interior,
dialogue or Ely modules. These are scoped results, not a clean-workspace claim.
