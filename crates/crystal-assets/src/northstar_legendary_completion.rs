//! Final, finite legendary encounter contracts and visible recovery directions.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::battle::capture::OPTIONAL_LEGENDARY_BATTLE_TYPE;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_rival_arc::{map, publish};

const MANIFEST: &str = "northstar-legendary-completion-v1";

struct Encounter {
    species: &'static str,
    map: &'static str,
    actor: &'static str,
    caught: &'static str,
    retry_text: &'static str,
}

const ENCOUNTERS: [Encounter; 9] = [
    Encounter {
        species: "BABEHEMOTH",
        map: "MnTallTaleTrail",
        actor: "MN_TALE_BABEHEMOTH",
        caught: "EVENT_MN_TALL_TALE_BABEHEMOTH_CAUGHT",
        retry_text: "MnTaleText_retry",
    },
    Encounter {
        species: "BUNJANT",
        map: "MnTallTaleTrail",
        actor: "MN_TALE_BUNJANT",
        caught: "EVENT_MN_TALL_TALE_BUNJANT_CAUGHT",
        retry_text: "MnTaleText_retry",
    },
    Encounter {
        species: "PEPIVERN",
        map: "MnMississippiLowerWoodsC01R00",
        actor: "MN_PEPIN_PEPIVERN",
        caught: "EVENT_MN_PEPIN_CAUGHT",
        retry_text: "MnPepinText_retreat",
    },
    Encounter {
        species: "ICEMYTH",
        map: "MnIcemythServicePassage",
        actor: "MN_EXHIBIT_ICEMYTH",
        caught: "EVENT_MN_ICEMYTH_CAUGHT",
        retry_text: "MnIcemythText_retreat",
    },
    Encounter {
        species: "STRANDRAK",
        map: "MnInnerBasin",
        actor: "MN_BASIN_STRANDRAK",
        caught: "EVENT_MN_BASIN_CAUGHT",
        retry_text: "MnBasinText_retreat",
    },
    Encounter {
        species: "FLARKINDLE",
        map: "MnLayeredFen",
        actor: "MN_FEN_FLARKINDLE",
        caught: "EVENT_MN_FEN_CAUGHT",
        retry_text: "MnFenText_retreat",
    },
    Encounter {
        species: "VERGLOOM",
        map: "MnR08C00R00",
        actor: "MN_ANCIENT_MIRE_VERGLOOM",
        caught: "EVENT_MN_ANCIENT_MIRE_VERGLOOM_CAUGHT",
        retry_text: "MnAncientRetreatText",
    },
    Encounter {
        species: "KETTLURK",
        map: "MnDevilsKettle",
        actor: "MN_DEVILS_KETTLURK",
        caught: "EVENT_MN_DEVILS_KETTLE_CAUGHT",
        retry_text: "MnKettleRetryText",
    },
    Encounter {
        species: "VERDIGRIM",
        map: "MnBoundaryIntake",
        actor: "MN_VERDIGRIM_INTAKE",
        caught: "EVENT_MN_VERDIGRIM_CAUGHT",
        retry_text: "MnVerdigrimRetryText",
    },
];

