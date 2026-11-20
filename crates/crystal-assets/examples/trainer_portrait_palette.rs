//! Optional source-color and whole-sprite sampling-alignment experiment.
//!
//! Usage: trainer_portrait_palette SOURCE OUTPUT LIGHT_RGB555 ACCENT_RGB555
//!        [--native PHASE_X PHASE_Y]
//! Native mode fits the whole source to 52x54 inside a 56x56 canvas, using one
//! sampling phase for every pixel. It adds no outlines or region-specific edits.
//! A phase of 0.5 samples the center of each footprint. Native mode also writes
//! same-stem JSON rows and 2bpp graphics. Plain mode preserves source dimensions;
//! both modes composite source transparency onto white through MinnesotaSheet.
//! Keep OUTPUT in an ignored review directory. The output must not already exist.
#[allow(dead_code)]
#[path = "../src/minnesota_pokemon_pixel_art.rs"]
mod minnesota_pokemon_pixel_art;
#[allow(dead_code)]
#[path = "../src/minnesota_sprite_sheet.rs"]
mod minnesota_sprite_sheet;

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use minnesota_sprite_sheet::MinnesotaSheet;
use serde::Serialize;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 4 || (args.len() == 7 && args[4] == "--native"),
        "usage: trainer_portrait_palette SOURCE OUTPUT LIGHT_RGB555 ACCENT_RGB555 [--native PHASE_X PHASE_Y]"
    );
    let light = args[2].parse::<u16>().context("invalid light RGB555 ink")?;
    let accent = args[3]
        .parse::<u16>()
        .context("invalid accent RGB555 ink")?;
    ensure!(
        (1..32767).contains(&light) && (1..32767).contains(&accent) && light != accent,
        "light and accent must be distinct RGB555 inks other than black and white"
    );
    let source = MinnesotaSheet::from_png(&fs::read(&args[0]).context("read source PNG")?)?;
    let inks = [32767, light, accent, 0];
    let palette = PortraitPalette::new(inks);
    let output = Path::new(&args[1]);
    if args.len() == 7 {
        let phase = [
            args[5]
                .parse::<f64>()
                .context("invalid horizontal sampling phase")?,
            args[6]
                .parse::<f64>()
                .context("invalid vertical sampling phase")?,
        ];
        let native = sample_native(&source, &palette, inks, phase)?;
        let paths = [
            output.to_path_buf(),
            output.with_extension("json"),
            output.with_extension("2bpp"),
        ];
        ensure!(
            paths.iter().all(|path| !path.exists()),
            "native output files must not already exist"
        );
        ensure!(
            output
                .extension()
                .is_some_and(|extension| extension == "png"),
            "OUTPUT must have a PNG extension"
        );
        let pixels = minnesota_pokemon_pixel_art::colored(&native.indices, inks);
        write_new(
            &paths[0],
            &minnesota_pokemon_pixel_art::png_rgb(56, 56, &pixels)?,
        )?;
        write_new(&paths[1], &serde_json::to_vec_pretty(&native)?)?;
        write_new(
            &paths[2],
            &minnesota_pokemon_pixel_art::bitplanes(&native.indices, 56, 56),
        )?;
        println!(
            "56x56 {:?} phase={phase:?} {}",
            native.source_bounds,
            output.display()
        );
        return Ok(());
    }
    let pixels = normalize(&source, &palette);
    let png = minnesota_pokemon_pixel_art::png_rgb(source.width, source.height, &pixels)?;
    write_new(output, &png)?;
    println!("{}x{} {}", source.width, source.height, output.display());
    Ok(())
}

fn write_new(output: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent).context("create review output directory")?;
    }
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .with_context(|| format!("create new review output {}", output.display()))?
        .write_all(bytes)
        .with_context(|| format!("write review output {}", output.display()))?;
    Ok(())
}

struct PortraitPalette {
    colors: Vec<[u8; 3]>,
    labs: Vec<[f64; 3]>,
}

impl PortraitPalette {
    fn new(inks: [u16; 4]) -> Self {
        let colors = minnesota_pokemon_pixel_art::colored(&[0, 1, 2, 3], inks);
        let labs = colors.iter().copied().map(srgb_to_lab).collect();
        Self { colors, labs }
    }

    fn nearest(&self, pixel: [u8; 3]) -> [u8; 3] {
        self.colors[self.nearest_index(pixel)]
    }

