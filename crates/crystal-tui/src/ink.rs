//! Ordered dot dithering of the actual pack pixels. No substitute scenery,
//! inferred materials, synthetic lighting, or replacement palette.
use ratatui::{buffer::Buffer, layout::Rect, style::Color};

const DOTS: [&str; 4] = [" ", "·", "•", "●"];
const PAPER: Color = Color::Rgb(3, 3, 18);
const BAYER: [u8; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];

fn sample(pixels: &[[u8; 3]; 256], x: f64, y: f64) -> [f64; 3] {
    // Area sampling, not a new material field: native cells cover several
    // source pixels; the browser's square detail cells retain every pixel.
    let x0 = x.floor().clamp(0., 15.) as usize;
    let y0 = y.floor().clamp(0., 15.) as usize;
    pixels[y0 * 16 + x0].map(f64::from)
}

pub(crate) fn tile(
    b: &mut Buffer,
    a: Rect,
    pixels: &[[u8; 3]; 256],
    x: i16,
    y: i16,
    phase: f64,
) -> Vec<u8> {
    let mut diameters = vec![0; usize::from(a.width) * usize::from(a.height)];
    if a.width == 0 || a.height == 0 {
        return diameters;
    }
    for row in 0..a.height {
        for col in 0..a.width {
            let mut sum = [0.; 3];
            let mut count = 0.;
            let sx = 16. / f64::from(a.width);
            let sy = 16. / f64::from(a.height);
            let nx = sx.ceil() as u16;
            let ny = sy.ceil() as u16;
            for yy in 0..ny {
                for xx in 0..nx {
                    let p = sample(
                        pixels,
                        (f64::from(col) + (f64::from(xx) + 0.5) / f64::from(nx)) * sx,
                        (f64::from(row) + (f64::from(yy) + 0.5) / f64::from(ny)) * sy,
                    );
                    for i in 0..3 {
                        sum[i] += p[i];
                    }
                    count += 1.;
                }
            }
            let rgb = sum.map(|v| v / count);
            let peak = rgb.into_iter().fold(1., f64::max);
            let wx = i32::from(x) * i32::from(a.width) + i32::from(col);
            let wy = i32::from(y) * i32::from(a.height) + i32::from(row);
            let threshold =
                (f64::from(BAYER[(wy.rem_euclid(4) * 4 + wx.rem_euclid(4)) as usize]) + 0.5) / 16.;
            // Coverage, not colored rectangular paper, carries the tone.
            // Dot areas match the browser's circle painter. Bayer chooses
            // between adjacent sizes without disturbing source silhouettes.
            let coverage = [0., 0.065, 0.28, 0.78];
            // Gentle world-space variation, never seeded by a metatile ID.
            let world_x = f64::from(x) + (f64::from(col) + 0.5) / f64::from(a.width);
            let world_y = f64::from(y) + (f64::from(row) + 0.5) / f64::from(a.height);
            let shape = 1.
                + 0.16 * (world_x * 1.73 + world_y * 0.81).sin()
                + 0.10 * (world_x * 0.39 - world_y * 1.17).cos();
            let breath = 1. + 0.10 * (world_x * 0.43 + world_y * 0.31 + phase).sin();
            let tone = ((peak / 255.) * shape * breath).min(0.78);
            let upper = coverage.iter().position(|v| *v >= tone).unwrap_or(3).max(1);
            let lower = upper - 1;
            let fraction = (tone - coverage[lower]) / (coverage[upper] - coverage[lower]);
            let step = if threshold < fraction { upper } else { lower };
            diameters[usize::from(row) * usize::from(a.width) + usize::from(col)] =
                (2. * (coverage[step] / std::f64::consts::PI).sqrt() * shape.sqrt() * 255.)
                    .clamp(0., 250.) as u8;
            // Same RGB ray, compensated for ink area; no replacement palette.
            let foreground_scale = 255. / peak;
            let color = |scale: f64| {
                let p = rgb.map(|v| (v * scale).round().clamp(0., 255.) as u8);
                Color::Rgb(p[0], p[1], p[2])
            };
            b[(a.x + col, a.y + row)]
                .set_symbol(DOTS[step])
                .set_fg(color(foreground_scale))
                .set_bg(PAPER);
        }
    }
    diameters
}

