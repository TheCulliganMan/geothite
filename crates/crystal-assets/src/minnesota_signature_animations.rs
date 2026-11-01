//! Original Minnesota move poses, played by the production Gen II OAM VM.
//!
//! Each pose is six 8×8 2bpp tiles in the existing 24×16 HORN layout. Reusing
//! that layout avoids changing any original frameset or OAM definition. Ink0
//! is transparent; the three visible inks use existing cartridge palettes.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::GameDataSet;

/// Signature IDs in their pack presentation-table order.
pub const SIGNATURE_MOVES: [&str; 5] = [
    "MAGNETIC_RAIL",
    "BOG_SIPHON",
    "TIP_UP_STRIKE",
    "SLIPPER_WALTZ",
    "PIKE_DRILL",
];

#[derive(Clone, Copy)]
enum Pose {
    Rail,
    Pellet,
    Needle,
    Pressure,
    FlagLow,
    FlagMiddle,
    FlagHigh,
    IceJaw,
    SlipperLeft,
    SlipperRight,
    Pike,
    PikeTurn,
}

const POSES: [(&str, Pose, &str); 12] = [
    ("RAIL", Pose::Rail, "GRAY"),
    ("PELLET", Pose::Pellet, "GRAY"),
    ("NEEDLE", Pose::Needle, "RED"),
    ("PRESSURE", Pose::Pressure, "RED"),
    ("FLAG_LOW", Pose::FlagLow, "RED"),
    ("FLAG_MIDDLE", Pose::FlagMiddle, "RED"),
    ("FLAG_HIGH", Pose::FlagHigh, "RED"),
    ("ICE_JAW", Pose::IceJaw, "BLUE"),
    ("SLIPPER_LEFT", Pose::SlipperLeft, "RED"),
    ("SLIPPER_RIGHT", Pose::SlipperRight, "RED"),
    ("PIKE", Pose::Pike, "BLUE"),
    ("PIKE_TURN", Pose::PikeTurn, "BLUE"),
];

/// Install five scripts and original pose tiles after their move records exist.
///
/// Returns an error for missing moves/table slots, malformed animation data, or
/// an existing pose asset. Original graphics, scripts and audio remain intact.
pub(crate) fn install_northwoods_signature_animations(
    data: &mut GameDataSet,
    runtime_files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    let mut bundle: Value = serde_json::from_str(&data.battle_anim_bundle)
        .context("decode battle animation bundle for Minnesota signatures")?;
    for (name, pose, palette) in POSES {
        let object = format!("BATTLE_ANIM_OBJ_MN_{name}");
        let gfx = format!("BATTLE_ANIM_GFX_MN_{name}");
        let source = format!("Minnesota{name}GFX");
        let path = format!("gfx/battle_anims/minnesota_{name}.2bpp");
        ensure!(
            !runtime_files.contains_key(&path),
            "signature graphics already installed: {path}"
        );
        runtime_files.insert(path.clone(), encode_tiles(&draw_pose(pose)));
        let upright = matches!(
            pose,
            Pose::FlagLow
                | Pose::FlagMiddle
                | Pose::FlagHigh
                | Pose::SlipperLeft
                | Pose::SlipperRight
        );
        bundle["objects"][&object] = json!({
            "object_id": object, "flags": if upright { 33 } else { 97 }, "fix_y": 144,
            "frameset": "BATTLE_ANIM_FRAMESET_HORN", "function": null,
            "palette": format!("PAL_BATTLE_OB_{palette}"), "gfx_id": gfx
        });
        bundle["gfx_table"][&gfx] = json!([6, source]);
        bundle["gfx_sources"][&source] = json!(path);
    }
    for (id, commands) in programs() {
        let move_data = data
            .moves
            .get(id)
            .with_context(|| format!("missing signature move {id}"))?;
        let index = usize::from(move_data.source_index);
        let label = animation_label(id);
        let slot = data
            .battle_animation_table
            .get_mut(index)
            .with_context(|| format!("signature animation table has no slot {index}"))?;
        *slot = label.clone();
        data.battle_animations.insert(label, commands);
    }
    data.battle_anim_bundle = serde_json::to_string(&bundle)?;
    Ok(())
}

/// Return the unique source-program label for a Minnesota signature move ID.
pub fn animation_label(id: &str) -> String {
    format!("BattleAnim_Minnesota_{id}")
}

