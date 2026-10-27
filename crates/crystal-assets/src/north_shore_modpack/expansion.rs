//! Transactional expansion of the installed North Shore, without reinstalling events.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use crystal_core::world::encounters::{WildEncounterData, WildEncounterTable};
use serde_json::{Value, json};

use crate::authored_route_design::RouteDesign;
use crate::authored_route_events as events;
use crate::authored_route_layout::Layout;
use crate::authored_route_revision::{installed_chunks, validate_anchors};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, CompiledMapExtension, MapModule, RuntimeSpawnPoint};

const MANIFEST: &str = "minnesota-north-shore-expansion-v1";
const ART_MANIFEST: &str = "minnesota-north-shore-stone-stairs-v1";
const NEW_CHUNKS: [(&str, u16, u16, u16); 5] = [
    ("MnR11C01R00", 1, 0, 5),
    ("MnR11C01R01", 1, 1, 6),
    ("MnR11C01R02", 1, 2, 7),
    ("MnR11C00R03", 0, 3, 8),
    ("MnR11C01R03", 1, 3, 9),
];

/// Expands an installed32×72 North Shore to its authored56×96 generation.
///
/// Previous layout/dossier must match installed terrain, actor/party identities,
/// room geometry, numeric IDs, encounters and split payloads. Existing event,
/// trainer, item, script, room and story contracts must be identical in the new
/// dossier. Only terrain, the southern planned port, authored navigation features
/// and internal connections change. Two explicitly named optional pickups and
/// three signs are appended through the native event compiler. Five chunks are
/// appended with explicit IDs5–9, preserving lighthouse4. Exact native stone stairs are added
/// only as declared by the source recipe. HOME, recovery, audio and unrelated
/// content remain unchanged. Regional town connections are not installed here.
/// No output is written; this is not native controller/visual acceptance.
///
/// # Errors
/// Returns an error for stale/unsupported source generations, native preimage or
/// event drift, occupied identities/labels, unsupported source-art recipes,
/// invalid revised geometry, or failed complete native pack validation. Inputs
/// remain unchanged on all failures.
pub fn expand_north_shore_modpack(
    base: &CompiledGamePack,
    previous_layout_json: &str,
    revised_layout_json: &str,
    previous_dossier_json: &str,
    revised_dossier_json: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        !base.report.manifests.iter().any(|id| id == MANIFEST),
        "North Shore expansion already installed"
    );
    let old = Layout::parse(previous_layout_json).context("previous North Shore layout")?;
    let revised = Layout::parse(revised_layout_json).context("expanded North Shore layout")?;
    let previous =
        RouteDesign::parse(previous_dossier_json).context("previous North Shore dossier")?;
    let design =
        RouteDesign::parse(revised_dossier_json).context("expanded North Shore dossier")?;
    ensure!(
        old.id == "MN_R11"
            && revised.id == old.id
            && previous.id == old.id
            && design.id == old.id
            && old.dimensions_metatiles == [32, 72]
            && previous.dimensions_metatiles == [32, 72]
            && revised.dimensions_metatiles == [56, 96]
            && design.dimensions_metatiles == [56, 96],
        "North Shore expansion requires exact old/new generations"
    );
    validate_source_contract(previous_dossier_json, revised_dossier_json)?;
    let stairs = validate_art_contract(previous_layout_json, revised_layout_json)?;
    let old_blocks = old.compile()?;
    let blocks = revised.compile()?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let chunks = installed_chunks(base, &previous, &old, &old_blocks)?;
    validate_anchors(base, &previous)?;
    let mut split_data = base.data.clone();
    for chunk in &chunks {
        let module = &base.data.maps[&chunk.name];
        let row = chunk.origin[1] / 24;
        let constant = events::constant(&chunk.name);
        let metadata = base
            .data
            .runtime_map_metadata
            .get(&constant)
            .context("installed North Shore metadata")?;
        ensure!(
            metadata.group_id == 41 && metadata.map_id == row + 1 && metadata.name == chunk.name,
            "installed North Shore numeric identity drift"
        );
        ensure!(
            module.attributes.connections == connections(0, row, 1, 3),
            "installed North Shore connection drift"
        );
        validate_split(&mut split_data, module)?;
        ensure!(
            base.data.wild_encounters.get(&chunk.name) == Some(&encounters(&previous, &chunk.name)),
            "installed North Shore encounter drift"
        );
    }
    let lighthouse = base
        .data
        .maps
        .get("MnR11SplitRockLighthouse")
        .context("installed lighthouse")?;
    let room_constant = events::constant(&lighthouse.id);
    let room_metadata = base
        .data
        .runtime_map_metadata
        .get(&room_constant)
        .context("lighthouse metadata")?;
    ensure!(
        room_metadata.group_id == 41 && room_metadata.map_id == 4,
        "lighthouse numeric identity drift"
    );
    validate_split(&mut split_data, lighthouse)?;
    ensure!(
        base.data
            .runtime_map_metadata
            .values()
            .filter(|m| m.group_id == 41)
            .count()
            == 4,
        "unexpected installed North Shore group members"
    );
    for &(name, _, _, id) in &NEW_CHUNKS {
        ensure!(
            !base.data.maps.contains_key(name)
                && !base.data.map_attributes.contains_key(name)
                && !base
                    .data
                    .map_dimensions
                    .contains_key(&events::constant(name))
                && !base
                    .data
                    .runtime_map_metadata
                    .contains_key(&events::constant(name))
                && !base
                    .data
                    .runtime_map_metadata
                    .values()
                    .any(|m| m.group_id == 41 && m.map_id == id),
            "new North Shore map identity occupied: {name}"
        );
    }
    for label in [
        "MN_R11_ITEM_05",
        "MN_R11_ITEM_06",
        "MN_R11_PALISADE_SIGN",
        "MN_R11_PALISADE_SIGNText",
        "MN_R11_STONEY_SIGN",
        "MN_R11_STONEY_SIGNText",
        "MN_R11_LESTER_SIGN",
        "MN_R11_LESTER_SIGNText",
    ] {
        ensure!(
            !base.data.map_scripts.contains_key(label)
                && base
                    .data
                    .maps
                    .values()
                    .all(|m| !m.scripts.contains_key(label)
                        && !m.script_text_bodies.contains_key(label)),
            "expanded event label already owned: {label}"
        );
    }
    crate::north_shore_art::validate_installed_north_shore_art(base)?;
    let mut result = crate::north_shore_art::append_native_cave_stairs(
        base,
        "mn_north_shore",
        0x81,
        &stairs,
        ART_MANIFEST,
    )?;
    crate::authored_route_modpack::validate_geometry(&result, &design, &blocks, "mn_north_shore")
        .context("complete expanded North Shore geometry")?;
    let occupied = design
        .trainers
        .iter()
        .map(|a| a.tile)
        .chain(design.residents.iter().map(|a| a.tile))
        .chain(design.items.iter().map(|a| a.tile))
        .collect::<BTreeSet<_>>();
    let reachable = crate::authored_route_surface::reachable_with_surf(
        &result,
        &design,
        &blocks,
        "mn_north_shore",
        &occupied,
    )?;
    // Preserve old native executable fields, then lower only the two new
    // optional pickups and three new signs through the existing compiler.
    let mut modules = Vec::new();
    for chunk in chunks {
        let mut module = base.data.maps[&chunk.name].clone();
        set_geometry(&mut module, 0, chunk.origin[1] / 24, &blocks);
        modules.push(module);
    }
    for &(name, column, row, _) in &NEW_CHUNKS {
        let mut module = authoring::blank(base, "MnR11C00R00", name, &events::constant(name))?;
        set_geometry(&mut module, column, row, &blocks);
        modules.push(module);
    }
    let mut additions: RouteDesign = serde_json::from_str(revised_dossier_json)?;
    additions.trainers.clear();
    additions.residents.clear();
    additions.items.drain(..4);
    additions.signs.drain(..6);
    let mut next = result
        .data
        .story_event_script_constants
        .global
        .iter()
        .filter(|(key, _)| key.starts_with("EVENT_"))
        .map(|(_, value)| *value)
        .max()
        .context("existing event flag namespace")?
        .checked_add(1)
        .context("event flag namespace overflow")?;
    for pickup in &additions.items {
        ensure!(
            !result
                .data
                .story_event_script_constants
                .global
                .contains_key(&pickup.flag),
            "expanded pickup event already exists"
        );
        result
            .data
            .story_event_script_constants
            .global
            .insert(pickup.flag.clone(), next);
        next = next
            .checked_add(1)
            .context("event flag namespace overflow")?;
    }
    events::apply(&mut result, &additions, &mut modules)?;
    let mut existing = Vec::new();
    for module in &modules {
        if base.data.maps.contains_key(&module.id) {
            existing.push(module.clone());
        }
    }
    for module in existing {
        replace_existing(&mut result, module)?;
    }
    let temporary_spawn = result
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .unwrap_or(255)
        .max(255)
        .checked_add(1)
        .context("temporary spawn overflow")?;
    let original_metadata = base.data.runtime_map_metadata["MN_R11_C00R00"].clone();
    for &(name, column, row, id) in &NEW_CHUNKS {
        let constant = events::constant(name);
        let index = modules
            .iter()
            .position(|m| m.id == name)
            .context("new expansion module")?;
        let module = modules.remove(index);
        ensure!(
            module.attributes.width == (56 - column * 32).min(32)
                && module.attributes.height == 24
                && module.attributes.connections == connections(column, row, 2, 4),
            "new expansion geometry drift"
        );
        // The low-level atomic insertion helper does not validate intermediate
        // spawns. Sea-only chunks legitimately use a reachable WATER anchor;
        // this temporary bookkeeping record is removed before final validation
        // and is never a recovery destination or a usable spawn API result.
        let point = reachable
            .iter()
            .find(|&&tile| events::chunk("MnR11", tile) == name)
            .copied()
            .with_context(|| format!("expanded chunk {name} has no reachable walk/Surf cell"))?;
        let local = events::local(point);
        let mut metadata = original_metadata.clone();
        metadata.constant = constant.clone();
        metadata.name = name.into();
        metadata.map_id = id;
        metadata.width = module.attributes.width;
        metadata.height = module.attributes.height;
        result
            .data
            .pokegear_landmarks
            .map_to_landmark
            .insert(name.into(), "LANDMARK_MN_NORTH_SHORE".into());
        crate::insert_compiled_map_extension(
            &mut result,
            CompiledMapExtension {
                manifest_id: format!("{MANIFEST}-{id}"),
                map_name: name.into(),
                map_constant: constant.clone(),
                metadata,
                spawn_key: temporary_spawn.to_string(),
                spawn: RuntimeSpawnPoint {
                    identifier: temporary_spawn,
                    map_constant: constant,
                    map_name: name.into(),
                    group_id: 41,
                    map_id: i16::try_from(id)?,
                    group_name: original_metadata.group_name.clone(),
                    tile_x: i16::try_from(local[0])?,
                    tile_y: i16::try_from(local[1])?,
                    metatile_x: i16::try_from(local[0] / 2)?,
                    metatile_y: i16::try_from(local[1] / 2)?,
                    subtile_x: i16::try_from(local[0] % 2)?,
                    subtile_y: i16::try_from(local[1] % 2)?,
                },
                module,
                wild_encounters: Some(encounters(&design, name)),
                start_new_game_here: false,
            },
        )?;
        result
            .data
            .runtime_spawn_points
            .remove(&temporary_spawn.to_string());
    }
    result.report.manifests.push(MANIFEST.into());
    result.report.maps = result.data.maps.len();
    result.report.reachable_maps.sort();
    result.report.solvable_maps.sort();
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

