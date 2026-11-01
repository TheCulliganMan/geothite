//! Add the hand-authored Rainy River Icefields without rebuilding old maps.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::WildEncounterData;
use serde_json::Value;

use crate::grand_marais_modpack::authoring;
use crate::winter_region_design::{
    self as design, ALIAS, GROUP, GROUP_CONSTANT, GROUP_ID, HUT, HUT_CONSTANT, NEIGHBOR,
    NEIGHBOR_HASH, OUTDOOR, OUTDOOR_CONSTANT, Region,
};
use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, RuntimeMapMetadata, RuntimeSpawnPoint,
};

/// Installs the frozen32×24 outdoor region and its real5×4 warming shelter.
///
/// Requires the already installed native winter-art alias, exact original R20
/// neighbor and native templates, and the reviewed source inputs. The only old
/// map change is R20's previously unused west connection and its attributes
/// mirror. Existing programs, rooms, terrain, art, audio, HOME and recovery are
/// retained. New trainers use native learned moves and freshly allocated flags
/// and class-local identifiers. The shelter heals without awarding story flags
/// or changing blackout recovery. Both maps reuse the existing R20 landmark.
///
/// # Errors
/// Returns an error for stale source/template/art preimages, occupied identities,
/// unsafe geometry/arrivals, unavailable native species/items/classes, failed
/// complete production assembly, unrelated mutation or native pack validation.
/// The borrowed input remains unchanged on failure.
pub fn build_winter_region_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let (layout, region) = design::parse(layout_json, dossier_json)?;
    preflight(base)?;
    let blocks = layout.compile()?;
    let bytes = blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(
        design::hash(&bytes) == design::SOURCE_BLOCKS,
        "native compiled icefields source differs"
    );
    let landmark = base.data.pokegear_landmark_for_map(NEIGHBOR)?;
    ensure!(
        (1..=127).contains(&landmark.id),
        "existing R20 landmark must fit native table"
    );
    let location = landmark.constant.clone();
    let mut outdoor = authoring::blank(base, "Route30", OUTDOOR, OUTDOOR_CONSTANT)?;
    outdoor.attributes.width = 32;
    outdoor.attributes.height = 24;
    outdoor.attributes.tileset_name = ALIAS.into();
    outdoor.attributes.border_block = 0x3f;
    outdoor.attributes.environment = Some("ROUTE".into());
    ensure!(
        base.audio_manifest.music.contains_key(&region.music)
            && base.data.maps["Route42"].attributes.music.as_deref() == Some(region.music.as_str()),
        "native winter music missing"
    );
    outdoor.attributes.music = Some(region.music.clone());
    outdoor.blocks = blocks;
    crate::lake_country_geometry::connect(&mut outdoor, "east", NEIGHBOR, 0)?;
    let room = &region.interiors[0];
    let mut hut = authoring::blank(base, &room.template, HUT, HUT_CONSTANT)?;
    authoring::blocks(&mut hut, 5, 4, &room.rows.join(" "))?;
    hut.attributes.border_block = room.border_block.context("shelter border")?;
    for at in [[9, 27], [9, 28]] {
        authoring::warp(&mut outdoor, (at[0], at[1]), HUT_CONSTANT, 2);
    }
    for at in [[5, 7], [5, 6]] {
        authoring::warp(&mut hut, (at[0], at[1]), OUTDOOR_CONSTANT, 2);
    }
    for module in [&mut outdoor, &mut hut] {
        module.attributes.map_group_constant = Some(GROUP_CONSTANT.into());
        module.attributes.location = Some(location.clone());
    }
    let mut pack = base.clone();
    pack.data
        .story_event_script_constants
        .global
        .insert(GROUP_CONSTANT.into(), i64::from(GROUP_ID));
    let constants =
        crate::winter_region_events::install(&mut pack, &mut outdoor, &mut hut, &region)?;
    crate::winter_region_validation::geometry(&pack, &outdoor, &hut, &region)?;
    let mut neighbor = base
        .data
        .maps
        .get(NEIGHBOR)
        .context("R20 neighbor")?
        .clone();
    crate::lake_country_geometry::connect(&mut neighbor, "west", OUTDOOR, 0)?;
    pack.data
        .map_attributes
        .insert(NEIGHBOR.into(), neighbor.attributes.clone());
    pack.data.maps.insert(NEIGHBOR.into(), neighbor);
    let temporary = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("existing spawn namespace")?
        .checked_add(1)
        .context("temporary spawn overflow")?;
    for (module, constant, id, arrival, encounters) in [
        (outdoor, OUTDOOR_CONSTANT, 1, [60, 44], Some(wild(&region))),
        (hut, HUT_CONSTANT, 2, [5, 6], None),
    ] {
        publish(
            &mut pack, module, constant, id, arrival, temporary, encounters,
        )?;
    }
    // Only new maps are assembled. Legacy typed indexes and split programs are
    // never reparsed or rewritten for the R20 attributes-only connection.
    let mut expected = BTreeMap::new();
    for name in [OUTDOOR, HUT] {
        let mut authored = pack.data.maps.remove(name).context("new winter module")?;
        let assembled = pack.data.assemble_map_module_from_compiled_payloads(name)?;
        for label in [
            &authored.attributes.map_scripts_label,
            &authored.attributes.map_events_label,
        ] {
            let label = label.as_ref().context("new winter transport label")?;
            authored.scripts.insert(
                label.clone(),
                pack.data
                    .map_scripts
                    .get(label)
                    .context("new winter transport payload")?
                    .clone(),
            );
        }
        ensure!(
            authored == assembled,
            "new winter full cached/uncached module differs: {name}"
        );
        expected.insert(name.to_owned(), assembled.clone());
        pack.data.maps.insert(name.into(), assembled);
    }
    pack.report.maps = pack.data.maps.len();
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::winter_region_validation::connections(&pack)?;
    crate::winter_region_validation::preserve(base, &pack, &expected, &constants, &region)?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn preflight(base: &CompiledGamePack) -> Result<()> {
    ensure!(
        base.data.maps.len() == 681,
        "completed681-map winter-art parent required"
    );
    for (name, expected) in [
        (NEIGHBOR, NEIGHBOR_HASH),
        (
            "Route30",
            "ac4bba9a231d7370893f56e30ffa14cd27c33e5f88266eaa57836220b19e8fec",
        ),
        (
            "VioletNicknameSpeechHouse",
            "d59569387754b0f921b20598774e15ae21006c878e337c68a3037333c7010706",
        ),
    ] {
        let module = base
            .data
            .maps
            .get(name)
            .context("required native winter template")?;
        ensure!(
            crate::authored_river_connections::module_hash(module)? == expected,
            "required original winter preimage differs: {name}"
        );
    }
    ensure!(
        !base
            .data
            .runtime_map_metadata
            .values()
            .any(|m| m.group_id == GROUP_ID)
            && !base
                .data
                .story_event_script_constants
                .global
                .contains_key(GROUP_CONSTANT)
            && !base.data.maps.contains_key(OUTDOOR)
            && !base.data.maps.contains_key(HUT),
        "winter map identities/group84 occupied"
    );
    let alias = base
        .data
        .tilesets
        .get(ALIAS)
        .context("install exact winter art before region")?;
    let original = base
        .data
        .tilesets
        .get("mn_border_pines")
        .context("native winter donor")?;
    ensure!(
        alias.collision.len() == 65,
        "bounded winter art block count"
    );
    for (key, collision) in &original.collision {
        ensure!(
            alias.collision.get(key) == Some(collision),
            "native art-prefix collision changed"
        );
    }
    for (block, kind) in [
        (0x3c, "FLOOR"),
        (0x3d, "TALL_GRASS"),
        (0x3e, "ICE"),
        (0x3f, "WALL"),
        (0x40, "WALL"),
    ] {
        ensure!(
            alias
                .collision
                .get(&format!("{block:02x}"))
                .is_some_and(|q| q.iter().all(|v| v == kind)),
            "winter snow/grass/ice/canopy vocabulary differs"
        );
    }
    for (path, expected) in [
        (
            "gfx/tilesets/mn_winter_outdoor.2bpp",
            "07930abcc97f8e7242a1d5d239f922f06187afe50c5af71772630d2b4f2d0435",
        ),
        (
            "data/tilesets/mn_winter_outdoor_metatiles.bin",
            "db16ee0f4ad6c2b14298712656d6ae1e1f2205c6bc513e05767662097d3564d2",
        ),
    ] {
        ensure!(
            design::hash(
                base.runtime_files()
                    .get(path)
                    .context("installed winter native art")?
            ) == expected,
            "installed winter art recipe differs: {path}"
        );
    }
    let foregrounds: Value = serde_json::from_slice(
        base.runtime_files()
            .get("data/tilesets/mn_winter_outdoor_foregrounds.json")
            .context("winter colors")?,
    )?;
    let specification_json =
        include_str!("../../../planning/minnesota/regions/ice/winter-art-v01.json");
    ensure!(
        design::hash(specification_json.as_bytes())
            == "4efd8ef08fc06bfc870144014602aca5ca639b34015bade3dddb5ac769aedd74",
        "frozen winter art specification changed"
    );
    let specification: Value = serde_json::from_str(specification_json)?;
    let native_blocks = base
        .runtime_files()
        .get("data/tilesets/mn_winter_outdoor_metatiles.bin")
        .context("winter blocks")?;
    for (index, recipe) in specification["recipes"]
        .as_array()
        .context("winter recipes")?
        .iter()
        .enumerate()
    {
        for glyph in &native_blocks[(60 + index) * 16..(61 + index) * 16] {
            ensure!(
                alias.palette_map[usize::from(*glyph)] == (3 | if *glyph >= 128 { 8 } else { 0 }),
                "winter new glyph bank/palette differs"
            );
            ensure!(
                foregrounds[glyph.to_string()] == recipe["foregrounds"],
                "winter selective foreground colors differ"
            );
        }
    }
    Ok(())
}

