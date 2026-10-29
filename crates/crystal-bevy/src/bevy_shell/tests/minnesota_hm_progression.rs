//! Local position/badges are explicit setup; conversations and gifts use input.
use crystal_core::input::GameButton;

use super::super::VisibleShellController;
use super::interior_life::{load, walk};

fn fixture(
    root: &crystal_assets::AssetRoot,
    runtime: &crystal_runtime::CrystalRuntime,
    map: &str,
    at: (i16, i16),
) -> VisibleShellController {
    let mut c = super::interior_life::fixture(root, runtime, map, at);
    // An outdoor starting-town fixture represents an already completed visit.
    for destination in runtime.data().fly_destinations.values() {
        let spawn = &runtime.data().runtime_spawn_points
            [&destination.destination_spawn_identifier.to_string()];
        if spawn.map_name == map {
            c.shell
                .shell
                .session_mut()
                .state_mut()
                .flags
                .set_engine_flag(&destination.flypoint_flag, true)
                .expect("starting town visit fixture");
        }
    }
    c
}

fn badge_fixture(c: &mut VisibleShellController, index: usize) {
    let flag = [
        "ENGINE_ZEPHYRBADGE",
        "ENGINE_HIVEBADGE",
        "ENGINE_PLAINBADGE",
        "ENGINE_FOGBADGE",
        "ENGINE_STORMBADGE",
        "ENGINE_MINERALBADGE",
        "ENGINE_GLACIERBADGE",
        "ENGINE_RISINGBADGE",
    ][index];
    let state = c.shell.shell.session_mut().state_mut();
    state.badges.johto[index] = true;
    state
        .flags
        .set_engine_flag(flag, true)
        .expect("matching badge flag fixture");
}
use super::r10_gallery::continue_save;
use super::r10_gallery::press;

fn talk(c: &mut VisibleShellController, target: (i16, i16)) -> String {
    let start = c.snapshot().expect("HM conversation start").overworld.tile;
    let facing = match (target.0 - start.x, target.1 - start.y) {
        (0, -1) => GameButton::Up,
        (-1, 0) => GameButton::Left,
        (1, 0) => GameButton::Right,
        (0, 1) => GameButton::Down,
        delta => panic!("nonadjacent HM interaction {delta:?}"),
    };
    press(c, facing);
    assert_eq!(c.snapshot().expect("body or counter").overworld.tile, start);
    press(c, GameButton::A);
    let mut pages = String::new();
    for _ in 0..256 {
        let snapshot = c.snapshot().expect("native HM/service dialogue");
        if let Some(text) = &snapshot.ui.text {
            pages.push_str(text.asm_text.as_deref().unwrap_or_default());
            pages.push('\n');
        }
        if !snapshot.ui.text_window_open
            && !c.has_pending_script_work()
            && snapshot.pending_shop.is_none()
            && c.shell.field_notice.is_none()
            && c.shell.visible_field_item_notice.is_none()
        {
            return pages;
        }
        let button = if snapshot.pending_shop.is_none()
            || c.shell.shop_notice.is_some()
            || !c.shell.shop_welcome_seen
        {
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
        press(c, button);
    }
    panic!("HM gift/original service did not release input: {pages}");
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with staged Minnesota HMs"]
fn minnesota_hm_progression_controller_all_seven_badge_denials_gifts_repeats_and_continue() {
    let (root, runtime, _) = load();
    for (map, at, target, item, badge, badge_name) in [
        ("MnR01Cave1F", (5, 6), (5, 5), "HM_FLASH", 0, "CARE"),
        ("MnR02ToolShed", (6, 4), (6, 3), "HM_CUT", 1, "PRAIRIE"),
        (
            "MnMinneapolisMillForemanHome",
            (2, 5),
            (2, 4),
            "HM_STRENGTH",
            2,
            "CURRENT",
        ),
        (
            "MnStPaulAnglerHouse",
            (5, 4),
            (5, 3),
            "HM_SURF",
            3,
            "CAPITOL",
        ),
        ("MnStCloud", (37, 14), (37, 13), "HM_FLY", 4, "WIND"),
        ("MnBemidji", (34, 36), (34, 35), "HM_WHIRLPOOL", 6, "PINE"),
        // Talk west over the outfitter counter at (4,5) to its host at (3,5).
        (
            "MnDuluthLakeOutfitter",
            (5, 5),
            (4, 5),
            "HM_WATERFALL",
            7,
            "HARBOR",
        ),
    ] {
        let mut c = fixture(&root, &runtime, map, at);
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&runtime.data().items[item]),
            0
        );
        let denied = talk(&mut c, target);
        assert!(
            denied.contains(badge_name) && denied.contains("BADGE"),
            "{map}: {denied}"
        );
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&runtime.data().items[item]),
            0,
            "no premature HM {item}"
        );
        badge_fixture(&mut c, badge);
        super::super::mark_runtime_snapshot_dirty(&mut c.shell);
        let badges = c.shell.shell.session().state().badges.clone();
        let money = c.shell.shell.session().state().money;
        let given = talk(&mut c, target);
        assert!(
            given.contains("HM") && given.contains("teach"),
            "{map}: {given}"
        );
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&runtime.data().items[item]),
            1
        );
        talk(&mut c, target);
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&runtime.data().items[item]),
            1,
            "repeat {item}"
        );
        assert_eq!(c.shell.shell.session().state().badges, badges);
        assert_eq!(c.shell.shell.session().state().money, money);
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        talk(&mut c, target);
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&runtime.data().items[item]),
            1,
            "Continue {item}"
        );
        if item == "HM_SURF" {
            assert!(
                !c.shell
                    .shell
                    .session()
                    .state()
                    .flags
                    .is_event_flag_set("EVENT_MN_BRAINERD_REPORT")
                    .expect("report flag")
            );
            assert!(
                !c.shell
                    .shell
                    .session()
                    .state()
                    .flags
                    .is_event_flag_set("EVENT_MN_SURF_GIFT_RECEIVED")
                    .expect("report gift flag")
            );
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with staged Minnesota HMs"]
fn minnesota_hm_progression_controller_early_surf_does_not_skip_the_brainerd_report() {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, "MnBrainerd", (25, 20));
    // Ownership is explicit setup here; the first case earns the early Surf
    // handoff through St Paul's actual conversation. No report flag is supplied.
    assert!(
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .add_item(&runtime.data().items["HM_SURF"], 1)
            .expect("early Surf fixture")
    );
    super::super::mark_runtime_snapshot_dirty(&mut c.shell);
    talk(&mut c, (25, 19));
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_BRAINERD_REPORT")
            .expect("report flag")
    );
    badge_fixture(&mut c, 5);
    super::super::mark_runtime_snapshot_dirty(&mut c.shell);
    talk(&mut c, (25, 19));
    for flag in ["EVENT_MN_BRAINERD_REPORT", "EVENT_MN_SURF_GIFT_RECEIVED"] {
        assert!(
            c.shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(flag)
                .expect("earned report flag"),
            "{flag}"
        );
    }
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["HM_SURF"]),
        1,
        "no duplicate HM"
    );
    c = continue_save(root, runtime.clone(), &mut c);
    talk(&mut c, (25, 19));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["HM_SURF"]),
        1
    );
}

