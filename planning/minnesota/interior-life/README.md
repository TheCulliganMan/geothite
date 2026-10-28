# Minnesota interior life

The goal remains **all Minnesota interiors except gyms**, including the rooms
registered in other publication branches. This pass authors the 187 non-gym
interiors actually installed in the northern-winter parent. It does not merge
the other branches or claim their absent rooms are finished.

The current publication is [International Falls v02](INTERNATIONAL-FALLS.md),
with 210 installed non-gym interiors and the repaired St. Paul city. Its two-room
winter town and real ranger circuit follow [Moorhead](MOORHEAD.md),
[Detroit Lakes](DETROIT-LAKES.md) and the protected canonical chapter lineage.
Four Falls controller cases, strict preservation, audio and native 2D review
pass. Regional 3D acceptance remains open after the fresh city capture timeout. Earlier v03 story and
fair/ice v01 outputs are historical: the receiver chapter test later exposed one
flavor-text collision, corrected throughout the current lineage.

The subsequent [physical layout pass](LAYOUTS.md) replaces 24 weak room
compositions and adds 25 readable furniture details. Its current publication is
`content-packs/minnesota-interior-layout-v06.crystalpack`, built over the story
pack below. It retains the same 187-room story coverage and original services.
The subsequent [Duluth club pass](CLUBS.md) adds 13 more room contracts and
16 readable details over the St. Paul repair/Ely child, keeping the existing
weather and labor errands. The [coverage record](COVERAGE.md) tracks the
older shore-gate contract, three planned stations and regional 3D review. The earlier
[699-map composition](INTEGRATION.md) contains 201 interiors and passes all
20 targeted controller cases, native room/seam review and all 377 audio checks.
The all-interiors objective remains active.

[stories.json](stories.json) is the authoritative room-by-room contract. Each
entry names the place's purpose, exact revised text labels and any new actor's
sprite, position and adjacent approach. It includes 174 ordinary indoor/gate
rooms, four dungeon-type buildings and nine caves. There are 117 new residents,
111 revised text bodies and six ordinary hosts promoted to remembered
conversations. Existing services, battles, terrain, furniture, stairs, doors,
story conditions and rewards are retained.

The publisher requires complete coverage of the installed non-gym interior set
when `all_installed_interiors` is true. An omitted room fails publication.
It also requires a real resident rather than counting an item ball as an NPC.
It refuses to replace an existing service or quest program with a conversation.
New scripts compile through the existing native parsers; there is no separate
frontend dispatcher.

## Things to find

- Dorothy and another paddler occupy the Root Beer Cabin. Dorothy provides
  repeatable free native party care without touching expedition state or fees.
- A cousin visits the Winona starting bedroom, clear of the spawn and stairs.
- Duluth's upstairs lookout and reading-room log give two different accounts
  of harbor fog. Hear both, then visit Weather Study 2F for one Mint Berry.
- Duluth's union-hall greeter and warehouse loft keeper tell the story of
  restoring night-shift workers' names. The Union Hall 2F recorder gives one
  Ether after both conversations.
- The hillside seed saver shares one Berry. A full pocket leaves
  the gift unclaimed, with visible feedback. Remembered visits and rewards use
  native event flags and survive Save/Continue.
- St. Paul's cut-off descriptions become complete conversations about the
  actual archive walk, river project, quiz, optional matches, lending rooms,
  neighborhood suppers, gardens and fair crafts. Existing activities remain.
- Minneapolis homes and workshops connect the mills and falls to workers,
  families, artists, growers and river habitat. All five department-store
  floors have a shopper as well as their original clerk.
- Smaller-town and route buildings retain their specific original story
  witnesses and services. New neighbors add family voices, trail observations
  and local hospitality. The two formerly item-only northern caves now have
  a lichen observer and a cleft scout.

## Publication and verification

The current executed publisher path is:

