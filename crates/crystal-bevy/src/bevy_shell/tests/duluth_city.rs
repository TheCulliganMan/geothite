//! Duluth production input contracts after explicit local review fixtures.
//! Position, legal partners/injury, money and upstream badges/depot are setup;
//! no test claims an earned campaign or supplies progression after setup.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::state::EventFlagMemory;
use crystal_core::systems::script_flags::{
    BADGE_ENGINE_FLAGS, ScriptFlagCommand, apply_script_flag_mutation,
};
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::super::{
    BevyRuntimeShell, BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, indoor_walk};

const CITY: &str = "MnDuluth";
const CLINIC: &str = "MnDuluthPokecenter1F";
const WORKSHOP: &str = "MnDuluthHarborWorkshop";
const SIGNAL: &str = "EVENT_MN_LIGHTHOUSE_SIGNAL";
const DEPOT: &str = "EVENT_MN_DEPOT_CLEARED";

#[path = "duluth_cliff_stairs.rs"]
mod cliff_stairs;

#[path = "outdoor_cliff_corners.rs"]
mod cliff_corners;

#[path = "outdoor_trainer_sightlines.rs"]
mod sightlines;

#[path = "outdoor_terrain_repairs.rs"]
mod outdoor_repairs;

fn load() -> (AssetRoot, CrystalRuntime) {
    let path = std::env::var_os("CRYSTAL_DULUTH_CITY_CONTROLLER_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/minnesota-cities-review.crystalpack")
        })
        .canonicalize()
        .expect("published external Minnesota cities pack");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(path).expect("verified native city pack"),
    )
    .expect("production Duluth runtime");
    for name in [CITY, CLINIC, WORKSHOP, "MnDuluthGym"] {
        assert!(
            runtime.data().maps.contains_key(name),
            "missing city map {name}"
        );
    }
    (root, runtime)
}

struct Setup<'a> {
    map: &'a str,
    at: Point,
    badges: [bool; 8],
    depot: bool,
    injured: bool,
    money: u32,
}

impl<'a> Setup<'a> {
    fn at(map: &'a str, at: Point) -> Self {
        Self {
            map,
            at,
            badges: [false; 8],
            depot: false,
            injured: false,
            money: 10000,
        }
    }
}

fn fixture(root: &AssetRoot, runtime: &CrystalRuntime, setup: Setup<'_>) -> VisibleShellController {
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("preserved HOME"),
            map_name: setup.map.into(),
            tile_x: setup.at.0,
            tile_y: setup.at.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit initial city placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    for (species, level) in [("POUCHID", 45), ("CINDORE", 12)] {
        let mut mon = runtime
            .data()
            .create_pokemon(species, level, Dv::default())
            .expect("legal review partner");
        if setup.injured {
            mon.hp = mon.max_hp.saturating_sub(15).max(1);
            mon.status = Some("POISON".into());
            for movement in &mut mon.moves {
                movement.current_pp = 0;
            }
        }
        assert!(
            controller
                .shell
                .shell
                .session_mut()
                .state_mut()
                .storage
                .party
                .add_pokemon(mon)
        );
    }
    let state = controller.shell.shell.session_mut().state_mut();
    state.sync_party_from_storage();
    for (index, (flag, awarded)) in BADGE_ENGINE_FLAGS.iter().zip(setup.badges).enumerate() {
        apply_script_flag_mutation(
            state,
            ScriptFlagCommand {
                command: if awarded { "setflag" } else { "clearflag" }.into(),
                flag_id: (*flag).into(),
                source_script: "DuluthReviewFixture".into(),
                command_index: index,
            },
        )
        .expect("consistent initial badge bits and engine flags");
    }
    assert_eq!(state.badges.johto, setup.badges);
    state.money = setup.money;
    state
        .flags
        .set_event_flag(DEPOT, setup.depot)
        .expect("explicit upstream depot fixture");
    state
        .flags
        .set_event_flag(SIGNAL, false)
        .expect("receiver initially inactive");
    // All setup ends here. Every action below uses VisibleShellController;
    // neither coordinates nor flags/party/menu cursors are rewritten afterward.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_at(&mut controller, setup.map, setup.at);
    controller
}

fn press(controller: &mut VisibleShellController, button: GameButton) {
    controller.press(button).expect("production Duluth input");
    controller
        .wait_frames(16)
        .expect("native presentation settling");
    assert_eq!(controller.shell.last_error, None);
}

