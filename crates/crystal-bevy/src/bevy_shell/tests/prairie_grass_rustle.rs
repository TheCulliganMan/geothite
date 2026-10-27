//! Regional grass rustle must use the actual terrain palette without changing OAM.
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::asset::Assets;
use bevy::image::Image;
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_runtime::CrystalRuntime;
use image::ImageEncoder;
use sha2::{Digest, Sha256};

use super::super::{
    RenderedTilesetArt, SpriteFrame, grass_rustle_art_key, grass_rustle_frames_for_art,
    load_npc_sprite_palette_bank, load_tileset_palette_bank, palette_index_from_gray,
};

const PRAIRIE_TILESETS: [&str; 2] = ["mn_prairie_kanto", "mn_prairie_headwaters"];
const TIMES: [&str; 4] = ["morn", "day", "nite", "dark"];

type ColorCounts = BTreeMap<[u8; 3], usize>;

fn fixture() -> (AssetRoot, CrystalRuntime) {
    let path = std::env::var_os("CRYSTAL_PRAIRIE_GRASS_RUSTLE_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/minnesota-cities-review.crystalpack")
        })
        .canonicalize()
        .expect("completed external pack containing both golden prairie variants");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified prairie rustle fixture"),
    )
    .expect("runtime registers the actual retained pack assets");
    for tileset in PRAIRIE_TILESETS {
        assert!(runtime.data().tilesets.contains_key(tileset));
    }
    let root = runtime
        .materialize_runtime_files()
        .expect("owned mount of the pack's actual presentation files");
    (root, runtime)
}

fn source_index_counts(root: &AssetRoot) -> BTreeMap<usize, usize> {
    let path = root.runtime_assets().join("gfx/overworld/grass_rustle.png");
    let source = crate::open_runtime_image(&path)
        .expect("actual native rustle PNG")
        .to_rgba8();
    assert_eq!(source.dimensions(), (8, 8));
    let transparent = source.get_pixel(0, 0);
    let mut counts = BTreeMap::new();
    for pixel in source.pixels().filter(|pixel| *pixel != transparent) {
        *counts.entry(palette_index_from_gray(pixel[0])).or_default() += 1;
    }
    assert!(
        !counts.is_empty(),
        "fixture must exercise visible rustle pixels"
    );
    counts
}

fn expected_colors(source: &BTreeMap<usize, usize>, palette: &[[u8; 3]; 4]) -> ColorCounts {
    let mut colors = BTreeMap::new();
    for (&index, &count) in source {
        // Each native OAM frame displays the retained source piece twice.
        *colors.entry(palette[index]).or_default() += count * 2;
    }
    colors
}

fn frame_pixels<'a>(frame: &SpriteFrame, images: &'a Assets<Image>) -> &'a [u8] {
    let image = images.get(&frame.handle).expect("loaded rustle image");
    let extent = image.texture_descriptor.size;
    assert_eq!(
        (extent.width, extent.height, extent.depth_or_array_layers),
        (18, 9, 1)
    );
    assert_eq!((frame.size.x, frame.size.y), (72.0, 36.0));
    let pixels = image.data.as_deref().expect("CPU rustle image pixels");
    assert_eq!(pixels.len(), 18 * 9 * 4);
    assert!(
        pixels
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| matches!(pixel[3], 0 | 255))
    );
    pixels
}

fn visible_colors(pixels: &[u8]) -> ColorCounts {
    let mut colors = BTreeMap::new();
    for pixel in pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| pixel[3] == 255)
    {
        *colors.entry([pixel[0], pixel[1], pixel[2]]).or_default() += 1;
    }
    colors
}

fn capture_frames(style: &str, time: &str, frames: &[SpriteFrame; 2], images: &Assets<Image>) {
    let Some(directory) = std::env::var_os("CRYSTAL_PRAIRIE_GRASS_RUSTLE_CAPTURE_DIR") else {
        return;
    };
    let directory = PathBuf::from(directory);
    std::fs::create_dir_all(&directory).expect("create requested rustle capture directory");
    for (index, frame) in frames.iter().enumerate() {
        let pixels = frame_pixels(frame, images);
        let mut encoded = Vec::new();
        image::codecs::png::PngEncoder::new(&mut encoded)
            .write_image(pixels, 18, 9, image::ExtendedColorType::Rgba8)
            .expect("encode actual emitted 18x9 RGBA pixels without conversion");
        let digest = format!("{:x}", Sha256::digest(&encoded));
        let path = directory.join(format!("{style}-{time}-frame{index}-{digest}.png"));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut output) => output
                .write_all(&encoded)
                .expect("write original rustle capture"),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                assert_eq!(
                    std::fs::read(&path).expect("existing capture"),
                    encoded,
                    "immutable capture path must contain exactly the same encoded pixels"
                );
            }
            Err(error) => panic!("create original rustle capture {}: {error}", path.display()),
        }
    }
}