#[test]
fn minnesota_fly_menu_names_the_town_instead_of_its_fallback_region() {
    let destination = crystal_runtime::RuntimeFlyDestinationKey {
        flypoint_flag: "ENGINE_FLYPOINT_MN_ST_CLOUD".into(),
        destination_spawn_identifier: 512,
        label: "LANDMARK_MN_RIVER_BLUFFS".into(),
    };
    assert_eq!(
        super::super::fly_destination_label(&destination),
        "ST CLOUD"
    );
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with Minnesota Fly points"]
fn minnesota_hm_progression_controller_town_visit_unlocks_only_its_fly_point() {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, "MnWinonaPokecenter1F", (4, 6));
    let flag = "ENGINE_FLYPOINT_MN_WINONA";
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .flags
            .is_engine_flag_set(flag)
            .expect("unvisited Winona")
    );
    walk(&mut c, &runtime, (4, 7));
    assert_eq!(
        c.snapshot().expect("real clinic exit").overworld.map_name,
        "MnWinona"
    );
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_engine_flag_set(flag)
            .expect("earned town visit")
    );
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .flags
            .is_engine_flag_set("ENGINE_FLYPOINT_MN_DULUTH")
            .expect("unvisited Duluth")
    );
    let _ = continue_save(root, runtime, &mut c);
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with Minnesota Fly points"]
fn minnesota_hm_progression_controller_fly_uses_visited_towns_and_a_real_safe_landing() {
    use super::super::{PartyAction, PartyFieldMove, StartMenuOption};
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, "MnStCloud", (37, 14));
    // Initial location, Wind Badge, legal party and a previously visited Winona
    // are fixtures. HM acquisition and every menu/travel action use real input.
    let pokemon = runtime
        .data()
        .create_pokemon("LOOBIT", 40, crystal_core::models::Dv::default())
        .expect("compatible regional bird");
    let state = c.shell.shell.session_mut().state_mut();
    state.badges.johto[4] = true;
    state
        .flags
        .set_engine_flag("ENGINE_STORMBADGE", true)
        .expect("Wind badge flag fixture");
    assert!(state.storage.party.add_pokemon(pokemon));
    state.sync_party_from_storage();
    state
        .flags
        .set_engine_flag("ENGINE_FLYPOINT_MN_WINONA", true)
        .expect("previous visit fixture");
    super::super::mark_runtime_snapshot_dirty(&mut c.shell);
    talk(&mut c, (37, 13));
    runtime
        .data()
        .use_bag_tmhm_on_party_pokemon(
            c.shell.shell.session_mut().state_mut(),
            "HM_FLY",
            0,
            Some(0),
        )
        .expect("legal teaching fixture");
    super::super::mark_runtime_snapshot_dirty(&mut c.shell);
    super::r10_gallery::settle(&mut c, "before native Fly menu");
    press(&mut c, GameButton::Start);
    for _ in 0..16 {
        let snapshot = c.snapshot().expect("Start menu");
        let options = super::super::visible_start_menu_options(&c.shell, &snapshot);
        if options[c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("production Start cursor")
            .option_index]
            == StartMenuOption::Pokemon
        {
            break;
        }
        press(&mut c, GameButton::Down);
    }
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::A);
    for _ in 0..16 {
        let snapshot = c.snapshot().expect("party actions");
        let actions = super::super::visible_party_actions(&snapshot, &c.shell)
            .expect("production field actions");
        if actions[c
            .shell
            .party_action_cursor
            .as_ref()
            .expect("production party cursor")
            .option_index]
            == PartyAction::FieldMove(PartyFieldMove::Fly)
        {
            break;
        }
        press(&mut c, GameButton::Down);
    }
    press(&mut c, GameButton::A);
    assert!(
        c.shell.fly_cursor.is_some(),
        "FLY opens the native destination menu"
    );
    for _ in 0..32 {
        let snapshot = c.snapshot().expect("Fly destinations");
        let destinations = super::super::active_fly_destinations(&snapshot, &c.shell.shell)
            .expect("visited destinations");
        assert!(
            !destinations
                .iter()
                .any(|d| d.flypoint_flag == "ENGINE_FLYPOINT_MN_DULUTH")
        );
        let entries = super::super::visible_fly_destination_entries(&snapshot, &c.shell)
            .expect("native Fly menu");
        if entries.iter().any(|line| line == ">WINONA") {
            break;
        }
        press(&mut c, GameButton::Down);
    }
    let snapshot = c.snapshot().expect("selected Fly destination");
    let destinations = super::super::active_fly_destinations(&snapshot, &c.shell.shell)
        .expect("visited Fly destinations");
    let selected = c
        .shell
        .fly_cursor
        .as_ref()
        .expect("Fly cursor")
        .option_index;
    assert_eq!(
        destinations[selected].flypoint_flag,
        "ENGINE_FLYPOINT_MN_WINONA",
        "visible entries: {:?}",
        super::super::visible_fly_destination_entries(&snapshot, &c.shell)
    );
    press(&mut c, GameButton::A);
    assert!(
        c.shell.visible_fly_animation.is_some(),
        "native Fly departure"
    );
    let departure = c.snapshot().expect("departure tile").overworld.tile;
    press(&mut c, GameButton::Right);
    assert_eq!(
        c.snapshot()
            .expect("Fly owns directional input")
            .overworld
            .tile,
        departure
    );
    for _ in 0..512 {
        let snapshot = c.snapshot().expect("native Fly travel");
        if snapshot.overworld.map_name == "MnWinona" {
            break;
        }
        if snapshot.ui.text_window_open {
            press(&mut c, GameButton::A);
        } else {
            c.wait_frames(16).expect("Fly presentation frames");
        }
    }
    let destination = &runtime.data().fly_destinations["ENGINE_FLYPOINT_MN_WINONA"];
    let spawn =
        &runtime.data().runtime_spawn_points[&destination.destination_spawn_identifier.to_string()];
    let landed = c.snapshot().expect("actual Fly landing");
    assert_eq!(
        landed.overworld.map_name,
        "MnWinona",
        "Fly events: {:?}; animation: {:?}; pending: {:?}",
        c.shell.last_audio_events,
        c.shell.visible_fly_animation,
        c.shell
            .shell
            .session()
            .state()
            .script_runtime
            .pending_field_travel
    );
    assert_eq!(
        (landed.overworld.tile.x, landed.overworld.tile.y),
        (spawn.tile_x, spawn.tile_y)
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["HM_FLY"]),
        1
    );
    let _ = continue_save(root, runtime, &mut c);
}
