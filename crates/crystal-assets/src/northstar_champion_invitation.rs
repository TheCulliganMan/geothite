//! Casey invites the returning Champion to the existing rematch.

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_dialogue;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::command;

const ROOM: &str = "MnLeagueChampion";
const INVITATION: &str = "MnLeagueChampionRematchText";

/// Installs the canonical invitation at the Champion's returning-player branch.
///
/// Returns an error if the existing invitation or its native text cannot be found.
pub(crate) fn revise_champion_invitation(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(ROOM).context("Champion room")?.clone();
    let lines = room
        .scripts
        .get(INVITATION)
        .and_then(|text| text.as_array())
        .context("Champion rematch invitation")?;
    ensure!(
        lines.get(4) == Some(&command("para", &["\"I have a new plan.\""])),
        "unexpected parent Champion invitation"
    );
    authored_dialogue::install_canonical(&mut room, INVITATION)?;
    room.script_text_bodies = crate::parse_script_text_bodies(ROOM, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}
