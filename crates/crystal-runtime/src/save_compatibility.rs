//! Explicit release compatibility and non-destructive slot selection.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_assets::GameDataSet;
use crystal_core::save::{self, SaveModpackIdentity};
use crystal_core::state::{BattleMemory, GameState, OverworldMemory};
use crystal_core::world::session::object_event_starts_fixed_and_sliding;
use serde::Deserialize;

use crate::CrystalRuntime;
use crate::cave_save_migration::{self, CaveReentry};
use crate::gym_save_migration::{self, GymReentry};
use crate::iron_save_migration::{self, IronRelocations};
use crate::room_save_migration::RoomReentry;

#[path = "fair_save_migration.rs"]
mod fair_save_migration;

#[path = "dex_order_save_migration.rs"]
mod dex_order_save_migration;

#[path = "cave_audit_save_migration.rs"]
mod cave_audit_save_migration;

#[path = "road_trainer_save_migration.rs"]
mod road_trainer_save_migration;

pub const SAVE_COMPATIBILITY_FILE: &str = "data/northstar-save-compatibility.json";
pub const SAVE_COMPATIBILITY_MANIFEST: &str = "northstar-refinement-v1";
const STORY_FLOW_MANIFEST: &str = "northstar-story-flow-v1";
const LAKESIDE_MANIFEST: &str = "northstar-lakeside-v1";
const CROSSINGS_MANIFEST: &str = "northstar-crossings-v1";
const WAYFINDING_MANIFEST: &str = "northstar-wayfinding-v1";
const REGIONAL_DESIGN_MANIFEST: &str = "northstar-regional-design-v1";

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
    gym_reentry: Option<GymReentry>,
    cave_reentry: Option<CaveReentry>,
    iron_relocations: Option<IronRelocations>,
    layout_reentry: Option<Vec<String>>,
    #[serde(skip)]
    cave_audit_reentry: Option<RoomReentry>,
    #[serde(skip)]
    fair_layouts: BTreeMap<String, fair_save_migration::MapChange>,
    #[serde(skip)]
    regional_parents: Vec<String>,
    #[serde(skip)]
    unchanged_checkpoint_parents: Vec<String>,
}

/// Returns the exact source identities approved by this target's hashed metadata.
///
/// # Errors
/// Rejects malformed metadata, unknown target markers, and duplicate identities.
pub fn compatible_save_sources(runtime: &CrystalRuntime) -> Result<Vec<CompatibleSaveSource>> {
    let Some(bytes) = runtime.runtime_file(SAVE_COMPATIBILITY_FILE) else {
        return Ok(Vec::new());
    };
    Ok(parse_compatibility(bytes, runtime.modpack().id())?.sources)
}

fn parse_compatibility(bytes: &[u8], target_id: &str) -> Result<Compatibility> {
    #[derive(Deserialize)]
    struct Header {
        schema_version: u8,
    }
    let header: Header = serde_json::from_slice(bytes).context("save compatibility version")?;
    if header.schema_version == 12 {
        return cave_audit_save_migration::parse(bytes, target_id);
    }
    if header.schema_version == 11 {
        return dex_order_save_migration::parse(bytes, target_id);
    }
    if header.schema_version == 10 {
        return fair_save_migration::parse_presentation(bytes, target_id);
    }
    if header.schema_version == 9 {
        return fair_save_migration::parse(bytes, target_id);
    }
    if matches!(header.schema_version, 5..=8) {
        return parse_regional_compatibility(bytes, target_id, header.schema_version);
    }
    parse_prior_compatibility(bytes, target_id)
}

