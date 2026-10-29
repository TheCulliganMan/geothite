//! Explicit local setup; ranger, travel, wildlife and commands use real input.

#[path = "safari_capture_delivery.rs"]
mod capture_delivery;
#[path = "safari_fishing_controller.rs"]
mod fishing_controller;
#[path = "safari_postcatch_controller.rs"]
mod postcatch_controller;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::random::RuntimeDividerSource;
use crystal_core::systems::safari::{SafariAction, SafariPhase, SafariResolution, SafariSession};
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::{collision, fixture, load};
use super::r10_gallery::press;

type Point = (i16, i16);

#[test]
#[ignore = "requires enabled review pack and CRYSTAL_SAFARI_RENDER_SAVE reached menu"]
fn safari_native_compositor_draws_and_refreshes_the_saved_command_window() {
    use super::super::{
        BattleCommandMarker, BattlePortraitMarker, PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH,
    };
    use bevy::prelude::{Sprite, Transform, Vec2};
    let (root, runtime, _) = load();
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_SAFARI_RENDER_SAVE").expect("actual reached encounter save"),
    );
    let c = VisibleShellController::load_save(root, runtime, path, None).expect("native Continue");
    let mut app = super::battle_render_regression_app(c.shell);
    for _ in 0..4 {
        app.update();
    }
    let world = app.world_mut();
    assert!(
        world
            .resource::<super::super::BevyRuntimeShell>()
            .last_error
            .is_none()
    );
    let mut sprites =
        world.query_filtered::<(&Sprite, &Transform), bevy::prelude::With<BattleCommandMarker>>();
    assert_eq!(
        sprites
            .iter(world)
            .filter(|(sprite, _)| sprite.custom_size
                == Some(Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT)))
            .count(),
        1,
        "saved encounter needs one real full-LCD capture canvas"
    );
    assert_eq!(
        world
            .query_filtered::<&Sprite, bevy::prelude::With<BattlePortraitMarker>>()
            .iter(world)
            .count(),
        1,
        "Safari must retain the trainer back sprite instead of a partner Pokemon"
    );
    #[cfg(feature = "fullscreen-scaling")]
    assert!(
        super::super::fullscreen_battle_active(world.resource::<super::super::BevyRuntimeShell>()),
        "Safari must use the full battle display, not a floating field window"
    );
    let before = super::render_live_battle_canvas_for_test(world, "safari-menu-before");
    let mut c = VisibleShellController {
        shell: world
            .remove_resource::<super::super::BevyRuntimeShell>()
            .expect("controller owner"),
    };
    assert_eq!(active(&mut c).command_cursor, 3);
    press(&mut c, GameButton::Left);
    assert_eq!(active(&mut c).command_cursor, 2);
    app.insert_resource(c.shell);
    for _ in 0..4 {
        app.update();
    }
    let after = super::render_live_battle_canvas_for_test(app.world_mut(), "safari-menu-after");
    assert_ne!(
        before, after,
        "native direction input must change actual cursor pixels"
    );
    // Appearance-only render fixture: the production journey below never edits
    // authoritative state after input. Both source-art backpics must work here.
    let world = app.world_mut();
    let mut shell = world.resource_mut::<super::super::BevyRuntimeShell>();
    shell.shell.session_mut().state_mut().player_gender = super::super::PLAYER_GENDER_FEMALE;
    mark_runtime_snapshot_dirty(&mut shell);
    for _ in 0..4 {
        app.update();
    }
    let female = super::render_live_battle_canvas_for_test(app.world_mut(), "safari-menu-kris");
    assert_ne!(
        image::imageops::crop_imm(&after, 64, 192, 192, 192).to_image(),
        image::imageops::crop_imm(&female, 64, 192, 192, 192).to_image(),
        "Chris and Kris must use their distinct native back sprites"
    );
    assert_eq!(
        image::imageops::crop_imm(&after, 384, 0, 224, 224).to_image(),
        image::imageops::crop_imm(&female, 384, 0, 224, 224).to_image(),
        "changing the trainer must preserve the actual wild Pokemon's picture"
    );
    if let Some(directory) = std::env::var_os("CRYSTAL_SAFARI_BATTLE_RENDER_DIR") {
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("new battle render directory");
        for (name, image) in [
            ("chris-run.png", before),
            ("chris-rock.png", after),
            ("kris-rock.png", female),
        ] {
            let path = directory.join(name);
            assert!(!path.exists(), "retain every earlier render");
            image
                .save(path)
                .expect("source-art battle compositor image");
        }
    }
    let world = app.world_mut();
    let mut c = VisibleShellController {
        shell: world
            .remove_resource::<super::super::BevyRuntimeShell>()
            .expect("controller owner"),
    };
    run(&mut c);
    app.insert_resource(c.shell);
    for _ in 0..4 {
        app.update();
    }
    let world = app.world_mut();
    assert!(!super::super::safari_visible::battle_active(
        world.resource::<super::super::BevyRuntimeShell>()
    ));
    assert_eq!(
        world
            .query_filtered::<&Sprite, bevy::prelude::With<BattlePortraitMarker>>()
            .iter(world)
            .count(),
        0,
        "returning to exploration must remove the trainer battle sprite"
    );
}

