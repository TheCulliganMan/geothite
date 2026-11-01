//! Compile authored six-pose cutouts into literal GBC OBJ pixels and tile banks.
//!
//! Ink zero is transparent everywhere, including enclosed holes. Foreground
//! source colors quantize only to inks 1..=3. All poses share a single sampling
//! transform per species; walking never independently rescales the body.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Component, Path};

use anyhow::{Context, Result, ensure};
use flate2::write::ZlibEncoder;
use flate2::{Compression, Crc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::minnesota_pokemon_pixel_art::bitplanes;
use crate::minnesota_sprite_sheet::MinnesotaSheet;

#[path = "minnesota_sprite_archive.rs"]
mod archive;

const SIZE: usize = 16;
const FRAMES: usize = 6;
const PIXELS: usize = SIZE * SIZE;

/// Stored native order. Right-facing pictures use the exact mirror of left.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverworldPose {
    DownIdle,
    UpIdle,
    LeftIdle,
    DownStep,
    UpStep,
    LeftStep,
}

const POSES: [OverworldPose; FRAMES] = [
    OverworldPose::DownIdle,
    OverworldPose::UpIdle,
    OverworldPose::LeftIdle,
    OverworldPose::DownStep,
    OverworldPose::UpStep,
    OverworldPose::LeftStep,
];

/// Interior sampling policy. Neither policy invents a contour or fills holes.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverworldSampling {
    #[default]
    AreaVote,
    SourceCenter,
}

/// One explicitly isolated source window; all six windows share dimensions.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OverworldFrameSource {
    pub pose: OverworldPose,
    pub source: String,
    pub crop: [usize; 4],
}

fn default_fit() -> [usize; 2] {
    [14, 14]
}
fn default_alpha() -> u8 {
    128
}
fn default_coverage() -> u8 {
    16
}
fn default_black() -> u8 {
    32
}
fn default_interior_coverage() -> u8 {
    60
}

/// Independent palettes and one common native fit for a species' six poses.
/// Palette slot zero is unused OBJ ink; visible slots 1..=3 must be distinct.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OverworldSpeciesSource {
    pub species_id: String,
    pub runtime_token: String,
    pub palette_rgb555: [u16; 4],
    pub shiny_palette_rgb555: [u16; 4],
    #[serde(default = "default_fit")]
    pub fit: [usize; 2],
    #[serde(default = "default_alpha")]
    pub alpha_threshold: u8,
    #[serde(default)]
    pub sampling: OverworldSampling,
    #[serde(default = "default_coverage")]
    pub minimum_foreground_samples: u8,
    #[serde(default = "default_black")]
    pub black_priority_samples: u8,
    /// Optional source-backed black detail priority, restricted to opaque interiors.
    #[serde(default)]
    pub interior_black_samples: Option<u8>,
    #[serde(default = "default_interior_coverage")]
    pub opaque_interior_minimum_samples: u8,
    pub frames: [OverworldFrameSource; FRAMES],
}

/// A complete original evolutionary family, with no roster/pack mutation.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OverworldManifest {
    pub schema_version: u16,
    pub family_id: String,
    pub species: Vec<OverworldSpeciesSource>,
}

/// Validated runtime files are separate from retained source/review artifacts.
#[derive(Debug)]
pub struct OverworldCompilation {
    pub runtime_files: BTreeMap<String, Vec<u8>>,
    pub review_files: BTreeMap<String, Vec<u8>>,
}

