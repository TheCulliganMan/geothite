//! Native 8 by 8, 2bpp badge tiles on the 160 by 144 Trainer Card.
//!
//! Every animation frame retains the badge's silhouette. Only earned badges
//! catch the light; unearned slots are still, recessed impressions.

pub(super) const NAMES: [&str; 8] = [
    "CARE", "PRAIRIE", "CURRENT", "CAPITOL", "WIND", "LAKE", "PINE", "HARBOR",
];

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const INK: [u8; 3] = [32, 48, 64];
const PAPER: [u8; 3] = [248, 248, 224];
const PALE: [u8; 3] = [216, 224, 208];
const MUTED: [u8; 3] = [136, 160, 152];
const GOLD: [u8; 3] = [200, 152, 64];
const WHITE: [u8; 3] = [255, 255, 240];

// Source patterns compile to nine native 8 by 8, 2bpp tiles per badge.
// c/g use color 2, l/w use color 3, and s/contours use color 1.
// Color 0 is transparent. Each badge uses one four-color sprite palette.
const ART: [[&str; 20]; 8] = [
    [
        "",
        "",
        "...gggg....gggg",
        "..glcllg..gllclg",
        ".glcccclgglcccclg",
        ".glcccccllcccccclg",
        ".glcccccccccccclg",
        ".glccccwwcccccclg",
        "..glcccwwccccclg",
        "..glcwwwwwwccclg",
        "...glccwwcccclg",
        "...glccwwcccclg",
        "....glcccccclg",
        ".....glcccclg",
        "......glcclg",
        ".......gllg",
        "........gg",
        "",
        "",
        "",
    ],
    [
        ".........g",
        "........glg",
        "........glg",
        "....ll..glg..ll",
        "...lccg.glg.gccl",
        "....lccgglggccl",
        ".....lccglgccl",
        "...ll.lgglggl.ll",
        "..lccg..glg..gccl",
        "...lccg.glg.gccl",
        "....lccgglggccl",
        ".....lccglgccl",
        "......lgglggl",
        "....ll..glg..ll",
        "...lccggglgggccl",
        "....lccgglggccl",
        ".....lllglglll",
        "........glg",
        ".......ggggg",
        "........ggg",
    ],
    [
        "..........ggggg",
        ".........glclg",
        "........glcclg",
        ".......glccclg",
        "......glccclg",
        ".....glcccclg",
        "....glccccclg",
        "...glcccccclggggg",
        "..glccccccccccclg",
        "..ggggggcccccclg",
        ".......glcccclg",
        ".......glccclg",
        "......glccclg",
        "......glcclg",
        ".....glcclg",
        ".....glclg",
        "....glclg",
        "....gllg",
        "...glg",
        "...gg",
    ],
    [
        ".........g",
        "........gwg",
        ".........g",
        "........ggg",
        "......gglllgg",
        ".....glllccllg",
        "....glllccccclg",
        "...glllccccccclg",
        "...gllcccccccclg",
        "..ggggggggggggggg",
        "..glllllllllllllg",
        "...ggggggggggggg",
        "...glgclgclgclg",
        "...glgclgclgclg",
        "...glgclgclgclg",
        "...glgclgclgclg",
        "..ggggggggggggggg",
        "..glllllllllllllg",
        ".ggggggggggggggggg",
        "",
    ],
    [
        "",
        "",
        "................ll",
        "..............llcl",
        "...........lllcccl",
        "........lllcccccl",
        ".....lllcccccccl",
        "...llccccccccll",
        "..lccccccccll..ll",
        ".lcccccccll..llcl",
        ".lccccccl..llccl",
        "..lccccclllcccl",
        "...lccccccccll",
        "....lcccccll",
        ".....lllll",
        "........ss",
        ".....sssss",
        "...sss",
        "",
        "",
    ],
    [
        ".........l",
        "........llc",
        ".......llccc",
        "......llccccc",
        ".....llccccccc",
        "....llccccccccc",
        "...llccccccccccc",
        "..llccccccccccccc",
        "..lcccccccccccccc",
        "..lcccccccccccccc",
        "..lccccclllcccccc",
        "..lcccclwwwlccccc",
        "...lccclwwlccccc",
        "....lcccllccccc",
        ".....lcccccccc",
        "......lllllll",
        "",
        "...ggg.....ggg",
        "..glllggggglllg",
        "....ggggggggg",
    ],
    [
        ".........l",
        "........llc",
        ".......llccc",
        "......llccccc",
        ".....llccccccc",
        ".......lcccc",
        "......llccccc",
        ".....llccccccc",
        "....llccccccccc",
        "...llccccccccccc",
        "......lcccccc",
        ".....llccccccc",
        "....llccccccccc",
        "...llccccccccccc",
        "..llccccccccccccc",
        ".....sssssssss",
        "........ggg",
        "........glg",
        ".......ggggg",
        "",
    ],
    [
        "........gggg",
        ".......glwwlg",
        ".......gl..lg",
        ".......gl..lg",
        "........gllg",
        ".........gg",
        ".........gl",
        ".....ggggllgggg",
        ".....gllllllllg",
        ".....ggggllgggg",
        ".........gl",
        "..gg.....gl.....gg",
        "..glg....gl....glg",
        "..glcg...gl...gclg",
        "..glclg..gl..glclg",
        "...glclggllgglclg",
        "....glcllllllclg",
        ".....gllccccllg",
        ".......gllllg",
        ".........gg",
    ],
];

