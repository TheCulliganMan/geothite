//! Restore executable map commands omitted by regional content builders.

use std::collections::BTreeMap;

use anyhow::{Result, ensure};
use crystal_core::systems::script_warps::ScriptMapCommand;

use crate::CompiledGamePack;

/// Restores missing Minnesota map-command records from the pack's own scripts.
///
/// Existing records must agree with their authored commands. Scripts, geography,
/// events, trainers, artwork, audio and non-Minnesota maps are preserved. A repair
/// receives a new pack identity; an already complete pack is returned unchanged.
/// Neither the input pack nor existing saves are modified.
///
/// # Errors
/// Returns an error for an invalid pack, malformed script, conflicting existing
/// command record, or a repaired pack that fails runtime verification.
pub fn repair_minnesota_script_commands(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let names = base.data.map_name_by_constant_from_attributes()?;
    let mut pack = base.clone();
    let mut additions = 0;
    for (name, map) in &mut pack.data.maps {
        if !name.starts_with("Mn") {
            continue;
        }
        let expected = crate::parse_script_map_commands(name, &map.scripts, &names)?;
        additions += restore_commands(&mut map.script_map_commands, expected)?;
    }
    if additions == 0 {
        return Ok(pack);
    }
    pack.report
        .manifests
        .push("minnesota-script-map-command-repair-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn restore_commands(
    existing: &mut Vec<ScriptMapCommand>,
    expected: Vec<ScriptMapCommand>,
) -> Result<usize> {
    let indexed: BTreeMap<_, _> = existing
        .iter()
        .map(|command| {
            (
                (command.source_script.as_str(), command.command_index),
                command,
            )
        })
        .collect();
    ensure!(
        indexed.len() == existing.len(),
        "duplicate map-command records"
    );
    let mut missing = Vec::new();
    for command in expected {
        match indexed.get(&(command.source_script.as_str(), command.command_index)) {
            Some(stored) => ensure!(
                **stored == command,
                "conflicting map command {}:{}",
                command.source_script,
                command.command_index
            ),
            None => missing.push(command),
        }
    }
    let count = missing.len();
    existing.extend(missing);
    Ok(count)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::restore_commands;

    #[test]
    fn minnesota_script_repair_restores_battle_return_without_replacing_existing_warp() {
        let scripts = BTreeMap::from([(
            "Cyclist".into(),
            json!([
                {"command":"warp", "args":["MN_MINNEAPOLIS", "4", "6"]},
                {"command":"reloadmapafterbattle", "args":[]},
                {"command":"end", "args":[]}
            ]),
        )]);
        let names = BTreeMap::from([("MN_MINNEAPOLIS".into(), "MnMinneapolis".into())]);
        let expected = crate::parse_script_map_commands("MnMinneapolis", &scripts, &names)
            .expect("authored commands");
        let mut stored = vec![expected[0].clone()];
        assert_eq!(
            restore_commands(&mut stored, expected.clone()).expect("repair"),
            1
        );
        assert_eq!(stored, expected);
        assert_eq!(
            restore_commands(&mut stored, expected.clone()).expect("repeat"),
            0
        );

        stored[0].x = Some(9);
        let before = stored.clone();
        assert!(restore_commands(&mut stored, expected).is_err());
        assert_eq!(
            stored, before,
            "conflicts cannot partially change the command table"
        );
    }

    #[test]
    #[ignore = "requires CRYSTAL_MINNESOTA_SCRIPT_PACK with the installed cyclists"]
    fn minnesota_script_repair_preserves_the_complete_external_pack() {
        let path = std::env::var_os("CRYSTAL_MINNESOTA_SCRIPT_PACK").expect("external pack");
        let base = crate::read_verified_compiled_game_pack(path).expect("verified base");
        let repaired = super::repair_minnesota_script_commands(&base).expect("repair");
        let mut expected = base.clone();
        let mut added = 0;
        for (name, map) in &mut expected.data.maps {
            let after = &repaired.data.maps[name];
            if name.starts_with("Mn") {
                added += after.script_map_commands.len() - map.script_map_commands.len();
                map.script_map_commands
                    .clone_from(&after.script_map_commands);
            }
            assert_eq!(
                map, after,
                "only Minnesota command indexes may change: {name}"
            );
        }
        assert_eq!(added, 18, "all Minneapolis battle returns must be restored");
        expected.report = repaired.report.clone();
        expected.identity = repaired.identity.clone();
        assert_eq!(
            expected, repaired,
            "unrelated catalogs and audio must remain exact"
        );
        assert_eq!(
            super::repair_minnesota_script_commands(&repaired).expect("idempotent repair"),
            repaired
        );
    }

    #[test]
    #[ignore = "requires CRYSTAL_MINNESOTA_SCRIPT_BASE before the cyclists were installed"]
    fn minnesota_script_repair_new_cyclists_compile_their_battle_returns() {
        let path = std::env::var_os("CRYSTAL_MINNESOTA_SCRIPT_BASE").expect("external parent");
        let base = crate::read_verified_compiled_game_pack(path).expect("verified parent");
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../planning/minnesota/trainers/minneapolis-cyclists.json"),
        )
        .expect("tracked cyclist authoring");
        let built = crate::build_minneapolis_cyclists_modpack(&base, &source)
            .expect("fresh cyclist publication");
        assert_eq!(
            super::repair_minnesota_script_commands(&built).expect("audit fresh publication"),
            built,
            "fresh content must already have complete executable command records"
        );
    }
}