impl OverworldManifest {
    /// Parse a strict authoring manifest. Returns an error for invalid JSON.
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).context("parse overworld sprite manifest")
    }

    /// Compile source crops, preserving literal transparency and shared scale.
    ///
    /// Returns errors for unsafe paths, invalid palettes/geometry, missing poses,
    /// clipped or empty source art, blank native frames, or identical walk pairs.
    /// Source paths are relative to `source_root`; no files are modified.
    pub fn compile(&self, source_root: &Path) -> Result<OverworldCompilation> {
        ensure!(
            self.schema_version == 1,
            "unsupported overworld manifest version"
        );
        ensure!(
            self.family_id.starts_with("MN_F")
                && self.family_id.len() > 4
                && self.family_id[4..].bytes().all(|c| c.is_ascii_digit()),
            "invalid family ID"
        );
        ensure!(
            (1..=3).contains(&self.species.len()),
            "one complete family of 1..3 species required"
        );
        let mut identities = BTreeSet::new();
        let mut tokens = BTreeSet::new();
        let mut sources = BTreeMap::new();
        let mut source_hashes = BTreeMap::new();
        let mut runtime_files = BTreeMap::new();
        let mut review_files = BTreeMap::new();
        for species in &self.species {
            validate_species(species)?;
            ensure!(
                identities.insert(&species.species_id) && tokens.insert(&species.runtime_token),
                "duplicate species identity/token"
            );
            for frame in &species.frames {
                safe_relative(&frame.source)?;
                if !sources.contains_key(&frame.source) {
                    let bytes = fs::read(source_root.join(&frame.source))
                        .with_context(|| format!("read {}", frame.source))?;
                    let sheet = MinnesotaSheet::from_png(&bytes)?;
                    let hash = sha256(&bytes);
                    review_files.insert(format!("sources/{hash}.png"), bytes);
                    source_hashes.insert(frame.source.clone(), hash);
                    sources.insert(frame.source.clone(), sheet);
                }
            }
        }
        let mut records = Vec::new();
        let mut normal_board = vec![0_u8; 96 * 16 * self.species.len() * 4];
        let mut shiny_board = normal_board.clone();
        for (row, species) in self.species.iter().enumerate() {
            let mut bounds = Vec::new();
            for frame in &species.frames {
                let sheet = &sources[&frame.source];
                let [x, y, width, height] = frame.crop;
                ensure!(
                    width > 2
                        && height > 2
                        && x.checked_add(width)
                            .is_some_and(|right| right <= sheet.width)
                        && y.checked_add(height)
                            .is_some_and(|bottom| bottom <= sheet.height),
                    "{:?}: crop outside source",
                    frame.pose
                );
                bounds.push(
                    source_bounds(sheet, frame, species.alpha_threshold)
                        .with_context(|| format!("{} {:?}", species.runtime_token, frame.pose))?,
                );
            }
            let union = union_bounds(&bounds);
            let transform = Transform::new(union, species.fit);
            review_files.insert(
                format!(
                    "review/{}.source-bounds.png",
                    species.runtime_token.to_ascii_lowercase()
                ),
                source_bounds_overlay(species, &sources, &bounds, union)?,
            );
            let palette = species.palette_rgb555.map(rgb555);
            let mut bank = Vec::with_capacity(PIXELS * FRAMES);
            let mut frame_records = Vec::new();
            for (frame, source_bounds) in species.frames.iter().zip(&bounds) {
                let sheet = &sources[&frame.source];
                let pixels = sample_frame(sheet, frame, species, &transform, &palette);
                validate_frame(&pixels)
                    .with_context(|| format!("{} {:?}", species.runtime_token, frame.pose))?;
                frame_records.push(serde_json::json!({
                    "pose": frame.pose,
                    "source": frame.source,
                    "source_sha256": source_hashes[&frame.source],
                    "crop": frame.crop,
                    "source_alpha_bounds": source_bounds,
                    "native_bounds": pixel_bounds(&pixels),
                    "rows": rows(&pixels),
                    "pixel_sha256": sha256(&pixels),
                }));
                bank.extend(pixels);
            }
            for direction in 0..3 {
                ensure!(
                    bank[direction * PIXELS..(direction + 1) * PIXELS]
                        != bank[(direction + 3) * PIXELS..(direction + 4) * PIXELS],
                    "{} direction{direction}: idle and step became identical; revise source/crops",
                    species.runtime_token
                );
            }
            let token = species.runtime_token.to_ascii_lowercase();
            let prefix = format!("gfx/pokemon/{token}/overworld");
            let tiles = bitplanes(&bank, SIZE, SIZE * FRAMES);
            ensure!(
                decode_overworld_bitplanes(&tiles)? == bank,
                "native tile roundtrip mismatch"
            );
            let gray = rgba_pixels(&bank, &[[255; 3], [170; 3], [85; 3], [0; 3]]);
            let png = png_rgba(16, 96, &gray)?;
            validate_indexed_png(&png, &bank)?;
            runtime_files.insert(format!("{prefix}.png"), png);
            runtime_files.insert(format!("{prefix}.2bpp"), tiles);
            runtime_files.insert(
                format!("{prefix}.gbcpal"),
                palette_bytes(species.palette_rgb555),
            );
            runtime_files.insert(
                format!("{prefix}-shiny.gbcpal"),
                palette_bytes(species.shiny_palette_rgb555),
            );
            for (name, words, board) in [
                ("normal", species.palette_rgb555, &mut normal_board),
                ("shiny", species.shiny_palette_rgb555, &mut shiny_board),
            ] {
                let horizontal = horizontal_rgba(&bank, &words.map(rgb555));
                board[row * 96 * 16 * 4..(row + 1) * 96 * 16 * 4].copy_from_slice(&horizontal);
                review_files.insert(
                    format!("review/{token}.{name}.png"),
                    png_rgba(96, 16, &horizontal)?,
                );
                review_files.insert(
                    format!("review/{token}.{name}-8x.png"),
                    png_rgba(768, 128, &nearest_zoom(&horizontal, 96, 16, 8))?,
                );
            }
            records.push(serde_json::json!({
                "species_id": species.species_id,
                "runtime_token": species.runtime_token,
                "palette_rgb555": species.palette_rgb555,
                "shiny_palette_rgb555": species.shiny_palette_rgb555,
                "fit": species.fit,
                "baseline": 14,
                "shared_source_bounds": union,
                "sampling": species.sampling,
                "alpha_threshold": species.alpha_threshold,
                "minimum_foreground_samples": species.minimum_foreground_samples,
                "black_priority_samples": species.black_priority_samples,
                "interior_black_samples": species.interior_black_samples,
                "opaque_interior_minimum_samples": species.opaque_interior_minimum_samples,
                "frames": frame_records,
            }));
        }
        for (name, board) in [("normal", normal_board), ("shiny", shiny_board)] {
            let height = 16 * self.species.len();
            review_files.insert(
                format!("review/line.{name}.png"),
                png_rgba(96, height, &board)?,
            );
            review_files.insert(
                format!("review/line.{name}-8x.png"),
                png_rgba(768, height * 8, &nearest_zoom(&board, 96, height, 8))?,
            );
        }
        review_files.insert(
            "compiled-index.json".into(),
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": 1,
                "family_id": self.family_id,
                "frame_size": [16, 16],
                "frame_order": POSES,
                "obj_zero": "transparent_everywhere",
                "species": records,
            }))?,
        );
        review_files.insert(
            "source-manifest.json".into(),
            serde_json::to_vec_pretty(self)?,
        );
        Ok(OverworldCompilation {
            runtime_files,
            review_files,
        })
    }
}

