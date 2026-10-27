# Northwoods tile-item review

The user rejected this batch after review and requested no grass, with attention
focused on a better campfire. Keep these images as rejected history; do not use
the earlier visual review below as current acceptance.

The selected original sheet was generated with built-in ImageGen using the
verbatim [saved prompt](prompt.txt). Its exact source SHA-256 and six
fixed-grid crops are pinned in [catalog.json](catalog.json). The source remains
unchanged; whole-pixel offsets place the canoe/firewood above their southern
floor approach and the marker east of its western standing floor.

The mapping owner supplied
[prioritized native-art and collision requests](../../reviews/minnesota-tile-request.md).
The final handoff provides a separate Johto-compatible tileset, not changes to
frozen city inputs or shared pack publication. Pines, fitted rock-bank families
and tied buoy booms are future requests outside this six-item batch.

Final artifact: ignored
`content-packs/minnesota-northwoods-props-v2.crystalpack`.
SHA-256: `f6c44dcaca8c0f8016ad104408c2f3e998d1cb469cb1befbbed8c2af598fe576`.
It derives from the supplied 538-map
`minnesota-northstar-map-quality.crystalpack`,
SHA-256 `9aa950a35453f06cfee2e4a9b674c6d902af7c311f9ca64a083665ac6c60ddb2`.
This is a map-art extension; it does not include the separately authored
Minnesota species overlay unless a map owner supplies that newer base.

## Visual findings

Inspected every isolated 32px/8× item and the actual published-pack morning,
day, night and collision galleries. The canoe has pointed ends and two supports;
the table has a top and two separate benches; the marker retains a small canoe
pictogram. Firewood, three cobbles and cattail stems remain distinct at native
scale. All use the original native background palette banks. Neither the
marker nor the canoe claims automatic interaction behavior.

The first review showed flat source-background patches on textured native
ground. Publication v2 replaces completely empty 8×8 margin glyphs with exact
native ground/grass glyphs and retains their original palette entries. The
second review shows textured southern approaches below canoe/firewood and
western floor beside the marker. Occupied glyphs preserve their four-color
source clusters. The table's full silhouette occupies all four gameplay
quadrants, so it remains a full blocking block.

The review tool compares all 128 original block images against Johto at
morning/day/night and reports exact pixel equality. The gallery is an art
placement fixture, not traversal, Surf-boundary or earned progression proof.
Map authors still choose legal approaches, connected shore boundaries and
native sign scripts in their own maps.

All source crops, native previews, collision views and both review passes
remain under ignored `planning/minnesota/renders/tiles/`, with content-addressed
render histories. The first ImageGen PNG and prompt remain authoritative
source assets in this directory. See the [runbook](../README.md) for exact
build, publication and verification commands.

The final supplied-pack regression ran with `--include-ignored`: three passed,
zero failed, zero ignored. It independently checks the source hashes, all prop
indices, bank addressing and native bitplanes; it also compares every original
runtime file, every old block/palette/collision and the entire game dataset
after removing only the explicitly added tileset and inherited field rules.

The final v2 pack also passes `verify_crystalpack --audio --desktop`:
538 maps, 251 species, 251 moves and 377 audio programs. Every synthesized PCM
hash, frame count and loop range matches. A separate read of the published
pack verifies all six prop bitplane streams against the importer’s saved native
indices, including exact original-ground addressing in empty margins.

Repository-wide `cargo fmt --all -- --check` reports existing formatting
differences including `crates/offset-allocator/src/lib.rs`,
`crates/offset-allocator/src/tests.rs` and existing `pack_core` exporter files.
The three new Rust files pass their direct `rustfmt --check`.
Required all-target Clippy attempts for assets and renderer are blocked by the
existing `clippy::bad_bit_mask` error at
`crates/crystal-core/src/multiplayer/mod.rs:1332`.
An assets diagnostic run allowing only that existing lint reaches eleven
existing `clippy::never_loop` errors in `mutation_protocol.rs`.
The new tile module's constant-chunk warning was fixed with `as_chunks`;
these unrelated files were not changed by this task.

Focused Clippy passes for the library, publisher and tile tests with only the
two established blocking lint classes allowed:
`cargo clippy --locked -p crystal-assets --lib --bin pack_minnesota_tiles
--test minnesota_tile_art --profile wasm-dev --config
profile.wasm-dev.opt-level=0 -- -A clippy::bad_bit_mask -A clippy::never_loop`.
No warning locations reference the new tile module, publisher or tile test.
Full check logs are retained with the ignored final gallery under
`renders/tiles/northwoods-v2/verification/`.
