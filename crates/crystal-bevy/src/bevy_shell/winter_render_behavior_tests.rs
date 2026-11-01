use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bevy::asset::Assets;
use bevy::image::Image;
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::world::map::TilePosition;
use crystal_runtime::CrystalRuntime;
use image::{ImageEncoder, RgbaImage};
use sha2::{Digest, Sha256};

use super::super::{
    Palette, RenderedTilesetArt, SpriteFrame, grass_rustle_art_key, grass_rustle_frames_for_art,
    load_npc_sprite_palette_bank, load_tileset_art, palette_index_from_gray,
};
use super::{
    animation_source, grass_cache_style, grass_palette, has_north_shore_waterfall,
    resolve_grass_palette, rustle_tileset, rustle_tileset_at,
};

const WINTER: [(&str, &str, u16); 3] = [
    ("mn_winter_outdoor", "mn_border_pines", 0x3d),
    ("mn_winter_north_shore", "mn_north_shore", 0x84),
    ("mn_winter_gunflint", "mn_gunflint", 0x3d),
];
const TIMES: [&str; 4] = ["morn", "day", "nite", "dark"];

const INLAND: [(&str, &str); 6] = [
    ("mn_winter_bemidji", "mn_bemidji"),
    ("mn_winter_bemidji_northwoods", "johto"),
    ("mn_winter_grand_rapids", "johto_modern"),
    ("mn_winter_iron_range", "mn_iron_range"),
    ("mn_winter_border_pines", "mn_border_pines"),
    ("mn_winter_border_pines_south", "mn_border_pines"),
];

#[test]
fn inland_rustles_resolve_both_border_habitats_and_do_not_color_off_map_tiles() {
    for (winter, source) in INLAND {
        assert_eq!(animation_source(winter), source);
        assert!(super::super::seasonal_palette::is_winter(winter));
        assert!(!super::super::seasonal_palette::is_winter(&format!(
            "{winter}_extra"
        )));
        for point in [
            TilePosition::new(-1, 0),
            TilePosition::new(2, 0),
            TilePosition::new(0, 2),
        ] {
            assert_eq!(rustle_tileset_at(winter, 1, &[0x37], point), source);
        }
    }
    for block in [0x3d, 0x37] {
        assert_eq!(
            rustle_tileset_at(
                "mn_winter_border_pines",
                1,
                &[block],
                TilePosition::new(1, 1)
            ),
            "mn_winter_border_pines"
        );
    }
    assert_eq!(
        rustle_tileset("mn_winter_border_pines", Some(3)),
        "mn_border_pines"
    );
    assert_eq!(animation_source("mn_bemidji_plateau_stairs"), "mn_bemidji");
}

