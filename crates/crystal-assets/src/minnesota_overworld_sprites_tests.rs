use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::minnesota_pokemon_pixel_art::bitplanes;
use crate::minnesota_sprite_sheet::MinnesotaSheet;

use super::{
    FRAMES, OverworldFrameSource, OverworldManifest, OverworldPose, OverworldSampling,
    OverworldSpeciesSource, PIXELS, POSES, Transform, decode_overworld_bitplanes, palette_bytes,
    pixel_bounds, png_rgba, rgb555, sample_frame, sha256, validate_frame,
};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct SourceDirectory(PathBuf);
impl SourceDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "geothite-overworld-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("create isolated source directory");
        Self(path)
    }
}
impl Drop for SourceDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fixture() -> (SourceDirectory, OverworldManifest) {
    let directory = SourceDirectory::new();
    let mut pixels = vec![0; 120 * 20 * 4];
    for frame in 0..FRAMES {
        for y in 3..=15 {
            for x in 4..=14 {
                let offset = (y * 120 + frame * 20 + x) * 4;
                let rgb = if x == 4 || x == 14 || y == 3 || y == 15 {
                    [0, 0, 0]
                } else {
                    [0, 255, 0]
                };
                pixels[offset..offset + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 253]);
            }
        }
        for (x, y, color) in [
            (8, 8, [0, 0, 0, 0]), // Enclosed transparent ink must remain transparent.
            (10, 8, [255, 255, 255, 253]), // Source white must never become ink zero.
            (2, 3, [0, 0, 0, 31]), // Faint generated halo is discarded.
            (5 + frame, 16, [0, 0, 0, 253]), // A different stepping foot per frame.
            (6 + frame, 10, [255, 0, 0, 253]),
        ] {
            let offset = (y * 120 + frame * 20 + x) * 4;
            pixels[offset..offset + 4].copy_from_slice(&color);
        }
    }
    fs::write(
        directory.0.join("source.png"),
        png_rgba(120, 20, &pixels).expect("encode fixture"),
    )
    .expect("write source");
    let source = OverworldSpeciesSource {
        species_id: "MN_001".into(),
        runtime_token: "TESTMON".into(),
        palette_rgb555: [32767, 992, 31, 0],
        shiny_palette_rgb555: [32767, 31744, 1023, 0],
        fit: [14, 14],
        alpha_threshold: 128,
        sampling: OverworldSampling::AreaVote,
        minimum_foreground_samples: 16,
        black_priority_samples: 8,
        interior_black_samples: None,
        opaque_interior_minimum_samples: 60,
        frames: std::array::from_fn(|frame| OverworldFrameSource {
            pose: POSES[frame],
            source: "source.png".into(),
            crop: [frame * 20, 0, 20, 20],
        }),
    };
    (
        directory,
        OverworldManifest {
            schema_version: 1,
            family_id: "MN_F001".into(),
            species: vec![source],
        },
    )
}

#[test]
fn overworld_tiles_use_native_frame_and_row_major_tile_order() {
    let mut bytes = vec![0; 384];
    bytes[0] = 0x80; // Frame zero, upper left, ink one.
    bytes[16 + 1] = 0x80; // Upper right, ink two.
    bytes[32] = 0x80; // Lower left, ink three.
    bytes[33] = 0x80;
    bytes[5 * 64 + 48 + 14] = 0x01; // Last frame, lower right, last row/pixel.
    let pixels = decode_overworld_bitplanes(&bytes).expect("valid six-frame bank");
    assert_eq!(pixels[0], 1);
    assert_eq!(pixels[8], 2);
    assert_eq!(pixels[8 * 16], 3);
    assert_eq!(pixels[5 * PIXELS + 15 * 16 + 15], 1);
    assert_eq!(bitplanes(&pixels, 16, 96), bytes);
    assert!(decode_overworld_bitplanes(&bytes[..383]).is_err());
}

