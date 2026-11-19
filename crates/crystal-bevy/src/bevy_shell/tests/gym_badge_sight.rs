//! Start one step outside each undefeated teacher's actual lane, under the
//! puzzle state reached from the public door in the Badge Trails geometry audit.
use crystal_core::input::GameButton::{self, Left, Right, Up};
use crystal_core::models::Dv;
use crystal_core::state::{BattleMemory, OverworldMemory};
use crystal_core::world::map::{Direction, TilePosition};

use super::super::{
    mark_runtime_snapshot_dirty, render_visible_script_text_body, visible_rival_name,
};
use super::badge_trails::load;
use super::interior_life::fixture;

struct Approach {
    from: (i16, i16),
    step: GameButton,
    flags: &'static [&'static str],
}

const fn approach(from: (i16, i16), step: GameButton, flags: &'static [&'static str]) -> Approach {
    Approach { from, step, flags }
}

const SOUTH_HEDGE: &str = "EVENT_MN_GYM_7_CUT_SOUTH_HEDGE_PRUNED";
const LIVRA: &str = "EVENT_MN_GYM_8_TRAINER_1_DEFEATED";

// In badge order, then the map's numbered trainer order. St Paul's students
// each have an independently accessible wing. Bemidji's third student and
// Duluth's third student guard optional routes but still initiate on sight.
const GYMS: [(&str, &[Approach]); 8] = [
    (
        "MnRochesterGym",
        &[
            approach((13, 15), Up, &[]),
            approach((14, 8), Up, &["EVENT_MN_GYM_1_TRAINER_1_DEFEATED"]),
        ],
    ),
    (
        "MnMankatoGym",
        &[
            approach((4, 16), Left, &[]),
            approach((12, 9), Up, &["EVENT_MN_GYM_2_TRAINER_1_DEFEATED"]),
        ],
    ),
    (
        "MnMinneapolisGym",
        &[
            approach((2, 16), Up, &[]),
            approach(
                (7, 8),
                Right,
                &[
                    "EVENT_MN_GYM_3_TRAINER_1_DEFEATED",
                    "EVENT_MN_GYM_3_V2_WEST_BRIDGE_OPEN",
                    "EVENT_MN_GYM_3_V2_WEST_FEED_ON",
                ],
            ),
        ],
    ),
    (
        "MnStPaulGym",
        &[approach((2, 8), Right, &[]), approach((26, 7), Left, &[])],
    ),
    (
        "MnStCloudGym",
        &[
            approach((5, 12), Up, &[]),
            approach(
                (12, 7),
                Up,
                &[
                    "EVENT_MN_GYM_5_TRAINER_1_DEFEATED",
                    "EVENT_MN_GYM_5_V2_LOWER_TILE_A",
                    "EVENT_MN_GYM_5_V2_LOWER_TILE_B",
                    "EVENT_MN_GYM_5_V2_RELAY_EAST",
                    "EVENT_MN_GYM_5_V2_UPPER_TILE_A",
                    "EVENT_MN_GYM_5_V2_UPPER_TILE_B",
                ],
            ),
        ],
    ),
    (
        "MnBrainerdGym",
        &[approach((7, 8), Left, &[]), approach((11, 6), Up, &[])],
    ),
    (
        "MnBemidjiGym",
        &[
            approach((3, 11), Right, &[SOUTH_HEDGE]),
            approach(
                (7, 5),
                Right,
                &[
                    SOUTH_HEDGE,
                    "EVENT_MN_GYM_7_CUT_NORTH_HEDGE_PRUNED",
                    "EVENT_MN_GYM_7_TRAINER_3_DEFEATED",
                    "EVENT_MN_GYM_7_V2_BRAMBLE_LINK_OPEN",
                ],
            ),
            approach((3, 9), Up, &[SOUTH_HEDGE]),
        ],
    ),
    (
        "MnDuluthGym",
        &[
            approach((2, 11), Right, &[]),
            approach((5, 7), Up, &[LIVRA, "EVENT_MN_GYM_8_TRAINER_3_DEFEATED"]),
            approach((4, 7), Left, &[LIVRA]),
        ],
    ),
];

