//! Shared native publication primitives for the two authored story lairs.
use anyhow::{Context, Result, ensure};
use crystal_core::map::{BackgroundEvent, MapScriptSectionCommand};
use crystal_core::models::TrainerPartyPokemon;
use serde_json::Value;

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::{CompiledGamePack, CompiledMapExtension, MapModule, RuntimeSpawnPoint};

pub(crate) fn flag(pack: &mut CompiledGamePack, name: &str) -> Result<()> {
    let g = &mut pack.data.story_event_script_constants.global;
    ensure!(!g.contains_key(name), "occupied story identity {name}");
    let n = g
        .iter()
        .filter(|(k, _)| k.starts_with("EVENT_"))
        .map(|(_, n)| *n)
        .max()
        .context("native events")?
        .checked_add(1)
        .context("event overflow")?;
    u16::try_from(n)?;
    g.insert(name.into(), n);
    Ok(())
}

pub(crate) fn trainer(
    pack: &mut CompiledGamePack,
    id: &str,
    class: &str,
    name: &str,
    party: &[(&str, u8)],
) -> Result<()> {
    ensure!(
        !pack.data.trainers.trainers.contains_key(id),
        "occupied trainer {id}"
    );
    let mut t = pack
        .data
        .trainers
        .trainers
        .values()
        .find(|t| t.trainer_class == class)
        .context("native trainer class")?
        .clone();
    let n = pack
        .data
        .trainers
        .trainers
        .values()
        .filter(|t| t.trainer_class == class)
        .filter_map(|t| {
            pack.data
                .story_event_script_constants
                .global
                .get(&t.trainer_id)
        })
        .copied()
        .max()
        .context("class identities")?
        + 1;
    ensure!(n <= 255, "trainer class is full");
    let dvs = t.party.first().context("class DVs")?.dvs;
    t.trainer_id = id.into();
    t.name = name.into();
    t.party = party
        .iter()
        .map(|&(species, level)| {
            Ok(TrainerPartyPokemon {
                species: species.into(),
                level,
                item: None,
                moves: pack.data.create_pokemon(species, level, dvs)?.moves,
                dvs,
            })
        })
        .collect::<Result<_>>()?;
    t.win_quote.clear();
    t.lose_quote.clear();
    pack.data.trainers.insert(t)?;
    ensure!(
        pack.data
            .story_event_script_constants
            .global
            .insert(id.into(), n)
            .is_none(),
        "occupied trainer alias"
    );
    Ok(())
}

pub(crate) fn room(pack: &CompiledGamePack, source: &Value, constant: &str) -> Result<MapModule> {
    let id = source["id"].as_str().context("lair id")?;
    let mut map = authoring::blank(
        pack,
        source["template"].as_str().context("native template")?,
        id,
        constant,
    )?;
    let [w, h]: [u16; 2] = serde_json::from_value(source["dimensions_metatiles"].clone())?;
    let rows: Vec<String> = serde_json::from_value(source["rows"].clone())?;
    ensure!(
        rows.len() == usize::from(h)
            && rows
                .iter()
                .all(|r| r.split_whitespace().count() == usize::from(w)),
        "lair source rows differ"
    );
    authoring::blocks(&mut map, w, h, &rows.join("\n"))?;
    map.attributes.border_block =
        u8::from_str_radix(source["border_block"].as_str().context("solid border")?, 16)?;
    ensure!(
        map.attributes.tileset_name == source["tileset"].as_str().context("native tileset")?,
        "lair tileset differs"
    );
    Ok(map)
}

pub(crate) fn actor(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    id: &str,
    sprite: &str,
    at: [u16; 2],
) -> Result<()> {
    ensure!(
        !map.objects.iter().any(|o| [o.x, o.y] == at),
        "overlapping actor {id}"
    );
    authoring::npc(
        pack,
        map,
        id,
        sprite,
        i16::try_from(at[0])?,
        i16::try_from(at[1])?,
    )?;
    let o = map.objects.last_mut().context("new actor")?;
    o.object_identifier = Some(id.into());
    Ok(())
}

pub(crate) fn guard(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    source: &Value,
    id: &str,
    at: [u16; 2],
    won: &str,
) -> Result<()> {
    let seen = format!("{id}SeenText");
    let win = format!("{id}WonText");
    let loss = format!("{id}LostText");
    let after = format!("{id}AfterText");
    let done = format!("{id}After");
    text(map, source, &[&seen, &win, &loss, &after])?;
    actor(pack, map, id, "SPRITE_ROCKET", at)?;
    let mut commands = vec![
        c("faceplayer", &[]),
        c("checkevent", &[won]),
        c("iftrue", &[&done]),
    ];
    commands.extend(say(&seen));
    commands.extend([
        c("winlosstext", &[&win, &loss]),
        c("loadtrainer", &["GRUNTM", id]),
        c("startbattle", &[]),
        c("reloadmapafterbattle", &[]),
        c("setevent", &[won]),
        c("sjump", &[&done]),
    ]);
    ensure!(
        map.scripts
            .insert(id.into(), serde_json::json!(commands))
            .is_none(),
        "occupied guard {id}"
    );
    map.scripts
        .insert(done, serde_json::json!([c("jumptext", &[&after])]));
    Ok(())
}

