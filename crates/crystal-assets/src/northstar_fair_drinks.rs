//! Tell customers what distinguishes the two Fair drinks before taking their order.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, authored_town_publication};

const FAIR: &str = "MnStateFairgrounds";
const SELLER: &str = "MnFairRootBeerVendor";
const WARNING: &str = "MnFairTwoBeersText";

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(FAIR).context(FAIR)?.clone();
    let program = room
        .scripts
        .get_mut(SELLER)
        .context("Fair drinks counter")?;
    let mart = "MART_MnFairRootBeerVendor";
    ensure!(
        program
            == &json!([
                command("faceplayer", &[]),
                command("opentext", &[]),
                command("pokemart", &["MARTTYPE_STANDARD", mart]),
                command("closetext", &[]),
                command("end", &[])
            ])
            && pack
                .data
                .marts
                .0
                .get(mart)
                .is_some_and(|items| items == &["FAIR_ROOT_BEER", "FAIR_BEER"]),
        "expected the root beer and fair beer seller"
    );
    let program = program.as_array_mut().context("drink seller program")?;
    program.splice(
        2..2,
        [command("writetext", &[WARNING]), command("waitbutton", &[])],
    );
    super::food_neighbors::install_text(&mut room, WARNING)?;
    super::food_neighbors::install_text(&mut room, "MnFairPavilionSignText")?;
    room.script_text_bodies = crate::parse_script_text_bodies(FAIR, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(FAIR, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(FAIR, &room.scripts)?;
    room.script_shop_commands = crate::parse_script_shop_commands(FAIR, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}