// Shadow, enamel, light: deliberately stepped, Game Boy-style color ramps.
const COLORS: [[[u8; 3]; 3]; 8] = [
    [[152, 48, 72], [232, 88, 112], [255, 240, 216]],
    [[152, 96, 40], [232, 184, 72], [255, 232, 144]],
    [[40, 104, 144], [64, 192, 224], [168, 248, 248]],
    [[88, 112, 136], [168, 192, 208], [232, 240, 240]],
    [[72, 104, 168], [136, 184, 240], [216, 240, 255]],
    [[40, 80, 160], [64, 136, 224], [128, 216, 248]],
    [[32, 96, 72], [56, 152, 104], [144, 216, 136]],
    [[144, 72, 48], [224, 128, 72], [255, 208, 136]],
];

/// Composes one native frame; `frame` wraps after eight steps at eight ticks each.
pub(super) fn render(owned: &[bool; 8], frame: u8) -> Vec<u8> {
    let mut pixels = vec![0; WIDTH * HEIGHT * 4];
    for pixel in pixels.as_chunks_mut::<4>().0 {
        pixel.copy_from_slice(&[PAPER[0], PAPER[1], PAPER[2], 255]);
    }
    rect(&mut pixels, 2, 2, 156, 140, INK);
    rect(&mut pixels, 3, 3, 154, 138, GOLD);
    rect(&mut pixels, 4, 4, 152, 136, PAPER);
    rect(&mut pixels, 5, 5, 150, 26, INK);
    for (index, tile) in TITLE_TILES.iter().enumerate() {
        draw_tile(
            &mut pixels,
            tile,
            &[INK, PAPER, PAPER, PAPER],
            16 + index * 8,
            10,
        );
    }
    text(&mut pixels, "GYM BADGES", 16, 23, 1, [168, 200, 200]);
    let count = owned.iter().filter(|badge| **badge).count();
    text(
        &mut pixels,
        &format!("{count}/8"),
        130,
        22,
        1,
        [248, 216, 128],
    );
    for (slot, earned) in owned.iter().copied().enumerate() {
        let x = 9 + (slot % 4) * 36;
        let y = 37 + (slot / 4) * 44;
        // Small stepped recesses give the set a fitted presentation case.
        rect(&mut pixels, x + 1, y, 30, 31, PALE);
        rect(&mut pixels, x, y + 1, 32, 29, PALE);
        rect(&mut pixels, x + 1, y + 30, 30, 1, MUTED);
        draw_badge(&mut pixels, slot, earned, frame & 7, x + 6, y + 4);
        let label_x = x + (33 - NAMES[slot].len() * 4) / 2;
        text(&mut pixels, NAMES[slot], label_x, y + 34, 1, INK);
        if earned {
            rect(&mut pixels, x + 27, y + 25, 3, 3, GOLD);
            put(&mut pixels, x + 28, y + 24, WHITE);
        } else {
            text(&mut pixels, "-", x + 27, y + 25, 1, MUTED);
        }
    }
    rect(&mut pixels, 9, 126, 142, 1, MUTED);
    text(&mut pixels, "< INFO", 10, 131, 1, INK);
    text(&mut pixels, "A/B CLOSE", 115, 131, 1, INK);
    pixels
}

// The stored animation is tile bytes, with the same interleaved low/high bit
// planes as Crystal's native 2bpp assets. No RGBA badge atlas or generated file
// is needed in the pack. Nine tiles form a 24 by 24 sprite for each badge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Tile([u8; 16]);

