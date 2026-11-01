//! Declared local fixtures, followed only by production input and Save/Continue.
#[path = "driftless_fossil_gallery.rs"]
mod fossil_gallery;

#[path = "gunflint_grotto.rs"]
mod gunflint_grotto;

#[path = "leech_lake_island.rs"]
mod leech_island;

#[path = "minnesota_field_teaching.rs"]
mod teaching;

use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::CrystalRuntime;

use super::super::{
    PartyAction, PartyFieldMove, StartMenuOption, VisibleShellController,
    mark_runtime_snapshot_dirty, visible_party_actions, visible_start_menu_options,
};
use super::interior_life::fixture;
use super::r10_gallery::continue_save;
use super::winona::{at, load, press};

const ROOM: &str = "MnDriftlessLampGallery";
const READ: &str = "EVENT_MN_LAMP_GALLERY_READ";
const CACHE: &str = "EVENT_MN_LAMP_GALLERY_CACHE";

fn retain(c: &mut VisibleShellController, name: &str) {
    let path = super::authored_route_fixture::reached_save_path(name);
    assert!(
        !path.exists(),
        "retain each reached visual state without replacement"
    );
    c.save(&path).expect("supported native reached-state save");
}

fn event(c: &VisibleShellController, flag: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(flag)
        .expect("authored field event")
}

fn finish(c: &mut VisibleShellController) {
    for _ in 0..128 {
        c.wait_frames(64).expect("native field presentation");
        let s = c.snapshot().expect("field dialogue");
        assert!(s.battle.is_none(), "resolve battles explicitly");
        assert!(s.ui.pending_yes_no.is_none(), "answer choices explicitly");
        if !s.ui.text_window_open
            && c.shell.field_notice.is_none()
            && c.shell.visible_flash_animation.is_none()
            && !c.shell.visible_wait_sfx_boundary
            && c.shell.wait_play_sfx_completion.is_none()
            && c.shell.pending_field_notice_effect_frames.is_none()
            && !c.shell.pending_whirlpool_sound_wait
            && !c.has_pending_script_work()
        {
            if super::super::has_visible_shell_direction_action(&mut c.shell) {
                press(c, GameButton::B);
                continue;
            }
            return;
        }
        press(c, GameButton::A);
    }
    let failed_ui = c.snapshot().expect("failed field presentation").ui;
    panic!(
        "field dialogue did not release input: flash={:?}, wait_sfx={}, completion={:?}, effect={:?}, notice={:?}, reveal={:?}, rendered={:?}, ui={:?}, audio={:?}",
        c.shell.visible_flash_animation,
        c.shell.visible_wait_sfx_boundary,
        c.shell.wait_play_sfx_completion,
        c.shell.pending_field_notice_effect_frames,
        c.shell.field_notice,
        c.shell.field_text_reveal,
        c.shell.rendered_field_text_identity,
        failed_ui,
        c.shell
            .last_audio_events
            .iter()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
    );
}

fn walk(c: &mut VisibleShellController, r: &CrystalRuntime, to: (i16, i16)) {
    for _ in 0..512 {
        if c.snapshot().expect("native travel").battle.is_some() {
            super::rochester::flee(c);
        }
        let path = super::northstar_ice_lair::path(c, r, to).expect("native dry path");
        let Some(button) = path.first() else {
            return;
        };
        press(c, *button);
    }
    panic!("field route did not arrive at {to:?}");
}