fn position(controller: &mut VisibleShellController) -> Point {
    let at = controller
        .snapshot()
        .expect("native position")
        .overworld
        .tile;
    (at.x, at.y)
}

fn assert_at(controller: &mut VisibleShellController, map: &str, at: Point) {
    let snapshot = controller.snapshot().expect("actual map and position");
    assert_eq!(
        (
            snapshot.overworld.map_name.as_str(),
            (snapshot.overworld.tile.x, snapshot.overworld.tile.y)
        ),
        (map, at)
    );
}

fn settle(controller: &mut VisibleShellController, ready: impl Fn(&BevyRuntimeShell) -> bool) {
    for _ in 0..192 {
        if ready(&controller.shell) {
            return;
        }
        press(controller, GameButton::A);
    }
    panic!(
        "native service did not reach its input boundary: {:?}",
        controller.snapshot().expect("diagnosis").ui
    );
}

fn finish_text(controller: &mut VisibleShellController) -> BTreeSet<String> {
    let mut labels = BTreeSet::new();
    for _ in 0..192 {
        let snapshot = controller.snapshot().expect("production dialogue");
        assert!(snapshot.battle.is_none());
        assert!(
            snapshot.ui.pending_yes_no.is_none(),
            "explicit choices must not be auto-accepted"
        );
        if let Some(text) = &snapshot.ui.text {
            labels.insert(text.label.clone());
        }
        if snapshot.phase == RuntimeShellPhase::Overworld
            && !snapshot.ui.text_window_open
            && !controller.has_pending_script_work()
            && controller.shell.field_notice.is_none()
            && controller.shell.battle_messages.is_empty()
        {
            return labels;
        }
        press(controller, GameButton::A);
    }
    panic!("dialogue did not release input");
}

fn talk(controller: &mut VisibleShellController, facing: GameButton) {
    let before = position(controller);
    press(controller, facing);
    assert_eq!(
        position(controller),
        before,
        "actor/counter blocks the facing input"
    );
    press(controller, GameButton::A);
}

fn move_to(controller: &mut VisibleShellController, direction: GameButton, target: Point) {
    for _ in 0..8 {
        if position(controller) == target {
            return;
        }
        press(controller, direction);
    }
    assert_eq!(position(controller), target, "native move {direction:?}");
}

fn enter(controller: &mut VisibleShellController, room: &str, arrival: Point) {
    for _ in 0..4 {
        if controller
            .snapshot()
            .expect("door approach")
            .overworld
            .map_name
            == room
        {
            break;
        }
        press(controller, GameButton::Up);
    }
    assert_at(controller, room, arrival);
    press(controller, GameButton::A);
    assert_at(controller, room, arrival); // Passive arrival never bounces out.
}

#[test]
#[ignore = "requires a published Duluth city pack"]
fn duluth_city_controller_all_seven_doors_fourteen_carpets_and_passive_returns() {
    let (root, runtime) = load();
    for (room, outside, arrival, exits) in [
        ("MnDuluthHillHouse", (9, 12), (4, 5), [(3, 7), (4, 7)]),
        ("MnDuluthGym", (16, 12), (6, 17), [(6, 19), (7, 19)]),
        (CLINIC, (37, 12), (10, 13), [(10, 15), (11, 15)]),
        ("MnDuluthMart", (48, 20), (6, 5), [(6, 7), (7, 7)]),
        ("MnDuluthHarborHouse", (7, 32), (3, 5), [(2, 7), (3, 7)]),
        ("MnDuluthHistoryHouse", (19, 30), (4, 7), [(4, 9), (5, 9)]),
        (WORKSHOP, (48, 36), (8, 9), [(8, 11), (9, 11)]),
    ] {
        let mut controller = fixture(&root, &runtime, Setup::at(CITY, outside));
        enter(&mut controller, room, arrival);
        for exit in exits {
            indoor_walk(&mut controller, &runtime, exit);
            for _ in 0..3 {
                if controller.snapshot().expect("carpet").overworld.map_name == CITY {
                    break;
                }
                press(&mut controller, GameButton::Down);
            }
            assert_at(&mut controller, CITY, outside);
            press(&mut controller, GameButton::A);
            assert_at(&mut controller, CITY, outside);
            enter(&mut controller, room, arrival);
        }
    }
}

