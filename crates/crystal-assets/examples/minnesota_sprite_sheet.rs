//! Convert original family sheets into native front/back frames and a review.
#[allow(dead_code)]
#[path = "../src/minnesota_pokemon_pixel_art.rs"]
mod minnesota_pokemon_pixel_art;
#[path = "../src/minnesota_sprite_archive.rs"]
mod minnesota_sprite_archive;
#[path = "../src/minnesota_sprite_refinement.rs"]
mod minnesota_sprite_refinement;
#[allow(dead_code)]
#[path = "../src/minnesota_sprite_sheet.rs"]
mod minnesota_sprite_sheet;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use minnesota_pokemon_pixel_art::{colored, png_rgb};
use minnesota_sprite_archive::write_render;
use minnesota_sprite_sheet::{
    ExteriorContour, MinnesotaSheet, SpriteConversionOptions, SpriteCrop, SpriteSampling,
    SpriteView,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
struct Registry {
    families: Vec<FamilyRef>,
}

#[derive(Deserialize)]
struct FamilyRef {
    id: String,
    manifest: String,
}

#[derive(Deserialize)]
struct FamilyManifest {
    family_id: String,
    sprites: Vec<SpriteImport>,
}

#[derive(Deserialize)]
struct SpriteImport {
    id: String,
    species_id: String,
    view: SpriteView,
    source: String,
    crop: [usize; 4],
    fit: [usize; 2],
    normal_palette_rgb555: [u16; 4],
    #[serde(default)]
    source_palette_rgb555: Option<[u16; 4]>,
    white_detail_min_samples: Option<u16>,
    interior_black_min_samples: Option<u16>,
    mirror_horizontal: Option<bool>,
    #[serde(default)]
    exterior_contour: ExteriorContour,
    #[serde(default)]
    sampling: SpriteSampling,
    #[serde(default)]
    pixel_edits: Vec<minnesota_sprite_refinement::PixelEdit>,
}

fn owned_path(root: &Path, value: &str) -> Result<PathBuf> {
    ensure!(
        value.starts_with("sprites/") && !value.contains(".."),
        "art input must remain in owned sprites directory"
    );
    Ok(root.join(value))
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    ensure!(
        args.is_empty()
            || (args.len() == 2 && matches!(args[0].as_str(), "--family" | "--manifest"))
            || (args.len() == 4 && args[0] == "--manifest" && args[2] == "--output-dir"),
        "usage: cargo run -p crystal-assets --example minnesota_sprite_sheet -- [--family MN_F001 | --manifest sprites/mn-f001/manifest.json [--output-dir target/proposal-art]]"
    );
    let selected = if args.first().is_some_and(|arg| arg == "--family") {
        args.get(1)
    } else {
        None
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let directory = root.join("planning/minnesota/pokemon");
    let proposal_output = args.get(3).map(|value| root.join(value));
    if let Some(output) = &proposal_output {
        ensure!(
            output.starts_with(root.join("target"))
                && !output
                    .components()
                    .any(|component| matches!(component, std::path::Component::ParentDir)),
            "proposal output must remain inside this checkout's ignored target directory"
        );
    }
    let registry: Registry = if args.first().is_some_and(|arg| arg == "--manifest") {
        let manifest_path = &args[1];
        let manifest: FamilyManifest =
            serde_json::from_slice(&fs::read(owned_path(&directory, manifest_path)?)?)?;
        Registry {
            families: vec![FamilyRef {
                id: manifest.family_id,
                manifest: manifest_path.clone(),
            }],
        }
    } else {
        serde_json::from_slice(&fs::read(directory.join("sprite-generation.json"))?)?
    };
    let mut converted = 0;
    for family in &registry.families {
        if selected.is_some_and(|id| id != &family.id) {
            continue;
        }
        ensure!(
            family.id.len() == 7
                && family.id.starts_with("MN_F")
                && family.id[4..].bytes().all(|c| c.is_ascii_digit()),
            "invalid family identity"
        );
        let output = proposal_output.clone().unwrap_or_else(|| {
            root.join("target/minnesota-pokemon/native")
                .join(&family.id)
        });
        fs::create_dir_all(&output)?;
        let manifest_bytes = fs::read(owned_path(&directory, &family.manifest)?)?;
        write_render(&output.join("source-manifest.json"), &manifest_bytes)?;
        let metadata: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
        if let Some(prompt) = metadata["prompt_file"].as_str() {
            write_render(
                &output.join("source-prompt.txt"),
                &fs::read(owned_path(&directory, prompt)?)?,
            )?;
        }
        if let Some(prompt) = metadata["back_prompt_file"].as_str() {
            write_render(
                &output.join("source-back-prompt.txt"),
                &fs::read(owned_path(&directory, prompt)?)?,
            )?;
        }
        if let Some(boards) = metadata["style_reference"]["boards"].as_array() {
            for (index, board) in boards.iter().enumerate() {
                let board = board.as_str().context("reference path")?;
                ensure!(
                    board.starts_with("target/minnesota-pokemon/gen2-reference/")
                        && !board.contains(".."),
                    "reference outside owned directory"
                );
                write_render(
                    &output.join(format!("source-reference-{index}.png")),
                    &fs::read(root.join(board))?,
                )?;
            }
        }
        let manifest: FamilyManifest = serde_json::from_slice(&manifest_bytes)?;
        ensure!(manifest.family_id == family.id, "family identity mismatch");
        let species: BTreeSet<&str> = manifest
            .sprites
            .iter()
            .map(|s| s.species_id.as_str())
            .collect();
        ensure!(
            (1..=3).contains(&species.len()),
            "one evolution family required"
        );
        let width = species.len() * 56;
        let mut review = vec![[255_u8; 3]; width * 112];
        let mut seen = BTreeSet::new();
        for sprite in &manifest.sprites {
            ensure!(
                !sprite.id.is_empty()
                    && sprite
                        .id
                        .bytes()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-'),
                "invalid sprite filename"
            );
            ensure!(
                seen.insert((&sprite.species_id, sprite.view.suffix())),
                "duplicate species view"
            );
            let source_bytes = fs::read(owned_path(&directory, &sprite.source)?)?;
            let source_hash = format!("{:x}", Sha256::digest(&source_bytes));
            let source_archive = write_render(
                &output.join(format!("source-{source_hash}.png")),
                &source_bytes,
            )?;
            let image = MinnesotaSheet::from_png(&source_bytes)?;
            let [x, y, width, height] = sprite.crop;
            let crop = SpriteCrop {
                x,
                y,
                width,
                height,
                fit_width: sprite.fit[0],
                fit_height: sprite.fit[1],
            };
            let suffix = sprite.view.suffix();
            let path = |extension: &str| output.join(format!("{}.{suffix}.{extension}", sprite.id));
            let crop_archive = write_render(&path("source-crop.png"), &image.crop_png(crop)?)?;
            let quantization_palette = sprite
                .source_palette_rgb555
                .unwrap_or(sprite.normal_palette_rgb555);
            let white_detail_min_samples = sprite.white_detail_min_samples.unwrap_or(16);
            let interior_black_min_samples = sprite.interior_black_min_samples.unwrap_or(32);
            let mut frame = image.candidate_with_conversion(
                crop,
                quantization_palette,
                sprite.view,
                SpriteConversionOptions {
                    minimum_white_samples: white_detail_min_samples,
                    minimum_black_samples: interior_black_min_samples,
                    exterior_contour: sprite.exterior_contour,
                    sampling: sprite.sampling,
                },
            )?;
            frame.palette_rgb555 = sprite.normal_palette_rgb555;
            let mirror_horizontal = sprite.mirror_horizontal.unwrap_or(false);
            if mirror_horizontal {
                frame.mirror_horizontal();
            }
            if !sprite.pixel_edits.is_empty() {
                let mut rows = frame.rows.clone();
                minnesota_sprite_refinement::apply(&mut rows, &sprite.pixel_edits)?;
                frame.set_rows(rows)?;
            }
            ensure!(frame.indices().contains(&3), "sprite lost its outline");
            let mut indexed = serde_json::to_value(&frame)?;
            if !sprite.pixel_edits.is_empty() {
                indexed["pixel_edits"] = serde_json::to_value(&sprite.pixel_edits)?;
            }
            indexed["source_sha256"] = source_hash.into();
            indexed["source_archive"] = source_archive
                .strip_prefix(&root)?
                .to_string_lossy()
                .into_owned()
                .into();
            indexed["crop_archive"] = crop_archive
                .strip_prefix(&root)?
                .to_string_lossy()
                .into_owned()
                .into();
            indexed["import_crop"] = serde_json::to_value(sprite.crop)?;
            indexed["import_fit"] = serde_json::to_value(sprite.fit)?;
            indexed["quantization_palette_rgb555"] = serde_json::to_value(quantization_palette)?;
            indexed["white_detail_min_samples"] = white_detail_min_samples.into();
            indexed["interior_black_min_samples"] = interior_black_min_samples.into();
            indexed["mirror_horizontal"] = mirror_horizontal.into();
            indexed["exterior_contour"] = serde_json::to_value(sprite.exterior_contour)?;
            write_render(&path("json"), &serde_json::to_vec_pretty(&indexed)?)?;
            write_render(&path("png"), &frame.png(sprite.normal_palette_rgb555, 1)?)?;
            write_render(&path("2bpp"), &frame.bitplanes())?;
            write_render(
                &path("review.png"),
                &frame.png(sprite.normal_palette_rgb555, 8)?,
            )?;
            let column = species
                .iter()
                .position(|id| *id == sprite.species_id)
                .context("species column")?;
            let canvas = sprite.view.canvas();
            let left = column * 56 + (56 - canvas) / 2;
            let top = if sprite.view == SpriteView::Back {
                60
            } else {
                0
            };
            let rgb = colored(frame.indices(), sprite.normal_palette_rgb555);
            let sheet_width = species.len() * 56;
            for y in 0..canvas {
                for x in 0..canvas {
                    review[(top + y) * sheet_width + left + x] = rgb[y * canvas + x];
                }
            }
            println!(
                "{} {suffix}: {canvas}x{canvas}, {} native bytes",
                sprite.species_id,
                frame.bitplanes().len()
            );
            converted += 1;
        }
        let width = species.len() * 56;
        write_render(
            &output.join("line.native.png"),
            &png_rgb(width, 112, &review)?,
        )?;
        let mut enlarged = Vec::with_capacity(width * 112 * 64);
        for y in 0..112 * 8 {
            for x in 0..width * 8 {
                enlarged.push(review[(y / 8) * width + x / 8]);
            }
        }
        write_render(
            &output.join("line.review.png"),
            &png_rgb(width * 8, 112 * 8, &enlarged)?,
        )?;
    }
    ensure!(converted > 0, "no matching authored family");
    println!(
        "Converted {converted} authored views; animation and pack registration remain separate."
    );
    Ok(())
}