    fn nearest_index(&self, pixel: [u8; 3]) -> usize {
        let lab = srgb_to_lab(pixel);
        let mut best = 0;
        let mut distance = f64::INFINITY;
        for (index, candidate) in self.labs.iter().enumerate() {
            let squared_delta_e = lab
                .iter()
                .zip(candidate)
                .map(|(value, target)| (value - target).powi(2))
                .sum::<f64>();
            if squared_delta_e < distance {
                best = index;
                distance = squared_delta_e;
            }
        }
        best
    }
}

#[derive(Serialize)]
struct NativePortrait {
    sampling: &'static str,
    size: [usize; 2],
    source_bounds: [usize; 4],
    fit_size: [usize; 2],
    phase: [f64; 2],
    palette_rgb555: [u16; 4],
    rows: Vec<String>,
    #[serde(skip)]
    indices: Vec<u8>,
}

fn sample_native(
    source: &MinnesotaSheet,
    palette: &PortraitPalette,
    inks: [u16; 4],
    phase: [f64; 2],
) -> Result<NativePortrait> {
    ensure!(
        phase
            .iter()
            .all(|value| value.is_finite() && (0.0..1.0).contains(value)),
        "sampling phases must be in [0, 1)"
    );
    // Match the existing importer's whole-source foreground bounds, while
    // deliberately omitting its exterior-contour and enclosed-detail passes.
    let mut bounds = [source.width, source.height, 0, 0];
    for y in 0..source.height {
        for x in 0..source.width {
            if source
                .pixel(x, y)
                .expect("in-bounds source pixel")
                .iter()
                .any(|value| *value < 220)
            {
                bounds[0] = bounds[0].min(x);
                bounds[1] = bounds[1].min(y);
                bounds[2] = bounds[2].max(x + 1);
                bounds[3] = bounds[3].max(y + 1);
            }
        }
    }
    ensure!(
        bounds[2] > bounds[0] && bounds[3] > bounds[1],
        "empty source sprite"
    );
    let source_width = bounds[2] - bounds[0];
    let source_height = bounds[3] - bounds[1];
    let scale = (52.0 / source_width as f64).min(54.0 / source_height as f64);
    let width = ((source_width as f64 * scale).round() as usize).clamp(1, 52);
    let height = ((source_height as f64 * scale).round() as usize).clamp(1, 54);
    let left = (56 - width) / 2;
    let top = 55 - height;
    let mut indices = vec![0; 56 * 56];
    for y in 0..height {
        for x in 0..width {
            let sx = bounds[0]
                + (((x as f64 + phase[0]) * source_width as f64 / width as f64) as usize)
                    .min(source_width - 1);
            let sy = bounds[1]
                + (((y as f64 + phase[1]) * source_height as f64 / height as f64) as usize)
                    .min(source_height - 1);
            indices[(top + y) * 56 + left + x] = palette.nearest_index(
                source
                    .pixel(sx, sy)
                    .expect("sample lies within source bounds"),
            ) as u8;
        }
    }
    let rows = indices
        .as_chunks::<56>()
        .0
        .iter()
        .map(|row| {
            row.iter()
                .map(|index| char::from(b".123"[usize::from(*index)]))
                .collect()
        })
        .collect();
    Ok(NativePortrait {
        sampling: "whole_source_phase_d65_lab",
        size: [56, 56],
        source_bounds: bounds,
        fit_size: [width, height],
        phase,
        palette_rgb555: inks,
        rows,
        indices,
    })
}

fn normalize(source: &MinnesotaSheet, palette: &PortraitPalette) -> Vec<[u8; 3]> {
    let mut cache = HashMap::new();
    let mut pixels = Vec::with_capacity(source.width * source.height);
    for y in 0..source.height {
        for x in 0..source.width {
            let pixel = source
                .pixel(x, y)
                .expect("coordinates are within source dimensions");
            pixels.push(*cache.entry(pixel).or_insert_with(|| palette.nearest(pixel)));
        }
    }
    pixels
}

