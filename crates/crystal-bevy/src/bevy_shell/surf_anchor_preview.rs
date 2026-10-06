/// Disposable developer fixture only. Leave the trainer on original land so
/// ordinary A/Yes, UsedSurfScript and its sixteen-frame slow_step own mounting.
/// No wild battle, wild species/level, RNG or divider samples are supplied here.
#[cfg(any(test, feature = "location-tester"))]
fn prepare_surf_encounter_preview(mut shell: BevyRuntimeShell) -> Result<BevyRuntimeShell> {
    use crate::core::world::collision::{permissions, sample_collision};
    anyhow::ensure!(
        shell.quick_save_path.is_none(),
        "Surf preview cannot write a user save"
    );
    complete_visible_smoke_player_name_if_needed(&mut shell, Some("CHRIS"))?;
    let initial = shell.shell.snapshot()?;
    anyhow::ensure!(
        initial.party.slots.is_empty() && initial.battle.is_none(),
        "Surf preview requires a fresh empty-party field session"
    );
    let runtime = shell.shell.runtime().clone();
    let data = runtime.data();
    let map_name = "Route44";
    let map = data
        .overworld_map(map_name)
        .context("compiled Surf encounter map")?;
    let tileset = data.tileset_collision(data.map_tileset_name(map_name)?)?;
    anyhow::ensure!(
        map.checked_tile_bounds() == Some((60, 18)),
        "Surf preview requires its compiled Route44 extent"
    );
    let module = &data.maps[map_name];
    anyhow::ensure!(
        module.events.coord_events.is_empty(),
        "Surf preview path must not dispatch a coordinate event"
    );
    let from = TilePosition::new(38, 3);
    anyhow::ensure!(
        sample_collision(&map, &tileset, from)
            .is_some_and(|sample| sample.permission == permissions::FLOOR),
        "Surf preview must start on its original shore floor"
    );
    // This rectangle contains the entry, ordinary approach, alternating
    // (37,6)<->(38,6) source steps, both derived targets and their body margins.
    for y in 4..=7 {
        for x in 34..=41 {
            let tile = TilePosition::new(x, y);
            anyhow::ensure!(
                sample_collision(&map, &tileset, tile)
                    .is_some_and(|sample| surf_plain_support(sample.permission)),
                "Surf preview requires original ordinary WATER at ({x},{y})"
            );
            anyhow::ensure!(
                !module
                    .objects
                    .iter()
                    .any(|object| object_tile_position_checked(object) == Some(tile))
                    && !module
                        .events
                        .warps
                        .iter()
                        .any(|warp| warp_tile_position_checked(warp) == Some(tile)),
                "Surf preview water must not contain an authored object or warp"
            );
        }
    }
    for tile in [
        from,
        TilePosition::new(38, 4),
        TilePosition::new(38, 5),
        TilePosition::new(38, 6),
        TilePosition::new(37, 6),
    ] {
        anyhow::ensure!(
            !module.objects.iter().any(|object| {
                object_tile_position_checked(object).is_some_and(|other| {
                    let dx = (i32::from(other.x) - i32::from(tile.x)).abs();
                    let dy = (i32::from(other.y) - i32::from(tile.y)).abs();
                    other == tile
                        || (object.object_type == "OBJECTTYPE_TRAINER"
                            && (dx == 0 || dy == 0)
                            && dx + dy <= i32::from(object.radius))
                })
            }),
            "Surf preview path overlaps an authored actor or trainer sight"
        );
    }
    let encounters = data
        .wild_encounters
        .get(map_name)
        .context("Surf preview encounter metadata")?;
    let water = encounters
        .water
        .as_ref()
        .context("Surf preview water encounter table")?;
    anyhow::ensure!(
        encounters.water_rate == Some(2)
            && [&water.morning, &water.day, &water.night]
                .into_iter()
                .all(|slots| slots
                    .iter()
                    .map(|slot| (slot.species.as_str(), slot.level))
                    .eq([("POLIWAG", 25), ("POLIWAG", 20), ("REMORAID", 20)])),
        "Surf preview requires the unchanged source water slots and rate"
    );
    shell.shell.add_party_pokemon(
        "TOTODILE",
        20,
        None,
        None,
        &initial.trainer.player_name,
        initial.trainer.player_id,
        Dv::from_non_hp(9, 9, 9, 9),
    )?;
    shell.shell.add_bag_item("HM_SURF", 1)?;
    shell
        .shell
        .use_bag_tmhm_on_party_pokemon("HM_SURF", 0, Some(0))?;
    shell
        .shell
        .award_badge(RuntimeBadgeRegion::Johto, 3)?;
    {
        let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
        data.transition_overworld_session(
            state,
            overworld,
            map_name,
            from,
            crate::core::systems::map_context::SpawnMemoryUpdate::Preserve,
            &runtime.music_ids(),
        )?;
        overworld.set_player_facing(Direction::Down);
        state.overworld = crate::core::state::OverworldMemory::from_snapshot(&overworld.snapshot());
    }
    reset_visible_navigation_state(&mut shell);
    mark_runtime_snapshot_dirty(&mut shell);
    settle_visible_shell_smoke_until_idle(&mut shell)?;
    let ready = shell.shell.snapshot()?;
    anyhow::ensure!(
        ready.battle.is_none()
            && ready.overworld.map_name == map_name
            && ready.overworld.tile == from
            && ready.overworld.facing == Direction::Down
            && ready.overworld.mode == MovementMode::Normal
            && ready.party.slots[0].pokemon.moves[0].name == "SURF",
        "Surf fixture must await A/Yes on its checked source shore"
    );
    let overworld = shell.shell.session().overworld();
    let occupied = overworld.occupied_tiles_checked()?;
    anyhow::ensure!(
        [
            from,
            TilePosition::new(38, 4),
            TilePosition::new(38, 5),
            TilePosition::new(38, 6),
            TilePosition::new(37, 6)
        ]
        .into_iter()
        .all(|tile| !occupied.iter().any(|entry| entry.tile == tile)),
        "Surf preview approach is occupied in the actual runtime"
    );
    mark_runtime_snapshot_dirty(&mut shell);
    Ok(shell)
}
