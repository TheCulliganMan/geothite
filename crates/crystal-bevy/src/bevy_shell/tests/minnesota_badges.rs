//! Native card review uses explicit badge fixtures, never claims earned Gym wins.

use super::super::{
    Assets, BevyRuntimeShell, GameButton, Image, KeyCode, RuntimeBadgeRegion, StartMenuOption,
    VisibleShellController, VisibleTrainerCardPage, load_trainer_card_frame,
    mark_runtime_snapshot_dirty, minnesota_badges, minnesota_trainer_card,
    selected_visible_start_menu_option, trainer_card_art_key, visible_start_menu_options,
    visible_trainer_card_entries,
};
use super::{menu_render_test_app, press_key_for_runtime_hotkey_app, save_live_menu_lcd_for_test};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_runtime::CrystalRuntime;

#[test]
#[ignore = "requires CRYSTAL_RENDER_TEST_PACK pointing to a completed Minnesota pack"]
fn minnesota_badges_production_input_animation_cache_and_return() {
    let path = std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("set the external Minnesota pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack directory"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified Minnesota pack"),
    )
    .expect("Minnesota runtime");
    let mut controller = VisibleShellController::new_game(root.clone(), runtime, "BADGES", None)
        .expect("production Minnesota New Game");
    assert!(minnesota_trainer_card(
        &controller.snapshot().expect("new game")
    ));
    controller.press(GameButton::Start).expect("Start menu");
    let snapshot = controller.snapshot().expect("menu snapshot");
    let options = visible_start_menu_options(&controller.shell, &snapshot);
    let index = options
        .iter()
        .position(|option| *option == StartMenuOption::TrainerCard)
        .expect("trainer card option");
    for _ in 0..index {
        controller
            .press(GameButton::Down)
            .expect("move Start cursor");
    }
    controller.press(GameButton::A).expect("open card");
    assert!(controller.shell.trainer_card_open);
    controller.press(GameButton::Right).expect("badge page");
    let snapshot = controller.snapshot().expect("badge snapshot");
    assert_eq!(
        visible_trainer_card_entries(&snapshot, &controller.shell),
        ["MINNESOTA BADGES"]
    );
    assert_eq!(
        controller.shell.trainer_card_page,
        VisibleTrainerCardPage::JohtoBadges
    );

    let empty_key = trainer_card_art_key(&snapshot, VisibleTrainerCardPage::JohtoBadges, false, 0);
    for slot in 0..8 {
        controller
            .shell
            .shell
            .award_badge(RuntimeBadgeRegion::Johto, slot)
            .expect("explicit earned-badge visual fixture");
    }
    mark_runtime_snapshot_dirty(&mut controller.shell);
    let snapshot = controller.snapshot().expect("fixture snapshot");
    assert_eq!(
        visible_trainer_card_entries(&snapshot, &controller.shell)[1..],
        minnesota_badges::NAMES
    );
    let earned_key = trainer_card_art_key(&snapshot, VisibleTrainerCardPage::JohtoBadges, false, 0);
    assert_ne!(
        empty_key, earned_key,
        "awarded badges must invalidate cached art"
    );
    assert_eq!(
        earned_key,
        trainer_card_art_key(&snapshot, VisibleTrainerCardPage::JohtoBadges, false, 8)
    );

    // Minnesota artwork remains regional on original maps retained in its pack.
    let mut retained_map = snapshot.clone();
    retained_map.overworld.map_name = "NewBarkTown".into();
    retained_map.boot.modpack_id = "renamed-review".into();
    assert!(minnesota_trainer_card(&retained_map));
    retained_map.maps.clear();
    assert!(!minnesota_trainer_card(&retained_map));
    assert_ne!(
        earned_key,
        trainer_card_art_key(&retained_map, VisibleTrainerCardPage::JohtoBadges, false, 0)
    );

    let mut images = Assets::<Image>::default();
    for phase in 0..8 {
        let frame = load_trainer_card_frame(
            &root,
            &snapshot,
            VisibleTrainerCardPage::JohtoBadges,
            phase,
            false,
            &mut images,
        )
        .expect("production native frame");
        let image = images.get(&frame.handle).expect("frame image");
        assert_eq!((image.width(), image.height()), (160, 144));
        assert_eq!(
            image.data.as_ref().expect("CPU pixels"),
            &minnesota_badges::render(&[true; 8], phase)
        );
    }

    let mut app = menu_render_test_app(controller.shell);
    app.update();
    app.update();
    let mut phases = std::collections::HashSet::new();
    for phase in 0..8 {
        let before = app
            .world()
            .resource::<BevyRuntimeShell>()
            .trainer_card_badge_frame;
        phases.insert(before);
        save_live_menu_lcd_for_test(app.world_mut(), &format!("minnesota-badges-{phase}.png"));
        for _ in 0..8 {
            app.update();
        }
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.trainer_card_badge_frame, (before + 1) & 7);
        assert_eq!(shell.last_error, None);
    }
    assert_eq!(
        phases.len(),
        8,
        "idle production clock must play the whole loop"
    );
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowLeft);
    assert_eq!(
        app.world().resource::<BevyRuntimeShell>().trainer_card_page,
        VisibleTrainerCardPage::Info
    );
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowRight);
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyX);
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    assert!(!shell.trainer_card_open);
    assert_eq!(
        selected_visible_start_menu_option(&mut shell).expect("returned menu"),
        StartMenuOption::TrainerCard
    );
    assert_eq!(shell.last_error, None);
}
