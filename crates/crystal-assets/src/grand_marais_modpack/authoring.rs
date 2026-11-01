//! Small authoring helpers using the pack compiler's existing script parsers.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::map::{MapEventSectionCommand, ObjectEvent, WarpEvent};
use serde_json::{Value, json};

use crate::{CompiledGamePack, MapModule};

pub(crate) fn blank(
    base: &CompiledGamePack,
    template: &str,
    name: &str,
    constant: &str,
) -> Result<MapModule> {
    let source = base
        .data
        .maps
        .get(template)
        .with_context(|| format!("missing {template} template"))?;
    let mut value = serde_json::to_value(source)?;
    for (key, field) in value.as_object_mut().context("map module object")? {
        if matches!(key.as_str(), "id" | "attributes" | "blocks") {
            continue;
        }
        *field = match field {
            Value::Array(_) => json!([]),
            Value::Object(_) => json!({}),
            _ => anyhow::bail!("unexpected map field {key}"),
        };
    }
    value["scenes"] = json!({"scenes":[]});
    value["events"] = json!({"warps":[],"coord_events":[],"bg_events":[]});
    let mut module: MapModule = serde_json::from_value(value)?;
    module.id = name.into();
    let a = &mut module.attributes;
    a.map_constant = Some(constant.into());
    a.map_group_constant = Some("MAPGROUP_MINNESOTA".into());
    a.blocks_label = Some(format!("{name}_Blocks"));
    a.map_scripts_label = Some(format!("{name}_MapScripts"));
    a.map_events_label = Some(format!("{name}_MapEvents"));
    a.connections.clear();
    a.connection_flags = None;
    Ok(module)
}

pub(crate) fn blocks(module: &mut MapModule, width: u16, height: u16, text: &str) -> Result<()> {
    module.attributes.width = width;
    module.attributes.height = height;
    module.blocks = text
        .split_whitespace()
        .map(|v| u16::from_str_radix(v, 16).context("authored metatile"))
        .collect::<Result<_>>()?;
    ensure!(
        module.blocks.len() == usize::from(width) * usize::from(height),
        "{} layout dimensions",
        module.id
    );
    Ok(())
}

pub(crate) fn command(name: &str, args: &[&str]) -> Value {
    json!({"command":name,"args":args})
}

pub(crate) fn text(module: &mut MapModule, label: &str, lines: &[&str]) {
    // Crystal's text box has 18 interior character columns. Keep authored
    // sentences readable in the bitmap client as well as the wider terminal.
    let mut wrapped = Vec::new();
    for line in lines {
        let mut row = String::new();
        for word in line.split_whitespace() {
            if !row.is_empty() && row.chars().count() + 1 + word.chars().count() > 18 {
                wrapped.push(row);
                row = String::new();
            }
            if !row.is_empty() {
                row.push(' ');
            }
            row.push_str(word);
        }
        if !row.is_empty() {
            wrapped.push(row);
        }
    }
    let lines = wrapped;
    let mut commands = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        commands.push(command(
            if index == 0 {
                "text"
            } else if index % 2 == 0 {
                "para"
            } else {
                "line"
            },
            &[&format!("\"{line}\"")],
        ));
    }
    commands.push(command("done", &[]));
    module.scripts.insert(label.into(), Value::Array(commands));
}

pub(crate) fn npc(
    base: &CompiledGamePack,
    module: &mut MapModule,
    script: &str,
    sprite: &str,
    x: i16,
    y: i16,
) -> Result<()> {
    let mut object: ObjectEvent = base.data.maps["NewBarkTown"].objects[0].clone();
    object.sprite = sprite.into();
    object.x = u16::try_from(x)?;
    object.y = u16::try_from(y)?;
    object.spritemovedata = "SPRITEMOVEDATA_STANDING_DOWN".into();
    object.move_range_x = 0;
    object.move_range_y = 0;
    object.pal = 0;
    object.script = script.into();
    object.object_identifier = Some(format!(
        "{}_NPC{}",
        module.id.to_ascii_uppercase(),
        module.objects.len() + 1
    ));
    object.event_flag = "-1".into();
    module.objects.push(object);
    Ok(())
}

