//! The survey crew lends a hand at the first breakable passage on Route 10.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;
use crate::{CompiledGamePack, northstar_lair_authoring, northstar_story_revision};

const ITEM: &str = "TM_ROCK_SMASH";
const RECEIPT: &str = "EVENT_MN_R10_RECEIVED_ROCK_SMASH";
const GUIDE: &str = "MN_R10_GALLERY_READER";
const DIRECTIONS: &str = "MN_R10_GALLERY_READERText";

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let item = pack.data.items.get(ITEM).context("native Rock Smash TM")?;
    ensure!(
        item.tmhm_index == Some(8) && item.price > 0,
        "expected saleable TM08"
    );
    let mut room = pack
        .data
        .maps
        .get("MnR10Cave1F")
        .context("survey gallery")?
        .clone();
    ensure!(
        room.scripts.get(GUIDE) == Some(&json!([c("jumptextfaceplayer", &[DIRECTIONS])]))
            && room
                .objects
                .iter()
                .any(|o| o.script == GUIDE && [o.x, o.y] == [9, 5]),
        "expected the gallery reader at the pillar"
    );
    let source: Value =
        serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))?;
    let texts: BTreeMap<String, Vec<AuthoredDialoguePage>> =
        serde_json::from_value(source["wayfinding_dialogue"]["MnR10Cave1F"].clone())?;
    for (label, pages) in texts {
        let previous = room
            .scripts
            .insert(label.clone(), authored_dialogue::compile(&pages)?);
        ensure!(
            previous.is_some() == (label == DIRECTIONS),
            "unexpected gallery text preimage for {label}"
        );
    }
    northstar_lair_authoring::flag(pack, RECEIPT)?;
    room.scripts.insert(
        GUIDE.into(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &[RECEIPT]),
            c("iftrue", &["MnGalleryDirections"]),
            c("opentext", &[]),
            c("writetext", &[DIRECTIONS]),
            c("promptbutton", &[]),
            c("writetext", &["MnGalleryRockSmashText"]),
            c("promptbutton", &[]),
            c("verbosegiveitem", &[ITEM, "1"]),
            c("iffalse", &["MnGalleryKeepTM"]),
            c("setevent", &[RECEIPT]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    room.scripts.insert(
        "MnGalleryDirections".into(),
        json!([c("jumptext", &[DIRECTIONS])]),
    );
    room.scripts.insert(
        "MnGalleryKeepTM".into(),
        json!([
            c("writetext", &["MnGalleryFullPackText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    northstar_story_revision::publish(pack, room)?;

    // Later teams can learn the move after spending the guide's single-use TM.
    let stock = pack
        .data
        .marts
        .0
        .get_mut("MART_MN_MP_DEPT_5_CLERK")
        .context("Minneapolis fifth-floor TM counter")?;
    ensure!(
        !stock.iter().any(|id| id == ITEM),
        "the counter already stocks TM08"
    );
    stock.push(ITEM.into());
    Ok(())
}