impl OverworldCompilation {
    /// Install these compiled native banks into a separate verified game pack.
    ///
    /// Species data, audio and existing runtime files are preserved. Identical
    /// banks may be installed again; replacing different existing art is rejected.
    /// Review boards and source images are never installed as runtime assets.
    ///
    /// # Errors
    /// Returns an error for an invalid base, unsafe asset path, conflicting
    /// existing asset, or a result that fails pack verification.
    pub fn build_pack(&self, base: &CompiledGamePack) -> Result<CompiledGamePack> {
        crate::verify_compiled_game_pack_for_runtime(base)?;
        let mut pack = base.clone();
        for (name, bytes) in &self.runtime_files {
            crate::validate_compiled_runtime_file_key(name)?;
            if let Some(existing) = pack.runtime_files.get(name) {
                ensure!(existing == bytes, "overworld asset already differs: {name}");
            } else {
                pack.runtime_files.insert(name.clone(), bytes.clone());
            }
        }
        pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?;
        crate::verify_compiled_game_pack_for_runtime(&pack)?;
        Ok(pack)
    }

    /// Write outputs while retaining every old/new byte version.
    ///
    /// The caller must choose an ignored output directory. Returns an error for
    /// an unsafe generated path or any filesystem failure.
    pub fn write_to(&self, output: &Path) -> Result<()> {
        for (name, bytes) in self.runtime_files.iter().chain(&self.review_files) {
            safe_relative(name)?;
            let path = output.join(name);
            fs::create_dir_all(path.parent().context("output parent")?)?;
            archive::write_render(&path, bytes)?;
        }
        Ok(())
    }
}

