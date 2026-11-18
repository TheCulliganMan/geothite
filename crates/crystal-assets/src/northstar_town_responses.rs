//! Residents describe their Minnesota partners before and after local events.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::authored_town_publication;
use crate::ely_league_rooms;
use crate::grand_marais_modpack::authoring::command;
use crate::northstar_regional_roster::RegionalRoster;

const SOURCE: &str =
    include_str!("../../../planning/minnesota/revisions/town-responses-v01/dialogue.json");

struct Resident {
    map: &'static str,
    script: &'static str,
    check: &'static str,
    flag: &'static str,
    after: &'static str,
}

const RESIDENTS: [Resident; 4] = [
    Resident {
        map: "MnStCloud",
        script: "MN_ST_CLOUD_STONECUTTER",
        check: "checkevent",
        flag: "EVENT_MN_REED_WORKER_RELEASED",
        after: "MnTownStonecutterAfterText",
    },
    Resident {
        map: "MnStCloud",
        script: "MN_ST_CLOUD_RIVER_WATCHER",
        check: "checkevent",
        flag: "EVENT_MN_SURVEY_RESTORED",
        after: "MnTownRiverWatcherAfterText",
    },
    Resident {
        map: "MnDuluth",
        script: "MN_DULUTH_X_DOCK_HAND",
        check: "checkevent",
        flag: "EVENT_MN_SURVEY_RESTORED",
        after: "MnTownDockHandAfterText",
    },
    Resident {
        map: "MnDuluth",
        script: "MnDuluth_MN_DULUTH_ISLAND_ARTIST",
        check: "checkevent",
        flag: "EVENT_MN_SURVEY_RESTORED",
        after: "MnTownPainterAfterText",
    },
];

