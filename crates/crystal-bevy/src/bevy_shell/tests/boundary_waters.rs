//! Production review of published preserve terrain/cabin/hospitality only.
//! Expedition admission remains closed. Explicit review placement beyond the
//! guards is never an admitted Safari session or earned regional journey.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{
    AssetRoot, ROOT_BEER_FIRST_GIFT_EVENT, ROOT_BEER_ITEM_ID, ROOT_BEER_MART_ID,
    read_loaded_verified_compiled_game_pack,
};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{
    Point, RouteGrid, RouteSpec, finish_text, indoor_walk, neighbors, position,
};

const SERVICE: &str = "BWS_ROOT_BEER_LADYRootBeerService";
const CABIN: &str = "MnBoundaryRootBeerCabin";
const EXTERIORS: [&str; 4] = [
    "MnBoundarySafariC00R00",
    "MnBoundarySafariC01R00",
    "MnBoundarySafariC00R01",
    "MnBoundarySafariC01R01",
];

fn fixture(at: Point, injured: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_BOUNDARY_WATERS_TEST_PACK")
            .expect("published external Boundary Waters terrain pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified terrain/cabin pack"),
    )
    .expect("native Boundary Waters runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: format!("MnBoundarySafariC{:02}R{:02}", at.0 / 64, at.1 / 48),
            tile_x: at.0 % 64,
            tile_y: at.1 % 48,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit terrain-review placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("review trainer name");
    let mut pokemon = runtime
        .data()
        .create_pokemon("MEW", 60, Dv::default())
        .expect("legal HM-compatible review partner");
    pokemon.moves = vec![LearnedMove {
        name: "SURF".into(),
        current_pp: runtime.data().moves["SURF"].pp,
        pp_ups: 0,
    }];
    if injured {
        pokemon.hp = pokemon.max_hp - 100;
        pokemon.status = Some("POISON".into());
        pokemon.moves[0].current_pp = 0;
    }
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(pokemon));
    state.badges.johto = [true; 8];
    state.money = 1000;
    state.sync_party_from_storage();
    // Initial partner, injury, badges, money and placement are explicit. Never
    // inject a permit, Safari flag/counter, service success or later position.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_eq!(position(&mut controller), at);
    assert_no_expedition(&controller);
    (root, runtime, controller)
}

fn grid(runtime: &CrystalRuntime, start: Point) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnBoundarySafari",
            columns: 2,
            rows: 2,
            tileset: "mn_boundary_safari",
            start,
            doors: BTreeMap::from([((51, 39), CABIN)]),
        },
    )
}

fn kind(runtime: &CrystalRuntime, at: Point) -> String {
    let name = format!("MnBoundarySafariC{:02}R{:02}", at.0 / 64, at.1 / 48);
    let map = &runtime.data().maps[&name];
    let block = map.blocks
        [(at.1 as usize % 48) / 2 * usize::from(map.attributes.width) + (at.0 as usize % 64) / 2];
    runtime.data().tilesets["mn_boundary_safari"].collision[&format!("{block:02x}")]
        [(at.1 % 2 * 2 + at.0 % 2) as usize]
        .clone()
}

pub(super) fn assert_no_expedition(controller: &VisibleShellController) {
    let state = controller.shell.shell.session().state();
    assert_eq!(
        state.flags.is_engine_flag_set("ENGINE_SAFARI_ZONE"),
        Ok(false)
    );
    for key in ["wSafariBalls", "wSafariSteps"] {
        assert!(
            !state.script_runtime.memory.contains_key(key),
            "terrain review must not fabricate paid Safari resources"
        );
    }
}

pub(super) fn press(controller: &mut VisibleShellController, button: GameButton) {
    controller
        .press(button)
        .expect("production Boundary Waters input");
    controller
        .wait_frames(16)
        .expect("native presentation delay");
    assert_eq!(controller.shell.last_error, None);
}

