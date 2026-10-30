//! Earned-campaign bridges for the acted rival/lair revision; no state writes.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::authored_route_fixture::indoor_walk;
use super::super::northstar_ice_lair::{choice, event, talk, walk};
use super::super::winona::{continue_in_place, finish, press, travel};
use super::{VisibleShellController, walk_to};

pub(super) fn ellis_home(
    mut c: VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
) -> VisibleShellController {
    let physical_copy = super::super::northstar_dispatch_course::has_physical_copy(r);
    let (office_outside, _) = super::super::native_city_ports::city_port(
        r,
        "MnMinneapolis",
        "MnMinneapolisDispatchOffice",
    );
    let (home_outside, home_inside) = super::super::native_city_ports::city_port(
        r,
        "MnMinneapolis",
        "MnMinneapolisDispatcherHouse",
    );
    walk(&mut c, r, (10, 14));
    travel(&mut c, GameButton::Down, "MnMinneapolis", office_outside);
    indoor_walk(&mut c, r, home_outside);
    travel(
        &mut c,
        GameButton::Up,
        "MnMinneapolisDispatcherHouse",
        home_inside,
    );
    walk(&mut c, r, (6, 5));
    talk(&mut c, GameButton::Right);
    finish(&mut c);
    if physical_copy {
        assert!(!event(&c, "EVENT_MN_DISPATCH_NOTE"));
        super::super::northstar_dispatch_course::read_copy_and_return_home(&mut c, root, r);
        walk(&mut c, r, (6, 5));
    }
    assert!(event(&c, "EVENT_MN_DISPATCH_NOTE"));
    continue_in_place(&mut c, root, r, "campaign-arc-ellis-home");
    c
}

pub(super) fn intake(c: &mut VisibleShellController, root: &AssetRoot, r: &CrystalRuntime) {
    walk_to(c, r, (12, 4));
    talk(c, GameButton::Up);
    choice(c, false);
    finish(c);
    assert!(!event(c, "EVENT_MN_ARC_LINK_SENT") && !event(c, "EVENT_MN_SURVEY_RESTORED"));
    talk(c, GameButton::Up);
    choice(c, true);
    finish(c);
    assert!(event(c, "EVENT_MN_ARC_LINK_SENT") && !event(c, "EVENT_MN_SURVEY_RESTORED"));
    walk_to(c, r, (4, 7));
    talk(c, GameButton::Left);
    finish(c);
    walk_to(c, r, (13, 7));
    travel(c, GameButton::Down, "MnBoundaryIntake", (4, 19));
    for _ in 0..8 {
        press(c, GameButton::Up);
        if c.has_pending_script_work() {
            break;
        }
    }
    finish(c);
    walk(c, r, (14, 9));
    talk(c, GameButton::Right);
    choice(c, true);
    assert!(super::super::rochester_earned::try_battle(
        c,
        r,
        "MN_REED_BOUNDARY",
        "EVENT_MN_REED_BOUNDARY_DEFEATED"
    ));
    finish(c);
    assert!(!event(c, "EVENT_MN_SURVEY_RESTORED"));
    walk(c, r, (17, 12));
    talk(c, GameButton::Up);
    choice(c, false);
    finish(c);
    talk(c, GameButton::Up);
    choice(c, true);
    finish(c);
    assert!(event(c, "EVENT_MN_KETTLURK_RELEASED") && event(c, "EVENT_MN_SURVEY_RESTORED"));
    continue_in_place(c, root, r, "campaign-arc-current-released");
    walk(c, r, (4, 18));
    travel(
        c,
        GameButton::Right,
        "MnBoundaryWatersSurveyShelter",
        (12, 7),
    );
}

