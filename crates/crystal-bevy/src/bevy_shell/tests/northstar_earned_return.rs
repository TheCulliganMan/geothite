//! Earned postgame return: real Fly, restoration reward and caught-Dex claims.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use crate::bevy_shell::tests::interior_life::talk;
use crate::bevy_shell::tests::native_city_ports::city_port;
use crate::bevy_shell::tests::winona::{continue_in_place, press, travel};
use crate::bevy_shell::{PartyAction, PartyFieldMove, StartMenuOption, VisibleShellController};

use super::northstar_campaign::learn_machine_on;
use super::northstar_earned_services::walk;

fn fly_home(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let slot = c
        .snapshot()
        .expect("earned Fly partner")
        .party
        .slots
        .iter()
        .find(|slot| {
            slot.pokemon
                .species
                .tmhm_learnset
                .iter()
                .any(|name| name == "FLY")
        })
        .map(|slot| slot.index)
        .expect("recruit a compatible regional Fly partner through ordinary play");
    learn_machine_on(c, r, "HM_FLY", "FLY", "GROWL", slot);
    press(c, GameButton::Start);
    for _ in 0..16 {
        let snapshot = c.snapshot().expect("native Start options");
        let options = crate::bevy_shell::visible_start_menu_options(&c.shell, &snapshot);
        if options[c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index]
            == StartMenuOption::Pokemon
        {
            break;
        }
        press(c, GameButton::Down);
    }
    press(c, GameButton::A);
    for _ in 0..16 {
        if c.shell.party_cursor == slot {
            break;
        }
        press(
            c,
            if c.shell.party_cursor < slot {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    assert_eq!(c.shell.party_cursor, slot);
    press(c, GameButton::A);
    for _ in 0..16 {
        let snapshot = c.snapshot().expect("native carried field actions");
        let actions =
            crate::bevy_shell::visible_party_actions(&snapshot, &c.shell).expect("party actions");
        if actions[c
            .shell
            .party_action_cursor
            .as_ref()
            .expect("party action cursor")
            .option_index]
            == PartyAction::FieldMove(PartyFieldMove::Fly)
        {
            break;
        }
        press(c, GameButton::Down);
    }
    press(c, GameButton::A);
    assert!(c.shell.fly_cursor.is_some(), "actual Fly menu required");
    for _ in 0..128 {
        let snapshot = c.snapshot().expect("actual visited towns");
        let destinations = crate::bevy_shell::active_fly_destinations(&snapshot, &c.shell.shell)
            .expect("native Fly destinations");
        let index = c
            .shell
            .fly_cursor
            .as_ref()
            .expect("Fly cursor")
            .option_index;
        if destinations[index].flypoint_flag == "ENGINE_FLYPOINT_MN_WINONA" {
            break;
        }
        press(c, GameButton::Down);
    }
    let snapshot = c.snapshot().expect("selected home destination");
    let destinations = crate::bevy_shell::active_fly_destinations(&snapshot, &c.shell.shell)
        .expect("native Fly destinations");
    assert_eq!(
        destinations[c
            .shell
            .fly_cursor
            .as_ref()
            .expect("Fly cursor")
            .option_index]
            .flypoint_flag,
        "ENGINE_FLYPOINT_MN_WINONA"
    );
    press(c, GameButton::A);
    assert!(c.shell.visible_fly_animation.is_some());
    for _ in 0..512 {
        if c.snapshot().expect("native Fly landing").overworld.map_name == "MnWinona" {
            break;
        }
        c.wait_frames(16).expect("native Fly animation");
    }
    for _ in 0..128 {
        if c.shell.visible_fly_animation.is_none() {
            break;
        }
        c.wait_frames(16).expect("native Fly landing settles");
    }
    assert!(c.shell.visible_fly_animation.is_none());
    assert_eq!(
        c.snapshot().expect("actual home return").overworld.map_name,
        "MnWinona"
    );
}

pub(super) fn play(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    assert_eq!(c.shell.shell.session().state().hall_of_fame.count, 1);
    return_and_claim(c, r, root);
}

fn return_and_claim(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    let (door, _) = city_port(r, "MnEly", "MnElyPokecenter1F");
    walk(c, r, (3, 6));
    travel(c, GameButton::Down, "MnEly", door);
    fly_home(c, r);
    continue_in_place(c, root, r, "campaign-earned-fly-home");
    let (door, arrival) = city_port(r, "MnWinona", "MnWinonaFieldStation");
    walk(c, r, door);
    travel(c, GameButton::Up, "MnWinonaFieldStation", arrival);
    walk(c, r, (5, 3));
    talk(c, (5, 2));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["MASTER_BALL"]),
        1
    );
    continue_in_place(c, root, r, "campaign-earned-restoration-master-ball");
    talk(c, (5, 2));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["MASTER_BALL"]),
        1
    );
    let state = c.shell.shell.session().state();
    let caught = r
        .data()
        .story_event_script_constants
        .global
        .keys()
        .filter_map(|key| key.strip_prefix("MN_DEX_MEMBER_"))
        .filter(|species| state.pokedex.has_caught(species))
        .count();
    let total = r
        .data()
        .story_event_script_constants
        .global
        .keys()
        .filter(|key| key.starts_with("MN_DEX_MEMBER_"))
        .count();
    let fifty = r.data().maps["MnWinonaFieldStation"].scripts["MnRewardDexClaim2"]
        .as_array()
        .expect("research gift program")
        .iter()
        .find(|row| row["command"] == "verbosegiveitem")
        .expect("reviewed fifty-caught gift");
    let (fifty_item, fifty_amount) = match (fifty["args"][0].as_str(), fifty["args"][1].as_str()) {
        (Some("EXP_SHARE"), Some("1")) => ("EXP_SHARE", 1),
        (Some("ULTRA_BALL"), Some("10")) => ("ULTRA_BALL", 10),
        other => panic!("unreviewed research gift: {other:?}"),
    };
    let awards = [
        (10, "GREAT_BALL", 10),
        (25, "ITEMFINDER", 1),
        (50, fifty_item, fifty_amount),
        (100, "LUCKY_EGG", 1),
        (total, "RARE_CANDY", 10),
    ];
    let before = awards
        .iter()
        .enumerate()
        .map(|(index, (_, item, _))| {
            let state = c.shell.shell.session().state();
            (
                state.bag.quantity(&r.data().items[*item]),
                state
                    .flags
                    .is_event_flag_set(&format!("EVENT_MN_DEX_REWARD_{index}"))
                    .expect("prior research claim"),
            )
        })
        .collect::<Vec<_>>();
    walk(c, r, (2, 8));
    for _ in 0..6 {
        talk(c, (2, 9));
    }
    for (index, threshold) in [10, 25, 50, 100, total].into_iter().enumerate() {
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(&format!("EVENT_MN_DEX_REWARD_{index}"))
                .expect("native research flag"),
            caught >= threshold,
            "actual caught-only milestone {threshold}"
        );
    }
    for ((threshold, item, amount), (quantity, claimed)) in awards.iter().zip(before) {
        let expected = quantity
            + if caught >= *threshold && !claimed {
                *amount
            } else {
                0
            };
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items[*item]),
            expected,
            "actual earned research delivery: {item}"
        );
    }
    let bag = c.shell.shell.session().state().bag.clone();
    continue_in_place(c, root, r, "campaign-earned-research-claims");
    talk(c, (2, 9));
    assert_eq!(
        c.shell.shell.session().state().bag,
        bag,
        "no duplicate claimed research rewards"
    );
}

