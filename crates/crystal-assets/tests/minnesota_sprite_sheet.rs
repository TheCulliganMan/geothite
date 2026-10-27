//! Verify that the authored mosquito sheet really yields native front frames.
#[allow(dead_code)]
#[path = "../src/minnesota_pokemon_pixel_art.rs"]
mod minnesota_pokemon_pixel_art;
#[path = "../src/minnesota_sprite_archive.rs"]
mod minnesota_sprite_archive;
#[allow(dead_code)]
#[path = "../src/minnesota_sprite_sheet.rs"]
mod minnesota_sprite_sheet;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use minnesota_pokemon_pixel_art::png_rgb;
use minnesota_sprite_sheet::{MinnesotaSheet, SpriteCrop, SpriteView};
use serde::Deserialize;

#[test]
fn thin_diagonal_white_wing_keeps_a_continuous_black_rim() -> Result<()> {
    let mut pixels = vec![[255_u8; 3]; 72 * 72];
    for offset in 0..57 {
        pixels[(8 + offset) * 72 + 8 + offset] = [0; 3];
        pixels[(8 + offset) * 72 + 8] = [0; 3];
        pixels[64 * 72 + 8 + offset] = [0; 3];
    }
    let sheet = MinnesotaSheet::from_png(&png_rgb(72, 72, &pixels)?)?;
    let frame = sheet.front_candidate(
        SpriteCrop {
            x: 0,
            y: 0,
            width: 72,
            height: 72,
            fit_width: 14,
            fit_height: 14,
        },
        [0x7fff, 0x2b78, 0x3caa, 0],
    )?;
    for offset in 0..14 {
        assert_eq!(
            frame.indices()[(41 + offset) * 56 + 21 + offset],
            3,
            "diagonal rim broke at {offset}"
        );
    }
    assert_eq!(frame.indices()[50 * 56 + 22], 0, "white wing was filled in");
    Ok(())
}

#[test]
fn mixed_interior_samples_keep_the_colored_limb_core() -> Result<()> {
    let palette = [0x7fff, 0x2b78, 0x3caa, 0];
    let ink = minnesota_pokemon_pixel_art::colored(&[1], palette)[0];
    let mut pixels = vec![[255_u8; 3]; 24 * 24];
    for y in 4..20 {
        for x in 4..20 {
            pixels[y * 24 + x] = if x == 4 || x == 19 || y == 4 || y == 19 || x == 10 {
                [0; 3]
            } else {
                ink
            };
        }
    }
    let sheet = MinnesotaSheet::from_png(&png_rgb(24, 24, &pixels)?)?;
    let frame = sheet.front_candidate(
        SpriteCrop {
            x: 0,
            y: 0,
            width: 24,
            height: 24,
            fit_width: 4,
            fit_height: 4,
        },
        palette,
    )?;
    assert_eq!(
        frame.indices()[52 * 56 + 27],
        1,
        "quarter-black interior sample swallowed its colored core"
    );
    assert_eq!(
        frame.indices()[51 * 56 + 26],
        3,
        "exterior rim was weakened with the interior"
    );
    let structural_seam = sheet.candidate_with_thresholds(
        SpriteCrop {
            x: 0,
            y: 0,
            width: 24,
            height: 24,
            fit_width: 4,
            fit_height: 4,
        },
        palette,
        SpriteView::Front,
        16,
        16,
    )?;
    assert_eq!(
        structural_seam.indices()[52 * 56 + 27],
        3,
        "authored thin structural seam was erased"
    );
    assert_eq!(
        structural_seam.indices()[51 * 56 + 26],
        frame.indices()[51 * 56 + 26],
        "interior setting changed the existing exterior rim"
    );
    Ok(())
}

#[test]
fn close_back_white_body_keeps_its_bottom_crop_open() -> Result<()> {
    let mut pixels = vec![[255_u8; 3]; 24 * 24];
    for y in 4..24 {
        for x in 4..21 {
            if x == 4 || x == 20 || y == 4 {
                pixels[y * 24 + x] = [0; 3];
            }
        }
    }
    let sheet = MinnesotaSheet::from_png(&png_rgb(24, 24, &pixels)?)?;
    let frame = sheet.back_candidate(
        SpriteCrop {
            x: 0,
            y: 0,
            width: 24,
            height: 24,
            fit_width: 10,
            fit_height: 10,
        },
        [0x7fff, 0x2b78, 0x3caa, 0],
    )?;
    assert_eq!(
        frame.indices()[47 * 48 + 23],
        0,
        "outline pass drew a false bottom cap"
    );
    assert_eq!(
        frame.indices()[47 * 48 + 19],
        3,
        "cropped shoulder lost its side rim"
    );
    Ok(())
}

