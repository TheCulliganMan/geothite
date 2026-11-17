//! Regional supplies and the affordable shared move-reminder service.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;
use crate::{CompiledGamePack, MapModule};

/// Uses native marts and the production party/move-selection special.
pub(crate) fn install(pack: &mut CompiledGamePack, story_source: &str) -> Result<()> {
    ensure!(
        !pack.data.special_routines.contains_key("MoveReminder"),
        "move reminder already registered"
    );
    let reminder = pack
        .data
        .special_routines
        .get("MoveTutor")
        .context("shared move tutor special metadata")?
        .clone();
    pack.data
        .special_routines
        .insert("MoveReminder".into(), reminder);
    let source: Value = serde_json::from_str(story_source)?;
    let dialogue: BTreeMap<String, BTreeMap<String, Vec<AuthoredDialoguePage>>> =
        serde_json::from_value(source["refinement_services"]["dialogue"].clone())?;
    for (mart, additions) in [
        ("MART_MN_WINONA", &["REPEL"][..]),
        ("MART_MN_ROCHESTER", &["SUPER_POTION", "REPEL"][..]),
        (
            "MART_MN_MP_DEPT_5_CLERK",
            &[
                "TM_DIG",
                "TM_HEADBUTT",
                "TM_GIGA_DRAIN",
                "TM_FIRE_PUNCH",
                "TM_THUNDERPUNCH",
                "TM_RETURN",
                "TM_REST",
                "TM_SUNNY_DAY",
            ][..],
        ),
        (
            "MART_MN_MP_DEPT_3_CLERK",
            &[
                "CHARCOAL",
                "MIRACLE_SEED",
                "MYSTIC_WATER",
                "MAGNET",
                "SOFT_SAND",
            ][..],
        ),
    ] {
        ensure!(
            additions
                .iter()
                .all(|item| pack.data.items.contains_key(*item)),
            "shop item catalog"
        );
        let stock = pack
            .data
            .marts
            .0
            .get_mut(mart)
            .with_context(|| format!("regional mart {mart}"))?;
        for item in additions {
            ensure!(
                !stock.iter().any(|entry| entry == *item),
                "duplicate refinement stock {mart}/{item}"
            );
            stock.push((*item).into());
        }
        ensure!(stock.len() <= 16, "native shop capacity");
    }
    let stock = [
        "ULTRA_BALL",
        "HYPER_POTION",
        "MAX_POTION",
        "FULL_HEAL",
        "REVIVE",
        "MAX_REPEL",
        "ESCAPE_ROPE",
        "GOLD_BERRY",
        "MINT_BERRY",
        "TM_ICY_WIND",
        "TM_SHADOW_BALL",
        "TM_PSYCHIC_M",
        "TM_TOXIC",
    ];
    ensure!(
        stock.iter().all(|item| pack.data.items.contains_key(*item)),
        "late supplies exist"
    );
    ensure!(
        !pack.data.marts.0.contains_key("MART_MN_GRAND_MARAIS"),
        "Grand Marais stock already exists"
    );
    pack.data.marts.0.insert(
        "MART_MN_GRAND_MARAIS".into(),
        stock.map(str::to_owned).to_vec(),
    );
    let mut marais = room(pack, "MnGrandMaraisMart")?;
    let clerk = marais
        .scripts
        .get_mut("MnGrandMaraisClerk")
        .and_then(Value::as_array_mut)
        .context("Grand Marais clerk")?;
    let commands: Vec<_> = clerk
        .iter_mut()
        .filter(|entry| entry["command"] == "pokemart")
        .collect();
    ensure!(commands.len() == 1, "one Grand Marais shop");
    for command in commands {
        ensure!(
            command["args"] == json!(["MARTTYPE_STANDARD", "MART_VIOLET"]),
            "Grand Marais mart preimage"
        );
        command["args"] = json!(["MARTTYPE_STANDARD", "MART_MN_GRAND_MARAIS"]);
    }
    texts(&mut marais, &dialogue)?;
    marais.scripts.insert(
        "MN_SMALL_INTERIOR_GRAND_MARAIS_SHOPPER".into(),
        json!([c(
            "jumptextfaceplayer",
            &["MnRefineGrandMaraisSuppliesText"]
        )]),
    );
    crate::northstar_story_revision::publish(pack, marais)?;
    for map in [
        "MnMinneapolisDepartmentStore3F",
        "MnMinneapolisDepartmentStore5F",
    ] {
        let mut store = room(pack, map)?;
        texts(&mut store, &dialogue)?;
        crate::northstar_story_revision::publish(pack, store)?;
    }
    let mut workshop = room(pack, "MnMinneapolisToolWorkshop")?;
    texts(&mut workshop, &dialogue)?;
    ensure!(
        workshop
            .objects
            .iter()
            .any(|actor| actor.script == "MN_MP_INTERIOR_TOOL_APPRENTICE"),
        "existing reminder apprentice"
    );
    workshop
        .scripts
        .insert("MN_MP_INTERIOR_TOOL_APPRENTICE".into(), reminder_program());
    for (label, text) in [
        ("MnRefineReminderPoor", "MnRefineReminderPoorText"),
        ("MnRefineReminderCancel", "MnRefineReminderCancelText"),
    ] {
        workshop.scripts.insert(
            label.into(),
            json!([
                c("writetext", &[text]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        );
    }
    crate::northstar_story_revision::publish(pack, workshop)?;
    Ok(())
}

fn reminder_program() -> Value {
    json!([
        c("faceplayer", &[]),
        c("opentext", &[]),
        c("writetext", &["MnRefineReminderOfferText"]),
        c("yesorno", &[]),
        c("iffalse", &["MnRefineReminderCancel"]),
        c("checkmoney", &["YOUR_MONEY", "500"]),
        c("ifequal", &["HAVE_LESS", "MnRefineReminderPoor"]),
        c("special", &["MoveReminder"]),
        c("ifnotequal", &["0", "MnRefineReminderCancel"]),
        c("writetext", &["MnRefineReminderDoneText"]),
        c("waitbutton", &[]),
        c("closetext", &[]),
        c("end", &[])
    ])
}

fn room(pack: &CompiledGamePack, name: &str) -> Result<MapModule> {
    pack.data
        .maps
        .get(name)
        .cloned()
        .with_context(|| format!("installed service room {name}"))
}

fn texts(
    room: &mut MapModule,
    dialogue: &BTreeMap<String, BTreeMap<String, Vec<AuthoredDialoguePage>>>,
) -> Result<()> {
    for (label, pages) in dialogue.get(&room.id).context("canonical service prose")? {
        room.scripts
            .insert(label.clone(), authored_dialogue::compile(pages)?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::reminder_program;

    #[test]
    fn reminder_checks_funds_but_leaves_the_atomic_charge_to_the_special() {
        let program = reminder_program();
        let commands = program.as_array().expect("script program");
        let index = |name: &str| {
            commands
                .iter()
                .position(|entry| entry["command"] == name)
                .expect("command")
        };
        assert!(index("checkmoney") < index("special"));
        assert!(!commands.iter().any(|entry| entry["command"] == "takemoney"));
        let cancel: &Value = &commands[index("special") + 1];
        assert_eq!(cancel["command"], "ifnotequal");
        assert_eq!(cancel["args"][0], "0");
        assert_eq!(cancel["args"][1], "MnRefineReminderCancel");
        assert!(
            commands[..index("special")]
                .iter()
                .any(|entry| entry["command"] == "iffalse")
        );
    }
}