const TILE_FRAMES: [[[Tile; 9]; 8]; 8] = compile_tiles();
const TITLE_TILES: [Tile; 9] = [
    title_tile([17, 27, 21, 21, 17, 17, 17]), // M
    title_tile([14, 4, 4, 4, 4, 4, 14]),      // I
    title_tile([17, 25, 21, 21, 19, 17, 17]), // N
    title_tile([17, 25, 21, 21, 19, 17, 17]), // N
    title_tile([31, 16, 16, 30, 16, 16, 31]), // E
    title_tile([15, 16, 16, 14, 1, 1, 30]),   // S
    title_tile([14, 17, 17, 17, 17, 17, 14]), // O
    title_tile([31, 4, 4, 4, 4, 4, 4]),       // T
    title_tile([14, 17, 17, 31, 17, 17, 17]), // A
];

const fn title_tile(rows: [u8; 7]) -> Tile {
    let mut tile = Tile([0; 16]);
    let mut row = 0;
    while row < 7 {
        tile.0[row * 2] = rows[row] << 2;
        row += 1;
    }
    tile
}

const fn source_color(slot: usize, x: i32, y: i32) -> u8 {
    if x < 0 || y < 0 || x >= 20 || y >= 20 {
        return 0;
    }
    let row = ART[slot][y as usize].as_bytes();
    if x as usize >= row.len() {
        return 0;
    }
    match row[x as usize] {
        b's' => 1,
        b'c' | b'g' => 2,
        b'l' | b'w' => 3,
        _ => 0,
    }
}

const fn frame_color(slot: usize, frame: usize, x: i32, y: i32) -> u8 {
    let color = source_color(slot, x, y);
    if color == 0 {
        return if source_color(slot, x - 1, y) != 0
            || source_color(slot, x + 1, y) != 0
            || source_color(slot, x, y - 1) != 0
            || source_color(slot, x, y + 1) != 0
        {
            1
        } else {
            0
        };
    }
    // Stagger the light across the eight badges, retaining every silhouette.
    let phase = (frame + slot * 3) & 7;
    if color == 2 && (x as usize + y as usize / 2 + 32 - phase * 4) % 32 < 2 {
        return 3;
    }
    let (glint_x, glint_y) = match slot {
        0 => (4 + frame, 6),
        1 => (8, 3 + frame * 2),
        2 => (5 + frame, 8),
        3 => (5 + frame, 7),
        4 => (5 + frame, 7),
        5 => (4 + frame, 9),
        6 => (5 + frame, 14),
        _ => [
            (9, 5),
            (9, 6),
            (5, 7),
            (14, 7),
            (9, 10),
            (9, 11),
            (9, 12),
            (9, 13),
        ][frame],
    };
    if color == 2 && x as usize == glint_x && y as usize == glint_y {
        3
    } else {
        color
    }
}

const fn compile_tiles() -> [[[Tile; 9]; 8]; 8] {
    let mut frames = [[[Tile([0; 16]); 9]; 8]; 8];
    let mut slot = 0;
    while slot < 8 {
        let mut frame = 0;
        while frame < 8 {
            let mut y = 0;
            while y < 24 {
                let mut x = 0;
                while x < 24 {
                    let color = frame_color(slot, frame, x as i32 - 2, y as i32 - 2);
                    let tile = &mut frames[slot][frame][(y / 8) * 3 + x / 8].0;
                    let bit = 1 << (7 - x % 8);
                    tile[(y % 8) * 2] |= (color & 1) * bit;
                    tile[(y % 8) * 2 + 1] |= ((color >> 1) & 1) * bit;
                    x += 1;
                }
                y += 1;
            }
            frame += 1;
        }
        slot += 1;
    }
    frames
}

fn draw_badge(pixels: &mut [u8], slot: usize, earned: bool, frame: u8, x: usize, y: usize) {
    let palette = if earned {
        [PAPER, INK, COLORS[slot][1], COLORS[slot][2]]
    } else {
        [PAPER, MUTED, PALE, PALE]
    };
    let frame = if earned { usize::from(frame & 7) } else { 0 };
    for (index, tile) in TILE_FRAMES[slot][frame].iter().enumerate() {
        draw_tile(
            pixels,
            tile,
            &palette,
            x + (index % 3) * 8 - 2,
            y + (index / 3) * 8 - 2,
        );
    }
}

fn draw_tile(pixels: &mut [u8], tile: &Tile, palette: &[[u8; 3]; 4], x: usize, y: usize) {
    for row in 0..8 {
        for column in 0..8 {
            let shift = 7 - column;
            let color =
                ((tile.0[row * 2] >> shift) & 1) | (((tile.0[row * 2 + 1] >> shift) & 1) << 1);
            if color != 0 {
                put(pixels, x + column, y + row, palette[usize::from(color)]);
            }
        }
    }
}

fn rect(pixels: &mut [u8], x: usize, y: usize, width: usize, height: usize, color: [u8; 3]) {
    for row in y..y + height {
        for column in x..x + width {
            put(pixels, column, row, color);
        }
    }
}

