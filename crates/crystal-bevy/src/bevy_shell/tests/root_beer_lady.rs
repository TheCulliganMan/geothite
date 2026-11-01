//! Explicit existing-house/party/inventory fixtures followed by production input.
//!
//! This exercises Dorothy's service in copied native Route 27 house geometry.
//! It does not publish or establish a completed Isle of Pines preserve or cabin.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{
    AssetRoot, MapModule, ROOT_BEER_FIRST_GIFT_EVENT, ROOT_BEER_ITEM_ID, ROOT_BEER_MART_ID,
    RootBeerRestPolicy, RuntimeMapMetadata, build_root_beer_lady_modpack,
    prepare_root_beer_lady_service, publish_root_beer_lady_service_map,
    read_loaded_verified_compiled_game_pack, read_verified_compiled_game_pack,
};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::world::map::TilePosition;
use crystal_runtime::CrystalRuntime;
use serde_json::{Value, json};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};

const MAP: &str = "RootBeerServiceFixtureHouse";
const MAP_CONSTANT: &str = "ROOT_BEER_SERVICE_FIXTURE_HOUSE";
const ACTOR: &str = "ROUTE27SANDSTORMHOUSE_GRANNY";

struct Fixture {
    directory: PathBuf,
    root: AssetRoot,
    runtime: CrystalRuntime,
    script: String,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.directory).expect("remove owned disposable service fixture");
    }
}

fn fixture() -> Fixture {
    let source_path = std::env::var_os("CRYSTAL_ROOT_BEER_TEST_PACK")
        .or_else(|| std::env::var_os("CRYSTAL_RENDER_TEST_PACK"))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/text-tui.crystalpack")
        })
        .canonicalize()
        .expect("resolve external native service source pack");
    let source = read_verified_compiled_game_pack(&source_path).expect("verified native source");
    let pack = build_root_beer_lady_modpack(&source).expect("additive drink overlay");
    let original = pack.data().maps["Route27SandstormHouse"].clone();
    // Preserve the exact native house geometry, Granny and door events while
    // clearing the original unrelated Sandstorm gift scripts and parsed tables.
    let mut payload = serde_json::to_value(&original).expect("existing-house fixture payload");
    for (name, field) in payload.as_object_mut().expect("native map object") {
        if matches!(
            name.as_str(),
            "id" | "attributes" | "objects" | "blocks" | "events"
        ) {
            continue;
        }
        *field = match field {
            Value::Array(_) => json!([]),
            Value::Object(_) => json!({}),
            _ => panic!("unexpected native fixture field {name}"),
        };
    }
    payload["scenes"] = json!({"scenes": []});
    let mut module: MapModule = serde_json::from_value(payload).expect("blank service fixture");
    module.id = MAP.into();
    module.attributes.map_constant = Some(MAP_CONSTANT.into());
    module.attributes.map_group_constant = Some("MAPGROUP_ROOT_BEER_SERVICE_FIXTURE".into());
    module.attributes.blocks_label = Some(format!("{MAP}_Blocks"));
    module.attributes.map_scripts_label = Some(format!("{MAP}_MapScripts"));
    module.attributes.map_events_label = Some(format!("{MAP}_MapEvents"));
    let prepared = prepare_root_beer_lady_service(&pack, &module, ACTOR, RootBeerRestPolicy::Offer)
        .expect("author service on existing native Granny anchor");
    let script = prepared.script_name.clone();
    assert_eq!(prepared.module.blocks, original.blocks);
    assert_eq!(prepared.module.events, original.events);
    let published = publish_root_beer_lady_service_map(
        prepared,
        RuntimeMapMetadata {
            constant: MAP_CONSTANT.into(),
            name: MAP.into(),
            group_name: "ROOT_BEER_SERVICE_FIXTURE".into(),
            group_id: 254,
            map_id: 1,
            width: module.attributes.width,
            height: module.attributes.height,
            environment: module
                .attributes
                .environment
                .clone()
                .expect("native room environment"),
            phone_service: module.attributes.phone_service,
        },
        [2, 5],
    )
    .expect("native fixture publication");
    assert_eq!(
        published.data().runtime_spawn_points,
        source.data().runtime_spawn_points
    );
    for (name, original) in &source.data().maps {
        assert_eq!(published.data().maps.get(name), Some(original));
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("fixture clock")
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("geothite-root-beer-{}-{stamp}", std::process::id()));
    std::fs::create_dir(&directory).expect("owned disposable pack directory");
    let path = directory.join("service-fixture.crystalpack");
    published
        .write_preserving_storage(&path)
        .expect("verified disposable fixture pack");
    let root = AssetRoot::new(&directory);
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("reload verified service fixture"),
    )
    .expect("native service runtime");
    Fixture {
        directory,
        root,
        runtime,
        script,
    }
}

