use std::collections::BTreeMap;
use std::path::PathBuf;

use bevy::asset::Assets;
use bevy::image::Image;
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_runtime::CrystalRuntime;

use super::super::{
    RenderedTilesetArt, grass_rustle_frames_for_art, load_tileset_art, load_tileset_palette_bank,
    parse_palette_file, winter_render_behavior,
};
use super::{
    is_prairie, is_winter, light_color, light_prairie_bank, light_winter_bank, load_foregrounds,
};

#[test]
fn seasonal_lighting_preserves_day_and_darkens_all_snow_and_gold_channels() {
    let day = [222, 255, 222];
    let night = [123, 115, 198];
    for color in [
        [239, 255, 255],
        [90, 140, 132],
        [255, 255, 132],
        [115, 74, 0],
    ] {
        assert_eq!(light_color(color, day, day), color);
        let lit = light_color(color, night, day);
        for channel in 0..3 {
            assert!(lit[channel] <= color[channel]);
        }
        assert_ne!(lit, color);
        assert_eq!(light_color(color, [0; 3], day), [0; 3]);
    }
    assert_eq!(light_color([255; 3], [255; 3], [1; 3]), [255; 3]);
}

#[test]
fn harvest_lighting_changes_only_its_gold_bank_and_recognizes_exact_aliases() {
    let mut day = [[[57; 3]; 4]; 8];
    day[0][0] = [222, 255, 222];
    day[7] = [[255, 255, 132], [255, 255, 132], [115, 74, 0], [0; 3]];
    let mut night = day;
    night[0][0] = [123, 115, 198];
    let original = night;
    light_prairie_bank(&mut night, &day);
    assert_eq!(night[..7], original[..7]);
    assert_ne!(night[7], original[7]);
    assert_eq!(night[7][3], [0; 3]);
    let original_day = day;
    light_prairie_bank(&mut day, &original_day);
    assert_eq!(day, original_day);
    for name in ["mn_prairie_kanto", "mn_prairie_headwaters"] {
        assert!(is_prairie(name));
    }
    for name in [
        "mn_winter_outdoor",
        "mn_winter_north_shore",
        "mn_outdoor_winter_stairs",
        "mn_winter_gunflint",
        "mn_ely_snow",
        "mn_international_falls",
    ] {
        assert!(is_winter(name));
    }
    for name in [
        "johto",
        "kanto",
        "ice_path",
        "mn_state_fair_native",
        "mn_winter_outdoor_extra",
        "mn_outdoor_winter_stairs_extra",
    ] {
        assert!(!is_prairie(name));
        assert!(!is_winter(name));
    }
}

#[test]
fn private_winter_banks_keep_day_overrides_and_restore_native_clock_colors() {
    let mut day = [[[57; 3]; 4]; 8];
    day[0][0] = [222, 255, 222];
    let mut night = [[[0; 3]; 4]; 8];
    night[0][0] = [123, 115, 198];
    let mut snow = day;
    snow[2][0] = [255; 3];
    let authored = snow;
    light_winter_bank(&mut snow, &day, &day);
    assert_eq!(snow, authored);
    light_winter_bank(&mut snow, &day, &night);
    for index in [0, 1, 3, 4, 5, 6, 7] {
        assert_eq!(snow[index], night[index]);
    }
    assert_ne!(snow[2][0], authored[2][0]);
    assert_eq!(snow[2][1..], night[2][1..]);
}

fn pixels(images: &Assets<Image>, handle: &bevy::asset::Handle<Image>) -> Vec<u8> {
    images
        .get(handle)
        .expect("production image")
        .data
        .as_ref()
        .expect("CPU pixels")
        .clone()
}

