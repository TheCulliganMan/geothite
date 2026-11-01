//! Free shelter/Q07 through production input after explicit initial fixtures.
//! Badges, upstream signal and a legal review party are setup, not earned play.
#[path = "northstar_relay.rs"]
mod northstar_relay;

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, indoor_walk};

const CAMP: &str = "MnBoundaryWaters";
const ROOM: &str = "MnBoundaryWatersSurveyShelter";
const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";
const DEFEATED: &str = "EVENT_MN_SURVEY_CHIEF_DEFEATED";
const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";
const RECEIVER: &str = "MN_BOUNDARY_SURVEY_RECEIVER";

fn load() -> (AssetRoot, CrystalRuntime) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_BOUNDARY_BASECAMP_TEST_PACK")
            .expect("actual published external free basecamp/story pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified actual basecamp pack"),
    )
    .expect("native basecamp runtime");
    (root, runtime)
}

#[derive(Clone, Copy)]
struct Setup {
    map: &'static str,
    at: Point,
    badges: [bool; 8],
    signal: bool,
    injured: bool,
    losing_party: bool,
}

impl Default for Setup {
    fn default() -> Self {
        Self {
            map: ROOM,
            at: (8, 9),
            badges: [true; 8],
            signal: true,
            injured: false,
            losing_party: false,
        }
    }
}

fn controller(root: &AssetRoot, runtime: &CrystalRuntime, setup: Setup) -> VisibleShellController {
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("original HOME"),
            map_name: setup.map.into(),
            tile_x: setup.at.0,
            tile_y: setup.at.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit initial local review placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("initial trainer name");
    let mut pokemon = runtime
        .data()
        .create_pokemon(
            if setup.losing_party {
                "MAGIKARP"
            } else {
                "MEW"
            },
            if setup.losing_party { 5 } else { 80 },
            Dv::default(),
        )
        .expect("actual native legal review partner");
    if !setup.losing_party {
        pokemon.moves = ["SURF", "PSYCHIC_M"]
            .into_iter()
            .map(|name| LearnedMove {
                name: name.into(),
                current_pp: runtime.data().moves[name].pp,
                pp_ups: 0,
            })
            .collect();
    }
    if setup.injured {
        pokemon.hp -= 60;
        pokemon.status = Some("POISON".into());
        for learned in &mut pokemon.moves {
            learned.current_pp = 0;
        }
    } else if setup.losing_party {
        pokemon.hp = 1;
        assert_eq!(pokemon.moves[0].name, "SPLASH");
    }
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(pokemon));
    state.sync_party_from_storage();
    state.badges.johto = setup.badges;
    state.money = 0;
    state
        .flags
        .set_event_flag(SIGNAL, setup.signal)
        .expect("explicit upstream fixture");
    assert!(
        !state
            .flags
            .is_event_flag_set(DEFEATED)
            .expect("chief event")
    );
    assert!(
        !state
            .flags
            .is_event_flag_set(RESTORED)
            .expect("restoration event")
    );
    state.random_state = Default::default();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    // Initial legal partner/injury, badges, upstream signal, money, location and
    // one random stream are explicit setup. No state/clock/divider/flag/position
    // writes follow; victories, healing and restoration use production inputs.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_at(&mut controller, setup.map, setup.at);
    assert_no_safari(&controller);
    controller
}

fn press(controller: &mut VisibleShellController, button: GameButton) {
    controller.press(button).expect("production Q07 input");
    controller
        .wait_frames(16)
        .expect("native presentation frames");
    assert_eq!(controller.shell.last_error, None);
}

fn assert_at(controller: &mut VisibleShellController, map: &str, at: Point) {
    let snapshot = controller.snapshot().expect("actual native location");
    assert_eq!(
        (
            snapshot.overworld.map_name.as_str(),
            (snapshot.overworld.tile.x, snapshot.overworld.tile.y)
        ),
        (map, at),
        "actual facing: {:?}",
        snapshot.overworld.facing,
    );
}

fn enter_shelter(controller: &mut VisibleShellController) {
    // An exterior return faces down. Hold the ordinary upward input through
    // the turn and door approach, stopping as soon as the native warp fires.
    for _ in 0..3 {
        if controller
            .snapshot()
            .expect("door approach")
            .overworld
            .map_name
            == ROOM
        {
            break;
        }
        press(controller, GameButton::Up);
    }
    assert_at(controller, ROOM, (8, 9));
}

fn event(controller: &VisibleShellController, name: &str) -> bool {
    controller
        .shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("registered Q07 event")
}

