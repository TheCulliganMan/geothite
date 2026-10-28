# Duluth clubs, lodge and shared rooms

[club-layouts.json](club-layouts.json) is the explicit 13-room continuation of
the physical interior pass. Eight rooms change terrain; five retain their useful
library, union-hall and weather-station layouts. Sixteen native background
interactions connect the furniture to the residents' existing stories.

| Room | Place and activity |
| --- | --- |
| Boat Club 1F | One crew briefing table and an open rowing practice floor; numbered oars and Sulo's partner-first practice board. |
| Boat Club 2F | A complete long sail-cover worktable, cloth cupboards and the apprentice's crooked cover kept by the winter sewing circle. |
| Sailor Club 1F | A shared mess table with the host seated beside new crew; a crew book that records what each person taught someone. |
| Sailor Club 2F | A quiet letter table, stored correspondence and spare Lake Superior postcards. |
| Lookout Lodge 1F | A central gathering table and a trail cupboard, with clear lanes to the upstairs weather witnesses. |
| Lookout Lodge 2F | A sketching table, window wall and chart archive; harbor sketches and separate RIDGE/LAKE records. |
| Community Kitchen | Timber floor, actual native stove/refrigerator art, a shared table, recipe card and late-shift supper note. The original drink shop remains. |
| Reading Hall 1F/2F | Retain both working book layouts; add the family borrowing card and public harbor-log catalogue. |
| Union Hall 1F/2F | Retain the welcome room; add its shift notice, repair the misaligned foot of the long archive table, and make its restored names readable. |
| Weather Study 1F/2F | Keep the PC, radio, two workspaces and reward conversations; add observation/wish columns and the two reports filed side by side. |

Every original actor, service, optional match, story program, item, stair and
warp index remains. The six floor pairs retain their original house stair
geometry. The single-floor kitchen uses the native timber house bank without
moving its two exterior carpets or passive arrival. No new tileset or source
asset is exported. The parent includes the St. Paul facade/river repair and
Ely/League; this publication keeps them.

The current v03 publication rebases the same club contract over the protected
canonical-story parent. [HARBOR-CARE.md](HARBOR-CARE.md) records its complete
lineage and subsequent clinic, shelter and workshop repairs.

## Rust publication

Executed from this checkout:

```sh
export CARGO_TARGET_DIR=target/interior-life
export CARGO_BUILD_JOBS=4
cargo build --locked --profile wasm-dev -p crystal-assets \
  --bin pack_minnesota_interior_layouts --config 'profile.wasm-dev.opt-level=1'
target/interior-life/wasm-dev/pack_minnesota_interior_layouts \
  content-packs/minnesota-st-paul-repair-v03.crystalpack \
  planning/minnesota/interior-life/club-layouts.json \
  content-packs/minnesota-interior-clubs-v03.crystalpack
```

The output has been written and reread by the strict Rust verifier. It retains
693 maps, 251 species, 251 moves and 377 audio programs. Packs, saves, source art
studies, captures and build products remain ignored. Regeneration should choose
fresh output names, and gameplay should use a save made for the output identity.

## Verification commands

The existing layout regressions now accept an explicit source path; the default
remains the original 24-room contract. Negative terrain fixtures block an actual
resident or exit cell, retaining the tested room's dimensions.

```sh
export CARGO_TARGET_DIR=target/interior-life
export CARGO_BUILD_JOBS=4
export RUST_MIN_STACK=67108864
export CRYSTAL_INTERIOR_LAYOUT_BASE="$PWD/content-packs/minnesota-st-paul-repair-v03.crystalpack"
export CRYSTAL_INTERIOR_PACK="$PWD/content-packs/minnesota-interior-clubs-v03.crystalpack"
export CRYSTAL_INTERIOR_LAYOUT_SOURCE="$PWD/planning/minnesota/interior-life/club-layouts.json"
cargo test --locked --profile wasm-dev -p crystal-assets --features test-fixtures \
  --test minnesota_interior_layouts --config 'profile.wasm-dev.opt-level=1' -- --ignored
cargo test --locked --profile wasm-dev -p crystal-bevy --lib interior_l \
  --config 'profile.wasm-dev.opt-level=1' -- --ignored --test-threads=1
export CRYSTAL_GYM_TEST_PACK="$CRYSTAL_INTERIOR_PACK"
export CRYSTAL_CITY_EXPANSION_CONTRACT="$PWD/planning/minnesota/towns/MnDuluth.expansion.contract.json"
cargo test --locked --profile wasm-dev -p crystal-bevy --lib \
  city_expansion_controller_all_facades_stairs_and_distinct_priced_shops \
  --config 'profile.wasm-dev.opt-level=1' -- --ignored
cargo test --locked --profile wasm-dev -p crystal-bevy --lib \
  city_expansion_controller_optional_trainers_spend_pp_and_persist \
  --config 'profile.wasm-dev.opt-level=1' -- --ignored
target/agent-northwoods-overworld-art-rebuild/debug/verify_crystalpack \
  content-packs/minnesota-interior-clubs-v03.crystalpack --audio --desktop
```

