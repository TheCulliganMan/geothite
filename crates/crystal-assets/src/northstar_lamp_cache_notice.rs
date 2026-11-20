//! Explain a full item pocket when examining the revealed Lamp Gallery cache.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, authored_dialogue, authored_town_publication};

const MAP: &str = "MnDriftlessLampGallery";
const CACHE: &str = "MN_LAMP_GALLERY_CACHE";
const FULL: &str = "MnLampCacheBagFull";
const NOTICE: &str = "MnLampCacheBagFullText";
const CLOSE: &str = "MnLampCacheClose";

pub(super) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(MAP).context(MAP)?.clone();
    for label in [FULL, NOTICE] {
        ensure!(
            !room.scripts.contains_key(label) && !pack.data.map_scripts.contains_key(label),
            "occupied Lamp Gallery label {label}"
        );
    }
    let program = room
        .scripts
        .get_mut(CACHE)
        .and_then(|v| v.as_array_mut())
        .context(CACHE)?;
    let grant = command("giveitem", &["RARE_CANDY", "1"]);
    let index = program
        .iter()
        .position(|cmd| cmd == &grant)
        .context("Lamp Gallery Rare Candy")?;
    let failure = program
        .get_mut(index + 1)
        .context("cache capacity branch")?;
    ensure!(
        *failure == command("iffalse", &[CLOSE]),
        "unexpected cache capacity branch"
    );
    *failure = command("iffalse", &[FULL]);
    room.scripts.insert(
        FULL.into(),
        json!([
            command("writetext", &[NOTICE]),
            command("waitbutton", &[]),
            command("sjump", &[CLOSE]),
        ]),
    );
    authored_dialogue::install_canonical(&mut room, NOTICE)?;
    room.script_control_commands = crate::parse_script_control_commands(MAP, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(MAP, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(MAP, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}