/// Return authored Minnesota scripts in the cartridge command vocabulary.
/// Waits include the VM yield tick.
pub fn programs() -> Vec<(&'static str, Vec<String>)> {
    let mut rail = Script::new();
    for spread in [18, 14, 10] {
        rail.clear();
        rail.pose("RAIL", 52, 84 - spread);
        rail.pose("RAIL", 52, 84 + spread);
        rail.pose("PELLET", 52, 84);
        rail.wait(6);
    }
    rail.sound("SFX_RAGE");
    for (x, y, delay) in [
        (64, 80, 4),
        (78, 75, 4),
        (92, 69, 3),
        (108, 62, 2),
        (128, 56, 2),
    ] {
        rail.clear();
        rail.pose("RAIL", 52, 74);
        rail.pose("RAIL", 52, 94);
        rail.pose("PELLET", x, y);
        rail.wait(delay);
    }
    rail.clear();
    rail.sound("SFX_COMET_PUNCH");
    rail.0.extend([
        "anim_obj BATTLE_ANIM_OBJ_HIT_YFIX, 132, 52, 0".into(),
        "anim_obj BATTLE_ANIM_OBJ_HIT_YFIX, 140, 60, 0".into(),
    ]);
    rail.wait(12);
    rail.finish();

    let mut bog = Script::new();
    bog.pose("NEEDLE", 126, 58);
    bog.wait(10);
    bog.sound("SFX_GIGA_DRAIN");
    for _ in 0..3 {
        for (x, y) in [(120, 56), (99, 64), (77, 73), (53, 84)] {
            bog.clear();
            bog.pose("NEEDLE", 126, 58);
            bog.pose("PRESSURE", x, y);
            bog.wait(4);
        }
    }
    bog.clear();
    bog.pose("PRESSURE", 48, 88);
    bog.wait(12);
    bog.finish();

    let mut tip = Script::new();
    for pose in ["FLAG_LOW", "FLAG_MIDDLE", "FLAG_HIGH"] {
        tip.clear();
        tip.pose(pose, 52, 82);
        tip.wait(8);
    }
    tip.sound("SFX_SHARPEN");
    tip.clear();
    tip.pose("ICE_JAW", 130, 48);
    tip.wait(5);
    tip.clear();
    tip.pose("ICE_JAW", 134, 54);
    tip.wait(5);
    tip.impact();
    tip.finish();

    let mut slipper = Script::new();
    for (left, right) in [(44, 60), (48, 56), (44, 60)] {
        slipper.clear();
        slipper.pose("SLIPPER_LEFT", left, 82);
        slipper.pose("SLIPPER_RIGHT", right, 90);
        slipper.wait(7);
    }
    slipper.sound("SFX_SWORDS_DANCE");
    for (x, y) in [
        (111, 58),
        (119, 38),
        (140, 36),
        (150, 53),
        (139, 70),
        (124, 59),
    ] {
        slipper.clear();
        slipper.pose("SLIPPER_LEFT", x, y);
        slipper.pose("SLIPPER_RIGHT", 264 - x, 106 - y);
        slipper.wait(5);
    }
    slipper.impact();
    slipper.finish();

    let mut pike = Script::new();
    pike.pose("PIKE", 52, 86);
    pike.wait(12);
    pike.sound("SFX_HORN_ATTACK");
    for (step, (x, y)) in [(64, 81), (76, 75), (90, 69), (106, 63), (124, 55)]
        .into_iter()
        .enumerate()
    {
        pike.clear();
        pike.pose(if step % 2 == 0 { "PIKE" } else { "PIKE_TURN" }, x, y);
        pike.wait(3);
    }
    pike.impact();
    pike.finish();
    SIGNATURE_MOVES
        .into_iter()
        .zip([rail.0, bog.0, tip.0, slipper.0, pike.0])
        .collect()
}

struct Script(Vec<String>);

