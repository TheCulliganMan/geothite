//! Additive, visit-unlocked Fly points at existing Minnesota dry landings.
use anyhow::{Context, Result, ensure};
use crystal_core::map::MapScriptSectionCommand;
use serde::Deserialize;

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, FlyDestination, RuntimeSpawnPoint};

#[derive(Deserialize)]
struct World {
    towns: Vec<Town>,
}

#[derive(Deserialize)]
struct Town {
    id: String,
}

/// Registers visit-unlocked Fly destinations on a completed Minnesota pack.
/// Existing maps, landing spawns, audio and unrelated game content are preserved.
///
/// # Errors
/// Returns an error for an invalid pack, already registered regional Fly points,
/// missing dry landings, conflicting flags or an invalid resulting pack.
pub fn build_minnesota_fly_destinations(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::minnesota_trainers::geography::require_compact_superior(base)?;
    let mut pack = base.clone();
    install(&mut pack)?;
    pack.report
        .manifests
        .push("minnesota-flight-atlas-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    ensure!(
        pack.compiled_audio == base.compiled_audio
            && pack.audio_manifest == base.audio_manifest
            && pack.runtime_files == base.runtime_files,
        "Fly registration must preserve all source art and audio"
    );
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let world: World =
        serde_json::from_str(include_str!("../../../planning/minnesota/world.json"))?;
    let mut next_spawn = 512_u16;
    let mut next_flag = pack
        .data
        .story_event_script_constants
        .global
        .iter()
        .filter(|(name, _)| name.starts_with("ENGINE_"))
        .map(|(_, value)| *value)
        .max()
        .context("native engine constants")?;
    let mut installed = 0;
    for town in world.towns {
        let Some(mut map) = pack.data.maps.get(&town.id).cloned() else {
            continue;
        };
        let constant = map
            .attributes
            .map_constant
            .clone()
            .context("Fly town constant")?;
        let label = map.attributes.location.clone().context("Fly landmark")?;
        ensure!(
            !pack
                .data
                .fly_destinations
                .values()
                .any(|d| d.label == label),
            "occupied Fly landmark {label}"
        );
        // Prefer a town's existing passive doorway return. It is
        // public dry floor, not an active carpet or a new fabricated landing.
        let landing = map
            .events
            .warps
            .iter()
            .map(|warp| [warp.x, warp.y])
            .find(|tile| {
                crate::authored_town_geometry::kind(pack, &map, *tile).ok() == Some("FLOOR")
                    && !map.objects.iter().any(|actor| [actor.x, actor.y] == *tile)
            })
            .or_else(|| {
                pack.data
                    .runtime_spawn_points
                    .values()
                    .filter(|spawn| spawn.map_name == town.id)
                    .filter_map(|spawn| {
                        Some([
                            u16::try_from(spawn.tile_x).ok()?,
                            u16::try_from(spawn.tile_y).ok()?,
                        ])
                    })
                    .find(|tile| {
                        crate::authored_town_geometry::kind(pack, &map, *tile).ok() == Some("FLOOR")
                            && !map.objects.iter().any(|actor| [actor.x, actor.y] == *tile)
                    })
            })
            .or_else(|| {
                map.events
                    .warps
                    .iter()
                    .filter(|warp| {
                        crate::authored_town_geometry::kind(pack, &map, [warp.x, warp.y]).ok()
                            == Some("DOOR")
                    })
                    .filter_map(|warp| Some([warp.x, warp.y.checked_add(1)?]))
                    .find(|tile| {
                        crate::authored_town_geometry::kind(pack, &map, *tile).ok() == Some("FLOOR")
                            && !map.objects.iter().any(|actor| [actor.x, actor.y] == *tile)
                            && !map
                                .events
                                .coord_events
                                .iter()
                                .any(|event| [event.x, event.y] == *tile)
                    })
            })
            .with_context(|| format!("existing dry public Fly landing for {}", town.id))?;
        while pack
            .data
            .runtime_spawn_points
            .contains_key(&next_spawn.to_string())
        {
            next_spawn = next_spawn.checked_add(1).context("Fly spawn overflow")?;
        }
        let metadata = pack
            .data
            .runtime_map_metadata
            .get(&constant)
            .context("Fly town metadata")?;
        let existing_spawn = pack
            .data
            .runtime_spawn_points
            .values()
            // Native Fly reserves source-byte IDs 28..=255. Keep those
            // existing home/blackout spawns intact and use a new Fly landing.
            .filter(|spawn| !(28..=255).contains(&spawn.identifier))
            .find(|spawn| spawn.map_name == town.id)
            .cloned();
        if let Some(existing) = &existing_spawn {
            let tile = [
                u16::try_from(existing.tile_x)?,
                u16::try_from(existing.tile_y)?,
            ];
            ensure!(
                crate::authored_town_geometry::kind(pack, &map, tile)? == "FLOOR"
                    && !map.objects.iter().any(|actor| [actor.x, actor.y] == tile),
                "existing town spawn must be a safe Fly landing: {}",
                town.id
            );
        }
        let spawn = RuntimeSpawnPoint {
            identifier: next_spawn,
            map_constant: constant.clone(),
            map_name: town.id.clone(),
            group_name: metadata.group_name.clone(),
            group_id: i16::try_from(metadata.group_id)?,
            map_id: i16::try_from(metadata.map_id)?,
            tile_x: i16::try_from(landing[0])?,
            tile_y: i16::try_from(landing[1])?,
            metatile_x: i16::try_from(landing[0] / 2)?,
            metatile_y: i16::try_from(landing[1] / 2)?,
            subtile_x: i16::try_from(landing[0] % 2)?,
            subtile_y: i16::try_from(landing[1] % 2)?,
        };
        let flag = format!("ENGINE_FLYPOINT_{constant}");
        ensure!(
            !pack.data.fly_destinations.contains_key(&flag)
                && !pack
                    .data
                    .story_event_script_constants
                    .global
                    .contains_key(&flag),
            "occupied Fly flag {flag}"
        );
        next_flag = next_flag.checked_add(1).context("Fly flag overflow")?;
        pack.data
            .story_event_script_constants
            .global
            .insert(flag.clone(), next_flag);
        let callback = format!("{}UnlockFlypoint", town.id);
        ensure!(
            !map.scripts.contains_key(&callback),
            "occupied Fly callback"
        );
        let programs = std::collections::BTreeMap::from([(
            callback.clone(),
            serde_json::json!([
                authoring::command("setflag", &[&flag]),
                authoring::command("endcallback", &[]),
            ]),
        )]);
        map.script_flag_commands
            .extend(crate::parse_script_flag_commands(&map.id, &programs)?);
        map.script_control_commands
            .extend(crate::parse_script_control_commands(&map.id, &programs)?);
        map.scripts.extend(programs);
        map.map_script_section_commands
            .push(MapScriptSectionCommand {
                command: "callback".into(),
                args: vec!["MAPCALLBACK_NEWMAP".into(), callback],
                command_index: map.map_script_section_commands.len(),
            });
        crate::authored_town_publication::replace(pack, map)?;
        let destination_spawn_identifier = if let Some(existing) = existing_spawn {
            existing.identifier
        } else {
            pack.data
                .runtime_spawn_points
                .insert(next_spawn.to_string(), spawn);
            next_spawn
        };
        pack.data.fly_destinations.insert(
            flag.clone(),
            FlyDestination {
                flypoint_flag: flag,
                destination_spawn_identifier,
                label,
            },
        );
        installed += 1;
    }
    ensure!(installed >= 15, "regional Fly destinations are incomplete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{AssetRoot, FlyDestination, GameDataSet, PlayabilityRules, verify_game_data};

    #[test]
    fn fly_spawn_ranges_preserve_the_reserved_source_byte_gap() {
        for identifier in [0, 27, 28, 255, 256, 512] {
            let mut data = GameDataSet::default();
            data.fly_destinations.insert(
                "ENGINE_FLYPOINT_TEST".into(),
                FlyDestination {
                    flypoint_flag: "ENGINE_FLYPOINT_TEST".into(),
                    destination_spawn_identifier: identifier,
                    label: "LANDMARK_TEST".into(),
                },
            );
            let report = verify_game_data(
                &AssetRoot::new(env!("CARGO_MANIFEST_DIR")),
                &data,
                &PlayabilityRules::default(),
            );
            let range_error = report
                .diagnostics
                .iter()
                .any(|error| error.code == "fly_destination_spawn_out_of_range");
            assert_eq!(
                range_error,
                (28..=255).contains(&identifier),
                "spawn {identifier}"
            );
            if !range_error {
                assert!(
                    report
                        .diagnostics
                        .iter()
                        .any(|error| error.code == "unknown_fly_destination_spawn"),
                    "registration is still mandatory for {identifier}"
                );
            }
        }
    }
}