fn dorothy(controller: &mut VisibleShellController) {
    assert_eq!(position(controller), (50, 43));
    press(controller, GameButton::Left);
    assert_eq!(position(controller), (50, 43));
    press(controller, GameButton::A);
}

fn diagnosis(controller: &mut VisibleShellController) -> String {
    let snapshot = controller.snapshot().expect("compact service diagnosis");
    let state = controller.shell.shell.session().state();
    format!(
        "phase={:?}, tile={:?}, text={:?}, prompt={:?}, shop={:?}, top={:?}, sell={:?}, quantity={:?}, notice={:?}, next={:?}, text_wait={:?}, queued_commands={}",
        snapshot.phase,
        snapshot.overworld.tile,
        snapshot.ui.text.as_ref().map(|text| &text.label),
        snapshot.ui.pending_yes_no,
        snapshot.pending_shop.as_ref().map(|shop| (
            &shop.mart_id,
            &shop.source_script,
            shop.command_index
        )),
        controller
            .shell
            .shop_top_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        controller
            .shell
            .sell_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        controller.shell.shop_quantity.as_ref().map(|quantity| (
            &quantity.item_id,
            quantity.quantity,
            quantity.confirmation
        )),
        controller.shell.shop_notice,
        state.script_runtime.next_script,
        state.script_runtime.pending_text_wait,
        state.script_runtime.command_queue.len(),
    )
}

pub(super) fn prompt(
    controller: &mut VisibleShellController,
    script: &str,
    index: usize,
) -> BTreeSet<String> {
    let mut labels = BTreeSet::new();
    for _ in 0..96 {
        let snapshot = controller.snapshot().expect("production prompt owner");
        if let Some(text) = &snapshot.ui.text {
            labels.insert(text.label.clone());
        }
        if let Some(prompt) = snapshot.ui.pending_yes_no {
            assert_eq!(prompt.source_script, script, "unexpected service choice");
            assert_eq!(
                prompt.command_index, index,
                "unexpected service choice index"
            );
            return labels;
        }
        assert!(
            snapshot.pending_shop.is_none(),
            "unexpected shop before requested choice"
        );
        press(controller, GameButton::A);
    }
    panic!(
        "service prompt {script}:{index} did not appear: {}",
        diagnosis(controller)
    );
}

pub(super) fn answer(controller: &mut VisibleShellController, yes: bool) {
    let original = controller
        .snapshot()
        .expect("live choice")
        .ui
        .pending_yes_no
        .expect("authored native choice");
    // Core publishes yesorno before the visible printer consumes every page.
    // Until that same native prompt resolves, A may accelerate or advance its
    // preceding text instead of selecting YES/NO. Read the production cursor
    // and move only when needed: its current generic menu adapter wraps at an
    // edge, so unconditional UP on the default YES would actually select NO.
    // Stop immediately once the authoritative prompt changes.
    for _ in 0..96 {
        let selected = controller
            .shell
            .yes_no_cursor
            .as_ref()
            .expect("native retained yes/no cursor")
            .option_index;
        if selected != usize::from(!yes) {
            press(
                controller,
                if yes {
                    GameButton::Up
                } else {
                    GameButton::Down
                },
            );
        }
        press(controller, GameButton::A);
        if controller
            .snapshot()
            .expect("choice continuation")
            .ui
            .pending_yes_no
            .as_ref()
            != Some(&original)
        {
            return;
        }
    }
    panic!("native choice failed to resolve: {original:?}");
}

pub(super) fn open_shop(controller: &mut VisibleShellController) {
    for _ in 0..64 {
        let snapshot = controller.snapshot().expect("native shop opening");
        if let Some(shop) = snapshot.pending_shop {
            assert_eq!(shop.mart_id, ROOT_BEER_MART_ID);
            assert_eq!(shop.inventory, [ROOT_BEER_ITEM_ID]);
            if controller.shell.shop_top_cursor.is_some()
                && controller.shell.shop_welcome_seen
                && controller.shell.shop_notice.is_none()
            {
                return;
            }
        }
        press(controller, GameButton::A);
    }
    panic!("production mart did not open: {}", diagnosis(controller));
}

