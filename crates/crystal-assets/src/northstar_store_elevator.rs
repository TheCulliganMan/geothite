//! Install the Minneapolis store lift using the native elevator command and return carpets.
#[path = "northstar_store_counters.rs"]
mod counters;

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::authored_dialogue::install_canonical;
use crate::grand_marais_modpack::authoring::{blank, command, refresh_event_section, warp};
use crate::northstar_lair_authoring::{publish_interior, read};
use crate::northstar_rival_arc::{jump, put};
use crate::{CompiledGamePack, authored_town_publication};

const CAB: &str = "MnMinneapolisStoreElevator";
const CONSTANT: &str = "MN_MINNEAPOLIS_STORE_ELEVATOR";
const PANEL: &str = "MnStoreElevatorPanel";
const LEVELS: &str = "MnStoreElevatorFloors";

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let ground = super::shops::DEPARTMENT;
    let mut cab = blank(pack, "GoldenrodDeptStoreElevator", CAB, CONSTANT)?;
    cab.attributes.fishing_group = pack.data.maps[ground].attributes.fishing_group.clone();
    let first = "MN_MINNEAPOLIS_DEPARTMENT_STORE_1F";
    for x in [1, 2] {
        warp(&mut cab, (x, 3), first, -1);
    }
    read(&mut cab, [3, 0], PANEL);
    let mut rows = vec![command("db", &["5"])];
    let mut names = BTreeMap::new();
    for floor in 1..=5 {
        let id = format!("MnMinneapolisDepartmentStore{floor}F");
        let mut room = pack.data.maps.get(&id).context("store floor")?.clone();
        let constant = room
            .attributes
            .map_constant
            .clone()
            .context("store constant")?;
        names.insert(constant.clone(), id);
        ensure!(
            room.blocks.get(1) == Some(&3)
                && room.events.bg_events.is_empty()
                && !room.events.warps.iter().any(|w| (w.x, w.y) == (2, 0)),
            "unexpected lift bay on floor {floor}"
        );
        // Native MART block 05 carries both the lift doors and their call plate.
        room.blocks[1] = 5;
        if floor == 5 {
            counters::finish_machine_counter(pack, &mut room)?;
        }
        warp(&mut room, (2, 0), CONSTANT, 1);
        let floor_warp = room.events.warps.last().context("new lift entrance")?.index;
        rows.push(command(
            "elevfloor",
            &[
                &format!("FLOOR_{floor}F"),
                &floor_warp.to_string(),
                &constant,
            ],
        ));
        let label = format!("MnStoreFloor{floor}Sign");
        let text = format!("{label}Text");
        read(&mut room, [3, 0], &label);
        if floor < 5 {
            read(&mut room, [14, 0], &label);
        }
        jump(&mut room, &label, &text);
        install_canonical(&mut room, &text)?;
        room.script_text_commands = crate::parse_script_text_commands(&room.id, &room.scripts)?;
        room.script_text_bodies = crate::parse_script_text_bodies(&room.id, &room.scripts)?;
        refresh_event_section(&mut room);
        authored_town_publication::replace(pack, room)?;
    }
    rows.push(command("db", &["-1"]));
    put(&mut cab, LEVELS, rows);
    let program: Vec<Value> = vec![
        command("opentext", &[]),
        command("elevator", &[LEVELS]),
        command("closetext", &[]),
        command("iffalse", &["MnStoreLiftDone"]),
        command("pause", &["5"]),
        command("playsound", &["SFX_ELEVATOR"]),
        command("earthquake", &["60"]),
        command("waitsfx", &[]),
        command("end", &[]),
    ];
    put(&mut cab, PANEL, program);
    put(&mut cab, "MnStoreLiftDone", vec![command("end", &[])]);
    cab.script_elevators = crate::parse_script_elevators(CAB, &cab.scripts, &names)?;
    publish_interior(
        pack,
        cab,
        ground,
        [1, 2],
        "minneapolis-store-elevator".into(),
    )
}
