//! Interleaved mine shafts with independent pressure circuits and real crosscuts.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;

use crate::grand_marais_modpack::authoring;
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_lair_authoring as a;
use crate::{CompiledGamePack, MapModule};

const FLOORS: [(&str, &str); 3] = [
    ("MnIronHaulage", "MN_IRON_HAULAGE"),
    ("MnIronSump", "MN_IRON_SUMP"),
    ("MnIronVentilation", "MN_IRON_VENTILATION"),
];
const APPROACH: &str = "MnIronDrillApproach";
const GALLERY: &str = "MnIronPumpGallery";

#[derive(Deserialize)]
struct Port {
    map: String,
    active: [u16; 2],
    arrival: [u16; 2],
}
#[derive(Deserialize)]
struct Link {
    id: String,
    a: Port,
    b: Port,
}
#[derive(Deserialize)]
struct Control {
    id: String,
    map: String,
    tile: [u16; 2],
    flag: String,
    door: [u16; 2],
}
#[derive(Deserialize)]
struct Gate {
    map: String,
    door: [u16; 2],
}
#[derive(Deserialize)]
struct Network {
    schema_version: u8,
    id: String,
    maps: Vec<String>,
    links: Vec<Link>,
    controls: Vec<Control>,
    gate: Gate,
}