#[test]
fn changed_and_rejected_render_bytes_remain_archived() -> Result<()> {
    let directory =
        std::env::temp_dir().join(format!("minnesota-render-archive-{}", std::process::id()));
    fs::create_dir_all(&directory)?;
    let path = directory.join("preview.png");
    fs::write(&path, b"earlier rejected render")?;
    let first = minnesota_sprite_archive::write_render(&path, b"selected render")?;
    let second = minnesota_sprite_archive::write_render(&path, b"recolored render")?;
    assert_eq!(fs::read(first)?, b"selected render");
    assert_eq!(fs::read(second)?, b"recolored render");
    assert!(fs::read_dir(directory.join("versions"))?.any(|entry| {
        entry
            .ok()
            .and_then(|e| fs::read(e.path()).ok())
            .is_some_and(|b| b == b"earlier rejected render")
    }));
    assert_eq!(fs::read(path)?, b"recolored render");
    fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn large_white_wing_retains_its_thin_black_contour() -> Result<()> {
    let mut pixels = vec![[255_u8; 3]; 24 * 24];
    for y in 2..22 {
        for x in 2..22 {
            if x == 2 || x == 21 || y == 2 || y == 21 {
                pixels[y * 24 + x] = [0; 3];
            }
        }
    }
    let sheet = MinnesotaSheet::from_png(&png_rgb(24, 24, &pixels)?)?;
    let frame = sheet.front_candidate(
        SpriteCrop {
            x: 0,
            y: 0,
            width: 24,
            height: 24,
            fit_width: 10,
            fit_height: 10,
        },
        [0x7fff, 0x2b78, 0x3caa, 0],
    )?;
    assert_eq!(
        frame.indices()[45 * 56 + 23],
        3,
        "white wing erased its outer rim"
    );
    assert_eq!(
        frame.indices()[50 * 56 + 28],
        0,
        "white wing interior disappeared"
    );
    Ok(())
}

#[derive(Deserialize)]
struct Manifest {
    sprites: Vec<Import>,
}

#[derive(Deserialize)]
struct Import {
    source: String,
    view: SpriteView,
    crop: [usize; 4],
    fit: [usize; 2],
    normal_palette_rgb555: [u16; 4],
    source_palette_rgb555: Option<[u16; 4]>,
}

#[test]
fn enclosed_eye_white_survives_downsampling_beside_a_black_rim() -> Result<()> {
    let mut pixels = vec![[255_u8; 3]; 16 * 16];
    for y in 4..12 {
        for x in 4..12 {
            pixels[y * 16 + x] = [0, 0, 0];
        }
    }
    pixels[6 * 16 + 7] = [255; 3];
    pixels[7 * 16 + 7] = [255; 3];
    let sheet = MinnesotaSheet::from_png(&png_rgb(16, 16, &pixels)?)?;
    let frame = sheet.front_candidate(
        SpriteCrop {
            x: 0,
            y: 0,
            width: 16,
            height: 16,
            fit_width: 4,
            fit_height: 4,
        },
        [0x7fff, 0x2b78, 0x3caa, 0],
    )?;
    assert_eq!(
        frame.indices()[52 * 56 + 27],
        0,
        "enclosed white eye was swallowed by its rim"
    );
    assert_eq!(
        frame.indices()[51 * 56 + 26],
        3,
        "outer black contour disappeared"
    );
    Ok(())
}

#[test]
fn authored_threshold_preserves_a_tiny_eye_without_global_changes() -> Result<()> {
    let mut pixels = vec![[255_u8; 3]; 24 * 24];
    for y in 4..20 {
        for x in 4..20 {
            pixels[y * 24 + x] = [0; 3];
        }
    }
    pixels[10 * 24 + 11] = [255; 3];
    let sheet = MinnesotaSheet::from_png(&png_rgb(24, 24, &pixels)?)?;
    let crop = SpriteCrop {
        x: 0,
        y: 0,
        width: 24,
        height: 24,
        fit_width: 4,
        fit_height: 4,
    };
    let palette = [0x7fff, 0x2b78, 0x3caa, 0];
    let ordinary = sheet.front_candidate(crop, palette)?;
    let detailed = sheet.candidate_with_highlight_threshold(crop, palette, SpriteView::Front, 4)?;
    assert_eq!(ordinary.indices()[52 * 56 + 27], 3);
    assert_eq!(detailed.indices()[52 * 56 + 27], 0);
    assert!(
        sheet
            .candidate_with_highlight_threshold(crop, palette, SpriteView::Front, 0)
            .is_err()
    );
    Ok(())
}

#[test]
fn mosquito_fronts_and_backs_preserve_palette_and_native_tile_pixels() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/pokemon");
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(root.join("sprites/mosquito-line/manifest.json"))?)?;
    assert_eq!(manifest.sprites.len(), 6);
    for import in manifest.sprites {
        let palette = import.normal_palette_rgb555;
        assert_eq!(palette[0], 0x7fff);
        assert_eq!(palette[3], 0);
        let rgb = palette.map(|word| {
            [0, 5, 10].map(|shift| {
                let channel = ((word >> shift) & 31) as u8;
                (channel << 3) | (channel >> 2)
            })
        });
        let sheet = MinnesotaSheet::from_png(&fs::read(root.join(import.source))?)?;
        let [x, y, width, height] = import.crop;
        let crop = SpriteCrop {
            x,
            y,
            width,
            height,
            fit_width: import.fit[0],
            fit_height: import.fit[1],
        };
        let quantization_palette = import.source_palette_rgb555.unwrap_or(palette);
        let mut front = match import.view {
            SpriteView::Front => sheet.front_candidate(crop, quantization_palette)?,
            SpriteView::Back => sheet.back_candidate(crop, quantization_palette)?,
        };
        front.palette_rgb555 = palette;
        let original_indices = front.indices().to_vec();
        let original_bytes = front.bitplanes();
        front.mirror_horizontal();
        for y in 0..import.view.canvas() {
            for x in 0..import.view.canvas() {
                assert_eq!(
                    front.indices()[y * import.view.canvas() + x],
                    original_indices[y * import.view.canvas() + import.view.canvas() - 1 - x]
                );
            }
        }
        front.mirror_horizontal();
        assert_eq!(
            front.bitplanes(),
            original_bytes,
            "two reflections must preserve all tile pixels"
        );
        let canvas = import.view.canvas();
        assert_eq!(front.size, [canvas, canvas]);
        for color in 0..4 {
            assert!(front.indices().contains(&color));
        }
        let bytes = front.bitplanes();
        assert_eq!(bytes.len(), (canvas / 8).pow(2) * 16);
        let png = MinnesotaSheet::from_png(&front.png(palette, 1)?)?;
        for y in 0..canvas {
            for x in 0..canvas {
                // Decode native 8x8 tiles independently of the encoder.
                let tile = match import.view {
                    SpriteView::Front => (y / 8) * (canvas / 8) + x / 8,
                    SpriteView::Back => (x / 8) * (canvas / 8) + y / 8,
                };
                let offset = tile * 16 + (y % 8) * 2;
                let bit = 7 - x % 8;
                let index = ((bytes[offset] >> bit) & 1) | (((bytes[offset + 1] >> bit) & 1) << 1);
                assert_eq!(index, front.indices()[y * canvas + x]);
                assert_eq!(png.pixel(x, y), Some(rgb[usize::from(index)]));
                if x == 0
                    || y == 0
                    || x == canvas - 1
                    || (import.view == SpriteView::Front && y == canvas - 1)
                {
                    assert_eq!(index, 0, "native frame clipped at border");
                }
            }
        }
        let bad_crop = SpriteCrop {
            x: sheet.width,
            ..crop
        };
        assert!(sheet.front_candidate(bad_crop, palette).is_err());
        if import.view == SpriteView::Back {
            assert!(
                sheet
                    .back_candidate(
                        SpriteCrop {
                            fit_width: 47,
                            ..crop
                        },
                        palette
                    )
                    .is_err()
            );
        }
        assert!(sheet.front_candidate(crop, [0; 4]).is_err());
        assert!(front.png(palette, 9).is_err());
        let [left, top, right, bottom] = front.source_bounds;
        assert!(left >= crop.x && top >= crop.y);
        assert!(right <= crop.x + crop.width && bottom <= crop.y + crop.height);
        assert!(png.pixel(canvas, 0).is_none());
        assert_eq!(front.rows.len(), canvas);
        assert_eq!(
            front.rows.first().context("first native row")?.len(),
            canvas
        );
    }
    Ok(())
}
