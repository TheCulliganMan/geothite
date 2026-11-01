//! Narrow publication and canonical Gym6 preflight for Brainerd.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::brainerd_city_modpack::{CONSTANT, GROUP, GROUP_ID, LANDMARK, MANIFEST, TOWN, Town};
use crate::gym_design::GymDesign;
use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, PokegearLandmark, RuntimeMapMetadata,
    RuntimeSpawnPoint,
};

pub(crate) fn available(base: &CompiledGamePack, town: &Town) -> Result<()> {
    ensure!(
        !base
            .data
            .runtime_map_metadata
            .values()
            .any(|m| m.group_id == GROUP_ID || m.group_name == GROUP)
            && !base
                .data
                .story_event_script_constants
                .global
                .iter()
                .any(|(key, value)| key.starts_with("MAPGROUP_") && *value == i64::from(GROUP_ID))
            && !base
                .data
                .story_event_script_constants
                .global
                .contains_key(&format!("MAPGROUP_{GROUP}"))
            && !base
                .report
                .manifests
                .iter()
                .any(|id| id.starts_with(MANIFEST)),
        "Brainerd group69/group constant/manifest occupied"
    );
    let expected = [
        ("MnBrainerdPokecenter1F", [10, 8], "pokecenter"),
        ("MnBrainerdMart", [6, 4], "mart"),
        ("MnBrainerdGym", [12, 12], "gym_6"),
        ("MnBrainerdRangerStation", [8, 6], "ranger_station"),
        ("MnBrainerdOutfitter", [6, 4], "camp_supply"),
    ];
    ensure!(
        town.interiors.len() == expected.len(),
        "Brainerd room roster"
    );
    for (room, (id, size, kind)) in town.interiors.iter().zip(expected) {
        ensure!(
            room.id == id
                && (id == "MnBrainerdGym" || room.dimensions_metatiles == size)
                && room.kind == kind,
            "retain complete Brainerd room identities/extents"
        );
    }
    for (name, constant) in std::iter::once((TOWN, CONSTANT)).chain(
        town.interiors
            .iter()
            .filter(|r| r.id != "MnBrainerdGym")
            .map(|r| (r.id.as_str(), r.native_constant.as_str())),
    ) {
        ensure!(
            !base.data.maps.contains_key(name)
                && !base.data.runtime_map_metadata.contains_key(constant)
                && !base
                    .data
                    .runtime_map_metadata
                    .values()
                    .any(|m| m.name == name)
                && !base
                    .data
                    .story_event_script_constants
                    .global
                    .contains_key(constant),
            "Brainerd map or constant occupied: {name}"
        );
    }
    ensure!(
        !base
            .data
            .pokegear_landmarks
            .landmarks
            .iter()
            .any(|l| l.constant == LANDMARK),
        "Brainerd landmark occupied"
    );
    Ok(())
}

fn sorted_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .map(|(key, value)| (key.clone(), sorted_json(value)))
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(sorted_json).collect()),
        _ => value.clone(),
    }
}

