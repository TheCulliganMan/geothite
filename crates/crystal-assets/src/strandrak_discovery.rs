//! Native terrace survey and independently acknowledged basin controls.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::grand_marais_modpack::authoring::command as c;

const PREFIX: &str = "EVENT_MN_BASIN_";
const ACTOR: &str = "MN_BASIN_STRANDRAK";
const OPEN_PATH: &str = "MnBasinOpenInnerOutlet";

/// Native basin programs awaiting guarded current-world geometry publication.
#[derive(Debug)]
pub struct StrandrakDiscoveryPrograms {
    /// Deliberately paged dialogue and native command arrays.
    pub scripts: BTreeMap<String, Value>,
    /// Event identities the publisher must allocate before script compilation.
    pub event_flags: Vec<String>,
    /// Object and physical-new-visit callbacks for the inner basin map.
    pub callbacks: [String; 2],
    /// Required local subroutine for the inspected optional inner outlet.
    /// Bind guarded native block changes, `refreshmap` and returning `end`.
    /// A TILES callback must restore the same state from the acknowledged
    /// outlet flag. The outside route and ordinary return must stay exact.
    pub terrain_hook: String,
}

/// Lowers the terrace observations, three controls and optional encounter.
/// Three acknowledged shore marks precede the surveyor's comparison. Each dial
/// cycles low/middle/high and records its setting after the displayed page is
/// acknowledged. All three dials must be used, including the initially low
/// lower dial. The lever checks upper/high, center/middle and lower/low without
/// resetting clues or settings on error. Solving precedes the encounter choice.
///
/// A publisher must bind the surveyor, three shore reads, three dials, lever,
/// creature and returned callbacks. The lever, creature and terrain hook must
/// share a map. Only the fictional inner basin changes. Capture alone marks
/// caught; refusal retains the encounter, Continue retains a withdrawn visit,
/// and physical reentry restores an uncaught creature. No pack is modified.
///
/// # Errors
/// Rejects missing/duplicate contracts, foreign species, missing dialogue,
/// malformed native pages and text exceeding the native text-box limits.
pub fn build_strandrak_discovery_programs(source: &str) -> Result<StrandrakDiscoveryPrograms> {
    let source: Value = serde_json::from_str(source)?;
    ensure!(
        source["schema_version"] == 1,
        "rare discovery source version"
    );
    let quests = source["quests"].as_array().context("discovery quests")?;
    let found = quests
        .iter()
        .filter(|quest| quest["id"] == "three-former-shores")
        .collect::<Vec<_>>();
    ensure!(found.len() == 1, "one former-shores contract required");
    let quest = found[0];
    ensure!(
        quest["species"] == serde_json::json!(["STRANDRAK"]),
        "original Minnesota Strandrak required"
    );
    let mut scripts = crate::discovery_dialogue::lower(quest, "MnBasinText_")?;
    for key in [
        "surveyor_intro",
        "high_shore",
        "middle_shore",
        "low_shore",
        "surveyor_partial",
        "surveyor_compare",
        "early",
        "wrong_release",
        "open_inner",
        "reveal",
        "ask",
        "decline",
        "retreat",
        "caught",
        "surveyor_after",
    ] {
        ensure!(
            scripts.contains_key(&text(key)),
            "missing basin dialogue {key}"
        );
    }
    for dial in ["upper", "center", "lower"] {
        for level in ["low", "middle", "high"] {
            ensure!(
                scripts.contains_key(&text(&format!("{dial}_{level}"))),
                "missing basin dial page"
            );
        }
    }
    let mut put = |label: &str, commands: Vec<Value>| {
        scripts.insert(label.into(), Value::Array(commands));
    };
    let flag = |name: &str| format!("{PREFIX}{name}");
    let mut surveyor = vec![
        c("faceplayer", &[]),
        c("checkevent", &[&flag("INNER_OUTLET_ACKNOWLEDGED")]),
        c("iftrue", &["MnBasinSurveyorAfter"]),
    ];
    for name in ["HIGH_SHORE_READ", "MIDDLE_SHORE_READ", "LOW_SHORE_READ"] {
        surveyor.extend([
            c("checkevent", &[&flag(name)]),
            c("iffalse", &["MnBasinSurveyorWaiting"]),
        ]);
    }
    surveyor.extend(say("surveyor_compare"));
    surveyor.extend([c("setevent", &[&flag("SHORES_COMPARED")]), c("end", &[])]);
    put("MnBasinSurveyor", surveyor);
    let mut waiting = vec![
        c("checkevent", &[&flag("SURVEYOR_READ")]),
        c("iftrue", &["MnBasinSurveyorPartial"]),
    ];
    waiting.extend(say("surveyor_intro"));
    waiting.extend([c("setevent", &[&flag("SURVEYOR_READ")]), c("end", &[])]);
    put("MnBasinSurveyorWaiting", waiting);
    for (label, key) in [
        ("MnBasinSurveyorPartial", "surveyor_partial"),
        ("MnBasinSurveyorAfter", "surveyor_after"),
        ("MnBasinEarly", "early"),
        ("MnBasinWrong", "wrong_release"),
    ] {
        put(label, vec![c("jumptext", &[&text(key)])]);
    }
    for (label, key, event) in [
        ("MnBasinHighShore", "high_shore", "HIGH_SHORE_READ"),
        ("MnBasinMiddleShore", "middle_shore", "MIDDLE_SHORE_READ"),
        ("MnBasinLowShore", "low_shore", "LOW_SHORE_READ"),
    ] {
        let mut commands = say(key);
        commands.extend([c("setevent", &[&flag(event)]), c("end", &[])]);
        put(label, commands);
    }
    let gate = || {
        vec![
            c("checkevent", &[&flag("INNER_OUTLET_ACKNOWLEDGED")]),
            c("iftrue", &["MnBasinEnd"]),
            c("checkevent", &[&flag("SHORES_COMPARED")]),
            c("iffalse", &["MnBasinEarly"]),
        ]
    };
    let mut event_names = vec![
        "HIGH_SHORE_READ".to_string(),
        "MIDDLE_SHORE_READ".into(),
        "LOW_SHORE_READ".into(),
        "SHORES_COMPARED".into(),
        "INNER_OUTLET_ACKNOWLEDGED".into(),
        "SURVEYOR_READ".into(),
        "CAUGHT".into(),
    ];
    for (dial, key) in [("Upper", "upper"), ("Center", "center"), ("Lower", "lower")] {
        let middle = format!("{}_MIDDLE", dial.to_ascii_uppercase());
        let high = format!("{}_HIGH", dial.to_ascii_uppercase());
        let touched = format!("{}_TOUCHED", dial.to_ascii_uppercase());
        event_names.extend([middle.clone(), high.clone(), touched.clone()]);
        let mut rotate = gate();
        rotate.extend([
            c("checkevent", &[&flag(&middle)]),
            c("iftrue", &[&format!("MnBasin{dial}High")]),
            c("checkevent", &[&flag(&high)]),
            c("iftrue", &[&format!("MnBasin{dial}Low")]),
            c("sjump", &[&format!("MnBasin{dial}Middle")]),
        ]);
        put(&format!("MnBasin{dial}Dial"), rotate);
        for (level, word) in [("Low", "low"), ("Middle", "middle"), ("High", "high")] {
            let mut setting = say(&format!("{key}_{word}"));
            setting.extend([
                c("clearevent", &[&flag(&middle)]),
                c("clearevent", &[&flag(&high)]),
            ]);
            if level == "Middle" {
                setting.push(c("setevent", &[&flag(&middle)]));
            }
            if level == "High" {
                setting.push(c("setevent", &[&flag(&high)]));
            }
            setting.extend([c("setevent", &[&flag(&touched)]), c("end", &[])]);
            put(&format!("MnBasin{dial}{level}"), setting);
        }
    }
    let mut lever = gate();
    for name in [
        "UPPER_TOUCHED",
        "CENTER_TOUCHED",
        "LOWER_TOUCHED",
        "UPPER_HIGH",
        "CENTER_MIDDLE",
    ] {
        lever.extend([
            c("checkevent", &[&flag(name)]),
            c("iffalse", &["MnBasinWrong"]),
        ]);
    }
    for name in ["UPPER_MIDDLE", "CENTER_HIGH", "LOWER_MIDDLE", "LOWER_HIGH"] {
        lever.extend([
            c("checkevent", &[&flag(name)]),
            c("iftrue", &["MnBasinWrong"]),
        ]);
    }
    lever.extend(say("open_inner"));
    lever.extend(say("reveal"));
    lever.extend([
        c("setevent", &[&flag("INNER_OUTLET_ACKNOWLEDGED")]),
        c("scall", &[OPEN_PATH]),
        c("appear", &[ACTOR]),
        c("end", &[]),
    ]);
    put("MnBasinLever", lever);
    put(
        ACTOR,
        vec![
            c("checkevent", &[&flag("INNER_OUTLET_ACKNOWLEDGED")]),
            c("iffalse", &["MnBasinEnd"]),
            c("checkevent", &[&flag("CAUGHT")]),
            c("iftrue", &["MnBasinEnd"]),
            c("opentext", &[]),
            c("writetext", &[&text("ask")]),
            c("yesorno", &[]),
            c("iffalse", &["MnBasinDeclined"]),
            c("closetext", &[]),
            c("cry", &["STRANDRAK"]),
            c(
                "loadvar",
                &[
                    "VAR_BATTLETYPE",
                    crystal_core::battle::capture::OPTIONAL_LEGENDARY_BATTLE_TYPE,
                ],
            ),
            c("loadwildmon", &["STRANDRAK", "60"]),
            c("startbattle", &[]),
            c("reloadmapafterbattle", &[]),
            c("special", &["CheckCaughtCelebi"]),
            c("iftrue", &["MnBasinCaught"]),
            c("disappear", &[ACTOR]),
            c("jumptext", &[&text("retreat")]),
        ],
    );
    put(
        "MnBasinDeclined",
        vec![
            c("writetext", &[&text("decline")]),
            c("promptbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    put(
        "MnBasinCaught",
        vec![
            c("setevent", &[&flag("CAUGHT")]),
            c("disappear", &[ACTOR]),
            c("jumptext", &[&text("caught")]),
        ],
    );
    put("MnBasinEnd", vec![c("end", &[])]);
    for (label, restore) in [("MnBasinObjects", false), ("MnBasinNewVisit", true)] {
        let mut callback = vec![
            c("checkevent", &[&flag("CAUGHT")]),
            c("iftrue", &["MnBasinHide"]),
            c("checkevent", &[&flag("INNER_OUTLET_ACKNOWLEDGED")]),
            c("iffalse", &["MnBasinHide"]),
        ];
        if restore {
            callback.push(c("appear", &[ACTOR]));
        }
        callback.push(c("endcallback", &[]));
        put(label, callback);
    }
    put(
        "MnBasinHide",
        vec![c("disappear", &[ACTOR]), c("endcallback", &[])],
    );
    Ok(StrandrakDiscoveryPrograms {
        scripts,
        event_flags: event_names.iter().map(|name| flag(name)).collect(),
        callbacks: ["MnBasinObjects".into(), "MnBasinNewVisit".into()],
        terrain_hook: OPEN_PATH.into(),
    })
}
fn text(key: &str) -> String {
    format!("MnBasinText_{key}")
}
fn say(key: &str) -> Vec<Value> {
    vec![
        c("opentext", &[]),
        c("writetext", &[&text(key)]),
        c("promptbutton", &[]),
        c("closetext", &[]),
    ]
}
