//! Repair repeated paperwork language in early regional ambient dialogue.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::northstar_rival_arc::map;

const MANIFEST: &str = "minnesota-ambient-dialogue-v1";

/// Replaces two river watchers' speeches and the Winona station sign.
///
/// The actors, interaction scripts, and all other game content are retained.
///
/// # Errors
/// Rejects invalid packs, repeated installation, or changed source conversations.
pub fn revise_minnesota_ambient_dialogue(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|item| item == MANIFEST),
        "dialogue already revised"
    );
    let mut pack = base.clone();
    for (map_name, replacements) in [
        (
            "MnWinona",
            &[
                (
                    "MN_W_RIVER_WATCHER",
                    "MN_W_RIVER_WATCHERText",
                    "We keep a record.",
                    &["Watching barges", "drift downriver", "is how I relax."][..],
                ),
                (
                    "MnWStationSign",
                    "MnWStationSignText",
                    "Keep original",
                    &["FIELD STATION", "PROF. TAMARACK"][..],
                ),
            ][..],
        ),
        (
            "MnGrandRapids",
            &[(
                "MN_GR_RIVER_WATCHER",
                "MN_GR_RIVER_WATCHERText",
                "Keep the records.",
                &[
                    "I came for a quiet",
                    "morning by the",
                    "water. The ducks",
                    "had other plans!",
                ][..],
            )][..],
        ),
    ] {
        let mut room = map(&pack, map_name)?;
        for &(interaction, label, expected, lines) in replacements {
            let script = room
                .scripts
                .get(interaction)
                .with_context(|| format!("missing interaction {map_name}/{interaction}"))?;
            ensure!(
                script
                    .as_array()
                    .is_some_and(|commands| commands.iter().any(|command| {
                        matches!(
                            command["command"].as_str(),
                            Some("jumptext" | "jumptextfaceplayer")
                        ) && command["args"][0] == label
                    })),
                "unexpected interaction {map_name}/{interaction}"
            );
            let original = room
                .scripts
                .get(label)
                .with_context(|| format!("missing {map_name}/{label}"))?;
            ensure!(
                original.to_string().contains(expected),
                "unexpected source dialogue {map_name}/{label}"
            );
            authoring::text(&mut room, label, lines);
        }
        room.script_text_bodies = crate::parse_script_text_bodies(&room.id, &room.scripts)?;
        crate::authored_town_publication::replace(&mut pack, room)?;
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
    use super::revise_minnesota_ambient_dialogue;

    #[test]
    fn authored_town_lines_match_the_revision() {
        for (source, collection, id, expected) in [
            (
                include_str!("../../../planning/minnesota/towns/MnWinona.json"),
                "residents",
                "MN_W_RIVER_WATCHER",
                &["Watching barges", "drift downriver", "is how I relax."][..],
            ),
            (
                include_str!("../../../planning/minnesota/towns/MnWinona.json"),
                "signs",
                "MnWStationSign",
                &["FIELD STATION", "PROF. TAMARACK"][..],
            ),
            (
                include_str!("../../../planning/minnesota/towns/MnGrandRapids.json"),
                "residents",
                "MN_GR_RIVER_WATCHER",
                &[
                    "I came for a quiet",
                    "morning by the",
                    "water. The ducks",
                    "had other plans!",
                ][..],
            ),
        ] {
            let town: serde_json::Value = serde_json::from_str(source).expect("town source");
            let item = town[collection]
                .as_array()
                .expect("town collection")
                .iter()
                .find(|item| item["id"] == id)
                .expect("authored interaction");
            assert_eq!(item["lines"], serde_json::json!(expected));
        }
    }

    #[test]
    #[ignore = "requires the external parent pack in CRYSTAL_AMBIENT_DIALOGUE_PACK"]
    fn revision_preserves_interactions_and_installs_reviewed_text() {
        let path = std::env::var_os("CRYSTAL_AMBIENT_DIALOGUE_PACK")
            .expect("set CRYSTAL_AMBIENT_DIALOGUE_PACK");
        let parent = crate::read_verified_compiled_game_pack(path).expect("valid parent pack");
        let child = revise_minnesota_ambient_dialogue(&parent).expect("revise dialogue");
        for (map_name, label, expected) in [
            ("MnWinona", "MN_W_RIVER_WATCHERText", "Watching barges"),
            ("MnWinona", "MnWStationSignText", "PROF. TAMARACK"),
            (
                "MnGrandRapids",
                "MN_GR_RIVER_WATCHERText",
                "water. The ducks",
            ),
        ] {
            let before = &parent.data.maps[map_name];
            let after = &child.data.maps[map_name];
            assert_eq!(before.events, after.events, "interaction bindings changed");
            assert_eq!(before.objects, after.objects, "actors changed");
            assert!(format!("{:?}", after.script_text_bodies[label]).contains(expected));
        }
        for (map_name, labels) in [
            (
                "MnWinona",
                &["MN_W_RIVER_WATCHERText", "MnWStationSignText"][..],
            ),
            ("MnGrandRapids", &["MN_GR_RIVER_WATCHERText"][..]),
        ] {
            let mut expected = parent.data.maps[map_name].clone();
            let actual = &child.data.maps[map_name];
            for label in labels {
                expected
                    .scripts
                    .insert((*label).into(), actual.scripts[*label].clone());
                expected
                    .script_text_bodies
                    .insert((*label).into(), actual.script_text_bodies[*label].clone());
            }
            assert_eq!(
                &expected, actual,
                "unrelated map content changed in {map_name}"
            );
        }
        for (name, before) in &parent.data.maps {
            if name != "MnWinona" && name != "MnGrandRapids" {
                assert_eq!(before, &child.data.maps[name], "unrelated map {name}");
            }
        }
        assert_eq!(parent.data.audio, child.data.audio);
        assert_eq!(parent.runtime_files, child.runtime_files);
        assert!(revise_minnesota_ambient_dialogue(&child).is_err());
    }
}
