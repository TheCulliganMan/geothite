//! Replace the unusable stone at the Minneapolis item counter with a regional partner's held item.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;

const ROOM: &str = "MnMinneapolisCornerHome";
const ITEM_MENU: &str = "MnCornerItemsData";
const TM_MENU: &str = "MnCornerTmsData";
const OFFER: &str = "MnCornerItemsPrize0";

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let claw = pack.data.items.get("QUICK_CLAW").context("Quick Claw")?;
    ensure!(
        claw.held_effect == "HELD_QUICK_CLAW" && claw.parameter > 0,
        "the prize must have its native held effect"
    );
    let room = pack.data.maps.get_mut(ROOM).context("Game Corner")?;
    let menu_line = |text: &str| json!({"command":"db","args":[format!("\"{text}@\"")]});
    for (label, index, old, new) in [
        (
            ITEM_MENU,
            2,
            menu_line("MOON STONE 300"),
            menu_line("QUICK CLAW 300"),
        ),
        (
            TM_MENU,
            2,
            menu_line("TM25 THUNDER 5500"),
            menu_line("THUNDER 5500"),
        ),
        (
            TM_MENU,
            3,
            menu_line("TM14 BLIZZARD 5500"),
            menu_line("BLIZZARD 5500"),
        ),
        (
            TM_MENU,
            4,
            menu_line("TM38 FIRE 5500"),
            menu_line("FIRE BLAST 5500"),
        ),
        (
            OFFER,
            5,
            json!({"command":"giveitem","args":["MOON_STONE","1"]}),
            json!({"command":"giveitem","args":["QUICK_CLAW","1"]}),
        ),
    ] {
        let script = room.scripts.get_mut(label).context(label)?;
        ensure!(
            pack.data.map_scripts.get(label) == Some(&*script),
            "{label} has mismatched map and global programs"
        );
        let row = script.get_mut(index).context("item counter command")?;
        ensure!(*row == old, "unexpected {label} command at {index}");
        *row = new;
        pack.data
            .map_scripts
            .insert(label.to_owned(), script.clone());
    }
    // The menu reader and the item grant consult separate typed indexes.
    // Rebuild the counter's records with the same parsers as the map compiler.
    let definitions = crate::parse_script_menu_definitions(ROOM, &room.scripts)?;
    let mut vertical = crate::parse_script_vertical_menus(ROOM, &room.scripts, &definitions)?;
    for (label, key) in [(ITEM_MENU, "MnCornerItems:7"), (TM_MENU, "MnCornerTms:7")] {
        let menu = definitions.get(label).context("prize menu data")?;
        room.script_menu_definitions
            .insert(label.to_owned(), menu.clone());
        room.script_vertical_menus.insert(
            key.to_owned(),
            vertical.remove(key).context("prize selection menu")?,
        );
    }
    let new = crate::parse_script_item_grants(ROOM, &room.scripts)?
        .into_iter()
        .find(|grant| grant.source_script == OFFER && grant.command_index == 5)
        .context("compiled Quick Claw prize")?;
    let entry = room
        .script_item_grants
        .iter_mut()
        .find(|grant| grant.source_script == OFFER && grant.command_index == 5)
        .context("original prize grant")?;
    ensure!(
        entry.item_id == "MOON_STONE" && new.item_id == "QUICK_CLAW",
        "unexpected prize grant"
    );
    *entry = new;
    Ok(())
}
