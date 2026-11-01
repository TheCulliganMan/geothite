//! Distinct cartridge-style attack programs for the Northwoods imported moves.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::{CompiledGamePack, GameDataSet};

#[path = "northwoods_move_animation_programs.rs"]
mod programs;

/// Manifest identity of the presentation-only nineteen-move animation overlay.
pub const NORTHWOODS_ANIMATIONS_MANIFEST_ID: &str = "northwoods-animations-v1";
/// Imported attacks whose original Northwoods donor programs are replaced.
pub const IMPORTED_MOVES: [&str; 14] = [
    "WATER_PULSE",
    "AQUA_JET",
    "ICE_SHARD",
    "ENERGY_BALL",
    "DRAIN_PUNCH",
    "SIGNAL_BEAM",
    "IRON_HEAD",
    "X_SCISSOR",
    "FAIRY_WIND",
    "DISARMING_VOICE",
    "DAZZLING_GLEAM",
    "PLAY_ROUGH",
    "MOONBLAST",
    "POISON_JAB",
];

// Existing native 2bpp graphics and framesets are reused; only stationary object
// definitions are added. Uniform native facing flags preserve the exact OAM
// mirror transform while the scripts explicitly author each attack's motion.
const GLYPHS: [(&str, &str, &str); 24] = [
    ("WAVE", "WAVE", "BLUE"),
    ("WATER", "WATER_GUN", "BLUE"),
    ("WAKE", "SPEED_LINE", "BLUE"),
    ("SHARD", "HORN", "BLUE"),
    ("FROST", "ICE", "BLUE"),
    ("ORB", "ABSORB_CENTER", "GREEN"),
    ("LEAF", "RAZOR_LEAF", "GREEN"),
    ("MOTE", "ABSORB", "GREEN"),
    ("FIST", "PUNCH", "GRAY"),
    ("BEAM_GREEN", "BEAM", "GREEN"),
    ("BEAM_BLUE", "BEAM", "BLUE"),
    ("RAM", "SHADOW_BALL", "GRAY"),
    ("METAL", "HORN", "GRAY"),
    ("CUT_LEFT", "CUT_LONG_DOWN_LEFT", "GREEN"),
    ("CUT_RIGHT", "CUT_LONG_DOWN_RIGHT", "GREEN"),
    ("CURL_LEFT", "RAZOR_WIND1", "RED"),
    ("CURL_RIGHT", "RAZOR_WIND2", "RED"),
    ("STAR", "ENCORE_STAR", "YELLOW"),
    ("VOICE", "SOUND", "RED"),
    ("PALM", "PALM", "GRAY"),
    ("MOON", "SHADOW_BALL", "YELLOW"),
    ("CRESCENT", "RAZOR_WIND2", "YELLOW"),
    ("VENOM", "HORN", "GREEN"),
    ("DROP", "HIT_SMALL", "GREEN"),
];

/// Return the stable unique program label for one of the fourteen imports.
pub fn animation_label(move_id: &str) -> String {
    format!("BattleAnim_NorthwoodsImport_{move_id}")
}