pub(super) fn gift_flag(controller: &VisibleShellController) -> bool {
    controller
        .shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(ROOT_BEER_FIRST_GIFT_EVENT)
        .expect("saved gift event")
}

pub(super) fn checkout_one(controller: &mut VisibleShellController) {
    press(controller, GameButton::A); // BUY -> inventory.
    assert!(controller.shell.shop_top_cursor.is_none());
    press(controller, GameButton::A); // ROOT BEER -> quantity.
    let quantity = controller
        .shell
        .shop_quantity
        .as_ref()
        .expect("native quantity selector");
    assert_eq!(quantity.item_id, ROOT_BEER_ITEM_ID);
    assert_eq!(quantity.quantity, 1);
    assert_eq!(quantity.unit_price, 300);
    assert_eq!(quantity.confirmation, None);
    press(controller, GameButton::A); // Quantity -> YES/NO confirmation.
    assert_eq!(
        controller
            .shell
            .shop_quantity
            .as_ref()
            .expect("purchase confirmation")
            .confirmation,
        Some(true)
    );
    press(controller, GameButton::A); // Actual native money/bag transaction.
    assert!(controller.shell.shop_quantity.is_none());
    assert!(controller.shell.shop_notice.is_some());
}

pub(super) fn close_service(controller: &mut VisibleShellController) {
    for _ in 0..96 {
        let snapshot = controller.snapshot().expect("service logoff");
        assert!(
            snapshot.ui.pending_yes_no.is_none(),
            "an explicit service choice remains"
        );
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
    panic!(
        "service failed to return walking: {}",
        diagnosis(controller)
    );
}

pub(super) fn drink_quantity(controller: &VisibleShellController, runtime: &CrystalRuntime) -> u16 {
    controller
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&runtime.data().items[ROOT_BEER_ITEM_ID])
}

pub(super) fn decline_rest_and_shop(controller: &mut VisibleShellController) {
    prompt(controller, &format!("{SERVICE}Choices"), 1);
    answer(controller, false);
    prompt(controller, &format!("{SERVICE}Shop"), 1);
    answer(controller, false);
    close_service(controller);
}

