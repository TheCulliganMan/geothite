//! Declared chapter setups supplement the separate earned New Game course.
#[cfg(feature = "fullscreen-scaling")]
#[path = "dialogue_actor_visibility.rs"]
mod actor_visibility;
#[path = "dialogue_dispatch_copy.rs"]
mod dispatch_copy;
#[path = "dialogue_holding_fence.rs"]
mod holding_fence;
#[path = "dialogue_report_comparison.rs"]
mod report_comparison;
#[path = "dialogue_retry_fixture.rs"]
mod retry;
#[path = "dialogue_story_pacing.rs"]
mod story_pacing;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController, arm_visible_current_scene_script,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::winona::{at, continue_in_place, finish, load, press};

fn fixture(
    map: &str,
    tile: (i16, i16),
    flags: &[&str],
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: map.into(),
            tile_x: tile.0,
            tile_y: tile.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit scene fixture");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    let state = controller.shell.shell.session_mut().state_mut();
    for flag in flags {
        state
            .flags
            .set_event_flag(flag, true)
            .expect("declared chapter access");
    }
    let partner = runtime
        .data()
        .create_pokemon("KINDLIT", 35, Dv::default())
        .expect("native fixture partner");
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    (root, runtime, controller)
}

#[test]
#[ignore = "declared approach fixture; requires revised pack and the production 2D schedule"]
fn overhaul_mom_intercepts_continuous_and_diagonal_2d_input() {
    use super::super::BevyRuntimeShell;
    use bevy::prelude::{ButtonInput, KeyCode};

    for diagonal in [false, true] {
        let (_, _, controller) = fixture("MnWinonaHome1F", (9, 0), &[]);
        let mut app = super::integrated_shell_test_app(controller.shell);
        app.update();
        if diagonal {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::ArrowRight);
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowDown);
        for _ in 0..512 {
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
        }
        let shell = app
            .world_mut()
            .remove_resource::<BevyRuntimeShell>()
            .expect("2D shell");
        assert_eq!(shell.last_error, None);
        let mut controller = VisibleShellController { shell };
        at(&mut controller, "MnWinonaHome1F", (9, 1));
        let snapshot = controller.snapshot().expect("Mom intercepts held movement");
        let mom = snapshot.visible_object_runtime_tiles["MN_W_GUARDIAN"];
        assert_eq!((mom.x, mom.y), (9, 2));
        assert_eq!(
            snapshot.ui.text.expect("Mom's first page").label,
            "MnWHomeIntroText"
        );
        assert!(
            !controller
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set("EVENT_MN_HOME_INTRO")
                .expect("unfinished conversation")
        );
    }
}

#[test]
#[ignore = "regional portrait fixture; requires revised CRYSTAL_WINONA_TEST_PACK"]
fn overhaul_title_showcase_uses_the_pack_species_and_cry() {
    use super::super::{
        VisibleOakIntroMode, empty_visible_oak_intro_sequence, start_visible_oak_intro_scene,
    };

    let (_, runtime, mut controller) = fixture("MnWinonaHome2F", (3, 4), &[]);
    let mut sequence = empty_visible_oak_intro_sequence(VisibleOakIntroMode::Intro);
    sequence.scene_index = 1;
    start_visible_oak_intro_scene(&mut sequence, runtime.title_presentation_program())
        .expect("regional showcase portrait and embedded cry agree");
    assert_eq!(sequence.current_sprite.as_deref(), Some("GILLPUP"));
    assert!(
        !sequence.wooper_cry_queued,
        "cry waits for its authored text boundary"
    );
    let mut images = bevy::prelude::Assets::<bevy::prelude::Image>::default();
    let mut art = super::super::RenderedTilesetArt::default();
    let trainer = controller.snapshot().expect("native trainer").trainer;
    let frame = super::super::oak_intro_sprite_frame(
        &mut art,
        &controller.shell.asset_root,
        &sequence,
        &trainer,
        &mut images,
    )
    .unwrap_or_else(|| {
        panic!(
            "regional portrait source art: {}",
            super::super::oak_intro_art_error(&art, &sequence, &trainer)
        )
    });
    let pixels = images
        .get(&frame.handle)
        .expect("portrait image")
        .data
        .as_ref()
        .expect("native portrait pixels");
    let colors = pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| pixel[3] != 0)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        colors.len() >= 3,
        "showcase contains the actual colored creature"
    );
}