fn publish_review_pack_if_requested() {
    let Some(base_path) = std::env::var_os("CRYSTAL_SAFARI_REVIEW_BASE") else {
        return;
    };
    let output =
        PathBuf::from(std::env::var_os("CRYSTAL_INTERIOR_PACK").expect("fresh review output"));
    assert!(
        !output.exists(),
        "review publication must preserve every previous pack"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(PathBuf::from(base_path))
        .expect("strict verified review parent");
    let source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/revisions/boundary-safari-runtime-v01/zone.json"),
    )
    .expect("authored expedition config");
    let zone = serde_json::from_str(&source).expect("typed reviewed habitat config");
    let candidate = base
        .with_safari_zone(zone)
        .expect("native Rust expedition publication");
    let mut expected = base.data().clone();
    expected.safari_zone = candidate.data().safari_zone.clone();
    assert_eq!(
        candidate.data(),
        &expected,
        "all unrelated game data remains exact"
    );
    assert_eq!(candidate.runtime_files(), base.runtime_files());
    assert_eq!(candidate.compiled_audio(), base.compiled_audio());
    candidate
        .write_preserving_storage(&output)
        .expect("external review pack");
    assert_eq!(
        crystal_assets::read_verified_compiled_game_pack(&output).expect("strict readback"),
        candidate
    );
}

fn active(c: &mut VisibleShellController) -> SafariSession {
    c.snapshot()
        .expect("native Safari snapshot")
        .safari
        .expect("active permit")
}

fn at(c: &mut VisibleShellController) -> Point {
    let tile = c.snapshot().expect("position").overworld.tile;
    (tile.x, tile.y)
}

fn save(c: &mut VisibleShellController, name: &str) {
    if let Some(folder) = std::env::var_os("CRYSTAL_SAFARI_REACHED_SAVES") {
        let path = PathBuf::from(folder).join(format!("{name}.crystalsave"));
        assert!(!path.exists(), "preserve reached saves");
        std::fs::create_dir_all(path.parent().expect("save folder")).expect("folder");
        c.save(path).expect("actual reached Safari save");
    }
}

fn continue_save(
    root: crystal_assets::AssetRoot,
    runtime: CrystalRuntime,
    c: &mut VisibleShellController,
) -> VisibleShellController {
    // This checkpoint intentionally owns input. The field-gallery helper
    // auto-presses A while settling text and would finish the very screen whose
    // Continue behavior we need to verify.
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "geothite-safari-owned-{}-{}.crystalsave",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    assert!(!path.exists(), "fresh owned-screen Continue checkpoint");
    let before = c.snapshot().expect("actual owned screen before save");
    c.save(&path).expect("native owned-screen save");
    let mut loaded = VisibleShellController::load_save(root, runtime, path.clone(), None)
        .expect("native owned-screen Continue");
    let after = loaded.snapshot().expect("actual continued screen");
    assert_eq!(before.safari, after.safari);
    assert_eq!(
        before.safari_catch_presentation,
        after.safari_catch_presentation
    );
    assert_eq!(before.party, after.party);
    assert_eq!(before.storage, after.storage);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    super::authored_route_fixture::cleanup_reached_save(&path);
    loaded
}

