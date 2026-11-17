//! Checked, non-destructive migration into the unselected eight-gym review pack.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_assets::{AssetRoot, MapModule};
use crystal_core::state::{
    BattleMemory, GameState, OverworldMemory, OverworldObjectMapMemory, OverworldObjectMemory,
};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;

use crate::CrystalRuntime;

const SOURCE_HASH: &str = "834d4be07c45079fdc81f1777856a37058d064b4188f548e31616684436da5e3";
const TARGET_HASHES: &[&str] = &[
    "ba3b067a92192dad36d9b8884296dc036653d2acf3e3eb22325cef8b3ee0fd73",
    "a4bb4b8069988d50d389606ff9e30ce4fc18bc5b6a634a76c5d7a0af5ddeec59",
];
const GYMS: &[&str] = &[
    "MnRochesterGym",
    "MnMankatoGym",
    "MnMinneapolisGym",
    "MnStPaulGym",
    "MnStCloudGym",
    "MnBrainerdGym",
    "MnBemidjiGym",
    "MnDuluthGym",
];
const SHOP: &str = "MnStPaulAnglerHouse";
const DISPLAY: &str = "MNSTPAULANGLERHOUSE_NPC3";

/// Migrate a verified selected-world save into the exact gym review candidate.
///
/// The original state is never modified. Recognized old gym tile overrides are
/// discarded so the target's native map callbacks rebuild gates from retained
/// event flags. Saved gym actors are moved to their new authored positions.
/// A player saved inside a revised gym resumes at its target arrival; a player
/// on either new canoe-shop obstruction moves one tile south. The new shop
/// display is added to an active saved object roster without resetting its
/// existing actors. Other progress and map state are retained.
///
/// The caller must read the original bytes against the source pack identity
/// before calling this function, and must write the result to a new save slot.
/// This helper does not select a pack, write a save, or publish anything.
///
/// # Errors
/// Rejects different packs, an invalid source state, a pending battle or scene,
/// unknown gym overrides, moving or unexpected gym actors, unsafe player
/// movement, incompatible canoe-shop state, or an invalid target state.
pub fn migrate_gym_review_save(
    source: &CrystalRuntime,
    target: &CrystalRuntime,
    asset_root: &AssetRoot,
    state: &GameState,
) -> Result<GameState> {
    ensure!(
        source.pack_identity().content_hash == SOURCE_HASH,
        "unexpected source pack"
    );
    ensure!(
        TARGET_HASHES.contains(&target.pack_identity().content_hash.as_str()),
        "unexpected gym review pack"
    );
    source.validate_save_state_for_runtime_pack(state)?;
    let script = &state.script_runtime;
    ensure!(
        matches!(state.battle, BattleMemory::Inactive)
            && state.pending_static_wild_terminal.is_none()
            && state.pending_move_learn.is_none()
            && state.pending_move_learn_queue.is_empty()
            && !script.window_open
            && !script.text_window_open
            && !script.player_input_locked
            && !script.all_input_locked
            && script.active_menu.is_none()
            && script.next_script.is_none()
            && script.call_stack.is_empty()
            && script.deferred_scripts.is_empty()
            && script.map_reentry_script.is_none()
            && script.pending_script_warp.is_none()
            && script.pending_map_load.is_none()
            && script.pending_map_refresh.is_none()
            && script.pending_field_travel.is_none()
            && script.pending_block_field_move.is_none()
            && script.pending_flash_field_move.is_none()
            && script.pending_surf_field_move.is_none()
            && script.pending_waterfall_field_move.is_none()
            && script.pending_yes_no.is_none()
            && script.pending_text_wait.is_none()
            && script.pending_text_label.is_none()
            && script.active_text_label.is_none()
            && script.pending_screen_fade.is_none()
            && script.pending_card_flip_input.is_none()
            && script.pending_slot_machine_input.is_none()
            && script.pending_memory_game_input.is_none()
            && script.pending_day_care_input.is_none()
            && script.pending_delays.is_empty()
            && script.pending_earthquakes.is_empty()
            && script.pending_emotes.is_empty()
            && script.command_queue.is_empty(),
        "save has a pending battle, scene, field move or transition"
    );
    let mut migrated = state.clone();
    for &name in GYMS {
        let old = source.data().maps.get(name).context("source gym")?;
        let new = target.data().maps.get(name).context("target gym")?;
        ensure!(
            old.objects.len() == new.objects.len()
                && old
                    .objects
                    .iter()
                    .zip(&new.objects)
                    .all(|(a, b)| a.object_identifier.is_some()
                        && a.object_identifier == b.object_identifier),
            "gym actor order changed: {name}"
        );
        ensure!(
            old.events.warps.len() == new.events.warps.len()
                && old
                    .events
                    .warps
                    .iter()
                    .zip(&new.events.warps)
                    .all(|(a, b)| {
                        a.index == b.index
                            && a.target_map_constant == b.target_map_constant
                            && a.target_warp_id == b.target_warp_id
                    }),
            "gym warp identities changed: {name}"
        );
        ensure!(
            new.map_script_section_commands.iter().any(|command| {
                command.command == "callback"
                    && command.args.first().map(String::as_str) == Some("MAPCALLBACK_TILES")
            }),
            "target gym lacks a tile callback: {name}"
        );
        if let Some(overrides) = migrated.map_block_overrides.get(name) {
            for (&(x, y), &block) in overrides {
                let scripted = old
                    .script_block_changes
                    .iter()
                    .any(|c| c.x / 2 == x && c.y / 2 == y && c.block_id == block);
                let original = old
                    .blocks
                    .get(usize::from(y) * usize::from(old.attributes.width) + usize::from(x))
                    .context("gym override coordinate")?;
                let cut = source
                    .data()
                    .field_moves
                    .cut
                    .replacements
                    .get(&old.attributes.tileset_name)
                    .and_then(|entries| entries.get(original))
                    .is_some_and(|entry| entry.replacement_block_id == block);
                ensure!(
                    scripted || cut,
                    "unknown saved gym block at {name} ({x},{y})"
                );
            }
        }
        migrated.map_block_overrides.remove(name);
        if let Some(memory) = migrated.map_object_overrides.get_mut(name) {
            remap_gym_actors(name, old, new, memory)?;
        }
    }
    if let Some(overrides) = migrated.map_block_overrides.get_mut("MnStPaul")
        && let Some(block) = overrides.remove(&(7, 31))
    {
        ensure!(
            block == 0x55,
            "unexpected saved terrain at the canoe-shop sign"
        );
    }
    let OverworldMemory::Active {
        map_name,
        tile,
        facing,
        mode,
    } = &mut migrated.overworld
    else {
        anyhow::bail!("save has no active overworld");
    };
    if GYMS.contains(&map_name.as_str()) {
        let room = target
            .data()
            .maps
            .get(map_name)
            .context("target current gym")?;
        let arrival = room
            .events
            .warps
            .iter()
            .find(|warp| warp.index == 3)
            .context("gym arrival")?;
        ensure!(
            !room
                .objects
                .iter()
                .any(|o| (o.x, o.y) == (arrival.x, arrival.y)),
            "gym arrival occupied"
        );
        *tile = TilePosition::new(i16::try_from(arrival.x)?, i16::try_from(arrival.y)?);
        *facing = Direction::Down;
        *mode = MovementMode::Normal;
        migrated
            .script_runtime
            .memory
            .remove(crystal_core::systems::field_moves::CANOE_ACTIVE_MEMORY);
        clear_player_footprint(migrated.map_object_overrides.get_mut(map_name))?;
    } else if (map_name.as_str(), tile.x, tile.y) == ("MnStPaul", 15, 63) {
        tile.y = 64;
        clear_player_footprint(migrated.map_object_overrides.get_mut(map_name))?;
    } else if (map_name.as_str(), tile.x, tile.y) == (SHOP, 11, 5) {
        tile.y = 6;
        clear_player_footprint(migrated.map_object_overrides.get_mut(map_name))?;
    }
    if matches!(&migrated.overworld, OverworldMemory::Active { map_name, .. } if map_name == SHOP)
        && migrated.map_object_overrides.contains_key(SHOP)
    {
        add_shop_display(target, asset_root, &mut migrated)?;
    }
    target.validate_save_state_for_runtime_pack(&migrated)?;
    Ok(migrated)
}