#[test]
#[ignore = "requires CRYSTAL_WINTER_RENDER_TEST_PACK containing all six inland aliases"]
fn inland_terrain_animation_clock_and_rustle_pixels_use_installed_art() {
    use super::super::{load_tileset_palette_bank, seasonal_palette};

    let (root, runtime) = fixture();
    let capture = capture_directory();
    let mut captures = Vec::new();
    let mut images = Assets::<Image>::default();
    let mut rendered = RenderedTilesetArt::default();
    let raw =
        crate::open_runtime_image(root.runtime_assets().join("gfx/overworld/grass_rustle.png"))
            .expect("original rustle")
            .to_rgba8();
    let zero = raw.get_pixel(0, 0);
    let mut source_counts = BTreeMap::new();
    for pixel in raw.pixels().filter(|pixel| *pixel != zero) {
        *source_counts
            .entry(palette_index_from_gray(pixel[0]))
            .or_default() += 1;
    }
    for (winter, source) in INLAND {
        let definition = &runtime.data().tilesets[winter];
        let original_foregrounds: BTreeMap<usize, [[u8; 3]; 3]> = serde_json::from_str(
            &crate::read_runtime_asset_to_string(
                root.runtime_assets()
                    .join(format!("data/tilesets/{winter}_foregrounds.json")),
            )
            .expect("installed foreground metadata"),
        )
        .expect("authored foregrounds");
        let day = load_tileset_palette_bank(&root, winter, "day")
            .expect("day colors")
            .expect("day bank");
        for time in TIMES {
            let bank = load_tileset_palette_bank(&root, winter, time)
                .expect("clock colors")
                .expect("winter bank");
            let foregrounds = seasonal_palette::load_foregrounds(&root, winter, Some(&bank))
                .expect("clock-lit foregrounds");
            if time == "day" {
                assert_eq!(foregrounds, original_foregrounds);
            } else {
                assert_ne!(bank, day, "{winter} {time}");
                assert_ne!(foregrounds, original_foregrounds, "{winter} {time}");
            }
            let art = load_tileset_art(&root, winter, time, &definition.palette_map, &mut images)
                .expect("production inland terrain");
            let original = load_tileset_art(
                &root,
                source,
                time,
                &runtime.data().tilesets[source].palette_map,
                &mut images,
            )
            .expect("native source behavior");
            let mut keys: Vec<_> = art.animated_tiles.keys().copied().collect();
            let mut source_keys: Vec<_> = original.animated_tiles.keys().copied().collect();
            keys.sort_unstable();
            source_keys.sort_unstable();
            assert_eq!(keys, source_keys, "{winter} {time}");
            if matches!(source, "johto" | "johto_modern" | "mn_bemidji") {
                assert_eq!(keys, [0x03, 0x14], "preserve flowers and running water");
            } else {
                assert!(keys.is_empty(), "do not add forest behavior");
            }
            let original_bank = load_tileset_palette_bank(&root, source, time)
                .expect("source colors")
                .expect("native bank");
            for key in keys {
                let a = &original.animated_tiles[&key];
                let b = &art.animated_tiles[&key];
                assert_eq!(a.frame_ticks, b.frame_ticks);
                assert_eq!(a.phase_offset, b.phase_offset);
                assert_eq!(a.requires_forest_restless, b.requires_forest_restless);
                assert_eq!(a.cave_water_composite, b.cave_water_composite);
                assert_eq!(a.advance_on_phase_offset, b.advance_on_phase_offset);
                assert_eq!(a.additional_schedule, b.additional_schedule);
                assert_eq!(a.frames.len(), b.frames.len());
                let source_palette = original_bank
                    [usize::from(runtime.data().tilesets[source].palette_map[key] & 7)];
                let palette = bank[usize::from(definition.palette_map[key] & 7)];
                for (before, after) in a.frames.iter().zip(b.frames.iter()) {
                    let before = images
                        .get(before)
                        .expect("source frame")
                        .data
                        .as_deref()
                        .expect("source pixels");
                    let after = images
                        .get(after)
                        .expect("inland frame")
                        .data
                        .as_deref()
                        .expect("inland pixels");
                    assert_eq!(before.len(), after.len());
                    for (native, derived) in before
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .zip(after.as_chunks::<4>().0)
                    {
                        assert_eq!(native[3], derived[3]);
                        assert!(
                            (0..4).any(|index| native[..3] == source_palette[index]
                                && derived[..3] == palette[index]),
                            "native animation indices and clock colors: {winter} {time} {key:02x}"
                        );
                    }
                }
            }
            if grass_cache_style(winter).is_some() {
                let palette = grass_palette(&root, winter, time)
                    .expect("rustle metadata")
                    .expect("icy grass");
                let frames =
                    grass_rustle_frames_for_art(&mut rendered, &root, winter, time, &mut images)
                        .expect("production icy rustle");
                let native =
                    grass_rustle_frames_for_art(&mut rendered, &root, "johto", time, &mut images)
                        .expect("native rustle");
                for (frame, baseline) in frames.iter().zip(&native) {
                    let pixels = frame_pixels(frame, &images);
                    assert_eq!(colors(pixels), expected_colors(&source_counts, &palette));
                    assert!(
                        pixels
                            .as_chunks::<4>()
                            .0
                            .iter()
                            .zip(frame_pixels(baseline, &images).as_chunks::<4>().0)
                            .all(|(icy, green)| icy[3] == green[3])
                    );
                }
                if let Some(directory) = &capture {
                    captures.extend(capture_rustle_frames(
                        directory, winter, time, &frames, &images,
                    ));
                }
            }
        }
    }
    assert!(rendered.grass_rustle_errors.is_empty());
    if let Some(directory) = capture {
        write_capture(&directory.join("receipt.json"), &serde_json::to_vec_pretty(&serde_json::json!({
            "scope": "Production CPU terrain/animation/rustle assets; GPU weather and traversal require separate captures",
            "aliases": INLAND, "time_banks": TIMES, "rustles": captures,
            "assertions": "Exact animation registrations and schedules, native frame indices/opacity with authored clock colors, lit foregrounds and icy rustle OAM/colors"
        })).expect("capture receipt"));
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINTER_RENDER_TEST_PACK with both Tamarack aliases"]
fn tamarack_mixed_art_keeps_native_warm_pixels_animations_clock_and_grass_masks() {
    assert_mixed_headwaters_art(
        "mn_tamarack_thaw",
        &["mn_winter_tamarack"],
        0xb1,
        &["MnR20C01R00", "MnR20C01R01"],
    );
}

#[test]
#[ignore = "requires CRYSTAL_WINTER_RENDER_TEST_PACK containing the complete Headwaters snowline"]
fn headwaters_mixed_art_keeps_native_warm_pixels_inherited_stairs_and_cut_grass_masks() {
    assert_mixed_headwaters_art(
        "mn_headwaters_thaw",
        &["mn_winter_headwaters", "mn_winter_headwaters_stairs"],
        0x9f,
        &["MnR08C00R01", "MnR08C01R01"],
    );
}

#[test]
#[ignore = "requires CRYSTAL_WINTER_RENDER_TEST_PACK containing the complete Itasca snowline"]
fn itasca_mixed_art_keeps_native_warm_pixels_clips_and_grass_masks() {
    assert_mixed_headwaters_art(
        "mn_itasca_thaw",
        &["mn_winter_itasca_trail"],
        0xb1,
        &["MnR14C00R00", "MnR14C00R01"],
    );
}

fn assert_mixed_headwaters_art(thaw_alias: &str, full_aliases: &[&str], melt: u16, maps: &[&str]) {
    use super::super::{load_tileset_palette_bank, seasonal_palette};

    let (root, runtime) = fixture();
    let mut images = Assets::<Image>::default();
    let mut rendered = RenderedTilesetArt::default();
    let capture = capture_directory();
    let mut captures = Vec::new();
    for time in TIMES {
        let source = load_tileset_art(
            &root,
            "mn_headwaters",
            time,
            &runtime.data().tilesets["mn_headwaters"].palette_map,
            &mut images,
        )
        .expect("native Headwaters art");
        let donor = load_tileset_art(
            &root,
            "mn_winter_outdoor",
            time,
            &runtime.data().tilesets["mn_winter_outdoor"].palette_map,
            &mut images,
        )
        .expect("original icy grass donor");
        let thaw = load_tileset_art(
            &root,
            thaw_alias,
            time,
            &runtime.data().tilesets[thaw_alias].palette_map,
            &mut images,
        )
        .expect("mixed native snowline art");
        assert_eq!(
            thaw.metatile_layout[..source.metatile_layout.len()],
            source.metatile_layout
        );
        for glyph in source
            .metatile_layout
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
        {
            let glyph = usize::from(glyph);
            assert_eq!(
                source.transition_tiles[glyph].indices,
                thaw.transition_tiles[glyph].indices
            );
            assert_eq!(
                source.transition_tiles[glyph].palette, thaw.transition_tiles[glyph].palette,
                "warm glyph {glyph:02x} keeps exact native {time} colors"
            );
            assert_eq!(
                images
                    .get(&source.tile_handles[glyph])
                    .expect("native pixels")
                    .data,
                images
                    .get(&thaw.tile_handles[glyph])
                    .expect("warm pixels")
                    .data
            );
            assert_eq!(
                images
                    .get(&source.priority_tile_handles[glyph])
                    .expect("native opacity")
                    .data,
                images
                    .get(&thaw.priority_tile_handles[glyph])
                    .expect("warm opacity")
                    .data
            );
        }
        let native_bank = load_tileset_palette_bank(&root, "mn_headwaters", time)
            .expect("native clock")
            .expect("native bank");
        let full_grass = [0x03];
        let thaw_grass = [0x88, melt];
        for (alias, grass_blocks) in full_aliases
            .iter()
            .map(|&alias| (alias, full_grass.as_slice()))
            .chain(std::iter::once((thaw_alias, thaw_grass.as_slice())))
        {
            assert!(seasonal_palette::is_winter(alias));
            assert_eq!(animation_source(alias), "mn_headwaters");
            let art = load_tileset_art(
                &root,
                alias,
                time,
                &runtime.data().tilesets[alias].palette_map,
                &mut images,
            )
            .expect("native seasonal alias");
            assert_eq!(
                art.animated_tiles.len(),
                6,
                "water, flowers and four native Whirlpool glyphs"
            );
            assert_eq!(source.animated_tiles.len(), art.animated_tiles.len());
            for (glyph, original) in &source.animated_tiles {
                let derived = &art.animated_tiles[glyph];
                assert_eq!(derived.frame_ticks, original.frame_ticks);
                assert_eq!(derived.phase_offset, original.phase_offset);
                assert_eq!(
                    derived.requires_forest_restless,
                    original.requires_forest_restless
                );
                assert_eq!(derived.cave_water_composite, original.cave_water_composite);
                assert_eq!(
                    derived.advance_on_phase_offset,
                    original.advance_on_phase_offset
                );
                assert_eq!(derived.additional_schedule, original.additional_schedule);
                assert_eq!(derived.frames.len(), original.frames.len());
                for (before, after) in original.frames.iter().zip(derived.frames.iter()) {
                    assert_eq!(
                        images.get(before).expect("native animation").data,
                        images.get(after).expect("mixed animation").data,
                        "{alias} {time} glyph{glyph:02x}"
                    );
                }
            }
            let palette = grass_palette(&root, alias, time)
                .expect("installed icy palette")
                .expect("icy style");
            for &block in grass_blocks {
                assert_eq!(rustle_tileset(alias, Some(block)), alias);
                for part in 0..16 {
                    let glyph = usize::from(art.metatile_layout[usize::from(block) * 16 + part]);
                    let native = usize::from(donor.metatile_layout[0x3d * 16 + part]);
                    assert_eq!(
                        art.transition_tiles[glyph].indices, donor.transition_tiles[native].indices,
                        "every native icy grass bitplane and zero hole"
                    );
                    assert_eq!(
                        art.transition_tiles[glyph].palette[1..],
                        palette[1..],
                        "ground and rustle use the same {time} icy inks"
                    );
                    if block == melt {
                        assert_eq!(
                            art.transition_tiles[glyph].palette[0], native_bank[0][0],
                            "melt grass has native clock-lit green behind unchanged blades"
                        );
                    }
                    let a = images
                        .get(&art.priority_tile_handles[glyph])
                        .expect("derived priority")
                        .data
                        .as_deref()
                        .expect("derived pixels");
                    let b = images
                        .get(&donor.priority_tile_handles[native])
                        .expect("native priority")
                        .data
                        .as_deref()
                        .expect("native pixels");
                    assert!(
                        a.as_chunks::<4>()
                            .0
                            .iter()
                            .zip(b.as_chunks::<4>().0)
                            .all(|(derived, native)| derived[3] == native[3])
                    );
                }
            }
            let frames =
                grass_rustle_frames_for_art(&mut rendered, &root, alias, time, &mut images)
                    .expect("icy rustle production assets");
            for frame in &frames {
                for pixel in frame_pixels(frame, &images)
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .filter(|pixel| pixel[3] != 0)
                {
                    assert!(palette[1..].iter().any(|ink| ink == &pixel[..3]));
                }
            }
            if let Some(directory) = &capture {
                captures.extend(capture_rustle_frames(
                    directory, alias, time, &frames, &images,
                ));
            }
        }
        assert_eq!(
            rustle_tileset(thaw_alias, Some(3)),
            "mn_headwaters",
            "original green grass retains its native cache"
        );
    }
    assert!(rendered.grass_rustle_errors.is_empty());
    for block in [3, 0x88, melt] {
        assert!(
            maps.iter()
                .any(|map| runtime.data().maps[*map].blocks.contains(&block)),
            "all three grass stages must actually be placed"
        );
    }
    if let Some(directory) = capture {
        write_capture(&directory.join("receipt.json"), &serde_json::to_vec_pretty(&serde_json::json!({"scope":"Production CPU renderer assets; GPU and traversal require separate checks", "times": TIMES, "assertions":"Original warm prefix exact at every clock, all six native animations exact, three placed grass stages retain native bitplanes/priority and clock-lit inks", "rustles":captures})).expect("native asset receipt"));
    }
}

#[test]
fn winter_aliases_are_exact_and_green_prefix_grass_keeps_native_rustle() {
    for (winter, source, block) in WINTER {
        assert_eq!(animation_source(winter), source);
        assert_eq!(grass_cache_style(winter), Some(winter));
        assert_eq!(rustle_tileset(winter, Some(block)), winter);
        assert_eq!(rustle_tileset(winter, Some(3)), source);
        assert_eq!(rustle_tileset(winter, None), source);
        assert_eq!(
            rustle_tileset_at(winter, 2, &[3, block, 3, 3], TilePosition::new(2, 0)),
            winter
        );
        for point in [
            TilePosition::new(-1, 0),
            TilePosition::new(4, 0),
            TilePosition::new(0, 4),
        ] {
            assert_eq!(
                rustle_tileset_at(winter, 2, &[3, block, 3, 3], point),
                source
            );
        }
    }
    assert!(has_north_shore_waterfall("mn_north_shore"));
    assert!(has_north_shore_waterfall("mn_winter_north_shore"));
    for original in [
        "mn_border_pines",
        "forest",
        "mn_gunflint",
        "mn_winter_outdoor_extra",
    ] {
        assert_eq!(animation_source(original), original);
        assert_eq!(grass_cache_style(original), None);
        assert!(!has_north_shore_waterfall(original));
    }
}

#[test]
fn rustle_colors_resolve_actual_glyph_metadata_and_reject_missing_or_mixed_foregrounds() {
    let glyphs = [11; 16];
    let mut palette_map = vec![0; 12];
    palette_map[11] = 3;
    let foreground = [[190, 220, 230], [100, 150, 170], [40, 60, 70]];
    let mut foregrounds = BTreeMap::from([(11, foreground)]);
    let mut bank = vec![[[0; 3]; 4]; 8];
    bank[0][0] = [222, 255, 222];
    let day = resolve_grass_palette(&glyphs, &palette_map, &foregrounds, &bank).expect("day");
    assert_eq!(
        day,
        [bank[0][0], foreground[0], foreground[1], foreground[2]]
    );
    bank[0][0] = [123, 115, 198];
    let night = resolve_grass_palette(&glyphs, &palette_map, &foregrounds, &bank).expect("night");
    assert_ne!(day[0], night[0]);
    assert_eq!(day[1..], night[1..]);
    assert!(resolve_grass_palette(&glyphs[..15], &palette_map, &foregrounds, &bank).is_err());
    assert!(resolve_grass_palette(&glyphs, &palette_map[..11], &foregrounds, &bank).is_err());
    let mut mixed = glyphs;
    mixed[15] = 10;
    assert!(resolve_grass_palette(&mixed, &palette_map, &foregrounds, &bank).is_err());
    foregrounds.insert(10, [[1, 2, 3]; 3]);
    assert!(resolve_grass_palette(&mixed, &palette_map, &foregrounds, &bank).is_err());
}

fn fixture() -> (AssetRoot, CrystalRuntime) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_WINTER_RENDER_TEST_PACK")
            .expect("external pack with all three native winter art variants"),
    )
    .canonicalize()
    .expect("completed winter pack path");
    let assets = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &assets,
        read_loaded_verified_compiled_game_pack(&path).expect("verified winter art pack"),
    )
    .expect("actual winter runtime registration");
    for (winter, _, _) in WINTER {
        assert!(runtime.data().tilesets.contains_key(winter));
    }
    let root = runtime
        .materialize_runtime_files()
        .expect("mount actual pack art");
    (root, runtime)
}

