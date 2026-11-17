//! Read the written pack and follow the pure read-only native text branches.
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
