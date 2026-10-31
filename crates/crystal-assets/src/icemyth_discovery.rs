//! Native witness investigation for the optional frozen exhibit encounter.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::grand_marais_modpack::authoring::command as c;

const ACTOR: &str = "MN_EXHIBIT_ICEMYTH";
const FROZEN: &str = "MN_EXHIBIT_FROZEN_CASE";
const PREFIX: &str = "EVENT_MN_ICEMYTH_";

/// Native exhibit programs awaiting reviewed map/actor publication.
#[derive(Debug)]
pub struct IcemythDiscoveryPrograms {
    /// Deliberately paged text and native script command arrays.
    pub scripts: BTreeMap<String, Value>,
    /// Event names that the map publisher must allocate before compilation.
    pub event_flags: Vec<String>,
    /// Object and physical-new-visit callbacks for the shaded creature map.
    pub callbacks: [String; 2],
}

/// Lowers the conflicting witnesses, physical clues and optional encounter.
/// Acknowledgment records each observation. All five observations are needed
/// before the attendant corrects the label; inspecting the empty case after
/// that correction reveals Icemyth. The case read and creature must share the
/// shaded service map because the reveal issues a local actor appearance.
///
/// A publisher must install the returned callbacks, bind the nonbattle frozen
/// case and `MnIcemythHallObjects` callback in the hall, bind the attendant,
/// delivery worker, visitor, latch, tray and case reads, allocate event names,
/// and parse the scripted wild battle. The route and service return must stay
/// accessible independently of this optional investigation and its capture.
/// Continue preserves a withdrawn visit; physical reentry restores uncaught
/// Icemyth. Reading the case again cannot bypass that reentry requirement.
/// This function does not install a catchable species or establish acceptance.
///
/// # Errors
/// Rejects missing/duplicate contracts, a foreign species, missing dialogue,
/// malformed pages, non-ASCII text and lines exceeding 18 columns.
pub fn build_icemyth_discovery_programs(source: &str) -> Result<IcemythDiscoveryPrograms> {
    let source: Value = serde_json::from_str(source)?;
    ensure!(
        source["schema_version"] == 1,
        "rare discovery source version"
    );
    let quests = source["quests"].as_array().context("discovery quests")?;
    let found = quests
        .iter()
        .filter(|quest| quest["id"] == "traveling-frozen-exhibit")
        .collect::<Vec<_>>();
    ensure!(found.len() == 1, "one frozen exhibit contract required");
    let quest = found[0];
    ensure!(
        quest["species"] == serde_json::json!(["ICEMYTH"]),
        "original Minnesota Icemyth required"
    );
    let mut scripts = crate::discovery_dialogue::lower(quest, "MnIcemythText_")?;
    for key in [
        "attendant_intro",
        "delivery_account",
        "visitor_account",
        "latch",
        "tray",
        "attendant_partial",
        "attendant_compare",
        "service_early",
        "empty_case",
        "reveal",
        "ask",
        "decline",
        "retreat",
        "caught",
        "attendant_complete",
        "visitor_complete",
        "case_occupied",
    ] {
        ensure!(
            scripts.contains_key(&text(key)),
            "missing exhibit dialogue {key}"
        );
    }
    let mut put = |label: &str, commands: Vec<Value>| {
        scripts.insert(label.into(), Value::Array(commands));
    };
    let flag = |name: &str| format!("{PREFIX}{name}");
    let mut attendant = vec![
        c("faceplayer", &[]),
        c("checkevent", &[&flag("LABEL_CORRECTED")]),
        c("iftrue", &["MnIcemythAttendantAfter"]),
    ];
    for name in [
        "ATTENDANT_ACCOUNT",
        "DELIVERY_ACCOUNT",
        "VISITOR_ACCOUNT",
        "LATCH_READ",
        "TRAY_READ",
    ] {
        attendant.extend([
            c("checkevent", &[&flag(name)]),
            c("iffalse", &["MnIcemythAttendantWaiting"]),
        ]);
    }
    attendant.extend(say("attendant_compare"));
    attendant.extend([
        c("setevent", &[&flag("LABEL_CORRECTED")]),
        c("disappear", &[FROZEN]),
        c("end", &[]),
    ]);
    put("MnIcemythAttendant", attendant);
    let mut waiting = vec![
        c("checkevent", &[&flag("ATTENDANT_ACCOUNT")]),
        c("iftrue", &["MnIcemythAttendantPartial"]),
    ];
    waiting.extend(say("attendant_intro"));
    waiting.extend([c("setevent", &[&flag("ATTENDANT_ACCOUNT")]), c("end", &[])]);
    put("MnIcemythAttendantWaiting", waiting);
    for (label, key) in [
        ("MnIcemythAttendantPartial", "attendant_partial"),
        ("MnIcemythAttendantAfter", "attendant_complete"),
        ("MnIcemythVisitorAfter", "visitor_complete"),
        ("MnIcemythServiceEarly", "service_early"),
        ("MnIcemythCaseDone", "empty_case"),
    ] {
        put(label, vec![c("jumptext", &[&text(key)])]);
    }
    for (label, key, event) in [
        ("MnIcemythDelivery", "delivery_account", "DELIVERY_ACCOUNT"),
        ("MnIcemythVisitor", "visitor_account", "VISITOR_ACCOUNT"),
        ("MnIcemythLatch", "latch", "LATCH_READ"),
        ("MnIcemythTray", "tray", "TRAY_READ"),
    ] {
        let mut observation = Vec::new();
        if key == "visitor_account" {
            observation.extend([
                c("faceplayer", &[]),
                c("checkevent", &[&flag("LABEL_CORRECTED")]),
                c("iftrue", &["MnIcemythVisitorAfter"]),
            ]);
        } else if key == "delivery_account" {
            observation.push(c("faceplayer", &[]));
        }
        observation.extend(say(key));
        observation.extend([c("setevent", &[&flag(event)]), c("end", &[])]);
        put(label, observation);
    }
    put(FROZEN, vec![c("jumptext", &[&text("case_occupied")])]);
    put(
        "MnIcemythHallObjects",
        vec![
            c("checkevent", &[&flag("LABEL_CORRECTED")]),
            c("iffalse", &["MnIcemythHallEnd"]),
            c("disappear", &[FROZEN]),
            c("endcallback", &[]),
        ],
    );
    put("MnIcemythHallEnd", vec![c("endcallback", &[])]);
    let mut case = vec![
        c("checkevent", &[&flag("LABEL_CORRECTED")]),
        c("iffalse", &["MnIcemythServiceEarly"]),
        c("checkevent", &[&flag("MOVED_ICE_FOUND")]),
        c("iftrue", &["MnIcemythCaseDone"]),
    ];
    case.extend(say("empty_case"));
    case.extend([
        c("setevent", &[&flag("MOVED_ICE_FOUND")]),
        c("appear", &[ACTOR]),
        c("end", &[]),
    ]);
    put("MnIcemythCase", case);
    let mut encounter = vec![
        c("checkevent", &[&flag("MOVED_ICE_FOUND")]),
        c("iffalse", &["MnIcemythEncounterEnd"]),
        c("checkevent", &[&flag("CAUGHT")]),
        c("iftrue", &["MnIcemythEncounterEnd"]),
    ];
    encounter.extend(say("reveal"));
    encounter.extend([
        c("opentext", &[]),
        c("writetext", &[&text("ask")]),
        c("yesorno", &[]),
        c("iffalse", &["MnIcemythDeclined"]),
        c("closetext", &[]),
        c("cry", &["ICEMYTH"]),
        c(
            "loadvar",
            &[
                "VAR_BATTLETYPE",
                crystal_core::battle::capture::OPTIONAL_LEGENDARY_BATTLE_TYPE,
            ],
        ),
        c("loadwildmon", &["ICEMYTH", "50"]),
        c("startbattle", &[]),
        c("reloadmapafterbattle", &[]),
        c("special", &["CheckCaughtCelebi"]),
        c("iftrue", &["MnIcemythCaught"]),
        c("disappear", &[ACTOR]),
        c("jumptext", &[&text("retreat")]),
    ]);
    put(ACTOR, encounter);
    put(
        "MnIcemythDeclined",
        vec![
            c("writetext", &[&text("decline")]),
            c("promptbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    put(
        "MnIcemythCaught",
        vec![
            c("setevent", &[&flag("CAUGHT")]),
            c("disappear", &[ACTOR]),
            c("jumptext", &[&text("caught")]),
        ],
    );
    put("MnIcemythEncounterEnd", vec![c("end", &[])]);
    for (label, restore) in [("MnIcemythObjects", false), ("MnIcemythNewVisit", true)] {
        let mut callback = vec![
            c("checkevent", &[&flag("CAUGHT")]),
            c("iftrue", &["MnIcemythHide"]),
            c("checkevent", &[&flag("MOVED_ICE_FOUND")]),
            c("iffalse", &["MnIcemythHide"]),
        ];
        if restore {
            callback.push(c("appear", &[ACTOR]));
        }
        callback.push(c("endcallback", &[]));
        put(label, callback);
    }
    put(
        "MnIcemythHide",
        vec![c("disappear", &[ACTOR]), c("endcallback", &[])],
    );
    Ok(IcemythDiscoveryPrograms {
        scripts,
        event_flags: [
            "ATTENDANT_ACCOUNT",
            "DELIVERY_ACCOUNT",
            "VISITOR_ACCOUNT",
            "LATCH_READ",
            "TRAY_READ",
            "LABEL_CORRECTED",
            "MOVED_ICE_FOUND",
            "CAUGHT",
        ]
        .into_iter()
        .map(flag)
        .collect(),
        callbacks: ["MnIcemythObjects".into(), "MnIcemythNewVisit".into()],
    })
}

fn text(key: &str) -> String {
    format!("MnIcemythText_{key}")
}
fn say(key: &str) -> Vec<Value> {
    vec![
        c("opentext", &[]),
        c("writetext", &[&text(key)]),
        c("promptbutton", &[]),
        c("closetext", &[]),
    ]
}
