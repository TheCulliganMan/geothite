//! Checked migration from the November 2026 hosted Northstar pack.
use anyhow::{Result, ensure};
use crystal_core::state::{BattleMemory, GameState, OverworldMemory};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;

use crate::CrystalRuntime;

const TARGET_HOSTED_HASH: &str = "2164f61b4d431999284bdab1584e2dda0b753d8944141a2c707de8bb7329aec8";
const RESTORED_CENTERS_MARKER: &str = "minnesota-native-center-restoration-v1";
const RESIZED_CENTERS: &[&str] = &[
    "MnBemidjiPokecenter1F",
    "MnBrainerdPokecenter1F",
    "MnDetroitLakesPokecenter1F",
    "MnDuluthFieldClinic",
    "MnDuluthPokecenter1F",
    "MnGrandRapidsPokecenter1F",
    "MnInternationalFallsPokecenter1F",
    "MnLeechLakeCareCabin",
    "MnMankatoPokecenter1F",
    "MnMinneapolisPokecenter1F",
    "MnRochesterPokecenter1F",
    "MnStCloudPokecenter1F",
];

/// Validate and migrate one saved state without modifying its original bytes.
///
/// # Errors
/// The caller must first verify the source save against the pinned previous identity.
/// Rejects a different destination, pending scenes, incompatible center overrides,
/// or a state invalid for the destination runtime.
pub fn migrate_northstar_reviewed_save(
    target: &CrystalRuntime,
    state: &GameState,
) -> Result<GameState> {
    ensure!(
        target.pack_identity().content_hash == TARGET_HOSTED_HASH,
        "unexpected target Northstar pack"
    );
    ensure!(
        target
            .modpack()
            .id()
            .split('+')
            .any(|part| part == RESTORED_CENTERS_MARKER),
        "target lacks restored centers"
    );
    ensure!(
        matches!(&state.battle, BattleMemory::Inactive)
            && state.pending_static_wild_terminal.is_none(),
        "save has an active battle or transition"
    );
    let script = &state.script_runtime;
    ensure!(
        !script.window_open
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
            && script.command_queue.is_empty(),
        "save has a pending scene or transition"
    );
    let mut migrated = state.clone();
    for &name in RESIZED_CENTERS {
        let new = target
            .data()
            .maps
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("restored center {name} is missing"))?;
        ensure!(
            migrated
                .map_block_overrides
                .get(name)
                .is_none_or(|blocks| blocks.is_empty()),
            "save has terrain changes in resized center {name}"
        );
        if let Some(memory) = migrated.map_object_overrides.get_mut(name) {
            ensure!(
                memory.following.is_none()
                    && memory.object_structs.following_queued_step.is_none()
                    && memory.object_structs.player_last_tile.is_none()
                    && memory
                        .object_structs
                        .player_last_tile_occupied_remaining_frames
                        == 0
                    && !memory.object_structs.player_sliding
                    && !memory.object_structs.player_normal_following,
                "save has a pending player or follower movement in resized center {name}"
            );
            for (id, position) in &mut memory.objects {
                let after = new
                    .objects
                    .iter()
                    .find(|object| object.object_identifier.as_deref() == Some(id.as_str()))
                    .ok_or_else(|| anyhow::anyhow!("new center actor {id} is missing"))?;
                position.x = after.x;
                position.y = after.y;
            }
            for actor in &mut memory.object_structs.structs {
                let index = usize::from(actor.map_object_index)
                    .checked_sub(1)
                    .ok_or_else(|| anyhow::anyhow!("invalid center actor index"))?;
                let after = new
                    .objects
                    .get(index)
                    .ok_or_else(|| anyhow::anyhow!("new center actor index is missing"))?;
                ensure!(
                    actor.last_tile.is_none()
                        && actor.step_duration.is_none()
                        && !actor.strength_moving
                        && !actor.sliding
                        && !actor.normal_following,
                    "center actor has a pending movement"
                );
                actor.live_tile =
                    TilePosition::new(i16::try_from(after.x)?, i16::try_from(after.y)?);
                actor.initial_tile = actor.live_tile;
            }
        }
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
    if RESIZED_CENTERS.contains(&map_name.as_str()) {
        *tile = TilePosition::new(3, 5);
        *facing = Direction::Down;
        *mode = MovementMode::Normal;
    }
    target.validate_save_state_for_runtime_pack(&migrated)?;
    Ok(migrated)
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};
    use std::path::PathBuf;

    use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
    use crystal_core::state::OverworldMemory;
    use crystal_core::world::map::TilePosition;

    use super::RESIZED_CENTERS;
    use crate::{CrystalRuntime, RuntimeGameShell, migrate_northstar_reviewed_save};

    #[test]
    #[ignore = "requires the verified previous and hosted candidate packs"]
    fn migrates_a_live_parent_center_save_without_changing_progress() {
        let previous_path = PathBuf::from(
            std::env::var_os("CRYSTAL_NORTHSTAR_PREVIOUS_PACK").expect("previous pack"),
        );
        let target_path = PathBuf::from(
            std::env::var_os("CRYSTAL_NORTHSTAR_HOSTED_PACK").expect("hosted candidate"),
        );
        let root = AssetRoot::new(previous_path.parent().expect("pack directory"));
        let previous = CrystalRuntime::from_loaded_compiled_pack(
            &root,
            read_loaded_verified_compiled_game_pack(previous_path).expect("previous pack verified"),
        )
        .expect("previous runtime");
        let target = CrystalRuntime::from_loaded_compiled_pack(
            &root,
            read_loaded_verified_compiled_game_pack(target_path).expect("candidate verified"),
        )
        .expect("target runtime");
        assert_eq!(
            previous.pack_identity().content_hash,
            "17bbf71fd2fa731036b93863bd4dceac0a1b6f18e2568d30cb1f187112ff1341"
        );
        assert_eq!(
            previous.modpack().hash(),
            "a129f66ca806cd4c9f5ed0030c622a600da86c0e748f844001898a890b9f6c91"
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(previous.modpack().id().as_bytes())),
            "d70761f0a31858f579c7f67f41deeb46b03e6e63152fea4cede8eaba47c67447"
        );
        let resized_centers: Vec<_> = previous
            .data()
            .maps
            .iter()
            .filter_map(|(name, old)| {
                let new = target.data().maps.get(name)?;
                (name.starts_with("Mn")
                    && (old.attributes.width, old.attributes.height)
                        != (new.attributes.width, new.attributes.height)
                    && matches!(
                        new.attributes.tileset_name.as_str(),
                        "pokecenter" | "pokecom_center"
                    )
                    && new
                        .objects
                        .iter()
                        .any(|object| object.sprite == "SPRITE_NURSE"))
                .then_some(name.as_str())
            })
            .collect();
        assert_eq!(resized_centers, RESIZED_CENTERS);
        for &name in RESIZED_CENTERS {
            let old = &previous.data().maps[name];
            let new = &target.data().maps[name];
            let identifiers = |room: &crystal_assets::MapModule| {
                room.objects
                    .iter()
                    .map(|object| object.object_identifier.clone())
                    .collect::<Vec<_>>()
            };
            assert_eq!(identifiers(old), identifiers(new), "actor order for {name}");
        }
        let mut game = RuntimeGameShell::new_game_at_runtime_tile(
            root.clone(),
            previous.clone(),
            previous.title_new_game_spawn_identifier().expect("spawn"),
            "MnRochesterPokecenter1F",
            10,
            13,
        )
        .expect("old center fixture");
        game.session_mut().state_mut().money = 4321;
        let original = game.session().state().clone();
        assert!(migrate_northstar_reviewed_save(&previous, &original).is_err());
        let migrated =
            migrate_northstar_reviewed_save(&target, &original).expect("checked migration");
        assert_eq!(migrated.money, 4321);
        assert!(
            matches!(migrated.overworld, OverworldMemory::Active { tile, .. } if tile == TilePosition::new(3, 5))
        );
        assert!(
            matches!(original.overworld, OverworldMemory::Active { tile, .. } if tile == TilePosition::new(10, 13))
        );
        let migrated_nurse =
            &migrated.map_object_overrides["MnRochesterPokecenter1F"].objects["MN_RC_NURSE"];
        assert_eq!((migrated_nurse.x, migrated_nurse.y), (3, 1));
        assert_eq!(
            migrated.map_object_overrides["MnRochesterPokecenter1F"].hidden_object_identifiers,
            original.map_object_overrides["MnRochesterPokecenter1F"].hidden_object_identifiers
        );
        let mut pending = original.clone();
        pending.script_runtime.player_input_locked = true;
        assert!(migrate_northstar_reviewed_save(&target, &pending).is_err());
        for (name, x, y) in [
            ("MnDuluthFieldClinic", 8, 9),
            ("MnLeechLakeCareCabin", 10, 13),
            ("MnInternationalFallsPokecenter1F", 3, 8),
        ] {
            let game = RuntimeGameShell::new_game_at_runtime_tile(
                root.clone(),
                previous.clone(),
                previous.title_new_game_spawn_identifier().expect("spawn"),
                name,
                x,
                y,
            )
            .expect("old clinic fixture");
            let migrated = migrate_northstar_reviewed_save(&target, game.session().state())
                .expect("checked clinic migration");
            assert!(
                matches!(migrated.overworld, OverworldMemory::Active { tile, .. } if tile == TilePosition::new(3, 5)),
                "{name}"
            );
        }
    }
}
