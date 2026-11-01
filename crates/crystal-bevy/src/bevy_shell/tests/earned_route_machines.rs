//! Optional route-machine visits using earned badges and ordinary native input.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::VisibleShellController;
use super::earned_rod_visits::encounter;
use super::interior_life::{target_name, walk};
use super::minnesota_hm_progression::talk;
use super::winona::{continue_in_place, press, travel};

type Point = (i16, i16);

fn gift(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    root: &AssetRoot,
    room: &str,
    stand: Point,
    actor: Point,
    item: &str,
) {
    let outside = c
        .snapshot()
        .expect("earned route approach")
        .overworld
        .map_name;
    let entrance = r.data().maps[&outside]
        .events
        .warps
        .iter()
        .find(|warp| target_name(r, &warp.target_map_constant) == room)
        .expect("actual route entrance");
    let inside = &r.data().maps[room];
    let arrival = inside
        .events
        .warps
        .iter()
        .find(|warp| {
            warp.index
                == u16::try_from(entrance.target_warp_id).expect("native entrance warp index")
        })
        .expect("indexed room arrival");
    travel(
        c,
        GameButton::Up,
        room,
        (arrival.x as i16, arrival.y as i16),
    );
    walk(c, r, stand);
    talk(c, actor);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items[item]),
        1
    );
    continue_in_place(c, root, r, &format!("campaign-earned-{item}"));
    talk(c, actor);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items[item]),
        1,
        "native machine repeat after Continue"
    );
    let exit = inside
        .events
        .warps
        .iter()
        .find(|warp| target_name(r, &warp.target_map_constant) == outside)
        .expect("original route return");
    walk(c, r, (exit.x as i16, exit.y as i16));
    for _ in 0..8 {
        if c.snapshot().expect("machine return").overworld.map_name == outside {
            break;
        }
        press(c, GameButton::Down);
    }
    assert_eq!(
        c.snapshot()
            .expect("actual route return")
            .overworld
            .map_name,
        outside
    );
}

pub(super) fn flash(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    super::rochester::trail_with(c, r, (18, 18), |c| encounter(c, r));
    gift(c, r, root, "MnR01Cave1F", (5, 6), (5, 5), "HM_FLASH");
}

pub(super) fn cut(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    walk(c, r, (46, 18));
    travel(c, GameButton::Right, "MnR02C00R00", (0, 30));
    assert!(super::prairie_story::try_road_with(c, r, (52, 34), |c| {
        encounter(c, r)
    }));
    gift(c, r, root, "MnR02ToolShed", (6, 4), (6, 3), "HM_CUT");
    assert!(super::prairie_story::try_road_with(c, r, (0, 30), |c| {
        encounter(c, r)
    }));
    travel(c, GameButton::Left, "MnMankato", (47, 18));
    travel(c, GameButton::Left, "MnMankato", (46, 18));
}

#[test]
#[ignore = "requires current Minnesota pack; declared prior badge/location, native door/gift/Continue only"]
fn earned_route_machines_native_flash_and_cut_doors_gifts_and_continue() {
    let (root, r) = super::winona::load();
    for (room, outside, stand, actor, item, badge) in [
        ("MnR01Cave1F", "MnR01C00R00", (5, 6), (5, 5), "HM_FLASH", 0),
        ("MnR02ToolShed", "MnR02C00R00", (6, 4), (6, 3), "HM_CUT", 1),
    ] {
        let mut c = super::interior_life::fixture(&root, &r, room, stand);
        {
            let state = c.shell.shell.session_mut().state_mut();
            for index in 0..=badge {
                state.badges.johto[index] = true;
                state
                    .flags
                    .set_engine_flag(
                        crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[index],
                        true,
                    )
                    .expect("declared earlier badge");
            }
        }
        super::super::mark_runtime_snapshot_dirty(&mut c.shell);
        let exit = r.data().maps[room]
            .events
            .warps
            .iter()
            .find(|warp| target_name(&r, &warp.target_map_constant) == outside)
            .expect("native original mouth");
        walk(&mut c, &r, (exit.x as i16, exit.y as i16));
        assert_eq!(
            c.snapshot().expect("native first exit").overworld.map_name,
            outside
        );
        gift(&mut c, &r, &root, room, stand, actor, item);
    }
}
