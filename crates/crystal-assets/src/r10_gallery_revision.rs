//! Bounded installed-room revision; terrain does not reinstall story programs.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, OccupiedTile, PlayerMovementState, StepOptions, StepOutcome,
    attempt_ledge_jump_with_occupied_tiles, attempt_step_with_occupied_tiles,
};
use serde_json::{Value, json};

use crate::authored_route_design::Interior;
use crate::authored_route_modpack::{collision, validate_room};
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnR10Cave1F";
const EXTERIOR: &str = "MnR10C00R00";
const READER: &str = "MN_R10_GALLERY_READER";
const PICKUP: &str = "MN_R10_ITEM_04";
const MANIFEST: &str = "minnesota-r10-seam-gallery-v1";

/// Revises only the installed R10 cave from its original12×8 room to the
/// hand-authored18×14 seam-gallery candidate. The original resident record,
/// dialogue, Revive program/flag/quantity and paired mouth/passive return stay
/// exact; only the pickup object moves from17,5 to29,23. Dimensions, blocks,
/// metadata, owned raw object coordinates and their split mirrors are updated
/// atomically. Full production uncached assembly reconstructs all typed indexes.
/// Original exterior maps, scripts, audio, art, HOME/recovery and story remain
/// unchanged. Existing saves and terrain overrides require separate compatibility
/// review. This does not publish a pack or establish controller/visual approval.
///
/// # Errors
/// Returns an error for invalid or stale before/after sources, installed native
/// identity/geometry/script/split-payload drift, shared owned labels, unsafe
/// revised room/stairs/ledge/water, or failed complete native pack verification.
/// The borrowed base and source strings are unchanged on every failure.
pub fn revise_r10_gallery_modpack(
    base: &CompiledGamePack,
    previous_room_json: &str,
    revised_room_json: &str,
) -> Result<CompiledGamePack> {
    let previous_value: Value =
        serde_json::from_str(previous_room_json).context("previous R10 room")?;
    let revised_value: Value =
        serde_json::from_str(revised_room_json).context("revised R10 room")?;
    validate_source_contract(&previous_value, &revised_value)?;
    let previous: Interior = serde_json::from_value(previous_value)?;
    let revised: Interior = serde_json::from_value(revised_value)?;
    let previous_blocks = blocks(&previous)?;
    let revised_blocks = blocks(&revised)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|id| id == MANIFEST),
        "R10 gallery revision already installed"
    );
    let original = base.data.maps.get(MAP).context("installed R10 cave")?;
    validate_preimage(base, original, &previous, &previous_blocks)?;
    let mut module = original.clone();
    module.blocks = revised_blocks;
    module.attributes.width = 18;
    module.attributes.height = 14;
    let pickup = module
        .objects
        .iter_mut()
        .find(|o| o.script == PICKUP)
        .context("existing Revive object")?;
    pickup.x = 29;
    pickup.y = 23;
    let rows = module
        .map_event_section_commands
        .iter_mut()
        .filter(|command| {
            command.command == "object_event" && command.args.get(11).is_some_and(|id| id == PICKUP)
        })
        .collect::<Vec<_>>();
    ensure!(rows.len() == 1, "one native Revive raw object row required");
    let command = rows
        .into_iter()
        .next()
        .context("validated Revive raw row")?;
    ensure!(
        command.args.len() == 13 && command.args[0] == "17" && command.args[1] == "5",
        "stale Revive raw coordinates"
    );
    command.args[0] = "29".into();
    command.args[1] = "23".into();
    validate_room(base, &module, &revised).context("complete revised gallery geometry")?;
    validate_features(base, &module)?;
    let mut result = base.clone();
    let constant = original
        .attributes
        .map_constant
        .as_ref()
        .context("existing cave constant")?;
    let block_label = original
        .attributes
        .blocks_label
        .as_ref()
        .context("existing block label")?;
    let scripts_label = original
        .attributes
        .map_scripts_label
        .as_ref()
        .context("existing script section")?;
    let events_label = original
        .attributes
        .map_events_label
        .as_ref()
        .context("existing event section")?;
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    result
        .data
        .map_blocks
        .insert(block_label.clone(), crate::encode_base64_bytes(&bytes));
    result
        .data
        .map_dimensions
        .insert(constant.clone(), json!({"width":18,"height":14}));
    result
        .data
        .map_attributes
        .insert(MAP.into(), module.attributes.clone());
    let metadata = result
        .data
        .runtime_map_metadata
        .get_mut(constant)
        .context("existing cave metadata")?;
    metadata.width = 18;
    metadata.height = 14;
    // A cached module may contain its transport section in `scripts`; update
    // that exact owned payload before refreshing split storage, never a program.
    let event_wire = Value::Array(
        module
            .map_event_section_commands
            .iter()
            .map(|c| json!({"command":c.command,"args":c.args}))
            .collect(),
    );
    if let Some(payload) = module.scripts.get_mut(events_label) {
        *payload = event_wire;
    }
    crate::insert_compiled_map_split_payloads(
        &mut result.data.map_scripts,
        &mut result.data.npcs,
        MAP,
        scripts_label,
        events_label,
        &module,
    )?;
    // Reconstruct every typed script/runtime/menu/object/pickup index through
    // the production assembler. authoring::finish is not a complete lowering API.
    result.data.maps.remove(MAP);
    let assembled = result
        .data
        .assemble_map_module_from_compiled_payloads(MAP)?;
    let mut expected = module;
    normalize_transport(&result.data, &mut expected)?;
    ensure!(
        assembled == expected,
        "R10 production assembly changed an existing executable index"
    );
    result.data.maps.insert(MAP.into(), assembled);
    result.report.manifests.push(MANIFEST.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}