fn progress(controller: &VisibleShellController) -> [bool; 3] {
    [SIGNAL, DEFEATED, RESTORED].map(|name| event(controller, name))
}

fn assert_no_safari(controller: &VisibleShellController) {
    let state = controller.shell.shell.session().state();
    assert_eq!(
        state.flags.is_engine_flag_set("ENGINE_SAFARI_ZONE"),
        Ok(false)
    );
    for key in ["wSafariBalls", "wSafariSteps"] {
        assert!(
            !state.script_runtime.memory.contains_key(key),
            "free story cannot manufacture Safari budgets"
        );
    }
}

fn talk(controller: &mut VisibleShellController, direction: GameButton) {
    let before = controller
        .snapshot()
        .expect("interaction approach")
        .overworld;
    press(controller, direction);
    let facing = controller
        .snapshot()
        .expect("real actor/console collision")
        .overworld;
    assert_eq!(before.tile, facing.tile);
    assert_eq!(before.map_name, facing.map_name);
    press(controller, GameButton::A);
}

fn finish_dialogue(controller: &mut VisibleShellController) -> BTreeSet<String> {
    let mut labels = BTreeSet::new();
    for _ in 0..192 {
        let snapshot = controller.snapshot().expect("native text ownership");
        assert!(
            snapshot.battle.is_none(),
            "unexpected battle in ordinary dialogue"
        );
        assert!(
            snapshot.ui.pending_yes_no.is_none(),
            "answer the actual native choice explicitly"
        );
        if let Some(text) = &snapshot.ui.text {
            labels.insert(text.label.clone());
        }
        if snapshot.phase == RuntimeShellPhase::Overworld
            && !snapshot.ui.text_window_open
            && !controller.has_pending_script_work()
            && controller.shell.field_notice.is_none()
            && controller.shell.battle_messages.is_empty()
        {
            return labels;
        }
        press(controller, GameButton::A);
    }
    panic!(
        "Q07 dialogue did not release input: {:?}",
        controller.snapshot().expect("diagnosis")
    );
}

fn walk(controller: &mut VisibleShellController, runtime: &CrystalRuntime, to: Point) {
    let map = controller
        .snapshot()
        .expect("walking map")
        .overworld
        .map_name;
    indoor_walk(controller, runtime, to);
    assert_at(controller, &map, to);
}

fn chief(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    walk(controller, runtime, (8, 5));
    talk(controller, GameButton::Right);
}

fn receiver(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    walk(controller, runtime, (12, 4));
    talk(controller, GameButton::Up);
}

fn prompt(controller: &mut VisibleShellController) {
    for _ in 0..96 {
        let snapshot = controller.snapshot().expect("receiver native prompt");
        assert!(snapshot.battle.is_none());
        if let Some(prompt) = snapshot.ui.pending_yes_no {
            assert_eq!(prompt.source_script, RECEIVER);
            assert_eq!(prompt.command_index, 6);
            return;
        }
        assert!(!event(controller, RESTORED));
        press(controller, GameButton::A);
    }
    panic!("receiver native yesorno did not appear");
}

fn answer(controller: &mut VisibleShellController, yes: bool) {
    let original = controller
        .snapshot()
        .expect("choice")
        .ui
        .pending_yes_no
        .expect("receiver choice");
    for _ in 0..96 {
        let selected = controller
            .shell
            .yes_no_cursor
            .as_ref()
            .expect("native yes/no cursor")
            .option_index;
        if selected != usize::from(!yes) {
            press(
                controller,
                if yes {
                    GameButton::Up
                } else {
                    GameButton::Down
                },
            );
        }
        press(controller, GameButton::A);
        if controller
            .snapshot()
            .expect("choice continuation")
            .ui
            .pending_yes_no
            .as_ref()
            != Some(&original)
        {
            return;
        }
    }
    panic!("receiver choice failed to resolve");
}

fn continue_save(
    controller: &mut VisibleShellController,
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    stage: &str,
) -> VisibleShellController {
    let before = controller.snapshot().expect("before production save");
    let flags = progress(controller);
    let badges = controller.shell.shell.session().state().badges.clone();
    let money = controller.shell.shell.session().state().money;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unique save name")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-q07-{stage}-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    controller.save(&path).expect("production Q07 save");
    let mut loaded =
        VisibleShellController::load_save(root.clone(), runtime.clone(), path.clone(), None)
            .expect("native Continue Q07");
    let after = loaded.snapshot().expect("continued Q07 state");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(progress(&loaded), flags);
    assert_eq!(loaded.shell.shell.session().state().badges, badges);
    assert_eq!(loaded.shell.shell.session().state().money, money);
    assert_no_safari(&loaded);
    if let Some(directory) = std::env::var_os("CRYSTAL_BOUNDARY_BASECAMP_SAVE_DIR") {
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("owned review save directory");
        loaded
            .save(directory.join(format!(
                "q07-{stage}-{}-{nonce}.crystalsave",
                std::process::id()
            )))
            .expect("retain unique actual review save");
    }
    std::fs::remove_file(path).expect("remove temporary save only, never a render");
    loaded
}

