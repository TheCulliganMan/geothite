//! A run in progress is the fixture. Retreat, battle loss, clinic recovery,
//! Save/Continue and registering again all go through the player controller.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton::{A, Down, Up};
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{RuntimeBadgeRegion, VisibleShellController, mark_runtime_snapshot_dirty};
use super::badge_trails::load;
use super::interior_life::fixture;
use super::minnesota_gyms::{interact, walk_to};
use super::northstar_ice_lair::{choice, event, page};
use super::r10_gallery::{continue_save, settle};
use super::rochester_earned::try_league_battle;
use super::winona::{at, press, travel};

const ACTIVE: &str = "EVENT_MN_LEAGUE_RUN_ACTIVE";
const CHAMPION: &str = "EVENT_MN_LEAGUE_CHAMPION";
const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";
const CLINIC: &str = "MnElyPokecenter1F";
const LOBBY: &str = "MnLeagueLobby";
const HALL: &str = "MnLeagueHallOfFame";
const CHAMBERS: [(&str, &str); 5] = [
    ("MnLeagueRoom1", "MN_LEAGUE_1"),
    ("MnLeagueRoom2", "MN_LEAGUE_2"),
    ("MnLeagueRoom3", "MN_LEAGUE_3"),
    ("MnLeagueRoom4", "MN_LEAGUE_4"),
    ("MnLeagueChampion", "MN_CHAMPION"),
];

fn in_progress(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    stage: usize,
    prior_champion: bool,
) -> VisibleShellController {
    let (map, start) = match CHAMBERS.get(stage) {
        Some(&(chamber, _)) => (chamber, (5, 8)),
        None => {
            assert_eq!(stage, CHAMBERS.len(), "all five matches precede the Hall");
            (HALL, (4, 12))
        }
    };
    let mut c = fixture(root, runtime, map, start);
    for slot in 0..8 {
        c.shell
            .shell
            .award_badge(RuntimeBadgeRegion::Johto, slot)
            .expect("previous badge");
    }
    let state = c.shell.shell.session_mut().state_mut();
    for (flag, value) in [(ACTIVE, true), (RESTORED, true), (CHAMPION, prior_champion)] {
        state
            .flags
            .set_event_flag(flag, value)
            .expect("prior campaign");
    }
    state
        .script_runtime
        .memory
        .insert("wMnLeagueRun".into(), stage.to_string());
    // The actual registrar sets this destination; it is asserted again after
    // the new run is registered with the controller.
    state.last_spawn_map_constant = Some("MN_ELY_POKECENTER_1F".into());
    let mut partner = runtime
        .data()
        .create_pokemon("CINDORE", 1, Dv::default())
        .expect("regional partner");
    partner.hp = 1;
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.money = 8_000;
    assert!(
        state
            .bag
            .add_item(&runtime.data().items["REPEL"], 1)
            .expect("carried Repel")
    );
    mark_runtime_snapshot_dirty(&mut c.shell);
    c
}

fn check_run(c: &VisibleShellController, active: bool, wins: usize, champion: bool) {
    assert_eq!(event(c, ACTIVE), active);
    assert_eq!(event(c, CHAMPION), champion);
    assert!(event(c, RESTORED));
    let state = c.shell.shell.session().state();
    assert_eq!(state.badges.johto, [true; 8]);
    assert_eq!(
        state.script_runtime.memory["wMnLeagueRun"],
        wins.to_string()
    );
}

fn registration(c: &mut VisibleShellController, r: &CrystalRuntime, yes: bool) {
    walk_to(c, r, (5, 4));
    press(c, Up);
    press(c, A);
    page(c, "MnElyRegistrationText");
    let mut pages = Vec::new();
    for _ in 0..16 {
        c.wait_frames(128).expect("registrar printer");
        let snapshot = c.snapshot().expect("registration page");
        let printed = snapshot.ui.text.expect("registrar's printed question");
        assert_eq!(printed.label, "MnElyRegistrationText");
        let text = printed
            .asm_text
            .expect("printed page")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let last = text.contains("this lobby.");
        if pages.last() != Some(&text) {
            pages.push(text);
        }
        if last {
            break;
        }
        press(c, A);
    }
    let text = pages.join(" ");
    for rule in [
        "You can go back to earlier rooms.",
        "you lose a match",
        "this lobby.",
    ] {
        assert!(text.contains(rule), "registration omitted {rule:?}: {text}");
    }
    choice(c, yes);
    if !yes {
        page(c, "MnElyPrepareOutsideText");
    }
    settle(c, "registrar decision");
}

fn restart_at_fern(c: &mut VisibleShellController, r: &CrystalRuntime, champion: bool) {
    registration(c, r, false);
    at(c, LOBBY, (5, 4));
    check_run(c, false, 0, champion);
    registration(c, r, true);
    at(c, CHAMBERS[0].0, (4, 16));
    check_run(c, true, 0, champion);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        Some("MN_ELY_POKECENTER_1F")
    );
    walk_to(c, r, (4, 4));
    press(c, Up);
    let blocked = c.snapshot().expect("closed northern door").overworld.tile;
    press(c, Up);
    assert_eq!(c.snapshot().expect("still gated").overworld.tile, blocked);
    interact(c, r, (5, 7));
    let intro = if champion {
        "MnLeagueRoom1RematchText"
    } else {
        "MnLeagueRoom1BeforeText"
    };
    page(c, intro);
}