#[test]
#[ignore = "explicit entrance fixtures; requires revised CRYSTAL_WINONA_TEST_PACK"]
fn overhaul_lab_all_authored_carpet_arrivals_reach_the_same_introduction() {
    for tile in [(4, 10), (4, 11), (5, 10), (5, 11)] {
        let (root, runtime, mut controller) =
            fixture("MnWinonaFieldStation", tile, &["EVENT_MN_HOME_INTRO"]);
        arm_visible_current_scene_script(&mut controller.shell, "declared_lab_entrance")
            .expect("native arrival dispatch");
        controller.wait_frames(512).expect("aisle movement");
        at(&mut controller, "MnWinonaFieldStation", (4, 2));
        let snapshot = controller.snapshot().expect("introduction");
        assert_eq!(
            snapshot.ui.text.expect("professor text").label,
            "MnOverhaulTamarackIntroText"
        );
        let doctor = snapshot.visible_object_runtime_tiles["MN_W_TAMARACK"];
        assert_eq!((doctor.x, doctor.y), (5, 2));
        press(&mut controller, GameButton::Left);
        at(&mut controller, "MnWinonaFieldStation", (4, 2));
        finish(&mut controller);
        continue_in_place(&mut controller, &root, &runtime, "lab-entrance-complete");
        arm_visible_current_scene_script(&mut controller.shell, "declared_lab_reentry")
            .expect("repeat arrival");
        controller.wait_frames(64).expect("settle reentry");
        assert!(
            !controller
                .snapshot()
                .expect("once-only entry")
                .ui
                .text_window_open
        );
    }
}

#[test]
#[ignore = "explicit depot entrance fixtures; requires revised CRYSTAL_WINONA_TEST_PACK"]
fn overhaul_depot_all_authored_arrivals_stage_the_courier_before_battle() {
    for tile in [
        (14, 23),
        (15, 23),
        (16, 23),
        (17, 23),
        (14, 22),
        (15, 22),
        (16, 22),
        (17, 22),
        (15, 21),
        (17, 10),
    ] {
        let (_, _, mut controller) =
            fixture("MnStPaulUnionDepot", tile, &["EVENT_MN_DISPATCH_NOTE"]);
        let state = controller.shell.shell.session_mut().state_mut();
        state.badges.johto[2] = true;
        state
            .flags
            .set_engine_flag("ENGINE_PLAINBADGE", true)
            .expect("declared Current badge");
        mark_runtime_snapshot_dirty(&mut controller.shell);
        // Re-arm the production map-scene scheduler after the fixture's flags
        // are installed. All approach steps and waits are native script work.
        arm_visible_current_scene_script(&mut controller.shell, "declared_depot_entrance")
            .expect("native arrival dispatch");
        controller.wait_frames(1024).expect("courier approach");
        at(&mut controller, "MnStPaulUnionDepot", (19, 16));
        let snapshot = controller.snapshot().expect("courier introduction");
        assert!(snapshot.ui.text_window_open && snapshot.battle.is_none());
        for (actor, expected) in [
            ("MN_ST_PAUL_DEPOT_COURIER", (20, 16)),
            ("MN_NSTP_MINA", (18, 16)),
        ] {
            let tile = snapshot
                .visible_object_runtime_tiles
                .get(actor)
                .expect("visible participant");
            assert_eq!((tile.x, tile.y), expected);
        }
        assert!(
            !controller
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set("EVENT_MN_COURIER_DEFEATED")
                .expect("victory flag")
        );
        press(&mut controller, GameButton::Down);
        at(&mut controller, "MnStPaulUnionDepot", (19, 16));
    }
}

