//! Explicit starting locations/inventory; harvesting and Continue use native input.
use std::collections::BTreeSet;
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::systems::time::{GameDate, TimeState};
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::{assert_at, load, settle};

fn point(value: &Value) -> (i16, i16) {
    (
        value[0].as_i64().expect("x") as i16,
        value[1].as_i64().expect("y") as i16,
    )
}

fn facing(tree: &Value) -> GameButton {
    let a = point(&tree["approach"]);
    let b = point(&tree["tile"]);
    match (b.0 - a.0, b.1 - a.1) {
        (1, 0) => GameButton::Right,
        (-1, 0) => GameButton::Left,
        (0, 1) => GameButton::Down,
        (0, -1) => GameButton::Up,
        other => panic!("nonadjacent tree approach {other:?}"),
    }
}

pub(super) fn fixture(tree: &Value, initial_quantity: u16) -> VisibleShellController {
    let (root, runtime) = load();
    let approach = point(&tree["approach"]);
    let mut shell = initialize_bevy_runtime_shell(
        root,
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: tree["map"].as_str().expect("tree map").into(),
            tile_x: approach.0,
            tile_y: approach.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit initial harvest approach");
    complete_visible_smoke_player_name_if_needed(&mut shell, Some("CHRIS"))
        .expect("fixture player name");
    let state = shell.shell.session_mut().state_mut();
    let partner = runtime
        .data()
        .create_pokemon("POUCHID", 30, Dv::default())
        .expect("active Minnesota partner");
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    if initial_quantity > 0 {
        assert!(
            state
                .bag
                .add_item(
                    &runtime.data().items[tree["item"].as_str().expect("item")],
                    initial_quantity
                )
                .expect("explicit initial inventory")
        );
    }
    state.time = TimeState::default();
    state
        .time
        .update_from_datetime(GameDate::new(2000, 1, 1), 12, 0, 0);
    state.kenji_break_timer = 2;
    *shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(vec![17; 262144]);
    mark_runtime_snapshot_dirty(&mut shell);
    let mut controller = VisibleShellController { shell };
    controller.set_runtime_journal_enabled(false);
    // Setup ends here. Later clock samples go through the production RTC API.
    controller
}

#[test]
#[ignore = "requires the written Minnesota fruit pack and fresh retained-save directory"]
fn minnesota_fruit_groves_all_trees_harvest_once_save_continue_and_regrow() {
    let plan: Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/fruit-groves/groves.compact-v01.json"
    ))
    .expect("explicit fruit placements");
    let directory =
        PathBuf::from(std::env::var_os("CRYSTAL_FRUIT_GROVE_SAVES").expect("immutable save epoch"));
    std::fs::create_dir_all(&directory).expect("retained save directory");
    let (root, runtime) = load();
    let mut tested_items = BTreeSet::new();
    let varieties_only = std::env::var_os("CRYSTAL_FRUIT_GROVE_VARIETIES_ONLY").is_some();
    for tree in plan["trees"].as_array().expect("trees") {
        let id = tree["id"].as_str().expect("tree identity");
        let map = tree["map"].as_str().expect("map");
        let item_id = tree["item"].as_str().expect("item");
        if varieties_only && tested_items.contains(item_id) {
            continue;
        }
        let item = &runtime.data().items[item_id];
        let flag = format!("{id}_COLLECTED");
        let mut c = fixture(tree, 0);
        c.press(facing(tree)).expect("face tree");
        c.wait_frames(16).expect("tree body settles");
        assert_at(
            &mut c,
            map,
            point(&tree["approach"]),
            "tree body blocks forward step",
        );
        let first_item = tested_items.insert(item_id);
        if first_item || map == "MnWinona" || id == "FRUITTREE_MN_GROVE_011" {
            let save = directory.join(format!("{id}-approach.crystalsave"));
            assert!(!save.exists(), "retain older approach saves");
            c.save(save).expect("native approach save for rendering");
        }
        c.press(GameButton::A).expect("harvest tree");
        settle(&mut c);
        assert_eq!(
            c.shell.shell.session().state().bag.quantity(item),
            1,
            "{id}"
        );
        assert!(
            c.shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(&flag)
                .expect("harvest flag")
        );
        c.press(GameButton::A).expect("same-day empty tree");
        settle(&mut c);
        assert_eq!(
            c.shell.shell.session().state().bag.quantity(item),
            1,
            "same day {id}"
        );
        if first_item {
            let save = directory.join(format!("{id}-harvested.crystalsave"));
            assert!(!save.exists(), "retain older harvest saves");
            c.save(&save).expect("harvest save");
            c = VisibleShellController::load_save(root.clone(), runtime.clone(), save, None)
                .expect("native Continue with regional fruit flag");
            c.press(GameButton::A).expect("continued empty tree");
            settle(&mut c);
            assert_eq!(
                c.shell.shell.session().state().bag.quantity(item),
                1,
                "Continue {id}"
            );
            c.shell
                .shell
                .update_clock_from_datetime(GameDate::new(2000, 1, 2), 12, 0, 0)
                .expect("next-day production RTC sample");
            mark_runtime_snapshot_dirty(&mut c.shell);
            assert!(
                !c.shell
                    .shell
                    .session()
                    .state()
                    .flags
                    .is_event_flag_set(&flag)
                    .expect("renewed flag")
            );
            c.press(GameButton::A).expect("harvest regrown fruit");
            settle(&mut c);
            assert_eq!(
                c.shell.shell.session().state().bag.quantity(item),
                2,
                "regrowth {id}"
            );
        }
        eprintln!("native fruit harvest passed: {id} {map} {item_id}");
    }
    assert_eq!(
        tested_items.len(),
        16,
        "all native berry/apricorn varieties"
    );
    let tree = &plan["trees"][0];
    let item = &runtime.data().items[tree["item"].as_str().expect("full-bag item")];
    let slots = match item.pocket.as_str() {
        "ITEM" => crystal_core::models::ITEM_POCKET_CAPACITY,
        "BALL" => crystal_core::models::BALL_POCKET_CAPACITY,
        _ => 1,
    };
    let full_quantity = crystal_core::models::MAX_ITEM_STACK
        * u16::try_from(slots).expect("native pocket capacity");
    let mut full = fixture(tree, full_quantity);
    assert_eq!(
        full.shell.shell.session().state().bag.quantity(item),
        full_quantity
    );
    full.press(facing(tree)).expect("face full-bag tree");
    full.press(GameButton::A).expect("attempt full-bag harvest");
    settle(&mut full);
    let id = tree["id"].as_str().expect("full-bag tree");
    assert!(
        !full
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set(&format!("{id}_COLLECTED"))
            .expect("full bag leaves harvest available")
    );
}
