//! Reached green water rendered by the production native animation cache.
use std::collections::BTreeMap;

use bevy::asset::{Assets, Handle};
use bevy::image::Image;

use super::super::super::load_tileset_art;

fn pixels<'a>(images: &'a Assets<Image>, handle: &Handle<Image>) -> &'a [u8] {
    images
        .get(handle)
        .expect("production tile image")
        .data
        .as_deref()
        .expect("native RGBA pixels")
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Verdigrim intake scene"]
fn verdigrim_water_native_production_frames_move_recolor_and_leave_floor_static() {
    let (_, runtime, mut c) = super::setup(false);
    // Reach contamination with input; do not edit the map or its animation state.
    super::enter(&mut c);
    super::page(&mut c, "MnVerdigrimGreenText");
    let green = super::water(&c)[0];
    let mount = runtime
        .materialize_runtime_files()
        .expect("native runtime art");
    let mut images = Assets::<Image>::default();
    let art = load_tileset_art(
        &mount,
        "mn_verdigrim_intake",
        "day",
        &runtime.data().tilesets["mn_verdigrim_intake"].palette_map,
        &mut images,
    )
    .expect("production private intake art");
    assert_eq!(art.animated_tiles.len(), 2);
    let source =
        crate::read_runtime_asset(mount.runtime_assets().join("gfx/tilesets/water/water.2bpp"))
            .expect("supplied native water frames");
    assert_eq!(source.len(), 64);
    let mut clean_inks = BTreeMap::new();
    let mut green_inks = BTreeMap::new();
    let mut moving = Vec::new();
    for (phase, frame) in [0, 22, 44, 66].into_iter().enumerate() {
        let clean = art
            .tile_handle_at_frame(0x40, 0, 0, frame, false)
            .expect("native clean-water phase");
        let colored = art
            .tile_handle_at_frame(green, 0, 0, frame, false)
            .expect("native contaminated-water phase");
        let clean = pixels(&images, &clean);
        let colored = pixels(&images, &colored);
        assert_eq!(clean.len(), 8 * 8 * 4);
        assert_eq!(colored.len(), clean.len());
        let pairs = clean
            .as_chunks::<4>()
            .0
            .iter()
            .zip(colored.as_chunks::<4>().0);
        for (pixel, (a, b)) in pairs.enumerate() {
            assert_eq!(a[3], 255);
            assert_eq!(b[3], 255);
            // Native 2bpp is two bitplanes per eight-pixel row. Compare each
            // rendered pixel with that source ink index, rather than assuming
            // distinct native palettes share an identical background RGB.
            let row = phase * 16 + pixel / 8 * 2;
            let shift = 7 - pixel % 8;
            let ink = ((source[row] >> shift) & 1) | (((source[row + 1] >> shift) & 1) << 1);
            if let Some(previous) = clean_inks.insert(ink, *a) {
                assert_eq!(previous, *a, "native wave mask changed");
            }
            if let Some(previous) = green_inks.insert(ink, *b) {
                assert_eq!(previous, *b, "contaminated wave mask changed");
            }
        }
        assert_ne!(clean, colored, "contamination must remain visibly distinct");
        moving.push(colored.to_vec());
    }
    assert_ne!(
        clean_inks[&0], clean_inks[&2],
        "retain clean-water wave contrast"
    );
    assert_ne!(
        green_inks[&0], green_inks[&2],
        "retain contaminated-water wave contrast"
    );
    let ink = green_inks[&2];
    assert!(
        ink[1] > ink[0] && ink[1] > ink[2],
        "green ink must dominate"
    );
    assert_ne!(moving[0], moving[1], "green water must visibly animate");
    assert!(moving.windows(2).all(|pair| pair[0] != pair[1]));
    let looped = art
        .tile_handle_at_frame(green, 0, 0, 88, false)
        .expect("water loop");
    assert_eq!(pixels(&images, &looped), moving[0]);
    let held = art
        .tile_handle_at_frame(green, 0, 0, 21, false)
        .expect("held first phase");
    assert_eq!(pixels(&images, &held), moving[0]);
    let first = art
        .tile_handle_at_frame(7, 0, 0, 0, false)
        .expect("native floor");
    let later = art
        .tile_handle_at_frame(7, 0, 0, 66, false)
        .expect("later native floor");
    assert_eq!(pixels(&images, &first), pixels(&images, &later));
}
