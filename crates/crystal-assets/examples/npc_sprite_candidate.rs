//! Convert an isolated original NPC battle source through the Pokemon pipeline.
#[allow(dead_code)]
#[path = "../src/minnesota_pokemon_pixel_art.rs"]
mod minnesota_pokemon_pixel_art;
#[allow(dead_code)]
#[path = "../src/minnesota_sprite_sheet.rs"]
mod minnesota_sprite_sheet;

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use minnesota_sprite_sheet::{
    ExteriorContour, MinnesotaSheet, SpriteConversionOptions, SpriteCrop, SpriteSampling,
    SpriteView,
};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() == 3 && args[0] == "--native" {
        return native(&args[1], &args[2]);
    }
    ensure!(
        args.len() == 6 || args.len() == 8,
        "usage: npc_sprite_candidate SOURCE OUTPUT X Y WIDTH HEIGHT [LIGHT_RGB555 MID_RGB555]"
    );
    let source = MinnesotaSheet::from_png(&fs::read(&args[0])?)?;
    let output = PathBuf::from(&args[1]);
    let values = args[2..6]
        .iter()
        .map(|v| v.parse::<usize>().context("invalid crop"))
        .collect::<Result<Vec<_>>>()?;
    let mut palette = [
        32767,
        24 | (17 << 5) | (9 << 10),
        6 | (10 << 5) | (7 << 10),
        0,
    ];
    if args.len() == 8 {
        palette[1] = args[6].parse().context("invalid light RGB555 ink")?;
        palette[2] = args[7].parse().context("invalid middle RGB555 ink")?;
    }
    let candidate = source.candidate_with_conversion(
        SpriteCrop {
            x: values[0],
            y: values[1],
            width: values[2],
            height: values[3],
            fit_width: 52,
            fit_height: 54,
        },
        palette,
        SpriteView::Front,
        SpriteConversionOptions {
            minimum_white_samples: 16,
            minimum_black_samples: 32,
            exterior_contour: ExteriorContour::SourceBacked,
            sampling: SpriteSampling::SourceCenter,
        },
    )?;
    fs::create_dir_all(&output)?;
    fs::write(
        output.join("battle.candidate.json"),
        serde_json::to_vec_pretty(&candidate)?,
    )?;
    fs::write(output.join("battle.png"), candidate.png(palette, 1)?)?;
    fs::write(output.join("battle.zoom.png"), candidate.png(palette, 8)?)?;
    fs::write(output.join("battle.2bpp"), candidate.bitplanes())?;
    fs::write(
        output.join("battle.gbcpal"),
        palette
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>(),
    )?;
    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeNpc {
    schema_version: u8,
    id: String,
    palette_rgb555: [u16; 4],
    battle_rows: Vec<String>,
    overworld_frames: Vec<NativeFrame>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeFrame {
    pose: String,
    rows: Vec<String>,
}

fn pixels(rows: &[String], size: usize) -> Result<Vec<u8>> {
    ensure!(
        rows.len() == size && rows.iter().all(|r| r.len() == size),
        "wrong native grid size"
    );
    rows.iter()
        .flat_map(|r| r.bytes())
        .map(|c| match c {
            b'.' => Ok(0),
            b'1'..=b'3' => Ok(c - b'0'),
            _ => anyhow::bail!("invalid ink"),
        })
        .collect()
}

fn verify_tiles(bytes: &[u8], expected: &[u8], width: usize, height: usize) -> Result<()> {
    ensure!(bytes.len() == width * height / 4, "wrong tile byte count");
    let mut decoded = vec![0; width * height];
    for ty in 0..height / 8 {
        for tx in 0..width / 8 {
            for y in 0..8 {
                let offset = ((ty * (width / 8) + tx) * 8 + y) * 2;
                for x in 0..8 {
                    decoded[(ty * 8 + y) * width + tx * 8 + x] = ((bytes[offset] >> (7 - x)) & 1)
                        | (((bytes[offset + 1] >> (7 - x)) & 1) << 1);
                }
            }
        }
    }
    ensure!(
        decoded == expected,
        "independent tile decode differs from native source"
    );
    Ok(())
}

fn native(source_path: &str, output: &str) -> Result<()> {
    use minnesota_pokemon_pixel_art::{bitplanes, colored, png_rgb, png_rgba};
    let spec: NativeNpc = serde_json::from_slice(&fs::read(source_path)?)?;
    ensure!(spec.schema_version == 1, "unsupported NPC schema");
    ensure!(
        !spec.id.is_empty() && spec.id.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'),
        "invalid identity"
    );
    ensure!(
        spec.palette_rgb555[0] == 32767 && spec.palette_rgb555[3] == 0,
        "native palette endpoints required"
    );
    ensure!(
        spec.palette_rgb555.iter().all(|v| *v <= 32767),
        "invalid palette"
    );
    ensure!(
        (0..4).all(|i| (0..i).all(|j| spec.palette_rgb555[i] != spec.palette_rgb555[j])),
        "duplicate palette inks"
    );
    let poses = [
        "down_idle",
        "up_idle",
        "left_idle",
        "down_step",
        "up_step",
        "left_step",
    ];
    ensure!(
        spec.overworld_frames.len() == poses.len(),
        "six native poses required"
    );
    let mut bank = Vec::new();
    for (frame, pose) in spec.overworld_frames.iter().zip(poses) {
        ensure!(frame.pose == pose, "wrong native pose order");
        let frame_pixels = pixels(&frame.rows, 16)?;
        ensure!(
            frame_pixels.iter().any(|v| *v != 0),
            "empty overworld frame"
        );
        bank.extend(frame_pixels);
    }
    for direction in 0..3 {
        ensure!(
            bank[direction * 256..(direction + 1) * 256]
                != bank[(direction + 3) * 256..(direction + 4) * 256],
            "identical walking pair"
        );
    }
    let battle = pixels(&spec.battle_rows, 56)?;
    let overworld_tiles = bitplanes(&bank, 16, 96);
    let battle_tiles = bitplanes(&battle, 56, 56);
    verify_tiles(&overworld_tiles, &bank, 16, 96)?;
    verify_tiles(&battle_tiles, &battle, 56, 56)?;
    let output = PathBuf::from(output);
    let sprites = output.join("gfx/sprites");
    let trainers = output.join("gfx/trainers");
    fs::create_dir_all(&sprites)?;
    fs::create_dir_all(&trainers)?;
    let palette_bytes: Vec<_> = spec
        .palette_rgb555
        .iter()
        .flat_map(|v| v.to_le_bytes())
        .collect();
    let grayscale: Vec<_> = bank
        .iter()
        .map(|v| {
            let g = [255, 170, 85, 0][usize::from(*v)];
            [g, g, g, if *v == 0 { 0 } else { 255 }]
        })
        .collect();
    fs::write(
        sprites.join(format!("{}.png", spec.id)),
        png_rgba(16, 96, &grayscale)?,
    )?;
    fs::write(sprites.join(format!("{}.2bpp", spec.id)), overworld_tiles)?;
    fs::write(sprites.join(format!("{}.gbcpal", spec.id)), &palette_bytes)?;
    fs::write(
        trainers.join(format!("{}.png", spec.id)),
        png_rgb(56, 56, &colored(&battle, spec.palette_rgb555))?,
    )?;
    fs::write(trainers.join(format!("{}.2bpp", spec.id)), battle_tiles)?;
    fs::write(trainers.join(format!("{}.gbcpal", spec.id)), &palette_bytes)?;
    let mut board = vec![[255; 3]; 96 * 16];
    let inks = colored(&bank, spec.palette_rgb555);
    for frame in 0..6 {
        for y in 0..16 {
            for x in 0..16 {
                board[y * 96 + frame * 16 + x] = inks[frame * 256 + y * 16 + x];
            }
        }
    }
    fs::write(
        output.join("overworld-review.png"),
        png_rgb(96, 16, &board)?,
    )?;
    let zoom: Vec<_> = (0..128)
        .flat_map(|y| (0..768).map(move |x| (y / 8) * 96 + x / 8))
        .map(|i| board[i])
        .collect();
    fs::write(
        output.join("overworld-review-8x.png"),
        png_rgb(768, 128, &zoom)?,
    )?;
    let battle_rgb = colored(&battle, spec.palette_rgb555);
    let zoom: Vec<_> = (0..448)
        .flat_map(|y| (0..448).map(move |x| (y / 8) * 56 + x / 8))
        .map(|i| battle_rgb[i])
        .collect();
    fs::write(
        output.join("battle-review-8x.png"),
        png_rgb(448, 448, &zoom)?,
    )?;
    fs::write(
        output.join("verification.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "id":spec.id,"source":source_path,"native_overworld_frames":6,"overworld_size":[16,16],
            "battle_size":[56,56],"palette_rgb555":spec.palette_rgb555,"independent_tile_roundtrip":true,
            "visual_acceptance":"pending","runtime_acceptance":"pending"
        }))?,
    )?;
    Ok(())
}