fn validate_source_contract(previous: &str, revised: &str) -> Result<()> {
    let old: Value = serde_json::from_str(previous)?;
    let new: Value = serde_json::from_str(revised)?;
    for field in [
        "id",
        "map_prefix",
        "tileset",
        "required_flags",
        "quest",
        "trainers",
        "residents",
        "doors",
        "interiors",
        "encounters",
        "shortcut",
        "cut_gates",
        "surf_only_actor_ids",
    ] {
        ensure!(
            old.get(field) == new.get(field),
            "North Shore expansion cannot change existing {field}"
        );
    }
    for (field, previous_count, new_count) in [("items", 4, 6), ("signs", 6, 9)] {
        let before = old[field].as_array().context("previous events")?;
        let after = new[field].as_array().context("expanded events")?;
        ensure!(
            before.len() == previous_count
                && after.len() == new_count
                && after[..previous_count] == before[..],
            "retain existing {field} exactly before bounded additions"
        );
    }
    for (index, id, item, flag) in [
        (4, "MN_R11_ITEM_05", "NUGGET", "EVENT_MN_R11_ITEM_05"),
        (5, "MN_R11_ITEM_06", "FULL_RESTORE", "EVENT_MN_R11_ITEM_06"),
    ] {
        let pickup = &new["items"][index];
        ensure!(
            pickup["id"] == id
                && pickup["item"] == item
                && pickup["quantity"] == 1
                && pickup["flag"] == flag,
            "unsupported expanded climb reward"
        );
    }
    for (index, id) in [
        "MN_R11_PALISADE_SIGN",
        "MN_R11_STONEY_SIGN",
        "MN_R11_LESTER_SIGN",
    ]
    .iter()
    .enumerate()
    {
        ensure!(
            new["signs"][index + 6]["id"] == *id,
            "unsupported expanded sign identity"
        );
    }
    let old_grass = old["required_grass_crossings"]
        .as_array()
        .context("previous grass contract")?;
    let new_grass = new["required_grass_crossings"]
        .as_array()
        .context("expanded grass contract")?;
    ensure!(
        old_grass.len() == 1
            && new_grass.len() == 2
            && new_grass.first() == old_grass.first()
            && new_grass[1]["id"] == "BIRCH_SWITCHBACK_GRASS",
        "retain original habitat and bounded birch crossing"
    );
    let ledges = new["one_way_ledges"]
        .as_array()
        .context("expanded ledges")?;
    ensure!(
        ledges.iter().filter_map(|l| l["id"].as_str()).eq([
            "PALISADE_RIDGE_DROP",
            "LESTER_TERRACE_DROP",
            "BIRCH_SWITCHBACK_DROP",
        ]),
        "retain three explicitly authored return ledges"
    );
    let endpoints = new["endpoints"].as_array().context("expanded ports")?;
    let old_endpoints = old["endpoints"].as_array().context("previous ports")?;
    for endpoint in endpoints {
        let place = endpoint["place"].as_str().context("port place")?;
        let previous = old_endpoints
            .iter()
            .find(|e| e["place"] == endpoint["place"])
            .context("existing port place")?;
        if place == "DULUTH" {
            let mut expected = previous.clone();
            expected["gate_metatile"] = json!([23, 94]);
            expected["landing_metatile"] = json!([23, 92]);
            ensure!(
                endpoint == &expected,
                "only the planned Duluth port geometry may move"
            );
        } else {
            ensure!(endpoint == previous, "retain the Grand Marais port");
        }
    }
    Ok(())
}