```sh
export CARGO_TARGET_DIR=target/interior-life
export CARGO_BUILD_JOBS=4
cargo build --locked --profile wasm-dev -p crystal-assets \
  --bin pack_minnesota_interiors --bin pack_northstar_dialogue --config 'profile.wasm-dev.opt-level=1'
target/interior-life/wasm-dev/pack_northstar_dialogue \
  content-packs/minnesota-northern-winter-v01.crystalpack \
  planning/minnesota/story.json \
  content-packs/minnesota-interior-canonical-base-v01.crystalpack
target/interior-life/wasm-dev/pack_minnesota_interiors \
  content-packs/minnesota-interior-canonical-base-v01.crystalpack \
  planning/minnesota/interior-life/stories.json \
  content-packs/minnesota-interior-life-v04.crystalpack
```

The v04 output has been written and reread by the Rust verifier. The earlier
v01/v02 outputs and all captures remain ignored review history. Use a fresh save
for this pack identity.

The original v03 pass established the following local results; current rebased
verification and the additional receiver regression are recorded in
[HARBOR-CARE.md](HARBOR-CARE.md):

- The external-pack preservation case passes. It checks original map programs,
  actors, services, battles, gyms, terrain, catalog data, title/recovery data,
  assets and audio, a deterministic rebuild identity, and rejected malformed
  placements, missing coverage, duplicate installation and service replacement.
- All 117 new residents pass production conversations, dialogue-owned movement
  rejection and real room returns. The native stair/carpet exit distinctions
  and constant-to-map resolution are part of the fixture path planner.
- Both multi-room errands pass actual walking, witness conversations, denied
  early awards, inventory mutation, Save/Continue and repeat-award denial.
- Dorothy restores actual party HP, PP and status for no money, accepts a second
  visit, and returns the player to the island. The full-pocket gift case retains
  its unclaimed flag and inventory after Save/Continue. Native inventory permits
  several stacks of an item, so the fixture fills all physical pocket slots.
- The three original Winona starter journeys pass their decline/retry, native
  gifts, return and Save/Continue courses with the added bedroom cousin present.
