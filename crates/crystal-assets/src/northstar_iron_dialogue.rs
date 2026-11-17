//! Pressure-gauge feedback, rescue dialogue and crew directions in the Iron Range.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;
use crate::{CompiledGamePack, MapModule};

const HAUL: &str = "EVENT_MN_IRON_HAUL_PRESSURE_OFF";
const VENT: &str = "EVENT_MN_IRON_VENT_PRESSURE_OFF";

/// Installs the reviewed pages and routes existing interactions by their saved flags.
/// The caller publishes the complete room, including its typed and global scripts.
pub(crate) fn install(
    pack: &CompiledGamePack,
    room: &mut MapModule,
    dialogue: &Value,
) -> Result<()> {
    let pages = dialogue[&room.id]
        .as_object()
        .with_context(|| format!("missing Iron Range dialogue for {}", room.id))?;
    let new_texts: &[&str] = match room.id.as_str() {
        "MnIronSump" => &[
            "MnIronSumpHaulOnlyText",
            "MnIronSumpVentOnlyText",
            "MnIronSumpOpenText",
        ],
        "MnIronPumpGallery" => &["MnNDrillGalleryStoppedText"],
        "MnIronDrillCore" => &["MnNDrillForemanStoppedText"],
        _ => &[],
    };
    for &label in new_texts {
        ensure!(
            pages.contains_key(label),
            "missing {} dialogue {label}",
            room.id
        );
    }
    for (label, raw) in pages {
        let pages: Vec<AuthoredDialoguePage> = serde_json::from_value(raw.clone())?;
        let mut body =
            authored_dialogue::compile(&pages).with_context(|| format!("{}/{label}", room.id))?;
        if new_texts.contains(&label.as_str()) {
            add(pack, room, label, body)?;
        } else {
            let (terminal, lines) = room
                .scripts
                .get(label)
                .and_then(Value::as_array)
                .and_then(|old| old.split_last())
                .with_context(|| format!("missing original {} text {label}", room.id))?;
            ensure!(
                room.script_text_bodies.contains_key(label)
                    && lines.iter().all(|line| matches!(
                        line["command"].as_str(),
                        Some("text" | "line" | "para" | "cont")
                    ))
                    && matches!(terminal["command"].as_str(), Some("done" | "prompt")),
                "expected plain {} dialogue {label}",
                room.id
            );
            *body
                .as_array_mut()
                .and_then(|program| program.last_mut())
                .context("compiled text terminator")? = terminal.clone();
            room.scripts.insert(label.clone(), body);
        }
    }
    match room.id.as_str() {
        "MnIronSump" => {
            add(
                pack,
                room,
                "MnIronSumpReadHaulOff",
                json!([
                    c("checkevent", &[VENT]),
                    c("iftrue", &["MnIronSumpReadOpen"]),
                    c("jumptext", &["MnIronSumpHaulOnlyText"])
                ]),
            )?;
            add(pack, room, "MnIronSumpReadOpen", jump("MnIronSumpOpenText"))?;
            add(
                pack,
                room,
                "MnIronSumpReadVentOnly",
                jump("MnIronSumpVentOnlyText"),
            )?;
            replace_read(
                room,
                "MnIronSumpGateRead",
                "MnIronSumpGateText",
                json!([
                    c("checkevent", &[HAUL]),
                    c("iftrue", &["MnIronSumpReadHaulOff"]),
                    c("checkevent", &[VENT]),
                    c("iftrue", &["MnIronSumpReadVentOnly"]),
                    c("jumptext", &["MnIronSumpGateText"])
                ]),
            )
        }
        "MnIronPumpGallery" => after_stop(
            pack,
            room,
            "MnNDrillGalleryAfter",
            "MnNDrillGalleryAfterText",
            "MnNDrillGalleryStopped",
            "MnNDrillGalleryStoppedText",
        ),
        "MnIronDrillCore" => after_stop(
            pack,
            room,
            "MN_NDRILL_ENGINEER",
            "MnNDrillForemanText",
            "MnNDrillForemanStopped",
            "MnNDrillForemanStoppedText",
        ),
        _ => Ok(()),
    }
}

fn jump(text: &str) -> Value {
    json!([c("jumptext", &[text])])
}

fn add(pack: &CompiledGamePack, room: &mut MapModule, label: &str, body: Value) -> Result<()> {
    ensure!(
        !pack.data.map_scripts.contains_key(label) && !room.scripts.contains_key(label),
        "occupied Iron Range label {label}"
    );
    room.scripts.insert(label.into(), body);
    Ok(())
}

fn replace_read(room: &mut MapModule, entry: &str, old: &str, body: Value) -> Result<()> {
    let original = room
        .scripts
        .get_mut(entry)
        .with_context(|| format!("missing {} entry {entry}", room.id))?;
    ensure!(
        *original == jump(old),
        "unexpected original Iron Range interaction {entry}"
    );
    *original = body;
    Ok(())
}

fn after_stop(
    pack: &CompiledGamePack,
    room: &mut MapModule,
    entry: &str,
    old: &str,
    branch: &str,
    stopped: &str,
) -> Result<()> {
    add(pack, room, branch, jump(stopped))?;
    replace_read(
        room,
        entry,
        old,
        json!([
            c("checkevent", &[crate::northstar_drill_lair::OFF]),
            c("iftrue", &[branch]),
            c("jumptext", &[old])
        ]),
    )
}