#[test]
#[ignore = "requires current Minnesota pack; declared restoration, visited towns, party and ten caught; no Champion claim"]
fn earned_return_native_fly_master_ball_caught_tier_and_continue() {
    let (root, r) = crate::bevy_shell::tests::winona::load();
    let mut c =
        crate::bevy_shell::tests::interior_life::fixture(&root, &r, "MnElyPokecenter1F", (3, 5));
    {
        let state = c.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.pokemon.iter().all(Option::is_none));
        assert!(
            state.storage.party.add_pokemon(
                r.data()
                    .create_pokemon("LOOBIT", 40, Default::default())
                    .expect("declared compatible regional partner")
            )
        );
        state.sync_party_from_storage();
        assert!(
            state
                .bag
                .add_item(&r.data().items["HM_FLY"], 1)
                .expect("declared earlier machine")
        );
        state.badges.johto[4] = true;
        for flag in [
            "ENGINE_STORMBADGE",
            "ENGINE_FLYPOINT_MN_WINONA",
            "ENGINE_FLYPOINT_MN_ELY",
        ] {
            state
                .flags
                .set_engine_flag(flag, true)
                .expect("declared prior field access");
        }
        for flag in [
            "EVENT_MN_SURVEY_RESTORED",
            "EVENT_MN_STARTER_RECEIVED",
            "EVENT_MN_WINONA_BALLS_RECEIVED",
            "EVENT_MN_TAMARACK_EGG_RECEIVED",
            "EVENT_MN_HOME_INTRO",
            "EVENT_MN_TAMARACK_INTRO",
            "EVENT_MN_REED_FIRST_MET",
            "EVENT_MN_REED_WINONA_DEFEATED",
        ] {
            state
                .flags
                .set_event_flag(flag, true)
                .expect("declared earlier chapter");
        }
        state.pokedex.record_caught(&r.data().pokemon["LOOBIT"]);
        for species in r
            .data()
            .story_event_script_constants
            .global
            .keys()
            .filter_map(|key| key.strip_prefix("MN_DEX_MEMBER_"))
            .filter(|name| *name != "LOOBIT")
            .take(9)
        {
            state.pokedex.record_caught(&r.data().pokemon[species]);
        }
    }
    crate::bevy_shell::mark_runtime_snapshot_dirty(&mut c.shell);
    return_and_claim(&mut c, &r, &root);
    assert_eq!(
        c.shell.shell.session().state().hall_of_fame.count,
        0,
        "this local fixture never earns or supplies a Champion result"
    );
}
