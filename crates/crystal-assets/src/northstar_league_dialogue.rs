//! Authored Elite Four, Champion and Hall of Fame dialogue on installed rooms.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::grand_marais_modpack::authoring::command as c;

const MANIFEST: &str = "northstar-league-finale-v1";
const CHAMPION: &str = "EVENT_MN_LEAGUE_CHAMPION";
const MEMBERS: [(&str, &str, &str); 5] = [
    ("MN_LEAGUE_1", "FERN", "MnLeagueRoom1"),
    ("MN_LEAGUE_2", "FLINT", "MnLeagueRoom2"),
    ("MN_LEAGUE_3", "VEIL", "MnLeagueRoom3"),
    ("MN_LEAGUE_4", "HALDEN", "MnLeagueRoom4"),
    ("MN_CHAMPION", "CASEY", "MnLeagueChampion"),
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    id: String,
    members: Vec<Member>,
    support: BTreeMap<String, BTreeMap<String, Vec<String>>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Member {
    id: String,
    name: String,
    map: String,
    role: String,
    texts: BTreeMap<String, Vec<String>>,
}

fn source(json: &str) -> Result<Source> {
    let source: Source = serde_json::from_str(json)?;
    ensure!(
        source.id == MANIFEST && source.members.len() == 5,
        "complete League cast required"
    );
    for (member, (id, name, map)) in source.members.iter().zip(MEMBERS) {
        ensure!(
            member.id == id && member.name == name && member.map == map && !member.role.is_empty(),
            "League identity differs"
        );
        let expected: BTreeSet<_> = ["Before", "Won", "Lost", "After", "NoRun", "Rematch"]
            .map(|kind| format!("{map}{kind}Text"))
            .into_iter()
            .collect();
        ensure!(
            member.texts.keys().cloned().collect::<BTreeSet<_>>() == expected,
            "complete member dialogue required: {map}"
        );
    }
    let support = BTreeMap::from([
        (
            "MnLeagueLobby",
            BTreeSet::from([
                "MnElyRegistrationText",
                "MnElyBadgeDeniedText",
                "MnElySurveyDeniedText",
                "MnElyRegistrationDeclinedText",
            ]),
        ),
        (
            "MnLeagueHallOfFame",
            BTreeSet::from(["MnElyRecordText", "MnElyRecordDeniedText"]),
        ),
    ]);
    ensure!(
        source.support.len() == support.len(),
        "League support rooms differ"
    );
    for (map, labels) in support {
        let texts = source.support.get(map).context("League support room")?;
        ensure!(
            texts.keys().map(String::as_str).collect::<BTreeSet<_>>() == labels,
            "League support texts differ"
        );
    }
    for texts in source
        .members
        .iter()
        .map(|m| &m.texts)
        .chain(source.support.values())
    {
        for (label, paragraphs) in texts {
            ensure!(
                !paragraphs.is_empty()
                    && paragraphs.len() <= 5
                    && paragraphs.iter().all(|p| !p.is_empty()
                        && p.is_ascii()
                        && !p.chars().any(|c| c.is_control() || c == '"')
                        && p.split_whitespace().all(|w| w.len() <= 18)),
                "invalid native League text: {label}"
            );
        }
    }
    Ok(source)
}

/// Writes distinct League scenes and championship-aware rematch introductions.
///
/// Keeps existing native teams, battle rules, victory gates, Hall of Fame,
/// terrain, art and audio. Corrects the fourth member to canonical HALDEN,
/// separating him from the story's rival REED. Reapplying is idempotent.
///
/// # Errors
/// Rejects incomplete dialogue, missing installed League programs, inactive
/// regional team species or invalid native script/pack metadata.
pub fn refresh_northstar_league_dialogue(
    base: &CompiledGamePack,
    dialogue: &str,
    roster: &str,
    curation: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let source = source(dialogue)?;
    let roster: Value = serde_json::from_str(roster)?;
    let curation: Value = serde_json::from_str(curation)?;
    let retired: BTreeSet<_> = curation["retired_families"]
        .as_array()
        .context("curation")?
        .iter()
        .filter_map(|f| f["id"].as_str())
        .collect();
    let active: BTreeSet<_> = roster["families"]
        .as_array()
        .context("roster families")?
        .iter()
        .filter(|f| f["id"].as_str().is_some_and(|id| !retired.contains(id)))
        .flat_map(|f| f["members"].as_array().into_iter().flatten())
        .filter_map(|m| m["name"].as_str().map(str::to_ascii_uppercase))
        .collect();
    let mut pack = base.clone();
    for member in &source.members {
        let trainer = pack
            .data
            .trainers
            .trainers
            .get_mut(&member.id)
            .context("installed League trainer")?;
        ensure!(
            !trainer.party.is_empty() && trainer.party.iter().all(|p| active.contains(&p.species)),
            "League team must use active Minnesota Pokemon: {}",
            member.id
        );
        trainer.name.clone_from(&member.name);
        let mut room = pack
            .data
            .maps
            .get(&member.map)
            .context("installed League room")?
            .clone();
        for (label, paragraphs) in &member.texts {
            ensure!(
                label.ends_with("RematchText") || room.script_text_bodies.contains_key(label),
                "missing League text {label}"
            );
            authoring::text(
                &mut room,
                label,
                &paragraphs.iter().map(String::as_str).collect::<Vec<_>>(),
            );
        }
        let challenge = format!("{}Challenge", member.map);
        let start = format!("{}BattleStart", member.map);
        let rematch = format!("{}RematchIntro", member.map);
        if !room.scripts.contains_key(&start) {
            let original = room
                .scripts
                .get(&challenge)
                .and_then(Value::as_array)
                .context("native League challenge")?;
            let intro = original
                .iter()
                .position(|v| v["command"] == "opentext")
                .context("native League introduction")?;
            let battle = original
                .iter()
                .position(|v| v["command"] == "winlosstext")
                .context("native League result boundary")?;
            ensure!(
                intro < battle
                    && original[battle..]
                        .iter()
                        .any(|v| v["command"] == "startbattle"),
                "native League battle sequence differs"
            );
            let mut root = original[..intro].to_vec();
            root.extend([c("checkevent", &[CHAMPION]), c("iftrue", &[&rematch])]);
            root.extend_from_slice(&original[intro..battle]);
            root.push(c("sjump", &[&start]));
            let tail = original[battle..].to_vec();
            room.scripts.insert(challenge, Value::Array(root));
            room.scripts.insert(start.clone(), Value::Array(tail));
            room.scripts.insert(
                rematch,
                Value::Array(vec![
                    c("opentext", &[]),
                    c("writetext", &[&format!("{}RematchText", member.map)]),
                    c("waitbutton", &[]),
                    c("closetext", &[]),
                    c("sjump", &[&start]),
                ]),
            );
        }
        crate::northstar_rival_arc::publish(&mut pack, room)?;
    }
    for (map, texts) in &source.support {
        let mut room = pack
            .data
            .maps
            .get(map)
            .context("installed League support room")?
            .clone();
        for (label, paragraphs) in texts {
            ensure!(
                room.script_text_bodies.contains_key(label),
                "missing League support text {label}"
            );
            authoring::text(
                &mut room,
                label,
                &paragraphs.iter().map(String::as_str).collect::<Vec<_>>(),
            );
        }
        crate::northstar_rival_arc::publish(&mut pack, room)?;
    }
    if !pack.report.manifests.iter().any(|m| m == MANIFEST) {
        pack.report.manifests.push(MANIFEST.into());
    }
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
    use super::*;

    const DIALOGUE: &str =
        include_str!("../../../planning/minnesota/revisions/league-finale-v01/dialogue.json");

    #[test]
    fn league_cast_has_complete_native_scenes_and_rejects_missing_or_moved_members() {
        let parsed = source(DIALOGUE).expect("authored League dialogue");
        assert_eq!(parsed.members[3].name, "HALDEN");
        for alteration in 0..3 {
            let mut bad: Value = serde_json::from_str(DIALOGUE).expect("source");
            match alteration {
                0 => {
                    bad["members"][0]["texts"]
                        .as_object_mut()
                        .expect("texts")
                        .remove("MnLeagueRoom1LostText");
                }
                1 => bad["members"][3]["name"] = "REED".into(),
                _ => {
                    bad["members"][4]["texts"]["MnLeagueChampionBeforeText"] =
                        serde_json::json!(["unrenderablewordover18columns"])
                }
            }
            assert!(source(&bad.to_string()).is_err());
        }
    }

    #[test]
    #[ignore = "requires completed regional pack"]
    fn league_dialogue_preserves_teams_terrain_audio_and_unrelated_maps() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let path = std::env::var_os("CRYSTAL_LEAGUE_DIALOGUE_PACK")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                root.join("content-packs/minnesota-region-polish-final-v03.crystalpack")
            });
        let base = crate::read_verified_compiled_game_pack(path).expect("regional pack");
        let roster = std::fs::read_to_string(root.join("planning/minnesota/pokemon/roster.json"))
            .expect("roster");
        let curation =
            std::fs::read_to_string(root.join("planning/minnesota/pokemon/roster-curation.json"))
                .expect("curation");
        let result = refresh_northstar_league_dialogue(&base, DIALOGUE, &roster, &curation)
            .expect("League dialogue overlay");
        assert_eq!(base.runtime_files(), result.runtime_files());
        assert!(base.compiled_audio() == result.compiled_audio());
        assert_eq!(base.data.pokemon, result.data.pokemon);
        assert_eq!(base.data.wild_encounters, result.data.wild_encounters);
        for (id, trainer) in &base.data.trainers.trainers {
            let mut expected = trainer.clone();
            if id == "MN_LEAGUE_4" {
                expected.name = "HALDEN".into();
            }
            assert_eq!(result.data.trainers.trainers[id], expected);
        }
        for (name, map) in &base.data.maps {
            let after = &result.data.maps[name];
            if name.starts_with("MnLeague") {
                assert_eq!(map.blocks, after.blocks);
                assert_eq!(map.attributes, after.attributes);
                assert_eq!(map.objects, after.objects);
                assert_eq!(map.events, after.events);
            } else {
                assert_eq!(map, after, "unrelated {name}");
            }
        }
        let repeated = refresh_northstar_league_dialogue(&result, DIALOGUE, &roster, &curation)
            .expect("idempotent refresh");
        assert_eq!(
            repeated.identity().expect("identity"),
            result.identity().expect("identity")
        );
    }
}