pub(crate) fn text(map: &mut MapModule, source: &Value, labels: &[&str]) -> Result<()> {
    for label in labels {
        let lines: Vec<String> = serde_json::from_value(source["texts"][label].clone())
            .with_context(|| format!("story text {label}"))?;
        ensure!(
            !lines.is_empty()
                && lines.iter().all(|l| !l.is_empty()
                    && l.len() <= 18
                    && l.is_ascii()
                    && !l.chars().any(|v| v.is_control() || v == '"')),
            "native text width/font: {label}"
        );
        ensure!(!map.scripts.contains_key(*label), "occupied text {label}");
        authoring::text(
            map,
            label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
    Ok(())
}

pub(crate) fn say(label: &str) -> Vec<Value> {
    vec![
        c("opentext", &[]),
        c("writetext", &[label]),
        c("waitbutton", &[]),
        c("closetext", &[]),
    ]
}

pub(crate) fn read(map: &mut MapModule, at: [u16; 2], script: &str) {
    map.events.bg_events.push(BackgroundEvent {
        x: at[0],
        y: at[1],
        event_type: "BGEVENT_READ".into(),
        script: script.into(),
    });
}

pub(crate) fn callback(map: &mut MapModule, kind: &str, script: &str) {
    if map.map_script_section_commands.is_empty() {
        for command in ["def_scene_scripts", "def_callbacks"] {
            map.map_script_section_commands
                .push(MapScriptSectionCommand {
                    command: command.into(),
                    args: vec![],
                    command_index: map.map_script_section_commands.len(),
                });
        }
    }
    map.map_script_section_commands
        .push(MapScriptSectionCommand {
            command: "callback".into(),
            args: vec![kind.into(), script.into()],
            command_index: map.map_script_section_commands.len(),
        });
}

pub(crate) fn publish(
    pack: &mut CompiledGamePack,
    mut map: MapModule,
    anchor: &str,
    tile: [u16; 2],
) -> Result<()> {
    crate::ely_league_rooms::finish(pack, &mut map)?;
    map.script_audio_commands = crate::parse_script_audio_commands(&map.id, &map.scripts)?;
    let a = &pack.data.maps[anchor].attributes;
    let mut meta =
        pack.data.runtime_map_metadata[a.map_constant.as_ref().context("anchor constant")?].clone();
    meta.map_id = pack
        .data
        .runtime_map_metadata
        .values()
        .filter(|m| m.group_id == meta.group_id)
        .map(|m| m.map_id)
        .max()
        .context("anchor group")?
        + 1;
    meta.name = map.id.clone();
    meta.constant = map
        .attributes
        .map_constant
        .clone()
        .context("new constant")?;
    meta.width = map.attributes.width;
    meta.height = map.attributes.height;
    meta.environment = map
        .attributes
        .environment
        .clone()
        .context("room environment")?;
    map.attributes.map_group_constant = a.map_group_constant.clone();
    map.attributes.location = a.location.clone();
    let landmark = pack
        .data
        .pokegear_landmarks
        .map_to_landmark
        .get(anchor)
        .context("anchor landmark mapping")?
        .clone();
    ensure!(
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(map.id.clone(), landmark)
            .is_none(),
        "occupied lair landmark mapping"
    );
    let identifier = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("recovery catalog")?
        .checked_add(1)
        .context("recovery overflow")?;
    let spawn = RuntimeSpawnPoint {
        identifier,
        map_name: meta.name.clone(),
        map_constant: meta.constant.clone(),
        group_name: meta.group_name.clone(),
        group_id: i16::try_from(meta.group_id)?,
        map_id: i16::try_from(meta.map_id)?,
        tile_x: i16::try_from(tile[0])?,
        tile_y: i16::try_from(tile[1])?,
        metatile_x: i16::try_from(tile[0] / 2)?,
        metatile_y: i16::try_from(tile[1] / 2)?,
        subtile_x: i16::try_from(tile[0] % 2)?,
        subtile_y: i16::try_from(tile[1] % 2)?,
    };
    let extension = CompiledMapExtension {
        manifest_id: format!("northstar-lair-{}", map.id.to_ascii_lowercase()),
        map_name: meta.name.clone(),
        map_constant: meta.constant.clone(),
        metadata: meta,
        spawn_key: identifier.to_string(),
        spawn,
        module: map,
        wild_encounters: None,
        start_new_game_here: false,
    };
    crate::validate_compiled_map_extension(&extension)?;
    crate::insert_compiled_map_extension(pack, extension)?;
    // A dungeon stair is not a new blackout or Fly destination.
    pack.data
        .runtime_spawn_points
        .remove(&identifier.to_string());
    Ok(())
}
