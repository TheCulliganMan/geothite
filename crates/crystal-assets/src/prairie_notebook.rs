//! Optional notebook clue: completion follows acknowledgment, never a gift.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_town_geometry::{kind, reachable};
use crate::grand_marais_modpack::authoring;
use crate::prairie_story::Notebook;
use crate::{CompiledGamePack, MapModule};

pub(super) fn install(
    pack: &mut CompiledGamePack,
    map: &mut MapModule,
    note: &Notebook,
    story: &Value,
) -> Result<()> {
    ensure!(
        note.map == "MnMankato"
            && map.id == note.map
            && note.id == "MN_MANKATO_NOTEBOOK"
            && note.sprite == "SPRITE_PAPER"
            && note.tile == [26, 25]
            && note.approach == [26, 26]
            && note.milestone == "MS02"
            && note.hint_actor == "MN_MK_GARDENER",
        "canonical prairie notebook anchor required"
    );
    let milestone = story["milestones"]
        .as_array()
        .context("milestones")?
        .iter()
        .find(|m| m["id"] == note.milestone)
        .context("MS02")?;
    const FLAG: &str = "EVENT_MN_PRAIRIE_NOTEBOOK";
    ensure!(
        milestone["location"] == note.map
            && milestone["requires"] == json!(["BADGE_1"])
            && milestone["sets"] == json!([FLAG]),
        "canonical notebook prerequisite and allocated completion required"
    );
    let globals = &mut pack.data.story_event_script_constants.global;
    ensure!(
        !globals.contains_key(FLAG),
        "notebook event already allocated in this generation"
    );
    let next = globals
        .iter()
        .filter(|(name, _)| name.starts_with("EVENT_"))
        .map(|(_, number)| *number)
        .max()
        .context("native event catalog")?
        .checked_add(1)
        .context("notebook event overflow")?;
    u16::try_from(next).context("notebook event outside native range")?;
    globals.insert(FLAG.into(), next);
    let pages: Vec<Vec<String>> = serde_json::from_value(milestone["dialogue_pages"].clone())?;
    ensure!(
        pages
            == [
                vec![
                    "Different lakes.".to_owned(),
                    "The same numbers.".to_owned()
                ],
                vec!["Who copied these?".to_owned()]
            ],
        "canonical two-page notebook clue required"
    );
    ensure!(
        kind(pack, map, note.tile)? == "FLOOR"
            && kind(pack, map, note.approach)? == "FLOOR"
            && !map.objects.iter().any(|o| [o.x, o.y] == note.tile)
            && !map.scripts.contains_key(&note.id),
        "notebook anchor occupied or blocked"
    );
    let hint = format!("{}Text", note.hint_actor);
    ensure!(
        map.scripts.contains_key(&hint),
        "garden notebook hint actor missing"
    );
    authoring::text(
        map,
        &hint,
        &[
            "Deep roots hold",
            "this prairie soil.",
            "Someone left a",
            "survey notebook.",
        ],
    );
    let c = authoring::command;
    authoring::text(
        map,
        "MnMKNotebookText",
        &pages
            .iter()
            .flatten()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    authoring::text(
        map,
        "MnMKNotebookDeniedText",
        &["Survey notes.", "Start with VAREL", "in ROCHESTER."],
    );
    authoring::text(
        map,
        "MnMKNotebookAfterText",
        &[
            "The same numbers",
            "for distant lakes.",
            "The MANKATO copy",
            "stays here.",
        ],
    );
    map.scripts.insert(
        note.id.clone(),
        json!([
            c("checkflag", &["ENGINE_ZEPHYRBADGE"]),
            c("iffalse", &["MnMKNotebookDenied"]),
            c("checkevent", &[FLAG]),
            c("iftrue", &["MnMKNotebookAfter"]),
            c("opentext", &[]),
            c("writetext", &["MnMKNotebookText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("setevent", &[FLAG]),
            c("end", &[])
        ]),
    );
    for (label, text) in [
        ("MnMKNotebookDenied", "MnMKNotebookDeniedText"),
        ("MnMKNotebookAfter", "MnMKNotebookAfterText"),
    ] {
        map.scripts
            .insert(label.into(), json!([c("jumptext", &[text])]));
    }
    authoring::npc(
        pack,
        map,
        &note.id,
        &note.sprite,
        i16::try_from(note.tile[0])?,
        i16::try_from(note.tile[1])?,
    )?;
    let actor = map.objects.last_mut().context("notebook object")?;
    actor.object_identifier = Some(note.id.clone());
    actor.spritemovedata = "SPRITEMOVEDATA_STILL".into();
    ensure!(
        reachable(pack, map, [40, 18], false)?.contains(&note.approach),
        "notebook cannot be approached from the public road"
    );
    Ok(())
}
