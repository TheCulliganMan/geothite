//! Preserve a frozen campaign while independently accounting for revised prose.
#[path = "northstar_supporting_wording_tests.rs"]
mod supporting;

use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::{CompiledGamePack, MapEvents, MapModule};

pub(super) fn retain_reviewed_facade_events(
    map: &str,
    expected: &mut MapModule,
    parent: &MapModule,
) {
    let originals = expected.events.bg_events.clone();
    retain_reviewed_sign_positions(map, &mut expected.events, &parent.events);
    for original in originals {
        let current = expected
            .events
            .bg_events
            .iter()
            .find(|sign| sign.script == original.script)
            .expect("retained background event");
        if (original.x, original.y) == (current.x, current.y) {
            continue;
        }
        let command = expected
            .map_event_section_commands
            .iter_mut()
            .find(|row| row.command == "bg_event" && row.args.get(3) == Some(&original.script))
            .expect("native sign event command");
        assert_eq!(
            command.args,
            [
                original.x.to_string(),
                original.y.to_string(),
                original.event_type,
                original.script,
            ]
        );
        command.args[0] = current.x.to_string();
        command.args[1] = current.y.to_string();
    }
}

// The accepted facade repair moved three signs. No other event field, label,
// or unreviewed position is normalized when comparing older frozen geography.
pub(super) fn retain_reviewed_sign_positions(
    map: &str,
    expected: &mut MapEvents,
    parent: &MapEvents,
) {
    if map != "MnGrandRapids" {
        return;
    }
    for (script, old, current) in [
        ("MN_GR_WEST_SIGN", (7, 23), (14, 22)),
        ("MN_GR_EAST_SIGN", (39, 23), (38, 22)),
        ("MN_GR_DEPOT_SIGN", (23, 13), (22, 12)),
    ] {
        if parent
            .bg_events
            .iter()
            .any(|sign| sign.script == script && (sign.x, sign.y) == current)
        {
            let sign = expected
                .bg_events
                .iter_mut()
                .find(|sign| sign.script == script)
                .expect("frozen Grand Rapids sign identity");
            if (sign.x, sign.y) != current {
                assert_eq!((sign.x, sign.y), old);
                (sign.x, sign.y) = current;
            }
        }
    }
}

