//! Isolated save copies made from exact prior and current release archives.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::save;
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::systems::field_moves::{CANOE_ACTIVE_MEMORY, CANOE_ITEM};
use crystal_core::systems::map_context::sync_state_object_overrides;
use crystal_core::world::collision::{permissions, sample_collision};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;
use crystal_core::world::session::object_tile_position_checked;
use serde_json::json;

use crate::{CrystalRuntime, RuntimeGameShell};

const BASIN: &str = "MnInnerBasin";
const STRANDRAK: &str = "MN_BASIN_STRANDRAK";
const OUTLET: &str = "EVENT_MN_BASIN_INNER_OUTLET_ACKNOWLEDGED";

#[test]
fn cave_audit_rejects_unapproved_sources_targets_and_rooms() -> Result<()> {
    let source = "test+northstar-dex-order-v1";
    let target = format!("{source}{}", super::MARKER);
    let mut policy = json!({
        "schema_version": 12,
        "sources": [
            {"modpack_id": source, "modpack_hash": super::PARENT_HASHES[0], "content_hash": super::PARENT_CONTENT[0]},
            {"modpack_id": format!("{source}{}", super::SUFFIX), "modpack_hash": super::PARENT_HASHES[1], "content_hash": super::PARENT_CONTENT[1]},
        ],
        "inherited": {},
        "reentry": {"policy": super::POLICY, "maps": []},
    });
    let check = |policy: &serde_json::Value, target: &str| -> Result<String> {
        Ok(super::parse(&serde_json::to_vec(policy)?, target)
            .expect_err("unapproved release must be rejected")
            .to_string())
    };
    assert!(check(&policy, source)?.contains("invalid cave audit target"));
    assert!(check(&policy, &target)?.contains("incomplete room arrival list"));
    policy["sources"][0]["modpack_hash"] = json!("0".repeat(64));
    assert!(check(&policy, &target)?.contains("expected exact Pokédex ordering sources"));
    Ok(())
}

fn load(path: &Path) -> Result<(AssetRoot, CrystalRuntime)> {
    let assets = AssetRoot::new(path.parent().context("pack directory")?);
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &assets,
        read_loaded_verified_compiled_game_pack(path)?,
    )?;
    Ok((assets, runtime))
}

fn checkpoint(
    parent: &CrystalRuntime,
    mut state: GameState,
    room: &str,
    mode: MovementMode,
) -> Result<GameState> {
    let data = parent.data();
    let module = data.map_module(room)?;
    let native = data.overworld_map(room)?;
    let bank = data.tileset_collision(&module.attributes.tileset_name)?;
    let (width, height) = native.checked_tile_bounds().context("map bounds")?;
    let permission = match mode {
        MovementMode::Surf => permissions::WATER,
        _ => permissions::FLOOR,
    };
    // Choose a real old floor or water tile towards the far corner of the
    // room, with no warp, coordinate event, or actor on the standing tile.
    let mut tile = None;
    'rows: for y in (0..i16::try_from(height)?).rev() {
        for x in (0..i16::try_from(width)?).rev() {
            let at = TilePosition::new(x, y);
            let event_tile = (u16::try_from(x)?, u16::try_from(y)?);
            if sample_collision(&native, &bank, at).is_some_and(|s| s.permission == permission)
                && module
                    .objects
                    .iter()
                    .all(|actor| object_tile_position_checked(actor) != Some(at))
                && module
                    .events
                    .warps
                    .iter()
                    .all(|warp| (warp.x, warp.y) != event_tile)
                && module
                    .events
                    .coord_events
                    .iter()
                    .all(|event| (event.x, event.y) != event_tile)
            {
                tile = Some(at);
                break 'rows;
            }
        }
    }
    let tile = tile.with_context(|| format!("old checkpoint tile in {room}"))?;
    state.overworld = OverworldMemory::Active {
        map_name: room.into(),
        tile,
        facing: Direction::Down,
        mode,
    };
    state.map_object_overrides.clear();
    state.map_block_overrides.clear();
    let mut session = data.overworld_session_for_traversal_with_state(
        room,
        tile,
        state.frame_counter,
        mode.traversal_state(),
        Some(&state),
    )?;
    session.player.facing = Direction::Down;
    session.player.mode = mode;
    session.set_time(state.time.registers.hours, state.time.time_of_day);
    session.sync_event_flag_memory(&state.flags);
    if room == BASIN {
        state.flags.set_event_flag(OUTLET, true)?;
        session.shown_object_identifiers.insert(STRANDRAK.into());
    }
    if room == "MnIronDrillCore" {
        state.flags.set_event_flag("EVENT_MN_DRILL_STOPPED", true)?;
    }
    session.begin_map_object_setup();
    session.finish_map_object_setup()?;
    sync_state_object_overrides(&mut state, &session)?;
    state
        .map_block_overrides
        .entry(room.into())
        .or_default()
        .insert(
            (module.attributes.width - 1, module.attributes.height - 1),
            *module.blocks.last().context("nonempty old map")?,
        );
    if mode == MovementMode::Surf {
        assert!(
            state
                .bag
                .add_item(data.item(CANOE_ITEM)?, 1)
                .map_err(anyhow::Error::msg)?
        );
        state
            .script_runtime
            .memory
            .insert(CANOE_ACTIVE_MEMORY.into(), "1".into());
    }
    parent.validate_save_state_for_runtime_pack(&state)?;
    Ok(state)
}

