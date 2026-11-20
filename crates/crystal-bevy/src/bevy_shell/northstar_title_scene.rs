//! Native title choreography independent of the GPU presentation adapter.
use anyhow::{Result, ensure};
use image::{Rgba, RgbaImage};

use super::northstar_opening::REVEAL_FRAMES;
use super::northstar_title_creature::{Creature, is_creature_pixel};
use super::{northstar_title_stage, northstar_title_water};

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;
const MIST: Rgba<u8> = Rgba([198, 231, 231, 255]);
const GOLD: Rgba<u8> = Rgba([255, 220, 123, 255]);

pub(super) struct Scene {
    title: RgbaImage,
    background: RgbaImage,
    creature: Creature,
}

impl Scene {
    pub(super) fn new(title: RgbaImage) -> Result<Self> {
        ensure!(
            title.dimensions() == (WIDTH, HEIGHT),
            "native Northstar title dimensions"
        );
        let creature = Creature::from_title(&title);
        let background = scene_backing(&title);
        Ok(Self {
            title,
            background,
            creature,
        })
    }

    pub(super) fn compose(&self, frame: u32) -> RgbaImage {
        let ambient = frame.saturating_sub(REVEAL_FRAMES);
        let mut out = self.background.clone();
        // Water never freezes during the entrance. Wrapping back by 480 frames
        // preserves its phase when the one-shot choreography becomes idle.
        northstar_title_water::draw(&mut out, &self.title, frame);
        northstar_title_stage::draw(&mut out, &self.title, frame);
        if frame < REVEAL_FRAMES {
            self.creature.draw_entrance(&mut out, frame);
        } else {
            self.creature.draw(&mut out, ambient);
        }
        // Native palette steps bring the complete silhouette out of the dark;
        // no horizontal wipe cuts across the face or separates the limbs.
        if frame < 64 {
            for y in 44..132 {
                for x in 24..136 {
                    let source = *out.get_pixel(x, y);
                    if is_creature_pixel(x, y, source) {
                        out.put_pixel(x, y, entrance_color(source, frame));
                    }
                }
            }
        }
        self.draw_lettering(&mut out, frame, ambient);
        out
    }

    fn draw_lettering(&self, out: &mut RgbaImage, frame: u32, ambient: u32) {
        if frame >= 40 {
            let pokemon_offset = entrance_offset(frame, 40, 24, 4);
            let gold = if frame < 52 {
                Rgba([189, 154, 94, 255])
            } else {
                GOLD
            };
            for y in 4..22 {
                for x in 0..WIDTH {
                    if *self.title.get_pixel(x, y) == GOLD {
                        dot(out, x as i32, y as i32 - pokemon_offset, gold);
                    }
                }
            }
        }
        // The name enters as one readable word on the creature's release.
        if frame >= 84 {
            let offset = entrance_offset(frame, 84, 24, 2);
            let sweep = if (108..152).contains(&frame) {
                20 + (frame - 108) as i32 * 3
            } else if frame >= REVEAL_FRAMES {
                (ambient / 2) as i32 - 32
            } else {
                -32
            };
            for y in 28..42 {
                for x in 27..133 {
                    if *self.title.get_pixel(x, y) != MIST {
                        continue;
                    }
                    let pixel = if (x as i32 - sweep).abs() < 3 {
                        Rgba([240, 250, 238, 255])
                    } else if frame < 96 {
                        Rgba([123, 190, 198, 255])
                    } else {
                        MIST
                    };
                    dot(out, x as i32, y as i32 + offset, pixel);
                }
            }
        }
        // Input remains available throughout the reveal; keep the prompt legible.
        for y in 133..140 {
            for x in 0..WIDTH {
                if *self.title.get_pixel(x, y) == GOLD {
                    out.put_pixel(x, y, GOLD);
                }
            }
        }
    }
}

fn entrance_color(source: Rgba<u8>, frame: u32) -> Rgba<u8> {
    if frame < 16 {
        return Rgba([24, 44, 68, 255]);
    }
    let color = match (
        frame < 40,
        source.0[..3] == [0, 0, 0],
        source.0[..3] == [99, 206, 239],
    ) {
        (true, true, _) => [10, 22, 41, 255],
        (true, false, true) => [43, 81, 113, 255],
        (true, false, false) => [31, 43, 84, 255],
        (false, true, _) => [0, 0, 0, 255],
        (false, false, true) => [68, 134, 172, 255],
        (false, false, false) => [44, 50, 126, 255],
    };
    Rgba(color)
}

fn entrance_offset(frame: u32, start: u32, duration: u32, distance: u32) -> i32 {
    let remaining = duration.saturating_sub(frame.saturating_sub(start));
    (distance * remaining * remaining / (duration * duration)) as i32
}

fn scene_backing(title: &RgbaImage) -> RgbaImage {
    let mut background = title.clone();
    for (x, y, pixel) in background.enumerate_pixels_mut() {
        if is_creature_pixel(x, y, *pixel) || *pixel == GOLD || *pixel == MIST {
            let boundary = 62 + 28 * (y as i32 - 72).unsigned_abs() / 72;
            *pixel = if x >= boundary {
                Rgba([30, 62, 82, 255])
            } else {
                Rgba([16, 32, 57, 255])
            };
        }
    }
    background
}

fn dot(out: &mut RgbaImage, x: i32, y: i32, pixel: Rgba<u8>) {
    if (0..WIDTH as i32).contains(&x) && (0..HEIGHT as i32).contains(&y) {
        out.put_pixel(x as u32, y as u32, pixel);
    }
}