pub(super) fn reauthor_frozen(mut pack: CompiledGamePack, source: &str) -> CompiledGamePack {
    let Some(path) = std::env::var_os("CRYSTAL_STORY_FROZEN_SOURCE") else {
        return pack;
    };
    let old: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(PathBuf::from(path)).expect("explicit frozen story source"),
    )
    .expect("frozen canonical JSON");
    let new: serde_json::Value = serde_json::from_str(source).expect("current canonical JSON");
    let old: super::Revision =
        serde_json::from_value(old["dialogue_overhaul"].clone()).expect("frozen dialogue schema");
    let new: super::Revision =
        serde_json::from_value(new["dialogue_overhaul"].clone()).expect("current dialogue schema");
    let original_maps = old.texts.keys().cloned().collect::<BTreeSet<_>>();
    let mut changes = 0;
    for (name, labels) in old.texts {
        let mut map = pack.data.maps[&name].clone();
        let additions = new.texts[&name]
            .keys()
            .filter(|label| !labels.contains_key(*label))
            .map(String::as_str)
            .collect::<Vec<_>>();
        let expected: &[&str] = match name.as_str() {
            "MnBoundaryWatersSurveyShelter" => {
                &["MnOverhaulChiefExposedText", "MnOverhaulChiefStoppedText"]
            }
            "MnBemidji" => &[
                "MnOverhaulOfficerDoneText",
                "MnOverhaulOfficerHarborText",
                "MnOverhaulOfficerIntakeText",
                "MnOverhaulOfficerMeterText",
                "MnOverhaulOfficerNorthText",
            ],
            "MnBrainerd" => &["MnOverhaulRangerComparedText"],
            "MnLakeItasca" => &["MnOverhaulItascaComparedText"],
            "MnGrandRapidsSurveyDepot" => &[
                "MN_GR_REPAIRERText",
                "MnOverhaulDepotDoneText",
                "MnOverhaulDepotIntakeText",
                "MnOverhaulDepotNorthText",
            ],
            "MnDuluth" => &[
                "MnDuluth_MN_DULUTH_NORTH_GUARDText",
                "MnDuluth_NORTH_GATE_SIGNText",
            ],
            "MnRochesterPokecenter1F" => &["MnRCMissingText"],
            "MnWinonaFieldStation" => &["MnNSEReedLostText", "MnNSEReedWonText"],
            _ => &[],
        };
        let reviewed = expected
            .iter()
            .copied()
            .chain(supporting::labels(&name).iter().copied())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            additions.iter().copied().collect::<BTreeSet<_>>(),
            reviewed,
            "only reviewed phase-specific and supporting replies"
        );
        for (label, pages) in labels {
            let before = crate::authored_dialogue::compile(&pages).expect("frozen native pages");
            assert_eq!(
                map.scripts.get(&label),
                Some(&before),
                "authentic frozen source {name}/{label}"
            );
            let after = crate::authored_dialogue::compile(&new.texts[&name][&label])
                .expect("current reviewed pages");
            changes += usize::from(before != after);
            map.scripts.insert(label, after);
        }
        for label in additions {
            map.scripts.insert(
                label.to_owned(),
                crate::authored_dialogue::compile(&new.texts[&name][label])
                    .expect("reviewed completed-chief reply"),
            );
        }
        acknowledge_authored_pages(&mut map, &new.texts[&name]);
        super::publish(&mut pack, map)
            .expect("only recompile the reviewed text on the frozen actors and programs");
    }
    assert_eq!(changes, 138, "reviewed full-story storyboard changes");
    let supporting_maps = new
        .texts
        .keys()
        .filter(|name| !original_maps.contains(*name))
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        supporting_maps,
        BTreeSet::from([
            "MnBrainerdRangerStation",
            "MnDuluthPokecenter1F",
            "MnGrandMarais",
            "MnGrandMaraisFieldStation",
            "MnGrandMaraisLighthouse",
            "MnInternationalFallsPokecenter1F",
            "MnR01Farmhouse",
            "MnR09C00R00",
            "MnR11C00R01",
            "MnR11SplitRockLighthouse",
            "MnRainyRiverIcefields",
            "MnWinonaHome2F",
            "MnIceMaintenance",
            "MnIceOperations",
            "MnIceRecords",
            "MnIceTransfer",
            "MnIronDrillApproach",
            "MnIronHaulage",
            "MnIronSump",
            "MnIronVentilation",
        ]),
        "only reviewed surrounding directions, staff rooms and mine passages"
    );
    for name in supporting_maps {
        let mut map = pack.data.maps[name].clone();
        for (label, pages) in &new.texts[name] {
            let original = map.scripts.get(label).expect("existing supporting text");
            assert!(
                original
                    .as_array()
                    .expect("native text program")
                    .iter()
                    .all(|row| {
                        row["command"].as_str().is_some_and(|command| {
                            matches!(command, "text" | "line" | "para" | "cont" | "done")
                        })
                    }),
                "supporting prose must not replace game commands: {name}/{label}"
            );
            map.scripts.insert(
                label.clone(),
                crate::authored_dialogue::compile(pages).expect("reviewed supporting pages"),
            );
        }
        acknowledge_authored_pages(&mut map, &new.texts[name]);
        super::publish(&mut pack, map).expect("reviewed supporting text and native mirrors");
    }
    crate::northstar_dispatch_revision::install(&mut pack)
        .expect("reviewed physical desk-copy handoff");
    let mut officer = pack.data.maps["MnBemidji"].clone();
    crate::northstar_report_revision::install(&mut officer).expect("reviewed comparison staging");
    super::publish(&mut pack, officer).expect("native report staging mirrors");
    crate::northstar_story_pacing::install(&mut pack)
        .expect("reviewed evidence acceptance and completed-chief routing");
    crate::northstar_story_staff::install(&mut pack)
        .expect("reviewed offscreen staff departure after the actual rescue");
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )
    .expect("the independently reauthored frozen story has a current identity");
    pack
}

fn acknowledge_authored_pages(
    map: &mut MapModule,
    texts: &std::collections::BTreeMap<String, Vec<crate::authored_dialogue::AuthoredDialoguePage>>,
) {
    for body in map
        .scripts
        .values_mut()
        .filter_map(serde_json::Value::as_array_mut)
    {
        for index in 1..body.len() {
            if body[index]["command"] == "waitbutton"
                && body[index - 1]["command"] == "writetext"
                && body[index - 1]["args"][0]
                    .as_str()
                    .is_some_and(|label| texts.contains_key(label))
            {
                body[index]["command"] = "promptbutton".into();
            }
        }
    }
}