#[test]
#[ignore = "requires CRYSTAL_CAVE_AUDIT_PARENT and CRYSTAL_CAVE_AUDIT_TARGET external archives"]
fn cave_audit_migrates_all_rebuilt_rooms_and_exact_outside_checkpoints() -> Result<()> {
    let parent_path =
        PathBuf::from(std::env::var_os("CRYSTAL_CAVE_AUDIT_PARENT").context("parent")?);
    let target_path =
        PathBuf::from(std::env::var_os("CRYSTAL_CAVE_AUDIT_TARGET").context("target")?);
    let (parent_assets, parent) = load(&parent_path)?;
    let (target_assets, target) = load(&target_path)?;
    let sources = crate::compatible_save_sources(&target)?;
    let prior = crate::compatible_save_sources(&parent)?;
    ensure!(
        sources.len() == 26 && prior.len() == 24,
        "incomplete migration history"
    );
    assert!(
        sources[..2]
            .iter()
            .any(|s| s.modpack_hash == parent.modpack().hash())
    );
    for (actual, expected) in sources[2..].iter().zip(&prior) {
        assert_eq!(actual.modpack_id, expected.modpack_id);
        assert_eq!(actual.modpack_hash, expected.modpack_hash);
        assert_eq!(actual.content_hash, expected.content_hash);
    }
    let policy = super::parse(
        target
            .runtime_file(crate::SAVE_COMPATIBILITY_FILE)
            .context("compatibility policy")?,
        target.modpack().id(),
    )?
    .cave_audit_reentry
    .context("cave arrival policy")?;
    super::validate_arrivals(&policy, target.data())?;
    let root = std::env::temp_dir().join(format!(
        "cave-audit-save-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos(),
    ));
    std::fs::create_dir(&root)?;
    let result = (|| -> Result<()> {
        let unchanged = "MnStateFairgrounds";
        let mut shell = RuntimeGameShell::new_game_at_runtime_tile(
            parent_assets,
            parent.clone(),
            parent.title_new_game_spawn_identifier()?,
            unchanged,
            36,
            28,
        )?;
        let initial = root.join("initial.crystalsave");
        shell.save(&initial)?;
        let mut base = parent.load_save(&initial)?;
        base.script_runtime = Default::default();
        base.money = 2923;
        base.coins = 291;
        base.flags
            .set_event_flag("EVENT_MN_GYM_2_TRAINER_2_DEFEATED", true)?;
        let pokemon = parent
            .data()
            .create_pokemon("POUCHID", 17, Default::default())?;
        base.pokedex.record_caught_pokemon(&pokemon);
        base.storage.party.add_pokemon(pokemon);
        base.sync_party_from_storage();
        assert!(
            base.bag
                .add_item(parent.data().item("POTION")?, 4)
                .map_err(anyhow::Error::msg)?
        );
        for (index, (room, mode)) in std::iter::once((unchanged, MovementMode::Normal))
            .chain(
                super::ROOMS
                    .into_iter()
                    .map(|room| (room, MovementMode::Normal)),
            )
            .chain(std::iter::once(("MnIronSump", MovementMode::Surf)))
            .enumerate()
        {
            let fixture = if room == unchanged {
                base.clone()
            } else {
                checkpoint(&parent, base.clone(), room, mode)?
            };
            let path = root.join(format!("source-{index}.crystalsave"));
            let backup = save::save_backup_path(&path);
            let destination = root.join(format!("target-{index}.crystalsave"));
            parent.save_game(&path, fixture.clone())?;
            let backup_bytes = std::fs::read(&backup)?;
            if room == BASIN {
                std::fs::write(&path, b"invalid primary")?;
            }
            let original_bytes = std::fs::read(&path)?;
            assert!(
                crate::prepare_runtime_save_slot(&target, &destination, |candidate| {
                    if candidate.modpack_hash == parent.modpack().hash() {
                        path.clone()
                    } else {
                        root.join("absent.crystalsave")
                    }
                })?,
                "failed migrating {room} {mode:?}"
            );
            assert_eq!(std::fs::read(&path)?, original_bytes);
            assert_eq!(std::fs::read(&backup)?, backup_bytes);
            let upgraded = target.load_save(&destination)?;
            if room == unchanged {
                assert_eq!(upgraded, fixture);
                continue;
            }
            let (tile, facing) = policy.arrival(room).context("tested room reentry")?;
            assert_eq!(
                upgraded.overworld.snapshot_identity(),
                Some((room, tile, facing, MovementMode::Normal))
            );
            assert_eq!(upgraded.money, fixture.money);
            assert_eq!(upgraded.coins, fixture.coins);
            assert_eq!(upgraded.pokedex, fixture.pokedex);
            assert_eq!(upgraded.storage, fixture.storage);
            assert_eq!(upgraded.bag, fixture.bag);
            assert_eq!(upgraded.flags, fixture.flags);
            assert_eq!(upgraded.scenes, fixture.scenes);
            assert!(!upgraded.map_block_overrides.contains_key(room));
            assert!(
                !upgraded
                    .script_runtime
                    .memory
                    .contains_key(CANOE_ACTIVE_MEMORY)
            );
            let resumed = RuntimeGameShell::resume_from_save(
                target_assets.clone(),
                target.clone(),
                &destination,
            )?;
            let state = resumed.session().state();
            assert_eq!(
                state.overworld.snapshot_identity(),
                Some((room, tile, facing, MovementMode::Normal))
            );
            if room == BASIN {
                let memory = &state.map_object_overrides[room];
                assert!(state.flags.is_event_flag_set(OUTLET)?);
                assert!(memory.shown_object_identifiers.contains(STRANDRAK));
                assert_eq!(state.map_block_overrides[room].get(&(10, 5)), Some(&18));
            }
            if room == "MnIronDrillCore" {
                let memory = &state.map_object_overrides[room];
                assert!(memory.shown_object_identifiers.contains("MN_NDRILL_FINN"));
                let finn = &memory.objects["MN_NDRILL_FINN"];
                assert_eq!((finn.x, finn.y), (8, 7));
            }
        }
        println!(
            "cave audit verified 19 room arrivals, canoe dismount, and exact outside save copy"
        );
        Ok(())
    })();
    std::fs::remove_dir_all(&root)?;
    result
}