fn path(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    target: impl Fn(Point) -> bool,
) -> Vec<Point> {
    let snapshot = c.snapshot().expect("native path origin");
    let map = &runtime.data().maps[&snapshot.overworld.map_name];
    let start = at(c);
    let mut queue = VecDeque::from([start]);
    let mut seen = BTreeSet::from([start]);
    let mut previous = BTreeMap::new();
    let mut end = None;
    while let Some(p) = queue.pop_front() {
        if target(p) {
            end = Some(p);
            break;
        }
        for q in [
            (p.0 - 1, p.1),
            (p.0 + 1, p.1),
            (p.0, p.1 - 1),
            (p.0, p.1 + 1),
        ] {
            if seen.contains(&q)
                || !matches!(collision(runtime, map, q), Some("FLOOR" | "TALL_GRASS"))
            {
                continue;
            }
            if c.shell
                .shell
                .session()
                .overworld()
                .visible_object_at_checked(crystal_core::world::map::TilePosition::new(q.0, q.1))
                .expect("actual object collision")
                .is_some()
            {
                continue;
            }
            previous.insert(q, p);
            seen.insert(q);
            queue.push_back(q);
        }
    }
    let mut point = end.expect("reachable native walking destination");
    let mut points = vec![point];
    while point != start {
        point = previous[&point];
        points.push(point);
    }
    points.reverse();
    points
}

fn button(from: Point, to: Point) -> GameButton {
    match (to.0 - from.0, to.1 - from.1) {
        (1, 0) => GameButton::Right,
        (-1, 0) => GameButton::Left,
        (0, 1) => GameButton::Down,
        (0, -1) => GameButton::Up,
        other => panic!("nonadjacent native path {other:?}"),
    }
}

fn run(c: &mut VisibleShellController) {
    press(c, GameButton::Down);
    press(c, GameButton::Right);
    assert_eq!(active(c).command_cursor, 3);
    press(c, GameButton::A);
    assert!(
        matches!(active(c).phase,SafariPhase::Presenting{result,..} if result.action==SafariAction::Run && result.resolution==SafariResolution::Ran)
    );
    press(c, GameButton::A);
    assert_eq!(active(c).phase, SafariPhase::Exploring);
}

fn finish_catch_without_nickname(c: &mut VisibleShellController) {
    use crystal_core::systems::safari_catch_presentation::SafariCatchStage as Stage;
    for _ in 0..64 {
        let snapshot = c.snapshot().expect("native capture completion");
        let Some(saved) = snapshot.safari_catch_presentation else {
            if matches!(active(c).phase, SafariPhase::Presenting { .. }) {
                press(c, GameButton::A);
            }
            return;
        };
        match saved.stage {
            Stage::CaughtText | Stage::DexNotice | Stage::DexEntry | Stage::TransferText => {
                press(c, GameButton::A)
            }
            Stage::NicknamePrompt => {
                if c.shell
                    .pending_name_choice
                    .as_ref()
                    .expect("native choice")
                    .nickname_pages
                    .len()
                    > 1
                {
                    press(c, GameButton::A);
                } else {
                    if saved.selected_yes {
                        press(c, GameButton::Down);
                    }
                    press(c, GameButton::A);
                }
            }
            Stage::Naming => panic!("this fixture expects optional nickname policy"),
        }
    }
    panic!("native completion screens did not release capture input");
}

fn walk(c: &mut VisibleShellController, points: &[Point], flee: bool) {
    for pair in points.windows(2) {
        if matches!(active(c).phase, SafariPhase::Encounter(_)) {
            if !flee {
                return;
            }
            run(c);
        }
        for _ in 0..4 {
            if at(c) == pair[1] {
                break;
            }
            press(c, button(pair[0], pair[1]));
        }
        assert_eq!(at(c), pair[1], "committed native travel");
    }
}

