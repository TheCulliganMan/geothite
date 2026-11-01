//! Repair three early Minnesota conversations in an existing pack.

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;

const MANIFEST: &str = "minnesota-rochester-dialogue-repair-v1";

/// Replaces the Rochester city sign, city guide and Driftless creek child's text.
///
/// # Errors
/// Returns an error if the expected actors or scripts are missing, the repair
/// is already installed, or the revised pack fails runtime validation.
pub fn repair_rochester_dialogue(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|entry| entry == MANIFEST),
        "repair already installed"
    );
    let mut pack = base.clone();
    for (map_id, entries) in [
        (
            "MnRochester",
            &[
                (
                    "MnRCCitySign",
                    "MnRCCitySignText",
                    &["ROCHESTER", "A CITY OF CARE"][..],
                ),
                (
                    "MN_RC_CITY_GUIDE",
                    "MN_RC_CITY_GUIDEText",
                    &[
                        "VAREL trains here.",
                        "Their partners are",
                        "strong, and they",
                        "look happy, too.",
                    ][..],
                ),
            ][..],
        ),
        (
            "MnR01C00R00",
            &[(
                "MN_R01_CREEK_CHILD",
                "MN_R01_CREEK_CHILDText",
                &[
                    "The man inside",
                    "loves fishing.",
                    "I like watching",
                    "the creek, too.",
                ][..],
            )][..],
        ),
    ] {
        let mut map = pack.data.maps.get(map_id).context("installed map")?.clone();
        for &(script, label, lines) in entries {
            let body = map.scripts.get(script).context("interaction script")?;
            ensure!(
                body.as_array().is_some_and(|commands| commands
                    .iter()
                    .any(|command| command["args"][0] == label)),
                "interaction {map_id}/{script} no longer points to {label}"
            );
            ensure!(
                map.scripts.contains_key(label),
                "missing text {map_id}/{label}"
            );
            authoring::text(&mut map, label, lines);
            pack.data
                .map_scripts
                .insert(label.into(), map.scripts[label].clone());
        }
        map.script_text_commands = crate::parse_script_text_commands(map_id, &map.scripts)?;
        map.script_text_bodies = crate::parse_script_text_bodies(map_id, &map.scripts)?;
        pack.data.maps.insert(map_id.into(), map);
    }
    pack.report.manifests.push(MANIFEST.into());
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

#[cfg(test)]
mod tests {
    use super::repair_rochester_dialogue;

    #[test]
    #[ignore = "requires CRYSTAL_ROCHESTER_DIALOGUE_PARENT"]
    fn revised_pack_preserves_interactions_and_audio() {
        let path =
            std::env::var_os("CRYSTAL_ROCHESTER_DIALOGUE_PARENT").expect("external review parent");
        let parent = crate::read_verified_compiled_game_pack(path).expect("verified parent");
        let revised = repair_rochester_dialogue(&parent).expect("repaired pack");
        for (map, label, expected) in [
            (
                "MnRochester",
                "MnRCCitySignText",
                &[("text", "ROCHESTER"), ("line", "A CITY OF CARE")][..],
            ),
            (
                "MnRochester",
                "MN_RC_CITY_GUIDEText",
                &[
                    ("text", "VAREL trains here."),
                    ("line", "Their partners are"),
                    ("para", "strong, and they"),
                    ("line", "look happy, too."),
                ][..],
            ),
            (
                "MnR01C00R00",
                "MN_R01_CREEK_CHILDText",
                &[
                    ("text", "The man inside"),
                    ("line", "loves fishing."),
                    ("para", "I like watching"),
                    ("line", "the creek, too."),
                ][..],
            ),
        ] {
            let before = &parent.data.maps[map];
            let after = &revised.data.maps[map];
            assert_eq!(before.events, after.events);
            assert_eq!(before.objects, after.objects);
            let actual = after.scripts[label].as_array().expect("text program");
            assert_eq!(actual.len(), expected.len() + 1, "{map}/{label}");
            for (command, &(kind, line)) in actual.iter().zip(expected) {
                assert_eq!(command["command"], kind, "{map}/{label}");
                assert_eq!(command["args"][0], format!("\"{line}\""), "{map}/{label}");
            }
            assert_eq!(actual.last().expect("terminator")["command"], "done");
            assert_eq!(revised.data.map_scripts[label], after.scripts[label]);
        }
        for (map, before) in &parent.data.maps {
            let after = &revised.data.maps[map];
            if map != "MnRochester" && map != "MnR01C00R00" {
                assert_eq!(before, after, "unrelated map changed: {map}");
            } else {
                for (label, program) in &before.scripts {
                    if !matches!(
                        label.as_str(),
                        "MnRCCitySignText" | "MN_RC_CITY_GUIDEText" | "MN_R01_CREEK_CHILDText"
                    ) {
                        assert_eq!(
                            Some(program),
                            after.scripts.get(label),
                            "unrelated script changed: {map}/{label}"
                        );
                    }
                }
            }
        }
        assert_eq!(parent.data.audio, revised.data.audio);
        assert!(repair_rochester_dialogue(&revised).is_err());
    }
}
