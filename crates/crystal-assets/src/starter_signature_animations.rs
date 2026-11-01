//! Original starter attack tiles and choreography for the native Gen II OAM VM.
//!
//! Each pose uses six literal 8×8 tiles in the existing 24×16 HORN frameset.
//! Effects use fixed cartridge object palettes and never flash the background.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::GameDataSet;

/// New starter attacks; Cyprielle retains the original `SLIPPER_WALTZ` program.
pub const STARTER_SIGNATURE_MOVES: [&str; 2] = ["RIMEBRAND", "PADDLE_SWEEP"];

const POSES: [(&str, &[&str], &str); 8] = [
    ("PAW_LOW", PAW_LOW, "RED"),
    ("PAW_HIGH", PAW_HIGH, "RED"),
    ("EMBER", EMBER, "RED"),
    ("CLAW_RISE", CLAW_RISE, "RED"),
    ("CLAW_FALL", CLAW_FALL, "RED"),
    ("TAIL_PADDLE", TAIL_PADDLE, "BLUE"),
    ("WATER_CRESCENT", WATER_CRESCENT, "BLUE"),
    ("DROPLETS", DROPLETS, "BLUE"),
];

/// Return the unique source-program label for a starter signature move ID.
#[must_use]
pub fn animation_label(move_id: &str) -> String {
    format!("BattleAnim_NorthwoodsStarter_{move_id}")
}

/// Install two original scripts and their small, fixed-palette object banks.
///
/// Existing moves, scripts, graphics, sounds and background palettes are kept.
///
/// # Errors
/// Returns an error if the animation bundle is malformed, a required move/table
/// slot is absent, or any starter object, program or graphics path already exists.
pub(crate) fn install_starter_signature_animations(
    data: &mut GameDataSet,
    runtime_files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    let mut bundle: Value = serde_json::from_str(&data.battle_anim_bundle)
        .context("decode battle animation bundle for starter signatures")?;
    ensure!(
        bundle["framesets"]
            .get("BATTLE_ANIM_FRAMESET_HORN")
            .is_some(),
        "starter signatures require the native HORN frameset"
    );
    for section in ["objects", "gfx_table", "gfx_sources"] {
        ensure!(
            bundle.get(section).is_some_and(Value::is_object),
            "starter animation bundle has an invalid {section} section"
        );
    }
    let scripts = programs();
    for (move_id, _) in &scripts {
        let entry = data
            .moves
            .get(*move_id)
            .with_context(|| format!("missing starter signature move {move_id}"))?;
        ensure!(
            data.battle_animation_table
                .get(usize::from(entry.source_index))
                .is_some(),
            "starter signature animation slot is absent: {move_id}"
        );
        ensure!(
            !data
                .battle_animations
                .contains_key(&animation_label(move_id)),
            "starter signature program already exists: {move_id}"
        );
    }
    for (name, _, _) in POSES {
        let path = graphics_path(name);
        let object = format!("BATTLE_ANIM_OBJ_MN_STARTER_{name}");
        let gfx = format!("BATTLE_ANIM_GFX_MN_STARTER_{name}");
        let source = format!("NorthwoodsStarter{name}GFX");
        ensure!(
            !runtime_files.contains_key(&path),
            "starter graphics already exist: {path}"
        );
        ensure!(
            bundle["objects"].get(&object).is_none(),
            "starter object already exists: {object}"
        );
        ensure!(
            bundle["gfx_table"].get(&gfx).is_none(),
            "starter gfx slot already exists: {gfx}"
        );
        ensure!(
            bundle["gfx_sources"].get(&source).is_none(),
            "starter gfx source already exists: {source}"
        );
    }
    for (name, mask, palette) in POSES {
        let object = format!("BATTLE_ANIM_OBJ_MN_STARTER_{name}");
        let gfx = format!("BATTLE_ANIM_GFX_MN_STARTER_{name}");
        let source = format!("NorthwoodsStarter{name}GFX");
        let path = graphics_path(name);
        runtime_files.insert(path.clone(), encode_tiles(&pixels(mask)));
        bundle["objects"][&object] = json!({
            "object_id": object, "flags": 97, "fix_y": 144,
            "frameset": "BATTLE_ANIM_FRAMESET_HORN", "function": null,
            "palette": format!("PAL_BATTLE_OB_{palette}"), "gfx_id": gfx
        });
        bundle["gfx_table"][&gfx] = json!([6, source]);
        bundle["gfx_sources"][&source] = json!(path);
    }
    for (move_id, commands) in scripts {
        let index = usize::from(data.moves[move_id].source_index);
        let label = animation_label(move_id);
        data.battle_animation_table[index] = label.clone();
        data.battle_animations.insert(label, commands);
    }
    data.battle_anim_bundle = serde_json::to_string(&bundle)?;
    Ok(())
}

