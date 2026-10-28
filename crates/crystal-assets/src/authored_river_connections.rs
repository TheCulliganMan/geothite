//! Add hand-authored Mississippi reaches without rebuilding existing game content.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::authored_river_connections_design::{Anchors, BridgeInventory, Manifest, NewMap};
use crate::authored_route_layout::Layout;
use crate::grand_marais_modpack::authoring;
use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, RuntimeMapMetadata, RuntimeSpawnPoint,
};

pub(crate) const BASE: &str = "23b3ca0c19fbc6c7f687645540aa9bdd12becfbdb7b693804d6f3b125a41ed41";
const BASE_FILE: &str = "2fe6fc2a2c59833886b9ffa2fa04584f1f95841efaf87e865f11230cf4cdf54e";

/// Installs explicit river chunks, reciprocal connections and inventoried bridge underpasses.
///
/// Layout texts are keyed by their exact manifest paths. Existing maps retain all
/// actors, programs, encounters, rooms, recovery and story; only declared cells
/// and previously unused connection sides can change. Native bridge glyphs are
/// copied exactly into appended opt-in collision blocks. No progress is granted.
///
/// # Errors
/// Rejects a stale parent, occupied identities, source/preimage hash mismatch,
/// unsupported activity, unsafe boundary, nonreciprocal connection or failed
/// native verification. The borrowed input is unchanged on every failure.
pub fn build_authored_river_connections(
    base: &CompiledGamePack,
    manifest_json: &str,
    layouts: &BTreeMap<String, String>,
    bridge_inventory_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let plan: Manifest = serde_json::from_str(manifest_json)?;
    let bridges: BridgeInventory = serde_json::from_str(bridge_inventory_json)?;
    ensure!(
        plan.schema_version == 1
            && bridges.schema_version == 1
            && plan.id == "MN_MISSISSIPPI_CONNECTIONS"
            && plan.base_pack_sha256 == BASE_FILE
            && bridges.base_pack_sha256 == BASE_FILE
            && plan.base_content_hash.as_deref() == Some(BASE)
            && base.identity.content_hash == BASE,
        "exact completed compact Twin Cities parent required"
    );
    ensure!(
        plan.registry.group_id == 82
            && plan.registry.group_name == "MINNESOTA_MISSISSIPPI"
            && plan.registry.map_group_constant == "MAPGROUP_MINNESOTA_MISSISSIPPI"
            && !base
                .data
                .runtime_map_metadata
                .values()
                .any(|m| m.group_id == 82)
            && !base
                .data
                .story_event_script_constants
                .global
                .contains_key(&plan.registry.map_group_constant)
            && !base.report.manifests.iter().any(|m| m == &plan.id)
            && !plan.new_maps.is_empty()
            && plan.backgrounds.is_empty(),
        "unused river identities and explicit supported events required"
    );
    ensure!(
        !plan.bridge_inventory.is_empty() && !bridges.bridges.is_empty(),
        "bridge inventory required"
    );
    let mut maps = BTreeMap::new();
    let mut changed = BTreeSet::new();
    for revision in &plan.terrain_revisions {
        ensure!(
            changed.insert(revision.map.clone()),
            "duplicate terrain revision"
        );
        let old = base
            .data
            .maps
            .get(&revision.map)
            .context("installed river neighbor")?;
        ensure!(
            module_hash(old)? == revision.native_module_sha256
                && [old.attributes.width, old.attributes.height] == revision.dimensions_metatiles,
            "{} terrain preimage differs",
            revision.map
        );
        anchors(old, &revision.anchor_preimages)?;
        let mut revised = old.clone();
        let mut cells = BTreeSet::new();
        for cell in &revision.changes {
            let [x, y] = cell.metatile;
            ensure!(
                x < old.attributes.width
                    && y < old.attributes.height
                    && cells.insert(cell.metatile)
                    && cell.after <= 255
                    && cell.before != cell.after,
                "invalid/repeated/empty river terrain cell"
            );
            let index = usize::from(y) * usize::from(old.attributes.width) + usize::from(x);
            ensure!(old.blocks[index] == cell.before, "stale river terrain cell");
            revised.blocks[index] = cell.after;
        }
        maps.insert(revision.map.clone(), revised);
    }
    for link in &plan.existing_connection_additions {
        if !maps.contains_key(&link.map) {
            maps.insert(
                link.map.clone(),
                base.data
                    .maps
                    .get(&link.map)
                    .context("existing river connection map")?
                    .clone(),
            );
        }
        crate::lake_country_geometry::connect(
            maps.get_mut(&link.map).context("river map")?,
            &link.direction,
            &link.target_map,
            link.offset,
        )?;
    }
    for bridge in &bridges.bridges {
        if !maps.contains_key(&bridge.map) {
            maps.insert(
                bridge.map.clone(),
                base.data
                    .maps
                    .get(&bridge.map)
                    .context("bridge map")?
                    .clone(),
            );
        }
    }
    let mut pack = base.clone();
    crate::authored_river_connections_art::install(base, &mut pack, &mut maps, &bridges)?;
    let mut ids = BTreeSet::new();
    let mut constants = BTreeSet::new();
    for source in &plan.new_maps {
        ensure!(
            ids.insert(source.map_id)
                && source.map_id > 0
                && source.map_id <= 255
                && constants.insert(source.map_constant.clone())
                && !base.data.maps.contains_key(&source.id)
                && !base
                    .data
                    .runtime_map_metadata
                    .contains_key(&source.map_constant)
                && !maps.contains_key(&source.id),
            "occupied/duplicate river map identity"
        );
        maps.insert(source.id.clone(), compile_map(base, source, layouts)?);
    }
    crate::authored_river_connections_events::install(&mut pack, &mut maps, &plan)?;
    // Existing scripts and NPC mirrors are never rewritten. Only new modules
    // pass through lowering and split-payload insertion.
    for (name, module) in &maps {
        if let Some(old) = base.data.maps.get(name) {
            let mut expected = old.clone();
            expected.blocks = module.blocks.clone();
            expected.attributes.connections = module.attributes.connections.clone();
            expected.attributes.connection_flags = module.attributes.connection_flags.clone();
            ensure!(
                &expected == module,
                "unrelated existing river module changed"
            );
            let label = old
                .attributes
                .blocks_label
                .as_ref()
                .context("owned existing river blocks")?;
            ensure!(
                !base.data.maps.values().any(|other| other.id != *name
                    && other.attributes.blocks_label.as_ref() == Some(label)),
                "shared existing river block label"
            );
            let bytes = module
                .blocks
                .iter()
                .copied()
                .map(u8::try_from)
                .collect::<std::result::Result<Vec<_>, _>>()?;
            *pack
                .data
                .map_blocks
                .get_mut(label)
                .context("existing river encoded blocks")? = crate::encode_base64_bytes(&bytes);
            pack.data
                .map_attributes
                .insert(name.clone(), module.attributes.clone());
            pack.data.maps.insert(name.clone(), module.clone());
        }
    }
    pack.data
        .story_event_script_constants
        .global
        .insert(plan.registry.map_group_constant.clone(), 82);
    let temporary = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("native spawn table")?
        .checked_add(1)
        .context("temporary spawn overflow")?;
    for source in &plan.new_maps {
        let module = maps.remove(&source.id).context("new river module")?;
        publish(&mut pack, module, source, &plan, temporary)?;
    }
    pack.report.maps = pack.data.maps.len();
    crate::authored_river_connections_validation::validate(base, &pack, &plan, &bridges)?;
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

/// Use existing geographic entries; do not consume the full native landmark table.
/// Lower reaches belong to Winona, the confluence to Minneapolis, and the upper
/// wooded reach to St Cloud. This policy changes no terrain or old city binding.
pub(crate) fn landmark(base: &CompiledGamePack, name: &str) -> Result<String> {
    let parent =
        if name.starts_with("MnMississippiLowerWoods") || name == "MnMississippiWinonaMouth" {
            "MnWinona"
        } else if name.starts_with("MnMississippiConfluence") {
            "MnMinneapolis"
        } else if name.starts_with("MnMississippiUpstreamWoods") {
            "MnStCloud"
        } else {
            anyhow::bail!("new river map has no explicit Minnesota landmark parent: {name}");
        };
    let existing = base.data.pokegear_landmark_for_map(parent)?;
    ensure!(
        (1..=0x7f).contains(&existing.id),
        "river parent landmark must fit native seven-bit table"
    );
    Ok(existing.constant.clone())
}

pub(crate) fn module_hash(module: &MapModule) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&serde_json::to_value(module)?)?)
    ))
}