#[test]
#[ignore = "requires the published external free Boundary Waters basecamp pack"]
fn boundary_basecamp_story_controller_free_ports_medic_both_carpets_and_continue() {
    let (root, runtime) = load();
    for landing in [(56, 24), (32, 40), (6, 24)] {
        let mut controller = controller(
            &root,
            &runtime,
            Setup {
                map: CAMP,
                at: landing,
                badges: [false; 8],
                signal: false,
                injured: true,
                ..Default::default()
            },
        );
        walk(&mut controller, &runtime, (43, 20));
        enter_shelter(&mut controller);
        // Passive arrival must remain in the room on an ordinary owned A.
        press(&mut controller, GameButton::A);
        assert_at(&mut controller, ROOM, (8, 9));
        let initial = controller
            .snapshot()
            .expect("injured party in actual free room");
        assert!(initial.party.slots[0].pokemon.hp < initial.party.slots[0].pokemon.max_hp);
        assert!(
            initial.party.slots[0]
                .pokemon
                .moves
                .iter()
                .all(|m| m.current_pp == 0)
        );
        walk(&mut controller, &runtime, (4, 7));
        talk(&mut controller, GameButton::Left);
        let labels = finish_dialogue(&mut controller);
        assert!(labels.contains("MN_BOUNDARY_SURVEY_MEDICText"));
        let healed = controller.snapshot().expect("native free HealParty result");
        let partner = &healed.party.slots[0].pokemon;
        assert_eq!(partner.hp, partner.max_hp);
        assert_eq!(partner.status, None);
        assert!(
            partner
                .moves
                .iter()
                .all(|m| m.current_pp == runtime.data().moves[&m.name].pp)
        );
        assert_eq!(healed.bag, initial.bag);
        assert_eq!(controller.shell.shell.session().state().money, 0);
        assert_eq!(progress(&controller), [false; 3]);
        assert_no_safari(&controller);
        for carpet in [(8, 11), (9, 11)] {
            indoor_walk(&mut controller, &runtime, carpet);
            // Native downward carpets warp when the player continues across
            // the map edge, rather than when the path finder reaches the tile.
            press(&mut controller, GameButton::Down);
            assert_at(&mut controller, CAMP, (43, 20));
            press(&mut controller, GameButton::A);
            assert_at(&mut controller, CAMP, (43, 20));
            enter_shelter(&mut controller);
        }
        let _continued = continue_save(&mut controller, &root, &runtime, "free-care");
    }
}

#[test]
#[ignore = "requires the published external free Boundary Waters basecamp pack"]
fn boundary_basecamp_story_controller_each_badge_signal_and_receiver_denial() {
    let (root, runtime) = load();
    for missing in 0..9 {
        let mut badges = [true; 8];
        if missing < 8 {
            badges[missing] = false;
        }
        let mut controller = controller(
            &root,
            &runtime,
            Setup {
                badges,
                signal: missing < 8,
                ..Default::default()
            },
        );
        let initial = controller.snapshot().expect("initial denial resources");
        chief(&mut controller, &runtime);
        let before = controller
            .snapshot()
            .expect("chief refusal owns input")
            .overworld
            .tile;
        press(&mut controller, GameButton::Down);
        assert_eq!(
            controller
                .snapshot()
                .expect("dialogue movement rejected")
                .overworld
                .tile,
            before
        );
        let labels = finish_dialogue(&mut controller);
        assert!(labels.contains(if missing < 8 {
            "MnBoundarySurveyChiefBadgeDeniedText"
        } else {
            "MnBoundarySurveyChiefSignalDeniedText"
        }));
        assert_eq!(progress(&controller), [missing < 8, false, false]);
        receiver(&mut controller, &runtime);
        assert!(finish_dialogue(&mut controller).contains("MnBoundarySurveyReceiverLockedText"));
        let after = controller.snapshot().expect("refused chief and receiver");
        assert_eq!(initial.party, after.party);
        assert_eq!(initial.bag, after.bag);
        assert_eq!(
            controller.shell.shell.session().state().badges.johto,
            badges
        );
        assert_eq!(controller.shell.shell.session().state().money, 0);
        assert_eq!(progress(&controller), [missing < 8, false, false]);
        assert_no_safari(&controller);
    }
}

