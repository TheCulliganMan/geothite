//! A depot worker gives the next quest after witnessing the courier's violence.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_lair_authoring as a;
use crate::{CompiledGamePack, MapModule};

const MINA: &str = "MN_NSTP_MINA";
const READ: &str = "EVENT_MN_DEEP_BORE_MANIFEST_READ";

pub(crate) fn install(pack: &mut CompiledGamePack, source: &Value) -> Result<()> {
    let mut map = pack
        .data
        .maps
        .get("MnStPaulUnionDepot")
        .context("depot bridge")?
        .clone();
    a::text(
        &mut map,
        source,
        &[
            "MnNSTPMinaInterruptText",
            "MnNSTPMinaReadyText",
            "MnNSTPMinaQuestText",
            "MnNSTPMinaAfterText",
            "MnNSTPPaperFlavorText",
        ],
    )?;
    // The old paper granted this historical event identity. Only MINA's
    // acknowledged conversation grants it now; the identity stays stable for
    // downstream checks, without pretending that a found note drives the plot.
    for rows in map.scripts.values_mut().filter_map(Value::as_array_mut) {
        rows.retain(|row| *row != c("setevent", &[READ]));
    }
    let worker = map
        .objects
        .iter_mut()
        .find(|o| o.object_identifier.as_deref() == Some("MN_SP_DEEP_BORE_MANIFEST"))
        .context("original floor-level manifest anchor")?;
    worker.object_identifier = Some(MINA.into());
    worker.script = MINA.into();
    worker.sprite = "SPRITE_POKEFAN_F".into();
    worker.sprite_has_facings = true;
    worker.spritemovedata = "SPRITEMOVEDATA_STANDING_RIGHT".into();
    map.scripts.insert(
        "MN_SP_DEEP_BORE_MANIFEST".into(),
        json!([c("jumptext", &["MnNSTPPaperFlavorText"])]),
    );
    let courier = map
        .scripts
        .get_mut("MN_ST_PAUL_DEPOT_COURIER")
        .and_then(Value::as_array_mut)
        .context("native courier battle")?;
    let at = courier
        .iter()
        .position(|row| *row == c("opentext", &[]))
        .context("courier scene after existing badge and ELLIS guards")?;
    courier.splice(
        at..at,
        [
            c("faceobject", &[MINA, "MN_ST_PAUL_DEPOT_COURIER"]),
            c("scall", &["MnNSTPMinaObjects"]),
            c("faceobject", &["MN_ST_PAUL_DEPOT_COURIER", "PLAYER"]),
        ],
    );
    let mut objects = a::say("MnNSTPMinaInterruptText");
    objects.push(c("end", &[]));
    put(&mut map, "MnNSTPMinaObjects", objects);
    put(
        &mut map,
        MINA,
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[READ]),
            c("iftrue", &["MnNSTPMinaAfter"]),
            c("checkevent", &["EVENT_MN_COURIER_DEFEATED"]),
            c("iffalse", &["MnNSTPMinaBefore"]),
            c("opentext", &[]),
            c("writetext", &["MnNSTPMinaReadyText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnNSTPMinaDeclined"]),
            c("writetext", &["MnNSTPMinaQuestText"]),
            c("waitbutton", &[]),
            c("setevent", &[READ]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    for (label, text) in [
        ("MnNSTPMinaAfter", "MnNSTPMinaAfterText"),
        ("MnNSTPMinaBefore", "MnNSTPMinaInterruptText"),
    ] {
        put(&mut map, label, vec![c("jumptext", &[text])]);
    }
    put(&mut map, "MnNSTPMinaDeclined", refusal_commands());
    authored_dialogue::install_canonical(&mut map, "MnNSTPMinaDeclinedText")?;
    crate::ely_league_rooms::finish(pack, &mut map)?;
    crate::authored_town_publication::replace(pack, map)
}

/// Refresh the repeatable refusal without granting Mina's acknowledged lead.
pub(crate) fn install_mina_refusal(
    pack: &mut CompiledGamePack,
    pages: &[AuthoredDialoguePage],
) -> Result<()> {
    let mut map = pack
        .data
        .maps
        .get("MnStPaulUnionDepot")
        .context("Mina's depot")?
        .clone();
    let expected = json!(refusal_commands());
    let before = map
        .scripts
        .get("MnNSTPMinaDeclined")
        .context("Mina refusal branch")?;
    ensure!(
        before == &json!([c("closetext", &[]), c("end", &[])]) || before == &expected,
        "Mina refusal staging changed"
    );
    map.scripts.insert("MnNSTPMinaDeclined".into(), expected);
    map.scripts.insert(
        "MnNSTPMinaDeclinedText".into(),
        authored_dialogue::compile(pages)?,
    );
    crate::northstar_story_revision::publish(pack, map)
}

fn refusal_commands() -> Vec<Value> {
    vec![
        c("writetext", &["MnNSTPMinaDeclinedText"]),
        c("waitbutton", &[]),
        c("closetext", &[]),
        c("end", &[]),
    ]
}

fn put(map: &mut MapModule, label: &str, commands: Vec<Value>) {
    map.scripts.insert(label.into(), json!(commands));
}
