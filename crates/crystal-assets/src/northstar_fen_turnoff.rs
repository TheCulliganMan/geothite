//! Mark the unsigned three-way junction between Bemidji, Route 17 and the Layered Fen.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_dialogue::install_canonical;
use crate::authored_town_geometry::kind;
use crate::authored_town_publication::replace;
use crate::grand_marais_modpack::authoring::{command, refresh_event_section};
use crate::northstar_lair_authoring::read;

const WOODS: &str = "MnBemidjiNorthwoods";
const SIGN: &str = "MnFenTurnoff";
const TEXT: &str = "MnFenTurnoffText";

/// Uses the woods' existing snowy sign metatile at the northwest verge of the T.
///
/// Returns an error if the original junction, sign collision or native event binding has changed.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut woods = pack
        .data
        .maps
        .get(WOODS)
        .context("Bemidji Northwoods")?
        .clone();
    let a = &woods.attributes;
    ensure!(
        a.tileset_name == "mn_winter_bemidji_northwoods"
            && a.width == 8
            && a.height == 14
            && a.connections
                .iter()
                .any(|c| c.direction == "west" && c.target_map == "MnLayeredFen"),
        "unexpected Fen junction"
    );
    let index = 7 * usize::from(a.width) + 1;
    ensure!(
        woods.blocks[index] == 0x05
            && kind(pack, &woods, [3, 16])? == "FLOOR"
            && woods.events.bg_events.is_empty()
            && !woods.scripts.contains_key(SIGN)
            && !woods.scripts.contains_key(TEXT),
        "unexpected snowy shoulder or trail sign"
    );
    woods.blocks[index] = 0x78;
    ensure!(
        kind(pack, &woods, [3, 15])? == "WALL",
        "wrong quarter of the winter sign"
    );
    read(&mut woods, [3, 15], SIGN);
    woods
        .scripts
        .insert(SIGN.into(), vec![command("jumptext", &[TEXT])].into());
    install_canonical(&mut woods, TEXT)?;
    woods.script_text_commands = crate::parse_script_text_commands(WOODS, &woods.scripts)?;
    woods.script_text_bodies = crate::parse_script_text_bodies(WOODS, &woods.scripts)?;
    refresh_event_section(&mut woods);
    replace(pack, woods)
}