#[test]
#[ignore = "requires a published Duluth city pack"]
fn duluth_city_controller_ledge_hops_down_blocks_reverse_and_walks_the_long_return() {
    let (root, runtime) = load();
    let mut controller = fixture(&root, &runtime, Setup::at(CITY, (14, 19)));
    move_to(&mut controller, GameButton::Down, (14, 20));
    move_to(&mut controller, GameButton::Down, (14, 22));
    for _ in 0..3 {
        press(&mut controller, GameButton::Up);
    }
    assert_at(&mut controller, CITY, (14, 22));
    indoor_walk(&mut controller, &runtime, (14, 19));
    assert_at(&mut controller, CITY, (14, 19));
}

#[test]
#[ignore = "requires a published Duluth city pack"]
fn duluth_city_controller_native_care_pc_deposit_withdraw_and_walking() {
    let (root, runtime) = load();
    let mut setup = Setup::at(CLINIC, (7, 5));
    setup.injured = true;
    let mut controller = fixture(&root, &runtime, setup);
    let initial = controller.snapshot().expect("injured partners");
    assert!(
        initial
            .party
            .slots
            .iter()
            .all(|slot| slot.pokemon.hp < slot.pokemon.max_hp
                && slot.pokemon.status.is_some()
                && slot.pokemon.moves.iter().all(|m| m.current_pp == 0))
    );
    let resources = {
        let state = controller.shell.shell.session().state();
        (
            state.money,
            state.bag.clone(),
            state.badges.clone(),
            state.flags.clone(),
        )
    };
    talk(&mut controller, GameButton::Up);
    let text_position = position(&mut controller);
    press(&mut controller, GameButton::Down);
    assert_eq!(
        position(&mut controller),
        text_position,
        "nurse dialogue owns input"
    );
    assert!(finish_text(&mut controller).contains("MN_DULUTH_NURSEText"));
    let healed = controller.snapshot().expect("actual HealParty result");
    assert!(healed.party.slots.iter().all(|slot| {
        slot.pokemon.hp == slot.pokemon.max_hp
            && slot.pokemon.status.is_none()
            && slot
                .pokemon
                .moves
                .iter()
                .all(|m| m.current_pp == runtime.data().moves[&m.name].pp)
    }));
    let state = controller.shell.shell.session().state();
    assert_eq!(
        (
            state.money,
            state.bag.clone(),
            state.badges.clone(),
            state.flags.clone()
        ),
        resources
    );
    let original = state
        .storage
        .party
        .pokemon
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    indoor_walk(&mut controller, &runtime, (13, 4));
    assert_at(&mut controller, CLINIC, (13, 4));
    press(&mut controller, GameButton::Up);
    assert_eq!(
        controller
            .shell
            .shell
            .current_overworld_interaction_checked()
            .expect("PC collision")
            .map(|interaction| interaction.script),
        Some("PCScript".into())
    );
    press(&mut controller, GameButton::A);
    settle(&mut controller, |shell| {
        shell.pc_hub_cursor.is_some()
            && shell.special_boundary.is_none()
            && shell.pc_notice.is_none()
            && shell.field_notice.is_none()
    });
    press(&mut controller, GameButton::A);
    settle(&mut controller, |shell| {
        shell.bill_pc_action_cursor.is_some()
            && shell.special_boundary.is_none()
            && shell.pc_notice.is_none()
            && shell.field_notice.is_none()
    });
    press(&mut controller, GameButton::Down);
    press(&mut controller, GameButton::A);
    assert!(controller.shell.bill_pc_deposit_open && controller.shell.storage_cursor.is_some());
    press(&mut controller, GameButton::A);
    assert!(controller.shell.bill_pc_pokemon_action_cursor.is_some());
    press(&mut controller, GameButton::A);
    settle(&mut controller, |shell| {
        shell.pc_transfer_sequence.is_none()
            && shell.pc_notice.is_none()
            && shell.bill_pc_pokemon_action_cursor.is_none()
    });
    let deposited = controller.snapshot().expect("real deposit mutation");
    assert_eq!(deposited.party.slots.len(), 1);
    assert_eq!(deposited.party.slots[0].pokemon.species.id, "CINDORE");
    let stored = deposited
        .storage
        .boxes
        .iter()
        .find(|b| b.index == deposited.storage.current_pc_box)
        .expect("current box");
    assert_eq!(stored.slots.len(), 1);
    assert_eq!(stored.slots[0].pokemon.species.id, "POUCHID");
    press(&mut controller, GameButton::B);
    assert_eq!(
        controller
            .shell
            .bill_pc_action_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        Some(0)
    );
    press(&mut controller, GameButton::A);
    press(&mut controller, GameButton::A);
    press(&mut controller, GameButton::A);
    settle(&mut controller, |shell| {
        shell.pc_transfer_sequence.is_none()
            && shell.pc_notice.is_none()
            && shell.bill_pc_pokemon_action_cursor.is_none()
    });
    let withdrawn = controller.snapshot().expect("real withdrawal mutation");
    assert_eq!(withdrawn.party.slots.len(), 2);
    assert!(withdrawn.storage.boxes.iter().all(|b| b.slots.is_empty()));
    let restored = controller
        .shell
        .shell
        .session()
        .state()
        .storage
        .party
        .pokemon
        .iter()
        .flatten()
        .collect::<Vec<_>>();
    assert!(original.iter().all(|mon| restored.contains(&mon)));
    for _ in 0..24 {
        if !controller.shell.pc_hub_session_open {
            break;
        }
        press(&mut controller, GameButton::B);
    }
    assert!(!controller.shell.pc_hub_session_open);
    finish_text(&mut controller);
    move_to(&mut controller, GameButton::Down, (13, 5));
    assert_at(&mut controller, CLINIC, (13, 5));
}