/// Inserts three interleaved shaft levels ahead of the existing pump gallery.
///
/// Each pressure control opens a physical crosscut. Both circuits must be off
/// before the sump shutter opens. No existing drill, rescue or rival result is
/// granted by these controls. Apply before the regional species binder.
///
/// # Errors
/// Returns an error for incompatible parents, occupied identities, malformed
/// layouts or ports, unsupported scripts, or failed runtime verification.
pub fn build_northstar_iron_workings(
    base: &CompiledGamePack,
    network: &str,
    layouts: &[String],
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.report
            .manifests
            .iter()
            .any(|m| m == "northstar-ice-headquarters-v1")
            && (!base
                .report
                .manifests
                .iter()
                .any(|m| m == "northstar-minnesota-only-v1")
                || crate::northstar_current_story::is_composing(base))
            && !base.data.maps.contains_key(FLOORS[0].0),
        "unbound headquarters parent required"
    );
    let source: Network = serde_json::from_str(network)?;
    ensure!(
        source.schema_version == 1
            && source.id == "northstar-iron-workings-v1"
            && source.maps == FLOORS.map(|(m, _)| m)
            && layouts.len() == 3
            && source.links.len() == 8
            && source.controls.len() == 2,
        "complete authored mine network required"
    );
    let mut pack = base.clone();
    let mut maps = BTreeMap::new();
    for ((name, constant), layout) in FLOORS.iter().zip(layouts) {
        let room = a::room(&pack, &serde_json::from_str(layout)?, constant)?;
        ensure!(room.id == *name, "shaft identity differs");
        maps.insert(name.to_string(), room);
    }
    for (name, old_target) in [
        (APPROACH, "MN_IRON_PUMP_GALLERY"),
        (GALLERY, "MN_IRON_DRILL_APPROACH"),
    ] {
        let mut room = pack.data.maps[name].clone();
        // Retain indices: external mouth/core links still target their original
        // safe records. Only the old approach-gallery pair is disconnected.
        for warp in room
            .events
            .warps
            .iter_mut()
            .filter(|w| w.target_map_constant == old_target)
        {
            warp.target_map = name.into();
            warp.target_map_constant = room
                .attributes
                .map_constant
                .clone()
                .context("map constant")?;
            warp.target_warp_id = if name == APPROACH { 4 } else { 2 };
        }
        maps.insert(name.into(), room);
    }
    for link in &source.links {
        connect(&mut maps, link)?;
    }
    // Replace the old port records with the new reciprocal records, preserving
    // their positions and indices for all external arrivals.
    for name in [APPROACH, GALLERY] {
        let room = maps.get_mut(name).context("original shaft")?;
        let end = room.events.warps.len();
        let replacements = room.events.warps[end - 2..].to_vec();
        for mut replacement in replacements {
            let old = room.events.warps[..end - 2]
                .iter_mut()
                .find(|w| w.x == replacement.x && w.y == replacement.y)
                .context("original port coordinate")?;
            replacement.index = old.index;
            *old = replacement;
        }
        room.events.warps.truncate(end - 2);
    }
    // New destinations must target the retained original safe indices rather
    // than the temporary appended pair used while constructing the network.
    for room in maps.values_mut() {
        for warp in &mut room.events.warps {
            if warp.target_map_constant == "MN_IRON_DRILL_APPROACH" {
                warp.target_warp_id = 4;
            }
            if warp.target_map_constant == "MN_IRON_PUMP_GALLERY" {
                warp.target_warp_id = 2;
            }
        }
    }
    for control in &source.controls {
        a::flag(&mut pack, &control.flag)?;
        control_program(maps.get_mut(&control.map).context("control room")?, control)?;
    }
    gate_program(
        maps.get_mut(&source.gate.map).context("sump room")?,
        &source,
    )?;
    install_people(&mut pack, &mut maps)?;
    for (name, _) in FLOORS {
        a::publish(
            &mut pack,
            maps.remove(name).context("new shaft")?,
            "MnR09Cave1F",
            [4, 19],
        )?;
    }
    for (_, mut room) in maps {
        crate::ely_league_rooms::finish(&pack, &mut room)?;
        room.script_audio_commands = crate::parse_script_audio_commands(&room.id, &room.scripts)?;
        crate::authored_town_publication::replace(&mut pack, room)?;
    }
    pack.report.maps = pack.data.maps.len();
    pack.report.manifests.push(source.id);
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn connect(maps: &mut BTreeMap<String, MapModule>, link: &Link) -> Result<()> {
    ensure!(link.a.map != link.b.map, "cross-level ladder required");
    let a_id = i16::try_from(maps.get(&link.a.map).context("origin")?.events.warps.len() + 2)?;
    let b_id = i16::try_from(
        maps.get(&link.b.map)
            .context("destination")?
            .events
            .warps
            .len()
            + 2,
    )?;
    for (from, to, target) in [(&link.a, &link.b, b_id), (&link.b, &link.a, a_id)] {
        let constant = maps
            .get(&to.map)
            .context("destination")?
            .attributes
            .map_constant
            .clone()
            .context("constant")?;
        let room = maps.get_mut(&from.map).context("origin")?;
        ensure!(from.active != from.arrival && [from.active,from.arrival].iter().all(|[x,y]|
            *x < room.attributes.width*2 && *y < room.attributes.height*2), "invalid ladder {}",link.id);
        if FLOORS.iter().any(|(m, _)| *m == room.id) {
            let [x, y] = from.active;
            ensure!(
                x % 2 == 1 && y % 2 == 0,
                "native upper-right ladder quadrant"
            );
            let i = usize::from(y / 2) * usize::from(room.attributes.width) + usize::from(x / 2);
            let floor = if room.attributes.tileset_name == "facility" {
                7
            } else {
                13
            };
            ensure!(
                room.blocks[i] == floor,
                "ladder {} overlaps machinery",
                link.id
            );
            room.blocks[i] = if floor == 7 { 4 } else { 2 };
        }
        for [x, y] in [from.active, from.arrival] {
            ensure!(
                !FLOORS.iter().any(|(m, _)| *m == room.id)
                    || !room.events.warps.iter().any(|w| w.x == x && w.y == y),
                "overlapping ladder"
            );
            authoring::warp(room, (x, y), &constant, target);
        }
    }
    Ok(())
}

fn open(door: [u16; 2], facility: bool) -> serde_json::Value {
    c(
        "changeblock",
        &[
            &(door[0] * 2).to_string(),
            &(door[1] * 2).to_string(),
            if facility { "7" } else { "13" },
        ],
    )
}

