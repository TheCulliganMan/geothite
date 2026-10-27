//! Native trainers, pickups, dialogue and signs use the production script parsers.
use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use crystal_core::models::TrainerPartyPokemon;
use serde_json::json;

use crate::authored_route_design::{Facing, Resident, ResidentService, RouteDesign};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn chunk(prefix: &str, [x, y]: [u16; 2]) -> String {
    format!("{prefix}C{:02}R{:02}", x / 64, y / 48)
}

pub(crate) fn local([x, y]: [u16; 2]) -> [u16; 2] {
    [x % 64, y % 48]
}

pub(crate) fn constant(name: &str) -> String {
    if name == "MnR12Cave1F" {
        "MN_R12_CAVE_1F".into()
    } else if name == "MnR12RangerHut" {
        "MN_R12_RANGER_HUT".into()
    } else if name == "MnR11SplitRockLighthouse" {
        "MN_R11_SPLIT_ROCK_LIGHTHOUSE".into()
    } else {
        name.replace("MnR11", "MN_R11_").replace("MnR12", "MN_R12_")
    }
}

fn module_at<'a>(
    modules: &'a mut [MapModule],
    prefix: &str,
    tile: [u16; 2],
) -> Result<&'a mut MapModule> {
    let name = chunk(prefix, tile);
    modules
        .iter_mut()
        .find(|m| m.id == name)
        .context("route anchor outside published chunks")
}