/// Revises partner stories and return replies without changing rewards or actors.
///
/// The combined release owns identity and save compatibility. This leaf accepts
/// the existing single-reply residents only, then rebuilds their runtime tables.
/// Returns an error for changed speakers, unexpected scripts or invalid pages.
pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let dialogue: BTreeMap<String, BTreeMap<String, Vec<AuthoredDialoguePage>>> =
        serde_json::from_str(SOURCE)?;
    ensure!(dialogue.len() == 2, "expected two town dialogue maps");
    let roster = RegionalRoster::current()?;
    for species in ["PENDULOP", "BIRCHIT", "LAKERNAUT", "LOOBIT"] {
        ensure!(
            roster.allowed.contains(species) && pack.data.pokemon.contains_key(species),
            "town partner must be an installed active Minnesota species: {species}"
        );
    }
    // Prepare both maps before publishing either, so preimage failures cannot
    // leave half of this revision installed.
    let mut revised = Vec::new();
    for (id, texts) in dialogue {
        let original = pack.data.maps.get(&id).context("town response map")?;
        let mut map = original.clone();
        let residents: Vec<_> = RESIDENTS.iter().filter(|r| r.map == id).collect();
        ensure!(texts.len() == residents.len() * 2, "unexpected town texts");
        for resident in residents {
            let before = format!("{}Text", resident.script);
            let branch = format!("{}After", resident.script);
            ensure!(
                map.objects
                    .iter()
                    .any(|actor| actor.script == resident.script)
                    && map.scripts.get(resident.script)
                        == Some(&json!([command("jumptextfaceplayer", &[&before])]))
                    && map.script_text_bodies.contains_key(&before)
                    && !map.scripts.contains_key(&branch)
                    && !map.scripts.contains_key(resident.after),
                "changed town resident: {id}/{}",
                resident.script
            );
            let first_pages = texts.get(&before).context("resident first pages")?;
            ensure!(
                map.scripts[&before].as_array().and_then(|rows| rows.last())
                    == Some(&command("done", &[])),
                "ordinary resident text terminator"
            );
            map.scripts
                .insert(before.clone(), authored_dialogue::compile(first_pages)?);
            let pages = texts.get(resident.after).context("resident return pages")?;
            map.scripts
                .insert(resident.after.into(), authored_dialogue::compile(pages)?);
            map.scripts.insert(
                resident.script.into(),
                json!([
                    command("faceplayer", &[]),
                    command(resident.check, &[resident.flag]),
                    command("iftrue", &[&branch]),
                    command("jumptext", &[&before]),
                ]),
            );
            map.scripts
                .insert(branch, json!([command("jumptext", &[resident.after])]));
        }
        ely_league_rooms::finish(pack, &mut map)?;
        map.script_audio_commands = crate::parse_script_audio_commands(&id, &map.scripts)?;
        map.map_event_section_commands
            .clone_from(&original.map_event_section_commands);
        revised.push(map);
    }
    for map in revised {
        authored_town_publication::replace(pack, map)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{RESIDENTS, install};
    use crate::grand_marais_modpack::authoring::command;

    #[test]
    #[ignore = "requires CRYSTAL_TOWN_RESPONSES_PACK; static pack inspection only"]
    fn town_responses_preserve_content_and_bind_real_milestones() {
        let path = PathBuf::from(
            std::env::var_os("CRYSTAL_TOWN_RESPONSES_PACK")
                .expect("external town response parent pack"),
        );
        let loaded = crate::read_verified_compiled_game_pack(path).expect("verified parent");
        let original = &loaded;
        let mut revised = original.clone();
        install(&mut revised).expect("install town responses");
        for (id, before) in &original.data.maps {
            let after = &revised.data.maps[id];
            if id != "MnStCloud" && id != "MnDuluth" {
                assert_eq!(before, after, "unrelated map {id}");
                continue;
            }
            assert_eq!(before.objects, after.objects);
            assert_eq!(before.events, after.events);
            assert_eq!(before.blocks, after.blocks);
            assert_eq!(
                before.map_event_section_commands,
                after.map_event_section_commands
            );
            for (label, body) in &before.scripts {
                if !RESIDENTS.iter().any(|r| {
                    r.map == id && (r.script == label || format!("{}Text", r.script) == *label)
                }) {
                    assert_eq!(
                        Some(body),
                        after.scripts.get(label),
                        "existing script {id}/{label}"
                    );
                }
            }
        }
        assert_eq!(original.audio_manifest, revised.audio_manifest);
        assert_eq!(original.runtime_files, revised.runtime_files);
        for resident in RESIDENTS {
            let map = &revised.data.maps[resident.map];
            let rows = map.scripts[resident.script]
                .as_array()
                .expect("resident script");
            assert_eq!(rows[0], command("faceplayer", &[]));
            assert_eq!(rows[1], command(resident.check, &[resident.flag]));
            assert_eq!(
                rows[3],
                command("jumptext", &[&format!("{}Text", resident.script)])
            );
            assert!(map.script_text_bodies.contains_key(resident.after));
        }
        revised.identity = crate::derive_compiled_game_pack_identity_from_manifest(
            revised.format_version,
            &revised.data,
            &revised.audio_manifest,
            &revised.runtime_files,
            &revised.report,
        )
        .expect("review identity");
        crate::verify_compiled_game_pack_for_runtime(&revised).expect("valid native tables");
        if let Some(path) = std::env::var_os("CRYSTAL_TOWN_RESPONSES_OUTPUT") {
            let path = PathBuf::from(path);
            assert!(!path.exists(), "preserve previous review output");
            assert!(
                std::process::Command::new("git")
                    .args(["check-ignore", "--quiet", "--no-index", "--"])
                    .arg(&path)
                    .status()
                    .expect("ignored output check")
                    .success()
            );
            revised
                .write_preserving_storage(&path)
                .expect("write review pack");
            assert_eq!(
                crate::read_verified_compiled_game_pack(&path).expect("readback"),
                revised
            );
        }
        // Reject duplicate installation rather than stacking another branch.
        assert!(install(&mut revised).is_err());
    }
}
