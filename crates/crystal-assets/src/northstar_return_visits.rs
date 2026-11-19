//! Match the Rochester clinic and Duluth crew book to what the player has done.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_dialogue;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::command;
use crate::northstar_rival_arc;
use crate::{CompiledGamePack, MapModule};

const CLINIC: &str = "MnRochesterPokecenter1F";
const LOOBIT: &str = "MN_ARC_GILLPUP";
const HALL: &str = "MnDuluthUnionHall1F";
const WAREHOUSE: &str = "MnDuluthOreWarehouse2F";
const UPSTAIRS: &str = "MnDuluthUnionHall2F";
const BOOK: &str = "MN_INTERIOR_DETAIL_RESTORED_NAMES";
const NAMES: &str = "EVENT_MN_INTERIOR_SHIFT_NAMES";
const RECEIPT: &str = "EVENT_MN_INTERIOR_UNION_REWARD";
const ENTERED: &str = "EVENT_MN_INTERIOR_UNION_NAMES_ENTERED";

fn clinic(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = northstar_rival_arc::map(pack, CLINIC)?;
    ensure!(
        room.objects.iter().any(|actor| {
            actor.script == LOOBIT
                && actor.sprite == "SPRITE_LOOBIT"
                && actor.event_flag == "EVENT_MN_SURVEY_RESTORED"
        }) && pack.data.pokemon.contains_key("LOOBIT"),
        "expected the clinic Loobit to leave on restoration"
    );
    let script = room.scripts.get_mut(LOOBIT).context("clinic Loobit")?;
    ensure!(
        script[0] == command("cry", &["GILLPUP"])
            && script[1] == command("waitsfx", &[])
            && script[2] == command("jumptext", &["MnOverhaulGillpupText"]),
        "unexpected clinic Loobit interaction"
    );
    script[0] = command("cry", &["LOOBIT"]);
    authored_dialogue::install_canonical(&mut room, "MnArcHabitatAfterText")?;
    room.script_audio_commands = crate::parse_script_audio_commands(CLINIC, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(CLINIC, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}

fn add_reply(room: &mut MapModule, branch: &str, page: &str) -> Result<()> {
    ensure!(
        !room.scripts.contains_key(page) && !room.scripts.contains_key(branch),
        "occupied crew book reply {branch}"
    );
    authored_dialogue::install_canonical(room, page)?;
    room.scripts.insert(
        branch.into(),
        json!([
            command("writetext", &[page]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("end", &[]),
        ]),
    );
    Ok(())
}

fn first_hall_visit(room: &mut MapModule, actor: &str) -> Result<()> {
    let original = format!("{actor}InteriorText");
    let first = &mut room.scripts.get_mut(actor).context("Union Hall host")?[4];
    ensure!(
        *first == command("writetext", &[&original]),
        "unexpected Union Hall welcome"
    );
    // After the subroutine returns, the existing host script waits for the
    // reader and sets WELCOME exactly where it always has.
    *first = command("scall", &["MnUnionFirstVisit"]);
    ensure!(
        !room.scripts.contains_key("MnUnionListInHandText"),
        "occupied Union Hall introduction"
    );
    authored_dialogue::install_canonical(room, "MnUnionListInHandText")?;
    for (branch, body) in [
        (
            "MnUnionFirstVisit",
            vec![
                command("checkevent", &[RECEIPT]),
                command("iftrue", &["MnUnionFirstAfter"]),
                command("checkevent", &[ENTERED]),
                command("iftrue", &["MnUnionFirstAfter"]),
                command("checkevent", &[NAMES]),
                command("iftrue", &["MnUnionFirstWithList"]),
                command("writetext", &[&original]),
                command("end", &[]),
            ],
        ),
        (
            "MnUnionFirstWithList",
            vec![
                command("writetext", &["MnUnionListInHandText"]),
                command("end", &[]),
            ],
        ),
        (
            "MnUnionFirstAfter",
            vec![
                command("writetext", &["MnUnionBookAfterText"]),
                command("end", &[]),
            ],
        ),
    ] {
        ensure!(
            room.scripts
                .insert(branch.into(), Value::Array(body))
                .is_none(),
            "occupied first-visit branch {branch}"
        );
    }
    Ok(())
}

fn crew_book(pack: &mut CompiledGamePack, map: &str, actor: &str, flag: &str) -> Result<()> {
    let mut room = northstar_rival_arc::map(pack, map)?;
    let repeat = format!("{actor}InteriorRepeat");
    let return_branch = format!("{actor}ReturnVisit");
    let host = room
        .scripts
        .get_mut(actor)
        .with_context(|| format!("missing {actor}"))?;
    ensure!(
        host[2] == command("checkevent", &[flag]) && host[3] == command("iftrue", &[&repeat]),
        "unexpected original crew book greeting {map}/{actor}"
    );
    // Redirect the existing branch without shifting saved instruction positions in either visit.
    host[3] = command("iftrue", &[&return_branch]);
    let (done, text) = if map == HALL {
        ("MnUnionBookAfter", "MnUnionBookAfterText")
    } else {
        ("MnWarehouseBookAfter", "MnWarehouseBookAfterText")
    };
    add_reply(&mut room, done, text)?;
    let mut replies = vec![
        command("checkevent", &[RECEIPT]),
        command("iftrue", &[done]),
        command("checkevent", &[ENTERED]),
        command("iftrue", &[done]),
    ];
    if map == HALL {
        first_hall_visit(&mut room, actor)?;
        add_reply(
            &mut room,
            "MnUnionRecorderUpstairs",
            "MnUnionRecorderUpstairsText",
        )?;
        replies.extend([
            command("checkevent", &[NAMES]),
            command("iftrue", &["MnUnionRecorderUpstairs"]),
        ]);
    } else {
        // An old completed book remains authoritative even if the copied-list
        // prerequisite is absent in an imported save. Leave the last native
        // check's boolean in place for the caller's existing iftrue.
        room.scripts.get_mut(actor).context("warehouse clerk")?[2] =
            command("scall", &["MnWarehouseListKnown"]);
        for (branch, body) in [
            (
                "MnWarehouseListKnown",
                vec![
                    command("checkevent", &[RECEIPT]),
                    command("iftrue", &["MnWarehouseKnowEnd"]),
                    command("checkevent", &[ENTERED]),
                    command("iftrue", &["MnWarehouseKnowEnd"]),
                    command("checkevent", &[NAMES]),
                    command("end", &[]),
                ],
            ),
            ("MnWarehouseKnowEnd", vec![command("end", &[])]),
        ] {
            ensure!(
                room.scripts
                    .insert(branch.into(), Value::Array(body))
                    .is_none(),
                "occupied list reminder {branch}"
            );
        }
        let original = format!("{repeat}Text");
        ensure!(
            room.scripts.get(&original).and_then(|rows| rows.get(0))
                == Some(&command("text", &["\"Have you still got\""])),
            "unexpected warehouse reminder"
        );
        authored_dialogue::install_canonical(&mut room, &original)?;
        authored_dialogue::install_canonical(&mut room, &format!("{actor}InteriorText"))?;
    }
    replies.push(command("sjump", &[&repeat]));
    ensure!(
        room.scripts
            .insert(return_branch, Value::Array(replies))
            .is_none(),
        "occupied crew book return branch"
    );
    publish_book_room(pack, room)
}

fn publish_book_room(pack: &mut CompiledGamePack, mut room: MapModule) -> Result<()> {
    let map = &room.id;
    room.script_flag_commands = crate::parse_script_flag_commands(map, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(map, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(map, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(map, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}

fn record_names(pack: &mut CompiledGamePack) -> Result<()> {
    const RECORDER: &str = "MN_DULUTH_X_UNIONHALL_F2_HOST";
    const TRANSCRIBE: &str = "MN_DULUTH_X_UNIONHALL_F2_HOSTInteriorText";
    const BLANK: &str = "MN_INTERIOR_DETAIL_RESTORED_NAMESText";
    let mut room = northstar_rival_arc::map(pack, UPSTAIRS)?;
    ensure!(
        room.events
            .bg_events
            .iter()
            .any(|read| read.script == BOOK && [read.x, read.y] == [9, 7])
            && room.scripts.get(BOOK) == Some(&json!([command("jumptext", &[BLANK])])),
        "unexpected Union Hall crew book"
    );
    let host = room
        .scripts
        .get_mut(RECORDER)
        .context("Union Hall recorder")?;
    ensure!(
        host[8] == command("writetext", &[TRANSCRIBE])
            && host[9] == command("waitbutton", &[])
            && host[10] == command("giveitem", &["ETHER", "1"])
            && host[12] == command("setevent", &[RECEIPT]),
        "unexpected Union Hall reward"
    );
    // Returning from the subroutine resumes the original giveitem and full-bag branch.
    host[8] = command("scall", &["MnUnionWriteNames"]);
    for page in [
        "MnUnionEnteredBookText",
        "MnUnionFirstEtherText",
        "MnUnionSavedEtherText",
    ] {
        ensure!(
            !room.scripts.contains_key(page),
            "occupied crew book page {page}"
        );
        authored_dialogue::install_canonical(&mut room, page)?;
    }
    authored_dialogue::install_canonical(&mut room, TRANSCRIBE)?;
    room.scripts.insert(
        BOOK.into(),
        json!([
            command("checkevent", &[RECEIPT]),
            command("iftrue", &["MnUnionReadNames"]),
            command("checkevent", &[ENTERED]),
            command("iftrue", &["MnUnionReadNames"]),
            command("jumptext", &[BLANK]),
        ]),
    );
    for (branch, body) in [
        (
            "MnUnionReadNames",
            vec![command("jumptext", &["MnUnionEnteredBookText"])],
        ),
        (
            "MnUnionWriteNames",
            vec![
                command("checkevent", &[ENTERED]),
                command("iftrue", &["MnUnionHoldEther"]),
                command("writetext", &[TRANSCRIBE]),
                command("waitbutton", &[]),
                command("setevent", &[ENTERED]),
                command("writetext", &["MnUnionFirstEtherText"]),
                command("end", &[]),
            ],
        ),
        (
            "MnUnionHoldEther",
            vec![
                command("writetext", &["MnUnionSavedEtherText"]),
                command("end", &[]),
            ],
        ),
    ] {
        ensure!(
            room.scripts
                .insert(branch.into(), Value::Array(body))
                .is_none(),
            "occupied recorder branch {branch}"
        );
    }
    publish_book_room(pack, room)
}

/// Matches the clinic animal and the names, pages and reward of the Duluth crew book.
///
/// Returns an error if the original actors, calls or quest branches have changed.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    crate::northstar_lair_authoring::flag(pack, ENTERED)?;
    clinic(pack)?;
    record_names(pack)?;
    crew_book(
        pack,
        HALL,
        "MN_DULUTH_X_UNIONHALL_F1_HOST",
        "EVENT_MN_INTERIOR_UNION_WELCOME",
    )?;
    crew_book(pack, WAREHOUSE, "MN_DULUTH_X_OREWAREHOUSE_F2_HOST", NAMES)
}

#[cfg(test)]
#[path = "northstar_crew_book_tests.rs"]
mod tests;