#[test]
#[ignore = "requires the external pack containing all three winter tilesets"]
fn winter_animations_preserve_source_frame_pixels_schedules_and_registration_sets() {
    let (root, runtime) = fixture();
    let mut images = Assets::<Image>::default();
    for time in TIMES {
        for (winter, source, _) in WINTER {
            let source_art = load_tileset_art(
                &root,
                source,
                time,
                &runtime.data().tilesets[source].palette_map,
                &mut images,
            )
            .expect("native source art");
            let winter_art = load_tileset_art(
                &root,
                winter,
                time,
                &runtime.data().tilesets[winter].palette_map,
                &mut images,
            )
            .expect("winter art");
            let mut source_keys = source_art
                .animated_tiles
                .keys()
                .copied()
                .collect::<Vec<_>>();
            let mut winter_keys = winter_art
                .animated_tiles
                .keys()
                .copied()
                .collect::<Vec<_>>();
            source_keys.sort_unstable();
            winter_keys.sort_unstable();
            assert_eq!(source_keys, winter_keys, "{winter} {time}");
            if source == "mn_north_shore" {
                assert_eq!(source_keys, [0x03, 0x14, 0x20], "flower, Water, waterfall");
            } else if source == "mn_gunflint" {
                assert_eq!(
                    source_keys,
                    [0x03, 0x0c, 0x0f, 0x14],
                    "flower, paired forest trees, Water"
                );
            } else {
                assert!(
                    source_keys.is_empty(),
                    "do not silently add forest behavior"
                );
            }
            for key in source_keys {
                let a = &source_art.animated_tiles[&key];
                let b = &winter_art.animated_tiles[&key];
                assert_eq!(a.frame_ticks, b.frame_ticks);
                assert_eq!(a.phase_offset, b.phase_offset);
                assert_eq!(a.requires_forest_restless, b.requires_forest_restless);
                assert_eq!(a.cave_water_composite, b.cave_water_composite);
                assert_eq!(a.advance_on_phase_offset, b.advance_on_phase_offset);
                assert_eq!(a.additional_schedule, b.additional_schedule);
                assert_eq!(a.frames.len(), b.frames.len());
                for (native, derived) in a.frames.iter().zip(b.frames.iter()) {
                    assert_eq!(
                        images.get(native).expect("source frame").data,
                        images.get(derived).expect("winter frame").data,
                        "{winter} {time} glyph{key:02x}"
                    );
                }
                assert!(
                    a.frames.windows(2).any(|frames| images
                        .get(&frames[0])
                        .expect("first frame")
                        .data
                        != images.get(&frames[1]).expect("next frame").data),
                    "actual animation must vary"
                );
            }
        }
    }
}

