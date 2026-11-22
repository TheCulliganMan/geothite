//! Exercise exact archives and the production copy-on-upgrade entry point without player input.
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::save;
use crystal_core::state::OverworldMemory;
use crystal_core::world::collision::{permissions, sample_collision};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;

use crate::{CrystalRuntime, RuntimeGameShell};

fn runtime(path: &Path) -> Result<CrystalRuntime> {
    CrystalRuntime::from_loaded_compiled_pack(
        &AssetRoot::new(path.parent().context("pack directory")?),
        read_loaded_verified_compiled_game_pack(path)?,
    )
}

#[test]
#[ignore = "requires CRYSTAL_FAIR_PARENT and CRYSTAL_FAIR_TARGET verified external archives"]
fn fair_release_copies_checkpoints_across_exact_archives() -> Result<()> {
    let source_path = PathBuf::from(std::env::var_os("CRYSTAL_FAIR_PARENT").context("source")?);
    let target_path = PathBuf::from(std::env::var_os("CRYSTAL_FAIR_TARGET").context("target")?);
    let parent = runtime(&source_path)?;
    let target = runtime(&target_path)?;
    let list = crate::compatible_save_sources(&target)?;
    ensure!(
        list.len() == 20
            && list[..2]
                .iter()
                .any(|s| s.modpack_hash == parent.modpack().hash()),
        "exact release sources"
    );
    let tmp = std::env::temp_dir().join(format!(
        "fair-save-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir(&tmp)?;
    let result = (|| -> Result<()> {
        let mut shell = RuntimeGameShell::new_game_at_runtime_tile(
            AssetRoot::new(source_path.parent().context("source pack directory")?),
            parent.clone(),
            parent.title_new_game_spawn_identifier()?,
            "MnR10C00R00",
            24,
            20,
        )?;
        let initial = tmp.join("initial.crystalsave");
        shell.save(&initial)?;
        let mut base = parent.load_save(&initial)?;
        base.money = 29870;
        base.coins = 672;
        base.flags
            .set_event_flag("EVENT_MN_GYM_2_TRAINER_2_DEFEATED", true)?;
        base.storage.party.add_pokemon(parent.data().create_pokemon(
            "POUCHID",
            17,
            Default::default(),
        )?);
        base.sync_party_from_storage();
        ensure!(
            base.bag
                .add_item(parent.data().item("POTION")?, 4)
                .map_err(anyhow::Error::msg)?
        );
        base.map_block_overrides
            .entry("MnMankatoGym".into())
            .or_default()
            .extend([((6, 3), 17), ((1, 7), 18)]);
        let absent = tmp.join("absent.crystalsave");
        use ExpectedLanding::{At, Declined, Relocated, WithPierWatcher};
        use MovementMode::{Normal, Surf};
        for (i, (map, x, y, mode, expected)) in [
            ("MnR10C00R00", 24, 20, Normal, At(24, 20)),
            ("MnR15C00R00", 22, 8, Normal, Relocated),
            ("MnR07C00R00", 28, 20, Surf, Relocated),
            ("MnIronSump", 9, 10, Surf, At(9, 10)),
            ("MnStateFairgrounds", 36, 28, Normal, At(36, 28)),
            // Saving after disembarking on the old island records walking mode.
            ("MnR12Cave1F", 28, 16, Normal, At(21, 16)),
            ("MnR12Cave1F", 29, 16, Normal, At(21, 16)),
            ("MnR12Cave1F", 28, 17, Normal, At(21, 16)),
            ("MnR12Cave1F", 29, 17, Normal, At(21, 16)),
            ("MnR15C00R00", 49, 18, Normal, At(47, 18)),
            ("MnR15C00R00", 49, 18, Normal, WithPierWatcher),
            ("MnR13C00R02", 42, 18, Normal, Declined),
        ]
        .into_iter()
        .enumerate()
        {
            let mut fixture = base.clone();
            let OverworldMemory::Active {
                map_name,
                tile,
                mode: m,
                facing,
            } = &mut fixture.overworld
            else {
                anyhow::bail!("uninitialized fixture")
            };
            *map_name = map.into();
            *tile = TilePosition::new(x, y);
            *m = mode;
            *facing = Direction::Right;
            let beyond_cut_tree = (map, x, y) == ("MnR15C00R00", 49, 18);
            if beyond_cut_tree {
                // Cut opens the west side of this two-square block in Lakeside.
                fixture
                    .map_block_overrides
                    .entry(map.into())
                    .or_default()
                    .insert((24, 9), 0x6f);
            }
            let mut old_room = parent.data().overworld_session_for_traversal_with_state(
                map,
                TilePosition::new(x, y),
                fixture.frame_counter,
                mode.traversal_state(),
                Some(&fixture),
            )?;
            if beyond_cut_tree {
                for x in [48, 49] {
                    let ground = sample_collision(
                        &old_room.map,
                        &old_room.tileset,
                        TilePosition::new(x, 18),
                    )
                    .context("old Cut passage")?;
                    assert_eq!(ground.permission, permissions::FLOOR);
                }
            }
            let pier_watcher = "MN_R15_PIER_WATCHER";
            let bank = TilePosition::new(47, 18);
            if matches!(expected, WithPierWatcher) {
                // Exercise a saved actor pose on the preferred landing.
                old_room
                    .object_runtime_tiles
                    .insert(pier_watcher.into(), bank);
                assert!(old_room.occupied_tiles_checked()?.iter().any(|actor| {
                    actor.tile == bank && actor.object_identifier.as_deref() == Some(pier_watcher)
                }));
            }
            fixture.map_object_overrides.clear();
            fixture
                .map_object_overrides
                .entry(map.into())
                .or_default()
                .object_structs = old_room.object_struct_roster_memory()?;

            if mode == MovementMode::Surf {
                fixture
                    .script_runtime
                    .memory
                    .insert("geothiteCanoeActive".into(), "1".into());
            }
            parent
                .validate_save_state_for_runtime_pack(&fixture)
                .with_context(|| format!("valid parent checkpoint {map} ({x},{y})"))?;
            let source = tmp.join(format!("source-{i}.crystalsave"));
            let dest = tmp.join(format!("dest-{i}.crystalsave"));
            save::write_save_game_for_modpack(
                &source,
                fixture.clone(),
                parent.modpack(),
                &parent.pack_identity().content_hash,
            )?;
            let before = std::fs::read(&source)?;
            let backup = save::save_backup_path(&source);
            let before_backup = std::fs::read(&backup)?;
            let migrated = crate::prepare_runtime_save_slot(&target, &dest, |s| {
                if s.modpack_hash == parent.modpack().hash() {
                    source.clone()
                } else {
                    absent.clone()
                }
            })?;
            assert_eq!(
                std::fs::read(&source)?,
                before,
                "source save for {map} ({x},{y})"
            );
            assert_eq!(
                std::fs::read(&backup)?,
                before_backup,
                "source backup for {map} ({x},{y})"
            );
            if matches!(expected, Declined) {
                assert!(!migrated, "sealed source corridor on {map} ({x},{y})");
                assert!(!dest.exists());
                assert!(!save::save_backup_path(&dest).exists());
                continue;
            }
            if !migrated {
                let policy = super::parse(
                    target
                        .runtime_file(crate::SAVE_COMPATIBILITY_FILE)
                        .context("policy")?,
                    target.modpack().id(),
                )?;
                let src = list
                    .iter()
                    .find(|s| s.modpack_hash == parent.modpack().hash())
                    .context("source")?;
                let mut state = super::super::upgrade_source_checkpoint(
                    fixture.clone(),
                    &policy,
                    src,
                    target.data(),
                )?;
                super::reconcile(&mut state, target.data(), &policy.fair_layouts, src)?;
                target.validate_save_state_for_runtime_pack(&state)?;
                anyhow::bail!("copy failed for {map} ({x},{y})");
            }
            let saved = target.load_save(&dest)?;
            let OverworldMemory::Active {
                map_name,
                tile,
                mode: new_mode,
                facing,
            } = &saved.overworld
            else {
                anyhow::bail!("lost overworld")
            };
            assert_eq!(map_name, map);
            assert_eq!(*new_mode, mode);
            assert_eq!(*facing, Direction::Right);
            match expected {
                At(dx, dy) => assert_eq!(*tile, TilePosition::new(dx, dy), "{map} ({x},{y})"),
                Relocated => assert_ne!(*tile, TilePosition::new(x, y), "{map}"),
                WithPierWatcher => {
                    assert_eq!(*tile, TilePosition::new(46, 18));
                    let mut room = target.data().overworld_session_for_traversal_with_state(
                        map,
                        *tile,
                        saved.frame_counter,
                        mode.traversal_state(),
                        Some(&saved),
                    )?;
                    target
                        .data()
                        .apply_saved_overworld_overrides(&mut room, &saved)?;
                    let occupied = room.occupied_tiles_checked()?;
                    assert!(occupied.iter().any(|actor| actor.tile == bank
                        && actor.object_identifier.as_deref() == Some(pier_watcher)));
                    assert!(occupied.iter().all(|actor| actor.tile != *tile));
                }
                Declined => unreachable!("checked before loading the destination save"),
            }
            if beyond_cut_tree {
                assert!(
                    saved
                        .map_block_overrides
                        .get(map)
                        .and_then(|b| b.get(&(24, 9)))
                        .is_none(),
                    "the removed Route 15 Cut tree should use the new block"
                );
            }
            if mode == MovementMode::Surf {
                let room = target.data().overworld_session_for_traversal_with_state(
                    map,
                    *tile,
                    0,
                    mode.traversal_state(),
                    Some(&saved),
                )?;
                let surface = crystal_core::world::collision::sample_collision(
                    &room.map,
                    &room.tileset,
                    *tile,
                )
                .context("water")?;
                assert!(super::on_traversal_layer(
                    surface.permission,
                    mode.traversal_state()
                ));
            }
            assert_eq!(
                saved.map_block_overrides["MnMankatoGym"].get(&(6, 3)),
                Some(&16)
            );
            assert_eq!(
                saved.map_block_overrides["MnMankatoGym"].get(&(1, 7)),
                Some(&18)
            );
            assert_eq!((saved.money, saved.coins), (fixture.money, fixture.coins));
            assert_eq!(saved.flags, fixture.flags);
            assert_eq!(saved.storage, fixture.storage);
            assert_eq!(saved.bag, fixture.bag);
            assert_eq!(saved.script_runtime.memory, fixture.script_runtime.memory);
            assert!(crate::prepare_runtime_save_slot(&target, &dest, |_| {
                absent.clone()
            })?);
            let wrong = tmp.join(format!("wrong-{i}.crystalsave"));
            assert!(!crate::prepare_runtime_save_slot(&target, &wrong, |s| {
                if s.modpack_hash != parent.modpack().hash() {
                    source.clone()
                } else {
                    absent.clone()
                }
            })?);
            assert!(!wrong.exists());
        }
        Ok(())
    })();
    std::fs::remove_dir_all(tmp)?;
    result
}

enum ExpectedLanding {
    At(i16, i16),
    Relocated,
    WithPierWatcher,
    Declined,
}
