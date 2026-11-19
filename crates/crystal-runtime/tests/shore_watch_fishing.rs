//! The signed piers use the same rod, battle, capture, and save paths as the rest of the region.
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_assets::AssetRoot;
use crystal_core::random::{CrystalRandomState, ReplayDivider};
use crystal_core::save::erase_save_game;
use crystal_core::state::{BattleMemory, GameState};
use crystal_core::world::encounters::TimeOfDay;
use crystal_core::world::fishing::roll_fishing_encounter;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_runtime::{CrystalRuntime, RuntimeWildEncounterOriginKey};

const MAP: &str = "MnR18C00R00";

// Native Random subtracts the second divider sample from its accumulator.
// Request an exact bite/slot, then supply varying bytes for battle generation.
fn cast_trace(bite: u8, slot: u8) -> ReplayDivider {
    let prefix = [0, 0_u8.wrapping_sub(bite), 0, bite.wrapping_sub(slot)];
    ReplayDivider::new(prefix.into_iter().chain((0..=255).cycle().take(2048)))
}

fn runtime_from_archive() -> Result<CrystalRuntime> {
    let file = std::env::var_os("MN_SHORE_WATCH_PACK").context("MN_SHORE_WATCH_PACK")?;
    let root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(Path::new(&file))?;
    CrystalRuntime::from_loaded_compiled_pack(&root, loaded)
}

#[test]
#[ignore = "requires an external Minnesota archive in MN_SHORE_WATCH_PACK"]
fn signed_piers_accept_the_good_rod_and_resume_a_lake_swarm_capture() -> Result<()> {
    let runtime = runtime_from_archive()?;
    let data = runtime.data();
    let music = data.audio_ids().into_iter().map(str::to_owned).collect();
    let (mut state, mut pier) =
        data.start_overworld_session_at_runtime_tile(MAP, TilePosition::new(29, 23), &music)?;
    state.player_name = "CHRIS".into();
    state.random_state = CrystalRandomState { add: 0, sub: 0 };
    ensure!(state.storage.party.add_pokemon(data.create_pokemon(
        "POUCHID",
        5,
        Default::default()
    )?));
    state.sync_party_from_storage();
    let rod = data.item("GOOD_ROD")?;
    let lure = data.item("LURE_BALL")?;
    ensure!(
        state.bag.add_item(rod, 1).map_err(anyhow::Error::msg)?
            && state.bag.add_item(lure, 3).map_err(anyhow::Error::msg)?
    );
    pier.player.facing = Direction::Up;
    let dry = state.clone();
    let error = data
        .use_bag_fishing_rod_in_field_with_divider(
            &mut state,
            &pier,
            "GOOD_ROD",
            &mut cast_trace(0, 0),
        )
        .expect_err("inland side of the pier");
    ensure!(
        error.downcast_ref::<crystal_core::world::fishing::FishingError>()
            == Some(&crystal_core::world::fishing::FishingError::FacingTileIsNotWater)
    );
    ensure!(state == dry);

    let mut lower = data.overworld_session(MAP, TilePosition::new(19, 35), 0)?;
    lower.player.facing = Direction::Down;
    let mut ordinary = state.clone();
    ordinary.time.registers.hours = 12;
    ordinary.time.time_of_day = TimeOfDay::Day;
    let empty = data.use_bag_fishing_rod_in_field_with_divider(
        &mut ordinary,
        &lower,
        "GOOD_ROD",
        &mut cast_trace(254, 0),
    )?;
    ensure!(
        empty.cast.checked_water_target == TilePosition::new(19, 36)
            && empty.cast.bite == Some(false)
    );
    ensure!(ordinary.bag.quantity(rod) == 1 && matches!(ordinary.battle, BattleMemory::Inactive));
    ordinary.random_state = CrystalRandomState { add: 0, sub: 0 };
    let day = data.use_bag_fishing_rod_in_field_with_divider(
        &mut ordinary,
        &lower,
        "GOOD_ROD",
        &mut cast_trace(0, 254),
    )?;
    let common = day
        .cast
        .session
        .outcome
        .encounter
        .context("daytime lake fish")?;
    ensure!(common.species == "SCUTLET" && common.level == 20);

    state.time.time_of_day = TimeOfDay::Night;
    state.fishing.daily_flags1 = 1 << 2;
    state.fishing.swarm_flag = 1;
    pier.player.facing = Direction::Down;
    let rare = data.use_bag_fishing_rod_in_field_with_divider(
        &mut state,
        &pier,
        "GOOD_ROD",
        &mut cast_trace(0, 254),
    )?;
    ensure!(rare.cast.checked_water_target == TilePosition::new(29, 24));
    ensure!(rare.cast.session.group != day.cast.session.group);
    let fish = rare
        .cast
        .wild_battle
        .context("swarm fishing battle")?
        .enemy_pokemon;
    ensure!(fish.species.id == "TAPDUCK" && fish.level == 20);

    let save = std::env::temp_dir().join(format!(
        "shore-watch-fishing-{}.crystalsave",
        std::process::id()
    ));
    runtime.save_game(&save, state)?;
    let mut restored = runtime.load_save(&save)?;
    ensure!(restored.bag.quantity(rod) == 1 && restored.bag.quantity(lure) == 3);
    let keys = runtime.wild_encounter_origin_keys();
    for map in [MAP, "MnR18C01R00", "MnMilleLacs"] {
        ensure!(data.map_fishing_encounter_has(map, "TAPDUCK", 20));
        ensure!(keys.contains(&RuntimeWildEncounterOriginKey {
            map_name: map.into(),
            species: "TAPDUCK".into(),
            level: 20
        }));
    }
    ensure!(!data.map_fishing_encounter_has("MnR01C00R00", "TAPDUCK", 20));
    ensure!(!data.map_fishing_encounter_has(MAP, "TAPDUCK", 19));
    let mut throw = ReplayDivider::new([0; 256]);
    restored.random_state = CrystalRandomState { add: 0, sub: 0 };
    let outcome =
        data.throw_ball_at_active_battle_with_divider(&mut restored, "LURE_BALL", &mut throw)?;
    ensure!(outcome.caught && !outcome.blocked);
    data.complete_active_wild_capture(&mut restored, &outcome, None, &mut cast_trace(0, 0))?;
    runtime.save_game(&save, restored)?;
    let caught = runtime.load_save(&save)?;
    erase_save_game(&save)?;
    ensure!(matches!(caught.battle, BattleMemory::Inactive));
    ensure!(
        caught
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .filter(|p| p.species.id == "TAPDUCK" && p.level == 20)
            .count()
            == 1
    );
    ensure!(caught.bag.quantity(lure) == 2 && caught.bag.quantity(rod) == 1);
    Ok(())
}

