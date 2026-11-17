//! Catch a native Cut partner on R02 before leaving its daytime habitat.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::super::authored_route_fixture::indoor_walk;
use super::super::super::native_city_ports::city_port;
use super::super::super::rochester::{flee, try_trail_with};
use super::super::super::rochester_earned::{checkout, try_recruit, try_recruit_without_weakening};
use super::super::super::winona::{continue_in_place, press, travel};
use super::{VisibleShellController, care, enter_road, leave_road, try_road_with};

fn has_cut_partner(c: &mut VisibleShellController) -> bool {
    c.snapshot()
        .expect("earned field team")
        .party
        .slots
        .iter()
        .any(|slot| {
            slot.pokemon
                .species
                .tmhm_learnset
                .iter()
                .any(|name| name == "CUT")
        })
}

fn buy_balls(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let item = &r.data().items["POKE_BALL"];
    let state = c.shell.shell.session().state();
    let quantity = (state.money / u32::from(item.price))
        .min(u32::from(8_u16.saturating_sub(state.bag.quantity(item)))) as u16;
    if quantity == 0 {
        return;
    }
    let (door, arrival) = city_port(r, "MnMankato", "MnMankatoMart");
    indoor_walk(c, r, door);
    travel(c, GameButton::Up, "MnMankatoMart", arrival);
    indoor_walk(c, r, (3, 3));
    press(c, GameButton::Left);
    press(c, GameButton::A);
    checkout(c, r, "POKE_BALL", quantity);
    indoor_walk(c, r, (6, 6));
    travel(c, GameButton::Down, "MnMankato", door);
}

pub(super) fn cut_partner(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    if has_cut_partner(c) {
        return;
    }
    let period = c.shell.shell.session().state().time.time_of_day;
    let grass = r.data().wild_encounters["MnR02C00R00"]
        .grass
        .as_ref()
        .expect("installed R02 grass table");
    let r02_has_partner = grass.slots(period).iter().any(|slot| {
        r.data().pokemon[&slot.species]
            .tmhm_learnset
            .iter()
            .any(|name| name == "CUT")
    });
    if !r02_has_partner {
        return_to_r01_for_cut_partner(c, r, root);
        return;
    }
    assert!(
        c.snapshot()
            .expect("room for a field partner")
            .party
            .slots
            .len()
            < 6
    );
    for outing in 0..8 {
        care(c, r);
        buy_balls(c, r);
        assert!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .balls
                .values()
                .any(|quantity| *quantity > 0),
            "earned money must buy actual capture supplies"
        );
        enter_road(c, r);
        for step in 0..192 {
            let target = if step % 2 == 0 { (36, 18) } else { (39, 18) };
            let arrived = try_road_with(c, r, target, |c| {
                let snapshot = c.snapshot().expect("actual R02 encounter");
                let battle = snapshot.battle.as_ref().expect("native wild encounter");
                if battle
                    .enemy_pokemon
                    .species
                    .tmhm_learnset
                    .iter()
                    .any(|name| name == "CUT")
                    && snapshot.bag.balls.iter().any(|slot| slot.quantity > 0)
                {
                    let _ = try_recruit(c, r);
                } else {
                    flee(c);
                }
            });
            if !arrived || has_cut_partner(c) {
                break;
            }
        }
        leave_road(c, r);
        continue_in_place(
            c,
            root,
            r,
            &format!("campaign-prairie-cut-partner-{outing}"),
        );
        if has_cut_partner(c) {
            return;
        }
    }
    panic!("ordinary R02 recruitment did not catch a native Cut learner");
}

