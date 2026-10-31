//! Declared restoration fixture, then native reward, travel, throw and Continue.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::{cleanup_reached_save, indoor_walk, reached_save_path};
use super::interior_life::{collision, fixture, talk};
use super::northstar_ice_lair::{event, path, walk};
use super::winona::{load, press, travel};

const CLAIM: &str = "EVENT_MN_MASTER_BALL_RECEIVED";

fn quantity(c: &VisibleShellController, r: &CrystalRuntime) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["MASTER_BALL"])
}

fn setup() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let mut c = fixture(&root, &r, "MnWinonaFieldStation", (5, 3));
    let state = c.shell.shell.session_mut().state_mut();
    for flag in [
        "EVENT_MN_HOME_INTRO",
        "EVENT_MN_STARTER_RECEIVED",
        "EVENT_MN_REED_FIRST_MET",
        "EVENT_MN_REED_WINONA_DEFEATED",
        "EVENT_MN_TAMARACK_EGG_RECEIVED",
        "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
        "EVENT_MN_WINONA_BALLS_RECEIVED",
        "EVENT_MN_SURVEY_RESTORED",
    ] {
        state
            .flags
            .set_event_flag(flag, true)
            .expect("declared completed story");
    }
    let mut partner = r
        .data()
        .create_pokemon("POUCHID", 60, Dv::default())
        .expect("native regional partner");
    partner.item = Some("SMOKE_BALL".into());
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert_eq!(
        quantity(&c, &r),
        0,
        "the Master Ball must be earned from Tamarack"
    );
    // No later state, bag, flags, RNG, position, party or HP writes.
    (root, r, c)
}

fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    name: &str,
) -> VisibleShellController {
    let save = reached_save_path(name);
    c.save(&save)
        .expect("save the actual pending reward boundary");
    let loaded = VisibleShellController::load_save(root.clone(), r.clone(), save.clone(), None)
        .expect("Continue reward boundary");
    cleanup_reached_save(&save);
    loaded
}

fn finish(c: &mut VisibleShellController, r: &CrystalRuntime) {
    for _ in 0..256 {
        let s = c.snapshot().expect("reward presentation");
        if event(c, CLAIM)
            && !s.ui.text_window_open
            && c.shell.field_notice.is_none()
            && !c.has_pending_script_work()
        {
            assert_eq!(quantity(c, r), 1);
            return;
        }
        press(c, GameButton::A);
    }
    panic!("Master Ball reward did not complete");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with restoration rewards"]
fn legendary_master_ball_interrupted_offer_receipt_and_claim_continue_once() {
    for stage in ["offer", "receipt", "explanation"] {
        let (root, r, mut c) = setup();
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        let mut reached = false;
        for _ in 0..128 {
            let s = c.snapshot().expect("native reward boundary");
            let q = quantity(&c, &r);
            reached = match stage {
                "offer" => {
                    q == 0
                        && s.ui
                            .text
                            .as_ref()
                            .is_some_and(|t| t.label == "MnRewardMasterIntroText")
                }
                "receipt" => q == 1 && c.shell.field_notice.is_some(),
                _ => {
                    q == 1
                        && event(&c, CLAIM)
                        && s.ui
                            .text
                            .as_ref()
                            .is_some_and(|t| t.label == "MnRewardMasterGivenText")
                }
            };
            if reached {
                break;
            }
            press(&mut c, GameButton::A);
        }
        assert!(reached, "missing actual {stage} boundary");
        let before = quantity(&c, &r);
        c = continued(&mut c, &root, &r, &format!("master-pending-{stage}"));
        assert_eq!(
            quantity(&c, &r),
            before,
            "Continue may not reroll the grant"
        );
        finish(&mut c, &r);
        c = continued(&mut c, &root, &r, &format!("master-complete-{stage}"));
        talk(&mut c, (5, 2));
        assert!(event(&c, CLAIM));
        assert_eq!(
            quantity(&c, &r),
            1,
            "repeat talk cannot duplicate the reward"
        );
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with current native Winona and R01"]
fn legendary_master_ball_earned_reward_walks_to_wild_capture_and_stays_spent() {
    let (root, r, mut c) = setup();
    talk(&mut c, (5, 2));
    assert_eq!(quantity(&c, &r), 1);
    assert!(event(&c, CLAIM));
    indoor_walk(&mut c, &r, (4, 10));
    travel(&mut c, GameButton::Down, "MnWinona", (18, 20));
    indoor_walk(&mut c, &r, (4, 18));
    travel(&mut c, GameButton::Left, "MnR01C00R00", (63, 24));
    let room = &r.data().maps["MnR01C00R00"];
    let mut approaches = Vec::new();
    for y in 1..i16::try_from(room.attributes.height * 2 - 1).expect("route height") {
        for x in 1..i16::try_from(room.attributes.width * 2 - 1).expect("route width") {
            if collision(&r, room, (x, y)) != Some("FLOOR") {
                continue;
            }
            for (grass, button) in [
                ((x - 1, y), GameButton::Left),
                ((x + 1, y), GameButton::Right),
                ((x, y - 1), GameButton::Up),
                ((x, y + 1), GameButton::Down),
            ] {
                if collision(&r, room, grass) == Some("TALL_GRASS")
                    && let Some(route) = path(&mut c, &r, (x, y))
                {
                    approaches.push((route.len(), (x, y), button));
                }
            }
        }
    }
    approaches.sort_by_key(|p| p.0);
    let (_, approach, toward_grass) = approaches
        .first()
        .copied()
        .expect("reachable opening grass");
    walk(&mut c, &r, approach);
    let away = match toward_grass {
        GameButton::Left => GameButton::Right,
        GameButton::Right => GameButton::Left,
        GameButton::Up => GameButton::Down,
        _ => GameButton::Up,
    };
    let mut species = None;
    for step in 0..1024 {
        if let Some(battle) = c.snapshot().expect("ordinary field step").battle {
            species = Some(battle.enemy_pokemon.species.id);
            break;
        }
        press(&mut c, if step % 2 == 0 { toward_grass } else { away });
        c.wait_frames(17 + step % 11)
            .expect("ordinary encounter timing");
    }
    let species = species.expect("ordinary opening grass must produce a wild battle");
    assert!(super::rochester_earned::try_recruit_with_ball(
        &mut c,
        &r,
        "MASTER_BALL"
    ));
    super::winona::finish(&mut c);
    assert_eq!(quantity(&c, &r), 0);
    assert!(c.shell.shell.session().state().pokedex.has_caught(&species));
    c = continued(&mut c, &root, &r, "master-earned-capture");
    assert_eq!(quantity(&c, &r), 0);
    assert!(event(&c, CLAIM));
    assert!(c.shell.shell.session().state().pokedex.has_caught(&species));
}