fn open_shop(controller: &mut VisibleShellController) {
    talk(controller, GameButton::Left);
    settle(controller, |shell| {
        shell.shop_top_cursor.is_some() && shell.shop_welcome_seen && shell.shop_notice.is_none()
    });
    let shop = controller
        .snapshot()
        .expect("real mart")
        .pending_shop
        .expect("native standard mart");
    assert_eq!(shop.mart_id, "MART_MN_DULUTH");
    assert_eq!(
        shop.inventory,
        [
            "ULTRA_BALL",
            "HYPER_POTION",
            "FULL_HEAL",
            "REVIVE",
            "MAX_REPEL"
        ]
    );
}

fn close_shop(controller: &mut VisibleShellController) {
    for _ in 0..96 {
        let snapshot = controller.snapshot().expect("shop logoff");
        if snapshot.pending_shop.is_none()
            && !snapshot.ui.text_window_open
            && snapshot.ui.menu.is_none()
            && !controller.has_pending_script_work()
            && controller.shell.field_notice.is_none()
        {
            return;
        }
        let button = if snapshot.pending_shop.is_none() || controller.shell.shop_notice.is_some() {
            GameButton::A
        } else if controller.shell.shop_quantity.is_some() {
            GameButton::B
        } else if let Some(cursor) = &controller.shell.shop_top_cursor {
            if cursor.option_index == 2 {
                GameButton::A
            } else {
                GameButton::Down
            }
        } else {
            GameButton::B
        };
        press(controller, button);
    }
    panic!("shop did not release walking input");
}

#[test]
#[ignore = "requires a published Duluth city pack"]
fn duluth_city_controller_native_mart_charges_real_price_and_closes() {
    let (root, runtime) = load();
    let mut controller = fixture(&root, &runtime, Setup::at("MnDuluthMart", (3, 3)));
    let before = controller.shell.shell.session().state().clone();
    open_shop(&mut controller);
    press(&mut controller, GameButton::A); // BUY.
    assert!(controller.shell.shop_top_cursor.is_none());
    press(&mut controller, GameButton::Down); // HYPER POTION is stock slot1.
    press(&mut controller, GameButton::A);
    let quantity = controller
        .shell
        .shop_quantity
        .as_ref()
        .expect("native quantity selector");
    assert_eq!(quantity.item_id, "HYPER_POTION");
    assert_eq!(quantity.quantity, 1);
    assert_eq!(
        quantity.unit_price,
        runtime.data().items["HYPER_POTION"].price
    );
    press(&mut controller, GameButton::A);
    assert_eq!(
        controller
            .shell
            .shop_quantity
            .as_ref()
            .expect("confirm")
            .confirmation,
        Some(true)
    );
    press(&mut controller, GameButton::A);
    assert!(controller.shell.shop_quantity.is_none());
    assert!(controller.shell.shop_notice.is_some());
    let state = controller.shell.shell.session().state();
    let item = &runtime.data().items["HYPER_POTION"];
    assert_eq!(state.money, before.money - u32::from(item.price));
    assert_eq!(state.bag.quantity(item), before.bag.quantity(item) + 1);
    assert_eq!(state.badges, before.badges);
    assert_eq!(state.flags, before.flags);
    close_shop(&mut controller);
    move_to(&mut controller, GameButton::Down, (3, 4));
    assert_at(&mut controller, "MnDuluthMart", (3, 4));
}