fn validate_art_contract(previous: &str, revised: &str) -> Result<Vec<u8>> {
    let old: Value = serde_json::from_str(previous)?;
    let new: Value = serde_json::from_str(revised)?;
    for field in ["tileset", "compiled_tileset", "source_pixels_modified"] {
        ensure!(
            old.get(field) == new.get(field),
            "expanded North Shore source art identity drift"
        );
    }
    let previous_blocks = old["derived_blocks"]
        .as_array()
        .context("previous waterfall recipe")?;
    let blocks = new["derived_blocks"]
        .as_array()
        .context("expanded native art recipe")?;
    ensure!(
        previous_blocks.len() == 1
            && blocks.len() >= 3
            && blocks.len() <= 4
            && blocks.first() == previous_blocks.first(),
        "retain waterfall80 and bounded native stair recipe"
    );
    let mut result = Vec::new();
    for (index, source) in blocks.iter().skip(1).enumerate() {
        let expected = [0x12, 0x30, 0x36][index];
        ensure!(
            source["id"] == format!("{:02x}", 0x81 + index)
                && source["source_tileset"] == "cave"
                && source["source_block"] == format!("{expected:02x}")
                && source.get("palette_override").is_none()
                && source.get("tile_row_order").is_none()
                && source.get("tile_row_sources").is_none(),
            "unsupported expanded native stair recipe"
        );
        result.push(expected);
    }
    Ok(result)
}