/// Encode the SAME colored circle field used by the browser in ordinary,
/// single-width terminal glyphs. This adapter does not sample tiles, choose
/// palettes, create scenery or draw collision: all of that is already in fine.
pub(crate) fn pack_terminal_dots(
    b: &mut Buffer,
    a: Rect,
    fine: &Buffer,
    sizes: &[u8],
    left: i16,
    top: i16,
) {
    assert_eq!(fine.area.width, a.width * 2);
    assert_eq!(fine.area.height, a.height * 4);
    assert_eq!(sizes.len(), fine.content.len());
    const BITS: [[u8; 2]; 4] = [[0, 3], [1, 4], [2, 5], [6, 7]];
    for row in 0..a.height {
        for col in 0..a.width {
            let mut mask = 0u8;
            let mut ink = [0f64; 3];
            let mut weight = 0f64;
            for dy in 0..4 {
                for dx in 0..2 {
                    let x = col * 2 + dx;
                    let y = row * 4 + dy;
                    let size = f64::from(
                        sizes[usize::from(y) * usize::from(fine.area.width) + usize::from(x)],
                    ) / 255.;
                    let area = size * size;
                    // A Braille dot cannot change diameter. Ordered density
                    // encodes circle area on its 2x4 subcell lattice instead.
                    // Offset the order from the shared size quantizer so its
                    // small circles do not correlate into missing edge lines.
                    let wx = i32::from(left) * 16 + i32::from(x);
                    let wy = i32::from(top) * 16 + i32::from(y);
                    let threshold = (f64::from(
                        BAYER[((wy + 1).rem_euclid(4) * 4 + (wx + 2).rem_euclid(4)) as usize],
                    ) + 0.5)
                        / 16.;
                    if area <= threshold {
                        continue;
                    }
                    mask |= 1 << BITS[usize::from(dy)][usize::from(dx)];
                    if let Color::Rgb(r, g, blue) = fine[(x, y)].fg {
                        for (i, value) in [r, g, blue].into_iter().enumerate() {
                            ink[i] += f64::from(value) * area;
                        }
                        weight += area;
                    }
                }
            }
            let rgb = ink.map(|v| (v / weight.max(0.001)).round().clamp(0., 255.) as u8);
            let symbol = if mask == 0 {
                ' '
            } else {
                char::from_u32(0x2800 + u32::from(mask)).unwrap()
            };
            b[(a.x + col, a.y + row)]
                .set_char(symbol)
                .set_fg(Color::Rgb(rgb[0], rgb[1], rgb[2]))
                .set_bg(PAPER);
        }
    }
}

/// Software canvas for terminal graphics. Exactly the same foreground colors,
/// radii and square-cell circles as the browser adapter, on the same paper.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn rasterize_dots(
    fine: &Buffer,
    sizes: &[u8],
    width: u32,
    height: u32,
) -> image::RgbImage {
    let mut image = image::RgbImage::from_pixel(width, height, image::Rgb([3, 3, 18]));
    if width == 0 || height == 0 || fine.area.width == 0 || fine.area.height == 0 {
        return image;
    }
    let cw = f64::from(width) / f64::from(fine.area.width);
    let ch = f64::from(height) / f64::from(fine.area.height);
    for (i, size) in sizes.iter().enumerate() {
        if *size == 0 {
            continue;
        }
        let x = (i % usize::from(fine.area.width)) as f64;
        let y = (i / usize::from(fine.area.width)) as f64;
        let center_x = (x + 0.5) * cw;
        let center_y = (y + 0.5) * ch;
        let radius = f64::from(*size) / 255. * cw * 0.5;
        let Color::Rgb(r, g, b) = fine.content[i].fg else {
            continue;
        };
        let ink = [r, g, b];
        let x0 = (center_x - radius - 1.).floor().max(0.) as u32;
        let y0 = (center_y - radius - 1.).floor().max(0.) as u32;
        let x1 = (center_x + radius + 1.).ceil().min(f64::from(width)) as u32;
        let y1 = (center_y + radius + 1.).ceil().min(f64::from(height)) as u32;
        for py in y0..y1 {
            for px in x0..x1 {
                let distance = ((f64::from(px) + 0.5 - center_x).powi(2)
                    + (f64::from(py) + 0.5 - center_y).powi(2))
                .sqrt();
                let alpha = (radius + 0.5 - distance).clamp(0., 1.);
                if alpha == 0. {
                    continue;
                }
                let pixel = image.get_pixel_mut(px, py);
                for channel in 0..3 {
                    pixel[channel] = (f64::from(pixel[channel]) * (1. - alpha)
                        + f64::from(ink[channel]) * alpha)
                        .round() as u8;
                }
            }
        }
    }
    image
}