fn play_chief_battle(controller: &mut VisibleShellController, losing: bool) -> BTreeSet<String> {
    let initial = controller.snapshot().expect("challenge resources");
    let mut saw_battle = false;
    let mut selected_attacks = 0;
    let mut opponents = BTreeSet::new();
    let mut admission = String::new();
    for _ in 0..2048 {
        let snapshot = controller
            .snapshot()
            .expect("actual native chief encounter");
        let Some(battle) = &snapshot.battle else {
            if saw_battle {
                break;
            }
            assert!(!event(controller, DEFEATED));
            if let Some(text) = &snapshot.ui.text
                && text.label == "MnBoundarySurveyChiefSeenText"
                && let Some(page) = &text.asm_text
            {
                admission.push_str(page);
                admission.push('\n');
            }
            press(controller, GameButton::A);
            continue;
        };
        saw_battle = true;
        assert!(
            matches!(&battle.kind, RuntimeBattleKind::Trainer { trainer_id, .. } if trainer_id == "MN_LODE_CHIEF")
        );
        opponents.insert(battle.enemy_pokemon.species.id.clone());
        assert!(
            !event(controller, DEFEATED),
            "victory cannot be granted during combat"
        );
        assert!(!event(controller, RESTORED));
        let button = if snapshot.ui.menu.is_none() && snapshot.ui.text.is_some() {
            controller.wait_frames(30).expect("battle narration");
            GameButton::A
        } else if snapshot.ui.menu.as_ref().is_some_and(|menu| {
            menu.layout
                .vertical_menus
                .iter()
                .flat_map(|m| &m.options)
                .any(|s| s.contains("change <PKMN>"))
        }) {
            GameButton::B
        } else if let Some(cursor) = &controller.shell.battle_move_cursor {
            let preferred = if losing {
                "SPLASH"
            } else if battle.enemy_pokemon.species.id == "MAGNETON" {
                "SURF"
            } else {
                "PSYCHIC_M"
            };
            let target = battle
                .player_moves
                .iter()
                .position(|m| m.name == preferred && m.current_pp > 0)
                .expect("actual selected move with PP");
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                selected_attacks += 1;
                GameButton::A
            }
        } else if let Some(cursor) = &controller.shell.battle_action_cursor {
            match cursor.option_index {
                0 => GameButton::A,
                1 => GameButton::Left,
                2 | 3 => GameButton::Up,
                _ => panic!("invalid battle cursor"),
            }
        } else {
            GameButton::A
        };
        press(controller, button);
    }
    assert!(
        saw_battle && selected_attacks > 0,
        "must actually enter and choose a move in the chief battle"
    );
    for page in super::northstar_dialogue::canonical_pages("MS11", "MnBoundarySurveyChiefSeenText")
    {
        assert!(
            admission.contains(&page),
            "missing native chief page: {page}"
        );
    }
    let after = controller.snapshot().expect("actual battle terminal state");
    assert!(after.battle.is_none(), "chief battle did not finish");
    if !losing {
        assert!(after.party.slots[0].pokemon.hp > 0);
        assert!(
            after.party.slots[0].pokemon.experience > initial.party.slots[0].pokemon.experience
        );
        let pp = |moves: &[LearnedMove]| moves.iter().map(|m| u32::from(m.current_pp)).sum::<u32>();
        assert!(
            pp(&after.party.slots[0].pokemon.moves) < pp(&initial.party.slots[0].pokemon.moves),
            "selected attacks must spend native PP"
        );
    }
    opponents
}

