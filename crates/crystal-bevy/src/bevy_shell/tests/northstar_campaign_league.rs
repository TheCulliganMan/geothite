//! Continue the actual regional party through registration and Hall of Fame.
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::rochester_earned::{checkout, try_league_battle};
use super::super::winona::{at, continue_in_place, finish, load, press, travel};
use super::ely_league::{registrar, restore_partner_with, run, until_map};
use super::north_shore_story::choice;
use super::northstar_campaign_finale::{finale, walk_to};
use super::northstar_campaign_review::boost;
use super::{VisibleShellController, event, interact};

const ACTIVE: &str = "EVENT_MN_LEAGUE_RUN_ACTIVE";
const CHAMPION: &str = "EVENT_MN_LEAGUE_CHAMPION";
const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";
const CLINIC: &str = "MnElyPokecenter1F";

fn arrival() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let Some(path) = std::env::var_os("CRYSTAL_NORTHSTAR_LEAGUE_SAVE") else {
        return finale();
    };
    let (root, r) = load();
    let mut c =
        VisibleShellController::load_save(root.clone(), r.clone(), PathBuf::from(path), None)
            .expect("actual same-pack League checkpoint");
    c.set_runtime_journal_enabled(false);
    assert_eq!(c.shell.shell.session().state().badges.johto, [true; 8]);
    for flag in [
        RESTORED,
        "EVENT_MN_SURVEY_CHIEF_DEFEATED",
        "EVENT_MN_LIGHTHOUSE_SIGNAL",
    ] {
        assert!(event(&c, flag), "played prerequisite {flag}");
    }
    assert!(!event(&c, CHAMPION));
    let s = c.snapshot().expect("settled League checkpoint");
    assert!(s.battle.is_none() && s.ui.text.is_none() && s.ui.menu.is_none());
    assert!(
        s.overworld.map_name == "MnR13C00R02"
            || s.overworld.map_name == "MnEly"
            || s.overworld.map_name.starts_with("MnLeague")
    );
    (root, r, c)
}

fn supplies(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk_to(c, r, (11, 22));
    travel(c, GameButton::Up, CLINIC, (3, 5));
    walk_to(c, r, (3, 3));
    press(c, GameButton::Up);
    press(c, GameButton::A);
    let money = c.shell.shell.session().state().money;
    choice(c, true);
    finish(c);
    assert_eq!(c.shell.shell.session().state().money, money);
    assert!(
        c.snapshot()
            .expect("free Ely care")
            .party
            .slots
            .iter()
            .all(|s| s.pokemon.hp == s.pokemon.max_hp
                && s.pokemon.status.is_none()
                && s.pokemon
                    .moves
                    .iter()
                    .all(|m| m.current_pp == r.data().moves[&m.name].pp))
    );
    walk_to(c, r, (3, 6));
    travel(c, GameButton::Down, "MnEly", (11, 22));
    walk_to(c, r, (27, 28));
    travel(c, GameButton::Up, "MnElyMart", (2, 5));
    for (id, desired) in [("FULL_HEAL", 2_u16), ("REVIVE", 1), ("MAX_POTION", 6)] {
        let item = &r.data().items[id];
        let state = c.shell.shell.session().state();
        let count = desired
            .saturating_sub(state.bag.quantity(item))
            .min((state.money / u32::from(item.price)) as u16);
        if count == 0 {
            continue;
        }
        walk_to(c, r, (3, 3));
        press(c, GameButton::Left);
        press(c, GameButton::A);
        checkout(c, r, id, count);
    }
    eprintln!(
        "Boosted League review carried medicine: MAX_POTION={}, FULL_HEAL={}, REVIVE={}, remaining earned money={}",
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["MAX_POTION"]),
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["FULL_HEAL"]),
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["REVIVE"]),
        c.shell.shell.session().state().money
    );
    walk_to(c, r, (2, 6));
    travel(c, GameButton::Down, "MnEly", (27, 28));
}

fn heal(c: &mut VisibleShellController, r: &CrystalRuntime) {
    for slot in 0..c.snapshot().expect("carried partners").party.slots.len() {
        if c.snapshot().expect("native partner").party.slots[slot]
            .pokemon
            .hp
            == 0
        {
            restore_partner_with(c, r, "REVIVE", slot);
        }
        if c.snapshot().expect("native status").party.slots[slot]
            .pokemon
            .status
            .is_some()
        {
            restore_partner_with(c, r, "FULL_HEAL", slot);
        }
        // Save a native Max Potion when a partner has lost only a few HP.
        let s = c.snapshot().expect("native pre-match HP");
        let p = &s.party.slots[slot].pokemon;
        if p.hp < p.max_hp * 3 / 4 {
            restore_partner_with(c, r, "MAX_POTION", slot);
        }
    }
}

