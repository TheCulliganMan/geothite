//! Real native steps and saved ranger ownership on the external preserve.

use std::collections::BTreeSet;

use crystal_core::input::GameButton;
use crystal_core::state::{BattleMemory, GameState};
use crystal_core::systems::map_context::SpawnMemoryUpdate;
use crystal_core::systems::safari::{self, SafariPhase};
use crystal_core::systems::safari_prompt;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;
use crystal_core::world::session::OverworldSession;

use super::{configured, native_press, ranger_fixture};
use crate::safari_zone::collision;
use crate::{GameDataSet, RuntimeMutationCommand, RuntimeSafariCommand, SafariSurface};

fn command(
    data: &GameDataSet,
    state: &mut GameState,
    map: &mut OverworldSession,
    command: RuntimeSafariCommand,
) {
    data.apply_runtime_mutation_command(
        state,
        map,
        RuntimeMutationCommand::Safari(command),
        &BTreeSet::new(),
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .expect("production native mutation");
}

#[test]
#[ignore = "requires CRYSTAL_SAFARI_TEST_PACK with expanded native preserve"]
fn safari_ranger_confirmation_survives_resume_and_blocks_other_gameplay() {
    let data = configured();
    let (mut state, mut map) = ranger_fixture(&data);
    let storage = state.storage.clone();
    let money = state.money;
    command(
        &data,
        &mut state,
        &mut map,
        RuntimeSafariCommand::OpenPrompt,
    );
    command(
        &data,
        &mut state,
        &mut map,
        RuntimeSafariCommand::NavigatePrompt {
            direction: Direction::Down,
        },
    );
    let music = data
        .audio
        .iter()
        .filter(|asset| asset.kind == crate::ModpackAudioKind::Music)
        .map(|asset| asset.id.clone())
        .collect();
    let (mut state, mut map) = data
        .resume_overworld_session_from_state(state, &music)
        .expect("Continue retains real ranger offer");
    assert!(
        !safari_prompt::prompt(&state)
            .expect("saved owner")
            .expect("offer")
            .selected_yes
    );
    let tile = map.player.tile;
    native_press(&data, &mut state, &mut map, GameButton::Down);
    assert_eq!(map.player.tile, tile, "YES/NO owns movement");
    let before = state.clone();
    assert!(
        data.require_safari_input_ownership(&state, &RuntimeMutationCommand::UpdateTime)
            .is_ok(),
        "presentation infrastructure remains available"
    );
    // A direct unrelated gameplay mutation must not bypass the same owner.
    assert!(
        data.require_safari_input_ownership(
            &state,
            &RuntimeMutationCommand::ResolveBlackoutToLastSpawn
        )
        .is_err()
    );
    assert_eq!(state.storage, before.storage);
    command(
        &data,
        &mut state,
        &mut map,
        RuntimeSafariCommand::ConfirmPrompt,
    );
    assert!(safari_prompt::prompt(&state).expect("valid").is_none());
    assert!(safari::session(&state).expect("valid").is_none());
    assert_eq!(state.money, money);
    assert_eq!(state.storage, storage);
    command(
        &data,
        &mut state,
        &mut map,
        RuntimeSafariCommand::OpenPrompt,
    );
    command(
        &data,
        &mut state,
        &mut map,
        RuntimeSafariCommand::ConfirmPrompt,
    );
    assert_eq!(state.money, money - safari::ADMISSION_FEE);
    assert!(safari_prompt::prompt(&state).expect("valid").is_none());
    assert_eq!(
        map.map.name,
        data.safari_zone.as_ref().expect("config").entry.map_name
    );
    let before = state.clone();
    assert!(
        data.apply_safari_command(
            &mut state,
            &mut map,
            RuntimeSafariCommand::ConfirmPrompt,
            &BTreeSet::new()
        )
        .is_err()
    );
    assert_eq!(state, before, "duplicate YES cannot charge or warp again");
}

#[test]
#[ignore = "requires CRYSTAL_SAFARI_TEST_PACK with expanded native preserve"]
fn safari_landmarks_must_exist_and_fit_the_native_caught_field() {
    let data = configured();
    let zone = data.safari_zone.as_ref().expect("config");
    zone.validate(&data).expect("real preserve references");
    let name = zone.maps.keys().next().expect("preserve map");
    let mut missing = data.clone();
    missing.pokegear_landmarks.map_to_landmark.remove(name);
    assert!(
        zone.validate(&missing).is_err(),
        "missing location must not become unknown"
    );
    let constant = &data.pokegear_landmarks.map_to_landmark[name];
    let mut too_large = data.clone();
    too_large
        .pokegear_landmarks
        .landmarks
        .iter_mut()
        .find(|landmark| &landmark.constant == constant)
        .expect("real landmark")
        .id = 128;
    assert!(
        zone.validate(&too_large).is_err(),
        "caught data cannot truncate a landmark into seven bits"
    );
}

#[test]
#[ignore = "requires CRYSTAL_SAFARI_TEST_PACK with expanded native preserve"]
fn safari_native_walking_and_surf_generate_each_habitat_without_combat() {
    let data = configured();
    let zone = data.safari_zone.as_ref().expect("config");
    for habitat in &zone.habitats {
        for surface in [SafariSurface::Grass, SafariSurface::Water] {
            let terrain = match surface {
                SafariSurface::Grass => "TALL_GRASS",
                SafariSurface::Water => "WATER",
            };
            let (name, a, b) = zone
                .maps
                .keys()
                .find_map(|name| {
                    let map = &data.maps[name];
                    (0..i16::try_from(map.attributes.height * 2).expect("height"))
                        .flat_map(|y| {
                            (0..i16::try_from(map.attributes.width * 2).expect("width"))
                                .map(move |x| TilePosition::new(x, y))
                        })
                        .find_map(|a| {
                            let b = TilePosition::new(a.x + 1, a.y);
                            let legal = [a, b].iter().all(|tile| {
                                collision(&data, name, *tile).is_ok_and(|kind| kind == terrain)
                                    && zone.habitat_at(&data, name, *tile).is_ok_and(|found| {
                                        found.is_some_and(|found| found.id == habitat.id)
                                    })
                                    && !map.objects.iter().any(|actor| {
                                        actor.x as i16 == tile.x && actor.y as i16 == tile.y
                                    })
                            });
                            legal.then_some((name.as_str(), a, b))
                        })
                })
                .expect("actual adjacent habitat tiles, no invented geometry");
            let (mut state, mut map) = ranger_fixture(&data);
            data.admit_safari(&mut state, &mut map, true, &BTreeSet::new())
                .expect("real admission");
            // Explicit starting-position/mode fixture; every encounter thereafter
            // must originate in committed native movement, never injected wildlife.
            data.transition_overworld_session(
                &mut state,
                &mut map,
                name,
                a,
                SpawnMemoryUpdate::Preserve,
                &BTreeSet::new(),
            )
            .expect("native habitat fixture");
            map.player.mode = match surface {
                SafariSurface::Grass => MovementMode::Normal,
                SafariSurface::Water => MovementMode::Surf,
            };
            map.player.facing = Direction::Right;
            data.commit_overworld_snapshot(&mut state, &map, SpawnMemoryUpdate::Preserve);
            state.wild_encounter_cooldown = 0;
            let storage = state.storage.clone();
            for _ in 0..160 {
                let button = if map.player.tile == a {
                    GameButton::Right
                } else {
                    GameButton::Left
                };
                native_press(&data, &mut state, &mut map, button);
                if matches!(
                    safari::session(&state)
                        .expect("valid")
                        .expect("permit")
                        .phase,
                    SafariPhase::Encounter(_)
                ) {
                    break;
                }
            }
            let active = safari::session(&state).expect("valid").expect("permit");
            let SafariPhase::Encounter(wild) = active.phase else {
                panic!("no generated {} {surface:?} encounter", habitat.id);
            };
            assert_eq!(wild.habitat_id, habitat.id);
            assert_eq!(wild.origin_map_name, name);
            assert_eq!(wild.origin_tile, map.player.tile);
            let landmark = u8::try_from(
                data.pokegear_landmark_for_map(name)
                    .expect("native Town Map landmark")
                    .id,
            )
            .expect("native landmark byte");
            assert_ne!(
                landmark, 0,
                "the real preserve must not lose its caught area"
            );
            assert_eq!(wild.caught_location, landmark);
            assert!([a, b].contains(&wild.origin_tile));
            assert!(
                habitat
                    .table(surface)
                    .iter()
                    .any(|slot| slot.species == wild.pokemon.species.id
                        && slot.level == wild.pokemon.level)
            );
            assert_eq!(wild.pokemon.hp, wild.pokemon.max_hp);
            assert!(wild.pokemon.status.is_none());
            assert_eq!(state.battle, BattleMemory::Inactive);
            assert!(state.script_runtime.active_battle_combat.is_none());
            assert_eq!(
                state.storage, storage,
                "Safari never sends a player combatant"
            );
            assert_eq!(active.balls_remaining, 30);
            assert!(active.steps_remaining < 500);
            data.validate_saved_safari_references(&state)
                .expect("generated wildlife survives strict pack validation");
        }
    }
}