/// Install distinct imported attacks after the completed Fairy move extension.
///
/// The original five signatures, every move fact, native program, audio asset,
/// sprite bank and runtime file are preserved. New objects reuse native tiles.
///
/// # Errors
/// Returns an error for an invalid/incomplete base, duplicate overlay, absent
/// native object/gfx/move slot, or failed verification of the completed pack.
pub fn build_northwoods_animations_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.data.moves.len() == 270,
        "animations require the completed Fairy move extension"
    );
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|id| id == NORTHWOODS_ANIMATIONS_MANIFEST_ID),
        "Northwoods animations already installed"
    );
    let mut pack = base.clone();
    install(&mut pack.data)?;
    pack.report
        .manifests
        .push(NORTHWOODS_ANIMATIONS_MANIFEST_ID.to_string());
    pack.identity = if pack.audio_compression.is_some() {
        crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?
    } else {
        crate::derive_compiled_game_pack_identity(
            pack.format_version,
            &pack.data,
            &pack.compiled_audio,
            &pack.runtime_files,
            &pack.report,
        )?
    };
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn install(data: &mut GameDataSet) -> Result<()> {
    let mut bundle: Value = serde_json::from_str(&data.battle_anim_bundle)
        .context("decode native animation object bank")?;
    for (name, donor, palette) in GLYPHS {
        let id = format!("BATTLE_ANIM_OBJ_MN_IMPORT_{name}");
        ensure!(
            bundle["objects"].get(&id).is_none(),
            "duplicate imported object {id}"
        );
        let donor_id = format!("BATTLE_ANIM_OBJ_{donor}");
        let mut object = bundle["objects"]
            .get(&donor_id)
            .with_context(|| format!("missing native glyph {donor_id}"))?
            .clone();
        object["object_id"] = id.clone().into();
        object["flags"] = 97.into();
        object["fix_y"] = 144.into();
        object["function"] = Value::Null;
        object["palette"] = format!("PAL_BATTLE_OB_{palette}").into();
        bundle["objects"][&id] = object;
    }
    for (id, commands) in programs::programs() {
        let source_index = data
            .moves
            .get(id)
            .with_context(|| format!("missing imported move {id}"))?
            .source_index;
        let label = animation_label(id);
        ensure!(
            !data.battle_animations.contains_key(&label),
            "duplicate imported program {label}"
        );
        let slot = data
            .battle_animation_table
            .get_mut(usize::from(source_index))
            .context("missing imported animation table slot")?;
        *slot = label.clone();
        data.battle_animations.insert(label, commands);
    }
    data.battle_anim_bundle = serde_json::to_string(&bundle)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imported_programs_have_unique_motion_and_explicit_cleanup() {
        let scripts = programs::programs();
        assert_eq!(scripts.len(), IMPORTED_MOVES.len());
        let mut unique = std::collections::BTreeSet::new();
        for ((id, commands), expected) in scripts.iter().zip(IMPORTED_MOVES) {
            assert_eq!(*id, expected);
            assert!(unique.insert(commands));
            assert_eq!(commands.last().map(String::as_str), Some("anim_ret"));
            assert_eq!(commands[commands.len() - 3], "anim_clearobjs");
            assert!(
                commands
                    .iter()
                    .any(|command| command.starts_with("anim_obj BATTLE_ANIM_OBJ_MN_IMPORT_"))
            );
        }
    }

    #[test]
    #[ignore = "requires the completed external Fairy pack"]
    fn imported_animation_overlay_preserves_gameplay_and_all_existing_assets() {
        let path = std::env::var_os("CRYSTAL_ANIMATION_BASE_PACK").expect("external Fairy pack");
        let base = crate::read_verified_compiled_game_pack(path).expect("verified base");
        let result = build_northwoods_animations_modpack(&base).expect("animation overlay");
        assert_eq!(base.data.moves, result.data.moves);
        assert_eq!(base.data.learnsets, result.data.learnsets);
        assert_eq!(base.data.pokemon, result.data.pokemon);
        assert_eq!(base.data.type_effectiveness, result.data.type_effectiveness);
        assert_eq!(base.data.type_categories, result.data.type_categories);
        assert_eq!(base.runtime_files, result.runtime_files);
        assert_eq!(base.audio_manifest, result.audio_manifest);
        assert_eq!(base.compiled_audio, result.compiled_audio);
        for (label, commands) in &base.data.battle_animations {
            assert_eq!(Some(commands), result.data.battle_animations.get(label));
        }
        for id in crate::minnesota_signature_animations::SIGNATURE_MOVES {
            let index = usize::from(base.data.moves[id].source_index);
            assert_eq!(
                base.data.battle_animation_table[index],
                result.data.battle_animation_table[index]
            );
        }
        assert!(build_northwoods_animations_modpack(&result).is_err());
    }
}
