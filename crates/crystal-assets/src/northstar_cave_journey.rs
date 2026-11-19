//! Distinguish the Driftless shell bed from its eastern field kit.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::command;

const GALLERY: &str = "MnDriftlessLampGallery";
const FOSSIL_TEXT: &str = "MnLampFossilsText";
const KIT_TEXT: &str = "MnLampEastShelfText";

#[derive(Deserialize)]
struct GalleryDesign {
    read_pages: Vec<AuthoredDialoguePage>,
}

/// Gives the lit west fossil its proper description. The field kit on the east
/// shelf is mentioned only while the player can still collect it.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get(GALLERY)
        .context("Driftless lamp gallery")?
        .clone();
    ensure!(
        room.scripts
            .get(FOSSIL_TEXT)
            .and_then(Value::as_array)
            .and_then(|rows| rows.first())
            == Some(&command("text", &["\"By the east shelf,\""])),
        "expected the earlier east-shelf fossil description"
    );
    let read = room
        .scripts
        .get_mut("MnLampFossils")
        .and_then(Value::as_array_mut)
        .context("Driftless fossil read")?;
    ensure!(
        read.ends_with(&[
            command("checkevent", &["EVENT_MN_LAMP_GALLERY_CACHE"]),
            command("iftrue", &["MnLampReadDone"]),
            command("appear", &["MN_LAMP_GALLERY_CACHE"]),
            command("sjump", &["MnLampReadDone"]),
        ]),
        "expected the conditional field-kit appearance"
    );
    let after_appearance = read.len() - 1;
    read.splice(
        after_appearance..after_appearance,
        [
            command("writetext", &[KIT_TEXT]),
            command("waitbutton", &[]),
        ],
    );
    authored_dialogue::install_canonical(&mut room, FOSSIL_TEXT)?;
    let design: GalleryDesign = serde_json::from_str(include_str!(
        "../../../planning/minnesota/caves/driftless-fossil-gallery-v01/gallery.json"
    ))?;
    let kit_page = design.read_pages.last().context("gallery field-kit clue")?;
    ensure!(
        !pack.data.map_scripts.contains_key(KIT_TEXT) && !room.scripts.contains_key(KIT_TEXT),
        "occupied field-kit clue {KIT_TEXT}"
    );
    room.scripts.insert(
        KIT_TEXT.into(),
        authored_dialogue::compile(std::slice::from_ref(kit_page))?,
    );
    room.script_control_commands = crate::parse_script_control_commands(GALLERY, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(GALLERY, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(GALLERY, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}