pub(super) fn use_move(c: &mut VisibleShellController, field: PartyFieldMove) {
    press(c, GameButton::Start);
    let s = c.snapshot().expect("Start menu");
    let target = visible_start_menu_options(&c.shell, &s)
        .iter()
        .position(|o| *o == StartMenuOption::Pokemon)
        .expect("Pokemon menu");
    for _ in 0..32 {
        let index = c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if index == target {
            break;
        }
        press(
            c,
            if index < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    press(c, GameButton::A);
    for _ in 0..8 {
        if c.shell.party_cursor == 0 {
            break;
        }
        press(c, GameButton::Up);
    }
    press(c, GameButton::A);
    let s = c.snapshot().expect("party action menu");
    let target = visible_party_actions(&s, &c.shell)
        .expect("native party actions")
        .iter()
        .position(|a| *a == PartyAction::FieldMove(field))
        .expect("known field move action");
    for _ in 0..32 {
        let index = c
            .shell
            .party_action_cursor
            .as_ref()
            .expect("action cursor")
            .option_index;
        if index == target {
            break;
        }
        press(
            c,
            if index < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    press(c, GameButton::A);
}

fn itemfinder(c: &mut VisibleShellController, found: bool) {
    use super::super::{FieldPackAction, FieldPackPocket, visible_selected_pack_item_actions};
    let bag = c.snapshot().expect("before detector").bag;
    press(c, GameButton::Start);
    let snapshot = c.snapshot().expect("Start for detector");
    let target = visible_start_menu_options(&c.shell, &snapshot)
        .iter()
        .position(|o| *o == StartMenuOption::Pack)
        .expect("PACK");
    for _ in 0..16 {
        let index = c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if index == target {
            break;
        }
        press(
            c,
            if index < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    press(c, GameButton::A);
    for _ in 0..4 {
        if c.shell.field_pack_pocket == Some(FieldPackPocket::KeyItems) {
            break;
        }
        press(c, GameButton::Right);
    }
    assert_eq!(c.shell.field_pack_pocket, Some(FieldPackPocket::KeyItems));
    let snapshot = c.snapshot().expect("key items");
    assert_eq!(
        snapshot.bag.key_items.len(),
        1,
        "only the declared detector"
    );
    assert_eq!(snapshot.bag.key_items[0].item_id, "ITEMFINDER");
    press(c, GameButton::A);
    let snapshot = c.snapshot().expect("detector actions");
    let target =
        visible_selected_pack_item_actions(&snapshot, &c.shell, &FieldPackPocket::KeyItems, false)
            .expect("native key-item actions")
            .iter()
            .position(|a| *a == FieldPackAction::Use)
            .expect("USE");
    for _ in 0..16 {
        let index = c
            .shell
            .field_pack_action_cursor
            .as_ref()
            .expect("item action cursor")
            .option_index;
        if index == target {
            break;
        }
        press(
            c,
            if index < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    press(c, GameButton::A);
    for _ in 0..64 {
        if c.shell.field_notice.is_some() {
            break;
        }
        c.wait_frames(64).expect("native detector sound sequence");
    }
    let snapshot = c.snapshot().expect("native detector result");
    let expected = super::super::visible_asm_text(
        &snapshot,
        if found {
            "_ItemfinderItemNearbyText"
        } else {
            "_ItemfinderNopeText"
        },
    )
    .expect("original detector result text");
    assert_eq!(c.shell.field_notice.as_deref(), Some(expected.as_str()));
    assert_eq!(
        snapshot.bag, bag,
        "detector neither consumes items nor awards the cache"
    );
    finish(c);
    for _ in 0..8 {
        if !super::super::has_visible_shell_direction_action(&mut c.shell) {
            return;
        }
        press(c, GameButton::B);
    }
    panic!("detector did not return native movement");
}

fn flash(c: &VisibleShellController) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_engine_flag_set("STATUSFLAGS_FLASH")
        .expect("native Flash illumination bit")
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the native field-use successor"]
fn native_field_gallery_teaches_flash_holds_acknowledgment_and_preserves_reward() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, ROOM, (5, 6));
    {
        let mut partner = runtime
            .data()
            .create_pokemon("BEETYNAMO", 40, Dv::default())
            .expect("active Minnesota Flash learner");
        assert!(partner.species.tmhm_learnset.iter().any(|m| m == "FLASH"));
        partner.moves.retain(|m| m.name != "FLASH");
        partner.moves.truncate(3);
        partner.item = Some("SMOKE_BALL".into());
        let state = c.shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[0] = Some(partner);
        state.sync_party_from_storage();
        state.badges.johto[0] = true;
        state
            .flags
            .set_engine_flag("ENGINE_ZEPHYRBADGE", true)
            .expect("declared first badge");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(!flash(&c));
    assert!(!event(&c, READ));
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(!event(&c, READ), "darkness must prevent the clue");
    retain(&mut c, "native-field-gallery-dark.crystalsave");
    assert!(!event(&c, CACHE));
    walk(&mut c, &runtime, (9, 12));
    press(&mut c, GameButton::Down);
    at(&mut c, "MnR01Cave1F", (10, 9));
    walk(&mut c, &runtime, (5, 6));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    teaching::teach(&mut c, "HM_FLASH", "FLASH");
    walk(&mut c, &runtime, (11, 8));
    super::winona::travel(&mut c, GameButton::Down, ROOM, (8, 13));
    use_move(&mut c, PartyFieldMove::Flash);
    finish(&mut c);
    assert!(flash(&c));
    walk(&mut c, &runtime, (5, 6));
    press(&mut c, GameButton::Right);
    retain(&mut c, "native-field-gallery-lit.crystalsave");
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    assert!(c.snapshot().expect("held fossil page").ui.text_window_open);
    assert!(!event(&c, READ));
    press(&mut c, GameButton::Left);
    at(&mut c, ROOM, (5, 6));
    let path =
        super::authored_route_fixture::reached_save_path("native-field-held-fossil.crystalsave");
    c.save(&path).expect("save held native page");
    c = VisibleShellController::load_save(root.clone(), runtime.clone(), path.clone(), None)
        .expect("same-pack held-page Continue");
    super::authored_route_fixture::cleanup_reached_save(&path);
    assert!(!event(&c, READ), "saving cannot acknowledge the page");
    finish(&mut c);
    assert!(event(&c, READ));
    let before = c
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&runtime.data().items["RARE_CANDY"]);
    walk(&mut c, &runtime, (8, 5));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(event(&c, CACHE));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["RARE_CANDY"]),
        before + 1
    );
    c = continue_save(root, runtime.clone(), &mut c);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["RARE_CANDY"]),
        before + 1
    );
    walk(&mut c, &runtime, (9, 12));
    press(&mut c, GameButton::Down);
    at(&mut c, "MnR01Cave1F", (10, 9));
    assert!(flash(&c), "illumination persists inside connected caves");
    walk(&mut c, &runtime, (9, 10));
    press(&mut c, GameButton::Down);
    assert_eq!(
        c.snapshot()
            .expect("original world exit")
            .overworld
            .map_name,
        "MnR01C00R00"
    );
    assert!(!flash(&c), "native outdoors resets illumination");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the native field-use successor"]
fn native_field_gallery_flash_requires_the_badge() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, ROOM, (5, 6));
    {
        let mut partner = runtime
            .data()
            .create_pokemon("BEETYNAMO", 40, Dv::default())
            .expect("active learner");
        partner.moves.retain(|m| m.name != "FLASH");
        partner.moves.truncate(3);
        partner.moves.push(LearnedMove {
            name: "FLASH".into(),
            current_pp: runtime.data().moves["FLASH"].pp,
            pp_ups: 0,
        });
        let state = c.shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[0] = Some(partner);
        state.sync_party_from_storage();
        state.badges.johto[0] = false;
        state
            .flags
            .set_engine_flag("ENGINE_ZEPHYRBADGE", false)
            .expect("no first badge");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    use_move(&mut c, PartyFieldMove::Flash);
    finish(&mut c);
    assert!(!flash(&c));
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(!event(&c, READ));
    assert!(!event(&c, CACHE));
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the native field-use successor"]
fn native_field_lake_teaches_whirlpool_collects_hidden_cache_and_returns() {
    use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
    use crystal_core::world::movement::MovementMode;

    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, "MnLeechLake", (35, 18));
    {
        let mut partner = runtime
            .data()
            .create_pokemon("GILLPUP", 40, Dv::default())
            .expect("active Minnesota water partner");
        for name in ["SURF", "WHIRLPOOL"] {
            assert!(partner.species.tmhm_learnset.iter().any(|m| m == name));
        }
        partner.moves = vec![LearnedMove {
            name: "SURF".into(),
            current_pp: runtime.data().moves["SURF"].pp,
            pp_ups: 0,
        }];
        partner.item = Some("SMOKE_BALL".into());
        let state = c.shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[0] = Some(partner);
        state.sync_party_from_storage();
        for (i, flag) in BADGE_ENGINE_FLAGS.iter().enumerate().take(7) {
            state.badges.johto[i] = true;
            state
                .flags
                .set_engine_flag(flag, true)
                .expect("declared prior badges");
        }
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["HM_WHIRLPOOL"], 1)
                .expect("declared prior HM gift")
        );
    }
    assert!(
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .add_item(&runtime.data().items["ITEMFINDER"], 1)
            .expect("declared prior research reward")
    );
    mark_runtime_snapshot_dirty(&mut c.shell);
    // All state changes after this point come from ordinary controller input.
    teaching::teach(&mut c, "HM_WHIRLPOOL", "WHIRLPOOL");
    super::minneapolis_falls::mount(&mut c, GameButton::Right, "MnLeechLake", (36, 18));
    super::pepin_discovery::paddle(&mut c, &runtime, (59, 20));
    itemfinder(&mut c, true);
    press(&mut c, GameButton::Right);
    at(&mut c, "MnLeechLake", (59, 20));
    assert!(!event(&c, "EVENT_MN_LEECH_EDDY_CACHE"));
    let before = c
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&runtime.data().items["PP_UP"]);
    retain(&mut c, "native-field-lake-current-closed.crystalsave");
    use_move(&mut c, PartyFieldMove::Whirlpool);
    if c.snapshot()
        .expect("Whirlpool confirmation")
        .ui
        .pending_yes_no
        .is_some()
    {
        super::northstar_ice_lair::choice(&mut c, true);
    }
    finish(&mut c);
    retain(&mut c, "native-field-lake-current-cleared.crystalsave");
    super::winona::travel(&mut c, GameButton::Right, "MnLeechLake", (60, 20));
    assert_eq!(
        c.snapshot().expect("cleared current").overworld.mode,
        MovementMode::Surf
    );
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_LEECH_EDDY_CACHE"));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["PP_UP"]),
        before + 1
    );
    c = continue_save(root, runtime, &mut c);
    assert_eq!(
        c.snapshot().expect("continued current").overworld.mode,
        MovementMode::Surf
    );
    press(&mut c, GameButton::A);
    finish(&mut c);
    itemfinder(&mut c, false);
    let item = &c.shell.shell.runtime().data().items["PP_UP"];
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        before + 1
    );
    super::winona::travel(&mut c, GameButton::Left, "MnLeechLake", (59, 20));
    assert_eq!(
        c.snapshot().expect("ordinary safe return").overworld.mode,
        MovementMode::Surf
    );
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the native field-use successor"]
fn native_field_gallery_full_pocket_native_toss_retry_and_continue() {
    use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
    use crystal_core::models::item::ITEM_POCKET_ITEM;

    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, ROOM, (5, 6));
    {
        let mut partner = runtime
            .data()
            .create_pokemon("BEETYNAMO", 40, Dv::default())
            .expect("declared active Flash partner");
        partner.moves.retain(|m| m.name != "FLASH");
        partner.moves.truncate(3);
        partner.moves.push(LearnedMove {
            name: "FLASH".into(),
            current_pp: runtime.data().moves["FLASH"].pp,
            pp_ups: 0,
        });
        partner.item = Some("SMOKE_BALL".into());
        let state = c.shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[0] = Some(partner);
        state.sync_party_from_storage();
        state.badges.johto[0] = true;
        state
            .flags
            .set_engine_flag("ENGINE_ZEPHYRBADGE", true)
            .expect("declared badge");
        for (_, item) in runtime
            .data()
            .items
            .iter()
            .filter(|(id, item)| item.pocket == ITEM_POCKET_ITEM && id.as_str() != "RARE_CANDY")
            .take(ITEM_POCKET_CAPACITY)
        {
            assert!(
                state
                    .bag
                    .add_item(item, 1)
                    .expect("declared full item pocket")
            );
        }
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // No inventory, flag, party, position or illumination writes after setup.
    let before = c.snapshot().expect("full native pocket").bag;
    assert_eq!(before.items.len(), ITEM_POCKET_CAPACITY);
    assert!(!before.items.iter().any(|i| i.item_id == "RARE_CANDY"));
    use_move(&mut c, PartyFieldMove::Flash);
    finish(&mut c);
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(event(&c, READ));
    walk(&mut c, &runtime, (8, 5));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(
        !event(&c, CACHE),
        "capacity refusal must retain the native cache"
    );
    assert_eq!(c.snapshot().expect("after refused cache").bag, before);
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    assert!(!event(&c, CACHE));
    super::northstar_progression_rewards::native_toss_first_pocket_item(
        &mut c,
        &super::super::FieldPackPocket::Items,
    );
    for _ in 0..8 {
        press(&mut c, GameButton::B);
    }
    assert_eq!(
        c.snapshot().expect("native space freed").bag.items.len(),
        ITEM_POCKET_CAPACITY - 1
    );
    assert!(!event(&c, CACHE));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(event(&c, CACHE));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["RARE_CANDY"]),
        1
    );
    c = continue_save(root, runtime.clone(), &mut c);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["RARE_CANDY"]),
        1
    );
}