#[test]
#[ignore = "requires the published external Boundary Waters terrain/cabin pack"]
fn boundary_waters_controller_five_maps_single_outside_dorothy_and_both_cabin_returns() {
    let (root, runtime, mut controller) = fixture((51, 40), false);
    let expected: BTreeSet<_> = EXTERIORS
        .into_iter()
        .chain([CABIN])
        .map(str::to_owned)
        .collect();
    let actual: BTreeSet<_> = runtime
        .data()
        .runtime_map_metadata
        .values()
        .filter(|metadata| metadata.group_id == 61)
        .map(|metadata| metadata.name.clone())
        .collect();
    assert_eq!(actual, expected);
    let mut blocks = 0;
    let mut grannies = 0;
    let mut neighbors_count = 0;
    for (index, name) in EXTERIORS.into_iter().enumerate() {
        let map = &runtime.data().maps[name];
        let expected_size = match index {
            0 => (32, 24),
            1 => (16, 24),
            2 => (32, 16),
            3 => (16, 16),
            _ => unreachable!(),
        };
        assert_eq!((map.attributes.width, map.attributes.height), expected_size);
        assert_eq!(
            runtime
                .data()
                .runtime_map_metadata
                .values()
                .find(|metadata| metadata.name == name)
                .expect("published exterior metadata")
                .map_id,
            index as u16 + 1
        );
        blocks += map.blocks.len();
        neighbors_count += map.attributes.connections.len();
        grannies += map
            .objects
            .iter()
            .filter(|o| o.sprite == "SPRITE_GRANNY")
            .count();
        assert!(
            !runtime.data().wild_encounters.contains_key(name),
            "no ordinary encounters stand in for Safari runtime"
        );
    }
    let room = &runtime.data().maps[CABIN];
    blocks += room.blocks.len();
    assert_eq!(blocks, 1940);
    assert_eq!(neighbors_count, 8);
    assert_eq!(grannies, 1);
    assert_eq!((room.attributes.width, room.attributes.height), (5, 4));
    assert!(
        room.objects.is_empty(),
        "default cabin contains no duplicate Dorothy"
    );
    let outside = &runtime.data().maps[EXTERIORS[0]];
    let actor = outside
        .objects
        .iter()
        .find(|o| o.script == SERVICE)
        .expect("actual outside hospitality actor");
    assert_eq!((actor.x, actor.y), (49, 43));
    let grid = grid(&runtime, (51, 40));
    for carpet_x in [5, 6] {
        grid.walk(&mut controller, (51, 39));
        let entered = controller.snapshot().expect("cabin passive arrival");
        assert_eq!(entered.overworld.map_name, CABIN);
        assert_eq!((entered.overworld.tile.x, entered.overworld.tile.y), (5, 6));
        assert!(
            controller
                .shell
                .shell
                .session()
                .overworld()
                .objects
                .is_empty()
        );
        indoor_walk(&mut controller, &runtime, (6, 6));
        indoor_walk(&mut controller, &runtime, (5, 6));
        assert_eq!(
            controller
                .snapshot()
                .expect("passive room Floor does not warp")
                .overworld
                .map_name,
            CABIN
        );
        // Continue once from the actual cabin Floor, without replacing state.
        if carpet_x == 5 {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos();
            let save = std::env::temp_dir().join(format!(
                "geothite-boundary-cabin-{}-{nonce}.crystalsave",
                std::process::id()
            ));
            controller.save(&save).expect("save actual cabin arrival");
            controller = VisibleShellController::load_save(
                root.clone(),
                runtime.clone(),
                save.clone(),
                None,
            )
            .expect("Continue inside actual cabin");
            let continued = controller.snapshot().expect("continued cabin");
            assert_eq!(continued.overworld.map_name, CABIN);
            assert_eq!(
                (continued.overworld.tile.x, continued.overworld.tile.y),
                (5, 6)
            );
            std::fs::remove_file(save).expect("remove cabin save");
        }
        indoor_walk(&mut controller, &runtime, (carpet_x, 6));
        indoor_walk(&mut controller, &runtime, (carpet_x, 7));
        for _ in 0..4 {
            if controller
                .snapshot()
                .expect("native carpet exit")
                .overworld
                .map_name
                != CABIN
            {
                break;
            }
            press(&mut controller, GameButton::Down);
        }
        assert_eq!(
            controller
                .snapshot()
                .expect("safe cabin return")
                .overworld
                .map_name,
            EXTERIORS[0]
        );
        assert_eq!(position(&mut controller), (51, 40));
        grid.walk(&mut controller, (51, 41));
        grid.walk(&mut controller, (51, 40));
        assert_eq!(position(&mut controller), (51, 40));
        assert_no_expedition(&controller);
    }
}

#[test]
#[ignore = "requires the published external Boundary Waters terrain/cabin pack"]
fn boundary_waters_controller_closed_admission_charges_nothing_and_blocks_the_real_gate() {
    let (_, runtime, mut controller) = fixture((15, 73), false);
    let grid = grid(&runtime, (15, 73));
    assert!(!grid.floor.contains(&(11, 68)));
    assert!(!grid.floor.contains(&(12, 68)));
    let before = controller.snapshot().expect("closed entry resources");
    for at in [(11, 69), (12, 69), (10, 71)] {
        assert!(
            grid.talk(&mut controller, at)
                .contains("EXPEDITIONS CLOSED")
        );
        assert_eq!(controller.shell.shell.session().state().money, 1000);
        assert_no_expedition(&controller);
    }
    for x in [11, 12] {
        grid.walk(&mut controller, (x, 70));
        for _ in 0..4 {
            press(&mut controller, GameButton::Up);
            assert_eq!(position(&mut controller), (x, 70));
        }
    }
    let after = controller
        .snapshot()
        .expect("closed admission preserves resources");
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.party, after.party);
}