fn frame_pixels<'a>(frame: &SpriteFrame, images: &'a Assets<Image>) -> &'a [u8] {
    let image = images.get(&frame.handle).expect("actual rustle frame");
    assert_eq!(
        (
            image.texture_descriptor.size.width,
            image.texture_descriptor.size.height
        ),
        (18, 9)
    );
    assert_eq!((frame.size.x, frame.size.y), (72.0, 36.0));
    image.data.as_deref().expect("retained CPU frame")
}

fn colors(pixels: &[u8]) -> BTreeMap<[u8; 3], usize> {
    let mut counts = BTreeMap::new();
    for p in pixels.as_chunks::<4>().0.iter().filter(|p| p[3] == 255) {
        *counts.entry([p[0], p[1], p[2]]).or_default() += 1;
    }
    counts
}

fn expected_colors(source: &BTreeMap<usize, usize>, palette: &Palette) -> BTreeMap<[u8; 3], usize> {
    let mut counts = BTreeMap::new();
    for (&index, &count) in source {
        *counts.entry(palette[index]).or_default() += count * 2;
    }
    counts
}

fn write_capture(path: &Path, bytes: &[u8]) {
    use std::io::Write;

    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .expect("create fresh capture without replacing prior evidence")
        .write_all(bytes)
        .expect("retain requested rustle evidence");
}