#[test]
#[ignore = "declared approach fixture; requires revised pack and the production 2D schedule"]
fn overhaul_depot_intercepts_continuous_and_diagonal_2d_input() {
    use super::super::BevyRuntimeShell;
    use bevy::prelude::{ButtonInput, KeyCode};

    for diagonal in [false, true] {
        let (_, _, mut c) = fixture("MnStPaulUnionDepot", (17, 17), &["EVENT_MN_DISPATCH_NOTE"]);
        let state = c.shell.shell.session_mut().state_mut();
        state.badges.johto[2] = true;
        state
            .flags
            .set_engine_flag("ENGINE_PLAINBADGE", true)
            .expect("declared Current badge");
        mark_runtime_snapshot_dirty(&mut c.shell);
        arm_visible_current_scene_script(&mut c.shell, "depot_coordinate_approach")
            .expect("native map entry outside interception tiles");
        c.wait_frames(64)
            .expect("entry scene returns without interception");
        at(&mut c, "MnStPaulUnionDepot", (17, 17));
        assert!(c.snapshot().expect("free approach").ui.text.is_none());
        let mut app = super::integrated_shell_test_app(c.shell);
        app.update();
        let mut buttons = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        buttons.press(KeyCode::ArrowRight);
        if diagonal {
            buttons.press(KeyCode::ArrowUp);
        }
        for _ in 0..1024 {
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
        }
        let shell = app
            .world_mut()
            .remove_resource::<BevyRuntimeShell>()
            .expect("2D shell");
        assert_eq!(shell.last_error, None);
        let mut c = VisibleShellController { shell };
        at(&mut c, "MnStPaulUnionDepot", (19, 16));
        let s = c.snapshot().expect("native courier interception");
        assert_eq!(
            s.ui.text.expect("courier page").label,
            "MnNSTPMinaInterruptText"
        );
        assert!(s.battle.is_none());
        let courier = s.visible_object_runtime_tiles["MN_ST_PAUL_DEPOT_COURIER"];
        let mina = s.visible_object_runtime_tiles["MN_NSTP_MINA"];
        assert_eq!((courier.x, courier.y), (20, 16));
        assert_eq!((mina.x, mina.y), (18, 16));
    }
}

