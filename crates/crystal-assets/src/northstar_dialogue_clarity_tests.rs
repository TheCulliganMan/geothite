//! Verify publication, separate guards and preservation against the real parent.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use super::{
    LOBBY, PREPARATION, REGISTRATION, Revision, SAVE_POLICY, SOUTH_CLEAR_TEXT, SOUTH_TALK,
    SOUTH_TEXT, revise_northstar_dialogue_clarity, revise_northstar_ely_preparation,
};
use crate::authored_dialogue::{self, AuthoredDialoguePage};

#[test]
#[ignore = "requires CRYSTAL_ELY_PREPARATION_PARENT"]
fn ely_preparation_only_replaces_the_registrar_text() -> Result<()> {
    let path = std::env::var_os("CRYSTAL_ELY_PREPARATION_PARENT").context("external Fair pack")?;
    let base = crate::read_verified_compiled_game_pack(path)?;
    let roster = crate::northstar_regional_roster::RegionalRoster::current()?;
    for species in ["DOUBLEHEX", "PRAIRONDO", "WAYWOLF", "CANALARM"] {
        assert!(roster.allowed.contains(species), "{species} must be active");
    }
    for (map, species, night) in [
        ("MnR13C00R02", "DOUBLEHEX", true),
        ("MnR13C00R00", "WAYWOLF", false),
        ("MnR10Cave1F", "CANALARM", false),
    ] {
        let grass = base.data.wild_encounters[map]
            .grass
            .as_ref()
            .with_context(|| format!("missing encounters at {map}"))?;
        let table = if night { &grass.night } else { &grass.day };
        assert!(table.iter().any(|entry| entry.species == species));
    }
    for (species, level, move_id) in [
        ("PRAIRONDO", 48, "EARTHQUAKE"),
        ("WAYWOLF", 41, "EARTHQUAKE"),
        ("CANALARM", 37, "THUNDERBOLT"),
    ] {
        assert!(
            base.data.learnsets[species]
                .iter()
                .any(|(learned_at, known)| *learned_at == level && known == move_id),
            "{species} should learn {move_id} at {level}"
        );
    }
    let source = include_str!("../../../planning/minnesota/story.json");
    let revised = revise_northstar_ely_preparation(&base, source)?;
    let json: Value = serde_json::from_str(source)?;
    let mut normalized = revised.data.clone();
    let lobby = normalized.maps.get_mut(LOBBY).context("Ely League lobby")?;
    let original = &base.data.maps[LOBBY];
    for label in [REGISTRATION, PREPARATION] {
        let pages: Vec<AuthoredDialoguePage> =
            serde_json::from_value(json["ely_preparation_dialogue"][label].clone())?;
        let expected = authored_dialogue::compile(&pages)?;
        assert_eq!(lobby.scripts[label], expected);
        assert_eq!(normalized.map_scripts[label], expected);
        let programs = std::collections::BTreeMap::from([(label.to_owned(), expected)]);
        assert_eq!(
            lobby.script_text_bodies[label],
            crate::parse_script_text_bodies(LOBBY, &programs)?[label]
        );
        lobby
            .scripts
            .insert(label.into(), original.scripts[label].clone());
        lobby
            .script_text_bodies
            .insert(label.into(), original.script_text_bodies[label].clone());
        normalized
            .map_scripts
            .insert(label.into(), base.data.map_scripts[label].clone());
    }
    assert!(
        normalized == base.data,
        "changed data outside the two text bodies"
    );
    assert_eq!(revised.audio_manifest, base.audio_manifest);
    Ok(())
}