pub(super) fn drink_gift(controller: &mut VisibleShellController) {
    finish_text(controller);
    controller
        .press(GameButton::Start)
        .expect("production Start");
    let snapshot = controller.snapshot().expect("Start options");
    let target = super::super::visible_start_menu_options(&controller.shell, &snapshot)
        .iter()
        .position(|o| *o == super::super::StartMenuOption::Pack)
        .expect("PACK option");
    for _ in 0..16 {
        let current = controller
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if current == target {
            controller.press(GameButton::A).expect("open PACK");
            break;
        }
        controller
            .press(if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            })
            .expect("select PACK");
    }
    let bag = controller.snapshot().expect("carried medicine").bag;
    let target = bag
        .items
        .iter()
        .filter(|s| s.quantity > 0)
        .position(|s| s.item_id == ROOT_BEER_ITEM_ID)
        .expect("collected Root Beer");
    for _ in 0..16 {
        let current = controller
            .shell
            .bag_cursor
            .as_ref()
            .expect("production bag cursor")
            .option_index;
        if current == target {
            controller.press(GameButton::A).expect("medicine actions");
            break;
        }
        controller
            .press(if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            })
            .expect("select medicine");
    }
    let snapshot = controller.snapshot().expect("medicine actions");
    let actions = super::super::visible_selected_pack_item_actions(
        &snapshot,
        &controller.shell,
        &super::super::FieldPackPocket::Items,
        false,
    )
    .expect("production item actions");
    let target = actions
        .iter()
        .position(|o| *o == super::super::FieldPackAction::Use)
        .expect("USE action");
    for _ in 0..16 {
        let current = controller
            .shell
            .field_pack_action_cursor
            .as_ref()
            .expect("item action cursor")
            .option_index;
        if current == target {
            controller.press(GameButton::A).expect("use medicine");
            break;
        }
        controller
            .press(if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            })
            .expect("select USE");
    }
    assert!(
        controller.shell.field_pack_target_mode
            == Some(super::super::FieldPackTargetMode::PartyPokemon),
        "medicine target did not open; selected={:?}, ui={:?}",
        super::super::selected_field_pack_item_id(&mut controller.shell),
        controller
            .snapshot()
            .expect("medicine target diagnostic")
            .ui
    );
    for _ in 0..6 {
        if controller.shell.party_cursor == 0 {
            break;
        }
        controller
            .press(GameButton::Up)
            .expect("select lead partner");
    }
    assert_eq!(controller.shell.party_cursor, 0);
    let before = controller.snapshot().expect("injured Mew").party.slots[0]
        .pokemon
        .hp;
    controller
        .press(GameButton::A)
        .expect("apply Root Beer to Mew");
    let healed = controller.snapshot().expect("native healing result");
    assert!(
        healed.party.slots[0].pokemon.hp == (before + 60).min(healed.party.slots[0].pokemon.max_hp),
        "medicine did not restore HP: before={before}, after={}, target={:?}, notice={:?}, field_notice={:?}, ui={:?}",
        healed.party.slots[0].pokemon.hp,
        controller.shell.field_pack_target_mode,
        controller.shell.visible_field_item_notice,
        controller.shell.field_notice,
        healed.ui
    );
    assert!(
        !healed
            .bag
            .items
            .iter()
            .any(|s| s.item_id == ROOT_BEER_ITEM_ID && s.quantity > 0),
        "medicine not consumed"
    );
    for _ in 0..16 {
        let current = controller.snapshot().expect("close field Pack");
        if controller.shell.field_notice.is_some() || current.ui.pending_text_wait.is_some() {
            controller
                .wait_frames(60)
                .expect("healing presentation frames");
            controller.press(GameButton::A).expect("healing notice");
        } else if current.ui.menu.is_some() {
            controller
                .press(GameButton::B)
                .expect("close medicine menu");
        } else {
            return;
        }
    }
    let final_ui = controller.snapshot().expect("healing close diagnostic").ui;
    panic!(
        "medicine did not return to walking: field_notice={:?}, item_notice={:?}, ui={:?}",
        controller.shell.field_notice, controller.shell.visible_field_item_notice, final_ui
    );
}