fn lines(module: &mut MapModule, label: &str, rows: &[String]) -> Result<()> {
    ensure!(
        !rows.is_empty()
            && rows
                .iter()
                .all(|row| !row.is_empty() && row.chars().count() <= 18),
        "{label}: dialogue must fit the native text box"
    );
    authoring::text(
        module,
        label,
        &rows.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    Ok(())
}

fn npc(
    base: &CompiledGamePack,
    module: &mut MapModule,
    script: &str,
    sprite: &str,
    tile: [u16; 2],
    facing: Facing,
) -> Result<()> {
    ensure!(
        base.runtime_files().contains_key(&format!(
            "gfx/sprites/{}.png",
            sprite.trim_start_matches("SPRITE_").to_ascii_lowercase()
        )),
        "missing native sprite {sprite}"
    );
    let [x, y] = local(tile);
    authoring::npc(
        base,
        module,
        script,
        sprite,
        i16::try_from(x)?,
        i16::try_from(y)?,
    )?;
    let object = module.objects.last_mut().context("inserted object")?;
    object.spritemovedata = facing.movement().into();
    object.object_identifier = Some(script.into());
    Ok(())
}

fn resident_script(module: &mut MapModule, resident: &Resident) -> Result<()> {
    let label = format!("{}Text", resident.id);
    lines(module, &label, &resident.lines)?;
    let commands = match resident.service {
        None => json!([authoring::command("jumptextfaceplayer", &[&label])]),
        Some(ResidentService::HealParty) => json!([
            authoring::command("faceplayer", &[]),
            authoring::command("opentext", &[]),
            authoring::command("writetext", &[&label]),
            authoring::command("waitbutton", &[]),
            authoring::command("special", &["HealParty"]),
            authoring::command("closetext", &[]),
            authoring::command("end", &[])
        ]),
    };
    module.scripts.insert(resident.id.clone(), commands);
    Ok(())
}

pub(crate) fn apply(
    pack: &mut CompiledGamePack,
    design: &RouteDesign,
    modules: &mut [MapModule],
) -> Result<()> {
    for trainer in &design.trainers {
        ensure!(
            !pack.data.trainers.trainers.contains_key(&trainer.id),
            "duplicate trainer {}",
            trainer.id
        );
        let mut opponent = pack
            .data
            .trainers
            .trainers
            .values()
            .find(|t| t.trainer_class == trainer.trainer_class)
            .with_context(|| format!("unknown source class {}", trainer.trainer_class))?
            .clone();
        let dvs = opponent.party.first().context("source class DVs")?.dvs;
        opponent.trainer_id = trainer.id.clone();
        opponent.name = trainer.display_name.to_ascii_uppercase();
        opponent.party = trainer
            .party
            .iter()
            .map(|member| {
                ensure!((1..=100).contains(&member.level), "invalid trainer level");
                let pokemon = pack
                    .data
                    .create_pokemon(&member.species, member.level, dvs)?;
                Ok(TrainerPartyPokemon {
                    species: member.species.clone(),
                    level: member.level,
                    item: None,
                    moves: pokemon.moves,
                    dvs,
                })
            })
            .collect::<Result<_>>()?;
        ensure!(
            !opponent.party.is_empty() && opponent.party.len() <= 6,
            "invalid trainer party size"
        );
        opponent.win_quote.clear();
        opponent.lose_quote.clear();
        pack.data.trainers.insert(opponent)?;
        let module = module_at(modules, &design.map_prefix, trainer.tile)?;
        let [seen, won, lost, after] =
            ["Seen", "Won", "Lost", "After"].map(|suffix| format!("{}{suffix}", trainer.id));
        lines(module, &seen, &trainer.before_lines)?;
        lines(module, &won, &trainer.after_lines)?;
        lines(module, &lost, &trainer.loss_lines)?;
        module.scripts.insert(
            trainer.id.clone(),
            json!([authoring::command(
                "trainer",
                &[
                    &trainer.trainer_class,
                    &trainer.id,
                    &trainer.defeated_flag,
                    &seen,
                    &won,
                    &lost,
                    &after
                ]
            )]),
        );
        module.scripts.insert(
            after,
            json!([
                authoring::command("endifjustbattled", &[]),
                authoring::command("jumptextfaceplayer", &[&won])
            ]),
        );
        npc(
            pack,
            module,
            &trainer.id,
            &trainer.sprite,
            trainer.tile,
            trainer.facing,
        )?;
        let object = module.objects.last_mut().context("trainer object")?;
        object.object_type = "OBJECTTYPE_TRAINER".into();
        object.radius = trainer.sight_range.into();
    }
    for resident in &design.residents {
        let module = module_at(modules, &design.map_prefix, resident.tile)?;
        resident_script(module, resident)?;
        npc(
            pack,
            module,
            &resident.id,
            &resident.sprite,
            resident.tile,
            resident.facing,
        )?;
    }
    for item in &design.items {
        ensure!(
            pack.data.items.contains_key(&item.item) && item.quantity > 0,
            "invalid route pickup {}",
            item.id
        );
        let module = module_at(modules, &design.map_prefix, item.tile)?;
        module.scripts.insert(
            item.id.clone(),
            json!([authoring::command(
                "itemball",
                &[&item.item, &item.quantity.to_string()]
            )]),
        );
        npc(
            pack,
            module,
            &item.id,
            "SPRITE_POKE_BALL",
            item.tile,
            Facing::South,
        )?;
        let object = module.objects.last_mut().context("pickup object")?;
        object.event_flag = item.flag.clone();
        object.object_type = "OBJECTTYPE_ITEMBALL".into();
    }
    for sign in &design.signs {
        let module = module_at(modules, &design.map_prefix, sign.tile)?;
        let label = format!("{}Text", sign.id);
        lines(module, &label, &sign.lines)?;
        module.scripts.insert(
            sign.id.clone(),
            json!([authoring::command("jumptext", &[&label])]),
        );
        let [x, y] = local(sign.tile);
        module.events.bg_events.push(BackgroundEvent {
            x,
            y,
            event_type: "BGEVENT_READ".into(),
            script: sign.id.clone(),
        });
    }
    for module in modules {
        authoring::finish(pack, module)?;
        module.script_runtime_commands =
            crate::parse_script_runtime_commands(&module.id, &module.scripts)?;
        module.script_object_commands =
            crate::parse_script_object_commands(&module.id, &module.scripts)?;
        module.trainer_scripts = crate::parse_trainer_scripts(&module.id, &module.scripts)?;
        module.script_field_pickups =
            crate::parse_script_field_pickups(&module.id, &module.scripts, &module.objects)?;
    }
    Ok(())
}

pub(crate) fn keeper(
    pack: &CompiledGamePack,
    module: &mut MapModule,
    design: &crate::authored_route_design::Interior,
) -> Result<()> {
    let label = "MnR11KeeperText";
    lines(module, label, &design.keeper_lines)?;
    let script = "MnR11Keeper";
    module.scripts.insert(
        script.into(),
        json!([authoring::command("jumptextfaceplayer", &[label])]),
    );
    let [x, y] = design.keeper_tile.context("authored keeper tile")?;
    authoring::npc(
        pack,
        module,
        script,
        "SPRITE_SAILOR",
        i16::try_from(x)?,
        i16::try_from(y)?,
    )?;
    authoring::finish(pack, module)
}

pub(crate) fn room_events(
    pack: &CompiledGamePack,
    module: &mut MapModule,
    room: &crate::authored_route_design::Interior,
) -> Result<()> {
    if room.keeper_tile.is_some() {
        keeper(pack, module, room)?;
    }
    for resident in &room.residents {
        resident_script(module, resident)?;
        npc(
            pack,
            module,
            &resident.id,
            &resident.sprite,
            resident.tile,
            resident.facing,
        )?;
    }
    for item in &room.items {
        ensure!(
            pack.data.items.contains_key(&item.item) && item.quantity > 0,
            "invalid room pickup {}",
            item.id
        );
        module.scripts.insert(
            item.id.clone(),
            json!([authoring::command(
                "itemball",
                &[&item.item, &item.quantity.to_string()]
            )]),
        );
        npc(
            pack,
            module,
            &item.id,
            "SPRITE_POKE_BALL",
            item.tile,
            Facing::South,
        )?;
        let object = module.objects.last_mut().context("room pickup object")?;
        object.event_flag = item.flag.clone();
        object.object_type = "OBJECTTYPE_ITEMBALL".into();
    }
    for sign in &room.signs {
        let label = format!("{}Text", sign.id);
        lines(module, &label, &sign.lines)?;
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
    authoring::finish(pack, module)?;
    module.script_runtime_commands =
        crate::parse_script_runtime_commands(&module.id, &module.scripts)?;
    module.script_object_commands =
        crate::parse_script_object_commands(&module.id, &module.scripts)?;
    module.script_field_pickups =
        crate::parse_script_field_pickups(&module.id, &module.scripts, &module.objects)?;
    Ok(())
}
