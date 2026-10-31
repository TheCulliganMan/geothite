//! One hand-composed native title card; all creature pixels come from the pack.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::minnesota_pokemon_pixel_art::{colored, png_rgb};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const INK: [u8; 3] = [16, 32, 57];
const GOLD: [u8; 3] = [255, 220, 123];
const MIST: [u8; 3] = [198, 231, 231];

pub(super) fn compose(files: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>> {
    let source = files
        .get("gfx/pokemon/kettlurk/front.2bpp")
        .context("Kettlurk native front")?;
    let palette = files
        .get("gfx/pokemon/kettlurk/normal.gbcpal")
        .context("Kettlurk native palette")?;
    ensure!(
        source.len() == 784 && palette.len() == 8,
        "56x56 native Kettlurk required"
    );
    let palette = std::array::from_fn(|i| u16::from_le_bytes([palette[i * 2], palette[i * 2 + 1]]));
    let mut pixels = vec![INK; WIDTH * HEIGHT];
    // Muted adaptation of the official 2024 Minnesota flag: the inward
    // state-shaped hoist and its eight-point star stay behind the hero.
    // Reference: sos.mn.gov/about-minnesota/state-symbols/state-flag
    for y in 0..HEIGHT {
        let distance = (y as i32 - 72).unsigned_abs() as usize;
        let boundary = 62 + 28 * distance / 72;
        for x in boundary..WIDTH {
            pixels[y * WIDTH + x] = [30, 62, 82];
        }
    }
    let star = [
        (34, 55),
        (37, 64),
        (46, 60),
        (42, 69),
        (51, 72),
        (42, 75),
        (46, 84),
        (37, 80),
        (34, 89),
        (31, 80),
        (22, 84),
        (26, 75),
        (17, 72),
        (26, 69),
        (22, 60),
        (31, 64),
    ];
    for y in 55..90 {
        for x in 17..52 {
            if inside_polygon(x, y, &star) {
                pixels[y as usize * WIDTH + x as usize] = [68, 101, 120];
            }
        }
    }
    for (x, top) in [(10, 84), (146, 70)] {
        for y in top..120 {
            for dx in 0..4 {
                pixels[y * WIDTH + x + dx] = if dx % 2 == 0 {
                    [74, 139, 156]
                } else {
                    [123, 190, 198]
                };
            }
        }
    }
    for (x, y, length) in [(17, 119, 23), (55, 122, 20), (88, 117, 22), (120, 124, 25)] {
        for dx in 0..length {
            pixels[y * WIDTH + x + dx] = [74, 139, 156];
        }
    }
    // Decode native Game Boy tile order without a second raster source/catalog.
    let mut indices = vec![0; 56 * 56];
    for ty in 0..7 {
        for tx in 0..7 {
            for y in 0..8 {
                for x in 0..8 {
                    let offset = (ty * 7 + tx) * 16 + y * 2;
                    indices[(ty * 8 + y) * 56 + tx * 8 + x] = ((source[offset] >> (7 - x)) & 1)
                        | (((source[offset + 1] >> (7 - x)) & 1) << 1);
                }
            }
        }
    }
    let occupied = indices
        .iter()
        .enumerate()
        .filter(|(_, ink)| **ink != 0)
        .map(|(i, _)| (i % 56, i / 56))
        .collect::<Vec<_>>();
    let left = occupied
        .iter()
        .map(|(x, _)| *x)
        .min()
        .context("empty title creature")?;
    let right = occupied
        .iter()
        .map(|(x, _)| *x)
        .max()
        .context("empty title creature")?;
    let top = occupied
        .iter()
        .map(|(_, y)| *y)
        .min()
        .context("empty title creature")?;
    let bottom = occupied
        .iter()
        .map(|(_, y)| *y)
        .max()
        .context("empty title creature")?;
    ensure!(
        (bottom - top + 1) * 2 <= 88,
        "title creature exceeds the hero area"
    );
    let origin_x = (WIDTH - (right - left + 1) * 2) / 2;
    for (i, rgb) in colored(&indices, palette).into_iter().enumerate() {
        if indices[i] == 0 {
            continue;
        }
        for dy in 0..2 {
            for dx in 0..2 {
                pixels[(44 + (i / 56 - top) * 2 + dy) * WIDTH
                    + origin_x
                    + (i % 56 - left) * 2
                    + dx] = rgb;
            }
        }
    }
    label(&mut pixels, "POKEMON", 2, 7, GOLD);
    // Accent over the E; original lettering stays sharp on the native grid.
    pixels[4 * WIDTH + 81] = GOLD;
    pixels[5 * WIDTH + 80] = GOLD;
    label(&mut pixels, "NORTHSTAR", 2, 28, MIST);
    label(&mut pixels, "PRESS START", 1, 133, GOLD);
    png_rgb(WIDTH, HEIGHT, &pixels)
}

fn label(pixels: &mut [[u8; 3]], text: &str, scale: usize, top: usize, color: [u8; 3]) {
    let left = (WIDTH - (text.len() * 6 - 1) * scale) / 2;
    for (index, ch) in text.chars().enumerate() {
        for (y, row) in glyph(ch).into_iter().enumerate() {
            for x in 0..5 {
                if row & (1 << (4 - x)) == 0 {
                    continue;
                }
                for dy in 0..scale {
                    for dx in 0..scale {
                        pixels[(top + y * scale + dy) * WIDTH
                            + left
                            + index * 6 * scale
                            + x * scale
                            + dx] = color;
                    }
                }
            }
        }
    }
}
fn glyph(ch: char) -> [u8; 7] {
    match ch {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 25, 21, 19, 19, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        _ => [0; 7],
    }
}

fn inside_polygon(x: i32, y: i32, points: &[(i32, i32)]) -> bool {
    let mut inside = false;
    let mut previous = points[points.len() - 1];
    for &(px, py) in points {
        if (py > y) != (previous.1 > y) && x < (previous.0 - px) * (y - py) / (previous.1 - py) + px
        {
            inside = !inside;
        }
        previous = (px, py);
    }
    inside
}