fn encounter(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    if matches!(active(c).phase, SafariPhase::Encounter(_)) {
        return;
    }
    let name = c.snapshot().expect("map").overworld.map_name;
    let map = &runtime.data().maps[&name];
    let course = path(c, runtime, |p| {
        collision(runtime, map, p) == Some("TALL_GRASS")
    });
    walk(c, &course, false);
    if matches!(active(c).phase, SafariPhase::Encounter(_)) {
        return;
    }
    let grass = at(c);
    let neighbor = [
        (grass.0 - 1, grass.1),
        (grass.0 + 1, grass.1),
        (grass.0, grass.1 - 1),
        (grass.0, grass.1 + 1),
    ]
    .into_iter()
    .find(|p| {
        matches!(collision(runtime, map, *p), Some("FLOOR" | "TALL_GRASS"))
            && c.shell
                .shell
                .session()
                .overworld()
                .visible_object_at_checked(crystal_core::world::map::TilePosition::new(p.0, p.1))
                .expect("native actor")
                .is_none()
    })
    .expect("real grass approach");
    for _ in 0..100 {
        if matches!(active(c).phase, SafariPhase::Encounter(_)) {
            return;
        }
        let from = at(c);
        let to = if from == grass { neighbor } else { grass };
        press(c, button(from, to));
    }
    panic!("native grass never produced Safari wildlife");
}