// Keep earlier reentry and relocation rules bound to the release that authored them.
fn parse_prior_compatibility(bytes: &[u8], target_id: &str) -> Result<Compatibility> {
    let metadata: Compatibility =
        serde_json::from_slice(bytes).context("save compatibility metadata")?;
    ensure!(
        matches!(metadata.schema_version, 3 | 4)
            || (metadata.iron_relocations.is_none() && metadata.layout_reentry.is_none()),
        "iron policies require a floorplan-compatible schema"
    );
    match metadata.schema_version {
        1 => ensure!(
            metadata.target_manifest == SAVE_COMPATIBILITY_MANIFEST
                && metadata.gym_reentry.is_none()
                && metadata.cave_reentry.is_none(),
            "unsupported refinement save compatibility policy"
        ),
        2..=4 => {
            ensure!(
                metadata.target_modpack_ids.len() == 2
                    && metadata.target_modpack_ids[0] != metadata.target_modpack_ids[1],
                "room compatibility requires two distinct targets"
            );
            metadata
                .gym_reentry
                .as_ref()
                .context("gym compatibility requires the gym reentry policy")?
                .validate_metadata()?;
            if metadata.schema_version == 2 {
                ensure!(
                    metadata.target_manifest == gym_save_migration::TARGET_MANIFEST
                        && metadata.sources.len() == 6,
                    "gym compatibility requires six exact sources"
                );
                if let Some(policy) = &metadata.cave_reentry {
                    policy.validate_metadata()?;
                }
            } else {
                validate_iron_policy(&metadata)?;
            }
        }
        _ => anyhow::bail!("unsupported save compatibility schema"),
    }
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
        if metadata.schema_version >= 2 {
            ensure!(
                id.split('+').any(|part| part == metadata.target_manifest),
                "gym target identity is missing its release marker"
            );
            if metadata.cave_reentry.is_some() {
                ensure!(
                    id.split('+')
                        .any(|part| part == cave_save_migration::TARGET_MANIFEST),
                    "cave target identity is missing its release marker"
                );
            }
        }
    }
    ensure!(
        !metadata.sources.is_empty()
            && metadata.sources.len() <= 10
            && (metadata.schema_version == 4 || metadata.sources.len() <= 8),
        "bounded compatibility source list required"
    );
    let mut identities = BTreeSet::new();
    let mut source_modpacks = BTreeSet::new();
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
        if metadata.schema_version >= 2 {
            ensure!(
                source_modpacks.insert((&source.modpack_id, &source.modpack_hash)),
                "duplicate gym source pack identity"
            );
        }
    }
    Ok(metadata)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionalCompatibility {
    schema_version: u8,
    target_manifest: String,
    target_modpack_ids: [String; 2],
    sources: [CompatibleSaveSource; 2],
    inherited: serde_json::Value,
}

