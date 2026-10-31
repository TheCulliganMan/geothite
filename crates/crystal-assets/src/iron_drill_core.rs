//! A rock-cut operating drill bay retaining native choreography and decisions.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, MapModule, authored_town_geometry, iron_drill_core_art};

const ROOM: &str = "MnIronDrillCore";

#[derive(Deserialize)]
struct Terrain {
    schema_version: u8,
    id: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    border_block: u16,
    palette_group: String,
    rows: Vec<String>,
}
impl Terrain {
    fn blocks(&self) -> Result<Vec<u16>> {
        ensure!(
            self.rows.len() == 10,
            "complete drill chamber rows required"
        );
        let mut blocks = Vec::new();
        for row in &self.rows {
            let cells = row.split_whitespace().collect::<Vec<_>>();
            ensure!(cells.len() == 12, "complete drill chamber columns required");
            for cell in cells {
                blocks.push(u16::from_str_radix(cell, 16).context("native drill chamber block")?);
            }
        }
        Ok(blocks)
    }
}

/// Rebuilds the complete drill core around a native mechanical bay in rock.
/// Preserves every original actor, port, state gate, drilling frame, shutdown
/// and recovery decision. Only the two eastern witness movements detour below
/// the original guard; both end at the original camera pose.
///
/// The private art bank preserves numeric18/22 as solid original machine
/// frames. All other maps, native source graphics and audio remain intact.
///
/// # Errors
/// Rejects stale complete-module/terrain inputs, unsupported machine art,
/// changed ports/actors, obstructed choreography or invalid resulting content.
pub fn rebuild_iron_drill_core(
    base: &CompiledGamePack,
    before: &str,
    after: &str,
    contract: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let old: Terrain = serde_json::from_str(before)?;
    let new: Terrain = serde_json::from_str(after)?;
    let design: Value = serde_json::from_str(after)?;
    let contract: Value = serde_json::from_str(contract)?;
    let original = base
        .data
        .maps
        .get(ROOM)
        .context("current drilling chamber")?;
    ensure!(
        contract["schema_version"] == 1
            && contract["map"] == ROOM
            && contract["native_module_sha256"]
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
                )
            && contract["preserved_warps"] == serde_json::to_value(&original.events.warps)?
            && old.schema_version == 1
            && new.schema_version == 1
            && old.id == ROOM
            && new.id == ROOM
            && old.dimensions_metatiles == [12, 10]
            && new.dimensions_metatiles == [12, 10]
            && old.tileset == "facility"
            && old.palette_group == "day"
            && old.border_block == 23
            && new.tileset == iron_drill_core_art::ALIAS
            && new.palette_group == "nite"
            && new.border_block == 46
            && original.attributes.width == 12
            && original.attributes.height == 10
            && original.attributes.tileset_name == old.tileset
            && original.blocks == old.blocks()?,
        "complete drill chamber preimage differs"
    );
    ensure!(
        original
            .objects
            .iter()
            .map(|o| (o.script.as_str(), o.x, o.y))
            .eq([
                ("MN_NDRILL_REED", 8, 8),
                ("MN_NDRILL_ENGINEER", 8, 5),
                ("MN_NDRILL_FINN", 15, 9),
                ("MN_LODE_ARC_CORE_WEST", 5, 11),
                ("MN_LODE_ARC_CORE_EAST", 18, 12)
            ]),
        "original crew, rival and guard anchors required"
    );
    ensure!(design["default_hidden_actors"] == json!(["MN_NDRILL_FINN"])
        && design["movement_detours"] == json!((19..=20).map(|x|json!({
            "script":format!("MnNDrillWatchFrom{x}"),"start":[x,12],"steps":[["DOWN",1],["LEFT",x-9],["UP",4]],"end":[9,9],
            "reason":"Pass below the original core-east guard at18,12 rather than through its body."
        })).collect::<Vec<_>>()), "declared hidden crew and eastern witness detours required");
    let label = original
        .attributes
        .blocks_label
        .as_ref()
        .context("drill block label")?;
    ensure!(
        base.data
            .maps
            .values()
            .filter(|m| m.attributes.blocks_label.as_ref() == Some(label))
            .count()
            == 1
            && crate::decode_base64_bytes(&base.data.map_blocks[label])?
                .into_iter()
                .map(u16::from)
                .eq(original.blocks.iter().copied()),
        "unique consistent drill block mirror required"
    );
    let mut pack = iron_drill_core_art::install(base, &design)?;
    let mut room = original.clone();
    room.blocks = new.blocks()?;
    room.attributes.tileset_name = iron_drill_core_art::ALIAS.into();
    room.attributes.border_block = 46;
    room.attributes.time_of_day = Some("nite".into());
    room.attributes.palette = Some("PALETTE_NITE".into());
    room.attributes.fishing_group = Some("FISHGROUP_MN_LAKE".into());
    for x in [19, 20] {
        let key = format!("MnNDrillWatchFrom{x}");
        let mut expected = (0..x - 9)
            .map(|_| command("step", &["LEFT"]))
            .collect::<Vec<_>>();
        expected.extend((0..3).map(|_| command("step", &["UP"])));
        expected.push(command("step_end", &[]));
        ensure!(
            room.scripts.get(&key) == Some(&json!(expected)),
            "original eastern witness movement drift"
        );
        let mut steps = vec![command("step", &["DOWN"])];
        steps.extend((0..x - 9).map(|_| command("step", &["LEFT"])));
        steps.extend((0..4).map(|_| command("step", &["UP"])));
        steps.push(command("step_end", &[]));
        room.scripts.insert(key, json!(steps));
    }
    room.script_movements =
        crate::parse_script_movements(ROOM, &room.scripts, &room.script_object_commands)?;
    validate_geometry(&pack, &room)?;
    crate::authored_town_publication::replace(&mut pack, room)?;
    pack.report
        .manifests
        .push("minnesota-iron-core-rock-chamber-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}

fn validate_geometry(pack: &CompiledGamePack, room: &MapModule) -> Result<()> {
    let kind = |p| authored_town_geometry::kind(pack, room, p);
    let bodies = room
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let mut floor = BTreeSet::new();
    let mut ladders = BTreeSet::new();
    for y in 0..20 {
        for x in 0..24 {
            let at = [x, y];
            let k = kind(at)?;
            ensure!(
                matches!(k, "FLOOR" | "WALL" | "UP_WALL" | "LADDER"),
                "unsupported drill-bay collision {k}"
            );
            if x == 0 || x == 23 || y == 0 || y == 19 {
                ensure!(k == "WALL", "unconnected drill-bay boundary");
            }
            if k == "FLOOR" && !bodies.contains(&at) {
                floor.insert(at);
            }
            if k == "LADDER" {
                ladders.insert(at);
            }
        }
    }
    ensure!(
        ladders == BTreeSet::from([[5, 14]]) && kind([4, 15])? == "FLOOR",
        "original drill shaft and safe arrival required"
    );
    let mut seen = BTreeSet::from([[4, 15]]);
    let mut queue = VecDeque::from([[4u16, 15]]);
    while let Some([x, y]) = queue.pop_front() {
        for at in [
            x.checked_sub(1).map(|x| [x, y]),
            x.checked_add(1).map(|x| [x, y]),
            y.checked_sub(1).map(|y| [x, y]),
            y.checked_add(1).map(|y| [x, y]),
        ]
        .into_iter()
        .flatten()
        {
            if floor.contains(&at) && seen.insert(at) {
                queue.push_back(at);
            }
        }
    }
    ensure!(seen == floor, "orphan floor in drill chamber");
    for at in [[9, 9], [8, 9], [8, 6], [16, 3], [20, 13], [19, 13], [5, 15]] {
        ensure!(
            seen.contains(&at),
            "drill interaction/scene landing unreachable: {at:?}"
        );
    }
    for body in &bodies {
        ensure!(kind(*body)? == "FLOOR", "drill actor embedded in wall");
    }
    // Every legal coordinate-event entry moves on ordinary floor and avoids
    // the original stationary actor bodies, including the east guard.
    let starts = (3u16..=20)
        .filter(|x| *x != 18)
        .map(|x| (format!("MnNDrillWatchFrom{x}"), [x, 12]));
    let mut paths = starts.collect::<BTreeMap<_, _>>();
    paths.insert("MnNDrillPlayerToCrew".into(), [16, 3]);
    paths.insert("MnNDrillCrewOut".into(), [15, 9]);
    for (script, mut at) in paths {
        for row in room.scripts[&script].as_array().context("scene movement")? {
            if row["command"] == "step_end" {
                continue;
            }
            ensure!(row["command"] == "step", "unexpected scene movement opcode");
            match row["args"][0].as_str().context("scene direction")? {
                "LEFT" => at[0] = at[0].checked_sub(1).context("scene left edge")?,
                "RIGHT" => at[0] += 1,
                "UP" => at[1] = at[1].checked_sub(1).context("scene top edge")?,
                "DOWN" => at[1] += 1,
                _ => anyhow::bail!("unsupported scene direction"),
            }
            ensure!(
                kind(at)? == "FLOOR" && !bodies.contains(&at),
                "scene walks through rock or an actor: {script} {at:?}"
            );
        }
        ensure!(
            at == if script == "MnNDrillCrewOut" {
                [8, 7]
            } else {
                [9, 9]
            },
            "original scene endpoint changed"
        );
    }
    let def = &pack.data.tilesets[&room.attributes.tileset_name];
    for block in ["12", "16", "43", "44", "45"] {
        ensure!(
            def.collision[block].iter().all(|k| k == "WALL"),
            "moving drive must stay solid"
        );
    }
    ensure!(
        room.blocks[3 * 12 + 5] == 0x44
            && room.blocks[4 * 12 + 5] == 0x44
            && room.blocks[5 * 12 + 5] == 0x44
            && [3, 4, 5].iter().all(|y| room.blocks[y * 12 + 6] == 0x43)
            && room.blocks[12 + 8] == 0x46,
        "whole native drill rack and isolation console required"
    );
    Ok(())
}