pub(crate) fn speech(
    base: &CompiledGamePack,
    module: &mut MapModule,
    suffix: &str,
    sprite: &str,
    at: (i16, i16),
    lines: &[&str],
) -> Result<()> {
    let script = format!("{}{suffix}", module.id);
    let label = format!("{script}Text");
    module.scripts.insert(
        script.clone(),
        json!([command("jumptextfaceplayer", &[&label])]),
    );
    text(module, &label, lines);
    npc(base, module, &script, sprite, at.0, at.1)
}

pub(crate) fn warp(module: &mut MapModule, at: (u16, u16), target: &str, target_id: i16) {
    module.events.warps.push(WarpEvent {
        index: module.events.warps.len() as u16 + 1,
        x: at.0,
        y: at.1,
        target_map_constant: target.into(),
        target_map: target.into(),
        target_warp_id: target_id,
    });
}

pub(crate) fn finish(base: &CompiledGamePack, module: &mut MapModule) -> Result<()> {
    let name = &module.id;
    let scripts = &module.scripts;
    module.gift_pokemon_scripts =
        crate::parse_gift_pokemon_scripts(name, scripts, &base.data.story_event_script_constants)?;
    module.script_flag_commands = crate::parse_script_flag_commands(name, scripts)?;
    module.script_control_commands = crate::parse_script_control_commands(name, scripts)?;
    module.script_text_commands = crate::parse_script_text_commands(name, scripts)?;
    module.script_text_bodies = crate::parse_script_text_bodies(name, scripts)?;
    module.script_item_grants = crate::parse_script_item_grants(name, scripts)?;
    (module.script_item_checks, module.script_item_takes) =
        crate::parse_script_item_accesses(name, scripts)?;
    module.script_shop_commands = crate::parse_script_shop_commands(name, scripts)?;
    module.script_economy_commands = crate::parse_script_economy_commands(name, scripts)?;
    module.script_scene_commands = crate::parse_script_scene_commands(name, scripts)?;
    refresh_event_section(module);
    if module.map_script_section_commands.is_empty() {
        module.map_script_section_commands = serde_json::from_value(json!([
            {"command":"def_scene_scripts","args":[],"command_index":0},
            {"command":"def_callbacks","args":[],"command_index":1}
        ]))?;
    }
    Ok(())
}

/// Refreshes only event wire rows, preserving all compiled story programs.
pub(crate) fn refresh_event_section(module: &mut MapModule) {
    let mut sections = Vec::new();
    let mut add = |command: &str, args: Vec<String>| {
        sections.push(MapEventSectionCommand {
            command: command.into(),
            args,
            command_index: sections.len(),
        });
    };
    add("db", vec!["0".into(), "0".into()]);
    add("def_warp_events", vec![]);
    for w in &module.events.warps {
        add(
            "warp_event",
            vec![
                w.x.to_string(),
                w.y.to_string(),
                w.target_map_constant.clone(),
                w.target_warp_id.to_string(),
            ],
        );
    }
    add("def_coord_events", vec![]);
    for e in &module.events.coord_events {
        // Typed always-active events use an empty scene; native wire commands
        // require the actual -1 token rather than an empty command argument.
        add(
            "coord_event",
            vec![
                e.x.to_string(),
                e.y.to_string(),
                if e.scene_id.is_empty() {
                    "-1".into()
                } else {
                    e.scene_id.clone()
                },
                e.script_name.clone(),
            ],
        );
    }
    add("def_bg_events", vec![]);
    for e in &module.events.bg_events {
        add(
            "bg_event",
            vec![
                e.x.to_string(),
                e.y.to_string(),
                e.event_type.clone(),
                e.script.clone(),
            ],
        );
    }
    add("def_object_events", vec![]);
    for o in &module.objects {
        add(
            "object_event",
            vec![
                o.x.to_string(),
                o.y.to_string(),
                o.sprite.clone(),
                o.spritemovedata.clone(),
                o.move_range_x.to_string(),
                o.move_range_y.to_string(),
                o.hram_x.to_string(),
                o.hram_y.to_string(),
                o.pal.to_string(),
                o.object_type.clone(),
                o.radius.to_string(),
                o.script.clone(),
                o.event_flag.clone(),
            ],
        );
    }
    module.map_event_section_commands = sections;
}

pub(super) fn scripts(module: &mut MapModule, entries: impl IntoIterator<Item = (String, Value)>) {
    module.scripts = entries.into_iter().collect::<BTreeMap<_, _>>();
}
