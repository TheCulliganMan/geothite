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
    state
        .flags
        .set_engine_flag("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", false)
        .expect("ordinary field encounters");
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
    let save = reached_save_path(&format!("{name}.crystalsave"));
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

fn throw_until_confirmed(c: &mut VisibleShellController) {
    for _ in 0..256 {
        if c.shell.pending_standard_capture.is_some() {
            if c.shell
                .battle_messages
                .front()
                .is_some_and(|text| text.contains("Gotcha!"))
            {
                return;
            }
            press(c, GameButton::A);
            continue;
        }
        let snapshot = c.snapshot().expect("native capture input");
        let button = if !c.shell.battle_messages.is_empty()
            || (snapshot.ui.menu.is_none() && snapshot.ui.text.is_some())
            || c.shell.field_pack_action_cursor.is_some()
            || c.shell.ball_cursor.is_some()
        {
            GameButton::A
        } else if c.shell.bag_cursor.is_some() {
            GameButton::Right
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match cursor.option_index {
                0 | 1 => GameButton::Down,
                3 => GameButton::Left,
                _ => GameButton::A,
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    panic!("native carried Master Ball did not resolve a catch");
}

fn finish_catch(c: &mut VisibleShellController) {
    for _ in 0..256 {
        if c.shell.pending_name_input.is_some() {
            press(c, GameButton::Start);
            press(c, GameButton::A);
            continue;
        }
        if let Some(choice) = &c.shell.pending_name_choice {
            if choice.nickname_pages.len() > 1 {
                press(c, GameButton::A);
            } else {
                press(c, GameButton::Down);
                press(c, GameButton::A);
            }
            continue;
        }
        let snapshot = c.snapshot().expect("capture continuation");
        if snapshot.battle.is_none()
            && !c.has_pending_script_work()
            && !snapshot.ui.text_window_open
        {
            return;
        }
        press(c, GameButton::A);
    }
    panic!("capture result did not finish after Continue");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with restoration rewards"]
fn legendary_master_ball_interrupted_offer_and_claim_continue_once() {
    for stage in ["offer", "explanation"] {
        let (root, r, mut c) = setup();
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        let mut reached = false;
        for _ in 0..128 {
            let s = c.snapshot().expect("native reward boundary");
            let q = quantity(&c, &r);
            if q == 1 {
                assert!(
                    event(&c, CLAIM),
                    "delivery must commit before its next input boundary"
                );
            }
            reached = match stage {
                "offer" => {
                    q == 0
                        && s.ui
                            .text
                            .as_ref()
                            .is_some_and(|t| t.label == "MnRewardMasterIntroText")
                }
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
        let field = c.snapshot().expect("ordinary field step");
        if let Some(battle) = field.battle {
            species = Some(battle.enemy_pokemon.species.id);
            break;
        }
        let here = (field.overworld.tile.x, field.overworld.tile.y);
        press(&mut c, if here == approach { toward_grass } else { away });
        c.wait_frames(17 + step % 11)
            .expect("ordinary encounter timing");
    }
    let species = species.expect("ordinary opening grass must produce a wild battle");
    throw_until_confirmed(&mut c);
    let resolved = c
        .shell
        .pending_standard_capture
        .as_ref()
        .expect("confirmed native catch")
        .outcome
        .clone();
    assert_eq!(quantity(&c, &r), 0);
    c = continued(&mut c, &root, &r, "master-pending-wild-result");
    assert_eq!(
        c.shell
            .pending_standard_capture
            .as_ref()
            .expect("restored catch")
            .outcome,
        resolved
    );
    finish_catch(&mut c);
    assert_eq!(quantity(&c, &r), 0);
    assert!(c.shell.shell.session().state().pokedex.has_caught(&species));
    c = continued(&mut c, &root, &r, "master-earned-capture");
    assert_eq!(quantity(&c, &r), 0);
    assert!(event(&c, CLAIM));
    assert!(c.shell.shell.session().state().pokedex.has_caught(&species));
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the native Kettlurk branch"]
fn legendary_master_ball_static_pending_catch_party_pc_and_repeated_continue_once() {
    use crystal_core::models::PcBox;
    for full in [false, true] {
        let (root, r) = load();
        let mut c = fixture(&root, &r, "MnDevilsKettle", (20, 13));
        let state = c.shell.shell.session_mut().state_mut();
        for flag in ["EVENT_MN_KETTLURK_RELEASED", "EVENT_MN_SURVEY_RESTORED"] {
            state
                .flags
                .set_event_flag(flag, true)
                .expect("declared restored story");
        }
        for _ in 0..if full { 6 } else { 1 } {
            assert!(
                state.storage.party.add_pokemon(
                    r.data()
                        .create_pokemon("POUCHID", 60, Dv::default())
                        .expect("native initial party")
                )
            );
        }
        state.current_pc_box = 3;
        while state.storage.pc_boxes.len() <= 3 {
            state
                .storage
                .pc_boxes
                .push(PcBox::new(state.storage.pc_boxes.len()));
        }
        state.sync_party_from_storage();
        assert!(
            state
                .bag
                .add_item(&r.data().items["MASTER_BALL"], 1)
                .expect("explicit capture fixture ball")
        );
        let (state, overworld) = c.shell.shell.session_mut().state_and_overworld_mut();
        r.data()
            .apply_map_setup_callbacks(state, overworld, "MnDevilsKettle", "MAPSETUP_WARP")
            .expect("native initial actor callbacks");
        mark_runtime_snapshot_dirty(&mut c.shell);
        // No mutations after declared setup; throw, naming and storage use input.
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        super::northstar_ice_lair::choice(&mut c, true);
        throw_until_confirmed(&mut c);
        assert_eq!(quantity(&c, &r), 0);
        let outcome = c
            .shell
            .pending_standard_capture
            .as_ref()
            .expect("resolved static catch")
            .outcome
            .clone();
        let caught = "EVENT_MN_DEVILS_KETTLE_CAUGHT";
        assert!(!event(&c, caught));
        for retry in 0..2 {
            c = continued(
                &mut c,
                &root,
                &r,
                &format!("master-static-pending-{full}-{retry}"),
            );
            assert_eq!(quantity(&c, &r), 0);
            assert_eq!(
                c.shell
                    .pending_standard_capture
                    .as_ref()
                    .expect("same resolved catch")
                    .outcome,
                outcome
            );
            assert!(!event(&c, caught));
        }
        if !full {
            for _ in 0..16 {
                if c.shell.pending_name_input.is_some() {
                    break;
                }
                press(&mut c, GameButton::A);
            }
            assert!(c.shell.pending_name_input.is_some());
            press(&mut c, GameButton::A);
            press(&mut c, GameButton::Right);
            let input = c
                .shell
                .pending_name_input
                .as_ref()
                .expect("real typed capture name")
                .clone();
            c = continued(&mut c, &root, &r, "master-static-partial-name");
            assert_eq!(
                c.shell
                    .pending_name_input
                    .as_ref()
                    .expect("restored typed name"),
                &input
            );
        }
        finish_catch(&mut c);
        assert!(event(&c, caught));
        let state = c.shell.shell.session().state();
        assert!(state.pokedex.has_caught("KETTLURK"));
        assert_eq!(state.current_pc_box, 3);
        if full {
            assert_eq!(state.storage.party.filled_slots(), 6);
            assert_eq!(state.storage.pc_boxes[3].filled_slots(), 1);
            assert_eq!(
                state.storage.pc_boxes[3].pokemon[0]
                    .as_ref()
                    .expect("selected PC capture")
                    .species
                    .id,
                "KETTLURK"
            );
        } else {
            assert_eq!(state.storage.party.filled_slots(), 2);
            assert_eq!(state.storage.pc_boxes[3].filled_slots(), 0);
        }
        let storage = state.storage.clone();
        c = continued(&mut c, &root, &r, &format!("master-static-complete-{full}"));
        assert!(event(&c, caught));
        assert_eq!(c.shell.shell.session().state().storage, storage);
        assert_eq!(quantity(&c, &r), 0);
    }
}

#[test]
#[ignore = "requires repaired CRYSTAL_WINONA_TEST_PACK and actual CRYSTAL_LEGENDARY_LEGACY_RECEIPT_SAVE"]
fn legendary_master_ball_old_receipt_keeps_its_original_pack_identity() {
    let (root, r) = load();
    let save = std::env::var_os("CRYSTAL_LEGENDARY_LEGACY_RECEIPT_SAVE")
        .expect("native receipt autosave from the old reward");
    let result = VisibleShellController::load_save(root, r, save.into(), None);
    let error = result
        .err()
        .expect("a different compiled pack must not silently take an old save");
    assert!(
        error.chain().any(|cause| matches!(
            cause.downcast_ref::<crystal_core::save::SaveError>(),
            Some(
                crystal_core::save::SaveError::ModpackIdMismatch { .. }
                    | crystal_core::save::SaveError::ModpackHashMismatch { .. }
                    | crystal_core::save::SaveError::PackContentHashMismatch { .. }
            )
        )),
        "expected an explicit pack-identity boundary: {error:#}"
    );
}