- All 377 audio programs pass PCM hashes, frame counts and loop ranges on v03.
- All 117 new-actor cameras were captured headlessly and visually inspected.
  There are 116 exact source-pixel passes: 114 v02 cameras and both v03 caves.
  Every v02 camera's complete map module equals its final v03 counterpart. The
  remaining bedroom image rendered at 4× rather than the helper's assumed 3×.
  That historical rejection remains in `v02-native/capture-progress.json`.
  The helper now independently derives the native room-fit transform, and a
  fresh complete bedroom frame passes on the 708-map Moorhead child. Its cousin,
  stairs, furniture and entire frame were personally inspected; see the
  [camera follow-up](MOORHEAD.md#bedroom-camera-follow-up).

Logs remain under ignored `target/interior-life/`. These local results do not
finish the physical layout work or the absent-branch room integration below.

The external-pack tests are:

```sh
export RUST_MIN_STACK=67108864
export CRYSTAL_INTERIOR_BASE="$PWD/content-packs/minnesota-interior-canonical-base-v01.crystalpack"
export CRYSTAL_INTERIOR_PACK="$PWD/content-packs/minnesota-interior-life-v04.crystalpack"
cargo test --locked --profile wasm-dev -p crystal-assets --features test-fixtures \
  --test minnesota_interior_life --config 'profile.wasm-dev.opt-level=1' -- --ignored
cargo test --locked --profile wasm-dev -p crystal-bevy --lib interior_life \
  --config 'profile.wasm-dev.opt-level=1' -- --ignored --test-threads=1
```

Whole-room source renders are retained under ignored
`planning/minnesota/renders/interior-life/v03-source/`. The source contact preview
uses the default downward frame for the four original `STILL` objects; those
previews are not GPU acceptance evidence. Actual native cameras are retained
under `v01-native/`, `v02-native/` and `v03-native-caves/`. The 115 v02 camera maps
were compared equal to the final v03 modules before reusing those captures.
All source/contact/crop originals and the mismatching bedroom reference remain
retained. The full native contact pages are in `v02-native-review-final/`.

Changed Rust leaf files pass `rustfmt --check`, and the scoped diff passes
`git diff --check`. The required whole-workspace format check fails on existing
formatting starting in `crates/offset-allocator/src/lib.rs`. Ordinary all-target
Clippy for both affected crates fails on the existing `bad_bit_mask` denial at
`crates/crystal-core/src/multiplayer/mod.rs:1332`. An assets library check with
only the existing `bad_bit_mask` and `never_loop` denials allowed passes; it
reports no warnings in the new interior builder. Existing warnings elsewhere
are not treated as a clean workspace check.

## Remaining registered rooms

An audit of `interiors.json` against the winter parent's actual map catalog found
these 24 absent contracts. Their existing source and completed alternative packs
need their own content/geometry review and integration; the current output must
not be used as evidence that they exist in this branch.

| Group | Registered rooms absent from the winter parent |
| --- | --- |
| Ely and League | MnElyPokecenter1F, MnElyMart, MnLeagueLobby, MnLeagueRoom1, MnLeagueRoom2, MnLeagueRoom3, MnLeagueRoom4, MnLeagueChampion, MnLeagueHallOfFame |
| Western/northern towns | MnDetroitLakesPokecenter1F, MnDetroitLakesMart, MnDetroitLakesStation, MnDetroitLakesCanoeCabin, MnDetroitLakesRangerCabin, MnMoorheadPokecenter1F, MnMoorheadMart, MnInternationalFallsPokecenter1F, MnInternationalFallsMart |
| Shore/transit | MnGrandMaraisShoreGate |
| Fairgrounds | MnFairGrandstand, MnFairBarn, MnFairDairy |
| Voyageurs ice | MnVoyageursIce1F, MnVoyageursIceB1F |

Fifteen of these have been positively located in completed alternative packs:
the nine Ely/League rooms plus Shore Gate in
`minnesota-northstar-ely-league.crystalpack` and
`minnesota-northstar-relay-v2.crystalpack`; the three fair halls in
`minnesota-state-fair-battles-v1.crystalpack`; and both ice floors in
`minnesota-voyageurs-ice-v02.crystalpack`. The nine western/northern town
contracts are absent from all four of those checked parents as well. Do not
treat their reservation records as installed buildings.

The active objective also retains whole-room visual review and gameplay
verification for every newly integrated place. Counts and a successful build
do not finish those requirements.

## Visual review still to enact

All 16 whole-room source contact pages were inspected. The added characters fit
the existing source scale and palette, and the reviewed new placements leave
the door carpets and ordinary aisles legible. The small Minneapolis rooms read
as inhabited homes; Dorothy's traditional-house cabin gives the two paddlers a
clear gathering place. Both new cave residents occupy existing wider floor
pockets rather than the one-tile passages.

Those observations led to the [24-room physical pass](LAYOUTS.md): the hostel
now has a kitchen and six beds, Brainerd's floor is repaired, the work rooms have
distinct compositions, ten Duluth homes/ranger rooms have different functional
corners, and seven St. Paul public rooms have purposeful aisles and displays.
Their v04 controller/native verification is recorded separately in that runbook.
The full build-out remains open for the missing rooms, transit and remaining
room-quality review. The bedroom and St. Paul off-map comparison corrections now
have fresh full-frame native passes, recorded in the Moorhead and city runbooks. In particular, the
[paired Duluth club/lounge floors](CLUBS.md) now have a separate physical pass;
the field clinic, beach shelter and original harbor workshop now have the
[three-room harbor-care repair](HARBOR-CARE.md). Other registered rooms and
the remaining regional acceptance work stay open.
