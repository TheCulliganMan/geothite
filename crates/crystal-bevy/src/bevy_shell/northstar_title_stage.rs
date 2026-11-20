//! One star-led entrance, followed by a quiet pulse before Kettlurk gathers.
use image::{Rgba, RgbaImage};

use super::northstar_opening::{LOOP_FRAMES, REVEAL_FRAMES};

const STAR: Rgba<u8> = Rgba([68, 101, 120, 255]);
const WATER: Rgba<u8> = Rgba([74, 139, 156, 255]);
const LIGHT: Rgba<u8> = Rgba([123, 190, 198, 255]);
const MIST: Rgba<u8> = Rgba([198, 231, 231, 255]);

/// Draws the existing star and two brief side-water crests behind the creature.
/// The hollow, planted feet and lettering are outside this layer's paint area.
pub(super) fn draw(out: &mut RgbaImage, title: &RgbaImage, frame: u32) {
    if frame < REVEAL_FRAMES {
        if let Some(radius) = entrance_radius(frame) {
            illuminate_star(out, title, radius, Some((radius / 4).min(6)));
        }
        if (44..80).contains(&frame) {
            gather_water(out, title, ((frame - 44) / 4) as usize);
        }
    } else {
        let phase = (frame - REVEAL_FRAMES) % LOOP_FRAMES;
        if (8..40).contains(&phase) {
            // A small, low-contrast core pulse anticipates the near-side hand.
            // The broad entrance illumination never repeats in the idle loop.
            let radius = [0, 2, 4, 5, 5, 4, 2, 0][((phase - 8) / 4) as usize];
            illuminate_star(out, title, radius, None);
        }
    }
}

fn entrance_radius(frame: u32) -> Option<i32> {
    match frame {
        12..36 => Some((frame - 12) as i32),
        36..68 => Some(24 - ((frame - 36) * 24 / 32) as i32),
        _ => None,
    }
}

fn illuminate_star(out: &mut RgbaImage, title: &RgbaImage, radius: i32, bright_core: Option<i32>) {
    // Use the actual visible eight-point shape from the title, not a second
    // geometric star. Native palette bands advance through that fixed shape.
    for y in 55..90 {
        for x in 17..52 {
            if x >= title.width()
                || y >= title.height()
                || x >= out.width()
                || y >= out.height()
                || *title.get_pixel(x, y) != STAR
            {
                continue;
            }
            let distance = (x as i32 - 34).abs() + (y as i32 - 72).abs();
            if distance <= radius {
                let color = if bright_core.is_some_and(|core| distance <= core) {
                    MIST
                } else {
                    LIGHT
                };
                out.put_pixel(x, y, color);
            }
        }
    }
}

fn gather_water(out: &mut RgbaImage, title: &RgbaImage, age: usize) {
    // Each crest makes one short inward approach along the side of the scene.
    // It stops outside the creature crop; the creature owns the next motion.
    const LEFT: [(u32, u32); 9] = [
        (2, 123),
        (5, 123),
        (8, 122),
        (11, 120),
        (14, 117),
        (17, 114),
        (20, 111),
        (22, 108),
        (22, 105),
    ];
    const RIGHT: [(u32, u32); 9] = [
        (157, 123),
        (154, 123),
        (151, 122),
        (148, 120),
        (145, 117),
        (142, 114),
        (139, 111),
        (137, 108),
        (137, 105),
    ];
    for (path, left) in [(LEFT, true), (RIGHT, false)] {
        let (x, y) = path[age];
        side_water_pixel(out, title, x, y, LIGHT);
        side_water_pixel(out, title, if left { x + 1 } else { x - 1 }, y, LIGHT);
        if age > 0 {
            let (tail_x, tail_y) = path[age - 1];
            side_water_pixel(out, title, tail_x, tail_y, WATER);
        }
    }
}

fn side_water_pixel(out: &mut RgbaImage, title: &RgbaImage, x: u32, y: u32, color: Rgba<u8>) {
    if (24..136).contains(&x)
        || y >= 128
        || x >= title.width()
        || y >= title.height()
        || x >= out.width()
        || y >= out.height()
    {
        return;
    }
    if matches!(
        title.get_pixel(x, y).0[..3],
        [16, 32, 57] | [30, 62, 82] | [74, 139, 156] | [123, 190, 198]
    ) {
        out.put_pixel(x, y, color);
    }
}