#[test]
#[ignore = "user-authorized boosted story review from the actual restored-survey journey"]
fn northstar_campaign_boosted_ely_league_hall_of_fame_and_postgame() {
    assert!(std::env::var_os("CRYSTAL_NORTHSTAR_REVIEW_LEVEL").is_some());
    let (root, r, mut c) = arrival();
    boost(&mut c, &r);
    if c.snapshot().expect("Ely approach").overworld.map_name == "MnR13C00R02" {
        travel(&mut c, GameButton::Down, "MnEly", (24, 0));
        travel(&mut c, GameButton::Down, "MnEly", (24, 1));
        continue_in_place(&mut c, &root, &r, "campaign-ely-arrival");
    }
    if c.snapshot()
        .expect("registration approach")
        .overworld
        .map_name
        == "MnEly"
    {
        supplies(&mut c, &r);
        continue_in_place(&mut c, &root, &r, "campaign-ely-supplies");
        walk_to(&mut c, &r, (25, 14));
        travel(&mut c, GameButton::Up, "MnLeagueLobby", (1, 4));
    }
    if c.snapshot()
        .expect("League registration")
        .overworld
        .map_name
        == "MnLeagueLobby"
    {
        registrar(&mut c, &r);
        choice(&mut c, false);
        finish(&mut c);
        assert!(!event(&c, ACTIVE) && run(&c) == 0);
        registrar(&mut c, &r);
        choice(&mut c, true);
        finish(&mut c);
        at(&mut c, "MnLeagueRoom1", (4, 16));
        assert!(event(&c, ACTIVE));
        continue_in_place(&mut c, &root, &r, "campaign-league-registered");
    }
    for (index, id) in [
        (1, "MN_LEAGUE_1"),
        (2, "MN_LEAGUE_2"),
        (3, "MN_LEAGUE_3"),
        (4, "MN_LEAGUE_4"),
        (5, "MN_CHAMPION"),
    ] {
        let map = if index == 5 {
            "MnLeagueChampion".into()
        } else {
            format!("MnLeagueRoom{index}")
        };
        if run(&c) < index {
            assert_eq!(
                c.snapshot().expect("sequential room").overworld.map_name,
                map
            );
            assert_eq!(run(&c), index - 1);
            heal(&mut c, &r);
            walk_to(&mut c, &r, (4, 4));
            if index < 5 {
                press(&mut c, GameButton::Up);
                let tile = c.snapshot().expect("closed door").overworld.tile;
                press(&mut c, GameButton::Up);
                assert_eq!(c.snapshot().expect("no battle skip").overworld.tile, tile);
            }
            interact(&mut c, &r, (5, 7));
            assert!(try_league_battle(&mut c, &r, id, index));
            assert_eq!(run(&c), index);
            assert!(
                !event(&c, CHAMPION),
                "recording is separate from the final battle"
            );
            let money = c.shell.shell.session().state().money;
            interact(&mut c, &r, (5, 7));
            finish(&mut c);
            assert_eq!(c.shell.shell.session().state().money, money);
            continue_in_place(&mut c, &root, &r, &format!("campaign-league-win-{index}"));
        }
        if c.snapshot().expect("won room").overworld.map_name != map {
            continue;
        }
        walk_to(&mut c, &r, if index == 5 { (4, 2) } else { (4, 3) });
        let next = if index == 5 {
            "MnLeagueHallOfFame".into()
        } else if index == 4 {
            "MnLeagueChampion".into()
        } else {
            format!("MnLeagueRoom{}", index + 1)
        };
        until_map(&mut c, GameButton::Up, &next);
    }
    assert_eq!(run(&c), 5);
    let dir = PathBuf::from(
        std::env::var_os("CRYSTAL_WINONA_SAVE_DIR").expect("separate review save folder"),
    );
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = dir.join(format!(
        "boosted-review-league-hof-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    c.shell.quick_save_path = Some(path.clone());
    walk_to(&mut c, &r, (5, 6));
    let team = c.shell.shell.session().state().storage.party.clone();
    interact(&mut c, &r, (5, 5));
    for _ in 0..128 {
        if c.shell.credits_screen.is_some() {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(
        c.shell
            .credits_screen
            .as_ref()
            .is_some_and(|s| s.hall_of_fame.is_some())
    );
    assert!(event(&c, CHAMPION) && !event(&c, ACTIVE) && run(&c) == 0);
    assert_eq!(c.shell.shell.session().state().hall_of_fame.count, 1);
    assert!(path.is_file(), "native ceremony autosaves the actual party");
    for _ in 0..2048 {
        if c.shell.credits_screen.is_none() {
            break;
        }
        c.wait_frames(64).expect("native ceremony clock");
        press(&mut c, GameButton::A);
    }
    assert!(c.shell.credits_screen.is_none());
    let mut c = VisibleShellController::load_save(root.clone(), r.clone(), path, None)
        .expect("actual postgame Continue");
    at(&mut c, CLINIC, (3, 5));
    assert!(event(&c, CHAMPION) && event(&c, RESTORED) && !event(&c, ACTIVE));
    assert_eq!(c.shell.shell.session().state().storage.party, team);
    assert_eq!(c.shell.shell.session().state().hall_of_fame.count, 1);
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .hall_of_fame
            .spawn_after_champion
            .is_none()
    );
    continue_in_place(&mut c, &root, &r, "campaign-league-postgame");
}
