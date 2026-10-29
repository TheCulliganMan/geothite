//! Declared research/access fixtures; rewards and Continue use native input.
use super::super::mark_runtime_snapshot_dirty;
use super::interior_life::{fixture, talk};
use super::r10_gallery::continue_save;
use super::winona::load;

fn quantity(c: &super::super::VisibleShellController, item: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&c.shell.shell.runtime().data().items[item])
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with progression rewards"]
fn progression_rewards_master_ball_requires_restoration_and_survives_continue_once() {
    let (root, r) = load();
    for restored in [false, true] {
        let mut c = fixture(&root, &r, "MnWinonaFieldStation", (5, 3));
        let state = c.shell.shell.session_mut().state_mut();
        for flag in [
            "EVENT_MN_HOME_INTRO",
            "EVENT_MN_STARTER_RECEIVED",
            "EVENT_MN_REED_FIRST_MET",
            "EVENT_MN_REED_WINONA_DEFEATED",
            "EVENT_MN_TAMARACK_EGG_RECEIVED",
        ] {
            state
                .flags
                .set_event_flag(flag, true)
                .expect("declared prior opening");
        }
        state
            .flags
            .set_event_flag("EVENT_MN_SURVEY_RESTORED", restored)
            .expect("declared chapter access");
        mark_runtime_snapshot_dirty(&mut c.shell);
        if restored {
            checkpoint(&mut c, "master-ball-before");
        }
        let pages = talk(&mut c, (5, 2));
        assert_eq!(quantity(&c, "MASTER_BALL"), u16::from(restored), "{pages}");
        if restored {
            assert!(pages.contains("MASTER BALL"), "{pages}");
            c = continue_save(root.clone(), r.clone(), &mut c);
            talk(&mut c, (5, 2));
            assert_eq!(quantity(&c, "MASTER_BALL"), 1);
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with progression rewards"]
fn progression_rewards_bike_badge_denial_real_gift_and_continue_no_duplicate() {
    let (root, r) = load();
    for earned in [false, true] {
        let mut c = fixture(&root, &r, "MnMinneapolisToolWorkshop", (2, 5));
        c.shell.shell.session_mut().state_mut().badges.johto[0] = earned;
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_engine_flag("ENGINE_ZEPHYRBADGE", earned)
            .expect("prior badge fixture");
        mark_runtime_snapshot_dirty(&mut c.shell);
        if earned {
            checkpoint(&mut c, "bicycle-before");
        }
        let pages = talk(&mut c, (2, 4));
        assert_eq!(quantity(&c, "BICYCLE"), u16::from(earned), "{pages}");
        if earned {
            assert!(pages.contains("SELECT"), "{pages}");
            c = continue_save(root.clone(), r.clone(), &mut c);
            talk(&mut c, (2, 4));
            assert_eq!(quantity(&c, "BICYCLE"), 1);
            register_and_ride(&mut c, &r);
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with progression rewards"]
fn progression_rewards_dex_counts_caught_native_species_not_seen_foreign_or_saved_mirror() {
    let (root, r) = load();
    let active: Vec<_> = r
        .data()
        .story_event_script_constants
        .global
        .keys()
        .filter_map(|k| k.strip_prefix("MN_DEX_MEMBER_"))
        .collect();
    assert!(active.len() >= 100);
    for caught in [0, 9, 10, 25, 50, 100, active.len() - 1, active.len()] {
        let mut c = fixture(&root, &r, "MnWinonaFieldStation", (2, 8));
        let state = c.shell.shell.session_mut().state_mut();
        state
            .flags
            .set_event_flag("EVENT_MN_STARTER_RECEIVED", true)
            .expect("prior starter");
        state
            .flags
            .set_event_flag("EVENT_MN_WINONA_BALLS_RECEIVED", true)
            .expect("prior supplies");
        state
            .script_runtime
            .variables
            .insert("VAR_MINNESOTA_DEX_CAUGHT".into(), "999".into());
        for (i, id) in active.iter().enumerate() {
            let species = &r.data().pokemon[*id];
            state.pokedex.record_seen(species);
            if i < caught {
                state.pokedex.record_caught(species);
            }
        }
        for id in ["CHIKORITA", "CYNDAQUIL", "TOTODILE"] {
            state.pokedex.record_caught(&r.data().pokemon[id]);
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        checkpoint(&mut c, &format!("dex-before-{caught}"));
        for _ in 0..6 {
            talk(&mut c, (2, 9));
        }
        for (threshold, item, amount) in [
            (10, "GREAT_BALL", 10),
            (25, "ITEMFINDER", 1),
            (50, "ULTRA_BALL", 10),
            (100, "LUCKY_EGG", 1),
            (active.len(), "RARE_CANDY", 10),
        ] {
            assert_eq!(
                quantity(&c, item),
                if caught >= threshold { amount } else { 0 },
                "caught={caught} item={item}"
            );
        }
        assert_eq!(quantity(&c, "EXP_SHARE"), 0);
        c = continue_save(root.clone(), r.clone(), &mut c);
        talk(&mut c, (2, 9));
        assert_eq!(quantity(&c, "EXP_SHARE"), 0);
        assert_eq!(
            quantity(&c, "GREAT_BALL"),
            if caught >= 10 { 10 } else { 0 }
        );
        assert_eq!(
            quantity(&c, "RARE_CANDY"),
            if caught == active.len() { 10 } else { 0 },
            "Continue must preserve the final reward boundary"
        );
    }
}

fn register_and_ride(
    c: &mut super::super::VisibleShellController,
    r: &crystal_runtime::CrystalRuntime,
) {
    use super::super::{FieldPackAction, FieldPackPocket, StartMenuOption};
    use super::winona::press;
    use crystal_core::input::GameButton;
    press(c, GameButton::Start);
    let snapshot = c.snapshot().expect("Start menu");
    let target = super::super::visible_start_menu_options(&c.shell, &snapshot)
        .iter()
        .position(|o| *o == StartMenuOption::Pack)
        .expect("PACK");
    for _ in 0..16 {
        let selected = c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if selected == target {
            press(c, GameButton::A);
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    for _ in 0..4 {
        if c.shell.field_pack_pocket == Some(FieldPackPocket::KeyItems) {
            break;
        }
        press(c, GameButton::Right);
    }
    assert_eq!(c.shell.field_pack_pocket, Some(FieldPackPocket::KeyItems));
    press(c, GameButton::A);
    let snapshot = c.snapshot().expect("bicycle actions");
    let actions = super::super::visible_selected_pack_item_actions(
        &snapshot,
        &c.shell,
        &FieldPackPocket::KeyItems,
        false,
    )
    .expect("key item actions");
    let target = actions
        .iter()
        .position(|a| *a == FieldPackAction::Select)
        .expect("SELECT registers bicycle");
    for _ in 0..8 {
        let selected = c
            .shell
            .field_pack_action_cursor
            .as_ref()
            .expect("action cursor")
            .option_index;
        if selected == target {
            press(c, GameButton::A);
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .registered_key_item
            .as_deref(),
        Some("BICYCLE")
    );
    for _ in 0..8 {
        press(c, GameButton::B);
    }
    super::interior_life::walk(c, r, (2, 6));
    for _ in 0..8 {
        if c.snapshot().expect("workshop door").overworld.map_name == "MnMinneapolis" {
            break;
        }
        press(c, GameButton::Down);
    }
    assert_eq!(
        c.snapshot().expect("workshop exit").overworld.map_name,
        "MnMinneapolis"
    );
    press(c, GameButton::Select);
    assert_eq!(
        c.shell.shell.session().overworld().player.mode,
        crystal_core::world::movement::MovementMode::Bike
    );
    assert_eq!(quantity(c, "BICYCLE"), 1);
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with progression rewards"]
fn progression_rewards_failed_ball_delivery_native_toss_retry_and_no_duplicate() {
    use super::winona::press;
    use crystal_core::input::GameButton;
    let (root, r) = load();
    let mut c = fixture(&root, &r, "MnWinonaFieldStation", (5, 3));
    let state = c.shell.shell.session_mut().state_mut();
    state
        .flags
        .set_event_flag("EVENT_MN_SURVEY_RESTORED", true)
        .expect("prior restoration");
    assert!(
        state
            .bag
            .add_item(&r.data().items["MASTER_BALL"], 99)
            .expect("declared capacity fixture")
    );
    // The native bag permits another stack after 99. Fill every ball slot,
    // retaining MASTER BALL as the first stack for the real TOSS input below.
    for _ in 1..crystal_core::models::bag::BALL_POCKET_CAPACITY {
        assert!(
            state
                .bag
                .add_item(&r.data().items["POKE_BALL"], 99)
                .expect("full native ball pocket")
        );
    }
    assert_eq!(
        state.bag.balls.len(),
        crystal_core::models::bag::BALL_POCKET_CAPACITY
    );
    mark_runtime_snapshot_dirty(&mut c.shell);
    let pages = talk(&mut c, (5, 2));
    assert!(pages.contains("Make room"), "{pages}");
    assert_eq!(quantity(&c, "MASTER_BALL"), 99);
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_MASTER_BALL_RECEIVED")
            .expect("claim flag")
    );
    c = continue_save(root.clone(), r.clone(), &mut c);
    native_toss_first_ball(&mut c);
    assert_eq!(quantity(&c, "MASTER_BALL"), 98);
    for _ in 0..8 {
        press(&mut c, GameButton::B);
    }
    talk(&mut c, (5, 2));
    assert_eq!(quantity(&c, "MASTER_BALL"), 99);
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_MASTER_BALL_RECEIVED")
            .expect("earned claim")
    );
    c = continue_save(root, r, &mut c);
    talk(&mut c, (5, 2));
    assert_eq!(quantity(&c, "MASTER_BALL"), 99);
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with progression rewards"]
fn progression_rewards_machine_part_recovery_and_no_replacement_after_signal() {
    let (root, r) = load();
    for signal_done in [false, true] {
        let mut c = fixture(&root, &r, "MnGrandMaraisFieldStation", (5, 4));
        let state = c.shell.shell.session_mut().state_mut();
        state
            .flags
            .set_event_flag("EVENT_MN_GRAND_MARAIS_STARTER", true)
            .expect("prior welcome");
        state
            .flags
            .set_event_flag("EVENT_MN_GRAND_MARAIS_SIGNAL", signal_done)
            .expect("prior repair fixture");
        mark_runtime_snapshot_dirty(&mut c.shell);
        talk(&mut c, (5, 3));
        assert_eq!(quantity(&c, "MACHINE_PART"), u16::from(!signal_done));
        c = continue_save(root.clone(), r.clone(), &mut c);
        talk(&mut c, (5, 3));
        assert_eq!(quantity(&c, "MACHINE_PART"), u16::from(!signal_done));
        assert_eq!(
            c.shell.shell.session().state().storage.party.filled_slots(),
            0,
            "retry must not give another starter"
        );
    }
}

fn checkpoint(c: &mut super::super::VisibleShellController, name: &str) {
    if let Some(directory) = std::env::var_os("CRYSTAL_MAP_RETAIN_SAVES_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("review save directory");
        let path = directory.join(format!("reward-{name}.crystalsave"));
        assert!(!path.exists(), "review checkpoint must be fresh");
        c.save(path).expect("production checkpoint save");
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the fifty-species Ultra Ball reward"]
fn progression_rewards_fifty_ultra_balls_native_toss_retry_and_once() {
    use super::winona::press;
    use crystal_core::input::GameButton;
    let (root, r) = load();
    let mut c = fixture(&root, &r, "MnWinonaFieldStation", (2, 8));
    {
        let state = c.shell.shell.session_mut().state_mut();
        for flag in [
            "EVENT_MN_STARTER_RECEIVED",
            "EVENT_MN_WINONA_BALLS_RECEIVED",
            "EVENT_MN_DEX_REWARD_0",
            "EVENT_MN_DEX_REWARD_1",
        ] {
            state
                .flags
                .set_event_flag(flag, true)
                .expect("declared prior opening/awards");
        }
        for name in r
            .data()
            .story_event_script_constants
            .global
            .keys()
            .filter_map(|key| key.strip_prefix("MN_DEX_MEMBER_"))
            .take(50)
        {
            state.pokedex.record_caught(&r.data().pokemon[name]);
        }
        assert!(
            state
                .bag
                .add_item(&r.data().items["ULTRA_BALL"], 90)
                .expect("partial first stack")
        );
        for _ in 1..crystal_core::models::bag::BALL_POCKET_CAPACITY {
            assert!(
                state
                    .bag
                    .add_item(&r.data().items["POKE_BALL"], 99)
                    .expect("full other stacks")
            );
        }
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // Every action below uses native input; capacity is freed through PACK.
    let bag = c.shell.shell.session().state().bag.clone();
    talk(&mut c, (2, 9));
    assert_eq!(c.shell.shell.session().state().bag, bag);
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_DEX_REWARD_2")
            .expect("failed claim")
    );
    c = continue_save(root.clone(), r.clone(), &mut c);
    native_toss_first_ball(&mut c);
    assert_eq!(quantity(&c, "ULTRA_BALL"), 89);
    for _ in 0..8 {
        press(&mut c, GameButton::B);
    }
    talk(&mut c, (2, 9));
    assert_eq!(quantity(&c, "ULTRA_BALL"), 99);
    assert_eq!(quantity(&c, "EXP_SHARE"), 0);
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_DEX_REWARD_2")
            .expect("earned supplies")
    );
    c = continue_save(root, r, &mut c);
    talk(&mut c, (2, 9));
    assert_eq!(quantity(&c, "ULTRA_BALL"), 99);
}

fn native_toss_first_ball(c: &mut super::super::VisibleShellController) {
    use super::super::{FieldPackAction, FieldPackPocket, StartMenuOption};
    use super::winona::press;
    use crystal_core::input::GameButton;
    press(c, GameButton::Start);
    let snapshot = c.snapshot().expect("Start");
    let target = super::super::visible_start_menu_options(&c.shell, &snapshot)
        .iter()
        .position(|o| *o == StartMenuOption::Pack)
        .expect("PACK");
    for _ in 0..16 {
        let selected = c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if selected == target {
            press(c, GameButton::A);
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    for _ in 0..4 {
        if c.shell.field_pack_pocket == Some(FieldPackPocket::Balls) {
            break;
        }
        press(c, GameButton::Right);
    }
    press(c, GameButton::A);
    let snapshot = c.snapshot().expect("ball actions");
    let actions = super::super::visible_selected_pack_item_actions(
        &snapshot,
        &c.shell,
        &FieldPackPocket::Balls,
        false,
    )
    .expect("ball actions");
    let target = actions
        .iter()
        .position(|a| *a == FieldPackAction::Toss)
        .expect("TOSS");
    for _ in 0..8 {
        let selected = c
            .shell
            .field_pack_action_cursor
            .as_ref()
            .expect("action cursor")
            .option_index;
        if selected == target {
            press(c, GameButton::A);
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    assert_eq!(
        c.shell
            .pack_toss
            .as_ref()
            .expect("native quantity")
            .quantity,
        1
    );
    press(c, GameButton::A);
    assert!(
        c.shell
            .pack_toss
            .as_ref()
            .expect("native confirmation")
            .confirming
    );
    press(c, GameButton::A);
}
