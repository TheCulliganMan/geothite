//! Kettlurk's planted stance, attentive upper body, and returning currents.
//!
//! The title supplies every creature pixel. A native-grid inverse rig keeps
//! the face rigid, bends the chest around a fixed hip, and joins both wrists
//! to their shoulders. Empty source pixels remain empty through every pose.

use image::{Rgba, RgbaImage};

const CYAN: Rgba<u8> = Rgba([99, 206, 239, 255]);
const DEEP_WATER: Rgba<u8> = Rgba([57, 57, 165, 255]);
const OUTLINE: Rgba<u8> = Rgba([0, 0, 0, 255]);
const GRID_WIDTH: usize = 80;
const GRID_HEIGHT: usize = 72;
const HEAD_BASE: i32 = 72;
const CHEST: i32 = 92;
// Both legs begin below this seam. Keeping it fixed preserves their complete
// contours, not merely the bottom row where the feet touch the water.
const HIP: i32 = 104;

// Follow the visible material toward the near rim of the hollow torso. The
// path ends at that rim: it never paints the empty space inside the creature.
const INWARD_CURRENT: &[(i32, i32)] = &[
    (38, 110),
    (38, 102),
    (46, 96),
    (52, 88),
    (60, 82),
    (68, 80),
    (72, 84),
    (74, 90),
    (80, 94),
];

// The second current appears behind the far shoulder after the first has
// vanished, then pours along the other arm. This is Kettlurk's own anatomy.
const RETURNING_CURRENT: &[(i32, i32)] = &[
    (82, 102),
    (92, 98),
    (94, 88),
    (94, 78),
    (92, 72),
    (98, 70),
    (106, 80),
    (114, 94),
    (122, 104),
    (124, 112),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    Body,
    LeftForearm,
    RightForearm,
}

#[derive(Clone, Copy, Default)]
struct Offset {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Default)]
struct Pose {
    head: Offset,
    chest: Offset,
    left_hand: Offset,
    right_hand: Offset,
}

const REST: Pose = Pose {
    head: Offset { x: 0, y: 0 },
    chest: Offset { x: 0, y: 0 },
    left_hand: Offset { x: 0, y: 0 },
    right_hand: Offset { x: 0, y: 0 },
};
const NEAR_DIP: Pose = Pose {
    head: Offset { x: -2, y: 4 },
    chest: Offset { x: -2, y: 2 },
    left_hand: Offset { x: 0, y: -2 },
    right_hand: Offset { x: 2, y: 0 },
};
const NEAR_OPEN: Pose = Pose {
    head: Offset { x: -4, y: 2 },
    chest: Offset { x: -2, y: 0 },
    left_hand: Offset { x: 0, y: -4 },
    right_hand: Offset { x: 2, y: 0 },
};
const FAR_DIP: Pose = Pose {
    head: Offset { x: 2, y: 4 },
    chest: Offset { x: 2, y: 2 },
    left_hand: Offset { x: -2, y: 0 },
    right_hand: Offset { x: 0, y: -2 },
};
const FAR_OPEN: Pose = Pose {
    head: Offset { x: 4, y: 2 },
    chest: Offset { x: 2, y: 0 },
    left_hand: Offset { x: -2, y: 0 },
    right_hand: Offset { x: 0, y: -4 },
};
const RELEASE: Pose = Pose {
    head: Offset { x: 0, y: 0 },
    chest: Offset { x: 0, y: 0 },
    left_hand: Offset { x: -2, y: -4 },
    right_hand: Offset { x: 2, y: -4 },
};

#[derive(Clone, Copy)]
struct Pixel {
    color: Rgba<u8>,
    part: Part,
    inward_position: Option<u32>,
    returning_position: Option<u32>,
}

/// Native title pixels and their explicitly authored articulation/current paths.
pub(super) struct Creature {
    pixels: Vec<Option<Pixel>>,
}

