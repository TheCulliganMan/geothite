//! Explicit initial Q06 prerequisites/party, then only production controller input.
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

const MINE: &str = "MnR09Cave1F";
const REPORT: &str = "EVENT_MN_HEADWATERS_REPORT";
const DEFEATED: &str = "EVENT_MN_DEPOT_FOREMAN_DEFEATED";
const RECOVERED: &str = "EVENT_MN_MINE_INSTRUMENT_RECOVERED";
const DEPOT: &str = "EVENT_MN_DEPOT_CLEARED";
const CACHE: &str = "MN_R09_SURVEY_CACHE";

fn fixture(
    badge: bool,
    report: bool,
    losing: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_IRON_RANGE_STORY_TEST_PACK")
            .expect("published actual Q06 mine pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("actual verified native pack"),
    )
    .expect("Q06 runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("original HOME"),
            map_name: MINE.into(),
            tile_x: 13,
            tile_y: 18,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local mine placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("initial fixture name");
    let mut pokemon = runtime
        .data()
        .create_pokemon(
            if losing { "MAGIKARP" } else { "MEW" },
            if losing { 5 } else { 80 },
            Dv::default(),
        )
        .expect("legal original species");
    if losing {
        pokemon.hp = 1;
        assert_eq!(pokemon.moves[0].name, "SPLASH");
    } else {
        pokemon.moves = ["SURF", "PSYCHIC_M"]
            .into_iter()
            .map(|name| LearnedMove {
                name: name.into(),
                current_pp: runtime.data().moves[name].pp,
                pp_ups: 0,
            })
            .collect();
    }
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(pokemon));
    state.sync_party_from_storage();
    state.badges.johto[..6].fill(true);
    state.badges.johto[6] = badge;
    state.money = 0;
    state
        .flags
        .set_event_flag(REPORT, report)
        .expect("explicit upstream report setup");
    for name in [DEFEATED, RECOVERED, DEPOT] {
        assert_eq!(state.flags.is_event_flag_set(name), Ok(false));
    }
    state.random_state = Default::default();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|i| ((i.wrapping_mul(53).wrapping_add(19)) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    mark_runtime_snapshot_dirty(&mut controller.shell);
    controller.set_runtime_journal_enabled(false);
    (root, runtime, controller)
}

fn press(controller: &mut VisibleShellController, button: GameButton) {
    controller.press(button).expect("production Q06 input");
    controller
        .wait_frames(16)
        .expect("native presentation wait");
    assert_eq!(controller.shell.last_error, None);
}

fn event(controller: &VisibleShellController, name: &str) -> bool {
    controller
        .shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("registered story event")
}

fn progress(controller: &VisibleShellController) -> [bool; 3] {
    [DEFEATED, RECOVERED, DEPOT].map(|name| event(controller, name))
}

fn finish(controller: &mut VisibleShellController) -> BTreeSet<String> {
    let mut labels = BTreeSet::new();
    for _ in 0..192 {
        let snapshot = controller.snapshot().expect("Q06 native dialogue");
        assert!(snapshot.battle.is_none() && snapshot.ui.pending_yes_no.is_none());
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
    panic!("Q06 dialogue did not release input");
}

fn talk(
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    approach: Point,
    direction: GameButton,
) {
    indoor_walk(controller, runtime, approach);
    let before = controller.snapshot().expect("real talk approach").overworld;
    assert_eq!(before.map_name, MINE);
    assert_eq!((before.tile.x, before.tile.y), approach);
    press(controller, direction);
    assert_eq!(
        controller
            .snapshot()
            .expect("actor collision")
            .overworld
            .tile,
        before.tile
    );
    press(controller, GameButton::A);
}

fn foreman(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    talk(controller, runtime, (9, 6), GameButton::Up);
}

fn cache(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    talk(controller, runtime, (7, 4), GameButton::Down);
}

fn prompt(controller: &mut VisibleShellController) {
    for _ in 0..96 {
        let snapshot = controller.snapshot().expect("cache prompt");
        assert!(snapshot.battle.is_none());
        assert!(!event(controller, RECOVERED));
        if let Some(prompt) = snapshot.ui.pending_yes_no {
            assert_eq!(prompt.source_script, CACHE);
            assert_eq!(prompt.command_index, 7);
            return;
        }
        press(controller, GameButton::A);
    }
    panic!("native cache choice did not appear");
}

fn answer(controller: &mut VisibleShellController, yes: bool) {
    let original = controller
        .snapshot()
        .expect("choice")
        .ui
        .pending_yes_no
        .expect("native yes/no");
    for _ in 0..96 {
        let selected = controller
            .shell
            .yes_no_cursor
            .as_ref()
            .expect("production choice cursor")
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
    panic!("cache choice did not resolve");
}

fn continue_save(
    controller: &mut VisibleShellController,
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    stage: &str,
) -> VisibleShellController {
    let before = controller.snapshot().expect("save snapshot");
    let flags = progress(controller);
    let state = controller.shell.shell.session().state();
    let badges = state.badges.clone();
    let money = state.money;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("save nonce")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-q06-{stage}-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    controller.save(&path).expect("production save");
    let mut loaded =
        VisibleShellController::load_save(root.clone(), runtime.clone(), path.clone(), None)
            .expect("production Continue");
    let after = loaded.snapshot().expect("continued state");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(progress(&loaded), flags);
    assert_eq!(loaded.shell.shell.session().state().badges, badges);
    assert_eq!(loaded.shell.shell.session().state().money, money);
    if let Some(directory) = std::env::var_os("CRYSTAL_IRON_RANGE_STORY_SAVE_DIR") {
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("owned review save directory");
        loaded
            .save(directory.join(format!(
                "q06-{stage}-{}-{nonce}.crystalsave",
                std::process::id()
            )))
            .expect("retained review save");
    }
    std::fs::remove_file(path).expect("remove task save");
    loaded
}

fn battle(controller: &mut VisibleShellController, losing: bool) -> BTreeSet<String> {
    let before = controller.snapshot().expect("pre-battle party");
    let mut saw_battle = false;
    let mut selected = 0;
    let mut opponents = BTreeSet::new();
    for _ in 0..2048 {
        let snapshot = controller.snapshot().expect("actual Q06 battle");
        let Some(battle) = &snapshot.battle else {
            if saw_battle {
                break;
            }
            assert_eq!(progress(controller), [false; 3]);
            press(controller, GameButton::A);
            continue;
        };
        saw_battle = true;
        assert!(
            matches!(&battle.kind, RuntimeBattleKind::Trainer { trainer_id, .. } if trainer_id == "MN_LODE_FOREMAN")
        );
        opponents.insert(battle.enemy_pokemon.species.id.clone());
        assert_eq!(
            progress(controller),
            [false; 3],
            "combat cannot grant quest completion"
        );
        let button = if snapshot.ui.menu.is_none() && snapshot.ui.text.is_some() {
            controller.wait_frames(30).expect("native battle narration");
            GameButton::A
        } else if let Some(cursor) = &controller.shell.battle_move_cursor {
            let name = if losing {
                "SPLASH"
            } else if battle.enemy_pokemon.species.id == "MACHOKE" {
                "PSYCHIC_M"
            } else {
                "SURF"
            };
            let target = battle
                .player_moves
                .iter()
                .position(|m| m.name == name && m.current_pp > 0)
                .expect("selected legal move with PP");
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                selected += 1;
                GameButton::A
            }
        } else if let Some(cursor) = &controller.shell.battle_action_cursor {
            match cursor.option_index {
                0 => GameButton::A,
                1 => GameButton::Left,
                2 | 3 => GameButton::Up,
                _ => panic!("invalid native battle cursor"),
            }
        } else {
            GameButton::A
        };
        press(controller, button);
    }
    assert!(
        saw_battle && selected > 0,
        "must play real selected battle turns"
    );
    let after = controller.snapshot().expect("battle terminal result");
    assert!(after.battle.is_none());
    if !losing {
        let old = &before.party.slots[0].pokemon;
        let new = &after.party.slots[0].pokemon;
        assert!(new.hp > 0 && new.experience > old.experience);
        let pp = |p: &[LearnedMove]| p.iter().map(|m| u32::from(m.current_pp)).sum::<u32>();
        assert!(
            pp(&new.moves) < pp(&old.moves),
            "selected moves must spend production PP"
        );
    }
    opponents
}

#[test]
#[ignore = "requires the published external Q06 story pack"]
fn iron_range_story_controller_refuses_missing_prerequisites_and_locked_cache() {
    for (badge, report, label) in [
        (false, true, "MnIronRangeForemanBadgeDeniedText"),
        (true, false, "MnIronRangeForemanReportDeniedText"),
    ] {
        let (_, runtime, mut controller) = fixture(badge, report, false);
        let before = controller.snapshot().expect("initial resources");
        foreman(&mut controller, &runtime);
        let at = controller
            .snapshot()
            .expect("dialogue owner")
            .overworld
            .tile;
        press(&mut controller, GameButton::Right);
        assert_eq!(
            controller
                .snapshot()
                .expect("dialogue blocks movement")
                .overworld
                .tile,
            at
        );
        assert!(finish(&mut controller).contains(label));
        cache(&mut controller, &runtime);
        assert!(finish(&mut controller).contains("MnIronRangeCacheLockedText"));
        let after = controller.snapshot().expect("refused resources");
        assert_eq!(before.party, after.party);
        assert_eq!(before.bag, after.bag);
        assert_eq!(progress(&controller), [false; 3]);
        assert_eq!(controller.shell.shell.session().state().money, 0);
    }
}

#[test]
#[ignore = "requires the published external Q06 story pack"]
fn iron_range_story_controller_actual_win_separate_recovery_decline_repeat_and_continue() {
    let (root, runtime, mut controller) = fixture(true, true, false);
    cache(&mut controller, &runtime);
    assert!(finish(&mut controller).contains("MnIronRangeCacheLockedText"));
    foreman(&mut controller, &runtime);
    assert_eq!(
        battle(&mut controller, false),
        BTreeSet::from(["GRAVELER".into(), "MACHOKE".into(), "MAGNETON".into()])
    );
    assert!(finish(&mut controller).contains("MnIronRangeForemanAfterText"));
    assert_eq!(progress(&controller), [true, false, false]);
    controller = continue_save(&mut controller, &root, &runtime, "foreman-won");
    let won = controller.snapshot().expect("won resources");
    let money = controller.shell.shell.session().state().money;
    foreman(&mut controller, &runtime);
    assert!(finish(&mut controller).contains("MnIronRangeForemanAfterText"));
    assert_eq!(
        won.party,
        controller.snapshot().expect("repeat foreman").party
    );
    assert_eq!(controller.shell.shell.session().state().money, money);
    cache(&mut controller, &runtime);
    prompt(&mut controller);
    answer(&mut controller, false);
    assert!(finish(&mut controller).contains("MnIronRangeCacheDeclinedText"));
    assert_eq!(progress(&controller), [true, false, false]);
    controller = continue_save(&mut controller, &root, &runtime, "meter-declined");
    cache(&mut controller, &runtime);
    prompt(&mut controller);
    answer(&mut controller, true);
    assert!(
        !event(&controller, RECOVERED),
        "recovery must wait for the meter text acknowledgment"
    );
    assert!(finish(&mut controller).contains("MnIronRangeCacheRecoveredText"));
    assert_eq!(progress(&controller), [true, true, false]);
    controller = continue_save(&mut controller, &root, &runtime, "meter-recovered");
    cache(&mut controller, &runtime);
    assert!(finish(&mut controller).contains("MnIronRangeCacheAlreadyText"));
    let repeated = controller.snapshot().expect("repeat recovery resources");
    assert_eq!(won.party, repeated.party);
    assert_eq!(won.bag, repeated.bag);
    assert_eq!(controller.shell.shell.session().state().money, money);
    assert_eq!(progress(&controller), [true, true, false]);
    indoor_walk(&mut controller, &runtime, (13, 19));
    let outside = controller.snapshot().expect("actual mine return");
    assert_eq!(outside.overworld.map_name, "MnR09C00R00");
    assert_eq!(
        (outside.overworld.tile.x, outside.overworld.tile.y),
        (40, 42)
    );
    assert_eq!(progress(&controller), [true, true, false]);
}

#[test]
#[ignore = "requires the published external Q06 story pack"]
fn iron_range_story_controller_played_loss_never_awards_victory_or_evidence() {
    let (root, runtime, mut controller) = fixture(true, true, true);
    foreman(&mut controller, &runtime);
    assert!(battle(&mut controller, true).contains("GRAVELER"));
    finish(&mut controller);
    assert_eq!(progress(&controller), [false; 3]);
    assert_ne!(
        controller
            .snapshot()
            .expect("production blackout")
            .overworld
            .map_name,
        MINE
    );
    controller = continue_save(&mut controller, &root, &runtime, "foreman-loss");
    assert_eq!(progress(&controller), [false; 3]);
}
