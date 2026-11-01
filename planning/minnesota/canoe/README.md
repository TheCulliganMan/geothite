# Canoe water travel

The CANOE is a passive, reusable KEY ITEM. Minnesota players who carry it
launch automatically when walking into an open water tile. Reaching ordinary
land dismounts the canoe for portages. No party move or PP is required. Native
Surf remains available; explicit Surf retains its actual Pokémon rider identity.
Johto and Kanto keep their original water-travel requirements.

St. Paul's existing `MnStPaulAnglerHouse` becomes the canoe shop. Its owner at
`(5,3)` gives a free canoe after the Capitol Badge (`ENGINE_FOGBADGE`), explains
launching and portages, then resumes the existing angler/HM/story program. The
bag must accept delivery before the gift is owned; repeat conversations do not
issue another boat. A cedar display at `(11,5)` is inspectable. The exterior
entrance and return warps remain intact. A native Kanto sign at `(9,63)` names
the shop, using metatile `79` at `(4,31)`; its bottom-right quadrant blocks
walking and the other three quadrants remain floor.

Water travel continues through the existing movement transaction and production
`VisibleShellController`. Collision, occupied tiles, shore walls, bridge layers,
water obstacles, encounters, connections and ordinary dismount stay authoritative.
The canoe cannot launch from an elevated bridge deck. Boundary Waters admission
accepts canoe ownership as an alternative to native Surf permission and keeps
its normal money, storage and expedition checks.

`canoe_sprites.rs` authors six native frames for Chris and Kris using the local
pack's original trainer heads, explicit cedar hulls and alternating paddle
strokes. The right view mirrors the authored left view, following the native
sprite contract. A separate static bank depicts the shop's canoe display.
All PNG/palette generation stays inside the Rust pack builder. Original sprite
banks, audio programs, Pokémon, encounters and unrelated maps are preserved.
Eight existing regional shore signs and NPC hints now describe paddling and
portaging instead of requiring Surf. Brainerd's actual Surf HM gift remains.
Native trainer PNGs use packed grayscale; the publisher reads their original
2bpp banks through the existing Rust decoder instead of converting the PNGs.

The publisher is `pack_canoe BASE OUTPUT`. Use a fresh ignored output beneath
`content-packs/`; the source and generated pack must differ. The source needs the
verified compact Minnesota coast, St. Paul room/owner and native Chris/Kris art.
No original source export is needed for this additive stage.

## Verified publication

The current candidate is `content-packs/minnesota-canoe-v04.crystalpack`, built
from `content-packs/minnesota-current-grass-r12-v01.crystalpack` (705 maps,
475 total species). Packs and renders remain ignored.

Executed Rust build and publication commands:

```sh
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/canoe cargo build --locked \
  --profile wasm-dev -p crystal-assets --features test-fixtures \
  --bin pack_canoe --bin verify_crystalpack \
  --config profile.wasm-dev.package.crystal-core.opt-level=0 \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0
./target/canoe/wasm-dev/pack_canoe \
  content-packs/minnesota-current-grass-r12-v01.crystalpack \
  content-packs/minnesota-canoe-v04.crystalpack
./target/canoe/wasm-dev/verify_crystalpack \
  content-packs/minnesota-canoe-v04.crystalpack --audio --desktop
```

Structural and desktop presentation verification pass. All 377 audio programs
match their expected PCM hashes, frame counts and loop ranges. Preservation and
canoe-only Safari admission regressions pass. An audit of all 263 regional trainer
definitions, the affected maps' wild/gift placements and all 123 active-species
evolution links found no out-of-region or retired species. New Rust files
pass scoped rustfmt checks. The workspace-wide formatting check reports existing
formatting differences outside this change.

Production controller regressions pass for the badge-gated shop gift, repeat
conversations, the inspectable display, automatic launch without a Surf partner,
portaging, and Save/Continue both on land and afloat. The nine focused canoe
regressions pass. Native 2D captures were reviewed offscreen at the shop and the
St. Paul shoreline; the revised pointed hulls, raised gunwales and diagonal paddle
strokes are visible in the game renderer. Chris and Kris retain native heads.

Review output is retained under `../renders/canoe-v03/`, including the six-frame
sprite preview and `native-v01/` controller captures. Earlier review attempts
remain under `../renders/canoe-v01/` and `../renders/canoe-v02/`. These ignored images are local review
output, not shipped source art.

The revised candidate's SHA-256 is
`a8d0193ce2c23c8117d4ea6c4a896a33ac4078af89e8a20a8dd6618fa8375701`.
Controller tests use `RUST_MIN_STACK=67108864`,
`CRYSTAL_CANOE_TEST_PACK` pointing to the absolute candidate path, and optionally
`CRYSTAL_CANOE_RENDER_DIR` pointing to a fresh capture directory. Run
`cargo test --locked -p crystal-bevy --lib canoe -- --include-ignored --test-threads=1`.
The asset suite uses `CRYSTAL_CANOE_BASE_PACK` for the parent and
`CRYSTAL_SAFARI_TEST_PACK` for the candidate; run the same filter with
`-p crystal-assets`. Use the build profile and config overrides above if desired.

Clippy was run with `--locked --all-targets` for `crystal-core`,
`crystal-assets`, `crystal-bevy` and `crystal-tui-view`. It reports existing
warnings and stops in the unrelated `crystal-core/tests/radio_text.rs` target:
ten `tools/asm-oracle/fixtures/radio-program-*.json` inputs are absent, including
`radio-program-rocket.json`. Those fixtures are also absent from the live
checkout. No canoe-specific diagnostics appeared. The complete all-target check
cannot be reported as passing. Workspace-wide `cargo fmt --all -- --check`
reports existing formatting differences; the new canoe modules pass rustfmt.

The v04 visual refinement adds tucked knee/boot outlines only in Chris and
Kris's two forward-facing frames. Pixel comparison against v03 confirms changes
are confined to columns 5-10 and rows 10-12 in those frames. All other sprite
pixels, palettes, maps, scripts, gameplay data, reports and audio are identical.
The before/after comparison, with native-size views, is retained at
`../renders/canoe-v04/front-leg-comparison-v01.png`; each row shows old/new idle
followed by old/new paddle stroke. Final controller captures are under
`../renders/canoe-v04/native-v01/`.

For the v04 pixel refinement, `cargo clippy --locked --profile wasm-dev
-p crystal-assets --all-targets` (with the two build-profile overrides above)
passes with existing warnings and no diagnostics in `canoe_sprites.rs`.
Scoped rustfmt passes; workspace-wide formatting still reports unrelated
existing differences. Both production canoe controller regressions and all
377 audio checks pass against v04.
