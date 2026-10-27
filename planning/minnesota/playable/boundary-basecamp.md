# Boundary Waters basecamp — native review build

The revised 32×24 town and 8×6 Survey Shelter are published with three guarded
walking links in the ignored
`content-packs/minnesota-boundary-basecamp-connected.crystalpack` (471 maps),
hash `9cb8936dbd1afeeae3abe69e237aeddf12d5c8bccb3a8eb9f8b7ed53bdb45841`.
The retained `minnesota-boundary-basecamp.crystalpack` is the bare-town variant.
The current terrain uses the aligned central spine and fitted Gunflint pine shoulders;
the rest-grove review camera stands beside the Guide at46,38. The source
layout, original residents, door, room and story anchors match the native pack.
Final user visual approval and the complete regional journey remain open.

[Whole town](../renders/boundary-basecamp/whole-town.png) ·
[Native engine review](../renders/boundary-basecamp/engine/derived-lcd-engine-review-cc1d8bffcca6.png) ·
[Town authoring](../towns/MnBoundaryWaters.md) ·
[Separate guarded connections](boundary-basecamp-connections.md)

## Reproduce the connected publication

Use the completed 469-map regional source pack, retaining its original HOME,
recovery table, all existing maps, native art and audio:

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo run --profile wasm-dev --locked \
  -p crystal-assets --bin pack_boundary_basecamp -- \
  content-packs/minnesota-region-review.crystalpack \
  planning/minnesota/towns/MnBoundaryWaters.json \
  planning/minnesota/towns/MnBoundaryWaters.layout.json \
  content-packs/minnesota-boundary-basecamp-connected.crystalpack --connect-routes

target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-boundary-basecamp-connected.crystalpack --audio --desktop

.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_boundary_basecamp_native.py \
  --pack content-packs/minnesota-boundary-basecamp-connected.crystalpack \
  --source-pack content-packs/minnesota-region-review.crystalpack
```

The CLI strictly reads back its full written CBOR pack before reporting success.
`--connect-routes` installs three reciprocal native walking seams and six
checkpoint Officers atomically, without changing adjoining route terrain.
Omit the flag and select the bare output filename to reproduce the six-resident
town without regional seams. Original maps/art/audio and HOME/recovery remain
preserved apart from the explicit connected-map script/connection additions.
Neither mode implements a paid Safari capture expedition.

## Executed evidence

Twenty asset/core tests pass for actual publication, original content
preservation, exact doors and both carpets, free native healing, source/story
agreement, Q07 and all six connection-component cases, including an actual full
CBOR write/read round trip. All 377 audio hashes, frame counts and loop ranges
match. The native writer uses raw `coord_event` scene token `-1` with the
corresponding typed always-active scene; strict validation remains enabled.

All nine fresh production-controller cases pass on the connected pack: four
town/story cases below and five [guarded-connection cases](boundary-basecamp-connections.md).
The latter cover both lanes in both directions, 28 prerequisite refusals,
unqualified inbound and transverse free-care escorts, public Surf boundaries,
room reentry and Continue.

- Zero-money, zero-badge walks from each public landing to the real shelter;
  native Medic restoration of HP, status and every move's PP; both carpet
  exits, passive return, ordinary upward turn/re-entry and Continue.
- Each missing badge independently, missing lighthouse signal and locked
  receiver refuse the story action without granting progression.
- All three chief opponents actually faint through selected moves that spend
  PP and award experience. Victory grants only the chief event. A separate
  receiver decline/retry/accept interaction grants restoration. Repeat and
  Continue preserve the intended state. This case then walks the newly unlocked
  South connection to `MnR13C00R00(30,0)` and Continues there; restoration is
  earned through those inputs rather than injected after setup.
- A played losing battle reaches ordinary whiteout and Continue without
  granting chief victory or restoration.

These tests explicitly initialize a legal level 80 Mew review partner, badges,
upstream signal, position and random stream; the losing fixture instead starts
with a level 5 Magikarp at 1 HP. No state, party, flags or divider are rewritten
after setup. This proves local input behavior, not an earned eight-badge trip,
combat balance or an earned Minnesota-region playthrough.

```sh
CARGO_TARGET_DIR=target/runtime-fps \
CRYSTAL_BOUNDARY_BASECAMP_TEST_PACK="$PWD/content-packs/minnesota-boundary-basecamp-connected.crystalpack" \
CRYSTAL_BOUNDARY_BASECAMP_CONNECTED_TEST_PACK="$PWD/content-packs/minnesota-boundary-basecamp-connected.crystalpack" \
CRYSTAL_BOUNDARY_BASECAMP_SAVE_DIR="$PWD/.local/boundary-basecamp-review-saves" \
cargo test --profile wasm-dev --locked -p crystal-bevy --features location-tester \
  boundary_basecamp_ -- --ignored --test-threads=1
