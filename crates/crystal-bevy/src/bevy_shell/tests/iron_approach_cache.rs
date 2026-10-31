//! Local setup is explicit; collection, retry, traversal and Continue use input.
use std::path::{Path, PathBuf};

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::state::OverworldMemory;
use crystal_core::world::map::Direction;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::{fixture, load};
use super::northstar_ice_lair::walk;
use super::r10_gallery::press;
use super::winona::{at, finish, travel};

const ROOM: &str = "MnIronDrillApproach";
const FLAG: &str = "EVENT_MN_IRON_APPROACH_ETHER";

fn output(name: &str) -> PathBuf {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_IRON_APPROACH_REVIEW_DIR")
            .expect("fresh ignored crosscut review directory"),
    )
    .join(name);
    std::fs::create_dir_all(&path).expect("crosscut evidence directory");
    path
}

fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    path: &Path,
) -> VisibleShellController {
    assert!(!path.exists(), "retain previous reached saves");
    c.save(path).expect("save actual reached crosscut state");
    let mut next =
        VisibleShellController::load_save(root.clone(), runtime.clone(), path.to_path_buf(), None)
            .expect("production crosscut Continue");
    next.set_runtime_journal_enabled(false);
    next
}

fn quantity(c: &VisibleShellController, runtime: &CrystalRuntime) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&runtime.data().items["ETHER"])
}

fn taken(c: &VisibleShellController) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(FLAG)
        .expect("native cache flag")
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the crosscut revision and CRYSTAL_IRON_APPROACH_REVIEW_DIR"]
fn iron_approach_cache_full_pocket_toss_retry_and_actual_continue() {
    let (root, runtime, _) = load();
    let out = output("cache");
    let mut c = fixture(&root, &runtime, ROOM, (4, 5));
    {
        let (state, overworld) = c.shell.shell.session_mut().state_and_overworld_mut();
        state.bag.items.clear();
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["POTION"], 1)
                .expect("tossable fixture item")
        );
        while state.bag.items.len() < crystal_core::models::bag::ITEM_POCKET_CAPACITY {
            assert!(
                state
                    .bag
                    .add_item(&runtime.data().items["BERRY"], 99)
                    .expect("full initial pocket")
            );
        }
        overworld.set_player_facing(Direction::Right);
        state.overworld = OverworldMemory::from_snapshot(&overworld.snapshot());
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // No live state writes after the initial pocket/position fixture.
    assert!(!taken(&c));
    c.press(GameButton::A)
        .expect("try actual item ball with full pocket");
    let before = c.snapshot().expect("native pickup dialogue");
    assert!(before.ui.text_window_open);
    c.press(GameButton::Down)
        .expect("dialogue owns directional input");
    assert_eq!(
        c.snapshot().expect("blocked during text").overworld.tile,
        before.overworld.tile
    );
    finish(&mut c);
    assert!(!taken(&c));
    assert_eq!(quantity(&c, &runtime), 0);
    press(&mut c, GameButton::Right);
    at(&mut c, ROOM, (4, 5));
    c = continued(
        &mut c,
        &root,
        &runtime,
        &out.join("full-pocket-reached.crystalsave"),
    );
    assert!(!taken(&c));
    assert_eq!(quantity(&c, &runtime), 0);
    super::detroit_lakes::toss_potion(&mut c);
    c.press(GameButton::A)
        .expect("retry native item ball after Toss");
    finish(&mut c);
    assert!(taken(&c));
    assert_eq!(quantity(&c, &runtime), 1);
    c = continued(
        &mut c,
        &root,
        &runtime,
        &out.join("ether-collected.crystalsave"),
    );
    assert!(taken(&c));
    assert_eq!(quantity(&c, &runtime), 1);
    press(&mut c, GameButton::Right);
    at(&mut c, ROOM, (5, 5));
    press(&mut c, GameButton::Left);
    press(&mut c, GameButton::Right);
    c.press(GameButton::A)
        .expect("no duplicate pickup after Continue");
    finish(&mut c);
    assert_eq!(quantity(&c, &runtime), 1);
    c.press(GameButton::Start)
        .expect("ordinary Start after collection");
    assert!(c.snapshot().expect("native Start").ui.menu.is_some());
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the crosscut revision and CRYSTAL_IRON_APPROACH_REVIEW_DIR"]
fn iron_approach_retains_both_real_stairs_and_each_safe_arrival() {
    let (root, runtime, _) = load();
    let out = output("stairs");
    let mut c = fixture(&root, &runtime, ROOM, (6, 21));
    {
        let state = c.shell.shell.session_mut().state_mut();
        state
            .flags
            .set_event_flag("EVENT_MN_DEPOT_FOREMAN_DEFEATED", true)
            .expect("declared prior mine-access fixture");
        for request in runtime.data().maps[ROOM].trainer_scripts.values() {
            if !request.event_flag.is_empty() && request.event_flag != "-1" {
                state
                    .flags
                    .set_event_flag(&request.event_flag, true)
                    .expect("declared prior local trainer victories");
            }
        }
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // Both original opponents/programs remain installed; this local route case
    // declares their prior wins and verifies ordinary walking and real warps.
    walk(&mut c, &runtime, (4, 5));
    walk(&mut c, &runtime, (25, 6));
    travel(&mut c, GameButton::Up, "MnIronHaulage", (4, 23));
    walk(&mut c, &runtime, (5, 23));
    travel(&mut c, GameButton::Up, ROOM, (24, 5));
    walk(&mut c, &runtime, (7, 20));
    travel(&mut c, GameButton::Down, "MnR09Cave1F", (19, 9));
    walk(&mut c, &runtime, (19, 10));
    travel(&mut c, GameButton::Down, ROOM, (6, 21));
    c = continued(
        &mut c,
        &root,
        &runtime,
        &out.join("both-mouths-reached.crystalsave"),
    );
    at(&mut c, ROOM, (6, 21));
    walk(&mut c, &runtime, (4, 5));
    assert!(
        !taken(&c),
        "crossing the room never consumes the optional cache"
    );
    let mut app = super::menu_render_test_app(c.shell);
    app.update();
    let world = app.world_mut();
    let images = world
        .remove_resource::<bevy::prelude::Assets<bevy::prelude::Image>>()
        .expect("native crosscut art");
    super::pokemon_overworld_context_canvas(world, &images)
        .save(out.join("cache-loop-native.png"))
        .expect("retain actual native view");
}
