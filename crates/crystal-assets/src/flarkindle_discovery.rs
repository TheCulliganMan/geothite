//! Native fen observations and a resettable root/water/living-peat sequence.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::grand_marais_modpack::authoring::command as c;

const PREFIX: &str = "EVENT_MN_FEN_";
const ACTOR: &str = "MN_FEN_FLARKINDLE";
const OPEN_PATH: &str = "MnFenOpenPath";

/// Native puzzle programs awaiting guarded fen geometry publication.
#[derive(Debug)]
pub struct FlarkindleDiscoveryPrograms {
    /// Deliberately paged dialogue and native command arrays.
    pub scripts: BTreeMap<String, Value>,
    /// Event identities the publisher must allocate before script compilation.
    pub event_flags: Vec<String>,
    /// Object and physical-new-visit callbacks for the inner fen map.
    pub callbacks: [String; 2],
    /// Required local subroutine to open the inspected optional path.
    /// The publisher must bind this label before parsing scripts, using guarded
    /// native block changes, `refreshmap` and a returning `end`. A TILES callback
    /// must restore the same path from `EVENT_MN_FEN_FOLDED_PATH_REVEALED`.
    pub terrain_hook: String,
}

/// Lowers the observations, ordered layer puzzle and voluntary encounter.
/// Three distinct viewpoints precede the observer's acknowledged comparison.
/// The inner loop accepts buried root, pool and living rib in that order.
/// Wrong input clears only two local sequence flags after its feedback page;
/// observations and a previously opened path never reset. Continue preserves
/// partial progress and pending acknowledgment. No map or pack is modified.
///
/// The publisher must bind separate observation and sequence viewpoints, the
/// observer, returned callbacks, creature and terrain hook. The final rib and
/// creature must share a map. The outer return must stay open in every state.
/// Refusal spends nothing; capture alone marks caught. Escape/knockout withdraws
/// for the visit, Continue retains withdrawal, and physical reentry retries.
/// This does not establish runtime, geometry or full-game acceptance.
///
/// # Errors
/// Rejects missing/duplicate contracts, foreign species, missing dialogue,
/// malformed native pages and lines exceeding the native text-box limits.
pub fn build_flarkindle_discovery_programs(source: &str) -> Result<FlarkindleDiscoveryPrograms> {
    let source: Value = serde_json::from_str(source)?;
    ensure!(
        source["schema_version"] == 1,
        "rare discovery source version"
    );
    let quests = source["quests"].as_array().context("discovery quests")?;
    let found = quests
        .iter()
        .filter(|quest| quest["id"] == "fen-memory")
        .collect::<Vec<_>>();
    ensure!(found.len() == 1, "one fen-memory contract required");
    let quest = found[0];
    ensure!(
        quest["species"] == serde_json::json!(["FLARKINDLE"]),
        "original Minnesota Flarkindle required"
    );
    let mut scripts = crate::discovery_dialogue::lower(quest, "MnFenText_")?;
    for key in [
        "observer_intro",
        "rib",
        "pool",
        "root",
        "observer_partial",
        "observer_compare",
        "root_selected",
        "pool_selected",
        "rib_selected",
        "wrong_order",
        "folded_path",
        "early",
        "reveal",
        "ask",
        "decline",
        "retreat",
        "caught",
        "observer_after",
    ] {
        ensure!(
            scripts.contains_key(&text(key)),
            "missing fen dialogue {key}"
        );
    }
    let mut put = |label: &str, commands: Vec<Value>| {
        scripts.insert(label.into(), Value::Array(commands));
    };
    let flag = |name: &str| format!("{PREFIX}{name}");
    let mut observer = vec![
        c("faceplayer", &[]),
        c("checkevent", &[&flag("FOLDED_PATH_REVEALED")]),
        c("iftrue", &["MnFenObserverAfter"]),
    ];
    for name in ["RIB_READ", "POOL_READ", "ROOT_READ"] {
        observer.extend([
            c("checkevent", &[&flag(name)]),
            c("iffalse", &["MnFenObserverWaiting"]),
        ]);
    }
    observer.extend(say("observer_compare"));
    observer.extend([c("setevent", &[&flag("LAYERS_COMPARED")]), c("end", &[])]);
    put("MnFenObserver", observer);
    let mut waiting = vec![
        c("checkevent", &[&flag("OBSERVER_READ")]),
        c("iftrue", &["MnFenObserverPartial"]),
    ];
    waiting.extend(say("observer_intro"));
    waiting.extend([c("setevent", &[&flag("OBSERVER_READ")]), c("end", &[])]);
    put("MnFenObserverWaiting", waiting);
    for (label, key) in [
        ("MnFenObserverPartial", "observer_partial"),
        ("MnFenObserverAfter", "observer_after"),
        ("MnFenEarly", "early"),
    ] {
        put(label, vec![c("jumptext", &[&text(key)])]);
    }
    for (label, key, event) in [
        ("MnFenReadRib", "rib", "RIB_READ"),
        ("MnFenReadPool", "pool", "POOL_READ"),
        ("MnFenReadRoot", "root", "ROOT_READ"),
    ] {
        let mut commands = say(key);
        commands.extend([c("setevent", &[&flag(event)]), c("end", &[])]);
        put(label, commands);
    }
    let gate = || {
        vec![
            c("checkevent", &[&flag("FOLDED_PATH_REVEALED")]),
            c("iftrue", &["MnFenEnd"]),
            c("checkevent", &[&flag("LAYERS_COMPARED")]),
            c("iffalse", &["MnFenEarly"]),
        ]
    };
    let mut root = gate();
    root.extend(say("root_selected"));
    root.extend([
        c("clearevent", &[&flag("SEQUENCE_POOL")]),
        c("setevent", &[&flag("SEQUENCE_ROOT")]),
        c("end", &[]),
    ]);
    put("MnFenChooseRoot", root);
    let mut pool = gate();
    pool.extend([
        c("checkevent", &[&flag("SEQUENCE_ROOT")]),
        c("iffalse", &["MnFenWrongOrder"]),
        c("checkevent", &[&flag("SEQUENCE_POOL")]),
        c("iftrue", &["MnFenWrongOrder"]),
    ]);
    pool.extend(say("pool_selected"));
    pool.extend([c("setevent", &[&flag("SEQUENCE_POOL")]), c("end", &[])]);
    put("MnFenChoosePool", pool);
    let mut rib = gate();
    for name in ["SEQUENCE_ROOT", "SEQUENCE_POOL"] {
        rib.extend([
            c("checkevent", &[&flag(name)]),
            c("iffalse", &["MnFenWrongOrder"]),
        ]);
    }
    rib.extend(say("rib_selected"));
    rib.extend(say("folded_path"));
    rib.extend([
        c("setevent", &[&flag("FOLDED_PATH_REVEALED")]),
        c("clearevent", &[&flag("SEQUENCE_ROOT")]),
        c("clearevent", &[&flag("SEQUENCE_POOL")]),
        c("scall", &[OPEN_PATH]),
        c("appear", &[ACTOR]),
        c("end", &[]),
    ]);
    put("MnFenChooseRib", rib);
    let mut wrong = say("wrong_order");
    wrong.extend([
        c("clearevent", &[&flag("SEQUENCE_ROOT")]),
        c("clearevent", &[&flag("SEQUENCE_POOL")]),
        c("end", &[]),
    ]);
    put("MnFenWrongOrder", wrong);
    let mut encounter = vec![
        c("checkevent", &[&flag("FOLDED_PATH_REVEALED")]),
        c("iffalse", &["MnFenEnd"]),
        c("checkevent", &[&flag("CAUGHT")]),
        c("iftrue", &["MnFenEnd"]),
    ];
    encounter.extend(say("reveal"));
    encounter.extend([
        c("opentext", &[]),
        c("writetext", &[&text("ask")]),
        c("yesorno", &[]),
        c("iffalse", &["MnFenDeclined"]),
        c("closetext", &[]),
        c("cry", &["FLARKINDLE"]),
        c(
            "loadvar",
            &[
                "VAR_BATTLETYPE",
                crystal_core::battle::capture::OPTIONAL_LEGENDARY_BATTLE_TYPE,
            ],
        ),
        c("loadwildmon", &["FLARKINDLE", "55"]),
        c("startbattle", &[]),
        c("reloadmapafterbattle", &[]),
        c("special", &["CheckCaughtCelebi"]),
        c("iftrue", &["MnFenCaught"]),
        c("disappear", &[ACTOR]),
        c("jumptext", &[&text("retreat")]),
    ]);
    put(ACTOR, encounter);
    put(
        "MnFenDeclined",
        vec![
            c("writetext", &[&text("decline")]),
            c("promptbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    put(
        "MnFenCaught",
        vec![
            c("setevent", &[&flag("CAUGHT")]),
            c("disappear", &[ACTOR]),
            c("jumptext", &[&text("caught")]),
        ],
    );
    put("MnFenEnd", vec![c("end", &[])]);
    for (label, restore) in [("MnFenObjects", false), ("MnFenNewVisit", true)] {
        let mut callback = vec![
            c("checkevent", &[&flag("CAUGHT")]),
            c("iftrue", &["MnFenHide"]),
            c("checkevent", &[&flag("FOLDED_PATH_REVEALED")]),
            c("iffalse", &["MnFenHide"]),
        ];
        if restore {
            callback.push(c("appear", &[ACTOR]));
        }
        callback.push(c("endcallback", &[]));
        put(label, callback);
    }
    put(
        "MnFenHide",
        vec![c("disappear", &[ACTOR]), c("endcallback", &[])],
    );
    Ok(FlarkindleDiscoveryPrograms {
        scripts,
        event_flags: [
            "RIB_READ",
            "POOL_READ",
            "ROOT_READ",
            "LAYERS_COMPARED",
            "FOLDED_PATH_REVEALED",
            "SEQUENCE_ROOT",
            "SEQUENCE_POOL",
            "OBSERVER_READ",
            "CAUGHT",
        ]
        .into_iter()
        .map(flag)
        .collect(),
        callbacks: ["MnFenObjects".into(), "MnFenNewVisit".into()],
        terrain_hook: OPEN_PATH.into(),
    })
}
fn text(key: &str) -> String {
    format!("MnFenText_{key}")
}
fn say(key: &str) -> Vec<Value> {
    vec![
        c("opentext", &[]),
        c("writetext", &[&text(key)]),
        c("promptbutton", &[]),
        c("closetext", &[]),
    ]
}