/// Return the two original cartridge-command programs, including OAM cleanup.
///
/// Rimebrand is an ember-paw stalk and claw strike. Paddle Sweep turns a broad
/// tail into a traveling water crescent. Neither displays a guaranteed status
/// effect; battle mechanics independently decide the optional secondary effect.
#[must_use]
pub fn programs() -> Vec<(&'static str, Vec<String>)> {
    let mut fire = Script::new();
    for (pose, x, y) in [
        ("PAW_LOW", 43, 91),
        ("PAW_HIGH", 51, 86),
        ("PAW_LOW", 58, 84),
    ] {
        fire.pose(&[(pose, x, y), ("EMBER", x - 10, y + 4)], 7);
    }
    fire.sound("SFX_EMBER");
    fire.pose(&[("PAW_HIGH", 58, 80), ("EMBER", 48, 88)], 6);
    for (x, y) in [(73, 77), (91, 70), (109, 62), (127, 55)] {
        fire.pose(&[("PAW_HIGH", x, y), ("EMBER", x - 14, y + 7)], 3);
    }
    fire.sound("SFX_CUT");
    fire.pose(&[("CLAW_RISE", 132, 52), ("EMBER", 119, 63)], 7);
    fire.pose(&[("CLAW_FALL", 135, 55), ("EMBER", 145, 44)], 7);
    fire.hit(6);
    fire.pose(&[("EMBER", 123, 43), ("EMBER", 142, 66)], 6);

    let mut water = Script::new();
    water.pose(&[("TAIL_PADDLE", 43, 89)], 8);
    water.pose(&[("TAIL_PADDLE", 53, 81), ("DROPLETS", 39, 93)], 7);
    water.sound("SFX_SURF");
    for (x, y) in [(64, 82), (79, 75), (96, 68), (113, 61), (131, 54)] {
        water.pose(&[("WATER_CRESCENT", x, y), ("DROPLETS", x - 16, y + 9)], 4);
    }
    water.pose(&[("WATER_CRESCENT", 135, 51), ("DROPLETS", 141, 65)], 8);
    water.hit(7);
    water.pose(&[("DROPLETS", 118, 44), ("DROPLETS", 145, 67)], 7);
    vec![
        ("RIMEBRAND", fire.finish()),
        ("PADDLE_SWEEP", water.finish()),
    ]
}

struct Script(Vec<String>);

impl Script {
    fn new() -> Self {
        Self(vec![
            "anim_1gfx BATTLE_ANIM_GFX_HIT".to_string(),
            "anim_wait 6".to_string(),
        ])
    }

    fn pose(&mut self, objects: &[(&str, i32, i32)], frames: u8) {
        self.0.push("anim_clearobjs".to_string());
        for (name, x, y) in objects {
            self.0.push(format!(
                "anim_obj BATTLE_ANIM_OBJ_MN_STARTER_{name}, {x}, {y}, 0"
            ));
        }
        self.0.push(format!("anim_wait {frames}"));
    }