#[test]
#[ignore = "declared full-pocket fixture; requires revised CRYSTAL_WINONA_TEST_PACK"]
fn overhaul_aide_refusal_native_toss_retry_and_repeat_preserve_reward() {
    use super::super::{FieldPackAction, FieldPackPocket, StartMenuOption};
    use crystal_core::models::bag::{BALL_POCKET_CAPACITY, MAX_ITEM_STACK};

    let (root, runtime, mut controller) = fixture(
        "MnWinonaFieldStation",
        (4, 8),
        &[
            "EVENT_MN_HOME_INTRO",
            "EVENT_MN_TAMARACK_INTRO",
            "EVENT_MN_STARTER_RECEIVED",
            "EVENT_MN_R01_PARCEL_RECEIVED",
            "EVENT_MN_REED_FIRST_MET",
            "EVENT_MN_REED_WINONA_DEFEATED",
        ],
    );
    let ball = &runtime.data().items["POKE_BALL"];
    for _ in 0..BALL_POCKET_CAPACITY {
        assert!(
            controller
                .shell
                .shell
                .session_mut()
                .state_mut()
                .bag
                .add_item(ball, MAX_ITEM_STACK)
                .expect("declared full ball pocket")
        );
    }
    mark_runtime_snapshot_dirty(&mut controller.shell);
    // All state changes below this fixture boundary use production input.
    let full = controller.shell.shell.session().state().bag.clone();
    press(&mut controller, GameButton::Down);
    controller.wait_frames(128).expect("aide approach");
    at(&mut controller, "MnWinonaFieldStation", (4, 9));
    let snapshot = controller.snapshot().expect("capacity refusal");
    let aide = snapshot.visible_object_runtime_tiles["MN_W_AIDE"];
    assert_eq!((aide.x, aide.y), (3, 9));
    assert_eq!(
        snapshot.ui.text.expect("refusal text").label,
        "MnWBallsFullText"
    );
    finish(&mut controller);
    assert_eq!(controller.shell.shell.session().state().bag, full);
    assert!(
        !controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_WINONA_BALLS_RECEIVED")
            .expect("unclaimed supplies")
    );
    continue_in_place(&mut controller, &root, &runtime, "aide-full-refusal");
    press(&mut controller, GameButton::Start);
    let snapshot = controller.snapshot().expect("Start");
    let target = super::super::visible_start_menu_options(&controller.shell, &snapshot)
        .iter()
        .position(|option| *option == StartMenuOption::Pack)
        .expect("Pack");
    for _ in 0..16 {
        let selected = controller
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if selected == target {
            press(&mut controller, GameButton::A);
            break;
        }
        press(
            &mut controller,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    for _ in 0..4 {
        if controller.shell.field_pack_pocket == Some(FieldPackPocket::Balls) {
            break;
        }
        press(&mut controller, GameButton::Right);
    }
    press(&mut controller, GameButton::A);
    let snapshot = controller.snapshot().expect("ball actions");
    let target = super::super::visible_selected_pack_item_actions(
        &snapshot,
        &controller.shell,
        &FieldPackPocket::Balls,
        false,
    )
    .expect("ball actions")
    .iter()
    .position(|action| *action == FieldPackAction::Toss)
    .expect("Toss");
    for _ in 0..8 {
        let selected = controller
            .shell
            .field_pack_action_cursor
            .as_ref()
            .expect("ball action cursor")
            .option_index;
        if selected == target {
            press(&mut controller, GameButton::A);
            break;
        }
        press(
            &mut controller,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    for _ in 0..4 {
        press(&mut controller, GameButton::Up);
    }
    assert_eq!(
        controller
            .shell
            .pack_toss
            .as_ref()
            .expect("quantity")
            .quantity,
        5
    );
    press(&mut controller, GameButton::A);
    controller.wait_frames(240).expect("native toss question");
    press(&mut controller, GameButton::A);
    for _ in 0..96 {
        controller
            .wait_frames(128)
            .expect("finish native Pack text");
        press(&mut controller, GameButton::B);
        let snapshot = controller.snapshot().expect("Pack closure");
        if controller.shell.bag_cursor.is_none()
            && controller.shell.start_menu_cursor.is_none()
            && controller.shell.field_notice.is_none()
            && snapshot.ui.menu.is_none()
            && !snapshot.ui.text_window_open
        {
            break;
        }
    }
    assert_eq!(
        controller.shell.shell.session().state().bag.quantity(ball),
        full.quantity(ball) - 5
    );
    super::authored_route_fixture::indoor_walk(&mut controller, &runtime, (3, 9));
    press(&mut controller, GameButton::Left);
    press(&mut controller, GameButton::A);
    finish(&mut controller);
    assert!(
        controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_WINONA_BALLS_RECEIVED")
            .expect("claimed supplies")
    );
    assert_eq!(
        controller.shell.shell.session().state().bag.quantity(ball),
        full.quantity(ball)
    );
    continue_in_place(&mut controller, &root, &runtime, "aide-supplies-retried");
    press(&mut controller, GameButton::A);
    finish(&mut controller);
    assert_eq!(
        controller.shell.shell.session().state().bag.quantity(ball),
        full.quantity(ball)
    );
}

#[test]
#[ignore = "declared late-story setup only; never New Game or earned-party evidence"]
fn overhaul_prepare_declared_late_story_fixture() {
    use crystal_core::models::LearnedMove;
    use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;

    let path = std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_DIALOGUE_LATE_FIXTURE").expect("explicit fixture output"),
    );
    assert!(path.to_string_lossy().contains("declared-late-story"));
    let (_, runtime, mut c) = fixture(
        "MnBrainerd",
        (32, 46),
        &[
            "EVENT_MN_ICE_CAPTIVE_RESCUED",
            "EVENT_MN_DISPATCH_NOTE",
            "EVENT_MN_DEEP_BORE_MANIFEST_READ",
            "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
            "EVENT_MN_COURIER_DEFEATED",
            "EVENT_MN_REED_WINONA_DEFEATED",
            "EVENT_MN_REED_ROCHESTER_DEFEATED",
            "EVENT_MN_REED_RELAY_DEFEATED",
            "EVENT_MN_REED_STPAUL_DEFEATED",
            "EVENT_MN_REED_WORKER_RELEASED",
        ],
    );
    let state = c.shell.shell.session_mut().state_mut();
    for (i, flag) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
        state.badges.johto[i] = i < 5;
        state
            .flags
            .set_engine_flag(flag, i < 5)
            .expect("prior fixture badge");
        state
            .flags
            .set_event_flag(&format!("EVENT_MN_GYM_{}_DEFEATED", i + 1), i < 5)
            .expect("prior fixture leader");
    }
    state.storage.party.pokemon.fill(None);
    for species in [
        "BOREALYNX",
        "VOYAGEAR",
        "FORTIGARD",
        "FLOURGEIST",
        "GOUGECREST",
        "BEETYNAMO",
    ] {
        let mut partner = runtime
            .data()
            .create_pokemon(species, 65, Dv::default())
            .expect("active Minnesota fixture partner");
        partner.moves = [
            "FLAMETHROWER",
            "ICE_BEAM",
            "THUNDERBOLT",
            "EARTHQUAKE",
            "PSYCHIC_M",
            "SHADOW_BALL",
            "HEADBUTT",
            "IRON_TAIL",
        ]
        .into_iter()
        .filter(|m| partner.species.tmhm_learnset.iter().any(|known| known == m))
        .take(4)
        .map(|name| LearnedMove {
            name: name.into(),
            current_pp: runtime.data().moves[name].pp,
            pp_ups: 0,
        })
        .collect();
        assert!(!partner.moves.is_empty());
        assert!(state.storage.party.add_pokemon(partner));
    }
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.save(path)
        .expect("save explicitly declared late-story fixture");
}

#[test]
#[ignore = "declared cumulative investigation stages; requires the revised external pack"]
fn overhaul_clinic_speakers_and_return_advice_follow_completed_actions() {
    let stages = [
        "EVENT_MN_ICE_CAPTIVE_RESCUED",
        "EVENT_MN_DEEP_BORE_MANIFEST_READ",
        "EVENT_MN_BRAINERD_REPORT",
        "EVENT_MN_HEADWATERS_REPORT",
        "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
        "EVENT_MN_DEPOT_CLEARED",
        "EVENT_MN_LIGHTHOUSE_SIGNAL",
        "EVENT_MN_SURVEY_RESTORED",
    ];
    let labels = [
        "MnRCDeliveryAfterText",
        "MnOverhaulIvyEllisSafeText",
        "MnOverhaulIvyNorthText",
        "MnOverhaulIvyBrainerdText",
        "MnOverhaulIvyMineText",
        "MnOverhaulIvyMeterText",
        "MnOverhaulIvyHarborText",
        "MnOverhaulClinicAideAwayText",
        "MnArcHabitatAfterText",
    ];
    for (stage, label) in labels.into_iter().enumerate() {
        let mut flags = vec![
            "EVENT_MN_HOME_INTRO",
            "EVENT_MN_TAMARACK_INTRO",
            "EVENT_MN_STARTER_RECEIVED",
            "EVENT_MN_R01_PARCEL_RECEIVED",
            "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
            "EVENT_MN_WINONA_BALLS_RECEIVED",
            "EVENT_MN_REED_ROCHESTER_DEFEATED",
            "EVENT_MN_REED_JOINED_ICE",
        ];
        flags.extend_from_slice(&stages[..stage]);
        let (root, r, mut c) = fixture("MnRochesterPokecenter1F", (10, 13), &flags);
        let (state, overworld) = c.shell.shell.session_mut().state_and_overworld_mut();
        r.data()
            .apply_map_setup_callbacks(state, overworld, "MnRochesterPokecenter1F", "MAPSETUP_WARP")
            .expect("initialize actors from the declared chapter flags");
        mark_runtime_snapshot_dirty(&mut c.shell);
        // The fixture changed flags after its initial map load. Reenter using
        // production warps so native object-event visibility is initialized.
        super::authored_route_fixture::indoor_walk(&mut c, &r, (10, 14));
        super::winona::travel(&mut c, GameButton::Down, "MnRochester", (15, 8));
        super::winona::travel(&mut c, GameButton::Up, "MnRochesterPokecenter1F", (10, 13));
        super::authored_route_fixture::indoor_walk(&mut c, &r, (8, 9));
        let snapshot = c.snapshot().expect("clinic actors in the same viewport");
        let present = |script| snapshot.visible_objects.iter().any(|o| o.script == script);
        assert_eq!(
            present("MN_ROCHESTER_RECIPIENT"),
            stage < 7,
            "Ivy leaves for the intake"
        );
        assert!(present("MN_RC_PATIENT"), "the aide stays at the clinic");
        assert_eq!(
            present("MN_ARC_GILLPUP"),
            stage < 8,
            "the displaced creature returns home"
        );
        let before = c.shell.shell.session().state().clone();
        let approach = if stage < 7 { (11, 9) } else { (6, 8) };
        super::authored_route_fixture::indoor_walk(&mut c, &r, approach);
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        super::northstar_dialogue::acknowledge(&mut c, "RETURN", label, "EVENT_MN_LEAGUE_CHAMPION");
        finish(&mut c);
        let after = c.shell.shell.session().state();
        assert_eq!(
            after.flags, before.flags,
            "advice cannot complete another task"
        );
        assert_eq!(after.bag, before.bag);
        assert_eq!(after.money, before.money);
        continue_in_place(
            &mut c,
            &root,
            &r,
            &format!("overhaul-clinic-return-{stage}"),
        );
    }
}

#[test]
#[ignore = "declared harbor access and completed-shutdown fixtures; requires revised pack"]
fn overhaul_harbor_refusals_and_relay_return_describe_actual_progress() {
    use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;

    for (stage, label) in [
        "MN_HARBOR_RECEIVERDeniedText",
        "MnOverhaulHarborMissingLogText",
        "MN_HARBOR_RECEIVERDoneText",
        "MnHarborSurveyReturnText",
    ]
    .into_iter()
    .enumerate()
    {
        let flags = match stage {
            0 => vec!["EVENT_MN_DEPOT_CLEARED"],
            1 => vec![],
            2 => vec!["EVENT_MN_DEPOT_CLEARED", "EVENT_MN_LIGHTHOUSE_SIGNAL"],
            _ => vec![
                "EVENT_MN_DEPOT_CLEARED",
                "EVENT_MN_LIGHTHOUSE_SIGNAL",
                "EVENT_MN_SURVEY_RESTORED",
            ],
        };
        let (root, r, mut c) = fixture("MnDuluthHarborWorkshop", (8, 9), &flags);
        let state = c.shell.shell.session_mut().state_mut();
        if stage == 0 && super::northstar_dialogue::timely_evidence(&r) {
            state
                .flags
                .set_event_flag("EVENT_MN_LIGHTHOUSE_SIGNAL", true)
                .expect("evidence already accepted; only travel preparation remains");
        }
        for (i, flag) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
            state.badges.johto[i] = stage != 0 || i < 7;
            state
                .flags
                .set_engine_flag(flag, state.badges.johto[i])
                .expect("declared harbor access");
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        let before = c.shell.shell.session().state().clone();
        super::authored_route_fixture::indoor_walk(&mut c, &r, (10, 5));
        press(&mut c, GameButton::Right);
        press(&mut c, GameButton::A);
        continue_in_place(
            &mut c,
            &root,
            &r,
            &format!("overhaul-harbor-branch-{stage}"),
        );
        super::northstar_dialogue::acknowledge(&mut c, "RETURN", label, "EVENT_MN_LEAGUE_CHAMPION");
        finish(&mut c);
        let after = c.shell.shell.session().state();
        assert_eq!(after.flags, before.flags);
        assert_eq!(after.bag, before.bag);
        assert_eq!(after.money, before.money);
    }
    let (root, r, mut c) = fixture(
        "MnBoundaryWatersSurveyShelter",
        (12, 4),
        &["EVENT_MN_SURVEY_RESTORED"],
    );
    let before = c.shell.shell.session().state().clone();
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    continue_in_place(&mut c, &root, &r, "overhaul-relay-completed-return");
    super::northstar_dialogue::acknowledge(
        &mut c,
        "RETURN",
        "MnBoundarySurveyReceiverAlreadyText",
        "EVENT_MN_LEAGUE_CHAMPION",
    );
    finish(&mut c);
    let after = c.shell.shell.session().state();
    assert_eq!(after.flags, before.flags);
    assert_eq!(after.bag, before.bag);
    assert_eq!(after.money, before.money);
}

#[test]
#[ignore = "declared clearance fixtures; requires the revised reciprocal League connection"]
fn overhaul_league_road_matches_both_border_lanes_and_requires_shutdown() {
    use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;

    for lane in 0..2 {
        for restored in [false, true] {
            let mut flags = vec!["EVENT_MN_LIGHTHOUSE_SIGNAL"];
            if restored {
                flags.push("EVENT_MN_SURVEY_RESTORED");
            }
            let (root, r, mut c) = fixture("MnBoundaryWaters", (32 + lane, 41), &flags);
            let state = c.shell.shell.session_mut().state_mut();
            state.badges.johto = [true; 8];
            for flag in &BADGE_ENGINE_FLAGS[..8] {
                state
                    .flags
                    .set_engine_flag(flag, true)
                    .expect("declared prior badge");
            }
            mark_runtime_snapshot_dirty(&mut c.shell);
            let before = c.shell.shell.session().state().clone();
            if !restored {
                for _ in 0..8 {
                    press(&mut c, GameButton::Down);
                    if c.snapshot().expect("clearance check").ui.text.is_some() {
                        break;
                    }
                }
                assert_eq!(
                    c.snapshot()
                        .expect("required shutdown")
                        .ui
                        .text
                        .expect("denial")
                        .label,
                    "MnBoundaryLeagueCheckRestoredText"
                );
                finish(&mut c);
                at(&mut c, "MnBoundaryWaters", (32 + lane, 41));
            } else {
                super::winona::travel(
                    &mut c,
                    GameButton::Down,
                    "MnBoundaryWaters",
                    (32 + lane, 44),
                );
                super::winona::travel(
                    &mut c,
                    GameButton::Down,
                    "MnBoundaryWaters",
                    (32 + lane, 47),
                );
                super::winona::travel(&mut c, GameButton::Down, "MnR13C00R00", (32 + lane, 0));
                continue_in_place(&mut c, &root, &r, &format!("overhaul-league-seam-{lane}"));
                super::winona::travel(&mut c, GameButton::Up, "MnBoundaryWaters", (32 + lane, 47));
                super::winona::travel(&mut c, GameButton::Up, "MnBoundaryWaters", (32 + lane, 42));
            }
            let after = c.shell.shell.session().state();
            for flag in r
                .data()
                .story_event_script_constants
                .global
                .keys()
                .filter(|flag| flag.starts_with("EVENT_MN_"))
            {
                assert_eq!(
                    after
                        .flags
                        .is_event_flag_set(flag)
                        .expect("resulting story flag"),
                    before
                        .flags
                        .is_event_flag_set(flag)
                        .expect("original story flag"),
                    "walking cannot grant story progress: {flag}"
                );
            }
            assert_eq!(after.badges, before.badges);
            assert_eq!(after.bag, before.bag);
            assert_eq!(after.money, before.money);
        }
    }
}