#[test]
#[ignore = "requires the selected MN_BADGE_TRAILS_PACK"]
fn badge_trails_all_eighteen_trainers_challenge_on_sight() {
    let (root, runtime) = load();
    let data = runtime.data();
    for (gym, (map, cases)) in GYMS.into_iter().enumerate() {
        let students = &data.maps[map].trainer_scripts;
        assert_eq!(students.len(), cases.len(), "{map} sight-test coverage");
        for (number, case) in cases.iter().enumerate() {
            let id = format!("MN_GYM_{}_TRAINER_{}", gym + 1, number + 1);
            let binding = &students[&id];
            assert_eq!(binding.trainer_id, id, "{map} changed this binding");
            let mut c = fixture(&root, &runtime, map, case.from);
            {
                let (state, overworld) = c.shell.shell.session_mut().state_and_overworld_mut();
                for flag in case.flags {
                    state
                        .flags
                        .set_event_flag(flag, true)
                        .expect("gym progress flag");
                }
                data.apply_map_setup_callbacks(state, overworld, map, "MAPSETUP_DOOR")
                    .expect("install earned bridges, relay settings and cut hedges");
                let facing = match case.step {
                    Up => Direction::Up,
                    Left => Direction::Left,
                    Right => Direction::Right,
                    _ => unreachable!("fixture must be a walking input"),
                };
                overworld.set_player_facing(facing);
                state.overworld = OverworldMemory::from_snapshot(&overworld.snapshot());
                let partner = data
                    .create_pokemon("CROOKIT", 15, Dv::default())
                    .expect("regional partner");
                assert!(state.storage.party.add_pokemon(partner));
                assert_eq!(state.storage.party.filled_slots(), 1);
                state.sync_party_from_storage();
                assert!(
                    !state
                        .flags
                        .is_event_flag_set(&binding.event_flag)
                        .expect("trainer flag")
                );
                assert!(
                    overworld
                        .check_trainer_sight_checked_with_filter(|o| o.script == id)
                        .expect("pre-step sight")
                        .is_none(),
                    "{id}: starting inside the trainer's lane"
                );
            }
            mark_runtime_snapshot_dirty(&mut c.shell);
            // The door callback can queue map music. Give the normal presentation
            // clock a neutral frame to release it before polling the held direction.
            c.wait_frames(2)
                .expect("settle map entry and release the joypad");
            let start = TilePosition::new(case.from.0, case.from.1);
            let mut approach_frames = Vec::new();
            for _ in 0..8 {
                c.press(case.step).expect("walk into the trainer's line");
                let frame = c.shell.shell.last_frame().expect("approach frame");
                approach_frames.push((
                    frame.input_mask,
                    frame.snapshot.clone(),
                    frame.movement.clone(),
                ));
                if frame.trainer_sight.is_some() || frame.snapshot.tile != start {
                    break;
                }
            }
            let step = c
                .shell
                .shell
                .last_frame()
                .expect("directional movement frame");
            let sight = step.trainer_sight.as_ref().unwrap_or_else(|| {
                let session = c.shell.shell.session();
                let script = &session.state().script_runtime;
                panic!(
                    "{id}: no automatic sight. Frames {approach_frames:?}. \
                    Menu {:?}, text {:?}, queued {:?}, map events {:?}, audio {:?}, field {:?}",
                    script.active_menu,
                    script.pending_text_label,
                    script.next_script,
                    script.map_events,
                    script.audio_events,
                    c.shell.field_notice
                )
            });
            assert!(step.interaction.is_none(), "{id}: no A was pressed");
            assert_eq!(sight.script, id);
            let (x, y) = case.from;
            let dest = match case.step {
                Up => TilePosition::new(x, y - 1),
                Left => TilePosition::new(x - 1, y),
                Right => TilePosition::new(x + 1, y),
                _ => unreachable!("fixture must be a walking input"),
            };
            assert_eq!(sight.player_tile, dest, "{id} sight occurs on entry");
            let intro = c
                .shell
                .pending_trainer_intro
                .as_ref()
                .expect("sight must open the trainer's challenge without a button");
            assert_eq!(intro.origin_map_name, map);
            assert_eq!(intro.source_script, id);
            let snapshot = c.snapshot().expect("trainer seen dialogue");
            assert!(
                snapshot.battle.is_none() && snapshot.ui.pending_yes_no.is_none(),
                "{id}"
            );
            let expected_text = render_visible_script_text_body(
                data.script_text_body_for_map(map, &binding.seen_text)
                    .expect("authored challenge"),
                &snapshot.script_events.named_buffers,
                &snapshot.trainer.player_name,
                visible_rival_name(&snapshot),
                snapshot.progression.time.day_of_week,
            );
            assert!(!expected_text.is_empty(), "{id} missing seen dialogue");
            assert_eq!(
                c.shell.field_notice.as_deref(),
                Some(expected_text.as_str()),
                "{id}"
            );
            for _ in 0..16 {
                if c.shell.shell.has_active_battle() {
                    break;
                }
                assert!(
                    c.shell.pending_trainer_intro.is_some(),
                    "{id}: A may only advance seen dialogue"
                );
                c.press(GameButton::A).expect("advance challenge dialogue");
            }
            let state = c.shell.shell.session().state();
            let BattleMemory::Trainer {
                trainer_id,
                source_script,
                event_flag,
                seen_text,
                enemy_party,
                ..
            } = &state.battle
            else {
                panic!("{id}: the authored trainer battle did not start");
            };
            assert_eq!(trainer_id, &id);
            assert_eq!(source_script, &id);
            assert_eq!(event_flag, &binding.event_flag);
            assert_eq!(seen_text, &binding.seen_text);
            assert!(
                !state
                    .flags
                    .is_event_flag_set(event_flag)
                    .expect("unresolved battle flag")
            );
            let authored = &data.trainers.get(&id).expect("native team").party;
            assert_eq!(enemy_party.len(), authored.len(), "{id} team size");
            for (actual, original) in enemy_party.iter().zip(authored) {
                assert_eq!(actual.species.id, original.species, "{id} species");
                assert_eq!(actual.level, original.level, "{id} level");
            }
        }
    }
}
