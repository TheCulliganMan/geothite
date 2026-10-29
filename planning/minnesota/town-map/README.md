# Minnesota Town Map and Fly atlas

The atlas is authored in
`crates/crystal-bevy/src/bevy_shell/minnesota_town_map.rs` as a native 20×18
Johto tilemap. It reads the original Town Map sheet and palette from the loaded
external pack through the production paletted renderer. There is no replacement
art sheet, copied source catalog, ImageGen asset or JavaScript renderer.

The layout reserves the native two-row landmark header, frame and footer;
19 location squares and the native selection brackets fit inside the 160×144
LCD. Minnesota's arrowhead, Iron Range, Lake Superior, northern lakes and
southern river corridor share one screen. Pale corridors follow the authored
regional route endpoint pairs; they are an overview, not walking geometry.
Town and route projections are shared by Town Map and Fly. The overview includes
all 19 world-plan location anchors; the verified current parent contains 16
installed towns, and only those 16 receive selectable Fly destinations. Regional cursor
navigation traverses the actual regional index list instead of incrementing
through intervening Johto landmark IDs. Johto/Kanto retain their existing maps.

The discarded schematic and every native review pass are retained in ignored
`planning/minnesota/renders/town-map-v01/`. The user clarified that the map must
use the actual Johto **tiles**; only the tile-based revision is implemented.

## Local verification

`CRYSTAL_TOWN_MAP_RENDER_DIR` opts tests into saving reusable render evidence.
The atlas unit tests check native tile IDs, every town/cursor's LCD bounds,
unique town positions and route cursors on native travel corridors.

The ignored integration test
`minnesota_town_map_and_fly_share_one_screen_art_and_positions` reads an external
pack, traverses the entire regional cursor cycle and composes the real
production UI sprites offscreen. It captures Winona, the North Shore,
the northern border and Fly. Location, a compatible Minnesota bird, Wind badge, HM ownership and visited-town
flags are explicit fixtures; these captures do not claim earned statewide
progression. The chooser opens through the production field-move handler.
Controller input selects Grand Marais, confirms Fly, rejects walking during the
departure and completes travel to the registered dry landing. Both native-resolution originals and nearest-neighbor 4× previews
are retained. No window is opened.

`review_town_map.py` renders a Rust-emitted review tilemap with the source
pack's exact native sheet/palettes for rapid composition review. Its images are
source previews, distinct from the production UI captures.

## Regional flights

`pack_minnesota_flight_atlas PACK OUTPUT` adds visit-unlocked Fly callbacks and
safe dry landings through the existing Rust Minnesota destination installer.
It requires a compact Superior descendant, preserves all source art/audio,
and rejects conflicting destination registrations. Packs remain ignored;
this stage does not replace shared packs or the game's HM/badge requirements.

### Verified pack publication

Executed with the pinned Rust toolchain on 2026-10-28:

```sh
CARGO_TARGET_DIR=target/town-map-review CARGO_BUILD_JOBS=2 \
  cargo run --locked -p crystal-assets --features test-fixtures \
  --bin pack_minnesota_flight_atlas \
  --config 'profile.dev.package."*".opt-level=0' \
  --config profile.dev.opt-level=0 -- \
  content-packs/minnesota-current-grass-r13-v01.crystalpack \
  content-packs/minnesota-town-map-flights-v01.crystalpack
```

Readback verified 705 maps and 16 regional Fly destinations. All Fly spawn IDs
avoid the reserved 28–255 range. The publisher requires a compact Superior
parent and asserts byte-identical source artwork, compiled audio and audio
manifest (including PCM hashes, frame counts and loop ranges). Existing home,
blackout and Teleport spawns remain intact when a separate Fly spawn is needed.

| Pack | SHA-256 |
| --- | --- |
| `minnesota-current-grass-r13-v01.crystalpack` | `9c1813ad83c35f0cfa0c54bcb422efbd49a107ad12b22293b8fbb5f839a9eccf` |
| `minnesota-town-map-flights-v01.crystalpack` | `febbe41548e05253003ad899fd68d27761ca99ff6db7f6f3d35113fda75aebb8` |

### Production UI and flight verification

Four tests passed with the verified child, including the ignored external-pack
integration test:

```sh
RUST_MIN_STACK=67108864 \
CARGO_TARGET_DIR=target/town-map-review CARGO_BUILD_JOBS=2 \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/minnesota-town-map-flights-v01.crystalpack" \
CRYSTAL_TOWN_MAP_RENDER_DIR="$PWD/planning/minnesota/renders/town-map-v01/production-v04" \
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
  cargo test --locked -p crystal-bevy --lib minnesota_town_map \
  --config 'profile.dev.package."*".opt-level=0' \
  --config profile.dev.opt-level=0 -- --include-ignored --nocapture
```

The larger test-thread stack is required for the regional pack fixture. The
first default-stack attempt aborted before rendering; it was not a passing
capture. An unrelated immutable controller binding in `tests/owen_player.rs`
was made mutable to unblock compilation. The completed integration case then
proved every regional cursor is reachable without entering Johto, all 16 Fly
points align with their town squares, Fly owns input during departure, and
native controller input lands at Grand Marais's registered spawn.

The source uses the exact original Johto tiles and palettes. Town Map and Fly
renders at the same selected destination have **16,384 identical atlas/cursor
pixels**, excluding the 16×16 player icon; their different footers are outside
the compared map area. Inspection covered the full image, header wrapping,
North Shore selection, northern border and southern player/footer clearance.
Native originals, 4× previews and the combined proof are retained under
`planning/minnesota/renders/town-map-v01/production-v04/` (ignored).

The Fly display also normalizes terrain suffixes in map constants:
`ENGINE_FLYPOINT_MN_LEECH_LAKE_SHORE` displays **LEECH LAKE**, matching the
atlas and preserving the actual flag/spawn identity.

### Final lint status

Affected-crate all-target Clippy completed successfully for `crystal-assets`
and `crystal-bevy` with `crystal-assets/test-fixtures` enabled. Existing warnings
remain, including `chunks_exact` in the unchanged Pokégear test above the new
integration case. No warnings reference the new atlas module or pack publisher.
`cargo fmt --all -- --check` still reports unrelated workspace formatting,
starting at `crates/offset-allocator/src/lib.rs:408`. Whole-worktree
`git diff --check` also reports an existing trailing blank line in
`bevy_shell/tests.rs`; those unrelated changes were preserved.