pub(super) fn recruit_party(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    use super::northstar_campaign_ice::{care, supplies};
    care(c, r);
    // Buy real balls before walking the same regional encounter strip used for
    // ordinary Minneapolis preparation. Duplicate catches are not selected.
    let item = &r.data().items["GREAT_BALL"];
    let quantity = (c.shell.shell.session().state().money / u32::from(item.price)).min(12) as u16;
    if quantity > 0 {
        indoor_walk(c, r, (73, 26));
        travel(c, GameButton::Up, "MnMinneapolisMart", (6, 5));
        indoor_walk(c, r, (3, 3));
        press(c, GameButton::Left);
        press(c, GameButton::A);
        super::super::rochester_earned::checkout(c, r, "GREAT_BALL", quantity);
        indoor_walk(c, r, (6, 6));
        travel(c, GameButton::Down, "MnMinneapolis", (73, 26));
    }
    supplies(c, r);
    for outing in 0..16 {
        if c.snapshot().expect("earned mixed party").party.slots.len() >= 4 {
            return;
        }
        care(c, r);
        indoor_walk(c, r, (40, 62));
        travel(c, GameButton::Down, "MnR03C00R00", (20, 0));
        for step in 0..64 {
            let target = if step % 2 == 0 { (16, 8) } else { (21, 8) };
            let arrived = super::river_cities::try_road_with(c, r, target, |c| {
                let s = c.snapshot().expect("regional recruitment encounter");
                let b = s.battle.as_ref().expect("wild");
                let species = &b.enemy_pokemon.species.id;
                let need_cut = !s
                    .party
                    .slots
                    .iter()
                    .any(|p| p.pokemon.species.tmhm_learnset.iter().any(|m| m == "CUT"));
                // This chapter needs a mixed battle party and a future Cut
                // user. Requiring a Surf user here made the night course
                // ignore common new partners while hunting a one-percent
                // slot, long before the player receives Surf.
                let useful = !need_cut
                    || b.enemy_pokemon
                        .species
                        .tmhm_learnset
                        .iter()
                        .any(|m| m == "CUT");
                if s.party.slots.len() < 4
                    && useful
                    && !s
                        .party
                        .slots
                        .iter()
                        .any(|p| p.pokemon.species.id == *species)
                    && s.bag.balls.iter().any(|b| b.quantity > 0)
                {
                    let _ = super::super::rochester_earned::try_recruit(c, r);
                } else {
                    super::super::rochester::flee(c);
                }
            });
            if !arrived || c.snapshot().expect("caught party").party.slots.len() >= 4 {
                break;
            }
        }
        if c.snapshot().expect("strip return").overworld.map_name == "MnR03C00R00" {
            super::river_cities::road(c, r, (20, 0));
            travel(c, GameButton::Up, "MnMinneapolis", (40, 63));
            travel(c, GameButton::Up, "MnMinneapolis", (40, 62));
        } else {
            care(c, r);
        }
        continue_in_place(c, root, r, &format!("campaign-arc-recruit-{outing}"));
    }
    assert!(
        c.snapshot()
            .expect("ordinary caught party")
            .party
            .slots
            .len()
            >= 4,
        "prepare four earned partners"
    );
}

/// Change the lead through the real Start/Party/Switch menus.
pub(in crate::bevy_shell::tests) fn switch_lead(c: &mut VisibleShellController, slot: usize) {
    use super::super::super::{
        PartyAction, StartMenuOption, has_visible_shell_direction_action, visible_party_actions,
        visible_start_menu_options,
    };
    if slot == 0 {
        return;
    }
    let wanted = c.snapshot().expect("carried partner").party.slots[slot]
        .pokemon
        .species
        .id
        .clone();
    press(c, GameButton::Start);
    let snapshot = c.snapshot().expect("native Start");
    let target = visible_start_menu_options(&c.shell, &snapshot)
        .iter()
        .position(|o| *o == StartMenuOption::Pokemon)
        .expect("Party option");
    for _ in 0..32 {
        let index = c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if index == target {
            break;
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
    press(c, GameButton::A);
    for _ in 0..32 {
        let index = c.shell.party_cursor;
        if index == slot {
            break;
        }
        press(
            c,
            if index < slot {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    press(c, GameButton::A);
    let snapshot = c.snapshot().expect("party actions");
    let target = visible_party_actions(&snapshot, &c.shell)
        .expect("native party submenu")
        .iter()
        .position(|a| matches!(a, PartyAction::Switch))
        .expect("Switch action");
    for _ in 0..32 {
        let index = c
            .shell
            .party_action_cursor
            .as_ref()
            .expect("party action cursor")
            .option_index;
        if index == target {
            break;
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
    press(c, GameButton::A);
    for _ in 0..32 {
        let index = c
            .shell
            .party_switch_cursor
            .as_ref()
            .expect("switch target")
            .option_index;
        if index == 0 {
            break;
        }
        press(c, GameButton::Up);
    }
    press(c, GameButton::A);
    for _ in 0..16 {
        if !has_visible_shell_direction_action(&mut c.shell) {
            break;
        }
        press(c, GameButton::B);
    }
    assert_eq!(
        c.snapshot().expect("actual reordered party").party.slots[0]
            .pokemon
            .species
            .id,
        wanted
    );
}

/// Evolve the caught woodpecker through earned experience for the later Cut HM.
pub(super) fn prepare_cut_carrier(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    root: &AssetRoot,
) {
    let snapshot = c.snapshot().expect("caught field partners");
    if snapshot
        .party
        .slots
        .iter()
        .any(|p| p.pokemon.species.tmhm_learnset.iter().any(|m| m == "CUT"))
    {
        return;
    }
    let slot = snapshot
        .party
        .slots
        .iter()
        .position(|p| p.pokemon.species.id == "PECKIT")
        .expect("caught future Cut carrier");
    let target = snapshot.party.slots[slot]
        .pokemon
        .level
        .saturating_add(1)
        .max(18);
    switch_lead(c, slot);
    super::northstar_campaign_ice::train(c, r, root, target);
    assert!(
        c.snapshot()
            .expect("native woodpecker evolution")
            .party
            .slots[0]
            .pokemon
            .species
            .tmhm_learnset
            .iter()
            .any(|m| m == "CUT")
    );
    switch_lead(c, slot);
    continue_in_place(c, root, r, "campaign-arc-earned-cut-carrier");
}