fn controller_fixture() -> (
    crystal_assets::AssetRoot,
    CrystalRuntime,
    VisibleShellController,
) {
    let (root, runtime, _) = load();
    let zone = runtime.data().safari_zone.as_ref().expect("review config");
    let origin = (zone.return_to.tile.x, zone.return_to.tile.y);
    let mut c = fixture(&root, &runtime, &zone.admission_map, origin);
    let mut partner = runtime
        .data()
        .create_pokemon("GILLPUP", 35, Dv::from_non_hp(10, 10, 10, 10))
        .expect("regional partner fixture");
    partner.moves = vec![LearnedMove {
        name: "SURF".into(),
        current_pp: runtime.data().moves["SURF"].pp,
        pp_ups: 0,
    }];
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.money = 1500;
    state.badges.johto[3] = true;
    state
        .flags
        .set_engine_flag("ENGINE_FOGBADGE", true)
        .expect("coherent native Surf badge fixture");
    state.random_state = Default::default();
    *c.shell.shell.session_mut().divider_mut_for_tests() =
        RuntimeDividerSource::replay((0u32..262144).map(|n| ((n * 73 + 19) % 256) as u8));
    mark_runtime_snapshot_dirty(&mut c.shell);
    (root, runtime, c)
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with an enabled Safari review config"]
fn safari_controller_real_ranger_generated_wildlife_commands_continue_and_retirement() {
    publish_review_pack_if_requested();
    let (root, runtime, mut c) = controller_fixture();
    let zone = runtime.data().safari_zone.as_ref().expect("review config");
    let origin = (zone.return_to.tile.x, zone.return_to.tile.y);
    // All explicit setup ends here. No state or wildlife injection below.
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    assert!(
        c.snapshot().expect("offer").safari_prompt.is_some(),
        "actual ranger A opens authoritative offer"
    );
    press(&mut c, GameButton::Down);
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    assert!(
        !c.snapshot()
            .expect("saved NO")
            .safari_prompt
            .expect("offer")
            .selected_yes
    );
    press(&mut c, GameButton::A);
    assert!(c.snapshot().expect("declined").safari.is_none());
    assert_eq!(c.shell.shell.session().state().money, 1500);
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::A);
    assert_eq!(c.shell.shell.session().state().money, 1000);
    assert_eq!(active(&mut c).steps_remaining, 500);
    assert_eq!(
        c.snapshot().expect("entry").overworld.map_name,
        zone.entry.map_name
    );
    save(&mut c, "entry-court");
    encounter(&mut c, &runtime);
    let before = active(&mut c);
    let before_key =
        super::super::safari_visible::render_key(&c.snapshot().expect("capture pixels"), 7);
    let storage = c.shell.shell.session().state().storage.clone();
    assert!(
        c.snapshot().expect("capture").battle.is_none(),
        "never create an ordinary combat battle"
    );
    press(&mut c, GameButton::Start);
    press(&mut c, GameButton::B);
    press(&mut c, GameButton::Select);
    assert_eq!(
        active(&mut c),
        before,
        "Start/B cannot open a combat/party/Pack surface"
    );
    assert!(c.shell.start_menu_cursor.is_none());
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::Right);
    let selected_key =
        super::super::safari_visible::render_key(&c.snapshot().expect("selected pixels"), 7);
    assert_ne!(
        selected_key, before_key,
        "native cursor changes must invalidate the retained LCD"
    );
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    assert_eq!(active(&mut c).command_cursor, 3);
    assert_eq!(
        super::super::safari_visible::render_key(&c.snapshot().expect("continued pixels"), 7),
        selected_key
    );
    save(&mut c, "wildlife-menu");
    run(&mut c);
    assert_eq!(c.shell.shell.session().state().storage, storage);
    for action in [SafariAction::Bait, SafariAction::Rock, SafariAction::Ball] {
        encounter(&mut c, &runtime);
        // Move to the chosen cell through the same saved native cursor.
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::Left);
        if action == SafariAction::Bait {
            press(&mut c, GameButton::Right);
        }
        if action == SafariAction::Rock {
            press(&mut c, GameButton::Down);
        }
        let balls = active(&mut c).balls_remaining;
        press(&mut c, GameButton::A);
        assert!(
            matches!(active(&mut c).phase,SafariPhase::Presenting{result,..} if result.action==action)
        );
        assert_eq!(
            active(&mut c).balls_remaining,
            balls - u8::from(action == SafariAction::Ball)
        );
        let result = active(&mut c);
        press(&mut c, GameButton::Start);
        assert_eq!(
            active(&mut c),
            result,
            "pending result cannot resolve twice"
        );
        save(
            &mut c,
            match action {
                SafariAction::Bait => "bait-result",
                SafariAction::Rock => "rock-result",
                _ => "ball-result",
            },
        );
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        assert_eq!(active(&mut c), result);
        if c.snapshot()
            .expect("saved completion owner")
            .safari_catch_presentation
            .is_some()
        {
            finish_catch_without_nickname(&mut c);
        } else {
            press(&mut c, GameButton::A);
        }
        if matches!(active(&mut c).phase, SafariPhase::Encounter(_)) {
            run(&mut c);
        }
    }
    let snapshot = c.snapshot().expect("retirement origin");
    let map = &runtime.data().maps[&snapshot.overworld.map_name];
    let ranger = map
        .objects
        .iter()
        .find(|actor| zone.retire_scripts.contains(&actor.script))
        .expect("actual return ranger");
    let actor = (ranger.x as i16, ranger.y as i16);
    let course = path(&mut c, &runtime, |p| {
        (p.0 - actor.0).abs() + (p.1 - actor.1).abs() == 1
    });
    walk(&mut c, &course, true);
    if matches!(active(&mut c).phase, SafariPhase::Encounter(_)) {
        run(&mut c);
    }
    let here = at(&mut c);
    press(&mut c, button(here, actor));
    press(&mut c, GameButton::A);
    assert!(c.snapshot().expect("return offer").safari_prompt.is_some());
    press(&mut c, GameButton::A);
    assert!(c.snapshot().expect("returned").safari.is_none());
    assert_eq!(
        c.snapshot().expect("free basecamp").overworld.map_name,
        zone.return_to.map_name
    );
    assert_eq!(at(&mut c), origin);
    assert_eq!(c.shell.shell.session().state().money, 1000);
    save(&mut c, "retired-basecamp");
}
