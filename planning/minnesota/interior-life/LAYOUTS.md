# Physical interior pass

The current authored layout contract is [layouts.json](layouts.json). It changes
24 ordinary interiors over the completed 187-room story pack, keeping every
original actor, script, doorway, stair index and room extent. It adds 25 readable
furniture details. This is another part of the active all-interiors objective;
the other publication branches and remaining physical review are still open.

The subsequent [St. Paul facade/river repair](../towns/st-paul-repair/README.md)
uses `minnesota-st-paul-repair-v03.crystalpack`, a completed 693-map child of
the v06 layout pack through the Ely/League publisher. It retains these 24 room
layouts and adds Ely's existing nine non-gym interiors. All 39 St. Paul ordinary
facades, clinic care/PCs, bridge walks, Surf seams and Continue are verified.
Missing branch integration and the final whole-scope interior audit remain part
of the active objective.

The [Duluth club continuation](CLUBS.md) then publishes
`minnesota-interior-clubs-v03.crystalpack` over the repaired city pack. Its
separate [13-room contract](club-layouts.json) revises eight room layouts,
retains five useful existing layouts and adds 16 readable details. Both source
passes together cover 37 room contracts and 41 furniture interactions.

The v6 manifest republishes the corrected v5 furniture bindings over the
protected canonical-story parent. The [harbor-care continuation](HARBOR-CARE.md)
records the full rebase and three additional Duluth room repairs.

## Rooms and reasons

| Place | Physical change |
| --- | --- |
| Brainerd outfitter | Replace the black walkable tiles with native carpet; finish the pack-check table and surround it with real gear shelves. |
| Duluth Hillside Hostel 1F | Timber kitchen/reception, complete communal dining table and luggage shelf; keep the original free care and both exterior carpets. |
| Duluth Hillside Hostel 2F | Six native beds with two open aisles, a quiet guest room and the original paired stair. |
| Duluth boat builder | A long repair bench faces the tool cupboards, with a separate apprentice work lane. |
| Duluth cargo office | A timber ledger office with a complete document table and filing shelves; retain the original requisition activity. |
| Duluth ore warehouse 1F/2F | Loading stacks and cross-aisles downstairs, a quieter tally loft upstairs; keep both directions of the original stair and labor-history conversations. |
| Duluth Fisher Cottage, Foreman Home, Sand House | Distinct timber households with actual sleeping places, family storage and different table positions. |
| Duluth Pottery House | Clay/stone displays, a complete worktable and the potter's first-bowl key shelf. |
| Duluth Hill Garden and Point Garden | Different native planter arrangements, seed storage and readable growing notes. |
| Duluth Lakeside Home, Terrace Home, Slip House, Hill Ranger | Complete tables and separate notebook, porch-gift, chart and trail-record corners. |
| St Paul Capitol Library | Two legible book aisles and a shared reading table, with the original host and reader. |
| St Paul Community Hall | Timber meeting/supper room and a coat-drive shelf. |
| St Paul South Landing Inn | An actual guest bed beside its supper table; retain native free care. |
| St Paul South Landing Survey | River charts, field-work table and file storage replace the sparse center. |
| St Paul Freight Office | Filing cupboards and a parcel table; the original main-story manifest stays in Union Depot. |
| St Paul fair agriculture/exhibition halls | Connected planter exhibits, a comparison counter, casting displays and demonstration space. |

The details connect objects to the people already in the rooms: a night-shift
tally with corrected names, the foreman's photo of ship cooks, Dorothy-adjacent
travel hospitality, a gull in an ice-out notebook, a potato-shaped owl on a good
trail map, a coat shelf needing no explanation, and a repaired waffle iron on its
way to Winona. These interactions are native background events, not decorative
labels or a new frontend handler.

## Publication

The executed Rust build/publish path is:

```sh
export CARGO_TARGET_DIR=target/interior-life
export CARGO_BUILD_JOBS=4
cargo build --locked --profile wasm-dev -p crystal-assets \
  --bin pack_minnesota_interior_layouts --config 'profile.wasm-dev.opt-level=1'
target/interior-life/wasm-dev/pack_minnesota_interior_layouts \
  content-packs/minnesota-interior-life-v04.crystalpack \
  planning/minnesota/interior-life/layouts.json \
  content-packs/minnesota-interior-layout-v06.crystalpack
```

The output has been written and reread by the Rust verifier. It has 683 maps,
251 species and 251 moves; this overlay changes rooms, not the region's roster.
Packs, executable files, saved fixtures, source-art contact sheets and native
captures remain ignored. Use a save made for this output's identity.

Each room specifies its complete old rows and tileset as a preimage, complete
new rows, a real passive arrival, an adjacent approach for every existing actor,
and exact furniture event positions. Validation rejects missing or buried
residents, inaccessible returns, false stairs, changed passive arrivals, open
room boundaries, occupied event positions and stale terrain. The hostel's old
ladder art becomes an actual native staircase at the same paired coordinates.

