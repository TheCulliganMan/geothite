//! Played rival opening and real egg rewards; capacity cases are explicit fixtures.
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty, visible_field_dialogue_is_entirely_consumed,
};
use super::authored_route_fixture::indoor_walk;
use super::winona::{at, finish, load, press, travel};

const LAB: &str = "MnWinonaFieldStation";
const MET: &str = "EVENT_MN_REED_FIRST_MET";
const WON: &str = "EVENT_MN_REED_WINONA_DEFEATED";
const EGG: &str = "EVENT_MN_TAMARACK_EGG_RECEIVED";

fn event(c: &VisibleShellController, name: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("event flag")
}

pub(super) fn observe_case(c: &mut VisibleShellController) {
    let s = c.snapshot().expect("live rival scene");
    if !s
        .ui
        .text
        .as_ref()
        .is_some_and(|t| t.label == "MnNSEReedDismissText")
    {
        return;
    }
    for id in ["MN_NSE_REED", "MN_NSE_EGG_CASE", "MN_W_TAMARACK"] {
        assert!(
            s.visible_objects
                .iter()
                .any(|o| o.object_identifier.as_deref() == Some(id)),
            "acted case scene needs visible {id}"
        );
    }
    if let Some(directory) = std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR") {
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("scene-save directory");
        c.save(directory.join("reed-case-visible.crystalsave"))
            .expect("actual scene checkpoint");
    }
}

