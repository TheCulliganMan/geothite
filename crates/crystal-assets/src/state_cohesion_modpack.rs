//! Bounded full-state joins; preserves the completed river and all old gameplay.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::authored_route_layout::Layout;
use crate::authored_town_geometry::kind;
use crate::grand_marais_modpack::authoring;
use crate::state_cohesion_design::{Dossier, NewMap, Patches, Plan};
use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, RuntimeMapMetadata, RuntimeSpawnPoint,
};

pub(crate) const BASE_IDENTITY: &str =
    "de528e31f3d06f37c41b47595806d492d8279768f71d7b508ae7ab5315b66b5e";
pub(crate) const BASE_FILE: &str =
    "6b36b4b835873e4c4508a7da7727ecb759273c26e219ec026c061c0e66f24775";
const PATCH_FILE: &str = "717f7744da7a5c1cfc13509144f0d2dd4f8d6f2b0fa0a6359efe4579252957c4";

/// Installs the four reviewed wooded connectors and exact bounded native joins.
///
/// Inputs are keyed by their exact contract paths and include all four layouts,
/// dossiers and the port-patch contract. The completed corrected river parent
/// is required. No old room, recovery, audio, art, progress flag or program is
/// replaced; new guard programs and actors are appended without reparsing old
/// tables. New maps bind existing geographic landmarks, not template locations.
///
/// # Errors
/// Rejects a stale parent/input, occupied identity, unreviewed delta, unsafe
/// traversal/boundary/checkpoint, stale native preimage, incoherent world,
/// split-payload mismatch or unrelated-data change. The borrowed input remains
/// unchanged on failure. The caller owns writing and strict readback.
pub fn build_state_cohesion(
    base: &CompiledGamePack,
    contract_json: &str,
    sources: &BTreeMap<String, String>,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let plan: Plan = serde_json::from_str(contract_json)?;
    let value: Value = serde_json::from_str(contract_json)?;
    ensure!(
        plan.schema_version == 1
            && plan.id == "MN_STATE_COHESION"
            && plan.base_pack_sha256 == BASE_FILE
            && plan.base_content_hash == BASE_IDENTITY
            && base.identity.content_hash == BASE_IDENTITY
            && plan.map_count_before == 677
            && base.data.maps.len() == 677,
        "exact corrected677-map Mississippi parent required"
    );
    for (field, expected) in [
        (
            "guard_proposals",
            "a202c39b0b840431f2228cf822d92332b15df997da76a91e5b18de9e687b74cb",
        ),
        (
            "new_reciprocal_links",
            "bf1d6042878f164afd01cb5fa4315a326ba1ca4a6f80d71aa632b3a8fed85a22",
        ),
        (
            "existing_connection_updates",
            "b91afebed3bd95629a0bf47200328f9442c61665566b840f21a69753f262d7dd",
        ),
    ] {
        ensure!(
            hash_value(&value[field])? == expected,
            "unreviewed fixed cohesion {field}"
        );
    }
    ensure!(
        plan.registry.group_id == 83
            && plan.registry.group_name == "MINNESOTA_COHESION"
            && plan.registry.map_group_constant == "MAPGROUP_MINNESOTA_COHESION"
            && !base
                .data
                .runtime_map_metadata
                .values()
                .any(|m| m.group_id == 83)
            && !base
                .data
                .story_event_script_constants
                .global
                .contains_key(&plan.registry.map_group_constant),
        "cohesion group83 already occupied/changed"
    );
    ensure!(
        plan.native_preimages.len() == 17 && plan.river_module_preservation_preimages.len() == 17,
        "complete native and river preimages required"
    );
    let mut preimages = BTreeSet::new();
    for row in plan
        .native_preimages
        .iter()
        .chain(&plan.river_module_preservation_preimages)
    {
        ensure!(
            preimages.insert(&row.map)
                && hash_value(&serde_json::to_value(
                    base.data
                        .maps
                        .get(&row.map)
                        .context("native preimage map")?
                )?)? == row.module_sha256,
            "whole native preimage differs: {}",
            row.map
        );
    }
    let patch_text = sources
        .get(&plan.native_port_patches)
        .context("exact port patch input")?;
    ensure!(
        hash_bytes(patch_text.as_bytes()) == PATCH_FILE
            && plan.native_port_patches_sha256 == PATCH_FILE,
        "only reviewed42-cell/wall-margin patch recipe permitted"
    );
    let patches: Patches = serde_json::from_str(patch_text)?;
    ensure!(
        patches.parent_pack_sha256 == BASE_FILE && patches.patches.len() == 12,
        "bounded patch parent/count differs"
    );
    let mut maps = BTreeMap::new();
    let mut cells = 0;
    for patch in &patches.patches {
        ensure!(!maps.contains_key(&patch.map), "duplicate terrain patch");
        let old = base.data.maps.get(&patch.map).context("native port map")?;
        ensure!(
            hash_value(&serde_json::to_value(old)?)? == patch.native_module_sha256,
            "stale port module"
        );
        let mut map = old.clone();
        if patch.kind.as_deref() == Some("remove_blocking_margin") {
            ensure!(
                patch.map == "MnR03C00R01"
                    && patch.before_dimensions_metatiles == Some([24, 8])
                    && patch.after_dimensions_metatiles == Some([21, 8])
                    && patch.removed_bounds_metatiles == Some([21, 0, 24, 8])
                    && [old.attributes.width, old.attributes.height] == [24, 8]
                    && patch.changes.is_empty(),
                "only literal blocking R03 margin may shrink"
            );
            for y in 0..16 {
                for x in 42..48 {
                    ensure!(
                        kind(base, old, [x, y])? == "WALL",
                        "R03 trim removes traversable collision"
                    );
                }
            }
            ensure!(
                old.objects
                    .iter()
                    .map(|o| [o.x, o.y])
                    .chain(old.events.warps.iter().map(|o| [o.x, o.y]))
                    .chain(old.events.coord_events.iter().map(|o| [o.x, o.y]))
                    .chain(old.events.bg_events.iter().map(|o| [o.x, o.y]))
                    .all(|p| p[0] < 42 && p[1] < 16),
                "R03 trim removes a native anchor"
            );
            map.blocks = (0..8)
                .flat_map(|y| old.blocks[y * 24..y * 24 + 21].iter().copied())
                .collect();
            map.attributes.width = 21;
        } else {
            ensure!(
                patch.kind.is_none()
                    && patch.dimensions_metatiles
                        == Some([old.attributes.width, old.attributes.height]),
                "port extent differs"
            );
            let mut seen = BTreeSet::new();
            for c in &patch.changes {
                let [x, y] = c.metatile;
                ensure!(
                    x < old.attributes.width
                        && y < old.attributes.height
                        && seen.insert(c.metatile)
                        && c.before != c.after
                        && c.after <= 255,
                    "invalid/duplicate port cell"
                );
                let i = usize::from(y) * usize::from(old.attributes.width) + usize::from(x);
                ensure!(old.blocks[i] == c.before, "stale native port cell");
                map.blocks[i] = c.after;
                cells += 1;
            }
        }
        maps.insert(patch.map.clone(), map);
    }
    ensure!(cells == 42, "exact42 port cells required");
    for update in &plan.existing_connection_updates {
        let map = existing(base, &mut maps, &update.map)?;
        let edge = map
            .attributes
            .connections
            .iter_mut()
            .find(|c| c.direction == update.side && c.target_map == update.target)
            .context("original reciprocal offset edge")?;
        ensure!(edge.offset == update.old_offset, "stale original offset");
        edge.offset = update.new_offset;
    }
    let expected = [
        ("MnDetroitLakesApproach", [24, 18]),
        ("MnMaraisCedarApproach", [32, 19]),
        ("MnPreservePineEntry", [24, 16]),
        ("MnBemidjiNorthwoods", [8, 14]),
    ];
    ensure!(
        plan.new_maps.len() == expected.len() && plan.landmark_bindings.len() == 4,
        "exact four new wooded maps required"
    );
    ensure!(
        plan.encounter_bindings_proposed.len() == 3
            && plan
                .encounter_bindings_proposed
                .iter()
                .map(|e| e.map.as_str())
                .collect::<BTreeSet<_>>()
                .len()
                == 3
            && plan.encounter_bindings_proposed.iter().all(|e| matches!(
                (e.map.as_str(), e.native_reference_map.as_str()),
                ("MnDetroitLakesApproach", "MnR15C00R00")
                    | ("MnMaraisCedarApproach", "MnR11C00R00")
                    | ("MnBemidjiNorthwoods", "MnR17C00R00")
            )),
        "exact contextual native grass bindings required"
    );
    let mut dossiers = BTreeMap::new();
    for (index, (source, (name, size))) in plan.new_maps.iter().zip(expected).enumerate() {
        ensure!(
            source.id == name
                && source.dimensions_metatiles == size
                && source.map_id == u16::try_from(index + 1)?
                && !base.data.maps.contains_key(name)
                && !base
                    .data
                    .runtime_map_metadata
                    .contains_key(&source.map_constant),
            "fixed new connector identity/extent occupied or changed"
        );
        let text = source_text(sources, &source.dossier_file, &source.dossier_sha256)?;
        let dossier: Dossier = serde_json::from_str(text)?;
        ensure!(
            dossier.id == name
                && dossier.actors.len() == (if index < 2 { 2 } else { 0 })
                && dossier.signs.len() == (if index < 3 { 1 } else { 0 }),
            "connector dossier identity/activity count differs"
        );
        maps.insert(source.id.clone(), compile(base, &plan, source, sources)?);
        dossiers.insert(source.id.clone(), dossier);
    }
    for link in &plan.new_reciprocal_links {
        ensure!(
            link.reverse_offset == link.offset.checked_neg().context("offset overflow")?,
            "reciprocal offset differs"
        );
        let reverse = crate::state_cohesion_validation::reverse(&link.direction)?;
        for (name, side, target, offset) in [
            (
                &link.from_map,
                link.direction.as_str(),
                &link.to_map,
                link.offset,
            ),
            (&link.to_map, reverse, &link.from_map, link.reverse_offset),
        ] {
            crate::lake_country_geometry::connect(
                existing(base, &mut maps, name)?,
                side,
                target,
                offset,
            )?;
        }
    }
    let mut pack = base.clone();
    crate::state_cohesion_validation::geometry(&pack, &maps, &plan, &dossiers, false)?;
    crate::state_cohesion_events::install(&mut pack, &mut maps, &plan, &dossiers)?;
    let expected_maps = maps.clone();
    for (name, map) in &maps {
        if let Some(old) = base.data.maps.get(name) {
            if !plan.guard_proposals.iter().any(|g| g.map == *name) {
                // Terrain-only publication must not rewrite inherited script,
                // event or NPC mirrors, including historical parser differences.
                publish_geometry(&mut pack, old, map)?;
                continue;
            }
            crate::authored_town_publication::replace(&mut pack, map.clone())?;
        }
    }
    pack.data
        .story_event_script_constants
        .global
        .insert(plan.registry.map_group_constant.clone(), 83);
    let temporary = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|p| p.identifier)
        .max()
        .context("native spawn table")?
        .checked_add(1)
        .context("temporary spawn overflow")?
        .max(256);
    for source in &plan.new_maps {
        publish(
            &mut pack,
            maps.remove(&source.id).context("new connector module")?,
            source,
            &plan,
            temporary,
        )?;
    }
    pack.report.maps = pack.data.maps.len();
    crate::state_cohesion_validation::validate(base, &pack, &plan, &dossiers, &expected_maps)?;
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