#[test]
#[ignore = "requires the written Cave Returns edition in MN_BADGE_TRAILS_PACK"]
fn league_return_trip_loss_and_withdrawal_restart_at_fern() {
    let (root, runtime) = load();
    let first_run = (0..CHAMBERS.len()).map(|stage| (stage, false));
    for (stage, champion) in first_run.chain([(4, true)]) {
        let (map, trainer) = CHAMBERS[stage];
        let mut c = in_progress(&root, &runtime, stage, champion);
        let carried = c.shell.shell.session().state().bag.clone();
        interact(&mut c, &runtime, (5, 7));
        if champion {
            page(&mut c, "MnLeagueChampionRematchText");
            choice(&mut c, true);
        } else {
            page(&mut c, &format!("{map}BeforeText"));
        }
        assert!(!try_league_battle(
            &mut c,
            &runtime,
            trainer,
            u8::try_from(stage + 1).expect("five stages")
        ));
        settle(&mut c, "defeated at the League");
        at(&mut c, CLINIC, (3, 5));
        check_run(&c, false, 0, champion);
        let state = c.shell.shell.session().state();
        assert_eq!(state.battle_result & 0x3f, 1);
        assert_eq!(state.money, 4_000);
        assert!(state.bag == carried);
        let partner = state.storage.party.pokemon[0].as_ref().expect("Cindore");
        assert_eq!(partner.hp, partner.max_hp);
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        at(&mut c, CLINIC, (3, 5));
        check_run(&c, false, 0, champion);
        walk_to(&mut c, &runtime, (3, 6));
        travel(&mut c, Down, "MnEly", (11, 22));
        walk_to(&mut c, &runtime, (25, 14));
        travel(&mut c, Up, LOBBY, (1, 4));
        restart_at_fern(&mut c, &runtime, champion);
    }
}

#[test]
#[ignore = "requires the written Cave Returns edition in MN_BADGE_TRAILS_PACK"]
fn league_return_trip_backtracking_preserves_wins_until_the_lobby() {
    let (root, runtime) = load();
    let mut c = in_progress(&root, &runtime, 4, true);
    // Prior champion declines Casey's rematch and takes a real return trip.
    interact(&mut c, &runtime, (5, 7));
    page(&mut c, "MnLeagueChampionRematchText");
    choice(&mut c, false);
    page(&mut c, "MnCaseyTakeABreathText");
    settle(&mut c, "declined rematch");
    walk_to(&mut c, &runtime, (4, 22));
    travel(&mut c, Down, CHAMBERS[3].0, (4, 4));
    check_run(&c, true, 4, true);
    c = continue_save(root, runtime.clone(), &mut c);
    check_run(&c, true, 4, true);
    // The completed fourth room still admits the player to Casey after Continue.
    walk_to(&mut c, &runtime, (4, 3));
    travel(&mut c, Up, CHAMBERS[4].0, (4, 22));
    for stage in (0..5).rev() {
        check_run(&c, true, 4, true);
        let door = if stage == 4 { (4, 22) } else { (4, 16) };
        walk_to(&mut c, &runtime, door);
        if stage > 0 {
            travel(&mut c, Down, CHAMBERS[stage - 1].0, (4, 4));
        } else {
            travel(&mut c, Down, LOBBY, (1, 4));
        }
    }
    check_run(&c, false, 0, true);
    assert_eq!(c.shell.shell.session().state().money, 8_000);
    restart_at_fern(&mut c, &runtime, true);
}

#[test]
#[ignore = "requires the installed Ely League in MN_BADGE_TRAILS_PACK"]
fn ely_hall_can_be_revisited_from_both_exits_before_recording() {
    let (root, runtime) = load();
    for lane in [4_u8, 5] {
        let mut c = in_progress(&root, &runtime, 5, false);
        let before = c.snapshot().expect("five wins before induction");
        walk_to(&mut c, &runtime, (u16::from(lane), 12));
        let lane = i16::from(lane);
        travel(&mut c, Down, CHAMBERS[4].0, (lane, 2));
        check_run(&c, true, 5, false);
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        at(&mut c, CHAMBERS[4].0, (lane, 2));
        travel(&mut c, Up, HALL, (lane, 12));
        check_run(&c, true, 5, false);
        let returned = c.snapshot().expect("back in the Hall");
        assert_eq!(returned.party, before.party);
        assert_eq!(returned.bag, before.bag);
        assert_eq!(c.shell.shell.session().state().money, 8_000);
        walk_to(&mut c, &runtime, (6, 5));
        press(&mut c, Up);
        at(&mut c, HALL, (6, 5));
        press(&mut c, A);
        page(&mut c, "MnElyRecordText");
        check_run(&c, true, 5, false);
    }
}