// CIE 1976 L*a*b* with the D65 reference white and the standard sRGB transfer
// function. Euclidean distance is Delta E 76, without hue-specific adjustments.
fn srgb_to_lab(pixel: [u8; 3]) -> [f64; 3] {
    let [red, green, blue] = pixel.map(|channel| {
        let value = f64::from(channel) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    });
    let x = (0.4124564 * red + 0.3575761 * green + 0.1804375 * blue) / 0.95047;
    let y = 0.2126729 * red + 0.7151522 * green + 0.0721750 * blue;
    let z = (0.0193339 * red + 0.1191920 * green + 0.9503041 * blue) / 1.08883;
    let [x, y, z] = [x, y, z].map(|value| {
        let delta: f64 = 6.0 / 29.0;
        if value > delta.powi(3) {
            value.cbrt()
        } else {
            value / (3.0 * delta.powi(2)) + 4.0 / 29.0
        }
    });
    [116.0 * y - 16.0, 500.0 * (x - y), 200.0 * (y - z)]
}

#[cfg(test)]
mod tests {
    use super::{PortraitPalette, normalize, sample_native, srgb_to_lab};
    use crate::minnesota_pokemon_pixel_art;
    use crate::minnesota_sprite_sheet::MinnesotaSheet;

    #[test]
    fn d65_reference_colors_match_lab() {
        for (rgb, expected) in [
            ([0, 0, 0], [0.0, 0.0, 0.0]),
            ([255, 255, 255], [100.0, 0.0, 0.0]),
            ([255, 0, 0], [53.2408, 80.0925, 67.2032]),
        ] {
            let actual = srgb_to_lab(rgb);
            for channel in 0..3 {
                assert!((actual[channel] - expected[channel]).abs() < 0.001);
            }
        }
    }

    #[test]
    fn neutral_antialias_edges_do_not_acquire_the_moss_accent() {
        let palette = PortraitPalette::new([32767, 25370, 8683, 0]);
        assert_eq!(palette.nearest([70, 70, 70]), palette.colors[3]);
        assert_eq!(palette.nearest([200, 200, 200]), palette.colors[1]);
        for ink in &palette.colors {
            assert_eq!(palette.nearest(*ink), *ink, "native ink must survive");
        }
        assert_eq!(palette.nearest([90, 120, 65]), palette.colors[2]);
    }

    #[test]
    fn normalization_preserves_dimensions_and_pixel_locations() {
        let palette = PortraitPalette::new([32767, 25370, 8683, 0]);
        let input = [
            [255, 255, 255],
            [70, 70, 70],
            [90, 120, 65],
            [200, 200, 200],
        ];
        let encoded = minnesota_pokemon_pixel_art::png_rgb(2, 2, &input).expect("encode fixture");
        let source = MinnesotaSheet::from_png(&encoded).expect("decode fixture");
        let pixels = normalize(&source, &palette);
        assert_eq!((source.width, source.height), (2, 2));
        assert_eq!(
            pixels,
            [
                palette.colors[0],
                palette.colors[3],
                palette.colors[2],
                palette.colors[1]
            ]
        );
    }

    #[test]
    fn native_phase_applies_uniformly_without_adding_contours() {
        let inks = [32767, 25370, 8683, 0];
        let palette = PortraitPalette::new(inks);
        let mut input = vec![[255, 255, 255]; 208 * 216];
        input[0] = [0, 0, 0];
        input[208 * 216 - 1] = [0, 0, 0];
        for [x, y] in [[10, 10], [20, 30]] {
            input[(4 * y + 1) * 208 + 4 * x + 1] = palette.colors[3];
            input[(4 * y + 2) * 208 + 4 * x + 2] = palette.colors[2];
        }
        let png = minnesota_pokemon_pixel_art::png_rgb(208, 216, &input).expect("encode fixture");
        let source = MinnesotaSheet::from_png(&png).expect("decode fixture");
        for (phase, expected) in [(0.25, 3), (0.5, 2), (0.75, 0)] {
            let native =
                sample_native(&source, &palette, inks, [phase, phase]).expect("sample fixture");
            assert_eq!(native.fit_size, [52, 54]);
            for [x, y] in [[10, 10], [20, 30]] {
                assert_eq!(native.indices[(1 + y) * 56 + 2 + x], expected);
            }
            // Only the globally selected source pixels appear; no black rim is added.
            let nonwhite = native.indices.iter().filter(|index| **index != 0).count();
            assert_eq!(nonwhite, if phase == 0.75 { 1 } else { 2 });
            assert_eq!(
                minnesota_pokemon_pixel_art::bitplanes(&native.indices, 56, 56).len(),
                784
            );
        }
    }
}