    fn sound(&mut self, sound_id: &str) {
        self.0.push(format!("anim_sound 0, 1, {sound_id}"));
    }

    fn hit(&mut self, frames: u8) {
        self.0.extend([
            "anim_clearobjs".to_string(),
            "anim_sound 0, 1, SFX_COMET_PUNCH".to_string(),
            "anim_obj BATTLE_ANIM_OBJ_HIT_YFIX, 134, 55, 0".to_string(),
            format!("anim_wait {frames}"),
        ]);
    }

    fn finish(mut self) -> Vec<String> {
        self.0.extend([
            "anim_clearobjs".to_string(),
            "anim_wait 7".to_string(),
            "anim_ret".to_string(),
        ]);
        self.0
    }
}

fn graphics_path(name: &str) -> String {
    format!("gfx/battle_anims/northwoods_starter_{name}.2bpp")
}

fn pixels(mask: &[&str]) -> [u8; 384] {
    let mut output = [0; 384];
    for (y, row) in mask.iter().enumerate() {
        for (x, byte) in row.bytes().enumerate() {
            output[y * 24 + x] = match byte {
                b'1' => 1,
                b'2' => 2,
                b'3' => 3,
                _ => 0,
            };
        }
    }
    output
}

fn encode_tiles(pixels: &[u8; 384]) -> Vec<u8> {
    let mut output = Vec::with_capacity(96);
    for tile_y in 0..2 {
        for tile_x in 0..3 {
            for y in 0..8 {
                let mut low = 0;
                let mut high = 0;
                for x in 0..8 {
                    let ink = pixels[(tile_y * 8 + y) * 24 + tile_x * 8 + x];
                    low |= (ink & 1) << (7 - x);
                    high |= ((ink >> 1) & 1) << (7 - x);
                }
                output.extend([low, high]);
            }
        }
    }
    output
}