fn publish_geometry(pack: &mut CompiledGamePack, old: &MapModule, map: &MapModule) -> Result<()> {
    crate::state_cohesion_validation::geometry_only(old, map)?;
    let label = old.attributes.blocks_label.as_ref().context("old blocks")?;
    ensure!(
        !pack.data.maps.values().any(
            |other| other.id != old.id && other.attributes.blocks_label.as_ref() == Some(label)
        ),
        "shared geometry-only block payload"
    );
    let bytes = map
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    *pack
        .data
        .map_blocks
        .get_mut(label)
        .context("old block payload")? = crate::encode_base64_bytes(&bytes);
    pack.data
        .map_attributes
        .insert(map.id.clone(), map.attributes.clone());
    if map.attributes.width != old.attributes.width {
        let constant = old
            .attributes
            .map_constant
            .as_ref()
            .context("old constant")?;
        pack.data.map_dimensions.insert(
            constant.clone(),
            serde_json::json!({"width":map.attributes.width,"height":map.attributes.height}),
        );
        let metadata = pack
            .data
            .runtime_map_metadata
            .get_mut(constant)
            .context("old metadata")?;
        metadata.width = map.attributes.width;
    }
    pack.data.maps.insert(map.id.clone(), map.clone());
    Ok(())
}

pub(crate) fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(crate) fn hash_value(value: &Value) -> Result<String> {
    Ok(hash_bytes(&serde_json::to_vec(value)?))
}
fn source_text<'a>(
    sources: &'a BTreeMap<String, String>,
    path: &str,
    sha: &str,
) -> Result<&'a str> {
    let text = sources
        .get(path)
        .with_context(|| format!("missing exact source {path}"))?;
    ensure!(
        hash_bytes(text.as_bytes()) == sha,
        "source file hash differs: {path}"
    );
    Ok(text)
}
fn existing<'a>(
    base: &CompiledGamePack,
    maps: &'a mut BTreeMap<String, MapModule>,
    name: &str,
) -> Result<&'a mut MapModule> {
    if !maps.contains_key(name) {
        maps.insert(
            name.into(),
            base.data
                .maps
                .get(name)
                .with_context(|| format!("missing old connector neighbor {name}"))?
                .clone(),
        );
    }
    maps.get_mut(name).context("cohesion map")
}
fn compile(
    base: &CompiledGamePack,
    plan: &Plan,
    source: &NewMap,
    sources: &BTreeMap<String, String>,
) -> Result<MapModule> {
    let text = source_text(sources, &source.layout_file, &source.layout_sha256)?;
    let value: Value = serde_json::from_str(text)?;
    let layout: Layout = serde_json::from_str(text)?;
    ensure!(
        layout.schema_version == 2
            && layout.id == source.id
            && layout.dimensions_metatiles == source.dimensions_metatiles
            && layout.compiled_tileset == "johto"
            && source.tileset == "johto"
            && source.environment == "ROUTE"
            && value["source_pixels_modified"] == false
            && value.get("derived_blocks").is_none()
            && base.audio_manifest.music.contains_key(&source.music),
        "native connector art/extent/environment contract"
    );
    let mut map = authoring::blank(base, "Route30", &source.id, &source.map_constant)?;
    map.attributes.width = source.dimensions_metatiles[0];
    map.attributes.height = source.dimensions_metatiles[1];
    map.attributes.tileset_name = "johto".into();
    map.attributes.environment = Some("ROUTE".into());
    map.attributes.music = Some(source.music.clone());
    map.attributes.map_group_constant = Some(plan.registry.map_group_constant.clone());
    map.attributes.border_block = 5;
    map.blocks = layout.compile()?;
    let expected_parent = match source.id.as_str() {
        "MnDetroitLakesApproach" => "MnBrainerd",
        "MnMaraisCedarApproach" => "MnGrandMarais",
        "MnPreservePineEntry" => "MnBoundaryWaters",
        "MnBemidjiNorthwoods" => "MnBemidji",
        _ => anyhow::bail!("unknown connector location family"),
    };
    let binding = plan
        .landmark_bindings
        .iter()
        .find(|b| b.map == source.id)
        .context("explicit connector landmark")?;
    let landmark = base
        .data
        .pokegear_landmark_for_map(&binding.reference_native_map)?;
    ensure!(
        !binding.add_landmark_row
            && binding.reference_native_map == expected_parent
            && landmark.constant == binding.landmark_constant
            && landmark.id == binding.existing_landmark_id
            && (1..=127).contains(&landmark.id),
        "exact existing geographic landmark required"
    );
    map.attributes.location = Some(landmark.constant.clone());
    Ok(map)
}
fn publish(
    pack: &mut CompiledGamePack,
    module: MapModule,
    source: &NewMap,
    plan: &Plan,
    temporary: u16,
) -> Result<()> {
    let location = module
        .attributes
        .location
        .clone()
        .context("native connector location")?;
    ensure!(
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(source.id.clone(), location)
            .is_none(),
        "occupied connector landmark mapping"
    );
    let wild = plan
        .encounter_bindings_proposed
        .iter()
        .find(|e| e.map == source.id)
        .map(|binding| {
            let mut data = pack
                .data
                .wild_encounters
                .get(&binding.native_reference_map)
                .context("native reference grass encounter table")?
                .clone();
            ensure!(
                data.grass.is_some() && data.grass_rates.is_some(),
                "reference has no native grass encounters"
            );
            data.map_name = source.id.clone();
            data.water = None;
            data.water_rate = None;
            data.swarm_overrides.clear();
            data.zones.clear();
            Ok::<_, anyhow::Error>(data)
        })
        .transpose()?;
    let arrival = (0..module.attributes.height * 2)
        .flat_map(|y| (0..module.attributes.width * 2).map(move |x| [x, y]))
        .find(|p| {
            kind(pack, &module, *p).is_ok_and(|k| k == "FLOOR")
                && !module.objects.iter().any(|o| [o.x, o.y] == *p)
        })
        .context("safe connector arrival")?;
    let extension = CompiledMapExtension {
        manifest_id: format!("{}-{}", plan.id, source.map_id),
        map_name: source.id.clone(),
        map_constant: source.map_constant.clone(),
        metadata: RuntimeMapMetadata {
            name: source.id.clone(),
            constant: source.map_constant.clone(),
            group_name: plan.registry.group_name.clone(),
            group_id: 83,
            map_id: source.map_id,
            width: source.dimensions_metatiles[0],
            height: source.dimensions_metatiles[1],
            environment: "ROUTE".into(),
            phone_service: module.attributes.phone_service,
        },
        spawn_key: temporary.to_string(),
        spawn: RuntimeSpawnPoint {
            identifier: temporary,
            map_name: source.id.clone(),
            map_constant: source.map_constant.clone(),
            group_name: plan.registry.group_name.clone(),
            group_id: 83,
            map_id: i16::try_from(source.map_id)?,
            tile_x: i16::try_from(arrival[0])?,
            tile_y: i16::try_from(arrival[1])?,
            metatile_x: i16::try_from(arrival[0] / 2)?,
            metatile_y: i16::try_from(arrival[1] / 2)?,
            subtile_x: i16::try_from(arrival[0] % 2)?,
            subtile_y: i16::try_from(arrival[1] % 2)?,
        },
        module,
        wild_encounters: wild,
        start_new_game_here: false,
    };
    crate::validate_compiled_map_extension(&extension)?;
    crate::insert_compiled_map_extension(pack, extension)?;
    pack.data
        .runtime_spawn_points
        .remove(&temporary.to_string());
    Ok(())
}