impl Script {
    fn new() -> Self {
        Self(vec!["anim_1gfx BATTLE_ANIM_GFX_HIT".into()])
    }
    fn pose(&mut self, name: &str, x: i32, y: i32) {
        self.0
            .push(format!("anim_obj BATTLE_ANIM_OBJ_MN_{name}, {x}, {y}, 0"));
    }
    fn wait(&mut self, frames: u8) {
        self.0.push(format!("anim_wait {frames}"));
    }
    fn clear(&mut self) {
        self.0.push("anim_clearobjs".into());
    }
    fn sound(&mut self, id: &str) {
        self.0.push(format!("anim_sound 0, 1, {id}"));
    }
    fn impact(&mut self) {
        self.clear();
        self.sound("SFX_COMET_PUNCH");
        self.0
            .push("anim_obj BATTLE_ANIM_OBJ_HIT_YFIX, 136, 56, 0".into());
        self.wait(12);
    }
    fn finish(&mut self) {
        self.clear();
        self.wait(6);
        self.0.push("anim_ret".into());
    }
}

fn draw_pose(pose: Pose) -> [u8; 384] {
    let mut pixels = [0; 384];
    match pose {
        Pose::Rail => {
            for y in [5, 10] {
                line(&mut pixels, (1, y), (22, y), 3);
                line(&mut pixels, (2, y + 1), (21, y + 1), 1);
            }
            for x in [3, 9, 15, 21] {
                line(&mut pixels, (x, 4), (x, 12), 2);
            }
        }
        Pose::Pellet => {
            for y in 1..15 {
                for x in 4..21 {
                    let distance = (x - 12_i32).abs() + (y - 8_i32).abs();
                    if distance < 11 {
                        pixels[(y * 24 + x) as usize] =
                            if distance >= 9 || x == 4 || x == 20 || y == 1 || y == 14 {
                                3
                            } else if x + y < 20 {
                                1
                            } else {
                                2
                            };
                    }
                }
            }
            line(&mut pixels, (0, 5), (4, 5), 2);
            line(&mut pixels, (1, 11), (5, 11), 3);
            line(&mut pixels, (10, 4), (16, 7), 1);
            line(&mut pixels, (15, 9), (18, 10), 3);
        }
        Pose::Needle => {
            line(&mut pixels, (1, 12), (22, 3), 3);
            line(&mut pixels, (4, 12), (21, 4), 1);
            line(&mut pixels, (19, 2), (22, 3), 3);
        }
        Pose::Pressure => {
            for offset in [0, 6, 12] {
                line(&mut pixels, (offset + 1, 7), (offset + 5, 3), 3);
                line(&mut pixels, (offset + 1, 7), (offset + 5, 11), 3);
                line(&mut pixels, (offset + 3, 7), (offset + 6, 7), 1);
            }
        }
        Pose::FlagLow | Pose::FlagMiddle | Pose::FlagHigh => {
            line(&mut pixels, (2, 14), (21, 14), 3);
            line(&mut pixels, (6, 13), (18, 13), 1);
            let tip = match pose {
                Pose::FlagLow => (20, 10),
                Pose::FlagMiddle => (18, 5),
                _ => (11, 0),
            };
            line(&mut pixels, (11, 13), tip, 3);
            for row in 0..5 {
                line(
                    &mut pixels,
                    (tip.0, tip.1 + row),
                    (tip.0 + (5 - row).min(23 - tip.0), tip.1 + row),
                    if row == 0 || row == 4 { 3 } else { 2 },
                );
            }
        }
        Pose::IceJaw => {
            for (a, b) in [
                ((1, 1), (11, 12)),
                ((11, 12), (22, 1)),
                ((4, 0), (11, 8)),
                ((11, 8), (19, 0)),
            ] {
                line(&mut pixels, a, b, 3);
                line(&mut pixels, (a.0, a.1 + 1), (b.0, b.1 + 1), 1);
            }
            for x in [7, 12, 17] {
                line(&mut pixels, (x, 2), (x, 5), 2);
            }
        }
        Pose::SlipperLeft | Pose::SlipperRight => {
            let mirror = matches!(pose, Pose::SlipperRight);
            for y in 5..13 {
                for x in 5..17 {
                    let dx = x - 11_i32;
                    let dy = y - 9_i32;
                    if dx * dx + dy * dy * 2 < 39 {
                        let x = if mirror { 23 - x } else { x };
                        pixels[(y * 24 + x) as usize] = if dy == -3 || dy == 3 || dx.abs() >= 5 {
                            3
                        } else if dy < 0 {
                            1
                        } else {
                            2
                        };
                    }
                }
            }
            for (a, b) in [
                ((10, 6), (3, 1)),
                ((12, 6), (20, 2)),
                ((11, 5), (12, 0)),
                ((7, 12), (3, 14)),
            ] {
                let (a, b) = if mirror {
                    ((23 - a.0, a.1), (23 - b.0, b.1))
                } else {
                    (a, b)
                };
                line(&mut pixels, a, b, 3);
            }
        }
        Pose::Pike | Pose::PikeTurn => {
            let narrow = matches!(pose, Pose::PikeTurn);
            line(&mut pixels, (1, 13), (22, 3), 3);
            line(&mut pixels, (2, 14), (22, 4), 1);
            let breadth = if narrow { 2 } else { 5 };
            line(&mut pixels, (22, 3), (13, 3 - breadth / 2), 3);
            line(&mut pixels, (22, 3), (17, 3 + breadth), 3);
            line(&mut pixels, (13, 3 - breadth / 2), (17, 3 + breadth), 2);
            for x in [4, 8] {
                line(&mut pixels, (x, 9), (x + 1, 12), 2);
            }
        }
    }
    pixels
}

