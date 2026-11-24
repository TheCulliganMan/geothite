//! Copy approved saves onto the rebuilt cave layouts without replacing progress.

use anyhow::{Context, Result, ensure};
use crystal_assets::GameDataSet;
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::systems::field_moves::CANOE_ACTIVE_MEMORY;
use crystal_core::world::movement::MovementMode;
use serde::Deserialize;

use crate::room_save_migration::{ArrivalSurface, RoomReentry};

use super::{Compatibility, CompatibleSaveSource, dex_order_save_migration};

const MARKER: &str = "+northstar-cave-audit-v5";
const SUFFIX: &str = "+realtime-clock+player-customization";
const POLICY: &str = "minnesota-cave-audit-v5";
const PARENT_HASHES: [&str; 2] = [
    "9415b9ae8def05303e8cd07ade5525c9c6f92c8a775498cb69e67975a19e7f13",
    "3f1df5134910d4c8b32e31428a668736ca8fbbc8b05e7c240865abcf6c7276d8",
];
const PARENT_CONTENT: [&str; 2] = [
    "8123e801db425666fefa8c5d2b532f0c9f130ecc4b59b1636fe20a5ee251add6",
    "d77d63d93dc30b953cc5a8f8a8681a6c2af2fca4a4b480b86360863a1f13a8f6",
];
const ROOMS: [&str; 19] = [
    "MnR01Cave1F",
    "MnDriftlessLampGallery",
    "MnR03Cave1F",
    "MnR05RockCut",
    "MnR13Cave1F",
    "MnR17Cave1F",
    "MnR20Cave1F",
    "MnR12Cave1F",
    "MnR10Cave1F",
    "MnInnerBasin",
    "MnR09Cave1F",
    "MnIronDrillApproach",
    "MnIronHaulage",
    "MnIronSump",
    "MnIronVentilation",
    "MnIronPumpGallery",
    "MnIronDrillCore",
    "MnVoyageursIce1F",
    "MnVoyageursIceB1F",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Release {
    schema_version: u8,
    sources: [CompatibleSaveSource; 2],
    inherited: serde_json::Value,
    reentry: RoomReentry,
}

pub(super) fn parse(bytes: &[u8], target: &str) -> Result<Compatibility> {
    let release: Release = serde_json::from_slice(bytes)?;
    let [raw, hosted] = &release.sources;
    let target_base = target.strip_suffix(SUFFIX).unwrap_or(target);
    let parent = target_base
        .strip_suffix(MARKER)
        .context("invalid cave audit target")?;
    ensure!(
        release.schema_version == 12
            && parent == raw.modpack_id
            && parent.ends_with("+northstar-dex-order-v1")
            && hosted.modpack_id.strip_prefix(parent) == Some(SUFFIX)
            && release.sources.iter().enumerate().all(|(index, source)| {
                source.modpack_hash == PARENT_HASHES[index]
                    && source.content_hash == PARENT_CONTENT[index]
            }),
        "expected exact Pokédex ordering sources"
    );
    let parent_id = if target_base == target {
        &raw.modpack_id
    } else {
        &hosted.modpack_id
    };
    release.reentry.validate_metadata(POLICY, &ROOMS)?;
    let mut policy =
        dex_order_save_migration::parse(&serde_json::to_vec(&release.inherited)?, parent_id)?;
    // The older cave and Iron landings describe terrain now replaced in full.
    // Earlier Gym and non-cave checkpoint transformations still apply.
    policy.cave_reentry = None;
    policy.iron_relocations = None;
    policy
        .fair_layouts
        .retain(|map, _| !ROOMS.contains(&map.as_str()));
    policy.cave_audit_reentry = Some(release.reentry);
    policy.sources.splice(0..0, release.sources);
    Ok(policy)
}

pub(super) fn is_direct_parent(source: &CompatibleSaveSource) -> bool {
    PARENT_HASHES.contains(&source.modpack_hash.as_str())
}

pub(super) fn validate_arrivals(policy: &RoomReentry, data: &GameDataSet) -> Result<()> {
    policy.validate_arrivals(data, ArrivalSurface::Floor)?;
    for room in ROOMS {
        let (arrival, _) = policy.arrival(room).context("missing cave audit arrival")?;
        let module = data.map_module(room)?;
        ensure!(
            module.events.coord_events.iter().all(|event| {
                i32::from(event.x) != i32::from(arrival.x)
                    || i32::from(event.y) != i32::from(arrival.y)
            }),
            "cave audit arrival overlaps a coordinate event: {room}"
        );
    }
    Ok(())
}

pub(super) fn reenter(
    policy: &RoomReentry,
    state: &mut GameState,
    data: &GameDataSet,
) -> Result<()> {
    state
        .map_block_overrides
        .retain(|map, _| policy.arrival(map).is_none());
    for (map, saved) in &mut state.map_object_overrides {
        if policy.arrival(map).is_none() {
            continue;
        }
        ensure!(
            !saved.player_hidden && !saved.object_structs.player_fixed_facing,
            "finish hidden or fixed-facing cave scenes before upgrading: {map}"
        );
        // Continue restores show/hide receipts without rerunning object callbacks.
        saved.objects.clear();
        saved.object_structs = Default::default();
        saved.last_talked_object_identifier = None;
    }
    let OverworldMemory::Active {
        map_name,
        tile,
        facing,
        mode,
    } = &mut state.overworld
    else {
        anyhow::bail!("cave audit upgrade requires an overworld checkpoint");
    };
    let Some((arrival, direction)) = policy.arrival(map_name) else {
        return Ok(());
    };
    *tile = arrival;
    *facing = direction;
    *mode = MovementMode::Normal;
    let map_name = map_name.clone();
    state.script_runtime.memory.remove(CANOE_ACTIVE_MEMORY);
    state.script_runtime.last_talked_object = None;

    // Continue does rerun tile callbacks. Rebuild sprite allocation separately
    // against the new authored positions, current flags, and saved visibility.
    let mut session = data.overworld_session(&map_name, arrival, state.frame_counter)?;
    session.set_time(state.time.registers.hours, state.time.time_of_day);
    session.sync_event_flag_memory(&state.flags);
    data.apply_saved_overworld_overrides(&mut session, state)?;
    if map_name == "MnIronDrillCore" {
        // Continue skips the object callback that stages Finn after the drill
        // stops. Keep only the reconstructed room state from this replay.
        let mut checkpoint = state.clone();
        data.apply_map_setup_callbacks(&mut checkpoint, &mut session, &map_name, "MAPSETUP_WARP")?;
        if let Some(objects) = checkpoint.map_object_overrides.remove(&map_name) {
            state.map_object_overrides.insert(map_name.clone(), objects);
        }
    }
    session.begin_map_object_setup();
    session.finish_map_object_setup()?;
    state
        .map_object_overrides
        .entry(map_name)
        .or_default()
        .object_structs = session.object_struct_roster_memory()?;
    Ok(())
}

#[cfg(test)]
#[path = "cave_audit_save_migration_tests.rs"]
mod tests;