fn validate_source_contract(old: &Value, new: &Value) -> Result<()> {
    ensure!(
        old["id"] == MAP
            && new["id"] == MAP
            && old["template"] == "UnionCave1F"
            && new["template"] == old["template"]
            && old["dimensions_metatiles"] == json!([12, 8])
            && new["dimensions_metatiles"] == json!([18, 14]),
        "exact R10 room generations required"
    );
    for source in [old, new] {
        ensure!(
            source["exits"] == json!([[3, 13]])
                && source["clear_arrival"] == json!([3, 12])
                && source["exit_collision"] == "LADDER"
                && source["border_block"] == 1
                && source["signs"] == json!([])
                && source.get("passive_arrival").is_none()
                && source.get("exit_doors").is_none_or(|v| v == &json!([]))
                && source
                    .get("native_arrival")
                    .is_none_or(|v| v == &json!([3, 13]))
                && source.get("keeper_tile").is_none()
                && source.get("derived_blocks").is_none()
                && source.get("tileset").is_none_or(|v| v == "cave")
                && source
                    .get("source_pixels_modified")
                    .is_none_or(|v| v == false),
            "retain the original mouth, source cave art and actor roster"
        );
        ensure!(
            source["residents"].as_array().is_some_and(|v| v.len() == 1)
                && source["items"].as_array().is_some_and(|v| v.len() == 1),
            "one reader and one original reward only"
        );
    }
    ensure!(
        old["residents"] == new["residents"]
            && old["residents"][0]["id"] == READER
            && old["residents"][0]["tile"] == json!([9, 5])
            && old["residents"][0]["sprite"] == "SPRITE_SUPER_NERD"
            && old["residents"][0]["facing"] == "east"
            && old["residents"][0]["lines"]
                == json!([
                    "Both passages",
                    "circle the pillar.",
                    "The foreman was",
                    "at the west mine."
                ]),
        "retain the entire original gallery reader record and text"
    );
    ensure!(
        new["review_contract"]["entry_core"] == json!([3, 12])
            && new["review_contract"]["exit_core"] == json!([3, 13])
            && new["review_contract"]["retained_return_exterior_core"] == json!([36, 18])
            && new["review_contract"]["native_stair_blocks"]
                == json!([[3, 4], [10, 3], [10, 4], [3, 7]])
            && new["review_contract"]["loop_waypoints_core"]
                == json!([[7, 11], [7, 5], [20, 5], [22, 11], [22, 21], [8, 19]]),
        "retain the reviewed source stair/ordinary loop/return contract"
    );
    let ledges = new["review_contract"]["one_way_ledges"]
        .as_array()
        .context("reviewed cave drop contract")?;
    ensure!(
        ledges.len() == 1
            && ledges[0]["id"] == "GALLERY_MOUTH_DROP"
            && ledges[0]["source_block"] == "39"
            && ledges[0]["direction"] == "south"
            && ledges[0]["upper_approach"] == json!([12, 5])
            && ledges[0]["hop_from"] == json!([12, 6])
            && ledges[0]["landing"] == json!([12, 8])
            && ledges[0]["reverse_blocked"] == json!([12, 7])
            && ledges[0]["side_barriers"] == json!([[11, 7], [14, 7]])
            && ledges[0]["minimum_return_steps"] == 9,
        "source gallery drop must describe the exact native hop and return"
    );
    let mut expected = old["items"][0].clone();
    ensure!(
        expected["id"] == PICKUP
            && expected["item"] == "REVIVE"
            && expected["quantity"] == 1
            && expected["flag"] == "EVENT_MN_R10_ITEM_04"
            && expected["tile"] == json!([17, 5]),
        "original Revive source drift"
    );
    expected["tile"] = json!([29, 23]);
    ensure!(
        new["items"][0] == expected,
        "only the original Revive location may change"
    );
    Ok(())
}