fn capture_png(path: &Path, pixels: &[u8], width: u32, height: u32) -> String {
    let mut encoded = Vec::new();
    image::codecs::png::PngEncoder::new(&mut encoded)
        .write_image(pixels, width, height, image::ExtendedColorType::Rgba8)
        .expect("encode actual RGBA pixels without conversion");
    let decoded = image::load_from_memory_with_format(&encoded, image::ImageFormat::Png)
        .expect("decode retained PNG")
        .to_rgba8();
    assert_eq!(decoded.dimensions(), (width, height));
    assert_eq!(decoded.as_raw(), pixels, "lossless RGBA capture");
    write_capture(path, &encoded);
    format!("{:x}", Sha256::digest(&encoded))
}

fn capture_directory() -> Option<PathBuf> {
    let directory = PathBuf::from(std::env::var_os("CRYSTAL_WINTER_RUSTLE_CAPTURE_DIR")?);
    if let Some(parent) = directory
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).expect("create capture parent directory");
    }
    std::fs::create_dir(&directory).expect("requested capture directory must be fresh");
    Some(directory)
}

fn capture_rustle_frames(
    directory: &Path,
    style: &str,
    time: &str,
    frames: &[SpriteFrame; 2],
    images: &Assets<Image>,
) -> Vec<serde_json::Value> {
    let mut records = Vec::new();
    for (index, frame) in frames.iter().enumerate() {
        // Export the actual production Assets<Image> buffer, not a second OAM
        // composition. The larger view is only a nearest-neighbor enlargement.
        let pixels = frame_pixels(frame, images);
        assert_eq!(pixels.len(), 18 * 9 * 4);
        let name = format!("{style}-{time}-frame{index}.png");
        let png_sha = capture_png(&directory.join(&name), pixels, 18, 9);
        let original = RgbaImage::from_raw(18, 9, pixels.to_vec()).expect("actual RGBA extent");
        let preview =
            image::imageops::resize(&original, 288, 144, image::imageops::FilterType::Nearest);
        let preview_name = format!("{style}-{time}-frame{index}-16x.png");
        let preview_sha = capture_png(&directory.join(&preview_name), preview.as_raw(), 288, 144);
        records.push(serde_json::json!({
            "style": style, "time_bank": time, "frame": index,
            "source": "production grass_rustle_frames_for_art Assets<Image>",
            "dimensions": [18, 9], "format": "RGBA8", "sprite_size": [frame.size.x, frame.size.y],
            "rgba_sha256": format!("{:x}", Sha256::digest(pixels)),
            "png": name, "png_sha256": png_sha,
            "nearest_preview": preview_name, "nearest_preview_sha256": preview_sha,
            "visible_pixels": pixels.as_chunks::<4>().0.iter().filter(|pixel| pixel[3] == 255).count(),
        }));
    }
    records
}