fn parse_regional_compatibility(
    bytes: &[u8],
    target_id: &str,
    schema: u8,
) -> Result<Compatibility> {
    let (marker, parent_hashes) = match schema {
        5 => (
            REGIONAL_DESIGN_MANIFEST,
            [
                (
                    "02e18ba71203d8aa2e10658c6babc53512daeef592ee53fb2900630f91aa57cb",
                    "79c1c85ebc25b200c9fe30689299af09bc88182828931576e512736395ebd521",
                ),
                (
                    "f4dd4819f039631c81372078d8640f7929114d46bdef90f5ae90d42612a789a0",
                    "9fadbe2b9ecf745d0cc2852e745b12d6fd2634dc694e38ac0f0986e4e38624e3",
                ),
            ],
        ),
        6 => (
            WAYFINDING_MANIFEST,
            [
                (
                    "ea89ae2a3df712b16018c3f7d89f96cfef5486f24778476cff5f7df490af74dd",
                    "9f34e8a1279499804524a383a7c641fcee7a2e11581cce5fc14468c65338d2d9",
                ),
                (
                    "19caa20f4616a72a378f757cbef7052c5bf2a91d079782e267a7b43f6b91a64e",
                    "4076beb45e8a5d5caeabc3e69449f243c3e57de2f6ec42653f418bf3b7051f49",
                ),
            ],
        ),
        7 => (
            CROSSINGS_MANIFEST,
            [
                (
                    "e0638a9327e3296b214cdb8d3fdb6e25c33616cdacd2adac583bdd4eb70958bd",
                    "0495ee73a16a6aa6ceb180a7a349b818d86d982e52f6b41118fd8eb60ad95e7d",
                ),
                (
                    "e17218b24ad2b8292d17658b1176e7e093851f7502b2b5a6177f67edf27d97c4",
                    "123e022fc3a37f46d8f06b7bae53f8b4ad1ffc5cd1d7b9b564b6a287dcbc73ac",
                ),
            ],
        ),
        8 => (
            LAKESIDE_MANIFEST,
            [
                (
                    "caecd60747ee26510651509c6567a3490d919c0d6c9f8c2528d9445de2ad1599",
                    "5ad096a303e84c34353f4ac61cca7e52184001f8df71c9a9de9f0cabfa115aff",
                ),
                (
                    "966e3a8ac71190642addc67a010ffd4e59cb289401fc748ddd36594de4820ae3",
                    "fd716ecef2fcc77ff97ac1f4946356c36fc19d2559b0d019735dfe74e4f15a9b",
                ),
            ],
        ),
        _ => anyhow::bail!("unsupported regional save policy"),
    };
    let release: RegionalCompatibility = serde_json::from_slice(bytes)?;
    let [raw, hosted] = &release.target_modpack_ids;
    let suffix = "+realtime-clock+player-customization";
    ensure!(
        release.schema_version == schema
            && release.target_manifest == marker
            && hosted.strip_prefix(raw) == Some(suffix)
            && (target_id == raw || target_id == hosted),
        "unsupported regional design target"
    );
    let parent = raw
        .strip_suffix(&format!("+{marker}"))
        .context("regional marker must follow its parent release")?;
    ensure!(
        !parent.split('+').any(|id| id == marker),
        "duplicate design marker"
    );
    let parent_ids = [parent.to_owned(), format!("{parent}{suffix}")];
    let inherited = serde_json::to_vec(&release.inherited)?;
    let mut policy = if schema > 5 {
        parse_regional_compatibility(&inherited, parent, schema - 1)?
    } else {
        let story = parse_prior_compatibility(&inherited, parent)?;
        ensure!(
            story.schema_version == 4 && story.target_modpack_ids == parent_ids,
            "regional design requires the released story policy"
        );
        story
    };
    for id in &release.target_modpack_ids {
        SaveModpackIdentity::validate_id(id)?;
    }
    for ((source, id), (pack_hash, content_hash)) in
        release.sources.iter().zip(&parent_ids).zip(parent_hashes)
    {
        ensure!(
            source.modpack_id == *id
                && source.modpack_hash == pack_hash
                && source.content_hash == content_hash,
            "regional revision requires its exact parent pack identities"
        );
    }
    if schema > 5 {
        policy.unchanged_checkpoint_parents.extend(parent_ids);
    } else {
        policy.regional_parents = parent_ids.into();
    }
    policy.sources.splice(0..0, release.sources);
    Ok(policy)
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
    let Some(bytes) = runtime.runtime_file(SAVE_COMPATIBILITY_FILE) else {
        return Ok(false);
    };
    let metadata = parse_compatibility(bytes, runtime.modpack().id())?;
    if let Some(policy) = &metadata.gym_reentry {
        policy.validate_arrivals(runtime.data())?;
    }
    if let Some(policy) = &metadata.cave_reentry {
        policy.validate_arrivals(runtime.data())?;
    }
    if let Some(policy) = &metadata.iron_relocations {
        policy.validate_landings(runtime.data())?;
    }
    if let Some(policy) = &metadata.cave_audit_reentry {
        cave_audit_save_migration::validate_arrivals(policy, runtime.data())?;
    }
    for source in &metadata.sources {
        let path = previous_path(source);
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
            let mut upgraded =
                upgrade_source_checkpoint(saved.into_state(), &metadata, source, runtime.data())?;
            if !dex_order_save_migration::preserves_fair_layout(source)
                && !cave_audit_save_migration::is_direct_parent(source)
            {
                fair_save_migration::reconcile(
                    &mut upgraded,
                    runtime.data(),
                    &metadata.fair_layouts,
                    source,
                )?;
            }
            if let Some(policy) = &metadata.cave_audit_reentry {
                cave_audit_save_migration::reenter(policy, &mut upgraded, runtime.data())?;
            }
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

fn upgrade_source_checkpoint(
    mut state: GameState,
    metadata: &Compatibility,
    source: &CompatibleSaveSource,
    data: &GameDataSet,
) -> Result<GameState> {
    if dex_order_save_migration::preserves_fair_layout(source)
        || cave_audit_save_migration::is_direct_parent(source)
    {
        validate_upgrade_checkpoint(&state, data)?;
        return Ok(state);
    }
    if metadata
        .unchanged_checkpoint_parents
        .contains(&source.modpack_id)
    {
        return normalize_upgrade_checkpoint(state, data);
    }
    if !metadata.regional_parents.is_empty() {
        road_trainer_save_migration::align_undefeated_trainers(data, &mut state)?;
    }
    // Story-only changes preserve the floorplan parent's exact room and actor positions.
    if metadata.regional_parents.contains(&source.modpack_id)
        || direct_story_parent(metadata, &source.modpack_id)
    {
        return normalize_upgrade_checkpoint(state, data);
    }
    let subset = metadata
        .layout_reentry
        .as_deref()
        .filter(|_| direct_iron_parent(metadata, &source.modpack_id));
    let mut upgraded = upgrade_checkpoint(state, metadata, subset, data)?;
    if let Some(policy) = &metadata.iron_relocations {
        policy.apply(data, &mut upgraded)?;
    }
    Ok(upgraded)
}

fn direct_story_parent(metadata: &Compatibility, source_id: &str) -> bool {
    metadata.schema_version == 4
        && metadata.target_modpack_ids.iter().any(|target| {
            target
                .split('+')
                .filter(|&id| id != STORY_FLOW_MANIFEST)
                .eq(source_id.split('+'))
        })
}

fn direct_iron_parent(metadata: &Compatibility, source_id: &str) -> bool {
    metadata.target_modpack_ids.iter().any(|target| {
        target
            .split('+')
            .filter(|&part| {
                part != iron_save_migration::TARGET_MANIFEST
                    && (metadata.schema_version != 4 || part != STORY_FLOW_MANIFEST)
            })
            .eq(source_id.split('+'))
    })
}

fn validate_iron_policy(metadata: &Compatibility) -> Result<()> {
    let story_release = metadata.schema_version == 4;
    let (marker, count) = if story_release {
        (STORY_FLOW_MANIFEST, 10)
    } else {
        (iron_save_migration::TARGET_MANIFEST, 8)
    };
    ensure!(
        metadata.target_manifest == marker && metadata.sources.len() == count,
        "wrong save compatibility marker or number of source identities"
    );

    metadata
        .cave_reentry
        .as_ref()
        .context("missing iron target's cave policy")?
        .validate_all_eight()?;
    metadata
        .iron_relocations
        .as_ref()
        .context("missing iron relocations")?
        .validate_metadata()?;
    let layouts = metadata
        .layout_reentry
        .as_ref()
        .context("missing revised layouts")?;
    let rooms: BTreeSet<_> = layouts.iter().map(String::as_str).collect();
    ensure!(
        rooms.len() == layouts.len()
            && rooms
                == BTreeSet::from([
                    "MnRochesterGym",
                    "MnBrainerdGym",
                    "MnStPaulGym",
                    "MnVoyageursIce1F",
                    "MnVoyageursIceB1F",
                    "MnR17Cave1F",
                ]),
        "iron layout reentry must name each redesigned floor exactly once"
    );
    let raw = metadata.target_modpack_ids[0].as_str();
    ensure!(
        metadata.target_modpack_ids[1].strip_prefix(raw)
            == Some("+realtime-clock+player-customization"),
        "iron release requires its exact raw and hosted target pair"
    );
    for target in &metadata.target_modpack_ids {
        ensure!(
            target
                .split('+')
                .filter(|&id| id == iron_save_migration::TARGET_MANIFEST)
                .count()
                == 1
                && target
                    .split('+')
                    .any(|id| id == gym_save_migration::TARGET_MANIFEST)
                && target
                    .split('+')
                    .any(|id| id == cave_save_migration::TARGET_MANIFEST),
            "iron target is missing its unique revision or prior room markers"
        );
        if story_release {
            let base = target
                .strip_suffix("+realtime-clock+player-customization")
                .unwrap_or(target);
            ensure!(
                base.ends_with(&format!("+{STORY_FLOW_MANIFEST}"))
                    && target
                        .split('+')
                        .filter(|&id| id == STORY_FLOW_MANIFEST)
                        .count()
                        == 1,
                "story marker must follow the floorplan release"
            );
            let floorplan = target.split('+').filter(|&id| id != STORY_FLOW_MANIFEST);
            ensure!(
                metadata
                    .sources
                    .iter()
                    .filter(|s| floorplan.clone().eq(s.modpack_id.split('+')))
                    .count()
                    == 1,
                "story target requires its floorplan parent"
            );
        }
        let parent = target.split('+').filter(|&id| {
            id != iron_save_migration::TARGET_MANIFEST
                && (!story_release || id != STORY_FLOW_MANIFEST)
        });
        ensure!(
            metadata
                .sources
                .iter()
                .filter(|s| parent.clone().eq(s.modpack_id.split('+')))
                .count()
                == 1,
            "iron target requires its exact immediate parent"
        );
    }
    for source in &metadata.sources {
        if !direct_iron_parent(metadata, &source.modpack_id)
            && !direct_story_parent(metadata, &source.modpack_id)
        {
            ensure!(
                !source.modpack_id.split('+').any(|id| matches!(
                    id,
                    STORY_FLOW_MANIFEST
                        | iron_save_migration::TARGET_MANIFEST
                        | gym_save_migration::TARGET_MANIFEST
                        | cave_save_migration::TARGET_MANIFEST
                )),
                "older iron sources must predate all three room revisions"
            );
        }
    }
    Ok(())
}

fn upgrade_checkpoint(
    state: GameState,
    metadata: &Compatibility,
    subset: Option<&[String]>,
    data: &GameDataSet,
) -> Result<GameState> {
    let mut upgraded = normalize_upgrade_checkpoint(state, data)?;
    if let Some(policy) = &metadata.gym_reentry {
        policy.apply(&mut upgraded, subset)?;
    }
    if let Some(policy) = &metadata.cave_reentry {
        policy.apply(&mut upgraded, subset)?;
    }
    Ok(upgraded)
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
fn normalize_upgrade_checkpoint(mut state: GameState, data: &GameDataSet) -> Result<GameState> {
    validate_upgrade_checkpoint(&state, data)?;
    state.script_runtime.script_ended = None;
    state.script_runtime.text_events.clear();
    state.script_runtime.active_text_label = None;
    Ok(state)
}

fn validate_upgrade_checkpoint(state: &GameState, data: &GameDataSet) -> Result<()> {
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
    for (map_name, memory) in &state.map_object_overrides {
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
        for actor in &memory.object_structs.structs {
            ensure!(
                actor.last_tile_occupied_remaining_frames == 0
                    && actor.strength_push_direction.is_none()
                    && !actor.strength_moving
                    && !actor.normal_following
                    && actor.following_not_exact_leader_slot.is_none(),
                "finish actor movement before upgrading"
            );
            if actor.sliding {
                // Some stationary objects carry both sprite flags from the moment they spawn.
                let map = data.map_module(map_name)?;
                let event = actor
                    .map_object_index
                    .checked_sub(1)
                    .and_then(|index| map.objects.get(usize::from(index)))
                    .context("saved sliding actor is missing its map event")?;
                ensure!(
                    actor.fixed_facing
                        && actor.step_duration.is_none()
                        && object_event_starts_fixed_and_sliding(&event.spritemovedata),
                    "finish actor movement before upgrading"
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_compatibility, read_pair, validate_upgrade_checkpoint};
    use crystal_assets::GameDataSet;
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
            parse_compatibility(&bytes, "core+northstar-refinement-v1")
                .expect("authorized destination")
                .sources
                .len(),
            1
        );
        assert!(parse_compatibility(&bytes, "core+northstar-refinement-v1+next-release").is_err());
        let mut invalid = metadata.clone();
        invalid["sources"][0]["modpack_hash"] = "wrong".into();
        assert!(
            parse_compatibility(
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
        assert!(validate_upgrade_checkpoint(&state, &GameDataSet::default()).is_ok());
        state.script_runtime.text_window_open = true;
        assert!(validate_upgrade_checkpoint(&state, &GameDataSet::default()).is_err());
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
        assert!(validate_upgrade_checkpoint(&state, &GameDataSet::default()).is_ok());
        state
            .map_object_overrides
            .get_mut("MnWinona")
            .expect("map")
            .object_structs
            .player_last_tile_occupied_remaining_frames = 1;
        assert!(validate_upgrade_checkpoint(&state, &GameDataSet::default()).is_err());
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

#[cfg(test)]
#[path = "gym_save_migration_tests.rs"]
mod gym_tests;

#[cfg(test)]
#[path = "cave_save_migration_tests.rs"]
mod cave_tests;

#[cfg(test)]
#[path = "iron_save_migration_tests.rs"]
mod iron_tests;