fn blocks(room: &Interior) -> Result<Vec<u16>> {
    let [width, height] = room.dimensions_metatiles;
    ensure!(
        room.rows.len() == usize::from(height),
        "complete cave row count"
    );
    let mut result = Vec::new();
    for row in &room.rows {
        let values = row
            .split_whitespace()
            .map(|token| u16::from_str_radix(token, 16).context("native cave metatile"))
            .collect::<Result<Vec<_>>>()?;
        ensure!(
            values.len() == usize::from(width) && values.iter().all(|&b| b < 0x40),
            "native cave row width/block extent"
        );
        result.extend(values);
    }
    Ok(result)
}

fn normalize_transport(data: &crate::GameDataSet, module: &mut MapModule) -> Result<()> {
    for label in [
        module.attributes.map_scripts_label.as_deref(),
        module.attributes.map_events_label.as_deref(),
    ] {
        let label = label.context("owned cave transport label")?;
        let payload = data
            .map_scripts
            .get(label)
            .context("owned cave raw transport payload")?;
        module
            .scripts
            .entry(label.into())
            .or_insert_with(|| payload.clone());
    }
    Ok(())
}

fn validate_preimage(
    base: &CompiledGamePack,
    module: &MapModule,
    source: &Interior,
    expected: &[u16],
) -> Result<()> {
    let a = &module.attributes;
    ensure!(
        module.id == MAP
            && a.map_constant.as_deref() == Some(MAP)
            && [a.width, a.height] == [12, 8]
            && a.tileset_name == "cave"
            && a.border_block == 1
            && a.connections.is_empty()
            && a.map_group_constant.as_deref() == Some("MAPGROUP_MINNESOTA_IRON_RANGE_DESCENT")
            && a.environment.as_deref() == Some("CAVE")
            && module.blocks == expected
            && base.data.map_attributes.get(MAP) == Some(a),
        "installed R10 room identity/terrain differs from previous source"
    );
    let meta = base
        .data
        .runtime_map_metadata
        .get(MAP)
        .context("R10 room numeric metadata")?;
    ensure!(
        meta.group_id == 55 && meta.map_id == 3 && meta.width == 12 && meta.height == 8,
        "R10 cave numeric identity/extent drift"
    );
    let labels = [
        a.blocks_label.as_deref(),
        a.map_scripts_label.as_deref(),
        a.map_events_label.as_deref(),
    ];
    ensure!(
        labels.into_iter().collect::<BTreeSet<_>>().len() == 3,
        "distinct cave section labels required"
    );
    for label in labels {
        let label = label
            .filter(|s| !s.is_empty())
            .context("cave owned label")?;
        ensure!(
            base.data.maps.iter().all(|(name, other)| name == MAP
                || ![
                    other.attributes.blocks_label.as_deref(),
                    other.attributes.map_scripts_label.as_deref(),
                    other.attributes.map_events_label.as_deref()
                ]
                .contains(&Some(label))),
            "shared cave section label"
        );
    }
    let bytes = crate::decode_base64_bytes(
        base.data
            .map_blocks
            .get(a.blocks_label.as_ref().context("block label")?)
            .context("encoded cave blocks")?,
    )?;
    ensure!(
        bytes
            .into_iter()
            .map(u16::from)
            .eq(expected.iter().copied()),
        "cave encoded/native previous blocks disagree"
    );
    ensure!(
        base.data.map_dimensions.get(MAP) == Some(&json!({"width":12,"height":8})),
        "cave dimension mirror drift"
    );
    let mut data = base.data.clone();
    data.maps.remove(MAP);
    let mut certified = module.clone();
    normalize_transport(&data, &mut certified)?;
    ensure!(
        data.assemble_map_module_from_compiled_payloads(MAP)? == certified,
        "old R10 native split payload/index drift"
    );
    ensure!(
        module.objects.len() == 2
            && module.events.warps.len() == 1
            && module.events.bg_events.is_empty()
            && module.events.coord_events.is_empty(),
        "retain exact cave events"
    );
    let expected_scripts = BTreeMap::from([
        (
            READER.to_owned(),
            json!([{"command":"jumptextfaceplayer","args":[format!("{READER}Text")]}]),
        ),
        (
            format!("{READER}Text"),
            json!([
            {"command":"text","args":["\"Both passages\""]},
            {"command":"line","args":["\"circle the pillar.\""]},
            {"command":"para","args":["\"The foreman was\""]},
            {"command":"line","args":["\"at the west mine.\""]},
            {"command":"done","args":[]}]),
        ),
        (
            PICKUP.to_owned(),
            json!([{"command":"itemball","args":["REVIVE","1"]}]),
        ),
    ]);
    let mut programs = module.scripts.clone();
    for label in [a.map_scripts_label.as_ref(), a.map_events_label.as_ref()]
        .into_iter()
        .flatten()
    {
        programs.remove(label);
    }
    ensure!(
        programs == expected_scripts,
        "original cave resident/pickup program differs"
    );
    for (label, payload) in &expected_scripts {
        ensure!(
            base.data.map_scripts.get(label) == Some(payload)
                && base
                    .data
                    .maps
                    .iter()
                    .all(|(name, other)| name == MAP || !other.scripts.contains_key(label)),
            "cave program label drift/shared ownership"
        );
    }
    let reader = &module.objects[0];
    let pickup = &module.objects[1];
    ensure!(
        reader.script == READER
            && reader.object_identifier.as_deref() == Some(READER)
            && reader.sprite == "SPRITE_SUPER_NERD"
            && [reader.x, reader.y] == [9, 5]
            && reader.spritemovedata == "SPRITEMOVEDATA_STANDING_RIGHT"
            && reader.object_type == "OBJECTTYPE_SCRIPT"
            && pickup.script == PICKUP
            && pickup.object_identifier.as_deref() == Some(PICKUP)
            && pickup.sprite == "SPRITE_POKE_BALL"
            && [pickup.x, pickup.y] == [17, 5]
            && pickup.object_type == "OBJECTTYPE_ITEMBALL"
            && pickup.event_flag == "EVENT_MN_R10_ITEM_04",
        "installed cave resident/reward source anchors differ"
    );
    let exit = &module.events.warps[0];
    ensure!(
        exit.index == 1
            && [exit.x, exit.y] == [3, 13]
            && exit.target_map == EXTERIOR
            && exit.target_map_constant == EXTERIOR
            && exit.target_warp_id == 2,
        "original cave exit pair drift"
    );
    let exterior = base
        .data
        .maps
        .get(EXTERIOR)
        .context("original exterior mouth")?;
    for (index, tile) in [(1, [36, 17]), (2, [36, 18])] {
        let warp = exterior
            .events
            .warps
            .iter()
            .find(|w| w.index == index)
            .context("paired exterior record")?;
        ensure!(
            [warp.x, warp.y] == tile
                && warp.target_map == MAP
                && warp.target_map_constant == MAP
                && warp.target_warp_id == 1,
            "original exterior door/passive pair drift"
        );
    }
    ensure!(
        collision(
            base,
            &exterior.blocks,
            exterior.attributes.width,
            [36, 18],
            &exterior.attributes.tileset_name
        )? == "FLOOR",
        "safe exterior return must stay Floor"
    );
    validate_room(base, module, source)?;
    Ok(())
}

