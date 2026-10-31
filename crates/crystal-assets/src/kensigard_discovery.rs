//! Native field-margin observations and the optional moving-stone reveal.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::grand_marais_modpack::authoring::command as c;

const ACTOR: &str = "MN_MARGIN_KENSIGARD";
const PREFIX: &str = "EVENT_MN_KENSIGARD_";

/// Native programs awaiting guarded field-margin geometry publication.
#[derive(Debug)]
pub struct KensigardDiscoveryPrograms {
    /// Native command arrays, including the authored dialogue pages.
    pub scripts: BTreeMap<String, Value>,
    /// Event identities the publisher must allocate before installation.
    pub event_flags: Vec<String>,
    /// Object and physical-new-visit callbacks for the field-margin map.
    pub callbacks: [String; 2],
}

/// Lowers the moving-stone field investigation and voluntary encounter.
/// The first grooves observation must be followed by physical map reentry
/// before the changed-grooves page can be recorded. All observations and the
/// final report commit only after acknowledgment. The optional reveal then
/// requires its own acknowledged scene on the creature's map.
///
/// The publisher must bind the caretaker, soil, buried edge, grooves, boundary
/// marker and reveal coordinate event. The grooves, reveal and creature must
/// share the map receiving the returned callbacks. All ordinary field travel
/// must remain open. A wrong order merely gives directions; it spends nothing.
/// Only capture sets caught. Continue retains a withdrawn visit, while actual
/// new-map entry restores an uncaught revealed creature. No pack is modified.
///
/// # Errors
/// Rejects missing/duplicate contracts, foreign species, missing text, malformed
/// native pages and text exceeding the native two-line, 18-column limits.
pub fn build_kensigard_discovery_programs(source: &str) -> Result<KensigardDiscoveryPrograms> {
    let source: Value = serde_json::from_str(source)?;
    ensure!(
        source["schema_version"] == 1,
        "rare discovery source version"
    );
    let quests = source["quests"].as_array().context("discovery quests")?;
    let found = quests
        .iter()
        .filter(|quest| quest["id"] == "field-margin-markers")
        .collect::<Vec<_>>();
    ensure!(found.len() == 1, "one field-margin contract required");
    let quest = found[0];
    ensure!(
        quest["species"] == serde_json::json!(["KENSIGARD"]),
        "original Minnesota Kensigard required"
    );
    let mut scripts = crate::discovery_dialogue::lower(quest, "MnKensigardText_")?;
    for key in [
        "caretaker_intro",
        "soil",
        "buried_edge",
        "grooves_first",
        "grooves_changed",
        "boundary_marker",
        "caretaker_partial",
        "caretaker_comparison",
        "claim_corrected",
        "margin_early",
        "reveal",
        "ask",
        "decline",
        "retreat",
        "caught",
        "caretaker_after",
    ] {
        ensure!(
            scripts.contains_key(&text(key)),
            "missing field-margin dialogue {key}"
        );
    }
    let mut put = |label: &str, commands: Vec<Value>| {
        scripts.insert(label.into(), Value::Array(commands));
    };
    let flag = |name: &str| format!("{PREFIX}{name}");
    let mut caretaker = vec![
        c("faceplayer", &[]),
        c("checkevent", &[&flag("NOTICE_CORRECTED")]),
        c("iftrue", &["MnKensigardCaretakerAfter"]),
        c("checkevent", &[&flag("BOUNDARY_LOCATED")]),
        c("iftrue", &["MnKensigardReport"]),
    ];
    for name in [
        "CARETAKER_READ",
        "SOIL_READ",
        "BURIED_EDGE_READ",
        "CHANGING_GROOVES_READ",
    ] {
        caretaker.extend([
            c("checkevent", &[&flag(name)]),
            c("iffalse", &["MnKensigardCaretakerWaiting"]),
        ]);
    }
    caretaker.extend(say("caretaker_comparison"));
    caretaker.extend([
        c("setevent", &[&flag("DIRECTIONS_ACKNOWLEDGED")]),
        c("end", &[]),
    ]);
    put("MnKensigardCaretaker", caretaker);
    let mut waiting = vec![
        c("checkevent", &[&flag("CARETAKER_READ")]),
        c("iftrue", &["MnKensigardCaretakerPartial"]),
    ];
    waiting.extend(say("caretaker_intro"));
    waiting.extend([c("setevent", &[&flag("CARETAKER_READ")]), c("end", &[])]);
    put("MnKensigardCaretakerWaiting", waiting);
    for (label, key) in [
        ("MnKensigardCaretakerPartial", "caretaker_partial"),
        ("MnKensigardCaretakerAfter", "caretaker_after"),
        ("MnKensigardEarly", "margin_early"),
    ] {
        put(label, vec![c("jumptext", &[&text(key)])]);
    }
    for (label, key, event) in [
        ("MnKensigardSoil", "soil", "SOIL_READ"),
        ("MnKensigardEdge", "buried_edge", "BURIED_EDGE_READ"),
    ] {
        let mut commands = say(key);
        commands.extend([c("setevent", &[&flag(event)]), c("end", &[])]);
        put(label, commands);
    }
    put(
        "MnKensigardGrooves",
        vec![
            c("checkevent", &[&flag("GROOVES_REVISIT_READY")]),
            c("iftrue", &["MnKensigardGroovesChanged"]),
            c("sjump", &["MnKensigardGroovesFirst"]),
        ],
    );
    let mut first = say("grooves_first");
    first.extend([c("setevent", &[&flag("GROOVES_FIRST_READ")]), c("end", &[])]);
    put("MnKensigardGroovesFirst", first);
    let mut changed = say("grooves_changed");
    changed.extend([
        c("setevent", &[&flag("CHANGING_GROOVES_READ")]),
        c("end", &[]),
    ]);
    put("MnKensigardGroovesChanged", changed);
    let mut marker = vec![
        c("checkevent", &[&flag("DIRECTIONS_ACKNOWLEDGED")]),
        c("iffalse", &["MnKensigardEarly"]),
    ];
    marker.extend(say("boundary_marker"));
    marker.extend([c("setevent", &[&flag("BOUNDARY_LOCATED")]), c("end", &[])]);
    put("MnKensigardBoundary", marker);
    let mut report = say("claim_corrected");
    report.extend([
        c("setevent", &[&flag("BOUNDARY_REPORTED")]),
        c("setevent", &[&flag("NOTICE_CORRECTED")]),
        c("end", &[]),
    ]);
    put("MnKensigardReport", report);
    let mut reveal = vec![
        c("checkevent", &[&flag("NOTICE_CORRECTED")]),
        c("iffalse", &["MnKensigardEnd"]),
        c("checkevent", &[&flag("REVEALED")]),
        c("iftrue", &["MnKensigardEnd"]),
    ];
    reveal.extend(say("reveal"));
    reveal.extend([
        c("setevent", &[&flag("REVEALED")]),
        c("appear", &[ACTOR]),
        c("end", &[]),
    ]);
    put("MnKensigardReveal", reveal);
    put(
        ACTOR,
        vec![
            c("checkevent", &[&flag("REVEALED")]),
            c("iffalse", &["MnKensigardEnd"]),
            c("checkevent", &[&flag("CAUGHT")]),
            c("iftrue", &["MnKensigardEnd"]),
            c("opentext", &[]),
            c("writetext", &[&text("ask")]),
            c("yesorno", &[]),
            c("iffalse", &["MnKensigardDeclined"]),
            c("closetext", &[]),
            c("cry", &["KENSIGARD"]),
            c(
                "loadvar",
                &[
                    "VAR_BATTLETYPE",
                    crystal_core::battle::capture::OPTIONAL_LEGENDARY_BATTLE_TYPE,
                ],
            ),
            c("loadwildmon", &["KENSIGARD", "50"]),
            c("startbattle", &[]),
            c("reloadmapafterbattle", &[]),
            c("special", &["CheckCaughtCelebi"]),
            c("iftrue", &["MnKensigardCaught"]),
            c("disappear", &[ACTOR]),
            c("jumptext", &[&text("retreat")]),
        ],
    );
    put(
        "MnKensigardDeclined",
        vec![
            c("writetext", &[&text("decline")]),
            c("promptbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    put(
        "MnKensigardCaught",
        vec![
            c("setevent", &[&flag("CAUGHT")]),
            c("disappear", &[ACTOR]),
            c("jumptext", &[&text("caught")]),
        ],
    );
    put("MnKensigardEnd", vec![c("end", &[])]);
    put(
        "MnKensigardNewVisit",
        vec![
            c("checkevent", &[&flag("GROOVES_FIRST_READ")]),
            c("iffalse", &["MnKensigardRestore"]),
            c("setevent", &[&flag("GROOVES_REVISIT_READY")]),
            c("sjump", &["MnKensigardRestore"]),
        ],
    );
    for (label, restore) in [("MnKensigardObjects", false), ("MnKensigardRestore", true)] {
        let mut callback = vec![
            c("checkevent", &[&flag("CAUGHT")]),
            c("iftrue", &["MnKensigardHide"]),
            c("checkevent", &[&flag("REVEALED")]),
            c("iffalse", &["MnKensigardHide"]),
        ];
        if restore {
            callback.push(c("appear", &[ACTOR]));
        }
        callback.push(c("endcallback", &[]));
        put(label, callback);
    }
    put(
        "MnKensigardHide",
        vec![c("disappear", &[ACTOR]), c("endcallback", &[])],
    );
    Ok(KensigardDiscoveryPrograms {
        scripts,
        event_flags: [
            "CARETAKER_READ",
            "SOIL_READ",
            "BURIED_EDGE_READ",
            "GROOVES_FIRST_READ",
            "GROOVES_REVISIT_READY",
            "CHANGING_GROOVES_READ",
            "DIRECTIONS_ACKNOWLEDGED",
            "BOUNDARY_LOCATED",
            "BOUNDARY_REPORTED",
            "NOTICE_CORRECTED",
            "REVEALED",
            "CAUGHT",
        ]
        .into_iter()
        .map(flag)
        .collect(),
        callbacks: ["MnKensigardObjects".into(), "MnKensigardNewVisit".into()],
    })
}
fn text(key: &str) -> String {
    format!("MnKensigardText_{key}")
}
fn say(key: &str) -> Vec<Value> {
    vec![
        c("opentext", &[]),
        c("writetext", &[&text(key)]),
        c("promptbutton", &[]),
        c("closetext", &[]),
    ]
}