fn anchors(map: &MapModule, expected: &Anchors) -> Result<()> {
    ensure!(
        map.objects.iter().map(|o| [o.x, o.y]).collect::<Vec<_>>() == expected.objects
            && map
                .events
                .bg_events
                .iter()
                .map(|o| [o.x, o.y])
                .collect::<Vec<_>>()
                == expected.backgrounds
            && map
                .events
                .coord_events
                .iter()
                .map(|o| [o.x, o.y])
                .collect::<Vec<_>>()
                == expected.coordinates
            && map
                .events
                .warps
                .iter()
                .map(|o| [o.x, o.y])
                .collect::<Vec<_>>()
                == expected.warps,
        "complete indexed native river anchors differ"
    );
    Ok(())
}

fn compile_map(
    base: &CompiledGamePack,
    source: &NewMap,
    layouts: &BTreeMap<String, String>,
) -> Result<MapModule> {
    let text = layouts
        .get(&source.layout)
        .context("manifest layout input missing")?;
    ensure!(
        format!("{:x}", Sha256::digest(text.as_bytes())) == source.layout_sha256,
        "river layout file hash differs"
    );
    let value: Value = serde_json::from_str(text)?;
    let layout: Layout = serde_json::from_str(text)?;
    ensure!(
        layout.schema_version == 2
            && layout.compiled_tileset == source.tileset
            && value["source_pixels_modified"] == false
            && value
                .get("derived_blocks")
                .is_none_or(|v| v.as_array().is_some_and(Vec::is_empty))
            && base.data.tilesets.contains_key(&source.tileset)
            && source.environment == "ROUTE"
            && base.audio_manifest.music.contains_key(&source.music),
        "river native source art/environment/music contract"
    );
    let [width, height] = source.dimensions_metatiles;
    let [x0, y0] = source.source_origin_metatiles;
    ensure!(
        width > 0
            && height > 0
            && width <= 32
            && height <= 24
            && x0
                .checked_add(width)
                .is_some_and(|n| n <= layout.dimensions_metatiles[0])
            && y0
                .checked_add(height)
                .is_some_and(|n| n <= layout.dimensions_metatiles[1]),
        "river chunk outside source"
    );
    let full = layout.compile()?;
    let mut module = authoring::blank(base, "Route30", &source.id, &source.map_constant)?;
    module.attributes.width = width;
    module.attributes.height = height;
    module.attributes.tileset_name = source.tileset.clone();
    module.attributes.environment = Some(source.environment.clone());
    module.attributes.music = Some(source.music.clone());
    module.attributes.location = Some(landmark(base, &source.id)?);
    module.attributes.map_group_constant = Some("MAPGROUP_MINNESOTA_MISSISSIPPI".into());
    module.attributes.border_block = 5;
    module.blocks.clear();
    for y in y0..y0 + height {
        let first = usize::from(y) * usize::from(layout.dimensions_metatiles[0]) + usize::from(x0);
        module
            .blocks
            .extend_from_slice(&full[first..first + usize::from(width)]);
    }
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(
        format!("{:x}", Sha256::digest(bytes)) == source.source_blocks_sha256,
        "clipped source blocks hash differs"
    );
    for link in &source.connections {
        crate::lake_country_geometry::connect(
            &mut module,
            &link.direction,
            &link.target_map,
            link.offset,
        )?;
    }
    Ok(module)
}