fn validate_features(base: &CompiledGamePack, module: &MapModule) -> Result<()> {
    let kind = |p| collision(base, &module.blocks, 18, p, "cave");
    let occupied = module
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let mut floor = BTreeSet::new();
    let mut ladders = BTreeSet::new();
    let mut hops = BTreeSet::new();
    for y in 0..28 {
        for x in 0..36 {
            let p = [x, y];
            if x == 0 || x == 35 || y == 0 || y == 27 {
                ensure!(
                    kind(p)? == "WALL",
                    "native gallery outer edge must visibly close with wall"
                );
            }
            match kind(p)? {
                "FLOOR" if !occupied.contains(&p) => {
                    floor.insert(p);
                }
                "LADDER" => {
                    ladders.insert(p);
                }
                "HOP_DOWN" => {
                    hops.insert(p);
                }
                "FLOOR" | "UP_WALL" | "WALL" => {}
                "WATER" => ensure!(
                    x > 0 && x < 35 && y > 0 && y < 27,
                    "unconnected cave outer water"
                ),
                other => anyhow::bail!("unexpected cave permission {other}"),
            }
        }
    }
    ensure!(
        ladders == BTreeSet::from([[3, 13]]) && hops == BTreeSet::from([[12, 6], [13, 6]]),
        "exact paired mouth and short gallery drop only"
    );
    ensure!(
        module.blocks.iter().filter(|&&b| b == 0x12).count() == 2
            && module.blocks.iter().filter(|&&b| b == 0x36).count() == 2
            && !module.blocks.contains(&0x30),
        "exact four reviewed native stair placements required"
    );
    for ([x, y], block, lane) in [
        ([3, 4], 0x12, 7),
        ([10, 3], 0x36, 20),
        ([10, 4], 0x36, 20),
        ([3, 7], 0x12, 7),
    ] {
        ensure!(
            module.blocks[usize::from(y) * 18 + usize::from(x)] == block,
            "native stone stair source differs"
        );
        for yy in y * 2 - 1..=y * 2 + 2 {
            ensure!(
                floor.contains(&[lane, yy]) && distance([3, 12], [lane, yy], &floor).is_some(),
                "disconnected ordinary stair lane"
            );
        }
    }
    ensure!(
        [[12, 7], [11, 7], [14, 7]]
            .iter()
            .all(|p| kind(*p).is_ok_and(|k| k == "WALL")),
        "gallery drop side/reverse wall missing"
    );
    for waypoint in [[7, 11], [7, 5], [20, 5], [22, 11], [22, 21], [8, 19]] {
        ensure!(
            floor.contains(&waypoint) && distance([3, 12], waypoint, &floor).is_some(),
            "ordinary gallery loop waypoint disconnected"
        );
    }
    ensure!(
        distance([12, 8], [12, 5], &floor).is_some_and(|steps| steps >= 9),
        "short drop requires useful ordinary return"
    );
    let map = OverworldMapData::from_attributes(MAP, &module.attributes, module.blocks.clone());
    let tileset = base.data.tileset_collision("cave")?;
    let bodies = module
        .objects
        .iter()
        .map(|o| OccupiedTile {
            tile: TilePosition::new(o.x as i16, o.y as i16),
            object_identifier: o.object_identifier.clone(),
        })
        .collect::<Vec<_>>();
    let mut player = PlayerMovementState::new(TilePosition::new(12, 6));
    ensure!(
        matches!(attempt_ledge_jump_with_occupied_tiles(&mut player,Direction::Down,&map,&tileset,StepOptions::default(),&bodies),
        LedgeJumpOutcome::Jumped{to,..}if to==TilePosition::new(12,8)),
        "native gallery hop must land on its declared floor"
    );
    player.facing = Direction::Up;
    ensure!(
        matches!(
            attempt_step_with_occupied_tiles(
                &mut player,
                Direction::Up,
                &map,
                &tileset,
                StepOptions::default(),
                &bodies
            ),
            StepOutcome::Blocked { .. }
        ),
        "native gallery reverse must reject wall"
    );
    Ok(())
}

fn distance(start: [u16; 2], target: [u16; 2], floor: &BTreeSet<[u16; 2]>) -> Option<u16> {
    let mut todo = VecDeque::from([(start, 0)]);
    let mut seen = BTreeSet::from([start]);
    while let Some(([x, y], steps)) = todo.pop_front() {
        if [x, y] == target {
            return Some(steps);
        }
        for next in [
            [x.saturating_sub(1), y],
            [x + 1, y],
            [x, y.saturating_sub(1)],
            [x, y + 1],
        ] {
            if floor.contains(&next) && seen.insert(next) {
                todo.push_back((next, steps + 1));
            }
        }
    }
    None
}
