//! Let the three visible peatland landmarks carry the investigation from first read to reveal.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::authored_dialogue;
use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, MapModule};

const FEN: &str = "MnLayeredFen";
const OPEN: &str = "EVENT_MN_FEN_FOLDED_PATH_REVEALED";
const COMPARED: &str = "EVENT_MN_FEN_LAYERS_COMPARED";
const ROOT: &str = "EVENT_MN_FEN_SEQUENCE_ROOT";
const POOL: &str = "EVENT_MN_FEN_SEQUENCE_POOL";
const OBSERVER: &str = "EVENT_MN_FEN_OBSERVER_READ";
type Feature = (&'static str, &'static str, (u16, u16), (u16, u16));
const FEATURES: [Feature; 3] = [
    ("Root", "EVENT_MN_FEN_ROOT_READ", (12, 12), (12, 15)),
    ("Pool", "EVENT_MN_FEN_POOL_READ", (12, 18), (12, 20)),
    ("Rib", "EVENT_MN_FEN_RIB_READ", (30, 12), (28, 14)),
];

fn body<'a>(room: &'a mut MapModule, label: &str) -> Result<&'a mut Vec<Value>> {
    room.scripts
        .get_mut(label)
        .and_then(Value::as_array_mut)
        .with_context(|| format!("missing Fen script {label}"))
}

fn branch(flag: &str, test: &str, label: &str) -> [Value; 2] {
    [command("checkevent", &[flag]), command(test, &[label])]
}

fn new_label(room: &mut MapModule, label: &str, rows: Vec<Value>) -> Result<()> {
    ensure!(
        room.scripts.insert(label.into(), rows.into()).is_none(),
        "duplicate Fen label {label}"
    );
    Ok(())
}

/// Preserves the six existing search squares and their persistent flags. A read at
/// either contact records the same observation and later operates that landmark.
///
/// Returns an error if the original landmarks, scripts or reveal no longer match.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(FEN).context("Layered Fen")?.clone();
    let mut recognize = Vec::new();
    for (feature, flag, original, nearby) in FEATURES {
        let read = format!("MnFenRead{feature}");
        let choose = format!("MnFenChoose{feature}");
        let look = format!("MnFenLook{feature}");
        let page = format!("MnFenText_{}", feature.to_lowercase());
        for (pos, label) in [(original, &read), (nearby, &choose)] {
            ensure!(
                room.events.bg_events.iter().any(|e| (e.x, e.y) == pos
                    && e.event_type == "BGEVENT_READ"
                    && &e.script == label),
                "missing Fen contact {label}"
            );
        }
        let rows = body(&mut room, &read)?;
        ensure!(
            rows.first() == Some(&command("opentext", &[]))
                && rows.ends_with(&[command("setevent", &[flag]), command("end", &[])]),
            "unexpected {read}"
        );
        rows.pop();
        rows.push(command("sjump", &["MnFenRecognizeLayers"]));
        rows.splice(
            ..0,
            branch(OPEN, "iftrue", &look)
                .into_iter()
                .chain(branch(COMPARED, "iftrue", &choose)),
        );

        let rows = body(&mut room, &choose)?;
        ensure!(
            rows.starts_with(
                &[
                    branch(OPEN, "iftrue", "MnFenEnd"),
                    branch(COMPARED, "iffalse", "MnFenEarly")
                ]
                .concat()
            ),
            "unexpected {choose}"
        );
        rows[1] = command("iftrue", &[&look]);
        rows[3] = command("iffalse", &[&read]);
        if feature == "Root" {
            let reset = rows
                .iter()
                .position(|r| *r == command("clearevent", &[POOL]))
                .context("old root reset")?;
            rows.remove(reset);
            rows.splice(4..4, branch(ROOT, "iftrue", "MnFenWrongOrder"));
        }
        new_label(&mut room, &look, vec![command("jumptext", &[&page])])?;
        recognize.extend(branch(flag, "iffalse", "MnFenEnd"));
    }
    recognize.extend([
        command("opentext", &[]),
        command("writetext", &["MnFenText_layers_settle"]),
        command("promptbutton", &[]),
        command("closetext", &[]),
        command("setevent", &[COMPARED]),
        command("end", &[]),
    ]);
    new_label(&mut room, "MnFenRecognizeLayers", recognize)?;

    // Retain the observer's original comparison for saves with all three
    // observations recorded before the sequence became available.
    let observer = body(&mut room, "MnFenObserver")?;
    ensure!(
        observer.get(1..3) == Some(branch(OPEN, "iftrue", "MnFenObserverAfter").as_slice()),
        "unexpected Fen observer"
    );
    observer.splice(3..3, branch(COMPARED, "iftrue", "MnFenWrongOrder"));
    let waiting = body(&mut room, "MnFenObserverWaiting")?;
    ensure!(
        waiting.starts_with(&branch(OBSERVER, "iftrue", "MnFenObserverPartial")),
        "unexpected first Fen visit"
    );
    waiting.splice(
        2..2,
        FEATURES
            .iter()
            .flat_map(|(_, flag, _, _)| branch(flag, "iftrue", "MnFenObserverPartial")),
    );
    let partial = body(&mut room, "MnFenObserverPartial")?;
    ensure!(
        partial.as_slice() == [command("jumptext", &["MnFenText_observer_partial"])],
        "unexpected Fen partial observation"
    );
    *partial = vec![command("setevent", &[OBSERVER])];
    partial.extend(branch(FEATURES[0].1, "iffalse", "MnFenHintRoot"));
    partial.extend(branch(FEATURES[1].1, "iffalse", "MnFenEarly"));
    partial.push(command("sjump", &["MnFenHintRib"]));

    let wrong = body(&mut room, "MnFenWrongOrder")?;
    ensure!(
        wrong.ends_with(&[
            command("clearevent", &[ROOT]),
            command("clearevent", &[POOL]),
            command("end", &[])
        ]),
        "unexpected Fen sequence reset"
    );
    *wrong = branch(ROOT, "iffalse", "MnFenHintRoot").into();
    wrong.extend(branch(POOL, "iffalse", "MnFenEarly"));
    wrong.push(command("sjump", &["MnFenHintRib"]));
    for (label, text) in [
        ("MnFenHintRoot", "MnFenText_observer_partial"),
        ("MnFenHintRib", "MnFenText_wrong_order"),
    ] {
        new_label(&mut room, label, vec![command("jumptext", &[text])])?;
    }
    for key in [
        "observer_intro",
        "observer_partial",
        "observer_compare",
        "observer_after",
        "root",
        "pool",
        "rib",
        "root_selected",
        "pool_selected",
        "rib_selected",
        "folded_path",
        "early",
        "wrong_order",
        "layers_settle",
    ] {
        authored_dialogue::install_canonical(&mut room, &format!("MnFenText_{key}"))?;
    }
    let scripts = &room.scripts;
    room.script_text_bodies = crate::parse_script_text_bodies(FEN, scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(FEN, scripts)?;
    room.script_runtime_commands = crate::parse_script_runtime_commands(FEN, scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(FEN, scripts)?;
    room.script_flag_commands = crate::parse_script_flag_commands(FEN, scripts)?;
    room.script_object_commands = crate::parse_script_object_commands(FEN, scripts)?;
    crate::authored_town_publication::replace(pack, room)
}