fn controller(fixture: &Fixture, money: u32, full_bag: bool) -> VisibleShellController {
    let shell = initialize_bevy_runtime_shell(
        fixture.root.clone(),
        fixture.runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: fixture
                .runtime
                .title_new_game_spawn_identifier()
                .expect("HOME"),
            map_name: MAP.into(),
            tile_x: 2,
            tile_y: 5,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit existing-house fixture position");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture trainer name");
    let mut partner = fixture
        .runtime
        .data()
        .create_pokemon("MEW", 60, Dv::default())
        .expect("legal fixture partner");
    partner.hp = partner.max_hp - 100;
    partner.status = Some("POISON".into());
    for learned in &mut partner.moves {
        learned.current_pp = 0;
    }
    // Only initial position, partner injury, money and bag contents are fixtures.
    // Gift success/flags, all menu cursors and later resource changes use input.
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.money = money;
    state.bag.items.clear();
    if full_bag {
        for item in fixture
            .runtime
            .data()
            .items
            .values()
            .filter(|item| item.pocket == "ITEM" && item.script_name != ROOT_BEER_ITEM_ID)
            .take(ITEM_POCKET_CAPACITY)
        {
            assert!(state.bag.add_item(item, 1).expect("full-pocket fixture"));
        }
        assert_eq!(state.bag.items.len(), ITEM_POCKET_CAPACITY);
    }
    state
        .flags
        .set_engine_flag("ENGINE_SAFARI_ZONE", true)
        .expect("Safari resource sentinel");
    for (name, value) in [("wSafariBalls", "7"), ("wSafariSteps", "123")] {
        state
            .script_runtime
            .memory
            .insert(name.into(), value.into());
    }
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    controller
}

fn press(controller: &mut VisibleShellController, button: GameButton) {
    controller
        .press(button)
        .expect("production Dorothy service input");
    controller
        .wait_frames(16)
        .expect("native text/menu input delay");
    assert_eq!(controller.shell.last_error, None);
}