fn publish(
    pack: &mut CompiledGamePack,
    module: MapModule,
    source: &NewMap,
    plan: &Manifest,
    temporary: u16,
) -> Result<()> {
    let location = module
        .attributes
        .location
        .as_ref()
        .context("river location")?;
    ensure!(
        pack.data
            .pokegear_landmarks
            .landmarks
            .iter()
            .any(|landmark| landmark.constant == *location && (1..=0x7f).contains(&landmark.id)),
        "river location must name an existing native landmark"
    );
    ensure!(
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(source.id.clone(), location.clone())
            .is_none(),
        "river landmark mapping already occupied"
    );
    let encounters = crate::authored_river_connections_events::encounters(pack, plan, &source.id)?;
    let mut arrival = [0u16, 0u16];
    'search: for y in 0..module.attributes.height * 2 {
        for x in 0..module.attributes.width * 2 {
            if crate::authored_town_geometry::kind(pack, &module, [x, y])? == "FLOOR"
                && !module.objects.iter().any(|o| o.x == x && o.y == y)
            {
                arrival = [x, y];
                break 'search;
            }
        }
    }
    let extension = CompiledMapExtension {
        manifest_id: format!("{}-{}", plan.id, source.map_id),
        map_name: source.id.clone(),
        map_constant: source.map_constant.clone(),
        metadata: RuntimeMapMetadata {
            name: source.id.clone(),
            constant: source.map_constant.clone(),
            group_name: plan.registry.group_name.clone(),
            group_id: 82,
            map_id: source.map_id,
            width: source.dimensions_metatiles[0],
            height: source.dimensions_metatiles[1],
            environment: source.environment.clone(),
            phone_service: module.attributes.phone_service,
        },
        spawn_key: temporary.to_string(),
        spawn: RuntimeSpawnPoint {
            identifier: temporary,
            map_name: source.id.clone(),
            map_constant: source.map_constant.clone(),
            group_name: plan.registry.group_name.clone(),
            group_id: 82,
            map_id: i16::try_from(source.map_id)?,
            tile_x: i16::try_from(arrival[0])?,
            tile_y: i16::try_from(arrival[1])?,
            metatile_x: i16::try_from(arrival[0] / 2)?,
            metatile_y: i16::try_from(arrival[1] / 2)?,
            subtile_x: i16::try_from(arrival[0] % 2)?,
            subtile_y: i16::try_from(arrival[1] % 2)?,
        },
        module,
        wild_encounters: encounters,
        start_new_game_here: false,
    };
    crate::validate_compiled_map_extension(&extension)?;
    crate::insert_compiled_map_extension(pack, extension)?;
    pack.data
        .runtime_spawn_points
        .remove(&temporary.to_string());
    // Assemble through the same complete typed production parser as pack loading.
    let expected = pack
        .data
        .maps
        .remove(&source.id)
        .context("inserted river module")?;
    let mut assembled = pack
        .data
        .assemble_map_module_from_compiled_payloads(&source.id)?;
    for label in [
        &expected.attributes.map_scripts_label,
        &expected.attributes.map_events_label,
    ] {
        assembled
            .scripts
            .remove(label.as_ref().context("river transport label")?);
    }
    ensure!(
        assembled == expected,
        "new river cached/uncached production module differs: {}",
        source.id
    );
    pack.data.maps.insert(source.id.clone(), assembled);
    Ok(())
}