/// Ledge chevrons and unresolved/barrier silhouettes use dots, not floating
/// letters or punctuation. Core collision resolution supplies the direction.
pub(crate) fn collision(b: &mut Buffer, a: Rect, permission: char) {
    let direction = match permission {
        '→' => (1, 0),
        '←' => (-1, 0),
        '↓' => (0, 1),
        '↑' => (0, -1),
        '↘' => (1, 1),
        '↙' => (-1, 1),
        '↗' => (1, -1),
        '↖' => (-1, -1),
        _ => (0, 0),
    };
    if direction == (0, 0) && !matches!(permission, '?' | '║') {
        return;
    }
    let w = i32::from(a.width);
    let h = i32::from(a.height);
    for y in 0..h {
        for x in 0..w {
            let u = (x as f64 + 0.5) / w as f64;
            let v = (y as f64 + 0.5) / h as f64;
            let mark = if permission == '?' {
                (x + y).rem_euclid(4) == 0
            } else if permission == '║' {
                x == w / 2 || x == w / 2 - 1
            } else {
                let (u, v) = match direction {
                    (1, 0) => (v, 1. - u),
                    (-1, 0) => (v, u),
                    (0, -1) => (u, 1. - v),
                    _ => (u, v),
                };
                (0.32..0.68).contains(&u) && (v - (0.78 - (u - 0.5).abs())).abs() < 1.1 / h as f64
            };
            if mark {
                let cell = &mut b[(a.x + x as u16, a.y + y as u16)];
                cell.set_symbol("●");
                // Keep the actual tile hue; boost the cue only against dark ink.
                if let Color::Rgb(r, g, blue) = cell.fg {
                    cell.set_fg(Color::Rgb(r.max(80), g.max(80), blue.max(80)));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terminal_packs_all_eight_shared_dot_positions_without_raster_blocks() {
        let mut fine = Buffer::empty(Rect::new(0, 0, 2, 4));
        fine.set_style(
            fine.area,
            ratatui::style::Style::default()
                .fg(Color::Rgb(240, 120, 40))
                .bg(PAPER),
        );
        let bits = [[0, 3], [1, 4], [2, 5], [6, 7]];
        for y in 0..4 {
            for x in 0..2 {
                let mut sizes = vec![0; 8];
                sizes[y * 2 + x] = 255;
                let mut output = Buffer::empty(Rect::new(0, 0, 3, 3));
                pack_terminal_dots(&mut output, Rect::new(1, 1, 1, 1), &fine, &sizes, 46, 12);
                assert_eq!(
                    output[(1, 1)].symbol().chars().next().unwrap() as u32,
                    0x2800 + (1 << bits[y][x])
                );
                assert_eq!(output[(1, 1)].fg, fine[(x as u16, y as u16)].fg);
                assert_eq!(output[(1, 1)].bg, PAPER);
                assert_eq!(output[(0, 0)].symbol(), " ");
            }
        }
    }

    #[test]
    fn terminal_encoding_is_stable_when_camera_and_buffer_origin_move() {
        let mut fine = Buffer::empty(Rect::new(0, 0, 24, 12));
        let first = tile(
            &mut fine,
            Rect::new(0, 0, 12, 12),
            &[[80, 150, 95]; 256],
            46,
            12,
            0.,
        );
        let second = tile(
            &mut fine,
            Rect::new(12, 0, 12, 12),
            &[[80, 150, 95]; 256],
            47,
            12,
            0.,
        );
        let mut sizes = vec![0; 24 * 12];
        for y in 0..12 {
            sizes[y * 24..y * 24 + 12].copy_from_slice(&first[y * 12..y * 12 + 12]);
            sizes[y * 24 + 12..y * 24 + 24].copy_from_slice(&second[y * 12..y * 12 + 12]);
        }
        let mut wide = Buffer::empty(Rect::new(0, 0, 12, 3));
        pack_terminal_dots(&mut wide, Rect::new(0, 0, 12, 3), &fine, &sizes, 46, 12);
        let mut narrow_fine = Buffer::empty(Rect::new(0, 0, 12, 12));
        for y in 0..12 {
            for x in 0..12 {
                narrow_fine[(x, y)] = fine[(x + 12, y)].clone();
            }
        }
        let mut narrow = Buffer::empty(Rect::new(0, 0, 9, 5));
        pack_terminal_dots(
            &mut narrow,
            Rect::new(2, 1, 6, 3),
            &narrow_fine,
            &second,
            47,
            12,
        );
        for y in 0..3 {
            for x in 0..6 {
                assert_eq!(wide[(x + 6, y)], narrow[(x + 2, y + 1)]);
            }
        }
    }
    #[test]
    fn real_tile_colors_and_shapes_survive_dithering() {
        let mut pixels = [[32, 80, 128]; 256];
        pixels[..4].fill([8, 16, 24]);
        for row in 0..16 {
            for col in 8..16 {
                pixels[row * 16 + col] = [160, 40, 20];
            }
        }
        let mut b = Buffer::empty(Rect::new(0, 0, 16, 16));
        let sizes = tile(&mut b, Rect::new(0, 0, 16, 16), &pixels, 46, 12, 0.);
        assert!(
            sizes
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                > 12
        );
        assert!(b.content.iter().all(|c| DOTS.contains(&c.symbol())));
        for y in 0..16 {
            for x in 0..16 {
                let Color::Rgb(r, g, blue) = b[(x, y)].fg else {
                    panic!("RGB ink");
                };
                if x < 8 {
                    assert!(blue > g && g > r);
                } else {
                    assert!(r > g && g > blue);
                }
            }
        }
        assert!(b.content.iter().any(|c| c.symbol() == "•"));
        assert!(b.content.iter().any(|c| c.symbol() == "·"));
        assert!(b.content.iter().all(|c| c.bg == PAPER));
    }
    #[test]
    fn moving_the_viewport_does_not_swim_the_dither() {
        let mut a = Buffer::empty(Rect::new(0, 0, 8, 4));
        let mut b = Buffer::empty(Rect::new(0, 0, 20, 12));
        tile(
            &mut a,
            Rect::new(0, 0, 8, 4),
            &[[80, 150, 95]; 256],
            46,
            12,
            0.,
        );
        tile(
            &mut b,
            Rect::new(7, 5, 8, 4),
            &[[80, 150, 95]; 256],
            46,
            12,
            0.,
        );
        for y in 0..4 {
            for x in 0..8 {
                assert_eq!(a[(x, y)], b[(x + 7, y + 5)]);
            }
        }
    }
    #[test]
    fn identical_tiles_have_different_world_space_dot_treatments() {
        let mut a = Buffer::empty(Rect::new(0, 0, 16, 16));
        let mut b = a.clone();
        let first = tile(
            &mut a,
            Rect::new(0, 0, 16, 16),
            &[[80, 150, 95]; 256],
            46,
            12,
            0.,
        );
        let second = tile(
            &mut b,
            Rect::new(0, 0, 16, 16),
            &[[80, 150, 95]; 256],
            47,
            12,
            0.,
        );
        assert_ne!(first, second, "Dot style is not repeated with the metatile");
    }
}
