//! A hostel roll for the road, a packed lunch in Brainerd, and a walk in Minneapolis.
use std::sync::OnceLock;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;
use crate::{CompiledGamePack, MapModule, northstar_lair_authoring, northstar_story_revision};

const HOSTEL: &str = "MnDuluthHillsideHostel1F";
const COOK: &str = "MN_DL_INTERIOR_HOSTEL_COOK";
const ROLL: &str = "MN_BUTTER_ROLL";
const RECEIPT: &str = "EVENT_MN_DULUTH_HOSTEL_BUTTER_ROLL";

/// Installs the named pages from this revision's regional dialogue.
pub(super) fn install_text(room: &mut MapModule, label: &str) -> Result<()> {
    static STORY: OnceLock<Value> = OnceLock::new();
    let story = STORY.get_or_init(|| {
        serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))
            .expect("canonical Minnesota story must be valid JSON")
    });
    let pages: Vec<AuthoredDialoguePage> =
        serde_json::from_value(story["fair_table_food_dialogue"][&room.id][label].clone())
            .with_context(|| format!("Fair Table dialogue {}/{label}", room.id))?;
    room.scripts
        .insert(label.into(), authored_dialogue::compile(&pages)?);
    Ok(())
}

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    ensure!(
        pack.data.items.contains_key(ROLL),
        "register the hostel's butter roll"
    );
    northstar_lair_authoring::flag(pack, RECEIPT)?;
    for (name, resident, old, new) in [
        (
            "MnBrainerdMart",
            "MN_BRAINERD_MART_CUSTOMER",
            "MN_BRAINERD_MART_CUSTOMERText",
            "MnFairTableMartLunchText",
        ),
        (
            "MnMinneapolisBridgeHome",
            "MN_MP_BRIDGEHOME_HOST",
            "MN_MP_BRIDGEHOME_HOSTText",
            "MnFairTableBridgeWalkText",
        ),
        (
            HOSTEL,
            COOK,
            "MN_DL_INTERIOR_HOSTEL_COOKInteriorText",
            "MnHostelButterRollOfferText",
        ),
    ] {
        let mut room = pack
            .data
            .maps
            .get(name)
            .context("regional neighbor")?
            .clone();
        ensure!(
            room.objects.iter().any(|o| o.script == resident),
            "missing {resident}"
        );
        ensure!(!room.scripts.contains_key(new), "occupied {name}/{new}");
        install_text(&mut room, new)?;
        if name == HOSTEL {
            install_gift(&mut room, old, new)?;
        } else {
            ensure!(
                room.scripts.get(resident) == Some(&json!([c("jumptextfaceplayer", &[old])])),
                "unexpected {resident} greeting"
            );
            room.scripts
                .insert(resident.into(), json!([c("jumptextfaceplayer", &[new])]));
        }
        northstar_story_revision::publish(pack, room)?;
    }
    Ok(())
}

fn install_gift(room: &mut MapModule, old: &str, offer: &str) -> Result<()> {
    ensure!(
        room.objects
            .iter()
            .any(|o| o.script == COOK && (o.x, o.y) == (10, 5))
            && room.scripts.get(COOK)
                == Some(&json!([
                    c("faceplayer", &[]),
                    c("opentext", &[]),
                    c("writetext", &[old]),
                    c("waitbutton", &[]),
                    c("closetext", &[]),
                    c("end", &[])
                ])),
        "expected the Hillside Hostel's cook"
    );
    for label in [
        "MnHostelButterRollFullText",
        "MnHostelButterRollLaterText",
        "MnHostelButterRollRepeatText",
    ] {
        install_text(room, label)?;
    }
    room.scripts.insert(
        COOK.into(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &[RECEIPT]),
            c("iftrue", &["MnHostelRollRepeat"]),
            c("opentext", &[]),
            c("writetext", &[offer]),
            c("yesorno", &[]),
            c("iffalse", &["MnHostelRollLater"]),
            c("verbosegiveitem", &[ROLL, "1"]),
            c("iffalse", &["MnHostelRollFull"]),
            c("setevent", &[RECEIPT]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    room.scripts.insert(
        "MnHostelRollRepeat".into(),
        json!([c("jumptext", &["MnHostelButterRollRepeatText"])]),
    );
    for (label, text) in [
        ("MnHostelRollLater", "MnHostelButterRollLaterText"),
        ("MnHostelRollFull", "MnHostelButterRollFullText"),
    ] {
        room.scripts.insert(
            label.into(),
            json!([
                c("writetext", &[text]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        );
    }
    Ok(())
}
