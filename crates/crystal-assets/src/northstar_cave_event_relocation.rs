//! Relocate a cave's existing ports and anchors without changing their identities.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use super::Layout;
use crate::{CompiledGamePack, MapModule};

type Tile = [u16; 2];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WarpMove {
    index: u16,
    from: Tile,
    to: Tile,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EventMove {
    script: String,
    from: Tile,
    to: Tile,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ScriptPatch {
    script: String,
    index: usize,
    before_args: Vec<String>,
    after_args: Vec<String>,
}

pub(super) fn apply(
    pack: &CompiledGamePack,
    original: &MapModule,
    room: &mut MapModule,
    layout: &Layout,
) -> Result<()> {
    let width = room.attributes.width * 2;
    let height = room.attributes.height * 2;
    let inside = |[x, y]: Tile| x < width && y < height;
    let mut moved = BTreeSet::new();
    for warp in &layout.warps {
        let pointer = match_position(
            original.events.warps.iter().map(|w| (w.index, [w.x, w.y])),
            warp.index,
            warp.from,
        )
        .with_context(|| format!("warp {} at {:?}", warp.index, warp.from))?;
        ensure!(
            moved.insert(pointer) && warp.from != warp.to,
            "duplicate or unchanged warp move"
        );
        let live = &mut room.events.warps[pointer];
        [live.x, live.y] = warp.to;
    }
    relocate(
        &layout.actors,
        &original.objects,
        &mut room.objects,
        |o| (&o.script, [o.x, o.y]),
        |o, at| [o.x, o.y] = at,
    )?;
    relocate(
        &layout.backgrounds,
        &original.events.bg_events,
        &mut room.events.bg_events,
        |e| (&e.script, [e.x, e.y]),
        |e, at| [e.x, e.y] = at,
    )?;
    relocate(
        &layout.coordinates,
        &original.events.coord_events,
        &mut room.events.coord_events,
        |e| (&e.script_name, [e.x, e.y]),
        |e, at| [e.x, e.y] = at,
    )?;

    let mut warps = room.events.warps.iter().map(|e| [e.x, e.y]);
    let mut actors = room.objects.iter().map(|e| [e.x, e.y]);
    let mut backgrounds = room.events.bg_events.iter().map(|e| [e.x, e.y]);
    let mut coordinates = room.events.coord_events.iter().map(|e| [e.x, e.y]);
    for row in &mut room.map_event_section_commands {
        let next = match row.command.as_str() {
            "warp_event" => warps.next(),
            "object_event" => actors.next(),
            "bg_event" => backgrounds.next(),
            "coord_event" => coordinates.next(),
            _ => continue,
        };
        let [x, y] = next.context("native event section has unmatched events")?;
        ensure!(
            row.args.len() >= 2 && inside([x, y]),
            "event outside replacement at ({x}, {y})"
        );
        row.args[0] = x.to_string();
        row.args[1] = y.to_string();
    }
    ensure!(
        warps.next().is_none()
            && actors.next().is_none()
            && backgrounds.next().is_none()
            && coordinates.next().is_none(),
        "native event section is missing anchors"
    );
    patch_scripts(pack, room, &layout.script_patches)?;
    for change in &room.script_block_changes {
        ensure!(
            inside([change.x, change.y]),
            "scripted change outside replacement: {}",
            change.source_script
        );
        let tileset = &pack.data.tilesets[&room.attributes.tileset_name];
        ensure!(
            tileset
                .collision
                .contains_key(&format!("{:02x}", change.block_id)),
            "unknown native mutation block {:02x}",
            change.block_id
        );
    }
    Ok(())
}

fn match_position<K: Eq>(
    points: impl Iterator<Item = (K, Tile)>,
    key: K,
    from: Tile,
) -> Result<usize> {
    let mut matches = points
        .enumerate()
        .filter(|(_, (id, tile))| *id == key && *tile == from);
    let (index, _) = matches.next().context("no exact native anchor")?;
    ensure!(matches.next().is_none(), "ambiguous native anchor");
    Ok(index)
}

fn relocate<T>(
    moves: &[EventMove],
    original: &[T],
    output: &mut [T],
    anchor: impl Fn(&T) -> (&String, Tile),
    set: impl Fn(&mut T, Tile),
) -> Result<()> {
    let mut moved = BTreeSet::new();
    for edit in moves {
        let index = match_position(original.iter().map(&anchor), &edit.script, edit.from)
            .with_context(|| format!("{} at {:?}", edit.script, edit.from))?;
        ensure!(
            moved.insert(index) && edit.from != edit.to,
            "duplicate or unchanged event move"
        );
        set(&mut output[index], edit.to);
    }
    Ok(())
}

fn patch_scripts(
    pack: &CompiledGamePack,
    room: &mut MapModule,
    patches: &[ScriptPatch],
) -> Result<()> {
    if patches.is_empty() {
        return Ok(());
    }
    let mut edited = BTreeSet::new();
    let mut text_changed = false;
    let mut control_changed = false;
    for patch in patches {
        ensure!(
            edited.insert((&patch.script, patch.index)),
            "duplicate script patch"
        );
        let entry = room
            .scripts
            .get_mut(&patch.script)
            .and_then(Value::as_array_mut)
            .and_then(|body| body.get_mut(patch.index))
            .with_context(|| format!("{} command {}", patch.script, patch.index))?;
        let name = entry["command"].as_str().context("native script command")?;
        ensure!(
            matches!(
                name,
                "changeblock"
                    | "moveobject"
                    | "moveobjectnoslide"
                    | "warp"
                    | "step"
                    | "slow_step"
                    | "fast_step"
                    | "big_step"
                    | "step_sleep"
                    | "ifequal"
                    | "ifnotequal"
                    | "text"
                    | "line"
                    | "para"
                    | "cont"
            ),
            "unsupported cave layout script patch: {name}"
        );
        ensure!(
            entry["args"] == json!(patch.before_args),
            "stale operands for {}",
            patch.script
        );
        ensure!(
            patch.after_args.len() == patch.before_args.len(),
            "script layout must retain command arity"
        );
        if matches!(name, "ifequal" | "ifnotequal") {
            ensure!(
                patch.after_args.len() == 2
                    && patch.after_args[0].parse::<u16>().is_ok()
                    && patch.before_args[1] == patch.after_args[1],
                "position comparison must keep its branch target"
            );
            control_changed = true;
        }
        if matches!(name, "text" | "line" | "para" | "cont") {
            ensure!(patch.after_args.len() == 1, "expected one native text line");
            let line = patch.after_args[0]
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .context("expected quoted native text")?;
            ensure!(
                !line.is_empty()
                    && line.len() <= 18
                    && line.is_ascii()
                    && !line.contains(['"', '<', '>'])
                    && !line.chars().any(char::is_control),
                "cave wayfinding text does not fit native text"
            );
            text_changed = true;
        }
        entry["args"] = json!(patch.after_args);
    }
    let name = &room.id;
    let scripts = &room.scripts;
    room.script_block_changes = crate::parse_script_block_changes(name, scripts)?;
    room.script_object_commands = crate::parse_script_object_commands(name, scripts)?;
    room.script_movements =
        crate::parse_script_movements(name, scripts, &room.script_object_commands)?;
    room.script_runtime_commands = crate::parse_script_runtime_commands(name, scripts)?;
    let native_names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|map| (map.constant.clone(), map.name.clone()))
        .collect();
    room.script_map_commands = crate::parse_script_map_commands(name, scripts, &native_names)?;
    if control_changed {
        room.script_control_commands = crate::parse_script_control_commands(name, scripts)?;
    }
    if text_changed {
        room.script_text_commands = crate::parse_script_text_commands(name, scripts)?;
        room.script_text_bodies = crate::parse_script_text_bodies(name, scripts)?;
    }
    Ok(())
}