fn receive(controller: &mut VisibleShellController) -> BTreeSet<String> {
    talk(controller, GameButton::Right);
    let before = position(controller);
    press(controller, GameButton::Down);
    assert_eq!(
        position(controller),
        before,
        "receiver dialogue owns movement"
    );
    finish_text(controller)
}

fn active_flags(flags: &EventFlagMemory) -> (BTreeSet<&str>, BTreeSet<&str>) {
    // Continue can materialize false entries. Compare every active event and
    // engine flag, without treating absent and explicitly false as different.
    (
        flags.active_event_flags().map(String::as_str).collect(),
        flags
            .engine_flags
            .iter()
            .filter_map(|(flag, active)| active.then_some(flag.as_str()))
            .collect(),
    )
}

#[test]
#[ignore = "requires a published Duluth city pack"]
fn duluth_city_controller_q07_each_missing_badge_and_depot_are_refused_without_mutation() {
    let (root, runtime) = load();
    for missing in 0..9 {
        let mut setup = Setup::at(WORKSHOP, (10, 5));
        setup.badges = [true; 8];
        setup.depot = missing < 8;
        if missing < 8 {
            setup.badges[missing] = false;
        }
        let mut controller = fixture(&root, &runtime, setup);
        let before = controller.shell.shell.session().state().clone();
        assert!(receive(&mut controller).contains("MN_HARBOR_RECEIVERDeniedText"));
        let state = controller.shell.shell.session().state();
        assert_eq!(state.flags, before.flags);
        assert_eq!(state.badges, before.badges);
        assert_eq!(state.money, before.money);
        assert_eq!(state.bag, before.bag);
        assert_eq!(state.storage, before.storage);
        assert!(
            !state
                .flags
                .is_event_flag_set(SIGNAL)
                .expect("registered signal")
        );
    }
}

#[test]
#[ignore = "requires a published Duluth city pack"]
fn duluth_city_controller_q07_success_sets_only_signal_and_repeat_survives_continue() {
    let (root, runtime) = load();
    let mut setup = Setup::at(WORKSHOP, (10, 5));
    setup.badges = [true; 8];
    setup.depot = true;
    let mut controller = fixture(&root, &runtime, setup);
    let before = controller.shell.shell.session().state().clone();
    assert!(receive(&mut controller).contains("MN_HARBOR_RECEIVERText"));
    let mut expected_flags = before.flags.clone();
    expected_flags
        .set_event_flag(SIGNAL, true)
        .expect("signal result");
    let state = controller.shell.shell.session().state();
    assert_eq!(state.flags, expected_flags);
    assert_eq!(state.badges, before.badges);
    assert_eq!(state.bag, before.bag);
    assert_eq!(state.money, before.money);
    assert_eq!(state.storage, before.storage);
    assert!(receive(&mut controller).contains("MN_HARBOR_RECEIVERDoneText"));
    assert_eq!(
        controller.shell.shell.session().state().flags,
        expected_flags
    );
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("save nonce")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-duluth-q07-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    controller
        .save(&path)
        .expect("production save after receiver");
    let old = controller.snapshot().expect("before Continue");
    let mut continued = VisibleShellController::load_save(root, runtime, path.clone(), None)
        .expect("production Continue");
    let loaded = continued.snapshot().expect("continued actual state");
    assert_eq!(loaded.overworld.map_name, old.overworld.map_name);
    assert_eq!(loaded.overworld.tile, old.overworld.tile);
    assert_eq!(loaded.party, old.party);
    assert_eq!(loaded.bag, old.bag);
    assert_eq!(
        active_flags(&continued.shell.shell.session().state().flags),
        active_flags(&expected_flags)
    );
    assert_eq!(
        continued.shell.shell.session().state().badges,
        before.badges
    );
    assert_eq!(continued.shell.shell.session().state().money, before.money);
    assert!(receive(&mut continued).contains("MN_HARBOR_RECEIVERDoneText"));
    assert_eq!(
        active_flags(&continued.shell.shell.session().state().flags),
        active_flags(&expected_flags)
    );
    std::fs::remove_file(path).expect("remove own temporary save, never a render");
}

