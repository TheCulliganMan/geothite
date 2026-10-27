//! Native ordinary services and the precise Q07 harbor receiver interaction.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use serde_json::{Value, json};

use super::design::{Actor, Room};
use crate::authored_route_design::Sign;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

pub(super) const MART: &str = "MART_MN_DULUTH";
const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";
const DEPOT: &str = "EVENT_MN_DEPOT_CLEARED";

pub(super) fn prepare_catalogs(pack: &mut CompiledGamePack, rooms: &[Room]) -> Result<()> {
    ensure!(
        !pack.data.marts.0.contains_key(MART),
        "Duluth mart identity already occupied"
    );
    let clerk = rooms
        .iter()
        .flat_map(|r| &r.residents)
        .find(|a| a.id == "MN_DULUTH_MART_CLERK")
        .context("Duluth native clerk")?;
    ensure!(
        clerk.stock
            == [
                "ULTRA_BALL",
                "HYPER_POTION",
                "FULL_HEAL",
                "REVIVE",
                "MAX_REPEL"
            ],
        "retain authored Duluth mart stock"
    );
    ensure!(
        clerk
            .stock
            .iter()
            .all(|item| pack.data.items.contains_key(item)),
        "unknown Duluth shop item"
    );
    pack.data.marts.0.insert(MART.into(), clerk.stock.clone());
    for name in [SIGNAL, DEPOT] {
        if let Some(id) = pack.data.story_event_script_constants.global.get(name) {
            ensure!(
                !pack
                    .data
                    .story_event_script_constants
                    .global
                    .iter()
                    .any(|(other, value)| other != name
                        && other.starts_with("EVENT_")
                        && value == id),
                "Duluth event identity collision: {name}"
            );
        } else {
            let next = pack
                .data
                .story_event_script_constants
                .global
                .iter()
                .filter(|(k, _)| k.starts_with("EVENT_"))
                .map(|(_, v)| *v)
                .max()
                .unwrap_or(0)
                .checked_add(1)
                .context("Duluth event identity overflow")?;
            pack.data
                .story_event_script_constants
                .global
                .insert(name.into(), next);
        }
    }
    Ok(())
}