/// Finishes recovery instructions on the nine existing active rare encounters.
///
/// Requires all authored discovery maps and the existing restoration reward.
/// No new species, geography, progression prerequisites or rewards are added.
/// Every battle keeps the native optional-legendary capture policy and saved
/// caught/withdrawal state.
/// Restoration's final page points home to the existing one-time Master Ball.
/// Item delivery and its saved claim complete before confirmation is shown.
/// Existing claim flags and saved script instruction offsets stay unchanged.
///
/// # Errors
/// Rejects an incomplete/changed active roster or encounter contract, missing
/// native flags, repeated installation, and invalid input/output packs.
pub fn build_northstar_legendary_completion(
    base: &CompiledGamePack,
    roster: &str,
    curation: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "legendary completion already installed"
    );
    let roster: Value = serde_json::from_str(roster)?;
    let curation: Value = serde_json::from_str(curation)?;
    let retired: BTreeSet<_> = curation["retired_families"]
        .as_array()
        .context("retired families")?
        .iter()
        .filter_map(|f| f["id"].as_str())
        .collect();
    let mut active = BTreeSet::new();
    for family in roster["families"].as_array().context("roster families")? {
        if retired.contains(family["id"].as_str().context("family ID")?)
            || !matches!(family["rarity"].as_str(), Some("legendary" | "mythical"))
        {
            continue;
        }
        for member in family["members"].as_array().context("family members")? {
            active.insert(
                member["name"]
                    .as_str()
                    .context("species name")?
                    .to_ascii_uppercase(),
            );
        }
    }
    ensure!(
        active == ENCOUNTERS.iter().map(|e| e.species.to_owned()).collect(),
        "the finite active legendary/mythical roster changed"
    );
    let mut pack = base.clone();
    let mut updated_text = BTreeSet::new();
    for encounter in &ENCOUNTERS {
        ensure!(
            base.data.pokemon.contains_key(encounter.species),
            "missing active species {}",
            encounter.species
        );
        ensure!(
            base.data
                .story_event_script_constants
                .global
                .contains_key(encounter.caught),
            "missing caught flag {}",
            encounter.caught
        );
        let mut room = map(&pack, encounter.map)?;
        ensure!(
            room.objects.iter().any(|o| o.script == encounter.actor),
            "missing encounter actor {}",
            encounter.actor
        );
        let program = room
            .scripts
            .get(encounter.actor)
            .and_then(Value::as_array)
            .context("native encounter root")?;
        ensure!(
            program
                .iter()
                .any(|r| r["command"] == "loadwildmon" && r["args"][0] == encounter.species)
                && program
                    .iter()
                    .any(|r| r["command"] == "loadvar"
                        && r["args"][1] == OPTIONAL_LEGENDARY_BATTLE_TYPE)
                && program
                    .iter()
                    .any(|r| r["command"] == "special" && r["args"][0] == "CheckCaughtCelebi"),
            "changed native encounter {}",
            encounter.species
        );
        ensure!(
            room.scripts.contains_key(encounter.retry_text),
            "missing retry text {}",
            encounter.retry_text
        );
        // Do not shift instruction offsets in existing scripts: Continue can
        // hold an instruction immediately after a battle or item handoff.
        if updated_text.insert((encounter.map, encounter.retry_text)) {
            append_pages(
                &mut room,
                encounter.retry_text,
                &[
                    ["Leave this area,", "then return to"],
                    ["try again. Your", "clues stay saved."],
                ],
            )?;
        }
        publish(&mut pack, room)?;
    }
    let mut intake = map(&pack, "MnBoundaryIntake")?;
    append_pages(
        &mut intake,
        "MnArcIntakeDoneText",
        &[
            ["TAMARACK kept a", "MASTER BALL for"],
            ["you. Visit the", "WINONA field lab."],
        ],
    )?;
    publish(&mut pack, intake)?;
    let mut station = map(&pack, "MnWinonaFieldStation")?;
    let reward = station
        .scripts
        .get_mut("MnRewardTamarack")
        .and_then(Value::as_array_mut)
        .context("restoration reward root")?;
    ensure!(
        reward
            .iter()
            .any(|r| r == &c("checkevent", &["EVENT_MN_SURVEY_RESTORED"]))
            && reward
                .iter()
                .any(|r| r == &c("verbosegiveitem", &["MASTER_BALL", "1"])
                    || r == &c("giveitem", &["MASTER_BALL", "1"]))
            && reward
                .iter()
                .any(|r| r == &c("iffalse", &["MnRewardStationFull"]))
            && reward
                .iter()
                .any(|r| r == &c("setevent", &["EVENT_MN_MASTER_BALL_RECEIVED"])),
        "existing one-time restoration reward required"
    );
    // Preserve offsets in the old root for saves holding its continuation.
    // A nonverbose native grant does not yield before the following claim.
    for row in reward {
        if *row == c("verbosegiveitem", &["MASTER_BALL", "1"]) {
            *row = c("giveitem", &["MASTER_BALL", "1"]);
        }
    }
    let given = station
        .scripts
        .get_mut("MnRewardMasterGivenText")
        .and_then(Value::as_array_mut)
        .context("Master Ball confirmation")?;
    if !given.iter().any(|row| {
        row["args"].as_array().is_some_and(|args| {
            args.iter()
                .any(|arg| arg.as_str().is_some_and(|s| s.contains("received")))
        })
    }) {
        ensure!(
            given.first().is_some_and(|row| row["command"] == "text"),
            "native confirmation first line"
        );
        given[0]["command"] = Value::String("para".into());
        given.splice(
            0..0,
            [
                c("text", &["\"You received a\""]),
                c("line", &["\"MASTER BALL!\""]),
            ],
        );
    }
    append_pages(
        &mut station,
        "MnRewardMasterGivenText",
        &[
            ["Use it in a wild", "Pokemon battle."],
            ["A trainer keeps", "their Pokemon."],
        ],
    )?;
    publish(&mut pack, station)?;
    // This overlay changes no terrain. Do not repair pre-existing differences
    // between a full map and the separate attribute cache as a side effect.
    pack.data
        .map_attributes
        .clone_from(&base.data.map_attributes);
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

