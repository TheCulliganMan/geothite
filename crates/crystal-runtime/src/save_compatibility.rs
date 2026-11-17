//! Explicit release compatibility and non-destructive slot selection.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_core::save::{self, SaveModpackIdentity};
use crystal_core::state::{BattleMemory, GameState, OverworldMemory};
use serde::Deserialize;

use crate::CrystalRuntime;

pub const SAVE_COMPATIBILITY_FILE: &str = "data/northstar-save-compatibility.json";
pub const SAVE_COMPATIBILITY_MANIFEST: &str = "northstar-refinement-v1";

/// One exact, author-approved source release. The hash covers serialized bytes.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompatibleSaveSource {
    pub modpack_id: String,
    pub modpack_hash: String,
    pub content_hash: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Compatibility {
    schema_version: u8,
    target_manifest: String,
    target_modpack_ids: Vec<String>,
    sources: Vec<CompatibleSaveSource>,
}

/// Returns the exact source identities approved by this target's hashed metadata.
///
/// # Errors
/// Rejects malformed metadata, unknown target markers, and duplicate identities.
pub fn compatible_save_sources(runtime: &CrystalRuntime) -> Result<Vec<CompatibleSaveSource>> {
    let Some(bytes) = runtime.runtime_file(SAVE_COMPATIBILITY_FILE) else {
        return Ok(Vec::new());
    };
    parse_sources(bytes, runtime.modpack().id())
}

fn parse_sources(bytes: &[u8], target_id: &str) -> Result<Vec<CompatibleSaveSource>> {
    let metadata: Compatibility =
        serde_json::from_slice(bytes).context("save compatibility metadata")?;
    ensure!(
        metadata.schema_version == 1 && metadata.target_manifest == SAVE_COMPATIBILITY_MANIFEST,
        "unsupported save compatibility schema"
    );
    ensure!(
        target_id
            .split('+')
            .any(|id| id == metadata.target_manifest),
        "save compatibility target marker is absent"
    );
    ensure!(
        !metadata.target_modpack_ids.is_empty()
            && metadata.target_modpack_ids.len() <= 2
            && metadata.target_modpack_ids.iter().any(|id| id == target_id),
        "save compatibility is not approved for this exact target"
    );
    for id in &metadata.target_modpack_ids {
        SaveModpackIdentity::validate_id(id)?;
    }
    ensure!(
        !metadata.sources.is_empty() && metadata.sources.len() <= 8,
        "bounded compatibility source list required"
    );
    let mut identities = BTreeSet::new();
    for source in &metadata.sources {
        SaveModpackIdentity::new(&source.modpack_id, &source.modpack_hash)?;
        ensure!(
            source.content_hash.len() == 64
                && source
                    .content_hash
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
            "invalid compatible content identity"
        );
        ensure!(
            identities.insert((
                &source.modpack_id,
                &source.modpack_hash,
                &source.content_hash
            )),
            "duplicate compatible source"
        );
    }
    Ok(metadata.sources)
}

/// Resumes/recover a current slot, or copies the first compatible prior slot.
///
/// Prior identities come only from this target's checked compatibility metadata.
/// Unknown, corrupt and unsafe older saves remain untouched and permit New Game.
/// Primary copies take precedence over backups. Current corrupt copies are
/// archived before recovery; valid prior copies are never modified.
///
/// # Errors
/// Rejects an unrecoverable current slot, invalid compatibility metadata, or a
/// destination/storage conflict. Full runtime validation precedes every write.
pub fn prepare_runtime_save_slot(
    runtime: &CrystalRuntime,
    destination: &Path,
    mut previous_path: impl FnMut(&CompatibleSaveSource) -> PathBuf,
) -> Result<bool> {
    let backup = save::save_backup_path(destination);
    if save::save_slot_exists(destination)? || save::save_slot_exists(&backup)? {
        let selected = read_pair(destination, |bytes| {
            let saved = save::read_save_game_bytes_for_modpack(
                bytes,
                destination.display().to_string(),
                runtime.modpack(),
                &runtime.pack_identity().content_hash,
            )?;
            runtime.validate_save_state_for_runtime_pack(saved.state())?;
            Ok(saved.into_state())
        })
        .context("neither current primary nor backup is valid; original files were preserved")?;
        if selected.backup {
            save::recover_save_primary(
                destination,
                selected.damaged_primary.as_deref(),
                &selected.bytes,
                runtime.modpack(),
                &runtime.pack_identity().content_hash,
            )?;
        }
        return Ok(true);
    }
    for source in compatible_save_sources(runtime)? {
        let path = previous_path(&source);
        if path == destination {
            continue;
        }
        let identity = SaveModpackIdentity::new(&source.modpack_id, &source.modpack_hash)?;
        let Some(selected) = read_pair(&path, |bytes| {
            let saved = save::read_save_game_bytes_for_modpack(
                bytes,
                path.display().to_string(),
                &identity,
                &source.content_hash,
            )?;
            let upgraded = normalize_upgrade_checkpoint(saved.into_state())?;
            runtime.validate_save_state_for_runtime_pack(&upgraded)?;
            Ok(upgraded)
        }) else {
            continue;
        };
        save::write_new_compatible_save(
            destination,
            selected.state,
            runtime.modpack(),
            &runtime.pack_identity().content_hash,
        )?;
        return Ok(true);
    }
    Ok(false)
}