#[test]
fn overworld_compile_preserves_literal_obj_alpha_shared_scale_and_own_palettes() {
    let (directory, manifest) = fixture();
    let compiled = manifest
        .compile(&directory.0)
        .expect("compile complete family");
    let prefix = "gfx/pokemon/testmon/overworld";
    assert_eq!(compiled.runtime_files.len(), 4);
    let bank = decode_overworld_bitplanes(&compiled.runtime_files[&format!("{prefix}.2bpp")])
        .expect("native bank");
    let sheet = MinnesotaSheet::from_png(&compiled.runtime_files[&format!("{prefix}.png")])
        .expect("literal indexed image");
    assert_eq!((sheet.width, sheet.height), (16, 96));
    for frame in 0..FRAMES {
        let pixels = &bank[frame * PIXELS..(frame + 1) * PIXELS];
        assert_eq!(pixel_bounds(pixels), Some([2, 1, 11, 14]));
        assert_eq!(pixels[6 * 16 + 6], 0, "enclosed alpha hole");
        assert_ne!(pixels[6 * 16 + 8], 0, "source white remains visible");
        assert_eq!(sheet.alpha(6, frame * 16 + 6), Some(0));
        assert_eq!(sheet.alpha(8, frame * 16 + 6), Some(255));
        validate_frame(pixels).expect("one pixel transparent border");
    }
    assert_eq!(
        compiled.runtime_files[&format!("{prefix}.gbcpal")],
        palette_bytes(manifest.species[0].palette_rgb555)
    );
    assert_ne!(
        compiled.runtime_files[&format!("{prefix}.gbcpal")],
        compiled.runtime_files[&format!("{prefix}-shiny.gbcpal")]
    );
    let index: serde_json::Value =
        serde_json::from_slice(&compiled.review_files["compiled-index.json"]).expect("index");
    assert_eq!(
        index["species"][0]["shared_source_bounds"],
        serde_json::json!([4, 3, 11, 14])
    );
    assert_eq!(
        index["species"][0]["frames"][0]["source_sha256"]
            .as_str()
            .expect("source hash")
            .len(),
        64
    );
    assert!(
        compiled
            .review_files
            .contains_key("review/testmon.source-bounds.png")
    );
}

#[test]
fn overworld_manifest_rejects_clipping_unsafe_sources_and_unreadable_walks() {
    let (directory, mut manifest) = fixture();
    manifest.family_id = "MN_F".into();
    assert!(manifest.compile(&directory.0).is_err());
    manifest.family_id = "MN_F001".into();
    manifest.species[0].frames[0].source = "../source.png".into();
    assert!(manifest.compile(&directory.0).is_err());
    manifest.species[0].frames[0].source = "source.png".into();
    manifest.species[0].frames[3].crop = manifest.species[0].frames[0].crop;
    let error = manifest
        .compile(&directory.0)
        .expect_err("identical step rejected");
    assert!(error.to_string().contains("idle and step became identical"));
    manifest.species[0].frames[3].crop = [60, 0, 20, 20];
    for frame in &mut manifest.species[0].frames {
        frame.crop[1] = 3;
        frame.crop[3] = 17;
    }
    let error = manifest
        .compile(&directory.0)
        .expect_err("clipped body rejected");
    assert!(format!("{error:#}").contains("touch crop"));
}

#[test]
fn overworld_manifest_is_strict_and_versions_retain_previous_native_bytes() {
    let (directory, manifest) = fixture();
    let mut json = serde_json::to_value(&manifest).expect("manifest JSON");
    json["unrecognized"] = true.into();
    assert!(OverworldManifest::from_json(&serde_json::to_vec(&json).expect("JSON")).is_err());
    let mut compiled = manifest.compile(&directory.0).expect("compile");
    let output = directory.0.join("output");
    compiled.write_to(&output).expect("first write");
    let path = "gfx/pokemon/testmon/overworld.2bpp";
    let before = compiled.runtime_files[path].clone();
    compiled.runtime_files.get_mut(path).expect("bank")[0] ^= 1;
    compiled.write_to(&output).expect("versioned write");
    let parent = output.join("gfx/pokemon/testmon/versions");
    assert!(
        parent
            .join(format!("{}.overworld.2bpp", sha256(&before)))
            .is_file()
    );
    assert_eq!(
        fs::read(output.join(path)).expect("latest"),
        compiled.runtime_files[path]
    );
}