fn append_pages(room: &mut crate::MapModule, label: &str, pages: &[[&str; 2]]) -> Result<()> {
    let text = room
        .scripts
        .get_mut(label)
        .and_then(Value::as_array_mut)
        .context("native existing text")?;
    ensure!(
        text.last() == Some(&c("done", &[])),
        "native text terminator {label}"
    );
    let mut extra = Vec::new();
    for [first, second] in pages {
        ensure!(
            first.is_ascii() && second.is_ascii() && first.len() <= 18 && second.len() <= 18,
            "native page width"
        );
        extra.push(c("para", &[&format!("\"{first}\"")]));
        extra.push(c("line", &[&format!("\"{second}\"")]));
    }
    text.splice(text.len() - 1..text.len() - 1, extra);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires CRYSTAL_LEGENDARY_PARENT_PACK and current authored roster"]
    fn legendary_completion_preserves_gameplay_programs_and_rejects_incomplete_roster() {
        let path =
            std::env::var_os("CRYSTAL_LEGENDARY_PARENT_PACK").expect("completed parent pack");
        let base = crate::read_verified_compiled_game_pack(path).expect("verified native parent");
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/pokemon");
        let roster = std::fs::read_to_string(source.join("roster.json")).expect("roster");
        let curation =
            std::fs::read_to_string(source.join("roster-curation.json")).expect("curation");
        let result = build_northstar_legendary_completion(&base, &roster, &curation)
            .expect("finite completion");
        assert_eq!(base.data.maps.len(), result.data.maps.len());
        assert_eq!(base.runtime_files, result.runtime_files);
        assert!(base.compiled_audio() == result.compiled_audio());
        for (name, before) in &base.data.maps {
            let after = &result.data.maps[name];
            let mut allowed = BTreeSet::new();
            for e in &ENCOUNTERS {
                if e.map == name {
                    allowed.insert(e.retry_text);
                }
            }
            if name == "MnBoundaryIntake" {
                allowed.insert("MnArcIntakeDoneText");
            }
            if name == "MnWinonaFieldStation" {
                allowed.insert("MnRewardMasterGivenText");
            }
            for (label, script) in &before.scripts {
                if name == "MnWinonaFieldStation" && label == "MnRewardTamarack" {
                    let mut expected = script.clone();
                    for row in expected.as_array_mut().expect("reward program") {
                        if *row == c("verbosegiveitem", &["MASTER_BALL", "1"]) {
                            *row = c("giveitem", &["MASTER_BALL", "1"]);
                        }
                    }
                    assert_eq!(&expected, &after.scripts[label]);
                } else if !allowed.contains(label.as_str()) {
                    assert_eq!(
                        script, &after.scripts[label],
                        "gameplay program changed: {name}/{label}"
                    );
                }
            }
            let mut restored = after.clone();
            restored.scripts = before.scripts.clone();
            restored.script_text_bodies = before.script_text_bodies.clone();
            restored.script_text_commands = before.script_text_commands.clone();
            if name == "MnWinonaFieldStation" {
                let mut expected = before.script_item_grants.clone();
                for grant in &mut expected {
                    if grant.source_script == "MnRewardTamarack" && grant.item_id == "MASTER_BALL" {
                        grant.command = "giveitem".into();
                        grant.verbose = false;
                    }
                }
                assert_eq!(after.script_item_grants, expected);
                restored.script_item_grants = before.script_item_grants.clone();
            }
            assert_eq!(
                &restored, before,
                "only reviewed text and atomic grant may change: {name}"
            );
        }
        let mut before = serde_json::to_value(&base.data).expect("parent data");
        let mut after = serde_json::to_value(&result.data).expect("result data");
        for value in [&mut before, &mut after] {
            let object = value.as_object_mut().expect("native dataset");
            object.remove("maps");
            object.remove("map_scripts");
        }
        for (field, value) in before.as_object().expect("parent fields") {
            assert!(
                value == &after[field],
                "unrelated catalog or split payload changed: {field}"
            );
        }
        assert!(build_northstar_legendary_completion(&result, &roster, &curation).is_err());
        let mut missing: Value = serde_json::from_str(&roster).expect("source roster");
        missing["families"]
            .as_array_mut()
            .expect("families")
            .retain(|f| f["id"] != "MN_F044");
        assert!(
            build_northstar_legendary_completion(&base, &missing.to_string(), &curation).is_err()
        );
    }
}