struct SelectedSave {
    state: GameState,
    bytes: Vec<u8>,
    backup: bool,
    damaged_primary: Option<Vec<u8>>,
}

fn read_pair(
    path: &Path,
    mut validate: impl FnMut(&[u8]) -> Result<GameState>,
) -> Option<SelectedSave> {
    let primary = save::read_save_slot_bytes(path).ok().flatten();
    if let Some(bytes) = primary.as_ref()
        && let Ok(state) = validate(bytes)
    {
        return Some(SelectedSave {
            state,
            bytes: primary.expect("validated primary exists"),
            backup: false,
            damaged_primary: None,
        });
    }
    let bytes = save::read_save_slot_bytes(&save::save_backup_path(path))
        .ok()
        .flatten()?;
    let state = validate(&bytes).ok()?;
    Some(SelectedSave {
        state,
        bytes,
        backup: true,
        damaged_primary: primary,
    })
}

// The caller has checked the exact source identity and checksum before this
// copy reaches us. Completed source-command positions are presentation history,
// not executable work; edited dialogue may legitimately move those positions.
// Never clear active ownership or gameplay state to make an upgrade validate.
fn normalize_upgrade_checkpoint(mut state: GameState) -> Result<GameState> {
    validate_upgrade_checkpoint(&state)?;
    state.script_runtime.script_ended = None;
    state.script_runtime.text_events.clear();
    state.script_runtime.active_text_label = None;
    Ok(state)
}