const PAW_LOW: &[&str] = &[
    "........................",
    "........................",
    "........................",
    "........................",
    "......33...33...33......",
    ".....3113.3113.3113.....",
    ".....3123.3123.3123.....",
    "......33...33...33......",
    "........3333333.........",
    "......33111111233.......",
    ".....3111111222233......",
    ".....3111122222223......",
    "......31122222223.......",
    ".......333333333........",
    "........................",
    "........................",
];
const PAW_HIGH: &[&str] = &[
    ".........33.............",
    "........3113............",
    "........3123..33........",
    ".....33..33..3113.......",
    "....3113.....3123.......",
    "....3123.3333.33........",
    ".....33.311113..........",
    ".......31111223.........",
    "......3111122223........",
    "......3112222223........",
    ".......31222223.........",
    "........333333..........",
    "........................",
    "........................",
    "........................",
    "........................",
];
const EMBER: &[&str] = &[
    "........................",
    "...........3............",
    "..........313...........",
    ".........3113...........",
    "........31123...........",
    ".......311223...........",
    "......31112233..........",
    "......311122123.........",
    ".......3111223..........",
    "........33333...........",
    "........................",
    "....33...........3......",
    "....313.........313.....",
    ".....3...........3......",
    "........................",
    "........................",
];
const CLAW_RISE: &[&str] = &[
    "....................33..",
    "..................3313..",
    ".................3113...",
    "..............333123....",
    "............3311123.....",
    "...........3111223...33.",
    "........333112223..3313.",
    "......331112223...3113..",
    ".....311122223..33123...",
    "...331122223...31123....",
    "..31122223...331123.....",
    ".3122223...3311223......",
    ".333333...3112223.......",
    "........3312223.........",
    ".......312223...........",
    ".......33333............",
];
const CLAW_FALL: &[&str] = &[
    "..33....................",
    "..3133..................",
    "...3113.................",
    "....321333..............",
    ".....3211133............",
    ".33...3221113...........",
    ".3133..322211333........",
    "..3113...322211133......",
    "...32133..32222113......",
    "....32113...32222133....",
    ".....321133...3222213...",
    "......3221133...322213..",
    ".......3222113...33333..",
    ".........3222133........",
    "...........322213.......",
    "............33333.......",
];
const TAIL_PADDLE: &[&str] = &[
    "..............3333......",
    "............33111133....",
    "..........33111111123...",
    ".........3111311111223..",
    "........31111311112223..",
    ".......311131111122223..",
    "......311113111122223...",
    ".....311113111122223....",
    "....311113111122223.....",
    "....31111111122223......",
    ".....311111222233.......",
    "......333122233.........",
    "........31233...........",
    ".......3123.............",
    "......3333..............",
    "........................",
];
const WATER_CRESCENT: &[&str] = &[
    "..........33333333......",
    ".......3331111111233....",
    ".....3311111111222223...",
    "....31111122222233333...",
    "...3111122233333........",
    "..311122233.............",
    ".3111223................",
    ".311223.................",
    ".311223.................",
    ".3111223................",
    "..311122233.............",
    "...3111122233333........",
    "....31111122222233333...",
    ".....3311111111222223...",
    ".......3331111111233....",
    "..........33333333......",
];
const DROPLETS: &[&str] = &[
    "....3...................",
    "...313..................",
    "..31123..........3......",
    "...323..........313.....",
    "....3..........31123....",
    "................323.....",
    "........3........3......",
    ".......313..............",
    "......31123.............",
    ".......323..............",
    "........3.........3.....",
    ".................313....",
    "...3............31123...",
    "..313............323....",
    "...3..............3.....",
    "........................",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starter_tiles_preserve_native_inks_and_unique_geometry() {
        let mut unique = std::collections::BTreeSet::new();
        for (name, mask, _) in POSES {
            assert_eq!(mask.len(), 16, "{name}: native pose height");
            assert!(
                mask.iter().all(|row| row.len() <= 24),
                "{name}: native pose width"
            );
            assert!(
                mask.iter()
                    .all(|row| row.bytes().all(|b| b == b'.' || (b'1'..=b'3').contains(&b))),
                "{name}: literal four-ink raster"
            );
            let raster = pixels(mask);
            let bank = encode_tiles(&raster);
            assert_eq!(bank.len(), 96, "{name}: six native tiles");
            assert!(
                unique.insert(bank.clone()),
                "{name}: distinct original geometry"
            );
            for y in 0..16 {
                for x in 0..24 {
                    let offset = ((y / 8) * 3 + x / 8) * 16 + y % 8 * 2;
                    let shift = 7 - x % 8;
                    assert_eq!(
                        raster[y * 24 + x],
                        ((bank[offset] >> shift) & 1) | (((bank[offset + 1] >> shift) & 1) << 1),
                        "{name}: literal 2bpp pixel ({x},{y})"
                    );
                }
            }
        }
    }

    #[test]
    fn starter_scripts_bound_oam_duration_and_restore_clean_layers() {
        for ((move_id, script), expected_id) in programs().iter().zip(STARTER_SIGNATURE_MOVES) {
            assert_eq!(*move_id, expected_id);
            let mut live = 0;
            let mut duration = 0_u16;
            for command in script {
                if command == "anim_clearobjs" {
                    live = 0;
                }
                if command.starts_with("anim_obj ") {
                    live += 1;
                }
                assert!(live <= 2, "{move_id}: limited original OAM occupancy");
                assert!(
                    !command.starts_with("anim_bgp"),
                    "{move_id}: no background flashes"
                );
                if let Some(wait) = command.strip_prefix("anim_wait ") {
                    duration += wait.parse::<u16>().expect("authored numeric wait") + 1;
                }
            }
            assert!(
                (65..=120).contains(&duration),
                "{move_id}: {duration} frames"
            );
            assert_eq!(script[script.len() - 3], "anim_clearobjs");
            assert_eq!(script.last().map(String::as_str), Some("anim_ret"));
        }
    }
}