#[test]
#[ignore = "requires an external Minnesota archive in MN_SHORE_WATCH_PACK"]
fn every_native_swarm_slot_has_a_valid_map_species_and_level() -> Result<()> {
    let runtime = runtime_from_archive()?;
    let data = runtime.data();
    let catalog = &data.fishing;
    let keys = runtime.wild_encounter_origin_keys();
    let mut regional_maps = std::collections::BTreeSet::new();
    let mut slots = 0;
    for map in data.map_ids() {
        let Some(base) = data.map_fishing_group(&map)? else {
            continue;
        };
        for rule in catalog
            .swarm_rules
            .values()
            .filter(|rule| rule.base_group == base)
        {
            let mut state = GameState::reset_wram_for_new_game();
            for expected in [base, rule.swarm_group.as_str()] {
                if expected != base {
                    state.fishing.daily_flags1 = 1 << rule.daily_flag_bit;
                    state.fishing.swarm_flag = rule.swarm;
                }
                for (rod, table) in &catalog.groups[expected].rod_tables {
                    for row in &table.slots {
                        for time in [TimeOfDay::Morning, TimeOfDay::Day, TimeOfDay::Night] {
                            let hook = roll_fishing_encounter(
                                &state,
                                catalog,
                                Some(base),
                                rod,
                                time,
                                0,
                                row.threshold,
                            )?;
                            ensure!(hook.group.as_deref() == Some(expected));
                            let fish = hook
                                .encounter
                                .context("authored fishing slot has no encounter")?;
                            ensure!(
                                data.map_fishing_encounter_has(&map, &fish.species, fish.level),
                                "{map} {expected} {rod} {time:?} omitted {} at level {}",
                                fish.species,
                                fish.level
                            );
                            ensure!(keys.contains(&RuntimeWildEncounterOriginKey {
                                map_name: map.clone(),
                                species: fish.species,
                                level: fish.level,
                            }));
                            slots += 1;
                        }
                    }
                }
            }
            if map.starts_with("Mn") {
                regional_maps.insert(map.clone());
            }
        }
    }
    ensure!(regional_maps.contains(MAP) && regional_maps.contains("MnMilleLacs"));
    println!(
        "{} Minnesota swarm maps; {slots} base/swarm rod and time combinations including Johto and Kanto",
        regional_maps.len()
    );
    Ok(())
}