#[test]
#[ignore = "requires the external pack containing all three winter tilesets"]
fn winter_rustle_uses_authored_colors_preserves_native_oam_and_retains_green_prefix_cache() {
    let (root, runtime) = fixture();
    let capture = capture_directory();
    let mut captures = Vec::new();
    let mut rendered = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    let raw =
        crate::open_runtime_image(root.runtime_assets().join("gfx/overworld/grass_rustle.png"))
            .expect("native rustle glyph")
            .to_rgba8();
    assert_eq!(raw.dimensions(), (8, 8));
    let zero = raw.get_pixel(0, 0);
    let mut source = BTreeMap::new();
    for pixel in raw.pixels().filter(|p| *p != zero) {
        *source.entry(palette_index_from_gray(pixel[0])).or_default() += 1;
    }
    assert!(!source.is_empty());
    for time in TIMES {
        let native = grass_rustle_frames_for_art(&mut rendered, &root, "johto", time, &mut images)
            .expect("native baseline rustle");
        let baseline = native.each_ref().map(|f| frame_pixels(f, &images).to_vec());
        let npc = load_npc_sprite_palette_bank(&root, time).expect("native NPC palette")[6];
        for pixels in &baseline {
            assert_eq!(colors(pixels), expected_colors(&source, &npc));
        }
        if let Some(directory) = &capture {
            captures.extend(capture_rustle_frames(
                directory, "native", time, &native, &images,
            ));
        }
        for (winter, original, block) in WINTER {
            assert_eq!(grass_rustle_art_key(winter, time).0, winter);
            let palette = grass_palette(&root, winter, time)
                .expect("authored icy palette")
                .expect("winter");
            let icy = grass_rustle_frames_for_art(&mut rendered, &root, winter, time, &mut images)
                .expect("winter rustle through production loader");
            for ((native_frame, icy_frame), before) in native.iter().zip(&icy).zip(&baseline) {
                assert_ne!(native_frame.handle.id(), icy_frame.handle.id());
                let pixels = frame_pixels(icy_frame, &images);
                assert_eq!(colors(pixels), expected_colors(&source, &palette));
                assert_ne!(
                    colors(pixels),
                    colors(before),
                    "must distinguish icy from green"
                );
                assert!(
                    pixels
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .zip(before.as_chunks::<4>().0)
                        .all(|(icy, native)| icy[3] == native[3]),
                    "every OAM opacity position is retained"
                );
                assert!(
                    pixels
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .all(|p| matches!(p[3], 0 | 255))
                );
            }
            if let Some(directory) = &capture {
                captures.extend(capture_rustle_frames(
                    directory, winter, time, &icy, &images,
                ));
            }
            // Green grass in a winter alias must return to the original cache.
            let green_block = if matches!(winter, "mn_winter_outdoor" | "mn_winter_gunflint") {
                0x37
            } else {
                3
            };
            assert!(
                runtime.data().tilesets[winter].collision[&format!("{green_block:02x}")]
                    .iter()
                    .all(|kind| kind == "TALL_GRASS")
            );
            let green_id = rustle_tileset(winter, Some(green_block));
            assert_eq!(green_id, original);
            let green =
                grass_rustle_frames_for_art(&mut rendered, &root, green_id, time, &mut images)
                    .expect("unchanged prefix grass rustle");
            for ((a, b), before) in native.iter().zip(&green).zip(&baseline) {
                assert_eq!(a.handle.id(), b.handle.id());
                assert_eq!(frame_pixels(b, &images), before);
            }
            assert_eq!(rustle_tileset(winter, Some(block)), winter);
            let again =
                grass_rustle_frames_for_art(&mut rendered, &root, winter, time, &mut images)
                    .expect("repeat cached winter rustle");
            for (a, b) in icy.iter().zip(&again) {
                assert_eq!(a.handle.id(), b.handle.id());
            }
        }
    }
    assert_eq!(
        rendered.grass_rustle_cache.len(),
        TIMES.len() * (WINTER.len() + 1)
    );
    assert!(rendered.grass_rustle_errors.is_empty());
    if let Some(directory) = capture {
        assert_eq!(captures.len(), TIMES.len() * (WINTER.len() + 1) * 2);
        let source_png =
            crate::read_runtime_asset(root.runtime_assets().join("gfx/overworld/grass_rustle.png"))
                .expect("actual native source PNG bytes");
        write_capture(
            &directory.join("native-source-grass-rustle.png"),
            &source_png,
        );
        let pack_path = PathBuf::from(
            std::env::var_os("CRYSTAL_WINTER_RENDER_TEST_PACK").expect("fixture pack path"),
        );
        let metadata = serde_json::json!({
            "status": "PRODUCTION_RENDERER_FRAME_ASSETS_NOT_GPU_OR_CONTROLLER_CAPTURE",
            "pack_sha256": format!("{:x}", Sha256::digest(std::fs::read(pack_path).expect("fixture pack bytes"))),
            "source_png_sha256": format!("{:x}", Sha256::digest(&source_png)),
            "test_source_sha256": format!("{:x}", Sha256::digest(include_bytes!("winter_render_behavior_tests.rs"))),
            "assertions": "Authored colors, native OAM opacity, original green cache and winter cache checks all passed.",
            "frames": captures,
        });
        write_capture(
            &directory.join("receipt.json"),
            &serde_json::to_vec_pretty(&metadata).expect("serialize capture metadata"),
        );
    }
}