fn validate_species(source: &OverworldSpeciesSource) -> Result<()> {
    ensure!(
        source.species_id.starts_with("MN_")
            && source.species_id.len() > 3
            && source.species_id[3..].bytes().all(|c| c.is_ascii_digit()),
        "invalid species ID"
    );
    ensure!(
        !source.runtime_token.is_empty()
            && source
                .runtime_token
                .bytes()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_'),
        "invalid runtime token"
    );
    for palette in [source.palette_rgb555, source.shiny_palette_rgb555] {
        ensure!(
            palette.iter().all(|&word| word < 0x8000)
                && palette[3] == 0
                && palette[1..].iter().collect::<BTreeSet<_>>().len() == 3,
            "palette requires 3 distinct visible RGB555 inks ending in black"
        );
    }
    ensure!(
        source.fit.iter().all(|value| (1..=14).contains(value)),
        "fit must preserve a 1px transparent border"
    );
    ensure!(
        source.alpha_threshold > 0
            && (1..=64).contains(&source.minimum_foreground_samples)
            && (1..=64).contains(&source.black_priority_samples)
            && source
                .interior_black_samples
                .is_none_or(|count| (1..=64).contains(&count))
            && (1..=64).contains(&source.opaque_interior_minimum_samples),
        "invalid alpha/sample thresholds"
    );
    for (frame, pose) in source.frames.iter().zip(POSES) {
        ensure!(frame.pose == pose, "native six-pose order mismatch");
        ensure!(
            frame.crop[2..] == source.frames[0].crop[2..],
            "all poses need same-size source windows for shared scale"
        );
    }
    Ok(())
}

fn safe_relative(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && Path::new(value)
                .components()
                .all(|part| matches!(part, Component::Normal(_))),
        "path must be relative with no parent traversal: {value}"
    );
    Ok(())
}

fn source_bounds(
    sheet: &MinnesotaSheet,
    frame: &OverworldFrameSource,
    threshold: u8,
) -> Result<[usize; 4]> {
    let [left, top, width, height] = frame.crop;
    let mut points = Vec::new();
    for y in 0..height {
        for x in 0..width {
            if sheet
                .alpha(left + x, top + y)
                .expect("validated source crop")
                >= threshold
            {
                points.push((x, y));
            }
        }
    }
    ensure!(
        !points.is_empty(),
        "{:?}: empty foreground after alpha threshold",
        frame.pose
    );
    let x = points.iter().map(|p| p.0).min().expect("nonempty");
    let y = points.iter().map(|p| p.1).min().expect("nonempty");
    let right = points.iter().map(|p| p.0).max().expect("nonempty");
    let bottom = points.iter().map(|p| p.1).max().expect("nonempty");
    ensure!(
        x > 0 && y > 0 && right + 1 < width && bottom + 1 < height,
        "{:?}: source bounds [{x}, {y}, {}, {}] touch crop {}x{}; isolate complete art",
        frame.pose,
        right - x + 1,
        bottom - y + 1,
        width,
        height
    );
    Ok([x, y, right - x + 1, bottom - y + 1])
}