fn clear_player_footprint(memory: Option<&mut OverworldObjectMapMemory>) -> Result<()> {
    if let Some(memory) = memory {
        let roster = &mut memory.object_structs;
        ensure!(
            memory.following.is_none()
                && !memory.player_hidden
                && roster.following_queued_step.is_none()
                && roster.player_last_tile_occupied_remaining_frames == 0
                && !roster.player_sliding
                && !roster.player_normal_following
                && !roster.player_fixed_facing,
            "player or follower movement is still active"
        );
        roster.player_last_tile = None;
        roster.last_step_direction = None;
    }
    Ok(())
}

fn remap_gym_actors(
    name: &str,
    old: &MapModule,
    new: &MapModule,
    memory: &mut OverworldObjectMapMemory,
) -> Result<()> {
    ensure!(
        memory.following.is_none() && memory.object_structs.following_queued_step.is_none(),
        "gym follower active: {name}"
    );
    for (id, position) in &mut memory.objects {
        let index = old
            .objects
            .iter()
            .position(|o| o.object_identifier.as_deref() == Some(id.as_str()))
            .context("old gym actor")?;
        let before = &old.objects[index];
        let after = &new.objects[index];
        ensure!(
            (position.x, position.y) == (before.x, before.y),
            "unexpected gym actor position: {name}/{id}"
        );
        position.x = after.x;
        position.y = after.y;
    }
    for actor in &mut memory.object_structs.structs {
        let index = usize::from(actor.map_object_index)
            .checked_sub(1)
            .context("gym actor index")?;
        let before = old.objects.get(index).context("old gym actor index")?;
        let after = new.objects.get(index).context("new gym actor index")?;
        let tile = TilePosition::new(i16::try_from(before.x)?, i16::try_from(before.y)?);
        ensure!(
            actor.live_tile == tile
                && actor.initial_tile == tile
                && actor.last_tile.is_none()
                && actor.step_duration.is_none()
                && !actor.strength_moving
                && !actor.sliding
                && !actor.normal_following
                && actor.following_not_exact_leader_slot.is_none(),
            "gym actor movement or position is unexpected: {name}"
        );
        actor.live_tile = TilePosition::new(i16::try_from(after.x)?, i16::try_from(after.y)?);
        actor.initial_tile = actor.live_tile;
    }
    Ok(())
}

