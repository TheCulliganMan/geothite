//! Actual imported pixel family and production tile_handle_at_frame scheduling.
use std::path::PathBuf;

use bevy::asset::Assets;
use bevy::image::Image;
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_runtime::CrystalRuntime;

use super::super::{load_tileset_art, minneapolis_falls_animation};

fn runtime(variable: &str, fallback: &str) -> CrystalRuntime {
    let path = std::env::var_os(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(fallback)
        })
        .canonicalize()
        .expect("completed external native pack");
    CrystalRuntime::from_loaded_compiled_pack(
        &AssetRoot::new(path.parent().expect("pack parent")),
        read_loaded_verified_compiled_game_pack(&path).expect("verified external pack"),
    )
    .expect("actual native runtime")
}

fn pixels<'a>(images: &'a Assets<Image>, handle: &bevy::asset::Handle<Image>) -> &'a [u8] {
    images
        .get(handle)
        .expect("CPU image")
        .data
        .as_deref()
        .expect("RGBA pixels")
}

#[test]
#[ignore = "requires completed compact Falls native pack"]
fn minneapolis_falls_native_pixels_and_production_phase_match_cave_scroll() {
    let runtime = runtime(
        "CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK",
        "content-packs/minnesota-minneapolis-compact-falls-v02.crystalpack",
    );
    let root = runtime
        .materialize_runtime_files()
        .expect("temporary runtime mount");
    let mut images = Assets::<Image>::default();
    let city = load_tileset_art(
        &root,
        "mn_minneapolis_city",
        "day",
        &runtime.data().tilesets["mn_minneapolis_city"].palette_map,
        &mut images,
    )
    .expect("city art");
    let cave = load_tileset_art(
        &root,
        "cave",
        "day",
        &runtime.data().tilesets["cave"].palette_map,
        &mut images,
    )
    .expect("cave art");
    let original = load_tileset_art(
        &root,
        "johto_modern",
        "day",
        &runtime.data().tilesets["johto_modern"].palette_map,
        &mut images,
    )
    .expect("original art");
    let glyph = minneapolis_falls_animation::glyph("mn_minneapolis_city", &city.metatile_layout)
        .expect("exact native append")
        .expect("falls glyph");
    let a = &city.animated_tiles[&glyph];
    assert_eq!((a.frame_ticks, a.phase_offset, a.frames.len()), (19, 16, 8));
    assert!(a.advance_on_phase_offset);
    assert!(!a.requires_forest_restless && !a.cave_water_composite);
    assert!(a.additional_schedule.is_empty());
    assert_eq!(city.animated_tiles.len(), original.animated_tiles.len() + 1);
    // Waterfront imports occupy previously unused logical slots. Native
    // waterfall scheduling must never animate the new house windows or roofs.
    for (index, &glyph) in city.metatile_layout.iter().enumerate() {
        if glyph < 224 {
            continue;
        }
        assert!(!city.animated_tiles.contains_key(&usize::from(glyph)));
        let block = index / 16;
        let column = index % 4;
        let row = index % 16 / 4;
        let first = city
            .tile_handle_at_frame(block as u16, column, row, 0, false)
            .expect("static waterfront frame");
        let later = city
            .tile_handle_at_frame(block as u16, column, row, 168, false)
            .expect("later static waterfront frame");
        assert_eq!(pixels(&images, &first), pixels(&images, &later));
    }
    for (id, retained) in &original.animated_tiles {
        let actual = &city.animated_tiles[id];
        assert_eq!(
            (
                actual.frame_ticks,
                actual.phase_offset,
                actual.advance_on_phase_offset
            ),
            (
                retained.frame_ticks,
                retained.phase_offset,
                retained.advance_on_phase_offset
            )
        );
        assert_eq!(
            (actual.requires_forest_restless, actual.cave_water_composite),
            (
                retained.requires_forest_restless,
                retained.cave_water_composite
            )
        );
        assert_eq!(actual.additional_schedule, retained.additional_schedule);
        assert_eq!(actual.frames.len(), retained.frames.len());
        for (a, b) in actual.frames.iter().zip(retained.frames.iter()) {
            assert_eq!(
                pixels(&images, a),
                pixels(&images, b),
                "unrelated glyph{id:02x}"
            );
        }
    }
    let base = pixels(&images, &a.frames[0]);
    assert_ne!(
        base,
        pixels(&images, &a.frames[1]),
        "fall must visibly animate"
    );
    // Independent pixel oracle: each successive frame scrolls DOWN three rows.
    for stage in 0..8 {
        let actual = pixels(&images, &a.frames[stage]);
        for y in 0..8 {
            for x in 0..8 {
                let from = (((y + 8 - (stage * 3 % 8)) % 8) * 8 + x) * 4;
                let to = (y * 8 + x) * 4;
                assert_eq!(&actual[to..to + 4], &base[from..from + 4]);
            }
        }
    }
    // These tick boundaries test the real production selector, not just fields.
    for (tick, phase) in [
        (0, 0),
        (15, 0),
        (16, 1),
        (34, 1),
        (35, 2),
        (54, 3),
        (73, 4),
        (92, 5),
        (111, 6),
        (130, 7),
        (148, 7),
        (149, 0),
        (168, 1),
    ] {
        for y in 0..4 {
            for x in 0..4 {
                let actual = city
                    .tile_handle_at_frame(0x99, x, y, tick, false)
                    .expect("fall frame");
                let native = cave
                    .tile_handle_at_frame(0x2c, x, y, tick, false)
                    .expect("cave frame");
                assert_eq!(
                    pixels(&images, &actual),
                    pixels(&images, &a.frames[phase]),
                    "tick{tick} quadrant{x},{y}"
                );
                assert_eq!(
                    pixels(&images, &actual),
                    pixels(&images, &native),
                    "native cave phase tick{tick}"
                );
            }
        }
    }
}

#[test]
#[ignore = "requires external original 64x48 Minneapolis pack"]
fn minneapolis_legacy_city_keeps_all_original_animation_pixels_and_schedules() {
    let runtime = runtime(
        "CRYSTAL_MINNEAPOLIS_LEGACY_ANIMATION_PACK",
        "content-packs/minnesota-route-quality-next-v01.crystalpack",
    );
    let root = runtime
        .materialize_runtime_files()
        .expect("temporary runtime mount");
    let mut images = Assets::<Image>::default();
    let city = load_tileset_art(
        &root,
        "mn_minneapolis_city",
        "day",
        &runtime.data().tilesets["mn_minneapolis_city"].palette_map,
        &mut images,
    )
    .expect("legacy city");
    let original = load_tileset_art(
        &root,
        "johto_modern",
        "day",
        &runtime.data().tilesets["johto_modern"].palette_map,
        &mut images,
    )
    .expect("native source");
    assert_eq!(city.metatile_layout.len(), 0x99 * 16);
    assert_eq!(city.animated_tiles.len(), original.animated_tiles.len());
    for (id, b) in &original.animated_tiles {
        let a = &city.animated_tiles[id];
        assert_eq!(
            (a.frame_ticks, a.phase_offset, a.advance_on_phase_offset),
            (b.frame_ticks, b.phase_offset, b.advance_on_phase_offset)
        );
        assert_eq!(
            (a.requires_forest_restless, a.cave_water_composite),
            (b.requires_forest_restless, b.cave_water_composite)
        );
        assert_eq!(a.additional_schedule, b.additional_schedule);
        assert_eq!(a.frames.len(), b.frames.len());
        for (a, b) in a.frames.iter().zip(b.frames.iter()) {
            assert_eq!(pixels(&images, a), pixels(&images, b));
        }
    }
}
