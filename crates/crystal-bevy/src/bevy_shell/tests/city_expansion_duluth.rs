//! Explicit local fixtures followed by real Duluth errands, battles and Pack input.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

use crystal_assets::{AssetRoot, MapModule};
use crystal_core::input::GameButton;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::{ITEM_POCKET_ITEM, is_mail_item_id};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    OccupiedTile, PlayerMovementState, StepOptions, StepOutcome, attempt_step_with_occupied_tiles,
};
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};
use serde_json::Value;

use super::super::super::r10_gallery::completed_visible_page;
use super::super::super::r12_shelter::use_pp_item;
use super::{
    VisibleShellController, at, battle, close_shop, continue_save, contract, controller, event,
    finish, interact, load, module, navigate, point, press, walk_to,
};

const DELIVERED: &str = "EVENT_MN_DULUTH_X_REQUISITION_DELIVERED";
const PAID: &str = "EVENT_MN_DULUTH_X_REQUISITION_REWARDED";
const AWARD: &str = "EVENT_MN_DULUTH_X_TRAINING_REWARDED";
const NILS: &str = "EVENT_MN_DULUTH_X_TRAINER_04";
const IVAR: &str = "EVENT_MN_DULUTH_X_TRAINER_05";

fn fixture(room: &str, slots: usize) -> (AssetRoot, CrystalRuntime, VisibleShellController, Value) {
    let plan = contract();
    assert_eq!(plan["city"], "MnDuluth");
    assert_eq!(plan["dimensions_metatiles"], serde_json::json!([64, 48]));
    let (root, runtime, gyms) = load();
    let definition = plan["rooms"]
        .as_array()
        .expect("authored rooms")
        .iter()
        .find(|entry| entry["id"] == room);
    let mut setup = gyms["gyms"][2].clone();
    setup["id"] = Value::String(room.into());
    setup["arrival_core"] = definition.map_or_else(
        || plan["arrival_core"].clone(),
        |room| room["arrival_core"].clone(),
    );
    let mut c = controller(&root, &runtime, &setup, false);
    // All setup mutations precede the first input. No earned regional access is claimed.
    let state = c.shell.shell.session_mut().state_mut();
    state.money = 999999;
    state.bag.items.clear();
    if slots > 0 {
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["ETHER"], 1)
                .expect("initial ETHER")
        );
        for (id, item) in runtime
            .data()
            .items
            .iter()
            .filter(|(id, item)| {
                item.pocket == ITEM_POCKET_ITEM
                    && !["ETHER", "X_DEFEND", "HP_UP", "METAL_COAT"].contains(&id.as_str())
                    && !is_mail_item_id(id)
            })
            .take(slots - 1)
        {
            assert!(
                state
                    .bag
                    .add_item(item, 1)
                    .unwrap_or_else(|error| { panic!("legal initial filler {id}: {error}") })
            );
        }
        state.storage.party.pokemon[0]
            .as_mut()
            .expect("legal review partner")
            .moves[0]
            .current_pp = 0;
        state.sync_party_from_storage();
    }
    assert_eq!(state.bag.items.len(), slots);
    super::super::mark_runtime_snapshot_dirty(&mut c.shell);
    (root, runtime, c, plan)
}

fn quantity(c: &VisibleShellController, r: &CrystalRuntime, id: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items[id])
}

fn allowed(plan: &Value) -> BTreeSet<String> {
    plan["rooms"]
        .as_array()
        .expect("new rooms")
        .iter()
        .map(|room| room["id"].as_str().expect("room id").to_owned())
        .chain(["MnDuluth".into()])
        .collect()
}

fn protected(c: &VisibleShellController) -> [bool; 3] {
    [
        "EVENT_MN_DEPOT_CLEARED",
        "EVENT_MN_LIGHTHOUSE_SIGNAL",
        "EVENT_MN_GYM_8_DEFEATED",
    ]
    .map(|name| event(c, name))
}

