//! Native optional Pepivern observations and retryable encounter programs.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::grand_marais_modpack::authoring::command as c;

const ACTOR: &str = "MN_PEPIN_PEPIVERN";
const PREFIX: &str = "EVENT_MN_PEPIN_";

/// Native scripts awaiting publication on reviewed current river geometry.
#[derive(Debug)]
pub struct PepinDiscoveryPrograms {
    /// Command arrays, including the deliberately paged native dialogue.
    pub scripts: BTreeMap<String, Value>,
    /// Unique event names that the geometry publisher must allocate.
    pub event_flags: Vec<String>,
    /// Object and physical-new-visit callback labels, respectively.
    pub callbacks: [String; 2],
}

/// Lowers the three-wake investigation without creating maps or modifying a pack.
/// The dock sketch, dry-land eddy and Surf-only reed observation become recorded
/// only after their final text acknowledgment. The operator then identifies an
/// optional inlet. Refusal preserves the encounter, actual capture alone marks
/// caught, and physical reentry restores an uncaught creature after withdrawal.
/// Continue must run only the object callback, preserving a withdrawn visit.
///
/// A publisher must bind the returned callbacks on the creature's map, register
/// the returned flags, and bind the five observation/actor scripts to reviewed
/// current shoreline coordinates. It must parse scripted wild battles after
/// installing these programs. This function establishes no runtime acceptance.
///
/// # Errors
/// Rejects duplicate or missing contracts, foreign species, malformed pages,
/// missing dialogue, non-ASCII text, and lines exceeding 18 columns.
pub fn build_pepin_discovery_programs(source: &str) -> Result<PepinDiscoveryPrograms> {
    let source: Value = serde_json::from_str(source)?;
    ensure!(
        source["schema_version"] == 1,
        "rare discovery source version"
    );
    let matches = source["quests"]
        .as_array()
        .context("rare discovery quests")?
        .iter()
        .filter(|quest| quest["id"] == "pepin-three-wakes")
        .collect::<Vec<_>>();
    ensure!(matches.len() == 1, "one Pepin discovery contract required");
    let quest = matches[0];
    ensure!(
        quest["species"] == serde_json::json!(["PEPIVERN"]),
        "original Minnesota Pepivern required"
    );
    let mut scripts = BTreeMap::new();
    for (key, pages) in quest["dialogue_pages"]
        .as_object()
        .context("Pepin native dialogue")?
    {
        let pages = pages.as_array().context("dialogue pages")?;
        ensure!(!pages.is_empty(), "empty dialogue {key}");
        let mut commands = Vec::new();
        for (page_index, page) in pages.iter().enumerate() {
            let lines = page.as_array().context("dialogue lines")?;
            ensure!(!lines.is_empty() && lines.len() <= 2, "two-line page {key}");
            for (line_index, line) in lines.iter().enumerate() {
                let line = line.as_str().context("native dialogue line")?;
                ensure!(
                    !line.is_empty()
                        && line.is_ascii()
                        && line.len() <= 18
                        && !line.contains(['"', '\n', '\r']),
                    "invalid native dialogue line in {key}"
                );
                let opcode = if line_index > 0 {
                    "line"
                } else if page_index > 0 {
                    "para"
                } else {
                    "text"
                };
                commands.push(c(opcode, &[&format!("\"{line}\"")]));
            }
        }
        commands.push(c("done", &[]));
        scripts.insert(text(key), Value::Array(commands));
    }
    for key in [
        "dock_intro",
        "dock_sketch",
        "eddy",
        "reeds",
        "needs_surf",
        "operator_intro",
        "operator_partial",
        "operator_compare",
        "inlet_early",
        "reveal",
        "ask",
        "decline",
        "retreat",
        "caught",
        "operator_complete",
        "dock_complete",
    ] {
        ensure!(
            scripts.contains_key(&text(key)),
            "missing Pepin dialogue {key}"
        );
    }
    let mut put = |label: &str, commands: Vec<Value>| {
        scripts.insert(label.into(), Value::Array(commands));
    };
    let flag = |name: &str| format!("{PREFIX}{name}");
    let mut dock = vec![
        c("faceplayer", &[]),
        c("checkevent", &[&flag("SAFE_INLET_IDENTIFIED")]),
        c("iftrue", &["MnPepinDockComplete"]),
    ];
    dock.extend(say("dock_intro"));
    dock.extend(say("dock_sketch"));
    dock.extend([c("setevent", &[&flag("DOCK_SKETCH_READ")]), c("end", &[])]);
    put("MnPepinDock", dock);
    let mut eddy = say("eddy");
    // The map binding must provide a dry-floor approach for this viewpoint.
    eddy.extend([c("setevent", &[&flag("EDDY_OBSERVED")]), c("end", &[])]);
    put("MnPepinEddy", eddy);
    put(
        "MnPepinReeds",
        vec![
            c("readvar", &["VAR_MOVEMENT"]),
            c("ifequal", &["4", "MnPepinReedsSurf"]),
            c("ifequal", &["8", "MnPepinReedsSurf"]),
            c("jumptext", &[&text("needs_surf")]),
        ],
    );
    let mut reeds = say("reeds");
    reeds.extend([c("setevent", &[&flag("REEDS_OBSERVED")]), c("end", &[])]);
    put("MnPepinReedsSurf", reeds);
    let mut operator = vec![
        c("faceplayer", &[]),
        c("checkevent", &[&flag("SAFE_INLET_IDENTIFIED")]),
        c("iftrue", &["MnPepinOperatorComplete"]),
    ];
    for name in ["DOCK_SKETCH_READ", "EDDY_OBSERVED", "REEDS_OBSERVED"] {
        operator.extend([
            c("checkevent", &[&flag(name)]),
            c("iffalse", &["MnPepinOperatorWaiting"]),
        ]);
    }
    operator.extend(say("operator_compare"));
    operator.extend([
        c("setevent", &[&flag("SAFE_INLET_IDENTIFIED")]),
        c("end", &[]),
    ]);
    // The actor lives on a different river chunk: its next entry callback
    // handles appearance. Never issue a remote appear against the current map.
    put("MnPepinOperator", operator);
    let mut waiting = say("operator_intro");
    waiting.extend(say("operator_partial"));
    waiting.push(c("end", &[]));
    put("MnPepinOperatorWaiting", waiting);
    for (label, key) in [
        ("MnPepinDockComplete", "dock_complete"),
        ("MnPepinOperatorComplete", "operator_complete"),
        ("MnPepinInletEarly", "inlet_early"),
    ] {
        put(label, vec![c("jumptext", &[&text(key)])]);
    }
    let mut encounter = vec![
        c("checkevent", &[&flag("SAFE_INLET_IDENTIFIED")]),
        c("iffalse", &["MnPepinInletEarly"]),
        c("checkevent", &[&flag("CAUGHT")]),
        c("iftrue", &["MnPepinEncounterEnd"]),
    ];
    encounter.extend(say("reveal"));
    encounter.extend([
        c("opentext", &[]),
        c("writetext", &[&text("ask")]),
        c("yesorno", &[]),
        c("iffalse", &["MnPepinDeclined"]),
        c("closetext", &[]),
        c("cry", &["PEPIVERN"]),
        c(
            "loadvar",
            &[
                "VAR_BATTLETYPE",
                crystal_core::battle::capture::OPTIONAL_LEGENDARY_BATTLE_TYPE,
            ],
        ),
        c("loadwildmon", &["PEPIVERN", "55"]),
        c("startbattle", &[]),
        c("reloadmapafterbattle", &[]),
        c("special", &["CheckCaughtCelebi"]),
        c("iftrue", &["MnPepinCaught"]),
        c("disappear", &[ACTOR]),
        c("jumptext", &[&text("retreat")]),
    ]);
    put(ACTOR, encounter);
    put(
        "MnPepinDeclined",
        vec![
            c("writetext", &[&text("decline")]),
            c("promptbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    put(
        "MnPepinCaught",
        vec![
            c("setevent", &[&flag("CAUGHT")]),
            c("disappear", &[ACTOR]),
            c("jumptext", &[&text("caught")]),
        ],
    );
    put("MnPepinEncounterEnd", vec![c("end", &[])]);
    for (label, restore) in [("MnPepinObjects", false), ("MnPepinNewVisit", true)] {
        let mut callback = vec![
            c("checkevent", &[&flag("CAUGHT")]),
            c("iftrue", &["MnPepinHide"]),
            c("checkevent", &[&flag("SAFE_INLET_IDENTIFIED")]),
            c("iffalse", &["MnPepinHide"]),
        ];
        if restore {
            callback.push(c("appear", &[ACTOR]));
        }
        callback.push(c("endcallback", &[]));
        put(label, callback);
    }
    put(
        "MnPepinHide",
        vec![c("disappear", &[ACTOR]), c("endcallback", &[])],
    );
    Ok(PepinDiscoveryPrograms {
        scripts,
        event_flags: [
            "DOCK_SKETCH_READ",
            "EDDY_OBSERVED",
            "REEDS_OBSERVED",
            "SAFE_INLET_IDENTIFIED",
            "CAUGHT",
        ]
        .into_iter()
        .map(flag)
        .collect(),
        callbacks: ["MnPepinObjects".into(), "MnPepinNewVisit".into()],
    })
}

fn text(key: &str) -> String {
    format!("MnPepinText_{key}")
}

fn say(key: &str) -> Vec<Value> {
    vec![
        c("opentext", &[]),
        c("writetext", &[&text(key)]),
        c("promptbutton", &[]),
        c("closetext", &[]),
    ]
}