#[test]
fn prairie_grass_rustle_keys_separate_exact_regional_aliases_and_normalize_time() {
    for tileset in PRAIRIE_TILESETS {
        assert_eq!(
            grass_rustle_art_key(tileset, "morning"),
            (tileset.to_string(), "morn".to_string())
        );
        assert_eq!(
            grass_rustle_art_key(tileset, "night"),
            (tileset.to_string(), "nite".to_string())
        );
    }
    for tileset in ["johto", "kanto", "mn_bemidji", "mn_prairie_kanto_extra"] {
        assert_eq!(
            grass_rustle_art_key(tileset, "DAY"),
            ("native".to_string(), "day".to_string())
        );
    }
}

#[test]
#[ignore = "requires the external pack with both golden prairie tilesets"]
fn prairie_grass_rustle_actual_pixels_keep_oam_and_palette_across_time_and_map_switches() {
    let (root, _runtime) = fixture();
    let source = source_index_counts(&root);
    let mut rendered = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    for time in TIMES {
        let native_palette = load_npc_sprite_palette_bank(&root, time).expect("native NPC bank")[6];
        let native = grass_rustle_frames_for_art(&mut rendered, &root, "johto", time, &mut images)
            .expect("native grass rustle");
        capture_frames("native", time, &native, &images);
        let native_pixels = native
            .each_ref()
            .map(|frame| frame_pixels(frame, &images).to_vec());
        for pixels in &native_pixels {
            assert_eq!(
                visible_colors(pixels),
                expected_colors(&source, &native_palette)
            );
        }
        for tileset in PRAIRIE_TILESETS {
            let palette = load_tileset_palette_bank(&root, tileset, time)
                .expect("actual regional terrain palette")
                .expect("gold terrain bank")[7];
            let expected = expected_colors(&source, &palette);
            assert_ne!(
                expected,
                expected_colors(&source, &native_palette),
                "fixture must distinguish golden grass from native rustle at {time}"
            );
            let golden =
                grass_rustle_frames_for_art(&mut rendered, &root, tileset, time, &mut images)
                    .expect("golden grass rustle");
            capture_frames(tileset, time, &golden, &images);
            for ((frame, native_frame), native_pixels) in
                golden.iter().zip(&native).zip(&native_pixels)
            {
                assert_ne!(frame.handle.id(), native_frame.handle.id());
                let pixels = frame_pixels(frame, &images);
                assert_eq!(visible_colors(pixels), expected, "{tileset} {time}");
                assert!(
                    pixels
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .zip(native_pixels.as_chunks::<4>().0.iter())
                        .all(|(gold, native)| gold[3] == native[3]),
                    "regional color must preserve every OAM/transparency position"
                );
            }
            let repeated =
                grass_rustle_frames_for_art(&mut rendered, &root, tileset, time, &mut images)
                    .expect("reused regional cache entry");
            for (first, again) in golden.iter().zip(&repeated) {
                assert_eq!(first.handle.id(), again.handle.id());
            }
        }
        let returned =
            grass_rustle_frames_for_art(&mut rendered, &root, "kanto", time, &mut images)
                .expect("return to native grass after both regional maps");
        for ((first, again), before) in native.iter().zip(&returned).zip(&native_pixels) {
            assert_eq!(
                first.handle.id(),
                again.handle.id(),
                "shared native cache identity"
            );
            assert_eq!(frame_pixels(again, &images), before);
        }
    }
    assert_eq!(rendered.grass_rustle_cache.len(), TIMES.len() * 3);
    assert!(rendered.grass_rustle_errors.is_empty());
}

#[test]
#[ignore = "requires the external pack with both golden prairie tilesets"]
fn prairie_grass_rustle_failed_style_does_not_poison_other_palette_caches() {
    let (root, _runtime) = fixture();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let missing_path = std::env::temp_dir().join(format!(
        "geothite-missing-rustle-{}-{unique}",
        std::process::id()
    ));
    assert!(!missing_path.exists());
    let missing = AssetRoot::new(missing_path);
    let mut rendered = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    assert!(
        grass_rustle_frames_for_art(&mut rendered, &missing, "johto", "day", &mut images,)
            .is_none(),
        "actual missing source image records a native-style error"
    );
    let native_key = grass_rustle_art_key("johto", "day");
    assert!(rendered.grass_rustle_errors.contains_key(&native_key));
    let golden = grass_rustle_frames_for_art(
        &mut rendered,
        &root,
        PRAIRIE_TILESETS[0],
        "day",
        &mut images,
    )
    .expect("native-style error must not hide actual golden assets");
    assert!(
        grass_rustle_frames_for_art(
            &mut rendered,
            &missing,
            PRAIRIE_TILESETS[1],
            "day",
            &mut images,
        )
        .is_none(),
        "the second regional alias has its own actual error entry"
    );
    let regional_key = grass_rustle_art_key(PRAIRIE_TILESETS[1], "day");
    assert_ne!(native_key, regional_key);
    assert!(rendered.grass_rustle_errors.contains_key(&regional_key));
    let repeated = grass_rustle_frames_for_art(
        &mut rendered,
        &root,
        PRAIRIE_TILESETS[0],
        "day",
        &mut images,
    )
    .expect("other style failures must not evict valid golden frames");
    for (first, again) in golden.iter().zip(&repeated) {
        assert_eq!(first.handle.id(), again.handle.id());
    }
    assert_eq!(rendered.grass_rustle_cache.len(), 1);
    assert_eq!(rendered.grass_rustle_errors.len(), 2);
}