pub(super) fn play_reed(c: &mut VisibleShellController) {
    let before = c.snapshot().expect("first rival battle");
    assert!(event(c, MET) && !event(c, WON));
    let first = &before.party.slots[0].pokemon;
    let attack = first
        .moves
        .iter()
        .find(|m| matches!(m.name.as_str(), "TACKLE" | "SCRATCH" | "POUND"))
        .expect("starter's actual attack")
        .clone();
    let mut attacks = 0;
    for _ in 0..1024 {
        let s = c.snapshot().expect("rival battle input");
        let Some(battle) = s.battle else {
            break;
        };
        assert!(
            matches!(battle.kind, RuntimeBattleKind::Trainer { ref trainer_id, .. } if trainer_id == "MN_REED_WINONA")
        );
        assert_eq!(battle.enemy_pokemon.species.id, "CROOKIT");
        assert!(!event(c, WON));
        let button = if s.ui.menu.is_none() && s.ui.text.is_some() {
            GameButton::A
        } else if let Some(cursor) = &c.shell.battle_move_cursor {
            let target = battle
                .player_moves
                .iter()
                .position(|m| m.name == attack.name && m.current_pp > 0)
                .expect("attack with actual PP");
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                attacks += 1;
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match cursor.option_index {
                0 => GameButton::A,
                1 => GameButton::Left,
                2 | 3 => GameButton::Up,
                _ => panic!("battle cursor"),
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    let after = c.snapshot().expect("rival terminal state");
    assert!(after.battle.is_none() && attacks > 0);
    let partner = &after.party.slots[0].pokemon;
    assert!(
        partner.experience > first.experience,
        "the selected turns must actually win: before {} L{} HP{} XP{}; after {} L{} HP{} XP{}; attacks={attacks}; map={}; won={}",
        first.species.id,
        first.level,
        first.hp,
        first.experience,
        partner.species.id,
        partner.level,
        partner.hp,
        partner.experience,
        after.overworld.map_name,
        event(c, WON)
    );
    assert!(
        partner
            .moves
            .iter()
            .find(|m| m.name == attack.name)
            .expect("retained attack")
            .current_pp
            < attack.current_pp
    );
}

fn checkpoint(
    c: &mut VisibleShellController,
    root: &crystal_assets::AssetRoot,
    r: &CrystalRuntime,
    stage: &str,
) -> VisibleShellController {
    let dir = std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&dir).expect("save directory");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = dir.join(format!(
        "events-{stage}-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    c.save(&path).expect("native event save");
    VisibleShellController::load_save(root.clone(), r.clone(), path, None)
        .expect("same-pack Continue")
}

fn offer(c: &mut VisibleShellController, yes: bool) {
    press(c, GameButton::Up);
    press(c, GameButton::A);
    choice(c, yes);
    finish(c);
}

fn choice(c: &mut VisibleShellController, yes: bool) {
    for _ in 0..64 {
        let s = c.shell.shell.snapshot().expect("authoritative offer");
        if s.ui.pending_yes_no.is_some()
            && visible_field_dialogue_is_entirely_consumed(&c.shell, &s)
        {
            let target = usize::from(!yes);
            for _ in 0..4 {
                let index = c
                    .shell
                    .yes_no_cursor
                    .as_ref()
                    .expect("visible YES/NO")
                    .option_index;
                if index == target {
                    press(c, GameButton::A);
                    return;
                }
                press(
                    c,
                    if index < target {
                        GameButton::Down
                    } else {
                        GameButton::Up
                    },
                );
            }
            panic!("offer cursor did not select the requested answer");
        }
        press(c, GameButton::A);
    }
    panic!("egg offer never finished displaying");
}

fn eggs(c: &VisibleShellController) -> usize {
    c.shell
        .shell
        .session()
        .state()
        .storage
        .party
        .pokemon
        .iter()
        .flatten()
        .filter(|p| p.is_egg)
        .count()
}

#[test]
#[ignore = "requires the event-driven opening pack and current Winona source"]
fn story_events_earned_reed_battle_ivy_quest_and_real_egg_decline_retry_continue() {
    // This course uses production New Game and earns the rival and delivery.
    // There are no party, level, position, result-flag or RNG writes here.
    let (root, r, mut c) = super::rochester::delivery();
    assert!(event(&c, MET) && event(&c, WON) && !event(&c, EGG));
    indoor_walk(&mut c, &r, (10, 14));
    travel(&mut c, GameButton::Down, "MnRochester", (15, 8));
    indoor_walk(&mut c, &r, (38, 18));
    travel(&mut c, GameButton::Right, "MnR01C00R00", (0, 24));
    super::rochester::trail(&mut c, &r, (63, 24));
    travel(&mut c, GameButton::Right, "MnWinona", (0, 18));
    travel(&mut c, GameButton::Right, "MnWinona", (4, 18));
    indoor_walk(&mut c, &r, (18, 20));
    travel(&mut c, GameButton::Up, LAB, (4, 10));
    indoor_walk(&mut c, &r, (5, 3));
    c = checkpoint(&mut c, &root, &r, "egg-ready");
    let before = c.shell.shell.session().state().clone();
    offer(&mut c, false);
    assert!(!event(&c, EGG));
    assert_eq!(eggs(&c), 0);
    assert_eq!(c.shell.shell.session().state().storage, before.storage);
    c = checkpoint(&mut c, &root, &r, "egg-declined");
    offer(&mut c, true);
    assert!(event(&c, EGG));
    assert_eq!(eggs(&c), 1);
    let egg = c
        .shell
        .shell
        .session()
        .state()
        .storage
        .party
        .pokemon
        .iter()
        .flatten()
        .find(|p| p.is_egg)
        .expect("real party egg");
    assert_eq!(egg.species.id, "GILLPUP");
    assert_eq!(egg.level, 5);
    let after = c.shell.shell.session().state().clone();
    let mut flags = before.flags;
    flags.set_event_flag(EGG, true).expect("only egg result");
    assert_eq!(after.flags, flags);
    assert_eq!(after.bag, before.bag);
    assert_eq!(after.money, before.money);
    assert_eq!(after.badges, before.badges);
    assert_eq!(
        after.pokedex, before.pokedex,
        "an egg is not a caught/seen species"
    );
    c = checkpoint(&mut c, &root, &r, "egg-given");
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(
        c.shell.shell.session().state().storage,
        after.storage,
        "repeat cannot duplicate the egg"
    );
    at(&mut c, LAB, (5, 3));
}

#[test]
#[ignore = "explicit initial six-member party; requires the event-driven opening pack"]
fn story_events_egg_full_party_is_retryable_without_pc_fallback_or_result() {
    let (root, r) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("HOME"),
            map_name: LAB.into(),
            tile_x: 5,
            tile_y: 3,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("capacity fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    for _ in 0..6 {
        let p = r
            .data()
            .create_pokemon("CROOKIT", 5, Dv::default())
            .expect("legal initial party");
        assert!(
            c.shell
                .shell
                .session_mut()
                .state_mut()
                .storage
                .party
                .add_pokemon(p)
        );
    }
    let state = c.shell.shell.session_mut().state_mut();
    state.sync_party_from_storage();
    for flag in [
        "EVENT_MN_HOME_INTRO",
        "EVENT_MN_STARTER_RECEIVED",
        "EVENT_MN_R01_PARCEL_RECEIVED",
        "EVENT_MN_WINONA_BALLS_RECEIVED",
        "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
        MET,
        WON,
    ] {
        state
            .flags
            .set_event_flag(flag, true)
            .expect("explicit prior-access fixture");
    }
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64).expect("settle fixture");
    // No game-state writes after this explicit setup boundary.
    let before = c.shell.shell.session().state().clone();
    for stage in ["full-first", "full-repeat"] {
        offer(&mut c, true);
        assert!(!event(&c, EGG));
        assert_eq!(eggs(&c), 0);
        assert_eq!(
            c.shell.shell.session().state().storage,
            before.storage,
            "giveegg must not fall back to the PC"
        );
        assert_eq!(c.shell.shell.session().state().flags, before.flags);
        c = checkpoint(&mut c, &root, &r, stage);
    }
}

#[test]
#[ignore = "explicit non-attacking party fixture; requires the event opening pack"]
fn story_events_first_rival_loss_keeps_the_parcel_and_recovers_without_repeating_the_scene() {
    let (root, r) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("HOME"),
            map_name: LAB.into(),
            tile_x: 6,
            tile_y: 4,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit first-loss fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    let partner = r
        .data()
        .create_pokemon("CROOKIT", 5, Dv::default())
        .expect("legal loss partner");
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.last_spawn_map_constant = Some("MN_WINONA_HOME_1F".into());
    for f in ["EVENT_MN_HOME_INTRO", "EVENT_MN_STARTER_RECEIVED"] {
        state
            .flags
            .set_event_flag(f, true)
            .expect("explicit prior access");
    }
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64).expect("fixture ready");
    // No state writes beyond setup. Select its actually learned TAIL WHIP;
    // ordinary enemy attacks, rather than a result mutation, cause the loss.
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    let mut saw = false;
    let mut spent_pp = false;
    for _ in 0..1024 {
        let s = c.snapshot().expect("native loss course");
        if let Some(battle) = &s.battle {
            saw = true;
            assert!(
                matches!(battle.kind,RuntimeBattleKind::Trainer {ref trainer_id,..} if trainer_id == "MN_REED_WINONA")
            );
            assert!(!event(&c, WON));
            spent_pp |= battle
                .player_moves
                .iter()
                .any(|m| m.name == "TAIL_WHIP" && m.current_pp < r.data().moves["TAIL_WHIP"].pp);
        } else if saw {
            break;
        }
        let button = if s.ui.menu.is_none() && s.ui.text.is_some() {
            GameButton::A
        } else if let Some(cursor) = &c.shell.battle_move_cursor {
            let target = s
                .battle
                .as_ref()
                .expect("native loss battle")
                .player_moves
                .iter()
                .position(|m| m.name == "TAIL_WHIP" && m.current_pp > 0)
                .expect("learned non-damaging move");
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else {
            GameButton::A
        };
        press(&mut c, button);
    }
    assert!(
        saw && spent_pp,
        "loss must execute the selected native turn"
    );
    finish(&mut c);
    assert!(event(&c, MET) && !event(&c, WON));
    assert!(
        event(&c, "EVENT_MN_R01_PARCEL_RECEIVED") && event(&c, "EVENT_MN_WINONA_BALLS_RECEIVED")
    );
    assert!(!event(&c, "EVENT_MN_DELIVERED_ROCHESTER_PARCEL") && !event(&c, EGG));
    at(&mut c, "MnWinonaHome1F", (7, 6));
    c = checkpoint(&mut c, &root, &r, "rival-loss-recovery");
    let balls = c
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["POKE_BALL"]);
    assert_eq!(balls, 5);
    indoor_walk(&mut c, &r, (6, 6));
    travel(&mut c, GameButton::Down, "MnWinona", (7, 8));
    indoor_walk(&mut c, &r, (18, 20));
    travel(&mut c, GameButton::Up, LAB, (4, 10));
    indoor_walk(&mut c, &r, (6, 4));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(
        !event(&c, WON),
        "repeat must not forge victory or restart the first battle"
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["POKE_BALL"]),
        balls
    );
}