fn connections(column: u16, row: u16, columns: u16, rows: u16) -> Vec<MapConnection> {
    [
        ("north", 0, -1),
        ("south", 0, 1),
        ("west", -1, 0),
        ("east", 1, 0),
    ]
    .into_iter()
    .filter_map(|(direction, dx, dy)| {
        let x = i32::from(column) + dx;
        let y = i32::from(row) + dy;
        (x >= 0 && x < i32::from(columns) && y >= 0 && y < i32::from(rows)).then(|| MapConnection {
            direction: direction.into(),
            target_map: events::chunk("MnR11", [x as u16 * 64, y as u16 * 48]),
            offset: 0,
        })
    })
    .collect()
}

fn set_geometry(module: &mut MapModule, column: u16, row: u16, blocks: &[u16]) {
    module.attributes.map_group_constant = Some("MAPGROUP_MINNESOTA_NORTH_SHORE".into());
    module.attributes.width = (56 - column * 32).min(32);
    module.attributes.height = 24;
    module.attributes.connections = connections(column, row, 2, 4);
    module.attributes.connection_flags = Some(
        module
            .attributes
            .connections
            .iter()
            .map(|c| c.direction.to_ascii_uppercase())
            .collect::<Vec<_>>()
            .join(" | "),
    );
    module.blocks.clear();
    for y in row * 24..row * 24 + 24 {
        let start = usize::from(y) * 56 + usize::from(column) * 32;
        module
            .blocks
            .extend_from_slice(&blocks[start..start + usize::from(module.attributes.width)]);
    }
}

