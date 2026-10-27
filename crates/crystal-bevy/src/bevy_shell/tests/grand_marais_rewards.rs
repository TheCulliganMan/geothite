//! A deliberate full-bag fixture isolates refusal and retry through native inputs.
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::{ITEM_POCKET_ITEM, is_mail_item_id};
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};

fn finish_text(controller: &mut VisibleShellController) {
    for _ in 0..32 {
        let snapshot = controller.snapshot().expect("reward dialogue");
        if snapshot.phase == RuntimeShellPhase::Overworld
            && snapshot.ui.pending_text_wait.is_none()
            && !controller.has_pending_script_work()
        {
            return;
        }
        controller.press(GameButton::A).expect("advance text");
    }
    panic!("keeper dialogue did not release input");
}

#[test]
#[ignore = "requires an external town pack via CRYSTAL_MINNESOTA_TEST_PACK"]
fn grand_marais_keeper_retries_a_full_bag_without_losing_the_part_or_reward() {
    let path =
        PathBuf::from(std::env::var_os("CRYSTAL_MINNESOTA_TEST_PACK").expect("external town pack"));
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified town pack"),
    )
    .expect("town runtime");
    let shell = initialize_bevy_runtime_shell(
        root,
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnGrandMaraisLighthouse".into(),
            tile_x: 5,
            tile_y: 4,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("disposable keeper fixture");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("finish fixture name");
    let bag = &mut controller.shell.shell.session_mut().state_mut().bag;
    assert!(
        bag.add_item(&runtime.data().items["MACHINE_PART"], 1)
            .expect("signal part")
    );
    for (_, item) in runtime
        .data()
        .items
        .iter()
        .filter(|(id, item)| {
            item.pocket == ITEM_POCKET_ITEM && id.as_str() != "SUPER_POTION" && !is_mail_item_id(id)
        })
        .take(ITEM_POCKET_CAPACITY)
    {
        assert!(bag.add_item(item, 1).expect("full-bag fixture item"));
    }
    assert_eq!(bag.items.len(), ITEM_POCKET_CAPACITY);
    let full_bag = bag.clone();
    mark_runtime_snapshot_dirty(&mut controller.shell);
    controller.press(GameButton::Up).expect("face keeper");
    controller.press(GameButton::A).expect("offer signal part");
    let refused = controller.snapshot().expect("full bag refusal");
    assert!(
        refused
            .ui
            .text
            .as_ref()
            .and_then(|text| text.asm_text.as_deref())
            .is_some_and(|text| text.contains("PACK is full"))
    );
    finish_text(&mut controller);
    let state = controller.shell.shell.session().state();
    assert_eq!(state.bag, full_bag, "refusal consumed the part or an item");
    assert!(
        !controller
            .snapshot()
            .expect("signal remains broken")
            .progression
            .active_event_flags
            .contains("EVENT_MN_GRAND_MARAIS_SIGNAL")
    );

    // Change only the capacity fixture. The following A presses still execute
    // the complete authored keeper script through the production controller.
    let bag = &mut controller.shell.shell.session_mut().state_mut().bag;
    let first_item = bag.items.keys().next().expect("full bag item").clone();
    bag.items.remove(&first_item);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_eq!(
        controller
            .snapshot()
            .expect("free item slot")
            .bag
            .items
            .len(),
        ITEM_POCKET_CAPACITY - 1
    );
    controller.press(GameButton::Up).expect("face keeper again");
    controller.press(GameButton::A).expect("retry delivery");
    finish_text(&mut controller);
    let delivered = controller.snapshot().expect("delivery accepted");
    assert!(
        delivered
            .progression
            .active_event_flags
            .contains("EVENT_MN_GRAND_MARAIS_SIGNAL"),
        "retry did not finish: {:?}",
        delivered.ui.text
    );
    let bag = &controller.shell.shell.session().state().bag;
    assert!(bag.key_items.get("MACHINE_PART").is_none());
    assert_eq!(bag.items.get("SUPER_POTION"), Some(&1));
    controller
        .press(GameButton::Up)
        .expect("face keeper once more");
    controller
        .press(GameButton::A)
        .expect("repeat conversation");
    assert!(
        controller
            .snapshot()
            .expect("repeat keeper dialogue")
            .ui
            .text
            .as_ref()
            .is_some_and(|text| text.label == "MnGrandMaraisSignalDoneText")
    );
    finish_text(&mut controller);
    assert_eq!(
        controller
            .shell
            .shell
            .session()
            .state()
            .bag
            .items
            .get("SUPER_POTION"),
        Some(&1),
        "repeated delivery duplicated the reward"
    );
}
