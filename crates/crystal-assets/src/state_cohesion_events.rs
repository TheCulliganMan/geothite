//! Additive checkpoint and optional-trainer lowering; old programs are not reparsed.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::authored_checkpoint::{Checkpoint, install as checkpoint};
use crate::authored_city_expansion::merge_new_content;
use crate::authored_city_expansion_events as events;
use crate::authored_route_design::RouteTrainer;
use crate::authored_town_geometry::kind;
use crate::grand_marais_modpack::authoring;
use crate::state_cohesion_design::{Dossier, Plan};
use crate::{CompiledGamePack, MapModule};

pub(crate) fn install(
    pack: &mut CompiledGamePack,
    maps: &mut BTreeMap<String, MapModule>,
    plan: &Plan,
    dossiers: &BTreeMap<String, Dossier>,
) -> Result<()> {
    let names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|m| (m.constant.clone(), m.name.clone()))
        .chain(
            plan.new_maps
                .iter()
                .map(|m| (m.map_constant.clone(), m.id.clone())),
        )
        .collect::<BTreeMap<_, _>>();
    let flags = dossiers
        .values()
        .flat_map(|d| d.actors.iter().map(|a| a.event_flag.clone()))
        .collect::<Vec<_>>();
    ensure!(
        flags.len() == 4 && flags.iter().collect::<BTreeSet<_>>().len() == 4,
        "four unique trainer defeat flags required"
    );
    events::flags(pack, &flags)?;
    for (name, dossier) in dossiers {
        let map = maps.get_mut(name).context("new connector map")?;
        for actor in &dossier.actors {
            ensure!(
                actor.role == "optional_trainer"
                    && actor.id.starts_with("MN_COHESION_")
                    && actor.event_flag == format!("EVENT_{}_DEFEATED", actor.id)
                    && kind(pack, map, actor.tile)? == "FLOOR",
                "scoped optional connector trainer required"
            );
            ensure!(
                !map.objects.iter().any(|o| [o.x, o.y] == actor.tile),
                "connector actor overlap"
            );
            ensure!(
                pack.runtime_files.contains_key(&format!(
                    "gfx/sprites/{}.png",
                    actor
                        .sprite
                        .trim_start_matches("SPRITE_")
                        .to_ascii_lowercase()
                )),
                "unknown connector sprite"
            );
            let class = &actor.trainer_class;
            let next = pack
                .data
                .trainers
                .trainers
                .values()
                .filter(|t| &t.trainer_class == class)
                .filter_map(|t| {
                    pack.data
                        .story_event_script_constants
                        .global
                        .get(&t.trainer_id)
                })
                .copied()
                .max()
                .context("native trainer class namespace")?
                .checked_add(1)
                .context("trainer namespace overflow")?;
            ensure!(
                next <= 255
                    && !pack
                        .data
                        .story_event_script_constants
                        .global
                        .contains_key(&actor.id),
                "occupied connector trainer identity"
            );
            pack.data
                .story_event_script_constants
                .global
                .insert(actor.id.clone(), next);
            let trainer: RouteTrainer = serde_json::from_value(json!({
                "id":actor.id,"display_name":actor.trainer_name,"trainer_class":class,
                "sprite":actor.sprite,"tile":actor.tile,"facing":match actor.facing { crate::authored_route_design::Facing::North=>"north", crate::authored_route_design::Facing::South=>"south", crate::authored_route_design::Facing::East=>"east", crate::authored_route_design::Facing::West=>"west" },
                "sight_range":0,"defeated_flag":actor.event_flag,
                "party":actor.party.iter().map(|p|json!({"species":p.species,"level":p.level})).collect::<Vec<_>>(),
                "before_lines":actor.text,"after_lines":["Safe travels!"],"loss_lines":["A good forest walk!"]
            }))?;
            events::activities(pack, map, &[trainer], &[])?;
            let object = map.objects.last_mut().context("new optional trainer")?;
            object.object_identifier = Some(actor.id.clone());
        }
        crate::authored_town_scripts::ordinary(pack, map, &[], &dossier.signs, "", "")?;
        events::finish(pack, map, &names)?;
    }
    let guarded = plan
        .guard_proposals
        .iter()
        .map(|g| &g.map)
        .collect::<BTreeSet<_>>();
    for name in guarded {
        let city = maps.get_mut(name).context("existing checkpoint map")?;
        let constant = city
            .attributes
            .map_constant
            .as_deref()
            .context("existing map constant")?;
        let mut additions = authoring::blank(pack, &city.id, &city.id, constant)?;
        additions.attributes = city.attributes.clone();
        additions.blocks = city.blocks.clone();
        // Build every new root for this city before native parsing. Native
        // typed indexes follow the complete ordered raw-script dictionary;
        // parsing each throat separately would append West then East while a
        // true split reload parses East then West. Never reorder old indexes.
        for g in plan.guard_proposals.iter().filter(|g| g.map == *name) {
            checkpoint(
                pack,
                &mut additions,
                &Checkpoint {
                    identifiers: &g.actor_ids,
                    bodies: g.bodies,
                    inside: g.inside,
                    outside: g.outside,
                    root: &g.id,
                    incoming_facing: &g.facing,
                    outward: &g.side,
                    required_event: &g.event,
                    badge_flags: &g.flags.iter().map(String::as_str).collect::<Vec<_>>(),
                    badge_lines: &g.denial_text,
                    evidence_lines: &g.denial_text,
                },
            )?;
        }
        // These are new always-active rows. Match the definitive native wire
        // token now; every original coordinate row remains unchanged in city.
        for coordinate in &mut additions.events.coord_events {
            ensure!(
                coordinate.scene_id.is_empty(),
                "new checkpoint scene contract"
            );
            coordinate.scene_id = "-1".into();
        }
        events::finish(pack, &mut additions, &names)?;
        // The shared merger deliberately omits coordinate rows; append only these
        // reviewed new rows, leaving the old coordinate table in exact order.
        let coordinates = additions.events.coord_events.clone();
        let control = city.script_control_commands.len();
        let flags = city.script_flag_commands.len();
        let text = city.script_text_commands.len();
        merge_new_content(city, additions)?;
        // These three tables already contain ordinary Grand Marais commands.
        // Its new MnCohesion roots sort before MnGrand roots in native parsing.
        // Insert only the new rows at their native positions; never reparse or
        // reorder the existing rows or change their command identities.
        let incoming = city.script_control_commands.split_off(control);
        insert_new_rows(&mut city.script_control_commands, incoming, |row| {
            (&row.source_script, row.command_index)
        });
        let incoming = city.script_flag_commands.split_off(flags);
        insert_new_rows(&mut city.script_flag_commands, incoming, |row| {
            (&row.source_script, row.command_index)
        });
        let incoming = city.script_text_commands.split_off(text);
        insert_new_rows(&mut city.script_text_commands, incoming, |row| {
            (&row.source_script, row.command_index)
        });
        city.events.coord_events.extend(coordinates);
        authoring::refresh_event_section(city);
    }
    Ok(())
}

fn insert_new_rows<T>(original: &mut Vec<T>, added: Vec<T>, key: impl Fn(&T) -> (&str, usize)) {
    for row in added {
        let at = original
            .iter()
            .position(|old| key(old) > key(&row))
            .unwrap_or(original.len());
        original.insert(at, row);
    }
}