fn add_shop_display(
    target: &CrystalRuntime,
    root: &AssetRoot,
    state: &mut GameState,
) -> Result<()> {
    let room = target.data().maps.get(SHOP).context("canoe shop")?;
    let display_index = room
        .objects
        .iter()
        .position(|o| o.object_identifier.as_deref() == Some(DISPLAY))
        .context("canoe display")?;
    ensure!(
        display_index == 2
            && (room.objects[display_index].x, room.objects[display_index].y) == (11, 5),
        "unexpected canoe display"
    );
    let mut probe = state.clone();
    probe.map_object_overrides.remove(SHOP);
    let session = target
        .resume_overworld_session(root, probe)
        .context("prepare canoe shop roster")?;
    let generated = session
        .state()
        .map_object_overrides
        .get(SHOP)
        .context("generated canoe shop roster")?;
    let memory = state
        .map_object_overrides
        .get_mut(SHOP)
        .context("saved canoe shop roster")?;
    ensure!(
        memory.following.is_none()
            && memory.object_structs.following_queued_step.is_none()
            && !memory.object_structs.player_normal_following
            && !memory.objects.contains_key(DISPLAY)
            && !memory
                .object_structs
                .structs
                .iter()
                .any(|s| usize::from(s.map_object_index) == display_index + 1),
        "canoe display already present"
    );
    let object = &room.objects[display_index];
    memory.objects.insert(
        DISPLAY.to_owned(),
        OverworldObjectMemory {
            x: object.x,
            y: object.y,
        },
    );
    let template = generated
        .object_structs
        .structs
        .iter()
        .find(|s| usize::from(s.map_object_index) == display_index + 1)
        .context("canoe display is not loaded in the target shop")?;
    let occupied: BTreeSet<u8> = memory
        .object_structs
        .structs
        .iter()
        .map(|s| s.slot)
        .collect();
    let slot = (1..=12)
        .find(|slot| !occupied.contains(slot))
        .context("no free canoe display slot")?;
    let mut display = template.clone();
    display.slot = slot;
    memory.object_structs.structs.push(display);
    Ok(())
}

#[cfg(test)]
#[path = "gym_review_save_migration_tests.rs"]
mod tests;