impl Creature {
    /// Reads the current title's creature without changing or copying a catalog.
    pub(super) fn from_title(title: &RgbaImage) -> Self {
        let inward = current_points(INWARD_CURRENT);
        let returning = current_points(RETURNING_CURRENT);
        let mut pixels = vec![None; GRID_WIDTH * GRID_HEIGHT];
        // The installed creature is a 2x rendering of the native sprite.
        for y in (0..title.height().min(144)).step_by(2) {
            for x in (0..title.width().min(160)).step_by(2) {
                let color = *title.get_pixel(x, y);
                if !is_creature_pixel(x, y, color) {
                    continue;
                }
                // Near hand ends at x53; near leg begins at x58. The far leg
                // ends at x107 before the far hand. Neither leg is articulated.
                let part = if (96..122).contains(&y) && x < 56 {
                    Part::LeftForearm
                } else if (96..122).contains(&y) && x >= 108 {
                    Part::RightForearm
                } else {
                    Part::Body
                };
                pixels[y as usize / 2 * GRID_WIDTH + x as usize / 2] = Some(Pixel {
                    color,
                    part,
                    inward_position: current_position(x, y, &inward),
                    returning_position: current_position(x, y, &returning),
                });
            }
        }
        Self { pixels }
    }

    /// Draws over a scene whose original creature has already been removed.
    ///
    /// Frame zero is the exact authored still. Each 480-frame loop leans toward
    /// the near current, gathers it, then answers with a far-side release.
    /// The face translates as a rigid native patch; the chest bends above the
    /// fixed legs. Inverse sampling leaves no cracks between transformed rows.
    /// The caller owns title/input timing and reduced motion.
    pub(super) fn draw(&self, out: &mut RgbaImage, frame: u32) {
        let frame = frame % 480;
        let pose = pose_at(
            frame,
            &[
                (0, REST),
                (32, REST),
                (64, NEAR_DIP),
                (112, NEAR_OPEN),
                (144, NEAR_OPEN),
                (192, REST),
                (232, REST),
                (264, FAR_DIP),
                (312, FAR_OPEN),
                (352, FAR_OPEN),
                (416, REST),
                (480, REST),
            ],
        );
        self.draw_pose(out, pose, Some(frame));
    }

    /// Performs the title's 180-frame anticipation, open-handed release, and rest.
    ///
    /// The body is at rest through frame 24, dips by 56, releases from 80 to
    /// 120, and settles by 160. This only draws pixels; it never delays input.
    pub(super) fn draw_entrance(&self, out: &mut RgbaImage, frame: u32) {
        let pose = pose_at(
            frame.min(180),
            &[
                (0, REST),
                (24, REST),
                (56, NEAR_DIP),
                (80, RELEASE),
                (120, RELEASE),
                (160, REST),
                (180, REST),
            ],
        );
        let current = match frame {
            56..112 => Some(32 + (frame - 56) * 2),
            112..176 => Some(256 + (frame - 112) * 3),
            _ => None,
        };
        self.draw_pose(out, pose, current);
    }

    fn draw_pose(&self, out: &mut RgbaImage, pose: Pose, current: Option<u32>) {
        for part in [Part::Body, Part::LeftForearm, Part::RightForearm] {
            for y in (44..132).step_by(2) {
                let source_y = native_coordinate(source_y(y, pose, part));
                for x in (20..140).step_by(2) {
                    let source_x = native_coordinate(x - horizontal_offset(source_y, pose, part));
                    let Some(pixel) = self.source_pixel(source_x, source_y) else {
                        continue;
                    };
                    if pixel.part != part {
                        continue;
                    }
                    let color = current.map_or(pixel.color, |phase| current_color(pixel, phase));
                    for dy in 0..2 {
                        for dx in 0..2 {
                            put_pixel(out, x + dx, y + dy, color);
                        }
                    }
                }
            }
        }
    }

    fn source_pixel(&self, x: i32, y: i32) -> Option<&Pixel> {
        if !(0..160).contains(&x) || !(0..144).contains(&y) {
            return None;
        }
        self.pixels[y as usize / 2 * GRID_WIDTH + x as usize / 2].as_ref()
    }
}

/// Identifies only the installed title's three-color creature, including ink.
///
/// Scene backing, the state-flag star, water marks, and lettering use different
/// colors. The caller may use this mask to restore their backing before draw.
pub(super) fn is_creature_pixel(x: u32, y: u32, pixel: Rgba<u8>) -> bool {
    (24..136).contains(&x) && (44..132).contains(&y) && matches!(pixel, CYAN | DEEP_WATER | OUTLINE)
}