```

The retained **bare-town** chief/receiver save was loaded through stdio MCP: Start,
room walking, the downward carpet, safe exterior43,20 and ordinary upward
re-entry all passed with byte-empty stderr. The fullscreen Ratatui client also
loaded it, opened production Start/PACK by keyboard and quit with empty stderr.
The connected earned-release save also loads through both frontends. MCP opens
production Start and walks the South connection inward and outward; the
fullscreen client opens Start/PACK by keyboard and quits normally. Both keep
stderr byte-empty. Use the connected review save directly:

```sh
target/runtime-fps/wasm-dev/geothite play \
  content-packs/minnesota-boundary-basecamp-connected.crystalpack \
  --load .local/minnesota-boundary-basecamp-connected-earned.crystalsave
```

The native pack verifies all 377 audio PCM hashes, frame counts and loop ranges.
The read-only comparison checks 768 town blocks, 48 shelter blocks, exact actors,
signs, original art and source/native whole-image equality. It labels bare and
connected initial-unqualified scenes separately and validates all six extra
guard bodies, twelve raw/typed coordinate planes, native prerequisite/movement
tables and reciprocal offsets. The current connected comparison passes with 141
retained artifacts; twelve adversarial source/raw/compiled direction mutations
are rejected. Static rendering does not replace the controller visibility proof.
Eleven successful 640×576 production 2D captures cover the junction, Gunflint entrance, dock and
launch, shelter yard, closed admission, rest grove and four room positions.
Their explicitly derived 480×432/160×144 center crops retain raw-file provenance.
The extra direct 480×432 junction image is also retained; its capture wrapper
rejected the smaller dimensions even though the pixels match the derived crop.
It is not counted as a passing screenshot command.

Originals, conversions, crops and rejected passes remain in immutable render
history. Source rendering remains independent of Cargo and native publication.

Current execution logs are `.local/boundary-basecamp-guarded-assets.log`
(20 passed), `.local/boundary-basecamp-controllers-current.log` (nine passed),
and `.local/boundary-basecamp-connected-native-compare-final.log`.
Connected frontend evidence is in `.local/boundary-basecamp-connected-mcp-review.log`
and `.local/boundary-basecamp-connected-tui-review.log`. The three extra native
checkpoint captures and their160-pixel crops are retained under
`renders/boundary-basecamp/engine/connected-*`.

Scoped Rust formatting passes for the12 changed basecamp/surface/authoring
leafs. Clippy passes for `crystal-assets` and `crystal-bevy` with the existing
`never_loop`, `bad_bit_mask` and `approx_constant` allowances; no warnings point
to the new basecamp leaves. All10 surface regressions pass. Full-workspace
`cargo fmt --all -- --check` still reports unrelated formatting beginning at
`crates/offset-allocator/src/lib.rs:408`; that code was not changed here.
The connected pack also passes the original production battle/PP and Mom/New
Bark regressions, real MCP movement/Start regression and clean-stderr stdio
regression. Their individual logs are `.local/boundary-basecamp-connected-`
`{battle,mom,mcp,stdio}-regression.log`.