fn dialogue(c: &mut VisibleShellController, accept: bool) -> String {
    let mut pages = BTreeSet::new();
    for _ in 0..128 {
        c.wait_frames(240).expect("print actual native quest page");
        if let Some((_, printed)) = completed_visible_page(c) {
            pages.insert(printed);
        }
        let snapshot = c.snapshot().expect("native quest presentation");
        assert!(snapshot.battle.is_none());
        if snapshot.phase == RuntimeShellPhase::Overworld
            && !snapshot.ui.text_window_open
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
            && snapshot.ui.menu.is_none()
        {
            return pages.into_iter().collect::<Vec<_>>().join("\n");
        }
        if snapshot.ui.pending_yes_no.is_some() {
            // Native yesorno owns a separate cursor. Sending Down as a text
            // ownership probe would select NO before the following A.
            let stand = at(c);
            press(c, GameButton::Left);
            assert_eq!(at(c), stand, "native YES/NO owns movement");
            let target = usize::from(!accept);
            for _ in 0..4 {
                let selected = c
                    .shell
                    .yes_no_cursor
                    .as_ref()
                    .expect("native quest YES/NO cursor")
                    .option_index;
                if selected == target {
                    break;
                }
                press(
                    c,
                    if selected < target {
                        GameButton::Down
                    } else {
                        GameButton::Up
                    },
                );
            }
            assert_eq!(
                c.shell
                    .yes_no_cursor
                    .as_ref()
                    .expect("selected native quest answer")
                    .option_index,
                target
            );
            press(c, GameButton::A);
        } else if let Some(cursor) = &c.shell.menu_cursor {
            let target = usize::from(!accept);
            press(
                c,
                if cursor.option_index == target {
                    GameButton::A
                } else if target == 0 {
                    GameButton::Up
                } else {
                    GameButton::Down
                },
            );
        } else {
            if snapshot.ui.text_window_open {
                let before = at(c);
                press(c, GameButton::Down);
                assert_eq!(at(c), before, "printed dialogue owns movement");
            }
            press(c, GameButton::A);
        }
    }
    panic!("native Duluth dialogue failed to release input; printed={pages:?}");
}

fn named_battle(c: &mut VisibleShellController, id: &str, class: &str, name: &str, species: &str) {
    for _ in 0..128 {
        if let Some(battle) = c.snapshot().expect("native optional battle entry").battle {
            let crystal_runtime::RuntimeBattleKind::Trainer {
                trainer_id,
                trainer_class,
                trainer_name,
                ..
            } = &battle.kind
            else {
                panic!("optional trainer started a non-trainer battle");
            };
            assert_eq!(trainer_id, id);
            assert_eq!(trainer_class, class);
            assert!(trainer_name.contains(name));
            assert_eq!(battle.enemy_pokemon.species.id, species);
            return;
        }
        press(c, GameButton::A);
    }
    panic!("native optional battle did not start: {id}");
}