#[test]
fn overworld_black_priority_cannot_turn_faint_coverage_into_a_solid_edge() {
    let (_directory, manifest) = fixture();
    let source = &manifest.species[0];
    let mut rgba = vec![0; 8 * 8 * 4];
    for x in 0..8 {
        rgba[x * 4 + 3] = 253;
    }
    let sheet = MinnesotaSheet::from_png(&png_rgba(8, 8, &rgba).expect("source PNG"))
        .expect("source sheet");
    let frame = OverworldFrameSource {
        pose: OverworldPose::DownIdle,
        source: "unused.png".into(),
        crop: [0, 0, 8, 8],
    };
    let transform = Transform {
        bounds: [0, 0, 8, 8],
        size: [1, 1],
        origin: [7, 14],
    };
    let pixels = sample_frame(
        &sheet,
        &frame,
        source,
        &transform,
        &source.palette_rgb555.map(rgb555),
    );
    assert!(
        pixels.iter().all(|&ink| ink == 0),
        "8/64 black samples are below the 16/64 foreground threshold"
    );
}

#[test]
fn overworld_interior_detail_priority_preserves_source_black_without_expanding_silhouette() {
    let (_directory, mut manifest) = fixture();
    let source = &mut manifest.species[0];
    source.black_priority_samples = 32;
    source.interior_black_samples = Some(8);
    source.opaque_interior_minimum_samples = 60;
    let mut rgba = [0, 255, 0, 253].repeat(64);
    for x in 0..8 {
        rgba[x * 4..x * 4 + 3].copy_from_slice(&[0, 0, 0]);
    }
    let frame = OverworldFrameSource {
        pose: OverworldPose::DownIdle,
        source: "unused.png".into(),
        crop: [0, 0, 8, 8],
    };
    let transform = Transform {
        bounds: [0, 0, 8, 8],
        size: [1, 1],
        origin: [7, 14],
    };
    let render = |rgba: &[u8]| {
        let sheet =
            MinnesotaSheet::from_png(&png_rgba(8, 8, rgba).expect("source")).expect("sheet");
        sample_frame(
            &sheet,
            &frame,
            source,
            &transform,
            &source.palette_rgb555.map(rgb555),
        )[14 * 16 + 7]
    };
    assert_eq!(
        render(&rgba),
        3,
        "8 real black samples survive in a fully opaque interior"
    );
    for x in 8..16 {
        rgba[x * 4 + 3] = 0;
    }
    assert_eq!(
        render(&rgba),
        1,
        "partial silhouette uses actual foreground plurality"
    );
}

#[test]
fn overworld_manifest_rejects_invalid_palette_fit_pose_order_and_empty_source() {
    let (directory, mut manifest) = fixture();
    let normal = manifest.species[0].palette_rgb555;
    for palette in [
        [32767, 0x8000, 31, 0],
        [32767, 992, 992, 0],
        [32767, 992, 31, 1],
    ] {
        manifest.species[0].palette_rgb555 = palette;
        assert!(
            manifest.compile(&directory.0).is_err(),
            "invalid RGB555/OBJ palette"
        );
    }
    manifest.species[0].palette_rgb555 = normal;
    manifest.species[0].fit = [16, 16];
    assert!(
        manifest.compile(&directory.0).is_err(),
        "native border is mandatory"
    );
    manifest.species[0].fit = [14, 14];
    manifest.species[0].frames[0].pose = OverworldPose::UpIdle;
    assert!(
        manifest.compile(&directory.0).is_err(),
        "pose order is not inferred"
    );
    manifest.species[0].frames[0].pose = OverworldPose::DownIdle;
    fs::write(
        directory.0.join("source.png"),
        png_rgba(120, 20, &vec![0; 120 * 20 * 4]).expect("empty PNG"),
    )
    .expect("replace source");
    let error = manifest
        .compile(&directory.0)
        .expect_err("empty source rejected");
    assert!(format!("{error:#}").contains("empty foreground"));
}
