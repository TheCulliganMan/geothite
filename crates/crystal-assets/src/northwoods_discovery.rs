//! Native script lowering for the optional, independently catchable northwoods pair.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::grand_marais_modpack::authoring::command as c;

const PREFIX: &str = "EVENT_MN_TALL_TALE_";
const CLUES: [(&str, &str); 3] = [
    ("Culvert", "CULVERT_READ"),
    ("Prints", "PRINTS_READ"),
    ("Stack", "STACK_READ"),
];

/// Native programs and event names awaiting geometry/actor publication.
/// This value does not install a map, register flags or change a pack.
#[derive(Debug)]
pub struct NorthwoodsDiscoveryPrograms {
    /// Native command arrays, including deliberately paged text programs.
    pub scripts: BTreeMap<String, Value>,
    /// Event identities that the publisher must allocate before installation.
    pub event_flags: Vec<String>,
    /// Object, new-visit and terrain callback labels, respectively.
    pub callbacks: [String; 3],
}

/// Lowers the authored northwoods discovery into native Crystal script commands.
/// The worker compares three acknowledged observations, the player clears an
/// optional path, and a marked-log report unlocks both encounters independently.
/// Only actual capture sets each caught flag. Visit callbacks preserve retreat
/// through Continue and restore uncaught creatures on physical reentry.
///
/// The geometry publisher must bind `MnTaleWorker`, `MnTalePath`, `MnTaleLog`,
/// `MnTaleCulvert`, `MnTalePrints` and `MnTaleStack`, and install native creature
/// objects `MN_TALE_BABEHEMOTH` and `MN_TALE_BUNJANT`. It must also bind all three
/// callback labels and allocate the returned event names. No publication or
/// gameplay acceptance is implied by lowering these programs alone.
///
/// # Errors
/// Rejects missing/duplicate northwoods contracts, malformed text, empty pages,
/// pages exceeding two lines, non-ASCII text or lines wider than 18 columns.
pub fn build_northwoods_discovery_programs(source: &str) -> Result<NorthwoodsDiscoveryPrograms> {
    let source: Value = serde_json::from_str(source)?;
    ensure!(
        source["schema_version"] == 1,
        "rare discovery source version"
    );
    let quests = source["quests"]
        .as_array()
        .context("rare discovery quests")?;
    let matches = quests
        .iter()
        .filter(|q| q["id"] == "northwoods-tall-tales")
        .collect::<Vec<_>>();
    ensure!(matches.len() == 1, "one northwoods contract required");
    let quest = matches[0];
    ensure!(
        quest["species"] == serde_json::json!(["BABEHEMOTH", "BUNJANT"]),
        "canonical original northwoods pair required"
    );
    let pages = quest["dialogue_pages"]
        .as_object()
        .context("native northwoods dialogue pages")?;
    let mut scripts = BTreeMap::new();
    for (key, pages) in pages {
        let pages = pages.as_array().context("dialogue page list")?;
        ensure!(!pages.is_empty(), "empty dialogue {key}");
        let mut commands = Vec::new();
        for (index, page) in pages.iter().enumerate() {
            let lines = page.as_array().context("dialogue lines")?;
            ensure!(!lines.is_empty() && lines.len() <= 2, "two-line page {key}");
            for (line, text) in lines.iter().enumerate() {
                let text = text.as_str().context("dialogue line")?;
                ensure!(
                    !text.is_empty()
                        && text.is_ascii()
                        && text.len() <= 18
                        && !text.contains(['"', '\n', '\r']),
                    "invalid native dialogue line in {key}"
                );
                let op = if line == 1 {
                    "line"
                } else if index == 0 {
                    "text"
                } else {
                    "para"
                };
                commands.push(c(op, &[&format!("\"{text}\"")]));
            }
        }
        commands.push(c("done", &[]));
        scripts.insert(text(key), Value::Array(commands));
    }
    for key in [
        "worker_introduction",
        "culvert",
        "hoofprint",
        "windthrow",
        "worker_partial",
        "worker_comparison",
        "blocked_path",
        "path_refusal",
        "path_cleared",
        "marked_log",
        "worker_log_account",
        "ox_reveal",
        "ox_ask",
        "keeper_reveal",
        "keeper_ask",
        "retry",
        "worker_after",
        "camp_record",
    ] {
        ensure!(scripts.contains_key(&text(key)), "missing dialogue {key}");
    }
    let mut put = |label: &str, commands: Vec<Value>| {
        scripts.insert(label.into(), Value::Array(commands));
    };
    let flag = |name: &str| format!("{PREFIX}{name}");
    put(
        "MnTaleWorker",
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[&flag("OX_REVEALED")]),
            c("iftrue", &["MnTaleAfter"]),
            c("checkevent", &[&flag("LOG_ACCOUNT_READ")]),
            c("iftrue", &["MnTaleReveal"]),
            c("checkevent", &[&flag("CULVERT_READ")]),
            c("iffalse", &["MnTaleIntroduction"]),
            c("checkevent", &[&flag("PRINTS_READ")]),
            c("iffalse", &["MnTaleIntroduction"]),
            c("checkevent", &[&flag("STACK_READ")]),
            c("iffalse", &["MnTaleIntroduction"]),
            c("scall", &["MnTaleComparisonTextPage"]),
            c("setevent", &[&flag("CAMP_DIRECTIONS")]),
            c("end", &[]),
        ],
    );
    for (label, key) in [
        ("MnTaleIntroduction", "worker_introduction"),
        ("MnTaleAfter", "worker_after"),
    ] {
        let mut commands = say(key);
        commands.push(c("end", &[]));
        put(label, commands);
    }
    let mut comparison = say("worker_comparison");
    comparison.push(c("end", &[]));
    put("MnTaleComparisonTextPage", comparison);
    let mut reveal = say("worker_log_account");
    for name in ["OX_REVEALED", "CAMP_KEEPER_REVEALED"] {
        reveal.push(c("setevent", &[&flag(name)]));
    }
    for actor in ["MN_TALE_BABEHEMOTH", "MN_TALE_BUNJANT"] {
        reveal.push(c("appear", &[actor]));
    }
    reveal.push(c("end", &[]));
    put("MnTaleReveal", reveal);
    for ((label, event), key) in CLUES.into_iter().zip(["culvert", "hoofprint", "windthrow"]) {
        let mut commands = say(key);
        commands.push(c("setevent", &[&flag(event)]));
        commands.push(c("end", &[]));
        put(&format!("MnTale{label}"), commands);
    }
    let mut path = vec![
        c("checkevent", &[&flag("CAMP_PATH_ACKNOWLEDGED")]),
        c("iftrue", &["MnTalePathDone"]),
        c("checkevent", &[&flag("CAMP_DIRECTIONS")]),
        c("iffalse", &["MnTaleNeedDirections"]),
        c("opentext", &[]),
        c("writetext", &[&text("blocked_path")]),
        c("yesorno", &[]),
        c("iffalse", &["MnTalePathDeclined"]),
        c("closetext", &[]),
        c("appear", &["MN_TALE_BUNJANT"]),
    ];
    path.extend(say("path_cleared"));
    path.extend([
        c("disappear", &["MN_TALE_BUNJANT"]),
        c("setevent", &[&flag("CAMP_PATH_ACKNOWLEDGED")]),
        c("changeblock", &["28", "16", "1"]),
        c("refreshmap", &[]),
        c("end", &[]),
    ]);
    put("MnTalePath", path);
    put("MnTalePathDone", vec![c("end", &[])]);
    put(
        "MnTaleNeedDirections",
        vec![c("jumptext", &[&text("worker_partial")])],
    );
    put(
        "MnTalePathDeclined",
        vec![
            c("writetext", &[&text("path_refusal")]),
            c("promptbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    let mut log = vec![
        c("checkevent", &[&flag("CAMP_PATH_ACKNOWLEDGED")]),
        c("iffalse", &["MnTaleNeedDirections"]),
    ];
    log.extend(say("marked_log"));
    log.extend([c("setevent", &[&flag("LOG_ACCOUNT_READ")]), c("end", &[])]);
    put("MnTaleLog", log);
    put(
        "MnTaleCampRecord",
        vec![c("jumptext", &[&text("camp_record")])],
    );
    // The two encounter branches never test the other creature's caught flag.
    for (species, ready, reveal, ask) in [
        ("BABEHEMOTH", "OX_REVEALED", "ox_reveal", "ox_ask"),
        (
            "BUNJANT",
            "CAMP_KEEPER_REVEALED",
            "keeper_reveal",
            "keeper_ask",
        ),
    ] {
        let actor = format!("MN_TALE_{species}");
        let caught = flag(&format!("{species}_CAUGHT"));
        let declined = format!("MnTale{species}Declined");
        let captured = format!("MnTale{species}Caught");
        let mut commands = vec![
            c("checkevent", &[&flag(ready)]),
            c("iffalse", &["MnTalePathDone"]),
            c("checkevent", &[&caught]),
            c("iftrue", &["MnTalePathDone"]),
        ];
        commands.extend(say(reveal));
        commands.extend([
            c("opentext", &[]),
            c("writetext", &[&text(ask)]),
            c("yesorno", &[]),
            c("iffalse", &[&declined]),
            c("closetext", &[]),
            c("cry", &[species]),
            c(
                "loadvar",
                &[
                    "VAR_BATTLETYPE",
                    crystal_core::battle::capture::OPTIONAL_LEGENDARY_BATTLE_TYPE,
                ],
            ),
            c("loadwildmon", &[species, "60"]),
            c("startbattle", &[]),
            c("reloadmapafterbattle", &[]),
            c("special", &["CheckCaughtCelebi"]),
            c("iftrue", &[&captured]),
            c("disappear", &[&actor]),
            c("jumptext", &[&text("retry")]),
        ]);
        put(&actor, commands);
        put(&declined, vec![c("closetext", &[]), c("end", &[])]);
        put(
            &captured,
            vec![
                c("setevent", &[&caught]),
                c("disappear", &[&actor]),
                c("end", &[]),
            ],
        );
    }
    install_callbacks(&mut scripts, &flag);
    scripts.insert(
        "MnTaleTerrain".into(),
        Value::Array(vec![
            c("checkevent", &[&flag("CAMP_PATH_ACKNOWLEDGED")]),
            c("iftrue", &["MnTaleTerrainOpen"]),
            c("changeblock", &["28", "16", "93"]),
            c("endcallback", &[]),
        ]),
    );
    scripts.insert(
        "MnTaleTerrainOpen".into(),
        Value::Array(vec![
            c("changeblock", &["28", "16", "1"]),
            c("endcallback", &[]),
        ]),
    );
    let event_flags = [
        "CULVERT_READ",
        "PRINTS_READ",
        "STACK_READ",
        "CAMP_DIRECTIONS",
        "CAMP_PATH_ACKNOWLEDGED",
        "LOG_ACCOUNT_READ",
        "OX_REVEALED",
        "CAMP_KEEPER_REVEALED",
        "BABEHEMOTH_CAUGHT",
        "BUNJANT_CAUGHT",
    ]
    .into_iter()
    .map(flag)
    .collect();
    Ok(NorthwoodsDiscoveryPrograms {
        scripts,
        event_flags,
        callbacks: [
            "MnTaleObjects".into(),
            "MnTaleNewVisit".into(),
            "MnTaleTerrain".into(),
        ],
    })
}

fn text(key: &str) -> String {
    format!("MnTaleText_{key}")
}
fn say(key: &str) -> Vec<Value> {
    vec![
        c("opentext", &[]),
        c("writetext", &[&text(key)]),
        c("promptbutton", &[]),
        c("closetext", &[]),
    ]
}

fn install_callbacks(scripts: &mut BTreeMap<String, Value>, flag: &impl Fn(&str) -> String) {
    for (label, restore) in [("MnTaleObjects", false), ("MnTaleNewVisit", true)] {
        let mut next = "MnTaleCallbackEnd".to_string();
        for (species, ready) in [
            ("BUNJANT", "CAMP_KEEPER_REVEALED"),
            ("BABEHEMOTH", "OX_REVEALED"),
        ] {
            let stage = format!("{label}{species}");
            let hidden = format!("{stage}Hide");
            let actor = format!("MN_TALE_{species}");
            let mut commands = vec![
                c("checkevent", &[&flag(&format!("{species}_CAUGHT"))]),
                c("iftrue", &[&hidden]),
                c("checkevent", &[&flag(ready)]),
                c("iffalse", &[&hidden]),
            ];
            if restore {
                commands.push(c("appear", &[&actor]));
            }
            commands.push(c("sjump", &[&next]));
            scripts.insert(stage.clone(), Value::Array(commands));
            scripts.insert(
                hidden,
                Value::Array(vec![c("disappear", &[&actor]), c("sjump", &[&next])]),
            );
            next = stage;
        }
        scripts.insert(label.into(), Value::Array(vec![c("sjump", &[&next])]));
    }
    scripts.insert(
        "MnTaleCallbackEnd".into(),
        Value::Array(vec![c("endcallback", &[])]),
    );
}

#[cfg(test)]
#[path = "northwoods_discovery_tests.rs"]
mod tests;