fn buy_two_defends(c: &mut VisibleShellController, r: &CrystalRuntime) {
    // This is a native quantity-two purchase, not an inventory fixture mutation.
    walk_to(c, r, (5, 5));
    press(c, GameButton::Left);
    press(c, GameButton::A);
    for _ in 0..64 {
        if c.shell.shop_top_cursor.is_some()
            && c.shell.shop_welcome_seen
            && c.shell.shop_notice.is_none()
        {
            break;
        }
        press(c, GameButton::A);
    }
    let shop = c
        .snapshot()
        .expect("real tool counter")
        .pending_shop
        .expect("native mart");
    let index = shop
        .inventory
        .iter()
        .position(|id| id == "X_DEFEND")
        .expect("X DEFEND stock");
    press(c, GameButton::A);
    for _ in 0..32 {
        let selected = c
            .shell
            .menu_cursor
            .as_ref()
            .expect("native purchase list")
            .option_index;
        if selected == index {
            break;
        }
        press(
            c,
            if selected < index {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    let money = c.shell.shell.session().state().money;
    press(c, GameButton::A);
    assert_eq!(
        c.shell
            .shop_quantity
            .as_ref()
            .expect("quantity selector")
            .item_id,
        "X_DEFEND"
    );
    press(c, GameButton::Up);
    assert_eq!(
        c.shell
            .shop_quantity
            .as_ref()
            .expect("quantity two")
            .quantity,
        2
    );
    press(c, GameButton::A);
    assert_eq!(
        c.shell.shell.session().state().money,
        money,
        "confirmation does not charge"
    );
    press(c, GameButton::A);
    assert_eq!(quantity(c, r, "X_DEFEND"), 2);
    assert_eq!(
        c.shell.shell.session().state().money,
        money - 2 * u32::from(r.data().items["X_DEFEND"].price)
    );
    close_shop(c);
}

#[test]
#[ignore = "requires the published tight Duluth pack and exact expansion contract"]
fn duluth_expansion_controller_requisition_native_payment_full_bag_retry_once_continue() {
    let (root, r, mut c, plan) = fixture("MnDuluthCargoOffice", ITEM_POCKET_CAPACITY - 1);
    let original_story = protected(&c);
    let bag = c.shell.shell.session().state().bag.clone();
    let money = c.shell.shell.session().state().money;
    interact(&mut c, &r, (5, 5));
    let printed = dialogue(&mut c, true);
    assert!(
        printed.contains("You need one") && printed.contains("X DEFEND"),
        "actual refusal: {printed}"
    );
    assert_eq!(c.shell.shell.session().state().bag, bag);
    assert_eq!(c.shell.shell.session().state().money, money);
    assert!(!event(&c, DELIVERED) && !event(&c, PAID));
    let allowed = allowed(&plan);
    navigate(&mut c, &r, "MnDuluthToolExchange", &allowed);
    buy_two_defends(&mut c, &r);
    assert_eq!(
        c.shell.shell.session().state().bag.items.len(),
        ITEM_POCKET_CAPACITY
    );
    navigate(&mut c, &r, "MnDuluthCargoOffice", &allowed);
    let money = c.shell.shell.session().state().money;
    let unpaid = c.shell.shell.session().state().bag.clone();
    interact(&mut c, &r, (5, 5));
    dialogue(&mut c, false);
    assert_eq!(
        c.shell.shell.session().state().bag,
        unpaid,
        "No keeps payment"
    );
    assert!(!event(&c, DELIVERED) && !event(&c, PAID));
    interact(&mut c, &r, (5, 5));
    let printed = dialogue(&mut c, true);
    assert!(
        printed.contains("Your PACK is full"),
        "actual pending refusal: {printed}"
    );
    assert!(event(&c, DELIVERED) && !event(&c, PAID));
    assert_eq!(quantity(&c, &r, "X_DEFEND"), 1);
    assert_eq!(quantity(&c, &r, "HP_UP"), 0);
    let pending = c.shell.shell.session().state().bag.clone();
    interact(&mut c, &r, (5, 5));
    dialogue(&mut c, true);
    assert_eq!(
        c.shell.shell.session().state().bag,
        pending,
        "pending retry never takes another X DEFEND"
    );
    assert!(!event(&c, PAID));
    use_pp_item(&mut c, &r, "ETHER", Some(0));
    assert_eq!(
        c.shell.shell.session().state().bag.items.len(),
        ITEM_POCKET_CAPACITY - 1
    );
    interact(&mut c, &r, (5, 5));
    let printed = dialogue(&mut c, true);
    assert!(
        printed.contains("HP UP"),
        "actual received reward: {printed}"
    );
    assert!(event(&c, DELIVERED) && event(&c, PAID));
    assert_eq!(quantity(&c, &r, "HP_UP"), 1);
    assert_eq!(quantity(&c, &r, "X_DEFEND"), 1);
    assert_eq!(c.shell.shell.session().state().money, money);
    let received = c.shell.shell.session().state().bag.clone();
    interact(&mut c, &r, (5, 5));
    dialogue(&mut c, true);
    assert_eq!(c.shell.shell.session().state().bag, received);
    c = continue_save(&mut c, &root, &r);
    assert!(event(&c, DELIVERED) && event(&c, PAID));
    interact(&mut c, &r, (5, 5));
    dialogue(&mut c, true);
    assert_eq!(c.shell.shell.session().state().bag, received);
    assert_eq!(c.shell.shell.session().state().money, money);
    assert_eq!(protected(&c), original_story);
}

#[test]
#[ignore = "legal review party; two actual optional battles and finite training award"]
fn duluth_expansion_controller_coop_two_native_wins_award_refusal_retry_continue() {
    let (root, r, mut c, _) = fixture("MnDuluthMaritimeCoop3F", ITEM_POCKET_CAPACITY);
    let original_story = protected(&c);
    interact(&mut c, &r, (5, 5));
    let printed = dialogue(&mut c, true);
    assert!(
        printed.contains("NILS and IVAR"),
        "actual prerequisite text: {printed}"
    );
    assert!(!event(&c, NILS) && !event(&c, IVAR) && !event(&c, AWARD));
    for (target, flag, id, class, name, species) in [
        (
            (6, 5),
            NILS,
            "MN_DULUTH_X_T04",
            "BLACKBELT_T",
            "NILS",
            "POLIWRATH",
        ),
        (
            (10, 7),
            IVAR,
            "MN_DULUTH_X_T05",
            "COOLTRAINERM",
            "IVAR",
            "URSARING",
        ),
    ] {
        let before = c.snapshot().expect("pre-battle partner").party.slots[0]
            .pokemon
            .moves
            .iter()
            .map(|m| u16::from(m.current_pp))
            .sum::<u16>();
        interact(&mut c, &r, target);
        named_battle(&mut c, id, class, name, species);
        battle(&mut c, false);
        finish(&mut c);
        assert!(event(&c, flag));
        let after = c.snapshot().expect("actual spent PP").party.slots[0]
            .pokemon
            .moves
            .iter()
            .map(|m| u16::from(m.current_pp))
            .sum::<u16>();
        assert!(after < before);
        interact(&mut c, &r, (5, 5));
        let printed = dialogue(&mut c, true);
        assert!(!event(&c, AWARD));
        assert_eq!(quantity(&c, &r, "METAL_COAT"), 0);
        assert!(
            printed.contains(if flag == NILS {
                "NILS and IVAR"
            } else {
                "Your PACK is full"
            }),
            "actual award refusal: {printed}"
        );
    }
    let money = c.shell.shell.session().state().money;
    let full = c.shell.shell.session().state().bag.clone();
    interact(&mut c, &r, (5, 5));
    dialogue(&mut c, true);
    assert_eq!(c.shell.shell.session().state().bag, full);
    use_pp_item(&mut c, &r, "ETHER", Some(0));
    interact(&mut c, &r, (5, 5));
    let printed = dialogue(&mut c, true);
    assert!(printed.contains("METAL COAT"));
    assert!(event(&c, AWARD));
    assert_eq!(quantity(&c, &r, "METAL_COAT"), 1);
    let received = c.shell.shell.session().state().bag.clone();
    interact(&mut c, &r, (5, 5));
    dialogue(&mut c, true);
    assert_eq!(c.shell.shell.session().state().bag, received);
    c = continue_save(&mut c, &root, &r);
    assert!(event(&c, NILS) && event(&c, IVAR) && event(&c, AWARD));
    interact(&mut c, &r, (5, 5));
    dialogue(&mut c, true);
    assert_eq!(c.shell.shell.session().state().bag, received);
    assert_eq!(c.shell.shell.session().state().money, money);
    assert_eq!(protected(&c), original_story);
}

#[test]
#[ignore = "requires the six actual tight Duluth field discoveries"]
fn duluth_expansion_controller_six_discoveries_once_and_continue() {
    let (root, r, mut c, plan) = fixture("MnDuluth", 0);
    let original_story = protected(&c);
    let items = plan["items"].as_array().expect("field discoveries");
    assert_eq!(items.len(), 6);
    for item in items {
        let flag = item["flag"].as_str().expect("collection flag");
        assert!(!event(&c, flag));
        let id = item["item"].as_str().expect("native item id");
        let before = quantity(&c, &r, id);
        interact(&mut c, &r, point(&item["tile"]));
        let printed = dialogue(&mut c, true);
        assert!(
            !printed.is_empty(),
            "native item-ball presentation must print"
        );
        assert!(event(&c, flag));
        assert_eq!(quantity(&c, &r, id), before + 1);
        let collected = c.shell.shell.session().state().bag.clone();
        press(&mut c, GameButton::A);
        finish(&mut c);
        assert_eq!(
            c.shell.shell.session().state().bag,
            collected,
            "empty tile cannot grant twice"
        );
    }
    for (id, count) in [
        ("NUGGET", 1),
        ("MAX_REPEL", 1),
        ("ULTRA_BALL", 2),
        ("PP_UP", 1),
        ("FULL_RESTORE", 1),
    ] {
        assert_eq!(quantity(&c, &r, id), count);
    }
    let before = c.shell.shell.session().state().bag.clone();
    c = continue_save(&mut c, &root, &r);
    for item in items {
        assert!(event(
            &c,
            item["flag"].as_str().expect("persisted collection flag")
        ));
    }
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag, before);
    assert_eq!(protected(&c), original_story);
}

fn outdoor_kind<'a>(r: &'a CrystalRuntime, m: &MapModule, at: (u16, u16)) -> &'a str {
    let block =
        m.blocks[usize::from(at.1 / 2) * usize::from(m.attributes.width) + usize::from(at.0 / 2)];
    &r.data().tilesets[&m.attributes.tileset_name].collision[&format!("{block:02x}")]
        [usize::from(at.1 % 2 * 2 + at.0 % 2)]
}