fn replace_existing(pack: &mut CompiledGamePack, module: MapModule) -> Result<()> {
    let label = module
        .attributes
        .blocks_label
        .as_ref()
        .context("existing block label")?;
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    pack.data
        .map_blocks
        .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    let constant = module
        .attributes
        .map_constant
        .as_ref()
        .context("existing constant")?;
    pack.data.map_dimensions.insert(
        constant.clone(),
        json!({"width":module.attributes.width,"height":module.attributes.height}),
    );
    pack.data
        .map_attributes
        .insert(module.id.clone(), module.attributes.clone());
    crate::insert_compiled_map_split_payloads(
        &mut pack.data.map_scripts,
        &mut pack.data.npcs,
        &module.id,
        module
            .attributes
            .map_scripts_label
            .as_deref()
            .context("existing script section")?,
        module
            .attributes
            .map_events_label
            .as_deref()
            .context("existing event section")?,
        &module,
    )?;
    pack.data.maps.insert(module.id.clone(), module);
    Ok(())
}

fn validate_split(data: &mut crate::GameDataSet, module: &MapModule) -> Result<()> {
    data.maps.remove(&module.id);
    let mut expected = module.clone();
    // Cached typed modules may omit only their two ASM transport sections;
    // uncached assembly necessarily includes these exact owned raw labels.
    for label in [
        module.attributes.map_scripts_label.as_deref(),
        module.attributes.map_events_label.as_deref(),
    ] {
        let label = label.context("installed North Shore transport section")?;
        let payload = data
            .map_scripts
            .get(label)
            .context("installed raw transport section")?;
        expected
            .scripts
            .entry(label.into())
            .or_insert_with(|| payload.clone());
    }
    ensure!(
        data.assemble_map_module_from_compiled_payloads(&module.id)? == expected,
        "installed North Shore split payload drift: {}",
        module.id
    );
    Ok(())
}

fn encounters(design: &RouteDesign, name: &str) -> WildEncounterData {
    let table = &design.encounters;
    WildEncounterData {
        map_name: name.into(),
        grass_rates: Some(table.grass_rates.clone()),
        grass: Some(table.grass.clone()),
        water_rate: Some(table.water_rate),
        water: Some(WildEncounterTable {
            morning: table.water.clone(),
            day: table.water.clone(),
            night: table.water.clone(),
        }),
        ..Default::default()
    }
}