fn control_program(room: &mut MapModule, control: &Control) -> Result<()> {
    let facility = room.attributes.tileset_name == "facility";
    let [x, y] = control.door;
    ensure!(
        x < room.attributes.width
            && y < room.attributes.height
            && room.blocks[usize::from(y) * usize::from(room.attributes.width) + usize::from(x)]
                == if facility { 0x19 } else { 0x38 },
        "pressure door preimage"
    );
    let [tx, ty] = control.tile;
    ensure!(
        tx < room.attributes.width * 2 && ty + 1 < room.attributes.height * 2,
        "control outside room"
    );
    room.blocks[usize::from(ty / 2) * usize::from(room.attributes.width) + usize::from(tx / 2)] =
        if facility { 0x16 } else { 0x2b };
    let label = format!("MnIronPressure{}", control.id);
    let end = format!("{label}End");
    let done = format!("{label}Done");
    let restore = format!("{label}Restore");
    authoring::text(
        room,
        &format!("{label}Text"),
        &[
            "The pressure line is feeding the drill.",
            "Close its valve?",
        ],
    );
    authoring::text(
        room,
        &format!("{label}DoneText"),
        &["The gauge falls.", "A crosscut shutter slides open."],
    );
    authoring::text(
        room,
        &format!("{label}AfterText"),
        &["This line is isolated. The crosscut is clear."],
    );
    room.scripts.insert(
        label.clone(),
        json!([
            c("checkevent", &[&control.flag]),
            c("iftrue", &[&done]),
            c("opentext", &[]),
            c("writetext", &[&format!("{label}Text")]),
            c("yesorno", &[]),
            c("iffalse", &[&end]),
            c("closetext", &[]),
            open(control.door, facility),
            c("refreshmap", &[]),
            c("opentext", &[]),
            c("writetext", &[&format!("{label}DoneText")]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("setevent", &[&control.flag]),
            c("end", &[])
        ]),
    );
    room.scripts.insert(
        done,
        json!([c("jumptext", &[&format!("{label}AfterText")])]),
    );
    room.scripts
        .insert(end.clone(), json!([c("closetext", &[]), c("end", &[])]));
    room.scripts.insert(
        restore.clone(),
        json!([
            c("checkevent", &[&control.flag]),
            c("iffalse", &[&format!("{label}Return")]),
            open(control.door, facility),
            c("endcallback", &[])
        ]),
    );
    room.scripts
        .insert(format!("{label}Return"), json!([c("endcallback", &[])]));
    a::callback(room, "MAPCALLBACK_TILES", &restore);
    a::read(room, control.tile, &label);
    Ok(())
}

fn gate_program(room: &mut MapModule, source: &Network) -> Result<()> {
    let [x, y] = source.gate.door;
    ensure!(
        x < room.attributes.width
            && y < room.attributes.height
            && room.blocks[usize::from(y) * usize::from(room.attributes.width) + usize::from(x)]
                == 0x38,
        "sump shutter preimage"
    );
    let mut callback = Vec::new();
    for control in &source.controls {
        callback.extend([
            c("checkevent", &[&control.flag]),
            c("iffalse", &["MnIronSumpGateEnd"]),
        ]);
    }
    callback.extend([open(source.gate.door, false), c("endcallback", &[])]);
    room.scripts
        .insert("MnIronSumpGate".into(), json!(callback));
    room.scripts
        .insert("MnIronSumpGateEnd".into(), json!([c("endcallback", &[])]));
    a::callback(room, "MAPCALLBACK_TILES", "MnIronSumpGate");
    authoring::text(
        room,
        "MnIronSumpGateText",
        &[
            "Two live circuits hold this shutter.",
            "The haulage and vent valves are on different levels.",
        ],
    );
    room.scripts.insert(
        "MnIronSumpGateRead".into(),
        json!([c("jumptext", &["MnIronSumpGateText"])]),
    );
    a::read(room, [x * 2, y * 2 + 1], "MnIronSumpGateRead");
    Ok(())
}

fn install_people(
    pack: &mut CompiledGamePack,
    maps: &mut BTreeMap<String, MapModule>,
) -> Result<()> {
    authoring::speech(
        pack,
        maps.get_mut("MnIronHaulage").context("haulage")?,
        "Ruth",
        "SPRITE_POKEFAN_F",
        (5, 15),
        &[
            "RUTH: They ran a second pressure line through the ventilation shaft.",
            "The upper ladder reaches the sump. Come back on the far side to reach the haulage valve.",
            "Close both lines. The pump shutter will release. FINN is still below it.",
        ],
    )?;
    authoring::speech(
        pack,
        maps.get_mut("MnIronVentilation").context("vent")?,
        "Vern",
        "SPRITE_POKEFAN_M",
        (25, 15),
        &[
            "VERN: I used to sign the safety checks.",
            "LODE stopped asking for signatures. ICE stopped us leaving.",
            "The crosscut opens from the other side. The service ladders still connect the east workings.",
        ],
    )?;
    Ok(())
}