fn check_duluth_38_paired_returns(root: &AssetRoot, runtime: &CrystalRuntime) {
    let city = runtime
        .data()
        .assemble_map_module_from_compiled_payloads(CITY)
        .expect("complete native city");
    assert_eq!((city.attributes.width, city.attributes.height), (64, 46));
    assert_eq!(city.events.warps.len(), 76);
    for pair in city.events.warps.chunks_exact(2) {
        let door = &pair[0];
        let passive = &pair[1];
        let room_header = runtime
            .data()
            .maps
            .values()
            .find(|map| map.attributes.map_constant.as_deref() == Some(&door.target_map_constant))
            .expect("retained real interior");
        let room = runtime
            .data()
            .assemble_map_module_from_compiled_payloads(&room_header.id)
            .expect("complete retained interior");
        let arrival = room
            .events
            .warps
            .iter()
            .find(|warp| {
                i16::try_from(warp.index).expect("small warp index") == door.target_warp_id
            })
            .expect("interior passive arrival");
        let outside = (
            i16::try_from(passive.x).expect("city x"),
            i16::try_from(passive.y).expect("city y"),
        );
        assert_eq!(door.x, passive.x);
        assert_eq!(door.y + 1, passive.y);
        for exit in room
            .events
            .warps
            .iter()
            .filter(|warp| warp.target_map_constant == "MN_DULUTH")
        {
            assert_eq!(
                exit.target_warp_id,
                i16::try_from(passive.index).expect("city passive index")
            );
            let mut controller = fixture(root, runtime, Setup::at(CITY, outside));
            enter(
                &mut controller,
                &room.id,
                (
                    i16::try_from(arrival.x).expect("room x"),
                    i16::try_from(arrival.y).expect("room y"),
                ),
            );
            indoor_walk(
                &mut controller,
                runtime,
                (
                    i16::try_from(exit.x).expect("exit x"),
                    i16::try_from(exit.y).expect("exit y"),
                ),
            );
            for _ in 0..4 {
                if controller.snapshot().expect("return").overworld.map_name == CITY {
                    break;
                }
                press(&mut controller, GameButton::Down);
            }
            assert_at(&mut controller, CITY, outside);
            press(&mut controller, GameButton::A);
            assert_at(&mut controller, CITY, outside);
        }
    }
}

#[test]
#[ignore = "requires the current expanded Duluth pack"]
fn duluth_hill_harbor_current_38_paired_returns() {
    let (root, runtime) = load();
    check_duluth_38_paired_returns(&root, &runtime);
}

#[test]
#[ignore = "requires the historical hill-harbor stair course pack"]
fn duluth_hill_harbor_all_38_paired_returns_and_native_stair() {
    let (root, runtime) = load();
    check_duluth_38_paired_returns(&root, &runtime);
    let mut controller = fixture(&root, &runtime, Setup::at(CITY, (54, 37)));
    move_to(&mut controller, GameButton::Down, (54, 42));
    move_to(&mut controller, GameButton::Up, (54, 37));
    assert_at(&mut controller, CITY, (54, 37));
    let mut controller = fixture(&root, &runtime, Setup::at(CITY, (46, 39)));
    move_to(&mut controller, GameButton::Down, (46, 40));
    move_to(&mut controller, GameButton::Down, (46, 42));
    for _ in 0..3 {
        press(&mut controller, GameButton::Up);
    }
    assert_at(&mut controller, CITY, (46, 42));
    // The new stairs sit eight walking tiles east, preserving a useful
    // ordinary return instead of opening an adjacent reverse bypass.
    // Turning is a native input too; do not spend the eight-step walk budget
    // on the initial facing change.
    press(&mut controller, GameButton::Right);
    move_to(&mut controller, GameButton::Right, (54, 42));
    move_to(&mut controller, GameButton::Up, (54, 37));
    press(&mut controller, GameButton::Left);
    move_to(&mut controller, GameButton::Left, (46, 37));
    move_to(&mut controller, GameButton::Down, (46, 39));
    assert_at(&mut controller, CITY, (46, 39));
}
