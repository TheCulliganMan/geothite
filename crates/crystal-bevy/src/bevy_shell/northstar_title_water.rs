//! Falling ribbons and small impact rings on the authored Northstar title water.
use image::{Rgba, RgbaImage};

const DEEP: Rgba<u8> = Rgba([74, 139, 156, 255]);
const LIGHT: Rgba<u8> = Rgba([123, 190, 198, 255]);
const FOAM: Rgba<u8> = Rgba([198, 231, 231, 255]);

/// Draws behind the creature, preserving every non-water foreground source pixel.
/// All motion periods divide the title's 480-frame loop.
pub(super) fn draw(out: &mut RgbaImage, title: &RgbaImage, frame: u32) {
    let frame = (frame % 480) as i32;
    for (left, top, offset) in [(10, 84, 0), (146, 70, 24)] {
        // Offset the four ribbons so the fall pours downward rather than
        // flashing as one horizontal bar. Dark gaps separate the foam heads.
        for y in top..120 {
            for (column, stagger) in [0, 2, 5, 1].into_iter().enumerate() {
                let phase = (y - top - frame / 3 + stagger + offset).rem_euclid(16);
                let color = match phase {
                    0..=1 => FOAM,
                    2..=4 | 12..=15 => LIGHT,
                    _ => DEEP,
                };
                paint(out, title, left + column as i32, y, color);
            }
        }
        impact_ring(out, title, left + 1, 121, frame + offset);
        spray(out, title, left + 1, frame + offset);
    }

    for (index, (left, y, length)) in [(17, 119, 23), (55, 122, 20), (88, 117, 22), (120, 124, 25)]
        .into_iter()
        .enumerate()
    {
        // Light travels through the existing pool strokes. One short reflected
        // edge below each stroke grows outward and then dissolves.
        for x in left..left + length {
            let phase = (x - left - frame / 4 + index as i32 * 3).rem_euclid(12);
            let color = match phase {
                0..=1 => FOAM,
                2..=4 => LIGHT,
                _ => DEEP,
            };
            paint(out, title, x, y, color);
        }
        let age = (frame / 4 + index as i32 * 5).rem_euclid(20);
        if age < 16 {
            let center = left + length / 2;
            let radius = 2 + age / 2;
            let color = if age < 8 { LIGHT } else { DEEP };
            for sign in [-1, 1] {
                let x = center + sign * radius;
                paint(out, title, x, y + 2, color);
                paint(out, title, x + sign, y + 2, color);
            }
        }
    }
}

fn impact_ring(out: &mut RgbaImage, title: &RgbaImage, x: i32, y: i32, frame: i32) {
    let age = (frame / 4).rem_euclid(20);
    let radius = 3 + age / 2;
    let color = if age < 9 { LIGHT } else { DEEP };
    // Flat, broken ellipses suggest a shallow pool without a filled-in basin.
    for sign in [-1, 1] {
        paint(out, title, x + sign * radius, y, color);
        paint(out, title, x + sign * (radius - 1), y + 1, color);
        for dx in radius / 2..radius - 2 {
            paint(out, title, x + sign * dx, y + 2, color);
        }
    }
    if age < 4 {
        for dx in -2..=2 {
            paint(out, title, x + dx, y - 1 + (dx.abs() / 2), FOAM);
        }
    }
}

fn spray(out: &mut RgbaImage, title: &RgbaImage, x: i32, frame: i32) {
    // Two staggered droplets follow six authored pixel positions. Each returns
    // to the pool before disappearing; nothing drifts around the title text.
    const ARC: [(i32, i32); 6] = [(2, -1), (3, -3), (5, -4), (6, -3), (7, -1), (8, 1)];
    for (sign, offset) in [(-1, 0), (1, 10)] {
        let age = (frame / 4 + offset).rem_euclid(20) as usize;
        if let Some(&(dx, dy)) = ARC.get(age) {
            paint(out, title, x + sign * dx, 119 + dy, LIGHT);
            if age < 3 {
                paint(out, title, x + sign * dx, 118 + dy, FOAM);
            }
        }
    }
}

fn paint(out: &mut RgbaImage, title: &RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    let (Ok(x), Ok(y)) = (u32::try_from(x), u32::try_from(y)) else {
        return;
    };
    if x >= title.width() || y >= title.height() || x >= out.width() || y >= out.height() {
        return;
    }
    // The original flag background is the only additional drawing surface.
    // The hero, star and lettering remain intact even where water passes them.
    if matches!(
        title.get_pixel(x, y).0[..3],
        [16, 32, 57] | [30, 62, 82] | [74, 139, 156] | [123, 190, 198]
    ) {
        out.put_pixel(x, y, color);
    }
}