fn validate_upgrade_checkpoint(state: &GameState) -> Result<()> {
    ensure!(
        matches!(state.battle, BattleMemory::Inactive)
            && state.pending_static_wild_terminal.is_none()
            && state.pending_move_learn.is_none()
            && state.pending_move_learn_queue.is_empty(),
        "finish the active battle before upgrading"
    );
    ensure!(
        matches!(state.overworld, OverworldMemory::Active { .. }),
        "upgrade requires an active overworld checkpoint"
    );
    let script = &state.script_runtime;
    ensure!(
        !script.window_open
            && !script.text_window_open
            && !script.player_input_locked
            && !script.all_input_locked
            && script.active_menu.is_none()
            && script.pending_yes_no.is_none()
            && script.pending_shop.is_none()
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
            && script.pending_delays.is_empty()
            && script.pending_earthquakes.is_empty()
            && script.pending_emotes.is_empty()
            && script.pending_text_label.is_none()
            && script.pending_text_wait.is_none()
            && script.active_pokemon_picture.is_none()
            && script.pending_screen_fade.is_none()
            && script.pending_music_fade.is_none()
            && !script.waiting_for_sound_effect
            && script.audio_events.is_empty()
            && script.graphics_events.is_empty()
            && script.money_events.is_empty()
            && script.map_events.is_empty()
            && script.control_events.is_empty()
            && script.shop_events.is_empty()
            && script.item_use_events.is_empty()
            && script.command_queue.is_empty(),
        "finish the active scene before upgrading"
    );
    for memory in state.map_object_overrides.values() {
        ensure!(
            memory.following.is_none()
                && memory.object_structs.following_queued_step.is_none()
                && memory
                    .object_structs
                    .player_last_tile_occupied_remaining_frames
                    == 0
                && !memory.object_structs.player_sliding
                && !memory.object_structs.player_normal_following,
            "finish movement before upgrading"
        );
        ensure!(
            memory.object_structs.structs.iter().all(|actor| actor
                .last_tile_occupied_remaining_frames
                == 0
                && actor.strength_push_direction.is_none()
                && !actor.strength_moving
                && !actor.sliding
                && !actor.normal_following
                && actor.following_not_exact_leader_slot.is_none()),
            "finish actor movement before upgrading"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_sources, read_pair, validate_upgrade_checkpoint};
    use crystal_core::save::{
        SaveModpackIdentity, read_save_game_bytes_for_modpack, write_new_compatible_save,
    };
    use crystal_core::state::{GameState, OverworldMemory};
    use crystal_core::world::map::{Direction, TilePosition};
    use crystal_core::world::movement::MovementMode;

    #[test]
    fn compatibility_requires_exact_destination_and_source_identities() {
        let metadata = serde_json::json!({"schema_version":1,"target_manifest":"northstar-refinement-v1",
            "target_modpack_ids":["core+northstar-refinement-v1"],
            "sources":[{"modpack_id":"core+minnesota-adventure","modpack_hash":"a".repeat(64),"content_hash":"b".repeat(64)}]});
        let bytes = serde_json::to_vec(&metadata).expect("metadata");
        assert_eq!(
            parse_sources(&bytes, "core+northstar-refinement-v1")
                .expect("authorized destination")
                .len(),
            1
        );
        assert!(parse_sources(&bytes, "core+northstar-refinement-v1+next-release").is_err());
        let mut invalid = metadata.clone();
        invalid["sources"][0]["modpack_hash"] = "wrong".into();
        assert!(
            parse_sources(
                &serde_json::to_vec(&invalid).expect("invalid metadata"),
                "core+northstar-refinement-v1"
            )
            .is_err()
        );
    }

    #[test]
    fn upgrade_rejects_pending_scenes_without_changing_progress() {
        let mut state = GameState {
            overworld: OverworldMemory::Active {
                map_name: "MnWinona".into(),
                tile: TilePosition::new(3, 3),
                facing: Direction::Down,
                mode: MovementMode::Normal,
            },
            money: 4321,
            ..GameState::default()
        };
        assert!(validate_upgrade_checkpoint(&state).is_ok());
        state.script_runtime.text_window_open = true;
        assert!(validate_upgrade_checkpoint(&state).is_err());
        assert_eq!(state.money, 4321);
    }

    #[test]
    fn completed_steps_retain_historical_tiles_without_blocking_upgrade() {
        use crystal_core::state::OverworldObjectMapMemory;
        use crystal_core::world::session::OverworldObjectStructMemory;
        let mut state = GameState {
            overworld: OverworldMemory::Active {
                map_name: "MnWinona".into(),
                tile: TilePosition::new(3, 3),
                facing: Direction::Down,
                mode: MovementMode::Normal,
            },
            ..GameState::default()
        };
        let mut memory = OverworldObjectMapMemory::default();
        memory.object_structs.player_last_tile = Some(TilePosition::new(3, 2));
        memory
            .object_structs
            .structs
            .push(OverworldObjectStructMemory {
                slot: 1,
                map_object_index: 1,
                live_tile: TilePosition::new(5, 5),
                last_tile: Some(TilePosition::new(5, 4)),
                initial_tile: TilePosition::new(5, 4),
                facing: None,
                step_duration: Some(12),
                last_tile_occupied_remaining_frames: 0,
                pending_random_wait: true,
                initialized_fixed_spin: false,
                strength_push_direction: None,
                strength_moving: false,
                fixed_facing: false,
                sliding: false,
                visible: true,
                normal_following: false,
                following_not_exact_leader_slot: None,
            });
        state.map_object_overrides.insert("MnWinona".into(), memory);
        assert!(validate_upgrade_checkpoint(&state).is_ok());
        state
            .map_object_overrides
            .get_mut("MnWinona")
            .expect("map")
            .object_structs
            .player_last_tile_occupied_remaining_frames = 1;
        assert!(validate_upgrade_checkpoint(&state).is_err());
    }

    #[test]
    fn candidate_selection_prefers_primary_and_falls_back_without_mutation() {
        let dir = std::env::temp_dir().join(format!(
            "geothite-save-pair-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir(&dir).expect("test directory");
        let path = dir.join("slot.crystalsave");
        let identity =
            SaveModpackIdentity::new("test-save-pair", "a".repeat(64)).expect("identity");
        let content = "b".repeat(64);
        write_new_compatible_save(&path, GameState::default(), &identity, &content)
            .expect("primary and backup");
        let validate = |bytes: &[u8]| -> anyhow::Result<GameState> {
            Ok(
                read_save_game_bytes_for_modpack(bytes, "candidate", &identity, &content)?
                    .into_state(),
            )
        };
        assert!(!read_pair(&path, validate).expect("primary selected").backup);
        std::fs::write(&path, b"damaged").expect("damaged primary");
        assert!(read_pair(&path, validate).expect("backup selected").backup);
        assert_eq!(std::fs::read(&path).expect("original retained"), b"damaged");
        let wrong =
            SaveModpackIdentity::new("other-release", "c".repeat(64)).expect("other identity");
        assert!(
            read_pair(&path, |bytes| Ok(read_save_game_bytes_for_modpack(
                bytes,
                "candidate",
                &wrong,
                &content
            )?
            .into_state()))
            .is_none()
        );
        std::fs::remove_dir_all(&dir).expect("remove owned fixture");
    }
}

#[cfg(test)]
#[path = "save_compatibility_history_tests.rs"]
mod history_tests;
