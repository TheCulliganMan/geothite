//! Encode the checked-in, native pixel grids used by regional story actors.
use std::collections::BTreeMap;

use anyhow::{Result, ensure};
use serde::Deserialize;

use crate::minnesota_pokemon_pixel_art::{bitplanes, colored, png_rgb, png_rgba};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeNpc {
    schema_version: u8,
    id: String,
    palette_rgb555: [u16; 4],
    battle_rows: Vec<String>,
    overworld_frames: Vec<NativeFrame>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeFrame {
    pose: String,
    rows: Vec<String>,
}

fn pixels(rows: &[String], size: usize) -> Result<Vec<u8>> {
    ensure!(
        rows.len() == size && rows.iter().all(|row| row.len() == size),
        "wrong native NPC grid size"
    );
    rows.iter()
        .flat_map(|row| row.bytes())
        .map(|ink| match ink {
            b'.' => Ok(0),
            b'1'..=b'3' => Ok(ink - b'0'),
            _ => anyhow::bail!("invalid native NPC ink"),
        })
        .collect()
}

/// Encodes accepted battle art and an optional matching field-art repair.
/// Rejects different identities/palettes, invalid grids or incomplete walking
/// poses. The existing bundle validator independently checks PNG/2bpp parity.
pub(crate) fn compile(
    battle_source: &str,
    field_source: Option<&str>,
) -> Result<BTreeMap<String, Vec<u8>>> {
    let battle: NativeNpc = serde_json::from_str(battle_source)?;
    let field: NativeNpc = serde_json::from_str(field_source.unwrap_or(battle_source))?;
    let stem = &battle.id;
    ensure!(
        battle.schema_version == 1
            && field.schema_version == 1
            && stem.starts_with("mn_")
            && stem
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
            && field.id == *stem
            && field.palette_rgb555 == battle.palette_rgb555,
        "matching native NPC identity and palette required"
    );
    let mut bank = Vec::new();
    let poses = [
        "down_idle",
        "up_idle",
        "left_idle",
        "down_step",
        "up_step",
        "left_step",
    ];
    ensure!(
        field.overworld_frames.len() == poses.len(),
        "six native NPC poses required"
    );
    for (frame, pose) in field.overworld_frames.iter().zip(poses) {
        ensure!(frame.pose == pose, "wrong native NPC pose order");
        let frame_pixels = pixels(&frame.rows, 16)?;
        ensure!(
            frame_pixels.iter().any(|ink| *ink != 0),
            "empty native NPC pose"
        );
        bank.extend(frame_pixels);
    }
    for direction in 0..3 {
        ensure!(
            bank[direction * 256..(direction + 1) * 256]
                != bank[(direction + 3) * 256..(direction + 4) * 256],
            "identical native NPC walking pair"
        );
    }
    let front = pixels(&battle.battle_rows, 56)?;
    let grayscale = bank
        .iter()
        .map(|ink| {
            let value = [255, 170, 85, 0][usize::from(*ink)];
            [value, value, value, if *ink == 0 { 0 } else { 255 }]
        })
        .collect::<Vec<_>>();
    let palette = battle.palette_rgb555;
    let palette_bytes = palette
        .iter()
        .flat_map(|ink| ink.to_le_bytes())
        .collect::<Vec<_>>();
    let files = BTreeMap::from([
        (
            format!("gfx/sprites/{stem}.png"),
            png_rgba(16, 96, &grayscale)?,
        ),
        (format!("gfx/sprites/{stem}.2bpp"), bitplanes(&bank, 16, 96)),
        (format!("gfx/sprites/{stem}.gbcpal"), palette_bytes.clone()),
        (
            format!("gfx/trainers/{stem}.png"),
            png_rgb(56, 56, &colored(&front, palette))?,
        ),
        (
            format!("gfx/trainers/{stem}.2bpp"),
            bitplanes(&front, 56, 56),
        ),
        (format!("gfx/trainers/{stem}.gbcpal"), palette_bytes),
    ]);
    crate::minnesota_trainers::verify_original_npc_bundle(&files, stem)?;
    Ok(files)
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!(
        "../../../planning/minnesota/npcs/tamarack-ice/01-tamarack/native-pixels-v4.json"
    );

    #[test]
    fn accepted_tamarack_pixels_encode_and_reject_mismatched_field_identity() {
        let files = super::compile(SOURCE, None).expect("accepted native art");
        assert_eq!(files["gfx/sprites/mn_prof_tamarack.2bpp"].len(), 384);
        assert_eq!(files["gfx/trainers/mn_prof_tamarack.2bpp"].len(), 784);
        let mut wrong: serde_json::Value = serde_json::from_str(SOURCE).expect("native source");
        wrong["id"] = "mn_other".into();
        assert!(super::compile(SOURCE, Some(&wrong.to_string())).is_err());
        wrong["id"] = "mn_prof_tamarack".into();
        wrong["palette_rgb555"][1] = 1234.into();
        assert!(super::compile(SOURCE, Some(&wrong.to_string())).is_err());
    }

    #[test]
    fn accepted_reed_keeps_battle_pixels_and_uses_repaired_field_frames() {
        let battle =
            include_str!("../../../planning/minnesota/npcs/cast/01-reed/native-pixels-v3.json");
        let field = include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-field-repairs/01-reed/native-pixels-v2.json"
        );
        let old = super::compile(battle, None).expect("accepted battle source");
        let fixed = super::compile(battle, Some(field)).expect("matched field repair");
        for extension in ["png", "2bpp", "gbcpal"] {
            let path = format!("gfx/trainers/mn_rival_reed.{extension}");
            assert_eq!(old[&path], fixed[&path], "preserve accepted battle art");
        }
        assert_ne!(
            old["gfx/sprites/mn_rival_reed.2bpp"],
            fixed["gfx/sprites/mn_rival_reed.2bpp"]
        );
    }
}