fn line(pixels: &mut [u8; 384], a: (i32, i32), b: (i32, i32), ink: u8) {
    let (mut x, mut y) = a;
    let dx = (b.0 - x).abs();
    let sx = if x < b.0 { 1 } else { -1 };
    let dy = -(b.1 - y).abs();
    let sy = if y < b.1 { 1 } else { -1 };
    let mut error = dx + dy;
    loop {
        if (0..24).contains(&x) && (0..16).contains(&y) {
            pixels[(y * 24 + x) as usize] = ink;
        }
        if (x, y) == b {
            break;
        }
        let doubled = 2 * error;
        if doubled >= dy {
            error += dy;
            x += sx;
        }
        if doubled <= dx {
            error += dx;
            y += sy;
        }
    }
}

fn encode_tiles(pixels: &[u8; 384]) -> Vec<u8> {
    let mut tiles = Vec::with_capacity(96);
    for ty in 0..2 {
        for tx in 0..3 {
            for row in 0..8 {
                let mut low = 0;
                let mut high = 0;
                for x in 0..8 {
                    let ink = pixels[(ty * 8 + row) * 24 + tx * 8 + x];
                    low |= (ink & 1) << (7 - x);
                    high |= ((ink >> 1) & 1) << (7 - x);
                }
                tiles.extend([low, high]);
            }
        }
    }
    tiles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poses_round_trip_six_native_tiles_without_resampling() {
        let mut unique = std::collections::BTreeSet::new();
        for (name, pose, _) in POSES {
            let pixels = draw_pose(pose);
            assert!(pixels.iter().all(|ink| *ink < 4), "{name}");
            assert!(
                pixels.iter().filter(|ink| **ink != 0).count() > 15,
                "{name}"
            );
            let tiles = encode_tiles(&pixels);
            assert_eq!(tiles.len(), 96);
            assert!(
                unique.insert(tiles.clone()),
                "poses need distinct geometry: {name}"
            );
            for y in 0..16 {
                for x in 0..24 {
                    let offset = ((y / 8) * 3 + x / 8) * 16 + (y % 8) * 2;
                    let shift = 7 - x % 8;
                    assert_eq!(
                        pixels[y * 24 + x],
                        ((tiles[offset] >> shift) & 1) | (((tiles[offset + 1] >> shift) & 1) << 1)
                    );
                }
            }
        }
    }

    #[test]
    fn scripts_bound_duration_slots_and_clear_terminal_oam() {
        for (id, commands) in programs() {
            let mut live = 0;
            let mut frames = 0;
            for command in &commands {
                if command == "anim_clearobjs" {
                    live = 0;
                }
                if command.starts_with("anim_obj ") {
                    live += 1;
                }
                assert!(live <= 3, "{id} exceeds original OAM scanline budget");
                if let Some(wait) = command.strip_prefix("anim_wait ") {
                    frames += wait.parse::<u16>().expect("numeric wait") + 1;
                }
            }
            assert!((45..=120).contains(&frames), "{id}: {frames}");
            assert_eq!(commands[commands.len() - 3], "anim_clearobjs");
            assert_eq!(commands.last().map(String::as_str), Some("anim_ret"));
        }
    }
}
