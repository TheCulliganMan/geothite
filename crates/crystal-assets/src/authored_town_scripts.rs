//! Shared ordinary dialogue/services and native script lowering for authored towns.
use std::collections::BTreeMap;

use anyhow::{Context, Result};
use crystal_core::map::BackgroundEvent;
use serde_json::json;

use crate::authored_town_design::{Actor, Town};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn ordinary(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    actors: &[Actor],
    signs: &[crate::authored_route_design::Sign],
    clinic: &str,
    mart: &str,
) -> Result<()> {
    let c = authoring::command;
    for actor in actors {
        let label = format!("{}Text", actor.id);
        let body = match actor.service.as_deref() {
            None => {
                authoring::text(
                    map,
                    &label,
                    &actor.lines.iter().map(String::as_str).collect::<Vec<_>>(),
                );
                json!([c("jumptextfaceplayer", &[&label])])
            }
            Some("heal_party") => {
                authoring::text(
                    map,
                    &label,
                    &actor.lines.iter().map(String::as_str).collect::<Vec<_>>(),
                );
                json!([
                    c("faceplayer", &[]),
                    c("opentext", &[]),
                    c("writetext", &[&label]),
                    c("waitbutton", &[]),
                    c("special", &["HealParty"]),
                    c("blackoutmod", &[clinic]),
                    c("closetext", &[]),
                    c("end", &[])
                ])
            }
            Some("pokemart") => json!([
                c("faceplayer", &[]),
                c("opentext", &[]),
                c("pokemart", &["MARTTYPE_STANDARD", mart]),
                c("closetext", &[]),
                c("end", &[])
            ]),
            Some("guardian" | "professor" | "supplies" | "recipient") => json!([c("end", &[])]),
            Some(service) => anyhow::bail!("unsupported authored town service {service}"),
        };
        map.scripts.insert(actor.id.clone(), body);
        authoring::npc(
            pack,
            map,
            &actor.id,
            &actor.sprite,
            i16::try_from(actor.tile[0])?,
            i16::try_from(actor.tile[1])?,
        )?;
        let object = map.objects.last_mut().context("authored town resident")?;
        object.object_identifier = Some(actor.id.clone());
        object.spritemovedata = actor.facing.movement().into();
    }
    for sign in signs {
        let label = format!("{}Text", sign.id);
        authoring::text(
            map,
            &label,
            &sign.lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
        map.scripts
            .insert(sign.id.clone(), json!([c("jumptext", &[&label])]));
        map.events.bg_events.push(BackgroundEvent {
            x: sign.tile[0],
            y: sign.tile[1],
            event_type: "BGEVENT_READ".into(),
            script: sign.id.clone(),
        });
    }
    Ok(())
}

pub(crate) fn finish(pack: &CompiledGamePack, map: &mut MapModule, town: &Town) -> Result<()> {
    authoring::finish(pack, map)?;
    let name = &map.id;
    let scripts = &map.scripts;
    map.script_runtime_commands = crate::parse_script_runtime_commands(name, scripts)?;
    map.script_object_commands = crate::parse_script_object_commands(name, scripts)?;
    map.script_variable_commands = crate::parse_script_variable_commands(name, scripts)?;
    map.script_movements =
        crate::parse_script_movements(name, scripts, &map.script_object_commands)?;
    map.script_menu_definitions = crate::parse_script_menu_definitions(name, scripts)?;
    map.script_vertical_menus =
        crate::parse_script_vertical_menus(name, scripts, &map.script_menu_definitions)?;
    let mut names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|m| (m.constant.clone(), m.name.clone()))
        .collect::<BTreeMap<_, _>>();
    names.insert(town.native_constant.clone(), town.id.clone());
    for r in &town.interiors {
        names.insert(r.native_constant.clone(), r.id.clone());
    }
    map.script_map_commands = crate::parse_script_map_commands(name, scripts, &names)?;
    Ok(())
}