#[test]
#[ignore = "requires CRYSTAL_SEASONAL_RENDER_TEST_PACK with winter and prairie variants"]
fn seasonal_terrain_and_rustles_follow_native_clock_banks_with_day_art_preserved() {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_SEASONAL_RENDER_TEST_PACK").expect("external seasonal pack"),
    )
    .canonicalize()
    .expect("existing pack");
    let assets = AssetRoot::new(path.parent().expect("pack directory"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &assets,
        read_loaded_verified_compiled_game_pack(&path).expect("verified seasonal pack"),
    )
    .expect("seasonal runtime");
    let root = runtime.materialize_runtime_files().expect("mount pack art");
    let shared = crate::read_runtime_asset_to_string(
        root.runtime_assets().join("gfx/tilesets/bg_tiles.pal"),
    )
    .expect("native shared palette");
    let capture = std::env::var_os("CRYSTAL_SEASONAL_CAPTURE_DIR").map(PathBuf::from);
    if let Some(path) = &capture {
        std::fs::create_dir(path).expect("fresh capture directory");
    }
    let mut images = Assets::<Image>::default();
    let mut rendered = RenderedTilesetArt::default();
    for (tileset, map) in [
        ("mn_winter_outdoor", "MnRainyRiverIcefields"),
        ("mn_winter_north_shore", ""),
        ("mn_winter_gunflint", ""),
        ("mn_ely_snow", "MnEly"),
        ("mn_international_falls", "MnInternationalFalls"),
        ("mn_prairie_kanto", "MnR02C00R00"),
        ("mn_prairie_headwaters", "MnR16C00R00"),
    ] {
        let definition = &runtime.data().tilesets[tileset];
        let foreground_path = root
            .runtime_assets()
            .join(format!("data/tilesets/{tileset}_foregrounds.json"));
        let original_foregrounds: BTreeMap<usize, [[u8; 3]; 3]> =
            if crate::runtime_asset_exists(&foreground_path) {
                serde_json::from_str(
                    &crate::read_runtime_asset_to_string(foreground_path).expect("authored colors"),
                )
                .expect("foreground metadata")
            } else {
                BTreeMap::new()
            };
        let mut day_pixels = Vec::new();
        let mut day_rustles = Vec::new();
        for time in ["day", "morn", "nite", "dark"] {
            let native = parse_palette_file(&shared, Some(time)).expect("native bank");
            let bank = load_tileset_palette_bank(&root, tileset, time)
                .expect("load bank")
                .expect("bank");
            if is_prairie(tileset) {
                assert_eq!(bank[..7], native[..7], "unaffected terrain banks");
                if time == "day" {
                    assert_eq!(bank[7], native[7]);
                } else {
                    assert_ne!(bank[7], native[7], "harvest gold follows {time}");
                }
            } else {
                let private_path = root
                    .runtime_assets()
                    .join(format!("gfx/tilesets/{tileset}.pal"));
                if crate::runtime_asset_exists(&private_path) {
                    let private = parse_palette_file(
                        &crate::read_runtime_asset_to_string(private_path)
                            .expect("private winter palette"),
                        None,
                    )
                    .expect("private colors");
                    if time == "day" {
                        assert_eq!(bank, private[..8], "private daylight stays exact");
                    } else {
                        assert_ne!(bank, private[..8], "private winter bank follows {time}");
                    }
                    assert_eq!(bank[0], native[0], "native clock background and contrast");
                } else {
                    assert_eq!(bank, native[..8], "winter keeps native banks");
                }
            }
            let foregrounds =
                load_foregrounds(&root, tileset, Some(&bank)).expect("lit foregrounds");
            if time == "day" {
                assert_eq!(foregrounds, original_foregrounds);
            } else if is_winter(tileset) {
                assert_ne!(foregrounds, original_foregrounds, "snow follows {time}");
            }
            let art = load_tileset_art(&root, tileset, time, &definition.palette_map, &mut images)
                .expect("production terrain");
            if is_prairie(tileset) {
                let source = if tileset == "mn_prairie_kanto" {
                    "kanto"
                } else {
                    "mn_headwaters"
                };
                let source_art = load_tileset_art(
                    &root,
                    source,
                    time,
                    &runtime.data().tilesets[source].palette_map,
                    &mut images,
                )
                .expect("original prairie source terrain");
                // Later source extensions were not part of these clones. Check
                // the original inherited prefix, before the two grass blocks.
                let inherited_len = if tileset == "mn_prairie_kanto" {
                    128 * 16
                } else {
                    131 * 16
                };
                assert_eq!(
                    art.metatile_layout[..inherited_len],
                    source_art.metatile_layout[..inherited_len]
                );
                for glyph in source_art.metatile_layout.into_iter().take(inherited_len) {
                    assert_eq!(
                        pixels(&images, &art.tile_handles[usize::from(glyph)]),
                        pixels(&images, &source_art.tile_handles[usize::from(glyph)]),
                        "all original prairie source glyphs stay exact at {time}"
                    );
                }
            }
            let all_pixels: Vec<_> = art
                .tile_handles
                .iter()
                .map(|handle| pixels(&images, handle))
                .collect();
            if time == "day" {
                day_pixels = all_pixels;
            } else {
                assert_ne!(
                    all_pixels, day_pixels,
                    "actual {tileset} terrain pixels follow {time}"
                );
            }
            if is_prairie(tileset) || tileset.starts_with("mn_winter_") {
                let frames =
                    grass_rustle_frames_for_art(&mut rendered, &root, tileset, time, &mut images)
                        .expect("production rustles");
                let frame_pixels: Vec<_> = frames
                    .iter()
                    .map(|frame| pixels(&images, &frame.handle))
                    .collect();
                if time == "day" {
                    day_rustles = frame_pixels;
                } else {
                    assert_ne!(frame_pixels, day_rustles, "rustles follow {time}");
                }
                if let Some(palette) = winter_render_behavior::grass_palette(&root, tileset, time)
                    .expect("winter rustle colors")
                {
                    for frame in frames {
                        for pixel in pixels(&images, &frame.handle)
                            .as_chunks::<4>()
                            .0
                            .iter()
                            .filter(|pixel| pixel[3] == 255)
                        {
                            assert!(
                                palette[1..].iter().any(|color| color == &pixel[..3]),
                                "rustles use lit terrain colors"
                            );
                        }
                    }
                }
            }
            if let Some(directory) = capture.as_ref().filter(|_| !map.is_empty()) {
                let map = &runtime.data().maps[map];
                let width = u32::from(map.attributes.width);
                let mut canvas =
                    image::RgbaImage::new(width * 32, u32::from(map.attributes.height) * 32);
                for (index, block) in map.blocks.iter().enumerate() {
                    let glyphs = &art.metatile_layout
                        [usize::from(*block) * 16..(usize::from(*block) + 1) * 16];
                    for (offset, glyph) in glyphs.iter().enumerate() {
                        let glyph = image::RgbaImage::from_raw(
                            8,
                            8,
                            pixels(&images, &art.tile_handles[usize::from(*glyph)]),
                        )
                        .expect("native glyph extent");
                        image::imageops::replace(
                            &mut canvas,
                            &glyph,
                            i64::from(index as u32 % width * 32 + offset as u32 % 4 * 8),
                            i64::from(index as u32 / width * 32 + offset as u32 / 4 * 8),
                        );
                    }
                }
                canvas
                    .save(directory.join(format!("{tileset}-{time}.png")))
                    .expect("save actual production terrain textures");
            }
        }
    }
    // Native regions and the gold UI palette retain the original palette bytes.
    for time in ["day", "morn", "nite", "dark"] {
        let native = parse_palette_file(&shared, Some(time)).expect("native bank");
        for tileset in ["johto", "kanto", "mn_state_fair_native"] {
            let actual = load_tileset_palette_bank(&root, tileset, time)
                .expect("original bank")
                .expect("bank");
            assert_eq!(actual, native[..8], "unrelated native palette unchanged");
        }
    }
}