pub(crate) fn canonical_gym(
    base: &CompiledGamePack,
    town: &Town,
    town_json: &str,
    source: &str,
) -> Result<MapModule> {
    let design: GymDesign = serde_json::from_str(source).context("canonical native gym source")?;
    design.validate()?;
    let gym = design
        .gyms
        .iter()
        .find(|g| g.slot == 6 && g.id == "MnBrainerdGym")
        .context("canonical Brainerd Gym6")?;
    let reference = town
        .interiors
        .iter()
        .find(|r| r.id == gym.id)
        .context("Brainerd Gym6 reference")?;
    let raw: Value = serde_json::from_str(source)?;
    let object = raw["gyms"]
        .as_array()
        .context("canonical gyms")?
        .iter()
        .find(|g| g["id"] == gym.id)
        .context("canonical Gym6 object")?;
    let digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&sorted_json(object))?)
    );
    ensure!(
        reference.canonical_source.as_deref() == Some("planning/minnesota/gyms/interiors.json")
            && reference.canonical_id.as_deref() == Some(gym.id.as_str())
            && reference.canonical_gym_sha256.as_deref() == Some(digest.as_str())
            && reference.dimensions_metatiles == gym.dimensions_metatiles
            && reference.native_constant == gym.constant
            && reference.rows.is_empty()
            && reference.residents.is_empty(),
        "stale or alternative Brainerd canonical Gym6 source reference"
    );
    let raw_town: Value = serde_json::from_str(town_json)?;
    let raw_reference = raw_town["interiors"]
        .as_array()
        .context("Brainerd interior references")?
        .iter()
        .find(|r| r["id"] == gym.id)
        .context("raw Gym6 reference")?;
    ensure!(
        [
            "rows",
            "art",
            "residents",
            "party",
            "stations",
            "gates",
            "reward",
            "tileset"
        ]
        .iter()
        .all(|key| raw_reference.get(key).is_none()),
        "Brainerd city must not redefine canonical Gym6"
    );
    let installed = base.data.maps.get(&gym.id).context(
        "Brainerd requires the already installed canonical Gym6, not an unbuilt source plan",
    )?;
    // Reuse the production authoring implementation to compare the complete
    // expected scripts/objects, not just a source SHA. Its installer needs an
    // owned working pack; this private scratch clone is never published.
    let mut scratch = base.clone();
    let target = gym.tileset();
    scratch.data.tilesets.remove(&target);
    for path in [
        format!("data/tilesets/{target}_metatiles.bin"),
        format!("data/tilesets/{target}.json"),
        format!("data/tilesets/{target}_palette_map.json"),
        format!("gfx/tilesets/{target}.2bpp"),
        format!("gfx/tilesets/{target}.png"),
    ] {
        scratch.runtime_files.remove(&path);
    }
    scratch.data.field_moves.cut.replacements.remove(&target);
    scratch
        .data
        .field_moves
        .whirlpool
        .replacements
        .remove(&target);
    scratch
        .report
        .manifests
        .retain(|m| m != "minnesota-gym-6-art-v1");
    let symbols = crate::gym_art::install(&mut scratch, gym)?;
    ensure!(
        scratch.data.tilesets.get(&target) == base.data.tilesets.get(&target),
        "installed Gym6 native collision/palette differs from canonical art"
    );
    for path in [
        format!("data/tilesets/{target}_metatiles.bin"),
        format!("data/tilesets/{target}.json"),
        format!("data/tilesets/{target}_palette_map.json"),
        format!("gfx/tilesets/{target}.2bpp"),
        format!("gfx/tilesets/{target}.png"),
    ] {
        ensure!(
            scratch.runtime_files.get(&path) == base.runtime_files.get(&path),
            "installed Gym6 source glyphs differ from canonical recipe: {path}"
        );
    }
    ensure!(
        scratch.data.field_moves.cut.replacements.get(&target)
            == base.data.field_moves.cut.replacements.get(&target)
            && scratch.data.field_moves.whirlpool.replacements.get(&target)
                == base.data.field_moves.whirlpool.replacements.get(&target),
        "installed Gym6 field replacement aliases differ from canonical art"
    );
    let trainer_ids = std::iter::once(gym.battle_id())
        .chain(gym.trainers.iter().map(|t| t.id.clone()))
        .collect::<Vec<_>>();
    for id in &trainer_ids {
        scratch.data.trainers.trainers.remove(id);
        scratch.data.story_event_script_constants.global.remove(id);
    }
    let mut expected = crate::gym_publication::room(&scratch, gym, &symbols, 5)?;
    crate::gym_scripts::install(&mut scratch, &mut expected, gym, &symbols)?;
    crate::gym_publication::validate(base, installed, gym, &symbols)?;
    ensure!(
        &expected == installed,
        "installed Gym6 module differs from current canonical geometry/scripts/actors or review-hall bindings"
    );
    for id in &trainer_ids {
        ensure!(
            scratch.data.trainers.trainers.get(id) == base.data.trainers.trainers.get(id),
            "installed canonical Brainerd party/moves differ: {id}"
        );
    }
    let door = town
        .doors
        .iter()
        .find(|d| d.target == gym.id)
        .context("Brainerd Gym6 real city facade")?;
    ensure!(
        door.target_warp == 3
            && door.return_warp_index == 6
            && door.tile == [12, 35]
            && door.return_tile == [12, 36]
            && reference.passive_arrival.index == 3
            && reference.passive_arrival.tile == gym.arrival_core
            && reference.exits.len() == 2
            && reference
                .exits
                .iter()
                .zip(gym.exit_tiles_core)
                .all(|(e, at)| e.tile == at && e.target == TOWN && e.target_warp == 6),
        "Brainerd native Gym6/city door pairing mismatch"
    );
    let mut rebound = installed.clone();
    ensure!(
        rebound.events.warps.len() == 3,
        "canonical Gym6 needs two carpets and passive3"
    );
    for warp in &mut rebound.events.warps {
        warp.target_map_constant = CONSTANT.into();
        warp.target_map = CONSTANT.into();
        warp.target_warp_id = 6;
    }
    let mut count = 0;
    for command in &mut rebound.map_event_section_commands {
        if command.command == "warp_event" {
            ensure!(command.args.len() == 4, "Gym6 raw native warp shape");
            command.args[2] = CONSTANT.into();
            command.args[3] = "6".into();
            count += 1;
        }
    }
    ensure!(count == 3, "Gym6 raw three-warps section mismatch");
    Ok(rebound)
}

