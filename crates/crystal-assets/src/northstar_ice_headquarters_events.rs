//! Employees, credentials, optional recovery and native sight trainers.
use std::collections::BTreeMap;

use anyhow::{Context, Result};
use serde_json::json;

use crate::grand_marais_modpack::authoring;
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_ice_headquarters::KEY;
use crate::northstar_lair_authoring as a;
use crate::{CompiledGamePack, MapModule};

pub(super) fn install(
    pack: &mut CompiledGamePack,
    maps: &mut BTreeMap<String, MapModule>,
) -> Result<()> {
    for (name, id, at, party, seen) in [
        (
            "MnIceOperations",
            "MN_ICE_HQ_OPERATIONS_GUARD",
            [11, 15],
            [("CINDORE", 19), ("MIDGEBIT", 20)],
            "A badge means you answer to me.",
        ),
        (
            "MnIceRecords",
            "MN_ICE_HQ_RECORDS_GUARD",
            [11, 13],
            [("CARTICK", 20), ("LEECHIMP", 20)],
            "Those names stay in this building.",
        ),
        (
            "MnIceMaintenance",
            "MN_ICE_HQ_MAINTENANCE_GUARD",
            [9, 13],
            [("SLUDGESKI", 19), ("MIDGEBIT", 20)],
            "The stairs are not an exit for you.",
        ),
        (
            "MnIceTransfer",
            "MN_ICE_HQ_TRANSFER_GUARD",
            [11, 15],
            [("LEECHIMP", 21), ("WRIGGLET", 21)],
            "Nobody calls home from here.",
        ),
    ] {
        let flag = format!("EVENT_{id}_DEFEATED");
        a::flag(pack, &flag)?;
        a::trainer(pack, id, "GRUNTM", "ICE", &party)?;
        let room = maps.get_mut(name).context("guard floor")?;
        let seen_label = format!("{id}SeenText");
        let won = format!("{id}WonText");
        let lost = format!("{id}LostText");
        let after = format!("{id}After");
        authoring::text(room, &seen_label, &[seen]);
        authoring::text(room, &won, &["Keep your voice down."]);
        authoring::text(room, &lost, &["Now turn around."]);
        room.scripts.insert(
            id.into(),
            json!([c(
                "trainer",
                &["GRUNTM", id, &flag, &seen_label, &won, &lost, &after]
            )]),
        );
        room.scripts.insert(
            after,
            json!([c("endifjustbattled", &[]), c("jumptextfaceplayer", &[&won])]),
        );
        a::actor(pack, room, id, "SPRITE_OFFICER", at)?;
        let actor = room.objects.last_mut().context("native trainer")?;
        actor.object_type = "OBJECTTYPE_TRAINER".into();
        actor.spritemovedata = "SPRITEMOVEDATA_STANDING_UP".into();
        actor.radius = 3;
    }
    let ops = maps.get_mut("MnIceOperations").context("operations")?;
    authoring::speech(
        pack,
        ops,
        "Worker",
        "SPRITE_POKEFAN_M",
        (5, 15),
        &[
            "IRA: They sealed the offices from the stairs.",
            "The transfer pads still reach inside. I kept them live.",
            "Find ADA. She has a card. I saw them take her clerk.",
        ],
    )?;
    let workshop = maps.get_mut("MnIceMaintenance").context("maintenance")?;
    authoring::speech(
        pack,
        workshop,
        "RoutingWorker",
        "SPRITE_SCIENTIST",
        (25, 17),
        &[
            "MIRA: This is the service side. No stairs here.",
            "The other pad returns to the public records hall.",
            "They cannot seal every way through without stopping their own shipments.",
        ],
    )?;
    supplies(pack, workshop)?;
    let records = maps.get_mut("MnIceRecords").context("records")?;
    ada(pack, records)?;
    authoring::text(
        records,
        "MnIceHQRestText",
        &["NORA: Keep the door shut. I can treat your Pokemon here."],
    );
    authoring::text(
        records,
        "MnIceHQRestDoneText",
        &["Take a breath. Then get them out."],
    );
    a::actor(pack, records, "MN_ICE_HQ_NORA", "SPRITE_NURSE", [25, 17])?;
    records.scripts.insert(
        "MN_ICE_HQ_NORA".into(),
        json!([
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("writetext", &["MnIceHQRestText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnIceHQCardEnd"]),
            c("closetext", &[]),
            c("special", &["HealParty"]),
            c("jumptext", &["MnIceHQRestDoneText"])
        ]),
    );
    let transfer = maps.get_mut("MnIceTransfer").context("transfer")?;
    authoring::text(
        transfer,
        "MnIceHQTessText",
        &[
            "TESS: They took me outside my shop. No warrant. No call.",
            "I asked where ELLIS was. That was enough.",
            "I am leaving by the loading stair. Tell JO they are still here.",
        ],
    );
    authoring::text(
        transfer,
        "MnIceHQTessAfterText",
        &["TESS: I will wait until you clear the stair."],
    );
    a::actor(pack, transfer, "MN_ICE_HQ_TESS", "SPRITE_TEACHER", [25, 5])?;
    let mut tess = vec![
        c("faceplayer", &[]),
        c("checkevent", &["EVENT_MN_ICE_HQ_TESS_HELPED"]),
        c("iftrue", &["MnIceHQTessAfter"]),
    ];
    tess.extend(a::say("MnIceHQTessText"));
    tess.extend([
        c("setevent", &["EVENT_MN_ICE_HQ_TESS_HELPED"]),
        c("end", &[]),
    ]);
    transfer
        .scripts
        .insert("MN_ICE_HQ_TESS".into(), json!(tess));
    transfer.scripts.insert(
        "MnIceHQTessAfter".into(),
        json!([c("jumptext", &["MnIceHQTessAfterText"])]),
    );
    Ok(())
}

