//! Let the three acknowledged field observations uncover the hollow in one visit to each.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, authored_town_publication};

const OPEN: &str = "EVENT_MN_FEN_FOLDED_PATH_REVEALED";
const COMPARED: &str = "EVENT_MN_FEN_LAYERS_COMPARED";
const GUIDE: &str = "EVENT_MN_FEN_OBSERVER_READ";
const FEATURES: [(&str, &str, &str); 3] = [
    ("Root", "EVENT_MN_FEN_ROOT_READ", "MnFenHintRoot"),
    ("Pool", "EVENT_MN_FEN_POOL_READ", "MnFenEarly"),
    ("Rib", "EVENT_MN_FEN_RIB_READ", "MnFenHintRib"),
];

pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let map = "MnLayeredFen";
    let mut room = pack.data.maps.get(map).context("Layered Fen")?.clone();
    let reveal = room.scripts.get("MnFenChooseRib").context("fen reveal")?;
    ensure!(
        reveal.as_array().is_some_and(|v| v.ends_with(&[
            command("scall", &["MnFenOpenPath"]),
            command("appear", &["MN_FEN_FLARKINDLE"]),
            command("end", &[])
        ])),
        "expected the Fen Trail encounter"
    );
    let scripts = &mut room.scripts;
    let mut recognized = Vec::new();
    let mut missing = Vec::new();
    let mut guide = vec![
        command("faceplayer", &[]),
        command("checkevent", &[OPEN]),
        command("iftrue", &["MnFenObserverAfter"]),
        command("checkevent", &[GUIDE]),
        command("iftrue", &["MnFenTrailHint"]),
    ];
    for (feature, flag, hint) in FEATURES {
        let read = format!("MnFenRead{feature}");
        let choose = format!("MnFenChoose{feature}");
        let look = format!("MnFenLook{feature}");
        let page = format!("MnFenText_{}", feature.to_lowercase());
        ensure!(
            scripts.contains_key(&read)
                && scripts.contains_key(&choose)
                && scripts.contains_key(&look)
                && scripts.contains_key(&page),
            "missing {feature} observations"
        );
        scripts.insert(
            read.clone(),
            Value::Array(vec![
                command("checkevent", &[OPEN]),
                command("iftrue", &[&look]),
                command("checkevent", &[flag]),
                command("iftrue", &["MnFenTrailHint"]),
                command("opentext", &[]),
                command("writetext", &[&page]),
                command("promptbutton", &[]),
                command("closetext", &[]),
                command("setevent", &[flag]),
                command("sjump", &["MnFenRecognizeLayers"]),
            ]),
        );
        scripts.insert(choose, Value::Array(vec![command("sjump", &[&read])]));
        recognized.extend([
            command("checkevent", &[flag]),
            command("iffalse", &["MnFenEnd"]),
        ]);
        missing.extend([command("checkevent", &[flag]), command("iffalse", &[hint])]);
        guide.extend([
            command("checkevent", &[flag]),
            command("iftrue", &["MnFenTrailHint"]),
        ]);
    }
    // All three reads can already be recorded in a returning player's state.
    // Talking to the observer or examining any feature finishes that investigation.
    missing.push(command("sjump", &["MnFenRecognizeLayers"]));
    ensure!(
        scripts
            .insert("MnFenTrailHint".into(), Value::Array(missing))
            .is_none(),
        "occupied fen hint"
    );
    guide.extend([
        command("opentext", &[]),
        command("writetext", &["MnFenText_observer_intro"]),
        command("promptbutton", &[]),
        command("closetext", &[]),
        command("setevent", &[GUIDE]),
        command("end", &[]),
    ]);
    scripts.insert("MnFenObserver".into(), Value::Array(guide));
    recognized.extend([
        command("opentext", &[]),
        command("writetext", &["MnFenText_layers_settle"]),
        command("promptbutton", &[]),
        command("closetext", &[]),
        command("setevent", &[COMPARED]),
        command("setevent", &[OPEN]),
        command("clearevent", &["EVENT_MN_FEN_SEQUENCE_ROOT"]),
        command("clearevent", &["EVENT_MN_FEN_SEQUENCE_POOL"]),
        command("scall", &["MnFenOpenPath"]),
        command("checkevent", &["EVENT_MN_FEN_CAUGHT"]),
        command("iftrue", &["MnFenTrailOpened"]),
        command("appear", &["MN_FEN_FLARKINDLE"]),
        command("sjump", &["MnFenTrailOpened"]),
    ]);
    scripts.insert("MnFenRecognizeLayers".into(), Value::Array(recognized));
    ensure!(
        scripts
            .insert(
                "MnFenTrailOpened".into(),
                Value::Array(vec![command("jumptext", &["MnFenText_folded_path"])])
            )
            .is_none(),
        "occupied fen opening"
    );
    room.script_runtime_commands = crate::parse_script_runtime_commands(map, scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(map, scripts)?;
    room.script_flag_commands = crate::parse_script_flag_commands(map, scripts)?;
    room.script_object_commands = crate::parse_script_object_commands(map, scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(map, scripts)?;
    authored_town_publication::replace(pack, room)
}
