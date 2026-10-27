//! Production script lowering for Brainerd's authored ordinary services.
use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde_json::json;

use crate::brainerd_city_modpack::{Resident, Room, Sign, Town};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const MART: &str = "MART_MN_BRAINERD";
const REPORT: &str = "EVENT_MN_BRAINERD_REPORT";
const SURF: &str = "EVENT_MN_SURF_GIFT_RECEIVED";
const CAMP: &str = "EVENT_MN_BRAINERD_CAMP_REPEL";

fn event(pack: &mut CompiledGamePack, name: &str) -> Result<()> {
    let globals = &mut pack.data.story_event_script_constants.global;
    if let Some(id) = globals.get(name) {
        ensure!(
            !globals
                .iter()
                .any(|(other, value)| other != name && other.starts_with("EVENT_") && value == id),
            "Brainerd event identifier collision {name}"
        );
    } else {
        let next = globals
            .iter()
            .filter(|(key, _)| key.starts_with("EVENT_"))
            .map(|(_, id)| *id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .context("Brainerd event overflow")?;
        globals.insert(name.into(), next);
    }
    Ok(())
}

pub(crate) fn prepare_catalogs(pack: &mut CompiledGamePack, town: &Town) -> Result<()> {
    pack.data.require_special_routine("HealParty")?;
    pack.data.require_special_routine("PokemonCenterPC")?;
    let mart = town
        .interiors
        .iter()
        .find(|r| r.id == "MnBrainerdMart")
        .context("Brainerd MART source")?;
    ensure!(
        !mart.inventory_contract.is_empty()
            && mart
                .inventory_contract
                .iter()
                .all(|item| pack.data.items.contains_key(item)),
        "Brainerd shop needs existing native items"
    );
    ensure!(
        !pack.data.marts.0.contains_key(MART),
        "Brainerd shop catalog already occupied"
    );
    pack.data
        .marts
        .0
        .insert(MART.into(), mart.inventory_contract.clone());
    for name in [REPORT, SURF, CAMP] {
        event(pack, name)?;
    }
    Ok(())
}

fn text(module: &mut MapModule, label: &str, lines: &[String]) -> Result<()> {
    ensure!(
        !lines.is_empty()
            && lines
                .iter()
                .all(|line| !line.is_empty() && line.chars().count() <= 18),
        "Brainerd dialogue must fit native18 columns: {label}"
    );
    authoring::text(
        module,
        label,
        &lines.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    Ok(())
}

pub(crate) fn people(
    pack: &CompiledGamePack,
    module: &mut MapModule,
    residents: &[Resident],
    signs: &[Sign],
) -> Result<()> {
    for resident in residents {
        ensure!(
            !module.scripts.contains_key(&resident.id)
                && !pack.data.map_scripts.contains_key(&resident.id),
            "Brainerd actor script identity occupied"
        );
        ensure!(
            pack.runtime_files().contains_key(&format!(
                "gfx/sprites/{}.png",
                resident
                    .sprite
                    .trim_start_matches("SPRITE_")
                    .to_ascii_lowercase()
            )),
            "Brainerd actor sprite absent"
        );
        let label = format!("{}Text", resident.id);
        text(module, &label, &resident.lines)?;
        let command = authoring::command;
        let script = if resident.id == "MN_BRAINERD_RANGER" {
            ranger(module, resident, &label)?
        } else {
            match resident.service.as_deref() {
                None => json!([command("jumptextfaceplayer", &[&label])]),
                Some("heal_party") => json!([
                    command("faceplayer", &[]),
                    command("opentext", &[]),
                    command("writetext", &[&label]),
                    command("waitbutton", &[]),
                    command("special", &["HealParty"]),
                    command("closetext", &[]),
                    command("end", &[])
                ]),
                Some("canonical_pokemart_pending") => {
                    ensure!(
                        module.id == "MnBrainerdMart",
                        "Brainerd shop belongs in MART"
                    );
                    json!([
                        command("faceplayer", &[]),
                        command("opentext", &[]),
                        command("pokemart", &["MARTTYPE_STANDARD", MART]),
                        command("closetext", &[]),
                        command("end", &[])
                    ])
                }
                Some("give_item_once") => gift(pack, module, resident)?,
                Some(other) => anyhow::bail!("unsupported Brainerd resident service {other}"),
            }
        };
        module.scripts.insert(resident.id.clone(), script);
        authoring::npc(
            pack,
            module,
            &resident.id,
            &resident.sprite,
            i16::try_from(resident.tile[0])?,
            i16::try_from(resident.tile[1])?,
        )?;
        let actor = module
            .objects
            .last_mut()
            .context("Brainerd actor inserted")?;
        actor.object_identifier = Some(resident.id.clone());
        actor.spritemovedata = match resident.facing.as_str() {
            "north" => "SPRITEMOVEDATA_STANDING_UP",
            "south" => "SPRITEMOVEDATA_STANDING_DOWN",
            "east" => "SPRITEMOVEDATA_STANDING_RIGHT",
            "west" => "SPRITEMOVEDATA_STANDING_LEFT",
            _ => anyhow::bail!("unknown Brainerd resident facing"),
        }
        .into();
        actor.object_type = "OBJECTTYPE_SCRIPT".into();
        actor.radius = 0;
    }
    for sign in signs {
        ensure!(
            !module.scripts.contains_key(&sign.id) && !pack.data.map_scripts.contains_key(&sign.id),
            "Brainerd sign script identity occupied"
        );
        let label = format!("{}Text", sign.id);
        text(module, &label, &sign.lines)?;
        module.scripts.insert(
            sign.id.clone(),
            json!([authoring::command("jumptext", &[&label])]),
        );
        module.events.bg_events.push(BackgroundEvent {
            x: sign.tile[0],
            y: sign.tile[1],
            event_type: "BGEVENT_READ".into(),
            script: sign.id.clone(),
        });
    }
    Ok(())
}

fn ranger(module: &mut MapModule, resident: &Resident, label: &str) -> Result<serde_json::Value> {
    let reward = resident
        .reward
        .as_ref()
        .context("Brainerd actual Surf gift")?;
    ensure!(
        module.id == "MnBrainerd"
            && resident.tile == [25, 19]
            && resident.requires == ["BADGE_6"]
            && reward.item == "HM_SURF"
            && reward.quantity == 1
            && reward.once_flag == SURF
            && reward.sets == [REPORT, SURF],
        "preserve Q04/MS05 ranger anchor and earned Surf contract"
    );
    let before = "MnBrainerdRangerBefore";
    let repeat = "MnBrainerdRangerRepeat";
    let full = "MnBrainerdRangerFull";
    let gift = "MnBrainerdRangerGift";
    for (name, lines) in [
        (before, &resident.before_lines),
        (repeat, &resident.repeat_lines),
    ] {
        let text_label = format!("{name}Text");
        text(module, &text_label, lines)?;
        module.scripts.insert(
            name.into(),
            json!([authoring::command("jumptextfaceplayer", &[&text_label])]),
        );
    }
    for (name, lines) in [
        (
            full,
            &["Make room in your", "TM POCKET, then", "come back."][..],
        ),
        (
            gift,
            &["SURF is yours.", "The dry north road", "still leads on."][..],
        ),
    ] {
        let text_label = format!("{name}Text");
        authoring::text(module, &text_label, lines);
        module.scripts.insert(
            name.into(),
            json!([authoring::command("jumptextfaceplayer", &[&text_label])]),
        );
    }
    let command = authoring::command;
    Ok(json!([
        command("faceplayer", &[]),
        command("checkevent", &[SURF]),
        command("iftrue", &[repeat]),
        command("checkflag", &["ENGINE_MINERALBADGE"]),
        command("iffalse", &[before]),
        command("opentext", &[]),
        command("writetext", &[label]),
        command("waitbutton", &[]),
        command("closetext", &[]),
        command("giveitem", &["HM_SURF", "1"]),
        command("iffalse", &[full]),
        command("setevent", &[REPORT]),
        command("setevent", &[SURF]),
        command("sjump", &[gift])
    ]))
}

fn gift(
    pack: &CompiledGamePack,
    module: &mut MapModule,
    resident: &Resident,
) -> Result<serde_json::Value> {
    let reward = resident.reward.as_ref().context("Brainerd local reward")?;
    ensure!(
        module.id == "MnBrainerdOutfitter"
            && resident.id == "MN_BRAINERD_GEAR_KEEPER"
            && reward.item == "SUPER_REPEL"
            && reward.quantity == 1
            && reward.once_flag == CAMP
            && reward.sets.is_empty()
            && pack.data.items.contains_key(&reward.item),
        "preserve success-only optional Brainerd camp gift"
    );
    let full = format!("{}Full", resident.id);
    let full_text = format!("{full}Text");
    authoring::text(
        module,
        &full_text,
        &["Your BAG is full.", "Come back for the", "SUPER REPEL."],
    );
    module.scripts.insert(
        full.clone(),
        json!([authoring::command("jumptextfaceplayer", &[&full_text])]),
    );
    let received = format!("{}GiftText", resident.id);
    authoring::text(
        module,
        &received,
        &["Take this", "SUPER REPEL!", "For the lake road."],
    );
    let command = authoring::command;
    Ok(json!([
        command("faceplayer", &[]),
        command("checkevent", &[CAMP]),
        command("iftrue", &[&format!("{}After", resident.id)]),
        command("giveitem", &[&reward.item, "1"]),
        command("iffalse", &[&full]),
        command("setevent", &[CAMP]),
        command("jumptextfaceplayer", &[&received])
    ]))
}

pub(crate) fn pc(pack: &CompiledGamePack, module: &mut MapModule, room: &Room) -> Result<()> {
    if let Some(pc) = &room.pc {
        ensure!(
            room.id == "MnBrainerdPokecenter1F",
            "Brainerd PC belongs in clinic"
        );
        pack.data.require_special_routine("PokemonCenterPC")?;
        let label = "MnBrainerdClinicPC";
        let command = authoring::command;
        module.scripts.insert(
            label.into(),
            json!([
                command("opentext", &[]),
                command("special", &["PokemonCenterPC"]),
                command("closetext", &[]),
                command("end", &[])
            ]),
        );
        module.events.bg_events.push(BackgroundEvent {
            x: pc.tile[0],
            y: pc.tile[1],
            event_type: "BGEVENT_READ".into(),
            script: label.into(),
        });
    }
    Ok(())
}

pub(crate) fn finish(pack: &CompiledGamePack, module: &mut MapModule) -> Result<()> {
    // Resolve the local gift's repeat label before parsing native control flow.
    let id = "MN_BRAINERD_GEAR_KEEPER";
    if module.scripts.contains_key(id) {
        let label = format!("{id}After");
        module.scripts.insert(
            label,
            json!([authoring::command(
                "jumptextfaceplayer",
                &[&format!("{id}Text")]
            )]),
        );
    }
    authoring::finish(pack, module)?;
    module.script_runtime_commands =
        crate::parse_script_runtime_commands(&module.id, &module.scripts)?;
    module.script_object_commands =
        crate::parse_script_object_commands(&module.id, &module.scripts)?;
    Ok(())
}