fn outdoor_step(
    c: &mut VisibleShellController,
    button: GameButton,
    target: (u16, u16),
    context: &str,
) {
    let from = at(c);
    for _ in 0..8 {
        press(c, button);
        if at(c) != from {
            break;
        }
    }
    c.wait_frames(80)
        .expect("settle actual outdoor step or hop");
    let snapshot = c.snapshot().expect("actual outdoor movement");
    assert_eq!(snapshot.overworld.map_name, "MnDuluth");
    assert!(snapshot.battle.is_none());
    assert_eq!(
        at(c),
        target,
        "native outdoor movement differs: {context}; from={from:?} input={button:?}"
    );
}

fn outdoor_blocked(c: &mut VisibleShellController, button: GameButton, context: &str) {
    let stand = at(c);
    // The first input may only turn. The second must still hit the real wall.
    press(c, button);
    press(c, button);
    c.wait_frames(80).expect("settle actual blocked input");
    assert_eq!(
        at(c),
        stand,
        "outdoor wall must reject real movement: {context}; input={button:?}"
    );
    assert_eq!(
        c.snapshot()
            .expect("blocked outdoor input")
            .overworld
            .map_name,
        "MnDuluth"
    );
}

fn outdoor_walk(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    target: (u16, u16),
    context: &str,
) -> usize {
    // This is only a route planner. Every returned edge is executed by press.
    // Excluding all nonordinary collisions avoids unrelated doors/carpets and
    // makes the return proof independent of automatic hops or map transitions.
    let m = module(c, r);
    assert_eq!(m.id, "MnDuluth");
    let native = OverworldMapData::from_attributes(&m.id, &m.attributes, m.blocks.clone());
    let collision = r
        .data()
        .tileset_collision(&m.attributes.tileset_name)
        .expect("actual outdoor collision");
    let bodies = m
        .objects
        .iter()
        .map(|actor| OccupiedTile {
            tile: TilePosition::new(actor.x as i16, actor.y as i16),
            object_identifier: None,
        })
        .collect::<Vec<_>>();
    let start = at(c);
    let mut parents = BTreeMap::from([(start, None)]);
    let mut queue = VecDeque::from([start]);
    while let Some(p) = queue.pop_front() {
        if p == target {
            break;
        }
        for (dx, dy, direction, button) in [
            (0_i16, -1_i16, Direction::Up, GameButton::Up),
            (1, 0, Direction::Right, GameButton::Right),
            (0, 1, Direction::Down, GameButton::Down),
            (-1, 0, Direction::Left, GameButton::Left),
        ] {
            let x = p.0 as i16 + dx;
            let y = p.1 as i16 + dy;
            if x < 0
                || y < 0
                || x >= m.attributes.width as i16 * 2
                || y >= m.attributes.height as i16 * 2
            {
                continue;
            }
            let next = (x as u16, y as u16);
            if !matches!(outdoor_kind(r, &m, next), "FLOOR" | "UP_WALL")
                || parents.contains_key(&next)
            {
                continue;
            }
            let mut probe = PlayerMovementState::new(TilePosition::new(p.0 as i16, p.1 as i16));
            if !matches!(
                attempt_step_with_occupied_tiles(
                    &mut probe,
                    direction,
                    &native,
                    &collision,
                    StepOptions {
                        force_step_after_turn: true,
                        ..Default::default()
                    },
                    &bodies,
                ),
                StepOutcome::Moved { .. }
            ) {
                continue;
            }
            assert_eq!((probe.tile.x as u16, probe.tile.y as u16), next);
            parents.insert(next, Some((p, button)));
            queue.push_back(next);
        }
    }
    assert!(
        parents.contains_key(&target),
        "ordinary route absent: {start:?} -> {target:?}; {context}"
    );
    let mut route = Vec::new();
    let mut p = target;
    while let Some((previous, button)) = parents[&p] {
        route.push((button, p));
        p = previous;
    }
    let steps = route.len();
    for (button, next) in route.into_iter().rev() {
        outdoor_step(c, button, next, context);
    }
    assert_eq!(at(c), target);
    steps
}