pub(crate) fn publish(
    mut pack: CompiledGamePack,
    modules: Vec<(MapModule, [u16; 2])>,
    gym: MapModule,
) -> Result<CompiledGamePack> {
    pack.data
        .story_event_script_constants
        .global
        .insert(format!("MAPGROUP_{GROUP}"), i64::from(GROUP_ID));
    let landmark = pack
        .data
        .pokegear_landmarks
        .landmarks
        .iter()
        .map(|l| l.id)
        .max()
        .context("source landmarks")?
        .checked_add(1)
        .context("Brainerd landmark overflow")?;
    pack.data
        .pokegear_landmarks
        .landmarks
        .push(PokegearLandmark {
            id: landmark,
            constant: LANDMARK.into(),
            label: CONSTANT.into(),
            name: "BRAINERD".into(),
            x: 88,
            y: 80,
            region: "JOHTO".into(),
        });
    let temporary = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("original recovery table")?
        .checked_add(1)
        .context("temporary Brainerd insertion anchor")?;
    for (index, (module, [x, y])) in modules.into_iter().enumerate() {
        ensure!(
            crate::brainerd_city_geometry::kind(&pack, &module, [x, y])? == "FLOOR"
                && !module.objects.iter().any(|o| [o.x, o.y] == [x, y])
                && module.attributes.connections.is_empty(),
            "Brainerd safe arrival and unbound regional ports"
        );
        let name = module.id.clone();
        let constant = module
            .attributes
            .map_constant
            .clone()
            .context("Brainerd native constant")?;
        let map_id = u16::try_from(index + 1)?;
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(name.clone(), LANDMARK.into());
        let extension = CompiledMapExtension {
            manifest_id: format!("{MANIFEST}-{index}"),
            map_name: name.clone(),
            map_constant: constant.clone(),
            metadata: RuntimeMapMetadata {
                name: name.clone(),
                constant: constant.clone(),
                group_name: GROUP.into(),
                group_id: GROUP_ID,
                map_id,
                width: module.attributes.width,
                height: module.attributes.height,
                environment: module
                    .attributes
                    .environment
                    .clone()
                    .context("Brainerd native environment")?,
                phone_service: module.attributes.phone_service,
            },
            spawn_key: temporary.to_string(),
            spawn: RuntimeSpawnPoint {
                identifier: temporary,
                map_constant: constant,
                map_name: name,
                group_name: GROUP.into(),
                group_id: i16::try_from(GROUP_ID)?,
                map_id: i16::try_from(map_id)?,
                tile_x: i16::try_from(x)?,
                tile_y: i16::try_from(y)?,
                metatile_x: i16::try_from(x / 2)?,
                metatile_y: i16::try_from(y / 2)?,
                subtile_x: i16::try_from(x % 2)?,
                subtile_y: i16::try_from(y % 2)?,
            },
            module,
            wild_encounters: None,
            start_new_game_here: false,
        };
        crate::validate_compiled_map_extension(&extension)?;
        crate::insert_compiled_map_extension(&mut pack, extension)?;
        pack.data
            .runtime_spawn_points
            .remove(&temporary.to_string());
    }
    crate::insert_compiled_map_split_payloads(
        &mut pack.data.map_scripts,
        &mut pack.data.npcs,
        &gym.id,
        gym.attributes
            .map_scripts_label
            .as_deref()
            .context("Gym6 script label")?,
        gym.attributes
            .map_events_label
            .as_deref()
            .context("Gym6 event label")?,
        &gym,
    )?;
    pack.data.maps.insert(gym.id.clone(), gym);
    pack.report
        .manifests
        .push(format!("{MANIFEST}-canonical-gym6-binding"));
    pack.report.maps = pack.data.maps.len();
    pack.report.reachable_maps.sort();
    pack.report.solvable_maps.sort();
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