#[test]
#[ignore = "requires CRYSTAL_DIALOGUE_CLARITY_PARENT"]
fn dialogue_clarity_preserves_game_and_installs_every_reviewed_page() -> Result<()> {
    let path =
        std::env::var_os("CRYSTAL_DIALOGUE_CLARITY_PARENT").context("external Fair parent")?;
    let base = crate::read_verified_compiled_game_pack(path)?;
    let source = include_str!("../../../planning/minnesota/story.json");
    let json: Value = serde_json::from_str(source)?;
    let revision: Revision = serde_json::from_value(json["dialogue_clarity_revision"].clone())?;
    let revised = revise_northstar_dialogue_clarity(&base, source)?;
    let mut normalized = revised.clone();
    for label in &revision.welcome_labels {
        let before = base
            .data
            .runtime_title_screen
            .program
            .text
            .iter()
            .find(|text| &text.id == label)
            .context("original welcome")?;
        let after = normalized
            .data
            .runtime_title_screen
            .program
            .text
            .iter_mut()
            .find(|text| &text.id == label)
            .context("revised welcome")?;
        ensure!(
            before.commands.last() == after.commands.last(),
            "welcome ending changed"
        );
        if label == "OakText3" {
            ensure!(
                before.commands.first() == after.commands.first(),
                "welcome cry acknowledgement changed"
            );
        }
        *after = before.clone();
    }
    for (map, labels) in &revision.dialogue_labels {
        for label in labels {
            let pages: Vec<AuthoredDialoguePage> =
                serde_json::from_value(json["dialogue_overhaul"]["texts"][map][label].clone())?;
            let mut expected = authored_dialogue::compile(&pages)?;
            let actual = &revised.data.maps[map].scripts[label];
            let ending = actual
                .as_array()
                .context("compiled text")?
                .last()
                .context("text ending")?;
            *expected
                .as_array_mut()
                .context("expected text")?
                .last_mut()
                .context("expected ending")? = ending.clone();
            ensure!(actual == &expected, "uninstalled pages: {map}/{label}");
            if let Some(old) = base.data.maps[map].scripts.get(label) {
                ensure!(
                    old.as_array().and_then(|rows| rows.last()) == Some(ending),
                    "changed text ending: {label}"
                );
                let room = normalized
                    .data
                    .maps
                    .get_mut(map)
                    .context("normalized map")?;
                room.scripts.insert(label.clone(), old.clone());
                room.script_text_bodies.insert(
                    label.clone(),
                    base.data.maps[map].script_text_bodies[label].clone(),
                );
            }
            if let Some(old) = base.data.map_scripts.get(label) {
                normalized
                    .data
                    .map_scripts
                    .insert(label.clone(), old.clone());
            }
        }
    }
    let winona = &revised.data.maps["MnWinona"];
    let north = winona
        .objects
        .iter()
        .find(|o| o.object_identifier.as_deref() == Some("MN_W_TRAIL_WARDEN_NORTH"))
        .context("north guard")?;
    let south = winona
        .objects
        .iter()
        .find(|o| o.object_identifier.as_deref() == Some("MN_W_TRAIL_WARDEN_SOUTH"))
        .context("south guard")?;
    ensure!(
        north.script == "MnWTrailWardenTalk" && south.script == SOUTH_TALK,
        "guards still share an interaction"
    );
    ensure!(
        winona.scripts["MnWTrailWardenText"] != winona.scripts[SOUTH_TEXT],
        "first guard replies repeat"
    );
    ensure!(
        winona.scripts["MnWTrailWardenClearedText"] != winona.scripts[SOUTH_CLEAR_TEXT],
        "return guard replies repeat"
    );
    ensure!(
        winona.scripts["MnWTrailWardenSouth"][5]["args"][0] == SOUTH_TEXT,
        "south road trigger uses north reply"
    );
    ensure!(
        winona.events == base.data.maps["MnWinona"].events
            && winona.blocks == base.data.maps["MnWinona"].blocks,
        "guard road or trigger locations changed"
    );
    let mut normalized_guards = winona.objects.clone();
    for (actor, old) in normalized_guards
        .iter_mut()
        .zip(&base.data.maps["MnWinona"].objects)
    {
        actor.script.clone_from(&old.script);
    }
    ensure!(
        normalized_guards == base.data.maps["MnWinona"].objects,
        "guard positions or identities changed"
    );
    let mut before_map = serde_json::to_value(&base.data.maps["MnWinona"])?;
    let mut after_map = serde_json::to_value(winona)?;
    for field in [
        "scripts",
        "script_text_bodies",
        "script_text_commands",
        "script_flag_commands",
        "script_object_commands",
        "script_control_commands",
        "objects",
        "map_event_section_commands",
    ] {
        before_map
            .as_object_mut()
            .context("original map fields")?
            .remove(field);
        after_map
            .as_object_mut()
            .context("revised map fields")?
            .remove(field);
    }
    ensure!(before_map == after_map, "unrelated Winona data changed");
    for field in [
        "script_text_commands",
        "script_flag_commands",
        "script_object_commands",
        "script_control_commands",
    ] {
        let before_rows = serde_json::to_value(&base.data.maps["MnWinona"])?;
        let after_rows = serde_json::to_value(winona)?;
        let unaffected = |value: &Value| -> Vec<Value> {
            value[field]
                .as_array()
                .expect("typed command array")
                .iter()
                .filter(|row| {
                    !matches!(
                        row["source_script"].as_str(),
                        Some(
                            "MnWTrailWardenSouth"
                                | "MnWTrailWardenSouthTalk"
                                | "MnWTrailWardenSouthCleared"
                        )
                    )
                })
                .cloned()
                .collect()
        };
        ensure!(
            unaffected(&before_rows) == unaffected(&after_rows),
            "unrelated command rows changed: {field}"
        );
    }
    // Exempt only the reviewed text programs and the four south-guard programs.
    let new_labels: BTreeSet<_> = winona
        .scripts
        .keys()
        .filter(|label| !base.data.maps["MnWinona"].scripts.contains_key(*label))
        .map(String::as_str)
        .collect();
    ensure!(
        new_labels
            == BTreeSet::from([
                SOUTH_TEXT,
                SOUTH_CLEAR_TEXT,
                SOUTH_TALK,
                "MnWTrailWardenSouthCleared"
            ]),
        "unexpected new scripts"
    );
    for label in &new_labels {
        normalized.data.map_scripts.remove(*label);
    }
    for (label, body) in &base.data.maps["MnWinona"].scripts {
        if !revision
            .dialogue_labels
            .get("MnWinona")
            .is_some_and(|labels| labels.contains(label))
            && label != "MnWTrailWardenSouth"
        {
            ensure!(
                winona.scripts.get(label) == Some(body),
                "unrelated Winona script changed: {label}"
            );
        }
        if let Some(mirror) = base.data.map_scripts.get(label) {
            normalized
                .data
                .map_scripts
                .insert(label.clone(), mirror.clone());
        }
    }
    let events = base.data.maps["MnWinona"]
        .attributes
        .map_events_label
        .as_ref()
        .context("event label")?;
    normalized
        .data
        .map_scripts
        .insert(events.clone(), base.data.map_scripts[events].clone());
    normalized
        .data
        .npcs
        .insert("MnWinona".into(), base.data.npcs["MnWinona"].clone());
    normalized
        .data
        .maps
        .insert("MnWinona".into(), base.data.maps["MnWinona"].clone());
    ensure!(
        normalized.data == base.data,
        "unrelated game content changed"
    );
    let mut expected_files = base.runtime_files.clone();
    expected_files.remove(SAVE_POLICY);
    ensure!(
        revised.runtime_files == expected_files && revised.audio_manifest == base.audio_manifest,
        "art or audio changed"
    );
    ensure!(
        revise_northstar_dialogue_clarity(&revised, source).is_err(),
        "repeat publication accepted"
    );
    let mut invalid = json.clone();
    invalid["dialogue_overhaul"]["texts"]["MnWinona"][SOUTH_TEXT] =
        serde_json::json!([["A line far wider than the native box"]]);
    ensure!(
        revise_northstar_dialogue_clarity(&base, &invalid.to_string()).is_err(),
        "invalid native page accepted"
    );
    Ok(())
}