pub(crate) fn wild(region: &Region) -> WildEncounterData {
    WildEncounterData {
        map_name: OUTDOOR.into(),
        grass_rates: Some(region.encounters.grass_rates.clone()),
        grass: Some(region.encounters.grass.clone()),
        ..Default::default()
    }
}

fn publish(
    pack: &mut CompiledGamePack,
    module: MapModule,
    constant: &str,
    id: u16,
    arrival: [u16; 2],
    temporary: u16,
    wild_encounters: Option<WildEncounterData>,
) -> Result<()> {
    let name = module.id.clone();
    let location = module
        .attributes
        .location
        .clone()
        .context("existing native winter landmark")?;
    ensure!(
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(name.clone(), location)
            .is_none(),
        "winter landmark binding occupied"
    );
    let extension = CompiledMapExtension {
        manifest_id: format!("minnesota-rainy-river-ice-{id}"),
        map_name: name.clone(),
        map_constant: constant.into(),
        metadata: RuntimeMapMetadata {
            name: name.clone(),
            constant: constant.into(),
            group_name: GROUP.into(),
            group_id: GROUP_ID,
            map_id: id,
            width: module.attributes.width,
            height: module.attributes.height,
            environment: module
                .attributes
                .environment
                .clone()
                .context("winter environment")?,
            phone_service: module.attributes.phone_service,
        },
        spawn_key: temporary.to_string(),
        spawn: RuntimeSpawnPoint {
            identifier: temporary,
            map_name: name,
            map_constant: constant.into(),
            group_name: GROUP.into(),
            group_id: i16::try_from(GROUP_ID)?,
            map_id: i16::try_from(id)?,
            tile_x: i16::try_from(arrival[0])?,
            tile_y: i16::try_from(arrival[1])?,
            metatile_x: i16::try_from(arrival[0] / 2)?,
            metatile_y: i16::try_from(arrival[1] / 2)?,
            subtile_x: i16::try_from(arrival[0] % 2)?,
            subtile_y: i16::try_from(arrival[1] % 2)?,
        },
        module,
        wild_encounters,
        start_new_game_here: false,
    };
    crate::validate_compiled_map_extension(&extension)?;
    crate::insert_compiled_map_extension(pack, extension)?;
    pack.data
        .runtime_spawn_points
        .remove(&temporary.to_string());
    Ok(())
}

#[cfg(test)]
#[path = "winter_region_tests.rs"]
mod tests;
