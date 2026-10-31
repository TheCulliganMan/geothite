//! Local deck/partner/badge fixtures; every later action uses production input.

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_bevy::VisibleShellController;
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::world::movement::MovementMode;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

const CITY: &str = "MnDuluth";
type Point = (i16, i16);

fn load() -> (AssetRoot, CrystalRuntime) {
    let path = std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_DULUTH_CITY_CONTROLLER_PACK")
            .expect("external current harbor pack"),
    )
    .canonicalize()
    .expect("pack path");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(path).expect("verified harbor pack"),
    )
    .expect("runtime");
    (root, runtime)
}

fn press(c: &mut VisibleShellController, button: GameButton) {
    c.press(button).expect("production harbor input");
    c.wait_frames(16).expect("presentation settling");
}

fn position(c: &mut VisibleShellController) -> Point {
    let at = c.snapshot().expect("position").overworld.tile;
    (at.x, at.y)
}

fn assert_at(c: &mut VisibleShellController, map: &str, at: Point) {
    let s = c.snapshot().expect("authoritative state");
    assert_eq!(s.overworld.map_name, map);
    assert_eq!((s.overworld.tile.x, s.overworld.tile.y), at);
}

fn move_to(c: &mut VisibleShellController, button: GameButton, to: Point) {
    for _ in 0..10 {
        if position(c) == to {
            return;
        }
        press(c, button);
    }
    assert_at(c, CITY, to);
}

fn finish_text(c: &mut VisibleShellController) {
    for _ in 0..96 {
        let s = c.snapshot().expect("native dialogue");
        assert!(s.ui.pending_yes_no.is_none());
        if s.phase == RuntimeShellPhase::Overworld
            && !s.ui.text_window_open
            && !c.has_pending_script_work()
        {
            return;
        }
        press(c, GameButton::A);
    }
    panic!("fisher dialogue failed to release input");
}

fn mount(c: &mut VisibleShellController, water: (i16, i16)) {
    press(c, GameButton::Left);
    for _ in 0..32 {
        press(c, GameButton::A);
        if c.snapshot().expect("Surf prompt").overworld.mode == MovementMode::Surf {
            assert_at(c, CITY, water);
            return;
        }
    }
    panic!("native Surf prompt did not launch from the floating deck");
}

fn talk(c: &mut VisibleShellController, at: (i16, i16)) {
    press(c, GameButton::Right);
    assert_at(c, CITY, at);
    press(c, GameButton::A);
    let before = c.snapshot().expect("fisher dialogue");
    assert!(before.ui.text.is_some());
    press(c, GameButton::Down);
    assert_at(c, CITY, at);
    press(c, GameButton::A);
    let after = c.snapshot().expect("second fisher page");
    assert_ne!(before.ui.text, after.ui.text);
    finish_text(c);
}

#[test]
#[ignore = "requires the external Duluth harbor pack and CRYSTAL_DULUTH_HARBOR_SAVES"]
fn duluth_floating_decks_board_talk_surf_continue_and_retain_outer_buoys() {
    let (root, r) = load();
    let directory = std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_DULUTH_HARBOR_SAVES").expect("retained harbor saves"),
    );
    std::fs::create_dir_all(&directory).expect("save directory");
    let mut spawn = r
        .data()
        .runtime_spawn_point(r.title_new_game_spawn_identifier().expect("HOME"))
        .expect("native spawn")
        .clone();
    spawn.map_name = CITY.into();
    spawn.tile_x = 53;
    spawn.tile_y = 75;
    spawn.metatile_x = 26;
    spawn.metatile_y = 37;
    spawn.subtile_x = 1;
    spawn.subtile_y = 1;
    let (mut state, _) = r
        .data()
        .start_overworld_session_from_spawn(
            &spawn,
            &r.data()
                .audio_ids()
                .into_iter()
                .map(str::to_owned)
                .collect(),
        )
        .expect("explicit initial deck fixture");
    let mut partner = r
        .data()
        .create_pokemon("GILLPUP", 60, Dv::default())
        .expect("legal regional Surf partner");
    assert!(
        r.data().pokemon["GILLPUP"]
            .tmhm_learnset
            .iter()
            .any(|m| m == "SURF")
    );
    partner.moves = vec![LearnedMove {
        name: "SURF".into(),
        current_pp: r.data().moves["SURF"].pp,
        pp_ups: 0,
    }];
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.repel_steps_remaining = 200;
    state.active_repel_item = Some("SUPER_REPEL".into());
    state.badges.johto[3] = true;
    state
        .flags
        .set_engine_flag("ENGINE_FOGBADGE", true)
        .expect("initial Surf badge fixture");
    let initial = directory.join("initial-west-platform.crystalsave");
    assert!(!initial.exists(), "retain previous fixtures");
    r.save_game(&initial, state)
        .expect("identity-bound setup save");
    let mut c = VisibleShellController::load_save(root.clone(), r.clone(), initial, None)
        .expect("production initial Continue");
    c.set_runtime_journal_enabled(false);
    // Setup ends here. The first deck placement is not earned regional access.
    move_to(&mut c, GameButton::Right, (54, 75));
    talk(&mut c, (54, 75));
    move_to(&mut c, GameButton::Left, (52, 75));
    mount(&mut c, (51, 75));
    for (button, to) in [
        (GameButton::Down, (51, 81)),
        (GameButton::Right, (58, 81)),
        (GameButton::Right, (65, 81)),
        (GameButton::Right, (68, 81)),
    ] {
        move_to(&mut c, button, to);
    }
    assert_eq!(
        c.snapshot().expect("second deck landing").overworld.mode,
        MovementMode::Normal
    );
    move_to(&mut c, GameButton::Right, (70, 81));
    talk(&mut c, (70, 81));
    let save = directory.join("reached-east-platform.crystalsave");
    c.save(&save).expect("save actual second platform arrival");
    let mut c = VisibleShellController::load_save(root, r, save, None).expect("Continue east deck");
    assert_at(&mut c, CITY, (70, 81));
    move_to(&mut c, GameButton::Left, (68, 81));
    mount(&mut c, (67, 81));
    move_to(&mut c, GameButton::Down, (67, 88));
    move_to(&mut c, GameButton::Down, (67, 90));
    for _ in 0..3 {
        press(&mut c, GameButton::Down);
    }
    assert_eq!(
        position(&mut c),
        (67, 90),
        "outer buoy boundary remains blocking"
    );
    move_to(&mut c, GameButton::Up, (67, 83));
    move_to(&mut c, GameButton::Up, (67, 81));
    move_to(&mut c, GameButton::Left, (60, 81));
    move_to(&mut c, GameButton::Left, (53, 81));
    move_to(&mut c, GameButton::Left, (51, 81));
    move_to(&mut c, GameButton::Up, (51, 75));
    move_to(&mut c, GameButton::Right, (52, 75));
    assert_eq!(
        c.snapshot().expect("first deck return").overworld.mode,
        MovementMode::Normal
    );
    c.save(directory.join("returned-west-platform.crystalsave"))
        .expect("save actual return");
    press(&mut c, GameButton::Start);
    assert!(c.snapshot().expect("production Start").ui.menu.is_some());
}