fn put(pixels: &mut [u8], x: usize, y: usize, color: [u8; 3]) {
    let offset = (y * WIDTH + x) * 4;
    pixels[offset..offset + 4].copy_from_slice(&[color[0], color[1], color[2], 255]);
}

fn text(pixels: &mut [u8], label: &str, x: usize, y: usize, scale: usize, color: [u8; 3]) {
    for (index, character) in label.bytes().enumerate() {
        let rows = match character {
            b'A' => [2, 5, 7, 5, 5],
            b'B' => [6, 5, 6, 5, 6],
            b'C' => [3, 4, 4, 4, 3],
            b'D' => [6, 5, 5, 5, 6],
            b'E' => [7, 4, 6, 4, 7],
            b'F' => [7, 4, 6, 4, 4],
            b'G' => [3, 4, 5, 5, 3],
            b'H' => [5, 5, 7, 5, 5],
            b'I' => [7, 2, 2, 2, 7],
            b'K' => [5, 5, 6, 5, 5],
            b'L' => [4, 4, 4, 4, 7],
            b'M' => [5, 7, 7, 5, 5],
            b'N' => [5, 7, 7, 7, 5],
            b'O' => [2, 5, 5, 5, 2],
            b'P' => [6, 5, 6, 4, 4],
            b'R' => [6, 5, 6, 5, 5],
            b'S' => [3, 4, 2, 1, 6],
            b'T' => [7, 2, 2, 2, 2],
            b'U' => [5, 5, 5, 5, 7],
            b'W' => [5, 5, 7, 7, 5],
            b'Y' => [5, 5, 2, 2, 2],
            b'0' => [7, 5, 5, 5, 7],
            b'1' => [2, 6, 2, 2, 7],
            b'2' => [6, 1, 2, 4, 7],
            b'3' => [6, 1, 2, 1, 6],
            b'4' => [5, 5, 7, 1, 1],
            b'5' => [7, 4, 6, 1, 6],
            b'6' => [3, 4, 6, 5, 2],
            b'7' => [7, 1, 2, 2, 2],
            b'8' => [2, 5, 2, 5, 2],
            b'/' => [1, 1, 2, 4, 4],
            b'<' => [1, 2, 4, 2, 1],
            b'-' => [0, 0, 7, 0, 0],
            _ => [0; 5],
        };
        for (row, bits) in rows.into_iter().enumerate() {
            for column in 0..3 {
                if bits & (4 >> column) != 0 {
                    rect(
                        pixels,
                        x + (index * 4 + column) * scale,
                        y + row * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ART, HEIGHT, TILE_FRAMES, WIDTH, frame_color, render};

    #[test]
    fn native_frames_are_nine_2bpp_tiles_with_a_stable_silhouette() {
        assert_eq!(std::mem::size_of_val(&TILE_FRAMES[0][0]), 9 * 16);
        for (slot, frames) in TILE_FRAMES.iter().enumerate() {
            for (frame, tiles) in frames.iter().enumerate() {
                for y in 0..24 {
                    for x in 0..24 {
                        let tile = &tiles[(y / 8) * 3 + x / 8].0;
                        let color = ((tile[(y % 8) * 2] >> (7 - x % 8)) & 1)
                            | (((tile[(y % 8) * 2 + 1] >> (7 - x % 8)) & 1) << 1);
                        assert_eq!(color, frame_color(slot, frame, x as i32 - 2, y as i32 - 2));
                        assert_eq!(
                            color == 0,
                            frame_color(slot, 0, x as i32 - 2, y as i32 - 2) == 0
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn art_fits_the_native_grid_and_uses_only_its_palette() {
        for badge in ART {
            for row in badge {
                assert!(row.len() <= 20, "badge row exceeds its grid: {row}");
                assert!(row.bytes().all(|symbol| b".sclgw".contains(&symbol)));
            }
        }
    }

    #[test]
    fn earned_badges_animate_individually_and_unearned_slots_stay_still() {
        let empty = render(&[false; 8], 0);
        assert_eq!(empty.len(), WIDTH * HEIGHT * 4);
        for frame in 1..8 {
            assert_eq!(empty, render(&[false; 8], frame));
        }
        for slot in 0..8 {
            let mut owned = [false; 8];
            owned[slot] = true;
            let frames: std::collections::HashSet<_> =
                (0..8).map(|frame| render(&owned, frame)).collect();
            assert_eq!(frames.len(), 8, "badge {slot} needs eight distinct frames");
            assert_eq!(render(&owned, 0), render(&owned, 8));
        }
    }
}