#[test]
#[ignore = "requires CRYSTAL_OUTDOOR_STAIR_ANIMATION_PACK with all four outdoor stair aliases and R10 rock stairs"]
fn outdoor_stair_aliases_preserve_native_animation_and_icy_rustle_pixels() {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_OUTDOOR_STAIR_ANIMATION_PACK")
            .expect("completed native stair pack"),
    );
    let assets = AssetRoot::new(path.parent().expect("stair pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &assets,
        read_loaded_verified_compiled_game_pack(&path).expect("verified native stair pack"),
    )
    .expect("stair runtime");
    let root = runtime
        .materialize_runtime_files()
        .expect("actual native alias art");
    let mut images = Assets::<Image>::default();
    let mut rendered = RenderedTilesetArt::default();
    for time in TIMES {
        for (alias, source) in [
            ("mn_outdoor_johto_stairs", "johto"),
            ("mn_outdoor_modern_stairs", "johto_modern"),
            ("mn_outdoor_headwaters_stairs", "mn_headwaters"),
            ("mn_outdoor_winter_stairs", "mn_winter_north_shore"),
            ("mn_r10_rockstairs", "johto"),
        ] {
            let original = load_tileset_art(
                &root,
                source,
                time,
                &runtime.data().tilesets[source].palette_map,
                &mut images,
            )
            .expect("source animation art");
            let cloned = load_tileset_art(
                &root,
                alias,
                time,
                &runtime.data().tilesets[alias].palette_map,
                &mut images,
            )
            .expect("stair alias animation art");
            assert_eq!(
                original.animated_tiles.len(),
                cloned.animated_tiles.len(),
                "{alias} {time} must retain every native animation"
            );
            assert!(
                original.animated_tiles.contains_key(&0x14),
                "actual water animation is required"
            );
            assert!(
                original.animated_tiles.contains_key(&0x03),
                "actual flower animation is required"
            );
            for (glyph, a) in &original.animated_tiles {
                let b = &cloned.animated_tiles[glyph];
                assert_eq!(a.frame_ticks, b.frame_ticks);
                assert_eq!(a.phase_offset, b.phase_offset);
                assert_eq!(a.requires_forest_restless, b.requires_forest_restless);
                assert_eq!(a.cave_water_composite, b.cave_water_composite);
                assert_eq!(a.advance_on_phase_offset, b.advance_on_phase_offset);
                assert_eq!(a.additional_schedule, b.additional_schedule);
                assert_eq!(a.frames.len(), b.frames.len());
                for (before, after) in a.frames.iter().zip(b.frames.iter()) {
                    assert_eq!(
                        images.get(before).expect("native animation frame").data,
                        images.get(after).expect("alias animation frame").data,
                        "{alias} {time} glyph{glyph:02x}"
                    );
                }
            }
            if alias == "mn_outdoor_winter_stairs" {
                assert!(has_north_shore_waterfall(alias));
                assert_eq!(
                    grass_palette(&root, alias, time).expect("alias icy palette"),
                    grass_palette(&root, source, time).expect("source icy palette")
                );
                let icy =
                    grass_rustle_frames_for_art(&mut rendered, &root, alias, time, &mut images)
                        .expect("actual icy alias rustle");
                let original_icy =
                    grass_rustle_frames_for_art(&mut rendered, &root, source, time, &mut images)
                        .expect("actual original icy rustle");
                for (a, b) in icy.iter().zip(&original_icy) {
                    assert_eq!(frame_pixels(a, &images), frame_pixels(b, &images));
                }
                let green_id = rustle_tileset(alias, Some(3));
                let original_green_id = rustle_tileset(source, Some(3));
                assert_eq!(green_id, original_green_id);
                let green =
                    grass_rustle_frames_for_art(&mut rendered, &root, green_id, time, &mut images)
                        .expect("green prefix rustle");
                assert_ne!(
                    frame_pixels(&icy[0], &images),
                    frame_pixels(&green[0], &images),
                    "icy grass must not reuse the green rustle"
                );
            }
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINTER_RENDER_TEST_PACK with both Iron Descent aliases"]
fn iron_descent_keeps_warm_prefix_and_corner_grass_masks_at_every_clock() {
    use super::super::load_tileset_palette_bank;

    let (root, runtime) = fixture();
    let mut images = Assets::<Image>::default();
    for time in TIMES {
        let source = load_tileset_art(
            &root,
            "mn_r10_rockstairs",
            time,
            &runtime.data().tilesets["mn_r10_rockstairs"].palette_map,
            &mut images,
        )
        .expect("native descent source");
        let art = load_tileset_art(
            &root,
            "mn_iron_descent_thaw",
            time,
            &runtime.data().tilesets["mn_iron_descent_thaw"].palette_map,
            &mut images,
        )
        .expect("installed descent art");
        assert_eq!(
            art.metatile_layout[..source.metatile_layout.len()],
            source.metatile_layout
        );
        for &glyph in &source.metatile_layout {
            let index = usize::from(glyph);
            assert_eq!(
                art.transition_tiles[index].indices,
                source.transition_tiles[index].indices
            );
            assert_eq!(
                art.transition_tiles[index].palette,
                source.transition_tiles[index].palette
            );
            assert_eq!(
                images
                    .get(&art.tile_handles[index])
                    .expect("warm pixels")
                    .data,
                images
                    .get(&source.tile_handles[index])
                    .expect("native pixels")
                    .data
            );
            assert_eq!(
                images
                    .get(&art.priority_tile_handles[index])
                    .expect("warm mask")
                    .data,
                images
                    .get(&source.priority_tile_handles[index])
                    .expect("native mask")
                    .data
            );
        }
        assert!(!source.animated_tiles.is_empty());
        assert_eq!(art.animated_tiles.len(), source.animated_tiles.len());
        for (glyph, original) in &source.animated_tiles {
            let derived = &art.animated_tiles[glyph];
            assert_eq!(derived.frame_ticks, original.frame_ticks);
            assert_eq!(derived.phase_offset, original.phase_offset);
            assert_eq!(derived.additional_schedule, original.additional_schedule);
            assert_eq!(derived.frames.len(), original.frames.len());
            for (a, b) in original.frames.iter().zip(derived.frames.iter()) {
                assert_eq!(
                    images.get(a).expect("native animation").data,
                    images.get(b).expect("derived animation").data
                );
            }
        }
        let palette = grass_palette(&root, "mn_iron_descent_thaw", time)
            .expect("native icy palette")
            .expect("registered icy grass");
        let bank = load_tileset_palette_bank(&root, "mn_r10_rockstairs", time)
            .expect("native clock")
            .expect("native bank");
        for (original_block, cold, melt) in
            [(0x81, 0xa5, 0xc9), (0x83, 0xa7, 0xcb), (0x84, 0xa8, 0xcc)]
        {
            let collision = &runtime.data().tilesets["mn_r10_rockstairs"].collision
                [&format!("{original_block:02x}")];
            for block in [cold, melt] {
                assert_eq!(
                    rustle_tileset("mn_iron_descent_thaw", Some(block)),
                    "mn_iron_descent_thaw"
                );
                for index in 0..16 {
                    if collision[(index / 8) * 2 + (index % 4) / 2] != "TALL_GRASS" {
                        continue;
                    }
                    let before = usize::from(source.metatile_layout[original_block * 16 + index]);
                    let after = usize::from(art.metatile_layout[usize::from(block) * 16 + index]);
                    assert_eq!(
                        art.transition_tiles[after].indices,
                        source.transition_tiles[before].indices,
                        "every half-grass blade and priority hole stays native"
                    );
                    assert_eq!(art.transition_tiles[after].palette[1..], palette[1..]);
                    if block == melt {
                        assert_eq!(art.transition_tiles[after].palette[0], bank[0][0]);
                    }
                    let a = images
                        .get(&art.priority_tile_handles[after])
                        .expect("icy priority")
                        .data
                        .as_deref()
                        .expect("icy bytes");
                    let b = images
                        .get(&source.priority_tile_handles[before])
                        .expect("source priority")
                        .data
                        .as_deref()
                        .expect("source bytes");
                    assert!(
                        a.as_chunks::<4>()
                            .0
                            .iter()
                            .zip(b.as_chunks::<4>().0)
                            .all(|(a, b)| a[3] == b[3])
                    );
                }
            }
            assert_eq!(
                rustle_tileset("mn_iron_descent_thaw", Some(original_block as u16)),
                "johto"
            );
        }
        assert_eq!(rustle_tileset("mn_iron_descent_thaw", Some(3)), "johto");
        assert_eq!(animation_source("mn_iron_descent_thaw"), "johto");
    }
}