The current v6 contract moves the lake notebook and slip-house coffee book
from floor cells onto their actual bookcases. Room terrain is identical to v04.
The publisher now rejects walkable furniture targets and native service targets
such as wall maps, PCs and radios. The earlier 24-room contract is retained at
[revisions/layouts-v04.json](revisions/layouts-v04.json) as review history.

The earlier 17-room v02 contract is retained at
[revisions/layouts-v02.json](revisions/layouts-v02.json). Rejected/intermediate
render originals remain in the ignored `layout-v01-source`, `layout-v02-source`,
`layout-v02-native`, `layout-v03-source` and `v04-art-study` directories. The
visual pass caught half-finished table glyphs and a table fragment used where a
planter was intended; those were corrected in v04 source geometry rather than
in screenshots. The current complete source rooms are in `layout-v04-source`.

## Verification

The v02 batch passed the preservation case, both physical-room controller cases,
all four original interior-story cases and 35 exact native source-pixel camera
comparisons. It also passed all 377 audio PCM/hash/frame/loop checks.

The historical v04 preservation case passes, as do all six production controller
cases (143.47 seconds). Every new furniture interaction, all 117 residents,
free care, both hostel stairs, room returns, both neighborhood errands,
full-pocket gift retry and Save/Continue pass on this exact candidate. The story
parent also rebuilds to the same identity with the updated approach metadata.

All 51 native cameras pass exact source-pixel comparison. Their whole-room
sources and actual images were visually inspected. The two additional feature
cameras verify the inn bed at `(13,6)` and survey worktable at `(11,4)`, because
both sit toward the right of their arrival-camera crops.
All 377 audio programs pass PCM hashes, frame counts and loop ranges.
Logs are retained under ignored `target/interior-life/layout-v04-*` and captures
under `planning/minnesota/renders/interior-life/layout-v04-native` and
`layout-v04-native-features`. All 24 complete source rooms are retained under
`layout-v04-source`; the main native contact pages are in
`layout-v04-native-review`.

Preview the complete rooms:
[first eight](../renders/interior-life/layout-v04-source/contact-01.png),
[next eight](../renders/interior-life/layout-v04-source/contact-02.png),
[last eight](../renders/interior-life/layout-v04-source/contact-03.png).
The [native hostel dormitory](../renders/interior-life/layout-v04-native/MnDuluthHillsideHostel2F-arrival/production.png)
and [native river-inn bed](../renders/interior-life/layout-v04-native-features/south-landing-inn-bed/production.png)
retain their untouched GPU images.

```sh
export RUST_MIN_STACK=67108864
export CRYSTAL_INTERIOR_LAYOUT_BASE="$PWD/content-packs/minnesota-interior-life-v04.crystalpack"
export CRYSTAL_INTERIOR_PACK="$PWD/content-packs/minnesota-interior-layout-v06.crystalpack"
cargo test --locked --profile wasm-dev -p crystal-assets --features test-fixtures \
  --test minnesota_interior_layouts --config 'profile.wasm-dev.opt-level=1' -- --ignored
cargo test --locked --profile wasm-dev -p crystal-bevy --lib interior_l \
  --config 'profile.wasm-dev.opt-level=1' -- --ignored --test-threads=1
```

The preservation case checks all unrelated maps, every gym, source assets,
audio, flags, catalog/recovery data and original compiled command tables. The
production cases walk to every resident and each listed furniture detail, read
actual native dialogue, use the original returns, and verify hostel HP/PP/status
care, both stairs, a bed note and Save/Continue. The existing 117 new residents,
Dorothy's care, full-pocket gift retry, weather and labor errands are replayed
against the revised geometry. Location/party setup is explicit fixture setup;
these checks do not claim earned full-campaign progression.

Ordinary all-target Clippy remains blocked by the existing `bad_bit_mask` error
in core multiplayer. The assets library check with only the existing
`bad_bit_mask` and `never_loop` denials allowed passes, with no warnings in the
new layout builder. Whole-workspace format checking remains blocked by existing
formatting; changed leaf files are checked separately.

The v05 production furniture/resident/return sweep passes (105.67 seconds).
Its unchanged room terrain is retained through the new Ely and St. Paul v02
parents into the final club v02 publication. The two corrected bookcase
approaches also pass exact final native captures and visual review; see
[the five read-position views](../renders/interior-life/clubs-v02-native-review/page-4.png)
and [the lake notebook](../renders/interior-life/clubs-v02-native-review/page-3.png).

The final v05 preservation/deterministic-build and negative-fixture case also
passes (27.13 seconds). All six interior controller cases pass again on the
[699-map composed child](INTEGRATION.md), with the default 24-room contract
retained separately from the club source.