The current contract moves the public harbor log onto its bookcase and the
weather notes onto filing cupboards. The native wall-map shortcuts remain
unbound by custom text. The rejected first contract is retained at
[revisions/club-layouts-v01.json](revisions/club-layouts-v01.json). The preceding
24-room pass also moves its lake notebook and slip-house book onto furniture;
its v05 parent is carried through Ely and the St. Paul repair to this v02 pack.

The room-detail fixture visits ordinary speakers and opens/closes the native
shop. Its prior optional-match flag is explicit setup for a post-match
conversation; the separate trainer regression must fight those matches, spend
real PP and retain the result through Continue. Existing multi-room weather and
labor errands, gift retry and Dorothy's care are replayed against the composed
pack. Position, party and prior access fixtures do not prove earned campaign
progression.

## Visual review and remaining scope

Whole-room source proposals and authentic original-room references are retained
under `renders/interior-life/clubs-v01-source`; the previous rooms and the
native house/timber art studies remain in `clubs-v01-before` and `clubs-v01-art`.
Native captures use [club-views.json](club-views.json), headless rendering and
exact full-frame source comparison in `clubs-v01-native`.

A source audit also rendered the other 18 ordinary Duluth interiors under
`duluth-remaining-review-v01`. Shops, the main Center, history display and
traditional elder home have coherent native compositions. The field clinic's
isolated counter fragments, the beach shelter and original harbor workshop
remain candidates for further physical refinement. They retain their existing
NPCs and services; this source audit is not new gameplay proof for those rooms.
The full all-interiors objective, branch integration and final coverage audit
remain active.

The first-candidate terrain/program preservation and deterministic-rebuild case
passed (77.75 seconds). Its controller furniture sweep then caught the floor
interaction repaired in the current contract; that candidate was not accepted
as a passing full gameplay run. All 377 PCM hashes, frame counts and loop ranges pass using the
existing optimized verifier named above. The unoptimized verifier run was
explicitly stopped before this successful optimized audit; it was not counted
as a completed check.

The first candidate's 17 full native frames match exactly: 13 arrivals and four centered worktable
views. Their raw frames and whole source rooms were visually inspected. The
extra views are necessary because the original upstairs stair arrivals crop the
far-right furniture. See [the native worktables](../renders/interior-life/clubs-v01-native-review/worktables.png)
and [the first native room page](../renders/interior-life/clubs-v01-native-review/page-1.png)
([second page](../renders/interior-life/clubs-v01-native-review/page-2.png)).

Normal all-target Clippy is still blocked by the existing core multiplayer
`bad_bit_mask` denial; whole-workspace formatting also reports existing changes
outside this pass. Changed layout-test leaf files pass Rust formatting. The
initial controller build additionally exposed two stale calls in the separate
river-bend test; they now pass the runtime required by the shared Repel helper.

The current review adds five furniture-approach cameras from
[furniture-read-views.json](furniture-read-views.json) to those 17 views. Fresh
frames and comparisons are retained under `clubs-v02-native`; the v1 source
studies and all rejected captures remain available.

On the final v02 pack, the 13-room furniture/resident/return controller case
passes (56.68 seconds). The preceding v05 24-room furniture/resident/return case
also passes (105.67 seconds). The v02 club terrain/program preservation and
deterministic-rebuild case passes (34.10 seconds), including new rejection
fixtures for floor targets and native wall-map targets. Both Ely asset cases
pass on the v05 parent (25.82 seconds), and all four applicable St. Paul asset
cases pass (28.03 seconds).

All 22 final full frames match exactly and were visually inspected: 13 arrivals,
four worktable views and five corrected furniture approaches. Native raw frames,
source comparisons and immutable contact pages remain under `clubs-v02-native`
and `clubs-v02-native-review` ([page 1](../renders/interior-life/clubs-v02-native-review/page-1.png),
[page 2](../renders/interior-life/clubs-v02-native-review/page-2.png),
[page 3](../renders/interior-life/clubs-v02-native-review/page-3.png),
[page 4](../renders/interior-life/clubs-v02-native-review/page-4.png)). The final
377-program PCM/hash/frame/loop audit also passes.

An exact cross-candidate audit confirms all 693 maps retain the same terrain,
attributes, actors, warps and coordinate scenes as v01. Only the five intended
furniture-program maps differ; all source files, compiled audio, species,
moves, items, encounters and recovery spawns remain exact. The final content
hash is `c3777bfcdacc1c487e96bd5e9e9679c6bf3dcd4287ecad698c5c70cf1e07c1ab`.

The interrupted broad Duluth facade/shop sweep did not finish and is not a
passing result. The [subsequent 699-map composition](INTEGRATION.md) now passes all six
interior-story cases plus the complete Duluth shop/facade and optional-match
sweeps. These final-lineage results supersede the interrupted sweep.
The [whole-scope coverage record](COVERAGE.md) keeps missing rooms and branch
integration explicit.
