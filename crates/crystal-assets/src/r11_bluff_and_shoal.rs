//! Apply the native Route 11 bluff passage and signed Split Rock Surf detour.
use anyhow::{Context, Result, ensure};
use crystal_core::map::{BackgroundEvent, MapEventSectionCommand};
use serde::Deserialize;

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule, authored_town_publication};

const SOURCE: &str = include_str!("../../../planning/minnesota/routes/MN_R11.bluff-and-shoal.json");

#[derive(Deserialize)]
struct Exterior {
    map: String,
    cells: Vec<Cell>,
    sign: Option<Sign>,
}

#[derive(Deserialize)]
struct Cell(u16, u16, String, String);

#[derive(Deserialize)]
struct Sign {
    tile: [u16; 2],
    script: String,
    lines: Vec<String>,
}

/// Builds a Route 11 successor with a bluff passage, Surf landing and two signs.
///
/// Uses the existing warm tileset and compact connections. The lighthouse, mooring
/// ring, trainers, encounters and story programs are inherited from the parent.
///
/// # Errors
///
/// Returns an error for a stale terrain preimage, an occupied event anchor or an
/// invalid parent or resulting pack.
pub fn build_r11_bluff_and_shoal_pack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let exteriors: Vec<Exterior> = serde_json::from_str(SOURCE)?;
    let mut output = base.clone();
    for exterior in exteriors {
        let mut map = base
            .data
            .maps
            .get(&exterior.map)
            .with_context(|| format!("missing Route 11 exterior {}", exterior.map))?
            .clone();
        let blocks_label = map
            .attributes
            .blocks_label
            .as_ref()
            .context("Route 11 blocks")?;
        ensure!(
            base.data.maps.values().all(|other| {
                other.id == map.id || other.attributes.blocks_label.as_ref() != Some(blocks_label)
            }),
            "shared block payload for {}",
            map.id
        );
        for Cell(x, y, before, after) in exterior.cells {
            ensure!(
                x < map.attributes.width && y < map.attributes.height,
                "metatile outside {} at {x},{y}",
                map.id
            );
            ensure!(
                map.objects.iter().all(|o| (o.x / 2, o.y / 2) != (x, y))
                    && map
                        .events
                        .warps
                        .iter()
                        .all(|w| (w.x / 2, w.y / 2) != (x, y))
                    && map
                        .events
                        .bg_events
                        .iter()
                        .all(|e| (e.x / 2, e.y / 2) != (x, y))
                    && map
                        .events
                        .coord_events
                        .iter()
                        .all(|e| (e.x / 2, e.y / 2) != (x, y))
                    && map
                        .script_block_changes
                        .iter()
                        .all(|e| (e.x / 2, e.y / 2) != (x, y)),
                "occupied metatile in {} at {x},{y}",
                map.id
            );
            let index = usize::from(y) * usize::from(map.attributes.width) + usize::from(x);
            let cell = map.blocks.get_mut(index).context("Route 11 block array")?;
            ensure!(
                *cell == u16::from_str_radix(&before, 16)?,
                "terrain preimage drift in {} at {x},{y}",
                map.id
            );
            *cell = u16::from_str_radix(&after, 16)?;
        }
        if let Some(sign) = exterior.sign {
            add_sign(base, &mut map, sign)?;
        }
        authored_town_publication::replace(&mut output, map)?;
    }
    output.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        output.format_version,
        &output.data,
        &output.audio_manifest,
        &output.runtime_files,
        &output.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    crate::validate_compact_superior_geography(&output)?;
    Ok(output)
}

fn add_sign(base: &CompiledGamePack, map: &mut MapModule, sign: Sign) -> Result<()> {
    let [x, y] = sign.tile;
    let text = format!("{}Text", sign.script);
    ensure!(
        [&sign.script, &text]
            .iter()
            .all(|&label| !base.data.map_scripts.contains_key(label)),
        "occupied Route 11 sign label"
    );
    let constant = map
        .attributes
        .map_constant
        .as_deref()
        .context("Route 11 map constant")?;
    let mut added = authoring::blank(base, &map.id, &map.id, constant)?;
    authoring::text(
        &mut added,
        &text,
        &sign.lines.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    added.scripts.insert(
        sign.script.clone(),
        serde_json::json!([authoring::command("jumptext", &[&text])]),
    );
    authoring::finish(base, &mut added)?;
    map.scripts.extend(added.scripts);
    map.script_text_commands.extend(added.script_text_commands);
    map.script_text_bodies.extend(added.script_text_bodies);
    map.events.bg_events.push(BackgroundEvent {
        x,
        y,
        event_type: "BGEVENT_READ".into(),
        script: sign.script.clone(),
    });
    // Insert a single background row and retain the original raw event operands.
    let sections = &mut map.map_event_section_commands;
    let index = sections
        .iter()
        .position(|row| row.command == "def_object_events")
        .context("Route 11 object event section")?;
    sections.insert(
        index,
        MapEventSectionCommand {
            command: "bg_event".into(),
            args: vec![
                x.to_string(),
                y.to_string(),
                "BGEVENT_READ".into(),
                sign.script,
            ],
            command_index: index,
        },
    );
    for (index, row) in sections.iter_mut().enumerate().skip(index + 1) {
        row.command_index = index;
    }
    Ok(())
}