fn ada(pack: &CompiledGamePack, room: &mut MapModule) -> Result<()> {
    for (label, lines) in [
        (
            "MnIceHQCardOfferText",
            vec![
                "ADA: They made me sort the names. No charges. Just names.",
                "I kept a card. Will you get ELLIS out?",
            ],
        ),
        (
            "MnIceHQCardGivenText",
            vec![
                "ADA gives you the CARD KEY.",
                "The blue shutters use it. Get back through the other pad.",
            ],
        ),
        (
            "MnIceHQCardFullText",
            vec!["There is no room for the card. Make space. I will wait."],
        ),
        (
            "MnIceHQCardAfterText",
            vec!["ADA: Open the staff doors too. Some of us will help."],
        ),
    ] {
        authoring::text(room, label, &lines);
    }
    a::actor(pack, room, "MN_ICE_HQ_ADA", "SPRITE_TEACHER", [25, 5])?;
    room.scripts.insert(
        "MN_ICE_HQ_ADA".into(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &[KEY]),
            c("iftrue", &["MnIceHQCardAfter"]),
            c("opentext", &[]),
            c("writetext", &["MnIceHQCardOfferText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnIceHQCardEnd"]),
            c("giveitem", &["CARD_KEY", "1"]),
            c("iffalse", &["MnIceHQCardFull"]),
            c("setevent", &[KEY]),
            c("writetext", &["MnIceHQCardGivenText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    room.scripts.insert(
        "MnIceHQCardEnd".into(),
        json!([c("closetext", &[]), c("end", &[])]),
    );
    room.scripts.insert(
        "MnIceHQCardFull".into(),
        json!([
            c("writetext", &["MnIceHQCardFullText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    room.scripts.insert(
        "MnIceHQCardAfter".into(),
        json!([c("jumptext", &["MnIceHQCardAfterText"])]),
    );
    Ok(())
}

fn supplies(pack: &CompiledGamePack, room: &mut MapModule) -> Result<()> {
    authoring::text(
        room,
        "MnIceHQSuppliesText",
        &["OLLIE: You found the workshop. Take these for the way back."],
    );
    authoring::text(
        room,
        "MnIceHQSuppliesFullText",
        &["Make room. I can hold them here."],
    );
    authoring::text(
        room,
        "MnIceHQSuppliesAfterText",
        &["OLLIE: The other pad reaches the transfer annex."],
    );
    a::actor(pack, room, "MN_ICE_HQ_OLLIE", "SPRITE_FISHER", [25, 7])?;
    room.scripts.insert(
        "MN_ICE_HQ_OLLIE".into(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &["EVENT_MN_ICE_HQ_SUPPLIES"]),
            c("iftrue", &["MnIceHQSuppliesAfter"]),
            c("opentext", &[]),
            c("writetext", &["MnIceHQSuppliesText"]),
            c("waitbutton", &[]),
            c("giveitem", &["SUPER_POTION", "2"]),
            c("iffalse", &["MnIceHQSuppliesFull"]),
            c("setevent", &["EVENT_MN_ICE_HQ_SUPPLIES"]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    room.scripts.insert(
        "MnIceHQSuppliesAfter".into(),
        json!([c("jumptext", &["MnIceHQSuppliesAfterText"])]),
    );
    room.scripts.insert(
        "MnIceHQSuppliesFull".into(),
        json!([
            c("writetext", &["MnIceHQSuppliesFullText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    Ok(())
}