#[test]
#[ignore = "requires the published external free Boundary Waters basecamp pack"]
fn boundary_basecamp_story_controller_actual_victory_separate_receiver_retry_and_continue() {
    let (root, runtime) = load();
    let mut controller = controller(&root, &runtime, Setup::default());
    receiver(&mut controller, &runtime);
    assert!(finish_dialogue(&mut controller).contains("MnBoundarySurveyReceiverLockedText"));
    chief(&mut controller, &runtime);
    let opponents = play_chief_battle(&mut controller, false);
    assert_eq!(
        opponents,
        BTreeSet::from(["MAGNETON".into(), "WEEZING".into(), "CROBAT".into()])
    );
    assert!(finish_dialogue(&mut controller).contains("MnBoundarySurveyChiefAfterText"));
    assert_eq!(progress(&controller), [true, true, false]);
    assert_at(&mut controller, ROOM, (8, 5));
    controller = continue_save(&mut controller, &root, &runtime, "chief-victory");
    let won = controller.snapshot().expect("won resources");
    let won_money = controller.shell.shell.session().state().money;
    chief(&mut controller, &runtime);
    assert!(finish_dialogue(&mut controller).contains("MnBoundarySurveyChiefAfterText"));
    assert_eq!(
        controller.snapshot().expect("repeat chief").party,
        won.party
    );
    assert_eq!(controller.shell.shell.session().state().money, won_money);
    receiver(&mut controller, &runtime);
    prompt(&mut controller);
    assert!(!event(&controller, RESTORED));
    answer(&mut controller, false);
    assert!(finish_dialogue(&mut controller).contains("MnBoundarySurveyReceiverDeclinedText"));
    assert_eq!(progress(&controller), [true, true, false]);
    receiver(&mut controller, &runtime);
    prompt(&mut controller);
    answer(&mut controller, true);
    assert!(
        !event(&controller, RESTORED),
        "the accepted prompt must still wait for alignment acknowledgment"
    );
    let labels = finish_dialogue(&mut controller);
    assert!(labels.contains("MnBoundarySurveyReceiverRestoreText"));
    assert!(labels.contains("MnBoundarySurveyReceiverDoneText"));
    assert_eq!(progress(&controller), [true; 3]);
    let restored = controller.snapshot().expect("restoration resources");
    assert_eq!(won.party, restored.party);
    assert_eq!(won.bag, restored.bag);
    assert_eq!(controller.shell.shell.session().state().money, won_money);
    assert_no_safari(&controller);
    controller = continue_save(&mut controller, &root, &runtime, "receiver-restored");
    receiver(&mut controller, &runtime);
    assert!(finish_dialogue(&mut controller).contains("MnBoundarySurveyReceiverAlreadyText"));
    chief(&mut controller, &runtime);
    assert!(finish_dialogue(&mut controller).contains("MnBoundarySurveyChiefAfterText"));
    let repeated = controller.snapshot().expect("once-only story repeats");
    assert_eq!(restored.party, repeated.party);
    assert_eq!(restored.bag, repeated.bag);
    assert_eq!(controller.shell.shell.session().state().money, won_money);
    assert_eq!(progress(&controller), [true; 3]);
    assert_no_safari(&controller);
    if runtime.data().maps[CAMP]
        .attributes
        .connections
        .iter()
        .any(|connection| connection.target_map == "MnR13C00R00")
    {
        // In the connected publication, the actual earned receiver event must
        // release the League road without another fixture flag assignment.
        indoor_walk(&mut controller, &runtime, (8, 11));
        press(&mut controller, GameButton::Down);
        assert_at(&mut controller, CAMP, (43, 20));
        walk(&mut controller, &runtime, (32, 41));
        for y in 42..=47 {
            for _ in 0..4 {
                if controller
                    .snapshot()
                    .expect("League departure")
                    .overworld
                    .tile
                    .y
                    == y
                {
                    break;
                }
                press(&mut controller, GameButton::Down);
                finish_dialogue(&mut controller);
            }
            assert_at(&mut controller, CAMP, (32, y));
        }
        press(&mut controller, GameButton::Down);
        assert_at(&mut controller, "MnR13C00R00", (30, 0));
        assert_eq!(progress(&controller), [true; 3]);
        let _continued = continue_save(&mut controller, &root, &runtime, "earned-league-release");
    }
}

#[test]
#[ignore = "requires the published external free Boundary Waters basecamp pack"]
fn boundary_basecamp_story_controller_played_loss_does_not_grant_victory_or_restoration() {
    let (root, runtime) = load();
    let mut controller = controller(
        &root,
        &runtime,
        Setup {
            losing_party: true,
            ..Default::default()
        },
    );
    chief(&mut controller, &runtime);
    let opponents = play_chief_battle(&mut controller, true);
    assert_eq!(opponents, BTreeSet::from(["MAGNETON".into()]));
    finish_dialogue(&mut controller);
    let after = controller
        .snapshot()
        .expect("played native boss loss/whiteout");
    assert_ne!(
        after.overworld.map_name, ROOM,
        "a normal chief loss must whiteout, not resume its victory continuation"
    );
    assert_eq!(progress(&controller), [true, false, false]);
    assert_no_safari(&controller);
    let _continued = continue_save(&mut controller, &root, &runtime, "played-loss");
}