fn talk(controller: &mut VisibleShellController) {
    press(controller, GameButton::Up);
    assert_eq!(
        controller
            .snapshot()
            .expect("Granny collision")
            .overworld
            .tile,
        TilePosition::new(2, 5)
    );
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

fn prompt(controller: &mut VisibleShellController, script: &str, index: usize) -> BTreeSet<String> {
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

fn answer(controller: &mut VisibleShellController, yes: bool) {
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

fn open_shop(controller: &mut VisibleShellController) {
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

fn gift_flag(controller: &VisibleShellController) -> bool {
    controller
        .shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(ROOT_BEER_FIRST_GIFT_EVENT)
        .expect("saved gift event")
}

fn drink_quantity(controller: &VisibleShellController, fixture: &Fixture) -> u16 {
    controller
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&fixture.runtime.data().items[ROOT_BEER_ITEM_ID])
}

fn checkout_one(controller: &mut VisibleShellController) {
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

fn close_service(controller: &mut VisibleShellController) {
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

fn decline_rest_and_shop(controller: &mut VisibleShellController, fixture: &Fixture) {
    prompt(controller, &format!("{}Choices", fixture.script), 1);
    answer(controller, false);
    prompt(controller, &format!("{}Shop", fixture.script), 1);
    answer(controller, false);
    close_service(controller);
}

fn walk_to(controller: &mut VisibleShellController, button: GameButton, target: TilePosition) {
    for _ in 0..8 {
        if controller
            .snapshot()
            .expect("walking position")
            .overworld
            .tile
            == target
        {
            return;
        }
        press(controller, button);
    }
    assert_eq!(
        controller
            .snapshot()
            .expect("actual walking result")
            .overworld
            .tile,
        target,
        "{}",
        diagnosis(controller)
    );
}

#[test]
#[ignore = "requires an external native pack; publishes a disposable existing-house fixture only"]
fn root_beer_lady_controller_gift_rest_purchase_and_continue_use_native_input() {
    let fixture = fixture();
    let mut controller = controller(&fixture, 1_000, false);
    let initial_hp = controller
        .shell
        .shell
        .session()
        .state()
        .storage
        .party
        .pokemon[0]
        .as_ref()
        .expect("injured fixture partner")
        .hp;
    talk(&mut controller);
    prompt(&mut controller, &fixture.script, 5);
    assert!(!gift_flag(&controller));
    answer(&mut controller, true);
    prompt(&mut controller, &format!("{}Choices", fixture.script), 1);
    assert_eq!(drink_quantity(&controller, &fixture), 1);
    assert!(gift_flag(&controller));
    assert_eq!(controller.shell.shell.session().state().money, 1_000);
    assert_eq!(
        controller
            .shell
            .shell
            .session()
            .state()
            .storage
            .party
            .pokemon[0]
            .as_ref()
            .expect("partner")
            .hp,
        initial_hp
    );
    // An owned choice rejects movement; UP afterward selects YES through input.
    press(&mut controller, GameButton::Down);
    assert_eq!(
        controller
            .snapshot()
            .expect("choice owns directions")
            .overworld
            .tile,
        TilePosition::new(2, 5)
    );
    answer(&mut controller, true);
    prompt(&mut controller, &format!("{}Shop", fixture.script), 1);
    let state = controller.shell.shell.session().state();
    let partner = state.storage.party.pokemon[0]
        .as_ref()
        .expect("rested partner");
    assert_eq!(partner.hp, partner.max_hp);
    assert_eq!(partner.status, None);
    assert!(partner.moves.iter().all(|learned| learned.current_pp > 0));
    assert_eq!(
        state
            .script_runtime
            .memory
            .get("wSafariBalls")
            .map(String::as_str),
        Some("7")
    );
    assert_eq!(
        state
            .script_runtime
            .memory
            .get("wSafariSteps")
            .map(String::as_str),
        Some("123")
    );
    assert_eq!(
        state.flags.is_engine_flag_set("ENGINE_SAFARI_ZONE"),
        Ok(true)
    );
    answer(&mut controller, true);
    open_shop(&mut controller);
    checkout_one(&mut controller);
    assert_eq!(drink_quantity(&controller, &fixture), 2);
    assert_eq!(controller.shell.shell.session().state().money, 700);
    close_service(&mut controller);
    walk_to(&mut controller, GameButton::Down, TilePosition::new(2, 6));
    assert_eq!(
        controller
            .snapshot()
            .expect("walking resumes")
            .overworld
            .tile,
        TilePosition::new(2, 6)
    );
    walk_to(&mut controller, GameButton::Up, TilePosition::new(2, 5));
    assert_eq!(
        controller
            .snapshot()
            .expect("return to Granny")
            .overworld
            .tile,
        TilePosition::new(2, 5)
    );
    let save = fixture.directory.join("hospitality.crystalsave");
    controller
        .save(&save)
        .expect("save actual gift, purchase and rest");
    let mut loaded = VisibleShellController::load_save(
        fixture.root.clone(),
        fixture.runtime.clone(),
        save,
        None,
    )
    .expect("production Continue of hospitality state");
    assert!(gift_flag(&loaded));
    assert_eq!(drink_quantity(&loaded, &fixture), 2);
    assert_eq!(loaded.shell.shell.session().state().money, 700);
    let partner = loaded.shell.shell.session().state().storage.party.pokemon[0]
        .as_ref()
        .expect("persisted rest");
    assert_eq!(partner.hp, partner.max_hp);
    assert_eq!(partner.status, None);
    talk(&mut loaded);
    decline_rest_and_shop(&mut loaded, &fixture);
    assert_eq!(
        drink_quantity(&loaded, &fixture),
        2,
        "saved flag must skip another free bottle"
    );
}

#[test]
#[ignore = "requires an external native pack; publishes a disposable existing-house fixture only"]
fn root_beer_lady_controller_full_bag_refuses_then_native_sale_allows_first_gift_retry() {
    let fixture = fixture();
    let mut controller = controller(&fixture, 900, true);
    let before_bag = controller.shell.shell.session().state().bag.clone();
    talk(&mut controller);
    prompt(&mut controller, &fixture.script, 5);
    answer(&mut controller, true);
    let labels = prompt(&mut controller, &format!("{}Choices", fixture.script), 1);
    assert!(
        labels.iter().any(|label| label.ends_with("FullText")),
        "native full-bag refusal text must appear"
    );
    assert!(!gift_flag(&controller));
    assert_eq!(drink_quantity(&controller, &fixture), 0);
    assert_eq!(controller.shell.shell.session().state().bag, before_bag);
    answer(&mut controller, false);
    prompt(&mut controller, &format!("{}Shop", fixture.script), 1);
    answer(&mut controller, true);
    open_shop(&mut controller);
    checkout_one(&mut controller);
    assert!(
        controller
            .shell
            .shop_notice
            .as_deref()
            .is_some_and(|text| text.contains("carry"))
    );
    assert_eq!(controller.shell.shell.session().state().money, 900);
    assert_eq!(controller.shell.shell.session().state().bag, before_bag);
    assert!(!gift_flag(&controller));

    // Free a slot by selling one actual item through the production mart.
    // No mid-test bag edit or injected successful gift result is involved.
    for _ in 0..24 {
        if controller.shell.shop_notice.is_some() {
            press(&mut controller, GameButton::A);
        } else if controller.shell.shop_top_cursor.is_some() {
            break;
        } else {
            press(&mut controller, GameButton::B);
        }
    }
    assert_eq!(
        controller
            .shell
            .shop_top_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        Some(0),
        "{}",
        diagnosis(&mut controller)
    );
    press(&mut controller, GameButton::Down); // SELL.
    assert_eq!(
        controller
            .shell
            .shop_top_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        Some(1),
        "{}",
        diagnosis(&mut controller)
    );
    press(&mut controller, GameButton::A);
    assert!(
        controller.shell.sell_cursor.is_some(),
        "{}",
        diagnosis(&mut controller)
    );
    press(&mut controller, GameButton::A);
    let sale = controller
        .shell
        .shop_quantity
        .as_ref()
        .expect("native sale quantity");
    assert!(sale.selling);
    let sold_item = sale.item_id.clone();
    let sale_price = u32::from(fixture.runtime.data().items[&sold_item].price) / 2;
    assert!(sale_price > 0);
    press(&mut controller, GameButton::A);
    press(&mut controller, GameButton::A);
    assert_eq!(
        controller.shell.shell.session().state().bag.items.len(),
        ITEM_POCKET_CAPACITY - 1
    );
    assert_eq!(
        controller.shell.shell.session().state().money,
        900 + sale_price
    );
    close_service(&mut controller);
    talk(&mut controller);
    prompt(&mut controller, &fixture.script, 5);
    answer(&mut controller, true);
    prompt(&mut controller, &format!("{}Choices", fixture.script), 1);
    assert!(gift_flag(&controller));
    assert_eq!(drink_quantity(&controller, &fixture), 1);
    assert_eq!(
        controller.shell.shell.session().state().money,
        900 + sale_price
    );
    decline_rest_and_shop(&mut controller, &fixture);
}

#[test]
#[ignore = "requires an external native pack; publishes a disposable existing-house fixture only"]
fn root_beer_lady_controller_decline_preserves_gift_and_no_money_purchase_is_atomic() {
    let fixture = fixture();
    let mut controller = controller(&fixture, 0, false);
    talk(&mut controller);
    prompt(&mut controller, &fixture.script, 5);
    answer(&mut controller, false);
    prompt(&mut controller, &format!("{}Choices", fixture.script), 1);
    assert!(!gift_flag(&controller));
    assert_eq!(drink_quantity(&controller, &fixture), 0);
    answer(&mut controller, false);
    prompt(&mut controller, &format!("{}Shop", fixture.script), 1);
    answer(&mut controller, true);
    open_shop(&mut controller);
    let before_bag = controller.shell.shell.session().state().bag.clone();
    checkout_one(&mut controller);
    assert!(
        controller
            .shell
            .shop_notice
            .as_deref()
            .is_some_and(|text| text.contains("money"))
    );
    assert_eq!(controller.shell.shell.session().state().money, 0);
    assert_eq!(controller.shell.shell.session().state().bag, before_bag);
    assert!(!gift_flag(&controller));
    close_service(&mut controller);
    talk(&mut controller);
    prompt(&mut controller, &fixture.script, 5);
    answer(&mut controller, true);
    prompt(&mut controller, &format!("{}Choices", fixture.script), 1);
    assert!(gift_flag(&controller));
    assert_eq!(drink_quantity(&controller, &fixture), 1);
    decline_rest_and_shop(&mut controller, &fixture);
}