fn return_to_r01_for_cut_partner(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    root: &AssetRoot,
) {
    let period = c.shell.shell.session().state().time.time_of_day;
    let grass = r.data().wild_encounters["MnR01C00R00"]
        .grass
        .as_ref()
        .expect("installed R01 habitat");
    assert!(
        grass
            .slots(period)
            .iter()
            .any(|slot| r.data().pokemon[&slot.species]
                .tmhm_learnset
                .iter()
                .any(|name| name == "CUT")),
        "R01 must offer an actual Cut learner at this game time"
    );
    care(c, r);
    buy_balls(c, r);
    assert!(
        c.snapshot()
            .expect("capture supplies")
            .bag
            .balls
            .iter()
            .any(|slot| slot.quantity > 0),
        "the earned party needs a carried ball"
    );
    enter_road(c, r);
    super::super::super::prairie_story::road(c, r, (59, 14));
    travel(c, GameButton::Right, "MnRochester", (4, 18));
    indoor_walk(c, r, (38, 18));
    travel(c, GameButton::Right, "MnR01C00R00", (0, 24));
    for step in 0..256 {
        let tile = if step % 2 == 0 { (54, 24) } else { (57, 24) };
        let arrived = try_trail_with(c, r, tile, |c| {
            let snapshot = c.snapshot().expect("real R01 encounter");
            let battle = snapshot.battle.as_ref().expect("wild encounter");
            if battle
                .enemy_pokemon
                .species
                .tmhm_learnset
                .iter()
                .any(|name| name == "CUT")
                && snapshot.bag.balls.iter().any(|slot| slot.quantity > 0)
            {
                // The native R01 partner is far below the earned lead's level.
                // Throwing immediately avoids fabricating HP or a weak move.
                let _ = try_recruit_without_weakening(c, r);
            } else {
                flee(c);
            }
        });
        assert!(arrived, "the real return trip ended outside R01");
        if has_cut_partner(c) {
            break;
        }
    }
    assert!(
        has_cut_partner(c),
        "ordinary R01 recruitment did not catch a Cut learner"
    );
    assert!(
        try_trail_with(c, r, (0, 24), flee),
        "the actual return to Rochester must remain playable"
    );
    travel(c, GameButton::Left, "MnRochester", (39, 18));
    indoor_walk(c, r, (4, 18));
    travel(c, GameButton::Left, "MnR02C00R00", (59, 14));
    super::super::super::prairie_story::road(c, r, (0, 30));
    travel(c, GameButton::Left, "MnMankato", (47, 18));
    travel(c, GameButton::Left, "MnMankato", (46, 18));
    continue_in_place(c, root, r, "campaign-r01-cut-partner");
}

#[test]
#[ignore = "requires an actual same-pack earned Mankato checkpoint and isolated retained saves"]
fn northstar_earned_prairie_checkpoint_keeps_or_catches_a_native_cut_partner() {
    let (root, runtime) = super::super::super::winona::load();
    let path = std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_NORTHSTAR_PRAIRIE_CUT_SAVE")
            .expect("actual two-badge Mankato checkpoint"),
    );
    let mut c = VisibleShellController::load_save(root.clone(), runtime.clone(), path, None)
        .expect("matching earned checkpoint identity");
    c.set_runtime_journal_enabled(false);
    let badges = c.shell.shell.session().state().badges.clone();
    assert_eq!(
        badges.johto,
        [true, true, false, false, false, false, false, false]
    );
    super::super::super::winona::at(&mut c, "MnMankato", (10, 12));
    let ready = has_cut_partner(&mut c);
    let before = c.snapshot().expect("actual earned field team");
    cut_partner(&mut c, &runtime, &root);
    assert!(has_cut_partner(&mut c));
    if ready {
        let after = c
            .snapshot()
            .expect("existing Cut partner needs no recruitment");
        assert_eq!(after.party, before.party);
        assert_eq!(after.bag, before.bag);
        assert_eq!(after.trainer.money, before.trainer.money);
    }
    assert_eq!(c.shell.shell.session().state().badges, badges);
    for flag in [
        "EVENT_MN_ICE_ENFORCER_DEFEATED",
        "EVENT_MN_DISPATCH_NOTE",
        "EVENT_MN_GYM_3_DEFEATED",
    ] {
        assert!(
            !super::super::event(&c, flag),
            "recruitment cannot supply {flag}"
        );
    }
}
