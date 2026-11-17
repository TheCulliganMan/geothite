//! Check the published native scripts and static Gym floorplans.
use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::MapModule;

fn displayed_label<'a>(room: &'a MapModule, entry: &'a str, flags: &[&str]) -> Result<&'a str> {
    let mut label = entry;
    let mut visited = BTreeSet::new();
    while visited.insert(label) {
        let mut set = false;
        let commands = room
            .scripts
            .get(label)
            .and_then(|p| p.as_array())
            .context("read program")?;
        let mut next = None;
        for command in commands {
            let argument = command["args"][0].as_str().context("native read operand")?;
            match command["command"].as_str() {
                Some("checkevent") => set = flags.contains(&argument),
                Some("iftrue") if set => {
                    next = Some(argument);
                    break;
                }
                Some("iftrue") => {}
                Some("jumptext") => {
                    anyhow::ensure!(
                        room.script_text_bodies.contains_key(argument),
                        "unpublished text"
                    );
                    return Ok(argument);
                }
                _ => anyhow::bail!("unexpected state mutation in a mine reading"),
            }
        }
        label = next.context("branch did not display any reading")?;
    }
    anyhow::bail!("cyclic mine reading")
}

#[test]
#[ignore = "requires CRYSTAL_IRON_TARGET; reads compiled maps only"]
fn brainerd_has_an_open_wading_lane_and_st_cloud_has_working_menu_options() -> Result<()> {
    let path = PathBuf::from(std::env::var_os("CRYSTAL_IRON_TARGET").context("target pack")?);
    let pack = crate::read_verified_compiled_game_pack(path)?;
    let room = &pack.data.maps["MnBrainerdGym"];
    let arrival = room.events.warps.get(2).context("Brainerd arrival")?;
    let leader = room.objects.iter().find(|object| {
        object.object_identifier.as_deref() == Some("MnBrainerdGymLeader")
    }).context("Sivren")?;
    assert_eq!(arrival.x, leader.x, "Sivren should be straight ahead from the door");
    assert!(room.events.bg_events.is_empty() && room.events.coord_events.is_empty());
    assert!(room.script_block_changes.is_empty());
    assert!(!room.scripts.keys().any(|label| label.contains("Routing")));
    let surface = crate::northstar_gym_surface::Surface::new(&pack, room)?;
    for y in leader.y + 1..=arrival.y {
        let at = [arrival.x, y];
        assert_eq!(surface.permission(at)?, crystal_core::world::collision::permissions::FLOOR);
        assert!(!surface.occupied(at)?, "an actor blocks the straight approach at {at:?}");
    }
    let airworks = &pack.data.maps["MnStCloudGym"];
    assert_eq!(airworks.script_vertical_menus.len(), 4);
    for menu in airworks.script_vertical_menus.values() {
        assert!(airworks.script_menu_definitions.contains_key(&menu.header_label));
        assert_eq!(menu.options.len(), 2);
        assert_ne!(menu.options[0], menu.options[1]);
        assert_eq!(
            airworks.scripts[&menu.source_script][menu.verticalmenu_command_index]["command"],
            "verticalmenu"
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires CRYSTAL_IRON_TARGET; reads compiled map data only"]
fn native_mine_readings_follow_all_pressure_and_shutdown_states() -> Result<()> {
    let path = PathBuf::from(std::env::var_os("CRYSTAL_IRON_TARGET").context("target pack")?);
    let pack = crate::read_verified_compiled_game_pack(path)?;
    let sump = &pack.data.maps["MnIronSump"];
    let haul = "EVENT_MN_IRON_HAUL_PRESSURE_OFF";
    let vent = "EVENT_MN_IRON_VENT_PRESSURE_OFF";
    for (flags, label) in [
        (vec![], "MnIronSumpGateText"),
        (vec![haul], "MnIronSumpHaulOnlyText"),
        (vec![vent], "MnIronSumpVentOnlyText"),
        (vec![haul, vent], "MnIronSumpOpenText"),
    ] {
        assert_eq!(displayed_label(sump, "MnIronSumpGateRead", &flags)?, label);
    }
    for (room, entry, running, stopped) in [
        (
            "MnIronPumpGallery",
            "MnNDrillGalleryAfter",
            "MnNDrillGalleryAfterText",
            "MnNDrillGalleryStoppedText",
        ),
        (
            "MnIronDrillCore",
            "MN_NDRILL_ENGINEER",
            "MnNDrillForemanText",
            "MnNDrillForemanStoppedText",
        ),
    ] {
        assert_eq!(displayed_label(&pack.data.maps[room], entry, &[])?, running);
        assert_eq!(
            displayed_label(
                &pack.data.maps[room],
                entry,
                &[crate::northstar_drill_lair::OFF]
            )?,
            stopped
        );
    }
    Ok(())
}