fn pose_at(frame: u32, keyframes: &[(u32, Pose)]) -> Pose {
    for pair in keyframes.windows(2) {
        let (start, from) = pair[0];
        let (end, to) = pair[1];
        if frame <= end {
            let blend = |a: Offset, b: Offset| Offset {
                x: interpolate(frame as i32, start as i32, end as i32, a.x, b.x),
                y: interpolate(frame as i32, start as i32, end as i32, a.y, b.y),
            };
            return Pose {
                head: blend(from.head, to.head),
                chest: blend(from.chest, to.chest),
                left_hand: blend(from.left_hand, to.left_hand),
                right_hand: blend(from.right_hand, to.right_hand),
            };
        }
    }
    REST
}

fn source_y(target: i32, pose: Pose, part: Part) -> i32 {
    let hand = match part {
        Part::Body => {
            if target <= HEAD_BASE + pose.head.y {
                return target - pose.head.y;
            }
            if target <= CHEST + pose.chest.y {
                return interpolate(
                    target,
                    HEAD_BASE + pose.head.y,
                    CHEST + pose.chest.y,
                    HEAD_BASE,
                    CHEST,
                );
            }
            if target <= HIP {
                return interpolate(target, CHEST + pose.chest.y, HIP, CHEST, HIP);
            }
            return target;
        }
        Part::LeftForearm => pose.left_hand,
        Part::RightForearm => pose.right_hand,
    };
    let wrist = 94 + body_offset(94, pose.head.y, pose.chest.y);
    if target <= 108 + hand.y {
        interpolate(target, wrist, 108 + hand.y, 94, 108)
    } else {
        target - hand.y
    }
}

fn horizontal_offset(y: i32, pose: Pose, part: Part) -> i32 {
    let body = body_offset(y, pose.head.x, pose.chest.x);
    let hand = match part {
        Part::Body => return body,
        Part::LeftForearm => pose.left_hand,
        Part::RightForearm => pose.right_hand,
    };
    body + interpolate(y, 94, 108, 0, hand.x)
}

fn body_offset(y: i32, head: i32, chest: i32) -> i32 {
    if y <= HEAD_BASE {
        head
    } else if y <= CHEST {
        interpolate(y, HEAD_BASE, CHEST, head, chest)
    } else {
        interpolate(y, CHEST, HIP, chest, 0)
    }
}

fn interpolate(value: i32, start: i32, end: i32, from: i32, to: i32) -> i32 {
    from + (to - from) * (value - start).clamp(0, end - start) / (end - start)
}

fn native_coordinate(value: i32) -> i32 {
    (value + 1).div_euclid(2) * 2
}

fn current_color(pixel: &Pixel, frame: u32) -> Rgba<u8> {
    if pixel.color == OUTLINE {
        return pixel.color;
    }
    let active = if (32..224).contains(&frame) {
        pixel
            .inward_position
            .map(|position| (position, (frame - 32) / 4))
    } else if (256..464).contains(&frame) {
        pixel
            .returning_position
            .map(|position| (position, (frame - 256) / 4))
    } else {
        None
    };
    let Some((position, head)) = active else {
        return pixel.color;
    };
    let distance = i64::from(head) - i64::from(position);
    match distance {
        -2..=2 => CYAN,
        3..=5 => DEEP_WATER,
        _ => pixel.color,
    }
}

fn current_points(path: &[(i32, i32)]) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    for segment in path.windows(2) {
        let (start_x, start_y) = segment[0];
        let (end_x, end_y) = segment[1];
        let steps = (end_x - start_x).abs().max((end_y - start_y).abs()) / 2;
        for step in 0..steps {
            points.push((
                start_x + (end_x - start_x) * step / steps,
                start_y + (end_y - start_y) * step / steps,
            ));
        }
    }
    if let Some(&last) = path.last() {
        points.push(last);
    }
    points
}

fn current_position(x: u32, y: u32, points: &[(i32, i32)]) -> Option<u32> {
    let mut nearest = None;
    let mut nearest_distance = 25;
    for (index, &(path_x, path_y)) in points.iter().enumerate() {
        // Sample the whole 2x2 native source block together, preventing noisy
        // single-display-pixel sparks inside the creature's original pixels.
        let dx = (x / 2 * 2) as i32 - path_x;
        let dy = (y / 2 * 2) as i32 - path_y;
        let distance = dx * dx + dy * dy;
        if distance < nearest_distance {
            nearest = Some(index as u32);
            nearest_distance = distance;
        }
    }
    nearest
}

fn put_pixel(out: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x >= 0 && y >= 0 && x < out.width() as i32 && y < out.height() as i32 {
        out.put_pixel(x as u32, y as u32, color);
    }
}
