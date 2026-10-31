# Minnesota Trainer Card badges

The native badge page displays Care, Prairie, Current, Capitol, Wind, Lake,
Pine, and Harbor in Gym order. It reads the existing eight regional badge
bits in `badges.johto`; it does not award badges or change save data.

Art and animation live in
`crates/crystal-bevy/src/bevy_shell/minnesota_badges.rs`. Each badge is a
24×24 sprite assembled from nine 8×8 tiles. Every tile uses Crystal's
interleaved 2bpp format: sixteen bytes, four palette indices, index zero
transparent. The source patterns compile to eight frames per badge. The
existing Trainer Card clock advances one frame per eight game ticks, with
staggered light accents. Unearned slots use a fixed frame and recessed palette.

The motifs are a heart and care cross, prairie seed head, electric bolt,
capitol dome, wing, lake drop, pine, and harbor anchor. Labels remain visible
before earning a badge; filled corner marks and the collection count show
progress. Left returns to the information page. A or B closes the badge page
and restores the Trainer Card's Start menu selection.

`trainer_card.rs` identifies Minnesota from pack identity or regional maps in
the catalog, including when standing on retained original maps. Both graphical
and text card entries use the Minnesota names. The art cache includes region,
badge ownership, and animation frame. Other regions retain their existing art.

Run the tile and production menu regressions with a completed Minnesota pack:

```sh
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/minnesota-current-st-cloud-fronts-v02.crystalpack" \
RUST_MIN_STACK=67108864 \
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
cargo test --locked -p crystal-bevy --lib --features location-tester \
  minnesota_badges -- --include-ignored --test-threads=1
```

The production regression starts New Game, opens the card through Start input,
checks empty and explicitly supplied earned-badge fixtures, verifies native
frame pixels and cache invalidation, plays the complete idle animation loop,
and returns through the information page to the Start menu. These are visual
fixtures, not evidence of earned Gym victories. Set `POKEGEAR_PC_RENDER_DIR`
to an ignored directory to capture the eight production LCD frames. Generated
captures, previews, and build output stay outside tracked source.

The larger test-thread stack accommodates the complete external regional
catalog in the unoptimized test profile.

Verified on 2026-10-30: all four badge checks pass on that external pack. The
eight production LCD captures contain phases 0 through 7 in order and exactly
match the tile renderer. The original Johto menu/page-return regression passes
with `core-modular.browser.crystalpack`, and the VBlank phase regression passes
with the native core pack. All-target Clippy with `location-tester` passes;
warnings elsewhere in the workspace remain. The standalone original-card
palette fixture cannot find its font PNG, and the workspace-wide formatting
check reports existing differences. The new Rust modules pass rustfmt.
