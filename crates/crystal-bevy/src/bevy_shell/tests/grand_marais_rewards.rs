//! A full bag defers the thank-you gift without delaying the lighthouse repair.
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

fn finish_text(controller: &mut VisibleShellController, label: &str) -> String {
    let expected = super::northstar_dialogue::canonical_pages("", label).join("\n") + "\n";
    let mut pages = String::new();
    for _ in 0..32 {
        let snapshot = controller.snapshot().expect("reward dialogue");
        if let Some(text) = snapshot
            .ui
            .text
            .as_ref()
            .and_then(|text| text.asm_text.as_deref())
        {
            pages.push_str(text);
            pages.push('\n');
        }
        if snapshot.phase == RuntimeShellPhase::Overworld
            && snapshot.ui.pending_text_wait.is_none()
            && !controller.has_pending_script_work()
        {
            assert_eq!(pages, expected, "displayed keeper pages for {label}");
            return pages;
        }
        controller.press(GameButton::A).expect("advance text");
    }
    panic!("keeper dialogue did not release input");
}

fn keeper_fixture() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path =
        PathBuf::from(std::env::var_os("CRYSTAL_MINNESOTA_TEST_PACK").expect("external town pack"));
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified town pack"),
    )
    .expect("town runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
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
    (root, runtime, controller)
}

#[test]
#[ignore = "requires an external town pack via CRYSTAL_MINNESOTA_TEST_PACK"]
fn grand_marais_keeper_accepts_the_part_with_room_and_does_not_reward_old_completion() {
    for already_completed in [false, true] {
        let (root, runtime, mut controller) = keeper_fixture();
        let state = controller.shell.shell.session_mut().state_mut();
        // Separate initial fixtures: a new delivery, or a completion from the
        // old quest where receiving the potion was required for repair.
        if already_completed {
            state
                .flags
                .set_event_flag("EVENT_MN_GRAND_MARAIS_SIGNAL", true)
                .expect("old completed repair");
        }
        assert!(
            state
                .bag
                .add_item(
                    &runtime.data().items[if already_completed {
                        "SUPER_POTION"
                    } else {
                        "MACHINE_PART"
                    }],
                    1
                )
                .expect("initial delivery or old reward")
        );
        mark_runtime_snapshot_dirty(&mut controller.shell);
        controller.press(GameButton::Up).expect("face keeper");
        controller
            .press(GameButton::A)
            .expect("keeper conversation");
        let label = if already_completed {
            "MnGrandMaraisSignalDoneText"
        } else {
            "MnGrandMaraisSignalFixedText"
        };
        assert_eq!(
            controller
                .snapshot()
                .expect("delivery reply")
                .ui
                .text
                .expect("keeper text")
                .label,
            label
        );
        let _ = finish_text(&mut controller, label);
        let state = controller.shell.shell.session().state();
        assert!(
            state
                .flags
                .is_event_flag_set("EVENT_MN_GRAND_MARAIS_SIGNAL")
                .expect("repaired signal")
        );
        assert!(
            !state
                .flags
                .is_event_flag_set("EVENT_MN_GRAND_MARAIS_SIGNAL_REWARD_PENDING")
                .expect("no held gift")
        );
        assert!(state.bag.key_items.get("MACHINE_PART").is_none());
        assert_eq!(state.bag.items.get("SUPER_POTION"), Some(&1));
        controller = super::r10_gallery::continue_save(root, runtime, &mut controller);
        controller
            .press(GameButton::Up)
            .expect("face keeper after Continue");
        controller
            .press(GameButton::A)
            .expect("repeat completed repair");
        let _ = finish_text(&mut controller, "MnGrandMaraisSignalDoneText");
        assert_eq!(
            controller
                .shell
                .shell
                .session()
                .state()
                .bag
                .items
                .get("SUPER_POTION"),
            Some(&1)
        );
    }
}

#[test]
#[ignore = "requires an external town pack via CRYSTAL_MINNESOTA_TEST_PACK"]
fn grand_marais_keeper_repairs_with_a_full_bag_and_holds_the_reward() {
    let (root, runtime, mut controller) = keeper_fixture();
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
    // No state, inventory, party, flags, position or RNG writes after setup.
    controller.press(GameButton::Up).expect("face keeper");
    controller.press(GameButton::A).expect("offer signal part");
    let reply = finish_text(&mut controller, "MnGrandMaraisSignalBagFullText");
    assert!(
        reply.replace('\n', " ").contains("PACK is full"),
        "actual repair pages: {reply}"
    );
    let state = controller.shell.shell.session().state();
    assert_eq!(
        state.bag.items, full_bag.items,
        "repair changed the full item pocket"
    );
    assert!(state.bag.key_items.get("MACHINE_PART").is_none());
    assert!(
        state
            .flags
            .is_event_flag_set("EVENT_MN_GRAND_MARAIS_SIGNAL_REWARD_PENDING")
            .expect("pending gift")
    );
    let repaired_bag = state.bag.clone();
    assert!(
        controller
            .snapshot()
            .expect("signal repaired despite a full bag")
            .progression
            .active_event_flags
            .contains("EVENT_MN_GRAND_MARAIS_SIGNAL")
    );

    controller = super::r10_gallery::continue_save(root.clone(), runtime.clone(), &mut controller);
    assert_eq!(controller.shell.shell.session().state().bag, repaired_bag);
    controller
        .press(GameButton::Up)
        .expect("face keeper while still full");
    controller
        .press(GameButton::A)
        .expect("ask about held gift");
    let _ = finish_text(&mut controller, "MnGrandMaraisSignalRewardFullText");
    assert_eq!(controller.shell.shell.session().state().bag, repaired_bag);
    super::northstar_progression_rewards::native_toss_first_pocket_item(
        &mut controller,
        &super::super::FieldPackPocket::Items,
    );
    for _ in 0..8 {
        super::winona::press(&mut controller, GameButton::B);
    }
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
    controller
        .press(GameButton::A)
        .expect("collect held thank-you gift");
    assert!(
        controller
            .snapshot()
            .expect("held reward reply")
            .ui
            .text
            .as_ref()
            .is_some_and(|text| text.label == "MnGrandMaraisSignalRewardText")
    );
    let _ = finish_text(&mut controller, "MnGrandMaraisSignalRewardText");
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
    assert!(
        !controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_GRAND_MARAIS_SIGNAL_REWARD_PENDING")
            .expect("gift collected")
    );
    controller = super::r10_gallery::continue_save(root.clone(), runtime.clone(), &mut controller);
    assert!(
        controller
            .snapshot()
            .expect("continued signal repair")
            .progression
            .active_event_flags
            .contains("EVENT_MN_GRAND_MARAIS_SIGNAL")
    );
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
    let _ = finish_text(&mut controller, "MnGrandMaraisSignalDoneText");
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