#[test]
#[ignore = "actual five imported outdoor stairs and four hop lanes require tight Duluth"]
fn duluth_expansion_controller_outdoor_stairs_hops_and_ordinary_returns() {
    let (_, r, mut c, _) = fixture("MnDuluth", 0);
    let source = std::env::var_os("CRYSTAL_DULUTH_EXPANSION_DOSSIER")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("CRYSTAL_CITY_EXPANSION_CONTRACT").expect("contract"))
                .parent()
                .expect("frozen input folder")
                .join("MnDuluth.candidate.json")
        });
    let dossier: Value = serde_json::from_str(
        &std::fs::read_to_string(source).expect("actual frozen Duluth dossier"),
    )
    .expect("Duluth dossier");
    assert_eq!(dossier["dimensions_metatiles"], serde_json::json!([64, 48]));
    let probes = &dossier["expansion_contract"];
    let before = c.snapshot().expect("explicit initial outdoor fixture");
    let original_story = protected(&c);
    let m = module(&mut c, &r);
    let stairs = probes["stairs_metatiles"]
        .as_array()
        .expect("native stair probes");
    assert_eq!(stairs.len(), 5);
    for stair in stairs {
        let (x, y) = point(&stair["at"]);
        let block = stair["block"].as_str().expect("imported stair orientation");
        let context = format!("stair {} block={block} meta=({x},{y})", stair["id"]);
        assert!(matches!(block, "80" | "81"));
        assert_eq!(
            m.blocks[usize::from(y) * usize::from(m.attributes.width) + usize::from(x)],
            u16::from_str_radix(block, 16).expect("native block id")
        );
        let lane_x = x * 2 + u16::from(block == "80");
        let lower = (lane_x, y * 2 + 2);
        let lower_stair = (lane_x, y * 2 + 1);
        let upper_stair = (lane_x, y * 2);
        let upper = (lane_x, y * 2 - 1);
        let wall = (x * 2 + u16::from(block == "81"), y * 2 + 1);
        for tile in [lower, lower_stair, upper_stair, upper] {
            assert_eq!(outdoor_kind(&r, &m, tile), "FLOOR");
        }
        assert_eq!(outdoor_kind(&r, &m, wall), "WALL");
        outdoor_walk(&mut c, &r, lower, &context);
        outdoor_step(&mut c, GameButton::Up, lower_stair, &context);
        outdoor_blocked(
            &mut c,
            if block == "80" {
                GameButton::Left
            } else {
                GameButton::Right
            },
            &context,
        );
        outdoor_step(&mut c, GameButton::Up, upper_stair, &context);
        outdoor_step(&mut c, GameButton::Up, upper, &context);
        for target in [upper_stair, lower_stair, lower] {
            outdoor_step(&mut c, GameButton::Down, target, &context);
        }
    }
    let ledges = probes["new_ledges"]
        .as_array()
        .expect("four native hop lanes");
    assert_eq!(ledges.len(), 2);
    let mut returns = Vec::new();
    for ledge in ledges {
        for shoulder in ledge["course_shoulders"]
            .as_array()
            .expect("course shoulders")
        {
            assert_eq!(outdoor_kind(&r, &m, point(shoulder)), "WALL");
        }
        let lanes = ledge["lanes"].as_array().expect("native lip lanes");
        assert_eq!(lanes.len(), 2);
        for (index, lane) in lanes.iter().enumerate() {
            let upper = point(&lane["upper"]);
            let lip = point(&lane["hop"]);
            let landing = point(&lane["landing"]);
            let context = format!(
                "hop {} lane={index} upper={upper:?} lip={lip:?} landing={landing:?}",
                ledge["id"]
            );
            assert_eq!(outdoor_kind(&r, &m, upper), "FLOOR");
            assert_eq!(outdoor_kind(&r, &m, point(&lane["hop"])), "HOP_DOWN");
            assert_eq!(outdoor_kind(&r, &m, point(&lane["reverse_wall"])), "WALL");
            assert_eq!(outdoor_kind(&r, &m, landing), "FLOOR");
            outdoor_walk(&mut c, &r, upper, &context);
            // Native TryJump reads the current HOP_DOWN collision. First walk
            // onto the lip, then jump the wall course with a second input.
            outdoor_step(&mut c, GameButton::Down, lip, &context);
            outdoor_step(&mut c, GameButton::Down, landing, &context);
            outdoor_blocked(&mut c, GameButton::Up, &context);
            returns.push(outdoor_walk(&mut c, &r, upper, &context));
        }
    }
    assert_eq!(returns, [25, 27, 7, 9]);
    let after = c.snapshot().expect("actual outdoor course completed");
    assert_eq!(after.party, before.party);
    assert_eq!(after.bag, before.bag);
    assert_eq!(c.shell.shell.session().state().money, 999999);
    assert_eq!(protected(&c), original_story);
}