#[test]
#[ignore = "requires the published external Boundary Waters terrain/cabin pack"]
fn boundary_waters_controller_actual_outside_gift_drink_rest_shop_and_continue() {
    let (root, runtime, mut controller) = fixture((50, 43), true);
    let initial_hp = controller
        .snapshot()
        .expect("injured review partner")
        .party
        .slots[0]
        .pokemon
        .hp;
    dorothy(&mut controller);
    prompt(&mut controller, SERVICE, 5);
    assert!(!gift_flag(&controller));
    answer(&mut controller, true);
    prompt(&mut controller, &format!("{SERVICE}Choices"), 1);
    assert!(gift_flag(&controller));
    assert_eq!(drink_quantity(&controller, &runtime), 1);
    assert_eq!(controller.shell.shell.session().state().money, 1000);
    assert_eq!(
        controller.snapshot().expect("gift is not rest").party.slots[0]
            .pokemon
            .hp,
        initial_hp
    );
    decline_rest_and_shop(&mut controller);
    drink_gift(&mut controller);
    assert_eq!(drink_quantity(&controller, &runtime), 0);
    let partner = &controller
        .snapshot()
        .expect("drink preserves PP")
        .party
        .slots[0]
        .pokemon;
    assert_eq!(partner.hp, initial_hp + 60);
    assert_eq!(partner.moves[0].current_pp, 0);
    assert_no_expedition(&controller);

    dorothy(&mut controller);
    prompt(&mut controller, &format!("{SERVICE}Choices"), 1);
    answer(&mut controller, true);
    prompt(&mut controller, &format!("{SERVICE}Shop"), 1);
    let state = controller.shell.shell.session().state();
    let partner = state.storage.party.pokemon[0]
        .as_ref()
        .expect("rested partner");
    assert_eq!(partner.hp, partner.max_hp);
    assert_eq!(partner.status, None);
    assert_eq!(partner.moves[0].current_pp, runtime.data().moves["SURF"].pp);
    assert_no_expedition(&controller);
    answer(&mut controller, true);
    open_shop(&mut controller);
    checkout_one(&mut controller);
    assert_eq!(drink_quantity(&controller, &runtime), 1);
    assert_eq!(controller.shell.shell.session().state().money, 700);
    close_service(&mut controller);

    let before = controller.snapshot().expect("actual completed hospitality");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let temporary = std::env::temp_dir().join(format!(
        "geothite-boundary-hospitality-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    let save = std::env::var_os("CRYSTAL_BOUNDARY_WATERS_HOSPITALITY_SAVE")
        .map(PathBuf::from)
        .unwrap_or_else(|| temporary.clone());
    controller
        .save(&save)
        .expect("save real hospitality transactions");
    let mut continued =
        VisibleShellController::load_save(root, runtime.clone(), save.clone(), None)
            .expect("Continue published Boundary Waters service");
    let after = continued
        .snapshot()
        .expect("continued native service state");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(position(&mut continued), (50, 43));
    assert_eq!(continued.shell.shell.session().state().money, 700);
    assert!(gift_flag(&continued));
    dorothy(&mut continued);
    decline_rest_and_shop(&mut continued);
    assert_eq!(
        drink_quantity(&continued, &runtime),
        1,
        "saved first-gift event skips another free bottle"
    );
    assert_eq!(continued.shell.shell.session().state().money, 700);
    assert_no_expedition(&continued);
    if save == temporary {
        std::fs::remove_file(save).expect("remove temporary hospitality save");
    }
}

pub(super) fn step(controller: &mut VisibleShellController, button: GameButton, target: Point) {
    for _ in 0..8 {
        if position(controller) == target {
            return;
        }
        press(controller, button);
        assert!(
            controller
                .snapshot()
                .expect("terrain-only travel")
                .battle
                .is_none()
        );
    }
    assert_eq!(
        position(controller),
        target,
        "native movement did not reach adjacent target"
    );
}

pub(super) fn mount_surf(
    controller: &mut VisibleShellController,
    direction: GameButton,
    target: Point,
) {
    assert_eq!(
        controller.snapshot().expect("land mode").overworld.mode,
        MovementMode::Normal
    );
    press(controller, direction);
    press(controller, GameButton::A);
    for _ in 0..128 {
        if controller.snapshot().expect("native launch").overworld.mode == MovementMode::Surf {
            controller.wait_frames(60).expect("Surf presentation");
            finish_text(controller);
            assert_eq!(position(controller), target);
            return;
        }
        press(controller, GameButton::A);
    }
    panic!(
        "native Surf launch did not finish: {}",
        diagnosis(controller)
    );
}

/// Plans only ordinary WATER cells from the published terrain. Every planned
/// edge still uses production input; this does not implement Surf game rules.
fn paddle(
    runtime: &CrystalRuntime,
    controller: &mut VisibleShellController,
    to: Point,
    visited: &mut BTreeSet<String>,
) {
    let from = position(controller);
    let mut parents = BTreeMap::from([(from, (from, GameButton::A))]);
    let mut queue = VecDeque::from([from]);
    while let Some(at) = queue.pop_front() {
        if at == to {
            break;
        }
        for (next, button) in neighbors(at) {
            if next.0 < 0
                || next.0 >= 96
                || next.1 < 0
                || next.1 >= 80
                || parents.contains_key(&next)
                || kind(runtime, next) != "WATER"
            {
                continue;
            }
            parents.insert(next, (at, button));
            queue.push_back(next);
        }
    }
    assert!(
        parents.contains_key(&to),
        "ordinary native water path {from:?} -> {to:?}"
    );
    let mut path = Vec::new();
    let mut cursor = to;
    while cursor != from {
        let (prior, button) = parents[&cursor];
        path.push((button, cursor));
        cursor = prior;
    }
    visited.insert(
        controller
            .snapshot()
            .expect("initial lake chunk")
            .overworld
            .map_name,
    );
    for (button, target) in path.into_iter().rev() {
        step(controller, button, target);
        let snapshot = controller.snapshot().expect("native paddle step");
        assert_eq!(snapshot.overworld.mode, MovementMode::Surf);
        visited.insert(snapshot.overworld.map_name);
    }
}

pub(super) fn blocked_water(controller: &mut VisibleShellController, button: GameButton) {
    let before = position(controller);
    for _ in 0..4 {
        press(controller, button);
        assert_eq!(position(controller), before);
        assert_eq!(
            controller
                .snapshot()
                .expect("native barrier")
                .overworld
                .mode,
            MovementMode::Surf
        );
    }
}

#[test]
#[ignore = "requires the published external Boundary Waters terrain/cabin pack"]
fn boundary_waters_controller_four_grassy_portages_have_native_launches_and_dismounts() {
    // Each beyond-gate starting point is explicit review placement, not paid
    // admission. No Safari runtime/encounters currently exist in this pack.
    for (west, west_water, west_button, east, east_water, east_button, minimum_grass) in [
        (
            (3, 58),
            (3, 57),
            GameButton::Up,
            (13, 60),
            (14, 60),
            GameButton::Right,
            4,
        ),
        (
            (22, 21),
            (21, 21),
            GameButton::Left,
            (29, 21),
            (30, 21),
            GameButton::Right,
            4,
        ),
        (
            (38, 17),
            (37, 17),
            GameButton::Left,
            (57, 21),
            (58, 21),
            GameButton::Right,
            8,
        ),
        (
            (36, 42),
            (35, 42),
            GameButton::Left,
            (65, 45),
            (66, 45),
            GameButton::Right,
            6,
        ),
    ] {
        let (_, runtime, mut controller) = fixture(west, false);
        assert_eq!(kind(&runtime, west_water), "WATER");
        assert_eq!(kind(&runtime, east_water), "WATER");
        mount_surf(&mut controller, west_button, west_water);
        let back = neighbors(west_water)
            .into_iter()
            .find_map(|(p, b)| (p == west).then_some(b))
            .expect("adjacent west landing");
        step(&mut controller, back, west);
        assert_eq!(
            controller.snapshot().expect("west dismount").overworld.mode,
            MovementMode::Normal
        );
        let grid = grid(&runtime, west);
        let mut grass = 0;
        for button in grid.path(west, east) {
            let at = position(&mut controller);
            let next = neighbors(at)
                .into_iter()
                .find_map(|(p, b)| (b == button).then_some(p))
                .expect("land direction");
            step(&mut controller, button, next);
            grass += usize::from(kind(&runtime, next) == "TALL_GRASS");
        }
        assert!(
            grass >= minimum_grass,
            "published portage did not cross authored grass"
        );
        mount_surf(&mut controller, east_button, east_water);
        let back = neighbors(east_water)
            .into_iter()
            .find_map(|(p, b)| (p == east).then_some(b))
            .expect("adjacent east landing");
        step(&mut controller, back, east);
        assert_eq!(
            controller.snapshot().expect("east dismount").overworld.mode,
            MovementMode::Normal
        );
        assert_no_expedition(&controller);
    }
}

#[test]
#[ignore = "requires the published external Boundary Waters terrain/cabin pack"]
fn boundary_waters_controller_all_four_lake_chunks_and_real_rock_pine_barriers() {
    let (_, runtime, mut controller) = fixture((65, 45), false);
    mount_surf(&mut controller, GameButton::Right, (66, 45));
    let mut visited = BTreeSet::new();
    paddle(&runtime, &mut controller, (89, 73), &mut visited);
    assert_eq!(kind(&runtime, (90, 73)), "WALL");
    blocked_water(&mut controller, GameButton::Right);
    paddle(&runtime, &mut controller, (40, 46), &mut visited);
    assert_eq!(kind(&runtime, (39, 46)), "BUOY");
    blocked_water(&mut controller, GameButton::Left);
    paddle(&runtime, &mut controller, (3, 57), &mut visited);
    paddle(&runtime, &mut controller, (35, 42), &mut visited);
    assert_eq!(visited, EXTERIORS.into_iter().map(str::to_owned).collect());
    step(&mut controller, GameButton::Right, (36, 42));
    assert_eq!(
        controller
            .snapshot()
            .expect("Isle shore dismount")
            .overworld
            .mode,
        MovementMode::Normal
    );
    // The whole outer margin is enclosed by native pine WALL banks. Native
    // blue-water rocks are BUOY inside the lake, not mislabeled forest art.
    let perimeter: BTreeSet<_> = (0..96)
        .flat_map(|x| [(x, 0), (x, 79)])
        .chain((0..80).flat_map(|y| [(0, y), (95, y)]))
        .collect();
    assert_eq!(perimeter.len(), 348);
    for at in perimeter {
        assert!(
            matches!(kind(&runtime, at).as_str(), "WALL" | "BUOY"),
            "unclosed outer edge {at:?}"
        );
    }
    assert_no_expedition(&controller);
}
