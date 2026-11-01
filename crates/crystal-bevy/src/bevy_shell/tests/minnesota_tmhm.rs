//! Explicit local fixtures; purchases, discoveries and Continue use production input.

use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::world::map::Direction;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::winona::{finish, press};

fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/minnesota-all-tmhm.crystalpack")
                .canonicalize()
                .expect("published pack path");
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("published machines"),
            )
            .expect("machine runtime");
            (root, runtime)
        })
        .clone()
}

fn fixture(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    map: &str,
    tile: [u16; 2],
    facing: Direction,
) -> VisibleShellController {
    let mut shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: map.into(),
            tile_x: tile[0] as i16,
            tile_y: tile[1] as i16,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local fixture");
    complete_visible_smoke_player_name_if_needed(&mut shell, Some("CHRIS")).expect("name");
    let state = shell.shell.session_mut().state_mut();
    state.money = 999_999;
    state.bag.tm_hm.fill(0);
    assert!(
        state.storage.party.add_pokemon(
            runtime
                .data()
                .create_pokemon("MEW", 5, Dv::default())
                .expect("compatible partner")
        )
    );
    state.sync_party_from_storage();
    shell
        .shell
        .session_mut()
        .overworld_mut()
        .set_player_facing(facing);
    mark_runtime_snapshot_dirty(&mut shell);
    VisibleShellController { shell }
}

fn continue_save(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    suffix: &str,
) -> VisibleShellController {
    let path =
        std::env::temp_dir().join(format!("tmhm-{}-{suffix}.crystalsave", std::process::id()));
    let before = c.snapshot().expect("before save");
    c.save(&path).expect("production save");
    let mut loaded =
        VisibleShellController::load_save(root.clone(), runtime.clone(), path.clone(), None)
            .expect("Continue");
    let after = loaded.snapshot().expect("continued inventory");
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.party, after.party);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    std::fs::remove_file(path).expect("remove owned save");
    loaded
}

#[test]
#[ignore = "requires the published Minnesota TM/HM distribution pack"]
fn minnesota_tmhm_controller_buys_all_57_machines_and_continues() {
    let (root, runtime) = load();
    let mut c = fixture(
        &root,
        &runtime,
        "MnMinneapolisDepartmentStore5F",
        [8, 3],
        Direction::Down,
    );
    let stock = runtime.data().marts.0["MART_MN_MP_DEPT_5_CLERK"].clone();
    let flags = c.shell.shell.session().state().flags.clone();
    let mut money = 999_999;
    press(&mut c, GameButton::A);
    for (target, id) in stock.iter().enumerate() {
        for _ in 0..32 {
            if (c.shell.shop_top_cursor.is_some() || c.shell.menu_cursor.is_some())
                && c.shell.shop_welcome_seen
                && c.shell.shop_notice.is_none()
            {
                break;
            }
            press(&mut c, GameButton::A);
        }
        assert_eq!(
            c.snapshot()
                .expect("clerk stock")
                .pending_shop
                .expect("shop")
                .inventory,
            stock
        );
        if c.shell.shop_top_cursor.is_some() {
            press(&mut c, GameButton::A);
        }
        for _ in 0..64 {
            let index = c
                .shell
                .menu_cursor
                .as_ref()
                .expect("buy cursor")
                .option_index;
            if index == target {
                break;
            }
            press(
                &mut c,
                if index < target {
                    GameButton::Down
                } else {
                    GameButton::Up
                },
            );
        }
        press(&mut c, GameButton::A);
        assert_eq!(
            c.shell
                .shop_quantity
                .as_ref()
                .expect("machine quantity")
                .item_id,
            *id
        );
        // Cancel first; no inventory or money can change before YES.
        press(&mut c, GameButton::B);
        assert_eq!(c.shell.shell.session().state().money, money);
        press(&mut c, GameButton::A);
        press(&mut c, GameButton::A);
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&runtime.data().items[id]),
            0
        );
        press(&mut c, GameButton::A);
        money -= u32::from(runtime.data().items[id].price);
        let state = c.shell.shell.session().state();
        assert_eq!(state.money, money, "paid for {id}");
        assert_eq!(
            state.bag.quantity(&runtime.data().items[id]),
            1,
            "owned {id}"
        );
        assert_eq!(state.flags, flags);
        press(&mut c, GameButton::A);
    }
    // Close the stock list, then the ordinary top menu and farewell.
    for _ in 0..96 {
        let snapshot = c.snapshot().expect("shop departure");
        if snapshot.pending_shop.is_none()
            && !snapshot.ui.text_window_open
            && !c.has_pending_script_work()
        {
            break;
        }
        let button = if snapshot.pending_shop.is_none() || c.shell.shop_notice.is_some() {
            GameButton::A
        } else if let Some(cursor) = &c.shell.shop_top_cursor {
            if cursor.option_index == 2 {
                GameButton::A
            } else {
                GameButton::Down
            }
        } else {
            GameButton::B
        };
        press(&mut c, button);
    }
    assert!(c.snapshot().expect("closed shop").pending_shop.is_none());
    let loaded = continue_save(&mut c, &root, &runtime, "all-machines");
    for id in &stock {
        assert_eq!(
            loaded
                .shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&runtime.data().items[id]),
            1
        );
    }
    assert_eq!(loaded.shell.shell.session().state().money, money);
}

#[test]
#[ignore = "requires the published Minnesota TM/HM distribution pack"]
fn minnesota_tmhm_controller_collects_all_hidden_caches_once_across_continue() {
    let (root, runtime) = load();
    let source: Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/tmhm-distribution.json"
    ))
    .expect("authored caches");
    for cache in source["hidden_caches"].as_array().expect("caches") {
        let point = |field: &str| {
            [
                cache[field][0].as_u64().expect("x") as u16,
                cache[field][1].as_u64().expect("y") as u16,
            ]
        };
        let tile = point("tile");
        let approach = point("approach");
        let facing = if tile[0] > approach[0] {
            Direction::Right
        } else if tile[0] < approach[0] {
            Direction::Left
        } else if tile[1] > approach[1] {
            Direction::Down
        } else {
            Direction::Up
        };
        let id = cache["item"].as_str().expect("TM");
        let mut c = fixture(
            &root,
            &runtime,
            cache["map"].as_str().expect("map"),
            approach,
            facing,
        );
        let flags = c.shell.shell.session().state().flags.clone();
        press(&mut c, GameButton::A);
        finish(&mut c);
        let item = &runtime.data().items[id];
        assert_eq!(
            c.shell.shell.session().state().bag.quantity(item),
            1,
            "discover {id}"
        );
        assert_eq!(c.shell.shell.session().state().money, 999_999);
        let flag = format!("EVENT_MN_CACHE_{id}");
        let state = c.shell.shell.session().state();
        assert!(
            state
                .flags
                .is_event_flag_set(&flag)
                .expect("collected flag")
        );
        let mut expected = flags;
        expected
            .set_event_flag(&flag, true)
            .expect("expected cache flag");
        assert_eq!(state.flags, expected, "only the cache is earned");
        let mut loaded = continue_save(&mut c, &root, &runtime, id);
        press(&mut loaded, GameButton::A);
        finish(&mut loaded);
        assert_eq!(
            loaded.shell.shell.session().state().bag.quantity(item),
            1,
            "no duplicate after Continue: {id}"
        );
    }
}