fn text(module: &mut MapModule, label: &str, lines: &[String]) -> Result<()> {
    ensure!(
        !lines.is_empty()
            && lines
                .iter()
                .all(|s| !s.is_empty() && s.chars().count() <= 18),
        "Duluth text must fit native 18-column pages: {label}"
    );
    authoring::text(
        module,
        label,
        &lines.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    Ok(())
}

pub(super) fn install(
    pack: &CompiledGamePack,
    module: &mut MapModule,
    actors: &[Actor],
    signs: &[Sign],
) -> Result<()> {
    for actor in actors {
        // Script labels are global in the split pack, while source actor IDs
        // are map-local (the city and clinic each have a CLINIC_VISITOR).
        // Preserve the three named service/story entry points; scope ordinary
        // interactions without changing their source object identifiers.
        let script = match actor.id.as_str() {
            "MN_HARBOR_RECEIVER" | "MN_DULUTH_NURSE" | "MN_DULUTH_MART_CLERK" => actor.id.clone(),
            _ => format!("{}_{}", module.id, actor.id),
        };
        ensure!(
            !module.scripts.contains_key(&script),
            "duplicate Duluth interaction {}",
            actor.id
        );
        ensure!(
            pack.runtime_files().contains_key(&format!(
                "gfx/sprites/{}.png",
                actor
                    .sprite
                    .trim_start_matches("SPRITE_")
                    .to_ascii_lowercase()
            )),
            "missing Duluth native sprite"
        );
        let label = format!("{script}Text");
        text(module, &label, &actor.lines)?;
        let commands = if actor.id == "MN_HARBOR_RECEIVER" {
            receiver(module, actor, &label)?
        } else if actor.id == "MN_DULUTH_NURSE" {
            ensure!(
                actor.service.as_deref() == Some("heal_party"),
                "retain native Duluth nurse service"
            );
            json!([
                authoring::command("faceplayer", &[]),
                authoring::command("opentext", &[]),
                authoring::command("writetext", &[&label]),
                authoring::command("waitbutton", &[]),
                authoring::command("special", &["HealParty"]),
                authoring::command("closetext", &[]),
                authoring::command("end", &[])
            ])
        } else if actor.id == "MN_DULUTH_MART_CLERK" {
            ensure!(
                actor.service.as_deref() == Some("canonical_pokemart_pending"),
                "retain native Duluth clerk service"
            );
            json!([
                authoring::command("faceplayer", &[]),
                authoring::command("opentext", &[]),
                authoring::command("pokemart", &["MARTTYPE_STANDARD", MART]),
                authoring::command("closetext", &[]),
                authoring::command("end", &[])
            ])
        } else {
            ensure!(
                actor.service.is_none() && actor.sets.is_empty(),
                "unsupported authored Duluth service: {}",
                actor.id
            );
            json!([authoring::command("jumptextfaceplayer", &[&label])])
        };
        module.scripts.insert(script.clone(), commands);
        authoring::npc(
            pack,
            module,
            &script,
            &actor.sprite,
            i16::try_from(actor.tile[0])?,
            i16::try_from(actor.tile[1])?,
        )?;
        let native = module
            .objects
            .last_mut()
            .context("Duluth resident object")?;
        native.object_identifier = Some(actor.id.clone());
        native.spritemovedata = actor.facing.movement().into();
        native.object_type = "OBJECTTYPE_SCRIPT".into();
        native.radius = 0;
    }
    for sign in signs {
        let script = format!("{}_{}", module.id, sign.id);
        let label = format!("{script}Text");
        text(module, &label, &sign.lines)?;
        module.scripts.insert(
            script.clone(),
            json!([authoring::command("jumptext", &[&label])]),
        );
        module.events.bg_events.push(BackgroundEvent {
            x: sign.tile[0],
            y: sign.tile[1],
            event_type: "BGEVENT_READ".into(),
            script,
        });
    }
    Ok(())
}

fn receiver(module: &mut MapModule, actor: &Actor, label: &str) -> Result<Value> {
    let mut expected = (1..=8)
        .map(|n| format!("BADGE_{n}"))
        .collect::<BTreeSet<_>>();
    expected.insert(DEPOT.into());
    ensure!(
        module.id == "MnDuluthHarborWorkshop"
            && actor.tile == [11, 5]
            && actor.requires.len() == 9
            && actor.requires.iter().cloned().collect::<BTreeSet<_>>() == expected
            && actor.sets == [SIGNAL],
        "retain exact Q07 receiver anchor, eight badges, depot prerequisite and signal result"
    );
    let done = format!("{}Done", actor.id);
    let done_text = format!("{done}Text");
    let denied = format!("{}Denied", actor.id);
    let denied_text = format!("{denied}Text");
    text(module, &done_text, &actor.repeat_lines)?;
    authoring::text(
        module,
        &denied_text,
        &[
            "Bring eight BADGES",
            "and restore the",
            "Grand Rapids",
            "depot.",
        ],
    );
    module.scripts.insert(
        done.clone(),
        json!([authoring::command("jumptextfaceplayer", &[&done_text])]),
    );
    module.scripts.insert(
        denied.clone(),
        json!([authoring::command("jumptextfaceplayer", &[&denied_text])]),
    );
    let mut commands = vec![
        authoring::command("faceplayer", &[]),
        authoring::command("checkevent", &[SIGNAL]),
        authoring::command("iftrue", &[&done]),
    ];
    for badge in &BADGE_ENGINE_FLAGS[..8] {
        commands.extend([
            authoring::command("checkflag", &[badge]),
            authoring::command("iffalse", &[&denied]),
        ]);
    }
    commands.extend([
        authoring::command("checkevent", &[DEPOT]),
        authoring::command("iffalse", &[&denied]),
        authoring::command("opentext", &[]),
        authoring::command("writetext", &[label]),
        authoring::command("waitbutton", &[]),
        authoring::command("setevent", &[SIGNAL]),
        authoring::command("closetext", &[]),
        authoring::command("end", &[]),
    ]);
    Ok(Value::Array(commands))
}

pub(super) fn finish(pack: &CompiledGamePack, module: &mut MapModule) -> Result<()> {
    authoring::finish(pack, module)?;
    module.script_runtime_commands =
        crate::parse_script_runtime_commands(&module.id, &module.scripts)?;
    module.script_object_commands =
        crate::parse_script_object_commands(&module.id, &module.scripts)?;
    module.script_variable_commands =
        crate::parse_script_variable_commands(&module.id, &module.scripts)?;
    Ok(())
}