fn source_bounds_overlay(
    species: &OverworldSpeciesSource,
    sources: &BTreeMap<String, MinnesotaSheet>,
    bounds: &[[usize; 4]],
    union: [usize; 4],
) -> Result<Vec<u8>> {
    let [_, _, width, height] = species.frames[0].crop;
    let output_width = width * FRAMES;
    let mut pixels = vec![0; output_width * height * 4];
    for (index, frame) in species.frames.iter().enumerate() {
        let sheet = &sources[&frame.source];
        for y in 0..height {
            for x in 0..width {
                let offset = (y * output_width + index * width + x) * 4;
                let point = (frame.crop[0] + x, frame.crop[1] + y);
                let rgb = if sheet.alpha(point.0, point.1).expect("valid crop")
                    >= species.alpha_threshold
                {
                    sheet.pixel(point.0, point.1).expect("valid crop")
                } else {
                    [if (x / 8 + y / 8) % 2 == 0 { 220 } else { 240 }; 3]
                };
                pixels[offset..offset + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
        }
        // Green is the common transform; red is this pose's actual alpha bound.
        for (rectangle, color) in [(union, [0, 160, 0, 255]), (bounds[index], [220, 0, 0, 255])] {
            let [left, top, w, h] = rectangle;
            for y in top..top + h {
                for x in left..left + w {
                    if x == left || y == top || x + 1 == left + w || y + 1 == top + h {
                        let offset = (y * output_width + index * width + x) * 4;
                        pixels[offset..offset + 4].copy_from_slice(&color);
                    }
                }
            }
        }
    }
    png_rgba(output_width, height, &pixels)
}

fn union_bounds(bounds: &[[usize; 4]]) -> [usize; 4] {
    let x = bounds.iter().map(|b| b[0]).min().expect("six poses");
    let y = bounds.iter().map(|b| b[1]).min().expect("six poses");
    let right = bounds.iter().map(|b| b[0] + b[2]).max().expect("six poses");
    let bottom = bounds.iter().map(|b| b[1] + b[3]).max().expect("six poses");
    [x, y, right - x, bottom - y]
}

struct Transform {
    bounds: [usize; 4],
    size: [usize; 2],
    origin: [usize; 2],
}
impl Transform {
    fn new(bounds: [usize; 4], fit: [usize; 2]) -> Self {
        let [_, _, width, height] = bounds;
        let size = if width * fit[1] >= height * fit[0] {
            [fit[0], (height * fit[0] / width).max(1)]
        } else {
            [(width * fit[1] / height).max(1), fit[1]]
        };
        Self {
            bounds,
            size,
            origin: [(16 - size[0]) / 2, 15 - size[1]],
        }
    }
}

fn sample_frame(
    sheet: &MinnesotaSheet,
    frame: &OverworldFrameSource,
    source: &OverworldSpeciesSource,
    t: &Transform,
    palette: &[[u8; 3]; 4],
) -> Vec<u8> {
    let mut output = vec![0; PIXELS];
    for y in 0..t.size[1] {
        for x in 0..t.size[0] {
            let mut counts = [0_u8; 4];
            let samples = if matches!(source.sampling, OverworldSampling::SourceCenter) {
                1
            } else {
                8
            };
            for sy in 0..samples {
                for sx in 0..samples {
                    let px = frame.crop[0]
                        + t.bounds[0]
                        + ((x * 2 * samples + 2 * sx + 1) * t.bounds[2]
                            / (t.size[0] * 2 * samples));
                    let py = frame.crop[1]
                        + t.bounds[1]
                        + ((y * 2 * samples + 2 * sy + 1) * t.bounds[3]
                            / (t.size[1] * 2 * samples));
                    if sheet.alpha(px, py).expect("validated transform") < source.alpha_threshold {
                        counts[0] += 1;
                        continue;
                    }
                    let rgb = sheet.pixel(px, py).expect("validated transform");
                    let ink = (1..=3)
                        .min_by_key(|&ink| {
                            rgb.iter()
                                .zip(palette[ink])
                                .map(|(&a, b)| {
                                    let delta = i32::from(a) - i32::from(b);
                                    delta * delta
                                })
                                .sum::<i32>()
                        })
                        .expect("three visible inks");
                    counts[ink] += 1;
                }
            }
            let ink = if samples == 1 {
                (0..=3).max_by_key(|&i| counts[i]).expect("four inks") as u8
            } else if 64 - counts[0] < source.minimum_foreground_samples {
                0
            } else if counts[3] >= source.black_priority_samples
                || source.interior_black_samples.is_some_and(|minimum| {
                    64 - counts[0] >= source.opaque_interior_minimum_samples && counts[3] >= minimum
                })
            {
                3
            } else {
                (1..=3).max_by_key(|&i| counts[i]).expect("three inks") as u8
            };
            output[(y + t.origin[1]) * 16 + x + t.origin[0]] = ink;
        }
    }
    output
}

fn validate_frame(pixels: &[u8]) -> Result<()> {
    ensure!(
        pixels.len() == PIXELS && pixels.iter().all(|&ink| ink <= 3),
        "invalid native indices"
    );
    ensure!(
        pixels.contains(&3) && pixels.iter().any(|&ink| ink == 1 || ink == 2),
        "native art lost outline/interior colors"
    );
    for y in 0..16 {
        for x in 0..16 {
            if x == 0 || x == 15 || y == 0 || y == 15 {
                ensure!(pixels[y * 16 + x] == 0, "opaque native border");
            }
        }
    }
    Ok(())
}

fn pixel_bounds(pixels: &[u8]) -> Option<[usize; 4]> {
    let points = pixels
        .iter()
        .enumerate()
        .filter(|(_, ink)| **ink != 0)
        .map(|(index, _)| (index % 16, index / 16))
        .collect::<Vec<_>>();
    let x = points.iter().map(|p| p.0).min()?;
    let y = points.iter().map(|p| p.1).min()?;
    Some([
        x,
        y,
        points.iter().map(|p| p.0).max()? - x + 1,
        points.iter().map(|p| p.1).max()? - y + 1,
    ])
}
fn rows(pixels: &[u8]) -> Vec<String> {
    pixels
        .as_chunks::<16>()
        .0
        .iter()
        .map(|row| row.iter().map(|&ink| char::from(b'0' + ink)).collect())
        .collect()
}
fn rgb555(word: u16) -> [u8; 3] {
    [0_u32, 5, 10].map(|shift| {
        let value = ((word >> shift) & 31_u16) as u8;
        (value << 3) | (value >> 2)
    })
}
fn palette_bytes(words: [u16; 4]) -> Vec<u8> {
    words.into_iter().flat_map(u16::to_le_bytes).collect()
}
fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Decode six frames of row-major 2×2 tiles. Returns an error unless 384 bytes.
pub fn decode_overworld_bitplanes(bytes: &[u8]) -> Result<Vec<u8>> {
    ensure!(bytes.len() == 384, "six 16x16 frames require 384 bytes");
    let mut pixels = vec![0; PIXELS * FRAMES];
    for frame in 0..6 {
        for ty in 0..2 {
            for tx in 0..2 {
                for y in 0..8 {
                    let offset = frame * 64 + (ty * 2 + tx) * 16 + y * 2;
                    for x in 0..8 {
                        pixels[frame * PIXELS + (ty * 8 + y) * 16 + tx * 8 + x] =
                            ((bytes[offset] >> (7 - x)) & 1)
                                | (((bytes[offset + 1] >> (7 - x)) & 1) << 1);
                    }
                }
            }
        }
    }
    Ok(pixels)
}

/// Decode a four-cell, transparent front walking strip for a web thumbnail.
///
/// Cells use idle, step, idle and mirrored step from the authored six-pose bank.
/// Returns an error for an invalid bank, palette, opaque border or static gait.
pub fn walking_thumbnail(bank: &[u8], palette: [u16; 4]) -> Result<Vec<u8>> {
    let indices = decode_overworld_bitplanes(bank)?;
    ensure!(
        palette.iter().all(|word| *word < 0x8000)
            && palette[3] == 0
            && palette[1..].iter().collect::<BTreeSet<_>>().len() == 3,
        "thumbnail requires three distinct RGB555 inks ending in black"
    );
    for frame in indices.as_chunks::<PIXELS>().0 {
        ensure!(
            (0..SIZE).all(|i| frame[i] == 0
                && frame[(SIZE - 1) * SIZE + i] == 0
                && frame[i * SIZE] == 0
                && frame[i * SIZE + SIZE - 1] == 0),
            "thumbnail bank has an opaque border"
        );
    }
    ensure!(
        indices[..PIXELS] != indices[3 * PIXELS..4 * PIXELS],
        "static front gait"
    );
    let mut strip = vec![0; SIZE * 4 * SIZE];
    for (cell, source) in [0, 3, 0, 3].into_iter().enumerate() {
        for y in 0..SIZE {
            for x in 0..SIZE {
                let source_x = if cell == 3 { SIZE - 1 - x } else { x };
                strip[y * SIZE * 4 + cell * SIZE + x] =
                    indices[source * PIXELS + y * SIZE + source_x];
            }
        }
    }
    png_rgba(SIZE * 4, SIZE, &rgba_pixels(&strip, &palette.map(rgb555)))
}

fn rgba_pixels(indices: &[u8], palette: &[[u8; 3]; 4]) -> Vec<u8> {
    indices
        .iter()
        .flat_map(|&ink| {
            let [r, g, b] = palette[usize::from(ink)];
            [r, g, b, if ink == 0 { 0 } else { 255 }]
        })
        .collect()
}
fn horizontal_rgba(bank: &[u8], palette: &[[u8; 3]; 4]) -> Vec<u8> {
    let mut result = vec![0; 96 * 16 * 4];
    for frame in 0..6 {
        for y in 0..16 {
            for x in 0..16 {
                let ink = bank[frame * PIXELS + y * 16 + x];
                let [r, g, b] = palette[usize::from(ink)];
                let offset = (y * 96 + frame * 16 + x) * 4;
                result[offset..offset + 4].copy_from_slice(&[
                    r,
                    g,
                    b,
                    if ink == 0 { 0 } else { 255 },
                ]);
            }
        }
    }
    result
}
fn nearest_zoom(pixels: &[u8], width: usize, height: usize, zoom: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(pixels.len() * zoom * zoom);
    for y in 0..height * zoom {
        for x in 0..width * zoom {
            let offset = ((y / zoom) * width + x / zoom) * 4;
            result.extend_from_slice(&pixels[offset..offset + 4]);
        }
    }
    result
}
fn validate_indexed_png(bytes: &[u8], indices: &[u8]) -> Result<()> {
    let sheet = MinnesotaSheet::from_png(bytes)?;
    ensure!(
        (sheet.width, sheet.height) == (16, 96),
        "runtime PNG geometry mismatch"
    );
    for (index, &ink) in indices.iter().enumerate() {
        let x = index % 16;
        let y = index / 16;
        ensure!(
            sheet.pixel(x, y) == Some([[255; 3], [170; 3], [85; 3], [0; 3]][usize::from(ink)])
                && sheet.alpha(x, y) == Some(if ink == 0 { 0 } else { 255 }),
            "indexed PNG disagrees with literal OBJ ink{ink} at{x},{y}"
        );
    }
    Ok(())
}
fn png_rgba(width: usize, height: usize, pixels: &[u8]) -> Result<Vec<u8>> {
    ensure!(pixels.len() == width * height * 4, "RGBA geometry mismatch");
    let mut raw = Vec::with_capacity(height * (width * 4 + 1));
    for row in pixels.chunks_exact(width * 4) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&raw)?;
    let compressed = encoder.finish()?;
    let mut output = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::new();
    header.extend(u32::try_from(width)?.to_be_bytes());
    header.extend(u32::try_from(height)?.to_be_bytes());
    header.extend([8, 6, 0, 0, 0]);
    for (kind, data) in [
        (b"IHDR", header.as_slice()),
        (b"IDAT", compressed.as_slice()),
        (b"IEND", &[]),
    ] {
        output.extend(u32::try_from(data.len())?.to_be_bytes());
        output.extend(kind);
        output.extend(data);
        let mut crc = Crc::new();
        crc.update(kind);
        crc.update(data);
        output.extend(crc.sum().to_be_bytes());
    }
    Ok(output)
}

#[cfg(test)]
#[path = "minnesota_overworld_sprites_tests.rs"]
mod tests;
